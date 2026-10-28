//! Star Trek Commander: the Klingon Fury precon's new cards
//! (`decks::trek_klingon`) — battlefront, the per-opponent attack count.

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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
    for seat in 0..n {
        for _ in 0..20 {
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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn attack(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn pass_to(g: &mut GameState, step: TurnStep) {
    for _ in 0..200 {
        if g.step == step {
            return;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    panic!("never reached {step:?}");
}

fn tokens_named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.is_token && c.definition.name == name).map(|c| c.id).collect()
}

/// Battlefront (CR 508.1: the opponents you attacked): Kurn attacking one
/// opponent while a second creature attacks another draws two and loses two.
#[test]
fn kurn_draws_one_per_opponent_attacked() {
    let mut g = pod(4);
    let kurn = ready(&mut g, 0, catalog::kurn_son_of_mogh());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    attack(&mut g, vec![(kurn, 1), (bear, 3)]);
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[0].life, life - 2);
}

/// Rite of Ascension: one hasty Warrior per opponent attacked, each attacking
/// that opponent — not the opponent left alone.
#[test]
fn rite_of_ascension_sends_a_warrior_at_each_attacked_opponent() {
    let mut g = pod(4);
    g.add_card_to_battlefield(0, catalog::rite_of_ascension());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::hill_giant());
    attack(&mut g, vec![(a, 1), (b, 2)]);
    let warriors = tokens_named(&g, 0, "Klingon Warrior");
    assert_eq!(warriors.len(), 2);
    let mut defenders: Vec<usize> = g
        .attacking
        .iter()
        .filter(|atk| warriors.contains(&atk.attacker))
        .map(|atk| match atk.target {
            AttackTarget::Player(p) => p,
            _ => usize::MAX,
        })
        .collect();
    defenders.sort();
    assert_eq!(defenders, vec![1, 2]);
}

/// Koloth: at your end step every player gets two Tribbles that can't block;
/// every Tribble is goaded (CR 701.15) and has +1/+0.
#[test]
fn koloth_hands_out_goaded_tribbles() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::koloth_tribble_hunter());
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    for seat in 0..3 {
        let t = tokens_named(&g, seat, "Tribble");
        assert_eq!(t.len(), 2, "seat {seat}");
        let c = g.computed_permanent(t[0]).unwrap();
        assert_eq!((c.power, c.toughness), (2, 1));
        assert!(c.keywords().contains(&Keyword::CantBlock));
        assert!(c.subtypes().creature_types.contains(&CreatureType::Tribble));
        assert!(g.is_goaded(g.battlefield_find(t[0]).unwrap()));
    }
}

/// The Promise of Kahless returns every *other* Klingon and Warrior: the one
/// sacrificed stays in the graveyard.
#[test]
fn promise_of_kahless_returns_all_but_the_sacrifice() {
    let mut g = pod(2);
    let fodder = g.add_card_to_battlefield(0, catalog::gagh_slinger());
    let a = g.add_card_to_graveyard(0, catalog::kurn_son_of_mogh());
    let b = g.add_card_to_graveyard(0, catalog::chadich_investigator());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::the_promise_of_kahless());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "and not returned");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "a Bear is no Warrior");
}

/// Alexander parleys, then deals the greatest revealed mana value to an
/// opponent.
#[test]
fn alexander_burns_for_the_greatest_revealed_mana_value() {
    let mut g = pod(2);
    let alex = ready(&mut g, 0, catalog::alexander_ships_fool());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::shivan_dragon());
    g.add_card_to_library(1, catalog::island());
    let life = g.players[1].life;
    let hand = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: alex,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 6, "Shivan Dragon's six");
    assert_eq!(g.players[1].hand.len(), hand + 1, "each player draws");
}

/// Glory to the Empire! makes one Warrior per opponent and gives melee.
#[test]
fn glory_to_the_empire_scales_with_opponents() {
    let mut g = pod(4);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::glory_to_the_empire());
    flood(&mut g, 0);
    cast(&mut g, e, None);
    assert_eq!(tokens_named(&g, 0, "Klingon Warrior").len(), 3);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Melee));
}

/// Right of Vengeance: this turn, a creature that deals you combat damage is
/// destroyed and you get a hasty Warrior.
#[test]
fn right_of_vengeance_punishes_attackers_this_turn() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::right_of_vengeance());
    flood(&mut g, 0);
    cast(&mut g, spell, None);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(tokens_named(&g, 0, "Klingon Warrior").len(), 1);
}

/// Martok: battlefront caps his counters at the opponents attacked.
#[test]
fn martok_counters_are_capped_by_battlefront() {
    let mut g = pod(4);
    let martok = ready(&mut g, 0, catalog::martok_hero_of_the_empire());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, vec![(martok, 2)]);
    let total = [martok, bear]
        .iter()
        .map(|&id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .sum::<u32>();
    assert_eq!(total, 1, "one opponent attacked: one counter");
}
