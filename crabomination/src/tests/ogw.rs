//! Functionality tests for the OGW/Eldrazi card pack — Devoid (CR 702.114)
//! and Ingest (CR 702.115).

use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::TurnStep;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 702.114 — Devoid makes a card colorless despite its colored pips.
#[test]
fn cr_702_114_devoid_card_is_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mist_intruder());
    let cp = g.computed_permanent(id).expect("on battlefield");
    assert!(cp.colors.is_empty(), "Devoid creature is colorless despite {{U}} in cost");
}

/// CR 702.115 — Ingest exiles the top card of the damaged player's library
/// when this creature deals combat damage to them.
#[test]
fn cr_702_115_ingest_exiles_top_of_library_on_combat_damage() {
    let mut g = two_player_game();
    // Give the defender a known library to exile from.
    for _ in 0..3 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib_before = g.players[1].library.len();
    let exile_before = g.exile.len();
    let atk = g.add_card_to_battlefield(0, catalog::mist_intruder());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 1, "ingest removes one from library");
    assert_eq!(g.exile.len(), exile_before + 1, "ingested card lands in exile");
}

/// Sludge Crawler's {2} pump grows it +1/+1 until end of turn.
#[test]
fn sludge_crawler_pumps_for_two_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sludge_crawler());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2), "Sludge Crawler is 2/2 after pump");
}

/// Touch of the Void is a Devoid sorcery dealing 3.
#[test]
fn touch_of_the_void_deals_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::touch_of_the_void());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].life;
    crate::game::cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, before - 3, "Touch of the Void deals 3");
    assert!(catalog::touch_of_the_void().keywords.contains(&crate::card::Keyword::Devoid));
}

fn scion_count(g: &GameState) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Scion").count()
}

/// Eldrazi Skyspawner makes a Scion on ETB; the Scion sacs for {C}.
#[test]
fn eldrazi_skyspawner_makes_scion_that_sacs_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eldrazi_skyspawner());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 1, "ETB mints one Eldrazi Scion");
    let scion = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Scion").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scion, ability_index: 0, target: None, x_value: None,
    }).expect("sac scion for mana");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "Scion sacs for {{C}}");
    assert_eq!(scion_count(&g), 0, "Scion is sacrificed");
}

/// Call the Scions mints two Eldrazi Scions.
#[test]
fn call_the_scions_makes_two_scions() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::call_the_scions());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 2, "Call the Scions makes two Scions");
}

/// Blisterpod mints a Scion when it dies.
#[test]
fn blisterpod_makes_scion_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::blisterpod());
    drain_stack(&mut g);
    assert_eq!(scion_count(&g), 0);
    // Lethal damage + SBA → death trigger.
    g.battlefield_find_mut(id).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(scion_count(&g), 1, "Blisterpod's death mints a Scion");
}

/// Eyeless Watcher mints two Scions on ETB.
#[test]
fn eyeless_watcher_makes_two_scions() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eyeless_watcher());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast(&mut g, id);
    assert_eq!(scion_count(&g), 2);
}

/// Eldrazi Devastator is an 8/9 colorless trampler.
#[test]
fn eldrazi_devastator_is_colorless_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 9));
    assert!(cp.colors.is_empty(), "generic-only cost → colorless");
    assert!(cp.keywords.contains(&crate::card::Keyword::Trample));
}

/// Incubator Drone and Catacomb Sifter each mint a Scion on ETB.
#[test]
fn incubator_and_catacomb_sifter_make_scions() {
    let mut g = two_player_game();
    let inc = g.add_card_to_hand(0, catalog::incubator_drone());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    crate::game::cast(&mut g, inc);
    assert_eq!(scion_count(&g), 1);
    let cs = g.add_card_to_hand(0, catalog::catacomb_sifter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    crate::game::cast(&mut g, cs);
    assert_eq!(scion_count(&g), 2, "Catacomb Sifter adds a second Scion");
}

/// Warden of Geometries taps for {C}; Cultivator Drone taps for {C}{C}.
#[test]
fn devoid_mana_dorks_tap_for_colorless() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::warden_of_geometries());
    let c = g.add_card_to_battlefield(0, catalog::cultivator_drone());
    g.clear_sickness(w);
    g.clear_sickness(c);
    g.perform_action(GameAction::ActivateAbility {
        card_id: w, ability_index: 0, target: None, x_value: None,
    }).expect("warden taps");
    g.perform_action(GameAction::ActivateAbility {
        card_id: c, ability_index: 0, target: None, x_value: None,
    }).expect("cultivator taps");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3, "{{C}} + {{C}}{{C}}");
}

/// Reality Hemorrhage is a Devoid burn instant dealing 2.
#[test]
fn reality_hemorrhage_deals_two_and_is_colorless() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::reality_hemorrhage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[1].life;
    crate::game::cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[1].life, before - 2, "Reality Hemorrhage deals 2");
}
