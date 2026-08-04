//! Planechase (CR 311 / 312 / 901) — `catalog::sets::ohop`.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, PlanarFace, Target};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..30 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

/// A planar deck for seat 0 with `defs` in order, with the first plane already
/// face up as the starting plane (CR 901.5).
fn planar(g: &mut GameState, defs: Vec<crabomination::card::CardDefinition>) {
    g.seat_planar_deck(0, defs);
    g.set_starting_plane(0);
}

fn roll(g: &mut GameState, face: PlanarFace) -> Result<Vec<GameEvent>, GameError> {
    let die = match face {
        PlanarFace::Planeswalker => 1,
        PlanarFace::Chaos => 2,
        PlanarFace::Blank => 5,
    };
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(die)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::RollPlanarDie)
}

#[test]
fn cr_901_5_the_starting_plane_skips_phenomena() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::mutual_epiphany(), catalog::krosa()]);
    let face_up = g.face_up_planes();
    assert_eq!(face_up.len(), 1);
    assert_eq!(g.players[0].command[0].definition.name, "Krosa");
    // The skipped phenomenon went to the bottom, and no ability triggered.
    assert_eq!(g.players[0].command.len(), 1);
    assert_eq!(g.players[0].hand.len(), 0);
}

#[test]
fn cr_901_7_a_planes_static_functions_from_the_command_zone() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    planar(&mut g, vec![catalog::krosa()]);
    let cp = g.computed_permanent(bear).expect("bear");
    assert_eq!((cp.power, cp.toughness), (4, 4), "Krosa pumps everyone's creatures");
}

#[test]
fn cr_901_9_the_roll_costs_one_more_each_time() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::krosa(), catalog::panopticon()]);
    // The first roll of the turn is free.
    assert!(roll(&mut g, PlanarFace::Blank).is_ok());
    assert_eq!(g.players[0].planar_die_rolls_this_turn, 1);
    // The second costs {1}, which an empty pool can't pay.
    g.players[0].mana_pool = Default::default();
    assert!(roll(&mut g, PlanarFace::Blank).is_err());
    g.players[0].mana_pool.add_colorless(1);
    assert!(roll(&mut g, PlanarFace::Blank).is_ok());
}

#[test]
fn cr_901_9_the_die_is_only_legal_in_your_own_main_phase() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::krosa()]);
    g.step = TurnStep::Upkeep;
    assert!(roll(&mut g, PlanarFace::Blank).is_err());
}

#[test]
fn cr_901_9b_chaos_fires_the_face_up_planes_chaos_trigger() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::panopticon()]);
    let before = g.players[0].hand.len();
    roll(&mut g, PlanarFace::Chaos).expect("roll");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

#[test]
fn cr_901_9c_the_planeswalker_symbol_turns_the_next_plane_up() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::krosa(), catalog::panopticon()]);
    roll(&mut g, PlanarFace::Planeswalker).expect("roll");
    drain_stack(&mut g);
    assert_eq!(g.players[0].command[0].definition.name, "Panopticon");
    // Panopticon's "when you planeswalk to this" drew a card.
    assert_eq!(g.players[0].hand.len(), 1);
    // Krosa went to the bottom of the planar deck.
    assert_eq!(g.players[0].planar_deck.last().expect("bottom").definition.name, "Krosa");
}

#[test]
fn cr_311_planeswalking_away_fires_the_leaving_planes_trigger() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    planar(&mut g, vec![catalog::sanctum_of_serra(), catalog::krosa()]);
    roll(&mut g, PlanarFace::Planeswalker).expect("roll");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Sanctum's farewell wipes the board");
}

#[test]
fn cr_312_a_phenomenon_resolves_then_planeswalks_away() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    planar(
        &mut g,
        vec![catalog::krosa(), catalog::planewide_disaster(), catalog::panopticon()],
    );
    roll(&mut g, PlanarFace::Planeswalker).expect("roll");
    drain_stack(&mut g);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield.is_empty(), "the phenomenon swept the board");
    // CR 704.6f — its trigger left the stack, so we planeswalked on to the plane.
    assert_eq!(g.players[0].command[0].definition.name, "Panopticon");
}

#[test]
fn cr_901_7_a_planes_step_trigger_fires_from_the_command_zone() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::lethe_lake()]);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 10);
}

#[test]
fn the_eon_fog_locks_untap_steps() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).expect("bear").tapped = true;
    planar(&mut g, vec![catalog::the_eon_fog()]);
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped);
    roll(&mut g, PlanarFace::Chaos).expect("roll");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).expect("bear").tapped, "chaos untaps your board");
}

#[test]
fn goldmeadow_pays_three_goats_per_land() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::goldmeadow()]);
    let land = g.add_card_to_hand(1, catalog::mountain());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goat").count(), 3);
}

#[test]
fn the_fourth_sphere_eats_a_nonblack_creature_each_upkeep() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    planar(&mut g, vec![catalog::the_fourth_sphere()]);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn academy_at_tolaria_west_refills_an_empty_hand() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::academy_at_tolaria_west()]);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7);
    // With cards in hand, the intervening 'if' keeps it quiet.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7);
}

#[test]
fn naar_isle_burns_hotter_every_upkeep() {
    let mut g = main_phase();
    planar(&mut g, vec![catalog::naar_isle()]);
    for _ in 0..2 {
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, 20 - 1 - 2);
}

#[test]
fn the_hippodrome_shrinks_everything_and_chaos_finishes_it() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    planar(&mut g, vec![catalog::the_hippodrome()]);
    assert_eq!(g.computed_permanent(bear).expect("bear").power, -3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DieRoll(2),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::RollPlanarDie).expect("roll");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}
