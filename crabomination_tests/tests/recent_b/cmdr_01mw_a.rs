//! Commander: the 2026-10-18 (`01MWhJbe`) first closer batch
//! (`decks::cmdr_01mw_a`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

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

fn cast(g: &mut GameState, card_id: CardId) {
    g.perform_action(GameAction::CastSpell { card_id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
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

/// Origin of Iron Man: chapter I's tap is locked to the Saga; chapter III
/// deploys an artifact of mana value 5 or less from hand and leaves a bigger
/// one behind.
#[test]
fn origin_of_iron_man_locks_then_deploys_a_small_artifact() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::origin_of_iron_man());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let i = catalog::origin_of_iron_man().saga_chapters[0].1.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(saga);
    ctx.targets = vec![Target::Permanent(bear)];
    let evs = g.resolve_effect(&i, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped);
    assert_eq!(b.untap_locked_while_present, Some(saga));
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let colossus = g.add_card_to_hand(0, catalog::darksteel_colossus());
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    lore(&mut g, saga);
    assert!(g.battlefield_find(stone).is_some(), "MV 2 artifact deployed");
    assert!(g.players[0].hand.iter().any(|c| c.id == colossus), "MV 11 stays in hand");
}

/// Odric, Blood-Cursed counts each listed keyword once: Serra Angel (flying,
/// vigilance) and Goblin Guide (haste) beside Odric make three Blood.
#[test]
fn odric_blood_cursed_counts_distinct_keywords() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 0, catalog::goblin_guide());
    let odric = g.add_card_to_hand(0, catalog::odric_blood_cursed());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, odric);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Blood").count(), 3);
}

/// Kellan, Daring Traveler: attacking takes a small creature off the top;
/// Journey On counts the opponents who control an artifact, at three seats.
#[test]
fn kellan_reveals_on_attack_and_maps_per_artifact_opponent() {
    let mut g = pod(3);
    let kellan = ready(&mut g, 0, catalog::kellan_daring_traveler());
    let bear = g.next_id();
    g.players[0].add_to_library_top(bear, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kellan, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));

    let mut g = pod(3);
    ready(&mut g, 1, catalog::sol_ring());
    let k = g.add_card_to_hand(0, catalog::kellan_daring_traveler());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.perform_action(GameAction::CastAdventure { card_id: k, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Journey On");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Map").count(), 2);
}

/// Invasion of Regatha's ETB splits 4 to an opponent and 1 to a creature;
/// CR 614.5 — Disciples of the Inferno adds 2 to a noncreature source's damage
/// to an opponent or any creature, and nothing to a creature source's.
#[test]
fn invasion_of_regatha_burns_and_its_back_adds_two() {
    let mut g = pod(3);
    let inv = ready(&mut g, 0, catalog::invasion_of_regatha());
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    let life = g.players[1].life;
    let etb = catalog::invasion_of_regatha().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(inv);
    ctx.targets = vec![Target::Player(1), Target::Permanent(bear)];
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);

    let mut g = pod(2);
    let disciples = ready(&mut g, 0, *catalog::invasion_of_regatha().back_face.unwrap());
    let own = ready(&mut g, 0, catalog::grizzly_bears());
    use crabomination::game::effects::EntityRef;
    let rock = ready(&mut g, 0, catalog::sol_ring());
    assert_eq!(g.scale_damage_to(Some(rock), EntityRef::Player(1), 1), 3, "noncreature source → opponent");
    assert_eq!(g.scale_damage_to(Some(rock), EntityRef::Permanent(own), 1), 3, "any creature, yours too");
    assert_eq!(g.scale_damage_to(Some(rock), EntityRef::Player(0), 1), 1, "not to you");
    assert_eq!(g.scale_damage_to(Some(disciples), EntityRef::Player(1), 4), 4, "a creature source");
}
