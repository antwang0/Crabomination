//! One submodule per Magic set, named by the set's three-letter code.
//! Helpers shared across all set modules live here.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, EventKind, EventScope, EventSpec, LandType,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::Color;

pub fn tap_add(color: Color) -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![color]),
        },
        ..Default::default()
    }
}

/// Mana ability shorthand: `{T}: Add one mana of any color.` — rainbow
/// rocks (Mana Tower Crystal, Manalith). Player chooses the color at
/// activation time via the `ManaPayload::AnyOneColor(1)` payload.
pub fn tap_add_any_color() -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::AnyOneColor(Value::Const(1)),
        },
        ..Default::default()
    }
}

/// Mana ability shorthand: `{T}: Add {C}.` (one true colorless pip, not
/// generic). Used by colorless-only lands (Wastes, Petrified Hamlet) and
/// Eldrazi-aligned utility lands.
pub fn tap_add_colorless() -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colorless(Value::Const(1)),
        },
        ..Default::default()
    }
}

pub fn no_abilities() -> Vec<ActivatedAbility> {
    vec![]
}

/// Painland (the allied/enemy "Wastes/Reef/Forge" cycle): `{T}: Add {C}` plus
/// two `{T}: Add {color}, this land deals 1 damage to you` abilities. No basic
/// land types; enters untapped. Adarkar Wastes, Underground River, etc.
pub fn painland(name: &'static str, color_a: Color, color_b: Color) -> CardDefinition {
    let colored = |color: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::Seq(vec![
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![color]) },
            Effect::DealDamage { to: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), colored(color_a), colored(color_b)],
        ..Default::default()
    }
}

/// Mana ability shorthand: `{T}, Pay N life: Add {color}.` — the horizon-land
/// / painland cost line. The life is paid up front during activation.
pub fn tap_pay_life_add(color: Color, life: u32) -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        life_cost: life,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![color]),
        },
        ..Default::default()
    }
}

/// Horizon land (Future Sight / Modern Horizons cycle): two
/// `{T}, Pay 1 life: Add {color}` abilities plus
/// `{1}, {T}, Sacrifice this: Draw a card`. No basic land types.
pub fn horizon_land(name: &'static str, color_a: Color, color_b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_pay_life_add(color_a, 1),
            tap_pay_life_add(color_b, 1),
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                sac_cost: true,
                mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Verge land (Foundations / Duskmourn): `{T}: Add {uncond}` unconditionally,
/// and `{T}: Add {cond}` only while you control a `type_a` or `type_b` land.
pub fn verge_land(
    name: &'static str,
    uncond: Color,
    cond: Color,
    type_a: LandType,
    type_b: LandType,
) -> CardDefinition {
    let gated = ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![cond]),
        },
        condition: Some(Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(
                SelectionRequirement::HasLandType(type_a)
                    .or(SelectionRequirement::HasLandType(type_b))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            n: Value::Const(1),
        }),
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(uncond), gated],
        ..Default::default()
    }
}

// ── Land helpers shared across set modules ───────────────────────────────────

/// Triggered ability: when this permanent enters the battlefield, tap it.
pub fn etb_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Tap { what: Selector::This },
    }
}

/// Triggered ability: when this permanent enters, tap it AND surveil 1.
pub fn etb_tap_then_surveil_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
    }
}

/// Triggered ability: when this permanent enters, tap it AND gain 1 life
/// (the Khans "life-gain tapland" cycle — Tranquil Cove et al.).
pub fn etb_tap_then_gain_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]),
    }
}

/// Fastland ETB trigger: "ETB tapped unless you control two or fewer other
/// lands." Counted against the post-ETB battlefield (which already contains
/// this land), so the threshold is "≥ 4 lands you control".
pub fn fastland_etb_conditional_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(4),
            },
            then: Box::new(Effect::Tap { what: Selector::This }),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// Shock-land ETB choice — "As this enters, you may pay 2 life. If you don't,
/// it enters tapped." Modeled as a self-source ETB `ChooseMode` trigger
/// (mode 0 = pay 2 life, mode 1 = tap self). The default `AutoDecider` and
/// the simulated bot both pick mode 0, which matches typical play (a single
/// untap is almost always worth 2 life). Note: this is a triggered ability,
/// not a true replacement effect — the land is briefly available untapped
/// before the trigger resolves. Functionally close enough for the demo decks.
pub fn shockland_pay_two_or_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::ChooseMode(vec![
            // Mode 0: Pay 2 life, stay untapped.
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            // Mode 1: enter tapped.
            Effect::Tap { what: Selector::This },
        ]),
    }
}

/// Skeleton for a non-basic land with two color-producing mana abilities and
/// optionally an ETB-tapped trigger and the corresponding `LandType`s.
pub fn dual_land_with(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
    triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        triggered_abilities: triggers,
        ..Default::default()
    }
}

pub mod all_factories;
pub mod akh;
pub mod all;
pub mod ap;
pub mod arn;
pub mod dis;
pub mod fem;
pub mod gpt;
pub mod ice;
pub mod inv;
pub mod khm;
pub mod kld;
pub mod ktk;
pub mod lci;
pub mod lea;
pub mod m11;
pub mod mkm;
pub mod ogw;
pub mod pc2;
pub mod por;
pub mod rav;
pub mod rtr;
pub mod ths;
pub mod tmp;
pub mod zen;
pub mod decks;
pub mod mod_set;
pub mod sos;
pub mod stx;
pub mod xtra;
