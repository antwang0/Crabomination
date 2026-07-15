//! Functionality tests for `catalog::sets::decks::recent180` (DSK/BLB batch).

use crabomination::card::{CounterType, CreatureType};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Possessed Goat's once-per-game ability grows it and makes it a black Demon.
#[test]
fn possessed_goat_becomes_black_demon_once() {
    let mut g = two_player_game();
    let goat = g.add_card_to_battlefield(0, catalog::possessed_goat());
    g.clear_sickness(goat);
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: goat,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Possessed Goat");
    drain_stack(&mut g);
    let cp = g.computed_permanent(goat).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "three +1/+1 counters → 4/4");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Demon), "became a Demon");
    assert!(cp.colors.contains(&Color::Black), "became black");
    assert!(cp.colors.contains(&Color::White), "kept its white color");
    // "Activate only once" — the second try is rejected.
    let second = g.perform_action(GameAction::ActivateAbility {
        card_id: goat,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(second.is_err(), "cannot activate a second time");
}

/// Hired Claw pings an opponent when you attack with a Lizard.
#[test]
fn hired_claw_pings_on_lizard_attack() {
    let mut g = two_player_game();
    let claw = g.add_card_to_battlefield(0, catalog::hired_claw());
    g.clear_sickness(claw);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let opp = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: claw,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack with the Lizard");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "Lizard attack pinged for 1");
}

/// Hired Claw's growth ability is gated on an opponent losing life this turn.
#[test]
fn hired_claw_growth_needs_crime() {
    let mut g = two_player_game();
    let claw = g.add_card_to_battlefield(0, catalog::hired_claw());
    g.clear_sickness(claw);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    // No opponent has lost life → rejected.
    let blocked = g.perform_action(GameAction::ActivateAbility {
        card_id: claw,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(blocked.is_err(), "growth gated until an opponent lost life");
    // Make the opponent lose life, then it works.
    g.adjust_life(1, -1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: claw,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("growth now legal");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(claw).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "grew a +1/+1 counter",
    );
}

/// Mistbreath Elder bounces another creature you control and grows on upkeep.
#[test]
fn mistbreath_elder_bounces_and_grows() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::mistbreath_elder());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Reach the controller's next upkeep.
    while !(g.step == TurnStep::Upkeep && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == friend), "friend bounced to hand");
    assert_eq!(
        g.battlefield_find(elder).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Elder grew a +1/+1 counter",
    );
}
