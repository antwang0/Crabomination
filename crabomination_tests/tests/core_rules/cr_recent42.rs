//! CR conformance for the Darksteel-completion pass:
//! - CR 609.4b — "spend mana as though it were mana of any color" changes only
//!   how a cost may be paid, not the cost itself.
//! - CR 616.1c/616.1g — the enters-as-a-copy replacement outranks the
//!   enters-tapped one, so tappedness is read off the copied characteristics.
//! - CR 704.7 — a single loss replacement covers every simultaneous loss SBA.

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 609.4b — the printed cost is untouched; only the payment relaxes.
#[test]
fn cr_609_4b_any_color_spend_leaves_the_printed_cost_alone() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mycosynth_lattice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let printed = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().definition.cost.clone();
    assert_eq!(printed.summary(), catalog::lightning_bolt().cost.summary(), "cost unchanged");
    // A single green mana pays {R}.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("green pays the red pip");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the green mana was actually spent");
}

/// CR 609.4b — with no such permission a green mana can't pay {R}.
#[test]
fn cr_609_4b_without_the_permission_colors_still_matter() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Green, 1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .is_err(),
        "{{R}} needs red",
    );
}

/// CR 616.1c/616.1g — Clone of an enters-tapped body enters tapped.
#[test]
fn cr_616_1c_copy_replacement_precedes_enters_tapped() {
    let mut g = main_phase();
    let sentinel = g.add_card_to_battlefield(1, catalog::rusted_sentinel());
    // Untap the pre-placed body so we're reading the *copy's* tapped state.
    g.battlefield_find_mut(sentinel).unwrap().tapped = false;
    let clone = g.add_card_to_hand(0, catalog::clone_card());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: clone, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Clone");
    drain_stack(&mut g);
    let cp = g.computed_permanent(clone).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "it copied the Sentinel");
    assert!(g.battlefield_find(clone).unwrap().tapped, "the copied ability taps it");
}

/// CR 616.1c — copying a body with no such ability still enters untapped.
#[test]
fn cr_616_1c_copy_of_an_untapped_body_enters_untapped() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let clone = g.add_card_to_hand(0, catalog::clone_card());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: clone, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Clone");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(clone).unwrap().tapped);
}

/// CR 704.7 — 0 life *and* 10 poison at once is one loss, so one replacement.
#[test]
fn cr_704_7_one_replacement_covers_simultaneous_loss_sbas() {
    let mut g = main_phase();
    let mirror = g.add_card_to_battlefield(0, catalog::lichs_mirror());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].hand.clear();
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].life = 0;
    g.players[0].poison_counters = 10;
    let _ = g.check_state_based_actions();
    assert!(!g.players[0].eliminated, "the Mirror replaced the loss");
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[0].poison_counters, 0);
    assert_eq!(g.players[0].hand.len(), 7, "a fresh seven");
    assert!(g.battlefield_find(mirror).is_none(), "the Mirror shuffled itself away");
    assert!(g.battlefield_find(bear).is_none(), "so did every permanent you own");
}

/// CR 704.7 — the replacement is consumed, so the next loss sticks.
#[test]
fn cr_704_7_loss_reset_is_used_up() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::lichs_mirror());
    g.players[0].hand.clear();
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].life = 0;
    let _ = g.check_state_based_actions();
    assert!(!g.players[0].eliminated);
    g.players[0].life = -1;
    let _ = g.check_state_based_actions();
    assert!(g.players[0].eliminated, "no Mirror left to replace the second loss");
}

/// CR 704.7 — the reset also covers a draw-from-empty loss (CR 704.5b).
#[test]
fn cr_704_7_loss_reset_covers_draw_from_empty() {
    let mut g = main_phase();
    g.players[0].library.clear();
    g.players[0].hand.clear();
    g.add_card_to_battlefield(0, catalog::lichs_mirror());
    for _ in 0..9 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.lose_to_empty_draw(0);
    assert!(!g.players[0].eliminated);
    assert_eq!(g.players[0].hand.len(), 7);
    // The Mirror was an artifact you owned, so it went to the library too.
    assert!(!g.battlefield.iter().any(|c| c.definition.card_types.contains(&CardType::Artifact)));
}
