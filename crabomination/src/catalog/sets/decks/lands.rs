//! Lands used by the BRG and Goryo's demo decks.
//!
//! Conditional ETB-tapped behaviors (shocklands, fastlands, pathways with
//! face choice) are stubbed in their simplest form: they enter untapped and
//! produce both colors via two separate mana abilities. This keeps the
//! decks playable while the engine grows the necessary primitives. Surveil
//! lands and tap lands enter tapped via a self-targeting `Tap` trigger.

use super::super::tap_add;
use crate::card::{
    CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec, LandType,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{ActivatedAbility, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, ManaCost, cost, generic, u};

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

/// Fastland ETB trigger: "ETB tapped unless you control two or fewer other
/// lands." Counted against the post-ETB battlefield (which already contains
/// this land), so the threshold is "≥ 4 lands you control".
fn fastland_etb_conditional_tap() -> TriggeredAbility {
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
fn shockland_pay_two_or_tap() -> TriggeredAbility {
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
        back_face: None,
    }
}

// ── Fastlands ────────────────────────────────────────────────────────────────
//
// Real Oracle: "ENTERS tapped unless you control two or fewer other lands."
// `fastland_etb_conditional_tap` evaluates the post-ETB land count: if you
// already control 4+ lands (this land plus 3+ others) it taps itself.

pub fn blackcleave_cliffs() -> CardDefinition {
    dual_land_with(
        "Blackcleave Cliffs",
        LandType::Swamp,
        LandType::Mountain,
        Color::Black,
        Color::Red,
        vec![fastland_etb_conditional_tap()],
    )
}

pub fn blooming_marsh() -> CardDefinition {
    dual_land_with(
        "Blooming Marsh",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        vec![fastland_etb_conditional_tap()],
    )
}

pub fn copperline_gorge() -> CardDefinition {
    dual_land_with(
        "Copperline Gorge",
        LandType::Mountain,
        LandType::Forest,
        Color::Red,
        Color::Green,
        vec![fastland_etb_conditional_tap()],
    )
}

// ── Pathways ─────────────────────────────────────────────────────────────────
//
// Real Oracle: a Modal-Double-Faced-Card with a single-color land on each
// face — the player picks a face when playing the card from hand. We model
// this with `CardDefinition.back_face`: each pathway's *front* definition
// lists the front face's land type / mana ability and stamps the back face's
// definition into `back_face`. The default `GameAction::PlayLand(id)` plays
// the front face; `GameAction::PlayLandBack(id)` plays the back face (the
// engine swaps the `CardInstance.definition` to the back face's definition
// before placing on battlefield).

/// Single-color basic-typed land face (no ETB-tap, no triggers).
fn pathway_face(name: &'static str, land_type: LandType, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![land_type],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Build an MDFC pathway from front-face and back-face descriptors.
fn pathway(
    front_name: &'static str,
    front_type: LandType,
    front_color: Color,
    back_name: &'static str,
    back_type: LandType,
    back_color: Color,
) -> CardDefinition {
    let mut front = pathway_face(front_name, front_type, front_color);
    front.back_face = Some(Box::new(pathway_face(back_name, back_type, back_color)));
    front
}

/// Blightstep Pathway // Searstep Pathway — B/R MDFC. Front face is a Swamp
/// that taps for {B}; back face (Searstep Pathway) is a Mountain that taps
/// for {R}. Played via `PlayLand(id)` (front) or `PlayLandBack(id)` (back).
pub fn blightstep_pathway() -> CardDefinition {
    pathway(
        "Blightstep Pathway", LandType::Swamp, Color::Black,
        "Searstep Pathway",   LandType::Mountain, Color::Red,
    )
}

/// Darkbore Pathway // Slitherbore Pathway — B/G MDFC. Front is a Swamp
/// for {B}; back (Slitherbore Pathway) is a Forest for {G}.
pub fn darkbore_pathway() -> CardDefinition {
    pathway(
        "Darkbore Pathway",    LandType::Swamp, Color::Black,
        "Slitherbore Pathway", LandType::Forest, Color::Green,
    )
}

// ── Shocklands ───────────────────────────────────────────────────────────────
//
// Real Oracle: "As this enters the battlefield, you may pay 2 life. If you
// don't, it enters tapped." Modeled as a self-source ETB `ChooseMode` trigger
// (mode 0 pays 2 life, mode 1 taps the land). The engine's `ChooseMode`
// picks mode 0 by default for non-UI players, matching typical play.

pub fn godless_shrine() -> CardDefinition {
    dual_land_with(
        "Godless Shrine",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
        vec![shockland_pay_two_or_tap()],
    )
}

pub fn hallowed_fountain() -> CardDefinition {
    dual_land_with(
        "Hallowed Fountain",
        LandType::Plains,
        LandType::Island,
        Color::White,
        Color::Blue,
        vec![shockland_pay_two_or_tap()],
    )
}

pub fn watery_grave() -> CardDefinition {
    dual_land_with(
        "Watery Grave",
        LandType::Island,
        LandType::Swamp,
        Color::Blue,
        Color::Black,
        vec![shockland_pay_two_or_tap()],
    )
}

pub fn overgrown_tomb() -> CardDefinition {
    dual_land_with(
        "Overgrown Tomb",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        vec![shockland_pay_two_or_tap()],
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

/// Gemstone Mine — Land. "Gemstone Mine enters with three mining counters
/// on it. {T}, Remove a mining counter from Gemstone Mine: Add one mana of
/// any color. If there are no mining counters on Gemstone Mine, sacrifice
/// it."
///
/// Modeled with a self-source ETB trigger that adds 3 charge counters
/// (engine has no `Mining` counter, so `Charge` stands in — gameplay-
/// equivalent for any non-proliferate interactions). The activated ability
/// folds the "remove a counter" cost into the resolved effect: remove → add
/// mana of any color → if no counters left, sacrifice. With the natural
/// progression (3 → 2 → 1 → 0 + sac), this taps the land for three mana
/// total before it dies.
pub fn gemstone_mine() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::ZoneDest;
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
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(1),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                Effect::If {
                    cond: Predicate::ValueAtMost(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                        Value::Const(0),
                    ),
                    then: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Graveyard,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::Const(3),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
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
        back_face: None,
    }
}

/// Cavern of Souls — Land. As it enters, choose a creature type. Tap for
/// colorless OR mana of any color usable only to cast a creature of the
/// chosen type, which can't be countered.
///
/// Approximation: taps for colorless mana, and the cast paths in
/// `actions.rs` flag any creature spell cast by a player who controls a
/// Cavern as `StackItem::Spell.uncounterable = true` — `CounterSpell`
/// skips those. The "name a type" / mana-provenance restriction is
/// collapsed: any creature spell becomes uncounterable while you control
/// any Cavern. Acceptable for the demo deck. TODO: full name-a-type
/// decision + per-cast tagging via mana provenance.
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
        back_face: None,
    }
}

/// Cephalid Coliseum — Land. Cephalid Coliseum enters tapped. Tap to add {U}.
/// "{2}{U}, {T}, Sacrifice Cephalid Coliseum: Each player draws three cards,
/// then discards three cards." (The Oracle has a threshold clause; we ship
/// the post-threshold version since the demo deck wants it as a graveyard
/// enabler.) The sacrifice is folded into the resolved effect via a `Move`
/// to graveyard before the draw / discard fires — a faithful approximation
/// since the only non-cost interaction it changes is "destroy in response
/// before sacrifice", which the bot/UI never attempts.
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
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::Seq(vec![
                    // Sacrifice as additional cost — modelled as the first
                    // step of the resolved effect (the bot never tries to
                    // respond to the trigger anyway).
                    Effect::Move {
                        what: Selector::This,
                        to: crate::effect::ZoneDest::Graveyard,
                    },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(3),
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(3),
                        random: false,
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
            },
        ],
        triggered_abilities: vec![etb_tap()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}
