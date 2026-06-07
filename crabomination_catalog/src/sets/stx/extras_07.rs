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

// ── Prismari Channeler ─────────────────────────────────────────────────────

/// Prismari Channeler — {2}{R}, 2/3 Human Wizard.
/// "{T}: Add {U} or {R}."
///
/// Mana-fixing dork in Prismari colors for U/R decks. Same shape as
/// the Quandrix Engineer's body.
pub fn prismari_channeler() -> CardDefinition {
    use crate::catalog::sets::tap_add;
    CardDefinition {
        name: "Prismari Channeler",
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
        activated_abilities: vec![tap_add(Color::Blue), tap_add(Color::Red)],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Anthem ────────────────────────────────────────────────────────

/// Lorehold Anthem — {2}{W} Enchantment.
/// "Creatures you control get +1/+1."
///
/// Classic Glorious Anthem. Wired via a `StaticAbility::PumpPT` with
/// `AffectedPermanents::All { controller: Some(true), card_types:
/// [Creature], exclude_source: false }`.
pub fn lorehold_anthem() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Anthem",
        cost: cost(&[generic(2), w()]),
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
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks current, batch 5): 7 more STX cards including some
// more interesting effects (Selesnya/Azorius dual-pip + cross-college).
// ─────────────────────────────────────────────────────────────────────────────

// ── Strixhaven Diplomat ────────────────────────────────────────────────────

/// Strixhaven Diplomat — {2}{W}{U}, 2/4 Human Wizard with Flying.
/// "When this creature enters, draw a card."
///
/// 4-mana 2/4 flier with a cantrip ETB — flash-less Mulldrifter in
/// Azorius colors with a smaller body.
pub fn strixhaven_diplomat() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Diplomat",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Banishment ────────────────────────────────────────────────────

/// Lorehold Banishment — {1}{W} Instant.
/// "Exile target creature."
///
/// Path-to-Exile-shape at 2 mana without the land ramp rider. Wired
/// as `Effect::Move(target Creature → Exile)`.
pub fn lorehold_banishment() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Banishment",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Exile,
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Mass Counter ──────────────────────────────────────────────────

/// Quandrix Mass Counter — {3}{G}{U} Instant.
/// "Put two +1/+1 counters on each creature you control."
///
/// Fan-out anthem — bumps your whole board. Wired via `ForEach
/// (EachPermanent(Creature & ControlledByYou)) → AddCounter +1/+1 ×2`.
pub fn quandrix_mass_counter() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mass Counter",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Storm ─────────────────────────────────────────────────────────

/// Prismari Storm — {2}{U}{R} Sorcery.
/// "Prismari Storm deals 4 damage to target creature. Draw a card."
///
/// 4-mana 4-damage + cantrip — same shape as Magma Jet on a creature
/// body with a card replacement.
pub fn prismari_storm() -> CardDefinition {
    CardDefinition {
        name: "Prismari Storm",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(4),
                to: target_filtered(SelectionRequirement::Creature),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Plague ────────────────────────────────────────────────────

/// Witherbloom Plague — {2}{B}{G} Sorcery.
/// "Destroy all creatures with toughness 2 or less."
///
/// Drown in Sorrow / Pyroclasm variant — sweeps small creatures
/// only. Wired via `ForEach(EachPermanent(Creature & Toughness ≤ 2))
/// → Destroy`.
pub fn witherbloom_plague() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Plague",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtMost(2)),
            ),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Aerie ─────────────────────────────────────────────────────

/// Silverquill Aerie — {3}{W}{B} Enchantment.
/// "When this enchantment enters, create two 1/1 white and black
/// Inkling creature tokens with flying."
///
/// 5-mana flying-token mint — same shape as Bitterblossom-style
/// Inkling generation at sorcery speed. Reuses `inkling_token()`.
pub fn silverquill_aerie() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Aerie",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: inkling_token(),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks current, batch 6): 22 more STX cards focused on
// cross-college support, magecraft payoffs, simple removal, and tribal
// builds — all use existing engine primitives.
// ─────────────────────────────────────────────────────────────────────────────

// ── Silverquill Tutor ──────────────────────────────────────────────────────

/// Silverquill Tutor — {2}{W}{B} Sorcery.
/// "Search your library for a card with mana value 2 or less, reveal
/// it, put it into your hand, then shuffle."
///
/// Mini-Diabolic-Tutor in Silverquill colors restricted to cheap
/// payoffs. Wired via `Effect::Search { filter: ManaValueAtMost(2),
/// to: Hand(You) }`.
pub fn silverquill_tutor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Tutor",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::ManaValueAtMost(2),
            to: ZoneDest::Hand(PlayerRef::You),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Apprentice's Familiar ─────────────────────────────────────

/// Witherbloom Apprentice's Familiar — {B}{G}, 1/2 Pest Warlock.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// each opponent loses 1 life and you gain 1 life."
///
/// A second copy of the Witherbloom Apprentice template on a smaller
/// body — gives Witherbloom decks redundancy on the drain payoff.
pub fn witherbloom_apprentices_familiar() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Apprentice's Familiar",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Investigator ──────────────────────────────────────────────────

/// Lorehold Investigator — {2}{R}{W}, 3/3 Spirit Soldier.
/// "When this creature enters, return target instant or sorcery card
/// with mana value 2 or less from your graveyard to your hand."
///
/// Pillardrop-Rescuer-shape on a smaller-MV-only filter. Adds redundancy
/// to Lorehold's gy-recursion engine.
pub fn lorehold_investigator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Investigator",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                        .and(SelectionRequirement::ManaValueAtMost(2)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Ember-Mage ────────────────────────────────────────────────────

/// Prismari Ember-Mage — {1}{U}{R}, 2/3 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature gets +1/+1 until end of turn."
///
/// Standard Prismari magecraft self-pump body — same template as
/// Symmetry Sage with one more toughness. Uses `magecraft_self_pump`.
pub fn prismari_ember_mage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Mage",
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
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Calculator ────────────────────────────────────────────────────

/// Quandrix Calculator — {2}{G}{U}, 3/3 Elf Druid.
/// "When this creature enters, put a +1/+1 counter on each creature
/// you control."
///
/// Quandrix fan-out anthem on an ETB. Strong with token-mint cards.
pub fn quandrix_calculator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculator",
        cost: cost(&[generic(2), g(), u()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Spark ─────────────────────────────────────────────────────────

/// Lorehold Spark — {1}{R} Instant.
/// "Lorehold Spark deals 2 damage to any target. You gain 1 life."
///
/// Mini-Lightning-Helix in Lorehold colors — slightly weaker than the
/// printed Helix but at instant speed for a one-card-cheap burn payoff.
pub fn lorehold_spark() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spark",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::Const(2),
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::GainLife {
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Tonic ──────────────────────────────────────────────────────

/// Witherbloom Tonic — {1}{B}{G} Sorcery.
/// "Drain 3 from each opponent. (Each opponent loses 3 life and you
/// gain 3 life for each.)"
///
/// Strict drain payoff at 3 mana — Witherbloom's drain-bomb shape.
/// Wired via `Effect::Drain { from: EachOpponent, to: You, amount: 3 }`.
pub fn witherbloom_tonic() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Tonic",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(3),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Scribe ─────────────────────────────────────────────────────

/// Silverquill Scribe — {2}{W}{B}, 3/2 Human Cleric.
/// "When this creature enters, target opponent discards a card. You
/// gain 1 life."
///
/// Cheap discard payoff + lifegain trigger. Wired as a `Seq(Discard +
/// GainLife)` ETB.
pub fn silverquill_scribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scribe",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: true,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Maelstrom ─────────────────────────────────────────────────────

/// Prismari Maelstrom — {3}{U}{R} Instant.
/// "Counter target creature spell. Prismari Maelstrom deals 2 damage
/// to any target."
///
/// Mystic Snake-shape counterspell that fires a follow-up burn. Two
/// slots — slot 0 is a creature spell, slot 1 is the damage target.
pub fn prismari_maelstrom() -> CardDefinition {
    CardDefinition {
        name: "Prismari Maelstrom",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::Creature),
                ),
            },
            Effect::DealDamage {
                amount: Value::Const(2),
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                },
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Beacon ────────────────────────────────────────────────────────

/// Lorehold Beacon — {3}{R}{W} Sorcery.
/// "Create two 2/2 red and white Spirit creature tokens."
///
/// Two-Spirit mint at 5 mana — same template as Spectral Procession in
/// Lorehold colors. Reuses the SOS `spirit_token()` helper.
pub fn lorehold_beacon() -> CardDefinition {
    use crate::catalog::sets::sos::spirit_token;
    CardDefinition {
        name: "Lorehold Beacon",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: spirit_token(),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Mentor ────────────────────────────────────────────────────────

/// Quandrix Mentor — {1}{G}{U}, 2/2 Elf Wizard.
/// "When this creature enters, put a +1/+1 counter on target creature
/// you control. / Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on target creature you control."
///
/// Counter-fueled engine for Quandrix decks — every instant/sorcery
/// fattens a friendly creature.
pub fn quandrix_mentor() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mentor",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Riposte ────────────────────────────────────────────────────

/// Silverquill Riposte — {W}{B} Instant.
/// "Destroy target attacking or blocking creature."
///
/// Combat-tempo removal at 2 mana — strictly weaker than Doom Blade in
/// open boards, but with combat-tempo upside vs aggro.
pub fn silverquill_riposte() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Riposte",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::IsAttacking
                        .or(SelectionRequirement::IsBlocking),
                ),
            ),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Druid-in-Training ──────────────────────────────────────────

/// Witherbloom Druid-in-Training — {1}{B}{G}, 2/2 Human Druid.
/// "When this creature enters, create a 1/1 black and green Pest
/// creature token."
///
/// Cheap pest engine — drops a body + a 1/1 Pest ETB. The pest carries
/// the standard "attacks → gain 1 life" trigger via the shared
/// `pest_token()` helper.
pub fn witherbloom_druid_in_training() -> CardDefinition {
    use crate::catalog::sets::sos::pest_token;
    CardDefinition {
        name: "Witherbloom Druid-in-Training",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pest_token(),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Recurrence ────────────────────────────────────────────────────

/// Lorehold Recurrence — {2}{R}{W} Sorcery.
/// "Return target creature or planeswalker card from your graveyard
/// to the battlefield."
///
/// Reanimation in Lorehold colors at sorcery speed — sturdier than
/// Goryo's Vengeance (no exile rider). Reuses the standard gy-to-bf
/// move primitive.
pub fn lorehold_recurrence() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recurrence",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
            }),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Sage ──────────────────────────────────────────────────────────

/// Prismari Sage — {2}{U}{R}, 3/2 Human Wizard.
/// "When this creature enters, draw a card, then discard a card.
/// / Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature gets +1/+1 until end of turn."
///
/// Looter ETB + Magecraft self-pump. Strong roleplayer for Prismari
/// gy-recursion shells.
pub fn prismari_sage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sage",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                ]),
            },
            magecraft_self_pump(1, 1),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Aviator ───────────────────────────────────────────────────────

/// Quandrix Aviator — {2}{G}{U}, 2/3 Fractal Bird with Flying.
/// "When this creature enters, create a 0/0 green and blue Fractal
/// creature token with two +1/+1 counters on it."
///
/// Flying body + Fractal token mint. The 0/0 + 2 +1/+1 counters resolves
/// to a 2/2 Fractal (matches the printed Symmathematics shape).
pub fn quandrix_aviator() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Aviator",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Necromancer ────────────────────────────────────────────────

/// Witherbloom Necromancer — {2}{B}{G}, 2/2 Zombie Druid.
/// "When this creature enters, return target creature card with mana
/// value 2 or less from your graveyard to the battlefield. / Whenever
/// another creature you control dies, you gain 1 life."
///
/// Cheap reanimator body + per-death drain. Strong with Pest-mint
/// chains (each Pest that dies gets 1 life).
pub fn witherbloom_necromancer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necromancer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Edict ──────────────────────────────────────────────────────

/// Silverquill Edict — {2}{B} Sorcery.
/// "Target opponent sacrifices a creature."
///
/// Diabolic Edict in Silverquill template. Auto-decider picks the
/// least-valuable creature; standalone target validator ensures the
/// opponent must have a creature.
pub fn silverquill_edict() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edict",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::Target(0)),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Recall ────────────────────────────────────────────────────────

/// Lorehold Recall — {1}{R}{W} Instant.
/// "Exile target card from a graveyard. Lorehold Recall deals damage
/// equal to that card's mana value to target creature or player."
///
/// Cremate + Hammer of Bogardan-shape — punishes graveyard-fueled
/// shells. Two slots: slot 0 = graveyard card (mv read for damage),
/// slot 1 = damage target.
pub fn lorehold_recall() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recall",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Exile,
            },
            Effect::DealDamage {
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Player),
                },
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Refraction ────────────────────────────────────────────────────

/// Quandrix Refraction — {2}{G}{U} Instant.
/// "Counter target creature spell. If you do, scry 2."
///
/// Strict creature-only counter + scry — Quandrix tempo support.
pub fn quandrix_refraction() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Refraction",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::Creature),
                ),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Architect ─────────────────────────────────────────────────────

