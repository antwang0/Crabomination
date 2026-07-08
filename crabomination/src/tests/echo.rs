//! Functionality tests for `catalog::sets::decks::echo` — echo classics.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Avalanche Riders hastes in, kills a land, then dies to unpaid echo.
#[test]
fn avalanche_riders_land_kill_then_echo() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let riders = g.add_card_to_hand(0, catalog::avalanche_riders());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: riders, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "ETB destroyed the land");
    // Upkeep with no mana sources: echo unpaid → sacrificed.
    g.active_player_idx = 0;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(riders).is_none(), "echo sacrifice");
}

/// Deranged Hermit mints four Squirrels that its own anthem pumps to 2/2,
/// and echo auto-taps lands to keep it.
#[test]
fn deranged_hermit_squirrels_and_echo_autotap() {
    let mut g = two_player_game();
    let hermit = g.add_card_to_battlefield(0, catalog::deranged_hermit());
    g.fire_self_etb_triggers(hermit, 0);
    drain_stack(&mut g);
    let squirrels: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Squirrel")
        .map(|c| c.id)
        .collect();
    assert_eq!(squirrels.len(), 4);
    let cp = g.computed_permanent(squirrels[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "global Squirrel anthem");
    // Echo auto-taps five forests instead of sacrificing.
    g.battlefield_find_mut(hermit).unwrap().echo_paid = false;
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.active_player_idx = 0;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(hermit).is_some(), "echo paid by auto-tap");
    let tapped = g.battlefield.iter().filter(|c| c.definition.is_land() && c.tapped).count();
    assert_eq!(tapped, 5, "lands tapped for the echo payment");
}

/// Keldon Vandals' ETB smashes an artifact.
#[test]
fn keldon_vandals_artifact_kill() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(1, catalog::chrome_mox());
    let vandals = g.add_card_to_battlefield(0, catalog::keldon_vandals());
    let mut ctx = crate::game::effects::EffectContext::for_trigger(vandals, 0, Some(Target::Permanent(mox)), 0);
    ctx.targets = vec![Target::Permanent(mox)];
    let eff = catalog::keldon_vandals().triggered_abilities[0].effect.clone();
    let events = g.resolve_effect(&eff, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(mox).is_none());
}

/// Great Whale untaps up to seven of your lands.
#[test]
fn great_whale_untaps_lands() {
    let mut g = two_player_game();
    for _ in 0..8 {
        let id = g.add_card_to_battlefield(0, catalog::island());
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let whale = g.add_card_to_battlefield(0, catalog::great_whale());
    g.fire_self_etb_triggers(whale, 0);
    drain_stack(&mut g);
    let untapped = g.battlefield.iter().filter(|c| c.definition.is_land() && !c.tapped).count();
    assert_eq!(untapped, 7, "up to seven");
}

/// Ticking Gnomes sacs for a ping; Radiant's Dragoons gains 5.
#[test]
fn ticking_gnomes_and_dragoons() {
    let mut g = two_player_game();
    let gnomes = g.add_card_to_battlefield(0, catalog::ticking_gnomes());
    g.clear_sickness(gnomes);
    let life1 = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gnomes, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("sac ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1);
    assert!(g.battlefield_find(gnomes).is_none(), "sacrificed as cost");
    let dragoons = g.add_card_to_battlefield(0, catalog::radiants_dragoons());
    let life0 = g.players[0].life;
    g.fire_self_etb_triggers(dragoons, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 5);
}

/// CR 310.7 — Portent Tracker tilts a battle's defense counters both ways.
#[test]
fn cr_310_7_portent_tracker_battle_defense() {
    let mut g = two_player_game();
    let tracker = g.add_card_to_battlefield(0, catalog::portent_tracker());
    g.clear_sickness(tracker);
    // An opponent-protected battle loses a counter.
    let siege = g.add_card_to_battlefield(0, catalog::invasion_of_zendikar());
    let base = {
        let c = g.battlefield_find_mut(siege).unwrap();
        c.protected_by = Some(1);
        c.counter_count(crate::card::CounterType::Defense)
    };
    g.step = crate::game::TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tracker, ability_index: 1, target: Some(Target::Permanent(siege)),
        additional_targets: vec![], x_value: None,
    }).expect("tilt");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(siege).unwrap().counter_count(crate::card::CounterType::Defense),
        base.saturating_sub(1),
        "opponent-protected battle loses a defense counter"
    );
}

/// CR 702.29b — a stolen echo permanent owes echo to its new controller,
/// even if the previous controller already paid.
#[test]
fn cr_702_29b_stolen_echo_owed_by_new_controller() {
    use crate::effect::{Duration, Effect, Selector};
    let mut g = two_player_game();
    let riders = g.add_card_to_battlefield(0, catalog::avalanche_riders());
    g.battlefield_find_mut(riders).unwrap().echo_paid = true;
    let ctx = crate::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(riders)), 0, 0);
    g.resolve_effect(
        &Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::Permanent },
        &ctx,
    )
    .unwrap();
    assert!(!g.battlefield_find(riders).unwrap().echo_paid, "control change re-arms echo");
    // New controller's upkeep with no mana: sacrificed.
    g.active_player_idx = 1;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    assert!(g.battlefield_find(riders).is_none(), "new controller owed echo");
}

/// A `wants_ui` controller gets an echo pay prompt; yes auto-taps and keeps.
#[test]
fn echo_wants_ui_prompt_pay_keeps_permanent() {
    use crate::decision::{Decision, DecisionAnswer};
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let riders = g.add_card_to_battlefield(0, catalog::avalanche_riders());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.active_player_idx = 0;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let pd = g.pending_decision.as_ref().expect("echo prompt suspends");
    assert!(matches!(pd.decision, Decision::OptionalTrigger { .. }));
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true))).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(riders).unwrap().echo_paid, "paid via prompt");
    let tapped = g.battlefield.iter().filter(|c| c.definition.is_land() && c.tapped).count();
    assert_eq!(tapped, 4, "echo payment auto-tapped");
}

/// A `wants_ui` controller declining the echo prompt sacrifices.
#[test]
fn echo_wants_ui_prompt_decline_sacrifices() {
    use crate::decision::DecisionAnswer;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let riders = g.add_card_to_battlefield(0, catalog::avalanche_riders());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.active_player_idx = 0;
    let events = g.process_echo();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.pending_decision.is_some(), "echo prompt suspends");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(false))).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(riders).is_none(), "declined echo → sacrificed");
}

/// Treetop Village animates into a 3/3 trampling Ape.
#[test]
fn treetop_village_animates() {
    let mut g = two_player_game();
    let village = g.add_card_to_battlefield(0, catalog::treetop_village());
    g.battlefield_find_mut(village).unwrap().tapped = false;
    g.clear_sickness(village);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: village, ability_index: 1, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(village).unwrap();
    assert!(cp.card_types.contains(&crate::card::CardType::Creature));
    assert!(cp.card_types.contains(&crate::card::CardType::Land), "still a land");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&crate::card::Keyword::Trample));
}
