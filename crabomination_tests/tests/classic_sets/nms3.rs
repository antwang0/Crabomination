//! Nemesis (NMS), third wave.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Parallax Wave exiles per fade counter and gives everything back when it
/// leaves.
#[test]
fn parallax_wave_exiles_then_returns_on_leave() {
    let mut g = main_phase();
    let wave = g.add_card_to_hand(0, catalog::parallax_wave());
    cast(&mut g, 0, wave, None);
    assert_eq!(g.battlefield_find(wave).unwrap().counter_count(CounterType::Fade), 5);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, wave, 0, Some(Target::Permanent(victim)));
    assert!(g.exile.iter().any(|c| c.id == victim));
    assert_eq!(g.battlefield_find(wave).unwrap().counter_count(CounterType::Fade), 4);

    g.sacrifice_one(wave, 0, &mut vec![]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).map(|c| c.controller), Some(1), "back to its owner");
}

/// Parallax Inhibitor re-charges every fading permanent you control.
#[test]
fn parallax_inhibitor_recharges_fading_permanents() {
    let mut g = main_phase();
    let wave = g.add_card_to_hand(0, catalog::parallax_wave());
    cast(&mut g, 0, wave, None);
    let inhibitor = g.add_card_to_battlefield(0, catalog::parallax_inhibitor());
    activate(&mut g, 0, inhibitor, 0, None);
    assert_eq!(g.battlefield_find(wave).unwrap().counter_count(CounterType::Fade), 6);
    assert!(g.battlefield_find(inhibitor).is_none(), "sacrificed itself");
}

/// Accumulated Knowledge draws one more per copy already in a graveyard.
#[test]
fn accumulated_knowledge_counts_its_own_copies() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::accumulated_knowledge());
    g.add_card_to_graveyard(1, catalog::accumulated_knowledge());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ak = g.add_card_to_hand(0, catalog::accumulated_knowledge());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, ak, None);
    // 1 + the two copies already in graveyards (the caster's own goes to the
    // graveyard on resolution, after the count).
    assert_eq!(g.players[0].hand.len(), before - 1 + 3);
}

/// Pack Hunt fetches copies of whatever it targets.
#[test]
fn pack_hunt_fetches_same_named_cards() {
    let mut g = main_phase();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let copy = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hunt = g.add_card_to_hand(0, catalog::pack_hunt());
    script(&mut g, vec![DecisionAnswer::Search(Some(copy))]);
    cast(&mut g, 0, hunt, Some(Target::Permanent(target)));
    assert!(g.players[0].hand.iter().any(|c| c.id == copy));
}

/// Mind Slash trades a creature for a hand-picked discard.
#[test]
fn mind_slash_trades_a_creature_for_a_discard() {
    let mut g = main_phase();
    let slash = g.add_card_to_battlefield(0, catalog::mind_slash());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    activate(&mut g, 0, slash, 0, Some(Target::Player(1)));
    assert!(g.battlefield_find(fodder).is_none(), "the creature paid for it");
    assert!(g.players[1].hand.is_empty());
}

/// Rising Waters locks lands down but hands one back each upkeep.
#[test]
fn rising_waters_locks_lands_and_untaps_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rising_waters());
    let a = g.add_card_to_battlefield(0, catalog::island());
    let b = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    g.battlefield_find_mut(b).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(g.battlefield_find(a).unwrap().tapped, "the untap step is blanked");
    assert!(g.battlefield_find(b).unwrap().tapped);

    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let untapped = [a, b].iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count();
    assert_eq!(untapped, 1, "exactly one land comes back");
}
