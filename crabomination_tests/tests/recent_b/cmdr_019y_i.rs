//! Commander: the 2026-10-17 (`019Y6J4R`) ninth closer batch
//! (`decks::cmdr_019y_i`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn lore(g: &mut GameState, saga: CardId) {
    let add = crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::ExactObjects(vec![saga]),
        kind: CounterType::Lore,
        amount: crabomination::card::Value::ONE,
    };
    let evs = g.resolve_effect(&add, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// The Brothers' War: I makes two tapped Powerstones; III splits X damage
/// (X = artifacts you control) across two different targets.
#[test]
fn the_brothers_war_powers_up_then_burns_twice() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_brothers_war());
    lore(&mut g, saga);
    let stones: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Powerstone").collect();
    assert_eq!(stones.len(), 2);
    assert!(stones.iter().all(|c| c.tapped));
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 1);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[1].life;
    // Chapter III resolved with its two targets named (the auto-picker's
    // choice is the bot's business, ENGINE_BACKLOG `019Y6J4R`).
    let iii = catalog::the_brothers_war().saga_chapters[2].1.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(saga);
    ctx.targets = vec![Target::Player(1), Target::Permanent(bear)];
    let evs = g.resolve_effect(&iii, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let hits = (g.players[1].life == life - 2) as u32 + g.battlefield_find(bear).is_none() as u32;
    assert_eq!(hits, 2, "2 damage to the opponent and 2 to the Bears");
}

/// Overwhelming Encounter's 10-19 band puts two +1/+1 counters on each.
#[test]
fn overwhelming_encounter_rolls_counters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let oe = g.add_card_to_hand(0, catalog::overwhelming_encounter());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(15)]));
    cast(&mut g, oe, None).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Basri's −1: every creature that attacks this turn gets a +1/+1 counter.
#[test]
fn basri_counters_each_attacker() {
    let mut g = pod(2);
    let basri = ready(&mut g, 0, catalog::basri_devoted_paladin());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: basri, ability_index: 1, target: None, x_value: None })
        .expect("−1");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Hold for Ransom: the creature can't attack; its controller pays {7} to free
/// it, and the Aura's controller draws.
#[test]
fn hold_for_ransom_pays_out_to_the_aura_owner() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let hfr = g.add_card_to_hand(0, catalog::hold_for_ransom());
    flood(&mut g, 0);
    cast(&mut g, hfr, Some(Target::Permanent(bear))).expect("aura");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::CantAttack));
    let hand = g.players[0].hand.len();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    let ok = (0..4).any(|i| activate(&mut g, bear, i, None).is_ok());
    assert!(ok, "the granted ransom ability");
    assert!(g.battlefield_find(hfr).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Aura's controller drew");
}

/// Renowned Weaponsmith fetches Heart-Piercer Bow by name.
#[test]
fn renowned_weaponsmith_tutors_its_bow() {
    let mut g = pod(2);
    let rw = ready(&mut g, 0, catalog::renowned_weaponsmith());
    let bow = g.add_card_to_library(0, crabomination::card::CardDefinition {
        name: "Heart-Piercer Bow",
        card_types: vec![crabomination::card::CardType::Artifact],
        ..Default::default()
    });
    g.players[0].mana_pool.add(Color::Blue, 1);
    activate(&mut g, rw, 1, None).expect("search");
    assert!(g.players[0].hand.iter().any(|c| c.id == bow));
}

/// Lurking Green Dragon can't attack a player without a flyer.
#[test]
fn lurking_green_dragon_needs_a_flying_defender() {
    let mut g = pod(2);
    let d = ready(&mut g, 0, catalog::lurking_green_dragon());
    g.step = TurnStep::DeclareAttackers;
    let attack = GameAction::DeclareAttackers(vec![Attack { attacker: d, target: AttackTarget::Player(1) }]);
    assert!(g.clone().perform_action(attack.clone()).is_err());
    ready(&mut g, 1, catalog::serra_angel());
    g.perform_action(attack).expect("a flyer on the other side");
}

/// Herdchaser Dragon unmorphed counters the other Dragons.
#[test]
fn herdchaser_dragon_feeds_the_flight() {
    let mut g = pod(2);
    let other = ready(&mut g, 0, catalog::lurking_green_dragon());
    let hd = g.add_card_to_hand(0, catalog::herdchaser_dragon());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: hd }).expect("face down");
    drain_stack(&mut g);
    flood(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: hd }).expect("megamorph");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Spare Supplies draws entering and again when cashed in.
#[test]
fn spare_supplies_draws_twice() {
    let mut g = pod(2);
    let ss = g.add_card_to_hand(0, catalog::spare_supplies());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, ss, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert!(g.battlefield_find(ss).unwrap().tapped);
    g.battlefield_find_mut(ss).unwrap().tapped = false;
    activate(&mut g, ss, 0, None).expect("sac");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}
