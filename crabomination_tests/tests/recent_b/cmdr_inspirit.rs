//! Commander: the Counter Intelligence precon (EOC, Inspirit,
//! `decks::cmdr_inspirit`), and its primitives: a spell gaining sunburst
//! (CR 702.44), counters carried off a leaving permanent (CR 603.10a), an
//! any-kind X-counter cost.

use crabomination::card::{CardId, CounterType, Keyword};
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
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate_x(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    activate_x(g, id, index, target, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn kw(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&k))
}

fn is_creature(g: &GameState, id: CardId) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.card_types().contains(&crabomination::card::CardType::Creature))
}

fn counters(g: &GameState, id: CardId, k: CounterType) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(k)).unwrap_or(0)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

/// Inspirit — 8+ charge counters: a 5/5 flier whose other artifacts have
/// hexproof and indestructible; its 1+ combat trigger counters another
/// artifact.
#[test]
fn inspirit_bands() {
    let mut g = pod(2);
    let ins = g.add_card_to_battlefield(0, catalog::inspirit_flagship_vessel());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.battlefield_find_mut(ins).unwrap().add_counters(CounterType::Charge, 1);
    assert!(!is_creature(&g, ins), "not a creature below 8");
    step(&mut g, TurnStep::BeginCombat);
    let on_ring = counters(&g, ring, CounterType::PlusOnePlusOne) + counters(&g, ring, CounterType::Charge);
    assert!(on_ring >= 1, "the 1+ trigger put a counter on Sol Ring");
    g.battlefield_find_mut(ins).unwrap().add_counters(CounterType::Charge, 7);
    assert_eq!(pt(&g, ins), (5, 5));
    assert!(kw(&g, ins, Keyword::Flying));
    assert!(kw(&g, ring, Keyword::Hexproof) && kw(&g, ring, Keyword::Indestructible));
    assert!(!kw(&g, ins, Keyword::Indestructible), "other artifacts only");
}

/// Kilo — becoming tapped proliferates.
#[test]
fn kilo_proliferates_when_tapped() {
    let mut g = pod(2);
    let kilo = g.add_card_to_battlefield(0, catalog::kilo_apogee_mind());
    let eng = g.add_card_to_battlefield(0, catalog::insight_engine());
    g.battlefield_find_mut(eng).unwrap().add_counters(CounterType::Charge, 1);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kilo, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(counters(&g, eng, CounterType::Charge), 2);
}

/// Chrome Host Seedshark — a noncreature spell incubates its mana value.
#[test]
fn seedshark_incubates_mana_value() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::chrome_host_seedshark());
    let d = g.add_card_to_hand(0, catalog::divination());
    flood(&mut g, 0);
    cast(&mut g, 0, d, None).expect("divination");
    let inc = named(&g, 0, "Incubator");
    assert_eq!(inc.len(), 1);
    assert_eq!(counters(&g, inc[0], CounterType::PlusOnePlusOne), 3);
}

/// Cloud Key — the chosen type costs {1} less.
#[test]
fn cloud_key_discounts_the_chosen_type() {
    let mut g = pod(2);
    // The choice order: creature, instant, sorcery, artifact, …
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(3)]));
    g.add_card_to_battlefield_entering(0, catalog::cloud_key());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, 0, ring, None).expect("Sol Ring for {0}");
    assert_eq!(named(&g, 0, "Sol Ring").len(), 1);
}

/// Depthshaker Titan — a noncreature artifact becomes a 3/3 with melee,
/// trample and haste, and is sacrificed at the next end step.
#[test]
fn depthshaker_titan_animates_artifacts() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let t = g.add_card_to_hand(0, catalog::depthshaker_titan());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: t,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    // Whatever the targeter picked, an animated artifact is a 3/3 with the grants.
    if is_creature(&g, ring) {
        assert_eq!(pt(&g, ring), (3, 3));
        assert!(kw(&g, ring, Keyword::Melee) && kw(&g, ring, Keyword::Haste));
        step(&mut g, TurnStep::End);
        assert!(g.battlefield_find(ring).is_none(), "sacrificed at the end step");
    }
    let titan = named(&g, 0, "Depthshaker Titan")[0];
    assert!(kw(&g, titan, Keyword::Trample) && kw(&g, titan, Keyword::Melee));
}

/// Empowered Autogenerator — each tap adds a counter and that much mana.
#[test]
fn empowered_autogenerator_grows() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::empowered_autogenerator());
    activate(&mut g, a, 0, None).expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 1);
    g.battlefield_find_mut(a).unwrap().tapped = false;
    activate(&mut g, a, 0, None).expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Gavel of the Righteous — +1/+1 per counter of any kind, double strike at
