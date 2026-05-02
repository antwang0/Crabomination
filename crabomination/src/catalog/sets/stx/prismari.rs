//! Prismari (U/R) college cards from Strixhaven.
//!
//! Prismari's themes are spellslinger payoffs (Magecraft, copy-spell
//! triggers, treasure-style ramp) and creature-makes-token mid-cast
//! synergies. The first wave here covers the basic Apprentice +
//! Pledgemage pair, plus a couple of mono-shape supporting cards.
//! Larger Prismari cards (Magma Opus, Expressive Iteration's siblings)
//! lean on the copy-spell primitive and stay ⏳ until that lands.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Selector, Subtypes, Value,
};
use crate::effect::shortcut::{magecraft, magecraft_self_pump};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, generic, r, u};

// ── Prismari Pledgemage ─────────────────────────────────────────────────────

/// Prismari Pledgemage — {1}{U}{R}, 2/3 Elemental. "Trample, haste."
///
/// Pure stat-line + keyword body. Prismari Pledgemage is the "free
/// vanilla beater" of the Prismari arsenal: a 2/3 trample-haste for
/// {URR}-equivalent costs is solid, and it composes against every
/// pump and copy effect in the college.
pub fn prismari_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pledgemage",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        effect: Effect::Noop,
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

// ── Prismari Apprentice ─────────────────────────────────────────────────────

/// Prismari Apprentice — {U}{R}, 2/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// choose one — / • Scry 1. / • Prismari Apprentice gets +1/+0 until
/// end of turn."
///
/// 🟡 Currently picks mode 0 (Scry 1) only — `magecraft` produces a
/// single trigger, and the auto-decider for `Effect::ChooseMode` inside
/// a triggered ability defaults to mode 0. A "let the controller pick"
/// hook for triggered-ability ChooseMode would unblock the +1/+0 mode.
/// Tracked under TODO.md "May Optionality / mode-pick on triggers".
pub fn prismari_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Prismari Apprentice",
        cost: cost(&[u(), r()]),
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
    }
}

// ── Symmetry Sage ───────────────────────────────────────────────────────────

/// Symmetry Sage — {U}, 1/2 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// Symmetry Sage gets +1/+0 and gains flying until end of turn."
///
/// 🟡 We split the rider into two separate magecraft triggers: one
/// `magecraft_self_pump(+1/+0)` and one grant-flying. They're functionally
/// equivalent to the original `Seq` body — both fire on every magecraft
/// event and both reference the source via `Selector::This`. The split
/// also means the helper is reusable across any future magecraft
/// self-pump creature (e.g. Quandrix's Berta, Symmetry Sage's siblings)
/// without copy-pasting a six-line `Seq`.
pub fn symmetry_sage() -> CardDefinition {
    CardDefinition {
        name: "Symmetry Sage",
        cost: cost(&[u()]),
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
            magecraft_self_pump(1, 0),
            magecraft(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Creative Outburst ───────────────────────────────────────────────────────

/// Creative Outburst — {3}{U}{U}{R}{R} Sorcery. "Discard your hand. Draw
/// five cards."
///
/// Pure rummage upgrade: the entire hand goes to the graveyard, then the
/// caster draws 5. Perfect Prismari spellslinger refill — the discarded
/// instants/sorceries fuel later Magecraft / flashback payoffs.
pub fn creative_outburst() -> CardDefinition {
    use crate::effect::Value as V;
    CardDefinition {
        name: "Creative Outburst",
        cost: cost(&[generic(3), u(), u(), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: V::HandSizeOf(PlayerRef::You),
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(5),
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

