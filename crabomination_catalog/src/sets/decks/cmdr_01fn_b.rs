//! Commander: Dromar, the Banisher's color hosers (`01FnAVVR`). Tests in
//! `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, Selector};
use crate::mana::{Color, ManaCost, b, cost, generic, u, w};

fn knight(name: &'static str, mana: ManaCost, p: i32, t: i32, ability: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: p,
        toughness: t,
        activated_abilities: vec![ability],
        ..Default::default()
    }
}

fn counter_color(color: Color) -> Effect {
    Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::HasColor(color))) }
}

/// Northern Paladin — {2}{W}{W} 3/3 Human Knight. {W}{W}, {T}: destroy
/// target black permanent.
pub fn northern_paladin() -> CardDefinition {
    knight(
        "Northern Paladin",
        cost(&[generic(2), w(), w()]),
        3,
        3,
        ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Permanent.and(R::HasColor(Color::Black))) },
            ..Default::default()
        },
    )
}

/// Order of the Sacred Torch — {1}{W}{W} 2/2 Human Knight. {T}, pay 1 life:
/// counter target black spell.
pub fn order_of_the_sacred_torch() -> CardDefinition {
    knight(
        "Order of the Sacred Torch",
        cost(&[generic(1), w(), w()]),
        2,
        2,
        ActivatedAbility { tap_cost: true, life_cost: 1, effect: counter_color(Color::Black), ..Default::default() },
    )
}

/// Stromgald Cabal — {1}{B}{B} 2/2 Human Knight. {T}, pay 1 life: counter
/// target white spell.
pub fn stromgald_cabal() -> CardDefinition {
    knight(
        "Stromgald Cabal",
        cost(&[generic(1), b(), b()]),
        2,
        2,
        ActivatedAbility { tap_cost: true, life_cost: 1, effect: counter_color(Color::White), ..Default::default() },
    )
}

/// Deathgrip — {B}{B} enchantment. {B}{B}: counter target green spell.
pub fn deathgrip() -> CardDefinition {
    CardDefinition {
        name: "Deathgrip",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            effect: counter_color(Color::Green),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flooded Woodlands — {2}{U}{B} enchantment. Each green creature carries
/// "can't attack unless its controller sacrifices a land" (CR 508.1g), so
/// the cost is one land per green attacker, paid as attackers are declared.
pub fn flooded_woodlands() -> CardDefinition {
    CardDefinition {
        name: "Flooded Woodlands",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Green creatures can't attack unless their controller sacrifices a land of their choice for each green creature they control that's attacking.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green))),
                keyword: Keyword::AttackCostSacrifice(Box::new(R::Land), 1),
            },
        }],
        ..Default::default()
    }
}

/// Nature's Ruin — {2}{B} sorcery. Destroy all green creatures.
pub fn natures_ruin() -> CardDefinition {
    CardDefinition {
        name: "Nature's Ruin",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green))) },
        ..Default::default()
    }
}
