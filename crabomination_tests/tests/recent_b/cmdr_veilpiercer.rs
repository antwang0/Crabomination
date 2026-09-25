//! Commander: the Miracle Worker precon (DSC, Aminatou, Veil Piercer,
//! `decks::cmdr_veilpiercer`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, cost, generic, u};

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

fn door(g: &mut GameState, id: CardId, right: bool) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastRoomDoor { card_id: id, right }).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn draw(g: &mut GameState, seat: usize) {
    let mut evs = Vec::new();
    g.draw_one(seat, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Aminatou: the turn's first card drawn, if an enchantment, gets miracle at
/// its cost less {4}; a later draw doesn't.
#[test]
fn aminatou_grants_miracle_to_the_first_enchantment_drawn() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::aminatou_veil_piercer());
    let second = g.add_card_to_library(0, catalog::one_with_the_multiverse());
    let first = g.add_card_to_library(0, catalog::mirrormade());
    let top = g.players[0].library[0].id;
    draw(&mut g, 0);
    draw(&mut g, 0);
    let card = |id| g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
    let (a, b) = if top == first { (first, second) } else { (second, first) };
    let a = card(a);
    assert!(a.may_play_until.is_some_and(|p| p.miracle));
    let want = if a.definition.name == "Mirrormade" { cost(&[u(), u()]) } else { cost(&[generic(2), u(), u()]) };
    assert_eq!(a.granted_alt_cast_cost_eot, Some(want), "{{1}}{{U}}{{U}} or {{6}}{{U}}{{U}} less {{4}}");
    assert!(card(b).may_play_until.is_none(), "not the first draw");
}

/// Cramped Vents deals 6 to an opponent's creature and gains the excess.
#[test]
fn cramped_vents_burns_and_gains_the_excess() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::cramped_vents_access_maze());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let life = g.players[0].life;
    door(&mut g, r, false).expect("cast Cramped Vents");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life + 4);
}

/// Access Maze: once on your turn, a spell from hand for life.
#[test]
fn access_maze_casts_for_life() {
    let mut g = main_phase(2);
    let r = g.add_card_to_hand(0, catalog::cramped_vents_access_maze());
    door(&mut g, r, true).expect("cast Access Maze");
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].mana_pool = Default::default();
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("life");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
    assert_eq!(g.players[0].life, life - 4);
}

/// Diabolic Vision: one of five to hand, the other four stay on top in order.
#[test]
fn diabolic_vision_keeps_the_rest_on_top() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    library(&mut g, 0, 4);
    let top5: Vec<CardId> = g.players[0].library.iter().take(5).map(|c| c.id).collect();
    let dv = g.add_card_to_hand(0, catalog::diabolic_vision());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![top5[0]])]));
    cast_at(&mut g, dv, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == top5[0]));
    let now: Vec<CardId> = g.players[0].library.iter().take(4).map(|c| c.id).collect();
    assert_eq!(now, top5[1..].to_vec());
    assert_eq!(g.players[0].library.len(), 7);
    let _ = bear;
}

/// Fear of Sleep Paralysis taps and stuns; the stun counter never comes off
/// an opponent's permanent, so it stays tapped.
#[test]
fn fear_of_sleep_paralysis_stuns_for_good() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::fear_of_sleep_paralysis());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast_at(&mut g, f, &[]).expect("cast");
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.tapped && c.counter_count(CounterType::Stun) == 1);
    g.active_player_idx = 1;
    g.do_untap();
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.tapped && c.counter_count(CounterType::Stun) == 1, "counter stays, no untap");
}

/// Mirrormade enters as a copy of an artifact.
#[test]
fn mirrormade_copies_an_artifact() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::sol_ring());
    let mm = g.add_card_to_hand(0, catalog::mirrormade());
    cast_at(&mut g, mm, &[]).expect("cast");
    assert_eq!(g.battlefield_find(mm).unwrap().definition.name, "Sol Ring");
}

/// Moon-Blessed Cleric tutors an enchantment to the top.
#[test]
fn moon_blessed_cleric_stacks_an_enchantment() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let e = g.add_card_to_library(0, catalog::life_insurance());
    library(&mut g, 0, 2);
    let c = g.add_card_to_hand(0, catalog::moon_blessed_cleric());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(e))]));
    cast_at(&mut g, c, &[]).expect("cast");
    assert_eq!(g.players[0].library[0].id, e);
}

/// Obscura Storefront trades itself for a tapped basic and 1 life.
#[test]
fn obscura_storefront_fetches_a_basic() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::island());
    let s = g.add_card_to_hand(0, catalog::obscura_storefront());
    let life = g.players[0].life;
    g.perform_action(GameAction::PlayLand(s)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(s).is_none());
    let island = named(&g, 0, "Island");
    assert!(island.len() == 1 && g.battlefield_find(island[0]).unwrap().tapped);
    assert_eq!(g.players[0].life, life + 1);
}

