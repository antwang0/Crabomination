//! Quandrix (G/U) college cards from Strixhaven.
//!
//! Quandrix cares about **Fractal tokens** (0/0 green-and-blue with
//! variable +1/+1 counters), spell-cast triggers, and X-cost scaling.
//! The first-pass set here covers the two college "Apprentice" /
//! "Pledgemage" creatures plus a couple of mono-flavour scaling cards.
//! Larger Fractal-creator effects (Body of Research, Fractal Anomaly)
//! are already wired in `mono` / SOS — those compose against the same
//! `LastCreatedToken` plumbing this module re-uses.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Selector,
    SelectionRequirement, Subtypes, Value,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::Duration;
use crate::mana::{cost, generic, g, u};

// ── Quandrix Apprentice ─────────────────────────────────────────────────────

/// Quandrix Apprentice — {G}{U}, 1/1 Elf Druid.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target creature you control gets +1/+1 until end of turn."
///
/// Same shape as Eager First-Year (the Silverquill apprentice), just
/// gated to a creature you control rather than any creature. Wired via
/// the new `effect::shortcut::magecraft` helper plus
/// `Predicate::EntityMatches` on the cast.
pub fn quandrix_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Apprentice",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Quandrix Pledgemage ─────────────────────────────────────────────────────

/// Quandrix Pledgemage — {1}{G}{U}, 2/2 Fractal Wizard. "{1}{G}{U}: Put
/// a +1/+1 counter on Quandrix Pledgemage."
///
/// Pure activated +1/+1 counter pump. The Fractal subtype is already in
/// the engine (added with the SOS Fractal package), so the body and
/// counter accrual are faithful to the printed card.
pub fn quandrix_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pledgemage",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), g(), u()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Decisive Denial ─────────────────────────────────────────────────────────

/// Decisive Denial — {G}{U} Instant. "Choose one — / • Counter target
/// noncreature spell unless its controller pays {2}. / • Target creature
/// you control deals damage equal to its power to target creature you
/// don't control."
///
/// Mode 1 is a Fight resolution; the printed "two target" prompt is
/// auto-resolved on the defender side, attacker is player-chosen via
/// `Target(0)`. Multi-target defender prompt remains a future engine
/// enhancement.
pub fn decisive_denial() -> CardDefinition {
    use crate::mana::{ManaCost, generic as gen_pip};
    let two = ManaCost { symbols: vec![gen_pip(2)] };
    CardDefinition {
        name: "Decisive Denial",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target noncreature spell unless its controller
            // pays {2}.
            Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Creature).negate()),
                ),
                mana_cost: two,
            },
            // Mode 1: target creature you control fights an auto-picked
            // opponent creature (same Chelonian Tackle pattern).
            Effect::Fight {
                attacker: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                defender: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Symmathematics ──────────────────────────────────────────────────────────

/// Symmathematics — {1}{G}{U}, Creature — Fractal (0/0). "Symmathematics
/// enters with two +1/+1 counters on it. / Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, double the number of +1/+1
/// counters on Symmathematics."
///
/// Body is a 0/0 Fractal that comes in as a 2/2 via the new
/// `CardDefinition.enters_with_counters` field (CR 614.12 replacement).
/// The two +1/+1 counters land **before** the new permanent is exposed
/// to state-based-action sweeps and before any ETB triggers fire, so a
/// printed 0/0 body survives ETB without the historic base-toughness
/// bump (was 1/1 base + ETB AddCounter approximation; now exact 0/0
/// printed with CR-compliant "enters with").
///
/// Magecraft is the standard `AddCounter { what: This, amount:
/// CountersOn(This, +1/+1) }` shape (same as Practical Research, Growth
/// Curve): adds N more counters where N is the current pile, producing
/// 2N total. `Selector::This` resolves to the trigger's listening
/// permanent (Symmathematics itself).
pub fn symmathematics() -> CardDefinition {
    CardDefinition {
        name: "Symmathematics",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        // Printed P/T is 0/0 — the +1/+1 counters from the CR 614.12
        // replacement now land before SBA, so the printed base survives.
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // Magecraft: double the +1/+1 counters on Symmathematics.
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        // CR 614.12 "enters with two +1/+1 counters on it" replacement.
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        exile_on_resolve: false,
    }
}

