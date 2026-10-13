//! Commander: Belbe's, Zurzoth's, Vannifar's, River Song's and The Cabbage
//! Merchant's last gaps (`decks::cmdr_edhrec31`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], mode: Option<usize>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn lives(g: &GameState) -> Vec<i32> {
    g.players.iter().map(|p| p.life).collect()
}

/// Mardu Shadowspear attacking drains each opponent for 1.
#[test]
fn mardu_shadowspear_attack_drain() {
    let mut g = pod(3);
    let id = ready(&mut g, 0, catalog::mardu_shadowspear());
    attack(&mut g, &[id]);
    assert_eq!(lives(&g), vec![20, 19, 19]);
}

/// Night Market Lookout drains when it becomes tapped (by attacking).
#[test]
fn night_market_lookout_drains_on_tap() {
    let mut g = pod(2);
    let id = ready(&mut g, 0, catalog::night_market_lookout());
    attack(&mut g, &[id]);
    assert_eq!(lives(&g), vec![21, 19]);
}

/// Thornbow Archer spares an opponent who controls an Elf.
#[test]
fn thornbow_archer_spares_elf_players() {
    let mut g = pod(3);
    ready(&mut g, 2, catalog::llanowar_elves());
    let id = ready(&mut g, 0, catalog::thornbow_archer());
    attack(&mut g, &[id]);
    assert_eq!(lives(&g), vec![20, 19, 20]);
}

/// Gibbering Fiend pings each opponent on entry, and on their upkeep under
/// delirium (CR 207.2c ability word; four card types in your graveyard).
#[test]
fn gibbering_fiend_pings() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::gibbering_fiend());
    cast(&mut g, 0, id, &[], None).expect("cast");
    assert_eq!(g.players[1].life, 19);
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "no delirium yet");
    for def in [catalog::forest, catalog::lightning_bolt, catalog::grizzly_bears, catalog::sol_ring] {
        g.add_card_to_graveyard(0, def());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Hobblefiend eats another creature for a counter.
#[test]
fn hobblefiend_eats() {
    let mut g = pod(2);
    let id = ready(&mut g, 0, catalog::hobblefiend());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Spiteful Prankster has first strike only on your turn and pings when
/// another creature dies.
#[test]
fn spiteful_prankster_pings_on_death() {
    let mut g = pod(2);
    let id = ready(&mut g, 0, catalog::spiteful_prankster());
    let fs = |g: &GameState| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::FirstStrike);
    assert!(fs(&g));
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, &[Target::Permanent(bear)], None).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(lives(&g).iter().sum::<i32>(), 39);
    g.active_player_idx = 1;
    assert!(!fs(&g));
}

/// Meneldor's connect blinks a creature you own.
#[test]
fn meneldor_blinks_on_connect() {
    let mut g = pod(2);
    let id = ready(&mut g, 0, catalog::meneldor_swift_savior());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    attack(&mut g, &[id]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 17);
    let b = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("back");
    assert!(!b.tapped, "a blinked permanent returns untapped");
}

/// Illusionist's Stratagem blinks two of your creatures and draws.
#[test]
fn illusionists_stratagem_blinks_and_draws() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::hill_giant());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::illusionists_stratagem());
    cast(&mut g, 0, id, &[Target::Permanent(a), Target::Permanent(b)], None).expect("cast");
    assert!(g.battlefield.iter().filter(|c| c.controller == 0).all(|c| !c.tapped));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Siren's Ruse draws only for a Pirate.
#[test]
fn sirens_ruse_draws_for_pirates() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::sirens_ruse());
    cast(&mut g, 0, id, &[Target::Permanent(bear)], None).expect("cast");
    assert!(g.players[0].hand.is_empty(), "a Bear isn't a Pirate");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Avengers Disassembled's first mode burns each creature for 3.
#[test]
fn avengers_disassembled_sweeps() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::avengers_disassembled());
    cast(&mut g, 0, id, &[], Some(0)).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none());
}

/// Geomancer's Gambit destroys a land, gives its controller a basic, and
/// draws.
#[test]
fn geomancers_gambit_swaps_a_land() {
    let mut g = pod(2);
    let land = ready(&mut g, 1, catalog::mountain());
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::geomancers_gambit());
    cast(&mut g, 0, id, &[Target::Permanent(land)], None).expect("cast");
    assert!(g.battlefield_find(land).is_none());
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("basic fetched");
    assert_eq!(forest.controller, 1);
    assert!(!forest.tapped);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Bosco makes a Food per legend and eats one for two counters and trample.
#[test]
fn bosco_feasts() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::bosco_just_a_bear());
    cast(&mut g, 0, id, &[], None).expect("cast");
    let foods = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Food").count();
    assert_eq!(foods(&g), 1);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(foods(&g), 0);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Trample));
}

/// Teething Wurmlet grows only on the first artifact each turn, and gains
/// deathtouch at three artifacts.
#[test]
fn teething_wurmlet_grows_once_a_turn() {
    let mut g = pod(2);
    let id = ready(&mut g, 0, catalog::teething_wurmlet());
    for _ in 0..3 {
        let ring = g.add_card_to_hand(0, catalog::sol_ring());
        cast(&mut g, 0, ring, &[], None).expect("ring");
    }
    assert_eq!(g.players[0].life, 23);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Deathtouch));
}

/// Bumi's Feast Lecture makes a Food, then earthbends twice the Food count.
#[test]
fn bumis_feast_lecture_earthbends() {
    let mut g = pod(2);
    let land = ready(&mut g, 0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::bumis_feast_lecture());
    cast(&mut g, 0, id, &[Target::Permanent(land)], None).expect("cast");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}
