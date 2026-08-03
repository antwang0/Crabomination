//! selfplay_train — the concurrent train-and-play loop for the SOS sealed
//! value net.
//!
//! KataGo-shaped, single process (see FEATURE_ROADMAP Tier 13): actor
//! threads play sealed self-play games and push encoded rows into a shared
//! sliding window while the learner thread trains from it, throttled by a
//! sample-reuse cap so it can never grind the recent window into overfit.
//! Checkpoints land in `--out` as `latest.safetensors` (atomic rename), and
//! a `stats.jsonl` line per checkpoint records the loss curve.
//!
//! Bootstrap phase: actors play with the heuristic default bot and the
//! heuristic sealed builder — the net is a passenger until it can pass a
//! gate (the gatekeeper and the net-eval bot profile are the next step;
//! nothing here reads the net back yet).
//!
//! ```text
//! selfplay_train [--actors N] [--games N] [--steps N] [--batch N]
//!                [--lr F] [--reuse F] [--window N] [--min-window N]
//!                [--checkpoint-every N] [--out DIR] [--seed N]
//!                [--use-best WEIGHTS.safetensors]
//! ```
//!
//! With `--games`/`--steps` unset it runs until killed; the latest
//! checkpoint is at most `--checkpoint-every` steps stale, and a fresh run
//! with the same `--out` resumes from `latest.safetensors` if the shapes
//! still match.
//!
//! Measured on the first smoke run (22 actors, debug build, defaults):
//! ~10 SOS sealed games/s (~36 rows each) with the learner sharing the
//! box, loss EMA 0.371 → 0.216 over 600 steps of batch 256, and the
//! reuse throttle engaging within 3 % of the 6× cap. Generation is the
//! bottleneck by design — the learner spends most wall clock waiting,
//! which is the sample-reuse cap doing its job.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crabomination::selfplay::{
    heuristic_sealed_build, play_recorded_game, sealed_game_template, sealed_pool,
};
use crabomination::server::bot::EvalWeights;
use crabomination::server::encode::Vocab;
use crabomination_ml::{NetConfig, SampleWindow, Trainer};
use crabomination_nn::TrainRow;
use rand::SeedableRng;
use rand::rngs::StdRng;

struct Args {
    actors: usize,
    games: Option<u64>,
    steps: Option<u64>,
    batch: usize,
    lr: f64,
    reuse: f64,
    window: usize,
    min_window: u64,
    checkpoint_every: u64,
    out: PathBuf,
    seed: u64,
    /// Weights for the actors to *play* with (a gate-passed promotion) —
    /// this is what closes the self-improvement loop. Unset, actors play
    /// the heuristic default (the bootstrap phase).
    use_best: Option<PathBuf>,
}

fn parse_args() -> Args {
    let mut a = Args {
        actors: std::thread::available_parallelism().map(|n| n.get().saturating_sub(2)).unwrap_or(4).max(1),
        games: None,
        steps: None,
        batch: 256,
        lr: 1e-3,
        reuse: 6.0,
        window: 250_000,
        min_window: 20_000,
        checkpoint_every: 2_000,
        out: PathBuf::from("nets"),
        seed: 0x0505_ACAD,
        use_best: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--actors" => a.actors = val().parse().expect("--actors"),
            "--games" => a.games = Some(val().parse().expect("--games")),
            "--steps" => a.steps = Some(val().parse().expect("--steps")),
            "--batch" => a.batch = val().parse().expect("--batch"),
            "--lr" => a.lr = val().parse().expect("--lr"),
            "--reuse" => a.reuse = val().parse().expect("--reuse"),
            "--window" => a.window = val().parse().expect("--window"),
            "--min-window" => a.min_window = val().parse().expect("--min-window"),
            "--checkpoint-every" => a.checkpoint_every = val().parse().expect("--checkpoint-every"),
            "--out" => a.out = PathBuf::from(val()),
            "--seed" => a.seed = val().parse().expect("--seed"),
            "--use-best" => a.use_best = Some(PathBuf::from(val())),
            other => panic!("unknown flag {other} (see the module doc for usage)"),
        }
    }
    a
}

