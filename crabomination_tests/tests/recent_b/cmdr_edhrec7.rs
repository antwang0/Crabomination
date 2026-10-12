//! Commander: one-card EDHREC gaps of EDHREC's top-600 commanders
//! (`decks::cmdr_edhrec7`).

use crabomination::card::{CardId, CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// Midnight Guard untaps whenever another creature enters.
#[test]
fn midnight_guard_untaps_on_another_creature() {
    let mut g = pod(2);
    let guard = ready(&mut g, 0, catalog::midnight_guard());
    g.battlefield_find_mut(guard).unwrap().tapped = true;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let _ = bear;
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, mine, &[]).expect("cast");
    assert!(!g.battlefield_find(guard).unwrap().tapped);
}

/// Gravelighter: with no death this turn every player sacrifices a creature;
/// after one, it draws instead.
#[test]
fn gravelighter_draws_or_makes_everyone_sacrifice() {
    let mut g = pod(3);
    library(&mut g, 0, 3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let gl = g.add_card_to_hand(0, catalog::gravelighter());
    cast(&mut g, gl, &[]).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(gl).is_none(), "its own controller sacrificed too — only Gravelighter");
    let hand = g.players[0].hand.len();
    let again = g.add_card_to_hand(0, catalog::gravelighter());
    cast(&mut g, again, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 1, "a creature died this turn: draw");
}

/// Raiders' Wake: an opponent's discard costs them 2 life; your end step after
/// an attack makes one discard.
#[test]
fn raiders_wake_punishes_discards() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::raiders_wake());
    g.add_card_to_hand(1, catalog::island());
    let life = g.players[1].life;
    g.players[0].attacked_this_turn = true;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
    assert_eq!(g.players[1].life, life - 2);
}

/// Alrund's Epiphany: two Birds, an extra turn, and it exiles itself.
#[test]
fn alrunds_epiphany_takes_a_turn_and_leaves() {
    let mut g = pod(2);
    let e = g.add_card_to_hand(0, catalog::alrunds_epiphany());
    cast(&mut g, e, &[]).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Bird").count(), 2);
    assert!(g.exile.iter().any(|c| c.id == e) && !g.players[0].graveyard.iter().any(|c| c.id == e));
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Tamiyo's Journal: upkeep Clues; three of them tutor a card.
#[test]
fn tamiyos_journal_trades_three_clues_for_a_tutor() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let j = ready(&mut g, 0, catalog::tamiyos_journal());
    for _ in 0..3 {
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Clue").count(), 3);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, j, 0, None).expect("tutor");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Clue").count(), 0);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Denethor: the end step after a death makes a Soldier; sacrificing another
/// creature drains each opponent.
#[test]
fn denethor_rewards_the_fallen() {
    let mut g = pod(3);
    let d = ready(&mut g, 0, catalog::denethor_ruling_steward());
    ready(&mut g, 0, catalog::grizzly_bears());
    let (a, b, me) = (g.players[1].life, g.players[2].life, g.players[0].life);
    activate(&mut g, d, 0, None).expect("sacrifice the Bears");
    assert_eq!((g.players[1].life, g.players[2].life, g.players[0].life), (a - 1, b - 1, me + 1));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Human Soldier"));
}

/// Fateful Showdown: damage equal to your hand, then a fresh hand that size.
#[test]
fn fateful_showdown_hits_for_the_hand() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    let fs = g.add_card_to_hand(0, catalog::fateful_showdown());
    let life = g.players[1].life;
    cast(&mut g, fs, &[Target::Player(1)]).expect("cast");
    assert_eq!(g.players[1].life, life - 3);
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count(), 3);
}


fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

/// Tippy-Toe: a token creation brings an additional Food (CR 614.1a); the end
/// step after gaining life draws.
#[test]
fn tippy_toe_adds_a_food_and_draws_after_lifegain() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::tippy_toe_terrific_partner());
    let alarm = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, alarm, &[]).expect("cast");
    assert_eq!(tokens_named(&g, "Food"), 1);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "no life gained yet");
    g.players[0].life_gained_this_turn = 3;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Enchanted Evening: every permanent is also an enchantment (CR 205.1b,
/// layer 4).
#[test]
fn enchanted_evening_makes_everything_an_enchantment() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::enchanted_evening());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.card_types().contains(&CardType::Enchantment) && cp.card_types().contains(&CardType::Creature));
}

