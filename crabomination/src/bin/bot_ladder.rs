//! Bot-vs-bot ladder: play two evaluation profiles head to head and report
//! the win rate with an honest error bar.
//!
//! The bot's unit tests assert individual decisions in hand-built states.
//! That catches "did this rule fire", but it cannot answer "is the bot
//! stronger", which is the only question that matters for an evaluation
//! change — weights are not right or wrong in isolation, they are better or
//! worse than the weights they replace. This binary is that measurement.
//!
//! Design notes, in decreasing order of how easy they are to get wrong:
//!
//! * **Mirror matches.** Both profiles play the *same* decklist, so deck
//!   strength cancels and the only difference between the seats is the
//!   pilot.
//! * **Seats swapped every game.** Turn order is worth real win rate on its
//!   own; `simulate_match_games_piloted` alternates which side starts, so
//!   it can't be confounded with the profile under test.
//! * **Antithetic seat pairs** (default; `--unpaired` for the old
//!   behaviour). Swapping seats across *independently shuffled* games
//!   averages deal luck away; replaying one shuffle from both seats
//!   *cancels* it. In a 40-card mirror the deal is most of the variance,
//!   so this buys real precision for free — see
//!   `simulate_match_pairs_piloted`. The realized efficiency is measured
//!   and printed rather than assumed, and the unpaired win rate is still
//!   reported next to it as the control.
//! * **Several archetypes.** A single deck measures "is this profile better
//!   at mono-red", which is how a weight set gets overfit. The deck list
//!   below deliberately spans aggro / fliers / midrange / control so a
//!   change that only helps one style shows up as a split result.
//! * **Wilson intervals**, not the normal approximation: well-behaved at
//!   small n and at the extremes, where the naive interval collapses to
//!   zero width and claims certainty it hasn't earned.
//! * **Undecided games are excluded** from the win rate and reported
//!   separately. A profile that stalls more games isn't winning them.
//!
//! ```text
//! cargo run --bin bot_ladder -- --games 200
//! cargo run --bin bot_ladder -- --a baseline --b v2 --games 400 --seed 7
//! ```

// Allocator swap, on by default. The bench workload spends ~16 % of its
// instructions in malloc/free/memcpy; the swap is worth +21.9 % here and
// more as actors scale (PERF.md). `--no-default-features` restores the
// system allocator, which is how the A/B is run. A `#[global_allocator]` is
// a whole-program choice, so it lives in the binary rather than the library.
#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crabomination::cube::{CardFactory, color_pair_name, cube_deck, random_color_pair};
use rand::SeedableRng;
use rand::rngs::StdRng;
use crabomination::recommend::{
    Pilot, SimCost, paired_stat, simulate_match_games_piloted, simulate_match_pairs_piloted,
    wilson,
};
use crabomination::server::{EvalWeights, MctsConfig};
use crabomination::sos_mode::{College, sos_deck};

/// One archetype in the ladder. Mirror-matched, so this is the deck *both*
/// profiles pilot.
struct Archetype {
    name: &'static str,
    deck: Vec<CardFactory>,
}

fn deck(spec: &[(CardFactory, usize)]) -> Vec<CardFactory> {
    let mut d = Vec::new();
    for &(f, n) in spec {
        for _ in 0..n {
            d.push(f);
        }
    }
    d
}

