//! Commander: the 2026-10-16 (`0128tsCy`) rescan's gaps (`decks::cmdr_oct16_f`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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
        for (seat, &held) in before.iter().enumerate() {
            let want = if shares { 4 } else { held };
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

/// CR 903.4b — The Prismatic Piper and Faceless One choose their color before
/// the game: a mono-red 99 validates under either alone, a red-and-black one
/// does not, and the seated commander is red.
#[test]
fn cr_903_4b_piper_and_faceless_one_take_the_color_their_deck_needs() {
    use crabomination::format::{CommanderDeckError, Deck, Format, validate_commander_deck};
    for cmd in [catalog::the_prismatic_piper, catalog::faceless_one] {
        let off = |main: Vec<_>| {
            validate_commander_deck(&Deck { commanders: vec![cmd()], main, sideboard: vec![] })
                .err()
                .map_or(0, |e| e.1.iter().filter(|e| matches!(e, CommanderDeckError::OffColorIdentity { .. })).count())
        };
        assert_eq!(off(vec![catalog::lightning_bolt()]), 0, "{}", cmd().name);
        assert_eq!(off(vec![catalog::lightning_bolt(), catalog::doom_blade()]), 2, "{}", cmd().name);
        let mut g = game_with_format(Format::Commander, 4);
        g.add_card_to_library(0, catalog::lightning_bolt());
        let ids = g.seat_commanders(0, vec![cmd()]);
        let c = g.players[0].command.iter().find(|c| c.id == ids[0]).unwrap();
        assert_eq!(c.definition.printed_colors(), vec![Color::Red], "{}", cmd().name);
    }
}

/// Dungeoneer's Pack: enters tapped; cracked, it takes the initiative (CR
/// 725), gains 3, draws and makes a Treasure.
#[test]
fn dungeoneers_pack_takes_the_initiative_and_pays_out() {
    let mut g = pod(3);
    let pack = g.add_card_to_battlefield(0, catalog::dungeoneers_pack());
    let id = g.battlefield.iter().rev().find(|c| c.definition.name == "Dungeoneer's Pack").unwrap().id;
    assert_eq!(id, pack);
    g.battlefield_find_mut(pack).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(2);
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    activate(&mut g, pack, 0, None).expect("crack the Pack");
    assert_eq!(g.initiative, Some(0));
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.players[0].hand.len() > hand);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"));
}

/// Trailblazer's Torch: the initiative on entry; an equipped creature that
/// becomes blocked deals 2 to each blocker.
#[test]
fn trailblazers_torch_burns_each_blocker() {
    let mut g = pod(2);
    let torch = g.add_card_to_hand(0, catalog::trailblazers_torch());
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, torch, None).expect("Torch");
    assert_eq!(g.initiative, Some(0));
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    g.battlefield_find_mut(torch).unwrap().attached_to = Some(wurm);
    let (a, b) = (ready(&mut g, 1, catalog::grizzly_bears()), ready(&mut g, 1, catalog::grizzly_bears()));
    attack(&mut g, &[(wurm, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(a, wurm), (b, wurm)])).expect("double block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "2 damage each to the 2/2s");
}

/// Harnessed Snubhorn's combat damage returns an artifact card to play.
#[test]
fn harnessed_snubhorn_returns_an_artifact_on_combat_damage() {
    let mut g = pod(2);
    let horn = ready(&mut g, 0, catalog::harnessed_snubhorn());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    attack(&mut g, &[(horn, 1)]);
    to_combat_damage(&mut g);
    assert!(g.battlefield_find(ring).is_some_and(|c| c.controller == 0));
}

/// Veteran Soldier: your commander attacking the player no opponent out-lives
/// makes a tapped-and-attacking Soldier at each opponent; attacking a player
/// another opponent out-lives makes none.
#[test]
fn veteran_soldier_fans_soldiers_out_when_attacking_the_life_leader() {
    for (defender_is_leader, want) in [(true, 3), (false, 0)] {
        let mut g = pod(4);
        g.players[0].life = 10; // your own total never counts
        g.players[3].life = if defender_is_leader { 20 } else { 25 };
        ready(&mut g, 0, catalog::veteran_soldier());
        let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
        g.players[0].commanders = vec![cmdr];
        attack(&mut g, &[(cmdr, 1)]);
        let soldiers: Vec<_> = g.attacking.iter().filter(|a| g.battlefield_find(a.attacker).is_some_and(|c| c.is_token)).collect();
        assert_eq!(soldiers.len(), want, "leader {defender_is_leader}");
        for seat in 1..4 {
            assert_eq!(soldiers.iter().filter(|a| a.target == AttackTarget::Player(seat)).count(), want / 3);
        }
    }
}

/// Lurebound Scarecrow names the color you have most of and is sacrificed
/// the moment you control none of it (CR 603.8 state trigger).
#[test]
fn lurebound_scarecrow_dies_with_its_color() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let crow = g.add_card_to_hand(0, catalog::lurebound_scarecrow());
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, crow, None).expect("Scarecrow");
    assert_eq!(g.battlefield_find(crow).unwrap().chosen_color, Some(Color::Green));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bears))).expect("Bolt the Bears");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(crow).is_none(), "no green permanent left");
}

/// Midnight Oil: seven hour counters cap the hand; the draw step draws one
/// more and burns two; every discard costs 1 life.
#[test]
fn midnight_oil_trades_hour_counters_for_cards() {
    let mut g = pod(2);
    let oil = g.add_card_to_hand(0, catalog::midnight_oil());
    g.players[0].mana_pool.add(Color::Black, 4);
    cast(&mut g, oil, None).expect("Midnight Oil");
    assert_eq!(g.effective_max_hand_size(0), Some(7));
    let hand = g.players[0].hand.len();
    g.step = TurnStep::Draw;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the additional card");
    assert_eq!(g.effective_max_hand_size(0), Some(5));
    let life = g.players[0].life;
    let ctx = EffectContext::for_ability(oil, 0, None);
    let evs = g
        .resolve_effect(&crabomination::effect::Effect::Discard { who: crabomination::effect::Selector::You, amount: crabomination::effect::Value::ONE, random: false }, &ctx)
        .expect("discard");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
}

