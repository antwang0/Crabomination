//! A fifteenth wave — Elf tribal payoffs. Tests in
//! `crabomination/src/tests/recent15.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement, Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, g, generic};

/// Shaman of the Pack — {1}{B}{G} Elf Shaman 3/2. ETB: each opponent loses life
/// equal to the number of Elves you control. (Printed "target opponent" is
/// modeled as each opponent — exact in 1v1.)
pub fn shaman_of_the_pack() -> CardDefinition {
    CardDefinition {
        name: "Shaman of the Pack",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Elf)
                        .and(SelectionRequirement::ControlledByYou),
                )),
                filter: SelectionRequirement::Any,
            },
        })],
        ..Default::default()
    }
}

/// Elvish Warmaster — {1}{G} Elf Warrior 2/2. Whenever one or more other Elves
/// you control enter, create a 1/1 green Elf Warrior (once each turn).
/// {5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.
pub fn elvish_warmaster() -> CardDefinition {
    let elf_token = TokenDefinition {
        name: "Elf Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    let elves = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Elf)
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Elvish Warmaster",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Elf),
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: elf_token,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: elves(),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: elves(),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
