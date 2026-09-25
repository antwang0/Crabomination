//! Commander: the Faceless Menace precon (C19, Kadena, `decks::cmdr_kadena`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Kadena (CR 903.3): the first face-down spell each turn costs {3} less
/// (CR 702.37c), and a face-down creature entering draws.
#[test]
fn kadena_discounts_and_draws() {
    assert!(catalog::kadena_slinking_sorcerer().can_be_commander);
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kadena_slinking_sorcerer());
    assert_eq!(g.face_down_cast_cost(0), 0);
    g.add_card_to_library(0, catalog::island());
    let aven = g.add_card_to_hand(0, catalog::icefeather_aven());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastFaceDown { card_id: aven }).expect("free morph");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(g.face_down_cast_cost(0), 3, "the second pays full");
}

/// Rayami exiles any nontoken creature that would die with a blood counter
/// (CR 614) and takes its keywords.
#[test]
fn rayami_collects_the_fallen() {
    let mut g = pod(2);
    let r = g.add_card_to_battlefield(0, catalog::rayami_first_of_the_fallen());
    let aven = g.add_card_to_battlefield(1, catalog::icefeather_aven());
    let m = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast_by(&mut g, 0, m, &[Target::Permanent(aven)]);
    let exiled = g.exile.iter().find(|c| c.id == aven).expect("exiled, not dead");
    assert_eq!(exiled.counter_count(CounterType::Blood), 1);
    assert!(g.computed_permanent(r).unwrap().keywords().contains(&Keyword::Flying));
}

/// Leadership Vacuum sends a commander home (CR 903.9).
#[test]
fn leadership_vacuum_sends_commanders_home() {
    let mut g = pod(2);
    let cmdr = g.add_card_to_battlefield(1, catalog::kadena_slinking_sorcerer());
    g.players[1].commanders.push(cmdr);
    let lv = g.add_card_to_hand(0, catalog::leadership_vacuum());
    flood(&mut g, 0);
    cast_by(&mut g, 0, lv, &[Target::Player(1)]);
    assert!(g.battlefield_find(cmdr).is_none());
    assert!(g.players[1].command.iter().any(|c| c.id == cmdr));
}

/// Kadena's Silencer, turned face up, counters every opponent ability.
#[test]
fn kadenas_silencer_silences() {
    let mut g = pod(2);
    let ks = g.add_card_to_hand(0, catalog::kadenas_silencer());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: ks }).expect("morph");
    drain_stack(&mut g);
    g.add_card_to_battlefield(1, catalog::bellowing_mauler());
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    assert!(!g.stack.is_empty(), "the Mauler's trigger waits");
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: ks }).expect("unmorph");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, g.players[0].starting_life, "the Mauler's toll was countered");
}

/// Sudden Substitution swaps an opponent's spell for your creature.
#[test]
fn sudden_substitution_swaps() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let div = g.add_card_to_hand(1, catalog::divination());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: div,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("divination");
    let ss = g.add_card_to_hand(0, catalog::sudden_substitution());
    flood(&mut g, 0);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    cast_by(&mut g, 0, ss, &[Target::Permanent(div), Target::Permanent(mine)]);
    assert_eq!(g.battlefield_find(mine).map(|c| c.controller), Some(1));
    assert_eq!(g.players[0].hand.len(), hand + 1, "we drew Divination's two, less the Substitution");
}

/// Thought Sponge enters as big as an opponent's draws this turn.
#[test]
fn thought_sponge_soaks_up_draws() {
    let mut g = pod(2);
    g.players[1].cards_drawn_this_turn = 3;
    let ts = g.add_card_to_hand(0, catalog::thought_sponge());
    flood(&mut g, 0);
    cast_by(&mut g, 0, ts, &[]);
    assert_eq!(g.battlefield_find(ts).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Bounty of the Luxa alternates a draw and three mana.
#[test]
fn bounty_of_the_luxa_alternates() {
    let mut g = pod(2);
    let b = g.add_card_to_battlefield(0, catalog::bounty_of_the_luxa());
    g.add_card_to_library(0, catalog::island());
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Flood), 1);
    let before = g.players[0].mana_pool.total();
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), before + 3);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Flood), 0);
}

/// Grismold grows from a dying token; each player gets a Plant at your end.
#[test]
fn grismold_sows_and_reaps() {
    let mut g = pod(2);
    let gr = g.add_card_to_battlefield(0, catalog::grismold_the_dreadsower());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let plant = g.battlefield.iter().find(|c| c.controller == 1 && c.definition.name == "Plant").map(|c| c.id);
    let m = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    g.step = TurnStep::PreCombatMain;
    cast_by(&mut g, 0, m, &[Target::Permanent(plant.expect("a Plant"))]);
    assert_eq!(g.battlefield_find(gr).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}
