//! Commander: two-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_h`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn tap_for_mana(g: &mut GameState, land: CardId) {
    let (idx, _) = g.effective_mana_abilities(land).into_iter().next().expect("mana ability");
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for mana");
}

/// CR 702.41a — Utrom Monitor costs {1} less per artifact you control.
#[test]
fn utrom_monitor_has_affinity_for_artifacts() {
    let mut g = pod(2);
    for _ in 0..4 {
        ready(&mut g, 0, catalog::sol_ring());
    }
    let um = g.add_card_to_hand(0, catalog::utrom_monitor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: um, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{U} after four artifacts");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == um));
}

/// CR 605.1b / 106.12a — Keeper of Progenitus mirrors a Forest's mana for
/// any player, and leaves an Island alone.
#[test]
fn keeper_of_progenitus_doubles_forests_not_islands() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::keeper_of_progenitus());
    let forest = ready(&mut g, 1, catalog::forest());
    let island = ready(&mut g, 1, catalog::island());
    g.priority.player_with_priority = 1;
    tap_for_mana(&mut g, forest);
    assert_eq!(g.players[1].mana_pool.amount(Color::Green), 2);
    tap_for_mana(&mut g, island);
    assert_eq!(g.players[1].mana_pool.amount(Color::Blue), 1);
}
