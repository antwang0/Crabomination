//! Functionality tests for the `catalog::sets::decks::recent8` batch — the
//! Avatar bending mechanics (earthbend / airbend) and Lorwyn's blight, plus
//! the rideable commons in the same wave.

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

// ── Earthbend (CR 701.66) ──────────────────────────────────────────────────

/// Badgermole Cub's ETB earthbends a land into a 1/1 hasty land creature.
#[test]
fn badgermole_cub_earthbends_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::badgermole_cub());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let comp = g.computed_permanent(land).expect("land still on battlefield");
    assert!(comp.card_types.contains(&CardType::Creature), "land is now a creature");
    assert!(comp.card_types.contains(&CardType::Land), "still a land");
    assert!(comp.keywords.contains(&Keyword::Haste), "has haste");
    assert_eq!(comp.power, 1, "0/0 + one +1/+1 counter");
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Badgermole earthbends 2 — the land gains two +1/+1 counters.
#[test]
fn badgermole_earthbends_two() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::badgermole());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
}

/// Earthbending Student earthbends 2 on ETB.
#[test]
fn earthbending_student_earthbends() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::earthbending_student());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
}

/// Earth Village Ruffians earthbends 2 when it dies.
#[test]
fn earth_village_ruffians_earthbends_on_death() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let ruffians = g.add_card_to_battlefield(0, catalog::earth_village_ruffians());
    g.remove_to_graveyard_with_triggers(ruffians);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "death trigger earthbent the land"
    );
}

/// Earthbender Ascension earthbends 2, then ramps a basic onto the battlefield
/// tapped.
#[test]
fn earthbender_ascension_earthbends_and_ramps() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let fetched = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::earthbender_ascension());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(fetched))]));
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
    let ramped = g.battlefield_find(fetched).expect("basic ramped to battlefield");
    assert!(ramped.tapped, "ramped land enters tapped");
}

// ── Blight (CR 701.68) ─────────────────────────────────────────────────────

/// Blighted Blackthorn blights itself for 2, draws, and loses 1 life.
#[test]
fn blighted_blackthorn_blights_draws_loses_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::blighted_blackthorn());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bb = id;
    cast(&mut g, bb);
    assert_eq!(
        g.battlefield_find(bb).unwrap().counter_count(CounterType::MinusOneMinusOne),
        2,
        "blighted itself (only creature)"
    );
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
}

/// Chaos Spewer blights 2 when its controller can't pay {2}.
#[test]
fn chaos_spewer_blights_when_unpaid() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chaos_spewer());
    // Exactly enough to cast {2}{B/R}, nothing left for the "pay {2}" rider.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::MinusOneMinusOne),
        2,
        "couldn't pay the rider, blighted itself"
    );
}

/// Boggart Mischief blights a creature for 1 and mints two Goblins.
#[test]
fn boggart_mischief_blights_and_makes_goblins() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::boggart_mischief());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(victim).unwrap().counter_count(CounterType::MinusOneMinusOne),
        1
    );
    let goblins = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin")
        .count();
    assert_eq!(goblins, 2, "two Goblin tokens");
}

// ── Airbend (CR 701.65) ────────────────────────────────────────────────────

/// Airbending Lesson exiles a nonland permanent with a {2} may-cast grant and
/// draws a card.
#[test]
fn airbending_lesson_exiles_and_draws() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::airbending_lesson());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, id, Target::Permanent(victim));
    let exiled = g.exile.iter().find(|c| c.id == victim).expect("airbent to exile");
    assert!(exiled.may_play_until.is_some(), "owner may cast it");
    assert_eq!(exiled.granted_alt_cast_cost_eot.as_ref().map(|c| c.cmc()), Some(2));
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// The exile view surfaces airbend's {2} alt-cast cost so the client can
/// render "play for {2}".
#[test]
fn airbend_exile_view_shows_alt_cost() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::airbending_lesson());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::forest());
    cast_at(&mut g, id, Target::Permanent(victim));
    let view = crate::server::view::project(&g, 1);
    let entry = view.exile.iter().find(|e| e.id == victim).expect("airbent card in exile view");
    assert_eq!(entry.may_play_recipient, Some(1), "owner may play it");
    assert_eq!(entry.may_play_alt_cost, Some(2), "renders play-for-{{2}}");
}

