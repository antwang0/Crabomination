//! CR conformance for rules exercised by this run's GTC waves and the
//! multi-target-loyalty engine change:
//! CR 122.3 (+1/+1 and −1/−1 counters annihilate as an SBA), CR 606.3 (loyalty
//! abilities are sorcery-speed and once per turn — Domri Rade), and CR 702.2
//! (deathtouch makes any nonzero combat damage lethal).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameError};

/// CR 122.3 — N of each of +1/+1 and −1/−1 are removed together as an SBA.
#[test]
fn cr_122_3_plus_minus_counters_annihilate() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::ruination_wurm());
    {
        let inst = g.battlefield_find_mut(c).unwrap();
        inst.add_counters(CounterType::PlusOnePlusOne, 3);
        inst.add_counters(CounterType::MinusOneMinusOne, 2);
    }
    g.check_state_based_actions();
    let inst = g.battlefield_find(c).unwrap();
    assert_eq!(inst.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1,
        "3 − 2 = 1 +1/+1 counter remains");
    assert_eq!(inst.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0), 0,
        "all −1/−1 counters annihilated");
}

/// CR 606.3 — a loyalty ability can be activated only once per turn.
#[test]
fn cr_606_3_loyalty_once_per_turn() {
    let mut g = two_player_game();
    let domri = g.add_card_to_battlefield(0, catalog::domri_rade());
    g.add_card_to_library(0, catalog::ruination_wurm());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: domri, ability_index: 0, target: None, x_value: None,
    }).expect("first activation");
    drain_stack(&mut g);
    let again = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: domri, ability_index: 0, target: None, x_value: None,
    });
    assert!(matches!(again, Err(GameError::LoyaltyAbilityAlreadyUsed(_))),
        "second loyalty activation the same turn is rejected");
}

/// CR 606.3 — loyalty abilities are sorcery-speed (main phase, empty stack).
#[test]
fn cr_606_3_loyalty_is_sorcery_speed() {
    let mut g = two_player_game();
    let domri = g.add_card_to_battlefield(0, catalog::domri_rade());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers; // not a main phase
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: domri, ability_index: 0, target: None, x_value: None,
    });
    assert!(matches!(res, Err(GameError::SorcerySpeedOnly)),
        "loyalty can't be activated outside a main phase");
}

/// CR 702.2 — any nonzero combat damage from a deathtouch source is lethal.
#[test]
fn cr_702_2_deathtouch_kills_big_creature() {
    let mut g = two_player_game();
    let dryad = g.add_card_to_battlefield(0, catalog::gnarlwood_dryad()); // 1/1 deathtouch
    let wurm = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    assert!(g.computed_permanent(dryad).unwrap().keywords.contains(&Keyword::Deathtouch));
    g.clear_sickness(dryad);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: dryad, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wurm, dryad)])).expect("block");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let _ = Target::Player(1);
    assert!(g.battlefield_find(wurm).is_none(), "the 7/6 died to 1 deathtouch damage");
}
