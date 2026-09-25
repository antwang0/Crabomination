//! Commander: the Explorers of the Deep precon (LCC, Hakbal,
//! `decks::cmdr_hakbal`), and its primitives: an explore-twice replacement
//! (CR 701.44), filtered retrace grants (CR 702.81), an opponent's targeted
//! ability tax.

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
            g.add_card_to_library(seat, catalog::grizzly_bears());
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

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
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

fn kw(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&k))
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn connect(g: &mut GameState, attackers: &[CardId]) {
    attack(g, attackers);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    g.step = TurnStep::PostCombatMain;
}

/// Hakbal — each Merfolk explores at combat (nonland tops: a counter each).
#[test]
fn hakbal_merfolk_explore_at_combat() {
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::hakbal_of_the_surging_soul());
    let elite = g.add_card_to_battlefield(0, catalog::deeproot_elite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!((plus(&g, h), plus(&g, elite), plus(&g, bear)), (1, 1, 0), "only Merfolk explore");
}

/// CR 701.44 — Topography Tracker: an explore becomes two.
#[test]
fn cr_701_44_topography_tracker_explores_twice() {
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::hakbal_of_the_surging_soul());
    g.add_card_to_battlefield(0, catalog::topography_tracker());
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(plus(&g, h), 2);
}

/// Hakbal attacking with a land in hand puts it onto the battlefield (on a
/// yes); with none, it draws.
#[test]
fn hakbal_attack_land_or_draw() {
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::hakbal_of_the_surging_soul());
    let hand = g.players[0].hand.len();
    attack(&mut g, &[h]);
    assert_eq!(g.players[0].hand.len(), hand + 1, "no land in hand: draw");

    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::hakbal_of_the_surging_soul());
    let island = g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![island])]));
    attack(&mut g, &[h]);
    assert_eq!(named(&g, 0, "Island").len(), 1);
}

/// Coralhelm Commander — level 4 makes it a 4/4 flier pumping other Merfolk.
#[test]
fn coralhelm_commander_levels() {
    let mut g = pod(2);
    let c = g.add_card_to_battlefield(0, catalog::coralhelm_commander());
    let elite = g.add_card_to_battlefield(0, catalog::deeproot_elite());
    flood(&mut g, 0);
    for _ in 0..4 {
        activate(&mut g, 0, c, 0, None).expect("level up");
    }
    assert_eq!(pt(&g, c), (4, 4));
    assert!(kw(&g, c, Keyword::Flying));
    assert_eq!(pt(&g, elite), (2, 2));
}

/// Deeproot Elite and Deeproot Waters — a Merfolk spell makes a hexproof
/// Merfolk, whose entry counters a Merfolk.
#[test]
fn deeproot_waters_and_elite() {
    let mut g = pod(2);
    let elite = g.add_card_to_battlefield(0, catalog::deeproot_elite());
    g.add_card_to_battlefield(0, catalog::deeproot_waters());
    let so = g.add_card_to_hand(0, catalog::seafloor_oracle());
    flood(&mut g, 0);
    cast(&mut g, 0, so, None).expect("cast");
    let tokens = named(&g, 0, "Merfolk");
    assert_eq!(tokens.len(), 1);
    assert!(kw(&g, tokens[0], Keyword::Hexproof));
    let total: u32 = [elite, tokens[0], named(&g, 0, "Seafloor Oracle")[0]].iter().map(|&i| plus(&g, i)).sum();
    assert_eq!(total, 2, "two other Merfolk entered");
}

/// CR 702.81 — Deeproot Historian: a Merfolk card in your graveyard can be
/// cast by discarding a land.
#[test]
fn cr_702_81_deeproot_historian_grants_retrace() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::deeproot_historian());
    let so = g.add_card_to_graveyard(0, catalog::seafloor_oracle());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.players[0].graveyard.iter().find(|c| c.id == so).unwrap().clone();
    assert!(g.effective_retrace(&c, 0));
    let c = g.players[0].graveyard.iter().find(|c| c.id == bear).unwrap().clone();
    assert!(!g.effective_retrace(&c, 0), "not a Merfolk or Druid");
}

/// Emperor Mihail II — a Merfolk spell may pay {1} for a Merfolk token.
#[test]
fn emperor_mihail_pays_for_merfolk() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::emperor_mihail_ii());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let so = g.add_card_to_hand(0, catalog::seafloor_oracle());
    flood(&mut g, 0);
    cast(&mut g, 0, so, None).expect("cast");
    assert_eq!(named(&g, 0, "Merfolk").len(), 1);
}

/// Herald of Secret Streams — countered creatures can't be blocked.
#[test]
fn herald_makes_countered_creatures_unblockable() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::herald_of_secret_streams());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!kw(&g, bear, Keyword::Unblockable));
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(kw(&g, bear, Keyword::Unblockable));
}

