//! Commander: the Undead Unleashed precon (MIC, Wilhelt, the Rotcleaver,
//! `decks::cmdr_wilhelt`) and the primitives it needed.

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn connect(g: &mut GameState, a: CardId, victim: usize) {
    g.clear_sickness(a);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(victim) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = victim;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        // A bare `advance_step` hands its events back rather than dispatching
        // them; the priority loop would.
        if let Ok(ev) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&ev);
        }
        drain_stack(g);
    }
}

/// Cleaver Skaab sacrifices another Zombie for two token copies of it.
#[test]
fn cleaver_skaab_doubles_the_sacrificed_zombie() {
    let mut g = pod(2);
    let skaab = g.add_card_to_battlefield(0, catalog::cleaver_skaab());
    g.add_card_to_battlefield(0, catalog::tomb_tyrant());
    g.clear_sickness(skaab);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skaab,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let tyrants = named(&g, 0, "Tomb Tyrant");
    assert_eq!(tyrants.len(), 2);
    assert!(tyrants.iter().all(|id| g.battlefield_find(*id).unwrap().is_token));
}

/// Curse of the Restless Dead: a land entering under the cursed player gives
/// the curse's controller a decayed Zombie; another player's land doesn't.
#[test]
fn curse_of_the_restless_dead_feeds_on_the_cursed_players_lands() {
    let mut g = pod(3);
    flood(&mut g, 0);
    let curse = g.add_card_to_hand(0, catalog::curse_of_the_restless_dead());
    cast(&mut g, curse, Some(Target::Player(1)), vec![], None).expect("curse");
    g.add_card_to_battlefield(2, catalog::forest());
    drain_stack(&mut g);
    assert!(named(&g, 0, "Zombie").is_empty(), "seat 2 isn't cursed");
    let land = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    drain_stack(&mut g);
    let z = named(&g, 0, "Zombie");
    assert_eq!(z.len(), 1);
    assert!(g.battlefield_find(z[0]).unwrap().definition.keywords.contains(&Keyword::Decayed));
}

/// Dark Salvation: X Zombies for the target player, then the second target
/// shrinks by that player's Zombie count.
#[test]
fn dark_salvation_shrinks_by_the_zombie_count() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::tomb_tyrant());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    flood(&mut g, 0);
    let ds = g.add_card_to_hand(0, catalog::dark_salvation());
    cast(&mut g, ds, Some(Target::Player(0)), vec![Target::Permanent(angel)], Some(2)).expect("cast");
    assert_eq!(named(&g, 0, "Zombie").len(), 2);
    // Three Zombies (the Tyrant and two tokens): the 4/4 Angel is a 1/1.
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Gorex's filtered delve: two creature cards exiled from the graveyard take
/// {4} off, and they're exiled with Gorex — its attack returns one of them.
#[test]
fn gorex_exiles_creature_cards_for_its_cost_and_returns_them() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.add_card_to_graveyard(0, catalog::island());
    let gorex = g.add_card_to_hand(0, catalog::gorex_the_tombshell());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellDelve {
        card_id: gorex,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        delve_cards: vec![a, b],
    })
    .expect("{2}{B}{B} after two creature cards");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gorex).is_some());
    assert!(g.exile.iter().filter(|c| c.exiled_with == Some(gorex)).count() == 2);
    connect(&mut g, gorex, 1);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.id == a || c.id == b).count(), 1, "one came back");
}

/// A land isn't a creature card, so Gorex can't exile it for the discount.
#[test]
fn gorex_rejects_a_noncreature_card_for_its_discount() {
    let mut g = pod(2);
    let land = g.add_card_to_graveyard(0, catalog::island());
    let gorex = g.add_card_to_hand(0, catalog::gorex_the_tombshell());
    flood(&mut g, 0);
    let res = g.perform_action(GameAction::CastSpellDelve {
        card_id: gorex,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        delve_cards: vec![land],
    });
    assert!(res.is_err());
}

/// CR 614 — Undead Alchemist: a Zombie's combat damage to a player is a mill
/// instead (no life lost); an opponent's milled creature card is exiled for a
/// Zombie token.
#[test]
fn undead_alchemist_turns_zombie_damage_into_mill() {
    let mut g = pod(2);
    let alchemist = g.add_card_to_battlefield(0, catalog::undead_alchemist());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    connect(&mut g, alchemist, 1);
    assert_eq!(g.players[1].life, life, "no damage dealt");
    assert_eq!(g.players[1].graveyard.len(), 3, "four milled, the Bears exiled");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(named(&g, 0, "Zombie").len(), 1);
}

