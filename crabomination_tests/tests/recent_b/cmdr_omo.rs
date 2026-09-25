//! Commander: the Tricky Terrain precon (M3C, Omo, Queen of Vesuva,
//! `decks::cmdr_omo`).

use crabomination::card::{CardId, CounterType, Keyword, LandType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn act_as(g: &mut GameState, seat: usize, action: GameAction) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act_as(g, 0, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn activate_x(g: &mut GameState, id: CardId, index: usize, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act_as(g, 0, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: x,
        mode: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    activate_x(g, id, index, targets, None)
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn land_types(g: &GameState, id: CardId) -> Vec<LandType> {
    g.computed_permanent(id).map(|c| c.subtypes().land_types.to_vec()).unwrap_or_default()
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Omo marks a land and a creature as it enters: the land is every land type
/// (a Plains is a Desert and a Gate too); the creature a changeling.
#[test]
fn omo_marks_everything() {
    let mut g = main_phase(2);
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let omo = g.add_card_to_hand(0, catalog::omo_queen_of_vesuva());
    cast_at(&mut g, omo, &[]).expect("cast");
    let p = g.battlefield_find(plains).unwrap().counter_count(CounterType::Everything);
    let b = g.battlefield_find(bear).unwrap().counter_count(CounterType::Everything);
    assert_eq!((p, b), (1, 1), "one land and one creature");
    let types = land_types(&g, plains);
    for lt in [LandType::Plains, LandType::Forest, LandType::Island, LandType::Desert, LandType::Gate, LandType::Cave] {
        assert!(types.contains(&lt), "{lt:?}");
    }
    assert!(g.computed_permanent(bear).is_some_and(|c| c.keywords().contains(&Keyword::Changeling)));
}

/// CR 701.5 — Summary Dismissal exiles the other spell (to exile, not the
/// graveyard) and counters an activated ability on the stack.
#[test]
fn cr_701_5_summary_dismissal_clears_the_stack() {
    let mut g = main_phase(2);
    // An ability on the stack: Prodigal Pyromancer's ping.
    let pyro = g.add_card_to_battlefield(1, catalog::prodigal_pyromancer());
    g.clear_sickness(pyro);
    let life = g.players[0].life;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pyro,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("ping");
    let spear = g.add_card_to_hand(1, catalog::searing_spear());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spear,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("spear");
    assert_eq!(g.stack.len(), 2);
    let sd = g.add_card_to_hand(0, catalog::summary_dismissal());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: sd, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("dismiss");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "neither the ping nor the spear resolved");
    assert!(g.exile.iter().any(|c| c.id == spear), "the spell is exiled, not countered to the yard");
}

/// Aggressive Biomancy: X token copies, each fighting as it enters.
#[test]
fn aggressive_biomancy_copies_fight() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ab = g.add_card_to_hand(0, catalog::aggressive_biomancy());
    cast_x(&mut g, ab, &[Target::Permanent(giant)], Some(1)).expect("cast");
    assert_eq!(named(&g, 0, "Hill Giant").len(), 2);
    assert!(g.battlefield_find(bear).is_none(), "the token's fight killed it");
}

/// Wonderscape Sage: returning a nonbasic-typed land draws without the
/// discard; a basic land costs the discard — unless an everything counter
/// made it every land type as it last existed (CR 603.10).
#[test]
fn wonderscape_sage_reads_the_returned_land() {
    let run = |land: fn() -> crabomination::card::CardDefinition, everything: bool| {
        let mut g = main_phase(2);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let ws = g.add_card_to_battlefield(0, catalog::wonderscape_sage());
        g.clear_sickness(ws);
        let l = g.add_card_to_battlefield(0, land());
        if everything {
            g.battlefield_find_mut(l).unwrap().add_counters(CounterType::Everything, 1);
            g.add_card_to_battlefield(1, catalog::omo_queen_of_vesuva());
        }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        activate(&mut g, ws, 0, &[]).expect("activate");
        g.players[0].hand.len() - hand
    };
    // Hand grows by the returned land plus the draw, minus any discard.
    assert_eq!(run(catalog::hashep_oasis, false), 2, "a Desert: no discard");
    assert_eq!(run(catalog::plains, false), 1, "a basic: discard");
    assert_eq!(run(catalog::plains, true), 2, "an everything-counter Plains is a Desert too");
}

/// Sunken Palace's mana copies the spell it pays for.
#[test]
fn sunken_palace_copies_the_spell() {
    let mut g = main_phase(2);
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let sp = g.add_card_to_battlefield(0, catalog::sunken_palace());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: sp, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("palace mana");
    assert!(g.players[0].graveyard.is_empty(), "seven exiled");
    g.players[0].mana_pool.add(Color::Red, 1);
    let spear = g.add_card_to_hand(0, catalog::searing_spear());
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: spear, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("spear on U + R");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 6, "the spear and its copy");
}

/// Desert Warfare: a sacrificed Desert returns at your next end step.
#[test]
fn desert_warfare_brings_deserts_back() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::desert_warfare());
    let oasis = g.add_card_to_battlefield(0, catalog::hashep_oasis());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, oasis, 2, &[Target::Permanent(bear)]).expect("sacrifice itself");
    assert!(g.battlefield_find(oasis).is_none());
    assert_eq!(pt(&g, bear), (5, 5));
    for _ in 0..12 {
        if g.step == TurnStep::Cleanup || g.battlefield_find(oasis).is_some() {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(oasis).is_some(), "back at this turn's end step");
}

