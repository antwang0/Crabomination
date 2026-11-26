//! EDHREC singles short only their commander (`decks::cmdr_nov26`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn cast(g: &mut GameState, id: CardId) {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// Attack `defender` with `attacker`, no blocks, through combat damage.
fn hit(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    for _ in 0..64 {
        if matches!(g.step, TurnStep::EndCombat | TurnStep::PostCombatMain) {
            drain_stack(g);
            return;
        }
        let r = g.perform_action(GameAction::PassPriority);
        if r.is_err() && g.step == TurnStep::DeclareBlockers {
            g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
        }
    }
    panic!("combat never ended: step {:?}, stack {}, pending {:?}", g.step, g.stack.len(), g.pending_decision.is_some());
}

/// Massimo: a creature entering exiles an MV-1 instant from your graveyard and
/// gains "copy it on combat damage to a player"; the copy is cast free. An MV-2
/// instant is not a legal target, so nothing is granted.
#[test]
fn massimo_links_an_exiled_cantrip_to_the_entering_creature() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::massimo_the_magician());
    let opt = g.add_card_to_graveyard(0, catalog::opt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != opt), "Opt exiled");
    assert_eq!(g.exile.iter().find(|c| c.id == opt).and_then(|c| c.exiled_with), Some(bear), "linked to the Bears");
    g.clear_sickness(bear);
    let (hand, life) = (g.players[0].hand.len(), g.players[2].life);
    hit(&mut g, bear, 2);
    assert_eq!(g.players[2].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the free Opt copy drew a card");
    assert!(g.exile.iter().any(|c| c.id == opt), "the original stays exiled");

    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::massimo_the_magician());
    let cn = g.add_card_to_graveyard(0, catalog::counterspell());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cn), "MV 2: not a target");
    assert!(g.battlefield_find(bear).is_some_and(|c| c.definition.triggered_abilities.is_empty()));
}
