//! Commander: the Draconic Domination precon (C17, The Ur-Dragon,
//! `decks::cmdr_urdragon`).

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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// Seat `seat` (made active) declares `attacks`.
fn declare(g: &mut GameState, seat: usize, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.active_player_idx = seat;
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, d)| Attack { attacker, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}
fn bolt(g: &mut GameState, from: usize, target: CardId) {
    let b = g.add_card_to_hand(from, catalog::lightning_bolt());
    cast(g, from, b, Some(Target::Permanent(target))).expect("bolt");
}

/// Boneyard Scourge returns for {1}{B} when another Dragon of yours dies.
#[test]
fn boneyard_scourge_returns_when_a_dragon_dies() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bs = g.add_card_to_graveyard(0, catalog::boneyard_scourge());
    let sd = g.add_card_to_battlefield(0, catalog::spellbound_dragon());
    flood(&mut g, 0);
    g.battlefield_find_mut(sd).unwrap().damage = 3;
    bolt(&mut g, 1, sd);
    assert!(g.battlefield_find(bs).is_some());
}

/// Dromoka bolsters 2 per attacking Dragon; Kolaghan pumps the team per one.
#[test]
fn dromoka_and_kolaghan_trigger_per_attacking_dragon() {
    let mut g = pod(2);
    let dr = g.add_card_to_battlefield(0, catalog::dromoka_the_eternal());
    let ko = g.add_card_to_battlefield(0, catalog::kolaghan_the_storms_fury());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    declare(&mut g, 0, vec![(dr, 1), (ko, 1)]).expect("attack");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 4, "bolster 2 twice");
    assert_eq!(pt(&g, bear), (2 + 4 + 2, 2 + 4), "+1/+0 twice");
}

/// Fortunate Few: each player spares one nonland permanent they don't
/// control; the rest die.
#[test]
fn fortunate_few_spares_one_per_player() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let their_big = g.add_card_to_battlefield(1, catalog::serra_angel());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let ff = g.add_card_to_hand(0, catalog::fortunate_few());
    cast(&mut g, 0, ff, None).expect("cast");
    assert!(g.battlefield_find(land).is_some(), "lands survive");
    assert!(g.battlefield_find(mine).is_some(), "the opponent spared mine");
    assert!(g.battlefield_find(theirs).is_some(), "I spared their cheapest");
    assert!(g.battlefield_find(their_big).is_none());
}

/// O-Kagachi exiles only from a player who attacked you during their last turn.
#[test]
fn o_kagachi_punishes_last_turns_attacker() {
    let mut g = pod(3);
    let ok = g.add_card_to_battlefield(0, catalog::o_kagachi_vengeful_kami());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    declare(&mut g, 1, vec![(a, 0)]).expect("seat 1 attacks me");
    g.attacking.clear();
    let ring1 = g.add_card_to_battlefield(1, catalog::sol_ring());
    declare(&mut g, 0, vec![(ok, 1)]).expect("attack seat 1");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(ring1).is_none() || g.battlefield_find(a).is_none(), "a nonland permanent of theirs exiled");
    // Seat 2 never attacked: hitting them exiles nothing.
    let mut g = pod(3);
    let ok = g.add_card_to_battlefield(0, catalog::o_kagachi_vengeful_kami());
    let ring = g.add_card_to_battlefield(2, catalog::sol_ring());
    declare(&mut g, 0, vec![(ok, 2)]).expect("attack seat 2");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(ring).is_some());
}

/// Ojutai taps an opponent's permanent that skips its next untap.
#[test]
fn ojutai_locks_a_permanent() {
    let mut g = pod(2);
    let oj = g.add_card_to_battlefield(0, catalog::ojutai_soul_of_winter());
    let ring = g.add_card_to_battlefield(1, catalog::serra_angel());
    declare(&mut g, 0, vec![(oj, 1)]).expect("attack");
    let c = g.battlefield_find(ring).unwrap();
    assert!(c.tapped);
}

