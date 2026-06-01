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

// ── Lorehold Mentor ────────────────────────────────────────────────────────

/// Lorehold Mentor — {3}{R}{W}, 3/3 Spirit Cleric with Mentor.
/// "Mentor (Whenever this creature attacks, put a +1/+1 counter on
/// target attacking creature with lesser power.)"
///
/// Mentor wired via `Attacks/SelfSource` + `AddCounter` against
/// `Attacking & PowerLessThanSource`, so the "lesser power" check tracks
/// Lorehold Mentor's current power (CR 702.114).
pub fn lorehold_mentor() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Mentor",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::IsAttacking)
                        .and(SelectionRequirement::PowerLessThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Prismari Bauble ────────────────────────────────────────────────────────

/// Prismari Bauble — {0} Artifact. "When this artifact enters, scry
/// 1. / {1}, Sacrifice this artifact: Draw a card."
///
/// Zero-mana cantrip artifact with a Scry-on-ETB rider. Same template
/// as Mishra's Bauble in Prismari colors — pure card velocity at no
/// cost.
pub fn prismari_bauble() -> CardDefinition {
    CardDefinition {
        name: "Prismari Bauble",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        additional_cast_cost: vec![],
    }
}

// ── Inkling Aether-Smith ───────────────────────────────────────────────────

/// Inkling Aether-Smith — {2}{W}{B}, 2/3 Inkling Artificer with
/// Flying (synthesised STX Silverquill / Quandrix-adjacent flavor).
/// "When this creature enters, choose one — / • Create a 1/1 white
/// and black Inkling creature token with flying. / • Put a +1/+1
/// counter on target creature you control."
///
/// Modal ETB: token or counter. Auto-decider picks mode 0 (token)
/// for go-wide play patterns. Wired via `Effect::ChooseMode([token,
/// counter])`. Tests:
/// `inkling_aether_smith_is_a_four_mana_two_three_inkling_artificer`,
/// `inkling_aether_smith_etb_default_creates_token`.
pub fn inkling_aether_smith() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Aether-Smith",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: inkling_token(),
                },
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
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
        additional_cast_cost: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks current): 22 new STX cards focused on finishing the
// Silverquill / Witherbloom colleges with depth + cross-college support.
// Each card uses existing engine primitives.
// ─────────────────────────────────────────────────────────────────────────────

// ── Disciplined Duelist ────────────────────────────────────────────────────

/// Disciplined Duelist — {1}{W}, 2/1 Human Cleric with First Strike.
/// Vanilla aggressive Silverquill body — First Strike on a 2-mana 2/1
/// trades up cleanly against the typical 2/2 ground creature.
pub fn disciplined_duelist() -> CardDefinition {
    CardDefinition {
        name: "Disciplined Duelist",
        cost: cost(&[generic(1), w()]),
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
    }
}

// ── Eager Scribe ────────────────────────────────────────────────────────────

/// Eager Scribe — {W}, 1/1 Human Cleric.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, scry 1."
///
/// Silverquill 1-drop magecraft body that turns each IS cast into card
/// selection. Pairs with any spellslinger shell.
pub fn eager_scribe() -> CardDefinition {
    CardDefinition {
        name: "Eager Scribe",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
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
    }
}

// ── Silverquill Pen ────────────────────────────────────────────────────────

/// Silverquill Pen — {2} Artifact. "{2}{W}{B}, {T}: Each opponent
/// loses 2 life and you gain 2 life."
///
/// Repeatable Silverquill drain in artifact form. Plays well in any
/// W/B drain deck (Tenured Inkcaster, Promising Duskmage, etc.).
pub fn silverquill_pen() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), w(), b()]),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
    }
}

// ── Witherbloom Acolyte ────────────────────────────────────────────────────

