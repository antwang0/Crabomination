//! `table_train` — the one-model-for-every-mode trainer (ML_NOTES "One model
//! for every mode", step 2 (c)). Actors play games drawn from the mode mixer
//! (`table_selfplay::draw_mode`: SOS / FRA sealed, cube, Commander pods of
//! 2–6) with the heuristic default on every seat, record every living seat's
//! view with the game's card pool and the hidden truth, and a learner trains
//! the table net (`crabomination_ml::table`) — value, life, length, and with
//! the pool the belief head — on the GPU when built with `--features cuda`.
//!
//! Each checkpoint writes `latest.safetensors` (and `step<N>.safetensors`)
//! and a `stats.jsonl` line: training losses, held-out value cross-entropy
//! and slot-0 Brier score per mode family, and the belief read **B1** —
//! hand top-5 recall over the pool against the uniform pool and a base-rate
//! ranking (how often each pool card sits in a training hand).
//!
//! ```text
//! table_train --actors 8 --steps 20000 --out .ladder/table/run1
//! ```

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crabomination::recommend::Pilot;
use crabomination::table_selfplay::{Mode, draw_mode, mode_pool, mode_template, record_table_game};
use crabomination_ml::table::{TableConfig, TableTrainer};
use crabomination_nn::table::TableRow;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

struct Args {
    actors: usize,
    games: Option<u64>,
    steps: u64,
    batch: usize,
    lr: f64,
    reuse: f64,
    window: usize,
    min_window: usize,
    checkpoint_every: u64,
    holdout: f64,
    blocks: usize,
    pool: bool,
    belief: bool,
    modes: Option<Vec<Mode>>,
    /// Training rows kept per recorded row: a game's positions share one
    /// outcome, and a pod makes hundreds, so the learner fit recent games
    /// from near-duplicates (run 1: train win CE 0.34, held-out above
    /// chance on pods). Subsampling decorrelates the window.
    row_keep: f64,
    out: PathBuf,
    seed: u64,
}

fn parse() -> Args {
    let mut a = Args {
        actors: 8,
        games: None,
        steps: 20_000,
        batch: 256,
        lr: 1e-3,
        reuse: 3.0,
        window: 250_000,
        min_window: 20_000,
        checkpoint_every: 1_000,
        holdout: 0.05,
        blocks: 0,
        pool: true,
        belief: true,
        modes: None,
        row_keep: 0.25,
        out: PathBuf::from("table_run"),
        seed: 43,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--actors" => a.actors = val().parse().expect("--actors"),
            "--games" => a.games = Some(val().parse().expect("--games")),
            "--steps" => a.steps = val().parse().expect("--steps"),
            "--batch" => a.batch = val().parse().expect("--batch"),
            "--lr" => a.lr = val().parse().expect("--lr"),
            "--reuse" => a.reuse = val().parse().expect("--reuse"),
            "--window" => a.window = val().parse().expect("--window"),
            "--min-window" => a.min_window = val().parse().expect("--min-window"),
            "--checkpoint-every" => a.checkpoint_every = val().parse().expect("--checkpoint-every"),
            "--holdout" => a.holdout = val().parse().expect("--holdout"),
            "--blocks" => a.blocks = val().parse().expect("--blocks"),
            "--no-pool" => {
                a.pool = false;
                a.belief = false;
            }
            "--no-belief" => a.belief = false,
            "--modes" => {
                a.modes = Some(
                    val()
                        .split(',')
                        .map(|m| match m {
                            "sos" => Mode::SealedSos,
                            "fra" => Mode::SealedFra,
                            "cube" => Mode::Cube,
                            p if p.starts_with("pod") => Mode::Pod(p[3..].parse().expect("--modes podN")),
                            other => panic!("--modes: {other:?} (sos, fra, cube, pod2..pod6)"),
                        })
                        .collect(),
                )
            }
            "--row-keep" => a.row_keep = val().parse().expect("--row-keep"),
            "--out" => a.out = PathBuf::from(val()),
            "--seed" => a.seed = val().parse().expect("--seed"),
            other => panic!("unknown flag {other}"),
        }
    }
    a
}

/// A row's game family, for the per-family held-out reads.
fn family(m: Mode) -> &'static str {
    match m {
        Mode::SealedSos | Mode::SealedFra => "sealed",
        Mode::Cube => "cube",
        Mode::Pod(2) => "pod2",
        Mode::Pod(_) => "pods",
    }
}

fn is_holdout(traj: u32, frac: f64) -> bool {
    let mut h = traj as u64 ^ 0x9E37_79B9_7F4A_7C15;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    ((h >> 11) as f64 / (1u64 << 53) as f64) < frac
}

