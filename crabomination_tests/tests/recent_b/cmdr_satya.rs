//! Commander: the Creative Energy precon (M3C, Satya, Aetherflux Genius,
//! `decks::cmdr_satya`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    let events = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

/// Satya — attacking copies another creature tapped and attacking and gets
/// {E}{E}; the copy stays past the end step only if its mana value is paid.
#[test]
fn satya_copies_and_charges_energy() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::satya_aetherflux_genius());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    attack(&mut g, &[s]);
    let copies: Vec<CardId> = named(&g, 0, "Hill Giant").into_iter().filter(|&id| id != giant).collect();
    assert_eq!(copies.len(), 1, "a token copy");
    assert!(g.battlefield_find(copies[0]).unwrap().tapped);
    assert_eq!(g.players[0].energy, 2);
    end_step(&mut g);
    assert!(g.battlefield_find(copies[0]).is_none(), "two {{E}} can't pay mana value 4");
    // With enough energy the copy is kept.
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::satya_aetherflux_genius());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, &[s]);
    end_step(&mut g);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "paid {{E}}{{E}}");
    assert_eq!(g.players[0].energy, 0);
}

/// Aether Refinery — energy doubles; its tap makes an X/X for the {E} paid.
#[test]
fn aether_refinery_doubles_and_mints() {
    let mut g = pod(2);
    let ar = g.add_card_to_battlefield(0, catalog::aether_refinery());
    run(&mut g, Effect::AddEnergy(Value::Const(2)), ar);
    assert_eq!(g.players[0].energy, 4);
    activate(&mut g, 0, ar, 0, None).expect("refinery");
    let t = named(&g, 0, "Aetherborn");
    assert_eq!(t.len(), 1);
    assert_eq!(pt(&g, t[0]), (6, 6), "4 + 2 (doubled) {{E}} paid");
    assert_eq!(g.players[0].energy, 0);
}

/// Aethergeode Miner — attacking gets {E}{E}; {E}{E} blinks it.
#[test]
fn aethergeode_miner_charges_and_blinks() {
    let mut g = pod(2);
    let m = g.add_card_to_battlefield(0, catalog::aethergeode_miner());
    attack(&mut g, &[m]);
    assert_eq!(g.players[0].energy, 2);
    g.step = TurnStep::PostCombatMain;
    activate(&mut g, 0, m, 0, None).expect("blink");
    assert_eq!(g.players[0].energy, 0);
    assert_eq!(named(&g, 0, "Aethergeode Miner").len(), 1);
}

/// Aethersphere Harvester — {E}{E} on entering; {E} buys lifelink.
#[test]
fn aethersphere_harvester_lifelinks() {
    let mut g = pod(2);
    let h = g.add_card_to_hand(0, catalog::aethersphere_harvester());
    cast(&mut g, 0, h, None).expect("harvester");
    assert_eq!(g.players[0].energy, 2);
    activate(&mut g, 0, h, 0, None).expect("lifelink");
    assert!(g.computed_permanent(h).unwrap().keywords().contains(&Keyword::Lifelink));
    assert_eq!(g.players[0].energy, 1);
}

/// Aetherstorm Roc — creatures entering give {E}; attacking pays {E}{E} for
/// a counter and a tapped blocker.
#[test]
fn aetherstorm_roc_taps_a_blocker() {
    let mut g = pod(2);
    let roc = g.add_card_to_hand(0, catalog::aetherstorm_roc());
    cast(&mut g, 0, roc, None).expect("roc");
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, b, None).expect("bears");
    assert_eq!(g.players[0].energy, 2);
    let blocker = g.add_card_to_battlefield(1, catalog::serra_angel());
    attack(&mut g, &[roc]);
    assert_eq!(g.battlefield_find(roc).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

/// Aurora Shifter — its combat damage becomes energy, and energy turns it
/// into a copy that keeps its triggers.
#[test]
fn aurora_shifter_copies_with_energy() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::aurora_shifter());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.players[0].energy = 2;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let c = g.battlefield_find(a).unwrap();
    assert_eq!(c.definition.name, "Serra Angel");
    assert!(c.definition.triggered_abilities.iter().any(|t| matches!(t.effect, Effect::AddEnergy(_))));
}

/// Blaster Hulk — energy spent this turn discounts it.
#[test]
fn blaster_hulk_discounts_by_energy_spent() {
    let mut g = pod(2);
    g.players[0].energy_spent_this_turn = 4;
    let bh = g.add_card_to_hand(0, catalog::blaster_hulk());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bh, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{2}{R}{R} after four {E}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bh).is_some());
}

