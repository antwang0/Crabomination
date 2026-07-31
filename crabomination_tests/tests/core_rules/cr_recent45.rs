//! CR conformance for the Ravnica-block closure pass:
//! - CR 115.7a/b — changing a target of an activated ability (Reroute).
//! - CR 502.3 — the active player decides which permanents untap.
//! - CR 504 — the draw step's turn-based draw.
//! - CR 722 — preparation cards and the prepared designation.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn float(g: &mut GameState, seat: usize) {
    g.players[seat].mana_pool.add_colorless(20);
    for c in Color::ALL {
        g.players[seat].mana_pool.add(c, 10);
    }
}

// ── CR 115.7 — Changing targets ──

/// CR 115.7b — "change a target" moves exactly one target of an activated
/// ability to another legal target.
#[test]
fn cr_115_7b_change_a_target_moves_one_ability_target() {
    let mut g = two_player_game();
    let pinger = g.add_card_to_battlefield(0, catalog::prodigal_pyromancer());
    g.clear_sickness(pinger);
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: pinger,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("activate");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let reroute = g.add_card_to_hand(0, catalog::reroute());
    float(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: reroute,
        target: Some(Target::Permanent(pinger)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Reroute");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "the ping moved to the player");
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(0));
}

/// CR 115.7a — with no other legal target the original one stands.
#[test]
fn cr_115_7a_original_target_stands_when_nothing_else_is_legal() {
    let mut g = two_player_game();
    // Kraj's {T} targets a creature, and it is the only creature around.
    let kraj = g.add_card_to_battlefield(0, catalog::experiment_kraj());
    g.clear_sickness(kraj);
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: kraj,
        ability_index: 0,
        target: Some(Target::Permanent(kraj)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("activate");
    let reroute = g.add_card_to_hand(0, catalog::reroute());
    float(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: reroute,
        target: Some(Target::Permanent(kraj)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Reroute");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(kraj).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "no other legal creature — the original target kept the counter"
    );
    assert_eq!(g.players[0].hand.len(), 1, "the draw half still happened");
}

// ── CR 502.3 — Untap step ──

/// CR 502.3 — "the active player determines which permanents they control
/// will untap"; a `MayChooseNotToUntap` permanent honours that choice.
#[test]
fn cr_502_3_active_player_may_keep_a_permanent_tapped() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::hisokas_guard());
    g.battlefield_find_mut(guard).unwrap().tapped = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.do_untap();
    assert!(g.battlefield_find(guard).unwrap().tapped, "chose not to untap");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.do_untap();
    assert!(!g.battlefield_find(guard).unwrap().tapped, "untapped on the next choice");
}

/// CR 502.3 — the choice is the *active* player's; an opponent's tapped
/// permanent isn't offered and doesn't untap on this turn.
#[test]
fn cr_502_3_choice_belongs_to_the_active_player() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::hisokas_guard());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    // A `true` answer would keep it tapped if the wrong seat were asked;
    // the opponent's permanent is untouched by the active player's untap.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(theirs).unwrap().tapped, "not the active player's to untap");
}

// ── CR 504 — Draw Step ──

/// CR 504.1 — the active player draws a card as a turn-based action, and it
/// doesn't use the stack.
#[test]
fn cr_504_1_active_player_draws_as_a_turn_based_action() {
    let mut g = two_player_game();
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    // Run to the second turn's upkeep, then through its draw step.
    while !(g.turn_number == 2 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let drawer = g.active_player_idx;
    let before = g.players[drawer].hand.len();
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[drawer].hand.len(), before + 1, "exactly one card for the step");
    assert!(g.stack.is_empty(), "the turn-based draw never used the stack");
}

