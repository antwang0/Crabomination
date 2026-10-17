//! Commander: the 2026-10-17 (`019Y6J4R`) sixth closer batch
//! (`decks::cmdr_019y_f`).

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
    cast_mode(g, id, target, None)
}

fn cast_mode(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })?;
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



/// Conch Horn draws two and puts one back on top.
#[test]
fn conch_horn_draws_two_puts_one_back() {
    let mut g = pod(2);
    let ch = ready(&mut g, 0, catalog::conch_horn());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    let lib = g.players[0].library.len();
    activate(&mut g, ch, 0, None).expect("activate");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].library.len(), lib - 1);
    assert!(g.battlefield_find(ch).is_none(), "sacrificed");
}

/// Lunar Force counters an opponent's spell and goes away.
#[test]
fn lunar_force_counters_the_next_opposing_spell() {
    let mut g = pod(2);
    let lf = ready(&mut g, 0, catalog::lunar_force());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    let life = g.players[0].life;
    cast(&mut g, bolt, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.players[0].life, life, "countered");
    assert!(g.battlefield_find(lf).is_none());
}

/// Korvold and the Noble Thief III lets you play three of an opponent's cards.
#[test]
fn korvold_and_the_noble_thief_steals_three() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::korvold_and_the_noble_thief());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let add = crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::ExactObjects(vec![saga]),
        kind: CounterType::Lore,
        amount: crabomination::card::Value::ONE,
    };
    let evs = g.resolve_effect(&add, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let playable = g.exile.iter().filter(|c| c.owner == 1 && c.may_play_until.is_some_and(|m| m.player == 0)).count();
    assert_eq!(playable, 3);
}

/// Thorin, Company's Leader: a Dwarf connecting makes two Treasures.
#[test]
fn thorin_companys_leader_pays_dwarves_in_treasure() {
    let mut g = pod(2);
    let th = ready(&mut g, 0, catalog::thorin_companys_leader());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: th, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let t = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    assert_eq!(t, 2);
}

/// Brain in a Jar casts a one-drop instant free at one counter, not a two-drop.
#[test]
fn brain_in_a_jar_casts_exactly_its_charge_count() {
    let mut g = pod(2);
    let bj = ready(&mut g, 0, catalog::brain_in_a_jar());
    g.add_card_to_hand(0, catalog::divination());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    activate(&mut g, bj, 0, None).expect("activate");
    assert!(!g.players[0].hand.iter().any(|c| c.id == bolt), "Bolt was cast");
    assert_eq!(g.players[1].life, life - 3);
    assert_eq!(g.players[0].hand.len(), 1, "Divination (MV 2) stayed");
}

/// Stockman loots entering.
#[test]
fn stockman_loots_on_entry() {
    let mut g = pod(2);
    g.add_card_to_hand(0, catalog::forest());
    let s = g.add_card_to_hand(0, catalog::stockman_mad_fly_entist());
    flood(&mut g, 0);
    let gy = g.players[0].graveyard.len();
    cast(&mut g, s, None).expect("cast");
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded one");
}