struct Shared {
    window: Mutex<std::collections::VecDeque<TableRow>>,
    /// Held-out rows: at most 4 per held-out game, reservoir-sampled over
    /// the whole run (run 1's first-come 20 k rows were a dozen pod games).
    val: Mutex<(Vec<(Mode, TableRow)>, u64)>,
    /// Per pool id, how often each pool card sat in a training hand.
    hand_counts: Mutex<HashMap<u64, Vec<u32>>>,
    rows_pushed: AtomicU64,
    games: AtomicU64,
    next_game: AtomicU64,
    stop: AtomicBool,
}

fn actor(sh: &Shared, a: &Args) {
    let vocab = crabomination::server::net_eval::vocab();
    loop {
        if sh.stop.load(Ordering::Relaxed) {
            return;
        }
        let g = sh.next_game.fetch_add(1, Ordering::Relaxed);
        if a.games.is_some_and(|n| g >= n) {
            return;
        }
        let seed = a.seed.wrapping_mul(1_000_003) ^ g.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut rng = StdRng::seed_from_u64(seed);
        let mode = match &a.modes {
            Some(ms) => ms[rng.random_range(0..ms.len())],
            None => draw_mode(&mut rng),
        };
        let t = mode_template(mode, seed);
        let pool = mode_pool(mode, vocab);
        let pilots = vec![Pilot::default(); t.players.len()];
        let max_actions = if matches!(mode, Mode::Pod(n) if n > 2) { 20_000 } else { 4_000 };
        let game = record_table_game(&t, &pilots, seed, max_actions, vocab, pool);
        sh.games.fetch_add(1, Ordering::Relaxed);
        let (mut val, train): (Vec<TableRow>, Vec<TableRow>) =
            game.rows.into_iter().partition(|r| is_holdout(r.traj, a.holdout));
        let train: Vec<TableRow> = train.into_iter().filter(|_| rng.random_bool(a.row_keep.clamp(0.0, 1.0))).collect();
        if !val.is_empty() {
            val.shuffle(&mut rng);
            val.truncate(4);
            let mut v = sh.val.lock().unwrap();
            for r in val {
                v.1 += 1;
                if v.0.len() < 20_000 {
                    v.0.push((mode, r));
                } else {
                    let j = rng.random_range(0..v.1);
                    if (j as usize) < v.0.len() {
                        v.0[j as usize] = (mode, r);
                    }
                }
            }
        }
        {
            let mut hc = sh.hand_counts.lock().unwrap();
            let counts = hc.entry(pool.pool.id()).or_insert_with(|| vec![0; pool.len()]);
            for r in &train {
                for h in r.hand.iter().skip(1) {
                    for &c in h {
                        if let Some(x) = counts.get_mut(c as usize) {
                            *x += 1;
                        }
                    }
                }
            }
        }
        sh.rows_pushed.fetch_add(train.len() as u64, Ordering::Relaxed);
        let mut w = sh.window.lock().unwrap();
        for r in train {
            if w.len() >= a.window {
                w.pop_front();
            }
            w.push_back(r);
        }
    }
}

