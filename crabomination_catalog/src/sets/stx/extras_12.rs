#![allow(unused_imports)]
//! Strixhaven supplemental cards — additions to the base STX catalog
//! that flesh out the set with more castable spells and creatures.
//!
//! Cards added here typically need only existing engine primitives
//! (ETB triggers, simple targeted effects, search/learn). Cards that
//! depend on Mentor/Mutate/Lesson-sideboard primitives ship as their
//! body half only and are marked 🟡 in `STRIXHAVEN2.md`.

use super::super::no_abilities;
use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Effect, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, Selector,
    SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb_drain, etb_gain_life, magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w, ManaCost};

// ── Bookwurm ────────────────────────────────────────────────────────────────

// Bookwurm — {5}{G}{G}, 5/5 Wurm. "Trample / When this creature enters,
// you gain 4 life and draw a card."
//
// ✅ ETB body is a simple `Seq(GainLife(4), Draw(1))`. The 5/5 trample
// body is a fine top-end finisher in any green deck.

/// Prismari Flamescholar (batch 119) — {2}{U}{R}, 3/2 Elemental Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to each opponent."
pub fn prismari_flamescholar_b119() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_each_opp;
    CardDefinition {
        name: "Prismari Flamescholar (Batch 119)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_each_opp(1)],
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
    }
}

/// Prismari Inferno (batch 119) — {2}{R} Sorcery.
///
/// Synthesised: "Prismari Inferno deals 2 damage to target creature and
/// 2 damage to target opponent." Uses two slots — slot 0 (creature) and
/// slot 1 (player) — so each clause picks a distinct target.
pub fn prismari_inferno_b119() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Prismari Inferno (Batch 119)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Player,
                },
                amount: Value::Const(2),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Prismari Magmaweaver (batch 119) — {3}{U}{R}, 4/3 Elemental Wizard.
///
/// Synthesised: "When this creature enters, deal 2 damage to target
/// creature."
pub fn prismari_magmaweaver_b119() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Prismari Magmaweaver (Batch 119)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(2),
        })],
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
    }
}

/// Prismari Reshape (batch 119) — {1}{U} Instant.
///
/// Synthesised: "Return target nonland permanent to its owner's hand.
/// Scry 2."
pub fn prismari_reshape_b119() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Prismari Reshape (Batch 119)",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix (G/U) — 5 new cards (batch 119) ────────────────────────────────

/// Quandrix Polymath (batch 119) — {G}{U}, 1/2 Elf Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on this creature."
pub fn quandrix_polymath_b119() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Polymath (Batch 119)",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
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
    }
}

/// Fractal Spawnmaster (batch 119) — {3}{G}{U}, 3/3 Elf Druid.
///
/// Synthesised: "When this creature enters, create a 0/0 green-and-blue
/// Fractal creature token with three +1/+1 counters on it."
pub fn fractal_spawnmaster_b119() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Fractal Spawnmaster (Batch 119)",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        ]))],
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
    }
}

/// Quandrix Druid (batch 119) — {2}{G}, 2/3 Elf Druid.
///
/// Synthesised: "When this creature enters, put a +1/+1 counter on each
/// Fractal creature you control."
pub fn quandrix_druid_b119() -> CardDefinition {
    use crate::effect::shortcut::etb_pump_each_with_type;
    CardDefinition {
        name: "Quandrix Druid (Batch 119)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_pump_each_with_type(CreatureType::Fractal)],
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
    }
}

/// Quandrix Calculus (batch 119) — {1}{G}{U} Instant.
///
/// Synthesised: "Put a +1/+1 counter on target creature. Draw a card."
pub fn quandrix_calculus_b119() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Quandrix Calculus (Batch 119)",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Fractal Hatchling (batch 119) — {G}, 1/1 Fractal.
///
/// Synthesised: "{1}{G}{U}: Put a +1/+1 counter on this creature."
pub fn fractal_hatchling_b119() -> CardDefinition {
    CardDefinition {
        name: "Fractal Hatchling (Batch 119)",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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
            ..Default::default()
        }],
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch 120 — 25 brand-new Strixhaven synthesised cards
// ═══════════════════════════════════════════════════════════════════════════
//
// 5 cards per school using only existing engine primitives. Card naming
// uses the `_b120` suffix to disambiguate from batch 104/119 entries.