/// Witherbloom Acolyte — {B}{G}, 2/1 Human Druid.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, you gain 1 life."
///
/// 2-mana B/G Magecraft body with pure lifegain payoff. Powers
/// Witherbloom's "gain life → grow Pests" subtheme (Blech / Old-Growth
/// Educator / Pestbrood Sloth) without any complicated rider.
pub fn witherbloom_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Acolyte",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::GainLife {
            who: Selector::You,
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
    }
}

// ── Witherbloom Toxicology ────────────────────────────────────────────────

/// Witherbloom Toxicology — {3}{B}{G} Sorcery.
/// "Destroy target creature. Create a 1/1 black and green Pest
/// creature token with 'When this creature dies, you gain 1 life.'"
///
/// Removal + Pest mint at the same cast. The Pest carries the
/// printed Witherbloom on-die-gain-1 rider via `TokenDefinition.
/// triggered_abilities` (SOS-VI). Plays as a 5-mana 2-for-1 in the
/// Witherbloom Pest deck.
pub fn witherbloom_toxicology() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Witherbloom Toxicology",
        cost: cost(&[generic(3), b(), g()]),
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pest,
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
    }
}

// ── Pest Brood Caller ───────────────────────────────────────────────────────

/// Pest Brood Caller — {2}{B}{G}, 2/2 Human Warlock.
/// "When this creature enters, create two 1/1 black and green Pest
/// creature tokens with 'When this creature dies, you gain 1 life.'"
///
/// ETB-mints-two-Pests. Each Pest carries the on-die lifegain rider
/// via `TokenDefinition.triggered_abilities`. Same shape as Pest
/// Summoning but with a 2/2 body attached.
pub fn pest_brood_caller() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Pest Brood Caller",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
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
                count: Value::Const(2),
                definition: pest,
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
        additional_cast_cost: vec![],
    }
}

// ── Inkling Caretaker ───────────────────────────────────────────────────────

/// Inkling Caretaker — {1}{W}{B}, 1/3 Inkling Cleric with Flying +
/// Lifelink.
/// Inkling-flavored Silverquill body — soaks up damage + grows the
/// life total. Slots into any Inkling tribal shell with Tenured
/// Inkcaster's +2/+2 anthem.
pub fn inkling_caretaker() -> CardDefinition {
    CardDefinition {
        name: "Inkling Caretaker",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
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
    }
}

// ── Silverquill Strike ──────────────────────────────────────────────────────

/// Silverquill Strike — {W}{B} Instant.
/// "Target opponent loses 3 life and you gain 3 life."
///
/// Classic Silverquill drain at instant speed — the Drain Life
/// template at 2 mana. Wired via `Effect::Drain` with `Target(0)`
/// as the source.
pub fn silverquill_strike() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Strike",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: target_filtered(
                SelectionRequirement::Player.and(SelectionRequirement::ControlledByOpponent),
            ),
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Reverie ────────────────────────────────────────────────────────

