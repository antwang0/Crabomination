//! Commander: the Peer Through Time precon (C14, Teferi, `decks::cmdr_teferi`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets).expect("cast");
}

fn activate(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    target: Option<Target>,
    additional_targets: Vec<Target>,
) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets,
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn face_down_then_up(g: &mut GameState, id: CardId) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("face down");
    drain_stack(g);
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("face up");
    drain_stack(g);
}

/// CR 601.2c — Aether Gale needs six different targets (the 2014 ruling); five is not a
/// legal cast, six go back to their owners' hands.
#[test]
fn aether_gale_takes_exactly_six_targets() {
    let mut g = main_phase(2);
    let ids: Vec<CardId> = (0..6).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    let gale = g.add_card_to_hand(0, catalog::aether_gale());
    let five: Vec<Target> = ids[..5].iter().map(|&id| Target::Permanent(id)).collect();
    assert!(cast_by(&mut g, 0, gale, &five).is_err(), "five targets is not six");
    let six: Vec<Target> = ids.iter().map(|&id| Target::Permanent(id)).collect();
    cast(&mut g, gale, &six);
    assert!(ids.iter().all(|&id| g.battlefield_find(id).is_none()));
    assert_eq!(g.players[1].hand.len(), 6);
}

/// Breaching Leviathan's intervening "if you cast it from your hand": cast,
/// it taps every nonblue creature (yours too) and they skip their next untap;
/// put onto the battlefield another way, nothing happens.
#[test]
fn breaching_leviathan_locks_nonblue_creatures_when_hand_cast() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blue = g.add_card_to_battlefield(1, catalog::azure_mage());
    let lev = g.add_card_to_hand(0, catalog::breaching_leviathan());
    cast(&mut g, lev, &[]);
    for id in [bear, mine] {
        let c = g.battlefield_find(id).unwrap();
        assert!(c.tapped && c.skip_next_untap, "nonblue creatures are locked");
    }
    assert!(!g.battlefield_find(blue).unwrap().tapped, "blue creatures are spared");

    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::breaching_leviathan());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "not cast, no trigger");
}

/// CR 708.8 — Brine Elemental turned face up: each opponent (both, at three
/// seats) skips their next untap step; you don't.
#[test]
fn brine_elemental_face_up_skips_every_opponents_untap() {
    let mut g = main_phase(3);
    let brine = g.add_card_to_hand(0, catalog::brine_elemental());
    face_down_then_up(&mut g, brine);
    assert_eq!(g.players[0].skip_next_untap_step, 0);
    assert_eq!(g.players[1].skip_next_untap_step, 1);
    assert_eq!(g.players[2].skip_next_untap_step, 1);
}

/// Crown of Doom: a creature attacking its controller gets +2/+0. Handed to
/// an opponent, it can go to anyone *but* its owner (the 2014 ruling).
#[test]
fn crown_of_doom_pumps_attackers_and_never_goes_home() {
    let mut g = main_phase(3);
    let crown = g.add_card_to_battlefield(0, catalog::crown_of_doom());
    activate(&mut g, 0, crown, Some(Target::Player(1)), vec![]).expect("give it to seat 1");
    assert_eq!(g.battlefield_find(crown).unwrap().controller, 1);

    // Seat 1 now holds it; on seat 1's turn the owner is not a legal target.
    g.active_player_idx = 1;
    assert!(activate(&mut g, 1, crown, Some(Target::Player(0)), vec![]).is_err(), "not back to its owner");
    activate(&mut g, 1, crown, Some(Target::Player(2)), vec![]).expect("on to seat 2");
    assert_eq!(g.battlefield_find(crown).unwrap().controller, 2);

    // Seat 0 attacks the Crown's controller: +2/+0.
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 2));
}