// ── Silverquill (W/B) — 5 new cards ─────────────────────────────────────────

/// Inkling Lawscribe (batch 120) — {1}{W}, 2/2 Inkling Soldier with
/// Vigilance.
///
/// Synthesised: "Vigilance. When this creature enters, you gain 1 life."
pub fn inkling_lawscribe_b120() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lawscribe (Batch 120)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(1)],
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
    }
}

/// Silverquill Devotee (batch 120) — {W}{B}, 2/2 Human Cleric with
/// Lifelink. Magecraft: each opponent loses 2 life.
///
/// Synthesised: "Lifelink. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, each opponent loses 2 life."
pub fn silverquill_devotee_b120() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Devotee (Batch 120)",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
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
    }
}

/// Silverquill Censurer (batch 120) — {2}{W}{B} Instant.
///
/// Synthesised: "Exile target creature with power 3 or less. You gain 2
/// life."
pub fn silverquill_censurer_b120() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Censurer (Batch 120)",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
                ),
                to: ZoneDest::Exile,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Inkling Battlescribe (batch 120) — {2}{W}{B}, 2/3 Inkling Knight with
/// Flying + Lifelink.
///
/// Synthesised: "Flying. Lifelink. When this creature enters, each
/// opponent loses 1 life and you gain 1 life."
pub fn inkling_battlescribe_b120() -> CardDefinition {
    CardDefinition {
        name: "Inkling Battlescribe (Batch 120)",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
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
    }
}

/// Silverquill Verdict (batch 120) — {3}{W}{B} Sorcery.
///
/// Synthesised: "Destroy target creature. Create a 1/1 white and black
/// Inkling creature token with flying."
pub fn silverquill_verdict_b120() -> CardDefinition {
    use crate::effect::shortcut::mint_inklings;
    CardDefinition {
        name: "Silverquill Verdict (Batch 120)",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            mint_inklings(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Witherbloom (B/G) — 5 new cards ─────────────────────────────────────────

/// Witherbloom Apprentice (batch 120) — {1}{B}{G}, 2/2 Human Druid.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you may pay 1 life. If you do, target creature gets
/// +1/+1 until end of turn." Engine-simplified to "magecraft: target
/// friendly creature gets +1/+1 EOT" (the optional life-payment rider
/// is a may-do shape we already use for similar cards).
pub fn witherbloom_apprentice_b120() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Witherbloom Apprentice (Batch 120)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1,
            1,
        )],
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
    }
}

/// Pest Brooder (batch 120) — {2}{B}{G}, 3/3 Pest Warlock.
///
/// Synthesised: "When this creature dies, create two 1/1 black-and-green
/// Pest creature tokens with 'When this creature dies, you gain 1 life'."
pub fn pest_brooder_b120() -> CardDefinition {
    use crate::effect::shortcut::{mint_pests, on_dies};
    CardDefinition {
        name: "Pest Brooder (Batch 120)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_dies(mint_pests(2))],
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
    }
}

/// Witherbloom Saprooter (batch 120) — {1}{B}{G} Sorcery.
///
/// Synthesised: "Each opponent loses 2 life and you gain 2 life. Create
/// a 1/1 black-and-green Pest creature token."
pub fn witherbloom_saprooter_b120() -> CardDefinition {
    use crate::effect::shortcut::mint_pests;
    CardDefinition {
        name: "Witherbloom Saprooter (Batch 120)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            mint_pests(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Witherbloom Grafter (batch 120) — {2}{G}, 2/3 Plant Druid with
/// Reach. ETB: scry 1.
pub fn witherbloom_grafter_b120() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Witherbloom Grafter (Batch 120)",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(1)],
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
    }
}