/// Lorehold Reverie — {R}{W} Sorcery.
/// "You gain 3 life. Lorehold Reverie deals 3 damage to target
/// opponent."
///
/// 2-mana Lorehold drain that hits exactly like a Lightning Helix
/// but redirected to player-only. Wired as a `Seq(GainLife 3 → You,
/// DealDamage 3 → target opp)`.
pub fn lorehold_reverie() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverie",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                amount: Value::Const(3),
                to: target_filtered(
                    SelectionRequirement::Player.and(SelectionRequirement::ControlledByOpponent),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Prismari Loot ───────────────────────────────────────────────────────────

/// Prismari Loot — {U}{R} Instant.
/// "Draw a card, then discard a card."
///
/// 2-mana Izzet rummage. The classic blue-red loot template, useful
/// for setting up graveyard for Lorehold/Witherbloom recursion or
/// for filtering toward your next finisher.
pub fn prismari_loot() -> CardDefinition {
    CardDefinition {
        name: "Prismari Loot",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
    }
}

// ── Quandrix Counterspell ───────────────────────────────────────────────────

/// Quandrix Counterspell — {G}{U}{U} Instant.
/// "Counter target spell. Put a +1/+1 counter on target creature you
/// control."
///
/// 3-mana hard counter + a body buff — Quandrix's growth payoff in a
/// reactive shell. Wired as `Seq(CounterSpell + AddCounter on
/// optional slot-1 friendly creature)`.
pub fn quandrix_counterspell() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterspell",
        cost: cost(&[g(), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
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
    }
}

// ── Spell Squelch ───────────────────────────────────────────────────────────

/// Spell Squelch — {2}{U} Instant.
/// "Counter target spell."
///
/// Cancel-shape at 3 mana. Wired with the existing
/// `Effect::CounterSpell` against `IsSpellOnStack`.
pub fn spell_squelch() -> CardDefinition {
    CardDefinition {
        name: "Spell Squelch",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
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
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Field-Worker ───────────────────────────────────────────────

/// Witherbloom Field-Worker — {1}{G}, 2/2 Human Druid.
/// "When this creature enters, you gain 2 life."
///
/// Classic Civic Wayfinder-style 2-mana body with a small lifegain
/// rider that turns on Old-Growth Educator's Infusion (LifeGained
/// this turn → +2/+2 counters) and Honor Troll's anthem.
pub fn witherbloom_field_worker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Field-Worker",
        cost: cost(&[generic(1), g()]),
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Wayfinder ──────────────────────────────────────────────────────

/// Lorehold Wayfinder — {2}{R}{W}, 3/3 Spirit Cleric.
/// "When this creature enters, mill 2."
///
/// Lorehold mill-and-attack body. Mills two cards, filling the
/// graveyard for Storm-Kiln Artist, Spirit Mascot, Garrison
/// Excavator. Reuses the engine's `Effect::Mill` primitive.
pub fn lorehold_wayfinder() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wayfinder",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
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
        additional_cast_cost: vec![],
    }
}

// ── Prismari Brilliance ─────────────────────────────────────────────────────

/// Prismari Brilliance — {U}{R} Sorcery.
/// "Scry 2. Draw a card."
///
/// 2-mana sorcery-speed card-selection. Wired as `Seq(Scry 2, Draw
/// 1)`. The "draw" is exactly Preordain's filter-and-draw template
/// in U/R colors.
pub fn prismari_brilliance() -> CardDefinition {
    CardDefinition {
        name: "Prismari Brilliance",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Tutor ──────────────────────────────────────────────────────────

/// Quandrix Tutor — {2}{G}{U} Sorcery.
/// "Search your library for a creature card, reveal it, put it into
/// your hand, then shuffle."
///
/// Eladamri's-Call-shape in Quandrix colors. Wired as a single
/// `Effect::Search` with a `Creature` filter and `Hand` destination.
pub fn quandrix_tutor() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tutor",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
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
        additional_cast_cost: vec![],
    }
}

// ── Silverquill Cantrip ─────────────────────────────────────────────────────

/// Silverquill Cantrip — {1}{W} Instant.
/// "You gain 2 life. Draw a card."
///
/// White cantrip with a small lifegain rider — same template as
/// Healing Salve + Brainstorm trimmed to a single draw. Plays in
/// any "gain-life-matters" shell to enable Comforting Counsel,
/// Light of Promise.
pub fn silverquill_cantrip() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cantrip",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Reanimator ─────────────────────────────────────────────────

/// Witherbloom Reanimator — {3}{B}{G}, 2/3 Human Warlock.
/// "When this creature enters, return target creature card from your
/// graveyard to your hand."
///
/// 5-mana reanimation-as-recursion — pulls a beater back to hand for
/// a re-cast, dodging exile-from-graveyard hate.
pub fn witherbloom_reanimator() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reanimator",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
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
                    filter: SelectionRequirement::Creature,
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Lightning ──────────────────────────────────────────────────────

/// Lorehold Lightning — {1}{R} Instant.
/// "Lorehold Lightning deals 3 damage to target creature."
///
/// Shock-curve 3-damage spell tuned for Strixhaven. Plain
/// `DealDamage 3 → Creature target` wire — the staple removal in
/// the catalog for taking down 3-toughness creatures.
pub fn lorehold_lightning() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Lightning",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            amount: Value::Const(3),
            to: target_filtered(SelectionRequirement::Creature),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Engineer ──────────────────────────────────────────────────────

/// Quandrix Engineer — {1}{G}{U}, 2/3 Elf Druid.
/// "{T}: Add {G} or {U}."
///
/// Quandrix mana dork that taps for either pip — same shape as Birds
/// of Paradise restricted to G/U. Bumps the curve to play Body of
/// Research / Tanazir Quandrix on turn 5.
pub fn quandrix_engineer() -> CardDefinition {
    use crate::catalog::sets::tap_add;
    CardDefinition {
        name: "Quandrix Engineer",
        cost: cost(&[generic(1), g(), u()]),
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
        activated_abilities: vec![tap_add(Color::Green), tap_add(Color::Blue)],
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
    }
}

// ── Prismari Pyromage ──────────────────────────────────────────────────────

/// Prismari Pyromage — {2}{R}, 2/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature deals 1 damage to any target."
///
/// Magecraft ping body — every IS cast doubles as a Shock-half. Uses
/// `target_filtered(Creature ∨ Player ∨ Planeswalker)` to keep the
/// ping flexible.
pub fn prismari_pyromage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyromage",
        cost: cost(&[generic(2), r()]),
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
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            amount: Value::Const(1),
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
    }
}

