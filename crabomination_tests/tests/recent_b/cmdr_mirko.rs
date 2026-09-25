//! Commander: the Revenant Recon precon (MKC, Mirko, `decks::cmdr_mirko`), and
//! its primitives: surveil and scry as separate events (CR 701.22, 701.42), a
//! surveiled card is put into the graveyard from the library but not milled
//! (CR 701.13), an additional beginning phase (CR 500.8), a copy that keeps
//! its own name (CR 707.9b), and "each player chooses … exile them until"
//! (CR 603.6e).

use crabomination::card::{CardId, CounterType, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector};
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

fn cast_no_drain(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_no_drain(g, seat, id, target)?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
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

fn cast_from_zone(g: &mut GameState, id: CardId) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
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

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn on_top(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.next_id();
    g.players[seat].add_to_library_top(id, def);
    id
}

/// Resolve "surveil `n`" for seat 0, graveyarding the listed cards.
fn surveil(g: &mut GameState, n: i32, bin: Vec<CardId>) {
    surveil_then(g, n, bin, vec![]);
}

/// `surveil`, with the answers that follow it (a "may" it triggers).
fn surveil_then(g: &mut GameState, n: i32, bin: Vec<CardId>, then: Vec<DecisionAnswer>) {
    let mut answers = vec![DecisionAnswer::ScryOrder { kept_top: vec![], bottom: bin }];
    answers.extend(then);
    g.decider = Box::new(ScriptedDecider::new(answers));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::Surveil { who: PlayerRef::You, amount: crabomination::card::Value::Const(n) },
            &ctx,
        )
        .expect("surveil");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// CR 701.42 / 701.22 — Mirko and Dimir Spybug grow whenever you surveil, and
/// a scry is not a surveil.
#[test]
fn cr_701_42_mirko_and_spybug_grow_on_surveil_not_scry() {
    let mut g = pod(2);
    let mirko = g.add_card_to_battlefield(0, catalog::mirko_obsessive_theorist());
    let bug = g.add_card_to_battlefield(0, catalog::dimir_spybug());
    flood(&mut g, 0);
    let rain = g.add_card_to_hand(0, catalog::notion_rain());
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, rain, None).expect("Notion Rain");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew two, cast one");
    assert_eq!(g.players[0].life, 18, "2 damage to you");
    assert_eq!((plus_ones(&g, mirko), plus_ones(&g, bug)), (1, 1));
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, 0, opt, None).expect("Opt");
    assert_eq!((plus_ones(&g, mirko), plus_ones(&g, bug)), (1, 1), "a scry is not a surveil");
    assert_eq!(pt(&g, mirko), (2, 4));
}

/// CR 701.42a / 701.13 — a surveiled creature card is put into your graveyard
/// from your library (Unshakable Tail investigates) but it was not milled
/// (Zellix's "mills one or more creature cards" stays quiet).
#[test]
fn cr_701_42a_surveil_is_from_library_but_not_a_mill() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::unshakable_tail());
    g.add_card_to_battlefield(0, catalog::zellix_sanity_flayer());
    let bears = on_top(&mut g, 0, catalog::grizzly_bears());
    surveil(&mut g, 1, vec![bears]);
    assert!(in_graveyard(&g, 0, bears));
    assert_eq!(named(&g, 0, "Clue").len(), 1, "Unshakable Tail investigated");
    assert!(named(&g, 0, "Horror").is_empty(), "a surveil is not a mill");
}

/// CR 701.42a — Narcomoeba ("put into your graveyard from your library")
/// enters when surveiled away.
#[test]
fn cr_701_42a_surveiled_narcomoeba_enters() {
    let mut g = pod(2);
    let moeba = on_top(&mut g, 0, catalog::narcomoeba());
    surveil_then(&mut g, 1, vec![moeba], vec![DecisionAnswer::Bool(true)]);
    assert!(g.battlefield_find(moeba).is_some(), "Narcomoeba put itself onto the battlefield");
}