/// Pest Reaper (batch 120) — {3}{B}{G}, 4/4 Pest Warlock with Deathtouch
/// and Trample.
pub fn pest_reaper_b120() -> CardDefinition {
    CardDefinition {
        name: "Pest Reaper (Batch 120)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

// ── Lorehold (R/W) — 5 new cards ────────────────────────────────────────────

/// Lorehold Tactician (batch 120) — {1}{R}{W}, 3/2 Human Warrior with
/// First Strike.
pub fn lorehold_tactician_b120() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Tactician (Batch 120)",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Loreseeker (batch 120) — {2}{R}{W}, 2/3 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target."
pub fn lorehold_loreseeker_b120() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Loreseeker (Batch 120)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Bondbreaker (batch 120) — {3}{R}, Sorcery.
///
/// Synthesised: "This spell deals 3 damage to target creature or
/// planeswalker. Create a 2/2 red and white Spirit creature token."
pub fn lorehold_bondbreaker_b120() -> CardDefinition {
    use crate::effect::shortcut::mint_lorehold_spirits;
    CardDefinition {
        name: "Lorehold Bondbreaker (Batch 120)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            mint_lorehold_spirits(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Spirit Stonewright (batch 120) — {2}{W}, 1/4 Spirit Soldier with
/// Vigilance and Lifelink.
pub fn spirit_stonewright_b120() -> CardDefinition {
    CardDefinition {
        name: "Spirit Stonewright (Batch 120)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Lorehold Flameherald (batch 120) — {1}{R}, 2/1 Human Shaman with
/// Haste. ETB ping any 1.
pub fn lorehold_flameherald_b120() -> CardDefinition {
    use crate::effect::shortcut::etb_ping_any;
    CardDefinition {
        name: "Lorehold Flameherald (Batch 120)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_ping_any(1)],
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
    }
}

// ── Prismari (U/R) — 5 new cards ────────────────────────────────────────────

/// Prismari Apprentice (batch 120) — {1}{U}{R}, 2/2 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, scry 1."
pub fn prismari_apprentice_b120() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Prismari Apprentice (Batch 120)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
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
    }
}

/// Prismari Pyrocaster (batch 120) — {2}{U}{R}, 3/2 Elemental Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, target opponent loses 1 life."
pub fn prismari_pyrocaster_b120() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Prismari Pyrocaster (Batch 120)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
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
    }
}

/// Prismari Tempest (batch 120) — {3}{U}{R} Instant.
///
/// Synthesised: "This spell deals 4 damage to target creature. Draw a
/// card."
pub fn prismari_tempest_b120() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tempest (Batch 120)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Prismari Stormwright (batch 120) — {3}{U}, 2/3 Elemental Wizard with
/// Flying. ETB Loot.
pub fn prismari_stormwright_b120() -> CardDefinition {
    use crate::effect::shortcut::etb_loot;
    CardDefinition {
        name: "Prismari Stormwright (Batch 120)",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_loot()],
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
    }
}

/// Prismari Crucible (batch 120) — {2}{R} Sorcery.
///
/// Synthesised: "Create a Treasure token. This spell deals 2 damage to
/// any target."
pub fn prismari_crucible_b120() -> CardDefinition {
    use crate::effect::shortcut::mint_treasures;
    CardDefinition {
        name: "Prismari Crucible (Batch 120)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_treasures(1),
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix (G/U) — 5 new cards ────────────────────────────────────────────

/// Quandrix Apprentice (batch 120) — {1}{G}{U}, 2/2 Human Druid.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on this creature."
pub fn quandrix_apprentice_b120() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Apprentice (Batch 120)",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
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
    }
}

/// Fractal Brood (batch 120) — {2}{G}{U}, 3/3 Fractal.
pub fn fractal_brood_b120() -> CardDefinition {
    CardDefinition {
        name: "Fractal Brood (Batch 120)",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Quandrix Equation (batch 120) — {1}{G}{U} Instant.
///
/// Synthesised: "Put a +1/+1 counter on target creature you control.
/// Draw a card."
pub fn quandrix_equation_b120() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equation (Batch 120)",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Quandrix Tutor (batch 120) — {2}{U}, 1/2 Human Wizard.
///
/// Synthesised: "When this creature enters, draw a card."
pub fn quandrix_tutor_b120() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Quandrix Tutor (Batch 120)",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_draw(1)],
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
    }
}

/// Witherbloom Cultivator (batch 120) — {1}{B}{G}, 1/3 Plant Warlock.
///
/// Synthesised: "{1}, Sacrifice another creature: Each opponent loses 1
/// life and you gain 1 life."
///
/// Wired via the new `sac_other_filter: Some((Creature, 1))` primitive
/// — the sacrifice picks a different creature the activator controls
/// (lowest-power first per the auto-picker), not the source. Drains 1
/// each opponent on resolution.
pub fn witherbloom_cultivator_b120() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Witherbloom Cultivator (Batch 120)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: drain(1),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Fractal Bloomwright (batch 120) — {3}{G}{U} Sorcery.
///
/// Synthesised: "Create a 0/0 green and blue Fractal creature token,
/// then put four +1/+1 counters on it."
pub fn fractal_bloomwright_b120() -> CardDefinition {
    use crate::effect::shortcut::mint_fractals;
    CardDefinition {
        name: "Fractal Bloomwright (Batch 120)",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_fractals(1),
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch 121 — 5 additional cards leveraging sac_other_filter primitive
// ═══════════════════════════════════════════════════════════════════════════

/// Pest Cultmaster (batch 121) — {2}{B}{G}, 2/2 Pest Warlock.
///
/// Synthesised: "{2}, Sacrifice another creature: Draw a card."
/// Uses sac_other_filter for the cost — picks the lowest-power
/// controlled creature (not self).
pub fn pest_cultmaster_b121() -> CardDefinition {
    CardDefinition {
        name: "Pest Cultmaster (Batch 121)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Witherbloom Sapdrinker (batch 121) — {1}{B}{G}, 2/2 Vampire Warlock.
///
/// Synthesised: "Sacrifice another creature: This creature gets +2/+0
/// until end of turn." A combat trick that requires fodder — Pest
/// tokens make this scale.
pub fn witherbloom_sapdrinker_b121() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Sapdrinker (Batch 121)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Witherbloom Bonechanter (batch 121) — {1}{B}, 1/2 Skeleton Wizard.
///
/// Synthesised: "{1}{B}, Sacrifice another creature: Target creature
/// gets -2/-2 until end of turn." Anti-grow removal at instant speed.
pub fn witherbloom_bonechanter_b121() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Bonechanter (Batch 121)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Pest Ringleader (batch 121) — {3}{B}{G}, 3/3 Pest Warlock.
///
/// Synthesised: "Sacrifice another creature: Each opponent loses 2
/// life and you gain 2 life." Pure aristocrats payoff.
pub fn pest_ringleader_b121() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Pest Ringleader (Batch 121)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: drain(2),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Witherbloom Reaper (batch 121) — {3}{B}, 3/2 Skeleton Warlock with
/// Deathtouch.
///
/// Synthesised: "Deathtouch. Sacrifice another creature: This creature
/// gains indestructible until end of turn." A combat phase survivor
/// against board wipes.
pub fn witherbloom_reaper_b121() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Witherbloom Reaper (Batch 121)",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch 122 — 22 new Strixhaven cards across all five colleges
// ═══════════════════════════════════════════════════════════════════════════
//
// 8 Witherbloom (B/G) cards focusing on sacrifice / drain / Pest themes,
// 5 Silverquill (W/B) cards (drain / lifegain / Inkling),
// 3 Lorehold (R/W) ping / Spirit cards,
// 3 Prismari (U/R) magecraft / loot cards,
// 3 Quandrix (G/U) Fractal / counter cards.
//
// All cards use only existing engine primitives — no new engine work
// required.

// ── Witherbloom (B/G) ──────────────────────────────────────────────────────

/// Pest Cultcaller (batch 122) — {1}{B}{G}, 2/2 Pest Warlock.
///
/// Synthesised: "{B}, Sacrifice another creature: Each opponent loses
/// 1 life and you gain 1 life." A repeatable drain engine that turns
/// any creature (including Pest tokens) into 1-mana drain.
pub fn pest_cultcaller_b122() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Pest Cultcaller (Batch 122)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[b()]),
            effect: drain(1),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Witherbloom Bloodgrafter (batch 122) — {2}{B}{G}, 3/3 Vampire Warlock.
///
/// Synthesised: "When this creature enters, each opponent loses 2 life
/// and you gain 2 life. Whenever you sacrifice a creature, put a +1/+1
/// counter on this creature." Pairs the etb_drain shortcut with a
/// `CreatureSacrificed/YourControl` payoff trigger.
pub fn witherbloom_bloodgrafter_b122() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bloodgrafter (Batch 122)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(2),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CreatureSacrificed,
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
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
    }
}

/// Witherbloom Composter (batch 122) — {1}{B}{G}, 2/2 Plant Druid.
///
/// Synthesised: "{1}, Sacrifice another creature: Draw a card and you
/// lose 1 life." A cheap card-draw engine that costs a body and a life.
pub fn witherbloom_composter_b122() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Composter (Batch 122)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
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
    }
}

