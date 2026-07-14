//! Functionality tests for `catalog::sets::decks::recent187`.

use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;
use crate::mana::Color;

/// Split Up mode 0 destroys tapped creatures and spares untapped ones.
#[test]
fn split_up_destroys_chosen_state() {
    let mut g = two_player_game();
    let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::split_up());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast Split Up (destroy tapped)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapped).is_none(), "tapped creature destroyed");
    assert!(g.battlefield_find(untapped).is_some(), "untapped creature spared");
}

/// Strongbox Raider's Raid ETB impulses two cards when you attacked this turn.
#[test]
fn strongbox_raider_raid_impulse() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    // No attack yet → no impulse.
    g.move_card_to_battlefield_for_test(0, catalog::strongbox_raider());
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 0, "no raid → no impulse");

    // Fresh board, mark an attack, then ETB the raider.
    let mut g = two_player_game();
    g.active_player_idx = 0;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].attacked_this_turn = true;
    g.move_card_to_battlefield_for_test(0, catalog::strongbox_raider());
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 2, "raid satisfied → top two exiled to impulse");
}

/// Fireglass Mentor impulses at your second main phase when an opponent lost life.
#[test]
fn fireglass_mentor_second_main_impulse() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(0, catalog::fireglass_mentor());
    g.adjust_life(1, -1); // opponent lost life this turn
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 2, "second main + opponent lost life → impulse two");
}

/// Menagerie Liberator's Melee grows it by the number of opponents attacked.
#[test]
fn menagerie_liberator_melee_pumps_on_attack() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let lib = g.add_card_to_battlefield(0, catalog::menagerie_liberator());
    g.clear_sickness(lib);
    // Before combat: base 3/2.
    assert_eq!(g.computed_permanent(lib).unwrap().power, 3);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lib,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack the lone opponent");
    drain_stack(&mut g);
    let cp = g.computed_permanent(lib).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "melee: +1/+1 for one opponent");
}
