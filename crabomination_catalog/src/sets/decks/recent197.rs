//! OTJ gap batch: Seize the Secrets (new `self_cost_reduction_if_crime`
//! primitive), Take for a Ride (Threaten), Silver Deputy (ETB dig + pump).
//! Tests in `tests/recent197.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, LandType,
    SelectionRequirement as R, Subtypes,
};
use crate::card::{EventKind, EventScope, EventSpec, Keyword, TriggeredAbility};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, generic, r, u};

/// Seize the Secrets — {2}{U} Sorcery. Costs {1} less if you've committed a
/// crime this turn. Draw two cards.
pub fn seize_the_secrets() -> CardDefinition {
    CardDefinition {
        name: "Seize the Secrets",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_crime: Some(1),
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Take for a Ride — {2}{R} Sorcery. Gain control of target creature until end
/// of turn, untap it, and it gains haste. (The "flash while you've committed a
/// crime" rider is omitted — no conditional-flash primitive.)
pub fn take_for_a_ride() -> CardDefinition {
    CardDefinition {
        name: "Take for a Ride",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Silver Deputy — {2} 1/2 Mercenary artifact creature. ETB: search for a basic
/// land or Desert card and put it on top of your library. {T}, sorcery-speed:
/// target creature you control gets +1/+0 until end of turn.
pub fn silver_deputy() -> CardDefinition {
    CardDefinition {
        name: "Silver Deputy",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.or(R::HasLandType(LandType::Desert)),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
