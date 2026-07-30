//! Charge-counter artifacts, hate-bears, and utility legends. Anchors the new
//! `Effect::DestroyEachNonlandWithManaValue` (Ratchet Bomb / Engineered
//! Explosives) and `StaticEffect::NoncreatureSpellsCantBeCastIf` (Gaddock Teeg).
//! Tests in `tests/recent42.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::{ActivatedAbility, ManaPayload, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, w, x};

/// Charge-counter mana value: the count of charge counters on the source.
fn charge_count() -> Value {
    Value::CountersOn {
        what: Box::new(Selector::This),
        kind: CounterType::Charge,
    }
}

/// `{T}, Sacrifice this: Destroy each nonland permanent with mana value equal to
/// this permanent's charge counters.`
fn charge_bomb_detonate(extra_mana: u32) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        mana_cost: if extra_mana == 0 {
            ManaCost::default()
        } else {
            cost(&[generic(extra_mana)])
        },
        sac_cost: true,
        effect: Effect::DestroyEachNonlandWithManaValue {
            value: charge_count(),
        },
        ..Default::default()
    }
}

/// Ratchet Bomb — {2} Artifact. `{T}: Put a charge counter on it.` `{T},
/// Sacrifice: Destroy each nonland permanent whose mana value equals its
/// charge counters.`
pub fn ratchet_bomb() -> CardDefinition {
    CardDefinition {
        name: "Ratchet Bomb",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
            charge_bomb_detonate(0),
        ],
        ..Default::default()
    }
}

/// Engineered Explosives — {X} Artifact. Sunburst (enters with a charge counter
/// per color of mana spent). `{2}, Sacrifice: Destroy each nonland permanent
/// whose mana value equals its charge counters.`
pub fn engineered_explosives() -> CardDefinition {
    CardDefinition {
        name: "Engineered Explosives",
        cost: cost(&[x()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Sunburst],
        activated_abilities: vec![charge_bomb_detonate(2)],
        ..Default::default()
    }
}

/// Sphere of the Suns — {2} Artifact that enters tapped with three charge
/// counters. `{T}, Remove a charge counter: Add one mana of any color.`
pub fn sphere_of_the_suns() -> CardDefinition {
    CardDefinition {
        name: "Sphere of the Suns",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::This,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mox Tantalite — Artifact. Suspend 3—{0}. `{T}: Add one mana of any color.`
pub fn mox_tantalite() -> CardDefinition {
    CardDefinition {
        name: "Mox Tantalite",
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Suspend(3, ManaCost::default())],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gaddock Teeg — {G}{W} 2/2 legendary Kithkin Advisor. Noncreature spells with
/// mana value 4+ or with {X} in their cost can't be cast.
pub fn gaddock_teeg() -> CardDefinition {
    CardDefinition {
        name: "Gaddock Teeg",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells with mana value 4 or greater can't be cast. Noncreature spells with {X} in their mana costs can't be cast.",
            effect: StaticEffect::NoncreatureSpellsCantBeCastIf {
                min_mana_value: 4,
                or_has_x: true,
            },
        }],
        ..Default::default()
    }
}

/// The Tabernacle at Pendrell Vale — Legendary Land. All creatures have "At the
/// beginning of your upkeep, destroy this creature unless you pay {1}."
pub fn the_tabernacle_at_pendrell_vale() -> CardDefinition {
    CardDefinition {
        name: "The Tabernacle at Pendrell Vale",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "All creatures have \"At the beginning of your upkeep, destroy this creature unless you pay {1}.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature,
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::Upkeep),
                        EventScope::YourControl,
                    ),
                    effect: Effect::UnlessPlayerPays {
                        who: PlayerRef::You,
                        cost: WardCost::generic(1),
                        then: Box::new(Effect::Destroy {
                            what: Selector::This,
                        }),
                    },
                }),
            },
        }],
        ..Default::default()
    }
}