/// Desert Warfare: five Deserts make five hasty Sand Warriors at combat.
#[test]
fn desert_warfare_raises_an_army() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::desert_warfare());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::desert_of_the_indomitable());
    }
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    let w = named(&g, 0, "Sand Warrior");
    assert_eq!(w.len(), 5);
    assert!(g.computed_permanent(w[0]).is_some_and(|c| c.keywords().contains(&Keyword::Haste)));
}

/// Jyoti makes a Forest Dryad per commander cast and pumps land creatures
/// by its power at each combat.
#[test]
fn jyoti_grows_the_forest() {
    let mut g = main_phase(2);
    g.seat_commanders(0, vec![catalog::omo_queen_of_vesuva()]);
    let omo = g.players[0].commanders[0];
    g.commander_cast_count.insert(omo, 2);
    let j = g.add_card_to_hand(0, catalog::jyoti_moag_ancient());
    cast_at(&mut g, j, &[]).expect("cast");
    let dryads = named(&g, 0, "Forest Dryad");
    assert_eq!(dryads.len(), 2);
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(pt(&g, dryads[0]), (3, 3));
}

/// Copy Land enters as a copy of a land, and is an enchantment too.
#[test]
fn copy_land_copies_a_land() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::hashep_oasis());
    let cl = g.add_card_to_hand(0, catalog::copy_land());
    cast_at(&mut g, cl, &[]).expect("cast");
    let c = g.computed_permanent(cl).expect("on the battlefield");
    assert_eq!(c.def.name, "Hashep Oasis");
    assert!(c.card_types().contains(&crabomination::card::CardType::Land));
    assert!(c.card_types().contains(&crabomination::card::CardType::Enchantment));
}

/// March from Velis Vel: your Deserts become hasty copies of your creature.
#[test]
fn march_from_velis_vel_marches_the_deserts() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let d1 = g.add_card_to_battlefield(0, catalog::desert_of_the_indomitable());
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    let m = g.add_card_to_hand(0, catalog::march_from_velis_vel());
    act_as(&mut g, 0, GameAction::CastSpell {
        card_id: m,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Desert mode");
    assert_eq!(pt(&g, d1), (3, 3));
    assert!(g.computed_permanent(d1).is_some_and(|c| c.keywords().contains(&Keyword::Haste)));
    assert!(g.computed_permanent(plains).is_some_and(|c| c.power == 0));
}

/// Rampant Frogantua gets +10/+10 per player who has lost.
#[test]
fn rampant_frogantua_feeds_on_the_fallen() {
    let mut g = main_phase(4);
    let f = g.add_card_to_battlefield(0, catalog::rampant_frogantua());
    assert_eq!(pt(&g, f), (3, 3));
    g.players[2].life = 0;
    g.check_state_based_actions();
    assert_eq!(pt(&g, f), (13, 13));
}

/// Basilisk Gate pumps by Gates; Sage of the Maze animates by twice them.
#[test]
fn gates_power_the_maze() {
    let mut g = main_phase(2);
    let bg = g.add_card_to_battlefield(0, catalog::basilisk_gate());
    g.add_card_to_battlefield(0, catalog::basilisk_gate());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, bg, 1, &[Target::Permanent(bear)]).expect("pump");
    assert_eq!(pt(&g, bear), (4, 4));
    let sage = g.add_card_to_battlefield(0, catalog::sage_of_the_maze());
    g.clear_sickness(sage);
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    activate(&mut g, sage, 1, &[Target::Permanent(plains)]).expect("animate");
    assert_eq!(pt(&g, plains), (4, 4));
    assert!(g.computed_permanent(plains).is_some_and(|c| c.keywords().contains(&Keyword::Haste)));
}

/// Magus of the Candelabra untaps X lands.
#[test]
fn magus_of_the_candelabra_untaps() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::magus_of_the_candelabra());
    g.clear_sickness(m);
    let lands: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::forest())).collect();
    for &l in &lands {
        g.battlefield_find_mut(l).unwrap().tapped = true;
    }
    activate_x(&mut g, m, 0, &[], Some(2)).expect("untap two");
    assert_eq!(lands.iter().filter(|&&l| !g.battlefield_find(l).unwrap().tapped).count(), 2);
}

/// Ulvenwald Hydra counts your lands and fetches one tapped.
#[test]
fn ulvenwald_hydra_fetches_and_grows() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let h = g.add_card_to_hand(0, catalog::ulvenwald_hydra());
    cast_at(&mut g, h, &[]).expect("cast");
    assert_eq!(pt(&g, h), (4, 4));
    assert!(kw_of(&g, h, Keyword::Reach));
}

fn kw_of(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&k))
}

/// Floriferous Vinewall takes a land from the top six.
#[test]
fn floriferous_vinewall_finds_a_land() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let f = g.add_card_to_library(0, catalog::forest());
    let v = g.add_card_to_hand(0, catalog::floriferous_vinewall());
    cast_at(&mut g, v, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == f));
    assert!(kw_of(&g, v, Keyword::Defender));
}

/// Horizon of Progress puts a land from hand onto the battlefield tapped.
#[test]
fn horizon_of_progress_drops_a_land() {
    let mut g = main_phase(2);
    let h = g.add_card_to_battlefield(0, catalog::horizon_of_progress());
    let f = g.add_card_to_hand(0, catalog::forest());
    activate(&mut g, h, 1, &[]).expect("put a land");
    assert!(g.battlefield_find(f).is_some_and(|c| c.tapped));
}
