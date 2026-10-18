//! Commander: the 2026-10-18 (`019Y6J4R`) nineteenth closer batch
//! (`decks::cmdr_019y_s`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}


/// CR 702.41a — Don & Raph's attack arms affinity for the next noncreature
/// spell: a creature spell cast first pays full, then Lotus Ring costs {0}
/// with three artifacts out.
#[test]
fn don_and_raph_give_the_next_noncreature_spell_affinity() {
    let mut g = pod(2);
    let dr = ready(&mut g, 0, catalog::don_raph_hard_science());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::fishing_pole());
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: dr, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::PostCombatMain;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("a creature spell pays its full cost");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0);
    let ring = g.add_card_to_hand(0, catalog::lotus_ring());
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("three artifacts pay for Lotus Ring");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
}

#[allow(dead_code)]
fn _unused(_: CounterType) {}
