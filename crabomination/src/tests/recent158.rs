//! Functionality tests for `catalog::sets::decks::recent158`.

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Starlit Soothsayer surveils at end step only if you gained or lost life.
#[test]
fn starlit_soothsayer_surveils_after_life_change() {
    let mut g = two_player_game();
    let sooth = g.add_card_to_battlefield(0, catalog::starlit_soothsayer());
    g.clear_sickness(sooth);
    g.players[0].library.clear();
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::island());
    g.players[0].life_gained_this_turn = 2;
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![top],
    }]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveiled the top card to gy");
}

/// Omenport Vigilante gains double strike only after you commit a crime.
#[test]
fn omenport_vigilante_double_strike_on_crime() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::omenport_vigilante());
    assert!(!g.computed_permanent(v).unwrap().keywords.contains(&Keyword::DoubleStrike));
    g.players[0].committed_crime_this_turn = true;
    assert!(g.computed_permanent(v).unwrap().keywords.contains(&Keyword::DoubleStrike), "crime → double strike");
}

/// Essence Channeler flies after you lose life, and grows on lifegain.
#[test]
fn essence_channeler_lost_life_flying_and_grows() {
    let mut g = two_player_game();
    let ec = g.add_card_to_battlefield(0, catalog::essence_channeler());
    assert!(!g.computed_permanent(ec).unwrap().keywords.contains(&Keyword::Flying));
    g.players[0].lost_life_this_turn = true;
    let c = g.computed_permanent(ec).unwrap();
    assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Vigilance), "lost life → flying + vigilance");
    // Gaining life adds a +1/+1 counter.
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ec).unwrap().power, 3, "lifegain grew it");
}

/// Cactarantula draws when an opponent targets it.
#[test]
fn cactarantula_draws_on_opponent_target() {
    let mut g = two_player_game();
    let cact = g.add_card_to_battlefield(0, catalog::cactarantula());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: cact, caster: 1 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off an opponent's target");
}

/// Inventive Wingsmith wings itself at end step if you cast no spells.
#[test]
fn inventive_wingsmith_gets_flying_counter() {
    let mut g = two_player_game();
    let smith = g.add_card_to_battlefield(0, catalog::inventive_wingsmith());
    g.clear_sickness(smith);
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.computed_permanent(smith).unwrap().keywords.contains(&Keyword::Flying), "gained a flying counter");
}

/// Mourner's Surprise returns a creature card and mints a Mercenary.
#[test]
fn mourners_surprise_reanimates_to_hand_and_makes_token() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mourners_surprise());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mourner's Surprise");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned the creature card to hand");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Mercenary" && c.controller == 0), "made a Mercenary");
}
