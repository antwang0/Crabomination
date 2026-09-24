//! Commander: the Draconic Destruction starter deck (SCD, Atarka,
//! `decks::cmdr_atarka`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets);
}

fn lost(g: &GameState, s: usize) -> i32 {
    g.players[s].starting_life - g.players[s].life
}

fn declare(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
}

/// Atarka: an attacking Dragon gains double strike; Crucible of Fire pumps
/// Dragons.
#[test]
fn atarka_doubles_attacking_dragons() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::atarka_world_render());
    g.add_card_to_battlefield(0, catalog::crucible_of_fire());
    let tb = g.add_card_to_battlefield(0, catalog::thunderbreak_regent());
    declare(&mut g, tb, 1);
    let cp = g.computed_permanent(tb).unwrap();
    assert!(cp.keywords().contains(&Keyword::DoubleStrike));
    assert_eq!((cp.power, cp.toughness), (7, 7));
}

/// Scourge of Valkas: each Dragon entering burns for your Dragon count.
#[test]
fn scourge_of_valkas_scales_with_dragons() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::thunderbreak_regent());
    let s = g.add_card_to_hand(0, catalog::scourge_of_valkas());
    cast(&mut g, s, &[]);
    assert_eq!(lost(&g, 1), 2, "two Dragons");
}

/// Thunderbreak Regent: the opponent who targets a Dragon takes 3.
#[test]
fn thunderbreak_regent_punishes_targeting() {
    let mut g = pod(2);
    let tb = g.add_card_to_battlefield(0, catalog::thunderbreak_regent());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_by(&mut g, 1, bolt, &[Target::Permanent(tb)]);
    assert_eq!(lost(&g, 1), 3);
}

/// Demanding Dragon: take 5 or sacrifice a creature.
#[test]
fn demanding_dragon_demands() {
    let mut g = pod(2);
    let d = g.add_card_to_hand(0, catalog::demanding_dragon());
    cast(&mut g, d, &[Target::Player(1)]);
    assert_eq!(lost(&g, 1), 5, "no creature to give");
}

/// Foe-Razer Regent fights on entry; the fighter gets two counters at the end
/// step.
#[test]
fn foe_razer_regent_fights_and_grows() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let f = g.add_card_to_hand(0, catalog::foe_razer_regent());
    cast(&mut g, f, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_none());
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(f).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Dragon's Hoard banks a gold counter per Dragon and cashes it for a card.
#[test]
fn dragons_hoard_banks_gold() {
    let mut g = pod(2);
    let hoard = g.add_card_to_battlefield(0, catalog::dragons_hoard());
    g.add_card_to_library(0, catalog::island());
    let d = g.add_card_to_hand(0, catalog::thunderbreak_regent());
    cast(&mut g, d, &[]);
    assert_eq!(g.battlefield_find(hoard).unwrap().counter_count(CounterType::Gold), 1);
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hoard,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Unleash Fury doubles power; Provoke the Trolls pumps the survivor.
#[test]
fn fury_and_trolls_pump() {
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let uf = g.add_card_to_hand(0, catalog::unleash_fury());
    cast(&mut g, uf, &[Target::Permanent(wurm)]);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 12);
    let pt = g.add_card_to_hand(0, catalog::provoke_the_trolls());
    cast(&mut g, pt, &[Target::Permanent(wurm)]);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 17);
}

/// Verix Bladewing kicked brings Karox.
#[test]
fn verix_kicked_brings_karox() {
    let mut g = pod(2);
    let v = g.add_card_to_hand(0, catalog::verix_bladewing());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked { card_id: v, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Karox Bladewing"));
}

/// Spit Flame returns from the graveyard when a Dragon enters, for {R}.
#[test]
fn spit_flame_returns_on_a_dragon() {
    let mut g = pod(2);
    let sf = g.add_card_to_graveyard(0, catalog::spit_flame());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let d = g.add_card_to_hand(0, catalog::thunderbreak_regent());
    cast(&mut g, d, &[]);
    assert!(g.players[0].hand.iter().any(|c| c.id == sf));
}

/// Demanding Dragon: a player with a creature to spare gives it up instead.
#[test]
fn demanding_dragon_takes_a_creature_instead() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let d = g.add_card_to_hand(0, catalog::demanding_dragon());
    cast(&mut g, d, &[Target::Player(1)]);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(lost(&g, 1), 0);
}
