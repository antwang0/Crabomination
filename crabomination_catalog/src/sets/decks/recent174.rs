//! Final Aetherdrift (DFT) gap batch: a max-speed spell-copier, an exhaust
//! team-trample Goblin, and an exhaust Vehicle that scales its counters to your
//! Mounts/Vehicles. Tests in `crabomination/src/tests/recent174.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{cost, g, generic, r, u};

/// Slick Imitator — {1}{U} 1/3 Ooze. Start your engines! Max speed — {1},
/// Sacrifice this: copy target spell you control (you may choose new targets).
pub fn slick_imitator() -> CardDefinition {
    CardDefinition {
        name: "Slick Imitator",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            condition: Some(Predicate::SpeedAtLeast {
                who: PlayerRef::You,
                speed: 4,
            }),
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(R::IsSpellOnStack.and(R::ControlledByYou)),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Boom Scholar — {1}{R}{G} 3/3 Goblin Advisor. Exhaust abilities of your other
/// permanents cost {2} less. Exhaust — {4}{R}{G}: creatures and Vehicles you
/// control gain trample until end of turn; put two +1/+1 counters on this.
pub fn boom_scholar() -> CardDefinition {
    CardDefinition {
        name: "Boom Scholar",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Exhaust abilities of your other permanents cost {2} less to activate.",
            effect: StaticEffect::OtherExhaustActivationCostReduction { amount: 2 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r(), g()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        R::Creature
                            .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                            .and(R::ControlledByYou),
                    ),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spire Mechcycle — {4}{R} Artifact — Vehicle 5/4. Haste. Exhaust — Tap another
/// untapped Mount or Vehicle you control: this becomes an artifact creature; put
/// a +1/+1 counter on it for each other Mount and/or Vehicle you control. Crew 2.
pub fn spire_mechcycle() -> CardDefinition {
    let vehicles_you_control = R::HasCreatureType(CreatureType::Mount)
        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
        .and(R::ControlledByYou)
        .and(R::OtherThanSource);
    CardDefinition {
        name: "Spire Mechcycle",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste, Keyword::Crew(2)],
        activated_abilities: vec![ActivatedAbility {
            exhaust: true,
            tap_other_filter: Some(vehicles_you_control.clone()),
            effect: Effect::Seq(vec![
                Effect::AddCardTypeIndefinitely {
                    what: Selector::This,
                    card_type: CardType::Creature,
                    until_eot: false,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(vehicles_you_control)),
                        filter: R::Any,
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
