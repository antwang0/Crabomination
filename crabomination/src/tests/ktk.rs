//! Functionality tests for the Khans-of-Tarkir Dash pack
//! (`catalog::sets::ktk`). Each test exercises the Dash alternative cost
//! (CR 702.110): haste on entry + return to hand at the next end step.

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn dash(g: &mut GameState, id: crate::card::CardId) {
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("dash cast should succeed");
    drain_stack(g);
}

#[test]
fn dash_enters_with_haste_and_returns_to_hand_at_end_step() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::screamreach_brawler());
    // Dash {1}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    dash(&mut g, id);

    // On the battlefield with haste.
    let brawler = g.battlefield.iter().find(|c| c.id == id).expect("dashed creature on battlefield");
    assert!(brawler.granted_keywords_eot.contains(&Keyword::Haste), "dash grants haste");

    // At the next end step it returns to its owner's hand.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "dashed creature left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "dashed creature returned to hand");
}

#[test]
fn normal_cast_does_not_dash_bounce() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::screamreach_brawler());
    // Pay the printed {2}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("normal cast for {2}{R}");
    drain_stack(&mut g);

    let brawler = g.battlefield.iter().find(|c| c.id == id).expect("on battlefield");
    assert!(!brawler.granted_keywords_eot.contains(&Keyword::Haste), "no haste on a normal cast");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "normal-cast creature stays on the battlefield");
}

#[test]
fn ponyback_brigade_dash_makes_three_goblins() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ponyback_brigade());
    // Dash {4}{B}{R}.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    dash(&mut g, id);

    let goblins = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin")
        .count();
    assert_eq!(goblins, 3, "Ponyback Brigade ETBs three Goblin tokens");
}

#[test]
fn mardu_scout_dashes_for_a_single_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mardu_scout());
    g.players[0].mana_pool.add(Color::Red, 1);
    dash(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Mardu Scout dashes for one red");
}