/// The ladder field. Each is a legal-ish 40-card limited-style deck built
/// from staples, chosen so the profiles under test are exercised on more
/// than one axis:
///
/// * `mono_red_aggro` — races and burns, so the *life* term dominates.
/// * `azorius_skies` — evasion and lifelink, so the *keyword* term
///   dominates: the whole point of scoring flying by power is that these
///   boards get evaluated differently.
/// * `golgari_midrange` — ground stalls and trades, where creature-body
///   valuation decides blocks and removal targets.
/// * `dimir_control` — removal, counters, card draw; the profile has to
///   value cards in hand and the board it is answering.
fn archetypes() -> Vec<Archetype> {
    use crabomination::catalog as c;
    vec![
        Archetype {
            name: "mono-red aggro",
            deck: deck(&[
                (c::mountain as CardFactory, 17),
                (c::lightning_bolt, 4),
                (c::shock, 3),
                (c::goblin_guide, 4),
                (c::monastery_swiftspear, 3),
                (c::gray_ogre, 3),
                (c::hill_giant, 3),
                (c::fire_elemental, 2),
                (c::shivan_dragon, 1),
            ]),
        },
        Archetype {
            name: "azorius skies",
            deck: deck(&[
                (c::plains as CardFactory, 9),
                (c::island, 8),
                (c::wind_drake, 4),
                (c::air_elemental, 3),
                (c::serra_angel, 2),
                (c::baneslayer_angel, 1),
                (c::wall_of_omens, 3),
                (c::pacifism, 3),
                (c::swords_to_plowshares, 3),
                (c::divination, 2),
                (c::counterspell, 2),
            ]),
        },
        Archetype {
            name: "golgari midrange",
            deck: deck(&[
                (c::forest as CardFactory, 9),
                (c::swamp, 8),
                (c::llanowar_elves, 4),
                (c::elvish_mystic, 3),
                (c::grizzly_bears, 3),
                (c::centaur_courser, 3),
                (c::craw_wurm, 2),
                (c::ambush_viper, 3),
                (c::vampire_nighthawk, 2),
                (c::sengir_vampire, 1),
                (c::doom_blade, 3),
                (c::rancor, 2),
                (c::giant_growth, 2),
            ]),
        },
        Archetype {
            name: "dimir control",
            deck: deck(&[
                (c::island as CardFactory, 9),
                (c::swamp, 9),
                (c::counterspell, 4),
                (c::doom_blade, 4),
                (c::divination, 4),
                (c::prodigal_sorcerer, 3),
                (c::vampire_nighthawk, 3),
                (c::sengir_vampire, 2),
                (c::air_elemental, 2),
            ]),
        },
    ]
}

/// Seeded random two-color decks drawn from the full cube pool.
///
/// The four hand-built archetypes above are a narrow slice: vanilla
/// creatures, basic removal, and — importantly — not one card that scries,
/// mills, makes a token or ticks a planeswalker. A change can only be
/// measured on cards the deck set actually contains, so measuring solely
/// on them both risks overfitting to that slice and leaves whole families
/// of known bot gaps unexercised.
///
/// Seeded from `--seed` so a run is reproducible, and each pair plays a
/// mirror of itself for the same reason the fixed decks do: deck strength
/// cancels and only the pilots differ.
fn cube_archetypes(seed: u64, count: usize) -> Vec<Archetype> {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xC0BE_5EED);
    (0..count)
        .map(|_| {
            let colors = random_color_pair(&mut rng);
            Archetype {
                name: Box::leak(format!("cube {}", color_pair_name(colors)).into_boxed_str()),
                deck: cube_deck(colors, &mut rng),
            }
        })
        .collect()
}

/// Seeded SOS college mirrors — one 60-card `sos_mode` deck per college.
///
/// The fixed and cube decks measure the bot on the staple catalog; none of
/// them contain a prepare creature, a ward body, an Opus payoff, or a
/// school land's surveil. A change aimed at Secrets of Strixhaven play can
/// only be measured on decks that hold those cards, and per-college rows
/// give the same split view the four fixed archetypes do — a change that
/// only helps Prismari shows up as a split result.
///
/// Seeded from `--seed` (same construction `bot_probe --deck sos` uses, so
/// the two tools describe the same decks), and each college plays a mirror
/// of itself for the same reason the fixed decks do: deck strength cancels
/// and only the pilots differ.
fn sos_archetypes(seed: u64) -> Vec<Archetype> {
    let mut rng = StdRng::seed_from_u64(seed ^ 0x0505_ACAD);
    College::ALL
        .into_iter()
        .map(|college| Archetype {
            name: Box::leak(format!("sos {}", college.name()).into_boxed_str()),
            deck: sos_deck(college, &mut rng),
        })
        .collect()
}

