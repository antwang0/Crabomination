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

/// CR 104.4b — Heliod and a lifelinking Shalai against Pariah on an
/// indestructible creature: every cycle gains life, puts a counter on Shalai
/// and deals its damage elsewhere, all mandatory, and no board repeats. A
/// counter is growth to the loop watch, so the game is a draw rather than an
/// action cap (four-seat audit pod seed 196001).
#[test]
fn cr_104_4b_a_counter_growing_mandatory_loop_is_a_draw() {
    use crabomination::game::types::{GameAction, Target};
    let mut g = main_phase();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_sun_crowned());
    let shalai = g.add_card_to_battlefield(0, catalog::shalai_and_hallar());
    let myr = g.add_card_to_battlefield(1, catalog::darksteel_myr());
    let pariah = g.add_card_to_battlefield(1, catalog::pariah());
    g.battlefield_find_mut(pariah).unwrap().attached_to = Some(myr);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: heliod,
        ability_index: 0,
        target: Some(Target::Permanent(shalai)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("lifelink");
    drain_stack(&mut g);
    let events = g
        .resolve_effect(
            &Effect::AddCounter {
                what: Selector::ExactObjects(vec![shalai]),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            &EffectContext::for_ability(shalai, 0, None),
        )
        .expect("resolves");
    g.dispatch_triggers_for_events(&events);
    for _ in 0..20_000 {
        if g.is_game_over() || g.stack.is_empty() {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(g.game_over, Some(None), "a mandatory loop is a draw");
    assert_eq!(g.players[1].life, 20, "Pariah took every point");
}

/// Vito and Exquisite Blood against `opp_life` with `angel` on the other side.
fn vito_blood_drain(opp_life: i32, angel: bool) -> GameState {
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::vito_thorn_of_the_dusk_rose());
    let blood = g.add_card_to_battlefield(0, catalog::exquisite_blood());
    if angel {
        g.add_card_to_battlefield(1, catalog::platinum_angel());
    }
    g.players[1].life = opp_life;
    let events = g
        .resolve_effect(
            &Effect::LoseLife { who: Selector::Player(PlayerRef::Seat(1)), amount: Value::Const(1) },
            &EffectContext::for_ability(blood, 0, None),
        )
        .expect("resolves");
    g.dispatch_triggers_for_events(&events);
    for _ in 0..20_000 {
        if g.is_game_over() || g.stack.is_empty() {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    g
}

/// CR 104.4b — Vito and Exquisite Blood against a seat Platinum Angel keeps
/// in the game: every lap drains it further below 0 and gains the same, all
/// mandatory, so no life total ever repeats. Life lost by a seat still in at
/// 0 or less is growth to the loop watch, and the game is a draw rather than
/// an action cap (pod seed 924354).
#[test]
fn cr_104_4b_a_life_drain_loop_against_a_seat_that_cannot_lose_is_a_draw() {
    let g = vito_blood_drain(0, true);
    assert_eq!(g.game_over, Some(None), "a mandatory loop is a draw");
    assert!(g.players[1].life < -300, "the drain ran laps first");
}

/// The negative control: the same drain against a seat that can lose ends
/// with that seat out, however long it runs above 0 (CR 104.3b).
#[test]
fn cr_104_4b_a_finite_life_drain_is_not_a_loop() {
    let g = vito_blood_drain(1_000, false);
    assert_eq!(g.game_over, Some(Some(0)), "the drain wins");
}

/// CR 704.3 / 704.5n — Grip of Phyresis takes Konda's Banner and attaches it
/// to a 0/0 Germ the Banner may not equip. The sweep unattaches it (the
/// Banner's +1/+1s go with it) and must then repeat, so the Germ dies before
/// anyone gets priority rather than at the next sweep (pod seed 924092).
#[test]
fn cr_704_3_an_illegal_attachment_removed_by_the_sweep_repeats_it() {
    use crabomination::game::types::Target;
    let mut g = main_phase();
    let banner = g.add_card_to_battlefield(1, catalog::kondas_banner());
    let grip = catalog::grip_of_phyresis();
    let src = g.add_card_to_battlefield(0, catalog::forest());
    g.resolve_effect(&grip.effect, &EffectContext::for_ability(src, 0, Some(Target::Permanent(banner))))
        .expect("resolves");
    g.check_state_based_actions();
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Phyrexian Germ"), "the 0/0 Germ died");
    let b = g.battlefield_find(banner).expect("the Banner stays");
    assert_eq!((b.controller, b.attached_to), (0, None));
}