// ── Lorehold Curator ───────────────────────────────────────────────────────

/// Lorehold Curator — {2}{W}, 2/3 Spirit Soldier.
/// "When this creature enters, return target creature card with
/// mana value 2 or less from your graveyard to your hand."
///
/// Lorehold mid-game value body — pulls Inkling/Pest/Spirit fodder
/// back from graveyard. The MV ≤ 2 cap mirrors Sun Titan's
/// reach-down for cheap creatures in Lorehold colors.
pub fn lorehold_curator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Curator",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
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
                    filter: SelectionRequirement::Creature
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
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Scholar ────────────────────────────────────────────────────

/// Witherbloom Scholar — {1}{B}, 2/1 Human Warlock.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, target opponent loses 1 life and you gain 1 life."
///
/// Black-only Witherbloom Apprentice slot — same drain payoff at a
/// flat {1}{B}. Plays a smaller body but works in mono-B drains.
pub fn witherbloom_scholar() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Scholar",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
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
        additional_cast_cost: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks current, batch 2): 14 more STX cards focused on the
// Quandrix / Prismari / Lorehold colleges with a few cross-college staples.
// ─────────────────────────────────────────────────────────────────────────────

// ── Quandrix Apprenticeship ───────────────────────────────────────────────

/// Quandrix Apprenticeship — {1}{G}{U} Sorcery.
/// "Put two +1/+1 counters on target creature you control. Scry 1."
///
/// Two-pronged growth-with-selection that pairs with any +1/+1 counter
/// payoff (Quandrix Cultivator, Symmathematics, Dragonsguard Elite).
pub fn quandrix_apprenticeship() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Apprenticeship",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
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
                amount: Value::Const(2),
            },
            Effect::Scry {
                who: PlayerRef::You,
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
    }
}

// ── Prismari Pyrotechnics ──────────────────────────────────────────────────

/// Prismari Pyrotechnics — {3}{R}{R} Sorcery.
/// "Prismari Pyrotechnics deals 5 damage to target creature or
/// planeswalker."
///
/// 5-mana 5-damage burn — same template as Lava Coil but flexible
/// to creature OR planeswalker, no exile rider.
pub fn prismari_pyrotechnics() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyrotechnics",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            amount: Value::Const(5),
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Strategist ────────────────────────────────────────────────────

