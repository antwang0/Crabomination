//! Commander: the Timey-Wimey precon (WHO, The Tenth Doctor + Rose Tyler,
//! `decks::cmdr_tenth`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..seats {
        for _ in 0..8 {
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn run_with(g: &mut GameState, effect: Effect, source: CardId, event_amount: u32) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    ctx.event_amount = event_amount;
    let ev = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    run_with(g, effect, source, 0);
}

fn time(g: &GameState, id: CardId) -> u32 {
    g.exile
        .iter()
        .find(|c| c.id == id)
        .or_else(|| g.battlefield_find(id))
        .map_or(0, |c| c.counter_count(CounterType::Time))
}

fn suspend(g: &mut GameState, id: CardId, n: u32) {
    run(g, Effect::GrantSuspend { what: Selector::ExactObjects(vec![id]), time_counters: n }, id);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// CR 702.62 — a card with printed suspend that also gains suspend (Kang
/// Prime, The Tenth Doctor) is one suspended card: one counter a turn.
#[test]
fn cr_702_62_a_printed_suspend_card_that_gains_suspend_ticks_once() {
    let mut g = main_phase(2);
    let whale = g.add_card_to_graveyard(0, catalog::star_whale());
    suspend(&mut g, whale, 3);
    assert_eq!(time(&g, whale), 3);
    g.process_suspend();
    assert_eq!(time(&g, whale), 2, "one counter per upkeep, not two");
}

/// CR 702.26 — Oubliette's phase-out is linked, and only a vanishing source
/// (Out of Time) counts what it phased out in time counters.
#[test]
fn cr_702_26_a_linked_phase_out_adds_no_time_counters_to_a_non_vanishing_source() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let oubliette = g.add_card_to_hand(0, catalog::oubliette());
    cast(&mut g, oubliette, Some(Target::Permanent(bear))).expect("Oubliette");
    assert!(g.battlefield_find(bear).is_none(), "the Bears phased out");
    assert_eq!(time(&g, oubliette), 0);
}

/// CR 702.62 — Amy Pond's damage takes that many time counters off a
/// suspended card through the suspend funnel: the last one casts it.
#[test]
fn cr_702_62_amy_pond_finishes_a_suspended_card() {
    let mut g = main_phase(2);
    let amy = g.add_card_to_battlefield(0, catalog::amy_pond());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    suspend(&mut g, bears, 2);
    run_with(&mut g, Effect::RemoveTimeCountersFromSuspended { amount: Value::TriggerEventAmount }, amy, 2);
    assert!(g.battlefield_find(bears).is_some(), "the last counter cast it");
}

/// CR 702.62e — The Tenth Doctor: attacking exiles from the top until a
/// nonland card, which gets three time counters and suspend.
#[test]
fn cr_702_62e_the_tenth_doctor_suspends_the_next_spell() {
    let mut g = main_phase(2);
    let doctor = g.add_card_to_battlefield(0, catalog::the_tenth_doctor());
    g.players[0].library.clear();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.clear_sickness(doctor);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: doctor, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("the Bolt is exiled");
    assert_eq!(exiled.counter_count(CounterType::Time), 3);
    assert!(exiled.has_suspend());
}

/// CR 702.62e — The Parting of the Ways I: each nonland card of the top five
/// gets its mana value in time counters and suspend.
#[test]
fn cr_702_62e_parting_of_the_ways_counts_mana_value() {
    let mut g = main_phase(2);
    g.players[0].library.clear();
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    let forest = g.add_card_to_library(0, catalog::forest());
    let saga = g.add_card_to_hand(0, catalog::the_parting_of_the_ways());
    cast(&mut g, saga, None).expect("The Parting of the Ways");
    assert_eq!(time(&g, giant), 4, "Hill Giant's mana value");
    assert!(g.exile.iter().any(|c| c.id == forest), "the land is exiled too");
    assert_eq!(time(&g, forest), 0, "but gets no counters");
}

