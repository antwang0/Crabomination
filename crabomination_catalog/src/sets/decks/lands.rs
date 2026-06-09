//! Lands used by the BRG and Goryo's demo decks.
//!
//! Conditional ETB-tapped behaviors (shocklands, fastlands, pathways with
//! face choice) are stubbed in their simplest form: they enter untapped and
//! produce both colors via two separate mana abilities. This keeps the
//! decks playable while the engine grows the necessary primitives. Surveil
//! lands and tap lands enter tapped via a self-targeting `Tap` trigger.

use super::super::{
    dual_land_with, etb_tap, etb_tap_then_gain_one, etb_tap_then_surveil_one,
    fastland_etb_conditional_tap, painland, shockland_pay_two_or_tap, tap_add, tap_add_colorless,
};
use crate::card::{
    CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec, LandType,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w};

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
        ..Default::default()
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

/// Branchloft Pathway // Boulderloft Pathway — G/W MDFC.
pub fn branchloft_pathway() -> CardDefinition {
    pathway(
        "Branchloft Pathway", LandType::Forest, Color::Green,
        "Boulderloft Pathway", LandType::Plains, Color::White,
    )
}

/// Clearwater Pathway // Murkwater Pathway — U/B MDFC.
pub fn clearwater_pathway() -> CardDefinition {
    pathway(
        "Clearwater Pathway", LandType::Island, Color::Blue,
        "Murkwater Pathway", LandType::Swamp, Color::Black,
    )
}

/// Cragcrown Pathway // Timbercrown Pathway — R/G MDFC.
pub fn cragcrown_pathway() -> CardDefinition {
    pathway(
        "Cragcrown Pathway", LandType::Mountain, Color::Red,
        "Timbercrown Pathway", LandType::Forest, Color::Green,
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
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
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
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

/// Gemstone Caverns — Legendary Land. "If Gemstone Caverns is in your
/// opening hand and you're not the starting player, you may begin the game
/// with Gemstone Caverns on the battlefield with a luck counter on it. If
/// you do, exile a card from your hand. {T}, Remove a luck counter from
/// Gemstone Caverns: Add one mana of any color. {T}: Add {C}."
///
/// Wired:
///   * `opening_hand: Some(StartInPlay { tapped: false, extra: AddCounter Luck })`
///     — `apply_opening_hand_effects` puts the land in play untapped with a
///     luck counter for **any** player who has it in their opening hand
///     (the starting-player restriction and the "exile a card" cost are
///     skipped — acceptable for the demo).
///   * Two activated abilities: `{T}: Add {C}` and the `{T}, RemoveCounter
///     Luck → Add 1 of any color`. The luck-counter ability gates its
///     mana-add behind an `If` over the counter total, so once the counter
///     is gone the ability still taps but produces nothing.
pub fn gemstone_caverns() -> CardDefinition {
    use crate::card::{CounterType, Supertype};
    use crate::effect::OpeningHandEffect;
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
        activated_abilities: vec![
            // {T}: Add {C}.
            ActivatedAbility {
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
            },
            // {T}, Remove a luck counter: Add one mana of any color.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                        Value::Const(1),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Charge,
                            amount: Value::Const(1),
                        },
                        Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::AnyOneColor(Value::Const(1)),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
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
            },
        ],
        triggered_abilities: vec![],
        opening_hand: Some(OpeningHandEffect::StartInPlay {
            tapped: false,
            // The engine has no dedicated `Luck` counter type, so we reuse
            // `Charge` — gameplay-equivalent here since only the
            // luck-removal ability reads it.
            extra: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::Const(1),
            },
        }),
        ..Default::default()
    }
}

