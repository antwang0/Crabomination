//! Commander: the 2026-10-20 (`01DDeUtc`) closers (`decks::cmdr_01dd_*`).

use crabomination::card::{CardId, CardType, CreatureType, Subtypes, TokenDefinition};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn beast(power: i32) -> TokenDefinition {
    TokenDefinition {
        name: "Beast".into(),
        power,
        toughness: power,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    }
}

fn treasures(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Treasure").count()
}

/// CR 702.110a-b — Henry Wu grants exploit to a Human entering; exploiting a
/// non-Human with power 3 draws a card and makes a Treasure.
#[test]
fn cr_702_110b_henry_wu_pays_for_a_big_non_human() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::henry_wu_ingen_geneticist());
    let token = g.add_token_to_battlefield(0, &beast(3));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, vanguard, None).expect("cast");
    assert!(g.battlefield_find(token).is_none(), "the Beast was exploited");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(treasures(&g, 0), 1, "power 3 or greater");
}

/// CR 702.110b — a non-Human with power under 3 draws without a Treasure.
#[test]
fn henry_wu_small_non_human_draws_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::henry_wu_ingen_geneticist());
    g.add_token_to_battlefield(0, &beast(2));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, vanguard, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(treasures(&g, 0), 0);
}

/// CR 702.110b — Skull Skaab's Zombie needs a nontoken exploit: a token
/// exploited makes nothing, a Grizzly Bears exploited makes a 2/2 Zombie.
#[test]
fn cr_702_110b_skull_skaab_needs_a_nontoken_creature() {
    let zombies = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Zombie" && c.is_token).count();
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &beast(1));
    let skaab = g.add_card_to_hand(0, catalog::skull_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, skaab, None).expect("cast");
    assert_eq!(zombies(&g), 0, "a token exploited");

    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let skaab = g.add_card_to_hand(0, catalog::skull_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, skaab, None).expect("cast");
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(zombies(&g), 1, "a nontoken creature exploited");
}
