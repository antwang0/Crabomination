//! Commander: the Necron Dynasties precon (40K, Szarekh, the Silent King,
//! `decks::cmdr_szarekh`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    g.players[0].mana_pool.add(Color::Black, 20);
    g.players[0].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, target: Option<Target>, x_value: Option<u32>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, id, target, None)
}

fn activate_x(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x_value: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: index, target, additional_targets: vec![], x_value, mode: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn attack(g: &mut GameState, attackers: &[CardId], seat: usize) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    let attacks = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(seat) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn run_combat_out(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Resolve `effect` as seat 0's and fire the triggers it caused.
fn run(g: &mut GameState, effect: &Effect) {
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn destroy(g: &mut GameState, id: CardId) {
    run(g, &Effect::Destroy { what: Selector::ExactObjects(vec![id]) });
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

/// Szarekh mills three on attack and takes an artifact creature milled.
#[test]
fn szarekh_takes_a_milled_artifact_creature() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    let flayed = g.add_card_to_library(0, catalog::flayed_one());
    let szarekh = g.add_card_to_battlefield(0, catalog::szarekh_the_silent_king());
    attack(&mut g, &[szarekh], 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == flayed), "the Necron went to hand");
    assert_eq!(g.players[0].graveyard.len(), 2, "the two Islands stay milled");
}

/// Anrakyr casts an artifact from the graveyard by paying life equal to its
/// mana value (CR 118.9 alternative cost, CR 119.4 life payment).
#[test]
fn anrakyr_casts_from_graveyard_for_life() {
    let mut g = main_phase(2);
    let spyder = g.add_card_to_graveyard(0, catalog::canoptek_spyder());
    let anrakyr = g.add_card_to_battlefield(0, catalog::anrakyr_the_traveller());
    attack(&mut g, &[anrakyr], 1);
    assert!(g.battlefield_find(spyder).is_some(), "cast from the graveyard");
    assert_eq!(g.players[0].life, 15, "paid five life, no mana");
}

/// Biotransference makes your creatures artifacts, and each artifact or
/// creature spell you cast costs 1 life for a Warrior.
#[test]
fn biotransference_turns_flesh_to_metal() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::biotransference());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().card_types().contains(&CardType::Artifact));
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, bear2, None).expect("cast");
    assert_eq!(count(&g, 0, "Necron Warrior"), 1);
    assert_eq!(g.players[0].life, 19);
}

/// Canoptek Scarab Swarm exiles a graveyard and makes an Insect per artifact
/// or land card exiled.
#[test]
fn canoptek_scarab_swarm_eats_a_graveyard() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(1, catalog::sol_ring());
    g.add_card_to_graveyard(1, catalog::island());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let swarm = g.add_card_to_hand(0, catalog::canoptek_scarab_swarm());
    cast(&mut g, swarm, Some(Target::Player(1))).expect("cast");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(count(&g, 0, "Insect"), 2);
}

/// CR 702.84a / 603.6a — Canoptek Tomb Sentinel's exile fires when it enters
/// from the graveyard (unearth), not when cast.
#[test]
fn tomb_sentinel_exiles_only_from_the_graveyard() {
    let mut g = main_phase(2);
    let foe = g.add_card_to_battlefield(1, catalog::sol_ring());
    let cast_one = g.add_card_to_hand(0, catalog::canoptek_tomb_sentinel());
    cast(&mut g, cast_one, None).expect("cast");
    assert!(g.battlefield_find(foe).is_some(), "cast from hand: no exile");
    let sentinel = g.add_card_to_graveyard(0, catalog::canoptek_tomb_sentinel());
    flood(&mut g);
    activate_x(&mut g, sentinel, 0, Some(Target::Permanent(foe)), None).expect("unearth {7}");
    assert!(g.exile.iter().any(|c| c.id == foe), "entered from a graveyard");
}

/// Convergence of Dominion — while you control your commander, unearth costs
/// {2} less, never below one mana.
#[test]
fn convergence_of_dominion_discounts_unearth() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::convergence_of_dominion());
    let praetorian = g.add_card_to_graveyard(0, catalog::triarch_praetorian());
    library(&mut g, 0, 3);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(activate_x(&mut g, praetorian, 0, None, None).is_err(), "no commander: full {{4}}{{B}}");
    let cmdr = g.add_card_to_battlefield(0, catalog::szarekh_the_silent_king());
    g.players[0].commanders.push(cmdr);
    activate_x(&mut g, praetorian, 0, None, None).expect("{2}{B} with the commander out");
    assert!(g.battlefield_find(praetorian).is_some());
}

