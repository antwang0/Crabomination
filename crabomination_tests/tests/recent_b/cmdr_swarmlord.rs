//! Commander: the Tyranid Swarm precon (40K, The Swarmlord,
//! `decks::cmdr_swarmlord`), and its primitives: a creature-cast counter
//! rider on mana (CR 106.6a), a pump by each creature's own counters, and a
//! remove-counters discount cost (CR 601.2f).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.turn_number = 3;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::island());
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
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

fn kill(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

/// CR 106.6a — Biophagus's mana gives the creature spell it funds a +1/+1
/// counter; a noncreature spell funded by it gets nothing.
#[test]
fn cr_106_6a_biophagus_mana_adds_a_counter() {
    let mut g = pod(2);
    let bio = g.add_card_to_battlefield(0, catalog::biophagus());
    g.clear_sickness(bio);
    activate(&mut g, 0, bio, 0, None, None).expect("any color");
    g.players[0].mana_pool.add(Color::Green, 1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_x(&mut g, 0, bear, None, None).expect("Bears on Biophagus's mana");
    assert_eq!(counters(&g, bear), 1);
    let plain = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast_x(&mut g, 0, plain, None, None).expect("Bears on plain mana");
    assert_eq!(counters(&g, plain), 0);
}

/// Clamavus — each creature gets +1/+1 per +1/+1 counter on itself.
#[test]
fn clamavus_pumps_by_own_counters() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::clamavus());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    assert_eq!(pt(&g, a), (6, 6), "2/2 + 2 counters + 2 from Clamavus");
    assert_eq!(pt(&g, b), (2, 2));
}

/// CR 601.2f — Hierophant Bio-Titan: three +1/+1 counters removed make it
/// {6} cheaper, and they come off as it's cast.
#[test]
fn cr_601_2f_hierophant_removes_counters_for_a_discount() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hbt = g.add_card_to_hand(0, catalog::hierophant_bio_titan());
    cast_x(&mut g, 0, hbt, None, None).expect("{10}{G}{G} - 6");
    assert!(g.battlefield_find(hbt).is_some());
    assert_eq!(counters(&g, bear), 0, "the three counters paid for it");
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// CR 601.2b / 601.2f — "remove any number of counters" is the caster's
/// choice: a manual-mana caster removes two (not the three the headless pick
/// takes), names the Elf's lone counter first, and the second comes off the
/// Bear (the only holder left); the spell costs {4} less.
#[test]
fn cr_601_2b_hierophant_caster_chooses_the_counters() {
    use crabomination::decision::DecisionAnswer;
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.players[0].manual_mana = true;
    let hbt = g.add_card_to_hand(0, catalog::hierophant_bio_titan());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: hbt, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("asks how many");
    assert!(g.pending_decision.is_some(), "how many counters");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Amount(2))).expect("two");
    assert!(g.pending_decision.is_some(), "which creature first");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Cards(vec![elf]))).expect("the elf");
    assert!(g.pending_decision.is_none(), "the bear is the only holder left");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hbt).is_some());
    assert_eq!((counters(&g, bear), counters(&g, elf)), (1, 0));
    assert_eq!(g.players[0].mana_pool.total(), 0, "{{10}}{{G}}{{G}} - 4");
}

/// CR 702.156 — Ravenous: X = 5 enters with five counters and draws.
#[test]
fn cr_702_156_ravenous_draws_at_five() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    let hh = g.add_card_to_hand(0, catalog::hormagaunt_horde());
    cast_x(&mut g, 0, hh, None, Some(5)).expect("X = 5");
    assert_eq!(counters(&g, hh), 5);
    assert_eq!(g.players[0].hand.len(), hand + 1, "cast one, drew one");
}

/// Exocrine — X damage to each player and each other creature.
#[test]
fn exocrine_blasts_everything_else() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ex = g.add_card_to_hand(0, catalog::exocrine());
    cast_x(&mut g, 0, ex, None, Some(2)).expect("X = 2");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(ex).is_some(), "not itself");
    assert_eq!(g.players[0].life, g.players[1].life);
    assert_eq!(g.players[1].life, 18, "20 - X");
}

/// Genestealer Patriarch — an infected creature dying becomes your Tyranid
/// copy.
#[test]
fn genestealer_patriarch_copies_the_infected() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::genestealer_patriarch());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::Infection, 1);
    kill(&mut g, bear);
    let copy = named(&g, 0, "Grizzly Bears");
    assert_eq!(copy.len(), 1);
    let c = g.battlefield_find(copy[0]).unwrap();
    assert!(c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Tyranid));
}

/// Toxicrene — a land loses its own ability and taps for any color.
#[test]
fn toxicrene_rewrites_lands() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::toxicrene());
    let tower = g.add_card_to_battlefield(1, catalog::command_tower());
    g.active_player_idx = 1;
    assert!(g.computed_permanent(tower).unwrap().lost_all_abilities, "its own ability is gone");
    activate(&mut g, 1, tower, 1, None, None).expect("the granted any-color ability, after the printed one");
    assert_eq!(g.players[1].mana_pool.total(), 1);
}

/// The Swarmlord — a creature of yours with a counter dying draws a card.
#[test]
fn swarmlord_draws_off_countered_deaths() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_swarmlord());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let hand = g.players[0].hand.len();
    kill(&mut g, b);
    assert_eq!(g.players[0].hand.len(), hand, "no counter, no card");
    kill(&mut g, a);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Ghyrson Starn — a 1-damage hit from another source becomes 3.
