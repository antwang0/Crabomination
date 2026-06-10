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

/// Quandrix Coursemage (batch 122) — {1}{G}{U}, 2/2 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on target creature you control."
/// Repeating counter rain on a friendly target — uses the new
/// `magecraft_add_counter_to_friendly()` shortcut.
pub fn quandrix_coursemage_b122() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_to_friendly;
    CardDefinition {
        name: "Quandrix Coursemage (Batch 122)",
        cost: cost(&[generic(1), g(), u()]),
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
        triggered_abilities: vec![magecraft_add_counter_to_friendly()],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Quandrix Expansion (batch 122) — {2}{G}{U} Sorcery.
///
/// Synthesised: "Create a 0/0 Fractal token. Put X +1/+1 counters on
/// it, where X = number of lands you control." Scales with landfall
/// payoff shells.
pub fn quandrix_expansion_b122() -> CardDefinition {
    use crate::effect::shortcut::mint_fractals;
    CardDefinition {
        name: "Quandrix Expansion (Batch 122)",
        cost: cost(&[generic(2), g(), u()]),
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
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou),
                ))),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch 123 — 20 new Strixhaven cards focused on finishing Witherbloom
// ═══════════════════════════════════════════════════════════════════════════
//
// 9 Witherbloom (B/G) cards — Pest/drain/sacrifice payoffs,
// 4 Silverquill (W/B) cards — Inkling / lifegain,
// 3 Lorehold (R/W) cards — Spirit-token / haste / ping,
// 2 Prismari (U/R) cards — loot / spell-slinger,
// 2 Quandrix (G/U) cards — counter / fractal.
//
// New engine helpers: `dies_lose_life_each_opp` (asymmetric on-death drain),
// `magecraft_drain` (symmetric magecraft drain) — both shipped in
// `effect::shortcut`. CR 704 — State-Based Actions audit row added to
// TODO.md.

// ── Witherbloom (B/G) ──────────────────────────────────────────────────────

