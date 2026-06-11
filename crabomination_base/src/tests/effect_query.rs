//! Walker-consistency guard for the targeted-effect query methods
//! (CR 115.1a/601.2c). `requires_target`, `primary_target_filter`, and
//! `target_filter_for_slot` are three parallel walks over the `Effect`
//! tree; when an arm exists in one but not the others, cast-time filter
//! enforcement silently disappears for that effect (the audit's
//! "Lyev Skyknight detains the caster's own land" class). Every targeted
//! effect shape in the catalog should appear here.

use crate::card::{CounterType, Keyword, SelectionRequirement};
use crate::effect::{shortcut::etb, Duration, Effect, Selector, Value};

fn filt() -> SelectionRequirement {
    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent)
}

fn tgt() -> Selector {
    Selector::TargetFiltered { slot: 0, filter: filt() }
}

/// Every targeted effect must agree across the three walkers: it demands a
/// target, and its slot-0 filter is discoverable for cast-time validation
/// and auto-targeting.
#[test]
fn targeted_effects_carry_slot_filters() {
    let cases: Vec<Effect> = vec![
        Effect::Detain { what: tgt() },
        Effect::Goad { what: tgt() },
        Effect::Regenerate { what: tgt() },
        Effect::Transform { what: tgt() },
        Effect::LoseAllAbilities { what: tgt(), duration: Duration::EndOfTurn },
        Effect::LoseKeywordThisTurn { what: tgt(), keyword: Keyword::Flying },
        Effect::ExileUntilSourceLeaves {
            what: tgt(),
            return_to: crate::card::ExileReturnZone::Battlefield,
        },
        Effect::ExileReturnNextEndStep { what: tgt() },
        Effect::ExileWithSource { what: tgt() },
        Effect::ExileSameNameAsTarget { what: tgt() },
        Effect::GrantTriggeredAbility {
            what: tgt(),
            trigger: Box::new(etb(Effect::Noop)),
            duration: Duration::EndOfTurn,
        },
        Effect::MoveCounter {
            from: tgt(),
            to: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
        Effect::SetLoyalty { what: tgt(), value: Value::Const(3) },
        Effect::BecomeChosenColor { what: tgt(), duration: Duration::EndOfTurn },
        Effect::SkipNextUntap { what: tgt() },
        Effect::Provoke { what: tgt() },
        Effect::Suspect { what: tgt() },
        Effect::Endure { target: tgt(), n: Value::Const(2) },
        Effect::DoubleLife { who: tgt() },
        Effect::CollectEvidence {
            amount: Value::Const(3),
            then: Box::new(Effect::AddCounter {
                what: tgt(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
        Effect::Forage {
            then: Box::new(Effect::Destroy { what: tgt() }),
        },
        Effect::NameCreatureType { what: tgt() },
        Effect::CreateTokenCopyOf {
            who: crate::effect::PlayerRef::You,
            count: Value::Const(1),
            source: tgt(),
            extra_creature_types: vec![],
            override_pt: None,
            non_legendary: false,
        },
    ];
    for e in &cases {
        assert!(
            e.requires_target(),
            "requires_target() must be true for {e:?}"
        );
        assert!(
            e.target_filter_for_slot(0).is_some(),
            "target_filter_for_slot(0) missing for {e:?} — cast-time filter \
             enforcement is silently off for this effect"
        );
        assert!(
            e.primary_target_filter().is_some(),
            "primary_target_filter() missing for {e:?} — the auto-targeter \
             treats this as 'any target'"
        );
    }
}
