//! Functionality tests for the `catalog::sets::decks::recent10` batch.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// Glider Kids scrys on ETB (and flies).
#[test]
fn glider_kids_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::glider_kids());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.permanent_has_keyword(id, &Keyword::Flying));
}

/// Messenger Hawk creates a Clue on ETB.
#[test]
fn messenger_hawk_makes_clue() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::messenger_hawk());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
}

/// Ostrich-Horse mills three and grabs a land.
#[test]
fn ostrich_horse_mills_and_grabs_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::ostrich_horse());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "land went to hand");
}

/// Rowdy Snowballers taps an opposing creature on ETB.
#[test]
fn rowdy_snowballers_taps_opponent() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rowdy_snowballers());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(victim));
    assert!(g.battlefield_find(victim).unwrap().tapped, "opposing creature tapped");
}

/// Treetop Freedom Fighters mints a 1/1 Ally and has haste.
#[test]
fn treetop_freedom_fighters_makes_ally() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::treetop_freedom_fighters());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.permanent_has_keyword(id, &Keyword::Haste));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ally"));
}

/// Pirate Peddlers grows when you sacrifice another permanent.
#[test]
fn pirate_peddlers_grows_on_sacrifice() {
    let mut g = two_player_game();
    let peddler = g.add_card_to_battlefield(0, catalog::pirate_peddlers());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.sacrifice_one(fodder, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(peddler).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Iguana Parrot has flying, vigilance, and prowess.
#[test]
fn iguana_parrot_keywords() {
    let d = catalog::iguana_parrot();
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert!(d.keywords.contains(&Keyword::Prowess));
}

/// Boar-q-pine grows when you cast a noncreature spell.
#[test]
fn boar_q_pine_grows_on_noncreature() {
    let mut g = two_player_game();
    let boar = g.add_card_to_battlefield(0, catalog::boar_q_pine());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(
        g.battlefield_find(boar).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Knowledge Seeker creates a Clue when it dies.
#[test]
fn knowledge_seeker_makes_clue_on_death() {
    let mut g = two_player_game();
    let seeker = g.add_card_to_battlefield(0, catalog::knowledge_seeker());
    g.remove_to_graveyard_with_triggers(seeker);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
}
