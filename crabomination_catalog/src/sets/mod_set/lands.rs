//! Modern-staple lands. Shocklands, the remaining fastlands, and the
//! Mirrodin / "indestructible" artifact land cycle.
//!
//! * Shocklands — dual land of two basic types; ETB choice "pay 2 life or
//!   enter tapped" via `shockland_pay_two_or_tap`.
//! * Fastlands — dual land that ETB-taps once you control four+ lands;
//!   reuses `fastland_etb_conditional_tap`.
//! * Artifact lands (Mirrodin cycle) — single-color land that's also an
//!   artifact. Built inline here since the existing `dual_land_with`
//!   helper doesn't compose `CardType::Artifact` onto a Land.

use super::super::{dual_land_with, fastland_etb_conditional_tap, shockland_pay_two_or_tap, tap_add};
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, Effect, Keyword, LandType, Subtypes,
};
use crate::effect::{ManaPayload, PlayerRef, Value};
use crate::mana::{Color, ManaCost};

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

// ── Fastlands ────────────────────────────────────────────────────────────────

/// Seachrome Coast — UW fastland (Plains/Island).
pub fn seachrome_coast() -> CardDefinition {
    dual_land_with(
        "Seachrome Coast",
        LandType::Plains,
        LandType::Island,
        Color::White,
        Color::Blue,
        vec![fastland_etb_conditional_tap()],
    )
}

/// Darkslick Shores — UB fastland (Island/Swamp).
pub fn darkslick_shores() -> CardDefinition {
    dual_land_with(
        "Darkslick Shores",
        LandType::Island,
        LandType::Swamp,
        Color::Blue,
        Color::Black,
        vec![fastland_etb_conditional_tap()],
    )
}

/// Spirebluff Canal — UR fastland (Island/Mountain).
pub fn spirebluff_canal() -> CardDefinition {
    dual_land_with(
        "Spirebluff Canal",
        LandType::Island,
        LandType::Mountain,
        Color::Blue,
        Color::Red,
        vec![fastland_etb_conditional_tap()],
    )
}

/// Botanical Sanctum — UG fastland (Forest/Island).
pub fn botanical_sanctum() -> CardDefinition {
    dual_land_with(
        "Botanical Sanctum",
        LandType::Forest,
        LandType::Island,
        Color::Green,
        Color::Blue,
        vec![fastland_etb_conditional_tap()],
    )
}

/// Razorverge Thicket — GW fastland (Forest/Plains).
pub fn razorverge_thicket() -> CardDefinition {
    dual_land_with(
        "Razorverge Thicket",
        LandType::Forest,
        LandType::Plains,
        Color::Green,
        Color::White,
        vec![fastland_etb_conditional_tap()],
    )
}

/// Concealed Courtyard — WB fastland (Plains/Swamp).
pub fn concealed_courtyard() -> CardDefinition {
    dual_land_with(
        "Concealed Courtyard",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
        vec![fastland_etb_conditional_tap()],
    )
}

/// Inspiring Vantage — RW fastland (Mountain/Plains).
pub fn inspiring_vantage() -> CardDefinition {
    dual_land_with(
        "Inspiring Vantage",
        LandType::Mountain,
        LandType::Plains,
        Color::Red,
        Color::White,
        vec![fastland_etb_conditional_tap()],
    )
}

// ── Artifact lands (Mirrodin cycle) ──────────────────────────────────────────

/// Build a single-color artifact land. Card types are `Land + Artifact` and
/// the basic land type is preserved on `subtypes.land_types` so non-basic
/// "is a Forest" lookups still work (Nature's Lore tutoring Tree of Tales,
/// e.g.). Each tap produces one mana of the chosen color.
fn artifact_land(
    name: &'static str,
    land_type: LandType,
    color: Color,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land, CardType::Artifact],
        subtypes: Subtypes {
            land_types: vec![land_type],
            ..Default::default()
        },
        keywords,
        activated_abilities: vec![tap_add(color)],
        ..Default::default()
    }
}

/// Ancient Den — Artifact Land — Plains. {T}: Add {W}.
pub fn ancient_den() -> CardDefinition {
    artifact_land("Ancient Den", LandType::Plains, Color::White, vec![])
}

/// Seat of the Synod — Artifact Land — Island. {T}: Add {U}.
pub fn seat_of_the_synod() -> CardDefinition {
    artifact_land("Seat of the Synod", LandType::Island, Color::Blue, vec![])
}

/// Vault of Whispers — Artifact Land — Swamp. {T}: Add {B}.
pub fn vault_of_whispers() -> CardDefinition {
    artifact_land("Vault of Whispers", LandType::Swamp, Color::Black, vec![])
}

/// Great Furnace — Artifact Land — Mountain. {T}: Add {R}.
pub fn great_furnace() -> CardDefinition {
    artifact_land("Great Furnace", LandType::Mountain, Color::Red, vec![])
}

/// Tree of Tales — Artifact Land — Forest. {T}: Add {G}.
pub fn tree_of_tales() -> CardDefinition {
    artifact_land("Tree of Tales", LandType::Forest, Color::Green, vec![])
}