/// Lorehold Strategist — {2}{W}, 2/2 Spirit Cleric with Flying.
/// "When this creature enters, you gain 2 life."
///
/// Strixhaven flier-with-lifegain. Plays in any lifegain-matters
/// shell (Comforting Counsel, Light of Promise, Honor Troll).
pub fn lorehold_strategist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Strategist",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Necromancy ─────────────────────────────────────────────────

/// Witherbloom Necromancy — {2}{B}{B} Sorcery.
/// "Return target creature card from your graveyard to the
/// battlefield. You lose 2 life."
///
/// Black classic reanimation at 4 mana with a life cost — Animate
/// Dead's modern variant in Witherbloom. Pairs with Sproutback Trudge
/// for a 5/6 swing that nets 5 life back.
pub fn witherbloom_necromancy() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necromancy",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::LoseLife {
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
    }
}

// ── Silverquill Resolve ────────────────────────────────────────────────────

/// Silverquill Resolve — {1}{W} Instant.
/// "Target creature gets +1/+3 and gains lifelink until end of turn."
///
/// Defensive combat trick — pumps toughness for a 4/4-vs-blocker
/// turn and lifelinks the bonus damage back. Same template as
/// Veteran's Sidearm.
pub fn silverquill_resolve() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Resolve",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
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
        additional_cast_cost: vec![],
    }
}

// ── Prismari Conduit ───────────────────────────────────────────────────────

/// Prismari Conduit — {1}{R}, 2/2 Elemental.
/// "Haste. Whenever this creature attacks, you may discard a card.
/// If you do, draw a card."
///
/// Hasty 2/2 with a loot-on-attack rider for graveyard-fillery /
/// hand-fixing. Wired as `Attacks/SelfSource → MayDo(Discard 1 + Draw
/// 1)`.
pub fn prismari_conduit() -> CardDefinition {
    CardDefinition {
        name: "Prismari Conduit",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Loot".to_string(),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Doubling ──────────────────────────────────────────────────────

/// Quandrix Doubling — {3}{G}{U} Sorcery.
/// "Double the number of +1/+1 counters on target creature you
/// control."
///
/// Pure counter-doubling at sorcery speed. Equivalent shape to
/// Practical Research (which also doubles counters). Wired via
/// `AddCounter(amount = CountersOn(target, +1/+1))` so the counter
/// pool grows to 2N total exactly per CR 701.10e.
pub fn quandrix_doubling() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doubling",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CountersOn {
                what: Box::new(Selector::Target(0)),
                kind: CounterType::PlusOnePlusOne,
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Smith ─────────────────────────────────────────────────────────

/// Lorehold Smith — {2}{R}, 2/3 Dwarf Warrior.
/// "When this creature enters, create a Treasure token."
///
/// 3-mana 2/3 with a Treasure-on-ETB rider. Same shape as Tavern
/// Smasher's mini-version, in Lorehold flavor. Reuses the engine's
/// `treasure_token()` definition.
pub fn lorehold_smith() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Lorehold Smith",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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
        additional_cast_cost: vec![],
    }
}

// ── Silverquill Decree ─────────────────────────────────────────────────────

/// Silverquill Decree — {3}{W}{B} Instant.
/// "Destroy target creature or planeswalker. You gain 2 life."
///
/// 5-mana flexible removal at instant speed with a small lifegain
/// rider — Silverquill's premium removal answer.
pub fn silverquill_decree() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Decree",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
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
    }
}

// ── Witherbloom Wand ───────────────────────────────────────────────────────

/// Witherbloom Wand — {2} Artifact.
/// "{2}{B}{G}, {T}: Target player loses 2 life and you gain 2 life."
///
/// Repeatable Witherbloom drain in artifact form. Same shape as
/// Silverquill Pen but at {B}{G} pips for the Witherbloom shell.
pub fn witherbloom_wand() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Wand",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), b(), g()]),
            effect: Effect::Drain {
                from: target_filtered(
                    SelectionRequirement::Player.and(SelectionRequirement::ControlledByOpponent),
                ),
                to: Selector::You,
                amount: Value::Const(2),
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
    }
}

