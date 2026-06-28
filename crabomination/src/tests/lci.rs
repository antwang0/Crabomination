//! Functionality tests for the LCI batch — Descend / fathomless descent and
//! assorted commons riding existing primitives.

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::*;
use crate::mana::Color;

/// Souls of the Lost is */*+1 = permanent cards in your graveyard.
#[test]
fn souls_of_the_lost_pt_tracks_graveyard_permanents() {
    let mut g = two_player_game();
    let soul = g.add_card_to_battlefield(0, catalog::souls_of_the_lost());
    // Empty graveyard → 0/1.
    let c = g.computed_permanent(soul).unwrap();
    assert_eq!((c.power, c.toughness), (0, 1));
    // Three permanent cards + an instant → power 3, toughness 4 (instant ignored).
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let c = g.computed_permanent(soul).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4));
}

/// Frilled Cave-Wurm gets +2/+0 only with 4+ permanent cards in the graveyard.
#[test]
fn frilled_cave_wurm_descend_4_pump() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::frilled_cave_wurm());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 2, "descend 3 → base 2/5");
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // → 4
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 4, "descend 4 → +2/+0");
}

/// Coati Scavenger returns a permanent card from the graveyard on ETB once
/// descend 4 is active.
#[test]
fn coati_scavenger_descend_4_recurs_permanent() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let coati = g.add_card_to_hand(0, catalog::coati_scavenger());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: coati, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // One bear returned to hand (the ETB trigger auto-targets a gy permanent card).
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "recurred a creature");
}

/// Acolyte of Aclazotz drains 1 by tapping and sacrificing another permanent.
#[test]
fn acolyte_of_aclazotz_drains_on_sac() {
    let mut g = two_player_game();
    let acolyte = g.add_card_to_battlefield(0, catalog::acolyte_of_aclazotz());
    g.clear_sickness(acolyte);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let start = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: acolyte, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, start - 1, "opponent lost 1");
}

/// Poison Dart Frog's {2} ability grants deathtouch until end of turn.
#[test]
fn poison_dart_frog_grants_deathtouch() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::poison_dart_frog());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    // Ability index 1 is the {2}: deathtouch (index 0 is the mana ability).
    g.perform_action(GameAction::ActivateAbility {
        card_id: frog, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(frog).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Bitter Triumph destroys a creature after its discard additional cost.
#[test]
fn bitter_triumph_destroys_with_discard_cost() {
    let mut g = two_player_game();
    let bt = g.add_card_to_hand(0, catalog::bitter_triumph());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bt, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "target destroyed");
}

/// Cavern Stomper's activated ability makes it unblockable by power-2 creatures.
#[test]
fn cavern_stomper_grants_evasion() {
    let mut g = two_player_game();
    let stomper = g.add_card_to_battlefield(0, catalog::cavern_stomper());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stomper, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(stomper).unwrap().keywords.contains(&Keyword::CantBeBlockedByPowerAtMost(2)),
    );
}
