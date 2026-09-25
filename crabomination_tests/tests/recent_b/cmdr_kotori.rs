//! Commander: the Buckle Up precon (NEC, Kotori, `decks::cmdr_kotori`), and
//! its primitives: a granted crew counts (CR 702.122), a copy that becomes a
//! Vehicle artifact (CR 707.9b).

use crabomination::card::{ArtifactSubtype, CardId, CardType, CounterType, Keyword};
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
            g.add_card_to_library(seat, catalog::sol_ring());
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
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

fn crew(g: &mut GameState, vehicle: CardId, crew: &[CardId]) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle, crew_creatures: crew.to_vec() }).map(|_| ()).map_err(|e| format!("{e:?}"))?;
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

fn is_creature(g: &GameState, id: CardId) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.card_types().contains(&CardType::Creature))
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

/// CR 702.122 — Kotori grants crew 2: a Crew 6 Colossal Plow crews with a
/// 2-power creature, and the combat trigger gives an artifact creature
/// lifelink and vigilance.
#[test]
fn cr_702_122_kotori_grants_crew_two() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kotori_pilot_prodigy());
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    crew(&mut g, plow, &[bear]).expect("crew 2 is enough");
    assert!(is_creature(&g, plow));
    step(&mut g, TurnStep::BeginCombat);
    assert!(kw(&g, plow, Keyword::Lifelink) && kw(&g, plow, Keyword::Vigilance));
}

/// Without Kotori the Plow needs 6 power.
#[test]
fn colossal_plow_needs_six_alone() {
    let mut g = pod(2);
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(crew(&mut g, plow, &[bear]).is_err());
}

/// Colossal Plow — attacking adds {W}{W}{W} and 3 life.
#[test]
fn colossal_plow_attack_mana() {
    let mut g = pod(2);
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let giant2 = g.add_card_to_battlefield(0, catalog::hill_giant());
    crew(&mut g, plow, &[giant, giant2]).expect("crew 6");
    let life = g.players[0].life;
    attack(&mut g, &[plow]);
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.players[0].mana_pool.amount(Color::White) >= 3);
}

/// Access Denied — counter, then Thopters equal to the spell's mana value.
#[test]
fn access_denied_counters_into_thopters() {
    let mut g = pod(2);
    let hg = g.add_card_to_hand(1, catalog::hill_giant());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: hg, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("giant");
    let ad = g.add_card_to_hand(0, catalog::access_denied());
    flood(&mut g, 0);
    cast(&mut g, 0, ad, Some(Target::Permanent(hg))).expect("counter");
    assert_eq!(named(&g, 0, "Thopter").len(), 4);
    assert!(named(&g, 1, "Hill Giant").is_empty());
}

/// Aeronaut Admiral — Vehicles fly.
#[test]
fn aeronaut_admiral_gives_vehicles_flying() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::aeronaut_admiral());
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    assert!(kw(&g, plow, Keyword::Flying));
}

/// CR 707.9b — Imposter Mech copies an opponent's creature as a Vehicle
/// artifact with crew 3 and no creature type.
#[test]
fn cr_707_9b_imposter_mech_is_a_vehicle_copy() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let im = g.add_card_to_hand(0, catalog::imposter_mech());
    flood(&mut g, 0);
    cast(&mut g, 0, im, None).expect("cast");
    let mech = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Vehicle))
        .map(|c| c.id)
        .expect("a Vehicle");
    let c = g.computed_permanent(mech).unwrap();
    if c.def.name == "Serra Angel" {
        assert!(!c.card_types().contains(&CardType::Creature), "not a creature until crewed");
        assert!(c.subtypes().creature_types.is_empty(), "no Angel type");
        assert_eq!(c.def.crew_cost(), Some(3));
        assert!(c.keywords().contains(&Keyword::Flying), "keeps the Angel's abilities");
    }
}

/// Imperial Recovery Unit — attacking returns a small creature card.
#[test]
fn imperial_recovery_unit_recurs() {
    let mut g = pod(2);
    let iru = g.add_card_to_battlefield(0, catalog::imperial_recovery_unit());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    crew(&mut g, iru, &[giant]).expect("crew");
    attack(&mut g, &[iru]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Ironsoul Enforcer — attacking alone returns an artifact card.
#[test]
fn ironsoul_enforcer_alone_returns_an_artifact() {
    let mut g = pod(2);
    let ie = g.add_card_to_battlefield(0, catalog::ironsoul_enforcer());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    attack(&mut g, &[ie]);
    assert!(g.battlefield_find(ring).is_some());
}

/// Katsumasa — animates a noncreature artifact as a 1/1 flier; upkeep counters.
#[test]
fn katsumasa_animates_and_counters() {
    let mut g = pod(2);
    let k = g.add_card_to_battlefield(0, catalog::katsumasa_the_animator());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.battlefield_find(ring).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    flood(&mut g, 0);
    activate(&mut g, k, 0, Some(Target::Permanent(ring))).expect("animate");
    assert_eq!(pt(&g, ring), (2, 2), "1/1 plus its counter");
    assert!(kw(&g, ring, Keyword::Flying));
}

/// Mobilizer Mech — becoming crewed animates another Vehicle.
#[test]
fn mobilizer_mech_animates_a_second_vehicle() {
    let mut g = pod(2);
    let mm = g.add_card_to_battlefield(0, catalog::mobilizer_mech());
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    crew(&mut g, mm, &[giant]).expect("crew 3");
    assert!(is_creature(&g, mm));
    assert!(is_creature(&g, plow), "the Plow rides along");
}

/// Myrsmith — an artifact spell pays {1} for a Myr.
#[test]
fn myrsmith_pays_for_myr() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::myrsmith());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    cast(&mut g, 0, ring, None).expect("cast");
    assert_eq!(named(&g, 0, "Myr").len(), 1);
}

