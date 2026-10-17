//! Commander: the 2026-10-17 (`3chree`) second closer batch
//! (`decks::cmdr_oct17_r`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

/// Chronomantic Escape: no creature may attack its caster until their next
/// turn (another opponent still can be attacked), and it suspends itself
/// again with three time counters (CR 702.62a).
#[test]
fn chronomantic_escape_shields_its_caster() {
    let mut g = pod(3);
    let esc = g.add_card_to_hand(0, catalog::chronomantic_escape());
    flood(&mut g, 0);
    cast(&mut g, esc, None).expect("escape");
    let card = g.exile.iter().find(|c| c.id == esc).expect("exiled");
    assert_eq!(card.counter_count(CounterType::Time), 3);
    step_to(&mut g, 1, TurnStep::DeclareAttackers);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.priority.player_with_priority = 1;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(p) }]);
    assert!(g.perform_action(at(0)).is_err(), "can't attack the caster");
    g.perform_action(at(2)).expect("another opponent is fair game");
}

/// Aurora Awakener: green alone is one color, so one permanent card comes in.
#[test]
fn aurora_awakener_reveals_per_color() {
    let mut g = pod(2);
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::aurora_awakener());
    flood(&mut g, 0);
    cast(&mut g, a, None).expect("awakener");
    assert!(g.battlefield_find(id).is_some());
}

/// Back for More: the returned creature fights the target (CR 701.14), and
/// with no enemy creature the reflexive fight is just declined.
#[test]
fn back_for_more_returns_then_fights() {
    let mut g = pod(2);
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::back_for_more());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(giant))).expect("back for more");
    assert!(g.battlefield_find(giant).is_some());
    assert!(g.battlefield_find(bear).is_none(), "the 3/3 fights the 2/2");
    let mut g = pod(2);
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::back_for_more());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(giant))).expect("no enemy creature");
    assert!(g.battlefield_find(giant).is_some());
}

/// Sword of Once and Future: a hit recasts a cheap graveyard sorcery free,
/// and that spell is exiled afterwards.
#[test]
fn sword_of_once_and_future_recasts_and_exiles() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = ready(&mut g, 0, catalog::sword_of_once_and_future());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let sign = g.add_card_to_graveyard(0, catalog::preordain());
    let (hand, life) = (g.players[0].hand.len(), g.players[1].life);
    step_to(&mut g, 0, TurnStep::DeclareAttackers);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    step_to(&mut g, 0, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 4);
    assert!(g.exile.iter().any(|c| c.id == sign), "cast free, then exiled");
    assert_eq!(g.players[0].hand.len(), hand + 1, "Preordain drew one");
}

/// Gnarlroot Trapper's mana only pays for an Elf creature spell, for 1 life.
#[test]
fn gnarlroot_trapper_mana_is_elf_only() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::gnarlroot_trapper());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: t, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    assert!(cast(&mut g, bear, None).is_err(), "not an Elf");
    cast(&mut g, elf, None).expect("an Elf");
}

/// Rowdy Crew: two discarded lands share a type — two +1/+1 counters.
#[test]
fn rowdy_crew_counters_on_a_matched_discard() {
    let mut g = pod(2);
    let crew = g.add_card_to_hand(0, catalog::rowdy_crew());
    flood(&mut g, 0);
    cast(&mut g, crew, None).expect("crew");
    let c = g.battlefield_find(crew).expect("crew");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "a library of Forests discards two lands");
}

/// Complete the Circuit: a sorcery cast at instant speed this turn is copied
/// twice.
#[test]
fn complete_the_circuit_flashes_and_triples_a_sorcery() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::complete_the_circuit());
    flood(&mut g, 0);
    cast(&mut g, c, None).expect("circuit");
    step_to(&mut g, 0, TurnStep::BeginCombat);
    let s = g.add_card_to_hand(0, catalog::divination());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    cast(&mut g, s, None).expect("a sorcery with flash");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "three Divinations");
}

/// Starlight Spectacular: three creatures in a line get +0, +1, +2.
#[test]
fn starlight_spectacular_parades() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::starlight_spectacular());
    let ids: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    step_to(&mut g, 0, TurnStep::DeclareAttackers);
    let mut powers: Vec<i32> = ids.iter().map(|id| g.computed_permanent(*id).unwrap().power).collect();
    powers.sort();
    assert_eq!(powers, vec![2, 3, 4]);
}

/// Soul Partition: the owner recasts the exiled creature from exile, paying
/// {2} more when they are your opponent.
#[test]
fn soul_partition_taxes_an_opponent_recast() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::soul_partition());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(bear))).expect("partition");
    assert!(g.exile.iter().any(|c| c.id == bear));
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    let recast = || GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(recast()).is_err(), "three mana is a {{2}} short");
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(recast()).expect("four mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}