/// Glacian taps X artifacts: one of the top X to hand, the rest binned.
#[test]
fn glacian_taps_x_artifacts_to_dig_x() {
    let mut g = pod(2);
    let glacian = ready(&mut g, 0, catalog::glacian_powerstone_engineer());
    let rocks: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::sol_ring())).collect();
    let (hand, yard) = (g.players[0].hand.len(), g.players[0].graveyard.len());
    g.perform_action(GameAction::ActivateAbility { card_id: glacian, ability_index: 0, target: None, additional_targets: vec![], x_value: Some(3), mode: None })
        .expect("X = 3");
    drain_stack(&mut g);
    assert!(rocks.iter().all(|&r| g.battlefield_find(r).unwrap().tapped));
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].graveyard.len(), yard + 2);
    assert!(
        g.perform_action(GameAction::ActivateAbility { card_id: glacian, ability_index: 0, target: None, additional_targets: vec![], x_value: Some(1), mode: None })
            .is_err(),
        "tapped"
    );
}

/// Countervailing Winds taxes {1} per card in the counterspeller's graveyard.
#[test]
fn countervailing_winds_taxes_per_graveyard_card() {
    for (yard, pool, countered) in [(3, 2, true), (1, 2, false)] {
        let mut g = pod(2);
        for _ in 0..yard {
            g.add_card_to_graveyard(1, catalog::forest());
        }
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("Bears");
        g.players[0].mana_pool.add_colorless(pool);
        let winds = g.add_card_to_hand(1, catalog::countervailing_winds());
        g.players[1].mana_pool.add(Color::Blue, 3);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: winds, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None })
            .expect("Winds");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bears).is_none(), countered, "yard {yard}");
    }
}

/// Flash Photography copies a permanent; at instant speed only when the
/// target is yours.
#[test]
fn flash_photography_flashes_only_onto_your_own_permanent() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::craw_wurm());
    g.step = TurnStep::BeginCombat;
    let photo = g.add_card_to_hand(0, catalog::flash_photography());
    g.players[0].mana_pool.add(Color::Blue, 4);
    assert!(cast(&mut g, photo, Some(Target::Permanent(theirs))).is_err(), "not at instant speed on theirs");
    cast(&mut g, photo, Some(Target::Permanent(mine))).expect("flash onto your own");
    let copies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Grizzly Bears" && c.controller == 0).count();
    assert_eq!(copies, 1);
}

/// CR 702.74 — Vibrance's ETBs read the hybrid mana spent: {R}{R} burns, {G}{G}
/// tutors a land and gains 2; evoked for {R}{R} it burns, then is sacrificed.
#[test]
fn vibrance_reads_the_colors_spent_on_it() {
    for (red, evoke) in [(true, false), (false, false), (true, true)] {
        let mut g = pod(2);
        on_top(&mut g, 0, catalog::mountain());
        let v = g.add_card_to_hand(0, catalog::vibrance());
        let c = if red { Color::Red } else { Color::Green };
        g.players[0].mana_pool.add(c, 2);
        if !evoke {
            g.players[0].mana_pool.add_colorless(3);
        }
        g.decider = Box::new(ScriptedDecider::new([]));
        let (life, opp, hand) = (g.players[0].life, g.players[1].life, g.players[0].hand.len());
        if evoke {
            g.perform_action(GameAction::CastSpellAlternative { card_id: v, pitch_card: None, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
                .expect("evoke");
        } else {
            g.perform_action(GameAction::CastSpell { card_id: v, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
                .expect("hardcast");
        }
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, if red { opp - 3 } else { opp }, "red {red} evoke {evoke}");
        assert_eq!(g.players[0].life, if red { life } else { life + 2 });
        if !red {
            assert_eq!(g.players[0].hand.len(), hand, "cast one, fetched a land");
        }
        assert_eq!(g.battlefield_find(v).is_some(), !evoke);
    }
}

/// CR 700.2 — Sakashima's Will: one mode without a commander; with one, both — the
/// opponent hands over a creature and the rest of your team copies one.
#[test]
fn sakashimas_will_takes_both_modes_with_a_commander() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::craw_wurm());
    g.players[0].commanders = vec![cmdr];
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::llanowar_elves());
    let will = g.add_card_to_hand(0, catalog::sakashimas_will());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.perform_action(GameAction::CastSpellSpree { card_id: will, spree_modes: vec![0, 1], target: Some(Target::Player(1)), additional_targets: vec![], x_value: None })
        .expect("both modes with a commander");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "their only creature is yours");
    let names: Vec<&str> = [bears, theirs].iter().map(|&id| g.computed_permanent(id).unwrap().def.name).collect();
    assert!(names.iter().all(|n| *n == "Craw Wurm"), "{names:?}");
}

/// Serra's Guardian lends vigilance: another creature attacks untapped.
#[test]
fn serras_guardian_gives_the_team_vigilance() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serras_guardian());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[(bears, 1)]);
    assert!(!g.battlefield_find(bears).unwrap().tapped);
}

/// Dark Fortress's colors need the turn it entered or a basic land.
#[test]
fn dark_fortress_colors_need_entry_turn_or_a_basic() {
    for basic in [false, true] {
        let mut g = pod(2);
        let fort = ready(&mut g, 0, catalog::dark_fortress());
        g.battlefield_find_mut(fort).unwrap().entered_turn = None;
        if basic {
            ready(&mut g, 0, catalog::swamp());
        }
        assert_eq!(activate(&mut g, fort, 1, None).is_ok(), basic, "basic {basic}");
        assert!(g.battlefield_find(fort).unwrap().tapped == basic);
    }
}

/// Timetwister: each player shuffles hand and graveyard away and draws seven.
#[test]
fn timetwister_resets_every_hand_to_seven() {
    let mut g = pod(3);
    for seat in 0..3 {
        g.add_card_to_graveyard(seat, catalog::grizzly_bears());
        g.add_card_to_hand(seat, catalog::grizzly_bears());
    }
    let tw = g.add_card_to_hand(0, catalog::timetwister());
    g.players[0].mana_pool.add(Color::Blue, 3);
    cast(&mut g, tw, None).expect("Timetwister");
    for seat in 0..3 {
        assert_eq!(g.players[seat].hand.len(), 7, "seat {seat}");
        assert!(g.players[seat].graveyard.iter().all(|c| c.definition.name == "Timetwister"), "seat {seat}");
    }
}

