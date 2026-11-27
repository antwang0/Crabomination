//! Commander: closers for Cleveland Booker, Empath's EDHREC average deck
//! (scan of 2026-11-27). Tests in `tests/recent_b/cmdr_01wr2y_a.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, LandType,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition,
};
use crate::effect::shortcut::{etb, landfall};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, StaticEffect, Value};
use crate::mana::{Color, cost, g, generic, hybrid, r};

/// Cleveland Booker, Empath — {2}{R/G}{R/G} legendary 2/2 Kwejian Rogue.
/// Landfall: a 2/2 red and green Worm token with "This token can't block".
/// {8}: creatures you control get +2/+0 until end of turn.
pub fn cleveland_booker_empath() -> CardDefinition {
    let worm = Arc::new(TokenDefinition {
        name: "Worm".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Worm], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    });
    CardDefinition {
        name: "Cleveland Booker, Empath",
        cost: cost(&[generic(2), hybrid(Color::Red, Color::Green), hybrid(Color::Red, Color::Green)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kwejian, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![landfall(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: worm })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Temporal Anomaly — land. {T}: {C}. {3}{G}, {T}, sacrifice it: mill four,
/// you may put a permanent card from among them into your hand, and a tapped
/// New Planet land token ("{T}: one mana of any color") enters.
pub fn temporal_anomaly() -> CardDefinition {
    let planet = Arc::new(TokenDefinition {
        name: "New Planet".into(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        name: "Temporal Anomaly",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3), g()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::MillThenToHandN {
                        amount: Value::Const(4),
                        filter: R::Permanent,
                        take: Value::ONE,
                        otherwise: None,
                    },
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: planet },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Xindi Surveyors — {2}{G} 3/2 Xindi Scientist. Landfall: a +1/+1 counter on it.
pub fn xindi_surveyors() -> CardDefinition {
    CardDefinition {
        name: "Xindi Surveyors",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Xindi, CreatureType::Scientist], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![landfall(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Hit It! — {1}{R}{G} enchantment. Creatures you control have haste. ETB:
/// reveal the top five; a land card among them enters tapped, the rest go to
/// your graveyard.
pub fn hit_it() -> CardDefinition {
    CardDefinition {
        name: "Hit It!",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            count: Value::Const(5),
            take: Some(Value::ONE),
            pick_filter: Some(R::Land),
            picked_lands_to_battlefield: true,
            rest_to_graveyard: true,
            ..Default::default()
        })))],
        ..Default::default()
    }
}