/// Cayth — fabricate on entering; its tap proliferates.
#[test]
fn cayth_fabricates_and_proliferates() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::cayth_famed_mechanist());
    cast(&mut g, 0, c, None).expect("cayth");
    let counters = g.battlefield_find(c).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let servos = named(&g, 0, "Servo").len();
    assert_eq!(counters as usize + servos, 1, "a counter or a Servo");
    g.clear_sickness(c);
    g.players[0].energy = 1;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: c,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: Some(1),
    })
    .expect("proliferate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "proliferate adds energy");
}

/// Conversion Apparatus — energy for mana of any colors.
#[test]
fn conversion_apparatus_converts() {
    let mut g = pod(2);
    let ca = g.add_card_to_battlefield(0, catalog::conversion_apparatus());
    g.players[0].energy = 3;
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ca,
        ability_index: 2,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("convert");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
    assert_eq!(g.players[0].energy, 0);
}

/// Gonti's Aether Heart — artifacts entering give {E}{E}.
#[test]
fn gontis_aether_heart_charges() {
    let mut g = pod(2);
    let h = g.add_card_to_hand(0, catalog::gontis_aether_heart());
    cast(&mut g, 0, h, None).expect("heart");
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, 0, ring, None).expect("ring");
    assert_eq!(g.players[0].energy, 4);
}

/// Hourglass of the Lost — ticks up for mana, then returns permanents of
/// that mana value.
#[test]
fn hourglass_of_the_lost_returns_mv_x() {
    let mut g = pod(2);
    let hg = g.add_card_to_battlefield(0, catalog::hourglass_of_the_lost());
    g.battlefield_find_mut(hg).unwrap().add_counters(CounterType::Time, 2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    activate(&mut g, 0, hg, 1, None).expect("hourglass");
    assert!(g.battlefield_find(bears).is_some(), "mana value 2");
    assert!(g.battlefield_find(giant).is_none(), "mana value 4");
    assert!(g.battlefield_find(hg).is_none(), "exiled");
}

/// Localized Destruction — a creature with power equal to the {E} paid
/// survives the wipe.
#[test]
fn localized_destruction_spares_the_matching_power() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.players[0].energy = 2;
    let ld = g.add_card_to_hand(0, catalog::localized_destruction());
    cast(&mut g, 0, ld, None).expect("destruction");
    assert!(g.battlefield_find(giant).is_some(), "power 3 = {{E}} paid");
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(theirs).is_none());
}

/// Overclocked Electromancer — {E}{E}{E} for a counter, and attacking
/// doubles its power.
#[test]
fn overclocked_electromancer_doubles() {
    let mut g = pod(2);
    let oe = g.add_card_to_battlefield(0, catalog::overclocked_electromancer());
    g.players[0].energy = 3;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    attack(&mut g, &[oe]);
    assert_eq!(pt(&g, oe), (6, 3));
}

/// Razorfield Ripper — attacking gets {E} and +X/+X for your {E}.
#[test]
fn razorfield_ripper_pumps_by_energy() {
    let mut g = pod(2);
    let rr = g.add_card_to_battlefield(0, catalog::razorfield_ripper());
    g.players[0].energy = 2;
    attack(&mut g, &[rr]);
    assert_eq!(pt(&g, rr), (6, 6));
}

/// Salvation Colossus — attacking pumps and protects your other creatures.
#[test]
fn salvation_colossus_rallies() {
    let mut g = pod(2);
    let sc = g.add_card_to_battlefield(0, catalog::salvation_colossus());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, &[sc]);
    assert_eq!(pt(&g, bears), (4, 4));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Sphinx of the Revelation — life gained becomes {E}, spent to draw.
#[test]
fn sphinx_of_the_revelation_draws_with_energy() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::sphinx_of_the_revelation());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    run(&mut g, Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(3) }, s);
    assert_eq!(g.players[0].energy, 3);
    g.clear_sickness(s);
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, s, 0, None).expect("draw");
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Stone Idol Generator — each attacker gives {E}; six make a Construct.
#[test]
fn stone_idol_generator_builds() {
    let mut g = pod(2);
    let sig = g.add_card_to_battlefield(0, catalog::stone_idol_generator());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    attack(&mut g, &[a, b]);
    assert_eq!(g.players[0].energy, 2);
    g.players[0].energy = 6;
    g.step = TurnStep::PostCombatMain;
    activate(&mut g, 0, sig, 0, None).expect("construct");
    let c = named(&g, 0, "Construct");
    assert_eq!(c.len(), 1);
    assert_eq!(pt(&g, c[0]), (6, 12));
}
