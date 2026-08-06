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
//!                [--lr F] [--reuse F] [--lambda F] [--relabel-every N]
//!                [--window N] [--min-window N]
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
    /// TD(lambda) mixing for the win target. 1.0 (default) is the
    /// historical pure Monte Carlo label — the row's own game result —
    /// so every prior gate round is reproducible. Below 1.0 the target
    /// bootstraps through the net's estimate of the next snapshot, which
    /// trades the variance of a twenty-turn outcome for the bias of the
    /// current net (see `SampleWindow::relabel_lambda`).
    lambda: f32,
    /// Learner steps between recomputing the lambda-returns. They are
    /// functions of the net, so they drift as it trains; recomputing
    /// costs one forward pass over the window. Ignored at lambda = 1.
    relabel_every: u64,
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
    /// Fraction of trajectories held out of training and scored at every
    /// checkpoint. 0 disables.
    ///
    /// This exists because its absence cost six gate rounds. `stats.jsonl`
    /// reported training loss only, and training loss here is a
    /// memorisation reading: at lambda=1 the net hit 0.017 MSE on the
    /// window while its out-of-sample log-loss was 1.12 — *worse than
    /// predicting 0.5 every time*. Nothing in the loop could see that, so
    /// the overfit was diagnosed only after thousands of gate games.
    ///
    /// Split by trajectory, not by row: consecutive snapshots of one game
    /// are near-duplicates, so a row-level split leaks the answer across
    /// it and the validation number comes back reassuringly good.
    holdout: f64,
    /// Train the play net with the pre-pool attention layer — the
    /// interaction model. Off is the pooled control.
    attn: bool,
    /// Build training decks with the gate-passed deck net as the judge
    /// (best-of-32 over the same noisy-greedy candidates the heuristic
    /// picks from) instead of taking the heuristic builder's own pick.
    ///
    /// The deck net is the one learned component that has cleared the
    /// house bar — twice, at 61.7 % and 60.7 % against the static judge —
    /// so the decks the actors play should be its picks. Until this flag
    /// the gate result had no consumer: every training game was still
    /// played with heuristic builds.
    use_deck_best: Option<PathBuf>,
    /// Diagnostic mode: local-discrimination test over N games. See
    /// `pairwise`.
    pairwise: Option<usize>,
    /// Diagnostic mode: play N self-play games and score the value net
    /// and the heuristic evaluation as *predictors of the winner* on the
    /// same positions. Requires `--use-best`.
    calibrate: Option<usize>,
}

fn parse_args() -> Args {
    let mut a = Args {
        actors: std::thread::available_parallelism().map(|n| n.get().saturating_sub(2)).unwrap_or(4).max(1),
        games: None,
        steps: None,
        batch: 256,
        lr: 1e-3,
        reuse: 6.0,
        lambda: 1.0,
        relabel_every: 200,
        window: 250_000,
        min_window: 20_000,
        checkpoint_every: 2_000,
        out: PathBuf::from("nets"),
        seed: 0x0505_ACAD,
        use_best: None,
        gate_builder: None,
        gate_builder_v2: None,
        calibrate: None,
        pairwise: None,
        use_deck_best: None,
        attn: false,
        holdout: 0.05,
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
            "--lambda" => a.lambda = val().parse().expect("--lambda"),
            "--relabel-every" => {
                a.relabel_every = val().parse::<u64>().expect("--relabel-every").max(1)
            }
            "--window" => a.window = val().parse().expect("--window"),
            "--min-window" => a.min_window = val().parse().expect("--min-window"),
            "--checkpoint-every" => a.checkpoint_every = val().parse().expect("--checkpoint-every"),
            "--out" => a.out = PathBuf::from(val()),
            "--seed" => a.seed = val().parse().expect("--seed"),
            "--use-best" => a.use_best = Some(PathBuf::from(val())),
            "--calibrate" => a.calibrate = Some(val().parse().expect("--calibrate")),
            "--pairwise" => a.pairwise = Some(val().parse().expect("--pairwise")),
            "--use-deck-best" => a.use_deck_best = Some(PathBuf::from(val())),
            "--holdout" => a.holdout = val().parse().expect("--holdout"),
            "--attn" => {
                a.attn = true;
                continue; // bare flag, consumes no value
            }
            "--gate-builder" => a.gate_builder = Some(val().parse().expect("--gate-builder")),
            "--gate-builder-v2" => {
                a.gate_builder_v2 = Some(val().parse().expect("--gate-builder-v2"))
            }
            other => panic!("unknown flag {other} (see the module doc for usage)"),
        }
    }
    a
}

