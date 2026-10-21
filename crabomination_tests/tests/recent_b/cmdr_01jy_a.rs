//! Commander: the 2026-10-21 (`01Jywzky`) pair-deck closers
//! (`decks::cmdr_01jy_a`) and the primitives they need.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
}

/// Through the next turn's upkeep, resolving whatever triggers there.
fn to_next_upkeep(g: &mut GameState) {
    let turn = g.turn_number;
    while !(g.step == TurnStep::Upkeep && g.turn_number > turn) {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(g);
}

/// Burnout may target any instant spell and counters it only if it's blue;
/// the delayed draw waits for the next turn's upkeep either way (CR 603.7).
#[test]
fn burnout_counters_only_a_blue_instant_and_draws_next_upkeep() {
    for (blue, spell) in [(true, catalog::opt as fn() -> _), (false, catalog::lightning_bolt)] {
        let mut g = pod(2);
        let instant = g.add_card_to_hand(1, spell());
        cast_as(&mut g, 1, instant, (!blue).then_some(Target::Player(0)));
        let burnout = g.add_card_to_hand(0, catalog::burnout());
        cast_as(&mut g, 0, burnout, Some(Target::Permanent(instant)));
        let life = g.players[0].life;
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == instant));
        assert_eq!(g.players[0].life == life, blue, "only the blue instant is countered");
        let hand = g.players[0].hand.len();
        to_next_upkeep(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "the draw lands at the next turn's upkeep");
    }
}

/// Mana Short taps every land the target player controls and empties their
/// pool — even mana Upwelling would keep past a step's end (CR 500.4 governs
/// only the step-end empty, not an effect).
#[test]
fn mana_short_taps_lands_and_empties_the_pool() {
    let mut g = pod(3);
    let land = g.add_card_to_battlefield(1, catalog::island());
    let other = g.add_card_to_battlefield(2, catalog::island());
    g.add_card_to_battlefield(1, catalog::upwelling());
    g.players[1].mana_pool.add(Color::Green, 4);
    let short = g.add_card_to_hand(0, catalog::mana_short());
    cast_as(&mut g, 0, short, Some(Target::Player(1)));
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
    assert!(!g.battlefield_find(other).unwrap().tapped, "only the target player's lands");
    assert_eq!(g.players[1].mana_pool.total(), 0);
}
