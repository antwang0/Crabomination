//! CR conformance for this run:
//! - CR 702.27a — buyback replaces the resolution-time trip to the graveyard,
//!   and only that trip: a countered buyback spell still gets binned.
//! - CR 405.2 / 405.6c — the stack is last-in-first-out, and mana abilities
//!   never touch it.
//! - CR 613.11 — game-rule-modifying continuous effects (a maximum hand size)
//!   apply in timestamp order, so the newest cap wins rather than the smallest.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, seat: usize, id: crabomination::card::CardId, target: Option<Target>) {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
}

// ── CR 702.27 — Buyback ─────────────────────────────────────────────────────

/// A bought-back spell resolves and goes to hand; the same spell cast without
/// buyback goes to the graveyard.
#[test]
fn cr_702_27a_buyback_replaces_the_graveyard_with_the_hand() {
    for buyback in [false, true] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::whispers_of_the_muse());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let action = if buyback {
            GameAction::CastSpellBuyback {
                card_id: id,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: id,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        assert_eq!(
            g.players[0].hand.iter().any(|c| c.id == id),
            buyback,
            "buyback={buyback} decides hand vs graveyard"
        );
        assert_eq!(g.players[0].graveyard.iter().any(|c| c.id == id), !buyback);
    }
}

/// The buyback replacement is scoped to resolution, so countering the spell
/// still bins it (CR 702.27a — "instead of into that player's graveyard as it
/// resolves").
#[test]
fn cr_702_27a_countered_buyback_spell_still_goes_to_the_graveyard() {
    let mut g = two_player_game();
    let whispers = g.add_card_to_hand(0, catalog::whispers_of_the_muse());
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: whispers,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback cast");

    g.players[1].mana_pool.add(Color::Blue, 2);
    cast(&mut g, 1, counter, Some(Target::Permanent(whispers)));
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == whispers), "countered, so binned");
    assert!(!g.players[0].hand.iter().any(|c| c.id == whispers));
}

// ── CR 405 — Stack ──────────────────────────────────────────────────────────

/// CR 405.2 — the stack resolves last-in-first-out: the second Bolt cast is
/// the first to deal damage, so it decides which creature dies.
#[test]
fn cr_405_2_stack_resolves_last_in_first_out() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let first = g.add_card_to_hand(0, catalog::lightning_bolt());
    let second = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, 0, first, Some(Target::Player(1)));
    cast(&mut g, 0, second, Some(Target::Permanent(bear)));

    // Shock went on last, so it is on top.
    assert_eq!(g.stack.len(), 2);
    let _ = g.resolve_top_of_stack();
    assert!(g.battlefield_find(bear).is_none(), "Shock resolved first and killed the 2/2");
    assert_eq!(g.players[1].life, 20, "the Bolt is still waiting");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// CR 405.6c — a mana ability resolves immediately and never uses the stack.
#[test]
fn cr_405_6c_mana_abilities_dont_use_the_stack() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    assert!(g.stack.is_empty(), "no stack object");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "the mana is already there");
}

// ── CR 613.11 — game-rule effects apply in timestamp order ──────────────────

/// Two "your maximum hand size is N" effects: the one that entered later sets
/// the cap, even when it is the *larger* number (a smallest-wins fold would
/// answer 2 here).
#[test]
fn cr_613_11_the_newest_max_hand_size_wins() {
    let mut g = two_player_game();
    let tight = g.add_card_to_battlefield(0, catalog::recycle()); // sets 2
    assert_eq!(g.effective_max_hand_size(0), Some(2));

    let loose = g.add_card_to_battlefield(0, catalog::necrodominance()); // sets 5, later
    assert_eq!(g.effective_max_hand_size(0), Some(5), "the later timestamp wins");

    // Remove the newer one and the older cap is back in charge.
    let mut events = vec![];
    g.destroy_permanent(loose, false, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.effective_max_hand_size(0), Some(2));
    assert!(g.battlefield_find(tight).is_some());
}
