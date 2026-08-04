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
//!                [--gate-builder GAMES_PER_POOL]
//!
//! The build net (Phase C) trains alongside the play net from the same
//! games — every decided game labels its two decklists — and checkpoints
//! as `deck-latest.safetensors`. `--gate-builder N` skips training and
//! races net-judged best-of-32 builds against static-judged best-of-32
//! of the *same* candidate sets on paired pools: the judges differ, the
//! candidates and pilots don't, so the result isolates the judge.
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
    best_build_by, heuristic_sealed_build, play_recorded_game, sealed_game_template, sealed_pool,
    static_deck_score,
};
use crabomination::server::bot::EvalWeights;
use crabomination::server::encode::{Vocab, encode_deck};
use crabomination_ml::{DeckNetConfig, DeckTrainer, NetConfig, SampleWindow, Trainer};
use crabomination_nn::{DeckNet, DeckRow, TrainRow};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

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
    /// Gate mode: skip training, load `<out>/deck-latest.safetensors`,
    /// and race net-judged builds against static-judged builds of the
    /// same candidate sets on paired pools, N games per pool.
    gate_builder: Option<usize>,
    /// Gate mode: race the repaired sealed builder (`builder_v2`)
    /// against the one it replaces, same pools, same pilots, N games
    /// per pool. No net involved — this measures the builder alone.
    gate_builder_v2: Option<usize>,
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
        gate_builder: None,
        gate_builder_v2: None,
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
            "--gate-builder" => a.gate_builder = Some(val().parse().expect("--gate-builder")),
            "--gate-builder-v2" => {
                a.gate_builder_v2 = Some(val().parse().expect("--gate-builder-v2"))
            }
            other => panic!("unknown flag {other} (see the module doc for usage)"),
        }
    }
    a
}

struct Shared {
    window: Mutex<SampleWindow>,
    /// Every game also labels its two decklists — the build net's stream.
    /// Small (2 rows/game), so a plain capped deque suffices.
    deck_window: Mutex<std::collections::VecDeque<DeckRow>>,
    /// Rows ever pushed (not evicted-adjusted) — the reuse cap's basis.
    rows_pushed: AtomicU64,
    /// Next self-play game index to claim — also the per-game seed salt.
    next_game: AtomicU64,
    games_done: AtomicU64,
    stalls: AtomicU64,
    live_actors: AtomicU64,
}

const DECK_WINDOW_CAP: usize = 200_000;

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
        {
            let mut w = shared.window.lock().unwrap();
            for row in rec.rows {
                w.push(row);
            }
        }
        if let Some(winner) = rec.winner {
            let mut dw = shared.deck_window.lock().unwrap();
            for (seat, deck) in [&deck_a, &deck_b].into_iter().enumerate() {
                let (cards, feats) = encode_deck(deck, vocab);
                if dw.len() == DECK_WINDOW_CAP {
                    dw.pop_front();
                }
                dw.push_back(DeckRow {
                    cards,
                    feats,
                    win: if seat == winner { 1.0 } else { 0.0 },
                });
            }
        }
    }
    shared.live_actors.fetch_sub(1, Ordering::Relaxed);
}

fn sample_owned(shared: &Shared, n: usize, rng: &mut StdRng) -> Vec<TrainRow> {
    let w = shared.window.lock().unwrap();
    w.sample(n, rng).into_iter().cloned().collect()
}

/// Wilson score interval (z = 1.96), same construction as bot_ladder's.
fn wilson(wins: u32, n: u32) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let z = 1.96f64;
    let (n, p) = (n as f64, wins as f64 / n as f64);
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    ((center - half).max(0.0), (center + half).min(1.0))
}

