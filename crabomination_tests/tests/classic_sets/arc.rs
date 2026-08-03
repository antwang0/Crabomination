//! Archenemy (CR 314 / 900 / 904) — the scheme deck, setting schemes in
//! motion, the 904.10 abandon SBA and ongoing schemes (`catalog::sets::arc`).

use crabomination::card::{CardType, Supertype};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

/// A two-player Archenemy game with seat 0 as the archenemy.
fn archenemy_game(schemes: Vec<crabomination::card::CardDefinition>) -> GameState {
    let mut g = two_player_game();
    g.apply_format(crabomination::format::Format::Archenemy);
    g.seat_archenemy(0, schemes);
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g
}

/// Walk from upkeep into the precombat main phase, resolving what triggers.
fn to_main(g: &mut GameState) {
    while g.step != TurnStep::PreCombatMain {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(g);
}

/// CR 904.5 / 904.6 — the archenemy starts at 40 and takes the first turn.
#[test]
fn cr_904_5_archenemy_starts_at_forty_and_goes_first() {
    let g = archenemy_game(vec![catalog::i_delight_in_your_convulsions()]);
    assert_eq!(g.players[0].life, 40);
    assert_eq!(g.players[1].life, 20);
    assert_eq!(g.active_player_idx, 0);
}

/// CR 904.9 — the top scheme is set in motion as the precombat main begins,
/// and its "when you set this scheme in motion" trigger resolves.
#[test]
fn cr_904_9_precombat_main_sets_the_top_scheme_in_motion() {
    let mut g = archenemy_game(vec![catalog::i_delight_in_your_convulsions()]);
    assert_eq!(g.players[0].scheme_deck.len(), 1);
    to_main(&mut g);
    assert_eq!(g.players[1].life, 17, "drained for 3");
    assert_eq!(g.players[0].life, 43);
}

/// CR 904.10 — a non-ongoing scheme is abandoned to the bottom of the scheme
/// deck once its triggers have left the stack.
#[test]
fn cr_904_10_a_finished_scheme_is_abandoned() {
    let mut g = archenemy_game(vec![
        catalog::i_delight_in_your_convulsions(),
        catalog::delight_in_the_hunt(),
    ]);
    to_main(&mut g);
    let _ = g.check_state_based_actions();
    assert!(g.face_up_schemes(0).is_empty(), "swept off the command zone");
    assert_eq!(g.players[0].scheme_deck.len(), 2, "back on the bottom");
    assert_eq!(
        g.players[0].scheme_deck.last().unwrap().definition.name,
        "I Delight in Your Convulsions",
        "bottom of the deck, so the next scheme is a fresh one"
    );
}

/// CR 314.2 / 904.11 — an ongoing scheme stays face up in the command zone.
#[test]
fn cr_904_11_an_ongoing_scheme_stays_face_up() {
    let mut g = archenemy_game(vec![catalog::fear_my_authority()]);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    to_main(&mut g);
    let _ = g.check_state_based_actions();
    assert_eq!(g.face_up_schemes(0).len(), 1);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 from the command zone");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Fear));
}

/// CR 904.8 — an ongoing scheme's static functions from the command zone.
#[test]
fn cr_904_8_ongoing_scheme_static_locks_opponents_to_one_spell() {
    let mut g = archenemy_game(vec![catalog::i_bask_in_your_silent_awe()]);
    to_main(&mut g);
    let _ = g.check_state_based_actions();
    assert_eq!(g.face_up_schemes(0).len(), 1);
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[1].mana_pool.add(c, 20);
    }
    let first = g.add_card_to_hand(1, catalog::lightning_bolt());
    let second = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: first,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("first spell is fine");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: second,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the second is locked out"
    );
}

/// CR 701.33 — an ongoing scheme abandons itself when its condition fires.
#[test]
fn cr_701_33_fear_my_authority_abandons_when_the_life_isnt_paid() {
    let mut g = archenemy_game(vec![catalog::fear_my_authority()]);
    to_main(&mut g);
    let _ = g.check_state_based_actions();
    assert_eq!(g.face_up_schemes(0).len(), 1);
    // Broke: the 3-life payment is impossible, so the else_ branch abandons.
    g.players[0].life = 2;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.face_up_schemes(0).is_empty(), "abandoned");
    assert_eq!(g.players[0].scheme_deck.len(), 1);
}

/// CR 314.2 — schemes are never permanents and never castable.
#[test]
fn cr_314_2_schemes_are_not_permanents() {
    let def = catalog::i_delight_in_your_convulsions();
    assert_eq!(def.card_types, vec![CardType::Scheme]);
    assert!(!def.is_permanent());
    assert!(catalog::fear_my_authority().supertypes.contains(&Supertype::Ongoing));
    assert!(catalog::fear_my_authority().is_ongoing_scheme());
    assert!(!catalog::i_delight_in_your_convulsions().is_ongoing_scheme());
}

/// Evil Comes to Fruition scales off ten lands.
#[test]
fn evil_comes_to_fruition_scales_with_lands() {
    let mut g = archenemy_game(vec![catalog::evil_comes_to_fruition()]);
    to_main(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Plant").count(), 7);

    let mut g = archenemy_game(vec![catalog::evil_comes_to_fruition()]);
    for _ in 0..10 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    to_main(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elemental").count(), 7);
}

/// Delight in the Hunt mints a Horror and fogs your board.
#[test]
fn delight_in_the_hunt_mints_a_horror_and_fogs() {
    let mut g = archenemy_game(vec![catalog::delight_in_the_hunt()]);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    to_main(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Horror"));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "prevented");
}

/// All in Good Time hands the archenemy an extra turn.
#[test]
fn all_in_good_time_grants_an_extra_turn() {
    let mut g = archenemy_game(vec![catalog::all_in_good_time()]);
    to_main(&mut g);
    assert!(g.players[0].extra_turns > 0);
}

/// Kneel Before My Legions' second mode pumps the team.
#[test]
fn kneel_before_my_legions_pumps_on_mode_one() {
    let mut g = archenemy_game(vec![catalog::kneel_before_my_legions()]);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Mode(1),
    ]));
    to_main(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Embrace My Diabolical Vision refills both sides.
#[test]
fn embrace_my_diabolical_vision_refills_hands() {
    let mut g = archenemy_game(vec![catalog::embrace_my_diabolical_vision()]);
    for seat in 0..2 {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
        g.players[seat].hand.clear();
    }
    to_main(&mut g);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 4);
}