/// Deranged Assistant: {T}, mill a card: {C}.
#[test]
fn deranged_assistant_mills_for_colorless() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let da = ready(&mut g, 0, catalog::deranged_assistant());
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: da,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
}

/// Serra the Benevolent's −6 emblem: with a creature, damage leaves you at 1
/// (CR 114.4 — an emblem's static works from the command zone).
#[test]
fn serra_emblem_keeps_you_at_one() {
    let mut g = pod(2);
    let serra = ready(&mut g, 0, catalog::serra_the_benevolent());
    g.battlefield_find_mut(serra).unwrap().counters.insert(CounterType::Loyalty, 6);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: serra, ability_index: 2, target: None, x_value: None })
        .expect("ultimate");
    drain_stack(&mut g);
    ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].life = 3;
    g.active_player_idx = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 1);
}

/// Greensleeves: */* equal to your lands; landfall makes a 3/3 Badger.
#[test]
fn greensleeves_counts_lands_and_makes_badgers() {
    let mut g = pod(2);
    let gs = ready(&mut g, 0, catalog::greensleeves_maro_sorcerer());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(gs).unwrap().power, 3);
    let land = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert!(tokens_named(&g, "Badger") >= 1);
    assert_eq!(g.computed_permanent(gs).unwrap().toughness, 4);
}

/// Astarion: the Feed mode makes an opponent lose what they lost this turn.
#[test]
fn astarion_feeds_on_the_turns_losses() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::astarion_the_decadent());
    g.players[1].life = 30;
    g.players[1].life_lost_this_turn = 4;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 26);
}

/// Jaws: a noncreature artifact sacrificed pings each opponent.
#[test]
fn jaws_pings_on_a_sacrificed_artifact() {
    let mut g = pod(3);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::jaws_relentless_predator());
    let stone = ready(&mut g, 0, catalog::mind_stone());
    let (a, b) = (g.players[1].life, g.players[2].life);
    activate(&mut g, stone, 1, None).expect("crack the Mind Stone");
    assert_eq!((g.players[1].life, g.players[2].life), (a - 1, b - 1));
}

/// Profane Transfusion: two players swap life totals and you get a Horror as
/// big as the gap.
#[test]
fn profane_transfusion_swaps_and_makes_the_gap() {
    let mut g = pod(3);
    g.players[1].life = 30;
    g.players[2].life = 12;
    let pt = g.add_card_to_hand(0, catalog::profane_transfusion());
    cast(&mut g, pt, &[Target::Player(1), Target::Player(2)]).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (12, 30));
    let horror = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Horror").expect("horror").id;
    assert_eq!(g.computed_permanent(horror).unwrap().power, 18);
}

/// The Ten Rings: your end step draws up to ten.
#[test]
fn the_ten_rings_refills_to_ten() {
    let mut g = pod(2);
    library(&mut g, 0, 12);
    ready(&mut g, 0, catalog::the_ten_rings());
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 10);
}

/// Oswald Fiddlebender: sacrifice a mana value 1 artifact for a mana value 2
/// one from the library.
#[test]
fn oswald_climbs_the_artifact_ladder() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::mind_stone());
    g.add_card_to_library(0, catalog::sol_ring());
    let os = ready(&mut g, 0, catalog::oswald_fiddlebender());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    activate(&mut g, os, 0, None).expect("tinker");
    assert!(g.battlefield_find(ring).is_none());
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Mind Stone"));
}

