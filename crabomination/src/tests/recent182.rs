//! Functionality tests for `catalog::sets::decks::recent182` (BLB wave 2).

use crate::card::CounterType;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Finneas counters your other Rabbits/tokens and draws at high total power.
#[test]
fn finneas_counters_rabbits_and_draws() {
    let mut g = two_player_game();
    let finneas = g.add_card_to_battlefield(0, catalog::finneas_ace_archer());
    g.clear_sickness(finneas);
    // A big Rabbit so total power clears 10 after the counter.
    let mut rabbit = catalog::hill_giant(); // 3/3
    rabbit.subtypes.creature_types = vec![crate::card::CreatureType::Rabbit];
    rabbit.power = 8;
    let bunny = g.add_card_to_battlefield(0, rabbit);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: finneas,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bunny).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the other Rabbit got a counter",
    );
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew at total power 10+");
}

/// Gev pings an opponent whenever you cast a Lizard spell.
#[test]
fn gev_pings_on_lizard_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gev_scaled_scorch());
    // A Lizard creature spell to cast.
    let mut lizard = catalog::grizzly_bears();
    lizard.name = "Basking Lizard";
    lizard.subtypes.creature_types = vec![crate::card::CreatureType::Lizard];
    let spell = g.add_card_to_hand(0, lizard);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Lizard");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "Lizard cast pinged the opponent");
}
