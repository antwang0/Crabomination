//! Commander: the Power Hungry precon (C13, Prossh, `decks::cmdr_prossh`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: x,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

/// Prossh: a Kobold per mana spent to cast it (six from the hand).
#[test]
fn prossh_makes_a_kobold_per_mana_spent() {
    let mut g = main_phase(2);
    let p = g.add_card_to_hand(0, catalog::prossh_skyraider_of_kher());
    cast_x(&mut g, p, &[], None).expect("cast");
    assert_eq!(named(&g, 0, "Kobolds of Kher Keep"), 6);
    let kobold = g.battlefield.iter().find(|c| c.definition.name == "Kobolds of Kher Keep").unwrap().id;
    activate(&mut g, p, 0, &[], None).expect("sacrifice");
    assert!(g.battlefield_find(kobold).is_none() || named(&g, 0, "Kobolds of Kher Keep") == 5);
    assert_eq!(g.computed_permanent(p).unwrap().power, 6);
}

/// Primal Vigor doubles an opponent's tokens and the +1/+1 counters put on
/// any creature (CR 614.13 / 614.16 — for every player, not only its own).
#[test]
fn primal_vigor_doubles_for_everyone() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::primal_vigor());
    let thrinax = g.add_card_to_battlefield(2, catalog::sprouting_thrinax());
    let mut evs = Vec::new();
    g.destroy_permanent(thrinax, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named(&g, 2, "Saproling"), 6, "an opponent's three Saprolings doubled");
    let their = g.add_card_to_battlefield(1, catalog::scarland_thrinax());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(their);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: their,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(their).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Brooding Saurian returns stolen nontoken permanents at each end step.
#[test]
fn brooding_saurian_sends_permanents_home() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::brooding_saurian());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    fire(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1);
}

/// Endrek Sahr: Thrulls equal to the creature spell's mana value; at seven
/// Thrulls it is sacrificed (a state trigger).
#[test]
fn endrek_sahr_breeds_thrulls_then_goes() {
    let mut g = main_phase(2);
    let endrek = g.add_card_to_battlefield(0, catalog::endrek_sahr_master_breeder());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    cast_x(&mut g, giant, &[], None).expect("cast");
    assert_eq!(named(&g, 0, "Thrull"), 4);
    assert!(g.battlefield_find(endrek).is_some());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    cast_x(&mut g, wurm, &[], None).expect("cast");
    assert!(g.battlefield_find(endrek).is_none(), "ten Thrulls: sacrificed");
}

/// Sudden Demise picks the color that hurts the opponents most.
#[test]
fn sudden_demise_hits_the_best_color() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let white = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let spell = g.add_card_to_hand(0, catalog::sudden_demise());
    cast_x(&mut g, spell, &[], Some(2)).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "green chosen");
    assert!(g.battlefield_find(mine).is_none(), "the caster's green Elf too");
    assert!(g.battlefield_find(white).is_some());
}

/// Shattergang Brothers: every other player sacrifices a creature.
#[test]
fn shattergang_edicts_each_other_player() {
    let mut g = main_phase(3);
    let sg = g.add_card_to_battlefield(0, catalog::shattergang_brothers());
    let fodder = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let _ = fodder;
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    activate(&mut g, sg, 0, &[], None).expect("edict");
    assert_eq!(named(&g, 1, "Grizzly Bears") + named(&g, 2, "Grizzly Bears"), 0);
    assert!(g.battlefield_find(sg).is_some(), "the Brothers sacrificed the Elves, not themselves");
}

/// Capricious Efreet destroys exactly one of its (up to three) targets.
#[test]
fn capricious_efreet_destroys_one_at_random() {
    let mut g = main_phase(2);
    let efreet = g.add_card_to_battlefield(0, catalog::capricious_efreet());
    let mine = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    fire(&mut g, TurnStep::Upkeep);
    let gone = [efreet, mine, a, b].iter().filter(|id| g.battlefield_find(**id).is_none()).count();
    assert_eq!(gone, 1);
}

/// Widespread Panic: a searching spell's shuffle puts a card from hand on top.
#[test]
fn widespread_panic_taxes_a_shuffle() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::widespread_panic());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let keep = g.add_card_to_hand(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::spoils_of_victory());
    cast_x(&mut g, spell, &[], None).expect("cast");
    assert!(!g.players[0].hand.iter().any(|c| c.id == keep), "the Bears went on top");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(keep));
}

/// Walker of the Grove, evoked, still leaves a 4/4 behind.
#[test]
fn walker_of_the_grove_leaves_an_elemental() {
    let mut g = main_phase(2);
    let w = g.add_card_to_hand(0, catalog::walker_of_the_grove());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: w,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(w).is_none());
    assert_eq!(named(&g, 0, "Elemental"), 1);
}

/// Deepfire Elemental destroys an artifact or creature whose mana value is X.
#[test]
fn deepfire_elemental_reads_x() {
    let mut g = main_phase(2);
    let d = g.add_card_to_battlefield(0, catalog::deepfire_elemental());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(activate(&mut g, d, 0, &[Target::Permanent(bear)], Some(3)).is_err() || g.battlefield_find(bear).is_some());
    activate(&mut g, d, 0, &[Target::Permanent(bear)], Some(2)).expect("X = 2");
    assert!(g.battlefield_find(bear).is_none());
}
