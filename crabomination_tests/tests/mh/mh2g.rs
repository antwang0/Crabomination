//! Functionality tests for `catalog::sets::decks::mh2g` — MH2 sweep batch 8.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// Chitterspitter's Squirrel anthem scales with its acorn counters.
#[test]
fn chitterspitter_acorn_anthem() {
    let mut g = two_player_game();
    let spitter = g.add_card_to_battlefield(0, catalog::chitterspitter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.clear_sickness(spitter);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spitter, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("token");
    drain_stack(&mut g);
    let squirrel = g.battlefield.iter().find(|c| c.definition.name == "Squirrel").unwrap().id;
    assert_eq!(g.computed_permanent(squirrel).unwrap().power, 1, "no acorns yet");
    g.battlefield_find_mut(spitter).unwrap().add_counters(CounterType::Acorn, 2);
    let cp = g.computed_permanent(squirrel).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 per acorn");
}

/// Chitterspitter's upkeep sacrifices a token for an acorn.
#[test]
fn chitterspitter_upkeep_sac_for_acorn() {
    let mut g = two_player_game();
    let spitter = g.add_card_to_battlefield(0, catalog::chitterspitter());
    g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(crabomination::game::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(spitter).unwrap().counter_count(CounterType::Acorn), 1);
    assert!(!g.battlefield.iter().any(|c| c.is_token), "token sacrificed");
}

/// Chrome Courier gains 3 only when an artifact goes to hand.
#[test]
fn chrome_courier_artifact_rider() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::parcel_myr()); // artifact on top
    g.add_card_to_library(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let courier = g.add_card_to_hand(0, catalog::chrome_courier());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, courier);
    // AutoDecider keeps the top revealed card (the artifact) → +3 life.
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Discerning Taste gains life equal to the greatest milled creature power.
#[test]
fn discerning_taste_greatest_power() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // top — kept
    g.add_card_to_library(0, catalog::grizzly_bears()); // 2/2 milled
    g.add_card_to_library(0, catalog::mahamoti_djinn()); // 5/6 milled
    g.add_card_to_library(0, catalog::island()); // milled, not a creature
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::discerning_taste().effect, &ctx).unwrap();
    assert_eq!(g.players[0].life, life + 5, "greatest milled power is the 5/6");
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// Break the Ice overloaded sweeps every colorless-producing land.
#[test]
fn break_the_ice_overload() {
    let mut g = two_player_game();
    let wastes1 = g.add_card_to_battlefield(1, catalog::power_depot());
    let island = g.add_card_to_battlefield(1, catalog::island());
    let d = catalog::break_the_ice();
    let over = d.alternative_cost.as_ref().unwrap().effect_override.clone().unwrap();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&over, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    // Power Depot is an artifact land that taps for {C} — destroyed.
    assert!(g.battlefield_find(wastes1).is_none(), "colorless land destroyed");
    assert!(g.battlefield_find(island).is_some(), "Island produces no {{C}}");
}

/// Obsidian Charmaw is discounted per opponent colorless land.
#[test]
fn obsidian_charmaw_discount() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::power_depot());
    g.add_card_to_battlefield(1, catalog::power_depot());
    let nonbasic = g.add_card_to_battlefield(1, catalog::power_depot());
    let charmaw = g.add_card_to_hand(0, catalog::obsidian_charmaw());
    g.players[0].mana_pool.add(Color::Red, 2);
    // {3}{R}{R} less {3} (three opponent {C} lands) = {R}{R}.
    g.priority.player_with_priority = 0;
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: charmaw,
        target: Some(Target::Permanent(nonbasic)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("discounted cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(charmaw).is_some());
    let depots = g.battlefield.iter().filter(|c| c.definition.name == "Power Depot").count();
    assert_eq!(depots, 2, "ETB destroyed one nonbasic");
}

/// CR 702.29 — echo: pay at upkeep (mana) or sacrifice.
#[test]
fn cr_702_29_echo_mana_pays_or_sacrifices() {
    let mut g = two_player_game();
    let slinger = g.add_card_to_battlefield(0, catalog::ghitu_slinger());
    g.battlefield_find_mut(slinger).unwrap().echo_paid = false;
    g.active_player_idx = 0;
    // Affordable: {2}{R} in pool → echo paid, stays.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(slinger).is_some(), "echo paid");
    assert!(g.battlefield_find(slinger).unwrap().echo_paid);
    // Second upkeep: no further echo owed.
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(slinger).is_some());
    // A fresh one with an empty pool dies.
    let broke = g.add_card_to_battlefield(0, catalog::ghitu_slinger());
    g.battlefield_find_mut(broke).unwrap().echo_paid = false;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(broke).is_none(), "unpaid echo sacrifices");
}

