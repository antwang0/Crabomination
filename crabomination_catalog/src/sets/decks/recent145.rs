//! A Wilds of Eldraine (WOE) legends wave: a tap-matters reflexive modal
//! (Hylda, showcasing `YouTapped`) and a Celebration attacker. Tests in
//! `crabomination/src/tests/recent145.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::on_attack;
use crate::effect::{Effect, EventKind, EventScope, EventSpec, PlayerRef, ZoneRef};
use crate::mana::{Color, cost, generic, r, u, w};

/// 4/4 white-and-blue Elemental token (Hylda's first mode).
fn elemental_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Hylda of the Icy Crown — {2}{W}{U} 3/4 legendary Human Warlock. Whenever you
/// tap an untapped creature an opponent controls, pay {1} for a modal payoff.
pub fn hylda_of_the_icy_crown() -> CardDefinition {
    CardDefinition {
        name: "Hylda of the Icy Crown",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::YouTapped).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            ),
            effect: Effect::MayPay {
                description: "pay {1} for Hylda's payoff".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::ChooseMode(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: elemental_token(),
                    },
                    Effect::AddCounter {
                        what: Selector::EachMatching {
                            zone: ZoneRef::Battlefield,
                            filter: R::Creature.and(R::ControlledByYou),
                        },
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::Seq(vec![
                        Effect::Scry {
                            who: PlayerRef::You,
                            amount: Value::Const(2),
                        },
                        Effect::Draw {
                            who: Selector::You,
                            amount: Value::ONE,
                        },
                    ]),
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Ash, Party Crasher — {R}{W} 2/2 legendary Human Peasant with haste.
/// Celebration — attacking, if 2+ nonland permanents entered this turn, grow.
pub fn ash_party_crasher() -> CardDefinition {
    CardDefinition {
        name: "Ash, Party Crasher",
        cost: cost(&[r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::CelebrationActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}
