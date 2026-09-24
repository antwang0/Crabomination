//! Commander: the Growing Threat precon (MOC, Brimaz, `decks::cmdr_brimaz`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
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

fn cast(g: &mut GameState, id: CardId) {
    cast_at(g, id, &[]).expect("cast");
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

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(kind)).unwrap_or(0)
}

fn incubators(g: &GameState, seat: usize) -> Vec<u32> {
    named(g, seat, "Incubator").into_iter().map(|id| counters(g, id, CounterType::PlusOnePlusOne)).collect()
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

/// CR 701.53 — Brimaz incubates a Phyrexian spell's mana value, and a
/// Phyrexian dying under your control proliferates at the end step.
#[test]
fn cr_701_53_brimaz_incubates_phyrexian_spells_and_proliferates() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::brimaz_blight_of_oreskos());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear);
    assert!(incubators(&g, 0).is_empty(), "a Bear isn't Phyrexian");
    let fox = g.add_card_to_hand(0, catalog::vulpine_harvester());
    cast(&mut g, fox);
    assert_eq!(incubators(&g, 0), vec![4]);
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, kill, &[Target::Permanent(fox)]).expect("murder");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(incubators(&g, 0), vec![5], "proliferated");
}

/// Ancient Stone Idol costs {1} less per attacker and leaves a 6/12.
#[test]
fn ancient_stone_idol_leaves_a_construct() {
    let mut g = main_phase(2);
    let idol = g.add_card_to_hand(0, catalog::ancient_stone_idol());
    cast(&mut g, idol);
    let kill = g.add_card_to_hand(0, catalog::murder());
    // An artifact creature, so Murder (not Doom Blade's nonblack clause) is fine.
    cast_at(&mut g, kill, &[Target::Permanent(idol)]).expect("murder");
    let c = named(&g, 0, "Construct");
    assert_eq!(c.len(), 1);
    let cp = g.computed_permanent(c[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 12));
    assert!(cp.keywords().contains(&Keyword::Trample));
}

/// Living weapon (CR 702.92) — the Germ carries Bitterthorn, and attacking
/// fetches a basic land tapped.
#[test]
fn bitterthorn_germ_attacks_for_a_land() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::forest());
    let bt = g.add_card_to_hand(0, catalog::bitterthorn_nissas_animus());
    cast(&mut g, bt);
    let germ = named(&g, 0, "Phyrexian Germ");
    assert_eq!(germ.len(), 1);
    let cp = g.computed_permanent(germ[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &germ, 1);
    let forests = named(&g, 0, "Forest");
    assert_eq!(forests.len(), 1);
    assert!(g.battlefield_find(forests[0]).unwrap().tapped);
}

/// Blight Titan mills two, then incubates per creature card in your
/// graveyard.
#[test]
fn blight_titan_incubates_its_graveyard() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::blight_titan());
    cast(&mut g, t);
    assert_eq!(incubators(&g, 0), vec![2]);
}

/// Cataclysmic Gearhulk leaves each player one nonland permanent of each
/// type.
#[test]
fn cataclysmic_gearhulk_keeps_one_of_each() {
    let mut g = main_phase(3);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let ring = g.add_card_to_battlefield(2, catalog::sol_ring());
    let gh = g.add_card_to_hand(0, catalog::cataclysmic_gearhulk());
    cast(&mut g, gh);
    assert!(g.battlefield_find(angel).is_some() && g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(land).is_some() && g.battlefield_find(ring).is_some());
    assert!(g.battlefield_find(gh).is_some());
}

/// Darksteel Splicer makes a Golem per opponent for itself and each later
/// nontoken Phyrexian, and its Golems are indestructible.
#[test]
fn darksteel_splicer_makes_a_golem_per_opponent() {
    let mut g = main_phase(3);
    let ds = g.add_card_to_hand(0, catalog::darksteel_splicer());
    cast(&mut g, ds);
    let golems = named(&g, 0, "Phyrexian Golem");
    assert_eq!(golems.len(), 2);
    assert!(g.computed_permanent(golems[0]).unwrap().keywords().contains(&Keyword::Indestructible));
    let fox = g.add_card_to_hand(0, catalog::vulpine_harvester());
    cast(&mut g, fox);
    assert_eq!(named(&g, 0, "Phyrexian Golem").len(), 4, "a nontoken Phyrexian entered; the tokens didn't");
}

/// Excise the Imperfect: its controller incubates the exiled card's mana
/// value.
#[test]
fn cr_701_53_excise_incubates_for_the_victim() {
    let mut g = main_phase(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let e = g.add_card_to_hand(0, catalog::excise_the_imperfect());
    cast_at(&mut g, e, &[Target::Permanent(angel)]).expect("excise");
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!(incubators(&g, 1), vec![5]);
    assert!(incubators(&g, 0).is_empty());
}

/// Filigree Vector counters your board, then proliferates by sacrificing
/// another artifact.
#[test]
fn filigree_vector_counters_and_proliferates() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let fv = g.add_card_to_hand(0, catalog::filigree_vector());
    cast(&mut g, fv);
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 1);
    assert_eq!(counters(&g, ring, CounterType::Charge), 1);
    g.clear_sickness(fv);
    activate(&mut g, fv, 0, None).expect("proliferate");
    assert!(g.battlefield_find(ring).is_none(), "Sol Ring was sacrificed");
    assert_eq!(counters(&g, bear, CounterType::PlusOnePlusOne), 2);
}