/// CR 702.29b — Echo—Discard a card (Rakdos Headliner).
#[test]
fn cr_702_29b_echo_discard() {
    let mut g = two_player_game();
    let devil = g.add_card_to_battlefield(0, catalog::rakdos_headliner());
    g.battlefield_find_mut(devil).unwrap().echo_paid = false;
    g.add_card_to_hand(0, catalog::island());
    g.active_player_idx = 0;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(devil).is_some(), "echo paid by discard");
    assert!(g.players[0].hand.is_empty(), "card discarded");
    // Next time, with an empty hand and echo unpaid, it dies.
    g.battlefield_find_mut(devil).unwrap().echo_paid = false;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(devil).is_none());
}

/// Wren's Run Hydra enters with X counters and reinforces for X.
#[test]
fn wrens_run_hydra_x_and_reinforce() {
    let mut g = two_player_game();
    let hydra = g.add_card_to_hand(0, catalog::wrens_run_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hydra, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast with X=3");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    // Reinforce X=2 from hand onto a bear.
    let second = g.add_card_to_hand(0, catalog::wrens_run_hydra());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: second, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: Some(2), mode: None,
    }).expect("reinforce");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == second), "discarded");
}

/// Ghost-Lit Drifter grants flying from the battlefield and via channel.
#[test]
fn ghost_lit_drifter_flying() {
    let mut g = two_player_game();
    let drifter = g.add_card_to_battlefield(0, catalog::ghost_lit_drifter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(drifter);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: drifter, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("grant flying");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Steel Dromedary stays tapped while countered, moves counters, then untaps.
#[test]
fn steel_dromedary_counter_lock() {
    let mut g = two_player_game();
    let camel = g.add_card_to_hand(0, catalog::steel_dromedary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, camel);
    let c = g.battlefield_find(camel).unwrap();
    assert!(c.tapped, "enters tapped");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
    // Untap step: still locked by its counters.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(camel).unwrap().tapped, "counter lock holds");
    // Move both counters onto the bear via the combat trigger.
    for _ in 0..2 {
        let eff = crabomination::effect::Effect::MoveCounters {
            from: crabomination::effect::Selector::This,
            to: crabomination::effect::Selector::Target(0),
            counter: CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::ONE,
        };
        let ctx = crabomination::game::effects::EffectContext::for_trigger(
            camel, 0, Some(Target::Permanent(bear)), 0);
        g.resolve_effect(&eff, &ctx).unwrap();
    }
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(camel).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    g.do_untap();
    assert!(!g.battlefield_find(camel).unwrap().tapped, "untaps once counterless");
}

/// CR 702.62e — Suspend exiles a creature that then returns via suspend.
#[test]
fn cr_702_62e_suspend_grants_suspend() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = {
        let mut c = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        c.targets = vec![Target::Permanent(bear)];
        c
    };
    g.resolve_effect(&catalog::suspend().effect, &ctx).unwrap();
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    assert!(exiled.granted_suspend);
    assert_eq!(exiled.counter_count(CounterType::Time), 2);
    // Two of the owner's upkeeps later it free-casts back.
    g.active_player_idx = 1;
    for _ in 0..2 {
        let _ = g.process_suspend();
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "cast back for free");
    let kws = g.computed_permanent(bear).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Haste), "suspend grants haste");
}

