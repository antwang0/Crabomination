//! Functionality tests for `catalog::sets::decks::recent147` (WOE wave).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::*;
use crabomination::game::two_player_game;

fn advance_to(g: &mut GameState, step: crabomination::game::TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Gingerbread Cabin enters tapped with fewer than three other Forests; with
/// three it enters untapped and mints a Food.
#[test]
fn gingerbread_cabin_conditional_tap_and_food() {
    let mut g = two_player_game();
    let tapped = g.move_card_to_battlefield_for_test(0, catalog::gingerbread_cabin());
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapped).unwrap().tapped, "enters tapped with no Forests");
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Food"),
        "no Food when it enters tapped",
    );
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let untapped = g.move_card_to_battlefield_for_test(0, catalog::gingerbread_cabin());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(untapped).unwrap().tapped, "untapped with three Forests");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food minted");
}

/// The Witch's Vanity chapter I destroys a small opposing creature.
#[test]
fn witchs_vanity_chapter_one_destroys_small_creature() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::the_witchs_vanity());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "MV-2 creature destroyed");
}

/// Imodane's Recruiter's ETB pumps and hastens your team.
#[test]
fn imodanes_recruiter_team_pump_and_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::imodanes_recruiter());
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 3, "+1/+0 to the team");
    assert!(c.keywords.contains(&crabomination::card::Keyword::Haste), "granted haste");
}

/// Elvish Vanguard grows whenever another Elf enters.
#[test]
fn elvish_vanguard_grows_on_elf() {
    let mut g = two_player_game();
    let vanguard = g.add_card_to_battlefield(0, catalog::elvish_vanguard());
    let elf = g.add_card_to_battlefield(0, catalog::yevas_forcemage()); // an Elf
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: elf }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vanguard).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "another Elf entering added a counter",
    );
}

/// Neutralizing Blast counters a multicolored spell but not a monocolored one.
#[test]
fn neutralizing_blast_only_hits_multicolored() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // A monocolored creature spell on the stack is not a legal target.
    let mono = g.add_card_to_hand(0, catalog::grizzly_bears());
    let blast = g.add_card_to_hand(1, catalog::neutralizing_blast());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: mono, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the bear");
    g.players[1].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: blast,
            target: Some(Target::Permanent(mono)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "monocolored spell is not a legal target",
    );
}

/// Hoard Robber mints a Treasure on combat damage to a player.
#[test]
fn hoard_robber_treasure_on_combat_damage() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let robber = g.add_card_to_battlefield(0, catalog::hoard_robber());
    g.clear_sickness(robber);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: robber,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
        "combat damage to a player made a Treasure",
    );
}
