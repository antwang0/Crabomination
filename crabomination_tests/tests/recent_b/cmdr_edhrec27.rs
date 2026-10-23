//! Commander: the 2026-10-13 rescan's two-card gaps, fourth pass
//! (`decks::cmdr_edhrec27`).

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
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Sarulf grows on an opponent's losses and cashes its counters in to exile
/// cheaper permanents at upkeep.
#[test]
fn sarulf_eats_and_purges() {
    let mut g = pod(2);
    let sarulf = ready(&mut g, 0, catalog::sarulf_realm_eater());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    for id in [bear, giant] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, 0, bolt, &[Target::Permanent(id)]).expect("bolt");
        g.check_state_based_actions();
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(sarulf).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none(), "mana value 1 ≤ 2");
    assert!(g.battlefield_find(wurm).is_some(), "mana value 6 > 2");
    assert!(g.battlefield_find(sarulf).is_some(), "not itself");
    assert_eq!(g.battlefield_find(sarulf).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Thunderous Debut unbargained puts up to two creatures into your hand.
#[test]
fn thunderous_debut_finds_two_creatures() {
    let mut g = pod(2);
    for def in [catalog::island, catalog::grizzly_bears, catalog::island, catalog::craw_wurm, catalog::island] {
        g.add_card_to_library(0, def());
    }
    let id = g.add_card_to_hand(0, catalog::thunderous_debut());
    cast(&mut g, 0, id, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2);
    assert!(g.players[0].hand.iter().all(|c| c.definition.is_creature()));
}

/// Arni Slays the Troll's last chapter gains life equal to your biggest power.
#[test]
fn arni_slays_the_troll_chapter_three() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::craw_wurm());
    let def = catalog::arni_slays_the_troll();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&def.saga_chapters[2].1, &ctx).expect("chapter III");
    assert_eq!(g.players[0].life, 26);
}

/// Hidetsugu's {2}{R} ability exiles a nonland card and burns for its mana
/// value; the {B} ability scries for a sacrifice.
#[test]
fn hidetsugu_exiles_and_burns() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::craw_wurm());
    let h = ready(&mut g, 0, catalog::hidetsugu_devouring_chaos());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: h,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let lost: i32 = (20 - g.players[1].life) + (20 - g.players[0].life);
    assert_eq!(lost, 6, "the Wurm's mana value, to some target");
    assert!(g.exile.iter().any(|c| c.definition.name == "Craw Wurm"));
}

/// Ravenloft Adventurer exiles an opponent's dying creature with a hit
/// counter — a replacement (CR 614.1a), so the creature never dies: Doomed
/// Traveler's dies trigger makes no Spirit.
#[test]
fn ravenloft_adventurer_marks_the_dead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ravenloft_adventurer());
    let traveler = ready(&mut g, 1, catalog::doomed_traveler());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, &[Target::Permanent(traveler)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    let card = g.exile.iter().find(|c| c.id == traveler).expect("exiled");
    assert_eq!(card.counter_count(CounterType::Hit), 1);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "it never died");
    assert!(g.players[1].graveyard.is_empty());
}

/// J. Jonah Jameson's suspect gives menace, and a menacing attacker makes a
/// Treasure.
#[test]
fn jjj_suspects_and_profits() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let jjj = g.add_card_to_hand(0, catalog::j_jonah_jameson());
    cast(&mut g, 0, jjj, &[Target::Permanent(bear)]).expect("cast");
    attack(&mut g, &[bear]);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
}

/// Acererak bounces itself and ventures until Tomb of Annihilation — that
/// dungeon, not any (CR 309.7) — is done; attacking, an opponent without
/// creatures hands you a Zombie.
#[test]
fn acererak_bounces_and_punishes() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::acererak_the_archlich());
    cast(&mut g, 0, id, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "back to hand");
    g.players[0].dungeons_completed = 1;
    g.players[0].completed_dungeons.push("Lost Mine of Phandelver".into());
    cast(&mut g, 0, id, &[]).expect("recast");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "another dungeon doesn't keep it");
    g.players[0].completed_dungeons.push("Tomb of Annihilation".into());
    cast(&mut g, 0, id, &[]).expect("third cast");
    assert!(g.battlefield.iter().any(|c| c.id == id), "the Tomb does");
    g.clear_sickness(id);
    attack(&mut g, &[id]);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(), 1);
}
