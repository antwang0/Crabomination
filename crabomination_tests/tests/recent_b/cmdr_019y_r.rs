//! Commander: the 2026-10-18 (`019Y6J4R`) eighteenth closer batch
//! (`decks::cmdr_019y_r`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
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




use crabomination::game::types::{GameAction, TurnStep};
use crabomination::effect::{Effect, Selector};
use crabomination::card::Keyword;

/// CR 702.62b — Veiling Oddity's exile trigger sees its last time counter come
/// off (the card is cast off it): creatures can't be blocked this turn.
#[test]
fn veiling_oddity_unblocks_on_its_last_counter() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let vo = g.add_card_to_graveyard(0, catalog::veiling_oddity());
    let ev = g
        .resolve_effect(
            &Effect::GrantSuspend { what: Selector::ExactObjects(vec![vo]), time_counters: 1 },
            &EffectContext::for_spell(0, None, 0, 0),
        )
        .unwrap();
    g.dispatch_triggers_for_events(&ev);
    let ev = g.process_suspend();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(vo).is_some(), "cast off its last counter");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// A time counter that isn't the last doesn't fire it.
#[test]
fn veiling_oddity_waits_for_the_last_counter() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let vo = g.add_card_to_graveyard(0, catalog::veiling_oddity());
    let ev = g
        .resolve_effect(
            &Effect::GrantSuspend { what: Selector::ExactObjects(vec![vo]), time_counters: 3 },
            &EffectContext::for_spell(0, None, 0, 0),
        )
        .unwrap();
    g.dispatch_triggers_for_events(&ev);
    let ev = g.process_suspend();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// CR 702.29e — Jhessian Zombies swampcycles for a Swamp.
#[test]
fn jhessian_zombies_landcycles() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let swamp = g.add_card_to_library(0, catalog::swamp());
    let jz = g.add_card_to_hand(0, catalog::jhessian_zombies());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Landcycle { card_id: jz }).expect("swampcycle");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == swamp));
}

#[allow(dead_code)]
fn _unused(_: CounterType, _: TurnStep) {}
