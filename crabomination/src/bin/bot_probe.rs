//! Instrumented bot-vs-bot games: what does the bot *actually do*?
//!
//! Companion to `bot_ladder`. The ladder answers "is profile A stronger
//! than B"; this answers "which of the bot's code paths ever run, and how
//! often". Those are different questions, and the second one has been the
//! more productive: the tap-out bug — worth eleven points of win rate —
//! was found by noticing that zero of 1366 opponent-turn priority windows
//! had a single untapped land, not by reading the bot's logic.
//!
//! The failure mode it hunts is a policy that looks right, is unit-tested,
//! and never runs. `bot.rs` is ~5000 lines of accreted heuristics whose
//! tests build their own board states by hand; a branch can be
//! individually correct and collectively unreachable, and nothing in the
//! test suite notices.
//!
//! ```text
//! cargo run --bin bot_probe -- --games 40
//! cargo run --bin bot_probe -- --deck dimir --games 100 --profile combat
//! ```

use std::collections::BTreeMap;

use crabomination::card::CardDefinition;
use crabomination::cube::{CardFactory, cube_deck, random_color_pair};
use crabomination::game::{GameAction, GameState};
use crabomination::player::Player;
use crabomination::recommend::STALE_ROUNDS;
use crabomination::server::{Bot, EvalWeights, RandomBot};
use crabomination::sos_mode::{College, sos_deck};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

/// Decision variants the bot answers with its own policy. Everything else
/// falls through to `AutoDecider`, whose answers are deliberately
/// conservative defaults rather than play decisions — `CombatDamageOrder`,
/// for instance, keeps the engine's declaration order no matter what dies.
const BOT_HANDLED: &[&str] = &[
    "Mulligan",
    "SearchLibrary",
    "OptionalTrigger",
    "ChooseCreatureType",
    "ChooseCards",
    "Discard",
    "ChooseTarget",
    "ChooseAmount",
    "PutOnLibrary",
    "Scry",
    "ChooseMode",
    "ChooseColor",
];

fn deck(spec: &[(CardFactory, usize)]) -> Vec<CardFactory> {
    let mut d = Vec::new();
    for &(f, n) in spec {
        for _ in 0..n {
            d.push(f);
        }
    }
    d
}