/// Kopala — an opponent's spell or ability aimed at your Merfolk costs {2}
/// more.
#[test]
fn kopala_taxes_targeting_merfolk() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kopala_warden_of_waves());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, 1, bolt, Some(Target::Permanent(k))).is_err(), "{{R}} isn't enough");
    g.players[1].mana_pool.add_colorless(2);
    cast(&mut g, 1, bolt, Some(Target::Permanent(k))).expect("{2}{R} is");
    // The ability half: a Prodigal Pyromancer's ping.
    let k2 = g.add_card_to_battlefield(0, catalog::kopala_warden_of_waves());
    let pyro = g.add_card_to_battlefield(1, catalog::prodigal_pyromancer());
    g.clear_sickness(pyro);
    g.players[1].mana_pool = Default::default();
    assert!(activate(&mut g, 1, pyro, 0, Some(Target::Permanent(k2))).is_err(), "the ping now costs {{2}}");
    g.players[1].mana_pool.add_colorless(2);
    activate(&mut g, 1, pyro, 0, Some(Target::Permanent(k2))).expect("paid");
}

/// Kumena — tapping five Merfolk counters each.
#[test]
fn kumena_tap_five() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::kumena_tyrant_of_orazca());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::deeproot_elite());
    }
    activate(&mut g, 0, k, 2, None).expect("tap five");
    assert_eq!(plus(&g, k), 1);
    assert_eq!(named(&g, 0, "Deeproot Elite").iter().map(|&i| plus(&g, i)).sum::<u32>(), 4);
}

/// Mist Dancer — other Merfolk get +1/+0 and flying.
#[test]
fn mist_dancer_lord() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mist_dancer());
    let elite = g.add_card_to_battlefield(0, catalog::deeproot_elite());
    assert_eq!(pt(&g, elite), (2, 1));
    assert!(kw(&g, elite, Keyword::Flying));
}

/// Sage of Fables — other Wizards enter with a counter; a counter buys a card.
#[test]
fn sage_of_fables_wizards() {
    let mut g = pod(2);
    let sage = g.add_card_to_battlefield(0, catalog::sage_of_fables());
    let so = g.add_card_to_hand(0, catalog::seafloor_oracle());
    flood(&mut g, 0);
    cast(&mut g, 0, so, None).expect("cast");
    let so = named(&g, 0, "Seafloor Oracle")[0];
    assert_eq!(plus(&g, so), 1);
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, sage, 0, None).expect("draw");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(plus(&g, so), 0);
}

/// Seafloor Oracle — a Merfolk connecting draws.
#[test]
fn seafloor_oracle_draws_on_merfolk_hits() {
    let mut g = pod(2);
    let so = g.add_card_to_battlefield(0, catalog::seafloor_oracle());
    let hand = g.players[0].hand.len();
    connect(&mut g, &[so]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Singer of Swift Rivers — Merfolk spells have flash; its entry shields
/// another creature.
#[test]
fn singer_grants_merfolk_flash() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::singer_of_swift_rivers());
    flood(&mut g, 0);
    cast(&mut g, 0, s, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Shield), 1);
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    let so = g.add_card_to_hand(0, catalog::seafloor_oracle());
    flood(&mut g, 0);
    cast(&mut g, 0, so, None).expect("a Merfolk at instant speed");
}

/// Surgespanner — becoming tapped, pay {1}{U} to bounce a permanent.
#[test]
fn surgespanner_bounces_when_tapped() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::surgespanner());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    attack(&mut g, &[s]);
    assert!(g.battlefield_find(bear).is_none(), "bounced");
}

/// Tishana — */* is your hand; entering draws per creature.
#[test]
fn tishana_draws_and_grows() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    let t = g.add_card_to_hand(0, catalog::tishana_voice_of_thunder());
    flood(&mut g, 0);
    cast(&mut g, 0, t, None).expect("cast");
    let t = named(&g, 0, "Tishana, Voice of Thunder")[0];
    assert_eq!(g.players[0].hand.len(), 3, "the Island and two draws");
    assert_eq!(pt(&g, t), (3, 3));
}

/// Tributary Instructor — a countered creature dying draws.
#[test]
fn tributary_instructor_draws_on_counter_death() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tributary_instructor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let hand = g.players[0].hand.len();
    let kill = g.add_card_to_hand(1, catalog::murder());
    flood(&mut g, 1);
    cast(&mut g, 1, kill, Some(Target::Permanent(bear))).expect("murder");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Wave Goodbye — only creatures without +1/+1 counters go home.
#[test]
fn wave_goodbye_spares_countered_creatures() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let w = g.add_card_to_hand(0, catalog::wave_goodbye());
    flood(&mut g, 0);
    cast(&mut g, 0, w, None).expect("cast");
    assert!(g.battlefield_find(a).is_some());
    assert!(g.battlefield_find(b).is_none());
}

/// Xolatoyac — its entry floods a land into an Island; the end step untaps
/// countered permanents.
#[test]
fn xolatoyac_floods_and_untaps() {
    let mut g = pod(2);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let x = g.add_card_to_hand(0, catalog::xolatoyac_the_smiling_flood());
    flood(&mut g, 0);
    cast(&mut g, 0, x, Some(Target::Permanent(land))).expect("cast");
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.subtypes().land_types.contains(&crabomination::card::LandType::Island));
    g.battlefield_find_mut(land).unwrap().tapped = true;
    step(&mut g, TurnStep::End);
    assert!(!g.battlefield_find(land).unwrap().tapped);
}

/// Bygone Marvels — returns a permanent card and exiles itself.
#[test]
fn bygone_marvels_returns_and_exiles() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bm = g.add_card_to_hand(0, catalog::bygone_marvels());
    flood(&mut g, 0);
    cast(&mut g, 0, bm, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.exile.iter().any(|c| c.definition.name == "Bygone Marvels"));
}

