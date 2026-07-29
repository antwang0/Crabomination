//! CR conformance for rules exercised by the Dissension gap batch:
//! CR 604.3 / 613.4 (a characteristic-defining dynamic P/T recomputes live as
//! its input changes — Dread Slag tracks hand size), CR 601.2f (cost increases
//! and reductions both apply — Grand Arbiter), and CR 704.5f (a creature with 0
//! or less toughness is put into its owner's graveyard as an SBA — Dread Slag).

use crabomination::catalog;
use crabomination::game::actions::{cost_reduction_for_spell, extra_cost_for_spell};
use crabomination::game::{two_player_game};

/// CR 604.3 / 613.4 — a CDA P/T recomputes live as its input changes: Dread Slag
/// (9/9, −4/−4 per card in your hand) is 9/9 with an empty hand and 5/5 with one
/// card, with no event needed beyond the hand-size change.
#[test]
fn cr_604_3_cda_recomputes_live() {
    let mut g = two_player_game();
    let slag = g.add_card_to_battlefield(0, catalog::dread_slag());
    g.players[0].hand.clear();
    let cp = g.computed_permanent(slag).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 9), "empty hand → 9/9");
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(slag).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "one card → −4/−4 → 5/5");
}

/// CR 601.2f — cost increases and reductions are applied together: with Grand
/// Arbiter out, your blue spell is discounted {1} while an opponent's identical
/// spell is taxed {1}. Reductions floor at the printed generic (can't go below).
#[test]
fn cr_601_2f_cost_increase_and_reduction_apply() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grand_arbiter_augustin_iv());
    let blue = crabomination::card::CardInstance::new(g.next_id(), catalog::counterspell(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &blue, None), 1, "your blue spell: {{1}} off");
    assert_eq!(extra_cost_for_spell(&g, 0, &blue, None), 0, "your spell isn't taxed");
    assert_eq!(extra_cost_for_spell(&g, 1, &blue, None), 1, "opponent's spell: {{1}} tax");
    assert_eq!(cost_reduction_for_spell(&g, 1, &blue, None), 0, "opponent gets no discount");
}

/// CR 704.5f — a creature whose toughness is 0 or less is put into its owner's
/// graveyard: Dread Slag (9/9, −4/−4 per card in hand) dies once its controller
/// holds three cards (9 − 12 = −3 toughness).
#[test]
fn cr_704_5f_zero_toughness_dies() {
    let mut g = two_player_game();
    let slag = g.add_card_to_battlefield(0, catalog::dread_slag());
    g.players[0].hand.clear();
    // Two cards → 1/1, alive.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let _ = g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == slag), "still alive at 1/1");
    // A third card → −3 toughness → dies as an SBA.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == slag), "0-or-less toughness → graveyard");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == slag));
}
