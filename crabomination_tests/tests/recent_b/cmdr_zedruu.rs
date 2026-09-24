//! Commander: the Political Puppets precon (CMD, Zedruu, `decks::cmdr_zedruu`).

use crabomination::card::{CardId, CounterType};
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], drain: bool) {
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
    if drain {
        drain_stack(g);
    }
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets, true);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn step_into(g: &mut GameState, active: usize, from: TurnStep, to: TurnStep) {
    g.active_player_idx = active;
    g.step = from;
    g.priority.player_with_priority = active;
    while g.step != to {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 702.24 — Jötun Grunt's cumulative upkeep bottoms two cards of a single
/// graveyard per age counter (an opponent's first); with too few, it's
/// sacrificed.
#[test]
fn jotun_grunt_eats_graveyards_or_dies() {
    let mut g = pod(2);
    let grunt = g.add_card_to_battlefield(0, catalog::jotun_grunt());
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert!(g.battlefield_find(grunt).is_some());
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.players[1].library.len(), 2);
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert!(g.battlefield_find(grunt).is_none(), "two age counters want four cards");
}

/// Martyr's Bond: your creature dies → each opponent sacrifices a creature;
/// the Bond itself going → each opponent sacrifices an enchantment.
#[test]
fn martyrs_bond_makes_the_table_match_your_losses() {
    let mut g = pod(3);
    let bond = g.add_card_to_battlefield(0, catalog::martyrs_bond());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for s in [1, 2] {
        g.add_card_to_battlefield(s, catalog::grizzly_bears());
        g.add_card_to_battlefield(s, catalog::sol_ring());
        g.add_card_to_battlefield(s, catalog::crescendo_of_war());
    }
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(mine)]);
    for s in [1, 2] {
        assert!(!g.battlefield.iter().any(|c| c.controller == s && c.definition.is_creature()), "seat {s}");
        assert!(g.battlefield.iter().any(|c| c.controller == s && c.definition.name == "Sol Ring"));
    }
    let ench = g.add_card_to_hand(0, catalog::disenchant());
    cast(&mut g, ench, &[Target::Permanent(bond)]);
    for s in [1, 2] {
        assert!(!g.battlefield.iter().any(|c| c.controller == s && c.definition.name == "Crescendo of War"));
    }
}

/// Spell Crumple: the countered spell and Spell Crumple both go to the
/// bottom of their owners' libraries.
#[test]
fn spell_crumple_bottoms_both() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_by(&mut g, 1, bolt, &[Target::Player(0)], false);
    let crumple = g.add_card_to_hand(0, catalog::spell_crumple());
    cast_by(&mut g, 0, crumple, &[Target::Permanent(bolt)], true);
    assert_eq!(g.players[0].life, g.players[0].starting_life);
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bolt));
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(crumple));
}

/// CR 701.30 — Pollen Lullaby's clash, won, keeps *the clashed opponent's*
/// creatures tapped through their next untap.
#[test]
fn pollen_lullaby_locks_the_clashed_opponent() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::craw_wurm());
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let pl = g.add_card_to_hand(0, catalog::pollen_lullaby());
    cast(&mut g, pl, &[]);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Whirlpool Whelm: a won clash puts the creature on top of its owner's
/// library; a lost one bounces it to hand.
#[test]
fn whirlpool_whelm_tops_on_a_win() {
    for (mine, theirs, on_top) in [(catalog::craw_wurm(), catalog::island(), true), (catalog::island(), catalog::craw_wurm(), false)] {
        let mut g = pod(2);
        g.add_card_to_library(0, mine);
        g.add_card_to_library(1, theirs);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ww = g.add_card_to_hand(0, catalog::whirlpool_whelm());
        cast(&mut g, ww, &[Target::Permanent(bear)]);
        assert!(g.battlefield_find(bear).is_none());
        assert_eq!(g.players[1].library.first().map(|c| c.id) == Some(bear), on_top);
        assert_eq!(g.players[1].hand.iter().any(|c| c.id == bear), !on_top);
    }
}

/// Brion Stoutarm flings another creature for its power at a player.
#[test]
fn brion_stoutarm_flings() {
    let mut g = pod(2);
    let brion = g.add_card_to_battlefield(0, catalog::brion_stoutarm());
    g.clear_sickness(brion);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    activate(&mut g, brion, Some(Target::Player(1)), None);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[1].life, g.players[1].starting_life - 3);
}

/// Crescendo of War: a strife counter each upkeep; attackers get +1/+0 per
/// counter.
#[test]
fn crescendo_of_war_escalates() {
    let mut g = pod(2);
    let cw = g.add_card_to_battlefield(0, catalog::crescendo_of_war());
    for s in 0..2 {
        g.add_card_to_library(s, catalog::island());
    }
    step_into(&mut g, 1, TurnStep::Untap, TurnStep::Draw);
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert_eq!(g.battlefield_find(cw).unwrap().counter_count(CounterType::Strife), 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (2, 2));
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    assert_eq!(pt(&g, bear), (4, 2));
}

/// Nin: X damage to a creature, and its controller draws X.
#[test]
fn nin_pays_the_victim() {
    let mut g = pod(2);
    let nin = g.add_card_to_battlefield(0, catalog::nin_the_pain_artist());
    g.clear_sickness(nin);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    activate(&mut g, nin, Some(Target::Permanent(wurm)), Some(2));
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 2);
    assert_eq!(g.players[1].hand.len(), 2);
}

/// Prison Term hops to an opponent's creature as it enters.
#[test]
fn prison_term_jumps_to_the_newcomer() {
    let mut g = pod(2);
    let first = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pt_id = g.add_card_to_hand(0, catalog::prison_term());
    cast(&mut g, pt_id, &[Target::Permanent(first)]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let big = g.add_card_to_hand(1, catalog::craw_wurm());
    g.active_player_idx = 1;
    cast_by(&mut g, 1, big, &[], true);
    assert_eq!(g.battlefield_find(pt_id).unwrap().attached_to, Some(big));
}

/// CR 508.1d — Ruhan must attack the opponent it picked at random this
/// combat.
#[test]
fn ruhan_attacks_its_random_opponent() {
    let mut g = pod(3);
    let ruhan = g.add_card_to_battlefield(0, catalog::ruhan_of_the_fomori());
    g.clear_sickness(ruhan);
    step_into(&mut g, 0, TurnStep::PreCombatMain, TurnStep::BeginCombat);
    let pick = g.battlefield_find(ruhan).unwrap().chosen_player.expect("an opponent was picked");
    assert_ne!(pick, 0);
    let other = if pick == 1 { 2 } else { 1 };
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: ruhan, target: AttackTarget::Player(p) }]);
    assert!(g.perform_action(at(other)).is_err());
    g.perform_action(at(pick)).expect("the picked opponent");
}

/// Rapacious One: combat damage to a player makes that many Eldrazi Spawn.
#[test]
fn rapacious_one_spawns_per_damage() {
    let mut g = pod(2);
    let r1 = g.add_card_to_battlefield(0, catalog::rapacious_one());
    g.clear_sickness(r1);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: r1, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count(), 5);
}
