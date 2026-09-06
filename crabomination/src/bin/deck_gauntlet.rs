//! One decklist against a fixed field of bot-built decks opened from deep
//! sealed pools.
//!
//! `deck_duel` races two lists and `recommend_pool` ranks builds of one pool
//! against a 6-pack field of single noisy builds. Neither answers "how does
//! *this* list do against good decks". The client's `--opponent-packs`
//! opponent — a best-of-16 build from an N-pack pool
//! ([`random_sealed_opponent_packs`]) — is that opponent, and this bin plays
//! a list against a fixed field of them in antithetic seat pairs, pooling
//! the paired estimate over the field. The field is a pure function of
//! `--field` and `--field-seed`, so every variant of a deck faces the same
//! opponents and the same shuffles (`--seed`): differences between two runs
//! are differences between the lists.
//!
//! ```text
//! deck_gauntlet --deck decks/x.txt [--field 9,12,16,20x6] [--field-seed 0xDECC0000]
//!     [--pairs 100] [--pilot PILOT] [--opp-pilot PILOT] [--seed 43]
//!     [--threads N] [--chunk 10] [--out results.txt] [--replays DIR]
//!     [--dump-field DIR] [--max-actions 4000]
//! PILOT: dflt | convlands | stunhold | gypick | both | convrarest | convfetch | trickmodes | trickoff | convfixes | mcts64 | mcts128 | mcts256 | mcts256-convlands | mcts256-stunhold | mcts256-gypick | mcts256-both | mcts256-convfixes
//! ```
//!
//! Two paths. The fast one is the in-process paired loop
//! ([`simulate_match_pairs_piloted`]), which records nothing. `--replays DIR`
//! plays every game through the server's [`run_match`] instead — the only
//! path that writes `CRAB_REPLAY_DIR` replays — and appends one `GAME` line
//! per game to `--out`, tagged (`@o<opp>g<pair>s<order>`) in both seat names
//! so a replay maps back to its game and an aborted run can be resumed by
//! hand from the lines it got through. The server's bot watchdog panics
//! after 15 s without an accepted action, which under `panic = "abort"`
//! ends the whole process: keep `--threads` low enough for a searched
//! decision to stay far under that.

use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crabomination::cube::CardFactory;
use crabomination::recommend::{Pilot, paired_stat, simulate_match_pairs_piloted, wilson};
use crabomination::selfplay::random_sealed_opponent_packs;
use crabomination::server::{EvalWeights, HeuristicBot, MctsBot, MctsConfig, SeatOccupant, run_match};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

struct Opts {
    deck: String,
    field: String,
    field_seed: u64,
    pairs: usize,
    pilot: String,
    opp_pilot: String,
    seed: u64,
    threads: usize,
    chunk: usize,
    out: Option<String>,
    replays: Option<String>,
    dump_field: Option<String>,
    max_actions: usize,
}

struct Opp {
    idx: usize,
    packs: usize,
    seed: u64,
    label: String,
    deck: Vec<CardFactory>,
}

/// Wins/losses from the deck's point of view, plus the per-pair sign the
/// paired estimate reads (+1 deck swept, -1 opponent swept, 0 split).
#[derive(Default, Clone)]
struct Cell {
    wins: u32,
    losses: u32,
    undecided: u32,
    pairs: Vec<i8>,
}

impl Cell {
    fn absorb(&mut self, o: &Cell) {
        self.wins += o.wins;
        self.losses += o.losses;
        self.undecided += o.undecided;
        self.pairs.extend_from_slice(&o.pairs);
    }
    fn decided(&self) -> u32 {
        self.wins + self.losses
    }
    fn pct(&self) -> f64 {
        100.0 * self.wins as f64 / self.decided().max(1) as f64
    }
}