/// Cavern of Souls — Land. As Cavern of Souls enters, choose a creature
/// type. {T}: Add {C}. {T}: Add one mana of any color. Spend this mana
/// only to cast a creature spell of the chosen type, and that spell
/// can't be countered.
///
/// Approximations:
///
/// - **Name-a-type ETB**: a self-source `ChooseMode` ETB trigger picks
///   one of the demo decks' relevant types (Eldrazi / Demon / Sphinx /
///   Frog / Phyrexian / Angel / Avatar / Beast). The chosen type is
///   discarded after the trigger resolves; the engine doesn't yet wire
///   per-cast mana provenance, so the actual "which creatures are
///   uncounterable" check still collapses to "any creature you cast"
///   while you control a Cavern (see `caster_grants_uncounterable` in
///   `actions.rs`). The mode pick keeps the modal-decision round-trip
///   available to the UI.
/// - **Activated mana**: only the `{T}: Add {C}` half is wired. The
///   uncounterable flag still fires correctly for creature spells via
///   the simplified rule.
pub fn cavern_of_souls() -> CardDefinition {
    use crate::card::TriggeredAbility;
    CardDefinition {
        name: "Cavern of Souls",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // "As ~ enters, choose a creature type." `NameCreatureType`
            // surfaces a `Decision::ChooseCreatureType` to the controller
            // (AutoDecider picks Demon, matching the demo Goryo's deck's
            // Griselbrand). The chosen type is stored on the Cavern's
            // `CardInstance.chosen_creature_type` and consulted by
            // `caster_grants_uncounterable` to gate which creature spells
            // the Cavern actually protects.
            effect: Effect::NameCreatureType {
                what: Selector::This,
            },
        }],
        // The "creature spells of the chosen type can't be countered"
        // half. `caster_grants_uncounterable_with_x` scans for this
        // static instead of matching on the printed name, then reads
        // `chosen_creature_type` off the permanent for the gate.
        static_abilities: vec![crate::card::StaticAbility {
            description:
                "Creature spells you cast of the chosen type can't be countered.",
            effect: crate::effect::StaticEffect::UncounterableCreaturesOfChosenType,
        }],
        ..Default::default()
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
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
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
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
        ],
        triggered_abilities: vec![etb_tap()],
        ..Default::default()
    }
}

/// Shelldock Isle — Legendary Land. Enters tapped. Hideaway 4 (CR 702.76). `{T}:
/// Add {U}.` `{U}, {T}: You may play the exiled card without paying its mana
/// cost if a player has 20 or less life.`
pub fn shelldock_isle() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Shelldock Isle",
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u()]),
                condition: Some(Predicate::PlayerLifeAtMost {
                    who: PlayerRef::EachPlayer,
                    life: 20,
                }),
                effect: Effect::CastWithoutPayingImmediate {
                    what: Selector::CardExiledWithSource,
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                },
                ..Default::default()
            },
        ],
        // Enters tapped + Hideaway 4 on ETB.
        triggered_abilities: vec![
            etb_tap(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Hideaway { count: Value::Const(4) },
            },
        ],
        ..Default::default()
    }
}

/// Shared body for the Lorwyn hideaway lands: enters tapped, Hideaway 4,
/// `{T}: Add {color}` + `{color}, {T}: play the hidden card free if `gate`.
fn lorwyn_hideaway_land(
    name: &'static str,
    color: Color,
    pip: crate::mana::ManaSymbol,
    gate: Predicate,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add(color),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[pip]),
                condition: Some(gate),
                effect: Effect::CastWithoutPayingImmediate {
                    what: Selector::CardExiledWithSource,
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![
            etb_tap(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Hideaway { count: Value::Const(4) },
            },
        ],
        ..Default::default()
    }
}

/// Mosswort Bridge — hideaway land; plays the hidden card if creatures you
/// control have total power 8 or greater.
pub fn mosswort_bridge() -> CardDefinition {
    lorwyn_hideaway_land(
        "Mosswort Bridge",
        Color::Green,
        g(),
        Predicate::ValueAtLeast(
            Value::PowerOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ))),
            Value::Const(8),
        ),
    )
}

/// Spinerock Knoll — hideaway land; plays the hidden card if an opponent lost
/// 7 or more life this turn.
pub fn spinerock_knoll() -> CardDefinition {
    lorwyn_hideaway_land(
        "Spinerock Knoll",
        Color::Red,
        r(),
        Predicate::ValueAtLeast(
            Value::LifeLostThisTurn(PlayerRef::EachOpponent),
            Value::Const(7),
        ),
    )
}

/// Windbrisk Heights — hideaway land; plays the hidden card if you attacked
/// with three or more creatures this turn.
pub fn windbrisk_heights() -> CardDefinition {
    lorwyn_hideaway_land(
        "Windbrisk Heights",
        Color::White,
        w(),
        Predicate::ValueAtLeast(
            Value::CreaturesAttackedWithThisTurn(PlayerRef::You),
            Value::Const(3),
        ),
    )
}