/// Cryptek brings a dying artifact creature back tapped.
#[test]
fn cryptek_reanimates_a_dying_necron() {
    let mut g = main_phase(2);
    let cryptek = g.add_card_to_battlefield(0, catalog::cryptek());
    g.clear_sickness(cryptek);
    let flayed = g.add_card_to_battlefield(0, catalog::flayed_one());
    flood(&mut g);
    activate_x(&mut g, cryptek, 0, Some(Target::Permanent(flayed)), None).expect("{1}{B}, {T}");
    destroy(&mut g, flayed);
    let back = g.battlefield_find(flayed).expect("returned");
    assert!(back.tapped);
}

/// Ghost Ark — crewing it gives graveyard artifact creatures unearth {3}
/// until end of turn; the grant ends at cleanup.
#[test]
fn ghost_ark_repair_barge() {
    let mut g = main_phase(2);
    let ark = g.add_card_to_battlefield(0, catalog::ghost_ark());
    let crew = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spyder = g.add_card_to_graveyard(0, catalog::canoptek_spyder());
    g.perform_action(GameAction::Crew { vehicle: ark, crew_creatures: vec![crew] }).expect("crew 2");
    drain_stack(&mut g);
    g.players[0].mana_pool.add_colorless(3);
    activate_x(&mut g, spyder, 0, None, None).expect("unearth {3}");
    assert!(g.battlefield_find(spyder).is_some());
}

/// Illuminor Szeras sacrifices a creature for {B} per its mana value.
#[test]
fn illuminor_szeras_secrets_of_the_soul() {
    let mut g = main_phase(2);
    let szeras = g.add_card_to_battlefield(0, catalog::illuminor_szeras());
    g.clear_sickness(szeras);
    g.add_card_to_battlefield(0, catalog::hexmark_destroyer());
    activate_x(&mut g, szeras, 0, None, None).expect("sac");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 6);
}

/// Imotekh — artifact cards leaving your graveyard make two Warriors once per
/// batch (CR 603.2c), however many left.
#[test]
fn imotekh_phaeron_fires_once_per_batch() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::imotekh_the_stormlord());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::mind_stone());
    run(&mut g, &Effect::ExilePlayerGraveyard { who: PlayerRef::You, filter: None });
    assert_eq!(count(&g, 0, "Necron Warrior"), 2);
}

/// Lokhust Heavy Destroyer makes each player sacrifice a creature — itself
/// included when it's its controller's only one.
#[test]
fn lokhust_heavy_destroyer_edicts_the_table() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let lokhust = g.add_card_to_hand(0, catalog::lokhust_heavy_destroyer());
    cast(&mut g, lokhust, None).expect("cast");
    assert_eq!(count(&g, 1, "Grizzly Bears") + count(&g, 2, "Grizzly Bears"), 0);
    assert!(in_graveyard(&g, 0, lokhust));
}

/// Lychguard returns every legendary creature card from your graveyard.
#[test]
fn lychguard_guardian_protocols() {
    let mut g = main_phase(2);
    let lych = g.add_card_to_battlefield(0, catalog::lychguard());
    let a = g.add_card_to_graveyard(0, catalog::szarekh_the_silent_king());
    let b = g.add_card_to_graveyard(0, catalog::trazyn_the_infinite());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g);
    activate_x(&mut g, lych, 0, None, None).expect("{3}{B}, sac");
    for id in [a, b] {
        assert!(g.players[0].hand.iter().any(|c| c.id == id));
    }
    assert!(in_graveyard(&g, 0, bear));
}

/// Necron Deathmark destroys a creature and mills a player three.
#[test]
fn necron_deathmark_disintegrates() {
    let mut g = main_phase(2);
    library(&mut g, 1, 5);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dm = g.add_card_to_hand(0, catalog::necron_deathmark());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: dm,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none());
    assert_eq!(g.players[1].graveyard.len(), 4, "the bear and three milled");
}

/// Necron Monolith mills three on attack, a Warrior per creature card milled.
#[test]
fn necron_monolith_eternity_gate() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::flayed_one());
    let mono = g.add_card_to_battlefield(0, catalog::necron_monolith());
    let crew: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    g.perform_action(GameAction::Crew { vehicle: mono, crew_creatures: crew }).expect("crew 4");
    attack(&mut g, &[mono], 1);
    assert_eq!(count(&g, 0, "Necron Warrior"), 2);
}

/// Necron Overlord taps X artifacts to drain X from a target opponent.
#[test]
fn necron_overlord_relentless_march() {
    let mut g = main_phase(2);
    let ov = g.add_card_to_battlefield(0, catalog::necron_overlord());
    g.clear_sickness(ov);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::sol_ring());
    }
    g.players[0].mana_pool.add_colorless(2);
    activate_x(&mut g, ov, 0, Some(Target::Player(1)), Some(2)).expect("X = 2");
    assert_eq!(g.players[1].life, 18);
}