fn usage() -> ! {
    eprintln!(
        "usage: deck_gauntlet --deck FILE [--field 9,12,16,20x6] [--field-seed 0xDECC0000]\n\
         \x20   [--pairs 100] [--pilot PILOT] [--opp-pilot PILOT] [--seed 43] [--threads N]\n\
         \x20   [--chunk 10] [--out FILE] [--replays DIR] [--dump-field DIR] [--max-actions 4000]\n\
         PILOT: dflt | convlands | stunhold | gypick | both | convrarest | convfetch | trickmodes | trickoff | convfixes | mcts64 | mcts128 | mcts256 | mcts256-convlands | mcts256-stunhold | mcts256-gypick | mcts256-both | mcts256-convfixes"
    );
    std::process::exit(2)
}

fn parse_u64(s: &str) -> Option<u64> {
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => u64::from_str_radix(h, 16).ok(),
        None => s.parse().ok(),
    }
}

fn parse_opts() -> Opts {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut o = Opts {
        deck: String::new(),
        field: "9,12,16,20x6".into(),
        field_seed: 0xDECC_0000,
        pairs: 100,
        pilot: "dflt".into(),
        opp_pilot: "dflt".into(),
        seed: 43,
        threads: 0,
        chunk: 10,
        out: None,
        replays: None,
        dump_field: None,
        max_actions: 4_000,
    };
    let mut i = 0;
    while i < argv.len() {
        let val = |i: usize| -> &str { argv.get(i + 1).map(String::as_str).unwrap_or_else(|| usage()) };
        match argv[i].as_str() {
            "--deck" => o.deck = val(i).to_string(),
            "--field" => o.field = val(i).to_string(),
            "--field-seed" => o.field_seed = parse_u64(val(i)).unwrap_or_else(|| usage()),
            "--pairs" => o.pairs = val(i).parse().unwrap_or_else(|_| usage()),
            "--pilot" => o.pilot = val(i).to_string(),
            "--opp-pilot" => o.opp_pilot = val(i).to_string(),
            "--seed" => o.seed = parse_u64(val(i)).unwrap_or_else(|| usage()),
            "--threads" => o.threads = val(i).parse().unwrap_or_else(|_| usage()),
            "--chunk" => o.chunk = val(i).parse().unwrap_or_else(|_| usage()),
            "--out" => o.out = Some(val(i).to_string()),
            "--replays" => o.replays = Some(val(i).to_string()),
            "--dump-field" => o.dump_field = Some(val(i).to_string()),
            "--max-actions" => o.max_actions = val(i).parse().unwrap_or_else(|_| usage()),
            _ => usage(),
        }
        i += 2;
    }
    if o.deck.is_empty() {
        usage();
    }
    o.chunk = o.chunk.max(1);
    o
}

fn parse_pilot(name: &str) -> Option<Pilot> {
    Some(match name {
        "dflt" => Pilot::Scored(EvalWeights::default()),
        "convlands" => Pilot::Scored(EvalWeights::converge_lands_on()),
        "stunhold" => Pilot::Scored(EvalWeights::stun_x_hold_on()),
        "gypick" => Pilot::Scored(EvalWeights::own_graveyard_picks_on()),
        "convrarest" => Pilot::Scored(EvalWeights::converge_rarest_on()),
        "convfetch" => Pilot::Scored(EvalWeights::converge_fetch_on()),
        "trickmodes" => Pilot::Scored(EvalWeights::trick_modes_combat_only_on()),
        "convfixes" => Pilot::Scored(EvalWeights::converge_fixes_on()),
        "trickoff" => Pilot::Scored(EvalWeights::trick_modes_off()),
        // 2026-09-06 targeting work: hostile player slots aimed at the
        // opponent (seat flag), the other player as a cast-time arm, both;
        // the own-graveyard pick with the X=0 no-op prune; everything.
        "hostile" => Pilot::Scored(EvalWeights::hostile_player_targets_on()),
        "parms" => Pilot::Scored(EvalWeights::player_target_arms_on()),
        "targetfix" => Pilot::Scored(EvalWeights::target_fixes_on()),
        "x0skip" => Pilot::Scored(EvalWeights::skip_noop_x0_on()),
        "gyfix" => Pilot::Scored(EvalWeights::graveyard_fixes_on()),
        "allfix" => Pilot::Scored(EvalWeights::all_fixes_on()),
        "r67off" => Pilot::Scored(EvalWeights::round67_off()),
        "mcts256-r67off" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::round67_off(),
            ..MctsConfig::default()
        }),
        "mcts256-targetfix" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::target_fixes_on(),
            ..MctsConfig::default()
        }),
        "mcts256-allfix" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::all_fixes_on(),
            ..MctsConfig::default()
        }),
        "both" => Pilot::Scored(EvalWeights { own_graveyard_picks: true, ..EvalWeights::stun_x_hold_on() }),
        "mcts64" => Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights::default(),
            ..MctsConfig::default()
        }),
        "mcts128" => Pilot::Mcts(MctsConfig {
            iterations: 128,
            horizon_turns: 3,
            weights: EvalWeights::default(),
            ..MctsConfig::default()
        }),
        "mcts256" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::default(),
            ..MctsConfig::default()
        }),
        "mcts256-convlands" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::converge_lands_on(),
            ..MctsConfig::default()
        }),
        "mcts256-stunhold" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::stun_x_hold_on(),
            ..MctsConfig::default()
        }),
        "mcts256-gypick" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::own_graveyard_picks_on(),
            ..MctsConfig::default()
        }),
        "mcts256-convfixes" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::converge_fixes_on(),
            ..MctsConfig::default()
        }),
        "mcts256-both" => Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights { own_graveyard_picks: true, ..EvalWeights::stun_x_hold_on() },
            ..MctsConfig::default()
        }),
        _ => return None,
    })
}