/// Vesuva — Land. Enters tapped as a copy of any land on the battlefield
/// (CR 707; the copy persists while Vesuva remains, via `BecomeCopyOfFor`
/// with a Permanent duration — it reverts to printed Vesuva on leaving).
pub fn vesuva() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Vesuva",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![
            etb_tap(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(
                        SelectionRequirement::Land.and(SelectionRequirement::OtherThanSource),
                    ),
                    duration: Duration::Permanent,
                    non_legendary: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Thespian's Stage — Land. `{T}: Add {C}`; `{2}, {T}`: becomes a copy of
/// target land. (The printed "except it has this ability" rider is dropped —
/// the copy loses the re-copy ability.)
pub fn thespians_stage() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Thespian's Stage",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(
                        SelectionRequirement::Land.and(SelectionRequirement::OtherThanSource),
                    ),
                    duration: Duration::Permanent,
                    non_legendary: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Manlands (CR 702 — lands that animate into creatures) ───────────────────

/// Build a creature-land (manland): enters tapped, taps for each of two
/// colors, and has a mana-cost activated ability that animates it into a
/// creature until end of turn via `Effect::BecomeCreature`. The animated
/// body keeps the Land type (it becomes a "land creature").
#[allow(clippy::too_many_arguments)]
fn manland(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
    animate_cost: ManaCost,
    power: i32,
    toughness: i32,
    keywords: Vec<crate::card::Keyword>,
) -> CardDefinition {
    use crate::card::CreatureType;
    let animate = ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: false,
        mana_cost: animate_cost,
        effect: Effect::BecomeCreature {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            creature_types: vec![CreatureType::Elemental],
            keywords,
            duration: crate::effect::Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        activated_abilities: vec![tap_add(color_a), tap_add(color_b), animate],
        triggered_abilities: vec![etb_tap()],
        ..Default::default()
    }
}

/// Celestial Colonnade — UW manland. Enters tapped, taps for {W}/{U}.
/// {3}{W}{U}: becomes a 4/4 white-blue Elemental with flying and vigilance
/// until end of turn (still a land).
pub fn celestial_colonnade() -> CardDefinition {
    use crate::card::Keyword;
    manland(
        "Celestial Colonnade",
        LandType::Plains,
        LandType::Island,
        Color::White,
        Color::Blue,
        cost(&[generic(3), crate::mana::w(), u()]),
        4,
        4,
        vec![Keyword::Flying, Keyword::Vigilance],
    )
}

/// Creeping Tar Pit — UB manland. Enters tapped, taps for {U}/{B}.
/// {1}{U}{B}: becomes a 3/2 blue-black Elemental that can't be blocked
/// until end of turn (still a land).
pub fn creeping_tar_pit() -> CardDefinition {
    use crate::card::Keyword;
    manland(
        "Creeping Tar Pit",
        LandType::Island,
        LandType::Swamp,
        Color::Blue,
        Color::Black,
        cost(&[generic(1), u(), crate::mana::b()]),
        3,
        2,
        vec![Keyword::Unblockable],
    )
}

/// Hissing Quagmire — BG manland. Enters tapped, taps for {B}/{G}.
/// {1}{B}{G}: becomes a 2/2 black-green Elemental with deathtouch until end
/// of turn (still a land).
pub fn hissing_quagmire() -> CardDefinition {
    use crate::card::Keyword;
    manland(
        "Hissing Quagmire",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        cost(&[generic(1), crate::mana::b(), crate::mana::g()]),
        2,
        2,
        vec![Keyword::Deathtouch],
    )
}

/// Needle Spires — RW manland. Enters tapped, taps for {R}/{W}.
/// {2}{R}{W}: becomes a 2/1 red-white Elemental with double strike until end
/// of turn (still a land).
pub fn needle_spires() -> CardDefinition {
    use crate::card::Keyword;
    manland(
        "Needle Spires",
        LandType::Mountain,
        LandType::Plains,
        Color::Red,
        Color::White,
        cost(&[generic(2), crate::mana::r(), crate::mana::w()]),
        2,
        1,
        vec![Keyword::DoubleStrike],
    )
}

// ── Colorless creature-lands ({T}: Add {C}; animate into a creature) ─────────

