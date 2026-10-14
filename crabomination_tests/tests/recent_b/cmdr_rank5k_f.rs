//! Commander: four-card gaps of EDHREC ranks 4501-5500, second batch (`decks::cmdr_rank5k_f`).

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
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x });
}

/// Lizard, Connors's Curse: the target is a vanilla green 4/4 Lizard for good.
#[test]
fn lizard_turns_a_creature_into_a_vanilla_lizard() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::lizard_connorss_curse());
    cast(&mut g, id, None, None);
    let c = g.computed_permanent(angel).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords().is_empty());
    assert!(c.subtypes().creature_types.contains(&crabomination::card::CreatureType::Lizard));
    assert_eq!(c.colors.to_vec(), vec![Color::Green]);
}

/// Study the Classics: one counter, doubled to two, and life for each.
#[test]
fn study_the_classics_doubles_and_gains() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::study_the_classics());
    cast(&mut g, id, Some(Target::Permanent(bear)), None);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.players[0].life, life + 2);
}

/// Swamp Mosquito poisons the defending player when unblocked.
#[test]
fn swamp_mosquito_poisons_when_unblocked() {
    let mut g = pod(3);
    let m = ready(&mut g, 0, catalog::swamp_mosquito());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: m,
        target: crabomination::game::types::AttackTarget::Player(2),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.players[2].poison_counters, 1);
    assert_eq!(g.players[1].poison_counters, 0);
}

/// Giant-Man adds {G} per creature you control with power 4 or more.
#[test]
fn giant_man_adds_green_per_big_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::giant_man_gargantuan_genius());
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::Draw;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}