/// CR 614 — Out of the Tombs replaces an empty-library draw with a
/// reanimation; with no creature card to return, the draw decks you.
#[test]
fn out_of_the_tombs_replaces_the_empty_draw() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::out_of_the_tombs());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    assert!(g.draw_one_or_deck(0, &mut events), "the replacement counts as the draw");
    assert!(g.battlefield_find(bear).is_some());
    assert!(!g.draw_one_or_deck(0, &mut events));
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "no creature card left: you lose");
}

/// Out of the Tombs' upkeep adds two eon counters and mills that many.
#[test]
fn out_of_the_tombs_mills_by_eon_counters() {
    let mut g = main_phase(2);
    library(&mut g, 0, 10);
    let tombs = g.add_card_to_battlefield(0, catalog::out_of_the_tombs());
    for n in [2, 6] {
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), n);
    }
    assert_eq!(g.battlefield_find(tombs).unwrap().counter_count(CounterType::Eon), 4);
}

/// Psychomancer drains when a nontoken artifact of yours is exiled from the
/// battlefield, and when it dies itself — not for a token.
#[test]
fn psychomancer_harbinger_of_despair() {
    let mut g = main_phase(2);
    let psy = g.add_card_to_battlefield(0, catalog::psychomancer());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let foe_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    run(&mut g, &crabomination::effect::Effect::Exile { what: crabomination::effect::Selector::ExactObjects(vec![ring, foe_ring]) });
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19), "only my Sol Ring counts");
    destroy(&mut g, psy);
    assert_eq!((g.players[0].life, g.players[1].life), (22, 18), "its own death");
}

/// Resurrection Orb brings the equipped creature back at the next end step.
#[test]
fn resurrection_orb_returns_the_fallen() {
    let mut g = main_phase(2);
    let orb = g.add_card_to_battlefield(0, catalog::resurrection_orb());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::Equip { equipment: orb, target: bear }).expect("equip {4}");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Lifelink));
    destroy(&mut g, bear);
    assert!(in_graveyard(&g, 0, bear));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "back at the end step");
}

/// Sautekh Immortal enters with a counter per creature that died this turn.
#[test]
fn sautekh_immortal_counts_the_dead() {
    let mut g = main_phase(2);
    for seat in [0, 1] {
        let b = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        let mut events = Vec::new();
        g.destroy_permanent(b, false, &mut events);
    }
    drain_stack(&mut g);
    let st = g.add_card_to_hand(0, catalog::sautekh_immortal());
    cast(&mut g, st, None).expect("cast");
    assert_eq!(g.battlefield_find(st).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Shard of the Nightbringer drains half a target opponent's life, rounded
/// up — only when cast.
#[test]
fn shard_of_the_nightbringer_drain_life() {
    let mut g = main_phase(2);
    g.players[1].life = 39;
    let shard = g.add_card_to_hand(0, catalog::shard_of_the_nightbringer());
    cast(&mut g, shard, None).expect("cast");
    assert_eq!((g.players[1].life, g.players[0].life), (19, 40));
    let copy = g.add_card_to_graveyard(0, catalog::shard_of_the_nightbringer());
    let back = Effect::Move { what: Selector::ExactObjects(vec![copy]), to: crabomination::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } };
    run(&mut g, &back);
    assert_eq!(g.players[1].life, 19, "reanimated, not cast: no drain");
}

/// Shard of the Void Dragon grows by two for each artifact put into a
/// graveyard from the battlefield, anyone's.
#[test]
fn shard_of_the_void_dragon_matter_absorption() {
    let mut g = main_phase(2);
    let shard = g.add_card_to_battlefield(0, catalog::shard_of_the_void_dragon());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    destroy(&mut g, ring);
    assert_eq!(g.battlefield_find(shard).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Skorpekh Lord: +1/+0 and menace to your other artifact creatures.
#[test]
fn skorpekh_lord_command_protocols() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::skorpekh_lord());
    let flayed = g.add_card_to_battlefield(0, catalog::flayed_one());
    let c = g.computed_permanent(flayed).unwrap();
    assert_eq!((c.power, c.toughness), (5, 1));
    assert!(c.keywords().contains(&Keyword::Menace));
}

/// Technomancer returns artifact creatures with total mana value 6 or less.
#[test]
fn technomancer_rebuilds_within_the_budget() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    let a = g.add_card_to_graveyard(0, catalog::flayed_one());
    let b = g.add_card_to_graveyard(0, catalog::royal_warden());
    let big = g.add_card_to_graveyard(0, catalog::hexmark_destroyer());
    let tm = g.add_card_to_hand(0, catalog::technomancer());
    cast(&mut g, tm, None).expect("cast");
    assert!(g.battlefield_find(a).is_some(), "the cheapest first");
    assert!(in_graveyard(&g, 0, b) && in_graveyard(&g, 0, big), "3 + 5 and 3 + 6 break the budget of 6");
}

