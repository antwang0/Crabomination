//! Commander: the 2026-10-16 (`0128tsCy`) rescan's gaps (`decks::cmdr_oct16_f`).

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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn on_top(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.next_id();
    g.players[seat].add_to_library_top(id, def);
    id
}

fn attack(g: &mut GameState, attackers: &[(CardId, usize)]) {
    g.step = TurnStep::DeclareAttackers;
    let attacks = attackers.iter().map(|&(a, p)| Attack { attacker: a, target: AttackTarget::Player(p) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn to_combat_damage(g: &mut GameState) {
    for _ in 0..6 {
        if g.step == TurnStep::EndCombat || g.step == TurnStep::PostCombatMain {
            break;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(g);
    }
}

/// CR 702.8a — Blur of Heroism draws on entry and lets Hero spells (only)
/// be cast at instant speed.
#[test]
fn blur_of_heroism_gives_heroes_flash() {
    let mut g = pod(2);
    let blur = g.add_card_to_hand(0, catalog::blur_of_heroism());
    g.players[0].mana_pool.add(Color::White, 3);
    let hand = g.players[0].hand.len();
    cast(&mut g, blur, None).expect("Blur");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    g.step = TurnStep::BeginCombat;
    let hero = g.add_card_to_hand(0, catalog::bucky_barnes_eager_ally());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(cast(&mut g, bears, None).is_err(), "not a Hero");
    cast(&mut g, hero, None).expect("a Hero has flash");
    assert!(g.battlefield_find(hero).is_some());
}

/// CR 120.3 / 601.2d — Death to Our Enemies: four noncreature spells make four
/// tapped Treasures, and the fourth plan counter sacrifices it for 7 damage
/// divided among one or two targets.
#[test]
fn death_to_our_enemies_fires_on_the_fourth_plan_counter() {
    let mut g = pod(2);
    let plan = ready(&mut g, 0, catalog::death_to_our_enemies());
    let opp_life = g.players[1].life;
    for i in 0..4 {
        assert!(g.battlefield_find(plan).is_some(), "still out before spell {i}");
        let s = g.add_card_to_hand(0, catalog::opt());
        g.players[0].mana_pool.add(Color::Blue, 1);
        cast(&mut g, s, None).expect("Opt");
    }
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").collect();
    assert_eq!(treasures.len(), 4);
    assert!(treasures.iter().all(|c| c.tapped));
    assert!(g.battlefield_find(plan).is_none(), "sacrificed");
    assert_eq!(g.players[1].life, opp_life - 7);
}

/// CR 714.2 — Origin of Captain America's chapters: a counter plus first
/// strike and vigilance, a Sturdy Shield token (+1/+2, equip {2}), then a
/// stun counter on a tapped creature.
#[test]
fn origin_of_captain_america_runs_its_three_chapters() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let saga = g.add_card_to_hand(0, catalog::origin_of_captain_america());
    g.players[0].mana_pool.add(Color::White, 3);
    cast(&mut g, saga, Some(Target::Permanent(bears))).expect("saga");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::FirstStrike) && cp.keywords().contains(&Keyword::Vigilance));
    let chapter = |g: &GameState, n: usize| g.battlefield_find(saga).unwrap().definition.saga_chapters[n].1.clone();
    let ch2 = chapter(&g, 1);
    g.resolve_effect(&ch2, &EffectContext::for_ability(saga, 0, None)).expect("II");
    let shield = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Sturdy Shield").expect("Shield").id;
    g.battlefield_find_mut(shield).unwrap().attached_to = Some(bears);
    assert_eq!(g.computed_permanent(bears).unwrap().toughness, 2 + 1 + 2);
    let ch3 = chapter(&g, 2);
    g.resolve_effect(&ch3, &EffectContext::for_ability(saga, 0, Some(Target::Permanent(wurm)))).expect("III");
    let w = g.battlefield_find(wurm).unwrap();
    assert!(w.tapped);
    assert_eq!(w.counter_count(CounterType::Stun), 1);
}

/// Fishing Gear: combat damage exiles the damaged player's top card — a
/// permanent card may join your side, anything else (or a decline) is a Fish.
#[test]
fn fishing_gear_steals_a_permanent_or_makes_a_fish() {
    for (permanent, take) in [(true, true), (true, false), (false, true)] {
        let mut g = pod(3);
        let bears = ready(&mut g, 0, catalog::grizzly_bears());
        let gear = g.add_card_to_battlefield(0, catalog::fishing_gear());
        g.battlefield_find_mut(gear).unwrap().attached_to = Some(bears);
        let top = on_top(&mut g, 2, if permanent { catalog::craw_wurm() } else { catalog::opt() });
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(take)]));
        attack(&mut g, &[(bears, 2)]);
        to_combat_damage(&mut g);
        let fish = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Fish" && c.controller == 0).count();
        let stolen = g.battlefield_find(top).is_some_and(|c| c.controller == 0);
        assert_eq!(stolen, permanent && take, "permanent {permanent} take {take}");
        assert_eq!(fish, usize::from(!(permanent && take)), "permanent {permanent} take {take}");
        if !stolen {
            assert!(g.exile.iter().any(|c| c.id == top));
        }
    }
}

