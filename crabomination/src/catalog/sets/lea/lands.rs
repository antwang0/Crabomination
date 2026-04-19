use super::tap_add;
use crate::card::{CardDefinition, CardType, Effect, LandType, Subtypes, Supertype};
use crate::mana::{Color, ManaCost};

pub fn plains() -> CardDefinition {
    CardDefinition {
        name: "Plains",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Plains],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::White)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

pub fn island() -> CardDefinition {
    CardDefinition {
        name: "Island",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Island],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

pub fn swamp() -> CardDefinition {
    CardDefinition {
        name: "Swamp",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Swamp],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Black)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

pub fn mountain() -> CardDefinition {
    CardDefinition {
        name: "Mountain",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Mountain],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Red)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

pub fn forest() -> CardDefinition {
    CardDefinition {
        name: "Forest",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Forest],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