fn main() {
    let a = parse();
    std::fs::create_dir_all(&a.out).expect("create --out");
    let vocab = crabomination::server::net_eval::vocab();
    let cfg = TableConfig { blocks: a.blocks, pool: a.pool, belief: a.belief, ..TableConfig::standard(vocab.size()) };
    let mut trainer = TableTrainer::new(&cfg, a.lr).expect("trainer");
    let latest = a.out.join("latest.safetensors");
    if latest.exists() {
        trainer.load(&latest).expect("resume latest.safetensors");
        eprintln!("resumed from {}", latest.display());
    }
    eprintln!(
        "table_train: {} actors, device {:?}, pool {}, belief {}, blocks {}, window {}, batch {}, reuse cap {}x",
        a.actors,
        trainer.device(),
        a.pool,
        a.belief,
        a.blocks,
        a.window,
        a.batch,
        a.reuse
    );
    let sh = Shared {
        window: Mutex::new(std::collections::VecDeque::new()),
        val: Mutex::new((Vec::new(), 0)),
        hand_counts: Mutex::new(HashMap::new()),
        rows_pushed: AtomicU64::new(0),
        games: AtomicU64::new(0),
        next_game: AtomicU64::new(0),
        stop: AtomicBool::new(false),
    };
    let start = Instant::now();
    let mut stats = std::fs::OpenOptions::new().create(true).append(true).open(a.out.join("stats.jsonl")).expect("stats");
    std::thread::scope(|scope| {
        for _ in 0..a.actors {
            // Engine games recurse deeply; each actor gets the engine's stack.
            std::thread::Builder::new()
                .stack_size(crabomination::server::ENGINE_STACK_BYTES)
                .spawn_scoped(scope, || actor(&sh, &a))
                .expect("spawn actor");
        }
        let mut rng = StdRng::seed_from_u64(a.seed ^ 0x7AB1E);
        let (mut step, mut ema) = (0u64, [0f32; 6]);
        while step < a.steps {
            let (wlen, pushed) = (sh.window.lock().unwrap().len(), sh.rows_pushed.load(Ordering::Relaxed));
            let actors_done = a.games.is_some_and(|n| sh.games.load(Ordering::Relaxed) >= n);
            let over_cap = (step + 1) as f64 * a.batch as f64 > a.reuse * pushed as f64;
            if wlen < a.min_window.min(a.window) || (over_cap && !actors_done) {
                if actors_done && wlen == 0 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            let batch: Vec<TableRow> = {
                let w = sh.window.lock().unwrap();
                (0..a.batch).map(|_| w[rng.random_range(0..w.len())].clone()).collect()
            };
            let refs: Vec<&TableRow> = batch.iter().collect();
            let l = trainer.train_step(&refs).expect("train step");
            for (e, v) in ema.iter_mut().zip([l.total, l.win, l.life, l.len, l.hand, l.lib]) {
                *e = if step == 0 { v } else { 0.99 * *e + 0.01 * v };
            }
            step += 1;
            if step.is_multiple_of(a.checkpoint_every) || step == a.steps {
                trainer.save(&latest).expect("save latest");
                trainer.save(&a.out.join(format!("step{step}.safetensors"))).expect("save step");
                let line = checkpoint(&trainer, &sh, step, ema, start.elapsed().as_secs_f64());
                eprintln!("{line}");
                writeln!(stats, "{line}").ok();
            }
        }
        sh.stop.store(true, Ordering::Relaxed);
    });
    eprintln!("done: {} games, {} rows", sh.games.load(Ordering::Relaxed), sh.rows_pushed.load(Ordering::Relaxed));
}

/// The held-out reads, as one JSON line.
fn checkpoint(t: &TableTrainer, sh: &Shared, step: u64, ema: [f32; 6], secs: f64) -> String {
    let guard = sh.val.lock().unwrap();
    let val = &guard.0;
    let base: HashMap<u64, Vec<u16>> = sh
        .hand_counts
        .lock()
        .unwrap()
        .iter()
        .map(|(id, c)| {
            let mut order: Vec<u16> = (0..c.len() as u16).collect();
            order.sort_by(|&x, &y| c[y as usize].cmp(&c[x as usize]));
            (*id, order)
        })
        .collect();
    let mut fams: Vec<String> = Vec::new();
    for fam in ["sealed", "cube", "pod2", "pods"] {
        let rows: Vec<&TableRow> = val.iter().filter(|(m, _)| family(*m) == fam).map(|(_, r)| r).take(4_000).collect();
        if rows.is_empty() {
            continue;
        }
        let states: Vec<&crabomination_nn::table::TableState> = rows.iter().map(|r| &r.state).collect();
        let mut ce = 0f64;
        let mut brier = 0f64;
        for (chunk_s, chunk_r) in states.chunks(128).zip(rows.chunks(128)) {
            let p = t.predict(chunk_s).expect("predict");
            for (pr, r) in p.iter().zip(chunk_r) {
                for (q, w) in pr.iter().zip(&r.win) {
                    if *w > 0.0 {
                        ce -= *w as f64 * (q.max(1e-6) as f64).ln();
                    }
                }
                brier += ((pr[0] - r.win[0]) as f64).powi(2);
            }
        }
        let n = rows.len() as f64;
        let (rec, uni, br, nb) = t.belief_recall(&rows, 5, &base).unwrap_or((f32::NAN, f32::NAN, f32::NAN, 0));
        fams.push(format!(
            "\"{fam}\":{{\"n\":{},\"ce\":{:.4},\"brier\":{:.4},\"b1\":{:.4},\"b1_uniform\":{:.4},\"b1_base\":{:.4},\"b1_n\":{nb}}}",
            rows.len(),
            ce / n,
            brier / n,
            rec,
            uni,
            br
        ));
    }
    format!(
        "{{\"step\":{step},\"loss\":{:.4},\"win\":{:.4},\"life\":{:.4},\"len\":{:.4},\"hand\":{:.4},\"lib\":{:.4},\"games\":{},\"rows\":{},\"secs\":{secs:.0},{}}}",
        ema[0],
        ema[1],
        ema[2],
        ema[3],
        ema[4],
        ema[5],
        sh.games.load(Ordering::Relaxed),
        sh.rows_pushed.load(Ordering::Relaxed),
        fams.join(",")
    )
}