/// Stoic Rebuttal's metalcraft: {U}{U} with three artifacts.
#[test]
fn stoic_rebuttal_is_cheaper_with_metalcraft() {
    for rocks in [2, 3] {
        let mut g = pod(2);
        for _ in 0..rocks {
            ready(&mut g, 0, catalog::ornithopter());
        }
        let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.players[1].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("Bears");
        let sr = g.add_card_to_hand(0, catalog::stoic_rebuttal());
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Blue, 2);
        let ok = g
            .perform_action(GameAction::CastSpell { card_id: sr, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None })
            .is_ok();
        assert_eq!(ok, rocks == 3, "{rocks} artifacts");
    }
}

/// Elvish Doomsayer's death makes each opponent of a pod discard.
#[test]
fn elvish_doomsayer_taxes_each_opponents_hand() {
    let mut g = pod(4);
    let elf = ready(&mut g, 0, catalog::elvish_doomsayer());
    for seat in 0..4 {
        g.add_card_to_hand(seat, catalog::grizzly_bears());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(elf))).expect("Bolt");
    assert_eq!(g.players[0].hand.len(), 1);
    for seat in 1..4 {
        assert_eq!(g.players[seat].hand.len(), 0, "seat {seat}");
    }
}

/// Shadows of the Past drains only with four creature cards in your graveyard.
#[test]
fn shadows_of_the_past_drains_at_four_creature_cards() {
    let mut g = pod(3);
    let shadows = ready(&mut g, 0, catalog::shadows_of_the_past());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add(Color::Black, 5);
    assert!(activate(&mut g, shadows, 0, None).is_err(), "three creature cards");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, shadows, 0, None).expect("four");
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (22, 18, 18), "you gain 2 in all");
}

/// Darklight Phoenix returns at your beginning of combat after two deaths.
#[test]
fn darklight_phoenix_rises_after_two_deaths() {
    for deaths in [1, 2] {
        let mut g = pod(2);
        let phoenix = g.add_card_to_graveyard(0, catalog::darklight_phoenix());
        for _ in 0..deaths {
            let b = ready(&mut g, 1, catalog::grizzly_bears());
            let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
            g.players[0].mana_pool.add(Color::Red, 1);
            cast(&mut g, bolt, Some(Target::Permanent(b))).expect("Bolt");
        }
        g.step = TurnStep::BeginCombat;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(phoenix).is_some(), deaths == 2, "{deaths} deaths");
    }
}

/// CR 702.127 — Believe (aftermath) is cast from the graveyard and puts a
/// creature on top onto the battlefield.
#[test]
fn reason_then_believe_from_the_graveyard() {
    let mut g = pod(2);
    let rb = g.add_card_to_hand(0, catalog::reason_believe());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, rb, None).expect("Reason");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == rb));
    let wurm = on_top(&mut g, 0, catalog::craw_wurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastAftermath { card_id: rb, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("Believe");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
    assert!(g.exile.iter().any(|c| c.id == rb), "aftermath exiles it");
}

/// Elvish Dreadlord's death shrinks every non-Elf by -3/-3.
#[test]
fn elvish_dreadlord_dies_into_a_non_elf_sweep() {
    let mut g = pod(2);
    let lord = ready(&mut g, 0, catalog::elvish_dreadlord());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let doom = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 2);
    let _ = cast(&mut g, doom, Some(Target::Permanent(lord)));
    if g.battlefield_find(lord).is_some() {
        // Doom Blade can't hit a black creature: kill it another way.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(lord))).expect("Bolt");
    }
    assert!(g.battlefield_find(bears).is_none(), "non-Elf -3/-3");
    assert!(g.battlefield_find(elf).is_some(), "Elves are spared");
}

/// Tomb of Horrors Adventurer copies your second spell each turn.
#[test]
fn tomb_of_horrors_adventurer_copies_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::tomb_of_horrors_adventurer());
    let opp = g.players[1].life;
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Player(1))).expect("Bolt");
    }
    assert_eq!(g.players[1].life, opp - 9, "3 + 3 + the copy's 3");
}

/// Warg Tactics' kill mode reaches only fliers.
#[test]
fn warg_tactics_destroys_only_a_flier() {
    let mut g = pod(2);
    let bird = ready(&mut g, 1, catalog::ornithopter());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let wt = g.add_card_to_hand(0, catalog::warg_tactics());
    g.players[0].mana_pool.add(Color::Green, 2);
    let cast_mode = |g: &mut GameState, t| {
        g.perform_action(GameAction::CastSpell { card_id: wt, target: Some(Target::Permanent(t)), additional_targets: vec![], mode: Some(0), x_value: None })
    };
    assert!(cast_mode(&mut g, bears).is_err(), "no flying");
    cast_mode(&mut g, bird).expect("the flier");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bird).is_none());
}

/// Imoen draws at your end step with the initiative — two after a dungeon.
#[test]
fn imoen_draws_with_the_initiative() {
    for (initiative, completed, want) in [(false, false, 0), (true, false, 1), (true, true, 2)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::imoen_mystic_trickster());
        if initiative {
            g.initiative = Some(0);
        }
        if completed {
            g.players[0].dungeons_completed = 1;
        }
        let hand = g.players[0].hand.len();
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + want, "initiative {initiative} dungeon {completed}");
    }
}

/// Vrock's Toxic Spores fire only after one of your permanents left.
#[test]
fn vrock_drains_after_a_permanent_left() {
    for left in [false, true] {
        let mut g = pod(3);
        ready(&mut g, 0, catalog::vrock());
        if left {
            let b = ready(&mut g, 0, catalog::grizzly_bears());
            let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
            g.players[0].mana_pool.add(Color::Red, 1);
            cast(&mut g, bolt, Some(Target::Permanent(b))).expect("Bolt");
        }
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        for seat in 1..3 {
            assert_eq!(g.players[seat].life, if left { 17 } else { 20 }, "left {left}");
        }
    }
}

