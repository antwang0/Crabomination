use super::tap_add;
use crate::card::{ActivatedAbility, CardDefinition, CardType, Effect, SelectionRequirement};
use crate::effect::{ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{Color, ManaCost, cost, generic};

/// Mox Pearl — {0} Artifact, {T}: Add {W}
pub fn mox_pearl() -> CardDefinition {
    CardDefinition {
        name: "Mox Pearl",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add(Color::White)],
        ..Default::default()
    }
}

/// Mox Sapphire — {0} Artifact, {T}: Add {U}
pub fn mox_sapphire() -> CardDefinition {
    CardDefinition {
        name: "Mox Sapphire",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Mox Jet — {0} Artifact, {T}: Add {B}
pub fn mox_jet() -> CardDefinition {
    CardDefinition {
        name: "Mox Jet",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add(Color::Black)],
        ..Default::default()
    }
}

/// Mox Ruby — {0} Artifact, {T}: Add {R}
pub fn mox_ruby() -> CardDefinition {
    CardDefinition {
        name: "Mox Ruby",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add(Color::Red)],
        ..Default::default()
    }
}

/// Mox Emerald — {0} Artifact, {T}: Add {G}
pub fn mox_emerald() -> CardDefinition {
    CardDefinition {
        name: "Mox Emerald",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add(Color::Green)],
        ..Default::default()
    }
}

/// Black Lotus — {0} Artifact, {T}: Add three mana of any one color
pub fn black_lotus() -> CardDefinition {
    CardDefinition {
        name: "Black Lotus",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
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
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sol Ring — {1} Artifact, {T}: Add {C}{C}
pub fn sol_ring() -> CardDefinition {
    CardDefinition {
        name: "Sol Ring",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
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
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nevinyrral's Disk — {4} Artifact, {1}{T}: Destroy all artifacts, creatures, and enchantments
pub fn nevinyrrals_disk() -> CardDefinition {
    CardDefinition {
        name: "Nevinyrral's Disk",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
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
            ..Default::default()
        }],
        ..Default::default()
    }
}