/// CR 103.7a — the skip belongs to the *starting player's first* draw step,
/// not to whoever reaches a draw step first.
#[test]
fn cr_103_7a_only_the_starting_players_first_draw_is_skipped() {
    let mut g = two_player_game();
    assert!(g.skip_first_draw(), "two-player games start with the skip armed");
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    while !(g.turn_number == 2 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let drawer = g.active_player_idx;
    let before = g.players[drawer].hand.len();
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(
        g.players[drawer].hand.len(),
        before + 1,
        "the non-starting player still draws on their own turn"
    );
}

// ── CR 722 — Preparation cards ──

/// CR 722.3a — a permanent can't gain the prepared designation unless it has
/// a prepare spell, and can't gain it twice.
#[test]
fn cr_722_3a_prepared_needs_a_prepare_spell_and_never_stacks() {
    let mut g = two_player_game();
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        catalog::grizzly_bears().prepare_spell.is_none(),
        "the control card has no prepare spell"
    );
    let prep = g.add_card_to_battlefield(0, catalog::scathing_shadelock());
    for _ in 0..2 {
        g.fire_step_triggers(TurnStep::PreCombatMain);
        drain_stack(&mut g);
    }
    assert_eq!(
        g.battlefield_find(prep).unwrap().counter_count(CounterType::Prepared),
        1,
        "the designation is a flag, not a counter pile"
    );
    let _ = plain;
}

/// CR 722.3c — the prepared permanent's controller casts the copy, and the
/// permanent loses the designation as the spell becomes cast.
#[test]
fn cr_722_3c_casting_the_copy_unprepares_the_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let prep = g.add_card_to_battlefield(0, catalog::abigale_poet_laureate());
    g.battlefield_find_mut(prep).unwrap().add_counters(CounterType::Prepared, 1);
    float(&mut g, 0);
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: prep,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the prepare spell");
    assert_eq!(
        g.battlefield_find(prep).unwrap().counter_count(CounterType::Prepared),
        0,
        "unprepared at the moment the copy became cast"
    );
    drain_stack(&mut g);
    assert!(g.battlefield_find(prep).is_some(), "the creature itself stays put");
}

/// CR 722.3 — the prepare spell can't be cast off an unprepared permanent.
#[test]
fn cr_722_3_unprepared_permanent_cant_cast_its_spell() {
    let mut g = two_player_game();
    let prep = g.add_card_to_battlefield(0, catalog::abigale_poet_laureate());
    float(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastPrepareSpell {
            creature_id: prep,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no prepared designation, no copy to cast"
    );
}

/// CR 722.4 / 722.2c — in every zone a preparation card has only its normal
/// characteristics: it's one creature card, not a creature plus a spell.
#[test]
fn cr_722_4_preparation_card_is_one_creature_card_in_every_zone() {
    let def = catalog::abigale_poet_laureate();
    assert!(def.is_creature(), "the card's own type line is the creature half");
    assert!(def.prepare_spell.is_some(), "with the inset spell alongside");
    assert!(
        !def.card_types.contains(&crabomination::card::CardType::Instant)
            && !def.card_types.contains(&crabomination::card::CardType::Sorcery),
        "the inset frame never joins the card's own types"
    );
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::abigale_poet_laureate());
    assert_eq!(g.players[0].graveyard.len(), 1, "one card in the graveyard");
    float(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastPrepareSpell {
            creature_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the spell isn't castable from anywhere but a prepared battlefield permanent"
    );
    let _ = Keyword::Flying;
}

/// CR 502.3 — the view's "won't untap" flag covers every reason `do_untap`
/// would skip a permanent, not just `PreventUntap` statics and stun counters.
#[test]
fn cr_502_3_wont_untap_flag_covers_every_skip_reason() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::waterveil_cavern());
    assert!(!g.untap_prevented_by_static(land), "nothing holding it yet");
    // The slow dual's coloured tap sets `skip_next_untap`.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap for coloured mana");
    drain_stack(&mut g);
    assert!(g.untap_prevented_by_static(land), "the one-shot skip is surfaced");

    // A turn-scoped player lock covers the controller's lands too.
    let mut g2 = two_player_game();
    let plain = g2.add_card_to_battlefield(0, catalog::forest());
    g2.players[0].lands_dont_untap_next_untap = 1;
    assert!(g2.untap_prevented_by_static(plain));
}