/// True when this trajectory belongs to the held-out set. A hash of the
/// id rather than a counter, so actors decide independently and the same
/// game always lands on the same side of the split.
fn is_holdout(traj: u32, frac: f64) -> bool {
    if frac <= 0.0 {
        return false;
    }
    let mut h = (traj as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    ((h >> 11) as f64 / (1u64 << 53) as f64) < frac
}

struct Shared {
    window: Mutex<SampleWindow>,
    /// Held-out rows, never trained on. Capped so a long run can't grow
    /// it without bound.
    val: Mutex<Vec<TrainRow>>,
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

fn actor_loop(shared: &Shared, args: &Args, vocab: &Vocab, deck_judge: Option<&DeckNet>) {
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
        // Best-of-N under the gate-passed deck net when one is loaded,
        // otherwise the heuristic builder's own pick. Same candidate
        // generator either way, so only the judge differs.
        const DECK_CANDS: usize = 32;
        let build = |pool: &[crabomination::cube::CardFactory], seed: u64| match deck_judge {
            Some(net) => best_build_by(pool, DECK_CANDS, seed, |d| {
                let (cards, feats) = encode_deck(d, vocab);
                net.forward(&cards, &feats) as f64
            }),
            None => heuristic_sealed_build(pool, seed),
        };
        let deck_a = build(&pool_a, salt(3));
        let deck_b = build(&pool_b, salt(4));
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
        // A whole game's rows share a trajectory pair, so the split is
        // decided per row by trajectory id and stays consistent.
        let (val_rows, train_rows): (Vec<TrainRow>, Vec<TrainRow>) =
            rec.rows.into_iter().partition(|r| is_holdout(r.traj, args.holdout));
        if !val_rows.is_empty() {
            const VAL_CAP: usize = 20_000;
            let mut v = shared.val.lock().unwrap();
            for row in val_rows {
                if v.len() < VAL_CAP {
                    v.push(row);
                }
            }
        }
        shared.rows_pushed.fetch_add(train_rows.len() as u64, Ordering::Relaxed);
        {
            let mut w = shared.window.lock().unwrap();
            for row in train_rows {
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

fn sample_owned(shared: &Shared, n: usize, rng: &mut StdRng) -> Vec<(TrainRow, f32)> {
    let w = shared.window.lock().unwrap();
    w.sample_with_targets(n, rng).into_iter().map(|(r, t)| (r.clone(), t)).collect()
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
    if let Some(games) = args.calibrate {
        calibrate(&args, &vocab, games);
        return;
    }
    if let Some(games) = args.pairwise {
        pairwise(&args, &vocab, games);
        return;
    }
    let cfg = if args.attn {
        NetConfig::with_attention(vocab.size())
    } else {
        NetConfig::standard(vocab.size())
    };
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    let mut deck_trainer =
        DeckTrainer::new(&DeckNetConfig::standard(vocab.size()), args.lr).expect("deck trainer");
    eprintln!(
        "learner device: {} (lambda {}, batch {}, lr {}, {})",
        trainer.device_label(),
        args.lambda,
        args.batch,
        args.lr,
        if args.attn { "attention" } else { "pooled" }
    );
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
    // The gate-passed build net, if the run wants its decks judged.
    let deck_judge: Option<DeckNet> = args.use_deck_best.as_ref().map(|p| {
        let bytes = std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let net = DeckNet::load(&bytes).expect("deck net loads");
        assert_eq!(net.vocab_size(), vocab.size(), "deck net vocab != encoder vocab");
        eprintln!("actors build decks with the net from {}", p.display());
        net
    });
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
        val: Mutex::new(Vec::new()),
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
                .spawn_scoped(scope, || actor_loop(&shared, &args, &vocab, deck_judge.as_ref()))
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
            // Refresh the λ-returns periodically: they are computed
            // through the net, so they go stale as it learns. Every
            // `relabel_every` steps is a compromise between staleness and
            // the cost of a forward pass over the whole window — with
            // λ = 1 the targets never change, so this is skipped entirely.
            if args.lambda < 1.0
                && step % args.relabel_every == 0
                && shared.window.lock().unwrap().len() as u64 >= args.min_window
            {
                let mut w = shared.window.lock().unwrap();
                w.relabel_lambda(args.lambda, |rows| {
                    trainer.predict_win_batch(rows, 512).unwrap_or_default()
                });
            }
            let rows = sample_owned(&shared, args.batch, &mut rng);
            let refs: Vec<(&TrainRow, f32)> = rows.iter().map(|(r, t)| (r, *t)).collect();
            let loss = trainer.train_step_with_targets(&refs).expect("train step");
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
    // Held-out scoring. `val_win` is directly comparable to `loss_win`
    // (same MSE, different rows), so the gap between them *is* the
    // overfit, visible while the run is happening instead of afterwards.
    // `val_auc` is the one that says whether the net knows anything —
    // MSE can improve while ranking does not.
    let (val_n, val_win, val_ll, val_auc) = {
        let v = shared.val.lock().unwrap();
        if v.len() < 200 {
            (0usize, f32::NAN, f32::NAN, f32::NAN)
        } else {
            let refs: Vec<&TrainRow> = v.iter().collect();
            match trainer.predict_win_batch(&refs, 512) {
                Ok(p) => {
                    let mse = p
                        .iter()
                        .zip(&refs)
                        .map(|(q, r)| (q - r.win) * (q - r.win))
                        .sum::<f32>()
                        / p.len() as f32;
                    let pairs: Vec<(f32, f32)> =
                        p.iter().zip(&refs).map(|(q, r)| (*q, r.win)).collect();
                    (
                        v.len(),
                        mse,
                        log_loss(pairs.iter().copied()),
                        auc(pairs.iter().copied()),
                    )
                }
                Err(_) => (0, f32::NAN, f32::NAN, f32::NAN),
            }
        }
    };
    let line = format!(
        "{{\"step\":{step},\"loss_ema\":{total:.5},\"loss_win\":{win:.5},\"loss_life\":{life:.5},\"loss_len\":{len:.5},\"loss_deck\":{deck_loss:.5},\"val_n\":{val_n},\"val_win\":{val_win:.5},\"val_logloss\":{val_ll:.5},\"val_auc\":{val_auc:.5},\"rows_consumed\":{consumed},\"rows\":{rows},\"games\":{games},\"stalls\":{stalls},\"elapsed_s\":{secs:.0}}}\n"
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stats_path)
        .expect("stats.jsonl");
    f.write_all(line.as_bytes()).expect("stats write");
    let val_note = if val_n > 0 {
        format!(" | val win {val_win:.4} auc {val_auc:.4} (n={val_n})")
    } else {
        String::new()
    };
    eprintln!(
        "step {step}: loss {total:.4} (win {win:.4}){val_note}, {games} games, {rows} rows ({:.1} games/s)",
        games as f64 / secs.max(0.001)
    );
}

// ────────────────────────────── calibration ──────────────────────────────

/// Score the value net and the heuristic evaluation as *predictors of the
/// winner* on identical self-play positions.
///
/// This exists because four gate rounds answered "is the net-piloted bot
/// stronger" (42–45 % as a replacement, ~49 % blended) without ever
/// answering "does the net know more than `eval_material` does". Those are
/// different questions with different fixes:
///
/// * If the net's log-loss is no better than the heuristic's, the net has
///   not learned and no amount of integration work will help.
/// * If it *is* better and the bot still loses, the loss is in the
///   integration — and the output histogram is the first place to look. A
///   sigmoid that saturates near 0 and 1 hands the search a flat
///   landscape in which every candidate line scores the same, which would
///   make a better predictor into a worse player.
///
/// The heuristic is put on the same footing by fitting a one-parameter
/// logistic to its score (`p = sigmoid(score / t)`, `t` chosen by scan),
/// so it is scored as the best probability forecast that evaluation can
/// support rather than penalised for not being calibrated.
fn calibrate(args: &Args, vocab: &Vocab, games: usize) {
    let best = args.use_best.as_ref().expect("--calibrate needs --use-best WEIGHTS");
    let cfg = if args.attn {
        NetConfig::with_attention(vocab.size())
    } else {
        NetConfig::standard(vocab.size())
    };
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    trainer
        .load(best)
        .expect("load weights for calibration (add --attn if these are attention weights)");

    // (net p, heuristic score, actual result)
    let mut obs: Vec<(f32, f32, f32)> = Vec::new();
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0xCA11B);
    for n in 0..games as u64 {
        let salt =
            |k: u64| args.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0xCA1);
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let deck_a = heuristic_sealed_build(&pool_a, salt(3));
        let deck_b = heuristic_sealed_build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            50_000,
            vocab,
        );
        if rec.rows.is_empty() {
            continue;
        }
        let refs: Vec<&TrainRow> = rec.rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("forward");
        for ((row, h), p) in rec.rows.iter().zip(&rec.heur).zip(preds) {
            obs.push((p, *h as f32, row.win));
        }
    }
    if obs.len() < 100 {
        eprintln!("calibrate: only {} positions — not enough to say anything", obs.len());
        return;
    }

    // Fit the heuristic's temperature by scanning: one parameter, and a
    // scan cannot land in a bad local optimum the way a gradient step can.
    let mut best_t = 1.0f32;
    let mut best_ll = f32::INFINITY;
    for k in 0..60 {
        let t = 2.0f32.powf(k as f32 / 4.0);
        let ll = log_loss(obs.iter().map(|&(_, h, y)| (sigmoid(h / t), y)));
        if ll < best_ll {
            best_ll = ll;
            best_t = t;
        }
    }

    let net_ll = log_loss(obs.iter().map(|&(p, _, y)| (p, y)));
    let net_brier = brier(obs.iter().map(|&(p, _, y)| (p, y)));
    let heur_brier = brier(obs.iter().map(|&(_, h, y)| (sigmoid(h / best_t), y)));
    let net_auc = auc(obs.iter().map(|&(p, _, y)| (p, y)));
    let heur_auc = auc(obs.iter().map(|&(_, h, y)| (h, y)));
    // The constant predictor: what "knows nothing" scores, so the two
    // numbers above have a floor to be read against.
    let base = obs.iter().map(|&(_, _, y)| y).sum::<f32>() / obs.len() as f32;
    let base_ll = log_loss(obs.iter().map(|&(_, _, y)| (base, y)));

    println!("calibration on {} positions from {games} games", obs.len());
    println!("  base rate {base:.3}  (log-loss {base_ll:.4} — the score of knowing nothing)");
    println!("  net        log-loss {net_ll:.4}  Brier {net_brier:.4}  AUC {net_auc:.4}");
    println!(
        "  heuristic  log-loss {best_ll:.4}  Brier {heur_brier:.4}  AUC {heur_auc:.4}  (t={best_t:.1})"
    );

    // Output histogram: the saturation check.
    let mut bins = [0usize; 10];
    for &(p, _, _) in &obs {
        bins[((p * 10.0) as usize).min(9)] += 1;
    }
    println!("  net output histogram (0.0..1.0 in tenths):");
    for (i, c) in bins.iter().enumerate() {
        let pct = 100.0 * *c as f64 / obs.len() as f64;
        println!("    {:.1}-{:.1}  {:>6}  {:5.1}%  {}", i as f32 / 10.0, (i + 1) as f32 / 10.0, c, pct, "#".repeat((pct / 2.0) as usize));
    }
    let extreme = obs.iter().filter(|&&(p, _, _)| !(0.05..=0.95).contains(&p)).count();
    println!(
        "  {:.1}% of positions score outside [0.05, 0.95] — the search cannot rank lines \
         inside a saturated band",
        100.0 * extreme as f64 / obs.len() as f64
    );
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn log_loss(it: impl Iterator<Item = (f32, f32)>) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0usize);
    for (p, y) in it {
        let p = p.clamp(1e-6, 1.0 - 1e-6);
        sum += -(y * p.ln() + (1.0 - y) * (1.0 - p).ln());
        n += 1;
    }
    sum / n.max(1) as f32
}

fn brier(it: impl Iterator<Item = (f32, f32)>) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0usize);
    for (p, y) in it {
        sum += (p - y) * (p - y);
        n += 1;
    }
    sum / n.max(1) as f32
}