/// The builder gate: over paired pools, both judges rank the *same*
/// best-of-N candidate set — net win-prob vs the heuristic's static
/// score — and their picks race with identical pilots. Whatever
/// difference shows up is attributable to the judge alone.
fn gate_builder(args: &Args, vocab: &Vocab, games_per_pool: usize) {
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    let path = args.out.join("deck-latest.safetensors");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (train a deck net first)", path.display()));
    let net = DeckNet::load(&bytes).expect("deck net loads");
    assert_eq!(net.vocab_size(), vocab.size(), "deck net vocab != encoder vocab");
    const POOLS: u64 = 12;
    const CANDS: usize = 32;
    println!(
        "builder gate: net-judged vs static-judged best-of-{CANDS}, {games_per_pool} games x {POOLS} pools, seed {}",
        args.seed
    );
    let (mut wins_net, mut wins_static, mut undecided) = (0u32, 0u32, 0u32);
    for i in 0..POOLS {
        let salt = args.seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xB111D);
        let pool = sealed_pool(salt);
        let net_deck = best_build_by(&pool, CANDS, salt ^ 1, |d| {
            let (cards, feats) = encode_deck(d, vocab);
            net.forward(&cards, &feats) as f64
        });
        let static_deck =
            best_build_by(&pool, CANDS, salt ^ 1, |d| static_deck_score(d) as f64);
        let tally = simulate_match_games_piloted(
            &net_deck,
            &static_deck,
            games_per_pool,
            [Pilot::default(), Pilot::default()],
            4_000,
            Some(salt ^ 2),
        );
        println!(
            "pool #{i}: net {} - {} static ({} n/d)",
            tally.wins_a, tally.wins_b, tally.undecided
        );
        wins_net += tally.wins_a;
        wins_static += tally.wins_b;
        undecided += tally.undecided;
    }
    let decided = wins_net + wins_static;
    let pct = 100.0 * wins_net as f64 / decided.max(1) as f64;
    let (lo, hi) = wilson(wins_net, decided);
    println!(
        "TOTAL: net {wins_net} - {wins_static} static ({undecided} n/d) = {pct:.1}% [{:.1}%, {:.1}%]",
        lo * 100.0,
        hi * 100.0
    );
    println!(
        "verdict: {}",
        if lo > 0.5 {
            "net-judged builds are stronger — the interval clears 50%"
        } else if hi < 0.5 {
            "static-judged builds are stronger — the interval is below 50%"
        } else {
            "inconclusive — the interval straddles 50%"
        }
    );
}

/// Race the repaired builder against the one it replaces: same pool,
/// same pilots, same seeds — only the build differs.
fn gate_builder_v2(args: &Args, games_per_pool: usize) {
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    const POOLS: u64 = 12;
    println!(
        "builder gate: builder_v2 vs legacy builder, {games_per_pool} games x {POOLS} pools, seed {}",
        args.seed
    );
    let (mut wins_v2, mut wins_old, mut undecided) = (0u32, 0u32, 0u32);
    for i in 0..POOLS {
        let salt = args.seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xB0111D);
        let pool = sealed_pool(salt);
        let v2 = crabomination::selfplay::heuristic_sealed_build_with(&pool, salt ^ 1, true);
        let old = crabomination::selfplay::heuristic_sealed_build_with(&pool, salt ^ 1, false);
        let tally = simulate_match_games_piloted(
            &v2,
            &old,
            games_per_pool,
            [Pilot::default(), Pilot::default()],
            4_000,
            Some(salt ^ 2),
        );
        println!(
            "pool #{i}: v2 {} - {} legacy ({} n/d)",
            tally.wins_a, tally.wins_b, tally.undecided
        );
        wins_v2 += tally.wins_a;
        wins_old += tally.wins_b;
        undecided += tally.undecided;
    }
    let decided = wins_v2 + wins_old;
    let pct = 100.0 * wins_v2 as f64 / decided.max(1) as f64;
    let (lo, hi) = wilson(wins_v2, decided);
    println!(
        "TOTAL: v2 {wins_v2} - {wins_old} legacy ({undecided} n/d) = {pct:.1}% [{:.1}%, {:.1}%]",
        lo * 100.0,
        hi * 100.0
    );
    println!(
        "verdict: {}",
        if lo > 0.5 {
            "builder_v2 is stronger — the interval clears 50%"
        } else if hi < 0.5 {
            "the legacy builder is stronger — the interval is below 50%"
        } else {
            "inconclusive — the interval straddles 50%"
        }
    );
}

