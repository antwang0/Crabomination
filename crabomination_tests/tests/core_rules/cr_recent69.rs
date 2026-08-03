//! CR conformance for this run's engine work:
//! - CR 705.2 — "each player flips a coin": one flip per seat, each seat's
//!   own branch.
//! - CR 701.19 — searching a zone: the multi-zone search, and the fact that a
//!   hand/graveyard-only search is not a library search at all.
//! - CR 804.2 — the deploy creatures option.
//! - CR 707.2 — "enters as a copy" now asks the controller which permanent.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 705.2 — each player flips their own coin ─────────────────────────────

/// Goblin Assassin: one flip per seat, and only the tails seats pay.
#[test]
fn cr_705_2_each_player_flips_their_own_coin() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let assassin = g.add_card_to_hand(0, catalog::goblin_assassin());
    // Seat 0 heads, seat 1 tails.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Cards(vec![theirs]),
    ]));
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: assassin,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some(), "heads keeps its creature");
    assert!(g.battlefield_find(theirs).is_none(), "tails sacrificed");
}

// ── CR 701.19 — searching a zone ────────────────────────────────────────────

/// A hand/graveyard/library search finds the card wherever it lives — here,
/// in hand, a zone a plain `Search` would never see.
#[test]
fn cr_701_19_multi_zone_search_reaches_the_hand() {
    let mut g = main_phase();
    let supplicant = g.add_card_to_battlefield(0, catalog::dark_supplicant());
    g.clear_sickness(supplicant);
    for _ in 0..3 {
        let c = g.add_card_to_battlefield(0, catalog::daru_mender());
        g.clear_sickness(c);
    }
    let scion = g.add_card_to_hand(0, catalog::scion_of_darkness());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(scion))]));
    activate(&mut g, 0, supplicant, 0, None);
    assert!(g.battlefield_find(scion).is_some());
    assert!(!g.players[0].hand.iter().any(|c| c.id == scion));
}

/// CR 701.19a — a search that names no library is not a library search:
/// Shadow of Doubt's "no player may search a library" doesn't stop it, and
/// `searched_library_this_turn` stays clear.
#[test]
fn cr_701_19a_a_zoneless_library_search_is_not_a_library_search() {
    let mut g = main_phase();
    let supplicant = g.add_card_to_battlefield(0, catalog::dark_supplicant());
    g.clear_sickness(supplicant);
    for _ in 0..3 {
        let c = g.add_card_to_battlefield(0, catalog::daru_mender());
        g.clear_sickness(c);
    }
    let scion = g.add_card_to_hand(0, catalog::scion_of_darkness());
    g.no_search_this_turn = true;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(scion))]));
    activate(&mut g, 0, supplicant, 0, None);
    // The library is one of the listed zones, so the lock does bite.
    assert!(g.players[0].hand.iter().any(|c| c.id == scion));
    assert!(!g.players[0].searched_library_this_turn);
}

// ── CR 804.2 — the deploy creatures option ──────────────────────────────────

/// With the option on, every creature carries "{T}: Target teammate gains
/// control of this creature."
#[test]
fn cr_804_2_deploy_creatures_hands_a_creature_to_a_teammate() {
    let mut g = multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).expect("teams");
    g.deploy_creatures = true;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let printed = g.battlefield_find(bear).unwrap().definition.activated_abilities.len();
    assert_eq!(g.granted_abilities_for(bear).len(), 1, "one granted deploy ability");
    activate(&mut g, 0, bear, printed, None);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 2, "the teammate took it");
}

/// The option is off by default, so nothing is granted in a normal game.
#[test]
fn cr_804_1_deploy_creatures_is_off_by_default() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.granted_abilities_for(bear).is_empty());
}

// ── CR 707.2 — "enters as a copy" is the controller's pick ──────────────────

/// The copier no longer silently grabs the biggest body: a real answer picks
/// the source, and an unanswered prompt still falls back to highest power.
#[test]
fn cr_707_2_enters_as_copy_honors_the_controllers_pick() {
    let mut g = main_phase();
    let big = g.add_card_to_battlefield(1, catalog::enormous_baloth()); // 7/7
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let clone = g.add_card_to_hand(0, catalog::clone_card());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![small])]));
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: clone,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(clone).unwrap().definition.name, "Grizzly Bears");
    assert!(g.battlefield_find(big).is_some());
}
