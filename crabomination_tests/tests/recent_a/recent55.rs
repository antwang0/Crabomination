//! Functionality tests for `catalog::sets::decks::recent55` — artifact-matters.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::*;

fn cast_artifact(g: &mut GameState, controller: usize) {
    // Mind Stone — a cheap colorless artifact spell (a noncreature spell that
    // also enters as an artifact).
    let id = g.add_card_to_hand(controller, catalog::mind_stone());
    g.players[controller].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = controller;
    g.priority.player_with_priority = controller;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast artifact");
    drain_stack(g);
}

#[test]
fn thopter_engineer_makes_a_thopter_with_haste() {
    let mut g = two_player_game();
    let eng = g.add_card_to_battlefield(0, catalog::thopter_engineer());
    g.fire_self_etb_triggers(eng, 0);
    drain_stack(&mut g);
    let thopter_id = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Thopter")
        .expect("made a Thopter")
        .id;
    let cp = g.compute_battlefield();
    let thopter = cp.iter().find(|c| c.id == thopter_id).unwrap();
    assert!(thopter.keywords.contains(&Keyword::Flying), "Thopter flies");
    assert!(thopter.keywords.contains(&Keyword::Haste), "artifact creature granted haste");
}

#[test]
fn maverick_thopterist_makes_two_thopters() {
    let mut g = two_player_game();
    let mav = g.add_card_to_battlefield(0, catalog::maverick_thopterist());
    assert!(catalog::maverick_thopterist().keywords.contains(&Keyword::Improvise));
    g.fire_self_etb_triggers(mav, 0);
    drain_stack(&mut g);
    let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
    assert_eq!(thopters, 2, "made two Thopters");
}

#[test]
fn ingenious_smith_grows_on_artifact_entry() {
    let mut g = two_player_game();
    let smith = g.add_card_to_battlefield(0, catalog::ingenious_smith());
    cast_artifact(&mut g, 0);
    assert_eq!(
        g.battlefield_find(smith).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "an artifact entering grew the Smith",
    );
}

#[test]
fn ravenous_intruder_eats_an_artifact() {
    let mut g = two_player_game();
    let intruder = g.add_card_to_battlefield(0, catalog::ravenous_intruder());
    let art = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: intruder, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac an artifact for +2/+2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact sacrificed as the cost");
    let cp = g.compute_battlefield();
    let i = cp.iter().find(|c| c.id == intruder).unwrap();
    assert_eq!((i.power, i.toughness), (3, 4), "+2/+2 until end of turn");
}

#[test]
fn saheeli_makes_a_servo_on_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::saheeli_sublime_artificer());
    cast_artifact(&mut g, 0);
    let servos = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Servo" && c.definition.card_types.contains(&CardType::Artifact))
        .count();
    assert_eq!(servos, 1, "a noncreature spell made a Servo");
}