/// Seeded sealed mirror decks — the learned-eval gate. Each row is one
/// 6-pack SOS sealed pool built by the heuristic builder, played as a
/// mirror: both pilots get the *same 40 cards*, so deck strength and build
/// quality cancel and the row measures piloting alone. That is the honest
/// comparison for the value net, which trains on sealed self-play — the
/// college mirrors are constructed 60-card decks it never sees.
fn sealed_archetypes(seed: u64, count: usize) -> Vec<Archetype> {
    (0..count as u64)
        .map(|i| {
            let salt = |k: u64| {
                seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x5E_A1ED)
            };
            let pool = crabomination::selfplay::sealed_pool(salt(1));
            let deck = crabomination::selfplay::heuristic_sealed_build(&pool, salt(2));
            Archetype {
                name: Box::leak(format!("sealed #{i}").into_boxed_str()),
                deck,
            }
        })
        .collect()
}

fn parse_profile(name: &str) -> Option<Pilot> {
    match name {
        "baseline" => Some(Pilot::Scored(EvalWeights::baseline())),
        "landseq" => Some(Pilot::Scored(EvalWeights::land_sequencing())),
        "mull" => Some(Pilot::Scored(EvalWeights::mulligan_quality())),
        "gang" => Some(Pilot::Scored(EvalWeights::block_gang_search())),
        // Paired re-tests of ideas dropped on unpaired nulls, rebased onto
        // the current default so the ladder measures the idea alone.
        "landseq2" => Some(Pilot::Scored(EvalWeights::land_sequencing_default())),
        "mull2" => Some(Pilot::Scored(EvalWeights::mulligan_quality_default())),
        "race2" => Some(Pilot::Scored(EvalWeights::attack_race_default())),
        "look1" => Some(Pilot::Scored(EvalWeights::lookahead1_default())),
        "look2" => Some(Pilot::Scored(EvalWeights::lookahead2_default())),
        "smarttap" => Some(Pilot::Scored(EvalWeights::smart_tap_on())),
        "det1" => Some(Pilot::Scored(EvalWeights::determinized())),
        "det3" => Some(Pilot::Scored(EvalWeights::determinized3())),
        "net" => Some(Pilot::Scored(EvalWeights::net_eval())),
        "net-blend" => Some(Pilot::Scored(EvalWeights::net_eval_blend())),
        "net-blend300" => Some(Pilot::Scored(EvalWeights::net_eval_blend300())),
        "net-q10" => Some(Pilot::Scored(EvalWeights::net_eval_q10())),
        "net-q20" => Some(Pilot::Scored(EvalWeights::net_eval_q20())),
        "netb-q10" => Some(Pilot::Scored(EvalWeights::net_blend_q10())),
        "netb-q20" => Some(Pilot::Scored(EvalWeights::net_blend_q20())),
        "v2" => Some(Pilot::Scored(EvalWeights::v2())),
        "pretap" => Some(Pilot::Scored(EvalWeights::legacy_mana())),
        "combat" => Some(Pilot::Scored(EvalWeights::combat_aware())),
        "holdsick" => Some(Pilot::Scored(EvalWeights::hold_sick())),
        "holdinst" => Some(Pilot::Scored(EvalWeights::hold_instants())),
        "holdsick+combat" => Some(Pilot::Scored(EvalWeights::hold_sick_combat())),
        "atk" => Some(Pilot::Scored(EvalWeights::attack_search())),
        "atk-cheap" => Some(Pilot::Scored(EvalWeights::attack_search_cheap())),
        "atk-hold" => Some(Pilot::Scored(EvalWeights::attack_search_hold())),
        "atk-sim" => Some(Pilot::Scored(EvalWeights::attack_search_sim())),
        "atk-race" => Some(Pilot::Scored(EvalWeights::attack_search_race())),
        "atk-life" => Some(Pilot::Scored(EvalWeights::attack_search_life())),
        "blk" => Some(Pilot::Scored(EvalWeights::block_search())),
        "dflt-life" => Some(Pilot::Scored(EvalWeights::hold_sick_combat_life())),
        "lookahead" => Some(Pilot::Scored(EvalWeights::lookahead1())),
        "planner" => Some(Pilot::Scored(EvalWeights::planner())),
        "v2+combat" => Some(Pilot::Scored(EvalWeights::v2_combat())),
        "scaled" => Some(Pilot::Scored(EvalWeights::scaled_control())),
        "keywords" => Some(Pilot::Scored(EvalWeights::keywords_only())),
        "kw25" => Some(Pilot::Scored(EvalWeights::keywords_quarter())),
        "base" => Some(Pilot::Scored(EvalWeights::creature_base_only())),
        "base+kw" => Some(Pilot::Scored(EvalWeights::base_and_keywords())),
        "life" => Some(Pilot::Scored(EvalWeights::life_only())),
        "power" => Some(Pilot::Scored(EvalWeights::power_emphasis_only())),
        "mcts" => Some(Pilot::Mcts(MctsConfig::default())),
        "mcts-heur" => Some(Pilot::Mcts(MctsConfig {
            heuristic_rollouts: true,
            ..MctsConfig::default()
        })),
        "mcts-deep" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            ..MctsConfig::default()
        })),
        "uniform" => Some(Pilot::Uniform),
        _ => None,
    }
}

