//! Commander: the Witherbloom Witchcraft precon (C21, Willowdusk,
//! `decks::cmdr_willowdusk`).

use crabomination::card::{CardId, CounterType, Keyword};
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

/// Willowdusk reads the greater of life gained and life lost this turn.
#[test]
fn willowdusk_counts_the_greater_of_gained_and_lost() {
    let mut g = pod(2);
    let wd = g.add_card_to_battlefield(0, catalog::willowdusk_essence_seer());
    g.clear_sickness(wd);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 1;
    g.players[0].life_lost_this_turn = 3;
    activate(&mut g, 0, wd, 0, Some(Target::Permanent(bear)), None).expect("activate");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Blight Mound turns a nontoken death into a Pest, and attacking Pests get
/// +1/+0 and menace.
#[test]
fn blight_mound_makes_pests_that_hit_harder() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::blight_mound());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear))).expect("bolt");
    let pests = named(&g, 0, "Pest");
    assert_eq!(pests.len(), 1);
    g.priority.player_with_priority = 0;
    attack_with(&mut g, &pests, 1);
    assert_eq!(pt(&g, pests[0]), (2, 1));
    assert!(g.computed_permanent(pests[0]).unwrap().keywords().contains(&Keyword::Menace));
}

/// Druidic Satchel: a creature on top makes a Saproling, a land goes onto the
/// battlefield, anything else gains 2 life.
#[test]
fn druidic_satchel_branches_on_the_top_card() {
    let mut g = pod(2);
    let sat = g.add_card_to_battlefield(0, catalog::druidic_satchel());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let land = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.players[0].library.reverse();
    activate(&mut g, 0, sat, 0, None, None).expect("creature");
    assert_eq!(named(&g, 0, "Saproling").len(), 1);
    g.players[0].library.remove(0);
    g.battlefield_find_mut(sat).unwrap().tapped = false;
    activate(&mut g, 0, sat, 0, None, None).expect("land");
    assert!(g.battlefield_find(land).is_some());
    g.battlefield_find_mut(sat).unwrap().tapped = false;
    activate(&mut g, 0, sat, 0, None, None).expect("other");
    assert_eq!(g.players[0].life, life + 2);
}

/// Essence Pulse shrinks every creature by all the life gained this turn,
/// its own 2 included.
#[test]
fn essence_pulse_shrinks_by_life_gained() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.players[0].life_gained_this_turn = 1;
    let ep = g.add_card_to_hand(0, catalog::essence_pulse());
    cast(&mut g, 0, ep, None).expect("cast");
    assert!(g.battlefield_find(bear).is_none(), "-3/-3");
    assert_eq!(pt(&g, angel), (1, 1));
}

/// Ezzaroot Channeler discounts creature spells by the life gained this turn.
#[test]
fn ezzaroot_channeler_discounts_creatures_by_life_gained() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ezzaroot_channeler());
    g.players[0].life_gained_this_turn = 3;
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell { card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{3}{W}{W} for {W}{W}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_some());
}

/// Gift of Paradise gains 3 on entering and the land taps for two of a color.
#[test]
fn gift_of_paradise_gains_three_and_doubles_the_land() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let life = g.players[0].life;
    let gift = g.add_card_to_hand(0, catalog::gift_of_paradise());
    cast(&mut g, 0, gift, Some(Target::Permanent(forest))).expect("cast");
    assert_eq!(g.players[0].life, life + 3);
    g.players[0].mana_pool = Default::default();
    let n = catalog::forest().activated_abilities.len() + 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: n - 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the granted ability");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Gluttonous Troll makes one Food per opponent.
#[test]
fn gluttonous_troll_feeds_per_opponent() {
    let mut g = pod(4);
    let troll = g.add_card_to_hand(0, catalog::gluttonous_troll());
    cast(&mut g, 0, troll, None).expect("cast");
    assert_eq!(named(&g, 0, "Food").len(), 3);
}

/// Gyome's end step counts nontoken creatures that entered this turn.
#[test]
fn gyome_cooks_a_food_per_nontoken_entry() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gyome_master_chef());
    for _ in 0..2 {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, 0, bear, None).expect("cast");
    }
    let gc = g.add_card_to_hand(0, catalog::grand_crescendo());
    cast_x(&mut g, 0, gc, None, Some(2)).expect("two tokens");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Food").len(), 2, "the two Bears, not the Citizens");
}

