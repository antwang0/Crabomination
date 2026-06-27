//! Functionality tests for `catalog::sets::decks::recent23` —
//! `Keyword::AssignsCombatDamageByToughness` (CR 510.1c).

use crate::catalog;
use crate::card::Keyword;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::TurnStep;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Doran makes every creature assign combat damage equal to its toughness: an
/// unblocked 0/5 Doran deals 5 to the defending player.
#[test]
fn doran_attacks_for_toughness() {
    let mut g = two_player_game();
    let doran = g.add_card_to_battlefield(0, catalog::doran_the_siege_tower()); // 0/5
    g.clear_sickness(doran);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: doran,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 15, "0/5 Doran assigns 5 (toughness)");
}

/// Doran's substitution is unconditional even when power exceeds toughness: a
/// 3/1 attacker assigns only 1.
#[test]
fn doran_caps_high_power_attacker_at_toughness() {
    let mut g = two_player_game();
    let doran = g.add_card_to_battlefield(0, catalog::doran_the_siege_tower());
    g.clear_sickness(doran);
    let bolt = g.add_card_to_battlefield(0, catalog::goblin_piker()); // 2/1
    g.clear_sickness(bolt);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bolt,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 19, "2/1 under Doran assigns 1 (toughness)");
}

/// Tapestry Warden only affects your creatures whose toughness exceeds their
/// power: a 1/4 Wall assigns 4, while a 2/1 you control assigns its normal 2.
#[test]
fn tapestry_warden_only_buffs_high_toughness() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::tapestry_warden());
    g.clear_sickness(warden);
    // Warden itself is 3/4 (T>P) → assigns 4.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: warden,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 16, "3/4 Warden assigns 4 (toughness)");
}

/// A creature you control with power ≥ toughness is left alone by Tapestry
/// Warden (a 2/1 still assigns 2, not 1).
#[test]
fn tapestry_warden_ignores_low_toughness() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::tapestry_warden());
    g.clear_sickness(warden);
    let piker = g.add_card_to_battlefield(0, catalog::goblin_piker()); // 2/1
    g.clear_sickness(piker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: piker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 18, "2/1 unaffected → assigns 2 (power)");
}

/// Ancient Lumberknot reuses Tapestry Warden's static: a 1/4 it controls (T>P)
/// assigns 4, attacking unblocked.
#[test]
fn ancient_lumberknot_buffs_high_toughness() {
    let mut g = two_player_game();
    let knot = g.add_card_to_battlefield(0, catalog::ancient_lumberknot()); // 1/4
    g.clear_sickness(knot);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: knot,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 16, "1/4 Lumberknot assigns 4 (toughness)");
}

/// Thrumming Hivepool's lord static grants double strike + haste to Slivers,
/// and its Affinity for Slivers reduces its {6} cost by {1} per Sliver (so two
/// Slivers let it cast for {4}).
#[test]
fn thrumming_hivepool_affinity_and_lord() {
    let mut g = two_player_game();
    let s1 = g.add_card_to_battlefield(0, catalog::muscle_sliver());
    g.add_card_to_battlefield(0, catalog::muscle_sliver());
    let pool = g.add_card_to_battlefield(0, catalog::thrumming_hivepool());
    assert!(
        g.computed_permanent(s1)
            .is_some_and(|c| c.keywords.contains(&Keyword::DoubleStrike)
                && c.keywords.contains(&Keyword::Haste)),
        "Slivers gain double strike + haste"
    );
    // Affinity: {6} reduced by {1} per Sliver (2 on board) → {4} generic.
    let inst = g.battlefield.iter().find(|c| c.id == pool).unwrap().clone();
    let reduced = crate::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
    assert_eq!(reduced, 2, "Affinity for Slivers gives {{2}} off with two Slivers");
}

/// Bill the Pony enters with two Food and can sacrifice one to grant the
/// toughness-damage keyword to a target creature you control until end of turn.
#[test]
fn bill_the_pony_etb_food_and_grant() {
    let mut g = two_player_game();
    let bill = g.move_card_to_battlefield_for_test(0, catalog::bill_the_pony());
    g.clear_sickness(bill);
    drain_stack(&mut g);
    let foods = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.is_token)
        .count();
    assert_eq!(foods, 2, "ETB makes two Food tokens");

    // Grant the keyword to Bill (a 1/4) by sacrificing a Food.
    g.perform_action(GameAction::ActivateAbility {
        card_id: bill,
        ability_index: 0,
        target: Some(Target::Permanent(bill)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate sac-a-Food grant");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bill)
            .is_some_and(|c| c.keywords.contains(&Keyword::AssignsCombatDamageByToughness)),
        "Bill now assigns combat damage by toughness"
    );
    let foods_after = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.is_token)
        .count();
    assert_eq!(foods_after, 1, "one Food sacrificed");
}
