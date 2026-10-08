//! Commander: most-built commanders seated from their EDHREC average decks
//! (`decks::cmdr_most_built`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn activate(g: &mut GameState, id: CardId, index: usize) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 603.2c — Ob Nixilis: three opponents each losing 1 life at once (Mount
/// Doom's ping) is ONE batch, so one trigger: one counter, one card exiled.
#[test]
fn ob_nixilis_triggers_once_for_a_batch_of_single_losses() {
    let mut g = pod(4);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let doom = ready(&mut g, 0, catalog::mount_doom());
    let library = g.players[0].library.len();
    activate(&mut g, doom, 1);
    assert_eq!([g.players[1].life, g.players[2].life, g.players[3].life], [19, 19, 19]);
    assert_eq!(plus_ones(&g, ob), 1);
    assert_eq!(g.players[0].library.len(), library - 1, "one impulse-exiled card");
}

/// Ruling 2023-05-12 — combat damage to a player is read in total: two 1/1s
/// connecting with one opponent is a loss of 2, so no trigger; one 1/1 on
/// each of two opponents is a loss of exactly 1 each, one trigger.
#[test]
fn ob_nixilis_reads_a_players_total_combat_loss() {
    let mut g = pod(3);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let a = ready(&mut g, 0, catalog::memnite());
    let b = ready(&mut g, 0, catalog::memnite());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(plus_ones(&g, ob), 0, "a loss of 2 is not exactly 1");

    let mut g = pod(3);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let a = ready(&mut g, 0, catalog::memnite());
    let b = ready(&mut g, 0, catalog::memnite());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
    assert_eq!(plus_ones(&g, ob), 1, "both opponents lost exactly 1 in one batch");
}

/// Mount Doom's last ability: the chosen two survive (Ob Nixilis and the
/// costliest by the headless pick), every other creature is destroyed, and
/// the land and a legendary artifact are the cost.
#[test]
fn mount_doom_keeps_up_to_two_creatures_and_destroys_the_rest() {
    let mut g = pod(3);
    let doom = ready(&mut g, 0, catalog::mount_doom());
    let mox = ready(&mut g, 0, catalog::mox_amber());
    for seat in 0..3 {
        ready(&mut g, seat, catalog::grizzly_bears());
        ready(&mut g, seat, catalog::grizzly_bears());
    }
    activate(&mut g, doom, 2);
    let creatures = g.battlefield.iter().filter(|c| c.definition.is_creature()).count();
    assert_eq!(creatures, 2);
    assert!(g.battlefield_find(doom).is_none() && g.battlefield_find(mox).is_none());
}

/// Manabarbs — each land tapped for mana is its own 1 damage to the player
/// who tapped it (2009-10-01 rulings), whoever controls the enchantment.
#[test]
fn manabarbs_burns_whoever_taps_a_land() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::manabarbs());
    let m1 = ready(&mut g, 1, catalog::mountain());
    let m2 = ready(&mut g, 1, catalog::mountain());
    for land in [m1, m2] {
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
    }
    g.priority.player_with_priority = 1;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20);
}

/// Shadow of the Goblin — a land PLAYED from the graveyard (Crucible of
/// Worlds) pings each opponent; one played from hand does not.
#[test]
fn shadow_of_the_goblin_pings_for_a_land_played_from_elsewhere() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::shadow_of_the_goblin());
    ready(&mut g, 0, catalog::crucible_of_worlds());
    let from_hand = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(from_hand)).expect("play from hand");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [20, 20]);
    g.players[0].lands_played_this_turn = 0;
    let from_yard = g.add_card_to_graveyard(0, catalog::mountain());
    g.perform_action(GameAction::PlayLandFromGraveyard(from_yard)).expect("play from graveyard");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
}
