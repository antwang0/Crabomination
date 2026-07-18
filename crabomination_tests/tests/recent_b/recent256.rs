//! Functionality tests for `catalog::sets::decks::recent256`
//! (Insidious Roots + Assemble the Players).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameEvent};
use crabomination::mana::Color;

/// When a creature card leaves your graveyard, Insidious Roots makes a Plant and
/// grows every Plant you control.
#[test]
fn insidious_roots_makes_plants_on_graveyard_departure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::insidious_roots());
    let leaver = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: leaver }]);
    drain_stack(&mut g);
    let plant = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Plant" && c.controller == 0)
        .expect("a Plant token was created");
    assert_eq!(
        plant.counter_count(CounterType::PlusOnePlusOne),
        1,
        "the new Plant got a +1/+1 counter",
    );
}

/// Assemble the Players lets you cast one small creature from the top of your
/// library each turn; a second top-of-library cast is blocked.
#[test]
fn assemble_the_players_casts_one_creature_from_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::assemble_the_players());
    let bear1 = g.add_card_to_library(0, catalog::grizzly_bears()); // power 2 — castable
    let bear2 = g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 4);
    g.perform_action(GameAction::CastSpell {
        card_id: bear1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("first small creature cast from the top succeeds");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear1).is_some(), "first bear resolved onto the battlefield");
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear2,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "second top-of-library cast is blocked by the once-per-turn cap",
    );
}
