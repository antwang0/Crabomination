//! Commander: 2026-10-19 (`016Xh16H`) — Thelon of Havenwood's spore Fungi.
//! Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic};
use std::sync::Arc;

fn fungus(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus],
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn is_fungus() -> R {
    R::HasCreatureType(CreatureType::Fungus)
}

/// "At the beginning of your upkeep, put a spore counter on this creature."
fn spore_upkeep() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Spore, amount: Value::ONE },
    }
}

/// "Remove three spore counters from this creature: [effect]."
fn three_spores(effect: Effect) -> ActivatedAbility {
    ActivatedAbility { remove_counter_cost: Some((CounterType::Spore, 3)), effect, ..Default::default() }
}

/// Thelon of Havenwood — {G}{G} 2/2 legendary Elf Druid. Each Fungus you
/// control gets +1/+1 per spore counter on it; {B}{G}, exile a Fungus card
/// from a graveyard: a spore counter on each Fungus.
pub fn thelon_of_havenwood() -> CardDefinition {
    CardDefinition {
        name: "Thelon of Havenwood",
        cost: cost(&[g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Druid], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Each Fungus creature gets +1/+1 for each spore counter on it.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: R::Creature.and(is_fungus()),
                count_filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_graveyard: false,
                exclude_self: false,
                per_own_color: false,
                per_own_counter: Some(CounterType::Spore),
                per_own_pips: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), g()]),
            exile_other_filter: Some((is_fungus(), 1)),
            exile_other_any_graveyard: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(is_fungus()),
                kind: CounterType::Spore,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Myconid Spore Tender — {3}{G} 4/1 Fungus. ETB destroy up to one target
/// artifact or enchantment.
pub fn myconid_spore_tender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Artifact.or(R::Enchantment),
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        })],
        ..fungus("Myconid Spore Tender", cost(&[generic(3), g()]), 4, 1)
    }
}

/// Spore Flower — {G}{G} 0/1 Fungus. Spores each upkeep; three of them: a
/// fog.
pub fn spore_flower() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spore_upkeep()],
        activated_abilities: vec![three_spores(Effect::PreventAllCombatDamageThisTurn)],
        ..fungus("Spore Flower", cost(&[g(), g()]), 0, 1)
    }
}

/// Thallid Devourer — {1}{G}{G} 2/2 Fungus. Spores each upkeep; three of
/// them: a 1/1 Saproling; sacrifice a Saproling: +1/+2 until end of turn.
pub fn thallid_devourer() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![spore_upkeep()],
        activated_abilities: vec![
            three_spores(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(saproling) }),
            ActivatedAbility {
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Saproling), 1)),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..fungus("Thallid Devourer", cost(&[generic(1), g(), g()]), 2, 2)
    }
}

/// Thallid Soothsayer — {3}{B} 2/3 Fungus. {2}, sacrifice a creature: draw.
pub fn thallid_soothsayer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..fungus("Thallid Soothsayer", cost(&[generic(3), b()]), 2, 3)
    }
}

/// Thorn Thallid — {1}{G}{G} 2/2 Fungus. Spores each upkeep; three of them:
/// 1 damage to any target.
pub fn thorn_thallid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spore_upkeep()],
        activated_abilities: vec![three_spores(Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker).or(R::Player)),
            amount: Value::ONE,
        })],
        ..fungus("Thorn Thallid", cost(&[generic(1), g(), g()]), 2, 2)
    }
}

/// Fungal Bloom — {G}{G} enchantment. {G}{G}: a spore counter on target
/// Fungus.
pub fn fungal_bloom() -> CardDefinition {
    CardDefinition {
        name: "Fungal Bloom",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            effect: Effect::AddCounter { what: target_filtered(is_fungus()), kind: CounterType::Spore, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
