//! Unfinity (UNF) — Attractions (CR 717) and the cards that open them.

use crabomination::card::{CardInstance, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Seat an Attraction deck for `p`, top card first.
fn stock_attraction_deck(g: &mut GameState, p: usize, defs: Vec<crabomination::card::CardDefinition>) {
    for def in defs {
        let id = g.next_id();
        g.players[p].attraction_deck.push(CardInstance::new(id, def, p));
    }
}

/// Advance to seat 0's next precombat main, where the roll-to-visit turn-based
/// action happens (CR 703.4g).
fn advance_to_your_main(g: &mut GameState) {
    for p in 0..g.players.len() {
        for i in 0..40 {
            let id = CardId(9000 + (p * 100 + i) as u32);
            g.players[p].library.push(CardInstance::new(id, catalog::forest(), p));
        }
    }
    // Leave the current phase first — the game starts on seat 0's precombat
    // main, which would otherwise match immediately.
    while g.step == TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    while !(g.step == TurnStep::PreCombatMain && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// Every UNF factory is registered under its printed name.
#[test]
fn unf_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::balloon_stand as fn() -> crabomination::card::CardDefinition,
        catalog::bounce_chamber,
        catalog::bumper_cars,
        catalog::clown_extruder,
        catalog::concession_stand,
        catalog::foam_weapons_kiosk,
        catalog::fortune_teller,
        catalog::information_booth,
        catalog::kiddie_coaster,
        catalog::roller_coaster,
        catalog::merry_go_round,
        catalog::spinny_ride,
        catalog::trash_bin,
        catalog::swinging_ship,
        catalog::lifetime_pass_holder,
        catalog::deadbeat_attendant,
        catalog::petting_zookeeper,
        catalog::seasoned_buttoneer,
        catalog::rad_rascal,
        catalog::quick_fixer,
        catalog::coming_attraction,
        catalog::the_most_dangerous_gamer,
        catalog::complaints_clerk,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// CR 701.51b — opening an Attraction moves the top of the Attraction deck
/// onto the battlefield.
#[test]
fn cr_701_51b_open_an_attraction_puts_the_top_card_onto_the_battlefield() {
    let mut g = two_player_game();
    stock_attraction_deck(&mut g, 0, vec![catalog::information_booth(), catalog::fortune_teller()]);
    g.move_card_to_battlefield_for_test(0, catalog::deadbeat_attendant());
    drain_stack(&mut g);
    assert_eq!(g.players[0].attraction_deck.len(), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Information Booth"));
}

/// An empty Attraction deck makes "open an Attraction" a no-op.
#[test]
fn open_an_attraction_is_a_noop_with_an_empty_deck() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::deadbeat_attendant());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Deadbeat Attendant"));
}

/// CR 701.52a / 717.5 — the precombat-main roll visits every Attraction whose
/// lit-up numbers match, and only those.
#[test]
fn cr_701_52a_roll_to_visit_fires_only_the_matching_attractions() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::information_booth()); // lights 2, 6
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::roller_coaster()); // lights 2, 6
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_n(DecisionAnswer::DieRoll(6), 4)));
    let hand = g.players[0].hand.len();
    advance_to_your_main(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew for the turn and for the visit");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "Roller Coaster also visited");
}

/// A roll that matches no lit-up number visits nothing.
#[test]
fn roll_to_visit_misses_when_no_light_matches() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::information_booth()); // lights 2, 6
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(4)]));
    let hand = g.players[0].hand.len();
    advance_to_your_main(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "only the turn's draw");
}

/// Only the active player rolls (CR 703.4g) — an opponent's Attractions sit
/// idle while it isn't their turn.
#[test]
fn only_the_active_player_visits_their_attractions() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::information_booth());
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_n(
        DecisionAnswer::DieRoll(6),
        4,
    )));
    for p in 0..g.players.len() {
        for i in 0..20 {
            let id = CardId(8000 + (p * 100 + i) as u32);
            g.players[p].library.push(CardInstance::new(id, catalog::forest(), p));
        }
    }
    let hand = g.players[1].hand.len();
    // Walk into seat 1's precombat main — seat 0's has already passed.
    while g.step == TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);
    assert_eq!(g.active_player_idx, 1, "it is now seat 1's turn");
    assert_eq!(g.players[1].hand.len(), hand + 2, "their draw step plus the visit");
}

/// CR 717.6 — an Attraction headed for the graveyard lands in its owner's
/// junkyard instead.
#[test]
fn cr_717_6_a_destroyed_attraction_goes_to_the_junkyard() {
    let mut g = two_player_game();
    let booth = g.add_card_to_battlefield(0, catalog::information_booth());
    let naturalize = g.add_card_to_hand(0, catalog::naturalize());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: naturalize,
        target: Some(crabomination::game::types::Target::Permanent(booth)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(booth).is_none());
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == booth), "not the graveyard");
    assert!(g.players[0].attraction_junkyard.iter().any(|c| c.id == booth));
}

/// The Most Dangerous Gamer opens on entry and grows for it.
#[test]
fn the_most_dangerous_gamer_opens_and_grows() {
    let mut g = two_player_game();
    stock_attraction_deck(&mut g, 0, vec![catalog::clown_extruder()]);
    let gamer = g.move_card_to_battlefield_for_test(0, catalog::the_most_dangerous_gamer());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clown Extruder"));
    assert_eq!(
        g.battlefield_find(gamer).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1)
    );
}

/// Kiddie Coaster's visit pumps the team.
#[test]
fn kiddie_coaster_visit_pumps_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kiddie_coaster()); // lights 2, 3, 6
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3)]));
    advance_to_your_main(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// The Attraction deck and junkyard reach the client view (CR 717.2/717.6a).
#[test]
fn attraction_zones_are_surfaced_to_the_client() {
    let mut g = two_player_game();
    g.seat_attraction_deck(0, vec![catalog::information_booth(), catalog::fortune_teller()]);
    let booth = g.add_card_to_battlefield(0, catalog::clown_extruder());
    let naturalize = g.add_card_to_hand(0, catalog::naturalize());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: naturalize,
        target: Some(crabomination::game::types::Target::Permanent(booth)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let view = crabomination::server::view::project(&g, 0);
    let me = view.players.iter().find(|p| p.seat == 0).expect("seat 0");
    assert_eq!(me.attraction_deck_size, 2);
    assert_eq!(me.attraction_junkyard, vec!["Clown Extruder".to_string()]);
}
