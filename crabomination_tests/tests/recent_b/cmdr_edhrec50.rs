//! Commander: Borborygmos and Fblthp's last gaps (`decks::cmdr_edhrec50`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, x: Option<u32>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: x })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Borborygmos and Fblthp enters, loots, and two discarded lands become four
/// damage to a creature.
#[test]
fn borborygmos_turns_discarded_lands_into_damage() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.players[0].hand.clear();
    let lands = [g.add_card_to_hand(0, catalog::forest()), g.add_card_to_hand(0, catalog::island())];
    let bf = g.add_card_to_hand(0, catalog::borborygmos_and_fblthp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(lands.to_vec())]));
    cast_x(&mut g, bf, None).expect("cast");
    assert!(lands.iter().all(|l| g.players[0].graveyard.iter().any(|c| c.id == *l)));
    assert!(g.battlefield_find(giant).is_none(), "four damage to the 3/3");
}

/// CR 104.2b — Fblthp, Impossibly Lost draws two off combat damage during your
/// turn and wins on an empty library; either way he shuffles back in.
#[test]
fn cr_104_2b_fblthp_wins_from_an_empty_library() {
    let mut g = pod(2);
    let fblthp = g.add_card_to_battlefield(0, catalog::fblthp_impossibly_lost());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    for _ in 0..30 {
        if g.is_game_over() || g.step == TurnStep::EndCombat {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(fblthp).is_none() || g.is_game_over());
    assert!(g.is_game_over(), "two drawn from two: an empty library wins");
}

/// Fblthp, Knows the Way fetches X basic lands and counts basic land types.
#[test]
fn fblthp_knows_the_way_fetches_x_basics() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::island(), catalog::mountain()] {
        g.add_card_to_library(0, land);
    }
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    g.players[0].hand.clear();
    let f = g.add_card_to_hand(0, catalog::fblthp_knows_the_way());
    g.decider = Box::new(ScriptedDecider::new([]));
    cast_x(&mut g, f, Some(2)).expect("X=2");
    assert_eq!(g.computed_permanent(f).unwrap().power, 2, "Forest and Island");
}

/// Fblthp's basics must have different names: two Forests and an Island in
/// the library, X = 2, fetch one of each.
#[test]
fn fblthp_knows_the_way_fetches_different_names() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for land in [catalog::forest(), catalog::forest(), catalog::island()] {
        g.add_card_to_library(0, land);
    }
    g.players[0].hand.clear();
    let f = g.add_card_to_hand(0, catalog::fblthp_knows_the_way());
    g.decider = Box::new(ScriptedDecider::new([]));
    cast_x(&mut g, f, Some(2)).expect("X=2");
    let mut names: Vec<&str> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    names.sort();
    assert_eq!(names, ["Forest", "Island"]);
}