/// Profile names accepted by `--a` / `--b`, for the help text and errors.
const PROFILES: &str = "baseline, combat, holdsick, holdsick+combat, atk, atk-cheap, atk-hold, atk-sim, atk-race, atk-life, dflt-life, blk, lookahead, holdinst, mcts, mcts-heur, mcts-deep, planner, v2+combat, pretap, scaled, keywords, kw25, base, base+kw, life, power, v2, uniform, landseq, mull, gang, landseq2, mull2, race2, look1, look2, smarttap, det1, det3, net, net-blend, net-blend300, net-q10, net-q20, netb-q10, netb-q20 (net* need CRAB_NET=<weights.safetensors>)";

/// The committed throughput configuration. `--bench` pins every knob that
/// moves the numbers so two runs on different days measure the same work:
/// the hand-built archetypes (cube/sealed fields are seed-dependent in
/// deck *content*, not just shuffles), a fixed seed, paired play, and a
/// mirror of one profile against itself.
///
/// `gang` is `EvalWeights::default()` — the profile the bot actually
/// plays, and therefore the one self-play training pays for. Benching
/// `baseline` would measure a code path (no attack search, no combat
/// sims, no gang-block search) that no real run takes.
const BENCH_SEED: u64 = 20250808;
const BENCH_GAMES: usize = 80;
const BENCH_PROFILE: &str = "gang";

struct Args {
    a: Pilot,
    b: Pilot,
    a_name: String,
    b_name: String,
    games: usize,
    seed: u64,
    threads: usize,
    deck_set: String,
    paired: bool,
    bench: bool,
}