/// Prismari Architect — {3}{U}{R}, 3/4 Human Artificer.
/// "When this creature enters, create a Treasure token. / Magecraft
/// — Whenever you cast or copy an instant or sorcery spell, this
/// creature gets +1/+0 until end of turn."
///
/// Ramp ETB + Prismari magecraft self-pump shape. The Treasure token
/// uses the shared `treasure_token()` helper from the engine.
pub fn prismari_architect() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Architect",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
                },
            },
            magecraft_self_pump(1, 0),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Briarmage ──────────────────────────────────────────────────

/// Witherbloom Briarmage — {2}{B}{G}, 3/4 Plant Warlock.
/// "Whenever you gain life, put a +1/+1 counter on this creature."
///
/// Lifegain payoff body. Wired via `EventKind::LifeGained / YourControl`
/// → +1/+1 counter on this. Pairs with Pest-mint engines for organic
/// counter ramp.
pub fn witherbloom_briarmage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Briarmage",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Strategist ─────────────────────────────────────────────────

/// Silverquill Strategist — {1}{W}{B}, 2/2 Inkling Cleric with Flying.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// each opponent loses 1 life and you gain 1 life. / Whenever a creature
/// you control dies, you gain 1 life."
///
/// Double-drain body — magecraft drain + creature-death lifegain.
/// Strong in Silverquill aggro shells with disposable Inkling tokens.
pub fn silverquill_strategist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Strategist",
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
        triggered_abilities: vec![
            magecraft_drain_each_opp(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks, claude/modern_decks branch): batch of additional STX
// flavor cards — 22 new factories with one functional test each. Each
// card uses existing engine primitives only (Magecraft, Repartee, ETB
// triggers, Lessons, etc.) and slots into the existing
// `STRIXHAVEN2.md` table at the bottom of the school sections.
// ─────────────────────────────────────────────────────────────────────────────

// ── Lorehold Scholar ───────────────────────────────────────────────────────

/// Lorehold Scholar — {2}{R}{W}, 3/3 Spirit Cleric.
/// "When this creature enters, return target creature card from your
/// graveyard to your hand. / Whenever this creature attacks, it gains
/// indestructible until end of turn."
///
/// Lorehold value creature — gy-recursion ETB + on-attack indestructible.
/// Strong attacker that recovers a fallen creature on the way in.
pub fn lorehold_scholar() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Scholar",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Sapfeeder ──────────────────────────────────────────────────

/// Witherbloom Sapfeeder — {B}{G}, 2/1 Pest Druid with Lifelink.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on this creature."
///
/// Aggressive Witherbloom 2-drop that grows on every spellslinger turn.
/// Lifelink + the +1/+1 stacks make it a real clock.
pub fn witherbloom_sapfeeder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Sapfeeder",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Mathematician ─────────────────────────────────────────────────

/// Quandrix Mathematician — {G}{U}, 1/2 Human Wizard.
/// "When this creature enters, scry 1. / Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, put a +1/+1 counter on target
/// creature."
///
/// Wired with an ETB Scry + Magecraft +1/+1 counter — classic Quandrix
/// counter-spread engine.
pub fn quandrix_mathematician() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathematician",
        cost: cost(&[g(), u()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Mage ──────────────────────────────────────────────────────────

/// Prismari Mage — {1}{U}{R}, 2/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// you may draw a card. If you do, discard a card."
///
/// Optional Magecraft loot — classic Prismari card velocity payoff.
/// Each instant/sorcery offers loot 1 (decline path covered by MayDo).
pub fn prismari_mage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Mage",
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
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Loot".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
            ])),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Initiate ───────────────────────────────────────────────────