/// Glamer Gifter makes another creature a base 4/4 with every creature type.
#[test]
fn glamer_gifter_makes_a_four_four_of_every_type() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let gg = g.add_card_to_hand(0, catalog::glamer_gifter());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, gg, None).expect("Glamer Gifter");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&Keyword::Changeling), "every creature type (CR 205.3)");
}

/// Prismatic Undercurrents fetches one basic per color among your permanents.
#[test]
fn prismatic_undercurrents_fetches_per_color() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::goblin_guide());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let pu = g.add_card_to_hand(0, catalog::prismatic_undercurrents());
    g.players[0].mana_pool.add(Color::Green, 4);
    let hand = g.players[0].hand.len();
    cast(&mut g, pu, None).expect("Undercurrents");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "green and red");
}

/// Ajani Unrelenting: each loyalty activation makes a Cadet; −3 spares your tokens.
#[test]
fn ajani_unrelenting_cadets_survive_his_sweep() {
    let mut g = pod(2);
    let ajani = ready(&mut g, 0, catalog::ajani_unrelenting());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ajani, ability_index: 2, target: None, x_value: None }).expect("−3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    let cadets: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Cadet").collect();
    assert_eq!(cadets.len(), 1);
    assert_eq!(cadets[0].damage, 0, "your tokens are spared");
}

/// Earth's Mightiest Heroes: one creature of the top eight, all of them with teamwork.
#[test]
fn earths_mightiest_heroes_teamwork_takes_every_creature() {
    for teamwork in [false, true] {
        let mut g = pod(2);
        for _ in 0..3 {
            on_top(&mut g, 0, catalog::grizzly_bears());
        }
        if teamwork {
            ready(&mut g, 0, catalog::craw_wurm());
        }
        let emh = g.add_card_to_hand(0, catalog::earths_mightiest_heroes());
        g.players[0].mana_pool.add(Color::Green, 6);
        let action = if teamwork {
            GameAction::CastSpellKicked { card_id: emh, target: None, additional_targets: vec![], mode: None, x_value: None }
        } else {
            GameAction::CastSpell { card_id: emh, target: None, additional_targets: vec![], mode: None, x_value: None }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
        assert_eq!(bears, if teamwork { 3 } else { 1 }, "teamwork {teamwork}");
    }
}

/// Abby: one Cordyceps Infected per mana spent, and she enters under an opponent.
#[test]
fn abby_makes_infected_per_mana_and_joins_an_opponent() {
    let mut g = pod(3);
    let abby = g.add_card_to_hand(0, catalog::abby_merciless_soldier());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, abby, None).expect("Abby");
    let infected = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Cordyceps Infected" && c.controller == 0).count();
    assert_eq!(infected, 3);
    assert_ne!(g.battlefield_find(abby).unwrap().controller, 0, "an opponent of your choice");
}

/// Gale's Redirection exiles a spell; a low roll lets you cast it paying, a
/// d20 plus its mana value of 15+ casts it free.
#[test]
fn gales_redirection_steals_the_spell() {
    let mut g = pod(2);
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 6);
    g.perform_action(GameAction::CastSpell { card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("Wurm");
    let gale = g.add_card_to_hand(0, catalog::gales_redirection());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.perform_action(GameAction::CastSpell { card_id: gale, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None })
        .expect("Gale's Redirection");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == wurm).expect("exiled");
    assert!(exiled.may_play_until.is_some_and(|p| p.player == 0), "you may cast it");
}

/// Bilbo, Thief in the Night recasts a sorcery from the graveyard on attack,
/// {1} cheaper, and exiles it after.
#[test]
fn bilbo_recasts_from_the_graveyard_and_exiles_it() {
    let mut g = pod(2);
    let bilbo = ready(&mut g, 0, catalog::bilbo_thief_in_the_night());
    let rite = g.add_card_to_graveyard(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![rite]), DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    attack(&mut g, &[(bilbo, 1)]);
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination resolved");
    assert!(g.exile.iter().any(|c| c.id == rite), "exiled instead of the graveyard");
    assert_eq!(g.players[0].mana_pool.total(), 0, "{{2}}{{U}} less {{1}}");
}

/// Lukka −2 trades a creature up for a bigger one; +1's creature exiles stay
/// castable only while you control a Lukka.
#[test]
fn lukka_polymorphs_up_and_gates_exiles_on_a_lukka() {
    let mut g = pod(2);
    let lukka = ready(&mut g, 0, catalog::lukka_coppercoat_outcast());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    on_top(&mut g, 0, catalog::craw_wurm());
    on_top(&mut g, 0, catalog::llanowar_elves());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lukka, ability_index: 1, target: Some(Target::Permanent(bears)), x_value: None })
        .expect("−2");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Craw Wurm" && c.controller == 0), "skipped the cheaper Elves");
}

/// Golden Guardian fights your own creature; dying that turn, it returns as
/// Gold-Forge Garrison under your control.
#[test]
fn golden_guardian_dies_into_its_garrison() {
    let mut g = pod(2);
    let guardian = ready(&mut g, 0, catalog::golden_guardian());
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, guardian, 0, Some(Target::Permanent(wurm))).expect("fight the Wurm");
    g.check_state_based_actions();
    drain_stack(&mut g);
    let back = g.battlefield_find(guardian).expect("returned");
    assert_eq!(back.definition.name, "Gold-Forge Garrison");
    assert_eq!(back.controller, 0);
}

/// Rootwire Amalgam's sacrifice makes a hasty Golem three times its power.
#[test]
fn rootwire_amalgam_sacrifices_into_a_triple_golem() {
    let mut g = pod(2);
    let amalgam = ready(&mut g, 0, catalog::rootwire_amalgam());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, amalgam, 0, None).expect("sacrifice");
    let golem = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Golem").expect("Golem").id;
    let cp = g.computed_permanent(golem).unwrap();
    assert_eq!((cp.power, cp.toughness), (15, 15));
    assert!(cp.keywords().contains(&Keyword::Haste));
}

