//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_g`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: x,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn has(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&k)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}


fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)), None);
}

/// Ultron Drone's power-up grows it and makes a Robot Villain, once.
#[test]
fn ultron_drone_powers_up_once() {
    let mut g = pod(2);
    let drone = ready(&mut g, 0, catalog::ultron_drone());
    activate(&mut g, drone, 0, None);
    assert_eq!(plus(&g, drone), 2);
    assert_eq!(named(&g, 0, "Robot Villain"), 1);
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: drone,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "power-up activates only once"
    );
}

/// Aang gives your other creatures vigilance; Katara counters the rest.
#[test]
fn aang_and_katara_lift_the_team() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::aang_air_nomad());
    assert!(has(&g, bear, Keyword::Vigilance));
    let k = g.add_card_to_hand(0, catalog::katara_heroic_healer());
    cast(&mut g, k, None, None);
    assert_eq!(plus(&g, bear), 1);
    assert_eq!(plus(&g, k), 0);
}

/// Winter pumps the team on a noncreature spell; Goblin Wizardry is two
/// prowess Wizards.
#[test]
fn winter_and_goblin_wizardry() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::winter_team_player());
    let gw = g.add_card_to_hand(0, catalog::goblin_wizardry());
    cast(&mut g, gw, None, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    assert_eq!(named(&g, 0, "Goblin Wizard"), 2);
}

/// Yuriko gives a lone attacker double strike.
#[test]
fn yuriko_doubles_a_lone_attacker() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::yuriko_blade_of_the_mighty());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(bear, 2)]);
    assert!(has(&g, bear, Keyword::DoubleStrike));
}

/// What Must Be Done's second mode returns a historic creature with two
/// extra counters.
#[test]
fn what_must_be_done_releases_a_legend() {
    let mut g = pod(2);
    let legend = ready(&mut g, 0, catalog::big_bertha());
    kill(&mut g, legend);
    let w = g.add_card_to_hand(0, catalog::what_must_be_done());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: w,
        target: Some(Target::Permanent(legend)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast mode 2");
    drain_stack(&mut g);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Big Bertha")
        .expect("returned");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Kumena's Awakening: everyone draws until you have the city's blessing.
#[test]
fn kumenas_awakening_feeds_everyone() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::kumenas_awakening());
    for seat in 0..3 {
        g.add_card_to_library(seat, catalog::island());
    }
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(
        (0..3).map(|s| g.players[s].hand.len()).collect::<Vec<_>>(),
        vec![1, 1, 1]
    );
}

/// Quakebringer burns each opponent at your upkeep and blanks their life gain.
#[test]
fn quakebringer_quakes_and_blocks_life_gain() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::quakebringer());
    step(&mut g, TurnStep::Upkeep);
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(
        &crabomination::effect::Effect::GainLife {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(5),
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(g.players[1].life, 18);
}

/// Bilbo returns a cheap permanent card from a graveyard when he connects.
#[test]
fn bilbo_brings_back_a_cheap_permanent() {
    let mut g = pod(2);
    let bilbo = ready(&mut g, 0, catalog::bilbo_unexpected_adventurer());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, shatter, Some(Target::Permanent(ring)), None);
    declare(&mut g, vec![(bilbo, 1)]);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Sol Ring")
        .expect("back");
    assert_eq!(back.controller, 1, "under its owner's control");
}

/// CR 506.1 — Yuriko locks spells (a flash creature included) during combat.
#[test]
fn yuriko_locks_spells_during_combat() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::yuriko_blade_of_the_mighty());
    g.step = TurnStep::DeclareAttackers;
    let viper = g.add_card_to_hand(0, catalog::ambush_viper());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    let cast = GameAction::CastSpell {
        card_id: viper,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(cast.clone()).is_err(), "no flash creature during combat");
    g.step = TurnStep::End;
    assert!(g.perform_action(cast).is_ok(), "outside combat it resolves as usual");
}
