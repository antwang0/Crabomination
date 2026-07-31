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

fn is_play(a: &GameAction) -> bool {
    !matches!(
        a,
        GameAction::PassPriority
            | GameAction::SubmitDecision(_)
            | GameAction::DeclareAttackers(_)
            | GameAction::DeclareBlockers(_)
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

    println!("\nplays by step (anything that isn't a pass or a declaration):");
    let total_plays: usize = c.plays_by_step.values().sum();
    for (k, v) in &c.plays_by_step {
        println!("  {k:<34} {v:>6}  {:>5.1}%", 100.0 * *v as f64 / total_plays.max(1) as f64);
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
