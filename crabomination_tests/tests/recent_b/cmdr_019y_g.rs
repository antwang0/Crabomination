//! Commander: the 2026-10-17 (`019Y6J4R`) seventh closer batch
//! (`decks::cmdr_019y_g`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

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













fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}




/// Alaundo exiles a card with its mana value in time counters (no suspend: no
/// upkeep tick), and each activation takes one off the others — the last one
/// casts it free.
#[test]
fn alaundo_ticks_its_exiled_cards_down_to_a_free_cast() {
    let mut g = pod(2);
    let al = ready(&mut g, 0, catalog::alaundo_the_seer());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].hand.retain(|c| c.id == bears);
    // Draw a land first so the Bears (MV 2) is the priciest card in hand.
    activate(&mut g, al, 0, None).expect("activate");
    let c = g.exile.iter().find(|c| c.id == bears).expect("the Bears was exiled");
    assert_eq!(c.counter_count(CounterType::Time), 2);
    g.step = TurnStep::Upkeep;
    g.process_suspend();
    assert_eq!(g.exile.iter().find(|c| c.id == bears).unwrap().counter_count(CounterType::Time), 2, "no suspend tick");
    g.step = TurnStep::PreCombatMain;
    for _ in 0..2 {
        g.battlefield_find_mut(al).unwrap().tapped = false;
        activate(&mut g, al, 0, None).expect("activate");
    }
    assert!(g.battlefield_find(bears).is_some(), "the last counter cast it");
}

/// Corrupted Grafstone taps for a color of a card in your graveyard only.
#[test]
fn corrupted_grafstone_reads_your_graveyard_colors() {
    let mut g = pod(2);
    let cg = ready(&mut g, 0, catalog::corrupted_grafstone());
    g.battlefield_find_mut(cg).unwrap().tapped = false;
    activate(&mut g, cg, 0, None).expect("activate");
    assert_eq!(g.players[0].mana_pool.total(), 0, "empty graveyard, no mana");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.battlefield_find_mut(cg).unwrap().tapped = false;
    activate(&mut g, cg, 0, None).expect("activate");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}
