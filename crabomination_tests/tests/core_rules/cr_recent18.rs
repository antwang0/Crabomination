//! CR conformance for rules exercised by the GTC wave-7 batch:
//! CR 305.6 (a land taps for mana of every basic type it has, including one
//! granted continuously — Realmwright), CR 701.12 (Fight deals damage
//! simultaneously, so both creatures can die — Gruul Ragebeast), and
//! CR 509.1c ("can't attack or block alone" — Ember Beast).

use crabomination::card::LandType;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::mana::{self, Color};
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState};
use crabomination::catalog;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 305.6 — Realmwright makes your lands the chosen basic type in addition,
/// and the intrinsic mana ability follows: a Plains chosen as Island taps for
/// blue while keeping its white.
#[test]
fn cr_305_6_realmwright_land_taps_for_chosen_color() {
    let mut g = two_player_game();
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.move_card_to_battlefield_for_test(0, catalog::realmwright());
    drain_stack(&mut g);
    let cp = g.computed_permanent(plains).unwrap();
    assert!(cp.subtypes.land_types.contains(&LandType::Island), "gained Island");
    assert!(cp.subtypes.land_types.contains(&LandType::Plains), "kept Plains");
    g.auto_tap_for_cost(0, &mana::cost(&[mana::u()]));
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "taps for blue");
}

/// CR 701.12b — Fight deals damage simultaneously; a 6/6 and a 7/6 fighting
/// via Gruul Ragebeast's ETB both take lethal and die together.
#[test]
fn cr_701_12_fight_is_simultaneous_both_die() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    let ragebeast = g.move_card_to_battlefield_for_test(0, catalog::gruul_ragebeast()); // 6/6
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "wurm took 6 (lethal) and died");
    assert!(g.battlefield_find(ragebeast).is_none(), "ragebeast took 7 and died");
}

/// CR 509.1c — Ember Beast can't attack alone: a lone declaration is rejected,
/// but a second attacker makes the batch legal.
#[test]
fn cr_509_1c_ember_beast_cant_attack_alone() {
    let mut g = two_player_game();
    let ember = g.add_card_to_battlefield(0, catalog::ember_beast());
    let buddy = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    g.clear_sickness(ember);
    g.clear_sickness(buddy);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ember, target: AttackTarget::Player(1),
        }])).is_err(),
        "lone Ember Beast can't attack",
    );
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: ember, target: AttackTarget::Player(1) },
        Attack { attacker: buddy, target: AttackTarget::Player(1) },
    ])).expect("attacking together is legal");
}