/// Domineering Will: you take up to three nonattacking creatures until end
/// of turn, untapped, and each must block.
#[test]
fn domineering_will_borrows_blockers() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    let will = g.add_card_to_hand(0, catalog::domineering_will());
    cast(&mut g, will, &[Target::Permanent(a), Target::Permanent(b)]);
    for id in [a, b] {
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.controller, 0);
        assert!(!c.tapped);
        assert!(g.computed_permanent(id).unwrap().keywords().contains(&Keyword::MustBlock));
    }
}

/// CR 508.1d — Dulcet Sirens: the target creature must attack the named
/// opponent this turn; attacking anyone else is rejected.
#[test]
fn dulcet_sirens_names_who_gets_attacked() {
    let mut g = main_phase(3);
    let sirens = g.add_card_to_battlefield(0, catalog::dulcet_sirens());
    g.clear_sickness(sirens);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    activate(&mut g, 0, sirens, Some(Target::Permanent(bear)), vec![Target::Player(2)]).expect("activate");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![]))
            .is_err(),
        "it must attack"
    );
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
            .is_err(),
        "not the named opponent"
    );
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]))
        .expect("attacks seat 2");
}

/// CR 702.36 — Fathom Seer's morph cost is two Islands back to hand; face up,
/// it draws two.
#[test]
fn fathom_seer_unmorphs_for_two_islands() {
    let mut g = main_phase(2);
    let isles: Vec<CardId> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::island())).collect();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let seer = g.add_card_to_hand(0, catalog::fathom_seer());
    face_down_then_up(&mut g, seer);
    assert!(isles.iter().all(|&id| g.battlefield_find(id).is_none()), "both Islands returned");
    assert_eq!(g.players[0].hand.len(), 4, "two Islands and two draws");
    assert_eq!(pt(&g, seer), (1, 3));
}

/// Fool's Demise: the enchanted creature comes back under your control when
/// it dies, and the Aura goes back to your hand.
#[test]
fn fools_demise_takes_the_dead_and_comes_home() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let demise = g.add_card_to_hand(0, catalog::fools_demise());
    cast(&mut g, demise, &[Target::Permanent(giant)]);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(giant)]);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Hill Giant").expect("returned");
    assert_eq!(back.controller, 0);
    assert!(g.players[0].hand.iter().any(|c| c.id == demise), "the Aura came home");
}

/// CR 707.2 — Infinite Reflection: your nontoken creatures become copies of
/// the enchanted creature, and nontoken creatures entering later enter as
/// copies (tokens don't).
#[test]
fn infinite_reflection_turns_the_team_into_copies() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let refl = g.add_card_to_hand(0, catalog::infinite_reflection());
    cast(&mut g, refl, &[Target::Permanent(angel)]);
    assert_eq!(g.battlefield_find(bear).unwrap().definition.name, "Serra Angel");
    let late = g.add_card_to_hand(0, catalog::hill_giant());
    cast(&mut g, late, &[]);
    assert_eq!(g.battlefield_find(late).unwrap().definition.name, "Serra Angel");
    assert_eq!(pt(&g, late), (4, 4));
}

/// Intellectual Offering at three seats: you and one opponent each draw
/// three; the other opponent draws nothing (not targets — chosen).
#[test]
fn intellectual_offering_feeds_one_opponent() {
    let mut g = main_phase(3);
    for seat in 0..3 {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let land = g.add_card_to_battlefield(0, catalog::island());
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.battlefield_find_mut(rock).unwrap().tapped = true;
    let io = g.add_card_to_hand(0, catalog::intellectual_offering());
    cast(&mut g, io, &[]);
    assert_eq!(g.players[0].hand.len(), 3);
    let mut opp = [g.players[1].hand.len(), g.players[2].hand.len()];
    opp.sort();
    assert_eq!(opp, [0, 3]);
    assert!(!g.battlefield_find(rock).unwrap().tapped, "nonland permanents untap");
    assert!(g.battlefield_find(land).unwrap().tapped, "lands don't");
}

/// Shaper Parasite turned face up: +2/−2 (the default mode) finishes a
/// two-toughness creature.
#[test]
fn shaper_parasite_face_up_shrinks_a_creature() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sp = g.add_card_to_hand(0, catalog::shaper_parasite());
    face_down_then_up(&mut g, sp);
    assert!(g.battlefield_find(bear).is_none());
}