fn parse_args() -> Result<Args, String> {
    let bench = std::env::args().any(|a| a == "--bench");
    let mut a_name = if bench { BENCH_PROFILE.to_string() } else { "baseline".to_string() };
    let mut b_name = if bench { BENCH_PROFILE.to_string() } else { "v2".to_string() };
    let mut games = if bench { BENCH_GAMES } else { 200usize };
    let mut seed = if bench { BENCH_SEED } else { 0u64 };
    let mut threads = 0usize;
    let mut deck_set = "fixed".to_string();
    let mut paired = true;
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let need = |i: usize| -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{} needs a value", argv[i]))
        };
        // Value-taking options advance by two, bare flags by one.
        let mut step = 2;
        match argv[i].as_str() {
            "--a" => a_name = need(i)?,
            "--b" => b_name = need(i)?,
            "--games" => games = need(i)?.parse().map_err(|_| "--games must be a number")?,
            "--seed" => seed = need(i)?.parse().map_err(|_| "--seed must be a number")?,
            "--threads" => threads = need(i)?.parse().map_err(|_| "--threads must be a number")?,
            "--decks" => deck_set = need(i)?,
            "--paired" => {
                paired = true;
                step = 1;
            }
            "--unpaired" => {
                paired = false;
                step = 1;
            }
            "--bench" => step = 1,
            "-h" | "--help" => {
                println!(
                    "bot_ladder [--a PROFILE] [--b PROFILE] [--games N] [--seed N] [--threads N]\n\
                     \n\
                     PROFILE is one of: {PROFILES}\n\
                     --decks fixed (4 hand-built archetypes) | cube (8 random cube pairs)\n\
                     | sos (5 seeded college mirrors) | sealed (12 seeded sealed mirrors)\n\
                     | both (fixed+cube) | all (fixed+cube+sos)\n\
                     --games is per archetype, split evenly across seats.\n\
                     --paired (default) plays each shuffle twice with the seats swapped\n\
                     and reports the variance-reduced estimate; --unpaired is the old\n\
                     independent-shuffle behaviour, kept as the measurement control.\n\
                     --bench runs the committed throughput configuration ({BENCH_PROFILE}\n\
                     mirror, {BENCH_GAMES} games x fixed decks, seed {BENCH_SEED}) and reports\n\
                     games/sec, decisions/sec, turns/game, stalls and peak RSS. Compare\n\
                     against the baseline in PERF.md; release builds only."
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other}")),
        }
        i += step;
    }
    let a = parse_profile(&a_name)
        .ok_or_else(|| format!("unknown profile {a_name}; expected one of: {PROFILES}"))?;
    let b = parse_profile(&b_name)
        .ok_or_else(|| format!("unknown profile {b_name}; expected one of: {PROFILES}"))?;
    if a_name.starts_with("net") || b_name.starts_with("net") {
        // The net profile silently equals atk-sim when the slot is empty,
        // which would make a forgotten CRAB_NET measure the wrong thing —
        // so an explicit weights file is mandatory here.
        let path = std::env::var("CRAB_NET")
            .map_err(|_| "profile `net` needs CRAB_NET=<weights.safetensors>".to_string())?;
        crabomination::server::net_eval::load_slot(
            crabomination::server::net_eval::SLOT_BEST,
            std::path::Path::new(&path),
        )?;
        eprintln!("loaded value net from {path}");
    }
    Ok(Args {
        a,
        b,
        a_name,
        b_name,
        games,
        seed,
        threads,
        deck_set,
        paired,
        bench,
    })
}

