//! Commander: the Mirror Mastery precon (CMD, Riku of Two Reflections,
//! `decks::cmdr_riku`) and the primitive it needed.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn yes(g: &mut GameState, n: usize) {
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); n]));
}

fn connect(g: &mut GameState, a: CardId, victim: usize) {
    g.clear_sickness(a);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(victim) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = victim;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Riku pays {U}{R} to copy an instant, and {G}{U} for a token copy of a
/// nontoken creature entering under its controller.
#[test]
fn riku_copies_spells_and_creatures() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::riku_of_two_reflections());
    yes(&mut g, 4);
    flood(&mut g, 0);
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Player(1)), vec![]).expect("bolt");
    assert_eq!(g.players[1].life, life - 6, "the copy dealt 3 too");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bear, None, vec![]).expect("bears");
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "the card and its token copy");
}

/// Animar grows on each creature spell and discounts the next by its counters.
#[test]
fn animar_discounts_creatures_by_its_counters() {
    let mut g = pod(2);
    let animar = g.add_card_to_battlefield(0, catalog::animar_soul_of_elements());
    g.players[0].mana_pool.add(Color::Green, 2);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bear, None, vec![]).expect("bears for {1}{G}");
    assert_eq!(g.battlefield_find(animar).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bear2, None, vec![]).expect("{1} less: a single {G}");
}

/// Firespout paid with {R} (not {G}) hits only the creatures without flying.
#[test]
fn firespout_reads_the_colour_spent() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bird = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::Red, 3);
    let spout = g.add_card_to_hand(0, catalog::firespout());
    cast_with(&mut g, spout, None, vec![]).expect("firespout");
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "no flying: hit");
    assert_eq!(g.battlefield_find(bird).unwrap().damage, 0, "no {{G}} spent: fliers spared");
}

/// CR 802 — Hydra Omnivore's combat damage to one opponent is dealt to each
/// other opponent too, never to its controller.
#[test]
fn hydra_omnivore_splashes_every_other_opponent() {
    let mut g = pod(4);
    let hydra = g.add_card_to_battlefield(0, catalog::hydra_omnivore());
    let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    connect(&mut g, hydra, 2);
    for seat in 1..4 {
        assert_eq!(g.players[seat].life, before[seat] - 8, "seat {seat}");
    }
    assert_eq!(g.players[0].life, before[0]);
}

/// Intet's exiled card is free to play while Intet stays, and not after.
#[test]
fn intet_grants_its_exile_only_while_it_remains() {
    let mut g = pod(2);
    yes(&mut g, 1);
    let intet = g.add_card_to_battlefield(0, catalog::intet_the_dreamer());
    let top = g.add_card_to_library(0, catalog::serra_angel());
    flood(&mut g, 0);
    connect(&mut g, intet, 1);
    let card = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(card.may_play_until.is_some(), "playable while Intet remains");
    g.step = TurnStep::PostCombatMain;
    flood(&mut g, 0);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_with(&mut g, murder, Some(Target::Permanent(intet)), vec![]).expect("murder");
    let card = g.exile.iter().find(|c| c.id == top).expect("still exiled");
    assert!(card.may_play_until.is_none(), "Intet left: the permission ended");
}

/// Trench Gorger's base P/T become the number of lands it exiled.
#[test]
fn trench_gorger_sizes_itself_by_the_lands_it_exiles() {
    let mut g = pod(2);
    yes(&mut g, 1);
    for _ in 0..9 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let gorger = g.add_card_to_hand(0, catalog::trench_gorger());
    cast_with(&mut g, gorger, None, vec![]).expect("cast");
    let cp = g.computed_permanent(gorger).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 9));
    assert_eq!(g.players[0].library.len(), 1, "the Bears stay");
}

/// Vengeful Rebirth returns a nonland card and deals its mana value, then
/// exiles itself.
#[test]
fn vengeful_rebirth_deals_the_returned_cards_mana_value() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    flood(&mut g, 0);
    let life = g.players[1].life;
    let rebirth = g.add_card_to_hand(0, catalog::vengeful_rebirth());
    cast_with(&mut g, rebirth, Some(Target::Permanent(angel)), vec![Target::Player(1)]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == angel));
    assert_eq!(g.players[1].life, life - 5);
    assert!(g.exile.iter().any(|c| c.id == rebirth), "exiled, not graveyard");
}

/// Ray of Command steals an opponent's creature untapped with haste and taps it
/// before handing it back.
#[test]
fn ray_of_command_steals_and_taps_on_return() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    flood(&mut g, 0);
    let ray = g.add_card_to_hand(0, catalog::ray_of_command());
    cast_with(&mut g, ray, Some(Target::Permanent(bear)), vec![]).expect("cast");
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.controller, 0);
    assert!(!b.tapped, "untapped");
    for _ in 0..8 {
        if g.step == TurnStep::End {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.step, TurnStep::End);
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped as it goes back");
}

/// Deadwood Treefolk's leave trigger returns *another* creature card, not
/// itself.
#[test]
fn deadwood_treefolk_returns_another_creature_on_leaving() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let tree = g.add_card_to_battlefield(0, catalog::deadwood_treefolk());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast_with(&mut g, murder, Some(Target::Permanent(tree)), vec![]).expect("murder");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the Bears came back");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == tree), "not itself");
}