/// Stitcher Geralf: everyone mills three, the two strongest creature cards
/// milled are exiled, and the Zombie is as big as their total power.
#[test]
fn stitcher_geralf_stitches_the_two_strongest() {
    let mut g = main_phase(2);
    for def in [catalog::grizzly_bears(), catalog::island(), catalog::hill_giant()] {
        g.add_card_to_library(0, def);
    }
    for def in [catalog::craw_wurm(), catalog::island(), catalog::island()] {
        g.add_card_to_library(1, def);
    }
    let sg = g.add_card_to_battlefield(0, catalog::stitcher_geralf());
    g.clear_sickness(sg);
    activate(&mut g, 0, sg, None, vec![]).expect("activate");
    let exiled: Vec<&str> = g.exile.iter().map(|c| c.definition.name).collect();
    assert!(exiled.contains(&"Craw Wurm") && exiled.contains(&"Hill Giant"), "{exiled:?}");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
    let z = g.battlefield.iter().find(|c| c.definition.name == "Zombie").expect("zombie").id;
    assert_eq!(pt(&g, z), (9, 9), "6 + 3");
}

/// CR 207.2c lieutenant — Stormsurge Kraken is a 7/7 only while you control
/// your commander.
#[test]
fn stormsurge_kraken_is_a_lieutenant() {
    let mut g = main_phase(2);
    let cmdr = g.seat_commanders(0, vec![catalog::teferi_temporal_archmage()])[0];
    let kraken = g.add_card_to_battlefield(0, catalog::stormsurge_kraken());
    assert_eq!(pt(&g, kraken), (5, 5));
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmdr,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast the planeswalker commander");
    drain_stack(&mut g);
    assert_eq!(pt(&g, kraken), (7, 7));
}

/// CR 606.3 — loyalty abilities are sorcery-speed; Teferi's −10 emblem lets
/// you activate them on an opponent's turn, once per walker per turn.
#[test]
fn teferi_emblem_grants_instant_speed_loyalty() {
    let mut g = main_phase(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let tef = g.add_card_to_battlefield(0, catalog::teferi_temporal_archmage());
    g.battlefield_find_mut(tef).unwrap().add_counters(CounterType::Loyalty, 10);
    let other = g.add_card_to_battlefield(0, catalog::teferi_temporal_archmage());

    // Opponent's turn, no emblem: rejected.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    assert!(loyalty(&mut g, 0, other, 0, None).is_err(), "sorcery speed without the emblem");

    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    loyalty(&mut g, 0, tef, 2, None).expect("−10");
    assert_eq!(g.players[0].emblems.len(), 1);

    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    let hand = g.players[0].hand.len();
    loyalty(&mut g, 0, other, 0, None).expect("instant speed with the emblem");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(loyalty(&mut g, 0, other, 0, None).is_err(), "still once per turn");
}

/// Well of Ideas: ETB draws two; each other player draws one extra in their
/// draw step, you two extra in yours.
#[test]
fn well_of_ideas_draw_steps() {
    let mut g = main_phase(3);
    for seat in 0..3 {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let well = g.add_card_to_hand(0, catalog::well_of_ideas());
    cast(&mut g, well, &[]);
    assert_eq!(g.players[0].hand.len(), 2);
    for (seat, extra) in [(1usize, 1usize), (0, 2)] {
        let before = g.players[seat].hand.len();
        g.active_player_idx = seat;
        g.step = TurnStep::Upkeep;
        g.priority.player_with_priority = seat;
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        assert_eq!(g.step, TurnStep::Draw);
        assert_eq!(g.players[seat].hand.len(), before + 1 + extra, "seat {seat}");
    }
}
