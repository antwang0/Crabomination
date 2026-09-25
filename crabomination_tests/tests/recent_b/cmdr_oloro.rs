//! Commander: the Eternal Bargain precon (C13, Oloro, `decks::cmdr_oloro`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn controller(g: &GameState, id: CardId) -> usize {
    g.battlefield_find(id).expect("on the battlefield").controller
}

/// Order of Succession at three seats: every player ends up with the next
/// player's best creature; the caster turns the circle the way that nets it
/// the most (CR 101.4 seating: left is the next seat).
#[test]
fn order_of_succession_passes_creatures_around_the_table() {
    let mut g = main_phase(3);
    let mine = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let left = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let right = g.add_card_to_battlefield(2, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::order_of_succession());
    cast_at(&mut g, spell, &[]).expect("cast");
    // Right: seat 0 takes seat 2's Craw Wurm, seat 2 takes seat 1's Bears,
    // seat 1 takes seat 0's Elves — the caster's best trade.
    assert_eq!(controller(&g, right), 0);
    assert_eq!(controller(&g, left), 2);
    assert_eq!(controller(&g, mine), 1);
}

/// Serene Master (CR 701.10g): blocking a big attacker swaps the two powers
/// until end of combat, so the attacker deals nothing.
#[test]
fn serene_master_swaps_powers_with_what_it_blocks() {
    let mut g = main_phase(2);
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let master = g.add_card_to_battlefield(1, catalog::serene_master());
    g.clear_sickness(wurm);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(master, wurm)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(master).unwrap().power, 6);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 0);
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(wurm).is_none(), "the 6-power Master killed the Wurm");
    assert!(g.battlefield_find(master).is_some(), "and took no damage");
}

/// Lim-Dûl's Vault digs past a window of lands for 1 life a step and leaves
/// the window it stopped on on top.
#[test]
fn lim_duls_vault_digs_for_a_spell() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::plains());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..9 {
        g.add_card_to_library(0, catalog::plains());
    }
    let life = g.players[0].life;
    let vault = g.add_card_to_hand(0, catalog::lim_duls_vault());
    cast_at(&mut g, vault, &[]).expect("cast");
    assert_eq!(g.players[0].life, life - 1, "one dig");
    assert!(g.players[0].library.iter().take(5).any(|c| c.id == bear), "the spell's window is on top");
}

/// Act of Authority's upkeep exile hands the enchantment to the exiled
/// permanent's controller.
#[test]
fn act_of_authority_changes_hands() {
    let mut g = main_phase(3);
    let act = g.add_card_to_battlefield(0, catalog::act_of_authority());
    let ring = g.add_card_to_battlefield(2, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    fire(&mut g, TurnStep::Upkeep);
    assert!(g.battlefield_find(ring).is_none(), "exiled");
    assert_eq!(controller(&g, act), 2, "its controller now controls Act of Authority");
}

/// Famine hits every creature and every player.
#[test]
fn famine_hits_everything() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::famine());
    cast_at(&mut g, spell, &[]).expect("cast");
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(wurm).is_some());
    assert!((0..3).all(|p| g.players[p].life == g.players[p].starting_life - 3));
}

/// Survival Cache draws only while you are ahead of an opponent, and
/// rebounds (CR 702.88) when cast from hand.
#[test]
fn survival_cache_draws_when_ahead() {
    let mut g = main_phase(3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    g.players[0].life = 40;
    g.players[1].life = 50;
    g.players[2].life = 41;
    let cache = g.add_card_to_hand(0, catalog::survival_cache());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, cache, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand, "42 > 41: drew one to replace the cast");
    assert!(g.exile.iter().any(|c| c.id == cache), "rebound exiles it");
}

/// Cradle of Vitality turns each life gained into a counter.
#[test]
fn cradle_of_vitality_counts_life_gained() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::cradle_of_vitality());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let disciple = g.add_card_to_battlefield(0, catalog::disciple_of_griselbrand());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let _ = (disciple, wurm);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: disciple,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice");
    drain_stack(&mut g);
    let gained = g.players[0].life - life;
    assert!(gained > 0, "Disciple gains the sacrificed toughness");
    let counters: u32 = [bear, wurm, disciple]
        .iter()
        .filter_map(|id| g.battlefield_find(*id))
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters as i32, gained, "a counter per life gained");
}

/// Divinity of Pride is 8/8 at 25 or more life.
#[test]
fn divinity_of_pride_grows_at_25_life() {
    let mut g = main_phase(2);
    let d = g.add_card_to_battlefield(0, catalog::divinity_of_pride());
    g.players[0].life = 24;
    assert_eq!(g.computed_permanent(d).unwrap().power, 4);
    g.players[0].life = 25;
    assert_eq!(g.computed_permanent(d).unwrap().power, 8);
}

/// Tempt with Immortality: with no taker you return one creature card.
#[test]
fn tempt_with_immortality_returns_a_creature() {
    let mut g = main_phase(3);
    let dead = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::tempt_with_immortality());
    cast_at(&mut g, spell, &[]).expect("cast");
    assert!(g.battlefield_find(dead).is_some());
}
