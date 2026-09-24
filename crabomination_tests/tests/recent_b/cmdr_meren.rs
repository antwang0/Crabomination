//! Commander: the Plunder the Graves precon (C15, Meren, `decks::cmdr_meren`).

use crabomination::card::{CardId, CounterType};
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

fn connect(g: &mut GameState, attackers: &[CardId], defender: usize) {
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
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

#[test]
fn banshee_makes_the_player_it_hits_discard() {
    let mut g = main_phase(2);
    g.add_card_to_hand(1, catalog::swamp());
    let b = g.add_card_to_battlefield(0, catalog::banshee_of_the_dread_choir());
    connect(&mut g, &[b], 1);
    assert!(g.players[1].hand.is_empty());
}

#[test]
fn blood_bairn_eats_another_creature() {
    let mut g = main_phase(2);
    let bairn = g.add_card_to_battlefield(0, catalog::blood_bairn());
    let food = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, bairn, 0, None).expect("sacrifice");
    assert!(g.battlefield_find(food).is_none());
    assert_eq!(pt(&g, bairn), (4, 4));
}

/// Devour 1, then each other creature you control enters with that many
/// extra +1/+1 counters.
#[test]
fn bloodspore_thrinax_feeds_the_next_arrivals() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::bloodspore_thrinax());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
    cast(&mut g, t, &[]);
    let counters = g.battlefield_find(t).expect("thrinax").counter_count(CounterType::PlusOnePlusOne);
    assert!(counters >= 1, "devoured something");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]);
    assert_eq!(g.battlefield_find(bear).expect("bear").counter_count(CounterType::PlusOnePlusOne), counters);
}

#[test]
fn centaur_vinecrasher_counts_every_graveyards_lands() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(1, catalog::swamp());
    g.add_card_to_graveyard(1, catalog::swamp());
    let v = g.add_card_to_hand(0, catalog::centaur_vinecrasher());
    cast(&mut g, v, &[]);
    assert_eq!(pt(&g, v), (4, 4));
}

/// From the graveyard, a land card hitting any graveyard offers {G}{G} to
/// return it to hand.
#[test]
fn centaur_vinecrasher_climbs_out_when_a_land_is_milled() {
    let mut g = main_phase(2);
    let v = g.add_card_to_graveyard(0, catalog::centaur_vinecrasher());
    g.add_card_to_library(1, catalog::swamp());
    g.add_card_to_library(1, catalog::swamp());
    g.add_card_to_library(0, catalog::swamp());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mill = g.add_card_to_hand(0, catalog::thought_scour());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: mill,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("mill");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == v));
}

#[test]
fn cloudthresher_evoked_clears_the_skies() {
    let mut g = main_phase(2);
    let bird = g.add_card_to_battlefield(1, catalog::birds_of_paradise());
    let c = g.add_card_to_hand(0, catalog::cloudthresher());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: c,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bird).is_none());
    assert!(g.battlefield_find(c).is_none(), "sacrificed");
    assert_eq!((g.players[0].life, g.players[1].life), (18, 18));
}

#[test]
fn corpse_augur_draws_off_a_graveyard() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::swamp());
    }
    let a = g.add_card_to_battlefield(0, catalog::corpse_augur());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    cast(&mut g, bolt, &[Target::Permanent(a)]);
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].life, 17);
}

#[test]
fn extractor_demon_mills_on_a_creature_leaving() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::extractor_demon());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::swamp());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(victim)]);
    assert_eq!(g.players[1].library.len() + g.players[0].library.len(), 1, "a player milled two");
}

#[test]
fn grim_backwoods_turns_a_creature_into_a_card() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::swamp());
    let land = g.add_card_to_battlefield(0, catalog::grim_backwoods());
    let food = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, land, 1, None).expect("draw");
    assert!(g.battlefield_find(food).is_none());
    assert_eq!(g.players[0].hand.len(), 1);
}

#[test]
fn kessig_cagebreakers_bring_a_wolf_per_creature_card() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let k = g.add_card_to_battlefield(0, catalog::kessig_cagebreakers());
    connect(&mut g, &[k], 1);
    assert_eq!(count_named(&g, 0, "Wolf"), 3);
    assert_eq!(g.players[1].life, 20 - 3 - 6);
}

#[test]
fn mazirek_grows_the_team_on_any_sacrifice() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::mazirek_kraul_death_priest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bairn = g.add_card_to_battlefield(0, catalog::blood_bairn());
    activate(&mut g, bairn, 0, None).expect("sacrifice the Bears");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.battlefield_find(bairn).expect("bairn").counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn mycoloth_spawns_a_saproling_per_counter() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::mycoloth());
    g.battlefield_find_mut(m).expect("mycoloth").add_counters(CounterType::PlusOnePlusOne, 4);
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(count_named(&g, 0, "Saproling"), 4);
}

#[test]
fn sever_the_bloodline_exiles_every_namesake() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::sever_the_bloodline());
    cast(&mut g, s, &[Target::Permanent(a)]);
    assert!([a, b, mine].iter().all(|&id| g.battlefield_find(id).is_none()));
    assert!(g.battlefield_find(other).is_some());
}

#[test]
fn spider_spawning_counts_your_creature_cards() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let s = g.add_card_to_hand(0, catalog::spider_spawning());
    cast(&mut g, s, &[]);
    assert_eq!(count_named(&g, 0, "Spider"), 3);
}

#[test]
fn thief_of_blood_steals_every_counter() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(a).expect("a").add_counters(CounterType::PlusOnePlusOne, 2);
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(b).expect("b").add_counters(CounterType::MinusOneMinusOne, 1);
    let t = g.add_card_to_hand(0, catalog::thief_of_blood());
    cast(&mut g, t, &[]);
    assert_eq!(pt(&g, t), (4, 4));
    assert_eq!(pt(&g, a), (2, 2));
}

#[test]
fn tribute_to_the_wild_takes_from_each_opponent() {
    let mut g = main_phase(3);
    let x = g.add_card_to_battlefield(1, catalog::ornithopter());
    let y = g.add_card_to_battlefield(2, catalog::pacifism());
    let t = g.add_card_to_hand(0, catalog::tribute_to_the_wild());
    cast(&mut g, t, &[]);
    assert!(g.battlefield_find(x).is_none() && g.battlefield_find(y).is_none());
}

/// Default modes: you draw and lose 1, -2/-2 on a creature, a creature card back.
#[test]
fn wretched_confluence_default_modes() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::swamp());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    let wc = g.add_card_to_hand(0, catalog::wretched_confluence());
    cast(&mut g, wc, &[Target::Player(0), Target::Permanent(bear), Target::Permanent(dead)]);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == dead));
    assert_eq!(g.players[0].life, 19);
}
