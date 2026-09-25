//! Commander: the Bedecked Brokers precon (NCC, Perrie, `decks::cmdr_perrie`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
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

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

fn kw_counters(g: &GameState, id: CardId, kw: Keyword) -> u32 {
    g.battlefield_find(id).and_then(|c| c.keyword_counters.get(&kw).copied()).unwrap_or(0)
}

/// A bear carrying a +1/+1 counter, a flying counter and a shield counter:
/// three kinds (CR 122.1b — a keyword counter is a kind of its own).
fn three_kind_bear(g: &mut GameState, seat: usize) -> CardId {
    let bear = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
    let c = g.battlefield_find_mut(bear).unwrap();
    c.add_counters(CounterType::PlusOnePlusOne, 1);
    c.add_counters(CounterType::Shield, 1);
    c.keyword_counters.add(Keyword::Flying, 1);
    bear
}

/// Bribe Taker: one counter per kind — shield and flying kept, the +1/+1
/// kind as +1/+1.
#[test]
fn bribe_taker_takes_a_counter_per_kind() {
    let mut g = main_phase(2);
    three_kind_bear(&mut g, 0);
    let taker = g.add_card_to_hand(0, catalog::bribe_taker());
    cast_at(&mut g, taker, &[]).expect("cast");
    assert_eq!(counters(&g, taker, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&g, taker, CounterType::Shield), 1);
    assert_eq!(kw_counters(&g, taker, Keyword::Flying), 1);
}

/// Storm of Forms is copied once per kind: three kinds, four bounces.
#[test]
fn storm_of_forms_copies_per_counter_kind() {
    let mut g = main_phase(2);
    three_kind_bear(&mut g, 0);
    let theirs: Vec<CardId> = (0..4).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    let storm = g.add_card_to_hand(0, catalog::storm_of_forms());
    cast_at(&mut g, storm, &[Target::Permanent(theirs[0])]).expect("cast");
    let gone = theirs.iter().filter(|id| g.battlefield_find(**id).is_none()).count();
    assert_eq!(gone, 4, "the original and three copies each bounced a Bear");
}

/// Damning Verdict spares a creature with any counter, a keyword counter
/// included.
#[test]
fn damning_verdict_spares_countered_creatures() {
    let mut g = main_phase(2);
    let plain = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(flier).unwrap().keyword_counters.add(Keyword::Flying, 1);
    let grown = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(grown).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let spell = g.add_card_to_hand(0, catalog::damning_verdict());
    cast_at(&mut g, spell, &[]).expect("cast");
    assert!(g.battlefield_find(plain).is_none());
    assert!(g.battlefield_find(flier).is_some() && g.battlefield_find(grown).is_some());
}

/// Declaration in Stone takes the target's namesakes under its controller
/// only, and that player investigates once per nontoken creature.
#[test]
fn declaration_in_stone_exiles_namesakes_and_investigates() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::declaration_in_stone());
    cast_at(&mut g, spell, &[Target::Permanent(a)]).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(mine).is_some(), "not the caster's Bears");
    let clues = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Clue").count();
    assert_eq!(clues, 2);
}

/// Contractual Safeguard in a main phase: a shield first, then the best kind
/// (the flying counter) goes on each other creature.
#[test]
fn contractual_safeguard_spreads_a_kind() {
    let mut g = main_phase(2);
    let flier = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(flier).unwrap().keyword_counters.add(Keyword::Flying, 1);
    let a = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::contractual_safeguard());
    cast_at(&mut g, spell, &[]).expect("cast");
    assert_eq!(counters(&g, wurm, CounterType::Shield), 1, "the addendum shield on the biggest");
    assert_eq!(kw_counters(&g, a, Keyword::Flying), 1);
    assert_eq!(kw_counters(&g, wurm, Keyword::Flying), 1);
    assert_eq!(kw_counters(&g, flier, Keyword::Flying), 1, "not on the creature it was chosen from");
}

/// Denry Klin: its own entry doubles the chosen counter, and each nontoken
/// creature after it copies Denry's counters.
#[test]
fn denry_klin_copies_its_counters() {
    let mut g = main_phase(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let denry = g.add_card_to_hand(0, catalog::denry_klin_editor_in_chief());
    cast_at(&mut g, denry, &[]).expect("cast");
    assert_eq!(counters(&g, denry, CounterType::PlusOnePlusOne), 2);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 2);
}