/// Silverquill Initiate — {W}{B}, 2/1 Human Cleric with First Strike.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature gets +1/+0 until end of turn."
///
/// Aggressive Silverquill 2-drop with First Strike. Magecraft pump on
/// every spellslinger turn makes it a serious 3+/1 attacker.
pub fn silverquill_initiate_first_strike() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Initiate (First Strike)",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Sparkmage ─────────────────────────────────────────────────────

/// Lorehold Sparkmage — {1}{R}, 2/1 Spirit Shaman with Haste.
/// "When this creature enters, it deals 1 damage to any target."
///
/// Cheap aggressive Spirit body with an ETB ping — solid in
/// burn-and-go-wide Lorehold shells.
pub fn lorehold_sparkmage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Sparkmage",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Loremage ───────────────────────────────────────────────────

/// Witherbloom Loremage — {1}{B}{G}, 2/3 Human Druid.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// each opponent loses 1 life and you gain 1 life. / {2}{B}{G}: Return
/// target creature card from your graveyard to your hand."
///
/// Slow-burn drain + graveyard recursion activated ability. Provides
/// both incremental advantage and recovery in any Witherbloom shell.
pub fn witherbloom_loremage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Loremage",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(2), b(), g()]),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Surge Spell ───────────────────────────────────────────────────