/// Marshland Bloodcaster lets the next spell be paid with life, once.
#[test]
fn marshland_bloodcaster_pays_the_next_spell_with_life() {
    let mut g = pod(2);
    let mb = g.add_card_to_battlefield(0, catalog::marshland_bloodcaster());
    g.clear_sickness(mb);
    activate(&mut g, 0, mb, 0, None, None).expect("activate");
    g.players[0].mana_pool = Default::default();
    let life = g.players[0].life;
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: angel,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("five life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 5);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: bear,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the next spell only"
    );
}

/// Paradise Plume gains 1 when anyone casts a spell of the chosen color.
#[test]
fn paradise_plume_gains_on_the_chosen_color() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Color(Color::Red), DecisionAnswer::Bool(true)]));
    let plume = g.add_card_to_hand(0, catalog::paradise_plume());
    cast(&mut g, 0, plume, None).expect("cast");
    let life = g.players[0].life;
    g.active_player_idx = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    cast(&mut g, 1, bear, None).expect("green");
    assert_eq!(g.players[0].life, life);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0))).expect("red");
    assert_eq!(g.players[0].life, life + 1 - 3);
}

/// CR 110.4 — Revival Experiment returns one card per permanent type, a
/// multi-typed card counting once, and costs 3 life a card.
#[test]
fn revival_experiment_returns_one_per_permanent_type() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let life = g.players[0].life;
    let re = g.add_card_to_hand(0, catalog::revival_experiment());
    cast(&mut g, 0, re, None).expect("cast");
    assert!(g.battlefield_find(ring).is_some() && g.battlefield_find(forest).is_some());
    assert!(g.battlefield_find(giant).is_some(), "the costlier creature");
    assert!(g.battlefield_find(bear).is_none(), "one creature");
    assert_eq!(g.players[0].life, life - 9);
    assert!(g.exile.iter().any(|c| c.id == re));
}

/// Sapling of Colfenor's attack reveals a creature: gain its toughness, lose
/// its power, take it.
#[test]
fn sapling_of_colfenor_takes_a_revealed_creature() {
    let mut g = pod(2);
    let top = g.add_card_to_library(0, catalog::trostani_selesnyas_voice());
    let sap = g.add_card_to_battlefield(0, catalog::sapling_of_colfenor());
    let life = g.players[0].life;
    attack_with(&mut g, &[sap], 1);
    assert_eq!(g.players[0].life, life + 5 - 2);
    assert!(g.players[0].hand.iter().any(|c| c.id == top));
}

/// Suffer the Past exiles X cards from one graveyard and drains per card.
#[test]
fn suffer_the_past_drains_per_exiled_card() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let stp = g.add_card_to_hand(0, catalog::suffer_the_past());
    cast_x(&mut g, 0, stp, Some(Target::Player(1)), Some(2)).expect("cast");
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!((g.players[0].life, g.players[1].life), (l0 + 2, l1 - 2));
}

/// Tivash trades the life gained this turn for an X/X flying Demon.
#[test]
fn tivash_pays_the_gain_for_a_demon() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::tivash_gloom_summoner());
    g.players[0].life_gained_this_turn = 4;
    let life = g.players[0].life;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let demon = named(&g, 0, "Demon");
    assert_eq!(demon.len(), 1);
    assert_eq!(pt(&g, demon[0]), (4, 4));
    assert_eq!(g.players[0].life, life - 4);
}

/// Trudge Garden and Veinwitch Coven both pay off a life gain.
#[test]
fn trudge_garden_and_veinwitch_coven_pay_off_life_gain() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::trudge_garden());
    g.add_card_to_battlefield(0, catalog::veinwitch_coven());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let gift = g.add_card_to_hand(0, catalog::gift_of_paradise());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    cast(&mut g, 0, gift, Some(Target::Permanent(forest))).expect("gain 3");
    assert_eq!(named(&g, 0, "Fungus Beast").len(), 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Yedora returns a dead creature face down as a Forest land, which its
/// mana cost cannot turn face up.
#[test]
fn yedora_returns_the_dead_as_a_face_down_forest() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::yedora_grave_gardener());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear))).expect("bolt");
    let c = g.battlefield_find(bear).expect("back as a land");
    assert!(c.face_down && c.definition.is_land() && !c.definition.is_creature());
    assert!(g.turn_up_mana_cost(0, bear, 0).is_none());
}