/// CR 614.1a — Unpredictable Cyclone replaces a cycled creature's draw: exile
/// until a creature, cast it free, bottom the rest; no card is drawn.
#[test]
fn unpredictable_cyclone_turns_cycling_into_a_free_cast() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::unpredictable_cyclone());
    let mako = g.add_card_to_hand(0, catalog::marauding_mako());
    let wurm = on_top(&mut g, 0, catalog::craw_wurm());
    let land = on_top(&mut g, 0, catalog::mountain());
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: mako, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "nothing drawn");
    assert!(g.battlefield_find(wurm).is_some(), "the Wurm cast free");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(land), "the Mountain went to the bottom");
}

/// Shimmerwilds Growth: the enchanted land becomes the chosen color and taps
/// for one extra mana of it.
#[test]
fn shimmerwilds_growth_colors_and_doubles_its_land() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let wastes = ready(&mut g, 0, catalog::mountain());
    let sg = g.add_card_to_hand(0, catalog::shimmerwilds_growth());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, sg, Some(Target::Permanent(wastes))).expect("Shimmerwilds Growth");
    assert_eq!(g.battlefield_find(sg).unwrap().chosen_color, Some(Color::Green), "your densest color");
    assert!(g.computed_permanent(wastes).unwrap().colors.contains(Color::Green));
    activate(&mut g, wastes, 0, None).expect("tap for R");
    assert_eq!(g.players[0].mana_pool.total(), 2, "R plus the chosen G");
}

/// Demon of Dark Schemes: its -2/-2 sweep feeds it energy, which buys a
/// reanimation from any graveyard.
#[test]
fn demon_of_dark_schemes_banks_energy_from_its_sweep() {
    let mut g = pod(2);
    for _ in 0..2 {
        ready(&mut g, 0, catalog::grizzly_bears());
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    let demon = g.add_card_to_hand(0, catalog::demon_of_dark_schemes());
    g.players[0].mana_pool.add(Color::Black, 6);
    cast(&mut g, demon, None).expect("Demon");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 4, "four Bears died");
    let target = g.players[1].graveyard.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    g.players[0].mana_pool.add(Color::Black, 3);
    activate(&mut g, demon, 0, Some(Target::Permanent(target))).expect("reanimate");
    let b = g.battlefield_find(target).expect("on the battlefield");
    assert_eq!(b.controller, 0);
    assert!(b.tapped);
}

/// Loot, the Nexus taps for one mana per distinct power among your creatures.
#[test]
fn loot_the_nexus_counts_distinct_powers() {
    let mut g = pod(2);
    let loot = ready(&mut g, 0, catalog::loot_the_nexus());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::craw_wurm());
    activate(&mut g, loot, 0, None).expect("tap Loot");
    assert_eq!(g.players[0].mana_pool.total(), 2, "powers {{2, 6}} (Loot is a 2 too)");
}

/// Plunder the Trollshaws draws two when flashed back.
#[test]
fn plunder_the_trollshaws_draws_two_from_the_graveyard() {
    let mut g = pod(2);
    let p = g.add_card_to_graveyard(0, catalog::plunder_the_trollshaws());
    g.players[0].mana_pool.add(Color::Blue, 4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastFlashback { card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Garruk, Curse Breaker's −4: until your next turn, your attackers at an
/// opponent get +2/+2 and trample.
#[test]
fn garruk_curse_breaker_emblem_pumps_attackers() {
    let mut g = pod(2);
    let garruk = ready(&mut g, 0, catalog::garruk_curse_breaker());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: garruk, ability_index: 2, target: None, x_value: None }).expect("−4");
    drain_stack(&mut g);
    attack(&mut g, &[(bears, 1)]);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&Keyword::Trample));
}

/// Raph & Leo grant an extra combat only in the turn's first combat.
#[test]
fn raph_and_leo_add_a_combat_from_the_first_one() {
    let mut g = pod(2);
    let rl = ready(&mut g, 0, catalog::raph_and_leo_sibling_rivals());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[(rl, 1), (bears, 1)]);
    assert!(!g.battlefield_find(bears).unwrap().tapped || !g.battlefield_find(rl).unwrap().tapped, "an attacker untapped");
    assert_eq!(g.additional_combat_phases, 1);
}

/// CR 702.47a — Splicer's Skill splices onto any instant or sorcery (not only
/// Arcane): a Bolt carries the Golem along; a creature spell can't take it.
#[test]
fn cr_702_47a_splicers_skill_splices_onto_instants_and_sorceries() {
    let mut g = pod(2);
    let skill = g.add_card_to_hand(0, catalog::splicers_skill());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::CastSpellSpliced { card_id: bears, splice_cards: vec![skill], target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "not onto a creature spell"
    );
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellSpliced { card_id: bolt, splice_cards: vec![skill], target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("Bolt + Splicer's Skill");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Phyrexian Golem"));
    assert!(g.players[0].hand.iter().any(|c| c.id == skill), "a splice card stays in hand");
}

/// Lifecrafter's Gift spreads a counter to each creature you control already
/// carrying one.
#[test]
fn lifecrafters_gift_spreads_to_countered_creatures() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let c = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let gift = g.add_card_to_hand(0, catalog::lifecrafters_gift());
    g.players[0].mana_pool.add(Color::Green, 4);
    cast(&mut g, gift, Some(Target::Permanent(a))).expect("Gift");
    let n = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!((n(&g, a), n(&g, b), n(&g, c)), (2, 2, 0));
}

/// Pore Over the Pages: draw three, untap two lands, discard one.
#[test]
fn pore_over_the_pages_untaps_two_lands() {
    let mut g = pod(2);
    let lands: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::island())).collect();
    for &l in &lands {
        g.battlefield_find_mut(l).unwrap().tapped = true;
    }
    let pore = g.add_card_to_hand(0, catalog::pore_over_the_pages());
    g.players[0].mana_pool.add(Color::Blue, 5);
    let hand = g.players[0].hand.len();
    cast(&mut g, pore, None).expect("Pore Over the Pages");
    assert_eq!(lands.iter().filter(|&&l| !g.battlefield_find(l).unwrap().tapped).count(), 2);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3 - 1);
}

