//! Secrets of Strixhaven (SOS) — Lands.
//!
//! The SOS school lands all share the same template: enters tapped, taps
//! for one of two colors, and has a `{2}{C1}{C2}, {T}: Surveil 1`
//! activated ability gated by tap. The engine has the surveil primitive
//! already (see `Effect::Surveil`), so wiring these is straightforward.

use super::super::etb_tap;
use crate::card::{
    CardDefinition, CardType, Effect, LandType, Subtypes,
};
use crate::effect::{ActivatedAbility, PlayerRef, Value};
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, u, w};

/// Build a Strixhaven school land — enters tapped, two color-pip mana
/// abilities, and a `{2}{c1}{c2}, {T}: Surveil 1` ability.
fn school_land(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
    surveil_pips: [ManaSymbol; 2],
) -> CardDefinition {
    use super::super::tap_add;
    let surveil = ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(2), surveil_pips[0], surveil_pips[1]]),
        effect: Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(1),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 0,
        exile_gy_cost: 0,
    };
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
        activated_abilities: vec![tap_add(color_a), tap_add(color_b), surveil],
        triggered_abilities: vec![etb_tap()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Forum of Amity — Silverquill (W/B) school land.
pub fn forum_of_amity() -> CardDefinition {
    school_land(
        "Forum of Amity",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
        [w(), b()],
    )
}

/// Fields of Strife — Lorehold (R/W) school land.
pub fn fields_of_strife() -> CardDefinition {
    school_land(
        "Fields of Strife",
        LandType::Mountain,
        LandType::Plains,
        Color::Red,
        Color::White,
        [r(), w()],
    )
}

/// Paradox Gardens — Quandrix (G/U) school land.
pub fn paradox_gardens() -> CardDefinition {
    school_land(
        "Paradox Gardens",
        LandType::Forest,
        LandType::Island,
        Color::Green,
        Color::Blue,
        [g(), u()],
    )
}

/// Titan's Grave — Witherbloom (B/G) school land.
pub fn titans_grave() -> CardDefinition {
    school_land(
        "Titan's Grave",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        [b(), g()],
    )
}

/// Spectacle Summit — Prismari (U/R) school land.
pub fn spectacle_summit() -> CardDefinition {
    school_land(
        "Spectacle Summit",
        LandType::Island,
        LandType::Mountain,
        Color::Blue,
        Color::Red,
        [u(), r()],
    )
}

/// Great Hall of the Biblioplex — colorless legendary utility land.
///
/// Real Oracle: "{T}: Add {C}. / {T}, Pay 1 life: Add one mana of any
/// color. Spend this mana only to cast an instant or sorcery spell. /
/// {5}: If this land isn't a creature, it becomes a 2/4 Wizard creature
/// with 'Whenever you cast an instant or sorcery spell, this creature
/// gets +1/+0 until end of turn.' It's still a land."
///
/// Wired (push XV):
/// - `{T}: Add {C}` via the shared `tap_add_colorless` helper.
/// - `{T}, Pay 1 life: Add one mana of any color.` Cost is approximated
///   as `Effect::LoseLife(1)` chained with `Effect::AddMana(AnyOneColor 1)`
///   — the engine has no first-class life-cost primitive on
///   `ActivatedAbility` (tracked in TODO.md). The spend-only-on-IS
///   restriction is omitted (no per-pip mana metadata yet).
/// - `{5}: becomes 2/4 Wizard creature` clause is omitted (no
///   land-becomes-creature primitive — same gap as Mishra's Factory).
///   The vanilla mana-fixing role still slots into Strixhaven decks as
///   a colorless 5-color rainbow tap-with-life-payment.
pub fn great_hall_of_the_biblioplex() -> CardDefinition {
    use super::super::tap_add_colorless;
    use crate::card::Supertype;
    use crate::effect::{ActivatedAbility, ManaPayload};
    // Push XV: now uses the new `ActivatedAbility.life_cost` field for
    // the printed "Pay 1 life" cost. The effect is a pure mana ability
    // (`AddMana` only), so it qualifies as a true mana ability and
    // resolves immediately without going on the stack — matching MTG's
    // mana-ability rules. The life is paid up front (pre-flight gate
    // rejects activation if controller would drop to 0 life).
    let pay_life_for_any = ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::AnyOneColor(Value::Const(1)),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 1,
        exile_gy_cost: 0,
    };
    CardDefinition {
        name: "Great Hall of the Biblioplex",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add_colorless(), pay_life_for_any],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Skycoach Waypoint — colorless utility Land.
/// Real Oracle: "{T}: Add {C}. / {3}, {T}: Target creature becomes
/// prepared."
///
/// Approximation: the Prepare keyword is not yet a first-class engine
/// concept (see TODO.md "Prepare mechanic" — toggling a creature
/// `prepared` state requires creatures with their own Prepare-grant
/// abilities). The `{3},{T}: prepare a creature` activation is omitted;
/// the colorless tap-for-{C} mana ability is wired faithfully via the
/// shared `tap_add_colorless` helper.
///
/// Push XIX promotes the row from ⏳ to 🟡 on the Colorless table.
pub fn skycoach_waypoint() -> CardDefinition {
    use super::super::tap_add_colorless;
    CardDefinition {
        name: "Skycoach Waypoint",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add_colorless()],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Petrified Hamlet — Land.
/// Real Oracle: "When this land enters, choose a land card name. /
/// Activated abilities of sources with the chosen name can't be activated
/// unless they're mana abilities. / Lands with the chosen name have
/// '{T}: Add {C}.' / {T}: Add {C}."
///
/// Approximation: the engine has no "choose a card name" prompt, no name-
/// match selector primitive, and no static "abilities of source X with
/// name Y can't activate" filter. We collapse to the printed colorless
/// mana ability ({T}: Add {C}) so the card still slots into colorless
/// utility roles. The lock-out clause is omitted (tracked in TODO.md
/// under "Choose a Card Name" engine work). Mana ability is a faithful
/// {T}: Add {C} via the shared `tap_add_colorless` helper.
pub fn petrified_hamlet() -> CardDefinition {
    use super::super::tap_add_colorless;
    CardDefinition {
        name: "Petrified Hamlet",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add_colorless()],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}
