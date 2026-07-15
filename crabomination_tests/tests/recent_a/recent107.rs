//! Functionality tests for `catalog::sets::decks::recent107` — MH2 Food /
//! artifact-combo batch.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

/// Cranial Ram enters as living weapon (attached to a Germ) and scales with
/// your artifact count.
#[test]
fn cranial_ram_living_weapon_scales() {
    let mut g = two_player_game();
    let ram = g.move_card_to_battlefield_for_test(0, catalog::cranial_ram());
    drain_stack(&mut g);
    let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ")
        .expect("germ token").id;
    assert_eq!(g.battlefield_find(ram).unwrap().attached_to, Some(germ));
    // One artifact (the Ram): germ is 0/0 +1/+1 → 1/1.
    let cp = g.computed_permanent(germ).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let cp = g.computed_permanent(germ).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "+X grows with artifacts");
}

/// The Underworld Cookbook turns discards into Food and sacs for a raise.
#[test]
fn underworld_cookbook_cooks() {
    let mut g = two_player_game();
    let book = g.add_card_to_battlefield(0, catalog::the_underworld_cookbook());
    g.add_card_to_hand(0, catalog::island());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: book, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("discard for Food");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"));
    assert_eq!(g.players[0].hand.len(), 0, "discarded the card");
}

/// Asmor is castable for {B/R} only after a discard, and fetches the Cookbook.
#[test]
fn asmor_alt_cast_after_discard() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let asmor = g.add_card_to_hand(0, catalog::asmoranomardicadaistinaculdacar());
    let book = g.add_card_to_library(0, catalog::the_underworld_cookbook());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    // No discard yet — the alternative cast is rejected.
    assert!(g.perform_action(GameAction::CastSpellAlternative {
        card_id: asmor, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no discard yet");
    // Discard a card, then it casts and fetches the Cookbook.
    let junk = g.add_card_to_hand(0, catalog::island());
    let mut evs = Vec::new();
    g.discard_card(0, junk, &mut evs);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(book))]));
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: asmor, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("alt cast after a discard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(asmor).is_some(), "Asmor resolved");
    assert!(g.players[0].hand.iter().any(|c| c.id == book), "Cookbook fetched");
}

/// Retract bounces all your artifacts (and only yours).
#[test]
fn retract_bounces_your_artifacts() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::sol_ring());
    let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
    let re = g.add_card_to_hand(0, catalog::retract());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: re, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("retract");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "yours bounced");
    assert!(g.battlefield_find(theirs).is_some(), "theirs stays");
}

/// Jeskai Ascendancy pumps + untaps your team on a noncreature cast.
#[test]
fn jeskai_ascendancy_pumps_and_untaps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jeskai_ascendancy());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("noncreature cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 until end of turn");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Fatestitcher unearths for {U} and taps a permanent.
#[test]
fn fatestitcher_unearths_and_taps() {
    let mut g = two_player_game();
    let stitcher = g.add_card_to_graveyard(0, catalog::fatestitcher());
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    let unearth_idx = catalog::fatestitcher().activated_abilities.len() - 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stitcher, ability_index: unearth_idx, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("unearth");
    drain_stack(&mut g);
    let c = g.computed_permanent(stitcher).expect("unearthed");
    assert!(c.keywords.contains(&Keyword::Haste));
    // {T}: tap another permanent (mode 0).
    g.perform_action(GameAction::ActivateAbility {
        card_id: stitcher, ability_index: 0, target: Some(Target::Permanent(land)),
        additional_targets: vec![], x_value: None,
    }).expect("tap the land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Urza mints a Karnstruct and taps artifacts for {U}.
#[test]
fn urza_construct_and_artifact_mana() {
    let mut g = two_player_game();
    let urza = g.move_card_to_battlefield_for_test(0, catalog::urza_lord_high_artificer());
    drain_stack(&mut g);
    let construct = g.battlefield.iter().find(|c| c.definition.name == "Construct")
        .expect("Karnstruct").id;
    // Urza + Construct = 2 artifacts... Urza isn't an artifact; Construct is.
    let cp = g.computed_permanent(construct).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "+1/+1 for itself");
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: urza, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap the Construct for {U}");
    assert!(g.battlefield_find(construct).unwrap().tapped, "artifact tapped for the cost");
    assert!(g.players[0].mana_pool.total() >= 1, "added blue mana");
}

/// Tezzeret animates an artifact into a 5/5.
#[test]
fn tezzeret_animates_an_artifact() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_agent_of_bolas());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tez, ability_index: 1, target: Some(Target::Permanent(ring)), x_value: None,
    }).expect("-1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ring).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "animated");
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Second Sunrise rebuilds only what hit the graveyard from the battlefield
/// this turn.
#[test]
fn second_sunrise_rebuilds_this_turns_losses() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let old = g.add_card_to_graveyard(0, catalog::sol_ring()); // not from bf this turn
    g.battlefield_find_mut(bear).unwrap().damage = 9;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    let sunrise = g.add_card_to_hand(0, catalog::second_sunrise());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sunrise, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("second sunrise");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the dead bear returns");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old),
        "a card that didn't come from the battlefield stays");
}