// ── Quandrix Survey ────────────────────────────────────────────────────────

/// Quandrix Survey — {2}{G}{U} Sorcery.
/// "Search your library for a land card, put it onto the battlefield
/// tapped, then shuffle. Draw a card."
///
/// Quandrix ramp-and-draw — Cultivate's body in 4 mana with a slimmer
/// land-into-play count. Reuses the engine's `Effect::Search` over
/// `IsBasicLand`-or-`Land` filter and the Draw primitive.
pub fn quandrix_survey() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Survey",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
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
    }
}

// ── Prismari Arsonist ──────────────────────────────────────────────────────

/// Prismari Arsonist — {2}{U}{R}, 3/2 Human Wizard with Flash.
/// "When this creature enters, this creature deals 2 damage to target
/// creature."
///
/// 4-mana flash 3/2 with a Flametongue Kavu-style ETB — drops at
/// the end of opp's turn to kill a 2-toughness creature + body up
/// for the next attack.
pub fn prismari_arsonist() -> CardDefinition {
    CardDefinition {
        name: "Prismari Arsonist",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                amount: Value::Const(2),
                to: target_filtered(SelectionRequirement::Creature),
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Banner ────────────────────────────────────────────────────────

/// Lorehold Banner — {3} Artifact.
/// "When this artifact enters, you gain 2 life. / {T}: Add {R} or
/// {W}."
///
/// Color-fixing artifact with a small lifegain ETB. Plays like a
/// Manalith / Coalition Banner in Lorehold colors.
pub fn lorehold_banner() -> CardDefinition {
    use crate::catalog::sets::tap_add;
    CardDefinition {
        name: "Lorehold Banner",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Red), tap_add(Color::White)],
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Verdict ───────────────────────────────────────────────────

/// Witherbloom Verdict — {2}{B} Sorcery.
/// "Target opponent sacrifices a creature."
///
/// Diabolic Edict / Cruel Edict template in Witherbloom colors. Wired
/// via the existing `Effect::Sacrifice` primitive aimed at an opp
/// player slot.
pub fn witherbloom_verdict() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Verdict",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Sacrifice {
            who: target_filtered(
                SelectionRequirement::Player.and(SelectionRequirement::ControlledByOpponent),
            ),
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
        additional_cast_cost: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks current, batch 3): 12 more STX cards — mono-color
// staples + a few more cross-college tools.
// ─────────────────────────────────────────────────────────────────────────────

// ── Strixhaven Footsoldier ─────────────────────────────────────────────────

/// Strixhaven Footsoldier — {W}, 1/2 Human Soldier with Vigilance.
/// Cheap white vigilance creature that swings + holds back to block.
pub fn strixhaven_footsoldier() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Footsoldier",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
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
    }
}

// ── Mage Tower Crystal ─────────────────────────────────────────────────────

/// Mage Tower Crystal — {2} Artifact.
/// "{T}: Add one mana of any color."
///
/// Manalith-style 3-mana rainbow rock at 3 mana — fixes colors for
/// 3-color college decks.
pub fn mage_tower_crystal() -> CardDefinition {
    use crate::catalog::sets::tap_add_any_color;
    CardDefinition {
        name: "Mage Tower Crystal",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add_any_color()],
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
    }
}

// ── Witherbloom Adept ──────────────────────────────────────────────────────

/// Witherbloom Adept — {2}{B}, 3/2 Human Warlock with Menace.
/// Aggressive mono-B Menace body — same shape as Tormented Hero
/// scaled up.
pub fn witherbloom_adept() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Adept",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
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
    }
}

// ── Lorehold Pyromancer ────────────────────────────────────────────────────