/// Toralf's Hammer: +3/+0 only on a legendary bearer.
#[test]
fn toralfs_hammer_favours_legends() {
    let mut g = pod(2);
    let hammer_def = *catalog::toralf_god_of_fury().back_face.unwrap();
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let legend = ready(&mut g, 0, catalog::oswald_fiddlebender());
    let h1 = ready(&mut g, 0, hammer_def.clone());
    let h2 = ready(&mut g, 0, hammer_def);
    g.battlefield_find_mut(h1).unwrap().attached_to = Some(bear);
    g.battlefield_find_mut(h2).unwrap().attached_to = Some(legend);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
    assert_eq!(g.computed_permanent(legend).unwrap().power, 5);
}

/// Ugin's Labyrinth: {C} alone, {C}{C} with a card imprinted.
#[test]
fn ugins_labyrinth_doubles_with_an_imprint() {
    let mut g = pod(2);
    let lab = ready(&mut g, 0, catalog::ugins_labyrinth());
    g.players[0].mana_pool = Default::default();
    activate_free(&mut g, lab, 0);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
    g.battlefield_find_mut(lab).unwrap().tapped = false;
    g.players[0].mana_pool = Default::default();
    let ulamog = g.add_card_to_hand(0, catalog::ulamog_the_infinite_gyre());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let lab2 = g.add_card_to_hand(0, catalog::ugins_labyrinth());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(lab2)).expect("land");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ulamog), "imprinted");
    g.clear_sickness(lab2);
    g.players[0].mana_pool = Default::default();
    activate_free(&mut g, lab2, 0);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
}

fn activate_free(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Death's Oasis: a nontoken creature of yours dying mills two, then returns
/// a creature card of lesser mana value.
#[test]
fn deaths_oasis_returns_a_lesser_creature() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::llanowar_elves());
    g.add_card_to_library(0, catalog::island());
    ready(&mut g, 0, catalog::deaths_oasis());
    let os = ready(&mut g, 0, catalog::oswald_fiddlebender());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[perm(os)]).expect("murder");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Llanowar Elves"));
}

/// Dance of the Dead: reanimates tapped at +1/+1; the creature skips its
/// untap step (CR 502.3).
#[test]
fn dance_of_the_dead_returns_a_tapped_dancer() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let dance = g.add_card_to_hand(0, catalog::dance_of_the_dead());
    cast(&mut g, dance, &[]).expect("cast");
    let p = g.battlefield_find(bear).expect("danced");
    assert!(p.tapped && p.controller == 0);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Black Panther: a creature entering grows a land; {3} moves the counters
/// onto a creature for life and a card.
#[test]
fn black_panther_mines_vibranium() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let bp = ready(&mut g, 0, catalog::black_panther_wakandan_king());
    let land = ready(&mut g, 0, catalog::forest());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]).expect("bear");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bp,
        ability_index: 0,
        target: Some(perm(land)),
        additional_targets: vec![perm(bear)],
        x_value: None,
        mode: None,
    })
    .expect("mine");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (life + 1, hand + 1));
}

/// Botanical Brawler: enters with two counters; the first counters on another
/// permanent each turn grow it once.
#[test]
fn botanical_brawler_grows_once_per_permanent() {
    let mut g = pod(2);
    let bb = g.add_card_to_hand(0, catalog::botanical_brawler());
    cast(&mut g, bb, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bb).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let bp = ready(&mut g, 0, catalog::black_panther_wakandan_king());
    let land = ready(&mut g, 0, catalog::forest());
    let _ = (bp, land);
    for _ in 0..2 {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, bear, &[]).expect("bear");
    }
    assert_eq!(g.battlefield_find(bb).unwrap().counter_count(CounterType::PlusOnePlusOne), 3, "the land grew twice, counted once");
}

/// Gixian Puppeteer: your second draw each turn drains 2.
#[test]
fn gixian_puppeteer_drains_on_the_second_draw() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    ready(&mut g, 0, catalog::gixian_puppeteer());
    let life = g.players[1].life;
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, &[]).expect("cast");
    assert_eq!(g.players[1].life, life - 2);
}

/// Expel from Orazca: without the city's blessing, the permanent goes home.
#[test]
fn expel_from_orazca_bounces() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ex = g.add_card_to_hand(0, catalog::expel_from_orazca());
    cast(&mut g, ex, &[perm(bear)]).expect("cast");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

