//! Comprehensive-Rules conformance for behaviours touched this batch:
//! CR 704.5n (Equipment unattaches from an illegal permanent, stays in play),
//! CR 706.5 / 706.2b (die-roll doubles clause + low-result reroll), CR 706.6
//! (ignored rolls never happened), CR 116.2j (revealing a hidden agenda is a
//! special action), and CR 611.2 (turn-gated statics).

use crabomination::card::{CardDefinition, CardType, Keyword, SelectionRequirement as R};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, StaticEffect, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::{two_player_game, GameState, Target};

/// CR 704.5n — an Equipment attached to a permanent that is no longer a legal
/// creature (its host died) becomes unattached but stays on the battlefield.
#[test]
fn cr_704_5n_equipment_unattaches_when_host_dies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, crabomination::catalog::bonesplitter()); // +2/+0
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(bear);
    // +2/+0 applies while attached.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "equipped bear is 4/2");
    // Destroy the host; the SBA sweep runs at the end of the destroy funnel.
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "host died");
    let axe_c = g.battlefield_find(axe).expect("Equipment stays on the battlefield");
    assert_eq!(axe_c.attached_to, None, "Equipment becomes unattached (CR 704.5n)");
}

/// CR 706.5 / 706.2b — rolling doubles fires the `on_doubles` rider once, and a
/// natural result at or below `reroll_at_most` is rerolled exactly once.
#[test]
fn cr_706_die_doubles_and_reroll() {
    // Two d6; scripted faces 4 then 4 → doubles → gain 5 life.
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(4),
        DecisionAnswer::DieRoll(4),
    ]));
    let start = g.players[0].life;
    let roll = Effect::RollDie {
        sides: 6,
        count: Value::Const(2),
        modifier: Value::ZERO,
        reroll_at_most: 0,
        results: vec![],
        on_doubles: Some(Box::new(Effect::GainLife {
            who: Selector::Player(PlayerRef::You),
            amount: Value::Const(5),
        })),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&roll, &ctx).unwrap();
    assert_eq!(g.players[0].life, start + 5, "doubles rider fired once (CR 706.5)");

    // reroll_at_most: a natural 1 is rerolled once to a 6, which hits the "6+"
    // arm and gains 3 life (CR 706.2b). Faces scripted 1 (rerolled) → 6.
    let mut g2 = two_player_game();
    g2.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(1),
        DecisionAnswer::DieRoll(6),
    ]));
    let start2 = g2.players[0].life;
    let roll2 = Effect::RollDie {
        sides: 6,
        count: Value::Const(1),
        modifier: Value::ZERO,
        reroll_at_most: 1,
        results: vec![(6, 6, Effect::GainLife {
            who: Selector::Player(PlayerRef::You),
            amount: Value::Const(3),
        })],
        on_doubles: None,
    };
    let ctx2 = EffectContext::for_spell(0, None, 0, 0);
    g2.resolve_effect(&roll2, &ctx2).unwrap();
    assert_eq!(g2.players[0].life, start2 + 3, "low roll rerolled into the 6 arm (CR 706.2b)");
}

/// CR 611.2 — a `WhileYourTurn` static grants its effect only during the source
/// controller's turn, even for a plain (non-live-filter) anthem.
#[test]
fn cr_611_2_while_your_turn_gates_anthem() {
    let mut g = two_player_game();
    // A permanent whose static grants trample to your creatures only on your turn.
    let lord = CardDefinition {
        name: "Turn Lord (test)",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![crabomination::card::StaticAbility {
            description: "During your turn, creatures you control have trample.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Trample,
                }),
            },
        }],
        ..Default::default()
    };
    let _ = g.add_card_to_battlefield(0, lord);
    let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let has_trample = |g: &GameState| g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample);
    g.active_player_idx = 0;
    assert!(has_trample(&g), "granted on the controller's turn");
    g.active_player_idx = 1;
    assert!(!has_trample(&g), "not granted on the opponent's turn (CR 611.2)");
}

/// CR 706.6 — Pixie Guide rolls an extra die and ignores the lowest result:
/// the ignored roll never happened, so its results-table arm never fires.
#[test]
fn cr_706_6_pixie_guide_ignores_the_lowest_roll() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, crabomination::catalog::pixie_guide());
    // One die requested → two rolled; the 1 is dropped, the 6 survives.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(1),
        DecisionAnswer::DieRoll(6),
    ]));
    let start = g.players[0].life;
    let roll = Effect::RollDie {
        sides: 6,
        count: Value::ONE,
        modifier: Value::ZERO,
        reroll_at_most: 0,
        results: vec![
            (1, 1, Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(5) }),
            (6, 6, Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(3) }),
        ],
        on_doubles: None,
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&roll, &ctx).unwrap();
    assert_eq!(g.players[0].life, start + 3, "only the surviving roll resolved an arm");
}

/// Without the static, both faces would be single rolls — the guard against
/// the test above passing for the wrong reason.
#[test]
fn cr_706_6_without_the_static_the_low_roll_resolves() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(1)]));
    let start = g.players[0].life;
    let roll = Effect::RollDie {
        sides: 6,
        count: Value::ONE,
        modifier: Value::ZERO,
        reroll_at_most: 0,
        results: vec![(
            1,
            1,
            Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(5) },
        )],
        on_doubles: None,
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&roll, &ctx).unwrap();
    assert_eq!(g.players[0].life, start - 5);
}

/// CR 116.2j — turning a face-down conspiracy face up is a special action the
/// player holding priority takes; it doesn't use the stack.
#[test]
fn cr_116_2j_reveal_conspiracy_is_a_special_action() {
    use crabomination::game::types::GameAction;
    let mut g = two_player_game();
    let id = g.seat_conspiracy(0, crabomination::catalog::unexpected_potential(), Some("Ornithopter"));
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::RevealConspiracy { card_id: id }).is_err(),
        "another seat can't reveal it",
    );
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::RevealConspiracy { card_id: id }).expect("reveal");
    assert!(g.stack.is_empty(), "special actions don't use the stack");
    assert!(!g.players[0].command[0].face_down);
}
