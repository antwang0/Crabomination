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

/// CR 701.20 — Chaotic Transformation exiles an opposing creature; its
/// controller reveals to their first creature card and puts it onto the
/// battlefield, the misses shuffled back.
#[test]
fn chaotic_transformation_swaps_a_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::forest());
    let giant = g.add_card_to_library(1, catalog::hill_giant());
    let ct = g.add_card_to_hand(0, catalog::chaotic_transformation());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 6);
    g.perform_action(GameAction::CastSpell {
        card_id: ct,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the Bear was exiled");
    assert_eq!(g.battlefield_find(giant).expect("the Giant came in").controller, 1);
    assert_eq!(g.players[1].library.len(), 1, "the Forest went back");
}

/// CR 115.3 / 601.2c — Chaotic Transformation's targets take distinct types:
/// two creatures can't both be named (no second creature slot), but a creature
/// and a land can.
#[test]
fn chaotic_transformation_takes_one_of_each_type() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::hill_giant());
    let land = ready(&mut g, 1, catalog::forest());
    let ct = g.add_card_to_hand(0, catalog::chaotic_transformation());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 12);
    let two_creatures = g.perform_action(GameAction::CastSpell {
        card_id: ct,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    });
    assert!(two_creatures.is_err(), "only one creature slot");
    g.perform_action(GameAction::CastSpell {
        card_id: ct,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    })
    .expect("a creature and a land");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == land));
}

/// Fishing Pole: the granted ability taps creature and Pole for a bait
/// counter; the creature untapping spends it on a Fish.
#[test]
fn fishing_pole_catches_a_fish() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let pole = ready(&mut g, 0, catalog::fishing_pole());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::Equip { equipment: pole, target: bear }).expect("equip");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("bait");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped && g.battlefield_find(pole).unwrap().tapped);
    assert_eq!(g.battlefield_find(pole).unwrap().counter_count(CounterType::Bait), 1);
    let untap = crabomination::effect::Effect::Untap { what: Selector::ExactObjects(vec![bear]), up_to: None };
    let evs = g.resolve_effect(&untap, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pole).unwrap().counter_count(CounterType::Bait), 0);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Fish"));
}