/// Kinship (ability word, CR 207.2c) — Sensation Gorger: a Goblin or Shaman on top lets you
/// wheel every player into four fresh cards; a non-sharing top card does nothing.
#[test]
fn sensation_gorger_kinship_wheels_every_player() {
    for shares in [true, false] {
        let mut g = pod(3);
        ready(&mut g, 0, catalog::sensation_gorger());
        for seat in 0..3 {
            for _ in 0..6 {
                on_top(&mut g, seat, catalog::grizzly_bears());
            }
            g.add_card_to_hand(seat, catalog::opt());
        }
        on_top(&mut g, 0, if shares { catalog::goblin_guide() } else { catalog::grizzly_bears() });
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let before: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        for seat in 0..3 {
            let want = if shares { 4 } else { before[seat] };
            assert_eq!(g.players[seat].hand.len(), want, "seat {seat}, shares {shares}");
        }
    }
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: idx, target, additional_targets: vec![], x_value: None, mode: None })?;
    drain_stack(g);
    Ok(())
}

/// Yotia Declares War II deals the tapped-artifact count; III makes a 4/4.
#[test]
fn yotia_declares_war_taps_artifacts_for_damage_then_animates_one() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::yotia_declares_war());
    let rocks: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::sol_ring())).collect();
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let chapter = |g: &GameState, n: usize| g.battlefield_find(saga).unwrap().definition.saga_chapters[n].1.clone();
    let ch2 = chapter(&g, 1);
    g.resolve_effect(&ch2, &EffectContext::for_ability(saga, 0, Some(Target::Permanent(wurm)))).expect("II");
    let tapped = rocks.iter().filter(|&&r| g.battlefield_find(r).unwrap().tapped).count();
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, tapped as u32);
    let ch3 = chapter(&g, 2);
    g.resolve_effect(&ch3, &EffectContext::for_ability(saga, 0, Some(Target::Permanent(rocks[0])))).expect("III");
    let cp = g.computed_permanent(rocks[0]).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Welding Sparks: 3 plus one per artifact you control.
#[test]
fn welding_sparks_scales_with_artifacts() {
    let mut g = pod(2);
    for _ in 0..2 {
        ready(&mut g, 0, catalog::sol_ring());
    }
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let sparks = g.add_card_to_hand(0, catalog::welding_sparks());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, sparks, Some(Target::Permanent(wurm))).expect("Sparks");
    assert!(g.battlefield_find(wurm).is_none(), "5 damage kills the 6/4 Craw Wurm");
}

/// CR 702.119 — Decimator of the Provinces emerges off a sacrificed creature
/// and its cast trigger pumps the team (+2/+2, trample) before it resolves.
#[test]
fn decimator_emerges_and_pumps_the_team_on_cast() {
    let mut g = pod(2);
    let fodder = ready(&mut g, 0, catalog::craw_wurm());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let dec = g.add_card_to_hand(0, catalog::decimator_of_the_provinces());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.perform_action(GameAction::CastSpellAlternative { card_id: dec, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("emerge {6}{G}{G}{G} less Craw Wurm's 6");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&Keyword::Trample));
    assert!(g.battlefield_find(dec).is_some());
}

/// Kraven's Last Hunt I: mill five, then the biggest milled creature's power
/// hits target creature.
#[test]
fn kravens_last_hunt_mills_then_fights_with_the_graveyard() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::kravens_last_hunt());
    on_top(&mut g, 0, catalog::craw_wurm());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let ch1 = g.battlefield_find(saga).unwrap().definition.saga_chapters[0].1.clone();
    g.resolve_effect(&ch1, &EffectContext::for_ability(saga, 0, None)).expect("I");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 5);
    assert!(g.battlefield_find(wurm).is_none(), "6 damage from the milled Craw Wurm");
}

