//! Comprehensive-Rules conformance tests for sections exercised by the
//! recent267–269 batches: Kicker provenance (CR 702.32), Affinity cost
//! reduction (CR 702.9), and Casualty copy-on-cast (CR 702.153).

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// CR 702.32 — a kicked spell's `SpellWasKicked` rider fires; the unkicked
/// cast leaves it dormant. Aggressive Sabotage burns for 3 only when kicked.
#[test]
fn cr_702_32_kicker_provenance() {
    // Unkicked: just the discard, no burn.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::island());
    let s = g.add_card_to_hand(0, catalog::aggressive_sabotage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast unkicked");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "unkicked: no burn");

    // Kicked: pay the extra {R}, burn resolves.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::island());
    let s = g.add_card_to_hand(0, catalog::aggressive_sabotage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: s,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "kicked: 3 damage");
}

/// CR 702.9 — Affinity reduces a spell's generic cost by the number of
/// matching permanents. Argivian Phalanx ({5}{W}) drops {1} per creature.
#[test]
fn cr_702_9_affinity_reduces_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ph = g.add_card_to_hand(0, catalog::argivian_phalanx());
    // {5}{W} - {3} for three creatures = {2}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ph,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast for the affinity-reduced cost");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Argivian Phalanx"));
}

/// CR 702.153 — paying a Casualty cost copies the spell. A Little Chat with
/// casualty 1 resolves twice, so the controller digs twice.
#[test]
fn cr_702_153_casualty_copies_spell() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power >= 1
    let chat = g.add_card_to_hand(0, catalog::a_little_chat());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellCasualty {
        card_id: chat,
        sacrifice: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with casualty");
    drain_stack(&mut g);
    // The spell + its copy each put one card into hand → +2 (chat itself left hand).
    assert_eq!(
        g.players[0].hand.len(),
        hand_before - 1 + 2,
        "casualty copy dug a second card"
    );
}
