//! One submodule per Magic set, named by the set's three-letter code.
//! Helpers shared across all set modules live here.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, EventKind, EventScope, EventSpec, LandType,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, StaticAbility, StaticEffect};
use crate::mana::{Color, cost, generic, hybrid};

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

/// CR 903.4 mana shorthand: `{T}: Add one mana of any color in your
/// commander's color identity.` — Command Tower, Arcane Signet, Commander's
/// Sphere, Path of Ancestry. Outside a Commander game the controller has no
/// commander and the ability behaves as plain "any color"; see
/// `ManaPayload::AnyColorInCommanderIdentity`.
pub fn tap_add_commander_identity() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::AnyColorInCommanderIdentity,
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
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![color]),
            },
            Effect::DealDamage {
                to: Selector::You,
                amount: Value::Const(1),
            },
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

/// Hybrid filter land (the Shadowmoor allied / Eventide enemy cycle, ten
/// cards): `{T}: Add {C}.` plus `{A/B}, {T}: Add {A}{A}, {A}{B}, or {B}{B}.`
///
/// "Hybrid" in the name because [`pay_one_filter_land`] is the *other* cycle
/// that goes by "filter land" — two cycles, one nickname, and one of them had
/// already taken the bare name as a module-private helper.
///
/// **One** activated ability for the filter, not three. The printed card is a
/// single mana ability whose payout is chosen as it resolves, and
/// `ManaPayload::OfColors` is exactly that — two pips, each chosen from the
/// pair, whose three outcomes are the three printed options. Modelling it as
/// three abilities (one per payout) gives the same *set* of outcomes but a
/// land with four activated abilities instead of two, and forces the choice at
/// activation time rather than at resolution. No basic land types; enters
/// untapped.
pub fn hybrid_filter_land(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[hybrid(a, b)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![a, b], Value::Const(2)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Reveal-or-tapped land: "As this land enters, you may reveal a [X] card
/// from your hand. If you don't, this land enters tapped." plus
/// `{T}: Add {A} or {B}.`
///
/// Fifteen cards over three cycles that differ only in what `reveal` looks
/// for — the Shadows over Innistrad five and the Strixhaven "Snarl" five name
/// two **land types**, the Lorwyn five name a **creature type**. None of them
/// carries the land type it asks about as a subtype: the card is a plain
/// "Land", and the type is the reveal's filter, not its own.
///
/// ⚠ **A replacement, not a trigger.** "As this land enters …" is CR 614.12:
/// the effect modifies *how* the permanent enters, so the land is never on
/// the battlefield untapped. Thirteen of the fifteen shipped as an
/// `EntersBattlefield` trigger that tapped the land afterwards — which lets
/// its controller hold priority with the trigger on the stack and tap the
/// land for mana it should never have produced. The condition is "a matching
/// card is in your hand", which is what the engine's `IfRevealFromHand` also
/// resolved to: it peeks and always accepts, since declining only buys the
/// printed downside.
pub fn reveal_or_tapped_land(
    name: &'static str,
    description: &'static str,
    reveal: SelectionRequirement,
    color_a: Color,
    color_b: Color,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        static_abilities: vec![crate::effect::StaticAbility {
            description,
            effect: crate::effect::StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Hand,
                    filter: reveal,
                }),
            },
        }],
        ..Default::default()
    }
}

/// [`reveal_or_tapped_land`] over the two land types the SOI / Snarl ten ask
/// for ("reveal a Plains **or** Island card").
pub fn land_type_reveal_land(
    name: &'static str,
    description: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
) -> CardDefinition {
    reveal_or_tapped_land(
        name,
        description,
        SelectionRequirement::HasLandType(type_a).or(SelectionRequirement::HasLandType(type_b)),
        color_a,
        color_b,
    )
}

/// Pay-one filter land (the Odyssey allied / modern enemy cycle, ten cards):
/// a single `{1}, {T}: Add {A}{B}.` and nothing else. The other cycle that
/// goes by "filter land"; see [`hybrid_filter_land`] for the Shadowmoor one.
///
/// Two fixed pips, so `ManaPayload::Colors` — no choice is made, and nobody
/// is asked. The five allied ones shipped as a `Seq` of two
/// `OfColors(vec![one_color], 1)` adds, which routes a *choice* through
/// `chosen_mana_color` over a one-element palette: the same two pips, but a
/// decision where the card offers none.
pub fn pay_one_filter_land(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![a, b]),
            },
            ..Default::default()
        }],
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
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
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
/// CR 614.1c — "This land enters tapped" as the **replacement** it is.
///
/// The sibling of [`etb_tap`], and the one to reach for. An
/// `EntersBattlefield` trigger puts the land on the battlefield untapped,
/// puts the trigger on the stack and hands its controller priority: it can
/// tap the land for mana it should never have made, and its opponents see an
/// untapped land while deciding whether to respond.
/// `GameState::apply_enters_tapped_replacement` runs this inside the
/// battlefield hop instead, so the land is never on the battlefield untapped.
///
/// `scripts/audit_enters_tapped.py` is the census over the cards still on the
/// trigger (ENGINE_BACKLOG's forty-ninth find).
pub fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

/// ⚠ **The trigger form, and CR 614.1c says it is the wrong one** — see
/// [`enters_tapped`]. Kept only for the `etb_tap_then_*` siblings, whose
/// "then" half really is a trigger.
pub fn etb_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Tap {
            what: Selector::This,
        },
    }
}

