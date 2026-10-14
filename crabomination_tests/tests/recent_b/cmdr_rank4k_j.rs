//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_j`).

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
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}


/// Karn, Argent Defender: an entering creature or artifact triggers nothing,
/// its own ETB included.
#[test]
fn karn_argent_defender_silences_arrivals() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::karn_argent_defender());
    let kiora = g.add_card_to_hand(0, catalog::kiora_of_fire_and_ashes());
    cast(&mut g, kiora, None, None);
    assert_eq!(named(&g, 0, "Dragon"), 0);
}

/// Ward of Bones: an opponent ahead of its controller on creatures can't cast
/// a creature spell, and one ahead on lands can't play a land; the controller
/// itself is never bound.
#[test]
fn ward_of_bones_binds_the_opponent_ahead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ward_of_bones());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::forest());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "one creature to none: no creature spell"
    );
    assert!(!g.can_player_play_land(1), "one land to none: no land drop");
    assert!(g.can_player_play_land(0));
    let ring = g.add_card_to_hand(1, catalog::sol_ring());
    g.players[1].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: ring,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok(),
        "behind on artifacts (the Ward is one), an artifact spell is free to cast"
    );
}