/// Enhanced Surveillance — surveil 1 looks at three cards.
#[test]
fn enhanced_surveillance_looks_at_two_more() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::enhanced_surveillance());
    let ids: Vec<CardId> = (0..3).map(|_| on_top(&mut g, 0, catalog::island())).collect();
    surveil(&mut g, 1, ids.clone());
    assert!(ids.iter().all(|&id| in_graveyard(&g, 0, id)), "all three looked-at cards were binnable");
}

/// Enhanced Surveillance — exile it: shuffle your graveyard into your library.
#[test]
fn enhanced_surveillance_exiles_to_shuffle_graveyard() {
    let mut g = pod(2);
    let es = g.add_card_to_battlefield(0, catalog::enhanced_surveillance());
    let isl = g.add_card_to_graveyard(0, catalog::island());
    activate(&mut g, es, 0, None, None).expect("exile it");
    assert!(g.players[0].graveyard.is_empty());
    assert!(g.players[0].library.iter().any(|c| c.id == isl));
    assert!(g.exile.iter().any(|c| c.id == es));
}

/// Eye of Duskmantle — a spell surveiled away this turn is cast from the
/// graveyard for life equal to its mana value.
#[test]
fn eye_of_duskmantle_casts_surveiled_spell_for_life() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::eye_of_duskmantle());
    let div = on_top(&mut g, 0, catalog::divination());
    surveil(&mut g, 1, vec![div]);
    let hand = g.players[0].hand.len();
    cast_from_zone(&mut g, div).expect("cast Divination from the graveyard");
    assert_eq!(g.players[0].life, 17, "paid 3 life");
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Without the surveil, the graveyard card isn't castable.
#[test]
fn eye_of_duskmantle_needs_the_surveil() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::eye_of_duskmantle());
    let div = g.add_card_to_graveyard(0, catalog::divination());
    assert!(cast_from_zone(&mut g, div).is_err());
}

/// CR 500.8 — Sphinx of the Second Sun: after the postcombat main, an extra
/// untap, upkeep and draw, then the end phase.
#[test]
fn cr_500_8_sphinx_adds_a_beginning_phase() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sphinx_of_the_second_sun());
    let land = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    // Past turn 1, whose draw the first player skips (CR 103.8a).
    g.turn_number = 3;
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.additional_beginning_phases, 1);
    let hand = g.players[0].hand.len();
    let turn = g.turn_number;
    let mut seen = Vec::new();
    for _ in 0..40 {
        if g.step == TurnStep::End || g.is_game_over() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
        if g.step != TurnStep::PostCombatMain && seen.last() != Some(&g.step) {
            seen.push(g.step);
        }
    }
    assert_eq!(seen, vec![TurnStep::Upkeep, TurnStep::Draw, TurnStep::End], "untap, upkeep, draw, then end");
    assert_eq!(g.turn_number, turn, "still the same turn");
    assert!(!g.battlefield_find(land).unwrap().tapped, "the extra untap step untapped the Island");
    assert_eq!(g.players[0].hand.len(), hand + 1, "the extra draw step drew");
    assert_eq!(g.additional_beginning_phases, 0);
}

/// CR 707.9b — Lazav becomes a copy of a graveyard creature card with mana
/// value X, keeping its name, legendary, and its ability.
#[test]
fn cr_707_9b_lazav_copies_keeping_its_name() {
    let mut g = pod(2);
    let lazav = g.add_card_to_battlefield(0, catalog::lazav_the_multifarious());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    flood(&mut g, 0);
    activate(&mut g, lazav, 0, Some(Target::Permanent(giant)), Some(4)).expect("X = 4");
    let c = g.battlefield_find(lazav).unwrap();
    assert_eq!(c.definition.name, "Lazav, the Multifarious");
    assert!(c.definition.supertypes.contains(&Supertype::Legendary));
    assert_eq!(pt(&g, lazav), (3, 3), "a Hill Giant");
    assert!(!c.definition.activated_abilities.is_empty(), "it has this ability");
    // Copying again reads its own name, not the Giant's.
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, lazav, 0, Some(Target::Permanent(bears)), Some(2)).expect("X = 2");
    assert_eq!(g.battlefield_find(lazav).unwrap().definition.name, "Lazav, the Multifarious");
    assert_eq!(pt(&g, lazav), (2, 2));
}