/// Pest Swarmcaller (batch 122) — {3}{B}{G} Sorcery.
///
/// Synthesised: "Create two 1/1 black-and-green Pest creature tokens
/// with 'When this creature dies, you gain 1 life.' Each opponent
/// loses 2 life and you gain 2 life."
pub fn pest_swarmcaller_b122() -> CardDefinition {
    use crate::effect::shortcut::{drain, mint_pests};
    CardDefinition {
        name: "Pest Swarmcaller (Batch 122)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![mint_pests(2), drain(2)]),
        activated_abilities: no_abilities(),
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
    }
}

/// Witherbloom Sapdrainer (batch 122) — {3}{B}{G}, 4/3 Vampire Warlock
/// with Lifelink.
///
/// Synthesised: ETB drain 2 stapled on a five-mana lifelink finisher.
/// Functionally a Vampire Nighthawk-style top-end racer for the
/// Witherbloom drain shell.
pub fn witherbloom_sapdrainer_b122() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapdrainer (Batch 122)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
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
    }
}

/// Witherbloom Necrotutor (batch 122) — {2}{B}, 2/2 Skeleton Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you may put a creature card from your graveyard on
/// top of your library." Skips the printed "may" optionality (always
/// returns when a legal target exists; falls through when graveyard is
/// empty). Wires reanimation via Move(target → Library top).
pub fn witherbloom_necrotutor_b122() -> CardDefinition {
    use crate::effect::{LibraryPosition, ZoneDest};
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Necrotutor (Batch 122)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
        })],
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
    }
}

