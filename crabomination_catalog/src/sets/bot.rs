//! Transformers (BOT) — the Universes Beyond companion to The Brothers' War.
//! Every card is a two-faced Robot / Vehicle: More Than Meets the Eye
//! (CR 702.162 / 701.28 — cast *converted* for the alt cost, entering back-face
//! up) plus Living metal on the Vehicle side (CR 702.161).

use crate::card::{
    AlternativeCost, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{ManaCost, cost, generic, r};

/// A Transformers front face: a legendary artifact creature Robot whose
/// More Than Meets the Eye cost casts it converted onto its Vehicle back.
fn robot(
    name: &'static str,
    c: ManaCost,
    mtmte: ManaCost,
    power: i32,
    toughness: i32,
    back: CardDefinition,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot],
            ..Default::default()
        },
        power,
        toughness,
        alternative_cost: Some(AlternativeCost {
            mana_cost: mtmte,
            converted: true,
            ..Default::default()
        }),
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// A Transformers back face: a legendary artifact Vehicle with living metal.
fn vehicle(name: &'static str, power: i32, toughness: i32) -> CardDefinition {
    CardDefinition {
        name,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power,
        toughness,
        keywords: vec![Keyword::LivingMetal],
        ..Default::default()
    }
}

/// Slicer, High-Speed Antagonist — the Vehicle back. Converts back after it
/// connects.
pub fn slicer_high_speed_antagonist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::LivingMetal, Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Transform { what: Selector::This }),
            },
        }],
        ..vehicle("Slicer, High-Speed Antagonist", 3, 2)
    }
}

/// Slicer, Hired Muscle — {4}{R} 3/4. Each opponent's upkeep, rent it out or
/// it converts. (The printed "it can't be sacrificed this turn" rider is
/// dropped — there is no sacrifice lock.)
pub fn slicer_hired_muscle() -> CardDefinition {
    let rent = Effect::Seq(vec![
        Effect::GainControl {
            what: Selector::This,
            to: Some(PlayerRef::ActivePlayer),
            duration: Duration::EndOfTurn,
        },
        Effect::Untap { what: Selector::This, up_to: None },
        Effect::Goad { what: Selector::This },
    ]);
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::MayDoElse {
                description: "Hand Slicer to that player until end of turn?".to_string(),
                body: Box::new(rent),
                else_: Box::new(Effect::Transform { what: Selector::This }),
            },
        }],
        ..robot(
            "Slicer, Hired Muscle",
            cost(&[generic(4), r()]),
            cost(&[generic(2), r()]),
            3,
            4,
            slicer_high_speed_antagonist(),
        )
    }
}
