//! Fallout (PIP) — the rad-counter cards (`catalog::sets::pip`).
//!
//! The CR 728.2 turn-based action itself (mill, the nonland-only life loss,
//! the counter that survives a land) is covered in `core_rules/counters`; these
//! assert only what is unique to each card — who gets how many counters.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn kill(g: &mut GameState, id: CardId) {
    let mut evs = Vec::new();
    g.destroy_permanent(id, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Contaminated Drink — "Draw X cards, then you get half X rad counters,
/// rounded up." X = 3 is two counters, not one: the rounding is up.
#[test]
fn contaminated_drink_rads_the_caster_for_half_x_rounded_up() {
    let mut g = main_phase();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let drink = g.add_card_to_hand(0, catalog::contaminated_drink());
    let hand_before = g.players[0].hand.len();
    cast_x(&mut g, 0, drink, None, Some(3));
    // -1 for the spell that left the hand, +3 drawn.
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew X");
    assert_eq!(g.players[0].rad_counters, 2, "half of 3, rounded up");
    assert_eq!(g.players[1].rad_counters, 0, "the caster takes them, nobody else");
}

/// Contaminated Drink at X = 0 draws nothing and rads nobody — the halving
/// must not round a zero up to one.
#[test]
fn contaminated_drink_at_x_zero_rads_nobody() {
    let mut g = main_phase();
    let drink = g.add_card_to_hand(0, catalog::contaminated_drink());
    cast_x(&mut g, 0, drink, None, Some(0));
    assert_eq!(g.players[0].rad_counters, 0, "half of 0 is 0");
}

/// Glowing One connects for four rad counters on the player it damaged.
#[test]
fn glowing_one_connects_for_four_rad_counters() {
    let mut g = main_phase();
    let one = g.add_card_to_battlefield(0, catalog::glowing_one());
    g.clear_sickness(one);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: one, target: AttackTarget::Player(1) }])
        .expect("attack");
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].rad_counters, 4, "the damaged player, four counters");
    assert_eq!(g.players[0].rad_counters, 0, "not its controller");
}

/// Glowing One's second half drains 1 life out of *any* player's nonland mill,
/// and nothing out of a land's.
#[test]
fn glowing_one_gains_life_on_a_nonland_mill_only() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::glowing_one());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::forest());
    }
    let life = g.players[0].life;
    let scour = g.add_card_to_hand(0, catalog::tome_scour());
    cast_x(&mut g, 0, scour, Some(Target::Player(1)), None);
    assert_eq!(g.players[0].life, life, "five lands milled, no life gained");

    for _ in 0..5 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let scour2 = g.add_card_to_hand(0, catalog::tome_scour());
    cast_x(&mut g, 0, scour2, Some(Target::Player(1)), None);
    assert_eq!(g.players[0].life, life + 5, "one life per nonland milled, any player's");
}

/// Feral Ghoul grows on your *other* creatures dying, then hands each opponent
/// rad counters equal to the power it had as it left — counters included
/// (CR 603.10e last known information).
#[test]
fn feral_ghoul_rads_each_opponent_for_its_last_known_power() {
    let mut g = main_phase();
    let ghoul = g.add_card_to_battlefield(0, catalog::feral_ghoul());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A friendly death grows it: 2/2 → 3/3.
    kill(&mut g, friend);
    assert_eq!(
        g.battlefield_find(ghoul).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "another creature you control dying grows it"
    );
    assert_eq!(g.computed_permanent(ghoul).unwrap().power, 3);

    kill(&mut g, ghoul);
    assert_eq!(g.players[1].rad_counters, 3, "its power as it died, counter included");
    assert_eq!(g.players[0].rad_counters, 0, "each opponent, not you");
}

/// Its own death does not trigger the grow half — "another creature".
#[test]
fn feral_ghoul_does_not_grow_off_itself() {
    let mut g = main_phase();
    let ghoul = g.add_card_to_battlefield(0, catalog::feral_ghoul());
    kill(&mut g, ghoul);
    assert_eq!(g.players[1].rad_counters, 2, "printed 2 power, no self-grow");
}
