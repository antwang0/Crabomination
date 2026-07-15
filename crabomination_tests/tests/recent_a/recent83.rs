//! Functionality tests for `catalog::sets::decks::recent83`.

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

#[test]
fn walls_have_expected_stats_and_keywords() {
    let mut g = two_player_game();
    let kraken = g.add_card_to_battlefield(0, catalog::kraken_hatchling());
    let angelic = g.add_card_to_battlefield(0, catalog::angelic_wall());
    let steel = g.add_card_to_battlefield(0, catalog::steel_wall());
    let rampart = g.add_card_to_battlefield(0, catalog::fortified_rampart());
    assert_eq!(g.computed_permanent(kraken).unwrap().toughness, 4);
    let aw = g.computed_permanent(angelic).unwrap();
    assert!(aw.keywords.contains(&Keyword::Defender) && aw.keywords.contains(&Keyword::Flying));
    assert!(g.battlefield_find(steel).unwrap().definition.card_types.contains(&CardType::Artifact));
    assert_eq!(g.computed_permanent(rampart).unwrap().toughness, 6);
}

#[test]
fn dazzling_ramparts_taps_a_creature() {
    let mut g = two_player_game();
    let dr = g.add_card_to_battlefield(0, catalog::dazzling_ramparts());
    g.clear_sickness(dr);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dr, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    }).expect("tap ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "target creature tapped");
}

#[test]
fn vine_trellis_taps_for_green() {
    let mut g = two_player_game();
    let vt = g.add_card_to_battlefield(0, catalog::vine_trellis());
    g.clear_sickness(vt);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for G");
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Green), 1);
}

#[test]
fn overgrown_battlement_scales_with_defenders() {
    let mut g = two_player_game();
    let ob = g.add_card_to_battlefield(0, catalog::overgrown_battlement());
    g.add_card_to_battlefield(0, catalog::steel_wall()); // another defender
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not a defender
    g.clear_sickness(ob);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ob, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for G per defender");
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Green), 2,
        "two defenders → GG");
}

#[test]
fn gatecreeper_vine_tutors_a_basic_land_to_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.players[0].library.clear();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest))]));
    let gv = g.add_card_to_hand(0, catalog::gatecreeper_vine());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, gv);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "fetched a basic land to hand");
}

#[test]
fn blunt_the_assault_gains_life_and_fogs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let id = g.add_card_to_hand(0, catalog::blunt_the_assault());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life + 2, "gained 1 per creature (2 on board)");
    // Opponent attacks; combat damage is prevented (fog).
    g.active_player_idx = 1;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    let before = g.players[0].life;
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before, "combat damage prevented by the fog");
}
