//! CR conformance for this run's engine work:
//! - CR 802 — the attack-multiple-players option: every opponent is a
//!   defending player, and each defender blocks only what attacks them.
//! - CR 803 — the attack-left / attack-right seat restrictions.
//! - CR 706.8 — stored die results and rerolling them.
//! - CR 702.15 / 702.43 — Domain landwalk.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::{AttackOption, multi_player_game};
use crabomination::game::*;

fn combat(seats: usize) -> GameState {
    let mut g = multi_player_game(seats);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g
}

fn attacker(g: &mut GameState, seat: usize) -> CardId {
    let id = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
    g.clear_sickness(id);
    id
}

/// CR 802.2 — with the default option every opponent is a defending player.
#[test]
fn cr_802_2_every_opponent_is_a_defending_player() {
    let mut g = combat(3);
    let a = attacker(&mut g, 0);
    let b = attacker(&mut g, 0);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("one attacker at each opponent");
}

/// CR 802.4a — a defending player can block only creatures attacking them.
#[test]
fn cr_802_4a_defender_blocks_only_its_own_attackers() {
    let mut g = combat(3);
    let at_one = attacker(&mut g, 0);
    let at_two = attacker(&mut g, 0);
    let blocker = attacker(&mut g, 2);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: at_one, target: AttackTarget::Player(1) },
        Attack { attacker: at_two, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 2;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, at_one)])).is_err(),
        "seat 2 isn't being attacked by that creature"
    );
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, at_two)])).is_ok());
}

/// CR 803.1a — attack left restricts the defender to the next seat up.
#[test]
fn cr_803_1a_attack_left_allows_only_the_seat_to_your_left() {
    let mut g = combat(3);
    g.attack_option = AttackOption::AttackLeft;
    let a = attacker(&mut g, 0);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: a,
            target: AttackTarget::Player(2),
        }]))
        .is_err(),
        "seat 2 is to the right"
    );
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: a,
            target: AttackTarget::Player(1),
        }]))
        .is_ok()
    );
}

/// CR 803.1b — attack right is the mirror, wrapping around the table.
#[test]
fn cr_803_1b_attack_right_wraps_to_the_last_seat() {
    let mut g = combat(3);
    g.attack_option = AttackOption::AttackRight;
    let a = attacker(&mut g, 0);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: a,
            target: AttackTarget::Player(1),
        }]))
        .is_err()
    );
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: a,
            target: AttackTarget::Player(2),
        }]))
        .is_ok(),
        "seat 2 is immediately to seat 0's right"
    );
}

/// CR 803.1a — a dead neighbour is "more than one seat away", so no attack is
/// legal at all.
#[test]
fn cr_803_1a_dead_neighbour_leaves_no_legal_defender() {
    let mut g = combat(3);
    g.attack_option = AttackOption::AttackLeft;
    g.players[1].life = 0;
    g.players[1].eliminated = true;
    let a = attacker(&mut g, 0);
    for target in [1usize, 2] {
        assert!(
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: a,
                target: AttackTarget::Player(target),
            }]))
            .is_err()
        );
    }
}

/// CR 706.8a/b — Centaur of Attention stores its rolls and can reroll them.
#[test]
fn cr_706_8_centaur_stores_and_rerolls_die_results() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.add_card_to_hand(0, catalog::centaur_of_attention());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 20);
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: hand,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let id = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Centaur of Attention")
        .unwrap()
        .id;
    let stored = g.battlefield_find(id).unwrap().stored_die_results.clone();
    assert_eq!(stored.len(), 5, "five stored d6 results");
    let biggest_set =
        (1..=6u8).map(|f| stored.iter().filter(|r| **r == f).count()).max().unwrap() as i32;
    assert_eq!(
        g.computed_permanent(id).unwrap().power,
        3 + biggest_set,
        "+X/+X off the biggest matching set"
    );
    // CR 706.8b — the reroll keeps the most common value and rerolls the rest.
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let after = g.battlefield_find(id).unwrap().stored_die_results.clone();
    assert_eq!(after.len(), 5, "rerolling never changes the count");
}

/// CR 702.15 / 702.43 — Magnigoth Treefolk's landwalk tracks Domain live.
#[test]
fn cr_702_43_domain_landwalk_needs_a_shared_basic_type() {
    let mut g = combat(2);
    let tree = g.add_card_to_battlefield(0, catalog::magnigoth_treefolk());
    g.clear_sickness(tree);
    let blocker = attacker(&mut g, 1);
    g.add_card_to_battlefield(1, catalog::mountain());
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tree,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, tree)])).is_ok(),
        "no Mountain on the attacker's side yet"
    );
    g.add_card_to_battlefield(0, catalog::mountain());
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, tree)])).is_err());
}
