//! Khans of Tarkir block — Dash (CR 702.110) creatures.
//!
//! Dash rides the `AlternativeCost { dash: true }` path (`shortcut::dash`):
//! cast for the dash cost, the creature enters with haste and returns to its
//! owner's hand at the next end step. Each card here is built on that plus
//! existing primitives — no per-card dash plumbing.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement,
    Subtypes, Supertype, TokenDefinition,
};
use crate::effect::shortcut::{dash, etb, on_attack, target_filtered};
use crate::effect::{Duration, PlayerRef, Value};
use crate::mana::{b, cost, generic, r};

/// Screamreach Brawler — {2}{R} 3/3 Orc Berserker. Dash {1}{R}.
pub fn screamreach_brawler() -> CardDefinition {
    CardDefinition {
        name: "Screamreach Brawler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Mardu Scout — {2}{R} 3/1 Human Warrior. Dash {R}.
pub fn mardu_scout() -> CardDefinition {
    CardDefinition {
        name: "Mardu Scout",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        alternative_cost: Some(dash(cost(&[r()]))),
        ..Default::default()
    }
}

/// Zurgo Bellstriker — {R} 2/2 Legendary Goblin Warrior. Dash {1}{R}.
/// (The "can't block creatures with power 2 or greater" rider collapses —
/// no power-gated block restriction primitive.)
pub fn zurgo_bellstriker() -> CardDefinition {
    CardDefinition {
        name: "Zurgo Bellstriker",
        cost: cost(&[r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Goblin Heelcutter — {3}{R} 3/2 Goblin Berserker. Whenever this attacks,
/// target creature can't block this turn. Dash {1}{R}.
pub fn goblin_heelcutter() -> CardDefinition {
    CardDefinition {
        name: "Goblin Heelcutter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Ponyback Brigade — {3}{B}{R} 2/2 Goblin. When this enters, create three
/// 1/1 red Goblin creature tokens. Dash {4}{B}{R}.
pub fn ponyback_brigade() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Ponyback Brigade",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: goblin,
        })],
        alternative_cost: Some(dash(cost(&[generic(4), b(), r()]))),
        ..Default::default()
    }
}

/// Every KTK factory, for snapshot name→factory registration.
pub fn all_ktk_card_factories() -> &'static [crate::CardFactory] {
    &[
        screamreach_brawler,
        mardu_scout,
        zurgo_bellstriker,
        goblin_heelcutter,
        ponyback_brigade,
    ]
}