fn named_deck(name: &str) -> Option<Vec<CardFactory>> {
    use crabomination::catalog as c;
    Some(match name {
        "mono-red" => deck(&[
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
        "skies" => deck(&[
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
        "golgari" => deck(&[
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
        "dimir" => deck(&[
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
        _ => return None,
    })
}

const DECKS: &[&str] = &["mono-red", "skies", "golgari", "dimir"];

/// The leading identifier of a `Debug` rendering — `Scry { .. }` -> `Scry`.
fn variant_name(dbg: &str) -> String {
    dbg.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')).next().unwrap_or("?").to_string()
}

/// Per-game aggregates kept separately for the games seat 0 won and the
/// games it lost. A mirror probe averages the two together and shows
/// nothing; the question "what does the losing profile do differently" is
/// only answerable once the two populations are held apart.
#[derive(Default)]
struct Split {
    games: usize,
    turns: u32,
    attacks_eligible: usize,
    attacks_declared: usize,
    casts: usize,
    /// Life totals when the game ended, summed. Seat 0 is "us".
    our_life: i64,
    opp_life: i64,
    /// Creatures on the battlefield at game end.
    our_creatures: usize,
    opp_creatures: usize,
}

impl Split {
    fn row(&self, label: &str) -> String {
        let g = self.games.max(1) as f64;
        format!(
            "  {label:<10} {:>5} games  {:>5.1} turns  attacked {:>4.0}% of eligible               {:>4.2} casts/turn  life {:>5.1} vs {:>5.1}  creatures {:>4.1} vs {:>4.1}",
            self.games,
            self.turns as f64 / g,
            100.0 * self.attacks_declared as f64 / self.attacks_eligible.max(1) as f64,
            self.casts as f64 / self.turns.max(1) as f64,
            self.our_life as f64 / g,
            self.opp_life as f64 / g,
            self.our_creatures as f64 / g,
            self.opp_creatures as f64 / g,
        )
    }
}

#[derive(Default)]
struct Counts {
    /// Seat 0's per-game aggregates, keyed by "won" / "lost" / "drawn".
    by_outcome: BTreeMap<&'static str, Split>,
    /// This game's running tallies, folded into `by_outcome` at game end.
    cur: Split,
    /// Priority windows on an opponent's turn, and how many had mana up.
    opp_windows: usize,
    opp_windows_with_mana: usize,
    /// Actions the bot emitted, keyed by "step / kind".
    actions: BTreeMap<String, usize>,
    /// Non-pass, non-decision plays keyed by the step they happened in.
    plays_by_step: BTreeMap<String, usize>,
    /// Decisions the bot was asked, by variant.
    decisions: BTreeMap<String, usize>,
    turns: u32,
    games: usize,
    /// Combats where we were the defender and something was attacking us.
    combats: usize,
    /// ...of those, how many we had at least one untapped creature for.
    combats_with_blockers: usize,
    /// ...of those, how many we actually declared a block in.
    combats_blocked: usize,
    /// Attackers faced / left unblocked, summed over all combats.
    attackers_faced: usize,
    attackers_unblocked: usize,
    /// Blockers available / assigned, summed over all combats.
    blockers_available: usize,
    blockers_used: usize,
    /// Creatures we controlled at DeclareBlockers, and how many of those
    /// were unavailable because they were tapped. A creature that attacked
    /// on our turn is still tapped on theirs (it untaps in *our* untap
    /// step), so this separates "empty board" from "we alpha-struck and
    /// have nothing left to block with".
    creatures_at_block: usize,
    creatures_tapped_at_block: usize,
    /// Combats with no available blocker, split by cause.
    no_blocker_empty: usize,
    no_blocker_tapped: usize,
    /// Our own combats: creatures eligible to attack vs actually declared.
    /// A bot that swings with everything every turn is the direct cause of
    /// the tapped-out blocking picture above.
    attacks_eligible: usize,
    attacks_declared: usize,
    /// How often we held back at least one eligible attacker.
    attack_combats: usize,
    attack_combats_all_in: usize,
    /// How each game ended. `stale` is the one that matters: eight
    /// consecutive rounds in which *neither* bot could act is a deadlock,
    /// not a long game, and it silently drops the game from every
    /// measurement built on simulated play — the ladder's win rates and the
    /// card recommender's per-slot attribution alike.
    ended_decided: usize,
    ended_action_cap: usize,
    ended_stale: usize,
    /// Where the deadlocked games got stuck, keyed by "step / turn band".
    stall_sites: BTreeMap<String, usize>,
    /// What the bot kept proposing once a game was wedged, and the engine's
    /// rejection reason.
    stuck_actions: BTreeMap<String, usize>,
    /// How many combats had a given (attackers x available blockers) size.
    /// The search only has room to work where this product exceeds 1.
    combat_shapes: BTreeMap<(usize, usize), usize>,
}

fn action_kind(a: &GameAction) -> &'static str {
    match a {
        GameAction::PassPriority => "PassPriority",
        GameAction::SubmitDecision(_) => "SubmitDecision",
        GameAction::PlayLand(_) | GameAction::PlayLandFromGraveyard(_) => "PlayLand",
        GameAction::ActivateAbility { .. } => "ActivateAbility",
        GameAction::ActivateLoyaltyAbility { .. } => "Loyalty",
        GameAction::DeclareAttackers(_) => "DeclareAttackers",
        GameAction::DeclareBlockers(_) => "DeclareBlockers",
        _ => "Cast",
    }
}

/// A *cast* — the thing whose timing the bot actually chooses. Land drops
/// are excluded deliberately: they are sorcery-speed by rule and can never
/// be held, so counting them as "plays" hides the timing signal under a
/// constant. (They were about 45 % of precombat "plays", which made a real
/// shift toward the second main look like no shift at all.)
fn is_play(a: &GameAction) -> bool {
    !matches!(
        a,
        GameAction::PassPriority
            | GameAction::SubmitDecision(_)
            | GameAction::DeclareAttackers(_)
            | GameAction::DeclareBlockers(_)
            | GameAction::PlayLand(_)
            | GameAction::PlayLandFromGraveyard(_)
    )
}

fn run(
    deck: &[CardFactory],
    games: usize,
    weights: EvalWeights,
    weights_b: EvalWeights,
    c: &mut Counts,
) {
    for _ in 0..games {
        let mut g = GameState::new(vec![Player::new(0, "A"), Player::new(1, "B")]);
        let mut r = rand::rng();
        for seat in 0..2 {
            for &f in deck {
                let def: CardDefinition = f();
                g.add_card_to_library(seat, def);
            }
            g.players[seat].library.shuffle(&mut r);
            g.players[seat].wants_ui = true;
        }
        g.start_mulligan_phase();
        // Seat 0 is the profile under study; seat 1 is what it is being
        // measured against, so a head-to-head reproduces the ladder pairing
        // rather than a mirror.
        c.cur = Split::default();
        let mut bots: Vec<Box<dyn Bot>> = vec![
            Box::new(RandomBot::with_weights(weights)),
            Box::new(RandomBot::with_weights(weights_b)),
        ];

        let (mut actions, mut stale) = (0usize, 0usize);
        while !g.is_game_over() && actions < 20_000 && stale < STALE_ROUNDS {
            let mut any = false;
            for (s, bot) in bots.iter_mut().enumerate() {
                // Only instrument seat 0 so the numbers are per-player.
                let observing = s == 0;
                if observing
                    && let Some(pd) = &g.pending_decision
                    && pd.acting_player() == s
                {
                    *c.decisions
                        .entry(variant_name(&format!("{:?}", pd.decision)))
                        .or_default() += 1;
                }
                let Some(a) = bot.next_action(&g, s) else {
                    if stale >= 4 {
                        *c.stuck_actions.entry(format!("seat{s} -> None")).or_default() += 1;
                    }
                    continue;
                };
                // Once the game is visibly wedged, record what the bot keeps
                // proposing and why the engine keeps refusing it. A rejected
                // action leaves `any` false, and the bot never falls back to
                // passing, so one un-castable candidate deadlocks the game.
                if stale >= 4 && let Err(e) = g.clone().perform_action(a.clone()) {
                    let ctx = match &g.pending_decision {
                        Some(pd) => {
                            let d = format!("{:?}", pd.decision);
                            d.chars().take(150).collect::<String>()
                        }
                        None => "no-decision".to_string(),
                    };
                    *c.stuck_actions
                        .entry(format!("{:?} / {} / {:?} -> {e:?}", g.step, ctx, action_kind(&a)))
                        .or_default() += 1;
                }
                if observing && g.pending_decision.is_none() && g.active_player_idx != s {
                    c.opp_windows += 1;
                    if g
                        .battlefield
                        .iter()
                        .any(|p| p.controller == s && p.definition.is_land() && !p.tapped)
                    {
                        c.opp_windows_with_mana += 1;
                    }
                }
                // Combat shape, sampled at the moment the bot commits its
                // blocks. `attacking()` is only populated between the two
                // declaration steps, so this is the one place the defender's
                // real option set is visible.
                if observing && let GameAction::DeclareBlockers(blocks) = &a {
                    let faced: Vec<_> = g
                        .attacking()
                        .iter()
                        .filter(|atk| g.defender_for(atk.target) == Some(s))
                        .collect();
                    if !faced.is_empty() {
                        let avail = g
                            .battlefield
                            .iter()
                            .filter(|c| c.controller == s && c.can_block())
                            .count();
                        c.combats += 1;
                        c.attackers_faced += faced.len();
                        c.blockers_available += avail;
                        c.blockers_used += blocks.len();
                        c.attackers_unblocked += faced
                            .iter()
                            .filter(|atk| !blocks.iter().any(|(_, a)| *a == atk.attacker))
                            .count();
                        let mine: Vec<_> = g
                            .battlefield
                            .iter()
                            .filter(|cr| cr.controller == s && cr.definition.is_creature())
                            .collect();
                        let tapped = mine.iter().filter(|cr| cr.tapped).count();
                        c.creatures_at_block += mine.len();
                        c.creatures_tapped_at_block += tapped;
                        if avail > 0 {
                            c.combats_with_blockers += 1;
                        } else if tapped > 0 {
                            c.no_blocker_tapped += 1;
                        } else {
                            c.no_blocker_empty += 1;
                        }
                        if !blocks.is_empty() {
                            c.combats_blocked += 1;
                        }
                        *c.combat_shapes.entry((faced.len(), avail)).or_default() += 1;
                    }
                }
                if observing && let GameAction::DeclareAttackers(atks) = &a {
                    // `can_attack()` is the engine's eligibility gate; the
                    // bot layers its own restraint on top, and the gap
                    // between the two is the whole question.
                    let eligible = g
                        .battlefield
                        .iter()
                        .filter(|cr| cr.controller == s && cr.can_attack())
                        .count();
                    if eligible > 0 {
                        c.cur.attacks_eligible += eligible;
                        c.cur.attacks_declared += atks.len();
                        c.attack_combats += 1;
                        c.attacks_eligible += eligible;
                        c.attacks_declared += atks.len();
                        if atks.len() >= eligible {
                            c.attack_combats_all_in += 1;
                        }
                    }
                }
                let step = format!("{:?}", g.step);
                let own = g.active_player_idx == s;
                let kind = action_kind(&a);
                let play = is_play(&a);
                if g.perform_action(a).is_ok() {
                    if observing {
                        let suffix = if own { "" } else { " (opp turn)" };
                        *c.actions.entry(format!("{step}{suffix} / {kind}")).or_default() += 1;
                        if play {
                            *c.plays_by_step.entry(format!("{step}{suffix}")).or_default() += 1;
                            c.cur.casts += 1;
                        }
                    }
                    any = true;
                    actions += 1;
                    if g.is_game_over() {
                        break;
                    }
                }
            }
            if any { stale = 0 } else { stale += 1 }
        }
        if g.is_game_over() {
            c.ended_decided += 1;
        } else if stale >= STALE_ROUNDS {
            c.ended_stale += 1;
            let band = match g.turn_number {
                0..=10 => "t<=10",
                11..=25 => "t11-25",
                26..=60 => "t26-60",
                _ => "t>60",
            };
            *c.stall_sites.entry(format!("{:?} / {band}", g.step)).or_default() += 1;
        } else {
            c.ended_action_cap += 1;
        }
        let label = match g.game_over.flatten() {
            Some(0) => "won",
            Some(_) => "lost",
            None => "drawn",
        };
        c.cur.games = 1;
        c.cur.turns = g.turn_number;
        c.cur.our_life = g.players[0].life as i64;
        c.cur.opp_life = g.players[1].life as i64;
        c.cur.our_creatures =
            g.battlefield.iter().filter(|x| x.controller == 0 && x.definition.is_creature()).count();
        c.cur.opp_creatures =
            g.battlefield.iter().filter(|x| x.controller == 1 && x.definition.is_creature()).count();
        let e = c.by_outcome.entry(label).or_default();
        e.games += 1;
        e.turns += c.cur.turns;
        e.attacks_eligible += c.cur.attacks_eligible;
        e.attacks_declared += c.cur.attacks_declared;
        e.casts += c.cur.casts;
        e.our_life += c.cur.our_life;
        e.opp_life += c.cur.opp_life;
        e.our_creatures += c.cur.our_creatures;
        e.opp_creatures += c.cur.opp_creatures;
        c.turns += g.turn_number;
        c.games += 1;
    }
}

fn profile_weights(name: &str) -> EvalWeights {
    match name {
        "baseline" => EvalWeights::baseline(),
        "combat" => EvalWeights::combat_aware(),
        "pretap" => EvalWeights::legacy_mana(),
        "holdsick" => EvalWeights::hold_sick(),
        "default" => EvalWeights::default(),
        "atk" => EvalWeights::attack_search(),
        "planner" => EvalWeights::planner(),
        "lookahead" => EvalWeights::lookahead1(),
        "holdsick+combat" => EvalWeights::hold_sick_combat(),
        "blk" => EvalWeights::block_search(),
        other => {
            eprintln!("unknown profile {other}");
            std::process::exit(2);
        }
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut games = 40usize;
    let mut which: Option<String> = None;
    let mut profile = "baseline".to_string();
    let mut seed = 23u64;
    let mut vs: Option<String> = None;
    let mut i = 0;
    while i < argv.len() {
        let val = || argv.get(i + 1).cloned().unwrap_or_default();
        match argv[i].as_str() {
            "--games" => games = val().parse().unwrap_or(games),
            "--deck" => which = Some(val()),
            "--profile" => profile = val(),
            "--seed" => seed = val().parse().unwrap_or(seed),
            "--vs" => vs = Some(val()),
            "-h" | "--help" => {
                println!(
                    "bot_probe [--deck NAME] [--games N] [--profile baseline|combat]\n\
                     [--seed N, matches bot_ladder's cube and sos decks]\n\
                     [--vs PROFILE puts a different bot in seat 1]\n\
                     decks: {}, cube, sos, sos:<college>",
                    DECKS.join(", ")
                );
                return;
            }
            other => {
                eprintln!("unknown argument {other}");
                std::process::exit(2);
            }
        }
        i += 2;
    }
    let lookup = |name: &str| -> EvalWeights { profile_weights(name) };
    let weights = lookup(&profile);
    let weights_b = vs.as_deref().map(lookup).unwrap_or(weights);

    // Whole games recurse through `Effect` trees with debug-build frames
    // big enough to overflow the default main-thread stack (the SOS decks
    // did, first try). `bot_ladder` sizes its workers at 32 MB for the
    // same reason; run the probe on a matching thread.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || probe(games, which, profile, seed, weights, weights_b))
        .expect("spawn probe thread")
        .join()
        .expect("probe thread panicked");
}

fn probe(
    games: usize,
    which: Option<String>,
    profile: String,
    seed: u64,
    weights: EvalWeights,
    weights_b: EvalWeights,
) {
    if which.as_deref() == Some("cube") {
        // Same construction the ladder uses, so stall rates are comparable
        // between the two tools.
        let mut rng = StdRng::seed_from_u64(seed ^ 0xC0BE_5EED);
        let mut c = Counts::default();
        for i in 0..8 {
            let colors = random_color_pair(&mut rng);
            let d = cube_deck(colors, &mut rng);
            let mut one = Counts::default();
            run(&d, games, weights, weights_b, &mut one);
            println!(
                "pair {i} {colors:?}: {} decided, {} cap, {} DEADLOCKED",
                one.ended_decided, one.ended_action_cap, one.ended_stale
            );
            for (k, v) in &one.stall_sites {
                println!("    stuck at {k:<28} {v:>5}");
            }
            c.ended_decided += one.ended_decided;
            c.ended_action_cap += one.ended_action_cap;
            c.ended_stale += one.ended_stale;
            c.games += one.games;
            c.turns += one.turns;
            for (k, v) in one.stall_sites {
                *c.stall_sites.entry(k).or_default() += v;
            }
            for (k, v) in one.stuck_actions {
                *c.stuck_actions.entry(k).or_default() += v;
            }
        }
        report("cube", &c, &profile);
        return;
    }
    if let Some(sos) = which.as_deref().and_then(|s| s.strip_prefix("sos")) {
        // Same seeded construction as the ladder's `--decks sos`, so the
        // probe describes the decks the ladder measures — every college's
        // deck is drawn from the stream even when only one is probed, so
        // `--deck sos:prismari --seed N` plays the exact deck the ladder's
        // "sos Prismari" row played at seed N. Bare `sos` folds all five
        // into one report (the decision mix of the format); `sos:<college>`
        // isolates one (what the pilot does differently in THAT deck).
        let only = match sos.strip_prefix(':') {
            None if sos.is_empty() => None,
            Some(name) => match College::ALL
                .into_iter()
                .find(|c| c.name().eq_ignore_ascii_case(name))
            {
                Some(c) => Some(c),
                None => {
                    eprintln!(
                        "unknown college {name}; expected one of: {}",
                        College::ALL.map(|c| c.name()).join(", ")
                    );
                    std::process::exit(2);
                }
            },
            None => {
                eprintln!("unknown deck sos{sos}; try sos or sos:<college>");
                std::process::exit(2);
            }
        };
        let mut rng = StdRng::seed_from_u64(seed ^ 0x0505_ACAD);
        let mut c = Counts::default();
        for college in College::ALL {
            let d = sos_deck(college, &mut rng);
            if only.is_none_or(|o| o == college) {
                run(&d, games, weights, weights_b, &mut c);
            }
        }
        let label = match only {
            Some(o) => format!("sos:{}", o.name()),
            None => "sos".to_string(),
        };
        report(&label, &c, &profile);
        return;
    }
    let decks: Vec<&str> = match &which {
        Some(n) => vec![n.as_str()],
        None => DECKS.to_vec(),
    };
    let mut c = Counts::default();
    for name in &decks {
        let Some(d) = named_deck(name) else {
            eprintln!("unknown deck {name}; known: {}", DECKS.join(", "));
            std::process::exit(2);
        };
        run(&d, games, weights, weights_b, &mut c);
    }

    report(&decks.join("+"), &c, &profile);
}

fn report(decks: &str, c: &Counts, profile: &str) {
    println!("profile={profile} decks={decks} games={} turns={}\n", c.games, c.turns);
    println!(
        "games ended: {} decided, {} action cap, {} DEADLOCKED ({:.1}%)",
        c.ended_decided,
        c.ended_action_cap,
        c.ended_stale,
        100.0 * c.ended_stale as f64 / c.games.max(1) as f64,
    );
    for (k, v) in &c.stall_sites {
        println!("  stuck at {k:<28} {v:>5}");
    }
    if !c.stuck_actions.is_empty() {
        println!("  what the bot kept proposing while wedged:");
        let mut v: Vec<_> = c.stuck_actions.iter().collect();
        v.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
        for (k, n) in v.iter().take(12) {
            println!("    {k:<70} {n:>6}");
        }
    }
    println!(
        "opponent-turn priority windows: {} ({} with mana up, {:.0}%)",
        c.opp_windows,
        c.opp_windows_with_mana,
        100.0 * c.opp_windows_with_mana as f64 / c.opp_windows.max(1) as f64,
    );

    println!("\ncasts by step (land drops excluded — they can't be held):");
    let total_plays: usize = c.plays_by_step.values().sum();
    // Casts per turn is the direct read on sequencing: a bot that can only
    // score one action at a time tends to take the single biggest play it
    // can afford and stop, so a lookahead that works shows up here before
    // it shows up on the ladder.
    println!(
        "  (total {total_plays} casts over {} turns = {:.2} per turn)",
        c.turns,
        total_plays as f64 / c.turns.max(1) as f64,
    );
    for (k, v) in &c.plays_by_step {
        println!("  {k:<34} {v:>6}  {:>5.1}%", 100.0 * *v as f64 / total_plays.max(1) as f64);
    }

    // Combat is the highest-frequency surface the bot has and the only one
    // never laddered. The question this section answers is not "does the bot
    // block well" but "is there room to" — a search over block assignments
    // can only pay where the defender had a real choice, so the shape
    // histogram sizes the ceiling before any work goes into the search.
    if c.by_outcome.len() > 1 {
        println!("\nseat 0 ({profile}) by outcome:");
        for (label, sp) in &c.by_outcome {
            println!("{}", sp.row(label));
        }
    }
    println!("\ncombats as defender: {}", c.combats);
    let pct = |n: usize, d: usize| 100.0 * n as f64 / d.max(1) as f64;
    println!(
        "  with any untapped creature   {:>6}  {:>5.1}%\n  \
         we declared a block in       {:>6}  {:>5.1}% (of those)",
        c.combats_with_blockers,
        pct(c.combats_with_blockers, c.combats),
        c.combats_blocked,
        pct(c.combats_blocked, c.combats_with_blockers),
    );
    println!(
        "  no blocker available: {} board was empty, {} all tapped\n  \
         creatures on board at DeclareBlockers {} ({} tapped, {:.0}%)",
        c.no_blocker_empty,
        c.no_blocker_tapped,
        c.creatures_at_block,
        c.creatures_tapped_at_block,
        pct(c.creatures_tapped_at_block, c.creatures_at_block),
    );
    println!(
        "  attackers faced {} / unblocked {} ({:.0}%)\n  \
         blockers available {} / used {} ({:.0}%)",
        c.attackers_faced,
        c.attackers_unblocked,
        pct(c.attackers_unblocked, c.attackers_faced),
        c.blockers_available,
        c.blockers_used,
        pct(c.blockers_used, c.blockers_available),
    );
    // A combat with one attacker and one blocker is a yes/no choice the
    // greedy rule already gets right or wrong on its own merits; anything
    // larger is where an assignment search has something to search.
    let (mut trivial, mut real) = (0usize, 0usize);
    for (&(atk, blk), &n) in &c.combat_shapes {
        if atk * blk.min(1) <= 1 && blk <= 1 { trivial += n } else { real += n }
    }
    println!(
        "  trivial shapes (<=1 attacker and <=1 blocker) {trivial}, \
         searchable {real} ({:.0}%)",
        pct(real, trivial + real),
    );
    println!(
        "\nour own combats with an eligible attacker: {}\n  \
         attacked with {} of {} eligible ({:.0}%)\n  \
         swung with everything in {} ({:.0}%)",
        c.attack_combats,
        c.attacks_declared,
        c.attacks_eligible,
        pct(c.attacks_declared, c.attacks_eligible),
        c.attack_combats_all_in,
        pct(c.attack_combats_all_in, c.attack_combats),
    );
    println!("\n  top shapes (attackers x blockers available):");
    let mut shapes: Vec<_> = c.combat_shapes.iter().collect();
    shapes.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    for &(&(atk, blk), &n) in shapes.iter().take(10) {
        println!("    {atk} x {blk:<3} {n:>6}  {:>5.1}%", pct(n, c.combats));
    }

    println!("\nactions by step and kind:");
    for (k, v) in &c.actions {
        println!("  {k:<44} {v:>6}");
    }

    println!("\ndecisions the bot was asked (* = falls through to AutoDecider):");
    let total_dec: usize = c.decisions.values().sum();
    for (k, v) in &c.decisions {
        let handled = BOT_HANDLED.contains(&k.as_str());
        println!(
            "  {}{:<28} {:>6}  {:>5.1}%",
            if handled { ' ' } else { '*' },
            k,
            v,
            100.0 * *v as f64 / total_dec.max(1) as f64,
        );
    }
    let unhandled: usize =
        c.decisions.iter().filter(|(k, _)| !BOT_HANDLED.contains(&k.as_str())).map(|(_, v)| *v).sum();
    println!(
        "\n{unhandled} of {total_dec} decisions ({:.0}%) use AutoDecider's default rather than a \
         play decision.",
        100.0 * unhandled as f64 / total_dec.max(1) as f64,
    );
}
