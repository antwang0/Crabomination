//! CR conformance for this run's engine work:
//! - CR 510.1a — a creature that would assign 0 or less combat damage assigns
//!   none, and CR 510.1a's "rather than the attacking player" hand-off.
//! - CR 514.3a — "at the beginning of the next cleanup step" triggers go on
//!   the stack in cleanup and grant priority.
//! - CR 724 — ending the turn exiles the stack and jumps to cleanup.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [
        crabomination::mana::Color::White,
        crabomination::mana::Color::Blue,
        crabomination::mana::Color::Black,
        crabomination::mana::Color::Red,
        crabomination::mana::Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn to_declare_attackers(g: &mut GameState) {
    while g.step != TurnStep::DeclareAttackers || g.active_player_idx != 0 {
        g.advance_step(vec![]).expect("advance");
    }
}

/// CR 510.1a — "Creatures that would assign 0 or less damage this way don't
/// assign combat damage at all." A 0-power blocker leaves its attacker intact.
#[test]
fn cr_510_1a_zero_power_creatures_assign_no_combat_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::ornithopter()); // 0/2
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    to_declare_attackers(&mut g);
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.advance_step(vec![]).expect("to blockers");
    g.declare_blockers(vec![(wall, attacker)]).expect("block");
    g.advance_step(vec![]).expect("to damage");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(attacker).map(|c| c.damage),
        Some(0),
        "a 0-power blocker deals nothing"
    );
    assert!(g.battlefield_find(wall).is_none(), "the 2/2 still killed the 0/2");
}

/// CR 510.1a — "Rather than the attacking player, you assign the combat damage
/// of each creature attacking you." Defensive Formation moves the assignment
/// decision to the defending seat.
#[test]
fn cr_510_1a_defender_may_assign_attackers_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::defensive_formation());
    let attacker = g.add_card_to_battlefield(0, catalog::okk()); // 4/4
    let partner = g.add_card_to_battlefield(0, catalog::serra_avatar());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [attacker, partner] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    to_declare_attackers(&mut g);
    g.declare_attackers(vec![
        Attack { attacker, target: AttackTarget::Player(1) },
        Attack { attacker: partner, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    g.advance_step(vec![]).expect("to blockers");
    g.declare_blockers(vec![(a, attacker), (b, attacker)]).expect("double block");
    // Seat 1 (the defender) is the assigner, so it's the seat that gets asked.
    g.players[1].wants_ui = true;
    g.advance_step(vec![]).expect("to damage");
    let pending = g.pending_decision.as_ref().expect("a decision was posed");
    assert!(
        matches!(
            pending.decision,
            crabomination::decision::Decision::CombatDamageOrder { .. }
        ),
        "the damage order went to the defending seat"
    );
}

/// CR 514.3a — a "at the beginning of the next cleanup step" trigger fires in
/// cleanup, after the end step has already come and gone.
#[test]
fn cr_514_3a_cleanup_triggers_fire_after_the_end_step() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::waylay());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    let knights: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Knight").map(|c| c.id).collect();
    assert_eq!(knights.len(), 3);
    while g.step != TurnStep::End {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert!(
        knights.iter().all(|id| g.battlefield_find(*id).is_some()),
        "the end step is not the cleanup step"
    );
    g.advance_step(vec![]).expect("to cleanup");
    drain_stack(&mut g);
    assert!(knights.iter().all(|id| g.battlefield_find(*id).is_none()));
}

/// CR 724.1b / 724.1d — ending the turn exiles everything still on the stack
/// and jumps straight to cleanup, so "until end of turn" grants expire.
#[test]
fn cr_724_1b_ending_the_turn_exiles_the_stack_and_expires_grants() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(0, catalog::sundial_of_the_infinite());
    g.battlefield_find_mut(sundial).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    mana(&mut g, 0);
    cast(&mut g, pump, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    // A second spell is left on the stack unresolved when the turn ends.
    let stranded = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::CastSpell {
        card_id: stranded,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sundial,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == stranded), "the stack was exiled");
    assert_eq!(g.players[1].life, life, "the exiled Bolt never resolved");
    assert_eq!(
        g.computed_permanent(bear).map(|c| c.power),
        Some(2),
        "cleanup ended the until-end-of-turn pump"
    );
}

/// CR 724.1d — ending the turn during combat removes every creature from it.
#[test]
fn cr_724_1d_ending_the_turn_clears_combat() {
    let mut g = two_player_game();
    let sundial = g.add_card_to_battlefield(0, catalog::sundial_of_the_infinite());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [sundial, attacker] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    to_declare_attackers(&mut g);
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    assert_eq!(g.attacking.len(), 1);
    let life = g.players[1].life;
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sundial,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.attacking.is_empty(), "combat was cleared");
    assert_eq!(g.players[1].life, life, "no combat damage was ever dealt");
}
