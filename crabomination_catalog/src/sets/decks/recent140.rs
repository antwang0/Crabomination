//! A Wilds of Eldraine (WOE) wave: Rat token payoffs, symmetric modal disruption,
//! exile-until-leaves removal. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent140.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, ExileReturnZone, Keyword, SelectionRequirement as R,
    Selector, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneRef};
use crate::game::effects::food_token;
use crate::mana::{Color, b, cost, generic, r, w, x};

/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

// ── White ─────────────────────────────────────────────────────────────────────

/// Food Coma — {3}{W} Enchantment. ETB: exile target creature an opponent
/// controls until this leaves the battlefield, and create a Food.
pub fn food_coma() -> CardDefinition {
    CardDefinition {
        name: "Food Coma",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                return_to: ExileReturnZone::Battlefield,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            },
        ]))],
        ..Default::default()
    }
}

// ── Black ─────────────────────────────────────────────────────────────────────

/// Rankle's Prank — {2}{B}{B} Sorcery. Choose one or more — each player discards
/// two cards; each player loses 4 life; each player sacrifices two creatures.
pub fn rankles_prank() -> CardDefinition {
    CardDefinition {
        name: "Rankle's Prank",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![1, 2, 3],
            modes: vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(2),
                    random: false,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(4),
                },
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::Const(2),
                    filter: R::Creature,
                },
            ],
        },
        ..Default::default()
    }
}

// ── Red ──────────────────────────────────────────────────────────────────────

/// Song of Totentanz — {X}{R} Sorcery. Create X Rats; creatures you control gain
/// haste until end of turn.
pub fn song_of_totentanz() -> CardDefinition {
    CardDefinition {
        name: "Song of Totentanz",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: rat_token(),
            },
            Effect::GrantKeyword {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}
