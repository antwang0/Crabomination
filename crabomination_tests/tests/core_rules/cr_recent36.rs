//! CR conformance for the end-of-combat step, removal from combat, and space
//! sculptor:
//! - CR 511.2 — "at end of combat" abilities trigger as the step begins.
//! - CR 506.4c — an attacker whose planeswalker left combat stays attacking but
//!   deals no combat damage.
//! - CR 702.158 / 704.5u — sector designations, the same-sector block lock, and
//!   the sector-wide payoffs.

use crabomination::card::{CounterType, Sector};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 511.2 — a `DelayedKind::EndOfCombat` trigger fires when the step begins.
#[test]
fn cr_511_2_end_of_combat_delayed_trigger_fires() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let attacker = g.add_card_to_battlefield(1, catalog::hill_giant());
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.declare_blockers(vec![(blocker, attacker)]).expect("block");
    drain_stack(&mut g);

    let tactics = g.add_card_to_hand(0, catalog::triton_tactics());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: tactics,
        target: Some(Target::Permanent(blocker)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.delayed_triggers.len(), 1, "armed for end of combat");

    while g.step != TurnStep::EndCombat {
        g.advance_step(Vec::new()).expect("advance");
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.delayed_triggers.is_empty(), "fired as the step began");
}

/// CR 506.4c — the attacked planeswalker leaving combat doesn't remove the
/// attacker from combat, but an unblocked attacker deals no damage.
#[test]
fn cr_506_4c_attacker_survives_its_planeswalker_leaving() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let walker = g.add_card_to_battlefield(1, catalog::xenagos_the_reveler());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack {
        attacker,
        target: AttackTarget::Planeswalker(walker),
    }])
    .expect("attack the walker");
    drain_stack(&mut g);

    // Remove the planeswalker from combat before damage.
    g.destroy_permanent(walker, false, &mut Vec::new());
    g.check_state_based_actions();
    assert!(
        g.attacking.iter().any(|a| a.attacker == attacker),
        "still an attacking creature"
    );
    let life = g.players[1].life;
    g.step = TurnStep::DeclareBlockers;
    g.advance_step(Vec::new()).expect("to damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "and deals no damage to anyone");
}

/// CR 704.5u — a space sculptor assigns every creature a sector, and the
/// designations clear when it leaves (CR 702.158b).
#[test]
fn cr_704_5u_space_sculptor_assigns_and_clears_sectors() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(bear).unwrap().sector, None, "no sculptor, no sectors");

    let beleren = g.add_card_to_battlefield(0, catalog::space_beleren());
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).unwrap().sector.is_some(), "assigned a sector");

    g.destroy_permanent(beleren, false, &mut Vec::new());
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(bear).unwrap().sector, None, "cleared with the last sculptor");
}

/// CR 702.158d — Space Beleren's +1 locks blocks to the same sector; his −5
/// wipes one sector and leaves the others alone.
#[test]
fn cr_702_158d_sector_block_lock_and_wipe() {
    let mut g = two_player_game();
    let beleren = g.add_card_to_battlefield(0, catalog::space_beleren());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.check_state_based_actions();
    g.battlefield_find_mut(attacker).unwrap().sector = Some(Sector::Alpha);
    g.battlefield_find_mut(blocker).unwrap().sector = Some(Sector::Beta);
    assert!(g.blocker_can_block_attacker(blocker, attacker), "no lock yet");

    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: beleren,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert!(!g.blocker_can_block_attacker(blocker, attacker), "different sectors can't block");
    g.battlefield_find_mut(blocker).unwrap().sector = Some(Sector::Alpha);
    assert!(g.blocker_can_block_attacker(blocker, attacker), "same sector can");

}

/// CR 702.158d — Space Beleren's −1 grows every creature in the chosen sector
/// and leaves the others alone.
#[test]
fn cr_702_158d_sector_pump_hits_one_sector() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let beleren = g.add_card_to_battlefield(0, catalog::space_beleren());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.check_state_based_actions();
    g.battlefield_find_mut(a).unwrap().sector = Some(Sector::Alpha);
    g.battlefield_find_mut(b).unwrap().sector = Some(Sector::Beta);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: beleren,
        ability_index: 1,
        target: None,
        x_value: None,
    })
    .expect("-1");
    drain_stack(&mut g);
    let counters = |g: &GameState, id| {
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
    };
    assert_ne!(counters(&g, a), counters(&g, b), "exactly one sector grew");
    assert_eq!(counters(&g, a) + counters(&g, b), 1);
}