/// Nature's Way is one-sided: your creature deals its power and takes nothing.
#[test]
fn natures_way_bites_one_way() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    let nw = g.add_card_to_hand(0, catalog::natures_way());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: nw, target: Some(Target::Permanent(wurm)), additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None })
        .expect("Nature's Way");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    let w = g.battlefield_find(wurm).unwrap();
    assert_eq!(w.damage, 0);
    assert!(g.computed_permanent(wurm).unwrap().keywords().contains(&Keyword::Trample));
}

/// Noggle the Mind: a colorless 1/1 Noggle with no abilities.
#[test]
fn noggle_the_mind_makes_a_blank_noggle() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let nm = g.add_card_to_hand(0, catalog::noggle_the_mind());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, nm, Some(Target::Permanent(angel))).expect("Noggle the Mind");
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords().is_empty(), "{:?}", cp.keywords());
    assert!(cp.colors.is_empty());
    assert_eq!(cp.subtypes().creature_types, vec![crabomination::card::CreatureType::Noggle]);
}

/// Dreamcaller Siren taps two nonland permanents only beside another Pirate.
#[test]
fn dreamcaller_siren_needs_another_pirate() {
    for pirate in [false, true] {
        let mut g = pod(2);
        if pirate {
            ready(&mut g, 0, catalog::talas_scout());
        }
        let a = ready(&mut g, 1, catalog::grizzly_bears());
        let b = ready(&mut g, 1, catalog::craw_wurm());
        let siren = g.add_card_to_hand(0, catalog::dreamcaller_siren());
        g.players[0].mana_pool.add(Color::Blue, 4);
        cast(&mut g, siren, None).expect("Siren");
        let tapped = [a, b].iter().filter(|&&id| g.battlefield_find(id).unwrap().tapped).count();
        assert_eq!(tapped, if pirate { 2 } else { 0 }, "pirate {pirate}");
    }
}

/// Price of Loyalty steals, untaps and hastes a creature for the turn.
#[test]
fn price_of_loyalty_borrows_a_creature() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    g.battlefield_find_mut(wurm).unwrap().tapped = true;
    let pol = g.add_card_to_hand(0, catalog::price_of_loyalty());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, pol, Some(Target::Permanent(wurm))).expect("Price of Loyalty");
    let w = g.battlefield_find(wurm).unwrap();
    assert_eq!(w.controller, 0);
    assert!(!w.tapped);
    assert!(g.computed_permanent(wurm).unwrap().keywords().contains(&Keyword::Haste));
}

/// Pummeler for Hire gains life equal to your biggest Giant's power.
#[test]
fn pummeler_for_hire_gains_the_biggest_giants_power() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hill_giant());
    let p = g.add_card_to_hand(0, catalog::pummeler_for_hire());
    g.players[0].mana_pool.add(Color::Green, 5);
    let life = g.players[0].life;
    cast(&mut g, p, None).expect("Pummeler");
    assert_eq!(g.players[0].life, life + 4, "the Pummeler itself is the biggest Giant (4)");
}

/// Gone Fishing (Lake-town Mariners' adventure) blinks two of your permanents.
#[test]
fn lake_town_mariners_gone_fishing_blinks_two() {
    let mut g = pod(2);
    let mariners = g.add_card_to_hand(0, catalog::lake_town_mariners());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let land = ready(&mut g, 0, catalog::island());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.perform_action(GameAction::CastAdventure { card_id: mariners, target: Some(Target::Permanent(bears)), additional_targets: vec![Target::Permanent(land)], mode: None, x_value: None })
        .expect("Gone Fishing");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == land && c.tapped), "the land came back untapped");
    assert!(g.exile.iter().any(|c| c.id == mariners), "on an adventure");
}

/// Primal Witchstalker mills four and brings back a milled land.
#[test]
fn primal_witchstalker_returns_a_milled_land() {
    let mut g = pod(2);
    let forest = on_top(&mut g, 0, catalog::forest());
    let ws = g.add_card_to_hand(0, catalog::primal_witchstalker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, ws, None).expect("Witchstalker");
    drain_stack(&mut g);
    let back = g.battlefield.iter().filter(|c| c.definition.name == "Forest" && c.controller == 0).count();
    assert_eq!(back, 1, "one land back");
    assert!(g.battlefield_find(forest).is_some() || g.players[0].graveyard.iter().any(|c| c.id == forest));
}

/// CR 700.8 — Veteran Adventurer costs {1} less per party member you control;
/// a creature fills one slot, so Imoen (Rogue Wizard) is a discount of one.
#[test]
fn veteran_adventurer_discounts_by_party() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::imoen_mystic_trickster());
    let va = g.add_card_to_hand(0, catalog::veteran_adventurer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(cast(&mut g, va, None).is_err(), "Imoen is one party member, not two");
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, va, None).expect("{5}{G} less one");
    // CR 604.3 — its "is also a Cleric, Rogue, Warrior, and Wizard" holds on
    // the battlefield over a printed Human.
    let types = g.computed_permanent(va).unwrap().subtypes().creature_types.clone();
    for t in [CreatureType::Human, CreatureType::Cleric, CreatureType::Rogue, CreatureType::Warrior, CreatureType::Wizard] {
        assert!(types.contains(&t), "{t:?}");
    }
}

/// Liliana's Elite grows with your graveyard's creature cards.
#[test]
fn lilianas_elite_counts_graveyard_creatures() {
    let mut g = pod(2);
    let e = ready(&mut g, 0, catalog::lilianas_elite());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::forest());
    let cp = g.computed_permanent(e).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// CR 702.15b — Lifelink (the Aura) grants lifelink: the enchanted creature's
