//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Henry Wu, InGen
//! Geneticist's exploit cluster. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{exploit, mint_token, mint_treasures, target_filtered};
use crate::effect::{Effect, KeywordAct, Predicate, Selector, Value};
use crate::mana::{Color, ManaCost, b, cost, g, generic, u};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn on_exploit(act: KeywordAct, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Performed(act), EventScope::YourControl), effect }
}

/// Henry Wu, InGen Geneticist — {B}{G}{U} 1/4 legendary Human Scientist.
/// Henry Wu and other Humans you control have exploit. Whenever a creature
/// you control exploits a non-Human creature, draw a card; if it had power 3
/// or greater, create a Treasure.
pub fn henry_wu_ingen_geneticist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Henry Wu and other Human creatures you control have exploit.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou),
                ability: Box::new(exploit(Effect::Noop)),
            },
        }],
        triggered_abilities: vec![on_exploit(
            KeywordAct::ExploitedNonHuman,
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(3)),
                    then: Box::new(mint_treasures(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )],
        ..creature(
            "Henry Wu, InGen Geneticist",
            cost(&[b(), g(), u()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            1,
            4,
        )
    }
}

/// Silumgar Sorcerer — {1}{U}{U} 2/1 Human Wizard Sorcerer. Flash, flying.
/// Exploit; when it exploits a creature, counter target creature spell.
pub fn silumgar_sorcerer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![exploit(Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))),
        })],
        ..creature(
            "Silumgar Sorcerer",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer],
            2,
            1,
        )
    }
}

/// Skull Skaab — {U}{B} 2/2 Zombie. Exploit. Whenever a creature you control
/// exploits a nontoken creature, create a 2/2 black Zombie.
pub fn skull_skaab() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            exploit(Effect::Noop),
            on_exploit(KeywordAct::ExploitedNontoken, mint_token(zombie, 1)),
        ],
        ..creature("Skull Skaab", cost(&[u(), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}
