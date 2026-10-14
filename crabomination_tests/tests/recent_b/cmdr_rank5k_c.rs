//! Commander: three-card gaps of EDHREC ranks 4501-5500 (`decks::cmdr_rank5k_c`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Prophesied End: a creature that wasn't attacking replaces itself for its
/// controller; an attacker doesn't.
#[test]
fn prophesied_end_draws_only_for_a_non_attacker() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::island());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prophesied_end());
    cast(&mut g, id, Some(Target::Permanent(bear)), None);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Invoke Despair: a missing permanent type costs the opponent 2 life and draws you a card.
#[test]
fn invoke_despair_punishes_each_missing_type() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::invoke_despair());
    cast(&mut g, id, Some(Target::Player(1)), None);
    assert!(g.battlefield_find(bear).is_none(), "the creature is sacrificed");
    assert_eq!(g.players[1].life, life - 4, "no enchantment, no planeswalker");
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Hellcat comes back with a counter, no abilities but haste.
#[test]
fn hellcat_returns_stripped_with_haste() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hellcat_undying_vigilante());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    cast(&mut g, wrath, None, None);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Hellcat, Undying Vigilante").map(|c| c.id);
    let back = back.expect("returned");
    assert_eq!(g.battlefield_find(back).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(keywords(&g, back).contains(&Keyword::Haste));
    // The second death stays dead: the return trigger was lost with her abilities.
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    cast(&mut g, wrath, None, None);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Hellcat, Undying Vigilante"));
}

/// Bonds of Faith: a Human grows, anything else is shut down.
#[test]
fn bonds_of_faith_splits_on_human() {
    let mut g = pod(2);
    let human = ready(&mut g, 0, catalog::huang_zhong_shu_general());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::bonds_of_faith());
    cast(&mut g, a, Some(Target::Permanent(human)), None);
    let b = g.add_card_to_hand(0, catalog::bonds_of_faith());
    cast(&mut g, b, Some(Target::Permanent(bear)), None);
    assert_eq!(pt(&g, human), (4, 5));
    assert!(keywords(&g, bear).contains(&Keyword::CantAttack));
    assert!(!keywords(&g, human).contains(&Keyword::CantAttack));
}

/// Go-Shintai of Hidden Cruelty: paying {1} at end step kills a creature with
/// toughness up to the Shrine count, and not one above it.
#[test]
fn go_shintai_of_hidden_cruelty_kills_up_to_shrine_toughness() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::go_shintai_of_hidden_cruelty());
    let small = ready(&mut g, 1, catalog::llanowar_elves());
    let big = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g);
    g.step = TurnStep::PostCombatMain;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "toughness 1 <= one Shrine");
    assert!(g.battlefield_find(big).is_some(), "toughness 2 > one Shrine");
}

/// Iron Man, Futurist Paragon turns a noncreature artifact into a 5/5 flier.
#[test]
fn iron_man_futurist_paragon_animates_an_artifact() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::iron_man_futurist_paragon());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    g.step = TurnStep::PreCombatMain;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let c = g.computed_permanent(ring).unwrap();
    assert!(c.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(c.card_types().contains(&crabomination::card::CardType::Artifact));
    assert_eq!((c.power, c.toughness), (5, 5));
    assert!(keywords(&g, ring).contains(&Keyword::Flying));
}

/// Doctor Doom is indestructible thanks to his own Doombots.
#[test]
fn doctor_doom_brings_doombots_and_indestructibility() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::doctor_doom());
    cast(&mut g, id, None, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Doombot").count(), 2);
    assert!(keywords(&g, id).contains(&Keyword::Indestructible));
}
