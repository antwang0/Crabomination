//! Functionality tests for the `catalog::sets::decks::recent21` batch.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
}

/// Skyknight Vanguard makes a Soldier when it attacks.
#[test]
fn skyknight_vanguard_makes_soldier_on_attack() {
    let mut g = two_player_game();
    let sky = g.add_card_to_battlefield(0, catalog::skyknight_vanguard());
    g.clear_sickness(sky);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sky,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Soldier"), 1);
}

/// Aerial Boost pumps and grants flying.
#[test]
fn aerial_boost_pumps_and_flies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boost = g.add_card_to_hand(0, catalog::aerial_boost());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, boost, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Boots of Speed grants +1/+0 and haste when equipped.
#[test]
fn boots_of_speed_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boots = g.add_card_to_battlefield(0, catalog::boots_of_speed());
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: boots, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Ankle Biter has deathtouch.
#[test]
fn ankle_biter_has_deathtouch() {
    let mut g = two_player_game();
    let snake = g.add_card_to_battlefield(0, catalog::ankle_biter());
    assert!(g.computed_permanent(snake).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Trick Shot deals 6 to a creature.
#[test]
fn trick_shot_deals_six() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let ts = g.add_card_to_hand(0, catalog::trick_shot());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, ts, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "6 damage kills the 4/4");
}

/// Patient Naturalist mills three and grabs a land.
#[test]
fn patient_naturalist_grabs_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let pn = g.add_card_to_battlefield(0, catalog::patient_naturalist());
    let hand = g.players[0].hand.len();
    g.fire_self_etb_triggers(pn, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a land went to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
}

/// Plan the Heist draws three.
#[test]
fn plan_the_heist_draws_three() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let plan = g.add_card_to_hand(0, catalog::plan_the_heist());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, plan);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "−1 cast + 3 drawn");
}

/// Wanted Griffin leaves a Mercenary on death.
#[test]
fn wanted_griffin_dies_to_mercenary() {
    let mut g = two_player_game();
    let griffin = g.add_card_to_battlefield(0, catalog::wanted_griffin());
    let mut evs = g.remove_to_graveyard_with_triggers(griffin);
    evs.push(GameEvent::CreatureDied { card_id: griffin });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Mercenary"), 1);
}

/// Sterling Hound surveils 2 on ETB (top two cards may go to the graveyard).
#[test]
fn sterling_hound_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let hound = g.add_card_to_battlefield(0, catalog::sterling_hound());
    // AutoDecider keeps cards on top by default; just assert it resolves cleanly
    // and the library is intact (no panic, no card loss).
    let lib = g.players[0].library.len();
    g.fire_self_etb_triggers(hound, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib, "surveil kept both on top");
}

/// Hardbristle Bandit untaps when you commit a crime.
#[test]
fn hardbristle_untaps_on_crime() {
    let mut g = two_player_game();
    let bandit = g.add_card_to_battlefield(0, catalog::hardbristle_bandit());
    g.battlefield_find_mut(bandit).unwrap().tapped = true;
    // Commit a crime: cast Lava Spike at the opponent.
    let ls = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, ls, Target::Player(1));
    assert!(!g.battlefield_find(bandit).unwrap().tapped, "untapped by the crime trigger");
}

/// Rumbling Rockslide deals damage equal to your land count.
#[test]
fn rumbling_rockslide_scales_with_lands() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let rr = g.add_card_to_hand(0, catalog::rumbling_rockslide());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, rr, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "4 lands → 4 damage kills the 4/4");
}
