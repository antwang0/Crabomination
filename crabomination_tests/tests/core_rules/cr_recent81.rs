//! CR conformance for this run:
//! - CR 603.2 — a "whenever you attack …" trigger's condition is read at fire
//!   time, not ignored.
//! - CR 603.4 — a self-source ETB trigger's intervening 'if' gates the fire.
//! - CR 709.5c — a Room's unlocked-door designations drive its live abilities,
//!   and a re-locked door goes inert.
//! - CR 613.4 — a layer-7a CDA sees the layer-4 land types granted this turn.

use crabomination::card::{CounterType, LandType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn game() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn unlock(g: &mut GameState, id: CardId, right: bool) {
    flood(g, 0);
    g.perform_action(GameAction::UnlockRoomDoor { card_id: id, right }).expect("unlock");
    drain_stack(g);
}

fn attack_with(g: &mut GameState, id: CardId) {
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// CR 603.2 — Military Intelligence needs two attackers; one doesn't fire it.
#[test]
fn cr_603_2_you_attack_condition_gates_the_trigger() {
    let mut g = game();
    g.add_card_to_battlefield(0, catalog::military_intelligence());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let lone = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    attack_with(&mut g, lone);
    assert_eq!(g.players[0].hand.len(), before, "one attacker doesn't meet the condition");

    let mut g = game();
    g.add_card_to_battlefield(0, catalog::military_intelligence());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    let before = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack with two");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "two attackers draw a card");
}

/// CR 603.4 — the Grimoire's "if you cast it" ETB doesn't fire off a
/// battlefield drop.
#[test]
fn cr_603_4_self_etb_intervening_if_gates_the_fire() {
    let mut g = game();
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_battlefield(0, catalog::marina_vendrells_grimoire());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "no cast, no draw-5");

    let mut g = game();
    for _ in 0..12 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::marina_vendrells_grimoire());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 5, "cast → draw five");
}

/// CR 709.5c — an unlocked door's static is live; re-locking it turns it off.
#[test]
fn cr_709_5c_relocking_a_door_drops_its_static() {
    let mut g = game();
    let room = g.add_card_to_battlefield(0, catalog::dazzling_theater_prop_room());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    unlock(&mut g, room, true);
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(!g.battlefield_find(bear).unwrap().tapped, "Prop Room untaps off-turn");

    g.relock_room_door(room, true);
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "the re-locked door is inert");
}

/// CR 613.4 — a `*/*` land-count CDA reads the types granted in layer 4.
#[test]
fn cr_613_4_cda_reads_granted_land_types() {
    let mut g = game();
    g.add_card_to_battlefield(0, catalog::island());
    let eluge = g.add_card_to_battlefield(0, catalog::eluge_the_shoreless_sea());
    assert_eq!(g.computed_permanent(eluge).unwrap().power, 1, "one printed Island");
    let waste = g.add_card_to_battlefield(0, catalog::wastes());
    g.battlefield_find_mut(waste).unwrap().add_counters(CounterType::Flood, 1);
    assert!(
        g.computed_permanent(waste).unwrap().subtypes.land_types.contains(&LandType::Island),
        "layer 4 granted the type"
    );
    assert_eq!(g.computed_permanent(eluge).unwrap().power, 2, "layer 7a counts it");
}