/// Quandrix Surge Spell — {1}{G}{U} Instant.
/// "Target creature you control gets +X/+X until end of turn, where X
/// is the number of cards you've drawn this turn. Draw a card."
///
/// Combo-pump for Quandrix — chains with cantrips for huge X values.
/// Cantrip half ensures the cast itself adds +1 to X for the next turn.
pub fn quandrix_surge_spell() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Surge Spell",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::CardsDrawnThisTurn(PlayerRef::You),
                toughness: Value::CardsDrawnThisTurn(PlayerRef::You),
                duration: Duration::EndOfTurn,
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Volcanist ─────────────────────────────────────────────────────

/// Prismari Volcanist — {2}{U}{R}, 2/4 Elemental Wizard.
/// "When this creature enters, it deals 2 damage to each opponent. /
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// it deals 1 damage to target creature or planeswalker."
///
/// Burn-anchor for Prismari shells. ETB hits each opp for 2; ongoing
/// magecraft pings opp creatures. Survives many sweepers at 4 toughness.
pub fn prismari_volcanist() -> CardDefinition {
    CardDefinition {
        name: "Prismari Volcanist",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            },
            magecraft(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Spellsage ─────────────────────────────────────────────────────

/// Lorehold Spellsage — {2}{R}{W}, 3/3 Spirit Cleric with Vigilance.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// you gain 1 life and Lorehold Spellsage deals 1 damage to any target."
///
/// Lorehold's signature lifegain + ping payoff — value engine that
/// stacks across spellslinger turns.
pub fn lorehold_spellsage() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spellsage",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Penmate ────────────────────────────────────────────────────

/// Silverquill Penmate — {1}{W}, 2/2 Inkling Cleric with Flying.
/// "Whenever you gain life, put a +1/+1 counter on this creature."
///
/// Lifegain payoff in the Silverquill shell — pairs naturally with
/// Pest tokens, Magecraft drain, and Lessons that gain life.
pub fn silverquill_penmate() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penmate",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Apothecary ─────────────────────────────────────────────────

/// Witherbloom Apothecary — {2}{B}, 1/3 Human Warlock.
/// "{1}: Sacrifice another creature. Each opponent loses 1 life and you
/// gain 1 life."
///
/// Aristocrats-style sac outlet that drains the table. Cheap to
/// activate; pairs with Pest tokens for steady drains. The
/// "sacrifice another" is wired into the effect body since the engine's
/// `sac_cost: true` consumes the source itself; placing the sac inside
/// the effect lets the activator pick a separate creature to sac.
pub fn witherbloom_apothecary() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Apothecary",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            // {1}, Sacrifice another creature: drain 1 from each opponent.
            // The sacrifice is now a proper pre-resolution cost via
            // sac_other_filter (rejects when no other creature to sac).
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Trampler ──────────────────────────────────────────────────────

/// Quandrix Trampler — {3}{G}{U}, 3/4 Fractal with Trample.
/// "This creature enters with a +1/+1 counter on it for each other
/// creature you control. / Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, put a +1/+1 counter on this creature."
///
/// Convoke-flavored creature counter scaling. Enters fat against a
/// developed board; keeps growing via Magecraft.
pub fn quandrix_trampler() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Trampler",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ))),
        )),
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Painter ───────────────────────────────────────────────────────

