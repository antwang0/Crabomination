//! Functionality tests for `catalog::sets::decks::tdm`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::*;
use crate::mana::Color;

/// Alesha's Legacy grants deathtouch + indestructible to your creature.
#[test]
fn aleshas_legacy_grants_two_keywords() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aleshas_legacy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Alesha's Legacy");
    drain_stack(&mut g);
    let kws = g.computed_permanent(mine).unwrap().keywords;
    assert!(kws.contains(&Keyword::Deathtouch), "gained deathtouch");
    assert!(kws.contains(&Keyword::Indestructible), "gained indestructible");
}

/// Fire-Rim Form pumps +2/+0 and grants first strike on enter.
#[test]
fn fire_rim_form_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::fire_rim_form());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Fire-Rim Form");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creature).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 → 4 power");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "ETB granted first strike");
}

/// Jade-Cast Sentinel bottoms a graveyard card.
#[test]
fn jade_cast_sentinel_bottoms_graveyard_card() {
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::jade_cast_sentinel());
    g.clear_sickness(sentinel);
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sentinel,
        ability_index: 0,
        target: Some(Target::Permanent(dead)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("bottom a graveyard card");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "left the graveyard");
    assert_eq!(g.players[1].library.last().unwrap().id, dead, "went to owner's library bottom");
}

/// Gurmag Nightwatch digs three, keeps one on top, mills the rest.
#[test]
fn gurmag_nightwatch_digs_and_mills() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let gy = g.players[0].graveyard.len();
    let lib = g.players[0].library.len();
    let creature = g.add_card_to_battlefield(0, catalog::gurmag_nightwatch());
    g.fire_self_etb_triggers(creature, 0);
    drain_stack(&mut g);
    // One kept on top, two milled → library down 2, graveyard up 2.
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "milled two");
    assert_eq!(g.players[0].library.len(), lib - 2, "kept one on top");
}

/// Kin-Tree Severance exiles a MV-3+ permanent (and can't hit a cheap one).
#[test]
fn kin_tree_severance_exiles_expensive_permanent() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    let spell = g.add_card_to_hand(0, catalog::kin_tree_severance());
    g.players[0].mana_pool.add_colorless(6);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("exile the MV-5 Angel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "MV-5 permanent exiled");
    assert!(g.exile.iter().any(|c| c.id == big), "went to exile");
}

/// Armament Dragon distributes three +1/+1 counters on enter.
#[test]
fn armament_dragon_distributes_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(0, catalog::armament_dragon());
    g.fire_self_etb_triggers(dragon, 0);
    drain_stack(&mut g);
    // AutoDecider spreads across available creatures; total distributed is 3.
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| *c.counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0))
        .sum();
    assert_eq!(total, 3, "three +1/+1 counters placed");
    let _ = a;
}

/// Fresh Start weakens and silences the enchanted creature.
#[test]
fn fresh_start_shrinks_and_removes_abilities() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer w/ vigilance
    let aura = g.add_card_to_hand(0, catalog::fresh_start());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant the Angel");
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!(cp.power, -1, "4 − 5 = −1 power");
    assert!(cp.keywords.is_empty(), "abilities removed");
}

/// Lie in Wait returns a creature and slings its power at a target.
#[test]
fn lie_in_wait_returns_and_deals_power() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant()); // 3/3, power 3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::lie_in_wait());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![Target::Permanent(foe)],
        mode: None,
        x_value: None,
    })
    .expect("cast Lie in Wait");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned to hand");
    assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
}