/// The War in Heaven III returns creatures as necrodermis artifacts.
#[test]
fn the_war_in_heaven_returns_necrodermis() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let eff = catalog::the_war_in_heaven().saga_chapters[2].1.clone();
    run(&mut g, &eff);
    let c = g.battlefield_find(bear).expect("returned");
    assert_eq!(c.counter_count(CounterType::Necrodermis), 1);
    assert!(g.computed_permanent(bear).unwrap().card_types().contains(&CardType::Artifact));
}

/// Their Name Is Death spares artifact creatures.
#[test]
fn their_name_is_death_spares_the_machines() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flayed = g.add_card_to_battlefield(0, catalog::flayed_one());
    let s = g.add_card_to_hand(0, catalog::their_name_is_death());
    cast(&mut g, s, None).expect("cast");
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(flayed).is_some());
}

/// Their Number Is Legion: X tapped Warriors, then life per artifact; cast
/// again from the graveyard, and exiled after.
#[test]
fn their_number_is_legion_from_the_graveyard() {
    let mut g = main_phase(2);
    let s = g.add_card_to_graveyard(0, catalog::their_number_is_legion());
    cast_x(&mut g, s, None, Some(2)).expect("cast from the graveyard");
    assert_eq!(count(&g, 0, "Necron Warrior"), 2);
    assert_eq!(g.players[0].life, 22);
    assert!(g.exile.iter().any(|c| c.id == s));
}

/// Tomb Blade: the damaged player loses life per creature they control
/// unless they sacrifice one (the bot sacrifices when the loss is bigger).
#[test]
fn tomb_blade_punishes() {
    let mut g = main_phase(2);
    let blade = g.add_card_to_battlefield(0, catalog::tomb_blade());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack(&mut g, &[blade], 1);
    run_combat_out(&mut g);
    let bears = count(&g, 1, "Grizzly Bears");
    assert!(g.players[1].life == 14 || (bears == 0 && g.players[1].life == 15), "life {}", g.players[1].life);
}

/// Tomb Fortress exiles itself to mill four and reanimate.
#[test]
fn tomb_fortress_raises_the_dead() {
    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let fort = g.add_card_to_battlefield(0, catalog::tomb_fortress());
    g.clear_sickness(fort);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g);
    activate_x(&mut g, fort, 1, None, None).expect("activate");
    assert!(g.exile.iter().any(|c| c.id == fort));
    assert!(g.battlefield_find(bear).is_some());
}

/// Trazyn has the activated abilities of the artifact cards in your
/// graveyard (and not of other cards).
#[test]
fn trazyn_prismatic_gallery() {
    let mut g = main_phase(2);
    let trazyn = g.add_card_to_battlefield(0, catalog::trazyn_the_infinite());
    g.clear_sickness(trazyn);
    g.add_card_to_graveyard(0, catalog::sol_ring());
    let before = g.players[0].mana_pool.colorless_amount();
    activate_x(&mut g, trazyn, 0, None, None).expect("Sol Ring's {T}: {C}{C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), before + 2);
}

/// Triarch Stalker — attackers of the chosen opponent have menace; attackers
/// of anyone else don't.
#[test]
fn triarch_stalker_targeting_relay() {
    let mut g = main_phase(3);
    let stalker = g.add_card_to_battlefield(0, catalog::triarch_stalker());
    g.battlefield.iter_mut().find(|c| c.id == stalker).unwrap().chosen_player = Some(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(2) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Menace));
    assert!(!g.computed_permanent(b).unwrap().keywords().contains(&Keyword::Menace));
}

/// Triarch Praetorian draws two only when it enters from a graveyard.
#[test]
fn triarch_praetorian_dynastic_codes() {
    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    let tp = g.add_card_to_graveyard(0, catalog::triarch_praetorian());
    let hand = g.players[0].hand.len();
    flood(&mut g);
    activate_x(&mut g, tp, 0, None, None).expect("unearth");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[0].life, 18);
}

/// Canoptek Spyder draws for another nontoken artifact creature entering,
/// not for a token.
#[test]
fn canoptek_spyder_fabricator_claw_array() {
    let mut g = main_phase(2);
    library(&mut g, 0, 4);
    g.add_card_to_battlefield(0, catalog::canoptek_spyder());
    let hand = g.players[0].hand.len();
    let rw = g.add_card_to_hand(0, catalog::royal_warden());
    cast(&mut g, rw, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the Warden, none for its tokens");
}