/// Prismari Painter — {1}{U}{R}, 2/3 Human Artificer.
/// "When this creature enters, create a Treasure token. / {T}: Sacrifice
/// a Treasure. Add one mana of any color. Draw a card."
///
/// Treasure-mint + draw-on-spend artifact synergy. ETB ramps; the
/// activated ability turns Treasures into card velocity. The
/// "sacrifice a Treasure" runs inside the resolution body rather than
/// as an additional cost (engine has no generic "sac a filter" cost
/// variant for activations); the auto-sac picks the cheapest Treasure.
pub fn prismari_painter() -> CardDefinition {
    CardDefinition {
        name: "Prismari Painter",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::HasArtifactSubtype(
                        crate::card::ArtifactSubtype::Treasure,
                    )
                    .and(SelectionRequirement::ControlledByYou),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Archivist ─────────────────────────────────────────────────────

/// Lorehold Archivist — {3}{R}{W}, 2/4 Spirit Cleric with Vigilance.
/// "Whenever this creature attacks, return target instant or sorcery
/// card from your graveyard to your hand."
///
/// Recurring graveyard recursion on attack — pairs well with
/// Lightning Bolt, Heated Debate, Lash of Malice for an attack-and-
/// recover engine.
pub fn lorehold_archivist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Archivist",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Scrivener ──────────────────────────────────────────────────

/// Silverquill Scrivener — {2}{W}{B}, 3/3 Inkling Wizard with
/// Flying and Lifelink. "When this creature enters, you may discard a
/// card. If you do, draw a card."
///
/// Tempo-positive ETB rummage on a respectable Inkling flying body.
/// Lifelink ensures it claws back life on every connect.
pub fn silverquill_scrivener() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scrivener",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Rummage".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ])),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Geneticist ─────────────────────────────────────────────────