/// CR 120.3 — Killer: 3 damage to the target and each other creature sharing
/// a creature type with it.
#[test]
fn cr_120_3_killer_hits_the_whole_tribe() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let other_giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::coward_killer());
    run(
        &mut g,
        Effect::DealDamageToTargetAndTypeSharers { what: Selector::ExactObjects(vec![giant]), amount: Value::Const(3) },
        spell,
    );
    assert!(g.battlefield_find(giant).is_none());
    assert!(g.battlefield_find(other_giant).is_none(), "another Giant shares the type");
    assert!(g.battlefield_find(bear).is_some(), "a Bear doesn't");
}

/// CR 118.9 — As Foretold: once each turn a spell with mana value up to its
/// time counters costs {0}.
#[test]
fn cr_118_9_as_foretold_casts_for_free_once_a_turn() {
    let mut g = main_phase(2);
    let foretold = g.add_card_to_battlefield(0, catalog::as_foretold());
    g.battlefield_find_mut(foretold).unwrap().add_counters(CounterType::Time, 2);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: bears,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast with no mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: second,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "only once each turn"
    );
}

/// CR 702.62 — The Face of Boe casts a suspend card from hand for its
/// suspend cost.
#[test]
fn cr_702_62_the_face_of_boe_pays_the_suspend_cost() {
    let mut g = main_phase(2);
    let boe = g.add_card_to_battlefield(0, catalog::the_face_of_boe());
    let whale = g.add_card_to_hand(0, catalog::star_whale());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    run(&mut g, Effect::CastFromHandPayingSuspendCost, boe);
    assert!(g.battlefield_find(whale).is_some(), "Star Whale for {{1}}{{U}}");
}

/// CR 702.62e — The Eleventh Doctor's damage exiles a card from hand with its
/// mana value in time counters and suspend.
#[test]
fn cr_702_62e_eleventh_doctor_suspends_from_hand() {
    let mut g = main_phase(2);
    let doctor = g.add_card_to_battlefield(0, catalog::the_eleventh_doctor());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    run(
        &mut g,
        Effect::MayExileFromHandSuspended { who: crabomination::effect::PlayerRef::You, filter: crabomination::card::SelectionRequirement::Any },
        doctor,
    );
    let c = g.exile.iter().find(|c| c.id == giant).expect("exiled from hand");
    assert_eq!(c.counter_count(CounterType::Time), 4);
    assert!(c.has_suspend());
}

/// CR 702.119 — Adipose Offspring: emerged, it makes X Aliens (X = the
/// sacrificed creature's toughness); cast normally, one.
#[test]
fn cr_702_119_adipose_offspring_reads_the_emerge_sacrifice() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::hill_giant());
    let adipose = g.add_card_to_hand(0, catalog::adipose_offspring());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: adipose,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("emerge");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Alien").len(), 3, "Hill Giant's toughness");
    let plain = g.add_card_to_hand(0, catalog::adipose_offspring());
    cast(&mut g, plain, None).expect("cast normally");
    assert_eq!(named(&g, 0, "Alien").len(), 4);
}

/// CR 702.95 — Donna Noble: her soulbond partner being dealt damage makes her
/// deal that much to an opponent.
#[test]
fn cr_702_95_donna_noble_answers_for_her_partner() {
    let mut g = main_phase(2);
    let donna = g.add_card_to_battlefield(0, catalog::donna_noble());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(donna).unwrap().soulbond_partner = Some(giant);
    g.battlefield_find_mut(giant).unwrap().soulbond_partner = Some(donna);
    let life = g.players[1].life;
    run(&mut g, Effect::DealDamage { to: Selector::ExactObjects(vec![giant]), amount: Value::Const(2) }, giant);
    assert_eq!(g.players[1].life, life - 2);
}

/// CR 500.7 — Regenerations Restored: its last time counter exiles it and
/// takes an extra turn.
#[test]
fn cr_500_7_regenerations_restored_ends_in_an_extra_turn() {
    let mut g = main_phase(2);
    let rr = g.add_card_to_battlefield(0, catalog::regenerations_restored());
    let c = g.battlefield_find_mut(rr).unwrap();
    let n = c.counter_count(CounterType::Time);
    c.remove_counters(CounterType::Time, n);
    c.add_counters(CounterType::Time, 1);
    let life = g.players[0].life;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rr).is_none());
    assert!(g.exile.iter().any(|c| c.id == rr));
    assert_eq!(g.players[0].life, life + 1);
    assert_eq!(g.players[0].extra_turns, 1);
}

