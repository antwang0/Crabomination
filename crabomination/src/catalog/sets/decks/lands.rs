//! Lands used by the BRG and Goryo's demo decks.
//!
//! Conditional ETB-tapped behaviors (shocklands, fastlands, pathways with
//! face choice) are stubbed in their simplest form: they enter untapped and
//! produce both colors via two separate mana abilities. This keeps the
//! decks playable while the engine grows the necessary primitives. Surveil
//! lands and tap lands enter tapped via a self-targeting `Tap` trigger.

use super::super::tap_add;
use crate::card::{
    CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec, LandType, Selector,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::{ActivatedAbility, ManaPayload, PlayerRef};
use crate::mana::{Color, ManaCost};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Triggered ability: when this permanent enters the battlefield, tap it.
fn etb_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Tap { what: Selector::This },
    }
}

/// Triggered ability: when this permanent enters, tap it AND surveil 1.
fn etb_tap_then_surveil_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
    }
}

/// Skeleton for a non-basic land with two color-producing mana abilities and
/// optionally an ETB-tapped trigger and the corresponding `LandType`s.
fn dual_land_with(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
    triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        triggered_abilities: triggers,
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

// ── Fastlands ────────────────────────────────────────────────────────────────
//
// Real Oracle: "ENTERS tapped unless you control two or fewer other lands."
// TODO: enforce the conditional ETB once the engine supports ETB-with-condition
// triggers. For now they enter untapped (which is the *good* case anyway).

pub fn blackcleave_cliffs() -> CardDefinition {
    dual_land_with(
        "Blackcleave Cliffs",
        LandType::Swamp,
        LandType::Mountain,
        Color::Black,
        Color::Red,
        vec![],
    )
}

pub fn blooming_marsh() -> CardDefinition {
    dual_land_with(
        "Blooming Marsh",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        vec![],
    )
}

pub fn copperline_gorge() -> CardDefinition {
    dual_land_with(
        "Copperline Gorge",
        LandType::Mountain,
        LandType::Forest,
        Color::Red,
        Color::Green,
        vec![],
    )
}

// ── Pathways ─────────────────────────────────────────────────────────────────
//
// Real Oracle: each face produces only one of the two colors and the player
// chooses a face on cast. With no MDFC support, both colors are exposed via
// separate mana abilities — gameplay-equivalent for a 60-card deck.
// TODO: gate behind a face-choice once MDFCs land.

pub fn blightstep_pathway() -> CardDefinition {
    dual_land_with(
        "Blightstep Pathway",
        LandType::Swamp,
        LandType::Mountain,
        Color::Black,
        Color::Red,
        vec![],
    )
}

pub fn darkbore_pathway() -> CardDefinition {
    dual_land_with(
        "Darkbore Pathway",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        vec![],
    )
}

// ── Shocklands ───────────────────────────────────────────────────────────────
//
// Real Oracle: "As this enters the battlefield, you may pay 2 life. If you
// don't, it enters tapped." Until the engine supports ETB-with-replacement
// choice, these enter untapped (the player would normally pay 2 life
// anyway). TODO: surface the pay-2-life decision.

pub fn godless_shrine() -> CardDefinition {
    dual_land_with(
        "Godless Shrine",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
        vec![],
    )
}

pub fn hallowed_fountain() -> CardDefinition {
    dual_land_with(
        "Hallowed Fountain",
        LandType::Plains,
        LandType::Island,
        Color::White,
        Color::Blue,
        vec![],
    )
}

pub fn watery_grave() -> CardDefinition {
    dual_land_with(
        "Watery Grave",
        LandType::Island,
        LandType::Swamp,
        Color::Blue,
        Color::Black,
        vec![],
    )
}

pub fn overgrown_tomb() -> CardDefinition {
    dual_land_with(
        "Overgrown Tomb",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        vec![],
    )
}

// ── Surveil lands (Murders at Karlov Manor) ─────────────────────────────────
//
// All surveil lands enter tapped and surveil 1 on ETB.

pub fn meticulous_archive() -> CardDefinition {
    dual_land_with(
        "Meticulous Archive",
        LandType::Plains,
        LandType::Island,
        Color::White,
        Color::Blue,
        vec![etb_tap_then_surveil_one()],
    )
}

pub fn shadowy_backstreet() -> CardDefinition {
    dual_land_with(
        "Shadowy Backstreet",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
        vec![etb_tap_then_surveil_one()],
    )
}

pub fn undercity_sewers() -> CardDefinition {
    dual_land_with(
        "Undercity Sewers",
        LandType::Island,
        LandType::Swamp,
        Color::Blue,
        Color::Black,
        vec![etb_tap_then_surveil_one()],
    )
}

// ── Special lands ────────────────────────────────────────────────────────────

/// Gemstone Mine — Land. ETB with three mining counters; tap, remove a
/// counter to add one mana of any color; sacrifice when last counter is
/// removed.
///
/// Stub: tap to add one mana of any color, no charge counters yet.
/// TODO: wire charge counters + sacrifice trigger.
pub fn gemstone_mine() -> CardDefinition {
    CardDefinition {
        name: "Gemstone Mine",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Gemstone Caverns — Legendary Land. Opening-hand: may put into play with a
/// luck counter. Tap to add a mana of any color, removing a luck counter.
///
/// Stub: simple "tap for any color" land; no opening-hand effect yet.
/// TODO: opening-hand pre-game install + luck counter mechanic.
pub fn gemstone_caverns() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Gemstone Caverns",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Cavern of Souls — Land. As it enters, choose a creature type. Tap for
/// colorless OR mana of any color usable only to cast a creature of the
/// chosen type, which can't be countered.
///
/// Stub: tap for colorless (no creature-type choice or uncounterable yet).
/// TODO: name-a-type ETB choice + uncounterable spell flag.
pub fn cavern_of_souls() -> CardDefinition {
    CardDefinition {
        name: "Cavern of Souls",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Cephalid Coliseum — Land. Tap to add {U}. {2}{U}, Tap, Sacrifice: each
/// player draws and discards three cards. (Threshold makes it draw + discard.)
///
/// Stub: tap for {U}; activated mill/draw ability omitted.
/// TODO: wire the {2}{U}, T, Sac mass-draw-and-discard ability.
pub fn cephalid_coliseum() -> CardDefinition {
    CardDefinition {
        name: "Cephalid Coliseum",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![etb_tap()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}
