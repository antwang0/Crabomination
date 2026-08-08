//! More green/colorless value: counter engines, a land-drop rock, and two
//! defensive bodies. Tests in `tests/recent48.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{ManaPayload, PlayerRef};
use crate::mana::{Color, ManaCost, cost, g, generic};

/// Predator Ooze — {G}{G}{G} 1/1 Ooze. Indestructible. Whenever it attacks, put
/// a +1/+1 counter on it. Whenever a creature it dealt damage to this turn dies,
/// put a +1/+1 counter on it.
pub fn predator_ooze() -> CardDefinition {
    let add_counter = || Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Predator Ooze",
        cost: cost(&[g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            on_attack(add_counter()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::DamagedBySourceThisTurn,
                    },
                ),
                effect: add_counter(),
            },
        ],
        ..Default::default()
    }
}

/// Hornet Nest — {2}{G} 0/2 Insect. Defender. Whenever it's dealt damage, create
/// that many 1/1 green Insect tokens with flying and deathtouch.
pub fn hornet_nest() -> CardDefinition {
    let hornet = TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Hornet Nest",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: Box::new(hornet),
            },
        }],
        ..Default::default()
    }
}

/// Aerie Ouphes — {4}{G} 3/3 Ouphe. Sacrifice it: it deals damage equal to its
/// power to target creature with flying. Persist.
pub fn aerie_ouphes() -> CardDefinition {
    CardDefinition {
        name: "Aerie Ouphes",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ouphe],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Persist],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamageEqualToPower {
                source: Selector::This,
                target: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Walking Atlas — {2} 1/1 Construct artifact. {T}: you may put a land card from
/// your hand onto the battlefield.
pub fn walking_atlas() -> CardDefinition {
    CardDefinition {
        name: "Walking Atlas",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::Const(1),
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rishkar, Peema Renegade — {2}{G} 2/2 legendary Elf Druid. ETB put a +1/+1
/// counter on each of up to two target creatures. Each creature you control with
/// a counter on it has "{T}: Add {G}."
pub fn rishkar_peema_renegade() -> CardDefinition {
    CardDefinition {
        name: "Rishkar, Peema Renegade",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a counter on it has \"{T}: Add {G}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::WithAnyCounter),
                ),
                ability: ActivatedAbility {
                    tap_cost: true,
                    mana_cost: ManaCost::default(),
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColor(Color::Green, Value::Const(1)),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Gnarlid Colony — {1}{G} 2/2 Beast. Kicker {2}{G}; if kicked, enters with two
/// +1/+1 counters. Each creature you control with a +1/+1 counter on it has
/// trample.
pub fn gnarlid_colony() -> CardDefinition {
    CardDefinition {
        name: "Gnarlid Colony",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Kicker(cost(&[generic(2), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it has trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}
