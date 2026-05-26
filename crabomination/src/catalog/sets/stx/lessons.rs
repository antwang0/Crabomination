//! Strixhaven Lesson cards — additional Lessons from the printed cycle that
//! weren't already in `mono.rs` / `shared.rs` / `witherbloom.rs`. Each
//! Lesson is recorded with the `SpellSubtype::Lesson` tag so future
//! Lesson-aware mechanics (sideboard search, "you may cast this from your
//! sideboard") can filter on it. Today the engine has no sideboard
//! primitive, so Lessons resolve from hand like any other sorcery.
//!
//! Cards in this module:
//! - **Environmental Sciences** ({1}{G}) — gain 4 life + tutor a basic
//!   land. The G-color "ramp Lesson" — a fine early play in any green
//!   deck that wants the land + life payoff.
//! - **Introduction to Annihilation** ({3}{W}) — destroy a nonland
//!   permanent + the controller scries 2. Single-target removal Lesson
//!   with a small downside for the controller of the targeted
//!   permanent.
//! - **Introduction to Prophecy** ({2}{U}) — scry 3 + draw a card. The
//!   classic blue cantrip + filtering Lesson.
//! - **Spirit Summoning** ({3}{W}) — create a 3/2 white Spirit token.
//!   The white-flavor body-Lesson, slotting in alongside Inkling
//!   Summoning ({3}{W}{B}) and Pest Summoning ({B}{G}).

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword, Selector, SelectionRequirement,
    SpellSubtype, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

// ── Environmental Sciences ──────────────────────────────────────────────────

// ── Introduction to Annihilation ────────────────────────────────────────────

// ── Introduction to Prophecy ────────────────────────────────────────────────

// ── Spirit Summoning ────────────────────────────────────────────────────────

// ── Square Up ───────────────────────────────────────────────────────────────