/// First-Sphere Gargantua draws on entry, and unearths from the graveyard.
#[test]
fn first_sphere_gargantua_draws_and_unearths() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::plains());
    let fs = g.add_card_to_graveyard(0, catalog::first_sphere_gargantua());
    let life = g.players[0].life;
    activate(&mut g, fs, 0, None).expect("unearth");
    assert!(g.battlefield_find(fs).is_some());
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, life - 1);
    assert!(g.computed_permanent(fs).unwrap().keywords().contains(&Keyword::Haste));
}

/// CR 901.9 — Fractured Powerstone rolls the planar die; with Ichor Elixir
/// out, a blank plus a Planeswalker face keeps the Planeswalker.
#[test]
fn cr_901_9_powerstone_and_elixir_roll_planar_dice() {
    let mut g = main_phase(2);
    g.seat_planar_deck(0, vec![catalog::naar_isle(), catalog::the_hippodrome()]);
    g.set_starting_plane(0);
    let plane = |g: &GameState| g.face_up_planes().first().and_then(|&id| g.find_card_anywhere(id)).map(|c| c.definition.name);
    let ps = g.add_card_to_battlefield(0, catalog::fractured_powerstone());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5)]));
    activate(&mut g, ps, 1, None).expect("roll");
    assert_eq!(plane(&g), Some("Naar Isle"), "a blank");
    g.add_card_to_battlefield(0, catalog::ichor_elixir());
    g.battlefield_find_mut(ps).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5), DecisionAnswer::DieRoll(1)]));
    activate(&mut g, ps, 1, None).expect("roll");
    assert_eq!(plane(&g), Some("The Hippodrome"), "the extra die planeswalked");
}

/// Keskit sacrifices three others to take two of the top three.
#[test]
fn keskit_digs_for_two() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let k = g.add_card_to_battlefield(0, catalog::keskit_the_flesh_sculptor());
    g.clear_sickness(k);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(activate(&mut g, k, 0, None).is_err(), "only two others");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, k, 0, None).expect("activate");
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[0].library.len(), 0);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Plains"));
}

/// Moira and Teshar return a permanent with haste per historic spell; it is
/// exiled at the next end step.
#[test]
fn moira_and_teshar_return_a_permanent_for_a_turn() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::moira_and_teshar());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring);
    assert!(g.battlefield_find(bear).is_some(), "returned");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.exile.iter().any(|c| c.id == bear));
}

/// Path of the Schemer mills everyone and reanimates the best creature card
/// as an artifact.
#[test]
fn path_of_the_schemer_steals_a_creature_as_an_artifact() {
    let mut g = main_phase(2);
    g.add_card_to_library(1, catalog::plains());
    let angel = g.add_card_to_library(1, catalog::serra_angel());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::plains());
    }
    let p = g.add_card_to_hand(0, catalog::path_of_the_schemer());
    cast(&mut g, p);
    let a = g.computed_permanent(angel).expect("the Angel came back");
    assert_eq!(a.controller, 0);
    assert!(a.card_types().contains(&CardType::Artifact));
}

/// Phyrexian Triniform leaves three Golems.
#[test]
fn phyrexian_triniform_splits_into_golems() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::phyrexian_triniform());
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast_at(&mut g, kill, &[Target::Permanent(t)]).expect("murder");
    assert_eq!(named(&g, 0, "Phyrexian Golem").len(), 3);
    assert!(in_graveyard(&g, 0, t));
}

/// Vulpine Harvester returns an artifact whose mana value fits under the
/// attacking Phyrexians' power, and not a bigger one.
#[test]
fn vulpine_harvester_returns_an_artifact_it_can_carry() {
    let mut g = main_phase(2);
    let fox = g.add_card_to_battlefield(0, catalog::vulpine_harvester());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ring))]));
    attack_with(&mut g, &[fox], 1);
    assert!(g.battlefield_find(ring).is_some(), "mana value 1 <= power 3");

    let mut g = main_phase(2);
    let fox = g.add_card_to_battlefield(0, catalog::vulpine_harvester());
    let idol = g.add_card_to_graveyard(0, catalog::ancient_stone_idol());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(idol))]));
    attack_with(&mut g, &[fox], 1);
    assert!(in_graveyard(&g, 0, idol), "mana value 10 > power 3");
}

/// CR 205.4e — Yawgmoth's Vile Offering needs a legendary creature; it
/// reanimates, destroys, and exiles itself.
#[test]
fn cr_205_4e_yawgmoths_vile_offering_reanimates_and_destroys() {
    let mut g = main_phase(2);
    let dead = g.add_card_to_graveyard(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let y = g.add_card_to_hand(0, catalog::yawgmoths_vile_offering());
    let ts = [Target::Permanent(dead), Target::Permanent(bear)];
    assert!(cast_at(&mut g, y, &ts).is_err(), "no legendary creature");
    g.add_card_to_battlefield(0, catalog::brimaz_blight_of_oreskos());
    cast_at(&mut g, y, &ts).expect("cast");
    assert_eq!(g.computed_permanent(dead).expect("reanimated").controller, 0);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == y), "exiled itself");
}
