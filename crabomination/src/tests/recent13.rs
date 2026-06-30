//! Functionality tests for the `catalog::sets::decks::recent13` batch.

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;

/// Misery's Shadow exiles a dying opponent creature instead of letting it hit
/// the graveyard, and its {1} pump grows it.
#[test]
fn miserys_shadow_exiles_dying_opponent_creature_and_pumps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::miserys_shadow());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(foe);
    g.check_state_based_actions();
    assert!(g.players[1].graveyard.iter().all(|c| c.id != foe), "not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == foe), "routed to exile instead");
    // {1} pump.
    let shadow = g.battlefield.iter().find(|c| c.definition.name == "Misery's Shadow").unwrap().id;
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: shadow, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(shadow).unwrap().power, 3);
}

/// Glarb lets the controller cast a MV-4+ spell off the top of the library.
#[test]
fn glarb_casts_high_mv_from_library_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::glarb_calamitys_augur());
    // Top of library: a 4-MV creature (Serra Angel is {3}{W}{W} = 5; use a 4-drop).
    g.add_card_to_library(0, catalog::wrath_of_god()); // {2}{W}{W} = MV 4 sorcery
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let top = g.players[0].library[0].id;
    cast(&mut g, top);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "cast from top resolved");
}

/// Archfiend enters with four oil counters and an opponent's dying creature
/// drains its controller 2 life.
#[test]
fn archfiend_oil_counters_and_opponent_death_drain() {
    let mut g = two_player_game();
    let arch = g.move_card_to_battlefield_for_test(0, catalog::archfiend_of_the_dross());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(arch).unwrap().counter_count(CounterType::Oil), 4);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    // Kill the foe through the normal damage/SBA funnel so the death trigger
    // dispatches the same way the live game does.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bolt, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "foe died");
    assert_eq!(g.players[1].life, life - 2, "opponent loses 2 when their creature dies");
}

/// Archfiend's upkeep removes an oil counter; with one left, removing it makes
/// the controller lose the game.
#[test]
fn archfiend_loses_game_at_zero_oil() {
    let mut g = two_player_game();
    let arch = g.move_card_to_battlefield_for_test(0, catalog::archfiend_of_the_dross());
    drain_stack(&mut g);
    // Drain to a single oil counter, then the next upkeep removal hits zero.
    let inst = g.battlefield.iter_mut().find(|c| c.id == arch).unwrap();
    while inst.counter_count(CounterType::Oil) > 1 {
        inst.remove_counters(CounterType::Oil, 1);
    }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "controller lost the game with no oil counters");
}

/// Seeds of Renewal returns up to two cards from the graveyard to hand.
#[test]
fn seeds_of_renewal_returns_two_from_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::seeds_of_renewal());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5); // {6}{G} - {1} undaunted = {5}{G}
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "two cards returned (spell left hand)");
    assert!(g.exile.iter().any(|c| c.definition.name == "Seeds of Renewal"), "self-exiled");
}

/// Spara's Headquarters is a GWU Triome that enters tapped with Cycling.
#[test]
fn sparas_headquarters_is_a_triome() {
    let d = catalog::sparas_headquarters();
    assert!(d.card_types.contains(&CardType::Land));
    assert_eq!(d.activated_abilities.len(), 3, "taps for three colors");
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
}

/// Mishra's Foundry animates into a 2/2 Assembly-Worker.
#[test]
fn mishras_foundry_animates() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mishras_foundry());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature) && cp.card_types.contains(&CardType::Land));
    assert_eq!((cp.power, cp.toughness), (2, 2));
}
