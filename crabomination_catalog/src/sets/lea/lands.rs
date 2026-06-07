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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
    }
}

/// Wastes — basic land with no land subtype; {T}: Add {C}.
pub fn wastes() -> CardDefinition {
    CardDefinition {
        name: "Wastes",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        power: 0,
        toughness: 0,
        activated_abilities: vec![super::tap_add_colorless()],
        ..Default::default()
    }
}

fn dual(name: &'static str, a: LandType, b: LandType, ca: Color, cb: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![a, b],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(ca), tap_add(cb)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

pub fn tundra() -> CardDefinition {
    dual("Tundra", LandType::Plains, LandType::Island, Color::White, Color::Blue)
}
pub fn underground_sea() -> CardDefinition {
    dual("Underground Sea", LandType::Island, LandType::Swamp, Color::Blue, Color::Black)
}
pub fn badlands() -> CardDefinition {
    dual("Badlands", LandType::Swamp, LandType::Mountain, Color::Black, Color::Red)
}
pub fn taiga() -> CardDefinition {
    dual("Taiga", LandType::Mountain, LandType::Forest, Color::Red, Color::Green)
}
pub fn savannah() -> CardDefinition {
    dual("Savannah", LandType::Forest, LandType::Plains, Color::Green, Color::White)
}
pub fn scrubland() -> CardDefinition {
    dual("Scrubland", LandType::Plains, LandType::Swamp, Color::White, Color::Black)
}
pub fn volcanic_island() -> CardDefinition {
    dual("Volcanic Island", LandType::Island, LandType::Mountain, Color::Blue, Color::Red)
}
pub fn bayou() -> CardDefinition {
    dual("Bayou", LandType::Swamp, LandType::Forest, Color::Black, Color::Green)
}
pub fn plateau() -> CardDefinition {
    dual("Plateau", LandType::Mountain, LandType::Plains, Color::Red, Color::White)
}
pub fn tropical_island() -> CardDefinition {
    dual("Tropical Island", LandType::Forest, LandType::Island, Color::Green, Color::Blue)
}
