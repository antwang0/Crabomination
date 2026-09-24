//! Commander: the Urza's Iron Alliance precon (BRC, Urza, Chief Artificer,
//! `decks::cmdr_urza`).

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
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
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

fn attack(g: &mut GameState, attacker: CardId, at: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(at) }]))
        .expect("attack");
    drain_stack(g);
}

/// Alela pumps your other fliers and makes a Faerie per artifact spell.
#[test]
fn alela_pumps_fliers_and_makes_faeries() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::alela_artful_provocateur());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    assert_eq!(pt(&g, angel), (5, 4));
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, 0, ring, None).expect("cast");
    let faerie = named(&g, 0, "Faerie");
    assert_eq!(faerie.len(), 1);
    assert_eq!(pt(&g, faerie[0]), (2, 1), "a flier itself, so it's pumped too");
}

/// Armix discards to give a defending creature -X/-X, X = artifacts you
/// control plus artifact cards in your graveyard.
#[test]
fn armix_shrinks_a_defender() {
    let mut g = pod(2);
    let armix = g.add_card_to_battlefield(0, catalog::armix_filigree_thrasher());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_hand(0, catalog::island());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    attack(&mut g, armix, 1);
    assert_eq!(pt(&g, angel), (1, 1), "Armix, the Ring, the Ring in the graveyard: -3/-3");
}

/// Hexavus trades its +1/+1 counters for flying counters, and takes counters
/// back off your other creatures.
#[test]
fn hexavus_moves_counters() {
    let mut g = pod(2);
    let hex = g.add_card_to_hand(0, catalog::hexavus());
    cast(&mut g, 0, hex, None).expect("cast");
    assert_eq!(pt(&g, hex), (6, 6));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, hex, 0, Some(Target::Permanent(bear))).expect("flying counter");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Flying));
    assert_eq!(pt(&g, hex), (5, 5));
    activate(&mut g, hex, 1, None).expect("take it back");
    assert_eq!(pt(&g, hex), (6, 6));
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Flying));
}

/// Metalcraft — with three artifacts, Indomitable Archangel's controller's
/// artifacts have shroud.
#[test]
fn indomitable_archangel_shrouds_artifacts_with_metalcraft() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::indomitable_archangel());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(!g.computed_permanent(ring).unwrap().keywords().contains(&Keyword::Shroud));
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(g.computed_permanent(ring).unwrap().keywords().contains(&Keyword::Shroud));
}

/// Kayla's Music Box banks the top card face down, then plays it this turn.
#[test]
fn kaylas_music_box_banks_and_plays() {
    let mut g = pod(2);
    let bx = g.add_card_to_battlefield(0, catalog::kaylas_music_box());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, bx, 0, None).expect("bank");
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_with == Some(bx)));
    g.battlefield_find_mut(bx).unwrap().tapped = false;
    activate(&mut g, bx, 1, None).expect("grant");
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("play it from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// March of Progress copies one artifact creature, or with overload each one.
#[test]
fn march_of_progress_copies_artifact_creatures() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    let m = g.add_card_to_hand(0, catalog::march_of_progress());
    cast(&mut g, 0, m, Some(Target::Permanent(a))).expect("cast");
    assert_eq!(named(&g, 0, "Ornithopter").len(), 3);
    let m = g.add_card_to_hand(0, catalog::march_of_progress());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative { card_id: m, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("overload");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Ornithopter").len(), 6);
}

/// One with the Machine draws the greatest mana value among your artifacts.
#[test]
fn one_with_the_machine_draws_by_mana_value() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::hexavus());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let s = g.add_card_to_hand(0, catalog::one_with_the_machine());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 6);
}

/// Sanwell — tapped by its attack, it exiles six and casts the artifact
/// creature among them; damage to it is prevented while an artifact creature
/// of yours attacks.
#[test]
fn sanwell_digs_when_tapped_and_hides_behind_artifacts() {
    let mut g = pod(2);
    let sanwell = g.add_card_to_battlefield(0, catalog::sanwell_avenger_ace());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let hexa = g.add_card_to_library(0, catalog::hexavus());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    attack(&mut g, sanwell, 1);
    assert!(g.battlefield_find(hexa).is_some(), "cast from among the six");
    assert!(g.exile.iter().all(|c| c.exiled_with != Some(sanwell)), "the rest went to the bottom");
    assert_eq!(g.players[0].library.len(), 5);

    let mut g = pod(2);
    let sanwell = g.add_card_to_battlefield(0, catalog::sanwell_avenger_ace());
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.clear_sickness(sanwell);
    g.clear_sickness(thopter);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: thopter, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(sanwell))).expect("bolt");
    assert!(g.battlefield_find(sanwell).is_some(), "prevented while the Ornithopter attacks");
}

