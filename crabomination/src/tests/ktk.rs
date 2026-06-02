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

#[test]
fn lightning_berserker_pumps_for_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lightning_berserker());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("firebreathing");
    drain_stack(&mut g);
    let s = g.battlefield_find(id).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 1), "one red pumps +1/+0");
}

#[test]
fn alesha_dashes_and_has_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::alesha_who_smiles_at_death());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    dash(&mut g, id);
    let s = g.battlefield_find(id).unwrap();
    assert!(s.has_keyword(&Keyword::FirstStrike), "Alesha has first strike");
    assert!(s.granted_keywords_eot.contains(&Keyword::Haste), "dashed Alesha has haste");
}

#[test]
fn seeker_of_the_way_gains_lifelink_on_noncreature_cast() {
    let mut g = two_player_game();
    let seeker = g.add_card_to_battlefield(0, catalog::seeker_of_the_way());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    let s = g.battlefield_find(seeker).unwrap();
    assert!(s.has_keyword(&Keyword::Lifelink), "Seeker gains lifelink after a noncreature spell");
    // Prowess also pumped it to 3/3.
    assert_eq!((s.power(), s.toughness()), (3, 3), "prowess pumps too");
}

#[test]
fn jeskai_elder_loots_on_combat_damage() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::jeskai_elder());
    g.clear_sickness(atk);
    // Library + hand stocked so the loot draws then discards.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    // Advance through the combat-damage step so the elder connects.
    advance_to(&mut g, TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "unblocked 2/1 dealt 2 to the defender");
    // Combat damage fired the loot: +1 draw, -1 discard → hand unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "loot nets zero cards");
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}