/// Lazav's X must match the card's mana value.
#[test]
fn lazav_x_must_match_mana_value() {
    let mut g = pod(2);
    let lazav = g.add_card_to_battlefield(0, catalog::lazav_the_multifarious());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    flood(&mut g, 0);
    assert!(activate(&mut g, lazav, 0, Some(Target::Permanent(giant)), Some(2)).is_err());
}

/// CR 603.6e — Foreboding Steamboat: each player exiles two of their nontoken
/// creatures until it leaves; its attack bins one of them for a Clue.
#[test]
fn cr_603_6e_foreboding_steamboat_exiles_two_each() {
    let mut g = pod(2);
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let t1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let t2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    let boat = g.add_card_to_hand(0, catalog::foreboding_steamboat());
    cast(&mut g, 0, boat, None).expect("Steamboat");
    for id in [b1, b2, t1, t2] {
        assert!(g.exile.iter().any(|c| c.id == id), "two each exiled");
    }
    assert!(g.battlefield_find(giant).is_some(), "seat 0 kept its Giant");
    // Crew 2 with the Giant and attack: an opponent's card goes to its graveyard.
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: boat, crew_creatures: vec![giant] }).expect("crew");
    drain_stack(&mut g);
    g.clear_sickness(boat);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: boat, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Clue").len(), 1, "investigated");
    let binned = [t1, t2].iter().filter(|&&id| in_graveyard(&g, 1, id)).count();
    assert_eq!(binned, 1, "one of the opponent's exiled creatures went to the graveyard");
    // The Steamboat leaves: the rest come back.
    g.step = TurnStep::PostCombatMain;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::Move { what: Selector::ExactObjects(vec![boat]), to: crabomination::effect::ZoneDest::Hand(PlayerRef::You) },
            &ctx,
        )
        .expect("bounce");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(b1).is_some() && g.battlefield_find(b2).is_some());
    assert_eq!([t1, t2].iter().filter(|&&id| g.battlefield_find(id).is_some()).count(), 1);
}

/// Counterpoint — counter a mana value 4 spell, then cast a mana value 3
/// sorcery from your graveyard for free.
#[test]
fn counterpoint_counters_and_recasts() {
    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let giant = g.add_card_to_hand(1, catalog::hill_giant());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    cast_no_drain(&mut g, 1, giant, None).expect("Giant on the stack");
    flood(&mut g, 0);
    let cp = g.add_card_to_hand(0, catalog::counterpoint());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, cp, Some(Target::Permanent(giant))).expect("Counterpoint");
    assert!(in_graveyard(&g, 1, giant), "countered");
    assert_eq!(g.players[0].hand.len(), hand + 1, "cast Counterpoint, then Divination drew two");
    assert!(in_graveyard(&g, 0, div));
}

/// Counterpoint's free cast is capped by the countered spell's mana value.
#[test]
fn counterpoint_cap_is_the_countered_mana_value() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::divination());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    cast_no_drain(&mut g, 1, bears, None).expect("Bears on the stack");
    flood(&mut g, 0);
    let cp = g.add_card_to_hand(0, catalog::counterpoint());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, cp, Some(Target::Permanent(bears))).expect("Counterpoint");
    assert_eq!(g.players[0].hand.len(), hand - 1, "Divination (3) is over the Bears' 2");
}

