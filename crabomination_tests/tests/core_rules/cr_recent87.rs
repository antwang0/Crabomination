//! CR conformance for this run:
//! - CR 708.10 — a face-down permanent that becomes a copy keeps its
//!   face-down characteristics; the copy is what it turns up as.
//! - CR 603.7c — "at the beginning of the next turn's upkeep" waits for a
//!   later turn than the one the watcher was registered on.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 708.10 — a copy effect on a face-down permanent rewrites what it turns
/// up as, not the 2/2 body everyone can see.
#[test]
fn cr_708_10_a_copy_onto_a_face_down_permanent_changes_only_its_copiable_values() {
    let mut g = two_player_game();
    let hidden = g.add_card_to_hand(0, catalog::fugitive_codebreaker());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: hidden }).expect("cast face down");
    drain_stack(&mut g);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(bears)),
        0,
        0,
    );
    g.resolve_effect(
        &crabomination::effect::Effect::BecomeCopyOf {
            what: crabomination::effect::Selector::EachPermanent(
                crabomination::card::SelectionRequirement::FaceDown,
            ),
            source: crabomination::effect::Selector::Target(0),
            extra_creature_types: vec![],
            keep_own_triggered: false,
            keep_own_activated: false,
        },
        &ctx,
    )
    .expect("copy onto the morph");
    let cp = g.computed_permanent(hidden).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "still a 2/2 face down");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::TurnFaceUp { card_id: hidden }).expect("turn up");
    assert_eq!(
        g.battlefield_find(hidden).unwrap().definition.name,
        "Grizzly Bears",
        "it turns up as the copy",
    );
}

/// CR 603.7c — a delayed trigger set for "the next turn's upkeep" skips the
/// upkeep of the turn it was created on.
#[test]
fn cr_603_7c_next_turn_upkeep_skips_the_current_turn() {
    let mut g = two_player_game();
    g.step = TurnStep::Upkeep;
    let turn = g.turn_number;
    g.delayed_triggers.push(crabomination::game::types::DelayedTrigger {
        controller: 0,
        source: crabomination::card::CardId(0),
        kind: crabomination::game::types::DelayedKind::NextUpkeep { after_turn: turn },
        effect: crabomination::effect::Effect::Draw {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::ONE,
        },
        target: None,
        bound_token: None,
        bound_subject: None,
        fires_once: true,
        expires_after_turn: None,
    });
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "this turn's upkeep is skipped");
    assert_eq!(g.delayed_triggers.len(), 1, "the watcher is still armed");
}
