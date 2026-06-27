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

/// Sneak (CR 702.190): during the declare blockers step you may cast Donatello's
/// Technique for {U} by returning an unblocked attacker you control to hand.
#[test]
fn donatello_sneak_returns_unblocked_attacker_for_cheap() {
    use crate::game::types::Attack;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let don = g.add_card_to_hand(0, catalog::donatellos_technique());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1); // only the {U} Sneak cost
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: don, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sneak-cast Donatello's Technique for {U}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "unblocked attacker returned to hand");
    // Drew 2 (+2), bear back (+1), Donatello left hand (-1) → net +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two and got the attacker back");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0, "only {{U}} was paid");
}

/// Ran and Shaw's firebending 2 adds {R}{R} on attack.
#[test]
fn ran_and_shaw_firebending_two() {
    let mut g = two_player_game();
    let rs = g.add_card_to_battlefield(0, catalog::ran_and_shaw());
    g.clear_sickness(rs);
    attack_with(&mut g, rs);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "firebending 2 added {{R}}{{R}}");
}

/// Jennika's Technique deals 2 damage to each creature.
#[test]
fn jennikas_technique_sweeps_two() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 dies
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 dies
    let jt = g.add_card_to_hand(0, catalog::jennikas_technique());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: jt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Jennika's Technique");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
        "both 2/2s died to 2 damage each");
}

fn cast_creature(g: &mut GameState, card: CardId) {
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast creature");
    drain_stack(g);
}

/// Bloodthirst 1 (CR 702.54): Bloodrage Vampire enters with a +1/+1 counter
/// only if an opponent took damage this turn.
#[test]
fn bloodrage_vampire_bloodthirst_conditional() {
    use crate::card::CounterType;
    // No opponent damage → enters a vanilla 3/1.
    let mut g = two_player_game();
    let v1 = g.add_card_to_hand(0, catalog::bloodrage_vampire());
    cast_creature(&mut g, v1);
    assert_eq!(g.battlefield_find(v1).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "no bloodthirst without opponent damage");

    // Opponent took damage this turn → enters with one +1/+1 counter.
    let mut g = two_player_game();
    g.players[1].was_dealt_damage_this_turn = true;
    let v2 = g.add_card_to_hand(0, catalog::bloodrage_vampire());
    cast_creature(&mut g, v2);
    assert_eq!(g.battlefield_find(v2).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "bloodthirst 1 adds a counter");
}

/// Furyborn Hellkite's bloodthirst 6 adds six counters after opponent damage.
#[test]
fn furyborn_hellkite_bloodthirst_six() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    g.players[1].was_dealt_damage_this_turn = true;
    let dragon = g.add_card_to_hand(0, catalog::furyborn_hellkite());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Furyborn Hellkite");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dragon).unwrap().counter_count(CounterType::PlusOnePlusOne), 6,
        "bloodthirst 6 adds six counters");
}

/// Sneak is only legal during your declare blockers step (CR 702.190a).
#[test]
fn sneak_rejected_outside_declare_blockers() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let don = g.add_card_to_hand(0, catalog::donatellos_technique());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    let res = g.perform_action(GameAction::CastSpellAlternative {
        card_id: don, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(res.is_err(), "Sneak only works during the declare blockers step");
}
