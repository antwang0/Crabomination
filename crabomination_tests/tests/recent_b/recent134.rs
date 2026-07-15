//! Functionality tests for `catalog::sets::decks::recent134` (WOE wave 7).

use crabomination::card::{CounterType, EnchantmentSubtype, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastAdventure {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast adventure");
    drain_stack(g);
}

/// Entry Denied bounces a small creature.
#[test]
fn entry_denied_bounces_small() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV2
    let card = g.add_card_to_hand(0, catalog::belunas_gatekeeper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_adventure(&mut g, card, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Freeze in Place taps and puts three stun counters on a creature.
#[test]
fn freeze_in_place_taps_and_stuns() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::freeze_in_place());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(enemy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Freeze in Place");
    drain_stack(&mut g);
    let ec = g.battlefield_find(enemy).unwrap();
    assert!(ec.tapped, "tapped");
    assert_eq!(ec.counter_count(CounterType::Stun), 3, "three stun counters");
}

/// Succumb to the Cold taps and stuns two creatures.
#[test]
fn succumb_stuns_two() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::succumb_to_the_cold());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast Succumb to the Cold");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::Stun), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Stun), 1);
}

/// Beat a Path stops a creature from blocking.
#[test]
fn beat_a_path_cant_block() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::bellowing_bruiser());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_adventure(&mut g, card, Some(Target::Permanent(enemy)));
    assert!(g.computed_permanent(enemy).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Gallant Pie-Wielder gains double strike under Celebration.
#[test]
fn gallant_pie_wielder_celebration() {
    let mut g = two_player_game();
    let g_id = g.add_card_to_battlefield(0, catalog::gallant_pie_wielder());
    assert!(!g.computed_permanent(g_id).unwrap().keywords.contains(&Keyword::DoubleStrike));
    g.players[0].nonland_permanents_entered_this_turn = 2;
    assert!(g.computed_permanent(g_id).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Woodland Acolyte draws on entry; Mend the Wilds recurs a graveyard permanent.
#[test]
fn woodland_acolyte_etb_and_mend() {
    let mut g = two_player_game();
    // ETB draw.
    let acolyte = g.add_card_to_battlefield(0, catalog::woodland_acolyte());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    g.fire_self_etb_triggers(acolyte, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "ETB draw");
    // Mend the Wilds from a fresh copy: put a graveyard creature on top.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::woodland_acolyte());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_adventure(&mut g, card, Some(Target::Permanent(bear)));
    assert!(g.players[0].library.first().map(|c| c.id) == Some(bear), "bear on top of library");
}

/// Stroke of Midnight destroys a permanent and gives its controller a Human.
#[test]
fn stroke_of_midnight_destroy_and_human() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::stroke_of_midnight());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(enemy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Stroke of Midnight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(enemy).is_none(), "destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Human"),
        "opponent got the Human",
    );
}

/// Return Triumphant reanimates a small creature with a Young Hero Role.
#[test]
fn return_triumphant_reanimates_with_role() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::return_triumphant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Return Triumphant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear reanimated");
    assert!(
        g.battlefield.iter().any(|c| c.attached_to == Some(bear)
            && c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Role)),
        "Young Hero Role attached",
    );
}

/// Price of Beauty hangs a Wicked Role that drains on death.
#[test]
fn price_of_beauty_wicked_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::conceited_witch());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_adventure(&mut g, card, Some(Target::Permanent(bear)));
    assert!(
        g.battlefield.iter().any(|c| c.attached_to == Some(bear) && c.definition.name == "Wicked"),
        "Wicked Role attached",
    );
}

/// Sugar Rush pumps and draws.
#[test]
fn sugar_rush_pump_and_draw() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sugar_rush());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Sugar Rush");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+0");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one → net same");
}