/// Build a colorless manland: enters untapped, `{T}: Add {C}`, and a
/// mana-cost ability that animates it into a creature until end of turn. The
/// "artifact creature" half of Mishra's/Inkmoth/Blinkmoth is approximated as a
/// plain creature (no artifact-type add yet).
fn colorless_manland(
    name: &'static str,
    animate_cost: ManaCost,
    power: i32,
    toughness: i32,
    creature_types: Vec<crate::card::CreatureType>,
    keywords: Vec<crate::card::Keyword>,
) -> CardDefinition {
    let animate = ActivatedAbility {
        mana_cost: animate_cost,
        effect: Effect::BecomeCreature {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            creature_types,
            keywords,
            duration: crate::effect::Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), animate],
        ..Default::default()
    }
}

/// Mutavault — `{T}: Add {C}`. `{1}`: becomes a 2/2 creature with all creature
/// types (Changeling) until end of turn (still a land).
pub fn mutavault() -> CardDefinition {
    use crate::card::Keyword;
    colorless_manland("Mutavault", cost(&[generic(1)]), 2, 2, vec![], vec![Keyword::Changeling])
}

/// Mishra's Factory — `{T}: Add {C}`. `{1}`: becomes a 2/2 Assembly-Worker
/// until end of turn (still a land). (The +1/+1 Assembly-Worker pump rider is
/// dropped.)
pub fn mishras_factory() -> CardDefinition {
    use crate::card::CreatureType;
    colorless_manland(
        "Mishra's Factory", cost(&[generic(1)]), 2, 2,
        vec![CreatureType::AssemblyWorker], vec![],
    )
}

/// Inkmoth Nexus — `{T}: Add {C}`. `{1}`: becomes a 1/1 Blinkmoth with flying
/// and infect until end of turn (still a land).
pub fn inkmoth_nexus() -> CardDefinition {
    use crate::card::{CreatureType, Keyword};
    colorless_manland(
        "Inkmoth Nexus", cost(&[generic(1)]), 1, 1,
        vec![CreatureType::Blinkmoth], vec![Keyword::Flying, Keyword::Infect],
    )
}

/// Blinkmoth Nexus — `{T}: Add {C}`. `{1}`: becomes a 1/1 Blinkmoth with flying
/// until end of turn (still a land). (The pump ability is dropped.)
pub fn blinkmoth_nexus() -> CardDefinition {
    use crate::card::{CreatureType, Keyword};
    colorless_manland(
        "Blinkmoth Nexus", cost(&[generic(1)]), 1, 1,
        vec![CreatureType::Blinkmoth], vec![Keyword::Flying],
    )
}

/// Build a Restless creature-land (MOM/LCI cycle): enters tapped, taps for two
/// colors, animates into a specific creature for a cost, and has an on-attack
/// trigger. The lands carry no basic land types (typeless), unlike `manland`.
#[allow(clippy::too_many_arguments)]
fn restless_land(
    name: &'static str,
    color_a: Color,
    color_b: Color,
    animate_cost: ManaCost,
    power: i32,
    toughness: i32,
    creature_types: Vec<crate::card::CreatureType>,
    keywords: Vec<crate::card::Keyword>,
    attack_effect: Effect,
) -> CardDefinition {
    use crate::effect::Duration;
    let animate = ActivatedAbility {
        mana_cost: animate_cost,
        effect: Effect::BecomeCreature {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            creature_types,
            keywords,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(color_a), tap_add(color_b), animate],
        triggered_abilities: vec![etb_tap(), crate::effect::shortcut::on_attack(attack_effect)],
        ..Default::default()
    }
}

/// Restless Reef — U/B. `{4}{U}{B}`: becomes a 4/3 Fish. Whenever it attacks,
/// surveil 2.
pub fn restless_reef() -> CardDefinition {
    use crate::card::CreatureType;
    restless_land(
        "Restless Reef", Color::Blue, Color::Black,
        cost(&[generic(4), u(), crate::mana::b()]), 4, 3,
        vec![CreatureType::Fish], vec![],
        Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
    )
}

/// Restless Bivouac — R/W. `{3}{R}{W}`: becomes a 2/2 Ox. Whenever it attacks,
/// put a +1/+1 counter on target creature you control.
pub fn restless_bivouac() -> CardDefinition {
    use crate::card::{CreatureType, SelectionRequirement as R};
    use crate::effect::shortcut::target_filtered;
    restless_land(
        "Restless Bivouac", Color::Red, Color::White,
        cost(&[generic(3), crate::mana::r(), crate::mana::w()]), 2, 2,
        vec![CreatureType::Ox], vec![],
        Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    )
}

