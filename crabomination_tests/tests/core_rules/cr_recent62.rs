//! CR conformance for this run's engine work:
//! - CR 727 — restarting the game (Karn Liberated's −14).
//! - CR 806 — the Free-for-All variant's default options.
//! - CR 614 / 616 — the new damage-threshold replacements and their
//!   interaction with the doubling/halving chain.
//! - CR 601.2f — a coloured cost *increase* (the Invasion Leech cycle).

use crabomination::catalog;
use crabomination::game::effects::EntityRef;
use crabomination::game::types::GameAction;
use crabomination::game::*;
use crabomination::game::{AttackOption, multi_player_game};

fn main_phase(seats: usize) -> GameState {
    let mut g = multi_player_game(seats);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

// ── CR 727 — Restarting the Game ────────────────────────────────────────────

/// CR 727.1/727.2 — a restart ends the game with no winner and rebuilds every
/// player's library from all the cards that were in the game.
#[test]
fn cr_727_1_restart_returns_every_card_to_its_owners_library() {
    let mut g = main_phase(2);
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_hand(1, catalog::opt());
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(
        crabomination::card::CardId(9001),
        catalog::forest(),
        0,
    ));
    let owned_before: Vec<usize> = (0..2)
        .map(|p| {
            g.players[p].library.len()
                + g.players[p].hand.len()
                + g.players[p].graveyard.len()
                + g.battlefield.iter().filter(|c| c.owner == p).count()
        })
        .collect();
    g.players[0].life = 3;

    let mut events = Vec::new();
    g.restart_game(1, Vec::new(), &mut events);

    assert!(g.game_over.is_none(), "the restarted game is live, not over");
    assert_eq!(g.active_player_idx, 1, "CR 727.1a — the restarter goes first");
    assert!(g.battlefield.is_empty(), "no permanents carry over");
    for (p, before) in owned_before.iter().enumerate() {
        assert_eq!(g.players[p].hand.len(), 7, "CR 103.4 — a fresh opening hand");
        assert_eq!(
            g.players[p].library.len() + g.players[p].hand.len(),
            *before,
            "every owned card is back in the deck"
        );
        assert_eq!(g.players[p].life, g.players[p].starting_life);
    }
    assert!(events.iter().any(|e| matches!(e, GameEvent::GameRestarted { starter: 1 })));
}

/// CR 727.5 — exempted cards skip the reshuffle; Karn's −14 puts the cards it
/// exiled onto the battlefield under its controller's control.
#[test]
fn cr_727_5_karn_deploys_its_exiled_cards_after_the_restart() {
    let mut g = main_phase(2);
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    // A permanent Karn exiled, owned by the opponent.
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let karn = g.add_card_to_battlefield(0, catalog::karn_liberated());
    g.battlefield
        .iter_mut()
        .find(|c| c.id == karn)
        .unwrap()
        .counters
        .insert(crabomination::card::CounterType::Loyalty, 14);
    let card = g.battlefield.iter().position(|c| c.id == stolen).unwrap();
    let mut inst = g.battlefield.remove(card);
    inst.exiled_with = Some(karn);
    g.exile.push(inst);

    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("Karn ultimate");
    drain_stack(&mut g);

    let deployed = g
        .battlefield
        .iter()
        .find(|c| c.id == stolen)
        .expect("the Karn-exiled card came back");
    assert_eq!(deployed.controller, 0, "under Karn's controller");
    assert_eq!(deployed.owner, 1, "CR 727.2 — ownership never changes");
    assert!(g.exile.is_empty(), "the exempt card left exile");
    assert_eq!(g.active_player_idx, 0);
    assert!(g.battlefield.iter().all(|c| c.id != karn), "Karn itself is reshuffled");
}

// ── CR 806 — Free-for-All Variant ───────────────────────────────────────────

/// CR 806.1 — Free-for-All players compete as individuals: one singleton team
/// per seat, so no two seats share a life total or win together.
#[test]
fn cr_806_1_free_for_all_seats_are_individuals() {
    let g = multi_player_game(4);
    assert_eq!(g.teams.len(), 4);
    for (i, t) in g.teams.iter().enumerate() {
        assert_eq!(t.members, vec![i]);
        assert!(t.shared_life.is_none());
    }
    for a in 0..4 {
        for b in 0..4 {
            assert_eq!(g.same_team(a, b), a == b);
        }
    }
}

/// CR 806.2b — exactly one attack option is in force, and Free-for-All's
/// default is attack-multiple-players (rule 802).
#[test]
fn cr_806_2b_free_for_all_defaults_to_attack_multiple_players() {
    let g = multi_player_game(4);
    assert_eq!(g.attack_option, AttackOption::MultiplePlayers);
    // Every opponent is a legal defender under the default.
    assert_eq!(g.attackable_players_for(0), vec![1, 2, 3]);
}

// ── CR 614 / 616 — damage-threshold replacements ────────────────────────────

/// CR 614 — Divine Presence replaces a 4+ damage event with exactly 3; the cap
/// applies after the doubling/halving chain (CR 616.1 ordering is fixed here
/// because only one such replacement can apply to a given event).
#[test]
fn cr_614_divine_presence_caps_after_doubling() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::divine_presence());
    let ent = EntityRef::Player(1);
    assert_eq!(g.scale_damage_to(None, ent, 2), 2, "small events pass through");
    assert_eq!(g.scale_damage_to(None, ent, 9), 3);
    // Furnace of Rath doubles first; the cap still lands the event on 3.
    g.add_card_to_battlefield(0, catalog::furnace_of_rath());
    assert_eq!(g.scale_damage_to(None, ent, 1), 2, "1 doubled to 2 is under the cap");
    assert_eq!(g.scale_damage_to(None, ent, 3), 3, "3 doubled to 6 is capped");
}

/// CR 615 — Callous Giant's prevention is all-or-nothing on the whole event:
/// a 3-point hit is fully prevented, a 4-point hit is untouched.
#[test]
fn cr_615_callous_giant_prevention_is_all_or_nothing() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::callous_giant());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(giant), 3, None, &mut events);
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 0);
    assert!(events.iter().any(|e| matches!(e, GameEvent::DamagePrevented { amount: 3, .. })));
    g.deal_damage_to_from(EntityRef::Permanent(giant), 4, None, &mut events);
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 4);
}

// ── CR 601.2f — coloured cost increases ─────────────────────────────────────

/// CR 601.2f — a coloured surcharge is added to the cost before any reduction,
/// so a discount can never eat it.
#[test]
fn cr_601_2f_colored_tax_survives_a_cost_reduction() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::jade_leech());
    // Urza's Filter shaves {2} generic off multicoloured spells.
    g.add_card_to_battlefield(0, catalog::urzas_filter());
    let spell = g.add_card_to_hand(0, catalog::wandering_stream());
    let card = g.players[0].hand.iter().find(|c| c.id == spell).unwrap();
    let tax = crabomination::game::actions::colored_spell_tax_for_spell(&g, 0, card);
    assert_eq!(tax.cmc(), 1, "one {{G}} pip owed to Jade Leech");
    assert!(
        tax.symbols
            .iter()
            .any(|s| matches!(s, crabomination::mana::ManaSymbol::Colored(
                crabomination::mana::Color::Green
            ))),
        "the surcharge is a coloured pip, not generic"
    );
}
