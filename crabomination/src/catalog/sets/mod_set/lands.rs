//! Modern-staple lands. Currently the six remaining Ravnica shocklands not
//! covered by the demo decks.
//!
//! Each shockland is a dual land of two basic land types whose ETB trigger
//! fires the shared `shockland_pay_two_or_tap` choice (pay 2 life to enter
//! untapped, otherwise enter tapped). See `sets/mod.rs` for the helper.

use super::super::{dual_land_with, shockland_pay_two_or_tap};
use crate::card::{CardDefinition, LandType};
use crate::mana::Color;

/// Sacred Foundry — RW shockland (Plains/Mountain).
pub fn sacred_foundry() -> CardDefinition {
    dual_land_with(
        "Sacred Foundry",
        LandType::Plains,
        LandType::Mountain,
        Color::White,
        Color::Red,
        vec![shockland_pay_two_or_tap()],
    )
}

/// Steam Vents — UR shockland (Island/Mountain).
pub fn steam_vents() -> CardDefinition {
    dual_land_with(
        "Steam Vents",
        LandType::Island,
        LandType::Mountain,
        Color::Blue,
        Color::Red,
        vec![shockland_pay_two_or_tap()],
    )
}

/// Stomping Ground — RG shockland (Mountain/Forest).
pub fn stomping_ground() -> CardDefinition {
    dual_land_with(
        "Stomping Ground",
        LandType::Mountain,
        LandType::Forest,
        Color::Red,
        Color::Green,
        vec![shockland_pay_two_or_tap()],
    )
}

/// Temple Garden — GW shockland (Forest/Plains).
pub fn temple_garden() -> CardDefinition {
    dual_land_with(
        "Temple Garden",
        LandType::Forest,
        LandType::Plains,
        Color::Green,
        Color::White,
        vec![shockland_pay_two_or_tap()],
    )
}

/// Breeding Pool — GU shockland (Forest/Island).
pub fn breeding_pool() -> CardDefinition {
    dual_land_with(
        "Breeding Pool",
        LandType::Forest,
        LandType::Island,
        Color::Green,
        Color::Blue,
        vec![shockland_pay_two_or_tap()],
    )
}

/// Blood Crypt — BR shockland (Swamp/Mountain).
pub fn blood_crypt() -> CardDefinition {
    dual_land_with(
        "Blood Crypt",
        LandType::Swamp,
        LandType::Mountain,
        Color::Black,
        Color::Red,
        vec![shockland_pay_two_or_tap()],
    )
}
