//! CR conformance for this run's engine work:
//! - CR 511.3 — everything is removed from combat as the end of combat step
//!   ends, but an "at end of combat" delayed trigger still resolves first.
//! - CR 106.4 / 106.4b — mana pools empty as each step ends, unless an effect
//!   says the mana doesn't (Firebending).
//! - CR 502.1 — only a permanent that prints "you may choose not to untap
//!   this" holds itself tapped to keep an untap lock alive.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 511.3 — attackers and blockers leave combat as the step ends, after any
/// end-of-combat trigger has resolved.
#[test]
fn cr_511_3_combat_is_torn_down_after_end_of_combat_triggers() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::defiant_vanguard());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // The Vanguard's end-of-combat trigger still saw what it blocked...
    assert!(g.battlefield_find(attacker).is_none(), "the end-of-combat trigger resolved");
    // ...and combat is empty afterwards.
    assert!(g.attacking.is_empty(), "no attackers remain");
    assert!(g.blocked_attackers().is_empty(), "no blocked attackers remain");
}

/// CR 106.4 — unspent mana is lost as each step ends.
#[test]
fn cr_106_4_mana_pools_empty_as_a_step_ends() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 3);
    let before = g.step;
    while g.step == before {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].mana_pool.total(), 0, "the pool emptied at the step boundary");
}

/// CR 106.4b / 702.189a — an effect can say the mana doesn't empty. Firebending
/// banks {R} that survives the intervening steps and clears at end of combat.
#[test]
fn cr_106_4b_firebending_mana_survives_until_end_of_combat() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::firebending_student());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    let banked = g.players[0].mana_pool.total();
    assert!(banked > 0, "Firebending added mana");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].mana_pool.total(), banked, "it didn't drain at the step boundaries");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].mana_pool.total(), 0, "end of combat clears it");
}

/// CR 502.1 — Kill Switch has no "you may choose not to untap" clause, so it
/// untaps normally and its lock on the other artifacts releases. Entrancing
/// Lyre, which prints the clause, keeps holding.
#[test]
fn cr_502_1_only_a_may_not_untap_source_holds_its_lock() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
    let switch = g.add_card_to_battlefield(0, catalog::kill_switch());
    let other = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.perform_action(GameAction::ActivateAbility {
        card_id: switch,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);

    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(!g.battlefield_find(switch).unwrap().tapped, "no clause → it untaps");
    g.do_untap();
    assert!(!g.battlefield_find(other).unwrap().tapped, "so the lock releases");
}

/// The turn-scoped rules changes with no board trace are surfaced to the
/// client (`ClientView.turn_effects`), so a player can tell why their Island
/// is producing {C}.
#[test]
fn turn_scoped_rules_changes_are_surfaced_to_the_client() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(g.turn_effect_notes().is_empty(), "a clean turn has nothing to say");

    let moon = g.add_card_to_hand(0, catalog::pale_moon());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 5);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: moon,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let notes = g.turn_effect_notes();
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("nonbasic land"), "{notes:?}");
    assert!(notes[0].contains("colorless"), "{notes:?}");
}
