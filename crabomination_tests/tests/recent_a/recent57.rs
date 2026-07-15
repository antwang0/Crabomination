//! Functionality tests for `catalog::sets::decks::recent57` — go-wide white.

use crabomination::card::CreatureType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;

fn bolt_kill(g: &mut GameState, victim: Target, controller: usize) {
    let bolt = g.add_card_to_hand(controller, catalog::lightning_bolt());
    g.players[controller].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = controller;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(victim), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(g);
}

#[test]
fn requiem_angel_makes_spirit_on_nonspirit_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::requiem_angel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt_kill(&mut g, Target::Permanent(bear), 0);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "non-Spirit death → Spirit");
}

#[test]
fn angel_of_the_dawn_pumps_team_until_eot() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(0, catalog::angel_of_the_dawn());
    g.fire_self_etb_triggers(angel, 0);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "+1/+1 to the team");
    assert!(b.keywords.contains(&crabomination::card::Keyword::Vigilance), "vigilance granted");
}

#[test]
fn elderfang_disciple_makes_opponent_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let disc = g.add_card_to_battlefield(0, catalog::elderfang_disciple());
    let opp_hand = g.players[1].hand.len();
    g.fire_self_etb_triggers(disc, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded a card");
}

#[test]
fn martial_coup_five_wraths_then_makes_five() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let coup = g.add_card_to_hand(0, catalog::martial_coup());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: coup, target: None, additional_targets: vec![], mode: None, x_value: Some(5),
    }).expect("cast Martial Coup for X=5");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "X≥5 destroyed the opponent's creature");
    let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier" && c.controller == 0).count();
    assert_eq!(soldiers, 5, "made five Soldiers that survive the wrath");
}

#[test]
fn beckon_apparition_exiles_and_makes_spirit() {
    let mut g = two_player_game();
    let gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let beckon = g.add_card_to_hand(0, catalog::beckon_apparition());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: beckon, target: Some(Target::Permanent(gy)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Beckon Apparition");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gy), "graveyard card exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "made a Spirit");
}

#[test]
fn kytheons_tactics_pumps_and_spell_mastery_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Two instants in the graveyard → spell mastery on.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let tac = g.add_card_to_hand(0, catalog::kytheons_tactics());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Kytheon's Tactics");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 3), "+2/+1");
    assert!(b.keywords.contains(&crabomination::card::Keyword::Vigilance), "spell mastery → vigilance");
}

#[test]
fn rally_the_ranks_anthems_chosen_type() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Bear)]));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a Bear
    let rally = g.add_card_to_battlefield(0, catalog::rally_the_ranks());
    g.fire_self_etb_triggers(rally, 0);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "chosen type (Bear) gets +1/+1");
}

#[test]
fn captains_claws_makes_kor_ally_on_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let claws = g.add_card_to_battlefield(0, catalog::captains_claws());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: claws, target: bear }).expect("equip");
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let ally = g.battlefield.iter().find(|c| c.definition.name == "Kor Ally");
    assert!(ally.is_some(), "equipped attacker made a Kor Ally");
    assert!(ally.unwrap().tapped, "the Kor Ally entered tapped and attacking");
}

#[test]
fn ancestral_blade_mints_and_equips_a_soldier() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::ancestral_blade());
    g.fire_self_etb_triggers(blade, 0);
    drain_stack(&mut g);
    let soldier = g.battlefield.iter().find(|c| c.definition.name == "Soldier").map(|c| c.id);
    assert!(soldier.is_some(), "made a Soldier token");
    // Blade attached to it → +1/+1 → 2/2.
    let cp = g.compute_battlefield();
    let s = cp.iter().find(|c| c.id == soldier.unwrap()).unwrap();
    assert_eq!((s.power, s.toughness), (2, 2), "the Soldier is equipped (+1/+1)");
}
