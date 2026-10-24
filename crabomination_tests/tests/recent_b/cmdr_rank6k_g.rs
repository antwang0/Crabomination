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

/// Permission Denied counters seat 1's Shock and locks every opponent out of
/// noncreature spells this turn, not only the Shock's caster (ruling); its
/// caster is unaffected.
#[test]
fn permission_denied_counters_and_locks_each_opponent() {
    let mut g = pod(3);
    let shock = g.add_card_to_hand(1, catalog::shock());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: shock, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Shock");
    let pd = g.add_card_to_hand(0, catalog::permission_denied());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: pd, target: Some(Target::Permanent(shock)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Permission Denied");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Shock countered");
    assert!(g.players[1].cant_cast_noncreature_this_turn && g.players[2].cant_cast_noncreature_this_turn);
    assert!(!g.players[0].cant_cast_noncreature_this_turn);
}

/// Helm of the Ghastlord on a blue-black creature: +2/+2, and damage to an
/// opponent both draws and makes that player discard (rulings: both bonuses).
#[test]
fn helm_of_the_ghastlord_reads_the_host_colours() {
    let mut g = pod(2);
    let host = ready(&mut g, 0, catalog::grizzly_bears());
    let helm = g.add_card_to_hand(0, catalog::helm_of_the_ghastlord());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.perform_action(GameAction::CastSpell {
        card_id: helm, target: Some(Target::Permanent(host)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Helm");
    drain_stack(&mut g);
    let p = |g: &GameState| g.computed_permanent(host).map(|c| (c.power, c.toughness)).unwrap();
    assert_eq!(p(&g), (2, 2), "green host: no bonus");
    let ub = ready(&mut g, 0, catalog::dimir_guildmage());
    g.battlefield_find_mut(helm).unwrap().attached_to = Some(ub);
    let base = catalog::dimir_guildmage();
    assert_eq!(
        g.computed_permanent(ub).map(|c| (c.power, c.toughness)).unwrap(),
        (base.power + 2, base.toughness + 2)
    );
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let hand0 = g.players[0].hand.len();
    attack(&mut g, 0, ub, 1).expect("attack");
    for _ in 0..40 {
        if g.step == TurnStep::PostCombatMain {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "blue: draw");
    assert!(g.players[1].hand.is_empty(), "black: discard");
}
