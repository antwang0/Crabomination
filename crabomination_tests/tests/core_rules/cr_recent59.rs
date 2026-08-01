//! CR conformance for this run's engine work:
//! - CR 702.161 — Living metal: a Vehicle is an artifact creature during its
//!   controller's turn, no crew needed.
//! - CR 702.162 / 701.28 — More Than Meets the Eye: casting a card *converted*
//!   puts it onto the battlefield with its back face up.
//! - CR 701.9 — a discard paid as a cost fires the "you discarded one or more
//!   cards" batch, same as a discard from an effect.

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::game::types::GameAction;
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// CR 702.161 — the Vehicle side is a creature only on its controller's turn.
#[test]
fn cr_702_161_living_metal_animates_on_your_turn_only() {
    let mut g = two_player_game();
    let slicer = g.add_card_to_battlefield(0, catalog::slicer_high_speed_antagonist());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(slicer).unwrap().card_types.contains(&CardType::Creature));
    g.active_player_idx = 1;
    assert!(
        !g.computed_permanent(slicer).unwrap().card_types.contains(&CardType::Creature),
        "it's back to a plain Vehicle on their turn"
    );
}

/// CR 702.162 / 701.28 — the More Than Meets the Eye cast enters converted.
#[test]
fn cr_702_162_more_than_meets_the_eye_enters_converted() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let slicer = g.add_card_to_hand(0, catalog::slicer_hired_muscle());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: slicer,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("cast converted");
    drain_stack(&mut g);
    let perm = g.battlefield_find(slicer).expect("on the battlefield");
    assert_eq!(perm.definition.name, "Slicer, High-Speed Antagonist");
    assert!(perm.transformed);
}

/// The printed cost still lands the Robot front face.
#[test]
fn cr_702_162_the_printed_cost_lands_the_front_face() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let slicer = g.add_card_to_hand(0, catalog::slicer_hired_muscle());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: slicer,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let perm = g.battlefield_find(slicer).expect("on the battlefield");
    assert_eq!(perm.definition.name, "Slicer, Hired Muscle");
    assert!(!perm.transformed);
}

/// CR 701.9 — a discard paid as an activation cost fires the batch trigger.
#[test]
fn cr_701_9_cost_payment_discard_fires_the_batch() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Magmakin Artillerist: "whenever you discard one or more cards, it deals
    // that much damage to each opponent."
    g.add_card_to_battlefield(0, catalog::magmakin_artillerist());
    let shaper = g.add_card_to_battlefield(0, catalog::alexi_zephyr_mage());
    g.clear_sickness(shaper);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaper,
        ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "the two-card cost discard billed for 2");
}
