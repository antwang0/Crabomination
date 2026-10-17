//! Commander: the 2026-10-17 (`3chree`) one-card-gap closers
//! (`decks::cmdr_oct17_q`).

use crabomination::card::{CardId, CounterType, CreatureType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 2);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

/// Dust of Moments: both modes reach suspended cards in any exile and
/// permanents with time counters (CR 702.62).
#[test]
fn dust_of_moments_moves_time_counters_both_ways() {
    let mut g = pod(2);
    let vision = g.add_card_to_exile(1, catalog::ancestral_vision());
    g.exile.iter_mut().find(|c| c.id == vision).unwrap().add_counters(CounterType::Time, 4);
    let bloom = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bloom).unwrap().add_counters(CounterType::Time, 3);
    let plain = ready(&mut g, 1, catalog::grizzly_bears());
    let time = |g: &GameState, id| {
        g.battlefield_find(id).or_else(|| g.exile.iter().find(|c| c.id == id)).unwrap().counter_count(CounterType::Time)
    };
    let d = g.add_card_to_hand(0, catalog::dust_of_moments());
    flood(&mut g, 0);
    cast(&mut g, d, None, Some(0)).expect("remove");
    assert_eq!((time(&g, vision), time(&g, bloom)), (2, 1));
    let d = g.add_card_to_hand(0, catalog::dust_of_moments());
    flood(&mut g, 0);
    cast(&mut g, d, None, Some(1)).expect("add");
    assert_eq!((time(&g, vision), time(&g, bloom), time(&g, plain)), (4, 3, 0));
}

/// On Thin Ice holds an opponent's creature until the Aura leaves.
#[test]
fn on_thin_ice_exiles_until_it_leaves() {
    let mut g = pod(2);
    let land = ready(&mut g, 0, catalog::snow_covered_plains());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ice = g.add_card_to_hand(0, catalog::on_thin_ice());
    flood(&mut g, 0);
    cast(&mut g, ice, Some(Target::Permanent(land)), None).expect("ice");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear));
    let wipe = g.add_card_to_hand(0, catalog::disenchant());
    flood(&mut g, 0);
    cast(&mut g, wipe, Some(Target::Permanent(ice)), None).expect("disenchant");
    assert!(g.battlefield_find(bear).is_some(), "the bear returns");
}

/// Dawn-Blessed Pennant: life per entering permanent of the chosen type, and
/// the sacrifice returns a card of that type (the choice survives the cost).
#[test]
fn dawn_blessed_pennant_tracks_its_chosen_type() {
    let mut g = pod(2);
    let p = g.add_card_to_hand(0, catalog::dawn_blessed_pennant());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    cast(&mut g, p, None, None).expect("pennant");
    let life = g.players[0].life;
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    flood(&mut g, 0);
    cast(&mut g, elf, None, None).expect("elves");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None, None).expect("bears");
    assert_eq!(g.players[0].life, life + 1);
    let dead_elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p,
        ability_index: 0,
        target: Some(Target::Permanent(dead_elf)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
        .expect("pennant activation");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead_elf));
}

/// The Flame of Keld's chapter III: a red source deals +2 this turn, to your
/// own permanents too (CR 614.1a); the bonus ends with the turn.
#[test]
fn flame_of_keld_chapter_three_adds_two() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_flame_of_keld());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    let ogre = ready(&mut g, 1, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::shock());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(ogre)), None).expect("shock");
    assert!(g.battlefield_find(ogre).is_none(), "2 + 2 kills the 3/3");
    assert_eq!(g.players[0].red_source_damage_bonus_this_turn, 2);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert_eq!(g.players[0].red_source_damage_bonus_this_turn, 0);
}

/// Celestial Reunion (CR 701.4a): beholding two Elves puts a found Elf onto
/// the battlefield; unbeheld, or a non-Elf, it goes to hand.
#[test]
fn celestial_reunion_beholds_for_the_battlefield() {
    for (kicked, find_elf, on_field) in [(true, true, true), (false, true, false), (true, false, false)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::llanowar_elves());
        let elf2 = g.add_card_to_hand(0, catalog::llanowar_elves());
        let id = g.next_id();
        let def = if find_elf { catalog::elvish_mystic() } else { catalog::grizzly_bears() };
        g.players[0].add_to_library_top(id, def);
        let s = g.add_card_to_hand(0, catalog::celestial_reunion());
        flood(&mut g, 0);
        let action = if kicked {
            GameAction::CastSpellKicked { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: Some(2) }
        } else {
            GameAction::CastSpell { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: Some(2) }
        };
        g.perform_action(action).expect("reunion");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).is_some(), on_field, "kicked {kicked} elf {find_elf}");
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == id), !on_field);
        assert!(g.players[0].hand.iter().any(|c| c.id == elf2), "behold reveals, nothing moves");
    }
}

/// March of Reckless Joy: X=3 exiles three; the first two plays go through,
/// the third grant is spent.
#[test]
fn march_of_reckless_joy_plays_up_to_two() {
    let mut g = pod(2);
    let ids: Vec<CardId> = (0..3)
        .map(|_| {
            let id = g.next_id();
            g.players[0].add_to_library_top(id, catalog::grizzly_bears());
            id
        })
        .collect();
    let m = g.add_card_to_hand(0, catalog::march_of_reckless_joy());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: m, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("march");
    drain_stack(&mut g);
    assert!(ids.iter().all(|id| g.exile.iter().any(|c| c.id == *id)));
    for (i, id) in ids.iter().enumerate() {
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        let r = g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: *id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        drain_stack(&mut g);
        assert_eq!(r.is_ok(), i < 2, "play {i}");
    }
}