struct Shared {
    window: Mutex<SampleWindow>,
    /// Rows ever pushed (not evicted-adjusted) — the reuse cap's basis.
    rows_pushed: AtomicU64,
    /// Next self-play game index to claim — also the per-game seed salt.
    next_game: AtomicU64,
    games_done: AtomicU64,
    stalls: AtomicU64,
    live_actors: AtomicU64,
}

fn actor_loop(shared: &Shared, args: &Args, vocab: &Vocab) {
    loop {
        let n = shared.next_game.fetch_add(1, Ordering::Relaxed);
        if let Some(max) = args.games
            && n >= max
        {
            break;
        }
        let salt = |k: u64| {
            args.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x9E37_79B9)
        };
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let deck_a = heuristic_sealed_build(&pool_a, salt(3));
        let deck_b = heuristic_sealed_build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let pilot = if args.use_best.is_some() {
            EvalWeights::net_eval()
        } else {
            EvalWeights::default()
        };
        let rec = play_recorded_game(&template, [pilot, pilot], salt(5), 4_000, vocab);
        shared.games_done.fetch_add(1, Ordering::Relaxed);
        if rec.rows.is_empty() {
            shared.stalls.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        shared.rows_pushed.fetch_add(rec.rows.len() as u64, Ordering::Relaxed);
        let mut w = shared.window.lock().unwrap();
        for row in rec.rows {
            w.push(row);
        }
    }
    shared.live_actors.fetch_sub(1, Ordering::Relaxed);
}

fn sample_owned(shared: &Shared, n: usize, rng: &mut StdRng) -> Vec<TrainRow> {
    let w = shared.window.lock().unwrap();
    w.sample(n, rng).into_iter().cloned().collect()
}

