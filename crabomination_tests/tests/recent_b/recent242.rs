//! Functionality tests for `catalog::sets::decks::recent242` (MKM Case
//! enchantments + the Case solve mechanic).

use crabomination::card::{
    CardDefinition, CardType, CounterType, CreatureType, Subtypes,
};
use crabomination::catalog;
use crabomination::game::types::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::{b, cost, g, r, u, w, ManaSymbol};

/// A vanilla 1/1 in one color, for board-state solve conditions.
fn mono(name: &'static str, pip: ManaSymbol) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[pip]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

fn solve_now(g: &mut crabomination::game::GameState) {
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(g);
}

fn is_solved(g: &crabomination::game::GameState, id: crabomination::card::CardId) -> bool {
    g.battlefield.iter().find(|c| c.id == id).map(|c| c.case_solved).unwrap_or(false)
}

// ── Case of the Crimson Pulse ────────────────────────────────────────────────

/// ETB discards a card, then draws two (net +1, one card binned).
#[test]
fn crimson_pulse_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::island());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    g.fire_self_etb_triggers(case, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3, "discard one, draw two");
    assert_eq!(g.players[0].graveyard.len(), 1, "one card binned");
}

/// Solves at the end step once the controller's hand is empty.
#[test]
fn crimson_pulse_solves_on_empty_hand() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    assert!(g.players[0].hand.is_empty());
    solve_now(&mut g);
    assert!(is_solved(&g, case), "empty hand solves the Case");
}

/// Does not solve while a card remains in hand.
#[test]
fn crimson_pulse_unsolved_with_cards_in_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest());
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "a card in hand blocks the solve");
}

/// Once solved, the upkeep ability wheels the whole hand into two fresh cards.
#[test]
fn crimson_pulse_solved_wheels_at_upkeep() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    solve_now(&mut g);
    assert!(is_solved(&g, case));
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "discard hand, draw two");
    assert!(
        g.players[0].hand.iter().all(|c| c.definition.name == "Island"),
        "new hand is the two drawn cards"
    );
}

// ── Case of the Trampled Garden ──────────────────────────────────────────────

/// ETB distributes two +1/+1 counters onto a creature you control.
#[test]
fn trampled_garden_etb_distributes_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, mono("Bear", g_pip()));
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_trampled_garden());
    g.fire_self_etb_triggers(case, 0);
    drain_stack(&mut g);
    let counters = g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 2, "two +1/+1 counters distributed");
}

/// Solves once your creatures' total power reaches eight.
#[test]
fn trampled_garden_solves_on_total_power() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_trampled_garden());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "total power 6 is not enough");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    solve_now(&mut g);
    assert!(is_solved(&g, case), "total power 8 solves the Case");
}

// ── Case of the Shattered Pact ───────────────────────────────────────────────

/// ETB fetches a basic land card into hand.
#[test]
fn shattered_pact_etb_fetches_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_shattered_pact());
    g.fire_self_etb_triggers(case, 0);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "basic land fetched to hand"
    );
}

/// Solves once there are five colors among permanents you control.
#[test]
fn shattered_pact_solves_on_five_colors() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_shattered_pact());
    for (n, pip) in [("W", w()), ("U", u()), ("B", b()), ("R", r())] {
        g.add_card_to_battlefield(0, mono(n, pip));
    }
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "four colors is not enough");
    g.add_card_to_battlefield(0, mono("G", g_pip()));
    solve_now(&mut g);
    assert!(is_solved(&g, case), "five colors solves the Case");
}

// ── Case of the Filched Falcon ───────────────────────────────────────────────

