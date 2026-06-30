//! Functionality tests for the `catalog::sets::decks::recent14` batch.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;

/// Quirion Beastcaller grows when you cast a creature spell.
#[test]
fn quirion_beastcaller_grows_on_creature_cast() {
    let mut g = two_player_game();
    let quirion = g.add_card_to_battlefield(0, catalog::quirion_beastcaller());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast(&mut g, bear);
    assert_eq!(
        g.battlefield_find(quirion).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "a +1/+1 counter per creature spell cast"
    );
}

/// Quirion's death distributes its counters onto another creature you control.
#[test]
fn quirion_beastcaller_distributes_counters_on_death() {
    let mut g = two_player_game();
    let quirion = g.add_card_to_battlefield(0, catalog::quirion_beastcaller());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == quirion).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 2);
    let _ = g.remove_to_graveyard_with_triggers(quirion);
    drain_stack(&mut g);
    assert!(g.battlefield_find(quirion).is_none(), "Quirion died");
    assert_eq!(
        g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "the two +1/+1 counters land on the other creature"
    );
}

/// Yotian Frontliner buffs another creature you control when it attacks.
#[test]
fn yotian_frontliner_buffs_on_attack() {
    let mut g = two_player_game();
    let yotian = g.add_card_to_battlefield(0, catalog::yotian_frontliner());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(yotian);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: yotian, target: AttackTarget::Player(1) }])
        .expect("Yotian attacks");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "ally got +1/+1");
}

/// Yotian Frontliner can return from the graveyard via Unearth.
#[test]
fn yotian_frontliner_unearths() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::yotian_frontliner());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("unearth");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "Yotian returned to the battlefield");
}

/// Heaped Harvest's ETB fetches a basic land onto the battlefield tapped.
#[test]
fn heaped_harvest_etb_fetches_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_battlefield(0, catalog::heaped_harvest());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let lands = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land() && c.tapped).count();
    assert_eq!(lands, 1, "a basic land entered tapped");
}
