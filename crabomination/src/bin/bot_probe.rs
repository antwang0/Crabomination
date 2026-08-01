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
use crabomination::cube::CardFactory;
use crabomination::game::{GameAction, GameState};
use crabomination::player::Player;
use crabomination::server::{Bot, EvalWeights, RandomBot};
use rand::seq::SliceRandom;

/// Decision variants the bot answers with its own policy. Everything else
/// falls through to `AutoDecider`, whose answers are deliberately
/// conservative defaults rather than play decisions — `Scry`, for
/// instance, keeps every card on top and bottoms nothing, which makes
/// every scry and surveil in the catalog a no-op under bot play.
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

#[derive(Default)]
struct Counts {
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

fn run(deck: &[CardFactory], games: usize, weights: EvalWeights, c: &mut Counts) {
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
        let mut bots: Vec<Box<dyn Bot>> =
            (0..2).map(|_| -> Box<dyn Bot> { Box::new(RandomBot::with_weights(weights)) }).collect();

        let (mut actions, mut stale) = (0usize, 0usize);
        while !g.is_game_over() && actions < 20_000 && stale < 8 {
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
                let Some(a) = bot.next_action(&g, s) else { continue };
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
        c.turns += g.turn_number;
        c.games += 1;
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut games = 40usize;
    let mut which: Option<String> = None;
    let mut profile = "baseline".to_string();
    let mut i = 0;
    while i < argv.len() {
        let val = || argv.get(i + 1).cloned().unwrap_or_default();
        match argv[i].as_str() {
            "--games" => games = val().parse().unwrap_or(games),
            "--deck" => which = Some(val()),
            "--profile" => profile = val(),
            "-h" | "--help" => {
                println!(
                    "bot_probe [--deck NAME] [--games N] [--profile baseline|combat]\n\
                     decks: {}",
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
    let weights = match profile.as_str() {
        "baseline" => EvalWeights::baseline(),
        "combat" => EvalWeights::combat_aware(),
        "pretap" => EvalWeights::legacy_mana(),
        "holdsick" => EvalWeights::hold_sick(),
        "default" => EvalWeights::default(),
        "atk" => EvalWeights::attack_search(),
        "planner" => EvalWeights::planner(),
        "lookahead" => EvalWeights::lookahead1(),
        other => {
            eprintln!("unknown profile {other}");
            std::process::exit(2);
        }
    };

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
        run(&d, games, weights, &mut c);
    }

    println!(
        "profile={profile} decks={} games={} turns={}\n",
        decks.join("+"),
        c.games,
        c.turns
    );
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