/// Pest Marrowfeast (batch 123) — {2}{B}{G}, 3/2 Pest Warlock.
///
/// Synthesised: "When this creature enters, create a 1/1 Pest token.
/// Whenever another Pest you control dies, target opponent loses 1
/// life and you gain 1 life." A small Pest tribal commander — every
/// Pest death now pings the opponent.
pub fn pest_marrowfeast_b123() -> CardDefinition {
    use crate::effect::shortcut::mint_pests;
    CardDefinition {
        name: "Pest Marrowfeast (Batch 123)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(mint_pests(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                    }),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Witherbloom Vinegrowth (batch 123) — {1}{B}{G}, 2/3 Plant Druid.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, each opponent loses 1 life and you gain 1 life."
/// Apprentice-template magecraft drain body on a 2-mana 2/3 frame.
pub fn witherbloom_vinegrowth_b123() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain;
    CardDefinition {
        name: "Witherbloom Vinegrowth (Batch 123)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain(1)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Witherbloom Crypttender (batch 123) — {3}{B}{G}, 3/4 Skeleton Druid.
///
/// Synthesised: "When this creature enters, return target creature
/// card from your graveyard to your hand. When this creature dies,
/// each opponent loses 2 life." A midrange recursion body with a
/// drain on death.
pub fn witherbloom_crypttender_b123() -> CardDefinition {
    use crate::effect::shortcut::dies_lose_life_each_opp;
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Crypttender (Batch 123)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            dies_lose_life_each_opp(2),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Pest Mawlord (batch 123) — {4}{B}{G}, 4/4 Pest Warlock.
///
/// Synthesised: "When this creature enters, create two 1/1 Pest
/// tokens. When this creature dies, each opponent loses 2 life."
/// A finisher Pest commander — ETB mints fodder, death drains hard.
pub fn pest_mawlord_b123() -> CardDefinition {
    use crate::effect::shortcut::{dies_lose_life_each_opp, mint_pests};
    CardDefinition {
        name: "Pest Mawlord (Batch 123)",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(mint_pests(2)),
            dies_lose_life_each_opp(2),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Witherbloom Bonesplitter (batch 123) — {2}{B}, 3/2 Skeleton Warlock.
///
/// Synthesised: "Deathtouch / {B}, Sacrifice another creature: Target
/// creature gets -1/-1 until end of turn." A repeatable removal
/// engine that double-uses as deathtouch blocker.
pub fn witherbloom_bonesplitter_b123() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Bonesplitter (Batch 123)",
        cost: cost(&[generic(2), b()]),
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
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Witherbloom Tombrooter (batch 123) — {2}{B}{G} Sorcery.
///
/// Synthesised: "Return target creature card from your graveyard to
/// the battlefield. Each opponent loses 1 life." A reanimate body
/// with a drain rider.
pub fn witherbloom_tombrooter_b123() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Tombrooter (Batch 123)",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Witherbloom Beetlecaller (batch 123) — {1}{B}{G}, 1/2 Insect Druid.
///
/// Synthesised: "When this creature enters, create a 1/1 Pest token.
/// Whenever another creature you control dies, put a +1/+1 counter on
/// this creature." A 3-mana aristocrats grow body that snowballs every
/// time a Pest token dies.
pub fn witherbloom_beetlecaller_b123() -> CardDefinition {
    use crate::effect::shortcut::mint_pests;
    CardDefinition {
        name: "Witherbloom Beetlecaller (Batch 123)",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(mint_pests(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Witherbloom Saproot (batch 123) — {B}{G}, 2/2 Plant Druid.
///
/// Synthesised: "When this creature dies, each opponent loses 1 life
/// and you gain 1 life." A 2-drop sacrificable body with a
/// dies-drain rider — pairs with sac outlets for value.
pub fn witherbloom_saproot_b123() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Witherbloom Saproot (Batch 123)",
        cost: cost(&[b(), g()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Pest Hivekeeper (batch 123) — {3}{B}{G} Sorcery.
///
/// Synthesised: "Create three 1/1 Pest tokens." Pure Pest mint at
/// sorcery speed — combos with Pest Brewmaster, Pest Marrowfeast.
pub fn pest_hivekeeper_b123() -> CardDefinition {
    use crate::effect::shortcut::mint_pests;
    CardDefinition {
        name: "Pest Hivekeeper (Batch 123)",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: mint_pests(3),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Silverquill (W/B) ──────────────────────────────────────────────────────

/// Inkling Crusader (batch 123) — {2}{W}{B}, 3/3 Inkling Cleric.
///
/// Synthesised: "Flying, Vigilance / When this creature enters, you
/// gain 2 life." A 4-mana evasive vigilant lifegainer.
pub fn inkling_crusader_b123() -> CardDefinition {
    CardDefinition {
        name: "Inkling Crusader (Batch 123)",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Silverquill Adjudicator (batch 123) — {3}{W}{B} Sorcery.
///
/// Synthesised: "Exile target creature. You gain 2 life." A clean
/// hard removal with lifegain attached.
pub fn silverquill_adjudicator_b123() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Adjudicator (Batch 123)",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Silverquill Sermonizer (batch 123) — {1}{W}, 2/1 Human Cleric.
///
/// Synthesised: "When this creature enters, you gain 1 life.
/// Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, you gain 1 life." A pure lifegain spellslinger body.
pub fn silverquill_sermonizer_b123() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life;
    CardDefinition {
        name: "Silverquill Sermonizer (Batch 123)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(1), magecraft_gain_life(1)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Inkling Pamphletter (batch 123) — {2}{W}{B}, 2/3 Inkling Wizard.
///
/// Synthesised: "Flying / When this creature enters, target opponent
/// loses 2 life and you gain 2 life." A 4-mana evasive drain body.
pub fn inkling_pamphletter_b123() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pamphletter (Batch 123)",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Lorehold (R/W) ─────────────────────────────────────────────────────────

/// Lorehold Vanguard (batch 123) — {2}{R}{W}, 3/3 Human Warrior.
///
/// Synthesised: "Haste, First Strike / Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature gets +1/+0
/// until end of turn." Spellslinger beatdown body.
pub fn lorehold_vanguard_b123() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguard (Batch 123)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste, Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Lorehold Spiritsong (batch 123) — {3}{R}{W} Sorcery.
///
/// Synthesised: "Create two 2/2 red and white Spirit creature
/// tokens. They gain haste until end of turn." Tempo-Spirit mint
/// with haste rider so they swing immediately.
pub fn lorehold_spiritsong_b123() -> CardDefinition {
    use crate::effect::shortcut::mint_lorehold_spirits;
    CardDefinition {
        name: "Lorehold Spiritsong (Batch 123)",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            mint_lorehold_spirits(2),
            Effect::GrantKeyword {
                what: Selector::LastCreatedTokens,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Lorehold Skirmisher (batch 123) — {1}{R}, 2/1 Human Warrior.
///
/// Synthesised: "Haste / Whenever this creature attacks, it deals 1
/// damage to any target." A 2-mana attack-trigger ping creature.
pub fn lorehold_skirmisher_b123() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmisher (Batch 123)",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::on_attack(
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(1),
            },
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Prismari (U/R) ─────────────────────────────────────────────────────────

/// Prismari Tutor (batch 123) — {2}{U}{R}, 2/2 Human Wizard.
///
/// Synthesised: "When this creature enters, draw two cards, then
/// discard a card." A 4-mana looter on a 2/2 body.
pub fn prismari_tutor_b123() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tutor (Batch 123)",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Prismari Sparkshow (batch 123) — {1}{U}{R} Instant.
///
/// Synthesised: "This spell deals 2 damage to any target. Draw a
/// card." Cantripping bolt — combines Lightning Bolt + Brainstorm
/// at instant speed on a 3-mana frame.
pub fn prismari_sparkshow_b123() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkshow (Batch 123)",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Quandrix (G/U) ─────────────────────────────────────────────────────────

/// Quandrix Surveyor (batch 123) — {1}{G}{U}, 2/2 Merfolk Wizard.
///
/// Synthesised: "When this creature enters, put a +1/+1 counter on
/// target creature you control. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on target
/// creature you control." A counter-fan engine.
pub fn quandrix_surveyor_b123() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_to_friendly;
    CardDefinition {
        name: "Quandrix Surveyor (Batch 123)",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            magecraft_add_counter_to_friendly(),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Fractal Pondlord (batch 123) — {3}{G}{U}, 3/3 Fractal.
///
/// Synthesised: "When this creature enters, create a 0/0 Fractal
/// token. Put X +1/+1 counters on it, where X is your devotion to
/// green and blue (count of {G}/{U} pips among colors of permanents
/// you control)." Approximated to "count of green+blue creatures you
/// control" via existing primitives.
pub fn fractal_pondlord_b123() -> CardDefinition {
    use crate::effect::shortcut::mint_fractals;
    CardDefinition {
        name: "Fractal Pondlord (Batch 123)",
        cost: cost(&[generic(3), g(), u()]),
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
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            mint_fractals(1),
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch 124 — 10 more Strixhaven cards rounding out Lorehold/Prismari/Quandrix
// ═══════════════════════════════════════════════════════════════════════════
//
// 4 Lorehold (R/W), 3 Prismari (U/R), 3 Quandrix (G/U).
// All cards use existing engine primitives — no new engine work.

// ── Lorehold (R/W) ─────────────────────────────────────────────────────────

/// Lorehold Pyromancer (batch 124) — {2}{R}, 2/3 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target." A
/// 3-mana Spellslinger ping body.
pub fn lorehold_pyromancer_b124() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Pyromancer (Batch 124)",
        cost: cost(&[generic(2), r()]),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Lorehold Skydefender (batch 124) — {3}{W}, 2/4 Spirit Soldier.
///
/// Synthesised: "Flying / When this creature enters, you gain 3
/// life." A 4-mana flyer with life-recovery.
pub fn lorehold_skydefender_b124() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skydefender (Batch 124)",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(3)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Lorehold Champion (batch 124) — {2}{R}{W}, 3/3 Human Warrior.
///
/// Synthesised: "Vigilance / Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +2/+0 until end of turn." A
/// midrange magecraft attacker.
pub fn lorehold_champion_b124() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Champion (Batch 124)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(2, 0)],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Lorehold Cremate (batch 124) — {R}{W} Sorcery.
///
/// Synthesised: "This spell deals 3 damage to target creature.
/// Create a 2/2 red and white Spirit creature token." Combat trick
/// + token mint.
pub fn lorehold_cremate_b124() -> CardDefinition {
    use crate::effect::shortcut::mint_lorehold_spirits;
    CardDefinition {
        name: "Lorehold Cremate (Batch 124)",
        cost: cost(&[r(), w()]),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Prismari (U/R) ─────────────────────────────────────────────────────────

/// Prismari Stormbreaker (batch 124) — {3}{U}{R}, 4/3 Elemental.
///
/// Synthesised: "Trample / When this creature enters, draw a card,
/// then discard a card." A 5-mana trampler with a loot rider.
pub fn prismari_stormbreaker_b124() -> CardDefinition {
    use crate::effect::shortcut::etb_loot;
    CardDefinition {
        name: "Prismari Stormbreaker (Batch 124)",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Prismari Burnmage (batch 124) — {1}{U}{R}, 2/2 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target." A
/// 3-mana magecraft Spellslinger ping body.
pub fn prismari_burnmage_b124() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Prismari Burnmage (Batch 124)",
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Prismari Tempest (batch 124) — {2}{U}{R} Sorcery.
///
/// Synthesised: "This spell deals 3 damage to any target. Draw a
/// card." Bolt + cantrip at sorcery speed.
pub fn prismari_tempest_b124() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tempest (Batch 124)",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(3),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Quandrix (G/U) ─────────────────────────────────────────────────────────

/// Quandrix Forester (batch 124) — {2}{G}, 3/3 Elf Druid.
///
/// Synthesised: "When this creature enters, put a +1/+1 counter on
/// target creature you control. Whenever this creature attacks, put
/// a +1/+1 counter on it." A two-trigger growth engine.
pub fn quandrix_forester_b124() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Forester (Batch 124)",
        cost: cost(&[generic(2), g()]),
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
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            crate::effect::shortcut::on_attack(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Quandrix Mathematician (batch 124) — {2}{G}{U}, 3/2 Merfolk
/// Wizard.
///
/// Synthesised: "Whenever this creature deals combat damage to a
/// player, put a +1/+1 counter on it. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, put a +1/+1 counter on
/// target creature you control." Two counter-fan triggers.
pub fn quandrix_mathematician_b124() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_to_friendly;
    CardDefinition {
        name: "Quandrix Mathematician (Batch 124)",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            magecraft_add_counter_to_friendly(),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Fractal Coursemate (batch 124) — {1}{G}{U}, 0/0 Fractal.
///
/// Synthesised: "This creature enters with X +1/+1 counters on it,
/// where X is twice the number of cards in your hand." Uses the
/// `enters_with_counters` replacement (CR 614.12) so the counters
/// land before SBAs check 0/0 toughness — otherwise the Fractal
/// would die before the ETB trigger resolved.
pub fn fractal_coursemate_b124() -> CardDefinition {
    CardDefinition {
        name: "Fractal Coursemate (Batch 124)",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
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
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Hand,
                    filter: SelectionRequirement::Any,
                }))),
            ),
        )),
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Cycling-keyword test card (CR 702.29) ──────────────────────────────────

/// Strixhaven Cycle-Decree (b145) — {5}{B} Sorcery: "All creatures
/// get -1/-1 until end of turn. / Cycling {3}{B}. / When you cycle
/// this card, draw 3 cards."
///
/// Synthesised filler test card to lock in the cycle-trigger pipeline
/// (CR 702.29c). On cycle, the source is in graveyard at dispatch
/// time; the new dispatcher pass walks the cycler's graveyard for
/// `EventKind::CardCycled` + `EventScope::SelfSource` triggers and
/// fires them with `source = cycled card id`. Test verifies the
/// draw-3 fires after cycling.
pub fn strixhaven_cycle_decree_b145() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Cycle-Decree (b145)",
        cost: cost(&[generic(5), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Cycling(cost(&[generic(3), b()]))],
        // Body: minus-one-minus-one to each creature is approximated as
        // a no-op here so the test focuses on the cycle trigger; the
        // cycling cost activation is the headline play pattern.
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: crate::card::EventSpec::new(
                crate::card::EventKind::CardCycled,
                crate::card::EventScope::SelfSource,
            ),
            effect: Effect::Draw {
                who: crate::effect::Selector::You,
                amount: crate::card::Value::Const(3),
            },
        }],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

/// Strixhaven Cycle-Glyph (b143) — {3}{U} Sorcery: "Draw two cards.
/// / Cycling {1}{U}".
///
/// Synthesised filler test card that exercises the new
/// `GameAction::Cycle` path. The body half is castable from hand as a
/// 4-mana draw-2 sorcery; the Cycling {1}{U} half discards-and-draws
/// from hand on demand at the 2-mana payment line. The same shape
/// covers any future STA reprint with Cycling (Decree of Pain,
/// Akroma's Vengeance, Boon of the Wish-Giver). Per CR 702.29c, "When
/// you cycle this card" triggers fire from whatever zone the card
/// winds up in after the discard — this filler card has none, but the
/// `CardDiscarded` event is emitted so future cycle-matters cards see
/// the cycle.
pub fn strixhaven_cycle_glyph_b143() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Cycle-Glyph (b143)",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Cycling(cost(&[generic(1), u()]))],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Strixhaven Stasis-Glyph (b160) ─────────────────────────────────────────

/// Strixhaven Stasis-Glyph (b160) — {3}{U} Enchantment.
/// "Lands you control don't untap during your untap step."
/// Wires the new `StaticEffect::PreventUntap` primitive (CR 502.3).
/// Standalone test card — closes a long-standing engine gap.
pub fn strixhaven_stasis_glyph_b160() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Stasis-Glyph (b160)",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Lands you control don't untap during your untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou),
                ),
            },
        }],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Back to Basics ────────────────────────────────────────────────────────────

/// Back to Basics — {2}{U} Enchantment.
/// "Nonbasic lands don't untap during their controllers' untap steps."
///
/// Uses the `PreventUntap` static effect targeting all nonbasic lands.
pub fn back_to_basics() -> CardDefinition {
    CardDefinition {
        name: "Back to Basics",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Nonbasic lands don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::IsBasicLand.negate()),
                ),
            },
        }],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Collector Ouphe ───────────────────────────────────────────────────────────

/// Collector Ouphe — {1}{G}, 2/2 Ouphe.
/// "Activated abilities of artifacts can't be activated."
///
/// Body-only: the "artifacts can't activate" static needs a new
/// `StaticEffect::PreventActivation { applies_to }` primitive. The 2/2
/// body in green is still useful for the cube.
pub fn collector_ouphe() -> CardDefinition {
    CardDefinition {
        name: "Collector Ouphe",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ouphe],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        // "Activated abilities of artifacts can't be activated unless
        // they're mana abilities."
        static_abilities: vec![crate::card::StaticAbility {
            description: "Activated abilities of artifacts can't be activated unless they're mana abilities.",
            effect: crate::effect::StaticEffect::ArtifactActivatedAbilitiesLocked,
        }],
        ..Default::default()
    }
}

// ── Arclight Phoenix ──────────────────────────────────────────────────────────

/// Arclight Phoenix — {3}{R}, 3/2 Phoenix.
/// "Flying, haste. / At the beginning of combat on your turn, if you've
/// cast three or more instant and/or sorcery spells this turn, return
/// Arclight Phoenix from your graveyard to the battlefield."
///
/// Body: 3/2 Flying Haste. The graveyard-recursion trigger is omitted
/// (needs a begin-combat trigger scoped to graveyard-resident cards +
/// 3+ IS spell gate). The body is a strong hasty flier for red decks.
/// Arclight Phoenix — {2}{R} Creature — Phoenix. 3/2 Flying, Haste. At the
/// beginning of combat on your turn, if you've cast three or more instant
/// and/or sorcery spells this turn, return this from your graveyard to the
/// battlefield (a `FromYourGraveyard` begin-combat trigger gated on
/// `InstantsOrSorceriesCastThisTurnAtLeast`).
pub fn arclight_phoenix() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Arclight Phoenix",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phoenix],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::FromYourGraveyard,
            )
            .with_filter(Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(3),
            }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..Default::default()
    }
}

// ── Opposition ────────────────────────────────────────────────────────────────

/// Opposition — {2}{U}{U} Enchantment.
/// "Tap an untapped creature you control: Tap target artifact, creature,
/// or land." (Tap-another-as-cost via `tap_other_filter`.)
pub fn opposition() -> CardDefinition {
    CardDefinition {
        name: "Opposition",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_other_filter: Some(SelectionRequirement::Creature), from_hand: false,
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Land),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Omniscience ───────────────────────────────────────────────────────────────

/// Omniscience — {7}{U}{U}{U} Enchantment.
/// "You may cast spells from your hand without paying their mana costs."
/// (Free cast via `GameAction::CastFromZoneWithoutPaying`.)
pub fn omniscience() -> CardDefinition {
    CardDefinition {
        name: "Omniscience",
        cost: cost(&[generic(7), u(), u(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may cast spells from your hand without paying \
                          their mana costs.",
            effect: StaticEffect::CastHandSpellsFree,
        }],
        ..Default::default()
    }
}

// ── Blustersquall ─────────────────────────────────────────────────────────────

/// Blustersquall — {U} Instant.
/// "Tap target creature you don't control. / Overload {3}{U}
/// (Tap each creature you don't control.)"
pub fn blustersquall() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Blustersquall",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Tap {
                    what: Selector::TriggerSource,
                }),
            }),
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Electrickery ──────────────────────────────────────────────────────────────

/// Electrickery — {R} Instant.
/// "Electrickery deals 1 damage to target creature you don't control.
/// / Overload {1}{R} (deals 1 damage to each creature you don't control.)"
pub fn electrickery() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Electrickery",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::Const(1),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(1),
                }),
            }),
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Teleportal ────────────────────────────────────────────────────────────────

/// Teleportal — {U}{R} Sorcery.
/// "Target creature you control gets +1/+0 and is unblockable this turn.
/// / Overload {3}{U}{R} (Each creature you control gets +1/+0 and is
/// unblockable this turn.)"
pub fn teleportal() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Teleportal",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u(), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(1),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}

// ── Street Spasm ──────────────────────────────────────────────────────────────

/// Street Spasm — {X}{R} Instant.
/// "Street Spasm deals X damage to target creature without flying you
/// don't control. / Overload {X}{X}{R}{R} (deals X damage to each
/// creature without flying you don't control.)"
pub fn street_spasm() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Street Spasm",
        cost: cost(&[crate::mana::x(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
            ),
            amount: Value::XFromCost,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[crate::mana::x(), crate::mana::x(), r(), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent)
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::XFromCost,
                }),
            }),
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
        room: None,
    }
}
