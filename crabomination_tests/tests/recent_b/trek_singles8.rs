//! Star Trek: the eighth batch of EDHREC closers (`decks::trek_singles8`).

use crabomination::card::{CardId, CounterType, Keyword, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn cast(g: &mut GameState, id: CardId) {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// Commander Raffi Musiker: the first mode exiles a graveyard card and gains
/// 2; the second investigates and loses 2.
#[test]
fn raffi_musiker_exiles_or_investigates() {
    let mut g = pod(3);
    g.add_card_to_graveyard(2, catalog::grizzly_bears());
    let raffi = g.add_card_to_hand(0, catalog::commander_raffi_musiker());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let life = g.players[0].life;
    cast(&mut g, raffi);
    assert!(g.players[2].graveyard.is_empty(), "the Bears exiled");
    assert_eq!(g.players[0].life, life + 2);

    let mut g = pod(2);
    let raffi = g.add_card_to_hand(0, catalog::commander_raffi_musiker());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let life = g.players[0].life;
    cast(&mut g, raffi);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
    assert_eq!(g.players[0].life, life - 2);
}

/// Culber and Stamets: the mana mode adds two of one color on entry.
#[test]
fn culber_and_stamets_adds_two_mana() {
    let mut g = pod(2);
    let cs = g.add_card_to_hand(0, catalog::culber_and_stamets_star_crossed());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast(&mut g, cs);
    assert!(g.battlefield_find(cs).is_some());
    assert_eq!(g.players[0].mana_pool.total(), 2, "two mana from the trigger");
}

/// Janice Rand: a commander you control entering gets a +1/+1 counter and
/// fetches the first land, tapped; a non-commander creature does nothing.
#[test]
fn janice_rand_rewards_only_your_commander() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::janice_rand_essential_aide());
    g.add_card_to_library(0, catalog::forest());
    let lands = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    cast(&mut g, bear);
    assert_eq!(lands(&g), 0, "not a commander");
    let cmd = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].commanders.push(cmd);
    cast(&mut g, cmd);
    let giant = g.battlefield_find(cmd).expect("the commander");
    assert_eq!(giant.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(lands(&g), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.is_land() && c.tapped), "enters tapped");
}

/// La'An Noonien-Singh: attacking with total power 8 or more makes a creature
/// unable to block; at 7 the trigger doesn't fire.
#[test]
fn laan_needs_total_power_eight() {
    for (extra, fires) in [(false, false), (true, true)] {
        let mut g = pod(2);
        let laan = ready(&mut g, 0, catalog::laan_noonien_singh_security());
        ready(&mut g, 0, catalog::hill_giant());
        if extra {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        let blocker = ready(&mut g, 1, catalog::grizzly_bears());
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: laan, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        let cant = g.permanent_has_keyword(blocker, &Keyword::CantBlock);
        assert_eq!(cant, fires, "total power {}", if extra { 9 } else { 7 });
    }
}

/// Lieutenant Nyota Uhura: X is your differently named lands; she casts one
/// instant or sorcery from the exiled cards free, the rest go to the bottom.
#[test]
fn uhura_casts_a_spell_off_the_opponents_library() {
    let mut g = pod(2);
    let uhura = ready(&mut g, 0, catalog::lieutenant_nyota_uhura());
    for land in [catalog::island(), catalog::forest(), catalog::forest()] {
        g.add_card_to_battlefield(0, land);
    }
    // Two distinct names: the top two of the opponent's library.
    g.add_card_to_library(1, catalog::divination());
    g.players[1].library.rotate_right(1);
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.players[1].library.rotate_right(1);
    let size = g.players[1].library.len();
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: uhura,
        ability_index: 0,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination cast free");
    assert_eq!(g.players[1].library.len(), size - 1, "the Bears went back");
    assert_eq!(g.players[1].library.last().map(|c| c.definition.name), Some("Grizzly Bears"));
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Divination"));
}

/// Rok-Tahk: entering makes Murf (legendary, indestructible), and every
/// creature you control taps for {G}{G}.
#[test]
fn rok_tahk_makes_murf_and_grants_gg() {
    let mut g = pod(2);
    let rok = g.add_card_to_hand(0, catalog::rok_tahk_budding_scientist());
    flood(&mut g, 0);
    cast(&mut g, rok);
    let murf = g.battlefield.iter().find(|c| c.definition.name == "Murf").expect("Murf");
    assert!(murf.definition.supertypes.contains(&Supertype::Legendary));
    let murf = murf.id;
    assert!(g.permanent_has_keyword(murf, &Keyword::Indestructible));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

/// Georgiou: her end step takes you below 0 without losing, and at 0 or less
/// life she makes ten Officers.
#[test]
fn georgiou_survives_below_zero_and_makes_officers() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::georgiou_terran_emperor());
    g.players[0].life = 2;
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, -1);
    assert!(!g.players[0].eliminated, "CR 104.3b: her static keeps her in");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    let officers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Officer").count();
    assert_eq!(officers, 10);
}

/// Cruel Bargain: half your life, rounded up (7 → 3).
#[test]
fn cruel_bargain_draws_four_and_halves_life_rounding_up() {
    let mut g = pod(2);
    g.players[0].life = 7;
    let spell = g.add_card_to_hand(0, catalog::cruel_bargain());
    g.players[0].mana_pool.add(Color::Black, 3);
    let hand = g.players[0].hand.len();
    cast(&mut g, spell);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4);
    assert_eq!(g.players[0].life, 3);
}

/// Daniels: from the graveyard, {2} and bottoming himself gains 2 life.
#[test]
fn daniels_bottoms_himself_for_life() {
    let mut g = pod(2);
    let daniels = g.add_card_to_graveyard(0, catalog::daniels_temporal_agent());
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: daniels,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate from graveyard");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(g.players[0].library.last().map(|c| c.definition.name), Some("Daniels, Temporal Agent"));
}

/// Stray from History: Rewrite the Past returns two creature cards.
#[test]
fn stray_from_history_returns_two_creatures() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::stray_from_history());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(crabomination::game::types::Target::Permanent(a)),
        additional_targets: vec![crabomination::game::types::Target::Permanent(b)],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
}
