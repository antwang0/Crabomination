//! Coveted Jewel ETB draw-three (modern_decks), and its CR 603.2c batch.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameAction};

#[test]
fn coveted_jewel_etb_draws_three() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::coveted_jewel());
    g.players[0].mana_pool.add_colorless(6);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    // Jewel leaves hand (-1); ETB draws 3 (+3): net +2.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

/// CR 603.2c — "whenever one or more creatures an opponent controls attack
/// you, that player gains control of this artifact and untaps it": **one**
/// fire for the declaration.
///
/// ⚠⚠ This card has carried `once_per_batch` and a comment claiming the fix
/// since it was written — and the site that pushed the trigger threw the flag
/// away until 2026-09-19, because `combat.rs`'s defender-side listener walk
/// pushed with no once-key at all. Every gate in the tree asked whether the
/// *card* was right. This one asks whether the *engine* read it.
#[test]
fn cr_603_2c_coveted_jewel_changes_hands_once_a_declaration() {
    let mut g = two_player_game();
    let jewel = g.add_card_to_battlefield(0, catalog::coveted_jewel());
    g.battlefield_find_mut(jewel).unwrap().tapped = true;
    let attackers: Vec<_> =
        (0..3).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    for &a in &attackers {
        g.clear_sickness(a);
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(0) }).collect(),
    ))
    .expect("three attackers");
    assert_eq!(g.stack.len(), 1, "three attackers, one trigger on the stack");
    drain_stack(&mut g);
    let j = g.battlefield_find(jewel).expect("still on the battlefield");
    assert_eq!(j.controller, 1, "the attacking player took it");
    assert!(!j.tapped, "…and untapped it");
}