/// One archetype's result.
struct Row {
    name: &'static str,
    wins_a: u32,
    wins_b: u32,
    undecided: u32,
    /// Per-pair scores under `--paired`; empty when unpaired.
    pairs: Vec<i8>,
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\ntry --help");
            std::process::exit(2);
        }
    };

    const CUBE_PAIRS: usize = 8;
    let field: Vec<Archetype> = match args.deck_set.as_str() {
        "fixed" => archetypes(),
        "cube" => cube_archetypes(args.seed, CUBE_PAIRS),
        "sos" => sos_archetypes(args.seed),
        "both" => {
            let mut f = archetypes();
            f.extend(cube_archetypes(args.seed, CUBE_PAIRS));
            f
        }
        "all" => {
            let mut f = archetypes();
            f.extend(cube_archetypes(args.seed, CUBE_PAIRS));
            f.extend(sos_archetypes(args.seed));
            f
        }
        "sealed" => sealed_archetypes(args.seed, 12),
        other => {
            eprintln!("unknown --decks {other}; expected fixed, cube, sos, sealed, both or all");
            std::process::exit(2);
        }
    };
    let threads = if args.threads > 0 {
        args.threads
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1)
    };

    println!(
        "ladder: {} (A) vs {} (B) — {} games x {} {} decks on {threads} threads, seed {}{}",
        args.a_name,
        args.b_name,
        args.games,
        field.len(),
        args.deck_set,
        args.seed,
        if args.paired { " (paired)" } else { " (unpaired)" },
    );

    // Split each archetype's games into chunks and hand them to a shared
    // job queue, so a slow archetype doesn't leave cores idle at the end.
    //
    // Under `--paired` a chunk is measured in *pairs*, each of which is two
    // games, so `--games` keeps meaning games per archetype and the two
    // modes cost the same. An odd `--games` loses the leftover game rather
    // than playing half a pair.
    const CHUNK: usize = 10;
    struct Job {
        arch: usize,
        /// Games when unpaired, pairs when paired.
        units: usize,
        seed: u64,
    }
    let per_arch = if args.paired { args.games / 2 } else { args.games };
    if args.paired && args.games % 2 == 1 {
        eprintln!("note: --games is odd; playing {} pairs ({} games) per archetype", per_arch, per_arch * 2);
    }
    let mut jobs = Vec::new();
    for (ai, _) in field.iter().enumerate() {
        let mut done = 0;
        while done < per_arch {
            let n = CHUNK.min(per_arch - done);
            jobs.push(Job {
                arch: ai,
                units: n,
                // Distinct stream per chunk; deterministic given --seed.
                seed: args
                    .seed
                    .wrapping_add((ai as u64).wrapping_mul(0x1_0000_0000))
                    .wrapping_add(done as u64),
            });
            done += n;
        }
    }

    // A worker that finds an empty queue exits immediately, so `--threads N`
    // silently runs min(N, jobs) workers. That reads as "scaling flattens at
    // 4 threads" in a measurement, when what actually happened is that the
    // run only ever had 4 chunks of work. Say so.
    if jobs.len() < threads {
        eprintln!(
            "note: only {} job chunk(s) for {threads} threads — {} worker(s) will idle. \
             Raise --games (chunks = decks x ceil(games/{}) ) before reading this as scaling.",
            jobs.len(),
            threads - jobs.len(),
            if args.paired { CHUNK * 2 } else { CHUNK },
        );
    }

    let next = AtomicUsize::new(0);
    let cost: Mutex<SimCost> = Mutex::new(SimCost::default());
    let rows: Mutex<Vec<Row>> = Mutex::new(
        field
            .iter()
            .map(|a| Row {
                name: a.name,
                wins_a: 0,
                wins_b: 0,
                undecided: 0,
                pairs: Vec::new(),
            })
            .collect(),
    );
    let started = std::time::Instant::now();

    std::thread::scope(|s| {
        for _ in 0..threads {
            // A worker plays whole games, and resolution recurses through
            // `Effect` trees with debug-build frames big enough that the
            // 2 MB spawn default runs out on deep boards — this crashed a
            // 16 000-game run outright. `recommend.rs`'s game workers do
            // the same for the same reason.
            let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024);
            builder
                .spawn_scoped(s, || {
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(job) = jobs.get(i) else { break };
                        let d = &field[job.arch].deck;
                        let (wins_a, wins_b, undecided, pairs, job_cost) = if args.paired {
                            let t = simulate_match_pairs_piloted(
                                d,
                                d,
                                job.units,
                                [args.a, args.b],
                                50_000,
                                job.seed,
                            );
                            (t.wins_a, t.wins_b, t.undecided, t.pairs, t.cost)
                        } else {
                            let t = simulate_match_games_piloted(
                                d,
                                d,
                                job.units,
                                [args.a, args.b],
                                50_000,
                                Some(job.seed),
                            );
                            (t.wins_a, t.wins_b, t.undecided, Vec::new(), t.cost)
                        };
                        *cost.lock().unwrap() += job_cost;
                        let mut rows = rows.lock().unwrap();
                        let row = &mut rows[job.arch];
                        row.wins_a += wins_a;
                        row.wins_b += wins_b;
                        row.undecided += undecided;
                        row.pairs.extend(pairs);
                    }
                })
                .expect("spawn ladder worker");
        }
    });

    let rows = rows.into_inner().unwrap();
    let cost = cost.into_inner().unwrap();
    const Z: f64 = 1.96;
    println!();
    println!(
        "{:<18} {:>6} {:>6} {:>6}  {:>8}  95% CI",
        "archetype", "A", "B", "n/d", "A win%"
    );
    println!("{}", "-".repeat(64));
    let (mut ta, mut tb, mut tu) = (0u32, 0u32, 0u32);
    for r in &rows {
        let decided = r.wins_a + r.wins_b;
        let (lo, hi) = wilson(r.wins_a, decided, Z);
        let pct = if decided == 0 {
            0.5
        } else {
            r.wins_a as f64 / decided as f64
        };
        println!(
            "{:<18} {:>6} {:>6} {:>6}  {:>7.1}%  [{:.1}%, {:.1}%]",
            r.name,
            r.wins_a,
            r.wins_b,
            r.undecided,
            100.0 * pct,
            100.0 * lo,
            100.0 * hi,
        );
        ta += r.wins_a;
        tb += r.wins_b;
        tu += r.undecided;
    }
    let decided = ta + tb;
    let (lo, hi) = wilson(ta, decided, Z);
    let pct = if decided == 0 {
        0.5
    } else {
        ta as f64 / decided as f64
    };
    println!("{}", "-".repeat(64));
    println!(
        "{:<18} {:>6} {:>6} {:>6}  {:>7.1}%  [{:.1}%, {:.1}%]",
        "TOTAL",
        ta,
        tb,
        tu,
        100.0 * pct,
        100.0 * lo,
        100.0 * hi,
    );
    println!();
    let wall = started.elapsed().as_secs_f64();
    println!("{decided} decided, {tu} undecided, in {wall:.1}s");

    if args.bench {
        // Wall-clock throughput of the simulator itself. Reported as
        // per-thread rates as well as aggregate: a change that only moves
        // the aggregate is a scaling change (contention, allocator), one
        // that moves the per-thread rate is a change to the game loop.
        let g = cost.games as f64;
        let stall_pct = 100.0 * cost.games.saturating_sub(decided as u64) as f64 / g.max(1.0);
        println!();
        println!("bench: {BENCH_PROFILE} mirror, {} decks, seed {}, {threads} threads, {} build",
            field.len(),
            args.seed,
            if cfg!(debug_assertions) { "DEBUG (numbers meaningless)" } else { "release" },
        );
        println!("  games          {}", cost.games);
        println!("  wall_s         {wall:.2}");
        println!("  games_per_s    {:.2}", g / wall.max(1e-9));
        println!("  games_per_s_th {:.3}", g / wall.max(1e-9) / threads as f64);
        println!("  decisions      {}", cost.decisions);
        println!("  decisions_per_s {:.0}", cost.decisions as f64 / wall.max(1e-9));
        println!("  turns_per_game {:.2}", cost.turns as f64 / g.max(1.0));
        println!("  decisions_per_game {:.1}", cost.decisions as f64 / g.max(1.0));
        println!("  stalls         {tu} ({stall_pct:.2}%)");
        match peak_rss_mib() {
            Some(m) => println!("  peak_rss_mib   {m:.1}"),
            None => println!("  peak_rss_mib   n/a"),
        }
        // Host fingerprint — see `host_calib_ms`. Printed last so it never
        // sits between two numbers that get diffed.
        println!("  host_cpu       {}", host_cpu_model());
        println!("  host_calib_ms  {:.0}", host_calib_ms());
        // A self-mirror on a shared seed plays each pair twice with only
        // the seat labels swapped, so every pair MUST split. A sweep means
        // the two runs of one game diverged — a determinism bug, and one
        // this harness gets to catch for free on every bench run.
        if args.paired && args.a_name == args.b_name {
            let sweeps = rows
                .iter()
                .flat_map(|r| r.pairs.iter())
                .filter(|&&s| s != 0)
                .count();
            if sweeps == 0 {
                println!("  determinism    ok (all pairs split)");
            } else {
                println!("  determinism    FAIL — {sweeps} of the mirrored pairs did not split");
                std::process::exit(1);
            }
        }
    }

    // The paired estimate, when we played pairs. Same estimand as the
    // unpaired win rate above — printed next to it, not instead of it, so
    // a run that disagrees with the control is visible rather than hidden
    // behind the tighter number.
    let all_pairs: Vec<i8> = rows.iter().flat_map(|r| r.pairs.iter().copied()).collect();
    let paired = if args.paired { paired_stat(&all_pairs) } else { None };
    if let Some(p) = &paired {
        let sweeps_a = all_pairs.iter().filter(|&&s| s > 0).count();
        let sweeps_b = all_pairs.iter().filter(|&&s| s < 0).count();
        let splits = p.n - sweeps_a - sweeps_b;
        println!();
        println!(
            "paired: {} pairs — {sweeps_a} A-sweeps, {sweeps_b} B-sweeps, {splits} splits",
            p.n,
        );
        println!(
            "        A win% {:>5.1}%  [{:.1}%, {:.1}%]  (±{:.2} pts)",
            100.0 * p.p,
            100.0 * (p.p - Z * p.se),
            100.0 * (p.p + Z * p.se),
            100.0 * Z * p.se,
        );
        // What the pairing actually bought, measured rather than assumed:
        // Var(S) = 2p(1−p)(1+ρ), so the same precision from independent
        // games would have taken 1/(1+ρ) times as many.
        let factor = 1.0 + p.rho;
        if factor > 0.0 {
            println!(
                "        within-pair rho {:+.3} — variance x{:.2} vs independent games, \
                 i.e. these {} games carry the precision of {:.0}",
                p.rho,
                factor,
                2 * p.n,
                (2 * p.n) as f64 / factor,
            );
        } else {
            println!("        within-pair rho {:+.3}", p.rho);
        }
    }

    // State the conclusion the interval actually supports, rather than
    // leaving a bare number to be read as whatever the reader hoped for.
    //
    // A bare "the interval clears 50%" is too eager: a result that barely
    // clears on one seed routinely lands astride 50% on the next, because
    // an interval that just excludes the null is by construction the kind
    // that fails to replicate about as often as not. So an edge under
    // MARGINAL is called out as needing more games rather than reported as
    // a finding — the failure mode this harness exists to prevent is
    // shipping a weight change on one lucky run.
    //
    // The verdict reads the *paired* interval when there is one: it
    // estimates the same quantity with less noise, so deferring to the
    // wider unpaired interval would throw away the precision the pairing
    // was played to get.
    const MARGINAL: f64 = 0.01;
    let (pct, lo, hi) = match &paired {
        Some(p) => (p.p, p.p - Z * p.se, p.p + Z * p.se),
        None => (pct, lo, hi),
    };
    let edge = (pct - 0.5).abs();
    let (leader, leader_name) = if pct > 0.5 {
        ("A", &args.a_name)
    } else {
        ("B", &args.b_name)
    };
    let verdict = if (lo > 0.5 || hi < 0.5) && edge < MARGINAL {
        format!(
            "marginal — {leader} ({leader_name}) leads by {:.1} points, but an edge this \
             small needs several times {decided} games before it can be trusted; \
             re-run with a different --seed to see whether it holds",
            100.0 * edge,
        )
    } else if lo > 0.5 {
        format!("A ({}) is stronger — the interval clears 50%", args.a_name)
    } else if hi < 0.5 {
        format!(
            "B ({}) is stronger — the interval is entirely below 50%",
            args.b_name
        )
    } else {
        format!(
            "inconclusive at {} games/archetype — the interval straddles 50%",
            args.games
        )
    };
    println!("verdict: {verdict}");
}

// The paired-statistics tests that used to live here moved to
// `crabomination::recommend` with `paired_stat` and `wilson` themselves —
// `deck_duel` needs the same estimator, and one copy with its tests beats
// two that drift.
