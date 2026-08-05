//! Duskmourn gap batch 2 (`decks::dsk2`).

use crabomination::card::{CardDefinition, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

/// Four card types in your graveyard.
fn stock_delirium(g: &mut GameState) {
    for def in [
        catalog::grizzly_bears(),
        catalog::lightning_bolt(),
        catalog::forest(),
        catalog::sol_ring(),
    ] {
        g.add_card_to_graveyard(0, def);
    }
}

/// Let's Play a Game runs one mode bare and every mode on delirium.
#[test]
fn lets_play_a_game_escalates_on_delirium() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_hand(1, catalog::island());
    }
    stock_delirium(&mut g);
    let spell = g.add_card_to_hand(0, catalog::lets_play_a_game());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "the discard mode ran");
    assert_eq!(g.players[1].life, 17, "and so did the drain mode");
}

/// Marina Vendrell pulls the enchantments out of the top seven.
#[test]
fn marina_vendrell_digs_seven_for_enchantments() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::mirror_room_fractured_realm());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    etb(&mut g, catalog::marina_vendrell());
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Mirror Room // Fractured Realm"),
        "the enchantment came to hand"
    );
}

/// Her tap ability opens a locked door.
#[test]
fn marina_vendrell_unlocks_a_door() {
    let mut g = main_phase();
    let marina = g.add_card_to_battlefield(0, catalog::marina_vendrell());
    g.battlefield_find_mut(marina).unwrap().summoning_sick = false;
    let room = g.add_card_to_battlefield(0, catalog::dazzling_theater_prop_room());
    g.perform_action(GameAction::ActivateAbility {
        card_id: marina,
        ability_index: 0,
        target: Some(Target::Permanent(room)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("work the door");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(room).unwrap().unlocked_doors, 0b01, "a door opened");
}

/// The Grimoire turns a life swing into cards.
#[test]
fn grimoire_draws_on_life_gain() {
    let mut g = main_phase();
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::island());
    }
    etb(&mut g, catalog::marina_vendrells_grimoire());
    assert_eq!(g.players[0].hand.len(), 0, "the draw-5 is gated on having cast it");
    let before = g.players[0].hand.len();
    g.adjust_life(0, 3);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 3, "gained 3 → drew 3");
}

/// Marvin borrows another creature's activated ability.
#[test]
fn marvin_wears_another_creatures_ability() {
    let mut g = main_phase();
    let marvin = g.add_card_to_battlefield(0, catalog::marvin_murderous_mimic());
    assert!(g.granted_abilities_for(marvin).is_empty(), "nothing to copy yet");
    g.add_card_to_battlefield(0, catalog::prodigal_sorcerer());
    assert_eq!(g.granted_abilities_for(marvin).len(), 1, "Tim's ping is Marvin's too");
}

/// Meathook Massacre II sweeps for X and buys the bodies back.
#[test]
fn meathook_massacre_ii_sweeps_then_reanimates() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hook = g.add_card_to_hand(0, catalog::meathook_massacre_ii());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: hook,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("cast for X=1");
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("bought back for 3 life");
    assert_eq!(back.counter_count(CounterType::Finality), 1, "with a finality counter");
    assert_eq!(g.players[0].life, 17);
}

/// Nashi grows when the mill turns up nothing worth keeping.
#[test]
fn nashi_grows_when_the_mill_whiffs() {
    let mut g = main_phase();
    let nashi = g.add_card_to_battlefield(0, catalog::nashi_searcher_in_the_dark());
    g.battlefield_find_mut(nashi).unwrap().summoning_sick = false;
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: nashi,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 2, "milled 2 for the 2 damage");
    assert_eq!(
        g.battlefield_find(nashi).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "no keeper → a +1/+1 counter"
    );
}

/// The Mindskinner replaces its damage with a mill.
#[test]
fn mindskinner_mills_instead_of_burning() {
    let mut g = main_phase();
    let skinner = g.add_card_to_battlefield(0, catalog::the_mindskinner());
    for _ in 0..12 {
        g.add_card_to_library(1, catalog::island());
    }
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(1), 3, Some(skinner), &mut evs);
    assert_eq!(g.players[1].life, 20, "no life lost");
    assert_eq!(g.players[1].graveyard.len(), 3, "milled 3 instead");
}

/// The Rollercrusher Ride doubles your noncombat damage under delirium.
#[test]
fn rollercrusher_ride_doubles_under_delirium() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::the_rollercrusher_ride());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    stock_delirium(&mut g);
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "3 became 6");
}

/// Tyvar's pump reads the biggest body on your board.
#[test]
fn tyvar_pumps_by_greatest_power() {
    let mut g = main_phase();
    let tyvar = g.add_card_to_battlefield(0, catalog::tyvar_the_pummeler());
    g.battlefield_find_mut(tyvar).unwrap().summoning_sick = false;
    g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tyvar,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(tyvar).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "+5/+5 from the Dragon");
}