/// Ronin's life-paid mana spends only on Equipment; sacrificing the attached
/// Equipment shrinks a creature by -4/-4.
#[test]
fn ronin_pays_life_for_equipment_mana_and_sacrifices_it() {
    let mut g = pod(2);
    let ronin = ready(&mut g, 0, catalog::ronin_shadow_stalker());
    let life = g.players[0].life;
    activate(&mut g, ronin, 0, None).expect("pay 2 life");
    assert_eq!(g.players[0].life, life - 2);
    assert!(activate(&mut g, ronin, 0, None).is_err(), "once each turn");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, bears, None).is_err(), "not an Equipment spell");
    let sword = g.add_card_to_hand(0, catalog::short_sword());
    cast(&mut g, sword, None).expect("Equipment spell");
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(ronin);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    activate(&mut g, ronin, 1, Some(Target::Permanent(wurm))).expect("sacrifice the Sword");
    assert!(g.battlefield_find(sword).is_none());
    assert!(g.battlefield_find(wurm).is_none(), "6/4 at -4/-4 dies");
}

/// Swordsman: a Villain entering attaches an Equipment; an equipped attacker connives.
#[test]
fn swordsman_attaches_for_villains_and_equipped_attackers_connive() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::swordsman_sharp_scoundrel());
    let sword = ready(&mut g, 0, catalog::short_sword());
    let boom = g.add_card_to_hand(0, catalog::boomerang_blade_flinger());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, boom, None).expect("Boomerang");
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, Some(boom));
    g.clear_sickness(boom);
    let hand = g.players[0].hand.len();
    attack(&mut g, &[(boom, 1)]);
    assert_eq!(g.players[0].hand.len(), hand, "connived: drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Boomerang's attack pings every opponent of a pod and gains one life.
#[test]
fn boomerang_pings_each_opponent_on_attack() {
    let mut g = pod(4);
    let boom = ready(&mut g, 0, catalog::boomerang_blade_flinger());
    attack(&mut g, &[(boom, 2)]);
    assert_eq!(g.players[0].life, 21);
    for seat in 1..4 {
        assert_eq!(g.players[seat].life, 19, "seat {seat}");
    }
}

/// CR 510.1a — Loot, the Anomaly's negative power assigns as though positive:
/// a -2/4 deals 2, and the threshold sacrifice (-2/-0) makes it 4.
#[test]
fn cr_510_1a_loot_assigns_negative_power_as_positive() {
    for threshold in [false, true] {
        let mut g = pod(2);
        let loot = ready(&mut g, 0, catalog::loot_the_anomaly());
        if threshold {
            for _ in 0..7 {
                g.add_card_to_graveyard(0, catalog::forest());
            }
            let bears = ready(&mut g, 0, catalog::grizzly_bears());
            activate(&mut g, loot, 0, None).expect("threshold");
            assert!(g.battlefield_find(bears).is_none());
        }
        attack(&mut g, &[(loot, 1)]);
        to_combat_damage(&mut g);
        assert_eq!(g.players[1].life, 20 - if threshold { 4 } else { 2 });
    }
}

/// Silence the Echo: pays {3} with nothing to sacrifice, sacrifices otherwise.
#[test]
fn silence_the_echo_sacrifices_or_pays_three() {
    for fodder in [false, true] {
        let mut g = pod(2);
        let bears = fodder.then(|| ready(&mut g, 0, catalog::grizzly_bears()));
        let wurm = ready(&mut g, 1, catalog::craw_wurm());
        let s = g.add_card_to_hand(0, catalog::silence_the_echo());
        g.players[0].mana_pool.add(Color::Black, 2);
        if !fodder {
            assert!(cast(&mut g, s, Some(Target::Permanent(wurm))).is_err(), "needs {{3}} more");
            g.players[0].mana_pool.add_colorless(3);
        }
        cast(&mut g, s, Some(Target::Permanent(wurm))).expect("Silence the Echo");
        assert!(g.battlefield_find(wurm).is_none());
        if let Some(b) = bears {
            assert!(g.battlefield_find(b).is_none(), "sacrificed");
        }
    }
}