/// Restless Vinestalk — G/U. `{3}{G}{U}`: becomes a 5/5 Plant with trample.
/// Whenever it attacks, put a +1/+1 counter on target creature you control.
pub fn restless_vinestalk() -> CardDefinition {
    use crate::card::{CreatureType, Keyword, SelectionRequirement as R};
    use crate::effect::shortcut::target_filtered;
    restless_land(
        "Restless Vinestalk", Color::Green, Color::Blue,
        cost(&[generic(3), crate::mana::g(), u()]), 5, 5,
        vec![CreatureType::Plant], vec![Keyword::Trample],
        Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    )
}

/// Restless Fortress — W/B. `{2}{W}{B}`: becomes a 1/4 Nightmare. Whenever it
/// attacks, an opponent loses 1 life and you gain 1 life.
pub fn restless_fortress() -> CardDefinition {
    use crate::card::CreatureType;
    restless_land(
        "Restless Fortress", Color::White, Color::Black,
        cost(&[generic(2), crate::mana::w(), crate::mana::b()]), 1, 4,
        vec![CreatureType::Nightmare], vec![],
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        },
    )
}

/// Restless Ridgeline — R/G. `{2}{R}{G}`: becomes a 3/4 Dinosaur. Whenever it
/// attacks, target creature you control gets +1/+1 until end of turn.
pub fn restless_ridgeline() -> CardDefinition {
    use crate::card::{CreatureType, SelectionRequirement as R};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    restless_land(
        "Restless Ridgeline", Color::Red, Color::Green,
        cost(&[generic(2), crate::mana::r(), crate::mana::g()]), 3, 4,
        vec![CreatureType::Dinosaur], vec![],
        Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Restless Cottage — B/G. `{2}{B}{G}`: becomes a 4/4 Horror. Whenever it
/// attacks, create a Food token. (The graveyard-exile rider is dropped.)
pub fn restless_cottage() -> CardDefinition {
    use crate::card::CreatureType;
    restless_land(
        "Restless Cottage", Color::Black, Color::Green,
        cost(&[generic(2), crate::mana::b(), crate::mana::g()]), 4, 4,
        vec![CreatureType::Horror], vec![],
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::food_token(),
        },
    )
}

/// Restless Spire — U/R creature-land. Enters tapped, `{T}: Add {U} or {R}`.
/// `{U}{R}`: becomes a 2/1 Elemental with first strike until end of turn (still
/// a land). Whenever it attacks, scry 1.
pub fn restless_spire() -> CardDefinition {
    use crate::card::{CreatureType, Keyword};
    use crate::effect::Duration;
    let animate = ActivatedAbility {
        mana_cost: cost(&[u(), crate::mana::r()]),
        effect: Effect::BecomeCreature {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(1),
            creature_types: vec![CreatureType::Elemental],
            keywords: vec![Keyword::FirstStrike],
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Restless Spire",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(Color::Blue), tap_add(Color::Red), animate],
        triggered_abilities: vec![
            etb_tap(),
            crate::effect::shortcut::on_attack(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

// ── Channel lands (Kamigawa: Neon Dynasty legendary lands) ───────────────────

/// Build a Channel land: a legendary land that taps for one color and has a
/// from-hand "Channel — [cost], Discard this card: [effect]" ability (CR
/// 702.16, via `from_hand` + `discard_self_cost`). Channel-cost-reduction and
/// basic-land-search riders are dropped.
fn channel_land(
    name: &'static str,
    color: Color,
    channel_cost: ManaCost,
    channel_effect: Effect,
) -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add(color),
            ActivatedAbility {
                mana_cost: channel_cost,
                effect: channel_effect,
                from_hand: true,
                discard_self_cost: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Boseiju, Who Endures — `{T}: Add {G}`. Channel — {1}{G}: destroy target
/// artifact, enchantment, or nonbasic land. (Opponent-only + basic-search
/// riders dropped.)
pub fn boseiju_who_endures() -> CardDefinition {
    use crate::card::SelectionRequirement as R;
    use crate::effect::shortcut::target_filtered;
    channel_land(
        "Boseiju, Who Endures",
        Color::Green,
        cost(&[generic(1), crate::mana::g()]),
        Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Enchantment).or(R::IsNonbasicLand)),
        },
    )
}

/// Otawara, Soaring City — `{T}: Add {U}`. Channel — {3}{U}: return target
/// artifact, creature, enchantment, or planeswalker to its owner's hand.
pub fn otawara_soaring_city() -> CardDefinition {
    use crate::card::SelectionRequirement as R;
    use crate::effect::shortcut::target_filtered;
    channel_land(
        "Otawara, Soaring City",
        Color::Blue,
        cost(&[generic(3), u()]),
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Planeswalker)),
            to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Eiganjo, Seat of the Empire — `{T}: Add {W}`. Channel — {1}{W}: deal 4
/// damage to target creature. (The printed attacking/blocking restriction and
/// X-scaling are simplified to a flat 4 to any creature.)
pub fn eiganjo_seat_of_the_empire() -> CardDefinition {
    use crate::card::SelectionRequirement as R;
    use crate::effect::shortcut::target_filtered;
    channel_land(
        "Eiganjo, Seat of the Empire",
        Color::White,
        cost(&[generic(1), crate::mana::w()]),
        Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
    )
}

/// Sokenzan, Crucible of Defiance — `{T}: Add {R}`. Channel — {1}{R}: create
/// two 1/1 red Spirit creature tokens with haste.
pub fn sokenzan_crucible_of_defiance() -> CardDefinition {
    let spirit = crate::card::TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![crate::card::Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![crate::card::CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    channel_land(
        "Sokenzan, Crucible of Defiance",
        Color::Red,
        cost(&[generic(1), crate::mana::r()]),
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: spirit },
    )
}

/// Takenuma, Abandoned Mire — `{T}: Add {B}`. Channel — {1}{B}: return a
/// creature or planeswalker card from your graveyard to your hand. (Cost
/// reduction dropped.)
pub fn takenuma_abandoned_mire() -> CardDefinition {
    use crate::card::SelectionRequirement as R;
    channel_land(
        "Takenuma, Abandoned Mire",
        Color::Black,
        cost(&[generic(1), crate::mana::b()]),
        Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature.or(R::Planeswalker),
                },
                Value::Const(1),
            ),
            to: crate::effect::ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Kessig Wolf Run — `{T}: Add {C}`. `{X}{R}, {T}`: target creature gets
/// +X/+0 and gains trample until end of turn.
pub fn kessig_wolf_run() -> CardDefinition {
    use crate::card::Keyword;
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    let pump = ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[crate::mana::x(), crate::mana::r()]),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(crate::card::SelectionRequirement::Creature),
                power: Value::XFromCost,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    };
    CardDefinition {
        name: "Kessig Wolf Run",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), pump],
        ..Default::default()
    }
}

/// Hengegate Pathway // Mistgate Pathway — W/U MDFC.
pub fn hengegate_pathway() -> CardDefinition {
    pathway(
        "Hengegate Pathway", LandType::Plains, Color::White,
        "Mistgate Pathway", LandType::Island, Color::Blue,
    )
}

/// Riverglide Pathway // Lavaglide Pathway — U/R MDFC.
pub fn riverglide_pathway() -> CardDefinition {
    pathway(
        "Riverglide Pathway", LandType::Island, Color::Blue,
        "Lavaglide Pathway", LandType::Mountain, Color::Red,
    )
}

/// Barkchannel Pathway // Tidechannel Pathway — G/U MDFC.
pub fn barkchannel_pathway() -> CardDefinition {
    pathway(
        "Barkchannel Pathway", LandType::Forest, Color::Green,
        "Tidechannel Pathway", LandType::Island, Color::Blue,
    )
}

/// Brightclimb Pathway // Grimclimb Pathway — W/B MDFC.
pub fn brightclimb_pathway() -> CardDefinition {
    pathway(
        "Brightclimb Pathway", LandType::Plains, Color::White,
        "Grimclimb Pathway", LandType::Swamp, Color::Black,
    )
}

/// Needleverge Pathway // Pillarverge Pathway — R/W MDFC.
pub fn needleverge_pathway() -> CardDefinition {
    pathway(
        "Needleverge Pathway", LandType::Mountain, Color::Red,
        "Pillarverge Pathway", LandType::Plains, Color::White,
    )
}

// ── Painlands (allied + enemy "Wastes/Reef/Forge" cycle) ─────────────────────
//
// `{T}: Add {C}` plus two `{T}: Add {color}, deals 1 damage to you` abilities;
// no basic land types, enters untapped. Built via `super::super::painland`.

pub fn adarkar_wastes() -> CardDefinition { painland("Adarkar Wastes", Color::White, Color::Blue) }
pub fn underground_river() -> CardDefinition { painland("Underground River", Color::Blue, Color::Black) }
pub fn sulfurous_springs() -> CardDefinition { painland("Sulfurous Springs", Color::Black, Color::Red) }
pub fn karplusan_forest() -> CardDefinition { painland("Karplusan Forest", Color::Red, Color::Green) }
pub fn brushland() -> CardDefinition { painland("Brushland", Color::Green, Color::White) }
pub fn caves_of_koilos() -> CardDefinition { painland("Caves of Koilos", Color::White, Color::Black) }
pub fn shivan_reef() -> CardDefinition { painland("Shivan Reef", Color::Blue, Color::Red) }
pub fn llanowar_wastes() -> CardDefinition { painland("Llanowar Wastes", Color::Black, Color::Green) }
pub fn yavimaya_coast() -> CardDefinition { painland("Yavimaya Coast", Color::Green, Color::Blue) }
pub fn battlefield_forge() -> CardDefinition { painland("Battlefield Forge", Color::Red, Color::White) }

// ── Rainbow lands ────────────────────────────────────────────────────────────

/// City of Brass — `{T}: Add one mana of any color. This land deals 1 damage to
/// you.` (The printed "whenever this becomes tapped" trigger collapses onto the
/// mana ability — its only natural tap source.)
pub fn city_of_brass() -> CardDefinition {
    CardDefinition {
        name: "City of Brass",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
                Effect::DealDamage { to: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mana Confluence — `{T}, Pay 1 life: Add one mana of any color.`
pub fn mana_confluence() -> CardDefinition {
    CardDefinition {
        name: "Mana Confluence",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghost Quarter — `{T}: Add {C}` plus `{T}, Sacrifice this land: Destroy
/// target land.` (The printed "its controller may search for a basic land"
/// rider — a downside-mitigation for the opponent — is dropped; tracked in
/// TODO.md.)
pub fn ghost_quarter() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Ghost Quarter",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Destroy {
                    what: crate::effect::shortcut::target_filtered(SelectionRequirement::Land),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Field of Ruin — Land. "{T}: Add {C}." "{2}, {T}, Sacrifice this: Destroy
/// target nonbasic land." (The symmetric "each player searches for a basic
/// land" rider is omitted.)
pub fn field_of_ruin() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Field of Ruin",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: crate::mana::cost(&[crate::mana::generic(2)]),
                effect: Effect::Destroy {
                    what: crate::effect::shortcut::target_filtered(
                        SelectionRequirement::IsNonbasicLand,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Khans life-gain taplands ───────────────────────────────────────────────
//
// "~ enters tapped. When ~ enters, you gain 1 life. {T}: Add {C1} or {C2}."
// `dual_land_with` supplies the two color mana abilities; the shared
// `etb_tap_then_gain_one` trigger taps it and gains the life.

pub fn tranquil_cove() -> CardDefinition {
    dual_land_with("Tranquil Cove", LandType::Plains, LandType::Island,
        Color::White, Color::Blue, vec![etb_tap_then_gain_one()])
}

pub fn dismal_backwater() -> CardDefinition {
    dual_land_with("Dismal Backwater", LandType::Island, LandType::Swamp,
        Color::Blue, Color::Black, vec![etb_tap_then_gain_one()])
}

pub fn bloodfell_caves() -> CardDefinition {
    dual_land_with("Bloodfell Caves", LandType::Swamp, LandType::Mountain,
        Color::Black, Color::Red, vec![etb_tap_then_gain_one()])
}

pub fn rugged_highlands() -> CardDefinition {
    dual_land_with("Rugged Highlands", LandType::Mountain, LandType::Forest,
        Color::Red, Color::Green, vec![etb_tap_then_gain_one()])
}

pub fn blossoming_sands() -> CardDefinition {
    dual_land_with("Blossoming Sands", LandType::Forest, LandType::Plains,
        Color::Green, Color::White, vec![etb_tap_then_gain_one()])
}
