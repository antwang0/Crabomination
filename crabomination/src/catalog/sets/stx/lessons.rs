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
    CardDefinition, CardType, CreatureType, Effect, Selector, SelectionRequirement,
    SpellSubtype, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{cost, g, generic, r, u, w, Color};

// ── Environmental Sciences ──────────────────────────────────────────────────

/// Environmental Sciences — {1}{G} Sorcery — Lesson.
///
/// "You gain 4 life. Search your library for a basic land card, reveal it,
/// put it into your hand, then shuffle."
///
/// Wired as a `Seq(GainLife(4), Search(IsBasicLand → hand))`. The Search
/// uses `ZoneDest::Hand(You)` so the land enters the hand (not the
/// battlefield) — matching the printed mode. Shuffle is implicit in
/// `Effect::Search`'s post-pick behaviour (the engine reshuffles the
/// library after a successful search).
pub fn environmental_sciences() -> CardDefinition {
    CardDefinition {
        name: "Environmental Sciences",
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
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
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
    }
}

// ── Introduction to Annihilation ────────────────────────────────────────────

/// Introduction to Annihilation — {3}{W} Sorcery — Lesson.
///
/// "Destroy target nonland permanent. Its controller scries 2."
///
/// Single-target removal that's softer than Vindicate (no "any" — only
/// nonland) but gives the targeted permanent's controller a free Scry 2
/// as a small consolation. Wired as `Seq(Destroy + Scry(target's
/// controller))` using `PlayerRef::ControllerOf(Target(0))` to thread the
/// post-destroy controller into the Scry call.
pub fn introduction_to_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Annihilation",
        cost: cost(&[generic(3), w()]),
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
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            // Controller of the destroyed permanent scries 2.
            // `ControllerOf(Target(0))` resolves to the player who
            // controlled the permanent at target-lock time (the same id
            // the destruction sliced through).
            Effect::Scry {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
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
    }
}

// ── Introduction to Prophecy ────────────────────────────────────────────────

/// Introduction to Prophecy — {2}{U} Sorcery — Lesson.
///
/// "Scry 3, then draw a card."
///
/// Classic blue cantrip-with-Scry. `Seq(Scry(3), Draw(1))` — cleanly
/// composes against the engine's existing primitives. Acts as both
/// filtering and card advantage in any deck that wants to dig.
pub fn introduction_to_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Prophecy",
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
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
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
    }
}

// ── Spirit Summoning ────────────────────────────────────────────────────────

/// Spirit Summoning — {3}{W} Sorcery — Lesson.
///
/// "Create a 3/2 white Spirit creature token with lifelink."
///
/// White-flavor body Lesson — slots alongside Inkling Summoning ({3}{W}{B},
/// 2/1 W/B flying) and Pest Summoning ({B}{G}, two 1/1 B/G Pests). The
/// 3/2 Spirit with lifelink rate puts a respectable mid-curve body on
/// the battlefield without needing the Magecraft/Mascot Exhibition payoff
/// stack — a fine Lesson for white-based decks that want a single big
/// body for four mana.
pub fn spirit_summoning() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".to_string(),
        power: 3,
        toughness: 2,
        keywords: vec![crate::card::Keyword::Lifelink],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Spirit Summoning",
        cost: cost(&[generic(3), w()]),
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
            definition: spirit,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

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
    }
}
