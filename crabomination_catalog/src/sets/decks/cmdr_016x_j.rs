//! Commander: 2026-10-19 (`016Xh16H`) — Zhou Yu, Chief Commander's
//! Island-gated navy. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, LandType, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Selector, Value};
use crate::mana::{ManaCost, cost, generic, u};

fn island_gated(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(LandType::Island)))],
        ..Default::default()
    }
}

/// Zhou Yu, Chief Commander — {5}{U}{U} 8/8 legendary Human Soldier; can't
/// attack unless defending player controls an Island.
pub fn zhou_yu_chief_commander() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..island_gated(
            "Zhou Yu, Chief Commander",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            8,
            8,
        )
    }
}

/// Wu Warship — {2}{U} 3/3 Human Soldier; can't attack unless defending
/// player controls an Island.
pub fn wu_warship() -> CardDefinition {
    island_gated("Wu Warship", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 3)
}

/// Floodchaser — {5}{U} 0/0 Elemental, enters with six +1/+1 counters; can't
/// attack unless defending player controls an Island; {U}, remove a +1/+1
/// counter: target land becomes an Island until end of turn (modelled as
/// gaining the type — what the attack gate reads).
pub fn floodchaser() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(6))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::GainLandType {
                what: target_filtered(R::Land),
                land_type: LandType::Island,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..island_gated("Floodchaser", cost(&[generic(5), u()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Phantasmal Terrain — {U}{U} Aura, enchant land. As it enters, choose a
/// basic land type; the enchanted land is that type (CR 305.7 — it loses
/// its other land types and abilities).
pub fn phantasmal_terrain() -> CardDefinition {
    CardDefinition {
        name: "Phantasmal Terrain",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        as_enters_effect: Some(Effect::ChooseBasicLandTypeForSource),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land is the chosen type.",
            effect: StaticEffect::LandTypeChangerChosen {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                replace: true,
            },
        }],
        ..Default::default()
    }
}

/// Magical Hack — {U} instant. Rewrite one basic land type to another in
/// target permanent's text, indefinitely (a spell target isn't offered).
pub fn magical_hack() -> CardDefinition {
    CardDefinition {
        name: "Magical Hack",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ReplaceBasicLandType { what: target_filtered(R::Permanent), duration: Duration::Permanent },
        ..Default::default()
    }
}
