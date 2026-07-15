//! Functionality tests for the `catalog::sets::decks::recent9` batch.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Haru earthbends a land when another Ally enters under your control.
#[test]
fn haru_earthbends_on_ally_entering() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::haru_hidden_talent());
    let ally = g.add_card_to_hand(0, catalog::master_pakku()); // a vanilla Ally
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ally);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Ally entering earthbent the land"
    );
}

/// Avatar Enthusiasts grows when another Ally enters.
#[test]
fn avatar_enthusiasts_grows_on_ally() {
    let mut g = two_player_game();
    let enth = g.add_card_to_battlefield(0, catalog::avatar_enthusiasts());
    let ally = g.add_card_to_hand(0, catalog::master_pakku());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ally);
    assert_eq!(
        g.battlefield_find(enth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Invasion Reinforcements mints a 1/1 Ally on ETB.
#[test]
fn invasion_reinforcements_makes_ally() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::invasion_reinforcements());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ally"));
}

/// Aang, Airbending Master airbends another creature on ETB.
#[test]
fn aang_airbending_master_airbends() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::aang_airbending_master());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    assert!(g.exile.iter().any(|c| c.id == victim), "airbent the opposing creature");
}

/// Sinister Gnarlbark draws and blights at the end step.
#[test]
fn sinister_gnarlbark_end_step() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let gnarl = g.add_card_to_battlefield(0, catalog::sinister_gnarlbark());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(
        g.battlefield_find(gnarl).unwrap().counter_count(CounterType::MinusOneMinusOne),
        1,
        "blighted itself"
    );
}

/// Dream Seizer blights and makes each opponent discard.
#[test]
fn dream_seizer_blights_and_discards() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::dream_seizer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(victim).unwrap().counter_count(CounterType::MinusOneMinusOne),
        1
    );
    assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded");
}

/// Sourbread Auntie blights 2 and mints two Goblins.
#[test]
fn sourbread_auntie_blights_for_goblins() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sourbread_auntie());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id);
    // grizzly is first in battlefield order → blighted 2 → 0/0 → dies.
    assert!(g.battlefield_find(victim).is_none(), "blighted 2/2 died");
    let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
    assert_eq!(goblins, 2);
}

/// Shadow Urchin blights when it attacks.
#[test]
fn shadow_urchin_blights_on_attack() {
    let mut g = two_player_game();
    let urchin = g.add_card_to_battlefield(0, catalog::shadow_urchin());
    g.clear_sickness(urchin);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: urchin,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(urchin).unwrap().counter_count(CounterType::MinusOneMinusOne),
        1,
        "blighted itself on attack"
    );
}

/// Knowledge Seeker grows on your second draw.
#[test]
fn knowledge_seeker_counters_on_second_draw() {
    let mut g = two_player_game();
    let seeker = g.add_card_to_battlefield(0, catalog::knowledge_seeker());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let div = g.add_card_to_hand(0, catalog::divination()); // draw 2
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, div);
    assert_eq!(
        g.battlefield_find(seeker).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Otter-Penguin pumps on your second draw.
#[test]
fn otter_penguin_pumps_on_second_draw() {
    let mut g = two_player_game();
    let otter = g.add_card_to_battlefield(0, catalog::otter_penguin());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, div);
    let cp = g.computed_permanent(otter).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/1 +1/+2 = 3/3");
}

/// Mai and Master Pakku both have prowess.
#[test]
fn prowess_creatures() {
    assert!(catalog::mai_jaded_edge().keywords.contains(&Keyword::Prowess));
    assert!(catalog::master_pakku().keywords.contains(&Keyword::Prowess));
}

/// Unlucky Cabbage Merchant creates a Food on ETB.
#[test]
fn unlucky_cabbage_merchant_makes_food() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::unlucky_cabbage_merchant());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
}

/// Curious Farm Animals gains 3 life when it dies.
#[test]
fn curious_farm_animals_gains_life_on_death() {
    let mut g = two_player_game();
    let cfa = g.add_card_to_battlefield(0, catalog::curious_farm_animals());
    let life = g.players[0].life;
    g.remove_to_graveyard_with_triggers(cfa);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
}

/// Deserter's Disciple makes a small creature unblockable.
#[test]
fn deserters_disciple_grants_unblockable() {
    let mut g = two_player_game();
    let disciple = g.add_card_to_battlefield(0, catalog::deserters_disciple());
    g.clear_sickness(disciple);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: disciple,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::Unblockable));
}

/// Turtle-Duck pumps itself and gains trample.
#[test]
fn turtle_duck_pumps_and_tramples() {
    let mut g = two_player_game();
    let duck = g.add_card_to_battlefield(0, catalog::turtle_duck());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: duck,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(duck).unwrap();
    assert_eq!(cp.power, 4, "0 base + 4");
    assert!(cp.keywords.contains(&Keyword::Trample));
}