/// Riddlesmith — an artifact spell loots.
#[test]
fn riddlesmith_loots() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::riddlesmith());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    cast(&mut g, 0, ring, None).expect("cast");
    assert_eq!(g.players[0].graveyard.len(), 1, "drew one, discarded one");
}

/// Peacewalker Colossus — {1}{W} animates another Vehicle.
#[test]
fn peacewalker_colossus_animates() {
    let mut g = pod(2);
    let pc = g.add_card_to_battlefield(0, catalog::peacewalker_colossus());
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    flood(&mut g, 0);
    activate(&mut g, pc, 0, Some(Target::Permanent(plow))).expect("animate");
    assert!(is_creature(&g, plow));
}

/// Prodigy's Prototype — attacking Vehicles make a boosted Pilot.
#[test]
fn prodigys_prototype_makes_pilots() {
    let mut g = pod(2);
    let pp = g.add_card_to_battlefield(0, catalog::prodigys_prototype());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    crew(&mut g, pp, &[bear]).expect("crew 2");
    attack(&mut g, &[pp]);
    assert_eq!(named(&g, 0, "Pilot").len(), 1);
}

/// Raff Capashen — historic spells at instant speed.
#[test]
fn raff_capashen_flashes_historic() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::raff_capashen_ships_mage());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g, 0);
    cast(&mut g, 0, ring, None).expect("an artifact on an opponent's upkeep");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, 0, bear, None).is_err(), "not historic");
}

/// Raiders' Karve — a land on top may enter tapped.
#[test]
fn raiders_karve_drops_a_land() {
    let mut g = pod(2);
    let rk = g.add_card_to_battlefield(0, catalog::raiders_karve());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    crew(&mut g, rk, &[giant]).expect("crew");
    attack(&mut g, &[rk]);
    let plains = named(&g, 0, "Plains");
    assert_eq!(plains.len(), 1);
    assert!(g.battlefield_find(plains[0]).unwrap().tapped);
}

/// Release to Memory — Spirits per creature card exiled.
#[test]
fn release_to_memory_counts_creatures() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::hill_giant());
    g.add_card_to_graveyard(1, catalog::sol_ring());
    let r = g.add_card_to_hand(0, catalog::release_to_memory());
    flood(&mut g, 0);
    cast(&mut g, 0, r, Some(Target::Player(1))).expect("cast");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(named(&g, 0, "Spirit").len(), 2);
}

/// Surgehacker Mech — twice your Vehicle count to an opposing creature.
#[test]
fn surgehacker_mech_scales_with_vehicles() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::colossal_plow());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let sm = g.add_card_to_hand(0, catalog::surgehacker_mech());
    flood(&mut g, 0);
    cast(&mut g, 0, sm, Some(Target::Permanent(giant))).expect("cast");
    assert!(g.battlefield_find(giant).is_none(), "4 damage (two Vehicles) kills a 3/3");
}

/// Swift Reconfiguration — the enchanted creature is a crew-5 Vehicle, not a
/// creature.
#[test]
fn swift_reconfiguration_makes_a_vehicle() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let sr = g.add_card_to_hand(0, catalog::swift_reconfiguration());
    flood(&mut g, 0);
    cast(&mut g, 0, sr, Some(Target::Permanent(angel))).expect("cast");
    assert!(!is_creature(&g, angel));
    assert_eq!(g.effective_crew_cost(angel), Some(5));
}

/// Dance of the Manse — X=2 returns two artifacts of MV ≤ 2.
#[test]
fn dance_of_the_manse_returns_artifacts() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::sol_ring());
    let b = g.add_card_to_graveyard(0, catalog::arcane_signet());
    let d = g.add_card_to_hand(0, catalog::dance_of_the_manse());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: d,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
}

/// Weatherlight — connecting digs for a historic card.
#[test]
fn weatherlight_digs_historic() {
    let mut g = pod(2);
    let wl = g.add_card_to_battlefield(0, catalog::weatherlight());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    crew(&mut g, wl, &[giant]).expect("crew");
    attack(&mut g, &[wl]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.players[0].hand.len() >= hand, "a Sol Ring (historic) or nothing");
}

/// Bot: CR 702.122 — a crewed Vehicle is an artifact creature, so the bot
/// attacks with it (the attacker walk read the printed type line and never
/// offered one; nor an animated land).
#[test]
fn bot_attacks_with_a_crewed_vehicle() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kotori_pilot_prodigy());
    let plow = g.add_card_to_battlefield(0, catalog::colossal_plow());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(plow);
    g.clear_sickness(bear);
    let mut bot = HeuristicBot::new();
    let mut declared = Vec::new();
    for _ in 0..30 {
        let seat = g.priority.player_with_priority;
        let Some(action) = bot.next_action(&g, seat) else { break };
        if let GameAction::DeclareAttackers(atts) = &action {
            declared = atts.iter().map(|a| a.attacker).collect();
        }
        let _ = g.perform_action(action);
        if g.step == TurnStep::DeclareBlockers {
            break;
        }
    }
    assert!(declared.contains(&plow), "the crewed Plow attacks: {declared:?}");
}
