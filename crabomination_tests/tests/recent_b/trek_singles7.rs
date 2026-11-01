//! Star Trek: the seventh batch of EDHREC closers (`decks::trek_singles7`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn to_end_step(g: &mut GameState) {
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

fn lands(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count()
}

/// Erica Ortegas: attacking taps her, and the land she puts from hand stays
/// until the end step, when a land you control goes back to hand.
#[test]
fn erica_drops_a_land_and_returns_one_at_end_step() {
    let mut g = pod(2);
    let erica = ready(&mut g, 0, catalog::erica_ortegas_plucky_pilot());
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: erica, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(lands(&g, 0), 1, "the Forest dropped");
    to_end_step(&mut g);
    assert_eq!(lands(&g, 0), 0, "a land went back");
}

/// Will Riker: his combat trigger reveals until two lands, which enter and
/// are sacrificed at your end step; the nonlands go to the bottom.
#[test]
fn will_riker_borrows_two_lands_until_end_step() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::will_riker_first_in_the_field());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].library.rotate_right(1);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(lands(&g, 0), 2);
    assert_eq!(g.players[0].library.last().map(|c| c.definition.name), Some("Grizzly Bears"));
    to_end_step(&mut g);
    assert_eq!(lands(&g, 0), 0, "sacrificed at the end step");
}

/// Armus, Skin of Evil: your end step makes the target opponent discard two;
/// from hand, {2}{B} and discarding Armus gives -4/-4.
#[test]
fn armus_discards_and_channels_from_hand() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::armus_skin_of_evil());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    to_end_step(&mut g);
    assert_eq!(g.players[1].hand.len(), 1);
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let armus = g.add_card_to_hand(0, catalog::armus_skin_of_evil());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: armus,
        ability_index: 0,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("channel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "3/3 at -4/-4");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == armus));
}

/// Hela's Arcanum: three points destroy each non-Zombie creature with power
/// 3 or less; a 4-power creature survives.
#[test]
fn helas_arcanum_wipes_small_non_zombies() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::air_elemental());
    let spell = g.add_card_to_hand(0, catalog::helas_arcanum());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: Some(2), x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(big).is_some());
}

/// Dr. McCoy: the second mode watches another creature; when it dies this
/// turn, it returns at the next end step with a +1/+1 counter.
#[test]
fn mccoy_brings_a_creature_back_at_end_step() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let doc = g.add_card_to_hand(0, catalog::dr_leonard_bones_mccoy());
    flood(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Mode(1)]));
    g.perform_action(GameAction::CastSpell { card_id: doc, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let ctx = EffectContext::for_spell(1, Some(Target::Permanent(bear)), 0, 0);
    let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
    let ev = g.resolve_effect(&kill, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    to_end_step(&mut g);
    let back = g.battlefield_find(bear).expect("returned");
    assert_eq!(back.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}