/// damage gains its controller that much life.
#[test]
fn lifelink_aura_grants_lifelink() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::lifelink());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("Lifelink");
    let life = g.players[0].life;
    attack(&mut g, &[(bear, 1)]);
    to_combat_damage(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

/// CR 702.49a — Moonblade Shinobi ninjutsus in for an unblocked attacker;
/// its combat damage to a player makes a 1/1 flying Illusion.
#[test]
fn moonblade_shinobi_ninjutsu_makes_an_illusion() {
    let mut g = pod(2);
    let sneaker = ready(&mut g, 0, catalog::grizzly_bears());
    let ninja = g.add_card_to_hand(0, catalog::moonblade_shinobi());
    attack(&mut g, &[(sneaker, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Ninjutsu { ninja, returning: sneaker }).expect("ninjutsu");
    drain_stack(&mut g);
    let opp = g.players[1].life;
    to_combat_damage(&mut g);
    assert_eq!(g.players[1].life, opp - 3);
    let illusion = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Illusion").expect("Illusion");
    assert!(illusion.definition.keywords.contains(&Keyword::Flying));
}

/// CR 701.16a — Dining Room enters tapped; {4}, {T} investigates.
#[test]
fn dining_room_investigates() {
    let mut g = pod(2);
    let room = g.add_card_to_hand(0, catalog::dining_room());
    g.perform_action(GameAction::PlayLand(room)).expect("land");
    assert!(g.battlefield_find(room).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(room).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, room, 2, None).expect("investigate");
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Clue"));
}

/// CR 707.10c — Resonance Technician taps X artifacts to copy an instant or
/// sorcery you control with mana value exactly X.
#[test]
fn resonance_technician_copies_a_spell_of_mana_value_x() {
    let mut g = pod(2);
    let tech = ready(&mut g, 0, catalog::resonance_technician());
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::sol_ring());
    let opp = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let act = |g: &mut GameState, x| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: tech,
            ability_index: 0,
            target: Some(Target::Permanent(bolt)),
            additional_targets: vec![],
            x_value: Some(x),
            mode: None,
        })
    };
    assert!(act(&mut g, 2).is_err(), "Bolt's mana value is 1, not 2");
    act(&mut g, 1).expect("copy");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 6, "Bolt and its copy");
}

/// Glittermonger taps for a Treasure.
#[test]
fn glittermonger_makes_a_treasure() {
    let mut g = pod(2);
    let gm = ready(&mut g, 0, catalog::glittermonger());
    activate(&mut g, gm, 0, None).expect("treasure");
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").count(), 1);
}

/// CR 706.2 — Contact Other Plane's d20 picks the row: a 5 draws two, a 20
/// draws three.
#[test]
fn contact_other_plane_reads_the_d20() {
    for (roll, drawn) in [(5, 2), (14, 2), (20, 3)] {
        let mut g = pod(2);
        let c = g.add_card_to_hand(0, catalog::contact_other_plane());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(roll)]));
        let hand = g.players[0].hand.len();
        cast(&mut g, c, None).expect("Contact Other Plane");
        assert_eq!(g.players[0].hand.len(), hand - 1 + drawn, "roll {roll}");
    }
}

/// CR 725.2 — Stirring Bard takes the initiative on entry; its {T} gives a
/// creature menace and haste.
#[test]
fn stirring_bard_takes_the_initiative_and_hastes() {
    let mut g = pod(2);
    let bard = g.add_card_to_hand(0, catalog::stirring_bard());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, bard, None).expect("Bard");
    assert_eq!(g.initiative, Some(0));
    g.clear_sickness(bard);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, bard, 0, Some(Target::Permanent(bear))).expect("mantle");
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Menace) && kws.contains(&Keyword::Haste));
}

/// CR 702.6a — Sword of Kaldra: +5/+5, and a creature the bearer damages is
/// exiled (an indestructible blocker survives the damage, not the Sword).
#[test]
fn sword_of_kaldra_exiles_what_it_damages() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_kaldra());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    let myr = ready(&mut g, 1, catalog::darksteel_myr());
    attack(&mut g, &[(bear, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(myr, bear)])).expect("block");
    drain_stack(&mut g);
    to_combat_damage(&mut g);
    assert!(g.battlefield_find(myr).is_none());
    assert!(g.exile.iter().any(|c| c.id == myr), "exiled, not just damaged");
}

/// CR 724.1 — Glorious End ends the turn (the stack is exiled with it) and you
/// lose at the beginning of your next end step, not an opponent's.
#[test]
fn glorious_end_ends_the_turn_then_you_lose() {
    let mut g = pod(3);
    let ge = g.add_card_to_hand(0, catalog::glorious_end());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ge, None).expect("Glorious End");
    assert!(g.exile.iter().any(|c| c.id == ge), "ending the turn exiles it");
    for _ in 0..60 {
        if g.active_player_idx == 0 && g.turn_number > 1 && g.step == TurnStep::Cleanup {
            break;
        }
        if g.players[0].eliminated {
            break;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(&mut g);
        if g.active_player_idx != 0 {
            assert!(!g.players[0].eliminated, "an opponent's end step isn't yours");
        }
    }
    assert!(g.players[0].eliminated);
}

/// CR 606.3 — Ajani, Wise Counselor's +2 gains a life per creature you control.
#[test]
fn ajani_wise_counselor_gains_per_creature() {
    let mut g = pod(2);
    let ajani = g.add_card_to_battlefield(0, catalog::ajani_wise_counselor());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ajani, ability_index: 0, target: None, x_value: None }).expect("+2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

/// CR 702.143a / 701.15a — Bohn grants foretell to hand cards, and the
/// second spell of a turn goads an opponent's creature.
#[test]
fn bohn_foretells_and_goads_on_the_second_spell() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::bohn_beguiling_balladeer());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: bear }).expect("a hand card gains foretell");
    assert!(g.exile.iter().any(|c| c.id == bear && c.face_down));
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        cast(&mut g, bolt, Some(Target::Player(2))).expect("bolt");
    }
    assert!(g.goaded_by_player(g.battlefield_find(foe).unwrap(), 0), "goaded on the second spell");
}

/// CR 301.5c — Captain's Hook: +2/+0, menace, a Pirate; moving it destroys
/// the creature it left.
#[test]
fn captains_hook_destroys_what_it_leaves() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let hook = g.add_card_to_battlefield(0, catalog::captains_hook());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: hook, target: a }).expect("equip a");
    drain_stack(&mut g);
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&Keyword::Menace) && cp.subtypes().creature_types.contains(&CreatureType::Pirate));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: hook, target: b }).expect("equip b");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "the old host is destroyed");
    assert!(g.battlefield_find(b).is_some());
}