/// ETB investigates; solving needs three artifacts and switches on the {2}{U},
/// Sacrifice activated ability.
#[test]
fn filched_falcon_solves_on_three_artifacts_and_arms_ability() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_filched_falcon());
    g.fire_self_etb_triggers(case, 0);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count(),
        1,
        "ETB investigate made a Clue"
    );
    // The Clue is one artifact; add two more to reach three.
    g.add_card_to_battlefield(0, catalog::ornithopter());
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "two artifacts is not enough");
    g.add_card_to_battlefield(0, catalog::ornithopter());
    solve_now(&mut g);
    assert!(is_solved(&g, case), "three artifacts solves the Case");
    let armed = g.battlefield.iter().find(|c| c.id == case).unwrap()
        .definition.activated_abilities.len();
    assert_eq!(armed, 1, "solved Case gains its activated ability");
}

// ── Case of the Uneaten Feast ────────────────────────────────────────────────

/// Each creature you control entering gains 1 life; five such gains solves it.
#[test]
fn uneaten_feast_gains_life_and_solves() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_uneaten_feast());
    let start = g.players[0].life;
    for _ in 0..5 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered {
            card_id: bear,
        }]);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, start + 5, "gained 1 life per creature");
    solve_now(&mut g);
    assert!(is_solved(&g, case), "gaining 5 life this turn solves the Case");
}

// ── Case of the Locked Hothouse ──────────────────────────────────────────────

/// Solves on seven lands and arms the play-from-top statics.
#[test]
fn locked_hothouse_solves_on_seven_lands_and_arms_top_play() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_locked_hothouse());
    let armed_before =
        g.battlefield.iter().find(|c| c.id == case).unwrap().definition.static_abilities.len();
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "six lands is not enough");
    g.add_card_to_battlefield(0, catalog::forest());
    solve_now(&mut g);
    assert!(is_solved(&g, case), "seven lands solves the Case");
    let armed_after =
        g.battlefield.iter().find(|c| c.id == case).unwrap().definition.static_abilities.len();
    assert_eq!(armed_after, armed_before + 2, "solved Case gains its two top-play statics");
}

// ── Case of the Gateway Express ──────────────────────────────────────────────

/// ETB: each creature you control pings the chosen enemy creature. Solved: your
/// creatures get +1/+0.
#[test]
fn gateway_express_pings_and_anthems() {
    let mut g = two_player_game();
    // Two 2/2s ping the enemy 0/4 for 2 total on ETB.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_gateway_express());
    let effect = catalog::case_of_the_gateway_express().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        case,
        0,
        Some(crabomination::game::types::Target::Permanent(wall)),
    );
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield.iter().find(|c| c.id == wall).unwrap().damage, 2, "two pings");
    // Solve via three attackers this turn, then the anthem applies.
    g.players[0].creatures_attacked_this_turn = 3;
    solve_now(&mut g);
    assert!(is_solved(&g, case));
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 anthem while solved");
}

// ── Case File Auditor ────────────────────────────────────────────────────────

/// "Whenever you solve a Case" fires the Auditor's look-six.
#[test]
fn case_file_auditor_triggers_on_solve() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::case_file_auditor());
    let ench = g.add_card_to_library(0, catalog::case_of_the_shattered_pact());
    // Auditor looks at the top six; a Case (enchantment) is on top to reveal.
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    assert!(g.players[0].hand.is_empty());
    solve_now(&mut g);
    assert!(is_solved(&g, case));
    assert!(
        g.players[0].hand.iter().any(|c| c.id == ench),
        "solving a Case let the Auditor pull an enchantment to hand"
    );
}

/// A Case's solved designation clears when it leaves the battlefield.
#[test]
fn solved_case_resets_on_leave() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    solve_now(&mut g);
    assert!(is_solved(&g, case));
    let ctx = crabomination::game::effects::EffectContext::for_trigger(case, 0, None, 0);
    let mut evs = vec![];
    g.move_card_to(case, &crabomination::effect::ZoneDest::Graveyard, &ctx, &mut evs);
    drain_stack(&mut g);
    let in_gy = g.players[0].graveyard.iter().find(|c| c.id == case);
    assert!(in_gy.map(|c| !c.case_solved).unwrap_or(true), "solved flag cleared off-battlefield");
}

fn g_pip() -> ManaSymbol {
    g()
}
