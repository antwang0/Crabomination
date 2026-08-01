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

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crabomination::cube::{CardFactory, color_pair_name, cube_deck, random_color_pair};
use rand::SeedableRng;
use rand::rngs::StdRng;
use crabomination::recommend::{Pilot, simulate_match_games_piloted};
use crabomination::server::{EvalWeights, MctsConfig};

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

fn parse_profile(name: &str) -> Option<Pilot> {
    match name {
        "baseline" => Some(Pilot::Scored(EvalWeights::baseline())),
        "v2" => Some(Pilot::Scored(EvalWeights::v2())),
        "pretap" => Some(Pilot::Scored(EvalWeights::legacy_mana())),
        "combat" => Some(Pilot::Scored(EvalWeights::combat_aware())),
        "holdsick" => Some(Pilot::Scored(EvalWeights::hold_sick())),
        "holdinst" => Some(Pilot::Scored(EvalWeights::hold_instants())),
        "holdsick+combat" => Some(Pilot::Scored(EvalWeights::hold_sick_combat())),
        "atk" => Some(Pilot::Scored(EvalWeights::attack_search())),
        "atk-cheap" => Some(Pilot::Scored(EvalWeights::attack_search_cheap())),
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
const PROFILES: &str = "baseline, combat, holdsick, holdsick+combat, atk, atk-cheap, atk-life, dflt-life, blk, lookahead, holdinst, mcts, mcts-heur, mcts-deep, planner, v2+combat, pretap, scaled, keywords, kw25, base, base+kw, life, power, v2, uniform";

/// Wilson score interval for `wins` out of `n` at `z`. Chosen over the
/// normal approximation because it stays sane at small n and at p̂ = 0 or 1,
/// where the naive interval has zero width and would report an undefeated
/// profile as certainly better.
fn wilson(wins: u32, n: u32, z: f64) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let (n, p) = (n as f64, wins as f64 / n as f64);
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    ((center - half).max(0.0), (center + half).min(1.0))
}

struct Args {
    a: Pilot,
    b: Pilot,
    a_name: String,
    b_name: String,
    games: usize,
    seed: u64,
    threads: usize,
    deck_set: String,
}

fn parse_args() -> Result<Args, String> {
    let mut a_name = "baseline".to_string();
    let mut b_name = "v2".to_string();
    let mut games = 200usize;
    let mut seed = 0u64;
    let mut threads = 0usize;
    let mut deck_set = "fixed".to_string();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let need = |i: usize| -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{} needs a value", argv[i]))
        };
        match argv[i].as_str() {
            "--a" => a_name = need(i)?,
            "--b" => b_name = need(i)?,
            "--games" => games = need(i)?.parse().map_err(|_| "--games must be a number")?,
            "--seed" => seed = need(i)?.parse().map_err(|_| "--seed must be a number")?,
            "--threads" => threads = need(i)?.parse().map_err(|_| "--threads must be a number")?,
            "--decks" => deck_set = need(i)?,
            "-h" | "--help" => {
                println!(
                    "bot_ladder [--a PROFILE] [--b PROFILE] [--games N] [--seed N] [--threads N]\n\
                     \n\
                     PROFILE is one of: {PROFILES}\n\
                     --decks fixed (4 hand-built archetypes) | cube (8 random cube pairs) | both\n\
                     --games is per archetype, split evenly across seats."
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    let a = parse_profile(&a_name)
        .ok_or_else(|| format!("unknown profile {a_name}; expected one of: {PROFILES}"))?;
    let b = parse_profile(&b_name)
        .ok_or_else(|| format!("unknown profile {b_name}; expected one of: {PROFILES}"))?;
    Ok(Args {
        a,
        b,
        a_name,
        b_name,
        games,
        seed,
        threads,
        deck_set,
    })
}

/// One archetype's result.
struct Row {
    name: &'static str,
    wins_a: u32,
    wins_b: u32,
    undecided: u32,
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
        "both" => {
            let mut f = archetypes();
            f.extend(cube_archetypes(args.seed, CUBE_PAIRS));
            f
        }
        other => {
            eprintln!("unknown --decks {other}; expected fixed, cube or both");
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
        "ladder: {} (A) vs {} (B) — {} games x {} {} decks on {threads} threads, seed {}",
        args.a_name,
        args.b_name,
        args.games,
        field.len(),
        args.deck_set,
        args.seed,
    );

    // Split each archetype's games into chunks and hand them to a shared
    // job queue, so a slow archetype doesn't leave cores idle at the end.
    const CHUNK: usize = 10;
    struct Job {
        arch: usize,
        games: usize,
        seed: u64,
    }
    let mut jobs = Vec::new();
    for (ai, _) in field.iter().enumerate() {
        let mut done = 0;
        while done < args.games {
            let n = CHUNK.min(args.games - done);
            jobs.push(Job {
                arch: ai,
                games: n,
                // Distinct stream per chunk; deterministic given --seed.
                seed: args
                    .seed
                    .wrapping_add((ai as u64).wrapping_mul(0x1_0000_0000))
                    .wrapping_add(done as u64),
            });
            done += n;
        }
    }

    let next = AtomicUsize::new(0);
    let rows: Mutex<Vec<Row>> = Mutex::new(
        field
            .iter()
            .map(|a| Row {
                name: a.name,
                wins_a: 0,
                wins_b: 0,
                undecided: 0,
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
                        let tally = simulate_match_games_piloted(
                            d,
                            d,
                            job.games,
                            [args.a, args.b],
                            50_000,
                            Some(job.seed),
                        );
                        let mut rows = rows.lock().unwrap();
                        let row = &mut rows[job.arch];
                        row.wins_a += tally.wins_a;
                        row.wins_b += tally.wins_b;
                        row.undecided += tally.undecided;
                    }
                })
                .expect("spawn ladder worker");
        }
    });

    let rows = rows.into_inner().unwrap();
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
    println!(
        "{decided} decided, {tu} undecided, in {:.1}s",
        started.elapsed().as_secs_f64()
    );

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
    const MARGINAL: f64 = 0.01;
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
