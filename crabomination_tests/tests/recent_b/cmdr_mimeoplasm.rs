//! Commander: the Devour for Power precon (CMD, The Mimeoplasm,
//! `decks::cmdr_mimeoplasm`) and the primitives it needed.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// One unblocked swing of `a` (controlled by the active seat) at `victim`.
fn connect(g: &mut GameState, a: CardId, victim: usize) {
    g.clear_sickness(a);
    g.battlefield_find_mut(a).unwrap().tapped = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(victim) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = victim;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 707.2 / 614.12 — The Mimeoplasm enters as a copy of one exiled creature
/// card with +1/+1 counters equal to the other's power, and the copied card's
/// own "when this enters" triggers (2011-09-22 ruling).
#[test]
fn the_mimeoplasm_enters_as_a_copy_with_the_other_cards_power() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let drifter = g.add_card_to_graveyard(1, catalog::mulldrifter());
    let elves = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    let m = g.add_card_to_hand(0, catalog::the_mimeoplasm());
    cast(&mut g, 0, m, None).expect("cast");
    let perm = g.battlefield_find(m).expect("survives as the copy");
    assert_eq!(perm.definition.name, "Mulldrifter");
    assert_eq!(plus_ones(&g, m), 1, "Llanowar Elves' power");
    assert!(g.exile.iter().any(|c| c.id == drifter) && g.exile.iter().any(|c| c.id == elves));
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "the copy's ETB drew two");
}

/// "You can't choose to exile just one creature card" — with one in the
/// graveyards nothing is exiled and the 0/0 dies.
#[test]
fn the_mimeoplasm_needs_two_creature_cards() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::the_mimeoplasm());
    cast(&mut g, 0, m, None).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(m).is_none(), "a 0/0");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "not exiled");
}

/// Damia draws up to seven at upkeep.
#[test]
fn damia_refills_to_seven_at_upkeep() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::damia_sage_of_stone());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7);
}

/// Skullbriar's counters stay with it into the graveyard and the command zone
/// and back out; a hand or a library still strips them (CR 122.2's exception).
#[test]
fn skullbriar_keeps_its_counters_off_the_battlefield() {
    let mut g = pod(2);
    let s = g.seat_commanders(0, vec![catalog::skullbriar_the_walking_grave()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == s).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    connect(&mut g, s, 1);
    connect(&mut g, s, 1);
    assert_eq!(plus_ones(&g, s), 2, "two connections");
    g.step = TurnStep::PostCombatMain;
    let murder = g.add_card_to_hand(1, catalog::murder());
    g.active_player_idx = 1;
    cast(&mut g, 1, murder, Some(Target::Permanent(s))).expect("murder");
    g.check_state_based_actions();
    let in_command = g.players[0].command.iter().find(|c| c.id == s).expect("returned to the command zone");
    assert_eq!(in_command.counter_count(CounterType::PlusOnePlusOne), 2, "kept through graveyard and command zone");
    g.active_player_idx = 0;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: s,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("recast from the command zone");
    drain_stack(&mut g);
    assert_eq!(plus_ones(&g, s), 2, "enters with the counters it kept");
    let _ = g.remove_from_battlefield_to_hand(s);
    let in_hand = g.players[0].hand.iter().find(|c| c.id == s).expect("bounced");
    assert_eq!(in_hand.counter_count(CounterType::PlusOnePlusOne), 0, "a hand strips them");
}

/// CR 802 / 506.2 — Riddlekeeper mills only the controller of a creature that
/// attacks Riddlekeeper's controller, not one attacking another opponent.
#[test]
fn riddlekeeper_mills_whoever_attacks_you() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::riddlekeeper());
    for seat in 0..3 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    connect(&mut g, bear, 2);
    assert_eq!(g.players[1].graveyard.len(), 0, "attacked someone else");
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    connect(&mut g, bear2, 0);
    assert_eq!(g.players[1].graveyard.len(), 2, "attacked Riddlekeeper's controller");
}

/// Vorosh pays {2}{G} on connecting for six +1/+1 counters.
#[test]
fn vorosh_grows_six_on_connecting() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let v = g.add_card_to_battlefield(0, catalog::vorosh_the_hunter());
    flood(&mut g, 0);
    connect(&mut g, v, 1);
    assert_eq!(plus_ones(&g, v), 6);
}

/// Desecrator Hag returns the greatest-power creature card from your graveyard.
#[test]
fn desecrator_hag_returns_the_biggest() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let hag = g.add_card_to_hand(0, catalog::desecrator_hag());
    cast(&mut g, 0, hag, None).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == angel));
}

/// Triskelavus: remove a counter for a flier that can ping.
#[test]
fn triskelavus_trades_counters_for_pinging_fliers() {
    let mut g = pod(2);
    let t = g.add_card_to_hand(0, catalog::triskelavus());
    cast(&mut g, 0, t, None).expect("cast");
    assert_eq!(plus_ones(&g, t), 3);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: t,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(plus_ones(&g, t), 2);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Triskelavite").map(|c| c.id).expect("token");
    g.perform_action(GameAction::ActivateAbility {
        card_id: token,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice to ping");
    let life = g.players[1].life;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.battlefield_find(token).is_none());
}