/// CR 406.3 — Everything Comes to Dust with no convokers exiles every
/// creature, artifact and enchantment; lands stay.
#[test]
fn cr_406_3_everything_comes_to_dust_clears_the_board() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let droid = g.add_card_to_battlefield(1, catalog::sol_ring());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::everything_comes_to_dust());
    run(&mut g, Effect::ExileAllButConvokerKin, spell);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(droid).is_none());
    assert!(g.battlefield_find(land).is_some());
}

fn upkeep_tick(g: &mut GameState) {
    g.active_player_idx = 0;
    let ev = g.process_fading_vanishing();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// CR 702.63a — a token with vanishing enters with its time counters: The
/// Girl in the Fireplace's Human Noble lives three upkeeps, not zero (a minted
/// token got none, and its first upkeep sacrificed it).
#[test]
fn cr_702_63a_the_girl_in_the_fireplace_noble_enters_with_time_counters() {
    let mut g = main_phase(2);
    let saga = g.add_card_to_battlefield(0, catalog::the_girl_in_the_fireplace());
    let chapter_one = catalog::the_girl_in_the_fireplace().saga_chapters[0].1.clone();
    run(&mut g, chapter_one, saga);
    let noble = named(&g, 0, "Human Noble")[0];
    assert_eq!(time(&g, noble), 3);
    upkeep_tick(&mut g);
    assert_eq!(time(&g, noble), 2, "still here after its first upkeep");
}

/// CR 702.63a / 707.9b — Flesh Duplicate's copy has vanishing 3 and enters
/// with three time counters (they were placed before the copy took hold).
#[test]
fn cr_702_63a_flesh_duplicate_copy_enters_with_three_time_counters() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dup = g.add_card_to_hand(0, catalog::flesh_duplicate());
    cast(&mut g, dup, None).expect("Flesh Duplicate");
    assert_eq!(g.battlefield_find(dup).map(|c| c.definition.name), Some("Grizzly Bears"));
    assert_eq!(time(&g, dup), 3);
    upkeep_tick(&mut g);
    assert!(g.battlefield_find(dup).is_some(), "not sacrificed at its first upkeep");
}

