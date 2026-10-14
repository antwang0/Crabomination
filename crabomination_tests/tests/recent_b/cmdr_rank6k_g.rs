//! Commander: one-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_g`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_mode(g: &mut GameState, id: CardId, mode: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 5);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: Some(mode),
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn attack(g: &mut GameState, seat: usize, attacker: CardId, at: usize) -> Result<(), GameError> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(at) }]))
        .map(|_| ())
}

/// CR 614.1a / 508.1 — Island Sanctuary: the draw-step draw is skipped (the
/// auto policy takes it facing three ground power), and until seat 0's next
/// turn a ground creature can't attack seat 0 while a flier still can.
#[test]
fn island_sanctuary_trades_the_draw_for_a_ground_shield() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::island_sanctuary());
    let ogre = ready(&mut g, 1, catalog::hill_giant());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.add_card_to_library(0, catalog::plains());
    g.turn_number = 3;
    g.step = TurnStep::Upkeep;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.step, TurnStep::Draw);
    assert!(g.players[0].hand.is_empty(), "the draw was skipped");
    assert_eq!(g.players[0].library.len(), 1);
    assert!(attack(&mut g.clone(), 1, ogre, 0).is_err(), "Hill Giant can't attack seat 0");
    assert!(attack(&mut g, 1, angel, 0).is_ok(), "Serra Angel can");
}

/// CR 509.1c — You've Been Caught Stealing's first mode makes each creature
/// block this turn if able; the second makes a Treasure per opponent dealt
/// damage this turn.
#[test]
fn youve_been_caught_stealing_modes() {
    let mut g = pod(4);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::youve_been_caught_stealing());
    cast_mode(&mut g, s, 0);
    let cp = g.computed_permanent(bears).unwrap();
    assert!(cp.keywords().contains(&Keyword::MustBlock));

    let mut g = pod(4);
    g.players[1].was_dealt_damage_this_turn = true;
    g.players[3].was_dealt_damage_this_turn = true;
    let s = g.add_card_to_hand(0, catalog::youve_been_caught_stealing());
    cast_mode(&mut g, s, 1);
    let treasures = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 5);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// CR 122.1 / 610.3 — Livio exiles another creature with an aegis counter
/// (its controller agreeing), then brings every aegis card back under its
/// owner's control.
#[test]
fn livio_exiles_with_aegis_and_returns() {
    let mut g = pod(2);
    let livio = ready(&mut g, 0, catalog::livio_oathsworn_sentinel());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    activate(&mut g, livio, 0, Some(Target::Permanent(bears)));
    let exiled = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").expect("exiled");
    assert_eq!(exiled.counter_count(crabomination::card::CounterType::Aegis), 1);
    activate(&mut g, livio, 1, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0));
    assert!(g.exile.is_empty());
}
