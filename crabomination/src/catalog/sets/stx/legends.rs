//! Iconic Strixhaven legendary creatures — the five college-head Dragons
//! plus a handful of additional set-defining legends.
//!
//! Most ship as faithfully-statted bodies: cost, P/T, supertypes, keywords,
//! and creature types are correct so the cards play, are blockable, and
//! feed catalog filtering. The full college-head ETBs (mana-of-each-color
//! Galazeth, life-payment-untap Beledros, library-trigger Tanazir,
//! attack-trigger Velomachus, choose-2-of-3 Shadrix) need engine features
//! that don't exist yet (mana-from-tap-target, mass-untap with cost, etc.)
//! — those gate to 🟡 in `STRIXHAVEN2.md`.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes, Supertype,
};
use crate::mana::{b, cost, g, generic, r, u, w};

// ── Galazeth Prismari (U/R) ─────────────────────────────────────────────────

/// Galazeth Prismari — {2}{U}{R}, 3/4 Legendary Dragon Wizard. Real Oracle:
/// "Flying / When Galazeth Prismari enters the battlefield, create a
/// Treasure token. / Artifacts you control have '{T}: Add one mana of any
/// color.'"
///
/// We ship the body + Flying + the ETB Treasure token via the existing
/// Treasure helper. The "artifacts you control are mana sources" static is
/// 🟡 — needs a static `GrantActivatedAbility(applies_to: Selector)`
/// primitive (untracked artifacts gaining a mana ability is also tricky
/// today since `tap_for_mana` walks each card's defined activated abilities
/// directly).
pub fn galazeth_prismari() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility, Value};
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Galazeth Prismari",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // ETB: create a Treasure token. (The "artifacts tap for any color"
        // static is omitted — see doc comment.)
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Beledros Witherbloom (B/G) ─────────────────────────────────────────────

/// Beledros Witherbloom — {3}{B}{B}{G}{G}, 6/6 Legendary Demon. Flying,
/// trample, lifelink. Activated: "Pay 10 life: Untap each land you
/// control. Activate only as a sorcery."
///
/// Fully wired: the activation uses the existing `life_cost: 10` field
/// + `sorcery_speed: true` gate + `Effect::Untap { what:
/// EachPermanent(Land & ControlledByYou), up_to: None }`. The
/// pre-flight life-cost gate (`GameError::InsufficientLife`) means the
/// activation is rejected cleanly when the controller can't pay 10
/// life. Existing `up_to: None` keeps the printed "untap each land"
/// semantics (cap-free mass untap).
pub fn beledros_witherbloom() -> CardDefinition {
    use crate::card::{ActivatedAbility, Selector, SelectionRequirement};
    CardDefinition {
        name: "Beledros Witherbloom",
        cost: cost(&[generic(3), b(), b(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: crate::mana::cost(&[]),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 10,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Velomachus Lorehold (R/W) ──────────────────────────────────────────────

/// Velomachus Lorehold — {3}{R}{R}{W}, 5/5 Legendary Dragon. Flying,
/// vigilance, haste. Real Oracle attack-trigger reveals top X cards and
/// lets you cast one with mana value ≤ X. Body wired with three keywords;
/// the cast-from-exile-without-paying attack-trigger is 🟡.
pub fn velomachus_lorehold() -> CardDefinition {
    CardDefinition {
        name: "Velomachus Lorehold",
        cost: cost(&[generic(3), r(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Tanazir Quandrix (G/U) ─────────────────────────────────────────────────

/// Tanazir Quandrix — {2}{G}{G}{U}{U}, 5/5 Legendary Dragon. Flying,
/// trample. Real Oracle ETB doubles +1/+1 counters on each creature you
/// control; attack trigger doubles toughness of target creature you
/// control.
///
/// Body + Flying + Trample wired. The ETB counter-doubling is still 🟡
/// (no counter-multiplier primitive). The **attack trigger** is now
/// wired via `Effect::PumpPT` with `toughness: Value::ToughnessOf(Target(0))`
/// — pumping the target's toughness by its current value cleanly doubles
/// it (printed toughness `T` becomes `T + T = 2T` until end of turn).
/// Auto-target picker prefers a friendly high-toughness creature for
/// the pump.
pub fn tanazir_quandrix() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, Selector, TriggeredAbility, Value};
    use crate::effect::Duration;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Tanazir Quandrix",
        cost: cost(&[generic(2), g(), g(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Shadrix Silverquill (W/B) ──────────────────────────────────────────────

/// Shadrix Silverquill — {2}{W}{B}, 4/4 Legendary Dragon. Flying, double
/// strike. Real Oracle attack-trigger: choose two among three modes (you
/// or target opp), each granting flavor-specific effects. Body wired with
/// flying + double strike; choose-2-of-3 mode picker is 🟡.
pub fn shadrix_silverquill() -> CardDefinition {
    CardDefinition {
        name: "Shadrix Silverquill",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