fn build_bot(p: &Pilot) -> Box<dyn crabomination::server::Bot> {
    match p {
        Pilot::Scored(w) => Box::new(HeuristicBot::with_weights(*w)),
        Pilot::Uniform => Box::new(HeuristicBot::uniform_baseline()),
        Pilot::Mcts(cfg) => Box::new(MctsBot::new(*cfg)),
    }
}

fn load_deck(path: &str) -> Vec<CardFactory> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(2);
    });
    let parse = crabomination::decklist::parse_decklist(&text);
    if !parse.unknown.is_empty() {
        eprintln!("{path}: unresolved names — fix these before playing:");
        for u in &parse.unknown {
            eprintln!("  {u}");
        }
        std::process::exit(1);
    }
    if !parse.sideboard.is_empty() {
        eprintln!("{path}: {} sideboard cards ignored", parse.sideboard.len());
    }
    if parse.main.len() < 40 {
        eprintln!("{path}: only {} cards — expected at least 40", parse.main.len());
        std::process::exit(1);
    }
    parse.main
}

/// `9,12,16,20x6`: those pack counts, six opponents each. Seed
/// `base + packs*1000 + i`, so a field is reproducible from its spec.
fn build_field(spec: &str, base: u64) -> Vec<Opp> {
    let (packs, n) = spec.split_once('x').unwrap_or((spec, "6"));
    let n: usize = n.trim().parse().unwrap_or_else(|_| usage());
    let mut out = Vec::new();
    for p in packs.split(',') {
        let p: usize = p.trim().parse().unwrap_or_else(|_| usage());
        for i in 0..n {
            let seed = base.wrapping_add(p as u64 * 1000 + i as u64);
            let (deck, label) = random_sealed_opponent_packs(seed, p);
            out.push(Opp { idx: out.len(), packs: p, seed, label, deck });
        }
    }
    out
}

fn decklist_text(deck: &[CardFactory]) -> String {
    let mut counts: std::collections::BTreeMap<&'static str, usize> = Default::default();
    for f in deck {
        *counts.entry(f().name).or_default() += 1;
    }
    counts.iter().map(|(name, n)| format!("{n} {name}\n")).collect()
}

