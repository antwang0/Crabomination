//! A twenty-third wave built around `Keyword::AssignsCombatDamageByToughness`
//! (CR 510.1c — "assigns combat damage equal to its toughness rather than its
//! power"): Doran, the Siege Tower (all creatures, unconditional), Tapestry
//! Warden (your creatures with toughness > power), and Bill the Pony (a
//! sacrifice-a-Food temporary grant). Tests in
//! `crabomination/src/tests/recent23.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, cost, g, generic, w};

/// Doran, the Siege Tower — {W}{B}{G} 0/5 legendary Treefolk Shaman. Each
/// creature assigns combat damage equal to its toughness rather than its power.
pub fn doran_the_siege_tower() -> CardDefinition {
    CardDefinition {
        name: "Doran, the Siege Tower",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Shaman],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Each creature assigns combat damage equal to its toughness",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(SelectionRequirement::Creature),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..Default::default()
    }
}

/// Tapestry Warden — {3}{G} 3/4 artifact Robot Soldier with vigilance. Each
/// creature you control with toughness greater than its power assigns combat
/// damage equal to its toughness rather than its power. (The "stations using
/// toughness" half is dropped.)
pub fn tapestry_warden() -> CardDefinition {
    CardDefinition {
        name: "Tapestry Warden",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Your creatures with toughness > power assign damage by toughness",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::ToughnessGreaterThanPower),
                ),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..Default::default()
    }
}

/// Ancient Lumberknot — {2}{B}{G} 1/4 Treefolk. Each creature you control with
/// toughness greater than its power assigns combat damage equal to its
/// toughness rather than its power.
pub fn ancient_lumberknot() -> CardDefinition {
    CardDefinition {
        name: "Ancient Lumberknot",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Your creatures with toughness > power assign damage by toughness",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::ToughnessGreaterThanPower),
                ),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..Default::default()
    }
}

/// Thrumming Hivepool — {6} Artifact with Affinity for Slivers. Slivers you
/// control have double strike and haste. At the beginning of your upkeep,
/// create two 1/1 colorless Sliver creature tokens.
pub fn thrumming_hivepool() -> CardDefinition {
    let sliver_lord = |keyword| StaticAbility {
        description: "Slivers you control have double strike and haste",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                SelectionRequirement::ControlledByYou
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Sliver)),
            ),
            keyword,
        },
    };
    CardDefinition {
        name: "Thrumming Hivepool",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        affinity_filter: Some(
            SelectionRequirement::ControlledByYou
                .and(SelectionRequirement::HasCreatureType(CreatureType::Sliver)),
        ),
        static_abilities: vec![
            sliver_lord(Keyword::DoubleStrike),
            sliver_lord(Keyword::Haste),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Sliver".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Sliver],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Bill the Pony — {3}{W} 1/4 legendary Horse. ETB: create two Food. Sacrifice
/// a Food: until end of turn, target creature you control assigns combat damage
/// equal to its toughness rather than its power.
pub fn bill_the_pony() -> CardDefinition {
    let food = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: crabomination_base::tokens::food_token(),
    };
    CardDefinition {
        name: "Bill the Pony",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![etb(food())],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Food),
                1,
            )),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::AssignsCombatDamageByToughness,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
