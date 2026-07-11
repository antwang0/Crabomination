//! An MKM wave — crime / suspect / sacrifice-matters gaps. Absolving Lammasu
//! exercises the new `Effect::ClearSuspected` alongside `Effect::Suspect`.
//! Tests in `crabomination/src/tests/recent159.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef};
use crate::mana::{b, cost, g, generic, r, w, Color};

/// Fanatical Strength — {1}{G} Instant. Target creature gets +3/+3 and gains
/// trample until end of turn.
pub fn fanatical_strength() -> CardDefinition {
    CardDefinition {
        name: "Fanatical Strength",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Festerleech — {B} 1/1 Zombie Leech. Combat damage to a player mills you two.
/// {1}{B}: +2/+2 until end of turn, once each turn.
pub fn festerleech() -> CardDefinition {
    CardDefinition {
        name: "Festerleech",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Leech],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Mill { who: Selector::You, amount: Value::Const(2) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cornered Crook — {4}{R} 5/4 Lizard Warrior. When it enters, you may sacrifice
/// an artifact; when you do, it deals 3 damage to any target.
pub fn cornered_crook() -> CardDefinition {
    CardDefinition {
        name: "Cornered Crook",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice an artifact to deal 3 damage?".into(),
            filter: R::Artifact,
            count: Value::ONE,
            then: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(3) }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Crime Novelist — {2}{R} 1/3 Goblin Bard. Whenever you sacrifice an artifact,
/// put a +1/+1 counter on it and add {R}.
pub fn crime_novelist() -> CardDefinition {
    CardDefinition {
        name: "Crime Novelist",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Bard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red]),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Absolving Lammasu — {4}{W} 4/3 Lammasu with flying. When it enters, all
/// suspected creatures are no longer suspected. When it dies, you gain 3 life
/// and suspect up to one target creature an opponent controls.
pub fn absolving_lammasu() -> CardDefinition {
    CardDefinition {
        name: "Absolving Lammasu",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lammasu],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::ClearSuspected {
                what: Selector::EachPermanent(R::Creature),
            }),
            on_dies(Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Suspect {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                },
            ])),
        ],
        ..Default::default()
    }
}