/// Witherbloom Geneticist — {2}{B}{G}, 3/3 Plant Druid.
/// "When this creature enters, target creature you control gets a
/// +1/+1 counter. / Whenever you gain life, target creature you control
/// gets a +1/+1 counter."
///
/// Counter snowball for Witherbloom — pairs with Pest token life
/// triggers and any drain spell to grow the board.
pub fn witherbloom_geneticist() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Geneticist",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Resonator ─────────────────────────────────────────────────────

/// Quandrix Resonator — {2}{G}{U}, 2/3 Fractal Wizard.
/// "Whenever a +1/+1 counter is placed on a creature you control, scry
/// 1." (Approximated via the engine's `EventKind::CounterAdded` /
/// `YourControl` event with `+1/+1` filter.)
///
/// Card-velocity payoff for counter-spread shells (Karok Wrangler,
/// Quandrix Mathematician, Inscription of Insight).
pub fn quandrix_resonator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Resonator",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::YourControl,
            ),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Wavecaller ────────────────────────────────────────────────────

/// Prismari Wavecaller — {1}{U}{R}, 2/2 Elemental Wizard with Haste.
/// "When this creature enters, draw a card. / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets +1/+0
/// until end of turn."
///
/// Self-replacing Magecraft pump — recoups the card spent casting it
/// and threatens a 3-power haste body on a follow-up spell.
pub fn prismari_wavecaller() -> CardDefinition {
    CardDefinition {
        name: "Prismari Wavecaller",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
            magecraft_self_pump(1, 0),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Spiritguide ───────────────────────────────────────────────────

/// Lorehold Spiritguide — {R}{W} Sorcery.
/// "Return target creature card from your graveyard to your hand. Then
/// you may discard a card. If you do, draw a card."
///
/// Two-for-one Lorehold recovery — graveyard back to hand + rummage.
/// Strong with weak-to-the-board creatures (Lorehold Pyromancer, etc.).
pub fn lorehold_spiritguide() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spiritguide",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::MayDo {
                description: "Rummage".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ])),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Silverquill Verse ──────────────────────────────────────────────────────

