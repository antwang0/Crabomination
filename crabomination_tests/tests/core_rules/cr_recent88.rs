//! CR conformance for this run:
//! - CR 707.4 — a permanent that becomes a copy while on the battlefield
//!   keeps the noncopy effects on it and doesn't re-fire ETB triggers.
//! - CR 116.2b/116.3 — turning a face-down creature face up is a special
//!   action: legal with a spell on the stack, and the actor keeps priority.
//! - CR 717.2/717.4/717.5 — the Attraction deck sits in the command zone
//!   outside the library, and the precombat-main roll visits only the
//!   Attractions whose lit numbers match.

use crabomination::card::{CardId, CardInstance};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Duration, Effect, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// A permanent that becomes a copy on the battlefield keeps the pump already
/// on it, and the copied card's ETB trigger doesn't fire.
#[test]
fn cr_707_4_becoming_a_copy_keeps_noncopy_effects_and_skips_etb() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let ctx = EffectContext::for_ability(bears, 0, None);
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(3),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("pump");

    let omens = g.add_card_to_battlefield(1, catalog::wall_of_omens()); // 0/4, ETB draw
    g.resolve_effect(
        &Effect::BecomeCopyOfFor {
            what: Selector::This,
            source: Selector::Target(0),
            duration: Duration::EndOfTurn,
            non_legendary: false,
        },
        &EffectContext { targets: vec![Target::Permanent(omens)], ..ctx.clone() },
    )
    .expect("become a copy");
    drain_stack(&mut g);

    let cp = g.computed_permanent(bears).expect("still on the battlefield");
    assert_eq!((cp.power, cp.toughness), (3, 7), "0/4 base, +3/+3 still applied");
    assert_eq!(g.players[0].hand.len(), hand, "the copied ETB never triggered");
}

/// Turning a face-down creature face up is a special action, so it works with
/// a spell on the stack — and the actor still has priority afterwards.
#[test]
fn cr_116_2b_turn_face_up_is_a_special_action_at_instant_speed() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::exalted_angel());
    g.battlefield_find_mut(angel).unwrap().turn_face_down();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    assert!(!g.stack.is_empty(), "the Bolt is waiting");

    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::TurnFaceUp { card_id: angel }).expect("special action");
    assert!(!g.battlefield_find(angel).unwrap().face_down);
    // CR 116.3 — a special action doesn't pass priority.
    assert_eq!(g.priority.player_with_priority, 0);
    assert!(!g.stack.is_empty(), "and it didn't use the stack");
}

/// The Attraction deck lives in the command zone, and the precombat-main roll
/// fires only the visit abilities whose lit numbers match the die.
#[test]
fn cr_717_2_and_717_5_only_lit_attractions_are_visited() {
    let mut g = two_player_game();
    let library = g.players[0].library.len();
    for def in [catalog::information_booth(), catalog::kiddie_coaster()] {
        let id = g.next_id();
        g.players[0].attraction_deck.push(CardInstance::new(id, def, 0));
    }
    assert_eq!(g.players[0].library.len(), library, "the Attraction deck isn't the library");
    assert_eq!(g.players[0].attraction_deck.len(), 2);

    // Both on the battlefield: Information Booth lights {2,6}, Kiddie Coaster
    // {2,3,6}. A rolled 3 visits only the Coaster.
    g.add_card_to_battlefield(0, catalog::information_booth());
    g.add_card_to_battlefield(0, catalog::kiddie_coaster());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for i in 0..40u32 {
        let id = CardId(9000 + i);
        for p in 0..2 {
            g.players[p].library.push(CardInstance::new(id, catalog::forest(), p));
        }
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3)]));
    let hand = g.players[0].hand.len();
    while g.step == TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    while !(g.step == TurnStep::PreCombatMain && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);

    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "the Coaster's +1/+0 landed");
    assert_eq!(
        g.players[0].hand.len(),
        hand + 1,
        "only the draw step — the Booth isn't lit on a 3"
    );
}

/// A visit ability is a normal triggered ability — its pump ends at cleanup.
#[test]
fn cr_717_5_a_visit_pump_wears_off_at_end_of_turn() {
    let mut g = two_player_game();
    let coaster = g.add_card_to_battlefield(0, catalog::kiddie_coaster());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_ability(coaster, 0, None);
    let def = catalog::kiddie_coaster();
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("visit");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    g.do_cleanup(&mut vec![]);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
}
