//! Commander: the 2026-10-18 (`019Y6J4R`) sixteenth closer batch
//! (`decks::cmdr_019y_p`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}


fn opposing_bear(g: &mut GameState) -> CardId {
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bear");
    g.priority.player_with_priority = 0;
    bear
}

/// The Raven's Warning I: a flying Bird and 2 life.
#[test]
fn the_ravens_warning_starts_with_a_bird() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_ravens_warning());
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Bird"));
    assert_eq!(g.players[0].life, 22);
}

/// CR 603.2c — The Raven's Warning II: a flyer connecting draws a card.
#[test]
fn the_ravens_warning_draws_when_a_flyer_connects() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_ravens_warning());
    let flyer = ready(&mut g, 0, catalog::aerial_guide());
    g.saga_advance(saga);
    drain_stack(&mut g);
    g.saga_advance(saga);
    drain_stack(&mut g);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: flyer, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Blood on the Snow's creature mode wipes the board; with no snow mana spent
/// (X = 0) only a mana-value-0 creature card could come back.
#[test]
fn blood_on_the_snow_wipes_creatures() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let z = ready(&mut g, 1, catalog::hill_giant());
    let bots = g.add_card_to_hand(0, catalog::blood_on_the_snow());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: bots, target: None, additional_targets: vec![], mode: Some(0), x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(z).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == a), "MV 2 > X = 0: it stays");
}

/// We Say Thee Nay! without teamwork: counter unless {2} — unpaid here.
#[test]
fn we_say_thee_nay_counters_an_unpaid_spell() {
    let mut g = pod(2);
    let bear = opposing_bear(&mut g);
    let nay = g.add_card_to_hand(0, catalog::we_say_thee_nay());
    flood(&mut g, 0);
    cast(&mut g, nay, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
}

/// HULK SMASH! unteamed: one mode — your creature bites theirs.
#[test]
fn hulk_smash_bites() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let hs = g.add_card_to_hand(0, catalog::hulk_smash());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: hs,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(giant).is_some(), "one-sided");
}
