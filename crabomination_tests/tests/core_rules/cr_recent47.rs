//! CR conformance for this run's sweep:
//! - CR 406 — the exile zone.
//! - CR 501 — the beginning phase.
//! - CR 513 — the end step.
//! - CR 703 — turn-based actions.

use crabomination::catalog;
use crabomination::card::Value;
use crabomination::effect::{Effect, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{DelayedKind, DelayedTrigger, GameAction, Target, TurnStep};
use crabomination::game::*;

fn exile_ctx(controller: usize, source: CardId) -> EffectContext {
    let mut ctx = EffectContext::for_spell(controller, None, 0, 0);
    ctx.source = Some(source);
    ctx
}

// ── CR 406 — Exile ──

/// CR 406.2 — exiling puts an object into exile from whatever zone it's in;
/// the graveyard, hand and library are all valid origins.
#[test]
fn cr_406_2_exile_takes_a_card_from_any_zone() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gy = g.add_card_to_hand(0, catalog::lightning_bolt());
    let mut evs = Vec::new();
    g.discard_card(0, gy, &mut evs);
    let hand = g.add_card_to_hand(0, catalog::lightning_bolt());
    let lib = g.add_card_to_library(0, catalog::lightning_bolt());
    let ctx = exile_ctx(0, src);
    for id in [gy, hand, lib] {
        g.move_card_to(id, &ZoneDest::Exile, &ctx, &mut evs);
    }
    for id in [gy, hand, lib] {
        assert!(g.exile.iter().any(|c| c.id == id), "{id:?} reached exile");
    }
    assert!(g.players[0].graveyard.is_empty() && g.players[0].hand.is_empty());
}

/// CR 406.7 — an exiled object that becomes exiled again stays in exile but is
/// a new object, so the first exiler's linked ability no longer sees it.
#[test]
fn cr_406_7_re_exiling_makes_a_new_object() {
    let mut g = two_player_game();
    let first = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let second = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_hand(0, catalog::lightning_bolt());
    let mut evs = Vec::new();
    g.move_card_to(victim, &ZoneDest::Exile, &exile_ctx(0, first), &mut evs);
    g.exile.iter_mut().find(|c| c.id == victim).unwrap().exiled_with = Some(first);

    g.move_card_to(victim, &ZoneDest::Exile, &exile_ctx(0, second), &mut evs);
    let card = g.exile.iter().find(|c| c.id == victim).expect("still exiled");
    assert_eq!(card.exiled_with, None, "the first exiler's link was dropped");
    assert_eq!(g.exile.iter().filter(|c| c.id == victim).count(), 1, "no duplicate");
}

// ── CR 501 — The beginning phase ──

/// CR 501.1 — the beginning phase is untap, then upkeep, then draw.
#[test]
fn cr_501_1_beginning_phase_steps_run_in_order() {
    let mut g = two_player_game();
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    let mut seen = vec![g.step];
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        if *seen.last().unwrap() != g.step {
            seen.push(g.step);
        }
    }
    assert_eq!(
        &seen[..4],
        &[TurnStep::Untap, TurnStep::Upkeep, TurnStep::Draw, TurnStep::PreCombatMain]
    );
}

// ── CR 513 — The end step ──

/// CR 513.1 — the end step has no turn-based actions; the active player just
/// gets priority (no draw, no untap, no discard).
#[test]
fn cr_513_1_end_step_has_no_turn_based_actions() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    g.step = TurnStep::EndCombat;
    g.priority.player_with_priority = 0;
    let (hand, lib) = (g.players[0].hand.len(), g.players[0].library.len());
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].hand.len(), hand, "no draw at the end step");
    assert_eq!(g.players[0].library.len(), lib);
    assert_eq!(g.player_with_priority(), 0, "the active player gets priority");
}

/// CR 513.2 — a delayed "at the beginning of the next end step" trigger created
/// *during* the end step waits for the next turn's end step; the step doesn't
/// back up.
#[test]
fn cr_513_2_end_step_does_not_back_up_for_a_new_delayed_trigger() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::End;
    g.delayed_triggers.push(DelayedTrigger {
        source: src,
        controller: 0,
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        kind: DelayedKind::NextEndStep,
        target: None,
        fires_once: true,
        expires_after_turn: None,
        bound_token: None,
        bound_subject: None,
    });
    let hand = g.players[0].hand.len();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "this end step already passed its trigger check");
    assert_eq!(g.delayed_triggers.len(), 1, "the trigger is still queued");

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "it fires at the next end step");
}

// ── CR 703 — Turn-based actions ──

/// CR 703.4d — the draw-step draw is a turn-based action: it happens
/// immediately as the step begins, with no player holding priority for it.
#[test]
fn cr_703_4d_draw_step_draw_is_a_turn_based_action() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    // CR 103.7a — the starting player skips only their *first* draw step.
    g.turn_number = 2;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::Draw {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "the card is already drawn on arrival");
}

/// CR 703.4q — unspent mana empties as each step ends (CR 500.5).
#[test]
fn cr_703_4q_mana_empties_as_a_step_ends() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("opponent passes");
    assert_eq!(g.step, TurnStep::BeginCombat);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the pool emptied on the step change");
}

/// CR 703.4n — the cleanup discard is a turn-based action, trimming the active
/// player's hand to their maximum hand size.
#[test]
fn cr_703_4n_cleanup_discards_down_to_maximum_hand_size() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    for _ in 0..9 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("opponent passes");
    assert_eq!(g.players[0].hand.len(), 7, "trimmed to the maximum hand size");
    assert_eq!(g.players[0].graveyard.len(), 2, "the excess went to the graveyard");
}

/// CR 703.4p — "until end of turn" effects end during cleanup, after the
/// discard.
#[test]
fn cr_703_4p_cleanup_ends_until_end_of_turn_effects() {
    use crabomination::card::Keyword;
    use crabomination::effect::Duration;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    ctx.source = Some(src);
    g.resolve_effect(
        &Effect::GrantKeyword {
            what: Selector::Target(0),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("grant");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));

    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("opponent passes");
    assert!(
        !g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "the grant expired in cleanup"
    );
}