/// Darksteel Citadel — Indestructible Artifact Land. {T}: Add {C}.
///
/// Built inline rather than via `artifact_land` because it produces
/// colorless rather than a single colored mana, and carries the
/// Indestructible keyword.
pub fn darksteel_citadel() -> CardDefinition {
    CardDefinition {
        name: "Darksteel Citadel",
        card_types: vec![CardType::Land, CardType::Artifact],
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
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

// ── Exotic Orchard ─────────────────────────────────────────────────────────

/// Exotic Orchard — Land. {T}: Add one mana of any color.
///
/// Approximation: the printed text is "Add one mana of any color that a
/// land an opponent controls could produce." Simplified to unrestricted
/// any-one-color since opponents always have basics in cube games and the
/// restriction rarely matters in practice.
pub fn exotic_orchard() -> CardDefinition {
    use super::super::tap_add_any_color;
    CardDefinition {
        name: "Exotic Orchard",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_any_color()],
        ..Default::default()
    }
}

// ── Commander mana lands ─────────────────────────────────────────────────────

/// Command Tower — Land. "{T}: Add one mana of any color in your commander's
/// color identity." Approximated as unrestricted any-one-color (no commander
/// identity gate), matching the Exotic Orchard / Arcane Signet convention.
pub fn command_tower() -> CardDefinition {
    use super::super::tap_add_any_color;
    CardDefinition {
        name: "Command Tower",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_any_color()],
        ..Default::default()
    }
}

/// Reflecting Pool — Land. "{T}: Add one mana of any color a land you control
/// could produce." Wired faithfully via `ManaPayload::AnyColorYouCouldProduce`
/// (the controller-side mirror used by Star Compass): the legal-color set is
/// the union of basic-land types you control, falling back to colorless if you
/// control no basic-typed land.
pub fn reflecting_pool() -> CardDefinition {
    CardDefinition {
        name: "Reflecting Pool",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColorYouCouldProduce,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gaea's Cradle — Land. "{T}: Add {G} for each creature you control."
/// The variable amount uses `Value::CreatureCountControlledBy(You)`.
pub fn gaeas_cradle() -> CardDefinition {
    CardDefinition {
        name: "Gaea's Cradle",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::CreatureCountControlledBy(PlayerRef::You),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Karn's Bastion — Land. "{T}: Add {C}. {4}, {T}: Proliferate."
pub fn karns_bastion() -> CardDefinition {
    use super::super::tap_add_colorless;
    CardDefinition {
        name: "Karn's Bastion",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::new(vec![crate::mana::ManaSymbol::Generic(4)]),
                effect: Effect::Proliferate,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Verge lands (Foundations / Duskmourn) ──────────────────────────────────
// "{T}: Add {first color}. {T}: Add {second color}, only if you control a
// land of either color's basic type." Built via `super::super::verge_land`.

pub fn blazemire_verge() -> CardDefinition {
    super::super::verge_land(
        "Blazemire Verge", Color::Black, Color::Red, LandType::Swamp, LandType::Mountain,
    )
}

pub fn thornspire_verge() -> CardDefinition {
    super::super::verge_land(
        "Thornspire Verge", Color::Red, Color::Green, LandType::Mountain, LandType::Forest,
    )
}

pub fn bleachbone_verge() -> CardDefinition {
    super::super::verge_land(
        "Bleachbone Verge", Color::White, Color::Black, LandType::Plains, LandType::Swamp,
    )
}

pub fn riverpyre_verge() -> CardDefinition {
    super::super::verge_land(
        "Riverpyre Verge", Color::Blue, Color::Red, LandType::Island, LandType::Mountain,
    )
}

pub fn wastewood_verge() -> CardDefinition {
    super::super::verge_land(
        "Wastewood Verge", Color::Black, Color::Green, LandType::Swamp, LandType::Forest,
    )
}

pub fn floodfarm_verge() -> CardDefinition {
    super::super::verge_land(
        "Floodfarm Verge", Color::White, Color::Blue, LandType::Plains, LandType::Island,
    )
}

pub fn gloomlake_verge() -> CardDefinition {
    super::super::verge_land(
        "Gloomlake Verge", Color::Blue, Color::Black, LandType::Island, LandType::Swamp,
    )
}

pub fn hushwood_verge() -> CardDefinition {
    super::super::verge_land(
        "Hushwood Verge", Color::Green, Color::White, LandType::Forest, LandType::Plains,
    )
}

/// Urza's Saga — Enchantment Land — Urza's Saga. I: gains "{T}: Add {C}."
/// II: gains "{2}, {T}: Create a 0/0 Construct with '+1/+1 for each
/// artifact you control.'" III: search for an artifact with mana value 0
/// or 1 (non-X), put it onto the battlefield; the Saga then sacrifices.
pub fn urzas_saga() -> CardDefinition {
    use crate::card::{
        CreatureType, EnchantmentSubtype, SelectionRequirement, StaticAbility, TokenDefinition,
    };
    use crate::effect::{Selector, StaticEffect, ZoneDest};
    use crate::mana::{cost, generic};
    let construct = TokenDefinition {
        name: "Construct".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::Artifact,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Urza's Saga",
        card_types: vec![CardType::Enchantment, CardType::Land],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::GainActivatedAbility {
                    what: Selector::This,
                    ability: Box::new(ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::Colorless(Value::Const(1)),
                        },
                        ..Default::default()
                    }),
                },
            ),
            (
                2,
                Effect::GainActivatedAbility {
                    what: Selector::This,
                    ability: Box::new(ActivatedAbility {
                        tap_cost: true,
                        mana_cost: cost(&[generic(2)]),
                        effect: Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::Const(1),
                            definition: construct,
                        },
                        ..Default::default()
                    }),
                },
            ),
            (
                3,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::ManaValueAtMost(1))
                        .and(SelectionRequirement::HasXInCost.negate()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
        ],
        ..Default::default()
    }
}

pub fn sunbillow_verge() -> CardDefinition {
    super::super::verge_land(
        "Sunbillow Verge", Color::White, Color::Red, LandType::Mountain, LandType::Plains,
    )
}

pub fn willowrush_verge() -> CardDefinition {
    super::super::verge_land(
        "Willowrush Verge", Color::Blue, Color::Green, LandType::Forest, LandType::Island,
    )
}