fn deck_stem(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

struct Job {
    opp: usize,
    chunk: usize,
    pairs: usize,
    seed: u64,
}

fn main() {
    // Same 32 MB stack workaround as `deck_duel` / `bot_ladder` — debug
    // builds need it for `run_effect`'s frame; harmless in release.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run)
        .expect("spawn driver thread")
        .join()
        .expect("driver thread panicked");
}

fn run() {
    let o = parse_opts();
    let pilot = parse_pilot(&o.pilot).unwrap_or_else(|| usage());
    let opp_pilot = parse_pilot(&o.opp_pilot).unwrap_or_else(|| usage());
    let deck = load_deck(&o.deck);
    let deck_name = deck_stem(&o.deck);
    let field = build_field(&o.field, o.field_seed);

    if let Some(dir) = &o.dump_field {
        std::fs::create_dir_all(dir).expect("create dump dir");
        let mut index = String::new();
        for opp in &field {
            let file = format!("opp-{:02}-{}p-{}.txt", opp.idx, opp.packs, opp.seed);
            std::fs::write(format!("{dir}/{file}"), decklist_text(&opp.deck)).expect("write opponent list");
            index.push_str(&format!("{:02}\t{}\t{}\t{}\t{}\n", opp.idx, opp.packs, opp.seed, opp.label, file));
        }
        std::fs::write(format!("{dir}/index.tsv"), index).expect("write index");
        eprintln!("field dumped to {dir} ({} opponents)", field.len());
    }

    if let Some(dir) = &o.replays {
        std::fs::create_dir_all(dir).expect("create replay dir");
        // The recorder reads the variable when a match begins; set it before
        // any worker thread exists. Single-threaded here, so this is sound.
        unsafe { std::env::set_var("CRAB_REPLAY_DIR", dir) };
    }

    let chunks = o.pairs.div_ceil(o.chunk);
    let mut jobs: Vec<Job> = Vec::new();
    for opp in &field {
        for c in 0..chunks {
            let pairs = o.chunk.min(o.pairs - c * o.chunk);
            let seed = o.seed ^ ((opp.idx as u64 * 7919 + c as u64 + 1).wrapping_mul(GOLDEN));
            jobs.push(Job { opp: opp.idx, chunk: c, pairs, seed });
        }
    }
    let threads = if o.threads == 0 {
        std::thread::available_parallelism().map(|n| n.get().saturating_sub(1)).unwrap_or(1).max(1)
    } else {
        o.threads
    }
    .min(jobs.len().max(1));

    let path = if o.replays.is_some() { "server+replays" } else { "fast" };
    let banner = format!(
        "deck_gauntlet: {} ({} cards) as the deck, pilot {} vs opponents piloted {}\n\
         field {} seed {:#x} = {} opponents; {} pairs each ({} games), chunk {}, seed {}, {} threads, path {}",
        o.deck,
        deck.len(),
        o.pilot,
        o.opp_pilot,
        o.field,
        o.field_seed,
        field.len(),
        o.pairs,
        2 * o.pairs * field.len(),
        o.chunk,
        o.seed,
        threads,
        path
    );
    println!("{banner}");

    let out_file: Option<Mutex<std::fs::File>> = o.out.as_ref().map(|p| {
        Mutex::new(
            std::fs::OpenOptions::new().create(true).append(true).open(p).expect("open --out"),
        )
    });

    let t0 = Instant::now();
    let cursor = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Cell)>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024);
            let jobs = &jobs;
            let field = &field;
            let deck = &deck;
            let deck_name = &deck_name;
            let results = &results;
            let cursor = &cursor;
            let done = &done;
            let out_file = &out_file;
            let o = &o;
            builder
                .spawn_scoped(s, move || {
                    loop {
                        let j = cursor.fetch_add(1, Ordering::Relaxed);
                        if j >= jobs.len() {
                            break;
                        }
                        let job = &jobs[j];
                        let opp = &field[job.opp];
                        let cell = if o.replays.is_some() {
                            play_chunk_server(deck, deck_name, opp, job, [pilot, opp_pilot], o, out_file)
                        } else {
                            let t = simulate_match_pairs_piloted(
                                deck,
                                &opp.deck,
                                job.pairs,
                                [pilot, opp_pilot],
                                o.max_actions,
                                job.seed,
                            );
                            Cell { wins: t.wins_a, losses: t.wins_b, undecided: t.undecided, pairs: t.pairs.clone() }
                        };
                        results.lock().unwrap().push((job.opp, cell));
                        let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                        if n.is_multiple_of(threads.max(1)) || n == jobs.len() {
                            eprintln!(
                                "  {n}/{} jobs, {:.0}s elapsed",
                                jobs.len(),
                                t0.elapsed().as_secs_f64()
                            );
                        }
                    }
                })
                .expect("spawn worker");
        }
    });
    let secs = t0.elapsed().as_secs_f64();

    // ── report ──
    let results = results.into_inner().unwrap();
    let mut per_opp: Vec<Cell> = vec![Cell::default(); field.len()];
    for (i, c) in &results {
        per_opp[*i].absorb(c);
    }
    let mut report = String::new();
    report.push_str("\nopp packs  seed          wins losses undec   win%   pairs(+/0/-)\n");
    let mut per_packs: std::collections::BTreeMap<usize, Cell> = Default::default();
    let mut all = Cell::default();
    for opp in &field {
        let c = &per_opp[opp.idx];
        let plus = c.pairs.iter().filter(|&&s| s > 0).count();
        let zero = c.pairs.iter().filter(|&&s| s == 0).count();
        let minus = c.pairs.iter().filter(|&&s| s < 0).count();
        report.push_str(&format!(
            "{:>3} {:>5}  {:<12} {:>5} {:>6} {:>5}  {:5.1}%  {plus}/{zero}/{minus}\n",
            opp.idx,
            opp.packs,
            opp.seed,
            c.wins,
            c.losses,
            c.undecided,
            c.pct()
        ));
        per_packs.entry(opp.packs).or_default().absorb(c);
        all.absorb(c);
    }
    report.push('\n');
    for (packs, c) in &per_packs {
        let (lo, hi) = wilson(c.wins, c.decided(), 1.96);
        let paired = paired_stat(&c.pairs).map(|s| format!("paired {:.1}% ±{:.1}", 100.0 * s.p, 196.0 * s.se)).unwrap_or_default();
        report.push_str(&format!(
            "packs {packs:>2}: {:5.1}% [{:.1}%, {:.1}%] over {} decided; {paired}\n",
            c.pct(),
            100.0 * lo,
            100.0 * hi,
            c.decided()
        ));
    }
    let (lo, hi) = wilson(all.wins, all.decided(), 1.96);
    report.push_str(&format!(
        "\noverall: {} - {} ({} undecided) = {:.2}% unpaired [{:.1}%, {:.1}%]\n",
        all.wins,
        all.losses,
        all.undecided,
        all.pct(),
        100.0 * lo,
        100.0 * hi
    ));
    let (p, se) = match paired_stat(&all.pairs) {
        Some(s) => {
            let plus = all.pairs.iter().filter(|&&x| x > 0).count();
            let zero = all.pairs.iter().filter(|&&x| x == 0).count();
            let minus = all.pairs.iter().filter(|&&x| x < 0).count();
            report.push_str(&format!(
                "paired:  {} pairs — {plus} deck sweeps, {minus} opponent sweeps, {zero} splits\n\
                 \x20        deck win% {:.2}%  [{:.2}%, {:.2}%]  (±{:.2} pts)\n",
                all.pairs.len(),
                100.0 * s.p,
                100.0 * (s.p - 1.96 * s.se),
                100.0 * (s.p + 1.96 * s.se),
                196.0 * s.se
            ));
            (100.0 * s.p, 100.0 * s.se)
        }
        None => (all.pct(), f64::NAN),
    };
    let result_line = format!(
        "RESULT deck={deck_name} pilot={} opp_pilot={} field={}/{:#x} pairs={} seed={} games={} win={p:.2} se={se:.2} undecided={} secs={secs:.0}",
        o.pilot,
        o.opp_pilot,
        o.field,
        o.field_seed,
        o.pairs,
        o.seed,
        all.decided() + all.undecided,
        all.undecided
    );
    report.push_str(&format!("\n{result_line}\n"));
    print!("{report}");
    if let Some(f) = &out_file {
        let mut f = f.lock().unwrap();
        let _ = writeln!(f, "# {}", banner.replace('\n', "\n# "));
        let _ = write!(f, "{report}");
    }
}

