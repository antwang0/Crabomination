//! Commander: two-card gaps of EDHREC ranks 2501-3500
//! (`decks::cmdr_rank3k_l`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
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
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn resolve_at(
    g: &mut GameState,
    effect: &Effect,
    source: CardId,
    target: Option<Target>,
    mode: usize,
) {
    let mut ctx = EffectContext::for_spell(0, target, mode, 0);
    ctx.source = Some(source);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
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
/// CR 702.80a — Witherscale Wurm gives its blocker wither, so its own damage
/// lands as -1/-1 counters; hitting an opponent clears them.
#[test]
fn witherscale_wurm_hands_out_wither() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 0, catalog::witherscale_wurm());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    declare(&mut g, vec![(wurm, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(angel, wurm)]))
        .expect("block");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(angel)
            .unwrap()
            .keywords()
            .contains(&Keyword::Wither)
    );
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(
        g.battlefield_find(wurm)
            .unwrap()
            .counter_count(CounterType::MinusOneMinusOne),
        4
    );
    let trig = catalog::witherscale_wurm().triggered_abilities[2]
        .effect
        .clone();
    resolve_at(&mut g, &trig, wurm, None, 0);
    assert_eq!(
        g.battlefield_find(wurm)
            .unwrap()
            .counter_count(CounterType::MinusOneMinusOne),
        0
    );
}

/// CR 606.3 — Vivien on the Hunt pods up, digs creatures, and makes Rhinos.
#[test]
fn vivien_on_the_hunt_loyalty_abilities() {
    let mut g = pod(2);
    let vivien = ready(&mut g, 0, catalog::vivien_on_the_hunt());
    let bear = ready(&mut g, 0, catalog::llanowar_elves());
    let target = g.add_card_to_library(0, catalog::grizzly_bears());
    let abilities = catalog::vivien_on_the_hunt().loyalty_abilities;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    resolve_at(&mut g, &abilities[0].effect, vivien, None, 0);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(target).is_some(), "mana value 1 + 1");
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let a = g.add_card_to_library(0, catalog::serra_angel());
    let b = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    resolve_at(&mut g, &abilities[1].effect, vivien, None, 0);
    assert_eq!(g.players[0].hand.len(), 2);
    resolve_at(&mut g, &abilities[2].effect, vivien, None, 0);
    assert_eq!(named(&g, 0, "Rhino Warrior"), 1);
}

/// CR 611.3a — Rona's exiled historic card is castable while she stays.
#[test]
fn rona_casts_an_exiled_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let rona = ready(&mut g, 0, catalog::rona_disciple_of_gix());
    let etb = catalog::rona_disciple_of_gix().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, rona, Some(Target::Permanent(ring)), 0);
    assert!(g.exile.iter().any(|c| c.id == ring));
    flood(&mut g);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: ring,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
}

/// CR 714.2 — The Antiquities War digs for artifacts and animates them.
#[test]
fn the_antiquities_war_animates_artifacts() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::sol_ring());
    let saga = g.add_card_to_hand(0, catalog::the_antiquities_war());
    cast(&mut g, saga, None);
    assert_eq!(g.players[0].hand.len(), 1);
    let ch3 = catalog::the_antiquities_war().saga_chapters[2].1.clone();
    resolve_at(&mut g, &ch3, saga, None, 0);
    assert_eq!(pt(&g, ring), (5, 5));
}
