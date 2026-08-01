//! Invasion (INV) gap wave — taplands, Cameos, Attendants and kicker commons.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_kicked(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Coastal Tower arrives tapped.
#[test]
fn coastal_tower_enters_tapped() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::coastal_tower());
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Ancient Spring cracks for two off-colour pips.
#[test]
fn ancient_spring_cracks_for_two_colors() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::ancient_spring());
    g.battlefield.iter_mut().find(|c| c.id == land).unwrap().tapped = false;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("crack");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    assert!(g.battlefield.iter().all(|c| c.id != land), "sacrificed");
}

/// Crosis's Attendant pays out three colours.
#[test]
fn crosiss_attendant_cracks_for_grixis() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::crosiss_attendant());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: golem,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("crack");
    for c in [Color::Blue, Color::Black, Color::Red] {
        assert_eq!(g.players[0].mana_pool.amount(c), 1);
    }
}

/// Alloy Golem names its colour as it enters.
#[test]
fn alloy_golem_is_the_chosen_color() {
    let mut g = main_phase();
    let hand = g.add_card_to_hand(0, catalog::alloy_golem());
    cast(&mut g, 0, hand, None);
    let golem = g.battlefield.iter().find(|c| c.definition.name == "Alloy Golem").unwrap();
    let chosen = golem.chosen_color.expect("a color was chosen");
    assert_eq!(g.computed_permanent(golem.id).unwrap().colors, vec![chosen]);
}

/// Benalish Lancer's kicker is two counters and first strike.
#[test]
fn benalish_lancer_kicked_is_a_first_striker() {
    let mut g = main_phase();
    let lancer = g.add_card_to_hand(0, catalog::benalish_lancer());
    cast_kicked(&mut g, 0, lancer, None);
    let cp = g.computed_permanent(lancer).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Ardent Soldier stays a 1/2 when it isn't kicked.
#[test]
fn ardent_soldier_unkicked_gets_no_counter() {
    let mut g = main_phase();
    let soldier = g.add_card_to_hand(0, catalog::ardent_soldier());
    cast(&mut g, 0, soldier, None);
    assert_eq!(g.battlefield_find(soldier).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Benalish Emissary's land kill only fires on the kicker.
#[test]
fn benalish_emissary_kicked_kills_a_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let emissary = g.add_card_to_hand(0, catalog::benalish_emissary());
    cast_kicked(&mut g, 0, emissary, Some(Target::Permanent(land)));
    assert!(g.battlefield.iter().all(|c| c.id != land));
}

/// Backlash turns a creature's power on its own controller.
#[test]
fn backlash_hits_the_creatures_controller() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::backlash());
    cast(&mut g, 0, spell, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(bears).unwrap().tapped);
    assert_eq!(g.players[1].life, 18);
}

/// Agonizing Demise's kicker bills the corpse's controller for its power.
#[test]
fn agonizing_demise_kicked_burns_for_power() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::agonizing_demise());
    cast_kicked(&mut g, 0, spell, Some(Target::Permanent(bears)));
    assert!(g.battlefield.iter().all(|c| c.id != bears));
    assert_eq!(g.players[1].life, 18);
}

/// Breath of Darigaaz spares fliers and scales with the kicker.
#[test]
fn breath_of_darigaaz_kicked_sweeps_the_ground() {
    let mut g = main_phase();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::silver_drake());
    let spell = g.add_card_to_hand(0, catalog::breath_of_darigaaz());
    cast_kicked(&mut g, 0, spell, None);
    assert!(g.battlefield.iter().all(|c| c.id != ground), "2/2 died to 4");
    assert_eq!(g.battlefield_find(flier).map(|c| c.damage), Some(0), "fliers are spared");
    assert_eq!(g.players[1].life, 16);
}

/// Armadillo Cloak drains for whatever the enchanted creature deals.
#[test]
fn armadillo_cloak_drains_on_damage() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cloak = g.add_card_to_hand(0, catalog::armadillo_cloak());
    cast(&mut g, 0, cloak, Some(Target::Permanent(bears)));
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Trample));
    let before = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1),
        4,
        Some(bears),
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 4);
}

/// Exotic Curse shrinks by Domain.
#[test]
fn exotic_curse_shrinks_by_domain() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::island, catalog::swamp] {
        g.add_card_to_battlefield(0, land());
    }
    let ogre = g.add_card_to_battlefield(1, catalog::shivan_wurm());
    let curse = g.add_card_to_hand(0, catalog::exotic_curse());
    cast(&mut g, 0, curse, Some(Target::Permanent(ogre)));
    let cp = g.computed_permanent(ogre).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "7/7 minus three basic types");
}

/// Addle takes exactly one card off the revealed hand.
#[test]
fn addle_takes_one_card_from_the_revealed_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let addle = g.add_card_to_hand(0, catalog::addle());
    cast(&mut g, 0, addle, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Dredge's additional cost is a permanent off your own board.
#[test]
fn dredge_pays_with_a_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let dredge = g.add_card_to_hand(0, catalog::dredge());
    cast(&mut g, 0, dredge, None);
    assert!(g.battlefield.iter().all(|c| c.id != land), "the land paid for it");
}

/// Elfhame Sanctuary trades the draw step for a basic.
#[test]
fn elfhame_sanctuary_fetches_and_skips_the_draw() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::elfhame_sanctuary());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
    assert_eq!(g.players[0].skip_next_draw_step, 1);
}

/// Angelic Shield props up the team and cashes in for a bounce.
#[test]
fn angelic_shield_pumps_toughness_then_bounces() {
    let mut g = main_phase();
    let shield = g.add_card_to_battlefield(0, catalog::angelic_shield());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bears).unwrap().toughness, 3);
    activate(&mut g, 0, shield, 0, Some(Target::Permanent(bears)));
    assert!(g.battlefield.iter().all(|c| c.id != bears && c.id != shield));
}

/// Cinder Shade throws itself for its pumped power.
#[test]
fn cinder_shade_throws_its_pumped_power() {
    let mut g = main_phase();
    let shade = g.add_card_to_battlefield(0, catalog::cinder_shade());
    let victim = g.add_card_to_battlefield(1, catalog::shivan_wurm());
    activate(&mut g, 0, shade, 0, None);
    activate(&mut g, 0, shade, 1, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(2));
}
