//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_j`).

use crabomination::card::{CardId, CounterType};
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
    g
}

fn flood(g: &mut GameState) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}
fn prepare(g: &mut GameState, creature_id: CardId) {
    act(
        g,
        GameAction::CastPrepareSpell {
            creature_id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

/// CR 702.108b — Ruric Thar's two prowess instances each trigger; an
/// opponent's targeting draws a card.
#[test]
fn ruric_thar_double_prowess_and_target_draw() {
    let mut g = pod(2);
    let ruric = ready(&mut g, 0, catalog::ruric_thar_biomagus());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, None);
    assert_eq!(pt(&g, ruric), (6, 8));
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(ruric)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 702.188 — Variable Chaser's Arc of Fortune lets each player wheel.
#[test]
fn variable_chaser_arc_of_fortune_wheels() {
    let mut g = pod(2);
    let chaser = g.add_card_to_hand(0, catalog::variable_chaser());
    cast(&mut g, chaser, None);
    assert_eq!(
        g.battlefield_find(chaser)
            .unwrap()
            .counter_count(CounterType::Prepared),
        1
    );
    for seat in 0..2 {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::island());
        }
        g.add_card_to_hand(seat, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    prepare(&mut g, chaser);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(
        g.players[1].hand.len(),
        1,
        "the other seat declined (it's a may)"
    );
    assert_eq!(
        g.battlefield_find(chaser)
            .unwrap()
            .counter_count(CounterType::Prepared),
        0
    );
}

/// CR 106.12 — Pyre Rhymer's Molten Tide doubles Mountain mana this turn.
#[test]
fn pyre_rhymer_molten_tide_adds_red() {
    let mut g = pod(2);
    let rhymer = g.add_card_to_hand(0, catalog::pyre_rhymer());
    cast(&mut g, rhymer, None);
    prepare(&mut g, rhymer);
    let mountain = ready(&mut g, 0, catalog::mountain());
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mountain,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap mountain");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// CR 701.13 — Identity Echo trades a creature for the next one in the library.
#[test]
fn identity_echo_swaps_a_creature() {
    let mut g = pod(2);
    let echo = ready(&mut g, 0, catalog::identity_echo());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, echo, 0, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert!(g.battlefield_find(angel).is_some());
    assert_eq!(g.players[0].library.len(), 2, "misses go to the bottom");
}
