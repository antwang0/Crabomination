//! A twenty-third wave. The headline mechanic is
//! `Keyword::AssignsCombatDamageByToughness` (CR 510.1c — "assigns combat
//! damage equal to its toughness rather than its power"): Doran, the Siege
//! Tower (all creatures), Tapestry Warden / Ancient Lumberknot (your creatures
//! with toughness > power), Bill the Pony (a sacrifice-a-Food temporary grant).
//! Plus Thrumming Hivepool (Affinity for Slivers) and a clutch of DSK staples
//! on existing primitives. Tests in `crabomination/src/tests/recent23.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, etb, on_dies, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, cost, g, generic, r, u, w};

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

/// Bedhead Beastie — {4}{R}{R} 5/6 Beast with menace and Mountaincycling {2}.
pub fn bedhead_beastie() -> CardDefinition {
    CardDefinition {
        name: "Bedhead Beastie",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![
            Keyword::Menace,
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                SelectionRequirement::HasLandType(LandType::Mountain),
            ))),
        ],
        ..Default::default()
    }
}

/// Daggermaw Megalodon — {4}{U}{U} 5/7 Shark with vigilance and Islandcycling {2}.
pub fn daggermaw_megalodon() -> CardDefinition {
    CardDefinition {
        name: "Daggermaw Megalodon",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shark], ..Default::default() },
        power: 5,
        toughness: 7,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                SelectionRequirement::HasLandType(LandType::Island),
            ))),
        ],
        ..Default::default()
    }
}

/// Boilerbilges Ripper — {4}{R} 4/4 Human Assassin. When it enters, you may
/// sacrifice another creature or enchantment; if you do, it deals 2 damage to
/// any target.
pub fn boilerbilges_ripper() -> CardDefinition {
    CardDefinition {
        name: "Boilerbilges Ripper",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another creature or enchantment? (deal 2 to any target)".into(),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Enchantment)
                .and(SelectionRequirement::OtherThanSource),
            count: Value::Const(1),
            then: Box::new(deal(2, target_any())),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Bashful Beastie — {4}{G} 5/4 Beast. When it dies, manifest dread.
pub fn bashful_beastie() -> CardDefinition {
    CardDefinition {
        name: "Bashful Beastie",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![on_dies(Effect::ManifestDread { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Bear Trap — {1} Artifact with flash. {3}, {T}, Sacrifice this: it deals 3
/// damage to target creature.
pub fn bear_trap() -> CardDefinition {
    CardDefinition {
        name: "Bear Trap",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_cost: true,
            effect: deal(3, target_filtered(SelectionRequirement::Creature)),
            ..Default::default()
        }],
        ..Default::default()
    }
}
