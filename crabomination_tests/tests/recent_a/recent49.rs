//! Functionality tests for `catalog::sets::decks::recent49`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

#[test]
fn ghoultree_costs_less_per_graveyard_creature() {
    let mut g = two_player_game();
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let tree = g.add_card_to_hand(0, catalog::ghoultree());
    g.players[0].mana_pool.add(Color::Green, 1); // {7}{G} - 7 creatures = {G}
    g.perform_action(GameAction::CastSpell {
        card_id: tree, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ghoultree castable for {G} with 7 creatures in the yard");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ghoultree"));
}

#[test]
fn nyx_weaver_mills_on_upkeep() {
    let mut g = two_player_game();
    let nyx = g.add_card_to_battlefield(0, catalog::nyx_weaver());
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let gy0 = g.players[0].graveyard.len();
    let trig = catalog::nyx_weaver().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(nyx, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[0].graveyard.len(), gy0 + 2, "milled two");
}

#[test]
fn nyx_weaver_exiles_itself_to_recur() {
    let mut g = two_player_game();
    let nyx = g.add_card_to_battlefield(0, catalog::nyx_weaver());
    let want = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: nyx, ability_index: 0, target: Some(Target::Permanent(want)),
        additional_targets: vec![], x_value: None,
    }).expect("exile-self recursion");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == nyx), "Nyx Weaver exiled itself");
    assert!(g.players[0].hand.iter().any(|c| c.id == want), "returned the gy card");
}

#[test]
fn genesis_returns_a_creature_from_the_yard() {
    let mut g = two_player_game();
    let gcard = g.add_card_to_graveyard(0, catalog::genesis());
    let want = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let trig = catalog::genesis().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_trigger(gcard, 0, None, 0);
    ctx.targets = vec![Target::Permanent(want)];
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == want), "creature card returned to hand");
}

#[test]
fn elephant_guide_pumps_and_leaves_an_elephant() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::elephant_guide());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura on bear");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+3 from the Guide");
    // Bear dies (lethal SBA records the aura link) → 3/3 Elephant.
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elephant"), "Elephant token created");
}

#[test]
fn moldervine_cloak_pumps_and_has_dredge() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::moldervine_cloak());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura on bear");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 5, "+3/+3 from the Cloak");
    assert!(catalog::moldervine_cloak().keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Dredge(2))));
}
