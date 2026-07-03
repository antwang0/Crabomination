//! Functionality tests for the `catalog::sets::decks::recent99` Kamigawa: Neon
//! Dynasty batch 5.

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::types::Target;
use crate::game::*;

/// Guardian Kirin grows when another creature you control dies.
#[test]
fn guardian_kirin_grows_on_ally_death() {
    let mut g = two_player_game();
    let kirin = g.add_card_to_battlefield(0, catalog::guardian_kirin());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(kirin).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "a +1/+1 counter for the ally's death"
    );
}

/// Silver-Fur Master anthems other Ninja/Rogue creatures you control.
#[test]
fn silver_fur_master_anthems_ninjas() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::silver_fur_master());
    let ninja = g.add_card_to_battlefield(0, catalog::dokuchi_shadow_walker()); // 5/5 Ninja
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(ninja).unwrap().power, 6, "other Ninja gets +1/+1");
    assert_eq!(g.computed_permanent(master).unwrap().power, 2, "the lord doesn't pump itself");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-Ninja unaffected");
    assert!(master != ninja && bear != master);
}

/// Generous Visitor puts a counter when you cast an enchantment spell.
#[test]
fn generous_visitor_counters_on_enchantment_cast() {
    let mut g = two_player_game();
    let visitor = g.add_card_to_battlefield(0, catalog::generous_visitor());
    let ench = g.add_card_to_hand(0, catalog::golden_tail_disciple()); // enchantment creature
    for _ in 0..2 { g.players[0].mana_pool.add(crate::mana::Color::White, 1); }
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast an enchantment");
    drain_stack(&mut g);
    // The visitor is a legal target for its own trigger; assert a counter landed.
    let placed: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(placed, 1, "one +1/+1 counter from the enchantment cast");
    let _ = visitor;
}

/// Boon of Boseiju pumps by the greatest mana value you control and untaps.
#[test]
fn boon_of_boseiju_pumps_by_greatest_mv() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dokuchi_shadow_walker()); // MV 6
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::boon_of_boseiju());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast Boon of Boseiju");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 8, "2 + 6 (greatest MV among your permanents)");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped by the boon");
    assert!(!cp.keywords.contains(&Keyword::Defender));
}
