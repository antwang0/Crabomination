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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Beledros Witherbloom (B/G) ─────────────────────────────────────────────

/// Beledros Witherbloom — {3}{B}{B}{G}{G}, 6/6 Legendary Demon. Flying,
/// trample, lifelink. "Pay 10 life: Untap each land you control. Activate
/// only as a sorcery."
///
/// Now wired (push XX) via push XV's `ActivatedAbility.life_cost: u32`
/// gate plus `Effect::Untap` over `Selector::EachPermanent(Land &
/// ControlledByYou)` — the selector iterates every land you control on
/// the battlefield, pre-flight life-pay rejects activation cleanly when
/// life < 10 (`GameError::InsufficientLife`), and `sorcery_speed: true`
/// matches the printed activation restriction. The ability's body is
/// `Effect::Untap { up_to: None }` which untaps every matching
/// permanent (as opposed to "up to N" — Beledros's printed wording is
/// "untap each land you control", uncapped).
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
            mana_cost: cost(&[]),
            sac_cost: false,
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
            once_per_turn: false,
            sorcery_speed: true,
            condition: None,
            life_cost: 10,
            exile_gy_cost: 0,
        }],
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Tanazir Quandrix (G/U) ─────────────────────────────────────────────────

/// Tanazir Quandrix — {2}{G}{G}{U}{U}, 5/5 Legendary Dragon. Flying,
/// trample. Real Oracle ETB doubles +1/+1 counters on each creature you
/// control; attack trigger doubles toughness of target creature you
/// control. Body wired; counter-doubling is 🟡 (needs counter-multiplier
/// primitive).
pub fn tanazir_quandrix() -> CardDefinition {
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

// ── Shadrix Silverquill (W/B) ──────────────────────────────────────────────

/// Shadrix Silverquill — {2}{W}{B}, 4/4 Legendary Dragon. Flying, double
/// strike. Printed Oracle attack-trigger:
///   "Whenever Shadrix Silverquill attacks, choose two different modes —
///    • You draw a card.
///    • Target opponent loses 2 life.
///    • Put a +1/+1 counter on target creature."
///
/// Push XXXVII: ✅ — the choose-2-of-3 attack trigger now wires faithfully
/// via `Effect::ChooseModes { count: 2 }` re-used at trigger resolution
/// time. AutoDecider picks modes 0+1 (draw + drain — the canonical "value"
/// pair, no targeting friction). ScriptedDecider drives mode pairs that
/// involve targeting (mode 1 needs an opp; mode 2 needs a creature).
pub fn shadrix_silverquill() -> CardDefinition {
    use crate::card::{
        CounterType, EventKind, EventScope, EventSpec, Selector, SelectionRequirement,
        TriggeredAbility, Value,
    };
    use crate::effect::shortcut::target_filtered;
    use crate::effect::PlayerRef;
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ChooseModes {
                count: 2,
                up_to: false,
                allow_duplicates: false,
                modes: vec![
                    // Mode 0: You draw a card.
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    // Mode 1: Target opponent loses 2 life.
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                    // Mode 2: Put a +1/+1 counter on target creature.
                    Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                ],
            },
        }],
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
