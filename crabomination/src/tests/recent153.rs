//! Functionality tests for `catalog::sets::decks::recent153`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::game::two_player_game;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Gold Pan makes a Treasure on entry and buffs the creature it equips.
#[test]
fn gold_pan_makes_treasure_and_buffs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pan = g.move_card_to_battlefield_for_test(0, catalog::gold_pan());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
        "ETB minted a Treasure");
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: pan, target: bear })
        .expect("equip Gold Pan");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "equipped creature +1/+1");
}

/// Conductive Machete manifests a dread creature and equips it (+2/+1).
#[test]
fn conductive_machete_manifest_and_equip() {
    let mut g = two_player_game();
    let id1 = g.next_id();
    g.players[0].add_to_library_top(id1, catalog::grizzly_bears());
    let id2 = g.next_id();
    g.players[0].add_to_library_top(id2, catalog::forest());
    let machete = g.move_card_to_battlefield_for_test(0, catalog::conductive_machete());
    drain_stack(&mut g);
    let manifest = g.battlefield.iter().find(|c| c.face_down && c.controller == 0).map(|c| c.id);
    assert!(manifest.is_some(), "manifested a face-down creature");
    assert_eq!(g.battlefield_find(machete).unwrap().attached_to, manifest, "attached to the manifest");
    let c = g.computed_permanent(manifest.unwrap()).unwrap();
    assert_eq!((c.power, c.toughness), (4, 3), "2/2 manifest +2/+1 = 4/3");
}

/// Baron Bertram Graywater mints a Vampire when a token you control enters.
#[test]
fn baron_makes_vampire_on_token_enter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::baron_bertram_graywater());
    let tok = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    g.dispatch_triggers_for_events(&[GameEvent::TokenCreated { card_id: tok }]);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Vampire" && c.controller == 0),
        "a token entering minted a Vampire");
}

/// Jem Lightfoote draws at end step when you haven't cast a spell.
#[test]
fn jem_lightfoote_draws_when_spell_free() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jem_lightfoote_sky_explorer());
    g.add_card_to_library(0, catalog::forest());
    g.active_player_idx = 0;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card at end step (no spell cast)");
}

/// Canyon Crab's activated ability shifts it +2/-2.
#[test]
fn canyon_crab_pump_shifts_stats() {
    let mut g = two_player_game();
    let crab = g.add_card_to_battlefield(0, catalog::canyon_crab());
    g.clear_sickness(crab);
    fill_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crab, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Canyon Crab");
    drain_stack(&mut g);
    let c = g.computed_permanent(crab).unwrap();
    assert_eq!((c.power, c.toughness), (2, 3), "0/5 → 2/3 after +2/-2");
    assert!(!c.keywords.contains(&Keyword::Flying), "no flying (sanity)");
}