/// Lorehold Pyromancer — {2}{R}, 2/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature gets +2/+0 until end of turn."
///
/// Magecraft self-pump that turns each spell into +2 attack power.
pub fn lorehold_pyromancer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyromancer",
        cost: cost(&[generic(2), r()]),
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Defender ─────────────────────────────────────────────────────

/// Quandrix Defender — {1}{U}, 0/4 Wall with Defender + Flying.
/// "When this creature enters, scry 1."
///
/// Defender wall with flying that blocks anything in the air, plus
/// a Scry ETB for card selection.
pub fn quandrix_defender() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Defender",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        additional_cast_cost: vec![],
    }
}

// ── Silverquill Lifedrain ──────────────────────────────────────────────────

/// Silverquill Lifedrain — {1}{B} Sorcery.
/// "Each opponent loses 2 life. You gain 2 life."
///
/// Vampire's Bite / Sanguine Glorifier shape in mono-B. Pure drain
/// at 2 mana.
pub fn silverquill_lifedrain() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifedrain",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Plowman ────────────────────────────────────────────────────

/// Witherbloom Plowman — {3}{G}, 4/3 Human Druid with Reach.
/// "When this creature enters, you gain 3 life."
///
/// Reach beater that turns on Witherbloom's lifegain triggers.
pub fn witherbloom_plowman() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Plowman",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach],
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
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Prismari Spellfire-Sage ────────────────────────────────────────────────

/// Prismari Spellfire-Sage — {3}{U}{R}, 4/4 Human Wizard with Flash.
/// "When this creature enters, draw a card."
///
/// 5-mana flash 4/4 with a cantrip ETB — same shape as Mulldrifter
/// without the evoke alt-cost.
pub fn prismari_spellfire_sage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellfire-Sage",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flash],
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Justice ───────────────────────────────────────────────────────

/// Lorehold Justice — {2}{W} Instant.
/// "Destroy target creature with power 4 or greater."
///
/// White conditional removal — Stand Up for Yourself's mirror for
/// big creatures. Same shape as Crib Swap.
pub fn lorehold_justice() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Justice",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Recall ───────────────────────────────────────────────────────

/// Quandrix Recall — {1}{U} Instant.
/// "Return target creature to its owner's hand."
///
/// Unsummon-shape in Quandrix colors at 2 mana — tempo play to
/// reset a problematic opp body or replay your ETB creature.
pub fn quandrix_recall() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Recall",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Pestilence ─────────────────────────────────────────────────

/// Witherbloom Pestilence — {1}{B}{B} Sorcery.
/// "Each creature gets -2/-2 until end of turn."
///
/// Mass debuff that wipes 2-toughness boards (Pestilence-style
/// wrath). Wired via `ForEach(EachPermanent(Creature)) → PumpPT
/// (-2/-2 EOT)`.
pub fn witherbloom_pestilence() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestilence",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Combatant ─────────────────────────────────────────────────────

/// Lorehold Combatant — {1}{R}{W}, 2/2 Dwarf Soldier with Double Strike.
/// Aggressive R/W body with double strike — a 4-damage swinger when
/// pumped. Same template as Vexilus Praetor's body in Lorehold colors.
pub fn lorehold_combatant() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Combatant",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
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
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Push (modern_decks current, batch 4): 10 more STX cards — additional
// staples and a few impactful effects.
// ─────────────────────────────────────────────────────────────────────────────

// ── Owlin Tactician ────────────────────────────────────────────────────────

/// Owlin Tactician — {2}{W}, 2/3 Bird Soldier with Flying.
/// "When this creature enters, target creature gets +1/+1 and gains
/// flying until end of turn."
///
/// Flier with a flicker-like pump rider that grants another creature
/// evasion for a swing turn.
pub fn owlin_tactician() -> CardDefinition {
    CardDefinition {
        name: "Owlin Tactician",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
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
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
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
        additional_cast_cost: vec![],
    }
}

