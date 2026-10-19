//! Commander: 2026-10-19 (`01DJHkWk`) — Pharaoh Rama-Tut's madness pair and
//! Umara Wizard // Umara Skyfalls.
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Predicate, Selector, Value};
use crate::mana::{Color, cost, generic, u, x};

/// Umara Wizard // Umara Skyfalls — {4}{U} 4/3 Merfolk Wizard; whenever you
/// cast an instant, sorcery or Wizard spell, it gains flying until end of
/// turn. Back (modal, CR 712.12): Umara Skyfalls, a land that enters tapped
/// and taps for {U}.
pub fn umara_wizard() -> CardDefinition {
    let skyfalls = CardDefinition {
        name: "Umara Skyfalls",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![crate::sets::tap_add(Color::Blue)],
        ..Default::default()
    };
    CardDefinition {
        name: "Umara Wizard",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .or(R::HasCreatureType(CreatureType::Wizard)),
                ),
            ),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        }],
        back_face: Some(Box::new(skyfalls)),
        ..Default::default()
    }
}

/// Broken Concentration — {1}{U}{U} instant. Counter target spell. Madness
/// {3}{U}.
pub fn broken_concentration() -> CardDefinition {
    CardDefinition {
        name: "Broken Concentration",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Madness(cost(&[generic(3), u()]))],
        effect: Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack),
        },
        ..Default::default()
    }
}

/// Welcome to the Fold — {2}{U}{U} sorcery. Madness {X}{U}{U}. Gain control
/// of target creature if its toughness is 2 or less — X or less instead if
/// its madness cost was paid (CR 702.35).
pub fn welcome_to_the_fold() -> CardDefinition {
    let toughness = || Value::ToughnessOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Welcome to the Fold",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Madness(cost(&[x(), u(), u()]))],
        effect: Effect::If {
            cond: Predicate::ValueAtMost(
                toughness(),
                Value::IfPred {
                    pred: Box::new(Predicate::SpellWasMadness),
                    then: Box::new(Value::XFromCost),
                    else_: Box::new(Value::Const(2)),
                },
            ),
            then: Box::new(Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::Permanent,
            }),
            else_: Box::new(Effect::Noop),
        },
        ..Default::default()
    }
}