/// Triggered ability: when this permanent enters, tap it AND surveil 1.
pub fn etb_tap_then_surveil_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: Selector::This,
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
    }
}

/// Triggered ability: when this permanent enters, tap it AND scry 1
/// (the Theros "scry tapland" cycle — Temple of Abandon et al.).
pub fn etb_tap_then_scry_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: Selector::This,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
    }
}

/// Triggered ability: when this permanent enters, tap it AND gain 1 life
/// (the Khans "life-gain tapland" cycle — Tranquil Cove et al.).
pub fn etb_tap_then_gain_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: Selector::This,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
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
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(4),
            },
            then: Box::new(Effect::Tap {
                what: Selector::This,
            }),
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
            Effect::Tap {
                what: Selector::This,
            },
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

/// A two-color land that taps for both colors and prints **no** basic land
/// types — the fastland / gainland / scryland shape. Same body as
/// [`dual_land_with`] without the subtypes.
///
/// ⚠ These twenty-five lands carried `land_types` until 2026-09-01, which
/// made a Temple of Epiphany fetchable by "search for an Island card", live
/// to islandwalk and counted for domain. Only a land whose printed type line
/// has the basic types (shocklands, true duals, surveil lands) uses
/// [`dual_land_with`].
pub fn dual_land_untyped(
    name: &'static str,
    color_a: Color,
    color_b: Color,
    triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        triggered_abilities: triggers,
        ..Default::default()
    }
}

/// Enters-tapped tri-land that taps for any of three colors (the Khans
/// "wedge" cycle — Sandsteppe Citadel et al.). Untyped (matching the print).
pub fn tri_land(name: &'static str, a: Color, b: Color, c: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(a), tap_add(b), tap_add(c)],
        static_abilities: vec![enters_tapped()],
        ..Default::default()
    }
}

pub mod akh;
pub mod all;
pub mod all2;
pub mod all_factories;
pub mod ap;
pub mod arn;
pub mod bfz;
pub mod bng;
pub mod bng2;
pub mod bng3;
pub mod apc;
pub mod apc2;
pub mod bot;
pub mod bro;
pub mod c21;
pub mod cmdr;
pub mod bok;
pub mod bok2;
pub mod sok;
pub mod sok2;
pub mod sok3;
pub mod ulg;
pub mod unf;
pub mod uds;
pub mod usg;
pub mod usg2;
pub mod usg3;
pub mod cn2;
pub mod cns;
pub mod cns2;
pub mod cns3;
pub mod chk;
pub mod clb;
pub mod chk2;
pub mod chk3;
pub mod curses;
pub mod decks;
pub mod dgm;
pub mod dis;
pub mod eoe;
pub mod eoe2;
pub mod fem;
pub mod fin;
pub mod fin2;
pub mod gpt;
pub mod gtc;
pub mod gtc10;
pub mod gtc11;
pub mod gtc12;
pub mod gtc13;
pub mod gtc14;
pub mod gtc15;
pub mod gtc16;
pub mod gtc17;
pub mod gtc2;
pub mod gtc3;
pub mod gtc4;
pub mod gtc5;
pub mod gtc6;
pub mod gtc7;
pub mod gtc8;
pub mod gtc9;
pub mod ice;
pub mod inv;
pub mod jou;
pub mod jud;
pub mod jud2;
pub mod vanguard;
pub mod jou2;
pub mod jou3;
pub mod khm;
pub mod kld;
pub mod ktk;
pub mod lci;
pub mod lea;
pub mod m11;
pub mod m15;
pub mod mh3;
pub mod mh3b;
pub mod mh3c;
pub mod mh3d;
pub mod mh3e;
pub mod mkm;
pub mod mkm2;
pub mod mmq;
pub mod mmq2;
pub mod mmq3;
pub mod mmq4;
pub mod mmq5;
pub mod mmq6;
pub mod nph;
pub mod nms;
pub mod nms2;
pub mod nms3;
pub mod nms4;
pub mod ody;
pub mod pls;
pub mod pls2;
pub mod pcy;
pub mod pip;
pub mod pcy2;
pub mod pcy3;
pub mod pcy4;
pub mod mir;
pub mod mir2;
pub mod mir3;
pub mod mir4;
pub mod mir5;
pub mod mod_set;
pub mod ogw;
pub mod arc;
pub mod ante;
pub mod leg;
pub mod leg2;
pub mod leg3;
pub mod leg4;
pub mod leg5;
pub mod leg6;
pub mod atq;
pub mod drk;
pub mod drk2;
pub mod ohop;
pub mod hml;
pub mod hml2;
pub mod hml3;
pub mod leg7;
pub mod lgn;
pub mod ons;
pub mod ons2;
pub mod ons3;
pub mod ons4;
pub mod one;
pub mod pc2;
pub mod por;
pub mod rav;
pub mod rna;
pub mod rna2;
pub mod rtr;
pub mod mbs;
pub mod scg;
pub mod scg2;
pub mod shm;
pub mod sos;
pub mod exo;
pub mod exo2;
pub mod vis;
pub mod csp;
pub mod vis2;
pub mod wth;
pub mod wth2;
pub mod sth;
pub mod stx;
pub mod thb;
pub mod tor;
pub mod tor2;
pub mod ths;
pub mod tmp;
pub mod war;
pub mod zen2;
pub mod zen3;
pub mod wwk;
pub mod wwk2;
pub mod xtra;
pub mod zen;
