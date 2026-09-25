//! Commander: the Grand Larceny precon (OTC, Gonti, `decks::cmdr_gonti`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
}

fn swing(g: &mut GameState, attackers: &[CardId]) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn exiled_for(g: &GameState, seat: usize) -> usize {
    g.exile.iter().filter(|c| c.may_play_until.is_some_and(|m| m.player == seat)).count()
}

/// Gonti (CR 903.3): your creatures hitting a player exile its top card face
/// down for you; a spell you don't own costs {1} less.
#[test]
fn gonti_steals_and_discounts() {
    assert!(catalog::gonti_canny_acquisitor().can_be_commander);
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gonti_canny_acquisitor());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, &[bear]);
    assert_eq!(exiled_for(&g, 0), 1);
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gonti_canny_acquisitor());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    g.find_card_anywhere_mut(wurm).unwrap().owner = 1;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast_by(&mut g, 0, wurm, &[], None);
    assert!(g.battlefield_find(wurm).is_some(), "{{4}}{{G}}{{G}} for five");
}

/// Felix Five-Boots doubles a combat-damage trigger: Gonti exiles two.
#[test]
fn felix_doubles_the_heist() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gonti_canny_acquisitor());
    g.add_card_to_battlefield(0, catalog::felix_five_boots());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, &[bear]);
    assert_eq!(exiled_for(&g, 0), 2);
}

/// Thief of Sanity takes one of three face down; the others go to the
/// graveyard.
#[test]
fn thief_of_sanity_takes_one_of_three() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::thief_of_sanity());
    for f in [catalog::island, catalog::craw_wurm, catalog::grizzly_bears] {
        g.add_card_to_library(1, f());
    }
    swing(&mut g, &[t]);
    let taken = g.exile.iter().find(|c| c.may_play_until.is_some_and(|m| m.player == 0)).expect("exiled");
    assert!(taken.face_down);
    assert_eq!(taken.definition.name, "Craw Wurm");
    assert_eq!(g.players[1].graveyard.len(), 2);
}

/// Heartless Conscription exiles every creature for you to play, then
/// itself.
#[test]
fn heartless_conscription_drafts_the_board() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hc = g.add_card_to_hand(0, catalog::heartless_conscription());
    flood(&mut g, 0);
    cast_by(&mut g, 0, hc, &[], None);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));
    assert_eq!(exiled_for(&g, 0), 2);
    assert!(g.exile.iter().any(|c| c.id == hc));
}

/// Smirking Spelljacker exiles an opponent's spell, then casts it free on
/// attack.
#[test]
fn smirking_spelljacker_steals_a_spell() {
    let mut g = pod(2);
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: wurm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("wurm");
    let sj = g.add_card_to_hand(0, catalog::smirking_spelljacker());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sj,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flash");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == wurm && c.exiled_with == Some(sj)));
    g.active_player_idx = 0;
    yes(&mut g);
    swing(&mut g, &[sj]);
    assert_eq!(g.battlefield_find(wurm).map(|c| c.controller), Some(0));
}

/// Thieving Amalgam manifests at an opponent's upkeep (CR 701.34) and
/// drains when a creature it stole dies.
#[test]
fn thieving_amalgam_manifests_and_drains() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::thieving_amalgam());
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let m = g.battlefield_find(top).expect("manifested");
    assert!(m.controller == 0 && m.face_down);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast_by(&mut g, 0, murder, &[Target::Permanent(top)], None);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 2);
}

/// Thieving Skydiver kicked for X takes an artifact with mana value X or
/// less; unkicked it takes nothing.
#[test]
fn thieving_skydiver_takes_an_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let sd = g.add_card_to_hand(0, catalog::thieving_skydiver());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: sd,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ring).map(|c| c.controller), Some(0));
}

/// Mind's Dilation: an opponent's first spell each turn exiles their top
/// card, which you may cast free.
#[test]
fn minds_dilation_casts_their_top() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::minds_dilation());
    let wurm = g.add_card_to_library(1, catalog::craw_wurm());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    yes(&mut g);
    cast_by(&mut g, 1, bear, &[], None);
    assert_eq!(g.battlefield_find(wurm).map(|c| c.controller), Some(0));
}

/// Extract Brain: the opponent names X cards; you cast one free.
#[test]
fn extract_brain_casts_from_their_hand() {
    let mut g = pod(2);
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    g.add_card_to_hand(1, catalog::island());
    let eb = g.add_card_to_hand(0, catalog::extract_brain());
    flood(&mut g, 0);
    yes(&mut g);
    cast_by(&mut g, 0, eb, &[Target::Player(1)], Some(2));
    assert_eq!(g.battlefield_find(wurm).map(|c| c.controller), Some(0));
}

/// Tower Winder finds Command Tower in the graveyard first.
#[test]
fn tower_winder_finds_the_tower() {
    let mut g = pod(2);
    let ct = g.add_card_to_graveyard(0, catalog::command_tower());
    let tw = g.add_card_to_hand(0, catalog::tower_winder());
    flood(&mut g, 0);
    cast_by(&mut g, 0, tw, &[], None);
    assert!(g.players[0].hand.iter().any(|c| c.id == ct));
}
