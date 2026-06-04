//! Functionality tests for the Strixhaven base set
//! (`catalog::sets::stx`). New STX cards added here should ship with at
//! least one test exercising their primary play pattern.

use crate::card::CounterType;
use crate::catalog;
use crate::game::*;
use crate::game::drain_stack;
use crate::mana::Color;


// Suppress unused-import lint when CounterType isn't used in this batch.
#[allow(dead_code)]
fn _keepalive(_: CounterType) {}

// ── batch 125 — CR 706 (Roll a Die) primitive tests ────────────────────────

/// Test-only fixture: a Sorcery with a d6 results table. 1-2 → gain 1
/// life; 3-6 → opp loses 3 life. Drives the AutoDecider midpoint test.
fn test_card_die_roll_d6_midpoint() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Subtypes};
    use crate::effect::{Effect, Selector, Value};
    use crate::mana::{cost, generic};
    CardDefinition {
        name: "Test Die Roll d6 Midpoint",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RollDie {
            sides: 6,
            count: Value::Const(1),
            modifier: Value::Const(0),
            reroll_at_most: 0,
            on_doubles: None,
            results: vec![
                (1, 2, Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
                (3, 6, Effect::LoseLife {
                    who: Selector::Player(crate::effect::PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
            ],
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
    }
}

/// Test-only fixture: d6 sorcery whose 1-2 arm gains 5 life and 3-6
/// arm deals 3 damage to opp. Drives the ScriptedDecider branch test.
fn test_card_die_roll_d6_big_gain() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Subtypes};
    use crate::effect::{Effect, Selector, Value};
    use crate::mana::{cost, generic};
    CardDefinition {
        name: "Test Die Roll d6 Big Gain",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RollDie {
            sides: 6,
            count: Value::Const(1),
            modifier: Value::Const(0),
            reroll_at_most: 0,
            on_doubles: None,
            results: vec![
                (1, 2, Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
                (3, 6, Effect::LoseLife {
                    who: Selector::Player(crate::effect::PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
            ],
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
    }
}

/// Test-only fixture: d6 with only a 1-3 arm. Rolls of 4-6 run no
/// effect (CR 706.3a — "If the result was in this range").
fn test_card_die_roll_d6_partial_table() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Subtypes};
    use crate::effect::{Effect, Selector, Value};
    use crate::mana::{cost, generic};
    CardDefinition {
        name: "Test Die Roll d6 Partial Table",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RollDie {
            sides: 6,
            count: Value::Const(1),
            modifier: Value::Const(0),
            reroll_at_most: 0,
            on_doubles: None,
            results: vec![
                (1, 3, Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
                // 4-6 intentionally unmapped.
            ],
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
    }
}

/// Test-only fixture: "roll a d6 and add `modifier`. 7+: gain 5 life.
/// 1-6: lose 1 life." Exercises CR 706.2 result modifiers — only a
/// boosted roll can reach the 7+ arm on a six-sided die.
fn test_card_die_roll_d6_plus(modifier: i32) -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Subtypes};
    use crate::effect::{Effect, Selector, Value};
    use crate::mana::{cost, generic};
    CardDefinition {
        name: "Test Die Roll d6 Plus N",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RollDie {
            sides: 6,
            count: Value::Const(1),
            modifier: Value::Const(modifier),
            reroll_at_most: 0,
            on_doubles: None,
            results: vec![
                (1, 6, Effect::LoseLife { who: Selector::You, amount: Value::Const(1) }),
                (7, 255, Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
            ],
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
    }
}

/// Test-only fixture for CR 706.2b: "roll a d6, rerolling a result of
/// `reroll_at_most` or less once. 4-6: gain 5 life. 1-3: gain 1 life."
fn test_card_die_roll_d6_reroll(reroll_at_most: u8) -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType};
    use crate::effect::{Effect, Selector, Value};
    use crate::mana::{cost, generic};
    CardDefinition {
        name: "Test Die Roll d6 Reroll",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RollDie {
            sides: 6,
            count: Value::Const(1),
            modifier: Value::Const(0),
            reroll_at_most,
            on_doubles: None,
            results: vec![
                (1, 3, Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
                (4, 6, Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
            ],
        },
        ..Default::default()
    }
}

/// Test-only fixture for CR 706.5: "roll two d6. 4-6: gain 1 life each.
/// If the dice show doubles, draw a card."
fn test_card_die_roll_doubles() -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType};
    use crate::effect::{Effect, Selector, Value};
    use crate::mana::{cost, generic};
    CardDefinition {
        name: "Test Die Roll Doubles",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RollDie {
            sides: 6,
            count: Value::Const(2),
            modifier: Value::Const(0),
            reroll_at_most: 0,
            on_doubles: Some(Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            })),
            results: vec![(4, 6, Effect::GainLife { who: Selector::You, amount: Value::Const(1) })],
        },
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Witherbloom (B/G) creatures-died-this-turn
// payoffs (`Value::CreaturesDiedThisTurn` / `CreaturesDiedThisTurnTotal`).
// ─────────────────────────────────────────────────────────────────────────

/// Kill a creature controlled by `seat` via a Lightning Bolt so the SBA
/// death loop bumps `creatures_died_this_turn`. Returns after the bolt has
/// fully resolved.
fn bolt_own_creature(g: &mut GameState, seat: usize, target: CardId) {
    let bolt = g.add_card_to_hand(seat, catalog::lightning_bolt());
    g.players[seat].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(g);
}

// Test submodules: split from the original 76k-line tests/stx.rs to cut
// incremental compile time (~150 tests each); helpers above shared via super.
mod part_00;
mod part_01;
mod part_02;
mod part_03;
mod part_04;
mod part_05;
mod part_06;
mod part_07;
mod part_08;
mod part_09;
mod part_10;
mod part_11;
mod part_12;
mod part_13;
mod part_14;
mod part_15;
mod part_16;
mod part_17;
mod part_18;
mod part_19;
mod part_20;
mod part_21;
mod part_22;
mod part_23;
mod part_24;
mod part_25;
