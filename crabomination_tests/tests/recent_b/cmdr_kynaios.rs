//! Commander: the Stalwart Unity precon (C16, Kynaios and Tiro, `decks::cmdr_kynaios`).

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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn declare(g: &mut GameState, attacker: CardId, defender: usize) -> Result<(), String> {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(kind)).unwrap_or(0)
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

/// Untap everything, draw; an opponent's blocker this turn draws another.
#[test]
fn benefactors_draught_draws_per_opposing_blocker() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::benefactors_draught());
    cast(&mut g, d, &[]);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), 1);
    attack_with(&mut g, &[bear], 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "the block drew a card");
}

/// Any creature hitting one of your opponents lets its controller draw.
#[test]
fn edric_lets_the_attacker_draw() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::edric_spymaster_of_trest());
    g.add_card_to_library(1, catalog::plains());
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &[bear], 2);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].hand.len(), 1, "seat 1 hit Edric's opponent and drew");
}

#[test]
fn entrapment_maneuver_turns_an_attacker_into_soldiers() {
    let mut g = main_phase(2);
    g.active_player_idx = 1;
    let force = g.add_card_to_battlefield(1, catalog::celestial_force());
    attack_with(&mut g, &[force], 0);
    let e = g.add_card_to_hand(0, catalog::entrapment_maneuver());
    cast(&mut g, e, &[Target::Player(1)]);
    assert!(g.battlefield_find(force).is_none());
    assert_eq!(count_named(&g, 0, "Soldier"), 7);
}

#[test]
fn evolutionary_escalation_grows_one_of_each_side() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::evolutionary_escalation());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Draw);
    assert_eq!(counters(&g, mine, CounterType::PlusOnePlusOne), 3);
    assert_eq!(counters(&g, theirs, CounterType::PlusOnePlusOne), 3);
}

/// A bribed creature's controller draws, and the creature can't attack.
#[test]
fn gwafa_hazid_bribes_a_creature_out_of_combat() {
    let mut g = main_phase(2);
    let gwafa = g.add_card_to_battlefield(0, catalog::gwafa_hazid_profiteer());
    g.clear_sickness(gwafa);
    g.add_card_to_library(1, catalog::plains());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, gwafa, 0, Some(Target::Permanent(bear))).expect("bribe");
    assert_eq!(counters(&g, bear, CounterType::Bribery), 1);
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.permanent_has_keyword(bear, &Keyword::CantAttack));
    assert!(g.permanent_has_keyword(bear, &Keyword::CantBlock));
}

#[test]
fn homeward_path_returns_stolen_creatures() {
    let mut g = main_phase(2);
    let path = g.add_card_to_battlefield(0, catalog::homeward_path());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().controller = 1;
    activate(&mut g, path, 1, None).expect("tap");
    assert_eq!(g.computed_permanent(mine).unwrap().controller, 0);
}

#[test]
fn hoofprints_of_the_stag_counts_draws_into_an_elemental() {
    let mut g = main_phase(2);
    let h = g.add_card_to_battlefield(0, catalog::hoofprints_of_the_stag());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let d = g.add_card_to_hand(0, catalog::harmonize());
    g.add_card_to_library(0, catalog::plains());
    cast(&mut g, d, &[]);
    let dr = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, dr, &[]);
    assert!(counters(&g, h, CounterType::Hoofprint) >= 4);
    activate(&mut g, h, 0, None).expect("make the Elemental");
    assert_eq!(count_named(&g, 0, "Elemental"), 1);
}

#[test]
fn humble_defector_draws_two_then_defects() {
    let mut g = main_phase(2);
    let d = g.add_card_to_battlefield(0, catalog::humble_defector());
    g.clear_sickness(d);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::plains());
    }
    activate(&mut g, d, 0, None).expect("activate");
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.computed_permanent(d).unwrap().controller, 1);
}

#[test]
fn keening_stone_mills_a_graveyards_worth() {
    let mut g = main_phase(2);
    let k = g.add_card_to_battlefield(0, catalog::keening_stone());
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::plains());
    }
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::plains());
    }
    activate(&mut g, k, 0, Some(Target::Player(1))).expect("mill");
    assert_eq!(g.players[1].graveyard.len(), 6);
}

/// An opponent's second spell in a turn draws Kraum's controller a card.
#[test]
fn kraum_draws_on_an_opponents_second_spell() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::kraum_ludevics_opus());
    g.add_card_to_library(0, catalog::plains());
    g.active_player_idx = 1;
    for _ in 0..2 {
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("bear");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), 1);
}

/// End step: you draw and may drop a land; an opponent with no land draws.
#[test]
fn kynaios_and_tiro_share_lands_and_cards() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::kynaios_and_tiro_of_meletis());
    for seat in 0..3 {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
    let land = g.add_card_to_hand(1, catalog::forest());
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "you draw");
    assert!(g.battlefield_find(land).is_some(), "seat 1 put its land down");
    assert_eq!(g.players[1].hand.len(), 0, "and so didn't draw");
    assert_eq!(g.players[2].hand.len(), 1, "seat 2 had no land and drew");
}

/// Each end step, if a player other than Ludevic's controller lost life, the
/// active player may draw.
#[test]
fn ludevic_offers_a_draw_after_an_opponent_bleeds() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::ludevic_necro_alchemist());
    g.add_card_to_library(0, catalog::plains());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 508.1a — a player who takes the counters can't attack you.
#[test]
fn orzhov_advokist_buys_peace_with_counters() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::orzhov_advokist());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false), DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Draw);
    assert_eq!(counters(&g, theirs, CounterType::PlusOnePlusOne), 2);
    assert!(g.permanent_has_keyword(theirs, &Keyword::CantAttackPlayer(0)));
}

#[test]
fn prismatic_geoscope_taps_for_domain() {
    let mut g = main_phase(2);
    let geo = g.add_card_to_battlefield(0, catalog::prismatic_geoscope());
    for land in [catalog::plains(), catalog::island(), catalog::forest()] {
        g.add_card_to_battlefield(0, land);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: geo, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// CR 615 — all damage to you is prevented this turn, and the Squire grows
/// by what was prevented.
#[test]
fn selfless_squire_soaks_up_prevented_damage() {
    let mut g = main_phase(2);
    let s = g.add_card_to_hand(0, catalog::selfless_squire());
    cast(&mut g, s, &[]);
    let life = g.players[0].life;
    g.active_player_idx = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life);
    assert_eq!(counters(&g, s, CounterType::PlusOnePlusOne), 3);
}

#[test]
fn sidar_kondo_shields_small_attackers_from_ground_blockers() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::sidar_kondo_of_jamuraa());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::celestial_force());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    attack_with(&mut g, &[bear], 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).is_err(), "no flying or reach");
    g.perform_action(GameAction::DeclareBlockers(vec![(flier, bear)])).expect("a flier may block");
}