/// Mission Briefing — the chosen instant or sorcery is castable this turn,
/// and exiled as it resolves.
#[test]
fn mission_briefing_recasts_then_exiles() {
    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    flood(&mut g, 0);
    let mb = g.add_card_to_hand(0, catalog::mission_briefing());
    cast(&mut g, 0, mb, None).expect("Mission Briefing");
    let hand = g.players[0].hand.len();
    cast_from_zone(&mut g, div).expect("cast Divination");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert!(g.exile.iter().any(|c| c.id == div), "exiled instead of the graveyard");
}

/// CR 719 — Case of the Shifting Visage is solved with fifteen cards in the
/// graveyard; solved, a nonlegendary creature spell is copied.
#[test]
fn cr_719_shifting_visage_copies_creature_spells() {
    let mut g = pod(2);
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_shifting_visage());
    for _ in 0..14 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let mut evs = Vec::new();
    g.process_case_solves(&mut evs);
    assert!(!g.battlefield_find(case).unwrap().case_solved, "fourteen isn't enough");
    g.add_card_to_graveyard(0, catalog::island());
    g.process_case_solves(&mut evs);
    assert!(g.battlefield_find(case).unwrap().case_solved);
    flood(&mut g, 0);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bears, None).expect("Bears");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "the spell and its token copy");
    let mirko = g.add_card_to_hand(0, catalog::mirko_obsessive_theorist());
    cast(&mut g, 0, mirko, None).expect("Mirko");
    assert_eq!(named(&g, 0, "Mirko, Obsessive Theorist").len(), 1, "legendary spells aren't copied");
}

/// CR 702.62 — Watcher of Hours surveils each time a time counter comes off
/// while it's suspended.
#[test]
fn cr_702_62_watcher_of_hours_surveils_while_suspended() {
    let mut g = pod(2);
    let bug = g.add_card_to_battlefield(0, catalog::dimir_spybug());
    let watcher = g.add_card_to_hand(0, catalog::watcher_of_hours());
    flood(&mut g, 0);
    g.perform_action(GameAction::Suspend { card_id: watcher }).expect("suspend");
    drain_stack(&mut g);
    let evs = g.process_suspend();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().find(|c| c.id == watcher).unwrap().counter_count(CounterType::Time), 5);
    assert_eq!(plus_ones(&g, bug), 1, "surveil 1 happened");
}

/// Whispering Snitch triggers on the first surveil each turn only.
#[test]
fn whispering_snitch_once_per_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::whispering_snitch());
    surveil(&mut g, 1, vec![]);
    surveil(&mut g, 1, vec![]);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 21);
}

/// Final-Word Phantom — sorceries at instant speed in an opponent's end step,
/// but not in their upkeep.
#[test]
fn final_word_phantom_flash_in_opponents_end_step() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::final_word_phantom());
    flood(&mut g, 0);
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    let div = g.add_card_to_hand(0, catalog::divination());
    assert!(cast(&mut g, 0, div, None).is_err(), "not in their upkeep");
    g.step = TurnStep::End;
    cast(&mut g, 0, div, None).expect("in their end step");
}

/// Charnel Serenade — surveil 3, reanimate with a finality counter, then it
/// suspends itself with three time counters.
#[test]
fn charnel_serenade_reanimates_and_suspends() {
    let mut g = pod(2);
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    flood(&mut g, 0);
    let cs = g.add_card_to_hand(0, catalog::charnel_serenade());
    cast(&mut g, 0, cs, None).expect("Charnel Serenade");
    let c = g.battlefield_find(giant).expect("returned");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    let ex = g.exile.iter().find(|c| c.id == cs).expect("exiled");
    assert_eq!(ex.counter_count(CounterType::Time), 3);
}

/// Disinformation Campaign — ETB draw and discard; a surveil bounces it.
#[test]
fn disinformation_campaign_returns_on_surveil() {
    let mut g = pod(2);
    g.add_card_to_hand(1, catalog::island());
    flood(&mut g, 0);
    let dc = g.add_card_to_hand(0, catalog::disinformation_campaign());
    cast(&mut g, 0, dc, None).expect("Campaign");
    assert!(g.players[1].hand.is_empty(), "opponent discarded");
    surveil(&mut g, 1, vec![]);
    assert!(in_hand(&g, 0, dc), "back to hand");
}