/// Yusri draws per won flip and burns per lost flip.
#[test]
fn yusri_flip_mix() {
    let mut g = two_player_game();
    let yusri = g.add_card_to_battlefield(0, catalog::yusri_fortunes_flame());
    g.add_card_to_library(0, catalog::island());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    // Choose 2 flips: one win, one loss.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(2),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    let ctx = crabomination::game::effects::EffectContext::for_trigger(yusri, 0, None, 0);
    let eff = catalog::yusri_fortunes_flame().triggered_abilities[0].effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw");
    assert_eq!(g.players[0].life, life - 2, "one 2-damage loss");
    assert!(!g.players[0].free_spells_from_hand_this_turn);
}

/// A `wants_ui` Yusri controller is prompted for the flip count instead of
/// riding the synchronous decider.
#[test]
fn yusri_wants_ui_prompts_flip_count() {
    use crabomination::decision::Decision;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let yusri = g.add_card_to_battlefield(0, catalog::yusri_fortunes_flame());
    g.add_card_to_library(0, catalog::island());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    let eff = catalog::yusri_fortunes_flame().triggered_abilities[0].effect.clone();
    g.push_pending_trigger(
        crabomination::game::PendingTriggerPush {
            from_mana_ability: false,
            actor: None,
            source: yusri, controller: 0, effect: eff,
            subject: None, event_amount: 0, mode: None, intervening_if: None,
        },
        None,
    );
    drain_stack(&mut g);
    let pd = g.pending_decision.as_ref().expect("ChooseAmount suspends");
    assert!(matches!(pd.decision, Decision::ChooseAmount { .. }));
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Amount(1))).unwrap();
    drain_stack(&mut g);
    let drew = g.players[0].hand.len() == hand + 1;
    let burned = g.players[0].life == life - 2;
    assert!(drew ^ burned, "exactly one flip resolved (win draws, loss burns)");
}

/// Yusri's five-win jackpot: spells from hand are free this turn.
#[test]
fn yusri_jackpot_free_spells() {
    let mut g = two_player_game();
    let yusri = g.add_card_to_battlefield(0, catalog::yusri_fortunes_flame());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let mut answers = vec![DecisionAnswer::Amount(5)];
    answers.extend(std::iter::repeat_n(DecisionAnswer::Bool(true), 5));
    g.decider = Box::new(ScriptedDecider::new(answers));
    let ctx = crabomination::game::effects::EffectContext::for_trigger(yusri, 0, None, 0);
    let eff = catalog::yusri_fortunes_flame().triggered_abilities[0].effect.clone();
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.players[0].free_spells_from_hand_this_turn);
    // A big spell now costs nothing.
    let djinn = g.add_card_to_hand(0, catalog::mahamoti_djinn());
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, djinn);
    assert!(g.battlefield_find(djinn).is_some(), "cast for free");
}

/// Aeve storms into nonlegendary token copies with per-Ooze counters.
#[test]
fn aeve_storm_tokens() {
    let mut g = two_player_game();
    // Two spells cast before Aeve this turn.
    g.spells_cast_this_turn = 2;
    let aeve = g.add_card_to_hand(0, catalog::aeve_progenitor_ooze());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(2);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, aeve);
    drain_stack(&mut g);
    let aeves: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Aeve, Progenitor Ooze")
        .collect();
    assert_eq!(aeves.len(), 3, "original + 2 storm copies survive the legend rule");
    // The last-resolving (original) Aeve saw the two token copies enter first.
    let original = g.battlefield_find(aeve).unwrap();
    assert_eq!(original.counter_count(CounterType::PlusOnePlusOne), 2, "one per other Ooze");
}