/// four; the alternative equip removes a counter.
#[test]
fn gavel_counts_every_counter() {
    let mut g = pod(2);
    let gavel = g.add_card_to_battlefield(0, catalog::gavel_of_the_righteous());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(gavel).unwrap().add_counters(CounterType::Charge, 3);
    g.battlefield_find_mut(gavel).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    activate(&mut g, gavel, 0, Some(Target::Permanent(bear))).expect("equip by removing a counter");
    assert_eq!(g.battlefield_find(gavel).unwrap().attached_to, Some(bear));
    assert_eq!(pt(&g, bear), (6, 6), "four counters left");
    assert!(kw(&g, bear, Keyword::DoubleStrike));
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(pt(&g, bear), (7, 7));
}

/// Golem Foundry — artifact spells feed it; three counters make a Golem.
#[test]
fn golem_foundry_makes_golems() {
    let mut g = pod(2);
    let gf = g.add_card_to_battlefield(0, catalog::golem_foundry());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    cast(&mut g, 0, ring, None).expect("artifact spell");
    assert_eq!(counters(&g, gf, CounterType::Charge), 1);
    g.battlefield_find_mut(gf).unwrap().add_counters(CounterType::Charge, 2);
    activate(&mut g, gf, 0, None).expect("remove three");
    assert_eq!(named(&g, 0, "Golem").len(), 1);
    assert_eq!(counters(&g, gf, CounterType::Charge), 0);
}

/// Insight Engine — draws one, then two.
#[test]
fn insight_engine_draws_per_counter() {
    let mut g = pod(2);
    let e = g.add_card_to_battlefield(0, catalog::insight_engine());
    let h = g.players[0].hand.len();
    flood(&mut g, 0);
    activate(&mut g, e, 0, None).expect("first");
    g.battlefield_find_mut(e).unwrap().tapped = false;
    activate(&mut g, e, 0, None).expect("second");
    assert_eq!(g.players[0].hand.len(), h + 3);
}

/// Long-Range Sensor — a counter per player attacked.
#[test]
fn long_range_sensor_counts_players_attacked() {
    let mut g = pod(3);
    let s = g.add_card_to_battlefield(0, catalog::long_range_sensor());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(counters(&g, s, CounterType::Charge), 2);
}

/// CR 702.44 — Lux Artillery gives an artifact creature spell sunburst: one
/// +1/+1 counter per color spent (here white, blue and red on top of
/// Patrolling Peacemaker's own two).
#[test]
fn cr_702_44_lux_artillery_grants_sunburst() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::lux_artillery());
    let pp = g.add_card_to_hand(0, catalog::patrolling_peacemaker());
    for c in [Color::White, Color::Blue, Color::Red] {
        g.players[0].mana_pool.add(c, 1);
    }
    cast(&mut g, 0, pp, None).expect("cast");
    let pp = named(&g, 0, "Patrolling Peacemaker")[0];
    assert_eq!(counters(&g, pp, CounterType::PlusOnePlusOne), 2 + 3);
}

/// CR 702.44 — Solar Array's rider: the next artifact spell (noncreature)
/// enters with charge counters per color spent.
#[test]
fn cr_702_44_solar_array_next_artifact_gains_sunburst() {
    let mut g = pod(2);
    let sa = g.add_card_to_battlefield(0, catalog::solar_array());
    activate(&mut g, sa, 0, None).expect("tap for mana");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let ie = g.add_card_to_hand(0, catalog::insight_engine());
    cast(&mut g, 0, ie, None).expect("cast");
    let ie = named(&g, 0, "Insight Engine")[0];
    assert!(counters(&g, ie, CounterType::Charge) >= 2, "one charge counter per color spent");
}

/// Lux Artillery — thirty counters at your end step: 10 to each opponent.
#[test]
fn lux_artillery_fires_at_thirty() {
    let mut g = pod(3);
    let la = g.add_card_to_battlefield(0, catalog::lux_artillery());
    g.battlefield_find_mut(la).unwrap().add_counters(CounterType::Charge, 29);
    let life = g.players[1].life;
    step(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, life, "29 counters: nothing");
    g.battlefield_find_mut(la).unwrap().add_counters(CounterType::Charge, 1);
    step(&mut g, TurnStep::End);
    assert_eq!((g.players[1].life, g.players[2].life), (life - 10, life - 10));
}

/// Lux Cannon — three charges destroy a permanent.
#[test]
fn lux_cannon_destroys() {
    let mut g = pod(2);
    let lc = g.add_card_to_battlefield(0, catalog::lux_cannon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(lc).unwrap().add_counters(CounterType::Charge, 3);
    activate(&mut g, lc, 1, Some(Target::Permanent(bear))).expect("fire");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(counters(&g, lc, CounterType::Charge), 0);
}

/// Moxite Refinery — remove X counters of any kinds, put X +1/+1 counters.
#[test]
fn moxite_refinery_converts_counters() {
    let mut g = pod(2);
    let mr = g.add_card_to_battlefield(0, catalog::moxite_refinery());
    let eng = g.add_card_to_battlefield(0, catalog::insight_engine());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(eng).unwrap().add_counters(CounterType::Charge, 3);
    flood(&mut g, 0);
    activate_x(&mut g, mr, 1, Some(Target::Permanent(bear)), Some(3)).expect("X = 3");
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 3);
    assert_eq!(counters(&g, eng, CounterType::Charge), 0);
}

