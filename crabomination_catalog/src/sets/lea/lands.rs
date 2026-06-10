use super::tap_add;
use crate::card::{CardDefinition, CardType, LandType, Subtypes, Supertype};
use crate::mana::Color;

pub fn plains() -> CardDefinition {
    CardDefinition {
        name: "Plains",
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Plains],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(Color::White)],
        ..Default::default()
    }
}

pub fn island() -> CardDefinition {
    CardDefinition {
        name: "Island",
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Island],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(Color::Blue)],
        ..Default::default()
    }
}

pub fn swamp() -> CardDefinition {
    CardDefinition {
        name: "Swamp",
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Swamp],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(Color::Black)],
        ..Default::default()
    }
}

pub fn mountain() -> CardDefinition {
    CardDefinition {
        name: "Mountain",
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Mountain],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(Color::Red)],
        ..Default::default()
    }
}

pub fn forest() -> CardDefinition {
    CardDefinition {
        name: "Forest",
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Forest],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(Color::Green)],
        ..Default::default()
    }
}

/// Wastes — basic land with no land subtype; {T}: Add {C}.
pub fn wastes() -> CardDefinition {
    CardDefinition {
        name: "Wastes",
        supertypes: vec![Supertype::Basic],
        card_types: vec![CardType::Land],
        activated_abilities: vec![super::tap_add_colorless()],
        ..Default::default()
    }
}

fn dual(name: &'static str, a: LandType, b: LandType, ca: Color, cb: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![a, b],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(ca), tap_add(cb)],
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