/// One chunk of pairs through the server path. Mirrors the in-process
/// loop's seeding (one shuffle stream per pair, the game RNG and the bots'
/// tie-break jitter drawn from it) so a pair's two games deal the same
/// hands with the seats swapped.
fn play_chunk_server(
    deck: &[CardFactory],
    deck_name: &str,
    opp: &Opp,
    job: &Job,
    pilots: [Pilot; 2],
    o: &Opts,
    out: &Option<Mutex<std::fs::File>>,
) -> Cell {
    use crabomination::game::GameState;
    use crabomination::player::Player;
    let mut cell = Cell::default();
    for k in 0..job.pairs {
        let pair_seed = job.seed.wrapping_add((k as u64).wrapping_mul(GOLDEN));
        let global_pair = job.chunk * o.chunk + k;
        let mut score = 0i8;
        let mut decided_both = true;
        for order in 0..2usize {
            let deck_seat = order; // order 0: the deck on the play
            let tag = format!("@o{}g{global_pair}s{order}", opp.idx);
            let name = |seat: usize| {
                if seat == deck_seat {
                    format!("{deck_name} {tag}")
                } else {
                    format!("{} {tag}", opp.label)
                }
            };
            let mut g = GameState::new(vec![Player::new(0, name(0)), Player::new(1, name(1))]);
            for seat in 0..2 {
                let cards: &[CardFactory] = if seat == deck_seat { deck } else { &opp.deck };
                for &f in cards {
                    g.add_card_to_library(seat, crabomination::cube::card_arc(f));
                }
                g.players[seat].wants_ui = true;
            }
            let seated = if deck_seat == 0 { [pilots[0], pilots[1]] } else { [pilots[1], pilots[0]] };
            for (seat, p) in seated.iter().enumerate() {
                if let Pilot::Scored(w) = p {
                    g.players[seat].smart_tap = w.smart_tap;
                    g.players[seat].converge_rarest = w.converge_rarest;
                }
                // The polarity flag reaches search seats too (see the same
                // push in `recommend::play_seeded_game`).
                g.players[seat].hostile_player_targets = match p {
                    Pilot::Scored(w) => w.hostile_player_targets,
                    Pilot::Mcts(cfg) => cfg.weights.hostile_player_targets,
                    Pilot::Uniform => false,
                };
            }
            let mut rng = StdRng::seed_from_u64(pair_seed);
            for seat in 0..2 {
                g.players[seat].library.shuffle(&mut rng);
            }
            g.rng.reseed(rng.random::<u64>());
            crabomination::server::bot::set_jitter_seed(Some(pair_seed));
            let occupants: Vec<SeatOccupant> = seated.iter().map(|p| SeatOccupant::Bot(build_bot(p))).collect();
            let outcome = run_match(g, occupants);
            let winner = match outcome.winner {
                Some(Some(seat)) if seat == deck_seat => "deck",
                Some(Some(_)) => "opp",
                Some(None) => "draw",
                None => "abort",
            };
            match winner {
                "deck" => {
                    cell.wins += 1;
                    score += 1;
                }
                "opp" => {
                    cell.losses += 1;
                    score -= 1;
                }
                _ => {
                    cell.undecided += 1;
                    decided_both = false;
                }
            }
            if let Some(f) = out {
                let life: Vec<String> = outcome.final_life_totals.iter().map(|l| l.to_string()).collect();
                let mut f = f.lock().unwrap();
                let _ = writeln!(
                    f,
                    "GAME opp={} packs={} oseed={} pair={global_pair} order={order} seed={pair_seed} deck_seat={deck_seat} winner={winner} turns={} life={} tag={tag}",
                    opp.idx,
                    opp.packs,
                    opp.seed,
                    outcome.final_turn,
                    life.join("/")
                );
            }
        }
        if decided_both {
            cell.pairs.push(score.signum());
        }
    }
    cell
}
