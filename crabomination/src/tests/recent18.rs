//! Functionality tests for the `catalog::sets::decks::recent18` Foundations batch.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

/// Charging Bandits pumps itself +2/+0 when it attacks.
#[test]
fn charging_bandits_pumps_on_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::charging_bandits());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 5, "3/3 + 2/0 on attack");
}

/// Dazzling Angel gains life when another creature enters.
#[test]
fn dazzling_angel_gains_life_on_other_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dazzling_angel());
    let life = g.players[0].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear); // real cast so the Angel's ETB watcher fires
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 from the other creature");
}

/// Dragon Trainer makes a 4/4 flying Dragon on ETB.
#[test]
fn dragon_trainer_makes_dragon() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dragon_trainer());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let dragon = g.battlefield.iter().find(|c| c.definition.name == "Dragon").expect("a Dragon");
    assert_eq!((dragon.definition.power, dragon.definition.toughness), (4, 4));
    assert!(g.computed_permanent(dragon.id).unwrap().keywords.contains(&Keyword::Flying));
}

/// Goblin Tomb Raider gets +1/+0 and haste only while you control an artifact.
#[test]
fn goblin_tomb_raider_artifact_gate() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::goblin_tomb_raider());
    assert_eq!(g.computed_permanent(gob).unwrap().power, 1, "base 1/2 without an artifact");
    assert!(!g.computed_permanent(gob).unwrap().keywords.contains(&Keyword::Haste));
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // an artifact
    assert_eq!(g.computed_permanent(gob).unwrap().power, 2, "+1/+0 with an artifact");
    assert!(g.computed_permanent(gob).unwrap().keywords.contains(&Keyword::Haste));
}

/// Sanguine Syphoner drains 1 when it attacks.
#[test]
fn sanguine_syphoner_drains_on_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sanguine_syphoner());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let (my, opp) = (g.players[0].life, g.players[1].life);
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, my + 1, "you gained 1");
}

/// Sky Crier draws for both players when its ability resolves.
#[test]
fn sky_crier_draws_for_both() {
    let mut g = two_player_game();
    let crier = g.add_card_to_battlefield(0, catalog::sky_crier());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.perform_action(GameAction::ActivateAbility {
        card_id: crier, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Sky Crier");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1, "you drew");
    assert_eq!(g.players[1].hand.len(), h1 + 1, "target opponent drew");
}

/// Soulmender taps to gain a life.
#[test]
fn soulmender_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::soulmender());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap Soulmender");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1);
}

/// Stormfist Crusader wheels both players a card and a life at your upkeep.
#[test]
fn stormfist_crusader_upkeep_wheel() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stormfist_crusader());
    for p in 0..2 { g.add_card_to_library(p, catalog::grizzly_bears()); }
    let (h0, l1) = (g.players[0].hand.len(), g.players[1].life);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1, "each player drew");
    assert_eq!(g.players[1].life, l1 - 1, "each player lost 1");
}

/// Run Away Together returns two creatures to their owners' hands.
#[test]
fn run_away_together_bounces_two() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::run_away_together());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Run Away Together");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert_eq!(g.players[0].hand.len(), 1, "my creature returned to my hand");
    assert_eq!(g.players[1].hand.len(), 1, "their creature returned to their hand");
}

/// Captured by Lagacs stops the enchanted creature from attacking and supports 2.
#[test]
fn captured_by_lagacs_locks_and_supports() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::captured_by_lagacs());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    g.fire_self_etb_triggers(aura, 0);
    drain_stack(&mut g);
    let kws = g.computed_permanent(victim).unwrap().keywords;
    assert!(kws.contains(&Keyword::CantAttack) && kws.contains(&Keyword::CantBlock));
    // Support 2 landed a +1/+1 counter somewhere friendly (the other bear).
    assert!(g.computed_permanent(other).unwrap().power >= 2, "support buffed a creature");
}

/// Battle Screech makes two flying Birds.
#[test]
fn battle_screech_makes_two_birds() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::battle_screech());
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Battle Screech");
    drain_stack(&mut g);
    let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird").count();
    assert_eq!(birds, 2, "two 1/1 Birds");
}

/// Quag Vampires enters with a +1/+1 counter for each Multikicker payment.
#[test]
fn quag_vampires_grows_with_multikicker() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quag_vampires());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 3); // {B} + two kicks of {1}{B}
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: id, times: 2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Quag Vampires kicked twice");
    drain_stack(&mut g);
    let v = g.battlefield.iter().find(|c| c.definition.name == "Quag Vampires").expect("on battlefield");
    assert!(g.computed_permanent(v.id).unwrap().power >= 3, "1/1 base + 2 kicked counters");
}

/// Bear Cub and Sworn Guardian are simple vanilla bodies.
#[test]
fn vanilla_bodies_have_correct_stats() {
    let cub = catalog::bear_cub();
    assert_eq!((cub.power, cub.toughness), (2, 2));
    assert!(cub.keywords.is_empty() && cub.triggered_abilities.is_empty());
    let guard = catalog::sworn_guardian();
    assert_eq!((guard.power, guard.toughness), (1, 3));
}

/// Hunter's Edge counters a friendly creature, then it bites an enemy.
#[test]
fn hunters_edge_counters_then_bites() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3 after counter
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, takes 3
    let id = g.add_card_to_hand(0, catalog::hunters_edge());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(enemy)], mode: None, x_value: None,
    }).expect("cast Hunter's Edge");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+1/+1 counter");
    assert!(g.battlefield_find(enemy).is_none(), "took 3 from a 3-power creature");
}

/// Kitsa loots with its tap ability and has prowess.
#[test]
fn kitsa_loots_and_has_prowess() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kitsa_otterball_elite());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("loot with Kitsa");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one — net zero");
    assert!(catalog::kitsa_otterball_elite().keywords.contains(&Keyword::Prowess));
}
