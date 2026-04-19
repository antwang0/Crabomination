use super::tap_add;
use crate::card::{ActivatedAbility, CardDefinition, CardType, SelectionRequirement, SpellEffect, Subtypes};
use crate::mana::{cost, generic, Color, ManaCost};

/// Mox Pearl — {0} Artifact, {T}: Add {W}
pub fn mox_pearl() -> CardDefinition {
    CardDefinition {
        name: "Mox Pearl",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::White)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Mox Sapphire — {0} Artifact, {T}: Add {U}
pub fn mox_sapphire() -> CardDefinition {
    CardDefinition {
        name: "Mox Sapphire",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Mox Jet — {0} Artifact, {T}: Add {B}
pub fn mox_jet() -> CardDefinition {
    CardDefinition {
        name: "Mox Jet",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Black)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Mox Ruby — {0} Artifact, {T}: Add {R}
pub fn mox_ruby() -> CardDefinition {
    CardDefinition {
        name: "Mox Ruby",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Red)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Mox Emerald — {0} Artifact, {T}: Add {G}
pub fn mox_emerald() -> CardDefinition {
    CardDefinition {
        name: "Mox Emerald",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Black Lotus — {0} Artifact, {T}: Add three mana of any one color
pub fn black_lotus() -> CardDefinition {
    CardDefinition {
        name: "Black Lotus",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effects: vec![SpellEffect::AddManaAnyColor { amount: 3 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Sol Ring — {1} Artifact, {T}: Add {C}{C}
pub fn sol_ring() -> CardDefinition {
    CardDefinition {
        name: "Sol Ring",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effects: vec![SpellEffect::AddColorlessMana { amount: 2 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Nevinyrral's Disk — {4} Artifact, {1}{T}: Destroy all artifacts, creatures, and enchantments
pub fn nevinyrrals_disk() -> CardDefinition {
    CardDefinition {
        name: "Nevinyrral's Disk",
        cost: cost(&[generic(4)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effects: vec![
                SpellEffect::DestroyAll { target: SelectionRequirement::Artifact },
                SpellEffect::DestroyAll { target: SelectionRequirement::Creature },
                SpellEffect::DestroyAll { target: SelectionRequirement::Enchantment },
            ],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
