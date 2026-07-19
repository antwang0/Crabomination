//! MOM/BRO/LCI gap batch — a graveyard-nuke instant, an Incubator-flipping
//! Flyer, and a descend-modal removal. All on existing primitives. Tests in
//! `tests/recent_b/recent277.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::card::{EventKind, EventScope, EventSpec, Predicate};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{b, cost, generic, r, w};

/// Calamity's Wake — {1}{W} Instant. Exile all graveyards. Players can't cast
/// noncreature spells this turn. Exile Calamity's Wake.
pub fn calamitys_wake() -> CardDefinition {
    CardDefinition {
        name: "Calamity's Wake",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileAllGraveyards { filter: None, opponents_only: false },
            Effect::CantCastNoncreatureThisTurn { who: Selector::Player(PlayerRef::EachPlayer) },
            Effect::ExileSource,
        ]),
        ..Default::default()
    }
}

/// Attentive Skywarden — {2}{W} 2/2 Phyrexian Kor. Flying. Whenever it deals
/// combat damage to a player or battle, transform up to one target Incubator
/// token you control.
pub fn attentive_skywarden() -> CardDefinition {
    CardDefinition {
        name: "Attentive Skywarden",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Kor],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::HasName("Incubator".into()).and(R::IsToken).and(R::ControlledByYou),
                effect: Box::new(Effect::Transform { what: Selector::Target(0) }),
            },
        }],
        ..Default::default()
    }
}

/// Molten Collapse — {B}{R} Sorcery. Choose one; if you descended this turn you
/// may choose both — destroy target creature or planeswalker / destroy target
/// noncreature, nonland permanent with mana value 1 or less.
pub fn molten_collapse() -> CardDefinition {
    let modes = vec![
        Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        Effect::Destroy {
            what: target_filtered(
                R::Nonland.and(R::Creature.negate()).and(R::ManaValueAtMost(1)),
            ),
        },
    ];
    CardDefinition {
        name: "Molten Collapse",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::DescendedThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::ChooseModesCast {
                modes: modes.clone(),
                min: 1,
                max: 2,
                allow_repeats: false,
            }),
            else_: Box::new(Effect::ChooseModesCast {
                modes,
                min: 1,
                max: 1,
                allow_repeats: false,
            }),
        },
        ..Default::default()
    }
}