/// Prowling Geistcatcher exiles a sacrificed nontoken creature, grows off a
/// token, and returns what it holds when it leaves.
#[test]
fn prowling_geistcatcher_holds_the_sacrificed() {
    let mut g = pod(2);
    let gc = g.add_card_to_battlefield(0, catalog::prowling_geistcatcher());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let seer = g.add_card_to_battlefield(0, catalog::viscera_seer());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let sac = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: seer,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("sac");
        drain_stack(g);
    };
    sac(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_with == Some(gc)), "the Bears are held");
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast(&mut g, murder, Some(Target::Permanent(gc)), vec![], None).expect("murder");
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some_and(|c| c.controller == 0), "released onto the battlefield");
}

/// Ghouls' Night Out takes a creature card from each graveyard as a black
/// Zombie with decayed.
#[test]
fn ghouls_night_out_raids_every_graveyard() {
    let mut g = pod(3);
    let a = g.add_card_to_graveyard(1, catalog::serra_angel());
    let b = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    flood(&mut g, 0);
    let gno = g.add_card_to_hand(0, catalog::ghouls_night_out());
    cast(&mut g, gno, None, vec![], None).expect("cast");
    for id in [a, b] {
        let cp = g.computed_permanent(id).expect("on the battlefield");
        assert_eq!(g.battlefield_find(id).unwrap().controller, 0);
        assert!(cp.subtypes().creature_types.contains(&CreatureType::Zombie));
        assert!(cp.colors.contains(Color::Black));
        assert!(cp.keywords().contains(&Keyword::Decayed));
    }
}

/// Hour of Eternity: each exiled creature card becomes a 4/4 black Zombie
/// token copy.
#[test]
fn hour_of_eternity_mints_4_4_zombie_copies() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    flood(&mut g, 0);
    let hour = g.add_card_to_hand(0, catalog::hour_of_eternity());
    cast(&mut g, hour, Some(Target::Permanent(angel)), vec![], Some(1)).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == angel));
    let copies = named(&g, 0, "Serra Angel");
    assert_eq!(copies.len(), 1);
    let cp = g.computed_permanent(copies[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Zombie));
    assert!(cp.keywords().contains(&Keyword::Flying), "still a copy");
}

/// Ruthless Deathfang: sacrificing a creature makes the target opponent
/// sacrifice one.
#[test]
fn ruthless_deathfang_answers_each_sacrifice() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::ruthless_deathfang());
    let seer = g.add_card_to_battlefield(0, catalog::viscera_seer());
    let elves = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sac the Elves");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elves).is_none());
    assert!(g.battlefield_find(bear).is_none(), "the opponent answered it");
}

/// Shadow Kin mills each player three and becomes a copy of a milled creature
/// card, keeping its upkeep ability.
#[test]
fn shadow_kin_becomes_a_milled_creature() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let kin = g.add_card_to_battlefield(0, catalog::shadow_kin());
    for seat in 0..2 {
        for _ in 0..2 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.add_card_to_library(1, catalog::serra_angel());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let perm = g.battlefield_find(kin).unwrap();
    assert_eq!(perm.definition.name, "Serra Angel");
    assert_eq!(perm.definition.triggered_abilities.len(), 1, "kept the upkeep ability");
    assert!(g.exile.iter().any(|c| c.definition.name == "Serra Angel"));
}

/// Tomb Tyrant's return needs three Zombie creature cards in the graveyard.
#[test]
fn tomb_tyrant_needs_three_zombies_in_the_graveyard() {
    let mut g = pod(2);
    let tyrant = g.add_card_to_battlefield(0, catalog::tomb_tyrant());
    g.clear_sickness(tyrant);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let act = GameAction::ActivateAbility {
        card_id: tyrant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::cleaver_skaab());
    }
    assert!(!g.would_accept(act.clone()), "two Zombies");
    g.add_card_to_graveyard(0, catalog::undead_alchemist());
    g.perform_action(act).expect("three Zombies");
    drain_stack(&mut g);
    let back = g.battlefield.iter().filter(|c| c.controller == 0 && ["Cleaver Skaab", "Undead Alchemist"].contains(&c.definition.name)).count();
    assert_eq!(back, 1);
    let tyr = g.battlefield_find(tyrant).unwrap();
    assert_eq!(tyr.counter_count(CounterType::PlusOnePlusOne), 0);
}

