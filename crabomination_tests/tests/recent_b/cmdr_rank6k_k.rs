//! Commander: Rose Noble's changelings, EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_k`).

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
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

/// CR 603.4 / 702.73a — Kinseekers counts itself (changeling) with two Bears:
/// three creatures share a type, so it gets the counter; with one Bear it
/// doesn't.
#[test]
fn littjara_kinseekers_needs_three_sharing_a_type() {
    for (bears, want) in [(2, 1), (1, 0)] {
        let mut g = pod(2);
        for _ in 0..bears {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        g.add_card_to_library(0, catalog::island());
        let k = g.add_card_to_hand(0, catalog::littjara_kinseekers());
        cast(&mut g, k, None);
        let n = g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(n, want, "{bears} Bears");
    }
}

/// CR 613.4d — Turtleshell Changeling's switch makes it a 4/1.
#[test]
fn turtleshell_changeling_switches() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::turtleshell_changeling());
    act(
        &mut g,
        GameAction::ActivateAbility { card_id: t, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None },
    );
    let cp = g.computed_permanent(t).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1));
}

/// CR 611.2c — Ego Erasure shrinks only the target player's creatures.
#[test]
fn ego_erasure_shrinks_the_target_players_creatures() {
    let mut g = pod(3);
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    let other = ready(&mut g, 2, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::ego_erasure());
    cast(&mut g, s, Some(Target::Player(1)));
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 1);
    assert_eq!(g.computed_permanent(other).unwrap().power, 3);
    // "… and lose all creature types until end of turn" (CR 613.1d, layer 4).
    assert!(g.computed_permanent(theirs).unwrap().subtypes().creature_types.is_empty());
    assert!(!g.computed_permanent(other).unwrap().subtypes().creature_types.is_empty());
}
