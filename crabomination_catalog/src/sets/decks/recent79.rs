//! Retro batch 3 — a pump-Ant, an untap-lock Elf, a make-unblockable Dwarf, a
//! haste Rampage beater, and a graveyard-artifact-hating Rogue. Tests in
//! `tests/recent79.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Selector, Value};
use crate::mana::{b, cost, g, generic, r};

/// Carrion Ants — {2}{B}{B} 0/1 Insect. {1}: this creature gets +1/+1 until end
/// of turn.
pub fn carrion_ants() -> CardDefinition {
    CardDefinition {
        name: "Carrion Ants",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 0,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elvish Hunter — {1}{G} 1/1 Elf Archer. {1}{G}, {T}: Target creature doesn't
/// untap during its controller's next untap step.
pub fn elvish_hunter() -> CardDefinition {
    CardDefinition {
        name: "Elvish Hunter",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Archer], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: Effect::SkipNextUntap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dwarven Nomad — {2}{R} 1/1 Dwarf Nomad. {T}: Target creature with power 2 or
/// less can't be blocked this turn.
pub fn dwarven_nomad() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Nomad",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Nomad], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Balduvian War-Makers — {4}{R} 3/3 Human Barbarian. Haste, rampage 1.
pub fn balduvian_war_makers() -> CardDefinition {
    CardDefinition {
        name: "Balduvian War-Makers",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Barbarian], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste, Keyword::Rampage(1)],
        ..Default::default()
    }
}

/// Grave Robbers — {1}{B}{B} 1/1 Human Rogue. {B}, {T}: Exile target artifact
/// card from a graveyard. You gain 2 life.
pub fn grave_robbers() -> CardDefinition {
    CardDefinition {
        name: "Grave Robbers",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rogue], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Artifact.and(R::InGraveyard)) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
