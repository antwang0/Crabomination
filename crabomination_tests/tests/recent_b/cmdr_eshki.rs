//! Commander: the Temur Roar precon (TDC, Eshki, `decks::cmdr_eshki`).

use crabomination::card::{CardId, CreatureType, Keyword};
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).expect("on the battlefield").power
}

fn declare(g: &mut GameState, seat: usize, attacks: Vec<Attack>) -> Result<(), String> {
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(attacks)).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn at(attacker: CardId, p: usize) -> Attack {
    Attack { attacker, target: AttackTarget::Player(p) }
}

fn run_combat_out(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Eshki: a counter per creature spell; a 4-power spell draws, a 6-power one
/// also burns each opponent for Eshki's power.
#[test]
fn eshki_grows_draws_and_roars() {
    let mut g = main_phase(3);
    library(&mut g, 0, 3);
    let eshki = g.add_card_to_battlefield(0, catalog::eshki_temurs_roar());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("bear");
    assert_eq!((power(&g, eshki), g.players[0].hand.len()), (3, 0));
    let big = g.add_card_to_hand(0, catalog::hammerhead_tyrant());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    cast_at(&mut g, big, &[]).expect("a 6/6");
    assert_eq!(g.players[0].hand.len(), 1, "power 4+ draws");
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 4, l2 - 4), "Eshki is 4/4 when it roars");
}

/// Become the Avalanche: a card per big creature, then +X/+X for the hand.
#[test]
fn become_the_avalanche_draws_then_pumps_by_hand() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let a = g.add_card_to_battlefield(0, catalog::hammerhead_tyrant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::plains());
    let bta = g.add_card_to_hand(0, catalog::become_the_avalanche());
    cast_at(&mut g, bta, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2, "one kept + one drawn");
    assert_eq!((power(&g, a), power(&g, bear)), (8, 4));
}

/// Broodcaller Scourge: Dragons connecting for 5 let you drop a permanent of
/// mana value up to 5 from hand.
#[test]
fn broodcaller_scourge_cheats_by_the_damage() {
    let mut g = main_phase(2);
    let bs = g.add_card_to_battlefield(0, catalog::broodcaller_scourge());
    let big = g.add_card_to_hand(0, catalog::ureni_of_the_unwritten());
    let fits = g.add_card_to_hand(0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![fits])]));
    declare(&mut g, 0, vec![at(bs, 1)]).expect("attack");
    run_combat_out(&mut g);
    assert!(g.battlefield_find(fits).is_some());
    assert!(g.battlefield_find(big).is_none(), "mana value 7 is over 5");
}

/// Deceptive Frostkite copies your big creature and is a flying Dragon too.
#[test]
fn deceptive_frostkite_copies_a_big_creature() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(0, catalog::gadrak_the_crown_scourge());
    let df = g.add_card_to_hand(0, catalog::deceptive_frostkite());
    cast_at(&mut g, df, &[]).expect("cast");
    let cp = g.computed_permanent(df).unwrap();
    assert_eq!(g.battlefield_find(df).unwrap().definition.name, "Gadrak, the Crown-Scourge");
    assert!(cp.keywords().contains(&Keyword::Flying));
}

/// Draconic Lore costs {2} less with a Dragon out.
#[test]
fn draconic_lore_is_cheaper_with_a_dragon() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::gadrak_the_crown_scourge());
    let dl = g.add_card_to_hand(0, catalog::draconic_lore());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: dl, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("four mana is enough");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3);
}

/// Gadrak attacks only with four artifacts; your end step makes a Treasure
/// per nontoken creature that died.
#[test]
fn gadrak_needs_artifacts_and_mints_treasure() {
    let mut g = main_phase(2);
    let gd = g.add_card_to_battlefield(0, catalog::gadrak_the_crown_scourge());
    assert!(declare(&mut g.clone(), 0, vec![at(gd, 1)]).is_err(), "no artifacts");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, &[Target::Permanent(bear)]).expect("bolt");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Treasure"), 1);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::sol_ring());
    }
    declare(&mut g, 0, vec![at(gd, 1)]).expect("four artifacts");
}

/// Hammerhead Tyrant bounces an opponent's permanent up to the spell's mana
/// value. Regression: a battlefield cast trigger picked its target before the
/// cast spell's mana value was in scope, so a filter relative to it (this,
/// Skyfire Kirin) never found a target.
#[test]
fn hammerhead_tyrant_bounces_by_mana_value() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::hammerhead_tyrant());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(&mut g, shock, &[Target::Player(1)]).expect("a one-drop");
    assert!(g.battlefield_find(small).is_some() && g.battlefield_find(big).is_some(), "nothing at mana value 1");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("a two-drop");
    assert!(g.players[1].hand.iter().any(|c| c.id == small), "the Bears, mana value 2");
    assert!(g.battlefield_find(big).is_some(), "mana value 4 is over 2");
}

