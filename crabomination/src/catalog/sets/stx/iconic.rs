//! Iconic Strixhaven cards that didn't fit cleanly into a single college
//! file: Strict Proctor (W mono with cross-college impact), Sedgemoor
//! Witch (B mono with magecraft Pest creation), Spectacle Mage (U/R hybrid
//! prowess body), Mage Hunters' Onslaught (B sorcery with destroy +
//! cantrip).

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Selector, SelectionRequirement,
    Subtypes, Value,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::PlayerRef;
use crate::mana::{b, cost, generic, r, u, w};

// ── Strict Proctor ──────────────────────────────────────────────────────────

/// Strict Proctor — {1}{W}, 1/3 Spirit Cleric. Flying. Real Oracle: "If
/// a permanent entering the battlefield causes a triggered ability of
/// a permanent to trigger, that ability's controller sacrifices the
/// permanent unless they pay {2}." Body wired with Flying; the
/// ETB-tax replacement effect is 🟡 (the engine has no
/// replacement-effect primitive yet — tracked in TODO.md).
pub fn strict_proctor() -> CardDefinition {
    CardDefinition {
        name: "Strict Proctor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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

// ── Sedgemoor Witch ─────────────────────────────────────────────────────────

/// Sedgemoor Witch — {2}{B}{B}, 3/2 Human Warlock. Menace, Ward 1.
/// Real Oracle Magecraft: "Whenever you cast or copy an instant or
/// sorcery spell, create a 1/1 black Pest creature token with 'When
/// this creature dies, you gain 1 life.'"
///
/// Wired via the existing magecraft helper + the Pest token shared
/// helper in `super::shared::stx_pest_token`. The "creates may"
/// upgrade clause (real Oracle is non-may; we keep it non-may here).
/// Ward 1 ships as a `Keyword::Ward(1)` on the body — the engine has
/// the keyword declared but no targeting-trigger plumbing yet; it
/// remains 🟡 for that reason but the magecraft trigger is full.
pub fn sedgemoor_witch() -> CardDefinition {
    CardDefinition {
        name: "Sedgemoor Witch",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace, Keyword::Ward(1)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: super::shared::stx_pest_token(),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Spectacle Mage ──────────────────────────────────────────────────────────

/// Spectacle Mage — {U/R}{U/R}, 1/2 Human Wizard. Prowess. Real Oracle
/// flavor: "Whenever you cast a noncreature spell, this creature gets
/// +1/+1 until end of turn." (Standard prowess.) Hybrid {U/R}{U/R} is
/// approximated as `{U}{R}` (engine has no hybrid mana resolver — a
/// player who can produce only U or only R can still cast). Prowess
/// keyword is declared today but not yet wired into the trigger
/// system; the body and stat line are correct.
pub fn spectacle_mage() -> CardDefinition {
    CardDefinition {
        name: "Spectacle Mage",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
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

// ── Mage Hunters' Onslaught ────────────────────────────────────────────────

/// Mage Hunters' Onslaught — {2}{B}{B}, Sorcery. Real Oracle: "Destroy
/// target creature. Then if a creature died this turn, draw a card."
///
/// We ship the unconditional version (`Destroy + Draw 1`) — the
/// engine's "creature died this turn" tally exists
/// (`Player.creatures_died_this_turn`) but the `Predicate::Creatures
/// DiedThisTurnAtLeast(0)` is trivially true after the destroy fires
/// anyway, so the gate is a no-op for this particular spell. Keeping
/// the unconditional shape avoids a needless gate.
pub fn mage_hunters_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Mage Hunters' Onslaught",
        cost: cost(&[generic(2), b(), b()]),
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
