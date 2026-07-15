//! Functionality tests for `catalog::sets::decks::recent131` (WOE wave 4) plus
//! the recent113 deferred completions (Twisted Reflection, Bellowing Elk,
//! Windcaller Aven).

use crabomination::card::{
    CardDefinition, CardType, CounterType, EnchantmentSubtype, Keyword, Subtypes,
};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::{cost, generic, Color};

/// A vanilla enchantment fixture (optionally an Aura subtype) for the
/// enchantment-matters triggers.
fn dummy_enchantment(aura: bool) -> CardDefinition {
    CardDefinition {
        name: "Test Glyph",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: if aura { vec![EnchantmentSubtype::Aura] } else { vec![] },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Sacrifice a battlefield permanent, firing dies/LTB triggers (CR 701.16).
fn kill(g: &mut GameState, id: CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    // Flush synthesized non-creature `PermanentDied` events (CR 700.4) so
    // "whenever an enchantment you control … dies" watchers fire.
    g.dispatch_triggers_for_events(&[]);
    drain_stack(g);
}

/// Regal Bunnicorn's `*/*` equals the number of nonland permanents you control.
#[test]
fn regal_bunnicorn_pt_scales() {
    let mut g = two_player_game();
    let bunny = g.add_card_to_battlefield(0, catalog::regal_bunnicorn());
    // Bunny alone → 1 nonland permanent → 1/1.
    let cp = g.computed_permanent(bunny).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    // Two more nonland permanents → 3/3. A land doesn't count.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(bunny).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "3 nonland permanents");
}

/// Savior of the Sleeping grows when your enchantment dies.
#[test]
fn savior_counter_on_enchantment_death() {
    let mut g = two_player_game();
    let savior = g.add_card_to_battlefield(0, catalog::savior_of_the_sleeping());
    let glyph = g.add_card_to_battlefield(0, dummy_enchantment(false));
    kill(&mut g, glyph);
    assert_eq!(
        g.battlefield_find(savior).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "one +1/+1 counter from the enchantment death",
    );
}

/// Wicked Visitor drains each opponent when your enchantment dies.
#[test]
fn wicked_visitor_drains_on_enchantment_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wicked_visitor());
    let glyph = g.add_card_to_battlefield(0, dummy_enchantment(false));
    g.players[1].life = 20;
    kill(&mut g, glyph);
    assert_eq!(g.players[1].life, 19, "opponent loses 1 life");
}

/// Warehouse Tabby makes a Rat when your enchantment dies.
#[test]
fn warehouse_tabby_rat_on_enchantment_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::warehouse_tabby());
    let glyph = g.add_card_to_battlefield(0, dummy_enchantment(false));
    kill(&mut g, glyph);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
        "Rat token created",
    );
}

/// Harried Spearguard leaves a Rat behind when it dies.
#[test]
fn harried_spearguard_rat_on_death() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::harried_spearguard());
    kill(&mut g, guard);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Rat"),
        "Rat token on death",
    );
}

/// Redcap Thief makes a Treasure on entry.
#[test]
fn redcap_thief_treasure_on_etb() {
    let mut g = two_player_game();
    let thief = g.add_card_to_battlefield(0, catalog::redcap_thief());
    g.fire_self_etb_triggers(thief, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "Treasure created",
    );
}

/// Spiteful Hexmage shrinks a creature to 1/1 with a Cursed Role.
#[test]
fn spiteful_hexmage_cursed_role() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mage = g.add_card_to_battlefield(0, catalog::spiteful_hexmage());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    g.fire_self_etb_triggers(mage, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "Cursed Role sets base 1/1");
}

/// Toadstool Admirer grows itself with its activated ability.
#[test]
fn toadstool_admirer_self_pump() {
    let mut g = two_player_game();
    let toad = g.add_card_to_battlefield(0, catalog::toadstool_admirer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: toad,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Toadstool Admirer");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(toad).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Rootrider Faun taps for {G}.
#[test]
fn rootrider_faun_taps_for_green() {
    let mut g = two_player_game();
    let faun = g.add_card_to_battlefield(0, catalog::rootrider_faun());
    g.clear_sickness(faun);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: faun,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap Rootrider Faun for G");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "one green mana");
}

/// Stormkeld Prowler gains two counters when you cast a 5-drop.
#[test]
fn stormkeld_prowler_counters_on_big_spell() {
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::stormkeld_prowler());
    let big = g.add_card_to_hand(0, CardDefinition {
        name: "Test Bomb",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Sorcery],
        ..Default::default()
    });
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast 5-drop");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(prowler).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "two +1/+1 counters",
    );
}

/// Snaremaster Sprite taps and stuns when you pay {2}.
#[test]
fn snaremaster_sprite_pay_taps_and_stuns() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sprite = g.add_card_to_battlefield(0, catalog::snaremaster_sprite());
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(enemy)),
    ]));
    g.fire_self_etb_triggers(sprite, 0);
    drain_stack(&mut g);
    let ec = g.battlefield_find(enemy).unwrap();
    assert!(ec.tapped, "enemy tapped");
    assert_eq!(ec.counter_count(CounterType::Stun), 1, "stun counter placed");
}

/// Twisted Reflection's switch mode swaps a 2/4 into a 4/2.
#[test]
fn twisted_reflection_switches_pt() {
    let mut g = two_player_game();
    // Wall of Wonder is a 1/5; use a 2/4-ish body — grizzly bears is 2/2, so
    // build a fixture with distinct P/T.
    let creature = g.add_card_to_battlefield(1, CardDefinition {
        name: "Test Wall",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 2,
        toughness: 4,
        ..Default::default()
    });
    let spell = g.add_card_to_hand(0, catalog::twisted_reflection());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Choose mode 1 (switch), target the creature.
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast Twisted Reflection");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creature).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "power and toughness switched");
}

/// Bellowing Elk gains trample + indestructible once another creature entered.
#[test]
fn bellowing_elk_conditional_keywords() {
    let mut g = two_player_game();
    let elk = g.add_card_to_battlefield(0, catalog::bellowing_elk());
    // No other creature entered yet → bare 4/2.
    let cp = g.computed_permanent(elk).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Trample), "no keywords without another arrival");
    // Simulate another creature having entered under your control this turn.
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].creatures_entered_this_turn.push(other);
    let cp = g.computed_permanent(elk).unwrap();
    assert!(
        cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Indestructible),
        "gains trample + indestructible",
    );
    // The elk's own arrival must not satisfy the self-excluding predicate.
    g.players[0].creatures_entered_this_turn = vec![elk];
    let cp = g.computed_permanent(elk).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Trample), "own arrival doesn't count");
}

/// Windcaller Aven grants flying when cycled.
#[test]
fn windcaller_aven_cycle_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aven = g.add_card_to_hand(0, catalog::windcaller_aven());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    g.perform_action(GameAction::Cycle { card_id: aven, x_value: None }).expect("cycle Windcaller Aven");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "bear gains flying");
}