/// Opportunistic Dragon holds an opponent's artifact, stripped, while it stays.
#[test]
fn opportunistic_dragon_takes_an_artifact_while_it_stays() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let od = g.add_card_to_hand(0, catalog::opportunistic_dragon());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ring))]));
    cast_at(&mut g, od, &[]).expect("cast");
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, &[Target::Permanent(od)]).expect("bolt the Dragon");
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 1, "back when it leaves");
}

/// Temple of the Dragon Queen: untapped with a Dragon to reveal or control;
/// its mana is the chosen color.
#[test]
fn temple_of_the_dragon_queen_wants_a_dragon() {
    let mut g = main_phase(2);
    let t = g.add_card_to_hand(0, catalog::temple_of_the_dragon_queen());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(t)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(t).unwrap().tapped, "no Dragon");
    let mut g = main_phase(2);
    g.add_card_to_hand(0, catalog::gadrak_the_crown_scourge());
    let t = g.add_card_to_hand(0, catalog::temple_of_the_dragon_queen());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(t)).expect("play");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(t).unwrap().tapped, "a Dragon in hand");
}

/// Territorial Hellkite: each of your combats it must attack a random
/// opponent it didn't attack last combat; with none left, it taps instead.
#[test]
fn territorial_hellkite_rotates_its_victims() {
    let mut g = main_phase(3);
    let th = g.add_card_to_battlefield(0, catalog::territorial_hellkite());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let first = g.battlefield_find(th).unwrap().chosen_player.expect("an opponent");
    let other = 3 - first;
    assert!(declare(&mut g.clone(), 0, vec![at(th, other)]).is_err(), "it must attack the chosen one");
    declare(&mut g, 0, vec![at(th, first)]).expect("attack the chosen one");
    run_combat_out(&mut g);
    g.battlefield_find_mut(th).unwrap().tapped = false;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(th).unwrap().chosen_player, Some(other), "not the last one again");
    // Two seats: having attacked the only opponent, it taps.
    let mut g = main_phase(2);
    let th = g.add_card_to_battlefield(0, catalog::territorial_hellkite());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    declare(&mut g, 0, vec![at(th, 1)]).expect("attack");
    run_combat_out(&mut g);
    g.battlefield_find_mut(th).unwrap().tapped = false;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let c = g.battlefield_find(th).unwrap();
    assert!(c.tapped && c.chosen_player.is_none());
}

/// Ureni puts a Dragon from the top eight onto the battlefield.
#[test]
fn ureni_digs_for_a_dragon() {
    let mut g = main_phase(2);
    library(&mut g, 0, 5);
    let gd = g.add_card_to_library(0, catalog::gadrak_the_crown_scourge());
    library(&mut g, 0, 2);
    let u = g.add_card_to_hand(0, catalog::ureni_of_the_unwritten());
    cast_at(&mut g, u, &[]).expect("cast");
    assert!(g.battlefield_find(gd).is_some());
}

/// Will of the Temur with a commander out does both: a 4/4 flying Dragon copy
/// and a draw for your greatest mana value.
#[test]
fn will_of_the_temur_does_both_with_a_commander() {
    let mut g = main_phase(2);
    library(&mut g, 0, 8);
    let cmd = g.seat_commanders(0, vec![catalog::eshki_temurs_roar()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("commander");
    drain_stack(&mut g);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let w = g.add_card_to_hand(0, catalog::will_of_the_temur());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, w, &[Target::Permanent(ring), Target::Player(0)]).expect("cast");
    let copy = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Sol Ring").expect("the copy");
    let cp = g.computed_permanent(copy.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&Keyword::Flying) && copy.definition.subtypes.creature_types.contains(&CreatureType::Dragon));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "Eshki is the greatest mana value, 3");
}

/// Zenith Festival exiles the top X, playable through your next turn.
#[test]
fn zenith_festival_exiles_x_to_play() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    library(&mut g, 0, 1);
    let zf = g.add_card_to_hand(0, catalog::zenith_festival());
    cast_x(&mut g, zf, &[], Some(2)).expect("cast X=2");
    let c = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    let perm = c.may_play_until.expect("playable");
    assert_eq!(perm.player, 0);
    assert!(matches!(perm.duration, crabomination::card::MayPlayDuration::EndOfControllersNextTurn));
}