/// Witherbloom Spinecaster (batch 122) — {1}{B}, 1/3 Plant Wizard.
///
/// Synthesised: "When this creature enters, target creature gets -1/-1
/// until end of turn." Cheap shrink-removal with a 1/3 body that blocks
/// 2-power creatures profitably.
pub fn witherbloom_spinecaster_b122() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Spinecaster (Batch 122)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
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
    }
}

/// Pest Brewmaster (batch 122) — {2}{B}{G}, 1/1 Pest Warlock.
///
/// Synthesised: "When this creature enters, create two 1/1 Pest tokens.
/// Whenever another Pest you control dies, you gain 1 life." A Pest
/// engine commander — the ETB mints two fodder bodies and every Pest
/// death (own or token) bumps life by 1, on top of the token's printed
/// "die → +1 life" rider, for a 2-life-per-Pest-death effective swing.
pub fn pest_brewmaster_b122() -> CardDefinition {
    use crate::effect::shortcut::mint_pests;
    CardDefinition {
        name: "Pest Brewmaster (Batch 122)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(mint_pests(2)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                    }),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
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
    }
}

// ── Silverquill (W/B) ──────────────────────────────────────────────────────

/// Inkling Quillstrike (batch 122) — {1}{W}{B}, 2/2 Inkling Cleric with
/// Flying.
///
/// Synthesised: "When this creature enters, target creature an opponent
/// controls gets -2/-2 until end of turn." A 3-mana evasive Inkling
/// that doubles as a removal spell on opponent's small creatures.
pub fn inkling_quillstrike_b122() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillstrike (Batch 122)",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
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
    }
}

/// Silverquill Mentor (batch 122) — {2}{W}, 2/3 Human Cleric.
///
/// Synthesised: "When this creature enters, you gain 2 life. Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, target
/// creature you control gets +1/+1 until end of turn." Combines the
/// `etb_gain_life(2)` shortcut with the `magecraft_target_pump`
/// helper for a Lifelink-Yargle midrange body.
pub fn silverquill_mentor_b122() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Silverquill Mentor (Batch 122)",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(2),
            magecraft_target_pump(
                target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                1,
                1,
            ),
        ],
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
    }
}

