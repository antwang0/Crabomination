//! Commander: the Reign of Dragons precon (FDC, Lathliss,
//! `decks::cmdr_lathliss`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn no_blocks_finish(g: &mut GameState, defender: usize) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Breath Weapon spares Dragons.
#[test]
fn breath_weapon_spares_dragons() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::goldlust_triad());
    let bw = g.add_card_to_hand(0, catalog::breath_weapon());
    cast_at(&mut g, bw, &[]).expect("cast");
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(dragon).is_some());
}

/// Goddric celebrating is a 4/4 flying Dragon. Regression: two creature
/// *spells* resolving is a celebration (CR 608.3 — a resolving permanent
/// spell enters), not only tokens.
#[test]
fn goddric_celebrates_into_a_dragon() {
    let mut g = main_phase(2);
    let gd = g.add_card_to_battlefield(0, catalog::goddric_cloaked_reveler());
    assert_eq!(pt(&g, gd), (3, 3));
    for _ in 0..2 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_at(&mut g, b, &[]).expect("cast");
    }
    assert_eq!(pt(&g, gd), (4, 4));
    assert!(has(&g, gd, &Keyword::Flying));
    // "He loses all other creature types" (CR 613.1d, layer 4).
    let cp = g.computed_permanent(gd).unwrap();
    assert_eq!(cp.subtypes().creature_types, vec![crabomination::card::CreatureType::Dragon]);
}

/// Goldlust Triad makes a Treasure off combat damage.
#[test]
fn goldlust_triad_mints_treasure() {
    let mut g = main_phase(2);
    let gt = g.add_card_to_battlefield(0, catalog::goldlust_triad());
    attack_with(&mut g, &[gt], 1);
    no_blocks_finish(&mut g, 1);
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
}

/// Hit the Mother Lode: discover 10, then tapped Treasures for the shortfall.
#[test]
fn hit_the_mother_lode_pays_out_the_difference() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let h = g.add_card_to_hand(0, catalog::hit_the_mother_lode());
    cast_at(&mut g, h, &[]).expect("cast");
    let t = named(&g, 0, "Treasure");
    assert_eq!(t.len(), 8, "10 - the Bear's 2");
    assert!(g.battlefield_find(t[0]).unwrap().tapped);
}

/// Leyline Tyrant keeps red mana through step changes.
#[test]
fn leyline_tyrant_keeps_red_mana() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::leyline_tyrant());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add(Color::Green, 3);
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
}

/// CR 500.4 — two Leyline Tyrants keep the red mana once, not once each: the
/// kept amount was re-added per keeper and doubled every step until the
/// pool's counter overflowed (six-seat pod, decks 97-102, seed 34097).
#[test]
fn two_leyline_tyrants_keep_red_mana_once() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::leyline_tyrant());
    g.add_card_to_battlefield(0, catalog::leyline_tyrant());
    g.players[0].mana_pool.add(Color::Red, 3);
    for _ in 0..3 {
        g.empty_mana_pools();
    }
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

/// Pack tactics: Minion of the Mighty drops a Dragon in attacking.
#[test]
fn minion_of_the_mighty_brings_a_dragon() {
    let mut g = main_phase(2);
    let minion = g.add_card_to_battlefield(0, catalog::minion_of_the_mighty());
    let big = g.add_card_to_battlefield(0, catalog::goldlust_triad());
    let big2 = g.add_card_to_battlefield(0, catalog::leyline_tyrant());
    let d = g.add_card_to_hand(0, catalog::parapet_thrasher());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![d])]));
    attack_with(&mut g, &[minion, big, big2], 1);
    assert!(g.battlefield_find(d).is_some_and(|c| c.tapped));
    assert!(g.attacking.iter().any(|a| a.attacker == d));
}

/// Nogi discounts Dragons and becomes one with three out.
#[test]
fn nogi_joins_the_dragons() {
    let mut g = main_phase(2);
    let n = g.add_card_to_battlefield(0, catalog::nogi_draco_zealot());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::goldlust_triad());
    }
    attack_with(&mut g, &[n], 1);
    assert_eq!(pt(&g, n), (5, 5));
    assert!(has(&g, n, &Keyword::Flying));
}

/// Orb of Dragonkind finds a Dragon in the top seven.
#[test]
fn orb_of_dragonkind_digs_for_a_dragon() {
    let mut g = main_phase(2);
    let d = g.add_card_to_library(0, catalog::goldlust_triad());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let orb = g.add_card_to_battlefield(0, catalog::orb_of_dragonkind());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![d])]));
    act(&mut g, GameAction::ActivateAbility { card_id: orb, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("sacrifice");
    assert!(g.players[0].hand.iter().any(|c| c.id == d));
}

/// Parapet Thrasher: a Dragon connecting picks an unchosen mode.
#[test]
fn parapet_thrasher_punishes_on_connect() {
    let mut g = main_phase(3);
    let pt_id = g.add_card_to_battlefield(0, catalog::parapet_thrasher());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let l2 = g.players[2].life;
    attack_with(&mut g, &[pt_id], 1);
    no_blocks_finish(&mut g, 1);
    let destroyed = g.battlefield_find(ring).is_none();
    let burned = g.players[2].life == l2 - 4;
    let impulsed = g.exile.iter().any(|c| c.owner == 0);
    assert!(destroyed || burned || impulsed, "one mode resolved");
}

/// Shivan Devastator enters with X counters, flying and hasty.
#[test]
fn shivan_devastator_scales_with_x() {
    let mut g = main_phase(2);
    let sd = g.add_card_to_hand(0, catalog::shivan_devastator());
    cast_x(&mut g, sd, &[], Some(5)).expect("cast");
    assert_eq!(pt(&g, sd), (5, 5));
    assert!(has(&g, sd, &Keyword::Flying) && has(&g, sd, &Keyword::Haste));
    assert_eq!(g.battlefield_find(sd).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// The Elder Dragon War's chapter I burns every creature and each opponent.
#[test]
fn the_elder_dragon_war_opens_with_fire() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let w = g.add_card_to_hand(0, catalog::the_elder_dragon_war());
    cast_at(&mut g, w, &[]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, life - 2);
}

/// Thundermane Dragon casts a big creature off the top of the library.
#[test]
fn thundermane_dragon_casts_from_the_top() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::thundermane_dragon());
    let top = g.add_card_to_library(0, catalog::goldlust_triad());
    act(&mut g, GameAction::CastSpell { card_id: top, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast from the top");
    assert!(g.battlefield_find(top).is_some());
}

/// Carnelian Orb's red mana hastes the creature it pays for.
#[test]
fn carnelian_orb_hastes_its_dragon() {
    let mut g = main_phase(2);
    let orb = g.add_card_to_battlefield(0, catalog::carnelian_orb_of_dragonkind());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: orb, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap for R");
    g.players[0].mana_pool.add_colorless(3);
    let d = g.add_card_to_hand(0, catalog::leyline_tyrant());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(has(&g, d, &Keyword::Haste));
}