fn activate(g: &mut GameState, card_id: CardId, ability_index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 702.62b — Clockspinning reaches a suspended card: the caster's own
/// loses a time counter; an opponent's creature loses its +1/+1 counter.
#[test]
fn cr_702_62b_clockspinning_turns_a_suspended_cards_clock() {
    let mut g = main_phase(2);
    let whale = g.add_card_to_graveyard(0, catalog::star_whale());
    suspend(&mut g, whale, 3);
    let spell = g.add_card_to_hand(0, catalog::clockspinning());
    cast(&mut g, spell, Some(Target::Permanent(whale))).expect("Clockspinning on a suspended card");
    assert_eq!(time(&g, whale), 2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let spell = g.add_card_to_hand(0, catalog::clockspinning());
    cast(&mut g, spell, Some(Target::Permanent(bear))).expect("Clockspinning on a creature");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Everybody Lives! — players gain hexproof and can't lose life this turn
/// (they can still gain it).
#[test]
fn everybody_lives_shields_the_players() {
    let mut g = main_phase(2);
    let spell = g.add_card_to_hand(0, catalog::everybody_lives());
    cast(&mut g, spell, None).expect("Everybody Lives!");
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(cast(&mut g, bolt, Some(Target::Player(1))).is_err(), "a player has hexproof");
    run(&mut g, Effect::LoseLife { who: Selector::Player(crabomination::effect::PlayerRef::Seat(1)), amount: Value::Const(5) }, spell);
    assert_eq!(g.players[1].life, life, "can't lose life");
    run(&mut g, Effect::GainLife { who: Selector::Player(crabomination::effect::PlayerRef::Seat(1)), amount: Value::Const(2) }, spell);
    assert_eq!(g.players[1].life, life + 2, "can still gain it");
}

/// Idris has the exiled artifact's activated abilities — Sol Ring's mana
/// ability (a static reading the imprint link never saw this exile's link).
#[test]
fn idris_taps_for_the_exiled_sol_ring() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let idris = g.add_card_to_hand(0, catalog::idris_soul_of_the_tardis());
    cast(&mut g, idris, None).expect("Idris");
    assert!(g.exile.iter().any(|c| c.id == ring), "Sol Ring is exiled");
    g.clear_sickness(idris);
    g.players[0].mana_pool = Default::default();
    activate(&mut g, idris, 0, None).expect("Sol Ring's ability");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// The Day of the Doctor IV — up to three Doctors stay; every other creature
/// is exiled.
#[test]
fn the_day_of_the_doctor_keeps_three_doctors() {
    let mut g = main_phase(2);
    let saga = g.add_card_to_battlefield(0, catalog::the_day_of_the_doctor());
    let docs: Vec<CardId> = [
        catalog::the_tenth_doctor(),
        catalog::the_war_doctor(),
        catalog::the_ninth_doctor(),
        catalog::the_eleventh_doctor(),
    ]
    .into_iter()
    .map(|d| g.add_card_to_battlefield(0, d))
    .collect();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    run(
        &mut g,
        Effect::ExileOtherCreaturesKeepingUpTo {
            keep: crabomination::card::SelectionRequirement::HasCreatureType(crabomination::card::CreatureType::Doctor),
            max: 3,
        },
        saga,
    );
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(docs.iter().filter(|d| g.battlefield_find(**d).is_some()).count(), 3);
    assert!(g.battlefield_find(docs[2]).is_none(), "the weakest Doctor (The Ninth, 2 power) goes");
}

/// CR 707.9b — The Eleventh Hour III: the copy is a legendary Alien named
/// Prisoner Zero, not the copied creature's name and types.
#[test]
fn cr_707_9b_the_eleventh_hour_makes_prisoner_zero() {
    let mut g = main_phase(2);
    let saga = g.add_card_to_battlefield(0, catalog::the_eleventh_hour());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let chapter = catalog::the_eleventh_hour().saga_chapters[2].1.clone();
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    ctx.source = Some(saga);
    g.resolve_effect(&chapter, &ctx).expect("chapter III");
    let zero = named(&g, 0, "Prisoner Zero");
    assert_eq!(zero.len(), 1);
    let cp = g.computed_permanent(zero[0]).unwrap();
    assert_eq!(cp.subtypes().creature_types, vec![crabomination::card::CreatureType::Alien]);
    assert!(cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
    assert_eq!((cp.power, cp.toughness), (2, 2), "still a copy of the Bears");
}

/// CR 702.26 — The Pandorica holds a permanent phased out while it stays
/// tapped; untapping it phases the permanent back in.
#[test]
fn cr_702_26_the_pandorica_releases_on_untap() {
    let mut g = main_phase(2);
    let box_ = g.add_card_to_battlefield(0, catalog::the_pandorica());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    flood(&mut g, 0);
    activate(&mut g, box_, 0, Some(Target::Permanent(giant))).expect("The Pandorica");
    assert!(g.battlefield_find(giant).is_none(), "phased out");
    let c = g.battlefield_find_mut(box_).unwrap();
    c.tapped = false;
    let ev = vec![GameEvent::PermanentUntapped { card_id: box_ }];
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some(), "phased back in");
}

/// The War Doctor — "one or more other cards are put into exile": one
/// counter for a batch, not one per card.
#[test]
fn the_war_doctor_counts_an_exile_batch_once() {
    let mut g = main_phase(2);
    let doc = g.add_card_to_battlefield(0, catalog::the_war_doctor());
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(1, catalog::hill_giant());
    run(&mut g, Effect::Exile { what: Selector::ExactObjects(vec![a, b]) }, doc);
    assert_eq!(time(&g, doc), 1);
}

/// CR 702.62e — Dinosaurs on a Spaceship's last time counter coming off
/// casts it, and its exile trigger still saw that removal: a token for it.
#[test]
fn cr_702_62e_dinosaurs_on_a_spaceship_counts_its_last_counter() {
    let mut g = main_phase(2);
    let dinos = g.add_card_to_graveyard(0, catalog::dinosaurs_on_a_spaceship());
    suspend(&mut g, dinos, 1);
    g.active_player_idx = 0;
    let ev = g.process_suspend();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dinos).is_some(), "cast off its last counter");
    assert_eq!(named(&g, 0, "Dinosaur").len(), 1, "and a token for that counter");
}
