//! A fourteenth wave — counter-matters and value creatures (cast-creature
//! growth, unearth attacker, a Food land-tutor). Tests in
//! `crabomination/src/tests/recent14.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, SelectionRequirement, Selector, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{target_filtered, unearth};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{cost, g, generic, w};

/// Quirion Beastcaller — {1}{G} Dryad Warrior 2/2. Whenever you cast a creature
/// spell, put a +1/+1 counter on it. When it dies, distribute X +1/+1 counters
/// among any number of target creatures you control, where X is the number of
/// +1/+1 counters on it.
pub fn quirion_beastcaller() -> CardDefinition {
    CardDefinition {
        name: "Quirion Beastcaller",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Creature)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::DistributeCounters {
                    total: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                    counter: CounterType::PlusOnePlusOne,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                    max_targets: 4,
                },
            },
        ],
        ..Default::default()
    }
}

/// Yotian Frontliner — {1} Artifact Creature — Soldier 1/1. Whenever it attacks,
/// another target creature you control gets +1/+1 until end of turn. Unearth {W}.
pub fn yotian_frontliner() -> CardDefinition {
    CardDefinition {
        name: "Yotian Frontliner",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![unearth(cost(&[w()]))],
        ..Default::default()
    }
}

/// Heaped Harvest — {2}{G} Artifact — Food. When it enters and when you
/// sacrifice it, you may search your library for a basic land card and put it
/// onto the battlefield tapped. {2}, {T}, Sacrifice this: you gain 3 life.
pub fn heaped_harvest() -> CardDefinition {
    let search = || Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand,
        to: ZoneDest::Battlefield {
            controller: PlayerRef::You,
            tapped: true,
        },
    };
    CardDefinition {
        name: "Heaped Harvest",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: search(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::SelfSource),
                effect: search(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