/// Thoughtbound Phantasm attacks once it has three +1/+1 counters.
#[test]
fn thoughtbound_phantasm_attacks_at_three_counters() {
    let mut g = pod(2);
    let tp = g.add_card_to_battlefield(0, catalog::thoughtbound_phantasm());
    g.clear_sickness(tp);
    surveil(&mut g, 1, vec![]);
    surveil(&mut g, 1, vec![]);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let attack = vec![Attack { attacker: tp, target: AttackTarget::Player(1) }];
    assert!(g.perform_action(GameAction::DeclareAttackers(attack.clone())).is_err(), "defender at two");
    g.step = TurnStep::PreCombatMain;
    surveil(&mut g, 1, vec![]);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(attack)).expect("three counters");
}

/// Dispersal — each opponent bounces their greatest mana value nonland
/// permanent, then discards.
#[test]
fn dispersal_bounces_greatest_and_discards() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    flood(&mut g, 0);
    let dd = g.add_card_to_hand(0, catalog::discovery_dispersal());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSplitRight {
        card_id: dd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Dispersal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none() && g.battlefield_find(bears).is_some());
    assert_eq!(g.players[1].hand.len(), 1, "returned the Giant, discarded one");
}

/// Connive — gain control of a creature with power 2 or less.
#[test]
fn connive_steals_small_creatures() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    flood(&mut g, 0);
    let cc = g.add_card_to_hand(0, catalog::connive_concoct());
    assert!(cast(&mut g, 0, cc, Some(Target::Permanent(giant))).is_err(), "power 3 is too big");
    cast(&mut g, 0, cc, Some(Target::Permanent(bears))).expect("Connive");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Unshakable Tail — {2}, sacrifice a Clue: back to hand from the graveyard.
#[test]
fn unshakable_tail_returns_for_a_clue() {
    let mut g = pod(2);
    let tail = g.add_card_to_graveyard(0, catalog::unshakable_tail());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&crabomination::effect::shortcut::investigate(1), &ctx).expect("clue");
    assert_eq!(named(&g, 0, "Clue").len(), 1);
    flood(&mut g, 0);
    activate(&mut g, tail, 0, None, None).expect("return");
    assert!(in_hand(&g, 0, tail));
    assert!(named(&g, 0, "Clue").is_empty());
}

/// Mirko's end step returns a creature card with lesser power, with a
/// finality counter.
#[test]
fn mirko_end_step_reanimates_lesser_power() {
    let mut g = pod(2);
    let mirko = g.add_card_to_battlefield(0, catalog::mirko_obsessive_theorist());
    g.battlefield_find_mut(mirko).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::End);
    let c = g.battlefield_find(bears).expect("Bears (2) under Mirko's 3");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

/// A creature card with power equal to Mirko's stays put.
#[test]
fn mirko_end_step_needs_strictly_less_power() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mirko_obsessive_theorist());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step(&mut g, TurnStep::End);
    assert!(in_graveyard(&g, 0, bears), "Bears (2) isn't under Mirko's 1");
}

/// Marvo — winning its attack clash draws and casts a spell for free.
#[test]
fn marvo_wins_clash_and_casts_free() {
    let mut g = pod(2);
    let marvo = g.add_card_to_battlefield(0, catalog::marvo_deep_operative());
    on_top(&mut g, 0, catalog::hill_giant());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.clear_sickness(marvo);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: marvo, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let cast_free = g.battlefield_find(bears).is_some() || !named(&g, 0, "Hill Giant").is_empty();
    assert!(cast_free, "won the clash (4 vs Sol Ring's 1), drew, cast a spell free");
}