/// Silverquill Verdict (batch 122) — {3}{W}{B} Sorcery.
///
/// Synthesised: "Destroy target creature. You gain life equal to its
/// power." A removal spell with life recovery — wires `Effect::Move →
/// Graveyard` for the destruction (Destroy semantics) and reads
/// `Value::PowerOf(Target(0))` for the life gain. The two-step Seq
/// resolves both halves against the same target slot.
pub fn silverquill_verdict_b122() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Silverquill Verdict (Batch 122)",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Graveyard,
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Inkling Glyphwarden (batch 122) — {3}{W}{B}, 2/4 Inkling Wizard
/// with Flying and Lifelink.
///
/// Synthesised: A vanilla evasive lifelinker on a 5-mana frame. Pairs
/// with Tenured Inkcaster's +2/+2 anthem for a 4/6 flying lifelinker.
pub fn inkling_glyphwarden_b122() -> CardDefinition {
    CardDefinition {
        name: "Inkling Glyphwarden (Batch 122)",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
    }
}

/// Silverquill Reverence (batch 122) — {W}{B} Instant.
///
/// Synthesised: "Drain 1 + Draw 1." A clean 2-mana drain cantrip
/// using the existing `drain_and_draw(1)` shortcut.
pub fn silverquill_reverence_b122() -> CardDefinition {
    use crate::effect::shortcut::drain_and_draw;
    CardDefinition {
        name: "Silverquill Reverence (Batch 122)",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_draw(1),
        activated_abilities: no_abilities(),
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
    }
}

// ── Lorehold (R/W) ─────────────────────────────────────────────────────────

/// Lorehold Pyroscholar (batch 122) — {R}{W}, 2/2 Human Cleric.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target." A 2-mana
/// magecraft ping body — wires the canonical `magecraft_ping_any(1)`
/// shortcut.
pub fn lorehold_pyroscholar_b122() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Pyroscholar (Batch 122)",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
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
    }
}

/// Lorehold Reliquaer (batch 122) — {3}{R}{W} Sorcery.
///
/// Synthesised: "Create two 2/2 R/W Spirit tokens. This spell deals 1
/// damage to each opponent." Spirit token rain + drain.
pub fn lorehold_reliquaer_b122() -> CardDefinition {
    use crate::effect::shortcut::mint_lorehold_spirits;
    CardDefinition {
        name: "Lorehold Reliquaer (Batch 122)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_lorehold_spirits(2),
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Lorehold Battlescryer (batch 122) — {2}{R}{W}, 3/3 Human Soldier
/// with Haste.
///
/// Synthesised: "Haste. When this creature attacks, this creature
/// deals 1 damage to any target." A combat-trigger pinger that piles
/// damage onto blockers or the player on top of its 3 attacking power.
pub fn lorehold_battlescryer_b122() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Lorehold Battlescryer (Batch 122)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(1),
        })],
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
    }
}

// ── Prismari (U/R) ─────────────────────────────────────────────────────────

/// Prismari Loresage (batch 122) — {1}{U}{R}, 2/3 Human Wizard.
///
/// Synthesised: "When this creature enters, draw a card, then discard
/// a card." Standard loot ETB body — wires `etb_loot()`.
pub fn prismari_loresage_b122() -> CardDefinition {
    use crate::effect::shortcut::etb_loot;
    CardDefinition {
        name: "Prismari Loresage (Batch 122)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_loot()],
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
    }
}

/// Prismari Inferno (batch 122) — {3}{R} Sorcery.
///
/// Synthesised: "This spell deals 4 damage to any target. Draw a card."
/// Lava-Coil-class removal with a cantrip.
pub fn prismari_inferno_b122() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inferno (Batch 122)",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
    }
}

/// Prismari Sparkmage (batch 122) — {1}{R}, 1/2 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to target creature." A
/// repeating creature-only pinger via `magecraft_ping_creature(1)`.
pub fn prismari_sparkmage_b122() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_creature;
    CardDefinition {
        name: "Prismari Sparkmage (Batch 122)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_creature(1)],
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
    }
}

// ── Quandrix (G/U) ─────────────────────────────────────────────────────────

/// Fractal Multiplier (batch 122) — {2}{G}{U}, 3/3 Fractal.
///
/// Synthesised: "When this creature enters, put a +1/+1 counter on
/// target creature you control." Vanilla token-grow body.
pub fn fractal_multiplier_b122() -> CardDefinition {
    CardDefinition {
        name: "Fractal Multiplier (Batch 122)",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
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
    }
}