/// Patrolling Peacemaker — enters with two counters; an opponent's crime
/// proliferates.
#[test]
fn patrolling_peacemaker_proliferates_on_crime() {
    let mut g = pod(2);
    let pp = g.add_card_to_hand(0, catalog::patrolling_peacemaker());
    flood(&mut g, 0);
    cast(&mut g, 0, pp, None).expect("cast");
    let pp = named(&g, 0, "Patrolling Peacemaker")[0];
    assert_eq!(counters(&g, pp, CounterType::PlusOnePlusOne), 2);
    let bolt = g.add_card_to_hand(1, catalog::shock());
    flood(&mut g, 1);
    cast(&mut g, 1, bolt, Some(Target::Player(0))).expect("a crime");
    assert_eq!(counters(&g, pp, CounterType::PlusOnePlusOne), 3);
}

/// CR 603.10a — Resourceful Defense moves a dying permanent's counters (read
/// as it last existed) onto another of your permanents.
#[test]
fn cr_603_10a_resourceful_defense_keeps_counters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::resourceful_defense());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keeper = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Charge, 1);
    let kill = g.add_card_to_hand(1, catalog::murder());
    flood(&mut g, 1);
    cast(&mut g, 1, kill, Some(Target::Permanent(bear))).expect("murder");
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne) + c.counter_count(CounterType::Charge))
        .sum();
    assert_eq!(total, 3, "both kinds carried over");
    let _ = keeper;
}

/// Ripples of Potential — proliferate, then (on a yes) your permanents with
/// counters phase out.
#[test]
fn ripples_of_potential_proliferates_and_phases() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let r = g.add_card_to_hand(0, catalog::ripples_of_potential());
    flood(&mut g, 0);
    cast(&mut g, 0, r, None).expect("cast");
    let c = g.phased_out.iter().find(|c| c.id == bear).expect("phased out");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Surge Conductor — another nontoken artifact entering proliferates.
#[test]
fn surge_conductor_proliferates() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::surge_conductor());
    let eng = g.add_card_to_battlefield(0, catalog::insight_engine());
    g.battlefield_find_mut(eng).unwrap().add_counters(CounterType::Charge, 1);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    cast(&mut g, 0, ring, None).expect("cast");
    assert_eq!(counters(&g, eng, CounterType::Charge), 2);
}

/// Universal Surveillance — draw X.
#[test]
fn universal_surveillance_draws_x() {
    let mut g = pod(2);
    let us = g.add_card_to_hand(0, catalog::universal_surveillance());
    let h = g.players[0].hand.len();
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: us, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h - 1 + 3);
}

/// Uthros Research Craft — at 3+ an artifact spell draws and charges it; at
/// 12+ it's a flier with +1/+0 per artifact.
#[test]
fn uthros_research_craft_bands() {
    let mut g = pod(2);
    let u = g.add_card_to_battlefield(0, catalog::uthros_research_craft());
    g.battlefield_find_mut(u).unwrap().add_counters(CounterType::Charge, 3);
    let h = g.players[0].hand.len();
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    cast(&mut g, 0, ring, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), h + 1, "drew a card");
    assert_eq!(counters(&g, u, CounterType::Charge), 4);
    g.battlefield_find_mut(u).unwrap().add_counters(CounterType::Charge, 8);
    assert_eq!(pt(&g, u), (2, 8), "two artifacts: itself and Sol Ring");
    assert!(kw(&g, u, Keyword::Flying));
}


/// CR 613.8 — a layer-6/7 effect whose set is "artifact creatures" or
/// "creatures" must see a layer-4 type change: Tezzeret's −2 animates Sol
/// Ring and Depthshaker Titan's grants reach it; an activated Mutavault gets
/// Glorious Anthem's +1/+1.
#[test]
fn cr_613_8_type_filtered_sets_read_layer_four_types() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::depthshaker_titan());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_betrayer_of_flesh());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tez,
        ability_index: 1,
        target: Some(Target::Permanent(ring)),
        x_value: None,
    })
    .expect("−2");
    drain_stack(&mut g);
    assert_eq!(pt(&g, ring), (4, 4));
    assert!(kw(&g, ring, Keyword::Melee) && kw(&g, ring, Keyword::Trample) && kw(&g, ring, Keyword::Haste));

    g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let vault = g.add_card_to_battlefield(0, catalog::mutavault());
    flood(&mut g, 0);
    activate(&mut g, vault, 1, None).expect("animate");
    assert_eq!(pt(&g, vault), (3, 3), "a 2/2 land creature under the anthem");
}
