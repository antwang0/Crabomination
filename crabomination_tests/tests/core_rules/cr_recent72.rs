//! CR conformance for this run's engine work:
//! - CR 407 — the ante zone: the opening ante, ante-only deck legality,
//!   anteing off the library, ownership exchange, winner-takes-all.
//! - CR 615 — a blocker that prevents damage from what it's blocking.
//! - CR 616.1 — two prevention effects on one event only need one to apply.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
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

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 407 — the ante zone ─────────────────────────────────────────────────

/// CR 407.2 — each player antes one card from their deck before the game.
#[test]
fn cr_407_2_opening_ante_takes_one_card_from_each_deck() {
    let mut g = main_phase();
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g.begin_ante_game();
    assert!(g.playing_for_ante);
    for seat in 0..2 {
        assert_eq!(g.players[seat].ante.len(), 1);
        assert_eq!(g.players[seat].library.len(), 4);
    }
}

/// CR 407.3 — an ante-only card can't legally be in a non-ante deck.
#[test]
fn cr_407_3_ante_cards_are_illegal_outside_an_ante_game() {
    use crabomination::format::{DeckError, Format, validate_deck};
    let mut deck: Vec<_> = (0..59).map(|_| catalog::forest()).collect();
    deck.push(catalog::contract_from_below());
    let errors = validate_deck(&deck, Format::Legacy).expect_err("illegal");
    assert!(errors.iter().any(|e| matches!(
        e,
        DeckError::AnteCardOutsideAnteGame { card_name } if *card_name == "Contract from Below"
    )));
}

/// CR 407.4 — Demonic Attorney antes the top card of every library.
#[test]
fn cr_407_4_demonic_attorney_antes_from_every_library() {
    let mut g = main_phase();
    g.playing_for_ante = true;
    let mine = g.add_card_to_library(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_library(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::demonic_attorney());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].ante.iter().map(|c| c.id).collect::<Vec<_>>(), vec![mine]);
    assert_eq!(g.players[1].ante.iter().map(|c| c.id).collect::<Vec<_>>(), vec![theirs]);
}

/// Contract from Below empties the hand, antes one, and refills to seven.
#[test]
fn contract_from_below_wheels_into_seven() {
    let mut g = main_phase();
    g.playing_for_ante = true;
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::contract_from_below());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[0].ante.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 4, "3 discarded + the Contract");
}

/// Jeweled Bird antes itself, then reclaims everything else you own.
#[test]
fn jeweled_bird_trades_the_ante_for_a_card() {
    let mut g = main_phase();
    g.playing_for_ante = true;
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let bird = g.add_card_to_battlefield(0, catalog::jeweled_bird());
    g.clear_sickness(bird);
    // Two cards already in the ante, both owned by seat 0.
    for _ in 0..2 {
        g.ante_top_card_for_test(0);
    }
    g.add_card_to_library(0, catalog::mountain());
    activate(&mut g, 0, bird, None);
    assert_eq!(g.players[0].ante.len(), 1, "only the Bird remains");
    assert_eq!(g.players[0].ante[0].id, bird);
    assert_eq!(g.players[0].graveyard.len(), 2, "the other two came back as cards");
    assert_eq!(g.players[0].hand.len(), 1, "then draw a card");
}

/// CR 407.3 — Tempest Efreet permanently swaps ownership when the life goes
/// unpaid.
#[test]
fn cr_407_3_tempest_efreet_exchanges_ownership() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.playing_for_ante = true;
    let prize = g.add_card_to_hand(1, catalog::black_lotus());
    let efreet = g.add_card_to_battlefield(0, catalog::tempest_efreet());
    g.clear_sickness(efreet);
    // Seat 1 declines the 10 life.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    activate(&mut g, 0, efreet, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 20, "no life paid");
    let taken = g.players[0].hand.iter().find(|c| c.id == prize).expect("card changed hands");
    assert_eq!(taken.owner, 0, "ownership is permanent");
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Tempest Efreet"));
}

/// CR 407.2 — the winner ends up owning the whole ante zone.
#[test]
fn cr_407_2_winner_takes_the_ante() {
    let mut g = main_phase();
    g.playing_for_ante = true;
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(1, catalog::island());
    g.ante_top_card_for_test(0);
    g.ante_top_card_for_test(1);
    g.award_ante_to(0);
    assert_eq!(g.players[0].ante.len(), 2);
    assert!(g.players[1].ante.is_empty());
    assert!(g.players[0].ante.iter().all(|c| c.owner == 0));
}

// ── CR 615 / 616 — prevention ──────────────────────────────────────────────

/// CR 615 — a Wall of Vapor takes nothing from the creature it blocks, and
/// the attacker still takes the Wall's damage back.
#[test]
fn cr_615_blocker_prevents_damage_from_the_creature_it_blocks() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_shadows());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 0);
    assert_eq!(g.players[1].life, 20, "the attacker was blocked");
}

/// CR 616.1 — with both a blocked-creature shield and a fog in play, the
/// event is still fully prevented (applying either one suffices).
#[test]
fn cr_616_1_two_prevention_effects_still_prevent_the_event() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_vapor());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    let fog = g.add_card_to_hand(1, catalog::holy_day());
    cast(&mut g, 1, fog, None);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 0);
    assert!(g.battlefield_find(attacker).unwrap().damage == 0);
}
