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

/// CR 608.2d — Order of Succession's direction is the caster's choice: turning
/// it left (option 1, after the headless right) hands each player the
/// creature of the next seat in turn order.
#[test]
fn order_of_succession_direction_is_the_casters_choice() {
    let mut g = main_phase(3);
    let mine = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let left = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let right = g.add_card_to_battlefield(2, catalog::craw_wurm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
    let spell = g.add_card_to_hand(0, catalog::order_of_succession());
    cast_at(&mut g, spell, &[]).expect("cast");
    assert_eq!(controller(&g, left), 0);
    assert_eq!(controller(&g, right), 1);
    assert_eq!(controller(&g, mine), 2);
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

/// Serene Master's 2013-10-17 rulings: each power becomes the other's former
/// power, and counters still apply to the new value — a Wurm with a +1/+1
/// counter (7) blocked by a Master with one (1) leaves the Master at 7 + 1
/// and the Wurm at 1 + 1.
#[test]
fn serene_master_exchange_keeps_counters_on_top() {
    let mut g = main_phase(2);
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let master = g.add_card_to_battlefield(1, catalog::serene_master());
    g.battlefield_find_mut(wurm).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(master).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(wurm);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(master, wurm)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(master).unwrap().power, 8);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 2);
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

/// Lim-Dûl's Vault digs "as many times as you choose": a seat that digs past
/// the spell's window pays twice and keeps the third window on top.
#[test]
fn lim_duls_vault_digs_as_often_as_you_choose() {
    let mut g = main_phase(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::plains());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..9 {
        g.add_card_to_library(0, catalog::plains());
    }
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    let vault = g.add_card_to_hand(0, catalog::lim_duls_vault());
    cast_at(&mut g, vault, &[]).expect("cast");
    assert_eq!(g.players[0].life, life - 2, "two digs");
    assert!(g.players[0].library.iter().take(5).all(|c| c.id != bear));
}

/// Lim-Dûl's Vault — "put the last cards you looked at this way on top of
/// it in any order": the final window goes back in the order asked for.
#[test]
fn lim_duls_vault_orders_the_last_five() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let mut window = vec![bear];
    for _ in 0..4 {
        window.push(g.add_card_to_library(0, catalog::plains()));
    }
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let order: Vec<_> = window.iter().rev().copied().collect();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::ScryOrder { kept_top: order.clone(), bottom: vec![] },
    ]));
    let vault = g.add_card_to_hand(0, catalog::lim_duls_vault());
    cast_at(&mut g, vault, &[]).expect("cast");
    let top: Vec<_> = g.players[0].library.iter().take(5).map(|c| c.id).collect();
    assert_eq!(top, order, "the Bears sits fifth");
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

/// Springjack Pasture with `goats` Goat tokens made by its own {4},{T}
/// ability, untapped and with an empty pool afterwards.
fn pasture_with_goats(g: &mut GameState, goats: usize) -> CardId {
    let pasture = g.add_card_to_battlefield(0, catalog::springjack_pasture());
    for _ in 0..goats {
        flood(g, 0);
        g.perform_action(GameAction::ActivateAbility {
            card_id: pasture,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("make a Goat");
        drain_stack(g);
        g.battlefield_find_mut(pasture).unwrap().tapped = false;
    }
    g.players[0].mana_pool.empty();
    pasture
}

fn goats(g: &GameState) -> usize {
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Goat").count()
}

/// CR 107.3 / 605.1a — Springjack Pasture's "{T}, Sacrifice X Goats: Add X
/// mana of any one color" pays a spell through the auto-tapper, X sized to
/// what the cost still needs: {W}{W}{W} beside a Plains sacrifices two of
/// three Goats for {W}{W} and gains 2 life.
#[test]
fn springjack_pasture_pays_a_spell_with_x_sized_to_the_shortfall() {
    let mut g = main_phase(2);
    pasture_with_goats(&mut g, 3);
    g.add_card_to_battlefield(0, catalog::plains());
    let life = g.players[0].life;
    let marshal = g.add_card_to_hand(0, catalog::benalish_marshal());
    g.perform_action(GameAction::CastSpell { card_id: marshal, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Plains + two Goats pay {W}{W}{W}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(marshal).is_some());
    assert_eq!(goats(&g), 1, "only the two Goats the cost needed");
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(g.players[0].mana_pool.total(), 0, "no mana left floating");
}

/// With no Goat to sacrifice the Pasture still taps for {C}: a {1} cost is
/// paid by its first ability, not by an X of zero.
#[test]
fn springjack_pasture_without_goats_taps_for_colorless() {
    let mut g = main_phase(2);
    pasture_with_goats(&mut g, 0);
    let bears = g.add_card_to_hand(0, catalog::steel_overseer());
    g.add_card_to_battlefield(0, catalog::plains());
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Plains + the Pasture's {C} pay {2}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
}