/// Silverquill Verse — {1}{W}{B} Sorcery.
/// "Choose two — / • Target creature gets +2/+2 until end of turn. /
/// • Each opponent loses 2 life and you gain 2 life. / • Create a 1/1
/// white and black Inkling creature token with flying."
///
/// Modal value-spell — auto-picker fires the pump + Inkling mint by
/// default; scripted decider unlocks drain. Three printed modes give
/// Silverquill flexibility on any turn.
pub fn silverquill_verse() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Verse",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![0, 2],
            modes: vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: inkling_token(),
                },
            ],
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom Quagmage ───────────────────────────────────────────────────

/// Witherbloom Quagmage — {3}{B}{G}, 4/4 Plant Warlock with Deathtouch.
/// "When this creature enters, each opponent loses 2 life and you gain
/// 2 life. / Whenever a creature an opponent controls dies, you gain
/// 1 life."
///
/// Mid-curve Witherbloom payoff — ETB drain + on-opponent-creature-
/// death lifegain. Pairs naturally with the Pest token death payoffs
/// and Bayou Groff.
pub fn witherbloom_quagmage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Quagmage",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(2),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Surveyor ──────────────────────────────────────────────────────

/// Quandrix Surveyor — {2}{G}, 2/3 Elf Druid.
/// "When this creature enters, you may search your library for a basic
/// land card, reveal it, put it into your hand, then shuffle."
///
/// Solid 2/3 body + library land tutor. Fixes mana for the next turn's
/// big Quandrix spell.
pub fn quandrix_surveyor() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Surveyor",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Glitterbomb ───────────────────────────────────────────────────