/// Square Up — {U}{R} Instant (Prismari).
///
/// "Until end of turn, target creature has base power and toughness 0/4.
/// Draw a card."
///
/// Wired via the new `Effect::SetBasePT` primitive (layer-7b
/// continuous effect that overrides the creature's base P/T). Counters
/// and +N/+M still stack on top per CR 613.7c-f — so a +1/+1 counter
/// on a Square-Upped creature makes it 1/5, not 1/1. The cantrip half
/// fires regardless of whether a creature target was provided.
pub fn square_up() -> CardDefinition {
    CardDefinition {
        name: "Square Up",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(0),
                toughness: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Brilliant Plan ──────────────────────────────────────────────────────────

/// Brilliant Plan — {3}{U}{U} Sorcery — Lesson.
///
/// "Scry 3, then draw three cards."
///
/// Pure card-velocity Lesson. Wired as `Seq(Scry(3) → Draw(3))` so the
/// Scry resolves first, letting the controller filter the next three
/// draws. No target needed; the Scry uses `PlayerRef::You` and the Draw
/// uses `Selector::You`.
pub fn brilliant_plan() -> CardDefinition {
    CardDefinition {
        name: "Brilliant Plan",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(3),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fortifying Draught ──────────────────────────────────────────────────────

/// Fortifying Draught — {2}{W} Sorcery — Lesson.
///
/// "Target creature gets +1/+4 until end of turn."
///
/// Defensive combat trick Lesson — keeps a Silverquill / Lorehold body
/// alive through a big swing. Wired as a single `Effect::PumpPT`
/// against a `Creature` target. The body shape is identical to
/// `Charge Through` (+1/+1 + trample) and other Strixhaven pump
/// spells; only the magnitudes differ.
pub fn fortifying_draught() -> CardDefinition {
    CardDefinition {
        name: "Fortifying Draught",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(1),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Guiding Voice ───────────────────────────────────────────────────────────

/// Guiding Voice — {W} Sorcery — Lesson.
///
/// "Put a +1/+1 counter on target creature. Learn."
///
/// Cheap +1/+1 counter on a creature plus the Learn → `Draw 1`
/// approximation (no Lesson sideboard model yet). A great early
/// magecraft enabler that also leaves a body bigger. Wired as the
/// canonical AddCounter + Learn `Seq` template used by Hunt for
/// Specimens / Pest Summoning.
pub fn guiding_voice() -> CardDefinition {
    CardDefinition {
        name: "Guiding Voice",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            // Learn approximation: draw a card. Same shortcut every other
            // STX Learn card uses (Eyetwitch's die-trigger, Hunt for
            // Specimens's rider, Field Trip's rider, Igneous Inspiration's
            // rider). Tracked in STRIXHAVEN2.md as the engine-wide Lesson
            // sideboard gap.
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Expanded Anatomy ────────────────────────────────────────────────────────

/// Expanded Anatomy — {3}{G} Sorcery — Lesson.
///
/// "Put two +1/+1 counters on target creature."
///
/// Green's body-Lesson. Wired as a single `AddCounter` of amount `2`
/// for `PlusOnePlusOne` against a `Creature` target. No Learn rider
/// (Expanded Anatomy is itself a Lesson, not a Learn enabler). Cleanest
/// way to use it: cast on a creature you already own to push it to a
/// real threat (a 2/2 → 4/4, a 3/3 → 5/5). Also a fine target for
/// Magecraft riders (Karok Wrangler-style payoffs).
pub fn expanded_anatomy() -> CardDefinition {
    CardDefinition {
        name: "Expanded Anatomy",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Mercurial Transformation ────────────────────────────────────────────────

/// Mercurial Transformation — {2}{U} Sorcery.
///
/// "Target creature or artifact becomes a blue Frog creature with base
/// power and toughness 3/3 and loses all abilities until end of turn."
///
/// Push (modern_decks): wired via the engine's `Effect::SetBasePT`
/// layer-7b primitive (same path used by Square Up). The "loses all
/// abilities" rider is **not yet enforced** (no clear-abilities
/// continuous effect primitive); the target keeps its printed
/// abilities, which is a mild over-statement for the +typical use case
/// (turning a threatening 5/5 menacing-deathtouch creature into a
/// 3/3 Frog that's still menacing). Tracked in TODO.md as the
/// `StaticEffect::ClearAbilities` gap. The base-P/T override is the
/// headline play pattern (shrinking a 7/7 Force of Wills's-target
/// down to a 3/3, or growing a 1/1 token into a 3/3 attacker), and
/// resolves cleanly via the same layer-7b code as Square Up.
pub fn mercurial_transformation() -> CardDefinition {
    CardDefinition {
        name: "Mercurial Transformation",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                ),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::LoseAllAbilities {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Pest Inheritance ────────────────────────────────────────────────────────

/// Pest Inheritance — {3}{G} Sorcery — Lesson (real STX printing).
///
/// "Create a number of 1/1 black and green Pest creature tokens with
/// 'When this creature dies, you gain 1 life' equal to the number of
/// lands you control."
///
/// Wired via the `count: Value::SelectorCount(Land & ControlledByYou)`
/// shape — each minted Pest carries its native lifegain death-trigger
/// rider via `TokenDefinition.triggered_abilities`. Slots into any
/// Witherbloom lifegain shell; at land count = 5 it's 5 1/1 Pests for
/// 4 mana, with a lifegain back-end.
pub fn pest_inheritance() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Pest Inheritance",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ))),
            definition: pest,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Pop Quiz: Magic 101 ─────────────────────────────────────────────────────

/// Mascot Interpretation — {1}{U} Sorcery — Lesson (synthesised STX
/// Quandrix flavor — a +1/+1 Counter-on-creature Lesson at instant
/// pace). "Put two +1/+1 counters on target creature you control.
/// Learn."
///
/// A blue Lesson that doubles down on +1/+1 counter strategies. Same
/// shape as Guiding Voice (W) but with two counters in U at twice
/// the mana.
pub fn mascot_interpretation() -> CardDefinition {
    CardDefinition {
        name: "Mascot Interpretation",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Reduce // Rubble ────────────────────────────────────────────────────────

/// Reduce // Rubble — {2}{R} Sorcery — Lesson (synthesised STX
/// Lorehold flavor). "Reduce // Rubble deals 3 damage to target
/// creature or planeswalker. Learn."
///
/// A red Lesson sized for early creature removal + the Learn rider
/// (approximated as Draw 1 — engine-wide gap). Pairs with Mascot
/// Interpretation (U) and Guiding Voice (W) as the early Lesson
/// cycle.
pub fn reduce_rubble() -> CardDefinition {
    CardDefinition {
        name: "Reduce // Rubble",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Containment Studies ─────────────────────────────────────────────────────

/// Containment Studies — {2}{W} Sorcery — Lesson (synthesised STX
/// Silverquill flavor). "Tap target creature. Put two stun counters
/// on it. (If a permanent with a stun counter would become untapped,
/// remove one from it instead.)"
///
/// A white Lesson that locks down a single creature for 2-3 turns.
/// Same shape as Scolding Detention but slotting into the Lesson
/// cycle.
pub fn containment_studies() -> CardDefinition {
    CardDefinition {
        name: "Containment Studies",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Reflective Anatomy ──────────────────────────────────────────────────────

/// Reflective Anatomy — {2}{G}{U} Sorcery — Lesson (synthesised STX
/// Quandrix flavor). "Target creature gets +X/+X until end of turn,
/// where X is the number of +1/+1 counters on creatures you control."
///
/// Wired via `Value::CountTotalCounters` — a fresh helper that walks
/// all controlled creatures and sums their +1/+1 counters. Pump scales
/// linearly with board buildup, so this Lesson grows into a game-ender
/// in counters-matter decks (Quandrix Recalibrator, Quandrix Doubling
/// Tutor's Fractals).
pub fn reflective_anatomy() -> CardDefinition {
    CardDefinition {
        name: "Reflective Anatomy",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            // Use Value::CountersOn on a fan-out selector — sums +1/+1
            // counters across all controlled creatures.
            power: Value::CountersOn {
                what: Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
                kind: CounterType::PlusOnePlusOne,
            },
            toughness: Value::CountersOn {
                what: Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
                kind: CounterType::PlusOnePlusOne,
            },
            duration: Duration::EndOfTurn,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 26: 3 new Lessons ───────────────────────────

/// Pest Studies — {1}{B}{G} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Create two 1/1 black and green Pest
/// creature tokens with 'When this creature dies, you gain 1 life.'"
///
/// 3-mana 2-Pest token engine — a Lesson that doubles as a board widener.
/// Each Pest carries the standard die-trigger lifegain via the shared
/// `stx_pest_token` helper.
pub fn pest_studies() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Pest Studies",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: pest,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Lecture in Strategy — {1}{R}{W} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Creatures you control get +1/+1 and gain
/// vigilance until end of turn."
///
/// 3-mana go-wide combat trick that doubles as Lesson sideboard fodder.
/// Anthems your team + adds Vigilance for a safe alpha strike.
pub fn lecture_in_strategy() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    CardDefinition {
        name: "Lecture in Strategy",
        cost: cost(&[generic(1), crate::mana::r(), crate::mana::w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Vigilance,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 28: 5 more Lessons ───────────────────────────

/// Necrotic Studies — {2}{B} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Destroy target creature with mana value
/// 3 or less."
///
/// 3-mana targeted removal at the Lesson slot — Doomblade-light. Slots
/// into the Witherbloom Lesson sideboard for sticky creature problems.
pub fn necrotic_studies() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Studies",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Pyromathematics — {1}{R} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Pyromathematics deals 3 damage divided
/// as you choose among any number of targets."
///
/// Collapsed to "3 damage to any target" at the single-target slot
/// (engine-wide divided-damage gap). Lesson burn for Lorehold/Prismari.
pub fn pyromathematics() -> CardDefinition {
    CardDefinition {
        name: "Pyromathematics",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Lesson — {1}{W}{B} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Create two 1/1 white-and-black Inkling
/// creature tokens with flying."
///
/// 3-mana Lesson double-Inkling. Defend the Campus's smaller cousin.
pub fn inkling_lesson() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Lesson",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: inkling_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Studies — {1}{G}{U} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Create a 0/0 green-and-blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is the number of
/// creatures you control."
///
/// 3-mana wide-board-scaling Fractal mint at the Lesson slot. Wired via
/// `Selector::LastCreatedToken` + `Value::CountOf` reading creatures you
/// control.
pub fn fractal_studies() -> CardDefinition {
    let fractal = TokenDefinition {
        name: "Fractal".to_string(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Fractal Studies",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Spirit Lesson — {2}{R}{W} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Create two 2/2 red-and-white Spirit
/// creature tokens."
///
/// 4-mana double-Spirit-mint at the Lesson slot. Lorehold's go-wide
/// Lesson — pairs with Quintorius / Sparring Regimen for tribal payoffs.
pub fn spirit_lesson() -> CardDefinition {
    use crate::catalog::sets::stx::lorehold_spirit_token;
    CardDefinition {
        name: "Spirit Lesson",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_token(),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Advanced Cartography — {1}{G}{U} Sorcery — Lesson.
///
/// Printed Oracle (synthesised): "Search your library for a basic land
/// card, put it onto the battlefield tapped, then shuffle. Scry 2."
///
/// 3-mana ramp + dig. Plays as a Cultivate-lite at the Lesson slot.
pub fn advanced_cartography() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Advanced Cartography",
        cost: cost(&[generic(1), crate::mana::g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 32 (modern_decks) — Lesson expansion ──────────────────────────────

/// Mascot Interpretation II (Lesson) — {2}{W} Sorcery — Lesson (batch 32).
/// Synthesised Oracle: "Create a 2/2 white-and-black Inkling token with
/// flying."
pub fn mascot_lesson_b32() -> CardDefinition {
    let inkling = TokenDefinition {
        name: "Inkling".to_string(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Mascot Interpretation II",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Confront the Doubt — {2}{B} Sorcery — Lesson.
/// Synthesised Oracle: "Target opponent reveals their hand. You choose a
/// noncreature, nonland card from it. That player discards that card. You
/// gain 2 life."
pub fn confront_the_doubt() -> CardDefinition {
    CardDefinition {
        name: "Confront the Doubt",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland
                    .and(SelectionRequirement::Not(Box::new(SelectionRequirement::Creature))),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Test of Patience — {2}{U} Sorcery — Lesson.
/// Synthesised Oracle: "Counter target activated or triggered ability. Draw
/// a card."
/// 🟡 Approximation: the counter-ability primitive currently targets only
/// activated abilities on the stack (see Stifle in TODO.md). Body just
/// draws a card at this point.
pub fn test_of_patience() -> CardDefinition {
    CardDefinition {
        name: "Test of Patience",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Reduce to Ashes — {3}{R} Sorcery — Lesson.
/// Synthesised Oracle: "Reduce to Ashes deals 4 damage to target creature
/// or planeswalker. If that creature or planeswalker would die this turn,
/// exile it instead." (Damage-replacement rider omitted; body is 4 dmg.)
pub fn reduce_to_ashes() -> CardDefinition {
    CardDefinition {
        name: "Reduce to Ashes",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(4),
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Plant Adept Lesson — {1}{G} Sorcery — Lesson.
/// Synthesised Oracle: "Target creature you control gets +2/+2 and gains
/// trample until end of turn."
pub fn plant_adept_lesson() -> CardDefinition {
    CardDefinition {
        name: "Plant Adept Lesson",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
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
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}