fn main() {
    let args = parse_args();
    let vocab = Vocab::sos_sealed();
    if let Some(games) = args.gate_builder_v2 {
        gate_builder_v2(&args, games);
        return;
    }
    if let Some(games) = args.gate_builder {
        gate_builder(&args, &vocab, games);
        return;
    }
    let cfg = NetConfig::standard(vocab.size());
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    let mut deck_trainer =
        DeckTrainer::new(&DeckNetConfig::standard(vocab.size()), args.lr).expect("deck trainer");
    std::fs::create_dir_all(&args.out).expect("create --out dir");
    let latest = args.out.join("latest.safetensors");
    if latest.exists() {
        trainer.load(&latest).expect("resume from latest.safetensors (delete it to start fresh)");
        eprintln!("resumed weights from {}", latest.display());
    }
    let deck_latest = args.out.join("deck-latest.safetensors");
    if deck_latest.exists() {
        deck_trainer.load(&deck_latest).expect("resume deck-latest.safetensors");
        eprintln!("resumed deck weights from {}", deck_latest.display());
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
        deck_window: Mutex::new(std::collections::VecDeque::new()),
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
        let mut deck_loss_ema = f32::NAN;
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

            // The deck net rides along at a quarter cadence — its stream
            // is 2 rows/game, so training it every step would just churn
            // the same rows.
            if step.is_multiple_of(4) {
                let rows: Vec<DeckRow> = {
                    let dw = shared.deck_window.lock().unwrap();
                    if dw.len() < 4_000 {
                        Vec::new()
                    } else {
                        (0..args.batch)
                            .map(|_| dw[rng.random_range(0..dw.len())].clone())
                            .collect()
                    }
                };
                if !rows.is_empty() {
                    let refs: Vec<&DeckRow> = rows.iter().collect();
                    let dl = deck_trainer.train_step(&refs).expect("deck step");
                    deck_loss_ema =
                        if deck_loss_ema.is_nan() { dl } else { 0.99 * deck_loss_ema + 0.01 * dl };
                }
            }

            if step.is_multiple_of(args.checkpoint_every) {
                checkpoint(
                    &trainer,
                    &deck_trainer,
                    deck_loss_ema,
                    &args,
                    &shared,
                    step,
                    consumed,
                    loss_ema,
                    start,
                    &stats_path,
                );
            }
        }
        if step > 0 && !step.is_multiple_of(args.checkpoint_every) {
            checkpoint(
                &trainer,
                &deck_trainer,
                deck_loss_ema,
                &args,
                &shared,
                step,
                consumed,
                loss_ema,
                start,
                &stats_path,
            );
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
    deck_trainer: &DeckTrainer,
    deck_loss: f32,
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
    let dtmp = args.out.join("deck-latest.safetensors.tmp");
    deck_trainer.save(&dtmp).expect("save deck checkpoint");
    std::fs::rename(&dtmp, args.out.join("deck-latest.safetensors")).expect("publish deck");
    let games = shared.games_done.load(Ordering::Relaxed);
    let rows = shared.rows_pushed.load(Ordering::Relaxed);
    let stalls = shared.stalls.load(Ordering::Relaxed);
    let secs = start.elapsed().as_secs_f64();
    let [total, win, life, len] = loss_ema;
    let line = format!(
        "{{\"step\":{step},\"loss_ema\":{total:.5},\"loss_win\":{win:.5},\"loss_life\":{life:.5},\"loss_len\":{len:.5},\"loss_deck\":{deck_loss:.5},\"rows_consumed\":{consumed},\"rows\":{rows},\"games\":{games},\"stalls\":{stalls},\"elapsed_s\":{secs:.0}}}\n"
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