#[test]
fn ghyrson_starn_adds_two_to_a_single_point() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ghyrson_starn_kelermorph());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let ctx = EffectContext::for_ability(elf, 0, None);
    for amount in [1, 2] {
        let evs = g
            .resolve_effect(
                &Effect::DealDamage {
                    to: Selector::Player(crabomination::effect::PlayerRef::Seat(1)),
                    amount: crabomination::card::Value::Const(amount),
                },
                &ctx,
            )
            .expect("damage");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 20 - 1 - 2 - 2, "the 1 became 3; the 2 stayed 2");
}

/// Tyranid Invasion — a Warrior per opponent.
#[test]
fn tyranid_invasion_counts_opponents() {
    let mut g = pod(4);
    flood(&mut g, 0);
    let ti = g.add_card_to_hand(0, catalog::tyranid_invasion());
    cast_x(&mut g, 0, ti, None, None).expect("Invasion");
    assert_eq!(named(&g, 0, "Tyranid Warrior").len(), 3);
}

/// Mawloc — fights an opposing creature, which is exiled rather than dying.
#[test]
fn mawloc_fights_and_exiles() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::mawloc());
    cast_x(&mut g, 0, m, Some(Target::Permanent(bear)), Some(2)).expect("X = 2");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled, not dead");
    assert!(g.battlefield_find(m).is_some());
}

/// Malanthrope — exiles a graveyard and counts its creature cards.
#[test]
fn malanthrope_eats_a_graveyard() {
    let mut g = pod(2);
    flood(&mut g, 0);
    for _ in 0..2 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(1, catalog::island());
    let mal = g.add_card_to_hand(0, catalog::malanthrope());
    cast_x(&mut g, 0, mal, None, None).expect("Malanthrope");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(counters(&g, mal), 2);
}

/// Haruspex — grows off another death; its counters become mana.
#[test]
fn haruspex_feeds_and_spends() {
    let mut g = pod(2);
    let h = g.add_card_to_battlefield(0, catalog::haruspex());
    g.clear_sickness(h);
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        kill(&mut g, b);
    }
    assert_eq!(counters(&g, h), 2);
    activate(&mut g, 0, h, 0, None, Some(2)).expect("remove two");
    assert_eq!(g.players[0].mana_pool.total(), 2);
    assert_eq!(counters(&g, h), 0);
}

/// Winged Hive Tyrant — your countered creatures fly and have haste.
#[test]
fn winged_hive_tyrant_grants_to_countered() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::winged_hive_tyrant());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Flying));
    assert!(!g.computed_permanent(b).unwrap().keywords().contains(&Keyword::Flying));
}

/// Magus Lucea Kane's Psychic Stimulus copies the next ABILITY with {X} too —
/// once, and it spends the spell half of the same "when you next".
#[test]
fn magus_lucea_kane_copies_the_next_x_ability() {
    let mut g = pod(2);
    let magus = g.add_card_to_battlefield(0, catalog::magus_lucea_kane());
    g.clear_sickness(magus);
    let helix = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    flood(&mut g, 0);
    activate(&mut g, 0, magus, 0, None, None).expect("psychic stimulus");
    activate(&mut g, 0, helix, 0, None, Some(3)).expect("helix X=3");
    assert_eq!(g.battlefield_find(helix).unwrap().counter_count(CounterType::Charge), 6, "copied once");
    activate(&mut g, 0, helix, 0, None, Some(2)).expect("helix X=2");
    assert_eq!(g.battlefield_find(helix).unwrap().counter_count(CounterType::Charge), 8, "not again");
    let hydra = g.add_card_to_hand(0, catalog::primordial_hydra());
    cast_x(&mut g, 0, hydra, None, Some(2)).expect("hydra");
    assert_eq!(named(&g, 0, "Primordial Hydra").len(), 1, "the spell half is spent too");
}

/// A spell is a source too (CR 120.1): Spark Jolt's 1 damage has Ghyrson add
/// 2, and The Red Terror grows off the red spell's damage — and off red
/// Ghyrson's — once each.
#[test]
fn spell_damage_reaches_ghyrson_and_the_red_terror() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ghyrson_starn_kelermorph());
    let terror = g.add_card_to_battlefield(0, catalog::the_red_terror());
    let jolt = g.add_card_to_hand(0, catalog::spark_jolt());
    g.add_card_to_library(0, catalog::island());
    flood(&mut g, 0);
    cast_x(&mut g, 0, jolt, Some(Target::Player(1)), None).expect("jolt");
    assert_eq!(g.players[1].life, 20 - 1 - 2, "the jolt's 1 became 3");
    assert_eq!(counters(&g, terror), 2, "the red spell, then red Ghyrson");
}

/// CR 614.1c — The First Tyrannic War I: an {X} creature put onto the
/// battlefield from hand ENTERS with a +1/+1 counter per land you control, so
/// Walking Ballista (0/0) lives with three.
#[test]
fn first_tyrannic_war_i_enters_with_its_counters() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let ballista = g.add_card_to_hand(0, catalog::walking_ballista());
    let saga = catalog::the_first_tyrannic_war();
    let chapter = saga.saga_chapters[0].1.clone();
    let src = g.add_card_to_battlefield(0, saga);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![ballista])]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(src, 0, None);
    let evs = g.resolve_effect(&chapter, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let b = g.battlefield_find(ballista).expect("Ballista entered and survived");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 3);
}
