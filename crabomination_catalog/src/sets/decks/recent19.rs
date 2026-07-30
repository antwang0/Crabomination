//! A nineteenth wave — a few more Foundations (FDN) creatures: an enters-matters
//! beater, a firebreathing-toughness flyer, and a defender-mill Wall. Tests in
//! `crabomination/src/tests/recent19.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::mana::{cost, g, generic, u, w};

/// Beast-Kin Ranger — {2}{G} Elf Ranger 3/3 with trample. Whenever another
/// creature you control enters, it gets +1/+0 until end of turn.
pub fn beast_kin_ranger() -> CardDefinition {
    CardDefinition {
        name: "Beast-Kin Ranger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Ranger],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Marble Gargoyle — {2}{W} Artifact Creature — Gargoyle 2/2 with flying.
/// {W}: It gets +0/+1 until end of turn.
pub fn marble_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Marble Gargoyle",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gargoyle],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Coral Colony — {1}{U} Wall 1/4 with defender. {1}{U}, {T}: Target player mills
/// X, where X is the number of creatures with defender you control.
pub fn coral_colony() -> CardDefinition {
    CardDefinition {
        name: "Coral Colony",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasKeyword(Keyword::Defender)),
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
