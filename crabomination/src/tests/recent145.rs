//! Functionality tests for `catalog::sets::decks::recent145` (WOE legends).

use crate::catalog;
use crate::card::CounterType;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};

/// Hylda's reflexive modal fires when you tap an opponent's creature and pay {1}.
#[test]
fn hylda_reflexive_modal_on_you_tap() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hylda_of_the_icy_crown());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1); // to pay the reflexive {1}
    // Mode 0 = create a 4/4 Elemental.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Modes(vec![0]),
    ]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: enemy, actor: Some(0) }]);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Elemental"),
        "paid {{1}} and made a 4/4 Elemental",
    );
}

/// Ash grows when attacking while Celebration is active.
#[test]
fn ash_celebration_attack_counter() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let ash = g.add_card_to_battlefield(0, catalog::ash_party_crasher());
    g.clear_sickness(ash);
    g.players[0].nonland_permanents_entered_this_turn = 2;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ash,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ash).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "Celebration grew Ash");
}