/// Aang airbends another nonland permanent on ETB.
#[test]
fn aang_airbends_on_etb() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::aang_the_last_airbender());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.exile.iter().any(|c| c.id == victim), "Aang airbent the opposing creature");
}

/// Airbender Ascension airbends a creature on ETB.
#[test]
fn airbender_ascension_airbends() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::airbender_ascension());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.exile.iter().any(|c| c.id == victim), "airbent the creature");
}

/// Whirlwind Technique draws two, discards one, and airbends a creature.
#[test]
fn whirlwind_technique_draws_and_airbends() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::whirlwind_technique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast_at(&mut g, id, Target::Permanent(victim));
    assert!(g.exile.iter().any(|c| c.id == victim), "airbent a creature");
}

/// Glider Staff grants the equipped creature flying.
#[test]
fn glider_staff_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let staff = g.add_card_to_battlefield(0, catalog::glider_staff());
    g.players[0].mana_pool.add_colorless(1);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: staff, target: bear }).expect("equip");
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying), "equipped creature flies");
}

// ── Rideable commons ───────────────────────────────────────────────────────

/// Corrupt Court Official makes an opponent discard on ETB.
#[test]
fn corrupt_court_official_makes_opponent_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    let before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::corrupt_court_official());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded");
}

/// Jeong Jeong's Deserters puts a +1/+1 counter on a creature.
#[test]
fn jeong_jeongs_deserters_counters_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jeong_jeongs_deserters());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Forecasting Fortune Teller creates a Clue on ETB.
#[test]
fn forecasting_fortune_teller_makes_a_clue() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forecasting_fortune_teller());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"),
        "Clue token created"
    );
}

/// Pretending Poxbearers mints a 1/1 Ally when it dies.
#[test]
fn pretending_poxbearers_makes_ally_on_death() {
    let mut g = two_player_game();
    let pp = g.add_card_to_battlefield(0, catalog::pretending_poxbearers());
    g.remove_to_graveyard_with_triggers(pp);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ally"),
        "Ally token created on death"
    );
}

/// Merchant of Many Hats returns itself from the graveyard to hand.
#[test]
fn merchant_of_many_hats_returns_from_graveyard() {
    let mut g = two_player_game();
    let merchant = g.add_card_to_graveyard(0, catalog::merchant_of_many_hats());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: merchant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate gy ability");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == merchant), "returned to hand");
}

/// Yuyan Archers loots (discard a card to draw a card) on ETB.
#[test]
fn yuyan_archers_loots() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::yuyan_archers());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id);
    // Cast Yuyan (-1 hand), discard 1, draw 1 → net hand = before - 1 (the cast).
    assert_eq!(g.players[0].hand.len(), hand - 1);
}

/// Platypus-Bear mills two cards on ETB.
#[test]
fn platypus_bear_mills_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let lib = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::platypus_bear());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[0].library.len(), lib - 2, "milled two");
}

/// Compassionate Healer gains life and scrys when it becomes tapped.
#[test]
fn compassionate_healer_triggers_on_tap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let healer = g.add_card_to_battlefield(0, catalog::compassionate_healer());
    let life = g.players[0].life;
    g.battlefield_find_mut(healer).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: healer }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 life on tap");
}

/// Fire Nation Soldier is a 3/2 with haste.
#[test]
fn fire_nation_soldier_has_haste() {
    let def = catalog::fire_nation_soldier();
    assert_eq!((def.power, def.toughness), (3, 2));
    assert!(def.keywords.contains(&Keyword::Haste));
}