/// Exotic Pets: two unblockable Fish; each kind among your creatures lands on
/// one of them.
#[test]
fn exotic_pets_hands_out_each_kind() {
    let mut g = main_phase(2);
    three_kind_bear(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::exotic_pets());
    cast_at(&mut g, spell, &[]).expect("cast");
    let fish: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Fish").map(|c| c.id).collect();
    assert_eq!(fish.len(), 2);
    let total: u32 = fish
        .iter()
        .map(|f| {
            counters(&g, *f, CounterType::PlusOnePlusOne)
                + counters(&g, *f, CounterType::Shield)
                + kw_counters(&g, *f, Keyword::Flying)
        })
        .sum();
    assert_eq!(total, 3);
}

/// Oracle's Vault: the free ability waits for three brick counters.
#[test]
fn oracles_vault_counts_bricks() {
    let mut g = main_phase(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let vault = g.add_card_to_battlefield(0, catalog::oracles_vault());
    assert!(activate(&mut g, vault, 1, &[]).is_err(), "no bricks yet");
    for _ in 0..3 {
        activate(&mut g, vault, 0, &[]).expect("impulse");
        g.battlefield_find_mut(vault).unwrap().tapped = false;
    }
    assert_eq!(counters(&g, vault, CounterType::Brick), 3);
    activate(&mut g, vault, 1, &[]).expect("three bricks");
}

/// Littjara Mirrorlake: a copy of your creature with an extra +1/+1 counter.
#[test]
fn littjara_mirrorlake_copies_with_a_counter() {
    let mut g = main_phase(2);
    let lake = g.add_card_to_battlefield(0, catalog::littjara_mirrorlake());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    activate(&mut g, lake, 1, &[Target::Permanent(wurm)]).expect("copy");
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Craw Wurm").map(|c| c.id).unwrap();
    assert_eq!(counters(&g, copy, CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(lake).is_none(), "sacrificed");
}

/// Agent's Toolkit enters with four counters and hands one to an entering
/// creature.
#[test]
fn agents_toolkit_moves_a_counter() {
    let mut g = main_phase(2);
    let kit = g.add_card_to_hand(0, catalog::agents_toolkit());
    cast_at(&mut g, kit, &[]).expect("cast");
    assert_eq!(counters(&g, kit, CounterType::Shield), 1);
    assert_eq!(kw_counters(&g, kit, Keyword::Deathtouch), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, &[]).expect("cast");
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&g, kit, CounterType::PlusOnePlusOne), 0);
}

/// Kros: each upkeep an opposing creature gets a shield counter and is
/// tapped and goaded.
#[test]
fn kros_goads_the_creature_it_shields() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::kros_defense_contractor());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    fire(&mut g, TurnStep::Upkeep);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::Shield), 1);
    assert!(c.tapped);
    assert!(g.goaders(g.battlefield_find(bear).unwrap()).contains(&0), "goaded by Kros's controller");
}

/// Park Heights Maverick proliferates as it dies.
#[test]
fn park_heights_maverick_proliferates_on_death() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::park_heights_maverick());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let mut evs = Vec::new();
    g.destroy_permanent(m, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 2);
}

/// Aven Mimeomancer's feather counter makes a creature a 3/1 flier.
#[test]
fn aven_mimeomancer_feathers_a_creature() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::aven_mimeomancer());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.battlefield_find_mut(wurm).unwrap().add_counters(CounterType::Feather, 1);
    let cp = g.computed_permanent(wurm).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1));
    assert!(cp.keywords().contains(&Keyword::Flying));
}

/// Perrie's attack pumps by the kinds of counters you have (three) with
/// trample.
#[test]
fn perrie_pumps_by_counter_kinds() {
    let mut g = main_phase(2);
    let perrie = g.add_card_to_battlefield(0, catalog::perrie_the_pulverizer());
    g.clear_sickness(perrie);
    three_kind_bear(&mut g, 0);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: perrie, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let pumped = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .filter_map(|c| g.computed_permanent(c.id))
        .any(|cp| cp.keywords().contains(&Keyword::Trample) && cp.power >= cp.def.power + 3);
    assert!(pumped, "a creature of yours got +3/+3 and trample");
}

/// Brokers Confluence, proliferate three times.
#[test]
fn brokers_confluence_repeats_a_mode() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let spell = g.add_card_to_hand(0, catalog::brokers_confluence());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: spell,
        spree_modes: vec![0, 0, 0],
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 4);
}
