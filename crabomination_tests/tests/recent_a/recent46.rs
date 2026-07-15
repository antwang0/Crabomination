//! Functionality tests for `catalog::sets::decks::recent46` — green value.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

#[test]
fn greenwarden_etb_returns_a_graveyard_card() {
    let mut g = two_player_game();
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let gw = g.add_card_to_battlefield(0, catalog::greenwarden_of_murasa());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_self_etb_triggers(gw, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gy), "gy card returned to hand");
}

#[test]
fn greenwarden_dies_exiles_self_and_returns_a_card() {
    let mut g = two_player_game();
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let gw = g.add_card_to_battlefield(0, catalog::greenwarden_of_murasa());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut evs = g.remove_to_graveyard_with_triggers(gw);
    evs.push(GameEvent::CreatureDied { card_id: gw });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gy), "other gy card returned");
    assert!(g.exile.iter().any(|c| c.id == gw), "Greenwarden exiled itself");
}

#[test]
fn nantuko_vigilante_face_up_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(art)), 0, 0);
    let trig = &catalog::nantuko_vigilante().triggered_abilities[0].effect;
    g.resolve_effect(trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed on turn face up");
}

#[test]
fn bramble_sovereign_copies_an_entering_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bramble_sovereign());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    // {1}{G} for the bear + {1}{G} for the copy.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "original plus a token copy");
}

#[test]
fn verdurous_gearhulk_distributes_four_counters() {
    let mut g = two_player_game();
    let vg = g.add_card_to_battlefield(0, catalog::verdurous_gearhulk());
    g.fire_self_etb_triggers(vg, 0);
    drain_stack(&mut g);
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0))
        .sum();
    assert_eq!(total, 4, "four +1/+1 counters distributed");
}

#[test]
fn pathbreaker_ibex_pumps_team_by_greatest_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pathbreaker_ibex()); // 3/3
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let trig = &catalog::pathbreaker_ibex().triggered_abilities[0].effect;
    g.resolve_effect(trig, &ctx).unwrap();
    drain_stack(&mut g);
    // Greatest power is 3 (the Ibex), so the bear becomes 5/5 with trample.
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!(b.power, 5, "bear pumped +3/+3");
    assert!(b.keywords.contains(&crabomination::card::Keyword::Trample), "bear gained trample");
}

#[test]
fn ghalta_costs_less_per_total_power() {
    let mut g = two_player_game();
    // Two 5-power bodies → total power 10 → {10}{G}{G} becomes {G}{G}.
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 power
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 power
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power → total 10
    let ghalta = g.add_card_to_hand(0, catalog::ghalta_primal_hunger());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: ghalta, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ghalta castable for {G}{G} with 10 total power");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ghalta, Primal Hunger"));
}

#[test]
fn lifecrafters_bestiary_draws_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lifecrafters_bestiary());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    // {1}{G} for the bear + {G} for the draw.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let lib0 = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib0 - 1, "drew a card off the creature cast");
}