/// One with the Multiverse: cast from the top of the library; once on your
/// turn, a free spell from hand.
#[test]
fn one_with_the_multiverse_plays_the_top_and_one_free() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::one_with_the_multiverse());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].library.rotate_right(0);
    assert_eq!(g.players[0].library[0].id, top);
    cast_at(&mut g, top, &[]).expect("from the top");
    assert!(g.battlefield_find(top).is_some());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("free");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
}

/// Phenomenon Investigators: Believe turns nontoken deaths into Horrors.
#[test]
fn phenomenon_investigators_believe_makes_horrors() {
    let mut g = main_phase(2);
    let pi = g.add_card_to_hand(0, catalog::phenomenon_investigators());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    cast_at(&mut g, pi, &[]).expect("cast");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, m, &[Target::Permanent(bear)]).expect("kill");
    let horrors = named(&g, 0, "Horror");
    assert_eq!(horrors.len(), 1);
    assert!(g.battlefield_find(horrors[0]).unwrap().definition.card_types.contains(&CardType::Enchantment));
}

/// Redress Fate returns every artifact and enchantment card from your
/// graveyard.
#[test]
fn redress_fate_returns_artifacts_and_enchantments() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let li = g.add_card_to_graveyard(0, catalog::life_insurance());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rf = g.add_card_to_hand(0, catalog::redress_fate());
    cast_at(&mut g, rf, &[]).expect("cast");
    assert!(g.battlefield_find(ring).is_some() && g.battlefield_find(li).is_some());
    assert!(g.battlefield_find(bear).is_none());
}

/// Secret Arcade makes your nonland permanents enchantments; Dusty Parlor
/// grows a creature by an enchantment spell's mana value.
#[test]
fn secret_arcade_and_dusty_parlor() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::secret_arcade_dusty_parlor());
    door(&mut g, r, false).expect("Secret Arcade");
    assert!(g.computed_permanent(bear).unwrap().card_types().contains(&CardType::Enchantment));
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::secret_arcade_dusty_parlor());
    door(&mut g, r, true).expect("Dusty Parlor");
    let li = g.add_card_to_hand(0, catalog::life_insurance());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast_at(&mut g, li, &[]).expect("an enchantment spell");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// Soaring Lightbringer: a Glimmer at each player you attack, and your other
/// enchantment creatures fly.
#[test]
fn soaring_lightbringer_sends_glimmers() {
    let mut g = main_phase(3);
    let sl = g.add_card_to_battlefield(0, catalog::soaring_lightbringer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [sl, bear] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: sl, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let glimmers = named(&g, 0, "Glimmer");
    assert_eq!(glimmers.len(), 2);
    let targets: Vec<_> = g.attacking.iter().filter(|a| glimmers.contains(&a.attacker)).map(|a| a.target).collect();
    assert!(targets.contains(&AttackTarget::Player(1)) && targets.contains(&AttackTarget::Player(2)));
    assert!(g.computed_permanent(glimmers[0]).unwrap().keywords().contains(&Keyword::Flying));
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Flying));
}

/// Spirit-Sister's Call: sacrifice a creature to return a creature card,
/// which is exiled if it would leave again.
#[test]
fn spirit_sisters_call_trades_and_exiles() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::spirit_sisters_call());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    fire(&mut g, TurnStep::End);
    assert!(g.battlefield_find(dead).is_some() && g.battlefield_find(fodder).is_none());
    let m = g.add_card_to_hand(0, catalog::murder());
    g.step = TurnStep::PreCombatMain;
    cast_at(&mut g, m, &[Target::Permanent(dead)]).expect("kill");
    assert!(g.exile.iter().any(|c| c.id == dead), "exiled instead");
}

/// The Master of Keys: X counters and 2X milled; an enchantment card in the
/// graveyard escapes.
#[test]
fn the_master_of_keys_mills_and_escapes_enchantments() {
    let mut g = main_phase(2);
    library(&mut g, 0, 6);
    let mk = g.add_card_to_hand(0, catalog::the_master_of_keys());
    cast_x(&mut g, mk, &[], Some(2)).expect("cast X=2");
    assert_eq!(g.battlefield_find(mk).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.players[0].graveyard.len(), 4);
    let li = g.add_card_to_graveyard(0, catalog::life_insurance());
    let fodder: Vec<CardId> = g.players[0].graveyard.iter().filter(|c| c.id != li).take(3).map(|c| c.id).collect();
    flood(&mut g, 0);
    g.perform_action(GameAction::CastEscape {
        card_id: li,
        exile_cards: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("escape");
    drain_stack(&mut g);
    assert!(g.battlefield_find(li).is_some());
}