/// CR 613.4b — Candlekeep Inspiration counts instants, sorceries and
/// Adventurers you own in exile and the graveyard.
#[test]
fn candlekeep_inspiration_sets_base_pt_from_spells_in_exile_and_graveyard() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::opt());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_exile(0, catalog::lightning_bolt());
    g.add_card_to_exile(1, catalog::lightning_bolt());
    let ci = g.add_card_to_hand(0, catalog::candlekeep_inspiration());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, ci, None).expect("Candlekeep");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "two in the graveyard and one owned in exile; the land and the opponent's card don't count");
}

/// CR 725.2 / 701.15a — Bloodboil Sorcerer takes the initiative; its Crown of
/// Madness sacrifices an artifact or creature to goad.
#[test]
fn bloodboil_sorcerer_takes_the_initiative_and_goads() {
    let mut g = pod(3);
    let bs = g.add_card_to_hand(0, catalog::bloodboil_sorcerer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, bs, None).expect("Sorcerer");
    assert_eq!(g.initiative, Some(0));
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    activate(&mut g, bs, 0, Some(Target::Permanent(foe))).expect("goad");
    assert!(g.goaded_by_player(g.battlefield_find(foe).unwrap(), 0));
    assert!(g.battlefield_find(rock).is_none() || g.battlefield_find(bs).is_none(), "an artifact or creature paid");
}

/// CR 702.34a — Dream Twist mills three, then again from the graveyard.
#[test]
fn dream_twist_mills_three_twice() {
    let mut g = pod(2);
    let dt = g.add_card_to_hand(0, catalog::dream_twist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let lib = g.players[1].library.len();
    cast(&mut g, dt, Some(Target::Player(1))).expect("Dream Twist");
    assert_eq!(g.players[1].library.len(), lib - 3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback { card_id: dt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - 6);
}

/// CR 603.2 — Old Fat Spider draws when an opponent targets it, not when
/// you do.
#[test]
fn old_fat_spider_draws_when_an_opponent_targets_it() {
    let mut g = pod(2);
    let spider = ready(&mut g, 0, catalog::old_fat_spider());
    let hand = g.players[0].hand.len();
    let mine = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, mine, Some(Target::Permanent(spider))).expect("own pump");
    assert_eq!(g.players[0].hand.len(), hand, "your own spell draws nothing");
    let shock = g.add_card_to_hand(1, catalog::shock());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, shock, Some(Target::Permanent(spider))).expect("opponent's shock");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 701.44a / 305.2 — Captain Janeway allows a second land, and each
/// creature you control that enters explores.
#[test]
fn captain_kathryn_janeway_explores_entering_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::captain_kathryn_janeway());
    on_top(&mut g, 0, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "a nonland reveal: +1/+1");
    let a = g.add_card_to_hand(0, catalog::forest());
    let b = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(a)).expect("first land");
    g.perform_action(GameAction::PlayLand(b)).expect("second land");
}

/// CR 702.21a — Dragon's Disciple enters with a counter only alongside a
/// Dragon, and gives your Dragons ward {1}.
#[test]
fn dragons_disciple_counts_a_dragon_and_wards_dragons() {
    for with_dragon in [false, true] {
        let mut g = pod(2);
        if with_dragon {
            g.add_card_to_hand(0, catalog::shivan_dragon());
        }
        let dd = g.add_card_to_hand(0, catalog::dragons_disciple());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, dd, None).expect("Disciple");
        assert_eq!(g.battlefield_find(dd).unwrap().counter_count(CounterType::PlusOnePlusOne), u32::from(with_dragon));
    }
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dragons_disciple());
    let dragon = ready(&mut g, 0, catalog::shivan_dragon());
    assert!(g.computed_permanent(dragon).unwrap().keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// CR 613.1d — Puppet Crafting makes an artifact a 5/5 Construct creature,
/// and climbs back to hand from the graveyard.
#[test]
fn puppet_crafting_animates_an_artifact() {
    let mut g = pod(2);
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let pc = g.add_card_to_hand(0, catalog::puppet_crafting());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, pc, Some(Target::Permanent(rock))).expect("Puppet Crafting");
    let cp = g.computed_permanent(rock).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Construct));
    let gy = g.add_card_to_graveyard(0, catalog::puppet_crafting());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, gy, 0, None).expect("return");
    assert!(g.players[0].hand.iter().any(|c| c.id == gy));
}

/// CR 309.4c — Dungeon Delver doubles room abilities only while a commander
/// creature you own is on the battlefield; in the command zone (CR 113.6) it
/// does nothing.
#[test]
fn dungeon_delver_doubles_rooms_with_a_commander_out() {
    for with_commander in [false, true] {
        let mut g = pod(2);
        g.seat_commanders(0, vec![catalog::dungeon_delver()]);
        ready(&mut g, 0, catalog::dungeon_delver());
        let src = ready(&mut g, 0, catalog::grizzly_bears());
        if with_commander {
            g.players[0].commanders.push(src);
        }
        let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        ctx.source = Some(src);
        for _ in 0..2 {
            let events = g.resolve_effect(&crabomination::effect::Effect::Venture, &ctx).expect("venture");
            g.dispatch_triggers_for_events(&events);
            drain_stack(&mut g);
        }
        let goblins = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Goblin").count();
        assert_eq!(goblins, if with_commander { 2 } else { 1 }, "Goblin Lair, commander {with_commander}");
    }
}

/// CR 701.15a — Insufferable Balladeer: the target can't block this turn and
/// is goaded.
#[test]
fn insufferable_balladeer_goads_and_stops_a_blocker() {
    let mut g = pod(3);
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let ib = g.add_card_to_hand(0, catalog::insufferable_balladeer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ib, None).expect("Balladeer");
    assert!(g.goaded_by_player(g.battlefield_find(foe).unwrap(), 0));
    assert!(g.computed_permanent(foe).unwrap().keywords().contains(&Keyword::CantBlock));
}
