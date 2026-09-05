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
use crabomination::crossplay::{CrossGame, CrossLink, Msg};
use crabomination::recommend::{
    Pilot, SimCost, paired_stat, simulate_match_games_cross, simulate_match_pairs_cross, wilson,
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
        // The stack 2-for-1: removal in response to the opponent's own
        // buff spell. Gate as A against `gang` (the same weights minus
        // the flag).
        "buff2for1" => Some(Pilot::Scored(EvalWeights::buff_2for1_on())),
        // Converge-aware land drops (from the first recorded human game:
        // the bot played its third Plains where the human diversified
        // for converge). Gate as A against `gang`.
        "convlands" => Some(Pilot::Scored(EvalWeights::converge_lands_on())),
        // Desperation chump blocks (from the first recorded human game:
        // the bot at 5 life took 4 to the face holding a blocker,
        // because the profitable-blocks-only menu never offered the
        // chump). Gate as A against `gang`.
        "chumpblocks" => Some(Pilot::Scored(EvalWeights::chump_blocks_on())),
        // Value-ordered combat damage (CR 510.1c): gang blocks resolved
        // in declaration order for the program's whole history — the
        // damage-order decision had no policy arm. Gate as A against
        // `gang`.
        "dmgorder" => Some(Pilot::Scored(EvalWeights::damage_order_on())),
        // The attack chain (round 55): grow the declaration one creature
        // at a time from "nobody", each step priced by the attack sim,
        // and offer the finished set beside the holdback menu. Gate as A
        // against `gang`.
        "atk-chain" => Some(Pilot::Scored(EvalWeights::attack_chain_on())),
        // The adopted default itself (`EvalWeights::default()`) — moves with
        // every adoption, so a gate never uses it as its control — and the
        // frozen round-55 default, the base the round-56 gates read on.
        "dflt" => Some(Pilot::Scored(EvalWeights::default())),
        "dflt55" => Some(Pilot::Scored(EvalWeights::round55_default())),
        // Round 56: the wide attack chain (runs from an empty greedy,
        // pairs at the first step) and the block chain (pair-or-gang
        // moves, priced by the block sim). Gate each as A against `dflt55`.
        "atk-chain-wide" => Some(Pilot::Scored(EvalWeights::attack_chain_wide_on())),
        "blk-chain" => Some(Pilot::Scored(EvalWeights::block_chain_on())),
        // The open-board shortcut: no opposing creature / planeswalker /
        // battle, so the attack search takes greedy without a sim. A
        // throughput device; gate as A against `gang` for *no loss*.
        "atk-open" => Some(Pilot::Scored(EvalWeights::attack_skip_open_on())),
        // Outcome-judged mid-resolution targets (round 53): the
        // suspending ChooseTarget path was a polarity guess that buffs
        // the opponent's best creature whenever a beneficial trigger's
        // legal set spans both sides. Gate as A against `gang`.
        "targeteval" => Some(Pilot::Scored(EvalWeights::target_eval_on())),
        // Walker chip attacks (a recorded loss: a walker ultimated after
        // ten unpressured turns). Gate as A against `atk-sim`.
        "walkerchip" => Some(Pilot::Scored(EvalWeights::walker_chip_on())),
        // Activated-ability candidates (the recorded games' unused
        // Sundering Archaic exile). Gate as A against `gang`.
        "abilarms" => Some(Pilot::Scored(EvalWeights::ability_arms_on())),
        // Round 46: impulse-draw activations (Ark of Hunger's mill-and-play).
        // A whole ability class the generators never enumerated; gate as A
        // against the `gang` control.
        "impulse" => Some(Pilot::Scored(EvalWeights::impulse_draw_on())),
        // Round 49: simulation-based mulligan. Mulligan is 25 % of all
        // decisions (bot_probe) and the only high-volume one still
        // answered by a predicate rather than by playing it out. The
        // predicate refinement (`mull`) is a well-powered null; this is a
        // different mechanism. Gate as A against `gang`.
        "mullsim" => Some(Pilot::Scored(EvalWeights::mull_sim_on())),
        // Round 50 control: the pre-fix planeswalker cash-out read.
        "walkerlegacy" => Some(Pilot::Scored(EvalWeights::legacy_cashout_on())),
        // Round 51 control: the pre-fix library-search ranking.
        "legacyfetch" => Some(Pilot::Scored(EvalWeights::legacy_fetch_on())),
        "det1" => Some(Pilot::Scored(EvalWeights::determinized())),
        "det3" => Some(Pilot::Scored(EvalWeights::determinized3())),
        "net" => Some(Pilot::Scored(EvalWeights::net_eval())),
        "net-det1" => Some(Pilot::Scored(EvalWeights::net_eval_det1())),
        // The saturation fallback (replay diagnostic, 2026-08-31): the
        // scored combat pickers silence a net reading outside [0.05,
        // 0.95] for that decision and rank on material instead. Gate as
        // A against `net-det1`.
        "net-guard" => Some(Pilot::Scored(EvalWeights::net_tail_guard_on())),
        // The attack chain under the net pilot. Gate as A against
        // `net-det1`.
        "net-chain" => Some(Pilot::Scored(EvalWeights::net_attack_chain_on())),
        // Round 56 under the net pilot. Gate each as A against `net-chain`.
        "net-chain-wide" => Some(Pilot::Scored(EvalWeights::net_attack_chain_wide_on())),
        "net-bchain" => Some(Pilot::Scored(EvalWeights::net_block_chain_on())),
        // The pre-2026-08-22 net shape: branched off `atk-sim`, so
        // without either adopted blocking layer. The control for the
        // rebase, and the shape every net gate from round 26 to 46 ran
        // in.
        "net-preblocks" => Some(Pilot::Scored(EvalWeights::net_eval_preblocks())),
        "net-det3" => Some(Pilot::Scored(EvalWeights::net_eval_det3())),
        "net-blend" => Some(Pilot::Scored(EvalWeights::net_eval_blend())),
        "net-blend300" => Some(Pilot::Scored(EvalWeights::net_eval_blend300())),
        "netb-ply" => Some(Pilot::Scored(EvalWeights::net_eval_blend_ply())),
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
        // Net-evaluated MCTS: the champion's win probability is the UCB1
        // reward (native [0,1], no squash), rollouts redeal hidden zones
        // (determinize 1 in the weights). The historical "MCTS loses"
        // verdict was earned with the heuristic reward; these are its
        // rematch profiles.
        "mcts-net" => Some(Pilot::Mcts(MctsConfig {
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        "mcts-net-deep" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // The scaling curve above mcts-net-deep: iterations at fixed
        // horizon, and horizon at fixed iterations.
        "mcts-net-128" => Some(Pilot::Mcts(MctsConfig {
            iterations: 128,
            horizon_turns: 3,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        "mcts-net-256" => Some(Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Round-43 candidate: r42's 256-iteration strength at a
        // client-viable average latency. Early stop ALONE — no
        // close-call extension — is the untested half of round 29's
        // adaptive arm (that arm bundled a 4x extension which spent
        // ~2x and returned exactly its spend). The stop only quits
        // once the leader's confidence bound clears every rival, so
        // strength should hold at 256 while forced moves stop paying
        // the full budget. Gates: parity vs mcts-net-256, margin vs
        // mcts-net-deep, serial cost per game.
        "mcts-net-256es" => Some(Pilot::Mcts(MctsConfig {
            iterations: 256,
            horizon_turns: 3,
            early_stop: true,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Reserve a root arm for prepared casts (two banked Ancestral
        // Recalls sat unfired through a recorded loss; the six-arm cap
        // can crowd the class out). Gate as A against mcts-net-deep.
        "mcts-net-prep" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights { prepare_arm: true, ..EvalWeights::net_eval_det1() },
            ..MctsConfig::default()
        })),
        "mcts-net-h4" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 4,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Round 44: the horizon in the *other* direction — the
        // value-equivalence question. The rollout sim is 88 % of search
        // wall (PERF.md forty-first pass), so if the net can stand in
        // for most of the rollout, iterations get several times cheaper
        // and the r42 iterations curve becomes affordable to climb.
        // h0 evaluates the root candidate's immediate successor: the
        // spell is often still on the stack and determinization cannot
        // reach an observable-info encoder, so this arm approximates
        // the 1-ply `net` pilot and is the sweep's sanity anchor, not
        // its hypothesis. h1 resolves the stack and one exchange first
        // — that is the cell the round exists to measure, alone (A)
        // and at rollout-cost-matched iterations (B, `-h1-192`).
        // Round 46: alternative targetings on the search menu. The
        // candidate generators bake in one auto-targeted assignment per
        // spell, so a mis-aimed cast is the only arm of its kind and the
        // search cannot prefer the right target at any valuation — the
        // structural shape that made chump blocks worth +0.9 in r43.
        // Gate as A against mcts-net-deep, same net both sides.
        // The round-46 control: the pre-adoption baseline, kept so the
        // gate that adopted target arms stays reproducible.
        // The search-side control for the blocking rebase.
        "mcts-net-preblocks" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights {
                determinize: 1,
                target_arms: true,
                ..EvalWeights::net_eval_preblocks()
            },
            ..MctsConfig::default()
        })),
        "mcts-net-noarms" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights::target_arms_off(),
            ..MctsConfig::default()
        })),
        "mcts-net-targetarms" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights::target_arms_on(),
            ..MctsConfig::default()
        })),
        // Round 51: the library-search decision as searched arms rather
        // than a fixed heuristic answer. Gate as A against mcts-net-deep,
        // which is `net_eval_det1` — the same weights with the flag off.
        "mcts-net-fetcharms" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: EvalWeights::fetch_arms_on(),
            ..MctsConfig::default()
        })),
        "mcts-net-h0" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 0,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        "mcts-net-h1" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 1,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        "mcts-net-h1-192" => Some(Pilot::Mcts(MctsConfig {
            iterations: 192,
            horizon_turns: 1,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Round 42: above the round-27 curve. That curve stopped at 256
        // and was still climbing (24→64→128→256 = 49.4→53.0→54.35→55.0 %
        // vs the champion, ~+1.4 then ~+0.7 per doubling), and round 29
        // found raw iterations to be the *only* MCTS lever that pays —
        // so where it flattens is the one number that decides how much
        // strength is left in the search.
        "mcts-net-512" => Some(Pilot::Mcts(MctsConfig {
            iterations: 512,
            horizon_turns: 3,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        "mcts-net-1024" => Some(Pilot::Mcts(MctsConfig {
            iterations: 1024,
            horizon_turns: 3,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Round 29: the search internals, one knob at a time against the
        // mcts-net-deep control (64/h3, c=1.0, no priors, fixed budget).
        // (a) The exploration constant was never tuned for rewards that
        // live in [0,1] win-probability space.
        "mcts-net-c05" | "mcts-net-c14" | "mcts-net-c20" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            exploration: match name {
                "mcts-net-c05" => 0.5,
                "mcts-net-c14" => 1.4,
                _ => 2.0,
            },
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // (b) Root priors from the candidate generator's scores (P-UCT).
        "mcts-net-prior" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            prior_weight: 1.5,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // (c) Adaptive budget: early-stop decided roots, extend close
        // calls to 4x. Mean cost is measured, not assumed — the games/s
        // line in the ladder output is part of this profile's result.
        "mcts-net-adapt" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            early_stop: true,
            extend_close: 4.0,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Round 37: the Gumbel root search at the control's exact budget
        // and shape — Sequential Halving over policy-head priors (or the
        // candidate scores, on a net without the head) instead of UCB1.
        // Gate against mcts-net-deep; both sides share the weights, so
        // the cell isolates the allocator + prior.
        "mcts-net-gumbel" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            gumbel: true,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        // Round 39: belief-weighted determinization — the mcts-net-deep
        // shape with rollout redeals drawn from the net's opponent-hand
        // belief head instead of uniformly. Gate against mcts-net-deep
        // with the same weights; on a net without the head the flag is
        // inert and the cell measures nothing (the startup line says
        // which is running).
        "mcts-net-bdeep" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            weights: {
                let mut w = EvalWeights::net_eval_det1();
                w.belief_determinize = true;
                w
            },
            ..MctsConfig::default()
        })),
        // The scored pilot under the same flag: sims and planner
        // dry-runs redeal from the belief instead of uniformly.
        "net-bdet1" => Some(Pilot::Scored({
            let mut w = EvalWeights::net_eval_det1();
            w.belief_determinize = true;
            w
        })),
        // Round 31: search the combat declarations too — the round-26/27
        // wins came from main-phase search alone, with attacks and blocks
        // still the heuristic's. Same budget as the control so the gate
        // measures coverage, not compute.
        "mcts-net-combat" => Some(Pilot::Mcts(MctsConfig {
            iterations: 64,
            horizon_turns: 3,
            search_combat: true,
            weights: EvalWeights::net_eval_det1(),
            ..MctsConfig::default()
        })),
        "uniform" => Some(Pilot::Uniform),
        _ => None,
    }
}

/// Profile names accepted by `--a` / `--b`, for the help text and errors.
const PROFILES: &str = "baseline, combat, holdsick, holdsick+combat, atk, atk-cheap, atk-hold, atk-sim, atk-open, atk-race, atk-life, dflt-life, blk, lookahead, holdinst, mcts, mcts-heur, mcts-deep, planner, v2+combat, pretap, scaled, keywords, kw25, base, base+kw, life, power, v2, uniform, landseq, mull, gang, landseq2, mull2, race2, look1, look2, smarttap, dmgorder, atk-chain, dflt, dflt55, atk-chain-wide, blk-chain, targeteval, det1, det3, net, net-det1, net-det3, net-blend, net-blend300, net-q10, net-q20, netb-q10, netb-q20, netb-ply, net-guard, net-chain, net-chain-wide, net-bchain, mcts-net, mcts-net-deep, mcts-net-128, mcts-net-256, mcts-net-h4, mcts-net-c05, mcts-net-c14, mcts-net-c20, mcts-net-prior, mcts-net-adapt, mcts-net-combat, mcts-net-gumbel, mcts-net-bdeep, mcts-net-fetcharms, legacyfetch, net-bdet1 (*net* need CRAB_NET=<weights.safetensors> or the committed nets/champion.safetensors)";

/// Peak resident set size in MiB, or `None` where the OS doesn't expose it
/// cheaply. Linux keeps the high-water mark in `/proc/self/status`, which
/// is what makes it worth reporting at all: sampling RSS at exit would
/// miss the spike that matters.
fn peak_rss_mib() -> Option<f64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = s.lines().find(|l| l.starts_with("VmHWM:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}

/// The build profile this binary was compiled under, read off its own path.
///
/// The label here used to be `if cfg!(debug_assertions) { "DEBUG" } else {
/// "release" }` — and **that was a claim the value did not support**.
/// `release-fast`, `profiling-fast` and `overflow` all have
/// `debug_assertions` off and all printed "release build", while PERF.md's
/// own rule is that a `release-fast` number never compares to a `release`
/// one. A throughput reading filed under the wrong profile is exactly the
/// mistake this harness exists to prevent, and it is not hypothetical: the
/// forty-fifth pass quoted a `profiling-fast` games/s next to a `release`
/// baseline before catching it.
///
/// Cargo does not hand the profile name to the crate, but it does put the
/// binary in `target/<profile>/`, so `current_exe`'s parent directory is the
/// answer and needs no build script. It also names `overflow` for what it is,
/// which `cfg!(overflow_checks)` cannot — that cfg is still unstable.
fn build_profile() -> String {
    if cfg!(debug_assertions) {
        return "DEBUG (numbers meaningless)".to_string();
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent()?.file_name()?.to_str().map(str::to_string))
        .unwrap_or_else(|| "optimized (profile unknown)".to_string())
}

/// This binary's own size in bytes, printed beside the profile name.
///
/// [`build_profile`] catches a `release-fast` number filed as a `release`
/// one; it cannot catch the *other* build-side variable this branch found,
/// because LTO, PGO and `target-cpu` are `RUSTFLAGS` and leave no `cfg` and
/// no path difference. A PGO binary and a plain one both live in
/// `target/release-fast/` and both print `release-fast build`, while PERF's
/// rule is that a PGO reading only compares to another PGO reading — the same
/// class of mistake, one level up.
///
/// NEXT item 0 already names the diagnostic: **"the binary size is the
/// check"**, because a profile that was raised under a different profile
/// applies partially and silently, and the size is what moves. That check was
/// a thing someone had to remember to run; this prints it on every bench, so
/// two readings that differ by a build rather than by a commit say so on
/// their own line.
fn build_size_bytes() -> Option<u64> {
    std::fs::metadata(std::env::current_exe().ok()?).ok().map(|m| m.len())
}

/// Host speed probe: a fixed, deterministic mixed ALU + random-access
/// workload, timed on one thread.
///
/// Absolute games/sec is only comparable between runs on the *same class of
/// host*. Identical engine code read 12.39 games/s on one routine box and
/// 9.64 on another — a 22 % gap that looks exactly like a regression and
/// isn't. The committed baseline records this probe next to the throughput
/// numbers so a moved absolute has *something* to be checked against.
///
/// **It is not sufficient, and 2026-08-15 is the counter-example.** Two
/// containers reporting the same `host_cpu` and overlapping calib (47-57
/// against 53-66, i.e. this one reading slightly *slower*) differed by
/// **24 %** on `--bench`. This probe is single-threaded; `--bench` runs
/// three workers, so nothing here measures how the host schedules them.
/// Agreement is therefore weak evidence and disagreement is strong: a moved
/// calib means a moved host, an unmoved one means nothing. **Re-measure
/// both sides in one sitting, or use callgrind** — that is the only sound
/// attribution, not a scaling correction against this number.
fn host_calib_ms() -> f64 {
    // 4 MiB of u64 — past L2 on the boxes this runs on, so the loop pays
    // real memory latency the way the engine's pointer-chasing does, not
    // just ALU throughput.
    const N: usize = 1 << 19;
    let mut buf: Vec<u64> = (0..N as u64).collect();
    let t = std::time::Instant::now();
    let mut x: u64 = 0x2545_F491_4F6C_DD1D;
    let mut acc: u64 = 0;
    for _ in 0..20_000_000u32 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let i = (x as usize) & (N - 1);
        acc = acc.wrapping_add(buf[i]);
        buf[i] = acc;
    }
    std::hint::black_box((acc, buf));
    t.elapsed().as_secs_f64() * 1000.0
}

/// CPU model string from `/proc/cpuinfo`, for the same reason as
/// [`host_calib_ms`]. Coarse — cloud VMs often report a generic model — so
/// it's a hint, not the measurement.
fn host_cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

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
    /// `--vs PATH`: play this build's bot against the one in `PATH`.
    vs: Option<String>,
    /// `--peer`: this process is the far end of somebody's `--vs`.
    peer: bool,
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
    let mut vs: Option<String> = None;
    let mut peer = false;
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
            "--vs" => vs = Some(need(i)?),
            "--peer" => {
                peer = true;
                step = 1;
            }
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
                     against the baseline in PERF.md; release builds only.\n\
                     --vs PATH plays THIS build's bot (side A) against the bot in the\n\
                     bot_ladder at PATH (side B), one child process per thread, and\n\
                     reports the win rate. Both builds play the same seeded games; a\n\
                     state divergence aborts the run rather than being averaged in.\n\
                     Run it against a copy of itself first: the null is every pair\n\
                     split at 50.0%. --peer is the far end and is not run by hand."
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
    if a_name.contains("net") || b_name.contains("net") {
        // The net profile silently equals atk-sim when the slot is empty,
        // which would make a forgotten CRAB_NET measure the wrong thing —
        // so weights are mandatory here. CRAB_NET wins when set; the
        // committed champion (`nets/champion.safetensors`, the round-20
        // net: pooled replacement gates 51.8 % vs gang / 54.4 % vs
        // atk-sim) is the fallback so `--a net` works out of the box.
        const CHAMPION: &str = "nets/champion.safetensors";
        let path = std::env::var("CRAB_NET").unwrap_or_else(|_| {
            if std::path::Path::new(CHAMPION).exists() {
                CHAMPION.to_string()
            } else {
                String::new()
            }
        });
        if path.is_empty() {
            return Err(format!(
                "profile `net` needs CRAB_NET=<weights.safetensors> (or the committed {CHAMPION})"
            ));
        }
        crabomination::server::net_eval::load_slot(
            crabomination::server::net_eval::SLOT_BEST,
            std::path::Path::new(&path),
        )?;
        eprintln!("loaded value net from {path}");
        // A gumbel profile runs learned priors only if the loaded net
        // carries the policy head; on a headless net it falls back to
        // heuristic-score priors — a legitimate control arm, but a
        // different experiment, so which one is running is said here
        // rather than discovered from a null result.
        if a_name.contains("gumbel") || b_name.contains("gumbel") {
            let learned = crabomination::server::net_eval::slot_has_policy(
                crabomination::server::net_eval::SLOT_BEST,
            );
            eprintln!(
                "gumbel priors: {}",
                if learned {
                    "policy head"
                } else {
                    "heuristic candidate scores (net carries no policy head)"
                }
            );
        }
        // A belief profile without the belief head silently degrades to
        // the uniform redeal, and the cell would gate a no-op.
        if a_name.contains("bdet") || a_name.contains("bdeep")
            || b_name.contains("bdet") || b_name.contains("bdeep")
        {
            let has = crabomination::server::net_eval::slot_has_opp(
                crabomination::server::net_eval::SLOT_BEST,
            );
            eprintln!(
                "belief redeal: {}",
                if has {
                    "opponent-hand head"
                } else {
                    "INERT — net carries no head_opp; this cell measures nothing"
                }
            );
        }
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
        vs,
        peer,
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

/// Build the chunked job queue for `field` and run it on `threads` workers,
/// returning the summed cost and per-archetype rows. Factored out of `main`
/// so `CRAB_THREAD_CHECK` can replay the identical workload at a second thread
/// count and assert the aggregate is unchanged — one job-loop, one measured
/// invariant (see filter 11: a rule spelled out twice drifts). `quiet`
/// suppresses the idle-worker note on the replay pass.
///
/// `peers` is empty in the ordinary ladder and holds one peer process per
/// worker under `--vs`: worker `w` sends its chunk to `peers[w]` as a
/// [`Msg::Job`] and then plays it in lockstep with that process. The queue
/// itself is not shared with the peers — they are told which chunk to play,
/// so both sides walk the same schedule without a second job loop.
fn run_jobs(
    field: &[Archetype],
    args: &Args,
    threads: usize,
    quiet: bool,
    peers: &mut [Option<CrossGame>],
) -> (SimCost, Vec<Row>) {
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
    if args.paired && args.games % 2 == 1 && !quiet {
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
    if jobs.len() < threads && !quiet {
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

    // One worker faulting voids every other worker's games too, so they
    // stop at the next chunk boundary rather than finishing a run whose
    // aggregate no longer means anything.
    let aborted = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|s| {
        let mut slots = peers.iter_mut();
        for _ in 0..threads {
            let mut peer: Option<&mut CrossGame> = slots.next().and_then(|p| p.as_mut());
            // Only `peer` is owned per worker; the rest is shared, so the
            // `move` closure has to capture references to it.
            let (jobs, next, cost, rows, aborted) = (&jobs, &next, &cost, &rows, &aborted);
            // A worker plays whole games, and resolution recurses through
            // `Effect` trees with debug-build frames big enough that the
            // 2 MB spawn default runs out on deep boards — this crashed a
            // 16 000-game run outright. `recommend.rs`'s game workers do
            // the same for the same reason.
            let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024);
            builder
                .spawn_scoped(s, move || {
                    loop {
                        if aborted.load(Ordering::Relaxed) {
                            break;
                        }
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(job) = jobs.get(i) else { break };
                        // Tell this worker's peer which chunk to play before
                        // either side starts it; from here the two processes
                        // walk the same schedule off the same seed.
                        if let Some(cx) = peer.as_deref_mut() {
                            let m = Msg::Job {
                                arch: job.arch,
                                units: job.units,
                                seed: job.seed,
                            };
                            if let Err(e) = cx.link_mut().send(&m) {
                                cx.fault.get_or_insert(e);
                                aborted.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        let cx: Option<&mut CrossGame> = peer.as_deref_mut();
                        let d = &field[job.arch].deck;
                        let (wins_a, wins_b, undecided, pairs, job_cost) = if args.paired {
                            let t = simulate_match_pairs_cross(
                                d,
                                d,
                                job.units,
                                [args.a, args.b],
                                50_000,
                                job.seed,
                                cx,
                                true,
                            );
                            // `CRAB_PAIR_SWEEPS=1`: name every pair that did
                            // *not* split. In a self-mirror the two games of a
                            // pair are one game with the seats relabelled, so a
                            // sweep is a determinism failure, and the aggregate
                            // row only says one happened — not which. `pairs`
                            // holds `score.signum()`, so any non-zero entry is
                            // a sweep. The pair seed printed here is the one
                            // `simulate_match_pairs_piloted` derives, so it
                            // replays the exact game.
                            if std::env::var_os("CRAB_PAIR_SWEEPS").is_some() {
                                for (k, sc) in t.pairs.iter().enumerate() {
                                    if *sc != 0 {
                                        eprintln!(
                                            "sweep: arch {:?} job_seed {} pair {} \
                                             pair_seed {} score {}",
                                            field[job.arch].name,
                                            job.seed,
                                            k,
                                            job.seed.wrapping_add(
                                                (k as u64)
                                                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                                            ),
                                            sc,
                                        );
                                    }
                                }
                            }
                            (t.wins_a, t.wins_b, t.undecided, t.pairs, t.cost)
                        } else {
                            let t = simulate_match_games_cross(
                                d,
                                d,
                                job.units,
                                [args.a, args.b],
                                50_000,
                                Some(job.seed),
                                cx,
                                true,
                            );
                            (t.wins_a, t.wins_b, t.undecided, Vec::new(), t.cost)
                        };
                        // A faulted chunk is not a partial result: its tally
                        // stops mid-game, so it is dropped whole.
                        if peer.as_deref().is_some_and(|c| c.fault.is_some()) {
                            aborted.store(true, Ordering::Relaxed);
                            break;
                        }
                        *cost.lock().unwrap() += job_cost;
                        let mut rows = rows.lock().unwrap();
                        let row = &mut rows[job.arch];
                        row.wins_a += wins_a;
                        row.wins_b += wins_b;
                        row.undecided += undecided;
                        row.pairs.extend(pairs);
                    }
                    // Let the peer exit on a message rather than on EOF, so
                    // a clean end is distinguishable from a crash.
                    if let Some(cx) = peer
                        && cx.fault.is_none()
                    {
                        let _ = cx.link_mut().send(&Msg::Done);
                    }
                })
                .expect("spawn ladder worker");
        }
    });

    (cost.into_inner().unwrap(), rows.into_inner().unwrap())
}

/// FNV-1a over the field both builds must agree on before a single game is
/// worth playing: every archetype's name and the printed name of every card
/// in its deck, in order.
///
/// The peer is handed this build's argv, so it constructs its own field from
/// the same `--decks`/`--seed` rather than being sent one — which is only
/// sound while the two builds' deck lists and card definitions still line
/// up. A catalog rename, an archetype edit or a change to the cube/sealed
/// sampler moves this number, and the handshake refuses rather than
/// reporting a win rate over two different fields.
fn field_digest(field: &[Archetype]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for a in field {
        mix(a.name.as_bytes());
        mix(b"|");
        for &f in &a.deck {
            mix(crabomination::cube::card_arc(f).name.as_bytes());
            mix(b",");
        }
        mix(b";");
    }
    h
}

/// Start one peer process per worker and shake hands with each.
///
/// The child gets this process's argv with `--vs PATH` removed and `--peer`
/// appended, so it parses the same `--decks`, `--seed`, `--games`,
/// `--paired` and profiles and builds the same field from them. Nothing
/// about the schedule is sent; only the chunk assignments are.
fn spawn_peers(
    path: &str,
    threads: usize,
    field: &[Archetype],
) -> Result<(Vec<Option<CrossGame>>, Vec<std::process::Child>), String> {
    let mut argv: Vec<String> = Vec::new();
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == "--vs" {
            i += 2;
            continue;
        }
        argv.push(raw[i].clone());
        i += 1;
    }
    argv.push("--peer".to_string());
    let mine = field_digest(field);
    let (mut peers, mut children) = (Vec::new(), Vec::new());
    for w in 0..threads {
        let mut child = std::process::Command::new(path)
            .args(&argv)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn peer {w}: {e}"))?;
        let w_in = child.stdin.take().ok_or("peer stdin")?;
        let r_out = child.stdout.take().ok_or("peer stdout")?;
        let mut cx = CrossGame::new(CrossLink::new(Box::new(r_out), Box::new(w_in)));
        cx.link_mut()
            .send(&Msg::Hello { proto: crabomination::crossplay::PROTO, field: mine })
            .map_err(|e| format!("peer {w}: {e}"))?;
        match cx.link_mut().recv() {
            Ok(Msg::HelloOk { proto, field: theirs, build }) => {
                if proto != crabomination::crossplay::PROTO {
                    return Err(format!("peer speaks protocol {proto}, this build {}",
                        crabomination::crossplay::PROTO));
                }
                if theirs != mine {
                    return Err(format!(
                        "peer builds a different field ({theirs:#018x} vs {mine:#018x}) — \
                         the two builds' decks or card names differ, so there is no \
                         common workload to compare them on"
                    ));
                }
                if w == 0 {
                    println!("peer: {path} ({build} build)");
                }
            }
            Ok(other) => return Err(format!("peer {w}: expected HelloOk, got {other:?}")),
            Err(e) => return Err(format!("peer {w}: {e}")),
        }
        peers.push(Some(cx));
        children.push(child);
    }
    Ok((peers, children))
}

/// `--peer`: be the far end of somebody's `--vs`. Reads jobs off stdin and
/// plays each in lockstep with the parent, piloting side B.
///
/// Deliberately a mirror of the worker loop in [`run_jobs`] rather than a
/// second scheduler: the parent names the chunk, so this side never decides
/// what to play, only how to play it.
fn run_peer(field: &[Archetype], args: &Args) -> i32 {
    let mut cx = CrossGame::new(CrossLink::stdio());
    let mine = field_digest(field);
    match cx.link_mut().recv() {
        Ok(Msg::Hello { proto, field: theirs }) => {
            if proto != crabomination::crossplay::PROTO {
                eprintln!("peer: protocol {proto} vs {}", crabomination::crossplay::PROTO);
                return 3;
            }
            if theirs != mine {
                eprintln!("peer: field digest {mine:#018x} vs parent's {theirs:#018x}");
                return 3;
            }
        }
        Ok(other) => {
            eprintln!("peer: expected Hello, got {other:?}");
            return 3;
        }
        Err(e) => {
            eprintln!("peer: {e}");
            return 3;
        }
    }
    let ok = Msg::HelloOk { proto: crabomination::crossplay::PROTO, field: mine, build: build_profile().to_string() };
    if let Err(e) = cx.link_mut().send(&ok) {
        eprintln!("peer: {e}");
        return 3;
    }
    loop {
        let job = match cx.link_mut().recv() {
            Ok(Msg::Job { arch, units, seed }) => (arch, units, seed),
            Ok(Msg::Done) => return 0,
            Ok(other) => {
                eprintln!("peer: expected Job, got {other:?}");
                return 3;
            }
            Err(e) => {
                eprintln!("peer: {e}");
                return 3;
            }
        };
        let (arch, units, seed) = job;
        let Some(a) = field.get(arch) else {
            eprintln!("peer: parent named archetype {arch} of {}", field.len());
            return 3;
        };
        let d = &a.deck;
        if args.paired {
            simulate_match_pairs_cross(
                d, d, units, [args.a, args.b], 50_000, seed, Some(&mut cx), false,
            );
        } else {
            simulate_match_games_cross(
                d, d, units, [args.a, args.b], 50_000, Some(seed), Some(&mut cx), false,
            );
        }
        if let Some(f) = &cx.fault {
            eprintln!("peer: {f}");
            return 3;
        }
    }
}

/// Whether two runs of the same workload produced the same outcome, ignoring
/// order. Worker scheduling permutes which chunk finishes first (so a row's
/// pairs arrive in a different order, and even two runs at the same thread
/// count differ that way), so the pair lists are sorted before comparison;
/// the `SimCost` fields and per-archetype win tallies are already sums and
/// order-free.
fn outcomes_match(a: (&SimCost, &[Row]), b: (&SimCost, &[Row])) -> bool {
    let (ca, ra) = a;
    let (cb, rb) = b;
    let cost_eq = (ca.games, ca.decisions, ca.turns, ca.action_capped, ca.no_legal_move, ca.draws)
        == (cb.games, cb.decisions, cb.turns, cb.action_capped, cb.no_legal_move, cb.draws);
    if !cost_eq || ra.len() != rb.len() {
        return false;
    }
    ra.iter().zip(rb).all(|(x, y)| {
        if (x.wins_a, x.wins_b, x.undecided) != (y.wins_a, y.wins_b, y.undecided) {
            return false;
        }
        let (mut px, mut py) = (x.pairs.clone(), y.pairs.clone());
        px.sort_unstable();
        py.sort_unstable();
        px == py
    })
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\ntry --help");
            std::process::exit(2);
        }
    };

    // Encoder ablation, mirroring selfplay_train's --ablate. A net
    // trained with a block ablated has *never-trained* (random-init)
    // weight columns for those features — gating it under the full
    // encoder feeds live features into random weights, which measures
    // garbage, not the ablation. The gate must encode exactly as the
    // training run did.
    // An empty or whitespace-only value means "no ablation", not "one
    // block whose name is the empty string". Scripts that gate a mix of
    // ablated and full nets set this to "" for the full ones, and
    // exiting on that would fail exactly half a paired sweep.
    if let Ok(spec) = std::env::var("CRAB_ABLATE") {
        let off: Vec<&str> =
            spec.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        if let Err(e) = crabomination::server::encode::set_encode_ablation_off(&off) {
            eprintln!("CRAB_ABLATE: {e}");
            std::process::exit(2);
        }
        if !off.is_empty() {
            eprintln!("encoder ablation via CRAB_ABLATE: {} switched off", off.join(", "));
        }
    }

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
    if args.peer {
        std::process::exit(run_peer(&field, &args));
    }
    let threads = if args.threads > 0 {
        args.threads
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1)
    };

    // `--vs`: one peer process per worker, each handed this build's argv so
    // it constructs the identical field, queue-free. Spawned before the
    // banner so a handshake failure costs nothing.
    let mut peers: Vec<Option<CrossGame>> = Vec::new();
    let mut children: Vec<std::process::Child> = Vec::new();
    if let Some(path) = &args.vs {
        match spawn_peers(path, threads, &field) {
            Ok((p, c)) => {
                peers = p;
                children = c;
            }
            Err(e) => {
                eprintln!("error: --vs {path}: {e}");
                std::process::exit(3);
            }
        }
    }

    println!(
        "ladder: {} (A{}) vs {} (B{}) — {} games x {} {} decks on {threads} threads, seed {}{}",
        args.a_name,
        if args.vs.is_some() { ", this build" } else { "" },
        args.b_name,
        if args.vs.is_some() { ", peer build" } else { "" },
        args.games,
        field.len(),
        args.deck_set,
        args.seed,
        if args.paired { " (paired)" } else { " (unpaired)" },
    );

    // The chunked job queue and its worker pool live in `run_jobs` so the
    // opt-in thread-determinism guard below can replay the identical workload
    // without a second, drifting copy of the loop.
    let started = std::time::Instant::now();
    let (cost, rows) = run_jobs(&field, &args, threads, false, &mut peers);
    // A fault is not a result: the two builds stopped agreeing about what an
    // action does, so every game after it — on any worker — is void.
    if let Some(f) = peers.iter().flatten().find_map(|c| c.fault.as_ref()) {
        eprintln!("\ncross-ladder fault: {f}");
        eprintln!(
            "The two builds' ENGINES disagree, not just their bots. \
             `--vs` gates a change to how the bot chooses; a change to how an \
             action resolves shows up here and has to be gated on traces and \
             the suite instead."
        );
        drop(peers);
        for c in &mut children {
            let _ = c.kill();
            let _ = c.wait();
        }
        std::process::exit(3);
    }
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
    // Say *why* whenever there is a why to say. The tally is kept on every
    // run — `SimCost::record` splits the three the moment a game ends — but
    // until now only `--bench` printed it, so a robustness sweep on
    // `--decks` reported a bare undecided count and the next question ("is
    // that a draw or a stuck game?") cost a rebuild to answer. A draw is a
    // rules outcome and needs no fix; a capped game was doing work; a stuck
    // one had no bot able to move at all.
    if tu > 0 {
        println!(
            "  undecided_by   cap {} / stuck {} / draw {}",
            cost.action_capped, cost.no_legal_move, cost.draws,
        );
    }

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
            build_profile(),
        );
        println!("  games          {}", cost.games);
        println!("  wall_s         {wall:.2}");
        println!("  games_per_s    {:.2}", g / wall.max(1e-9));
        println!("  games_per_s_th {:.3}", g / wall.max(1e-9) / threads as f64);
        println!("  decisions      {}", cost.decisions);
        println!("  decisions_per_s {:.0}", cost.decisions as f64 / wall.max(1e-9));
        println!("  turns_per_game {:.2}", cost.turns as f64 / g.max(1.0));
        println!("  decisions_per_game {:.1}", cost.decisions as f64 / g.max(1.0));
        // Split by *why*: an action-capped game was still making moves and
        // ran out of budget, a stuck one had no bot able to move at all,
        // and a draw is a rules outcome, not a stall. They want different
        // fixes, so a moving stall rate names its own top cause.
        println!("  stalls         {tu} ({stall_pct:.2}%)");
        println!(
            "  stalls_by      cap {} / stuck {} / draw {}",
            cost.action_capped, cost.no_legal_move, cost.draws,
        );
        match peak_rss_mib() {
            Some(m) => println!("  peak_rss_mib   {m:.1}"),
            None => println!("  peak_rss_mib   n/a"),
        }
        // Build fingerprint — see `build_size_bytes`. Beside the host block
        // for the same reason: neither is a throughput number, and both are
        // what a moved absolute has to be checked against first.
        match build_size_bytes() {
            Some(b) => println!("  bin_bytes      {b}"),
            None => println!("  bin_bytes      n/a"),
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
    if let Some(t) = crabomination::server::mcts::timing::report() {
        println!("{t}");
    }

    // Thread-scheduling determinism guard (opt-in via `CRAB_THREAD_CHECK`;
    // deliberately off the throughput path so it never perturbs a bench
    // reading — it doubles the run). A fixed `--seed` fully determines every
    // job, and the aggregate is a commutative sum over jobs, so the tallies
    // must not depend on how many workers pull them. Replay the identical
    // workload at a contrasting thread count and assert the order-independent
    // fingerprint matches. This is the cheap in-process form of the wide
    // seed x thread sweep: any per-process RNG or global mutable state
    // leaking across threads (the filter-21 class — `restart_game` drawing
    // from OS entropy) diverges the two counts here.
    // PERF (-54)/(-55): what the simulation's own pickers proposed and the
    // engine threw out. Off unless `CRAB_SIM_REJECTS` is set.
    if crabomination::server::bot::sim_rejects::level() > 0 {
        let s = crabomination::server::bot::sim_rejects::snapshot();
        let (calls, errs): (u64, u64) =
            s.iter().fold((0, 0), |(a, b), (c, e)| (a + c, b + e));
        println!(
            "  sim_rejects {errs}/{calls} ({:.2} %) — attack {}/{}, block {}/{}, other {}/{}",
            if calls == 0 { 0.0 } else { 100.0 * errs as f64 / calls as f64 },
            s[0].1, s[0].0, s[1].1, s[1].0, s[2].1, s[2].0,
        );
    }
    // What the attack search (PERF (-21), ~60 % of `cube`) decides. Off
    // unless `CRAB_ATTACK_CENSUS` is set.
    if crabomination::server::bot::attack_census::on() {
        let [calls, cands, greedy, none, hold, tied, empty, empty_greedy, chain_new, chain_won, chain_sims, chain_reuse, chain_empty] =
            crabomination::server::bot::attack_census::snapshot();
        let pct = |n: u64, d: u64| if d == 0 { 0.0 } else { 100.0 * n as f64 / d as f64 };
        println!(
            "  attack_census {calls} searched, {cands} candidates ({:.2}/search); won: greedy \
             {greedy} ({:.1} %), none {none} ({:.1} %), holdback {hold} ({:.1} %); {tied} \
             tied the winner; defender creatureless {empty} ({:.1} %), greedy won there \
             {empty_greedy}; chain proposed a new set {chain_new} ({:.1} %), won {chain_won} \
             ({:.1} %); chain sims {chain_sims} ({:.2}/search), start reused {chain_reuse}, \
             from empty greedy {chain_empty}",
            if calls == 0 { 0.0 } else { cands as f64 / calls as f64 },
            pct(greedy, calls),
            pct(none, calls),
            pct(hold, calls),
            pct(empty, calls),
            pct(chain_new, calls),
            pct(chain_won, calls),
            if calls == 0 { 0.0 } else { chain_sims as f64 / calls as f64 },
        );
        let [bcalls, bcands, bsims, bnew, bwon, breuse] =
            crabomination::server::bot::block_census::snapshot();
        println!(
            "  block_census {bcalls} searched, {bcands} candidates ({:.2}/search); chain sims \
             {bsims} ({:.2}/search), proposed a new plan {bnew} ({:.1} %), won {bwon} ({:.1} %), \
             start reused {breuse}",
            if bcalls == 0 { 0.0 } else { bcands as f64 / bcalls as f64 },
            if bcalls == 0 { 0.0 } else { bsims as f64 / bcalls as f64 },
            pct(bnew, bcalls),
            pct(bwon, bcalls),
        );
    }
    // PERF (-88): how many state-based-action sweeps re-sweep a state the
    // previous sweep on the thread already saw. Off unless `CRAB_SBA_CENSUS`
    // is set.
    if crabomination::game::stack::sba_census::on() {
        let (sweeps, repeats, no_reach) = crabomination::game::stack::sba_census::snapshot();
        println!(
            "  sba_census {repeats}/{sweeps} repeats ({:.2} %), {no_reach} after no \
             &mut reach",
            if sweeps == 0 { 0.0 } else { 100.0 * repeats as f64 / sweeps as f64 },
        );
    }
    // PERF (-138): how many affordance probes ever write the resolution-scratch
    // half of `GameState` — the population a `CowBox<ResolutionScratch>` would
    // serve. Off unless `CRAB_SCRATCH_CENSUS` is set.
    if crabomination::game::affordances::scratch_census::on() {
        let (probes, same_all, same_coll) =
            crabomination::game::affordances::scratch_census::snapshot();
        let pct = |n: u64| if probes == 0 { 0.0 } else { 100.0 * n as f64 / probes as f64 };
        println!(
            "  scratch_census {same_all}/{probes} probes never wrote the group \
             ({:.2} %), {same_coll} never wrote its collections ({:.2} %)",
            pct(same_all),
            pct(same_coll),
        );
    }
    // PERF (-115): what the trigger dispatcher's member list is worth — hit
    // rate, and the board it skips against the members it keeps. Off unless
    // `CRAB_TRIG_CENSUS` is set.
    #[cfg(feature = "trig-census")]
    if crabomination::zone::trig_census::on() {
        let [asks, hits, board, members, granted, grant_board, visits] =
            crabomination::zone::trig_census::snapshot();
        let base = board + grant_board;
        println!(
            "  trig_census {hits}/{asks} hits ({:.2} %), {granted} grant-live \
             (board/dispatch {:.2}); board/ask {:.2}, members/hit {:.2}; \
             visits {visits}/{base} ({:.2} % of the walk left)",
            if asks == 0 { 0.0 } else { 100.0 * hits as f64 / asks as f64 },
            if granted == 0 { 0.0 } else { grant_board as f64 / granted as f64 },
            if asks == 0 { 0.0 } else { board as f64 / asks as f64 },
            if hits == 0 { 0.0 } else { members as f64 / hits as f64 },
            if base == 0 { 0.0 } else { 100.0 * visits as f64 / base as f64 },
        );
        // PERF (-120): which gate made each grant-live dispatch, and what the
        // event-kind filter the other three lack would have left.
        let [reason, reason_visits, filtered, filtered_visits] =
            crabomination::zone::trig_census::reason_snapshot();
        let names = crabomination::zone::trig_census::REASON_NAMES;
        for i in 0..16 {
            if reason[i] == 0 && filtered[i] == 0 {
                continue;
            }
            println!(
                "  trig_reason {:<18} {:>8} dispatches / {:>9} visits   filtered {:>8} / {:>9}",
                names[i], reason[i], reason_visits[i], filtered[i], filtered_visits[i],
            );
        }
        // PERF (-122): the activated-grant walk against the board-wide mask
        // that would replace it.
        let [scans, grant_scans, mask_evals, walk_evals] =
            crabomination::zone::grant_census::snapshot();
        println!(
            "  grant_census {grant_scans}/{scans} scans carry an EachPermanent grant; \
             mask would evaluate {mask_evals}, the walk evaluates {walk_evals} \
             ({:.2}x)",
            if walk_evals == 0 { 0.0 } else { mask_evals as f64 / walk_evals as f64 },
        );
        // PERF (-126): the requirement walker's recursion, by child shape.
        let [rcalls, comb, children, leaf_children, nested_children] =
            crabomination::zone::req_census::snapshot();
        println!(
            "  req_census {rcalls} calls, {comb} combinator arms making {children} \
             recursive calls: {leaf_children} to a leaf, {nested_children} to another \
             combinator ({:.1} % nested)",
            if children == 0 { 0.0 } else { 100.0 * nested_children as f64 / children as f64 },
        );
        // PERF (-129): which factor of (pairs x batch) the dispatcher's
        // innermost loop multiplies, and the share of it a pair no event in
        // the batch can match contributes.
        let [disp, batch, kinds, pairs, dead_pairs, calls, dead_calls] =
            crabomination::zone::ems_census::snapshot();
        println!(
            "  ems_census {disp} dispatches x {:.2} events x {:.2} pairs = {calls} calls; \
             dead {dead_pairs}/{pairs} pairs ({:.2} %) / {dead_calls} calls ({:.2} %); \
             distinct kinds/dispatch {:.2}",
            if disp == 0 { 0.0 } else { batch as f64 / disp as f64 },
            if disp == 0 { 0.0 } else { pairs as f64 / disp as f64 },
            if pairs == 0 { 0.0 } else { 100.0 * dead_pairs as f64 / pairs as f64 },
            if calls == 0 { 0.0 } else { 100.0 * dead_calls as f64 / calls as f64 },
            if disp == 0 { 0.0 } else { kinds as f64 / disp as f64 },
        );
    }
    // PERF (-51)(b): what the simulator's payments cost when they fail, split
    // by *why*. Off unless `CRAB_PAY_FAILS` is set.
    if crabomination::game::pay_census::level() > 0 {
        let (attempts, fails) = crabomination::game::pay_census::snapshot();
        let total: u64 = fails.iter().sum();
        let by: Vec<String> = crabomination::game::pay_census::CLASSES
            .iter()
            .zip(fails.iter())
            .filter(|(_, n)| **n > 0)
            .map(|(c, n)| format!("{c} {n}"))
            .collect();
        let t = crabomination::game::pay_census::tap_snapshot();
        println!(
            "  pay_taps {} auto-tap calls — {} returned early ({:.1} %), {} source tables built, {} sources tapped",
            t[0],
            t[1],
            if t[0] == 0 { 0.0 } else { 100.0 * t[1] as f64 / t[0] as f64 },
            t[2],
            t[3],
        );
        let ex = crabomination::game::pay_census::expensive_snapshot();
        println!(
            "  pay_fails_costly {} of {total} failures had built a source table ({:.1} %), {} tables",
            ex[0],
            if total == 0 { 0.0 } else { 100.0 * ex[0] as f64 / total as f64 },
            ex[1],
        );
        let (probe, committed) = crabomination::game::pay_census::origin_snapshot();
        let b = crabomination::game::pay_census::budget_snapshot();
        println!(
            "  pay_budget {} calls — widened {} ({:.1} %): relax {}, opaque {}, land-type {}",
            b[0],
            b[1] + b[2] + b[3],
            if b[0] == 0 { 0.0 } else { 100.0 * (b[1] + b[2] + b[3]) as f64 / b[0] as f64 },
            b[1], b[2], b[3],
        );
        println!(
            "  pay_fails {total}/{attempts} ({:.2} %) — {} | probe {probe}, committed {committed}",
            if attempts == 0 { 0.0 } else { 100.0 * total as f64 / attempts as f64 },
            if by.is_empty() { "none".to_string() } else { by.join(", ") },
        );
    }
    // The peers were spawned one per worker, so a replay at a different
    // thread count has no peer for the extra worker and would hang. The
    // guard is about this process's own scheduling anyway.
    if std::env::var_os("CRAB_THREAD_CHECK").is_some() && args.vs.is_none() {
        let alt = if threads <= 1 { 2 } else { 1 };
        let (alt_cost, alt_rows) = run_jobs(&field, &args, alt, true, &mut []);
        if outcomes_match((&cost, &rows), (&alt_cost, &alt_rows)) {
            println!("  thread_determinism ok ({threads} vs {alt} threads identical)");
        } else {
            println!(
                "  thread_determinism FAIL — {threads}-thread and {alt}-thread runs diverge \
                 (seed {}, {} decks)",
                args.seed,
                field.len(),
            );
            std::process::exit(1);
        }
    }

    // Peers exit on the `Done` their worker sent; reap them so a non-zero
    // status (a fault this side did not see first) is not lost.
    drop(peers);
    for (w, c) in children.iter_mut().enumerate() {
        match c.wait() {
            Ok(st) if st.success() => {}
            Ok(st) => eprintln!("note: peer {w} exited {st}"),
            Err(e) => eprintln!("note: peer {w}: {e}"),
        }
    }
}

// The paired-statistics tests that used to live here moved to
// `crabomination::recommend` with `paired_stat` and `wilson` themselves —
// `deck_duel` needs the same estimator, and one copy with its tests beats
// two that drift.
