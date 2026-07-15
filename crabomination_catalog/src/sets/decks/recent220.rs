//! Bloomburrow (BLB) gap batch — a supply-counter card-draw enchantment, a
//! can't-block Aura, a token-crewed Vehicle, and a graveyard-gated cantrip.
//! Tests in `tests/recent220.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
};
use crate::mana::{cost, b, g, generic, r};

/// Stocking the Pantry — {G} Enchantment. Whenever you put one or more +1/+1
/// counters on a creature you control, put a supply counter on this. {2}, remove
/// a supply counter: Draw a card.
pub fn stocking_the_pantry() -> CardDefinition {
    CardDefinition {
        name: "Stocking the Pantry",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Supply,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_cost: Some((CounterType::Supply, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// War Squeak — {R} Enchantment — Aura. Enchant creature; when it enters, a
/// target creature an opponent controls can't block this turn. Enchanted
/// creature gets +1/+1 and has haste.
pub fn war_squeak() -> CardDefinition {
    CardDefinition {
        name: "War Squeak",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Haste],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByOpponent) },
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Tangle Tumbler — {3} Artifact — Vehicle 6/6. Vigilance. {3}, {T}: Put a +1/+1
/// counter on target creature. Tap two untapped tokens you control: This Vehicle
/// becomes an artifact creature until end of turn.
pub fn tangle_tumbler() -> CardDefinition {
    CardDefinition {
        name: "Tangle Tumbler",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_n_filter: Some((R::IsToken.and(R::ControlledByYou), 2)),
                effect: Effect::AnimateAsCreature { what: Selector::This, duration: Duration::EndOfTurn },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Bonecache Overseer — {B} 1/1 Squirrel Warlock. {T}, Pay 1 life: Draw a card.
/// Activate only if three or more cards left your graveyard this turn. (The
/// printed "or sacrificed a Food" alternative is not modeled.)
pub fn bonecache_overseer() -> CardDefinition {
    CardDefinition {
        name: "Bonecache Overseer",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            condition: Some(Predicate::CardsLeftGraveyardThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(3),
            }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