fn main() {
    let args = parse_args();
    let vocab = Vocab::sos_sealed();
    let cfg = NetConfig::standard(vocab.size());
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    std::fs::create_dir_all(&args.out).expect("create --out dir");
    let latest = args.out.join("latest.safetensors");
    if latest.exists() {
        trainer.load(&latest).expect("resume from latest.safetensors (delete it to start fresh)");
        eprintln!("resumed weights from {}", latest.display());
    }
    if let Some(best) = &args.use_best {
        crabomination::server::net_eval::load_slot(
            crabomination::server::net_eval::SLOT_BEST,
            best,
        )
        .expect("--use-best weights load");
        eprintln!("actors play with the net from {}", best.display());
    }

    let shared = Shared {
        window: Mutex::new(SampleWindow::new(args.window)),
        rows_pushed: AtomicU64::new(0),
        next_game: AtomicU64::new(0),
        games_done: AtomicU64::new(0),
        stalls: AtomicU64::new(0),
        live_actors: AtomicU64::new(args.actors as u64),
    };
    eprintln!(
        "selfplay_train: {} actors, vocab {}, window {}, batch {}, reuse cap {}x",
        args.actors,
        vocab.size(),
        args.window,
        args.batch,
        args.reuse
    );

    let start = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..args.actors {
            // SOS games overflow the default stack (see bot_probe) — match
            // the ladder's 32 MB workers.
            std::thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, || actor_loop(&shared, &args, &vocab))
                .expect("spawn actor");
        }

        // Learner (this thread).
        let mut rng = StdRng::seed_from_u64(args.seed ^ 0x1EA4);
        let mut consumed = 0u64;
        let mut step = 0u64;
        // EMAs of [total, win, life, len] — decomposed so a regime change
        // in one head (or in effective sample reuse) is visible.
        let mut loss_ema = [f32::NAN; 4];
        // (samples consumed when the actors finished, tail allowance).
        let mut tail_budget = None::<(u64, u64)>;
        let stats_path = args.out.join("stats.jsonl");
        loop {
            if let Some(max) = args.steps
                && step >= max
            {
                break;
            }
            let pushed = shared.rows_pushed.load(Ordering::Relaxed);
            let actors_live = shared.live_actors.load(Ordering::Relaxed) > 0;
            // Once generation ends, the global reuse budget stops meaning
            // "≤ reuse visits per row": every further sample lands on the
            // final window, concentrating budget/window_len visits there
            // (measured ~14× on the first release run, where generation
            // outpaced the single-threaded learner 5:1). Grant the tail
            // half the nominal reuse on the window it has — the rows
            // already absorbed roughly the other half while streaming —
            // and stop.
            if !actors_live && tail_budget.is_none() {
                let wlen = shared.window.lock().unwrap().len() as u64;
                tail_budget = Some((consumed, (args.reuse * wlen as f64 / 2.0) as u64));
            }
            if let Some((at, budget)) = tail_budget
                && consumed - at + args.batch as u64 > budget
            {
                break;
            }
            let budget = (args.reuse * pushed as f64) as u64;
            if pushed < args.min_window || consumed + args.batch as u64 > budget {
                if !actors_live {
                    break; // no more data coming and the reuse budget is spent
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
                continue;
            }
            let rows = sample_owned(&shared, args.batch, &mut rng);
            let refs: Vec<&TrainRow> = rows.iter().collect();
            let loss = trainer.train_step(&refs).expect("train step");
            consumed += args.batch as u64;
            step += 1;
            for (ema, part) in loss_ema
                .iter_mut()
                .zip([loss.total, loss.win, loss.life, loss.len])
            {
                *ema = if ema.is_nan() { part } else { 0.99 * *ema + 0.01 * part };
            }

            if step.is_multiple_of(args.checkpoint_every) {
                checkpoint(&trainer, &args, &shared, step, consumed, loss_ema, start, &stats_path);
            }
        }
        if step > 0 && !step.is_multiple_of(args.checkpoint_every) {
            checkpoint(&trainer, &args, &shared, step, consumed, loss_ema, start, &stats_path);
        }
        eprintln!(
            "learner done: {} steps, {} rows consumed; waiting for actors...",
            step, consumed
        );
        // Scope exit joins the actors; with --games unset they run forever
        // and this process is ended by the operator (checkpoints are
        // already on disk).
    });

    let secs = start.elapsed().as_secs_f64();
    let games = shared.games_done.load(Ordering::Relaxed);
    eprintln!(
        "done: {games} games ({:.1}/s), {} rows, {} stalls, {:.0}s",
        games as f64 / secs.max(0.001),
        shared.rows_pushed.load(Ordering::Relaxed),
        shared.stalls.load(Ordering::Relaxed),
        secs
    );
}

#[allow(clippy::too_many_arguments)]
fn checkpoint(
    trainer: &Trainer,
    args: &Args,
    shared: &Shared,
    step: u64,
    consumed: u64,
    loss_ema: [f32; 4],
    start: Instant,
    stats_path: &std::path::Path,
) {
    let tmp = args.out.join("latest.safetensors.tmp");
    trainer.save(&tmp).expect("save checkpoint");
    std::fs::rename(&tmp, args.out.join("latest.safetensors")).expect("publish checkpoint");
    let games = shared.games_done.load(Ordering::Relaxed);
    let rows = shared.rows_pushed.load(Ordering::Relaxed);
    let stalls = shared.stalls.load(Ordering::Relaxed);
    let secs = start.elapsed().as_secs_f64();
    let [total, win, life, len] = loss_ema;
    let line = format!(
        "{{\"step\":{step},\"loss_ema\":{total:.5},\"loss_win\":{win:.5},\"loss_life\":{life:.5},\"loss_len\":{len:.5},\"rows_consumed\":{consumed},\"rows\":{rows},\"games\":{games},\"stalls\":{stalls},\"elapsed_s\":{secs:.0}}}\n"
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stats_path)
        .expect("stats.jsonl");
    f.write_all(line.as_bytes()).expect("stats write");
    eprintln!(
        "step {step}: loss {total:.4} (win {win:.4}), {games} games, {rows} rows ({:.1} games/s)",
        games as f64 / secs.max(0.001)
    );
}