/// Scholar of New Horizons pays with its own counter for a Plains to hand.
#[test]
fn scholar_of_new_horizons_fetches_a_plains() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::scholar_of_new_horizons());
    cast(&mut g, 0, s, None).expect("cast");
    assert_eq!(pt(&g, s), (2, 2));
    let plains = g.add_card_to_library(0, catalog::plains());
    g.clear_sickness(s);
    activate(&mut g, s, 0, None).expect("activate");
    assert!(g.players[0].hand.iter().any(|c| c.id == plains));
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Tawnos copies an artifact token and mills two.
#[test]
fn tawnos_copies_an_artifact_token() {
    let mut g = pod(2);
    let t = g.add_card_to_battlefield(0, catalog::tawnos_solemn_survivor());
    g.clear_sickness(t);
    run_treasure(&mut g);
    let treasure = named(&g, 0, "Treasure")[0];
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, t, 0, Some(Target::Permanent(treasure))).expect("copy");
    assert_eq!(named(&g, 0, "Treasure").len(), 2);
    assert_eq!(g.players[0].graveyard.len(), 2, "milled two");
}

fn run_treasure(g: &mut GameState) {
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&crabomination::effect::shortcut::mint_treasures(1), &ctx).unwrap();
}

/// Teshar returns a small creature card whenever you cast a historic spell.
#[test]
fn teshar_reanimates_on_historic_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::teshar_ancestors_apostle());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, 0, ring, None).expect("cast");
    assert!(g.battlefield_find(bear).is_some());
}

/// Thopter Shop makes Thopters and draws once a turn when they die.
#[test]
fn thopter_shop_draws_once_a_turn() {
    let mut g = pod(2);
    let shop = g.add_card_to_battlefield(0, catalog::thopter_shop());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    activate(&mut g, shop, 0, None).expect("thopter");
    let a = named(&g, 0, "Thopter")[0];
    let b = g.add_card_to_battlefield(0, catalog::ornithopter());
    let hand = g.players[0].hand.len();
    for id in [a, b] {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        cast(&mut g, 1, bolt, Some(Target::Permanent(id))).expect("bolt");
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "once each turn");
}

/// CR 205.4e — Urza's Ruinous Blast needs a legendary creature, then exiles
/// every nonland permanent that isn't legendary.
#[test]
fn urzas_ruinous_blast_spares_legends() {
    let mut g = pod(2);
    let blast = g.add_card_to_hand(0, catalog::urzas_ruinous_blast());
    assert!(cast(&mut g, 0, blast, None).is_err(), "no legendary creature");
    let alela = g.add_card_to_battlefield(0, catalog::alela_artful_provocateur());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let island = g.add_card_to_battlefield(1, catalog::island());
    cast(&mut g, 0, blast, None).expect("cast");
    assert!(g.battlefield_find(alela).is_some() && g.battlefield_find(island).is_some());
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(ring).is_none());
}

/// Metalcraft — Vedalken Humiliator's attack makes the opponents' creatures
/// vanilla 1/1s until end of turn.
#[test]
fn vedalken_humiliator_humiliates() {
    let mut g = pod(2);
    let v = g.add_card_to_battlefield(0, catalog::vedalken_humiliator());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::sol_ring());
    }
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    attack(&mut g, v, 1);
    assert_eq!(pt(&g, angel), (1, 1));
    assert!(!g.computed_permanent(angel).unwrap().keywords().contains(&Keyword::Flying));
}

/// Wire Surgeons gives an artifact creature card in your graveyard encore
/// at its mana cost: a hasty token copy per opponent.
#[test]
fn wire_surgeons_grants_encore() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::wire_surgeons());
    let thopter = g.add_card_to_graveyard(0, catalog::ornithopter());
    activate(&mut g, thopter, 0, None).expect("encore");
    assert_eq!(named(&g, 0, "Ornithopter").len(), 2, "one per opponent");
    assert!(g.exile.iter().any(|c| c.id == thopter));
}

/// Wreck Hunter counts the nonland cards the target player lost from the
/// battlefield this turn.
#[test]
fn wreck_hunter_makes_powerstones() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(bear))).expect("bolt");
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let wh = g.add_card_to_hand(0, catalog::wreck_hunter());
    cast(&mut g, 0, wh, Some(Target::Player(1))).expect("cast");
    let stones = named(&g, 0, "Powerstone");
    assert_eq!(stones.len(), 1, "only the Bears that died this turn");
    assert!(g.battlefield_find(stones[0]).unwrap().tapped);
}
