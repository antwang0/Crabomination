//! Shared / multi-college Strixhaven cards.
//!
//! Cards from each college that don't fit cleanly into a single-college file
//! (because they're cross-school commons or shared among colleges) live
//! here. Currently a small set of safer additions:
//!
//! - **Inkling Summoning** (W/B Lesson) — creates a 2/1 white-and-black
//!   Inkling token with flying.
//! - **Reduce to Memory** (W/U Lesson) — exile a permanent + create a
//!   colorless Spirit token whose P/T equals the exiled card's mana value.
//! - **Tend the Pests** (B/G) — sacrifice a creature, create X Pest tokens.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, SpellSubtype, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::PlayerRef;
use crate::mana::{cost, generic, b, g, w, Color};

/// Strixhaven Pest token: 1/1 black-and-green creature with
/// "When this creature dies, you gain 1 life." Shared by Pest
/// Summoning, Tend the Pests, Eyetwitch (death rider), Hunt for
/// Specimens, and any future STX Pest minter.
///
/// Built on top of the new `TokenDefinition.triggered_abilities`
/// slot so the death-trigger lifegain rides every Pest copy
/// consistently. Witherbloom payoffs (Witherbloom Apprentice's
/// magecraft drain, Killian's Confidence's draw chain, etc.) get
/// the printed lifegain trickle for free.
pub fn stx_pest_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pest".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
    }
}

// ── Inkling Summoning (Lesson) ──────────────────────────────────────────────

/// Inkling Summoning — {3}{W}{B} Sorcery — Lesson. "Create a 2/1 white and
/// black Inkling creature token with flying."
///
/// The Inkling subtype was added to `CreatureType` in the same patch that
/// added Pest / Fractal. Lesson sub-type is recorded so future Lesson-aware
/// effects can filter on it.
pub fn inkling_summoning() -> CardDefinition {
    let inkling = TokenDefinition {
        name: "Inkling".to_string(),
        power: 2,
        toughness: 1,
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
        name: "Inkling Summoning",
        cost: cost(&[generic(3), w(), b()]),
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
    }
}

// ── Spirit Summoning (Lesson) ───────────────────────────────────────────────

/// Spirit Summoning — {3}{W} Sorcery — Lesson. Printed Oracle:
/// "Create a 1/1 white Spirit creature token with flying."
///
/// White's slot in the STX Lesson cycle. Same shape as Pest Summoning
/// (Witherbloom) and Inkling Summoning (Silverquill) — a one-line
/// `Effect::CreateToken` with the Lesson `SpellSubtype` recorded so
/// future Lesson-aware cards can filter on it. The token is a vanilla
/// 1/1 white Spirit with flying — the same definition shape used by
/// Sparring Regimen (different P/T but same Spirit subtype) and
/// matching Lorehold Command's mode 1 flying-Spirit token.
pub fn spirit_summoning() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
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

// ── Tend the Pests ──────────────────────────────────────────────────────────

/// Tend the Pests — {1}{B}{G} Sorcery. "As an additional cost to cast this
/// spell, sacrifice a creature. Create X 1/1 black and green Pest creature
/// tokens with 'When this creature dies, you gain 1 life,' where X is the
/// sacrificed creature's power."
///
/// 🟡 simplification: the "additional cost" sacrifice is folded into
/// resolution (cost-as-first-step, same approximation Thud uses). The
/// bot/UI never tries to interrupt between the cost being paid and the
/// spell resolving. The spawned Pest tokens **now carry** the "die →
/// gain 1 life" trigger via the new `TokenDefinition.triggered_abilities`
/// field, so the Witherbloom lifegain chain works end-to-end (each Pest
/// dies → +1 life → Pest Mascot / Killian's Confidence riders fire).
pub fn tend_the_pests() -> CardDefinition {
    let pest = stx_pest_token();
    CardDefinition {
        name: "Tend the Pests",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::Repeat {
                count: Value::SacrificedPower,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: pest,
                }),
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
