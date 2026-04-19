use super::no_abilities;
use crate::card::{CardDefinition, CardType, SelectionRequirement, SpellEffect, Subtypes};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Swords to Plowshares — {W}: exile target creature
pub fn swords_to_plowshares() -> CardDefinition {
    CardDefinition {
        name: "Swords to Plowshares",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::ExilePermanent {
            target: SelectionRequirement::Creature,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Counterspell — {U}{U}: counter target spell
pub fn counterspell() -> CardDefinition {
    CardDefinition {
        name: "Counterspell",
        cost: cost(&[u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::CounterSpell {
            target: SelectionRequirement::Any,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Ancestral Recall — {U}: draw 3 cards
pub fn ancestral_recall() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Recall",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DrawCards { amount: 3 }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Dark Ritual — {B}: add {B}{B}{B} to your mana pool
pub fn dark_ritual() -> CardDefinition {
    CardDefinition {
        name: "Dark Ritual",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::AddMana {
            colors: vec![Color::Black, Color::Black, Color::Black],
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Terror — {1}{B}: destroy target non-black, non-artifact creature
pub fn terror() -> CardDefinition {
    CardDefinition {
        name: "Terror",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DestroyCreature {
            target: SelectionRequirement::Creature
                .and(SelectionRequirement::HasColor(Color::Black).negate())
                .and(SelectionRequirement::Artifact.negate()),
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Lightning Bolt — {R}: deal 3 damage to any target
pub fn lightning_bolt() -> CardDefinition {
    CardDefinition {
        name: "Lightning Bolt",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DealDamage {
            amount: 3,
            target: SelectionRequirement::Any,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Giant Growth — {G}: target creature gets +3/+3 until end of turn
pub fn giant_growth() -> CardDefinition {
    CardDefinition {
        name: "Giant Growth",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::PumpCreature {
            power_bonus: 3,
            toughness_bonus: 3,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
