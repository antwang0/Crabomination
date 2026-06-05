use super::tap_add;
use crate::card::{ActivatedAbility, CardDefinition, CardType, Effect, SelectionRequirement, Subtypes};
use crate::effect::{ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{Color, ManaCost, cost, generic};

/// Mox Pearl — {0} Artifact, {T}: Add {W}
pub fn mox_pearl() -> CardDefinition {
    CardDefinition {
        name: "Mox Pearl",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::White)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mox Sapphire — {0} Artifact, {T}: Add {U}
pub fn mox_sapphire() -> CardDefinition {
    CardDefinition {
        name: "Mox Sapphire",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mox Jet — {0} Artifact, {T}: Add {B}
pub fn mox_jet() -> CardDefinition {
    CardDefinition {
        name: "Mox Jet",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Black)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mox Ruby — {0} Artifact, {T}: Add {R}
pub fn mox_ruby() -> CardDefinition {
    CardDefinition {
        name: "Mox Ruby",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Red)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mox Emerald — {0} Artifact, {T}: Add {G}
pub fn mox_emerald() -> CardDefinition {
    CardDefinition {
        name: "Mox Emerald",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Black Lotus — {0} Artifact, {T}: Add three mana of any one color
pub fn black_lotus() -> CardDefinition {
    CardDefinition {
        name: "Black Lotus",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(3)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Sol Ring — {1} Artifact, {T}: Add {C}{C}
pub fn sol_ring() -> CardDefinition {
    CardDefinition {
        name: "Sol Ring",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Nevinyrral's Disk — {4} Artifact, {1}{T}: Destroy all artifacts, creatures, and enchantments
pub fn nevinyrrals_disk() -> CardDefinition {
    CardDefinition {
        name: "Nevinyrral's Disk",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::Destroy {
                    what: Selector::EachPermanent(SelectionRequirement::Artifact),
                },
                Effect::Destroy {
                    what: Selector::EachPermanent(SelectionRequirement::Creature),
                },
                Effect::Destroy {
                    what: Selector::EachPermanent(SelectionRequirement::Enchantment),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}
