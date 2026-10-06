//! CR 603.2 — an ability triggers the moment its event happens. A listener
//! that the same resolution removes later still saw the draws, discards,
//! life loss and counters that happened while it was on the battlefield.

use crabomination::card::{CounterType, SelectionRequirement as R};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::TurnStep;
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

/// `first`, then destroy every enchantment and creature — the listener with it.
fn then_sweep(g: &mut GameState, source: crabomination::card::CardId, first: Effect) {
    let seq = Effect::Seq(vec![
        first,
        Effect::ForEach {
            selector: Selector::EachPermanent(R::Enchantment.or(R::Creature)),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
    ]);
    let ctx = EffectContext::for_ability(source, 0, None);
    let events = g.resolve_effect(&seq, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
    assert!(g.battlefield_find(source).is_none(), "the sweep took the listener");
}

/// Underworld Dreams saw the opponent's two draws before the sweep.
#[test]
fn cr_603_2_a_departed_draw_listener_saw_the_draws() {
    let mut g = main_phase();
    let dreams = g.add_card_to_battlefield(0, catalog::underworld_dreams());
    then_sweep(&mut g, dreams, Effect::Draw { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::Const(2) });
    assert_eq!(g.players[1].life, 18, "one damage per card drawn");
}

/// Megrim saw the opponent's discard before the sweep.
#[test]
fn cr_603_2_a_departed_discard_listener_saw_the_discard() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::forest());
    let megrim = g.add_card_to_battlefield(0, catalog::megrim());
    then_sweep(
        &mut g,
        megrim,
        Effect::Discard { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::Const(1), random: true },
    );
    assert_eq!(g.players[1].life, 18);
}

/// Exquisite Blood saw the opponent lose 3 before the sweep.
#[test]
fn cr_603_2_a_departed_life_loss_listener_saw_the_loss() {
    let mut g = main_phase();
    let blood = g.add_card_to_battlefield(0, catalog::exquisite_blood());
    then_sweep(&mut g, blood, Effect::LoseLife { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::Const(3) });
    assert_eq!((g.players[0].life, g.players[1].life), (23, 17));
}

/// Shalai and Hallar saw two +1/+1 counters land on a creature of its
/// controller's before both died in the sweep.
#[test]
fn cr_603_2_a_departed_counter_listener_saw_the_counters() {
    let mut g = main_phase();
    let shalai = g.add_card_to_battlefield(0, catalog::shalai_and_hallar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    then_sweep(
        &mut g,
        shalai,
        Effect::AddCounter {
            what: Selector::ExactObjects(vec![bear]),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
    );
    assert_eq!(g.players[1].life, 18, "two damage to the only opponent");
}


/// And not what happened after it left: the sweep first, then the draws.
#[test]
fn cr_603_2_a_listener_gone_before_the_draws_did_not_see_them() {
    let mut g = main_phase();
    let dreams = g.add_card_to_battlefield(0, catalog::underworld_dreams());
    let seq = Effect::Seq(vec![
        Effect::Destroy { what: Selector::ExactObjects(vec![dreams]) },
        Effect::Draw { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::Const(2) },
    ]);
    let events = g.resolve_effect(&seq, &EffectContext::for_ability(dreams, 0, None)).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20);
}