// ── Pest Mediator ──────────────────────────────────────────────────────────

/// Pest Mediator — {1}{B}{G}, 2/2 Pest Cleric.
/// "Whenever you gain life, put a +1/+1 counter on this creature."
///
/// Witherbloom lifegain payoff body — grows each time life is
/// gained. Wired against `EventKind::LifeGained, YourControl` for
/// the "you gain life" trigger.
pub fn pest_mediator() -> CardDefinition {
    CardDefinition {
        name: "Pest Mediator",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        additional_cast_cost: vec![],
    }
}

// ── Inkling Aerialist ──────────────────────────────────────────────────────

/// Inkling Aerialist — {2}{W}{B}, 2/2 Inkling Wizard with Flying.
/// "Whenever another Inkling enters under your control, this creature
/// gets +1/+1 until end of turn."
///
/// Inkling tribal payoff body. Wired against EntersBattlefield/
/// AnotherOfYours with a `Predicate::EntityMatches { what: TriggerSource,
/// filter: HasCreatureType(Inkling) }` gate.
pub fn inkling_aerialist() -> CardDefinition {
    CardDefinition {
        name: "Inkling Aerialist",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Inkling),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Theorist ──────────────────────────────────────────────────────

/// Quandrix Theorist — {3}{G}{U}, 3/3 Elf Wizard.
/// "When this creature enters, draw a card for each creature you
/// control with a +1/+1 counter on it."
///
/// Counter payoff — pulls value from a wide +1/+1 board. Wired via
/// `Draw { amount: CountOf(EachPermanent(Creature & ControlledByYou &
/// WithCounter(+1/+1))) }`.
pub fn quandrix_theorist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Theorist",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ))),
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
        additional_cast_cost: vec![],
    }
}

// ── Prismari Inferno ──────────────────────────────────────────────────────

/// Prismari Inferno — {4}{R}{R} Sorcery.
/// "Prismari Inferno deals 3 damage to each creature."
///
/// Pyroclasm at scale — sweeps 3-toughness boards for 6 mana. Wired
/// as `ForEach(EachPermanent(Creature)) → DealDamage 3`.
pub fn prismari_inferno() -> CardDefinition {
    CardDefinition {
        name: "Prismari Inferno",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                amount: Value::Const(3),
                to: Selector::TriggerSource,
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Resurgence ────────────────────────────────────────────────────

/// Lorehold Resurgence — {2}{R}{W} Sorcery.
/// "Return target creature card with mana value 3 or less from your
/// graveyard to the battlefield."
///
/// Sun Titan's reach-down in sorcery form. 4 mana, returns up to a
/// 3-MV creature.
pub fn lorehold_resurgence() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Resurgence",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
            ),
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
        additional_cast_cost: vec![],
    }
}

// ── Witherbloom Studies ────────────────────────────────────────────────────

/// Witherbloom Studies — {1}{B}{G} Sorcery.
/// "Mill 3 cards, then return target creature card from your
/// graveyard to your hand."
///
/// Self-mill into selective reanimation-to-hand. Same shape as
/// Stitcher's Supplier-style mill + Raise Dead.
pub fn witherbloom_studies() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Studies",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

// ── Silverquill Vanguard ───────────────────────────────────────────────────

/// Silverquill Vanguard — {3}{W}{B}, 4/3 Inkling Cleric with Flying.
/// "Other Inkling creatures you control get +1/+1."
///
/// Inkling tribal anthem at +1/+1 (Tenured Inkcaster's lesser sibling
/// — Inkcaster is +2/+2 for 4 mana, Vanguard is +1/+1 for 5 mana with
/// a flying 4/3 body). Wired via `tribal_anthem_for_name` helper-
/// table extension (same path as Tenured Inkcaster).
pub fn silverquill_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vanguard",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
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
        additional_cast_cost: vec![],
    }
}