/// Spectrum Sentinel: an opponent's nonbasic land gains you 1, a basic none.
#[test]
fn spectrum_sentinel_taxes_nonbasics() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::spectrum_sentinel());
    let life = g.players[0].life;
    g.active_player_idx = 1;
    for land in [catalog::forest(), catalog::ugins_labyrinth()] {
        let id = g.add_card_to_hand(1, land);
        g.players[1].lands_played_this_turn = 0;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::PlayLand(id)).expect("land");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life + 1, "the Labyrinth only");
}

/// Ascend from Avernus: creature cards with mana value X or less return (the
/// X atom reads the spell's X, CR 107.3); it then exiles itself.
#[test]
fn ascend_from_avernus_returns_up_to_x() {
    let mut g = pod(2);
    let elves = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let os = g.add_card_to_graveyard(0, catalog::black_panther_wakandan_king());
    let big = g.add_card_to_graveyard(0, catalog::gixian_puppeteer());
    let a = g.add_card_to_hand(0, catalog::ascend_from_avernus());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: a, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("cast");
    drain_stack(&mut g);
    for id in [elves, bear, os] {
        assert!(g.battlefield_find(id).is_some(), "mana value 2 or less returns");
    }
    assert!(g.battlefield_find(big).is_none(), "mana value 4 stays");
    assert!(g.exile.iter().any(|c| c.id == a));
}

/// Sigarda's Splendor notes your life as it enters; the upkeep draws only at
/// or above the note.
#[test]
fn sigardas_splendor_draws_while_life_holds() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let life = g.players[0].life;
    let s = g.add_card_to_hand(0, catalog::sigardas_splendor());
    cast(&mut g, s, &[]).expect("cast");
    let guard = g.add_card_to_hand(0, catalog::midnight_guard());
    cast(&mut g, guard, &[]).expect("a white spell");
    assert_eq!(g.players[0].life, life + 1, "a white spell cast with Splendor out gains 1");
    let hand = g.players[0].hand.len();
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "life held");
    g.players[0].life -= 5;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "life fell below the note");
}

/// Jirina, Dauntless General: exile a graveyard; sacrifice for hexproof and
/// indestructible Humans.
#[test]
fn jirina_exiles_a_graveyard_and_shields_humans() {
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let j = g.add_card_to_hand(0, catalog::jirina_dauntless_general());
    cast(&mut g, j, &[Target::Player(1)]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == dead));
    let human = ready(&mut g, 0, catalog::oswald_fiddlebender());
    let _ = human;
    let soldier = ready(&mut g, 0, catalog::midnight_guard());
    g.clear_sickness(j);
    activate(&mut g, j, 0, None).expect("sacrifice");
    assert!(g.battlefield_find(j).is_none());
    let kws = g.computed_permanent(soldier).unwrap().keywords().to_vec();
    assert!(kws.contains(&crabomination::card::Keyword::Hexproof) && kws.contains(&crabomination::card::Keyword::Indestructible));
}

/// Embalmer's Tools: a creature card's graveyard ability costs {1} less; a
/// tapped Zombie mills a target player.
#[test]
fn embalmers_tools_mills_with_a_zombie() {
    let mut g = pod(2);
    library(&mut g, 1, 2);
    let tools = ready(&mut g, 0, catalog::embalmers_tools());
    let zombie = ready(&mut g, 0, catalog::gravelighter());
    let _ = zombie;
    let z = g.add_card_to_battlefield(0, crabomination::card::CardDefinition {
        subtypes: crabomination::card::Subtypes {
            creature_types: vec![crabomination::card::CreatureType::Zombie],
            ..Default::default()
        },
        ..catalog::grizzly_bears()
    });
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tools,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap a Zombie");
    drain_stack(&mut g);
    assert!(g.battlefield_find(z).unwrap().tapped);
    assert_eq!(g.players[1].graveyard.len(), 1);
}
