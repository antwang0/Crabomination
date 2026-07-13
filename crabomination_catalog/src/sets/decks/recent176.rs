//! Aetherdrift (DFT) gap batch, continued. Cards unblocked by threading the
//! cast's X onto ETB *triggered* abilities (`CardInstance.cast_x_value`) and by
//! the multi-slot up-to-one graveyard return (`Effect::ReturnFilteredSlots`).
//! Tests in `crabomination/src/tests/recent176.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, generic, u, w, x, Color};

/// Dune Drifter — {X}{W}{B} Artifact — Vehicle 3/3, Crew 2. When it enters,
/// return target artifact or creature card with mana value X or less from your
/// graveyard to the battlefield.
pub fn dune_drifter() -> CardDefinition {
    CardDefinition {
        name: "Dune Drifter",
        cost: cost(&[x(), w(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Artifact
                    .or(R::Creature)
                    .and(R::InYourGraveyard)
                    .and(R::ManaValueAtMostXFromCost),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..Default::default()
    }
}

/// Vnwxt, Verbose Host — {1}{U} Legendary 0/4 Homunculus. Start your engines!
/// You have no maximum hand size. Max speed — if you would draw a card, draw
/// two cards instead.
pub fn vnwxt_verbose_host() -> CardDefinition {
    CardDefinition {
        name: "Vnwxt, Verbose Host",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Homunculus],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::StartYourEngines],
        static_abilities: vec![
            StaticAbility {
                description: "You have no maximum hand size.",
                effect: StaticEffect::NoMaximumHandSize,
            },
            StaticAbility {
                description: "Max speed — if you would draw a card, draw two cards instead.",
                effect: StaticEffect::ControllerDrawsDoubledIf {
                    condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                },
            },
        ],
        ..Default::default()
    }
}

fn tapped_zombie_token() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped: true,
        ..Default::default()
    }
}

/// Zahur, Glory's Past — {W}{B} Legendary 3/2 Zombie Cat Warrior. Start your
/// engines! Sacrifice another creature: Surveil 1 (once each turn). Max speed —
/// whenever a nontoken creature you control dies, create a tapped 2/2 black
/// Zombie.
pub fn zahur_glorys_past() -> CardDefinition {
    CardDefinition {
        name: "Zahur, Glory's Past",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            once_per_turn: true,
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                },
            ),
            effect: Effect::If {
                cond: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: tapped_zombie_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}