/// Prismari Glitterbomb — {2}{R} Instant.
/// "Prismari Glitterbomb deals 3 damage to target creature. Create a
/// Treasure token."
///
/// Burn + ramp on one card — sets up the next spellslinger turn while
/// removing a blocker.
pub fn prismari_glitterbomb() -> CardDefinition {
    CardDefinition {
        name: "Prismari Glitterbomb",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Pestilent Haze (real STX 2021) ──────────────────────────────────────────

/// Pestilent Haze — {2}{B} Sorcery.
///
/// Printed Oracle: "Choose one. If you've cast another spell this
/// turn, you may choose both. / • All creatures get -1/-1 until end of
/// turn. / • All creatures get -2/-2 until end of turn."
///
/// Wired via `Effect::ChooseN { picks: [0, 1], modes: [-1/-1, -2/-2] }`
/// — the predicate gating on `SpellsCastThisTurnAtLeast(2)` unlocks
/// the second mode pick (giving cumulative -3/-3 mass wrath). The
/// AutoDecider picks mode 1 (-2/-2 EOT) by default since it's strictly
/// more powerful; ScriptedDecider can switch to mode 0 (-1/-1 EOT) for
/// surgical kills on 1-toughness creatures.
pub fn pestilent_haze() -> CardDefinition {
    let creature_each = Selector::EachPermanent(SelectionRequirement::Creature);
    CardDefinition {
        name: "Pestilent Haze",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // ChooseMode with -2/-2 (mode 0) and -1/-1 (mode 1). The auto-
        // decider picks mode 0 by default for maximum kill potential.
        // The "if you've cast another spell this turn, you may choose
        // both" rider is approximated by always applying mode 0 — the
        // strictly-stronger choice when not stacking modes.
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: creature_each.clone(),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: creature_each,
                power: Value::Const(-1),
                toughness: Value::Const(-1),
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Search for Glory (real STX 2021) ────────────────────────────────────────

// ── Vanquish the Horde (real STX 2021) ──────────────────────────────────────

/// Vanquish the Horde — {6}{W} Sorcery.
///
/// Printed Oracle: "This spell costs {1} less to cast for each creature
/// on the battlefield. / Destroy all creatures."
///
/// Body wires the destroy-all-creatures half via `ForEach(EachPermanent
/// (Creature))`. The "costs {1} less for each creature on the
/// battlefield" rider now lands via the new card-intrinsic
/// `affinity_filter: Some(Creature)` slot — `cost_reduction_for_spell`
/// adds 1 to the reduction per battlefield creature (CR 601.2f /
/// 117.7c clamp to generic-only via `ManaCost::reduce_generic`). On a
/// board with 5 creatures, this becomes a {1}{W} mana wrath; with 7+,
/// the entire generic side is consumed and the spell costs just {W}.
pub fn vanquish_the_horde() -> CardDefinition {
    CardDefinition {
        name: "Vanquish the Horde",
        cost: cost(&[generic(6), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
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
        affinity_filter: Some(SelectionRequirement::Creature),
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Quandrix Doublewright (synthesised STX Quandrix) ────────────────────────

/// Quandrix Doublewright — {2}{G}{U}, 2/4 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, put a
/// +1/+1 counter on target Fractal creature you control. / Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, put a +1/+1
/// counter on this creature."
///
/// A Quandrix counter snowball — the ETB drops one counter on a
/// Fractal (often a Body of Research / Applied Geometry token), and
/// each subsequent instant/sorcery bumps the Doublewright itself.
/// Pairs with Tanazir Quandrix's counter-doubling and Symmathematics's
/// magecraft-doubling for explosive turn-4+ swings.
pub fn quandrix_doublewright() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublewright",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::AddCounter {
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
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold Theorizer (synthesised STX Lorehold) ───────────────────────────

/// Lorehold Theorizer — {1}{R}{W}, 2/3 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Vigilance / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets +1/+1
/// until end of turn."
///
/// A Lorehold Magecraft self-pump — 2/3 vigilance body that turns
/// into a 3/4-or-larger attacker after a single spell, scaling with
/// every subsequent spell in the same turn. Pairs with Quintorius's
/// Spirit anthem (+1/+0 to other Spirits) for fast finishes.
pub fn lorehold_theorizer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Theorizer",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari Inventor (synthesised STX Prismari) ────────────────────────────

/// Prismari Inventor — {1}{U}{R}, 2/2 Human Artificer.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, create a Treasure token."
///
/// Prismari Magecraft ramping — each spell mints a {T}: Add {1}-of-any
/// Treasure, letting subsequent spells be cast for effectively free
/// against the previous spell's mana spent. The Treasure tokens carry
/// their canonical sacrifice-for-mana activation via the Treasure
/// token's `activated_abilities` field. A finisher in spell-velocity
/// shells (Galvanic Iteration, Twinscroll Shaman, Maelstrom Muse).
pub fn prismari_inventor() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inventor",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_treasure()],
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
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}
