//! CR 506.4 / 509.1 — combat bookkeeping around a block's costs.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::*;

/// CR 509.1d / 506.4 / 509.1h — Wall of Roots and a Bear block a Bear
/// attacking beside Archangel of Tithes; the {2} tax takes a floating {C} and the
/// Wall's own "-0/-1 counter: add {G}" at 0/1: the Wall dies to the cost and
/// leaves combat, and the attacker stays blocked by the Bear. (A 4-seat pod's debug
/// invariant, seed 121000 game 15.)
#[test]
fn cr_506_4_a_blocker_its_block_tax_killed_leaves_combat() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let angel = g.add_card_to_battlefield(0, catalog::archangel_of_tithes());
    let raider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(angel);
    g.clear_sickness(raider);
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_roots());
    g.battlefield_find_mut(wall).unwrap().add_counters(CounterType::MinusZeroMinusOne, 4);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    for id in [wall, bear] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: angel, target: AttackTarget::Player(1) },
        Attack { attacker: raider, target: AttackTarget::Player(1) },
    ]).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    let evs = g.declare_blockers(vec![(wall, raider), (bear, raider)]).expect("the wall pays its own tax");
    g.dispatch_triggers_for_events(&evs);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(wall).is_none(), "the -0/-1 counter killed it");
    assert!(!g.block_map.contains_key(&wall), "a blocker that left is out of combat: {:?}", g.block_map);
    assert!(g.block_map.contains_key(&bear));
    assert!(g.blocked_attackers().contains(&raider), "the attacker stays blocked");
}

/// CR 508.1h-j / 506.4 — the attacking side: Blood Pet attacks into Ghostly
/// Prison and the {2} tax takes a floating {C} and Blood Pet's own "sacrifice
/// this: add {B}" — it never makes it into combat.
#[test]
fn cr_506_4_an_attacker_its_attack_tax_killed_leaves_combat() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(1, catalog::ghostly_prison());
    let pet = g.add_card_to_battlefield(0, catalog::blood_pet());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(pet);
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: pet, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])
    .expect("the Pet pays for the attack with itself");
    assert!(g.battlefield_find(pet).is_none(), "sacrificed to the tax");
    assert_eq!(g.attacking.iter().map(|a| a.attacker).collect::<Vec<_>>(), vec![bear]);
}
