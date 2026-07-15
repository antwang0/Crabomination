//! Functionality tests for `catalog::sets::decks::recent35` — blink/tempo/tutor
//! staples, the Spike counter engine, and the new skip-combat primitive.

use crabomination::card::{CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::mana::Color;
use crabomination::game::two_player_game;
use crabomination::game::*;

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn spike_weaver_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spike_weaver());
    // Simulate the enters-with-counters replacement.
    let n = catalog::spike_weaver().enters_with_counters.unwrap();
    if let Value::Const(c) = n.1 {
        g.battlefield_find_mut(id).unwrap().add_counters(n.0, c as u32);
    }
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "0/0 + three +1/+1 = 3/3");
}

#[test]
fn spike_weaver_moves_a_counter_to_target() {
    let mut g = two_player_game();
    let weaver = g.add_card_to_battlefield(0, catalog::spike_weaver());
    g.battlefield_find_mut(weaver).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, weaver, 0, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "target gained a +1/+1 counter");
    assert_eq!(
        g.battlefield_find(weaver).unwrap().counters
            .get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        2,
        "weaver spent one counter"
    );
}

#[test]
fn glimmerpoint_stag_blinks_a_permanent() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stag = g.add_card_to_battlefield(0, catalog::glimmerpoint_stag());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(foe))]));
    g.fire_self_etb_triggers(stag, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "target exiled by the blink");
    assert!(g.exile.iter().any(|c| c.id == foe), "sitting in exile until end step");
}

#[test]
fn weathered_wayfarer_only_works_when_behind() {
    let mut g = two_player_game();
    let way = g.add_card_to_battlefield(0, catalog::weathered_wayfarer());
    g.clear_sickness(way);
    g.add_card_to_library(0, catalog::plains());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // No opponent lands → activation illegal.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: way, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    });
    assert!(res.is_err(), "can't activate while not behind on lands");
}

#[test]
fn weathered_wayfarer_fetches_when_behind() {
    let mut g = two_player_game();
    let way = g.add_card_to_battlefield(0, catalog::weathered_wayfarer());
    g.clear_sickness(way);
    for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
    let land = g.add_card_to_library(0, catalog::plains());
    g.players[0].mana_pool.add(Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
    let hand_before = g.players[0].hand.len();
    activate(&mut g, way, 0, None);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "fetched a land to hand");
}

#[test]
fn plea_for_guidance_tutors_two_enchantments() {
    let mut g = two_player_game();
    let plea = g.add_card_to_hand(0, catalog::plea_for_guidance());
    let e1 = g.add_card_to_library(0, catalog::pacifism());
    let e2 = g.add_card_to_library(0, catalog::narcolepsy());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(e1)), DecisionAnswer::Search(Some(e2)),
    ]));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: plea, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Plea for Guidance");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == e1));
    assert!(g.players[0].hand.iter().any(|c| c.id == e2));
}

#[test]
fn fleetfoot_dancer_has_the_three_keywords() {
    let g = two_player_game();
    let _ = g;
    let kw = catalog::fleetfoot_dancer().keywords;
    assert!(kw.contains(&Keyword::Trample) && kw.contains(&Keyword::Lifelink) && kw.contains(&Keyword::Haste));
}

#[test]
fn stormscape_apprentice_taps_a_creature() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::stormscape_apprentice());
    g.clear_sickness(app);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    activate(&mut g, app, 0, Some(Target::Permanent(foe)));
    assert!(g.battlefield_find(foe).unwrap().tapped, "target creature tapped");
}

#[test]
fn stormscape_apprentice_drains_one() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let app = g.add_card_to_battlefield(0, catalog::stormscape_apprentice());
    g.clear_sickness(app);
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, app, 1, None);
    assert_eq!(g.players[1].life, 19, "opponent loses 1 life");
}

#[test]
fn stonecloaker_exiles_a_graveyard_card() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a creature to bounce
    let cloak = g.add_card_to_battlefield(0, catalog::stonecloaker());
    // Answer the bounce-own (first ETB) and the gy-exile (second ETB).
    let own = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").unwrap().id;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(own)),
        DecisionAnswer::Target(Target::Permanent(dead)),
    ]));
    g.fire_self_etb_triggers(cloak, 0);
    drain_stack(&mut g);
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == dead), "graveyard card exiled");
}

#[test]
fn stonehorn_dignitary_makes_opponent_skip_combat() {
    let mut g = two_player_game();
    let dig = g.add_card_to_battlefield(0, catalog::stonehorn_dignitary());
    g.fire_self_etb_triggers(dig, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].skip_next_combat, 1, "opponent will skip their next combat");
}

#[test]
fn skip_next_combat_jumps_over_the_combat_phase() {
    let mut g = two_player_game();
    // Player 0 is the active player; give them a skip charge.
    g.active_player_idx = 0;
    g.players[0].skip_next_combat = 1;
    g.step = TurnStep::PreCombatMain;
    let evs = g.advance_step(Vec::new()).expect("advance from precombat main");
    let _ = evs;
    assert_eq!(g.step, TurnStep::PostCombatMain, "combat phase skipped");
    assert_eq!(g.players[0].skip_next_combat, 0, "skip charge consumed");
}

#[test]
fn bile_blight_hits_all_same_name() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blight = g.add_card_to_hand(0, catalog::bile_blight());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: blight, target: Some(Target::Permanent(a)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Bile Blight");
    drain_stack(&mut g);
    // Both 2/2 Bears take -3/-3 and die.
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
        "both same-named creatures destroyed");
}