/// Rank-based AUC. Scale-free, so the heuristic's raw score can be scored
/// against the net's probability without fitting anything.
fn auc(it: impl Iterator<Item = (f32, f32)>) -> f32 {
    let mut v: Vec<(f32, f32)> = it.collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (mut pos, mut neg, mut rank_sum) = (0.0f64, 0.0f64, 0.0f64);
    // Ties share the mean rank, which is what keeps a constant predictor
    // at exactly 0.5 instead of whatever the sort order happened to be.
    let mut i = 0;
    while i < v.len() {
        let mut j = i;
        while j < v.len() && v[j].0 == v[i].0 {
            j += 1;
        }
        let mean_rank = (i + j + 1) as f64 / 2.0;
        for item in &v[i..j] {
            if item.1 > 0.5 {
                pos += 1.0;
                rank_sum += mean_rank;
            } else {
                neg += 1.0;
            }
        }
        i = j;
    }
    if pos == 0.0 || neg == 0.0 {
        return 0.5;
    }
    ((rank_sum - pos * (pos + 1.0) / 2.0) / (pos * neg)) as f32
}

// ───────────────────────── local discrimination ──────────────────────────

/// Can the evaluator order two *adjacent* positions from the same game?
///
/// This exists because `--calibrate` measured the wrong thing for the
/// question that matters. AUC is a *global* ranking statistic: it asks
/// whether a winning board outscores a losing one across a diverse pool of
/// positions. The attention net wins that comparison against
/// `eval_material` (0.798 vs 0.760 on seed 43, replicated 0.761 vs 0.747
/// on seed 97) — and then loses the gate outright, 44.8 % as a
/// replacement and 48.8 % blended.
///
/// The reconciliation is that the search never asks the global question.
/// It compares *near-identical* boards — the same position differing by
/// one attack or one block — dozens of times per decision, and picks the
/// argmax. An evaluator can be excellent at "who is winning" and useless
/// at "which of these two almost-identical lines is better", and only the
/// second is consumed inside a resolved simulation.
///
/// Adjacent snapshots of one trajectory are the cheapest available proxy
/// for that: one turn apart, same deck, same seat, overwhelmingly similar
/// boards. Ground truth is the trajectory's direction — a winner's
/// position tends toward 1 and a loser's toward 0. That is imperfect per
/// pair (a winner's board does not improve monotonically) but it is
/// exactly the assumption the value target already makes, and it is
/// applied identically to both evaluators, so the *comparison* is fair
/// even where the label is noisy.
///
/// Also reported: mean separation. An evaluator that orders pairs
/// correctly but by a hair is still useless to a search whose candidates
/// differ by less than its own noise.
fn pairwise(args: &Args, vocab: &Vocab, games: usize) {
    let best = args.use_best.as_ref().expect("--pairwise needs --use-best WEIGHTS");
    let cfg = if args.attn {
        NetConfig::with_attention(vocab.size())
    } else {
        NetConfig::standard(vocab.size())
    };
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    trainer.load(best).expect("load weights (add --attn for attention weights)");

    let (mut net_ok, mut heur_ok, mut net_tie, mut heur_tie, mut n) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut net_sep, mut heur_sep) = (0.0f64, 0.0f64);
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0x9A1D);
    for g in 0..games as u64 {
        let salt = |k: u64| args.seed ^ g.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x9A1);
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let deck_a = heuristic_sealed_build(&pool_a, salt(3));
        let deck_b = heuristic_sealed_build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            50_000,
            vocab,
        );
        if rec.rows.len() < 4 {
            continue;
        }
        let refs: Vec<&TrainRow> = rec.rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("forward");

        // Group row indices by trajectory, ordered by ply.
        let mut by_traj: std::collections::HashMap<u32, Vec<usize>> = Default::default();
        for (i, r) in rec.rows.iter().enumerate() {
            by_traj.entry(r.traj).or_default().push(i);
        }
        for idxs in by_traj.values_mut() {
            idxs.sort_by_key(|&i| rec.rows[i].ply);
            for w in idxs.windows(2) {
                let (a, b) = (w[0], w[1]);
                // +1 when this seat won: later snapshots should score
                // higher. -1 when it lost.
                let dir = if rec.rows[a].win > 0.5 { 1.0f64 } else { -1.0 };
                let dn = (preds[b] - preds[a]) as f64;
                // Heuristic scores are unbounded ints; normalise by the
                // pair's own scale so "separation" is comparable to the
                // net's [0,1] output rather than a raw material delta.
                let dh = (rec.heur[b] - rec.heur[a]) as f64 / 1000.0;
                n += 1;
                if dn == 0.0 { net_tie += 1 } else if dn * dir > 0.0 { net_ok += 1 }
                if dh == 0.0 { heur_tie += 1 } else if dh * dir > 0.0 { heur_ok += 1 }
                net_sep += dn.abs();
                heur_sep += dh.abs();
            }
        }
    }
    if n == 0 {
        eprintln!("pairwise: no usable adjacent pairs");
        return;
    }
    // Report the correct-rate among pairs the evaluator actually
    // *separates*, alongside the tie rate. Scoring ties as wrong would
    // punish an evaluator for honestly declining to distinguish two
    // equal positions, which is the opposite of what we want to measure —
    // and mixing the two into one percentage hides the tie rate, which is
    // the interesting number here.
    //
    // Separations are NOT compared across evaluators: the heuristic is an
    // unbounded integer score and the net is a probability, so any shared
    // normalisation would be arbitrary. Each is reported against its own
    // scale only.
    let rate = |ok: usize, ties: usize| {
        let decided = n - ties;
        if decided == 0 { f64::NAN } else { 100.0 * ok as f64 / decided as f64 }
    };
    println!("local discrimination on {n} adjacent same-game pairs from {games} games");
    println!("  (chance is 50 %; this is the question the SEARCH asks, unlike AUC)");
    println!(
        "  net        {:.1}% of separated pairs ordered right   ties {:.1}%   mean |delta| {:.4} (prob scale)",
        rate(net_ok, net_tie),
        100.0 * net_tie as f64 / n as f64,
        net_sep / (n - net_tie).max(1) as f64
    );
    println!(
        "  heuristic  {:.1}% of separated pairs ordered right   ties {:.1}%   mean |delta| {:.1} (eval units)",
        rate(heur_ok, heur_tie),
        100.0 * heur_tie as f64 / n as f64,
        heur_sep * 1000.0 / (n - heur_tie).max(1) as f64
    );
}
