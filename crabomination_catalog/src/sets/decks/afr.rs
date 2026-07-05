//! Adventures in the Forgotten Realms — venture-into-the-dungeon cards
//! (CR 309 / 701.49, `base::dungeons`). Tests in `tests/afr.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, generic, u, w};

/// Shortcut Seeker — {3}{U} Human Rogue 2/5. Combat damage to a player:
/// venture into the dungeon.
pub fn shortcut_seeker() -> CardDefinition {
    CardDefinition {
        name: "Shortcut Seeker",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Venture,
        }],
        ..Default::default()
    }
}

/// Cloister Gargoyle — {2}{W} Artifact Creature — Gargoyle 0/4. ETB: venture.
/// While you've completed a dungeon it gets +3/+0 and has flying.
pub fn cloister_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Cloister Gargoyle",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Venture)],
        static_abilities: vec![StaticAbility {
            description: "As long as you've completed a dungeon, this creature gets +3/+0 and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::Const(1)),
                power: 3,
                toughness: 0,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..Default::default()
    }
}

/// Dungeon Crawler — {B} Zombie 2/1. Enters tapped. Whenever you complete a
/// dungeon, you may return this card from your graveyard to your hand.
pub fn dungeon_crawler() -> CardDefinition {
    CardDefinition {
        name: "Dungeon Crawler",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DungeonCompleted, EventScope::FromYourGraveyard),
            effect: Effect::MayDo {
                description: "Return Dungeon Crawler from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}
