//! Functionality tests for the `catalog::sets::decks::recent22` batch —
//! Firebending (CR 702.189): an attack-triggered mana ability adding N {R}
//! that survives until end of combat.

use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn attack_with(g: &mut GameState, attacker: CardId) {
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Jeong Jeong's firebending 1 adds {R} on attack, and that mana survives the
/// step change into combat damage (it doesn't empty until end of combat).
#[test]
fn jeong_jeong_firebending_adds_red_that_survives_steps() {
    let mut g = two_player_game();
    let jj = g.add_card_to_battlefield(0, catalog::jeong_jeong_the_deserter());
    g.clear_sickness(jj);
    attack_with(&mut g, jj);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "firebending 1 added {{R}}");
    // Move out of declare-attackers (mana would normally empty between steps).
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert!(
        g.players[0].mana_pool.amount(Color::Red) >= 1,
        "firebending mana persists across the step change in combat"
    );
}

/// Once combat ends the firebending mana is cleared (doesn't leak into the
/// second main phase).
#[test]
fn firebending_mana_clears_after_combat() {
    let mut g = two_player_game();
    let jj = g.add_card_to_battlefield(0, catalog::jeong_jeong_the_deserter());
    g.clear_sickness(jj);
    attack_with(&mut g, jj);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert_eq!(
        g.players[0].mana_pool.amount(Color::Red), 0,
        "firebending mana gone after combat"
    );
    assert_eq!(g.players[0].firebending_kept_red, 0, "kept-mana tracker reset");
}

/// Sozin's Comet grants firebending 5 to your creatures; a Grizzly Bears then
/// makes {R}{R}{R}{R}{R} when it attacks.
#[test]
fn sozins_comet_grants_firebending() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let comet = g.add_card_to_hand(0, catalog::sozins_comet());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: comet, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sozin's Comet");
    drain_stack(&mut g);
    attack_with(&mut g, bear);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 5, "granted firebending 5 added {{R}}×5");
}
