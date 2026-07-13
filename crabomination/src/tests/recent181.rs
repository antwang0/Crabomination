//! Functionality tests for `catalog::sets::decks::recent181` (BLB batch).

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Plumecreed Mentor puts a counter on a non-flyer when a flyer enters.
#[test]
fn plumecreed_mentor_counters_a_grounded_creature() {
    let mut g = two_player_game();
    let grounded = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no flying
    let mentor = g.move_card_to_battlefield_for_test(0, catalog::plumecreed_mentor());
    // The Mentor is itself a flyer; its "this or another flying creature you
    // control enters" trigger (YourControl scope) fires for its own entry.
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: mentor }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(grounded).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the grounded creature got a +1/+1 counter",
    );
}

/// Azure Beastbinder strips abilities and shrinks its target on attack.
#[test]
fn azure_beastbinder_strips_on_attack() {
    let mut g = two_player_game();
    let binder = g.add_card_to_battlefield(0, catalog::azure_beastbinder());
    g.clear_sickness(binder);
    let mut flyer = catalog::grizzly_bears();
    flyer.keywords.push(Keyword::Flying);
    flyer.power = 5;
    flyer.toughness = 5;
    let foe = g.add_card_to_battlefield(1, flyer);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: binder,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Flying), "lost its abilities");
    assert_eq!((cp.power, cp.toughness), (2, 2), "became a 2/2");
}

/// Byrke's attack trigger doubles the +1/+1 counters on an attacking creature.
#[test]
fn byrke_doubles_counters_on_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::byrke_long_ear_of_the_law());
    let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(beater).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(beater);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: beater,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(beater).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4,
        "2 counters doubled to 4",
    );
}

/// Dreamdew Entrancer stuns a creature and draws when it's yours.
#[test]
fn dreamdew_entrancer_stuns_and_draws() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::dreamdew_entrancer());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(mine).unwrap().counter_count(CounterType::Stun),
        3,
        "three stun counters",
    );
    assert!(g.battlefield_find(mine).unwrap().tapped, "tapped by the ETB");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two (you control the target)");
}
