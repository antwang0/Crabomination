//! Functionality tests for `catalog::sets::decks::recent152`.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::two_player_game;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Rowan's Grim Search draws two and loses 2 life.
#[test]
fn rowans_grim_search_draws_and_loses() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::rowans_grim_search());
    let hand_before = g.players[0].hand.len();
    let life = g.players[0].life;
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rowan's Grim Search");
    drain_stack(&mut g);
    // -1 (cast) + 2 (draw) = +1 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two");
    assert_eq!(g.players[0].life, life - 2, "lost 2 life");
}

/// Rite of the Moth reanimates a graveyard creature with a finality counter.
#[test]
fn rite_of_the_moth_reanimates_with_finality() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rite_of_the_moth());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rite of the Moth");
    drain_stack(&mut g);
    let c = g.battlefield_find(dead).expect("reanimated to the battlefield");
    assert_eq!(c.counter_count(CounterType::Finality), 1, "with a finality counter");
}

/// Hazel's Nocturne returns up to two graveyard creatures and drains 2.
#[test]
fn hazels_nocturne_recurs_and_drains() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::hazels_nocturne());
    fill_mana(&mut g);
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hazel's Nocturne");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b),
        "both creatures returned to hand");
    assert_eq!(g.players[1].life, opp_life - 2, "opponent lost 2");
    assert_eq!(g.players[0].life, my_life + 2, "you gained 2");
}

/// Form a Posse creates X Mercenary tokens.
#[test]
fn form_a_posse_makes_x_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::form_a_posse());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Form a Posse with X=3");
    drain_stack(&mut g);
    let mercs = g.battlefield.iter().filter(|c| c.definition.name == "Mercenary" && c.controller == 0).count();
    assert_eq!(mercs, 3, "made three Mercenary tokens");
}

/// Otterball Antics makes a prowess Otter (no counter when cast from hand).
#[test]
fn otterball_antics_makes_otter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::otterball_antics());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Otterball Antics");
    drain_stack(&mut g);
    let otter = g.battlefield.iter().find(|c| c.definition.name == "Otter" && c.controller == 0)
        .expect("made an Otter");
    assert_eq!(otter.counter_count(CounterType::PlusOnePlusOne), 0, "no counter when cast from hand");
    assert!(otter.definition.keywords.contains(&crate::card::Keyword::Prowess), "has prowess");
}