/// Palace Siege's Dragons mode drains each opponent for 2 at your upkeep.
#[test]
fn palace_siege_dragons_drains() {
    let mut g = pod(3);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    let ps = g.add_card_to_hand(0, catalog::palace_siege());
    cast(&mut g, 0, ps, None).expect("cast");
    let (l0, l1, l2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (l0 + 4, l1 - 2, l2 - 2));
}

/// Scalelord Reckoner answers an opponent targeting a Dragon of yours.
#[test]
fn scalelord_reckoner_retaliates() {
    let mut g = pod(2);
    let sr = g.add_card_to_battlefield(0, catalog::scalelord_reckoner());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    bolt(&mut g, 1, sr);
    assert!(g.battlefield_find(ring).is_none());
}

/// Scion of the Ur-Dragon becomes a copy of a Dragon it bins.
#[test]
fn scion_of_the_ur_dragon_becomes_a_binned_dragon() {
    let mut g = pod(2);
    let scion = g.add_card_to_battlefield(0, catalog::scion_of_the_ur_dragon());
    g.add_card_to_library(0, catalog::broodmate_dragon());
    activate(&mut g, 0, scion, 0, None, None).expect("activate");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Broodmate Dragon"));
    assert_eq!(g.battlefield_find(scion).unwrap().definition.name, "Broodmate Dragon");
}

/// Silumgar shrinks the defending player's creatures per attacking Dragon.
#[test]
fn silumgar_shrinks_the_defenders() {
    let mut g = pod(3);
    let si = g.add_card_to_battlefield(0, catalog::silumgar_the_drifting_death());
    let def = g.add_card_to_battlefield(1, catalog::hill_giant());
    let other = g.add_card_to_battlefield(2, catalog::hill_giant());
    declare(&mut g, 0, vec![(si, 1)]).expect("attack");
    assert_eq!(pt(&g, def), (2, 2));
    assert_eq!(pt(&g, other), (3, 3));
}

/// Spellbound Dragon pumps by the discarded card's mana value.
#[test]
fn spellbound_dragon_pumps_by_the_discard() {
    let mut g = pod(2);
    let sd = g.add_card_to_battlefield(0, catalog::spellbound_dragon());
    g.add_card_to_library(0, catalog::serra_angel());
    declare(&mut g, 0, vec![(sd, 1)]).expect("attack");
    assert_eq!(pt(&g, sd).0, 3 + 5);
}

/// Wasitora makes the player sacrifice, or mints a Cat Dragon if they can't.
#[test]
fn wasitora_eats_or_breeds() {
    let mut g = pod(2);
    let wa = g.add_card_to_battlefield(0, catalog::wasitora_nekoru_queen());
    declare(&mut g, 0, vec![(wa, 1)]).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, 0, "Cat Dragon").len(), 1, "no creature to sacrifice");
}

/// Crucible of the Spirit Dragon stores counters and pays them out.
#[test]
fn crucible_stores_dragon_mana() {
    let mut g = pod(2);
    let cr = g.add_card_to_battlefield(0, catalog::crucible_of_the_spirit_dragon());
    for _ in 0..2 {
        activate(&mut g, 0, cr, 1, None, None).expect("store");
        g.battlefield_find_mut(cr).unwrap().tapped = false;
    }
    assert_eq!(g.battlefield_find(cr).unwrap().counter_count(CounterType::Storage), 2);
}

/// Orator of Ojutai draws only with a Dragon around.
#[test]
fn orator_draws_with_a_dragon() {
    let mut g = pod(2);
    stock_libraries(&mut g, 3);
    let o = g.add_card_to_hand(0, catalog::orator_of_ojutai());
    let h = g.players[0].hand.len();
    cast(&mut g, 0, o, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), h - 1, "no Dragon: no draw");
    g.add_card_to_battlefield(0, catalog::broodmate_dragon());
    let o2 = g.add_card_to_hand(0, catalog::orator_of_ojutai());
    let h = g.players[0].hand.len();
    cast(&mut g, 0, o2, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), h, "drew one");
}
