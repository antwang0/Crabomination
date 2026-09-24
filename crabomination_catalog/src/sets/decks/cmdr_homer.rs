//! Commander batch — Homer, the Hermit (Sultai crab-mill landfall) and the
//! cards EDHREC lists alongside it that the catalog lacked. Tests in
//! `recent_b/cmdr_homer`.
//!
//! New primitives this batch rides:
//!   - `CreatureType::Lobster` (Rikala, Homarid King).
//!   - `ExtraManaKind::FixedWhileSourceCounters` — Rikala's tide-counter-gated
//!     extra {U} on an Island tap.
//!   - `Value::CardsPutIntoGraveyardThisTurn` — Fraying Sanity's X.
//!   - `EntersAsCopy::extra_static` — Sakashima of a Thousand Faces keeps its
//!     legend-rule exemption on the copy.
//!   - `StaticEffect::DoubleControllerLandEntryTriggers` (+ the
//!     `TriggerCandidate::triggered_by_land_entry` flag) — Ancient Greenwarden.
//!   - `AffectedPermanents::CardMatch` honours a bare `IsSource` requirement —
//!     Roaming Throne's "this creature is the chosen type".
//!   - `PlayFromLibraryTop` concretizes `IsSourceChosenCreatureType` against
//!     the granting permanent's choice — Realmwalker.
//!   - `StaticEffect::GraveyardCastBySacrificingOncePerTurn`, read off station
//!     bands too — Exploration Broodship's {8+}.
//!   - `StaticEffect::LegendRuleDoesntApplyToYourPermanents` — Sakashima's
//!     exemption is its controller's only.
//!
//! Residuals (approximated or omitted clauses — each also noted on its card):
//!   - Homer, the Hermit: "any number of target players" is capped at four
//!     targets (a four-seat pod).
//!   - Cruel Calculations: X counts every card put into the target player's
//!     graveyard this turn, not only those that came from their library (the
//!     engine keeps no per-player "milled this turn" tally).
//!   - Exploration Broodship: the {3+} extra land drop is granted at the start
//!     of each of your turns (and the moment the threshold is first crossed)
//!     rather than as a static; the {8+} graveyard cast sacrifices its land
//!     once the cast has gone through rather than mid-cast.
//!   - Rites of Flourishing / Druid Class level 2: "may play an additional
//!     land" is granted per turn (upkeep trigger, plus once on entering /
//!     levelling) rather than as a static the engine re-reads.
//!   - Druid Class level 3: the animated land's P/T is set to the land count
//!     when the ability resolves, not continuously re-evaluated.
//!   - Afterlife from the Loam: the per-player picks are chosen on resolution
//!     rather than targeted, and the Zombie rider covers every creature you
//!     control that entered from a graveyard this turn.
//!   - Maskwood Nexus: creatures you control gain changeling (layer 6), which
//!     computed-keyword readers see but on-battlefield creature-type filters
//!     (e.g. Homer's count) do not; the clause for creature spells and cards
//!     you own off the battlefield is omitted.
//!   - Masked Vandal: the graveyard exile is "one or more" rather than exactly
//!     one creature card.
//!   - Spiny Starfish: each regeneration schedules one Starfish at the next end
//!     step (the same count as the printed end-step tally).
//!   - Open the Way: "X can't be greater than the number of players" caps the
//!     lands revealed for rather than the X that may be paid.
//!   - Ancient Greenwarden: a land's *own* ETB trigger (fired from the land
//!     itself) is not doubled; landfall and other permanents' reactions are.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EntersAsCopy, Keyword, LandType,
    SelectionRequirement as R, StationBand, Subtypes, Supertype, TokenDefinition, WardCost, Zone,
};
use crate::effect::shortcut::{etb, landfall, on_dies, sneak, station, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ExtraManaKind, ManaPayload, PlayerRef,
    Predicate, Selector, StaticAbility, StaticEffect, TriggeredAbility, Value, ZoneDest,
};
use crate::game::effects::treasure_token;
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, u, x, Color, ManaCost, SpendRestriction};
use crate::sets::{tap_add_any_color, tap_add_colorless};
use std::sync::Arc;

// ── Shared frames ────────────────────────────────────────────────────────────

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, power, toughness)
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery { CardType::Sorcery } else { CardType::Instant }],
        effect,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

fn mill(who: Selector, n: i32) -> Effect {
    Effect::Mill { who, amount: Value::Const(n) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn plus_one(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn lands_you_control() -> Value {
    Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)))
}

/// "Create a token that's a copy of target creature you control, except it
/// isn't legendary" — the shared body of the three copy sorceries.
fn copy_your_creature(extra_keywords: Vec<Keyword>) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source: target_filtered(R::Creature.and(R::ControlledByYou)),
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: true,
        legendary: false,
        extra_keywords,
    }
}

/// A colorless Shapeshifter creature token with changeling.
fn shapeshifter_token(power: i32, toughness: i32, colors: Vec<Color>) -> TokenDefinition {
    TokenDefinition {
        name: "Shapeshifter".into(),
        power,
        toughness,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        keywords: vec![Keyword::Changeling],
        ..Default::default()
    }
}

/// The bond cycle's body is `sets::cmdr::crowd_land` — ten cards whose whole
/// text is a player count, so they share one definition rather than three.
use super::super::cmdr::crowd_land as pod_dual;

// ── Creatures ────────────────────────────────────────────────────────────────

/// Homer, the Hermit — {B}{G}{U} Legendary Creature — Crab Druid 0/9.
/// Landfall — Whenever a land you control enters, any number of target
/// players each mill X cards, where X is twice the number of Crabs, Lobsters,
/// Nautiluses, Starfish, and/or Trilobites you control. (Up to four targets —
/// one per seat of a four-player pod.)
pub fn homer_the_hermit() -> CardDefinition {
    let sea_life = R::HasCreatureType(CreatureType::Crab)
        .or(R::HasCreatureType(CreatureType::Lobster))
        .or(R::HasCreatureType(CreatureType::Nautilus))
        .or(R::HasCreatureType(CreatureType::Starfish))
        .or(R::HasCreatureType(CreatureType::Trilobite));
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Player,
            effect: Box::new(Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::count(Selector::EachPermanent(
                        sea_life.and(R::ControlledByYou),
                    ))),
                ),
            }),
        })],
        ..legend(
            "Homer, the Hermit",
            cost(&[b(), g(), u()]),
            vec![CreatureType::Crab, CreatureType::Druid],
            0,
            9,
        )
    }
}

/// Rikala, Homarid King — {1}{G}{U} Legendary Creature — Lobster Noble 0/4.
/// Your upkeep: put a tide counter on Rikala, then if it has four or more,
/// remove them. Creatures you control get +1/+0 per tide counter. Whenever you
/// tap an Island for mana with three or more tide counters on Rikala, add an
/// additional {U} (`ExtraManaKind::FixedWhileSourceCounters`).
pub fn rikala_homarid_king() -> CardDefinition {
    let tides = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Tide };
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::Tide, amount: Value::ONE },
            Effect::If {
                cond: Predicate::ValueAtLeast(tides(), Value::Const(4)),
                then: Box::new(Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Tide,
                    amount: tides(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +1/+0 for each tide counter on Rikala.",
                effect: StaticEffect::PumpPTPerCounterOnSource {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::Tide,
                    per_power: 1,
                    per_toughness: 0,
                },
            },
            StaticAbility {
                description: "Whenever you tap an Island for mana, if there are three or more \
                              tide counters on Rikala, add an additional {U}.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::HasLandType(LandType::Island).and(R::ControlledByYou),
                    extra: ExtraManaKind::FixedWhileSourceCounters(
                        Color::Blue,
                        CounterType::Tide,
                        3,
                    ),
                    while_monarch: false,
                },
            },
        ],
        ..legend(
            "Rikala, Homarid King",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Lobster, CreatureType::Noble],
            0,
            4,
        )
    }
}

/// Gandalf, Shadow's Foe — {5}{U}{U} Legendary Creature — Avatar Wizard 3/4.
/// Vigilance. ETB: exile up to three target lands you control, then return
/// them to the battlefield tapped under their owner's control. Landfall —
/// draw a card and put a +1/+1 counter on Gandalf. The blinked lands re-enter
/// in one batch and each is its own landfall (CR 603.2c).
pub fn gandalf_shadows_foe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Land.and(R::ControlledByYou),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::Target(0) },
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            tapped: true,
                        },
                    },
                ])),
            }),
            landfall(Effect::Seq(vec![draw(1), plus_one(Selector::This, Value::ONE)])),
        ],
        ..legend(
            "Gandalf, Shadow's Foe",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Uchuulon — {3}{B} Creature — Crab Ooze Horror */4. Power equals the number
/// of Crabs, Oozes, and/or Horrors you control. Horrific Symbiosis — at the
/// beginning of your end step, exile up to one target creature card from an
/// opponent's graveyard; if you do, create a token that's a copy of this.
pub fn uchuulon() -> CardDefinition {
    let kin = R::HasCreatureType(CreatureType::Crab)
        .or(R::HasCreatureType(CreatureType::Ooze))
        .or(R::HasCreatureType(CreatureType::Horror));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Uchuulon's power is equal to the number of Crabs, Oozes, and/or \
                          Horrors you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: kin.and(R::Creature),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::InOpponentGraveyard),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::Target(0) },
                    Effect::If {
                        cond: Predicate::SelectorExists(Selector::ExiledThisResolution {
                            filter: R::Creature,
                        }),
                        then: Box::new(crate::effect::shortcut::token_copy_of(
                            PlayerRef::You,
                            Value::ONE,
                            Selector::This,
                        )),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..creature(
            "Uchuulon",
            cost(&[generic(3), b()]),
            vec![CreatureType::Crab, CreatureType::Ooze, CreatureType::Horror],
            0,
            4,
        )
    }
}

/// Mirelurk Queen — {4}{U} Creature — Crab Mutant 4/4. Vigilance. ETB: target
/// player gets two rad counters. Whenever one or more nonland cards are milled,
/// draw a card, then put a +1/+1 counter on this. Triggers only once each turn.
pub fn mirelurk_queen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::AddRadCounters {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardMilled, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Nonland,
                    })
                    .once_per_turn(),
                effect: Effect::Seq(vec![draw(1), plus_one(Selector::This, Value::ONE)]),
            },
        ],
        ..creature(
            "Mirelurk Queen",
            cost(&[generic(4), u()]),
            vec![CreatureType::Crab, CreatureType::Mutant],
            4,
            4,
        )
    }
}

/// Charix, the Raging Isle — {2}{U}{U} Legendary Creature — Leviathan Crab
/// 0/17. Spells your opponents cast that target Charix cost {2} more. {3}:
/// Charix gets +X/-X until end of turn, X = Islands you control.
pub fn charix_the_raging_isle() -> CardDefinition {
    let islands = || {
        Value::count(Selector::EachPermanent(
            R::HasLandType(LandType::Island).and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast that target Charix cost {2} more to cast.",
            effect: StaticEffect::TaxOpponentSpellsTargetingThis { amount: 2 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: islands(),
                toughness: Value::Negate(Box::new(islands())),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Charix, the Raging Isle",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Leviathan, CreatureType::Crab],
            0,
            17,
        )
    }
}

/// Purple Pentapus — {B} Creature — Octopus Starfish 1/1. ETB: surveil 1.
/// {2}{B}, Tap an untapped creature you control: return this card from your
/// graveyard to the battlefield tapped.
pub fn purple_pentapus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Purple Pentapus",
            cost(&[b()]),
            vec![CreatureType::Octopus, CreatureType::Starfish],
            1,
            1,
        )
    }
}

/// Aesi, Tyrant of Gyre Strait — {4}{G}{U} Legendary Creature — Serpent 5/5.
/// You may play an additional land on each of your turns. Landfall — you may
/// draw a card.
pub fn aesi_tyrant_of_gyre_strait() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Draw a card?".into(),
            body: Box::new(draw(1)),
        })],
        ..legend(
            "Aesi, Tyrant of Gyre Strait",
            cost(&[generic(4), g(), u()]),
            vec![CreatureType::Serpent],
            5,
            5,
        )
    }
}

/// Chomping Changeling — {2}{G} Creature — Shapeshifter 1/2. Changeling. ETB:
/// destroy up to one target artifact or enchantment.
pub fn chomping_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Artifact.or(R::Enchantment),
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        })],
        ..creature(
            "Chomping Changeling",
            cost(&[generic(2), g()]),
            vec![CreatureType::Shapeshifter],
            1,
            2,
        )
    }
}

/// Purple-Crystal Crab — {1}{U} Creature — Crab 1/1. When it dies, draw a card.
pub fn purple_crystal_crab() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(draw(1))],
        ..creature("Purple-Crystal Crab", cost(&[generic(1), u()]), vec![CreatureType::Crab], 1, 1)
    }
}

/// Shorecomber Crab — {U} Creature — Crab 0/4. Vanilla.
pub fn shorecomber_crab() -> CardDefinition {
    creature("Shorecomber Crab", cost(&[u()]), vec![CreatureType::Crab], 0, 4)
}

/// Roaming Throne — {4} Artifact Creature — Golem 4/4. Ward {2}. As it enters,
/// choose a creature type; it is that type in addition to its other types. If
/// a triggered ability of another creature you control of the chosen type
/// triggers, it triggers an additional time.
pub fn roaming_throne() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "This creature is the chosen type in addition to its other types.",
                effect: StaticEffect::MatchingAreChosenTypeToo { filter: R::IsSource },
            },
            StaticAbility {
                description: "If a triggered ability of another creature you control of the \
                              chosen type triggers, it triggers an additional time.",
                effect: StaticEffect::DoubleControllerTriggersMatching {
                    filter: R::Creature
                        .and(R::IsSourceChosenCreatureType)
                        .and(R::OtherThanSource),
                },
            },
        ],
        ..creature("Roaming Throne", cost(&[generic(4)]), vec![CreatureType::Golem], 4, 4)
    }
}

/// Spiny Starfish — {2}{U} Creature — Starfish 0/1. {U}: Regenerate. At the
/// beginning of each end step, if it regenerated this turn, create a 0/1 blue
/// Starfish token for each time it regenerated this turn. (Each regeneration
/// schedules one token at the next end step — the same tally.)
pub fn spiny_starfish() -> CardDefinition {
    let starfish = TokenDefinition {
        name: "Starfish".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Starfish], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Regenerated, EventScope::SelfSource),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(starfish),
                }),
            },
        }],
        ..creature("Spiny Starfish", cost(&[generic(2), u()]), vec![CreatureType::Starfish], 0, 1)
    }
}

/// Ancient Greenwarden — {4}{G}{G} Creature — Elemental 5/7. Reach. You may
/// play lands from your graveyard. If a land entering causes a triggered
/// ability of a permanent you control to trigger, it triggers an additional
/// time (`StaticEffect::DoubleControllerLandEntryTriggers`; a land's own ETB
/// trigger is not doubled).
pub fn ancient_greenwarden() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![
            StaticAbility {
                description: "You may play lands from your graveyard.",
                effect: StaticEffect::MayPlayLandsFromGraveyard,
            },
            StaticAbility {
                description: "If a land entering causes a triggered ability of a permanent you \
                              control to trigger, that ability triggers an additional time.",
                effect: StaticEffect::DoubleControllerLandEntryTriggers,
            },
        ],
        ..creature(
            "Ancient Greenwarden",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Elemental],
            5,
            7,
        )
    }
}

/// Iceberg Cancrix — {1}{U} Snow Creature — Crab 0/4. Whenever another snow
/// permanent you control enters, you may have target player mill two cards.
pub fn iceberg_cancrix() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsSnow,
                }),
            effect: Effect::MayDo {
                description: "Have target player mill two cards?".into(),
                body: Box::new(mill(target_filtered(R::Player), 2)),
            },
        }],
        ..creature("Iceberg Cancrix", cost(&[generic(1), u()]), vec![CreatureType::Crab], 0, 4)
    }
}

/// Mirrorshell Crab — {5}{U}{U} Artifact Creature — Crab 5/7. Ward {3}.
/// Channel — {2}{U}, Discard this card: counter target spell or ability unless
/// its controller pays {3}.
pub fn mirrorshell_crab() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(3)])))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::CounterUnless {
                what: target_filtered(R::IsSpellOnStack.or(R::HasAbilityOnStack)),
                cost: WardCost::Mana(cost(&[generic(3)])),
            },
            ..Default::default()
        }],
        ..creature(
            "Mirrorshell Crab",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Crab],
            5,
            7,
        )
    }
}

/// Shore Keeper — {U} Creature — Trilobite 0/3. {7}{U}, {T}, Sacrifice this
/// creature: draw three cards.
pub fn shore_keeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), u()]),
            tap_cost: true,
            sac_cost: true,
            effect: draw(3),
            ..Default::default()
        }],
        ..creature("Shore Keeper", cost(&[u()]), vec![CreatureType::Trilobite], 0, 3)
    }
}

/// Mole Man, Moloid Master — {2}{G} Legendary Creature — Human Villain 1/1.
/// You may play lands from your graveyard. Landfall — create a 1/1 green
/// Minion token named Moloid with "Whenever this token attacks, you may mill a
/// card."
pub fn mole_man_moloid_master() -> CardDefinition {
    let moloid = TokenDefinition {
        name: "Moloid".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Minion], ..Default::default() },
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::MayDo {
            description: "Mill a card?".into(),
            body: Box::new(mill(Selector::You, 1)),
        })],
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may play lands from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyard,
        }],
        triggered_abilities: vec![landfall(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(moloid),
        })],
        ..legend(
            "Mole Man, Moloid Master",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Villain],
            1,
            1,
        )
    }
}

/// Scuttling Sentinel — {1}{G/U}{G/U} Creature — Crab Elf 3/2. Flash,
/// vigilance. ETB: put a +1/+1 counter on another target creature you control;
/// until end of turn it becomes a blue Crab in addition to its other types and
/// gains hexproof.
pub fn scuttling_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            plus_one(
                target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                Value::ONE,
            ),
            Effect::BecomeColor {
                what: Selector::Target(0),
                colors: vec![Color::Blue],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::AddCreatureTypes {
                what: Selector::Target(0),
                creature_types: vec![CreatureType::Crab],
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Scuttling Sentinel",
            cost(&[generic(1), hybrid(Color::Green, Color::Blue), hybrid(Color::Green, Color::Blue)]),
            vec![CreatureType::Crab, CreatureType::Elf],
            3,
            2,
        )
    }
}

/// Sakashima of a Thousand Faces — {3}{U} Legendary Creature — Human Rogue
/// 3/1. May enter as a copy of another creature you control, except it keeps
/// Sakashima's other abilities (`EntersAsCopy::extra_static`). The legend rule
/// doesn't apply to permanents you control. Partner.
pub fn sakashima_of_a_thousand_faces() -> CardDefinition {
    let legend_rule = || StaticAbility {
        description: "The \"legend rule\" doesn't apply to permanents you control.",
        effect: StaticEffect::LegendRuleDoesntApplyToYourPermanents,
    };
    CardDefinition {
        keywords: vec![Keyword::Partner],
        static_abilities: vec![legend_rule()],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByYou),
            extra_keywords: vec![Keyword::Partner],
            extra_static: vec![legend_rule()],
            ..Default::default()
        }),
        ..legend(
            "Sakashima of a Thousand Faces",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            1,
        )
    }
}

/// Chameleon, Master of Disguise — {3}{U} Legendary Creature — Human
/// Shapeshifter Villain 2/3. May enter as a copy of a creature you control,
/// except his name is Chameleon, Master of Disguise. Mayhem {2}{U}.
pub fn chameleon_master_of_disguise() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Mayhem(cost(&[generic(2), u()]))],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByYou),
            keep_name: true,
            ..Default::default()
        }),
        ..legend(
            "Chameleon, Master of Disguise",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Shapeshifter, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Cryptic Trilobite — {X}{X} Creature — Trilobite 0/0. Enters with X +1/+1
/// counters. Remove a +1/+1 counter: add {C}{C}, spend only to activate
/// abilities. {1}, {T}: put a +1/+1 counter on this creature.
pub fn cryptic_trilobite() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![
            ActivatedAbility {
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::Const(2))),
                        SpendRestriction::AbilitiesOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: plus_one(Selector::This, Value::ONE),
                ..Default::default()
            },
        ],
        ..creature("Cryptic Trilobite", cost(&[x(), x()]), vec![CreatureType::Trilobite], 0, 0)
    }
}

/// Masked Vandal — {1}{G} Creature — Shapeshifter 1/3. Changeling. ETB: you may
/// exile a creature card from your graveyard; if you do, exile target artifact
/// or enchantment an opponent controls. (The graveyard exile is "one or more"
/// under `MayExileFromYourGraveyard`.)
pub fn masked_vandal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![etb(Effect::MayExileFromYourGraveyard {
            filter: R::Creature,
            then: Box::new(Effect::Exile {
                what: target_filtered(
                    R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
                ),
            }),
        })],
        ..creature(
            "Masked Vandal",
            cost(&[generic(1), g()]),
            vec![CreatureType::Shapeshifter],
            1,
            3,
        )
    }
}

/// Loki, Lord of Misrule — {3}{U} Legendary Creature — God Sorcerer Villain
/// 3/4. {U}, {T}: choose target creature you control; each other creature you
/// control becomes a copy of it until end of turn, except it isn't legendary.
/// Sorcery speed.
pub fn loki_lord_of_misrule() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::BecomeCopyOfFor {
                what: Selector::EachPermanentExceptTargets(R::Creature.and(R::ControlledByYou)),
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
                non_legendary: true,
            },
            ..Default::default()
        }],
        ..legend(
            "Loki, Lord of Misrule",
            cost(&[generic(3), u()]),
            vec![CreatureType::God, CreatureType::Sorcerer, CreatureType::Villain],
            3,
            4,
        )
    }
}

/// Changeling Wayfinder — {3} Creature — Shapeshifter 1/2. Changeling. ETB: you
/// may search your library for a basic land card, reveal it, put it into your
/// hand, then shuffle.
pub fn changeling_wayfinder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a basic land card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Changeling Wayfinder",
            cost(&[generic(3)]),
            vec![CreatureType::Shapeshifter],
            1,
            2,
        )
    }
}

/// Crustacean Commando — {1}{U} Creature — Crab Mutant Soldier 0/3. ETB:
/// create a Mutagen token ({1}, {T}, sacrifice: +1/+1 counter on target
/// creature; sorcery speed).
pub fn crustacean_commando() -> CardDefinition {
    let mutagen = TokenDefinition {
        name: "Mutagen".into(),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: plus_one(target_filtered(R::Creature), Value::ONE),
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(mutagen),
        })],
        ..creature(
            "Crustacean Commando",
            cost(&[generic(1), u()]),
            vec![CreatureType::Crab, CreatureType::Mutant, CreatureType::Soldier],
            0,
            3,
        )
    }
}

/// Realmwalker — {2}{G} Creature — Shapeshifter 2/3. Changeling. As it enters,
/// choose a creature type. You may look at the top card of your library any
/// time, and may cast creature spells of the chosen type from it.
pub fn realmwalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast creature spells of the chosen type from the top of \
                              your library.",
                effect: StaticEffect::PlayFromLibraryTop {
                    filter: R::Creature.and(R::IsSourceChosenCreatureType),
                },
            },
        ],
        ..creature("Realmwalker", cost(&[generic(2), g()]), vec![CreatureType::Shapeshifter], 2, 3)
    }
}

// ── Instants and sorceries ───────────────────────────────────────────────────

/// Through the Forest Gate — {6}{G}{G} Sorcery. Look at the top twenty cards
/// of your library, put any number of land cards from among them onto the
/// battlefield tapped, then shuffle. You gain 8 life.
pub fn through_the_forest_gate() -> CardDefinition {
    spell(
        "Through the Forest Gate",
        cost(&[generic(6), g(), g()]),
        true,
        Effect::Seq(vec![
            Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::Const(20),
                filter: R::Land,
                then: None,
                max: None,
                tapped: true,
                exile_rest: false, rest_to_graveyard: false,
            },
            Effect::ShuffleLibrary { who: PlayerRef::You },
            Effect::GainLife { who: Selector::You, amount: Value::Const(8) },
        ]),
    )
}

/// Cruel Calculations — {2}{U} Sorcery. Draw X cards, where X is the number of
/// cards put into target player's graveyard from their library this turn.
/// (Approximated: X counts every card put into that graveyard this turn.)
pub fn cruel_calculations() -> CardDefinition {
    spell(
        "Cruel Calculations",
        cost(&[generic(2), u()]),
        true,
        Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::CardsPutIntoGraveyardThisTurn(PlayerRef::Target(0)),
            }),
        },
    )
}

/// Entish Restoration — {2}{G} Instant. Sacrifice a land. Search for up to two
/// basic lands onto the battlefield tapped; up to three instead if you control
/// a creature with power 4 or greater.
pub fn entish_restoration() -> CardDefinition {
    let fetch = |n: i32| Effect::SearchUpToN {
        who: PlayerRef::You,
        filter: R::IsBasicLand,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        count: Value::Const(n),
    };
    spell(
        "Entish Restoration",
        cost(&[generic(2), g()]),
        false,
        Effect::Seq(vec![
            Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Land },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                )),
                then: Box::new(fetch(3)),
                else_: Box::new(fetch(2)),
            },
        ]),
    )
}

/// Kitsune's Technique — {4}{U}{U} Instant. Sneak {1}{U}. Target opponent mills
/// half their library, rounded up.
pub fn kitsunes_technique() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sneak(cost(&[generic(1), u()]))),
        ..spell(
            "Kitsune's Technique",
            cost(&[generic(4), u(), u()]),
            false,
            Effect::MillHalf { who: target_filtered(R::OpponentPlayer), rounded_up: true },
        )
    }
}

/// Multiversal Recruitment — {3}{U} Sorcery. Token copy of target creature you
/// control, except it isn't legendary. Flashback {5}{U}{U}.
pub fn multiversal_recruitment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), u(), u()]))],
        ..spell("Multiversal Recruitment", cost(&[generic(3), u()]), true, copy_your_creature(vec![]))
    }
}

/// Irenicus's Vile Duplication — {3}{U} Sorcery. Token copy of target creature
/// you control, except it has flying and isn't legendary.
pub fn irenicuss_vile_duplication() -> CardDefinition {
    spell(
        "Irenicus's Vile Duplication",
        cost(&[generic(3), u()]),
        true,
        copy_your_creature(vec![Keyword::Flying]),
    )
}

/// Quantum Misalignment — {4}{U} Sorcery. Token copy of target creature you
/// control, except it isn't legendary. Rebound.
pub fn quantum_misalignment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell("Quantum Misalignment", cost(&[generic(4), u()]), true, copy_your_creature(vec![]))
    }
}

/// Will of the Sultai — {4}{G} Sorcery. Choose one (both if you control a
/// commander): target player mills three, then return all land cards from your
/// graveyard to the battlefield tapped; or put X +1/+1 counters on target
/// creature (X = lands you control) and it gains trample until end of turn.
pub fn will_of_the_sultai() -> CardDefinition {
    let modes = || {
        vec![
            Effect::Seq(vec![
                mill(target_filtered(R::Player), 3),
                Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Land,
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ]),
            Effect::Seq(vec![
                plus_one(target_filtered(R::Creature), lands_you_control()),
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        ]
    };
    spell(
        "Will of the Sultai",
        cost(&[generic(4), g()]),
        true,
        Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
    )
}

/// "For each player, choose a [filter] card in that player's graveyard. Put
/// those cards onto the battlefield under your control." (`up_to`: "up to one")
fn reanimate_one_from_each_graveyard(filter: R, up_to: bool) -> Effect {
    Effect::ForEach {
        selector: Selector::Player(PlayerRef::EachPlayer),
        body: Box::new(Effect::MoveChosen {
            from: Selector::CardsInZone { who: PlayerRef::Triggerer, zone: Zone::Graveyard, filter },
            filter: None,
            count: Value::ONE,
            up_to,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
    }
}

/// Breach the Multiverse — {5}{B}{B} Sorcery. Each player mills ten. For each
/// player, choose a creature or planeswalker card in that player's graveyard;
/// put those onto the battlefield under your control. Then each creature you
/// control becomes a Phyrexian in addition to its other types.
pub fn breach_the_multiverse() -> CardDefinition {
    spell(
        "Breach the Multiverse",
        cost(&[generic(5), b(), b()]),
        true,
        Effect::Seq(vec![
            mill(Selector::Player(PlayerRef::EachPlayer), 10),
            reanimate_one_from_each_graveyard(R::Creature.or(R::Planeswalker), false),
            Effect::AddCreatureTypes {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                creature_types: vec![CreatureType::Phyrexian],
                duration: Duration::Permanent,
            },
        ]),
    )
}

/// Nanogene Conversion — {3}{U} Sorcery. Choose target creature you control.
/// Each other creature becomes a copy of it until end of turn, except it isn't
/// legendary.
pub fn nanogene_conversion() -> CardDefinition {
    spell(
        "Nanogene Conversion",
        cost(&[generic(3), u()]),
        true,
        Effect::BecomeCopyOfFor {
            what: Selector::EachPermanentExceptTargets(R::Creature),
            source: target_filtered(R::Creature.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
            non_legendary: true,
        },
    )
}

/// Formless Genesis — {2}{G} Kindred Sorcery — Shapeshifter. Changeling. Create
/// an X/X colorless Shapeshifter token with changeling and deathtouch, X = land
/// cards in your graveyard. Retrace.
pub fn formless_genesis() -> CardDefinition {
    let x_lands = Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Land };
    let token = TokenDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Deathtouch],
        dynamic_pt: Some((x_lands.clone(), x_lands)),
        ..shapeshifter_token(0, 0, vec![])
    };
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        keywords: vec![Keyword::Changeling, Keyword::Retrace],
        ..spell(
            "Formless Genesis",
            cost(&[generic(2), g()]),
            true,
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
        )
    }
}

/// Open the Way — {X}{G}{G} Sorcery. X can't exceed the number of players.
/// Reveal until X lands; put them onto the battlefield tapped, the rest on the
/// bottom in a random order. (The cap bounds the lands revealed for.)
pub fn open_the_way() -> CardDefinition {
    spell(
        "Open the Way",
        cost(&[x(), g(), g()]),
        true,
        Effect::RevealUntilLandsToBattlefield {
            count: Value::Min(Box::new(Value::XFromCost), Box::new(Value::PlayerCount)),
            tapped: true,
        },
    )
}

/// Afterlife from the Loam — {5}{B}{B}{B} Sorcery. Delve. For each player,
/// choose up to one creature card in that player's graveyard; put those onto
/// the battlefield under your control. They're Zombies in addition to their
/// other types. (The picks are made on resolution, not targeted; "those" is
/// read as the creatures you control that entered from a graveyard this turn.)
pub fn afterlife_from_the_loam() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Delve],
        ..spell(
            "Afterlife from the Loam",
            cost(&[generic(5), b(), b(), b()]),
            true,
            Effect::Seq(vec![
                reanimate_one_from_each_graveyard(R::Creature, true),
                Effect::AddCreatureTypes {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::EnteredFromGraveyardThisTurn),
                    ),
                    creature_types: vec![CreatureType::Zombie],
                    duration: Duration::Permanent,
                },
            ]),
        )
    }
}

/// Reshape the Earth — {6}{G}{G}{G} Sorcery. Search for up to ten land cards,
/// put them onto the battlefield tapped, then shuffle.
pub fn reshape_the_earth() -> CardDefinition {
    spell(
        "Reshape the Earth",
        cost(&[generic(6), g(), g(), g()]),
        true,
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Land,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::Const(10),
        },
    )
}

// ── Artifacts ────────────────────────────────────────────────────────────────

/// Maskwood Nexus — {4} Artifact. Creatures you control are every creature
/// type (approximated: they gain changeling, which type filters that read the
/// permanent's type line don't see; the off-battlefield half is omitted).
/// {3}, {T}: create a 2/2 blue Shapeshifter token with changeling.
pub fn maskwood_nexus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control are every creature type.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Changeling,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(shapeshifter_token(2, 2, vec![Color::Blue])),
            },
            ..Default::default()
        }],
        ..artifact("Maskwood Nexus", cost(&[generic(4)]))
    }
}

/// Altar of the Brood — {1} Artifact. Whenever another permanent you control
/// enters, each opponent mills a card.
pub fn altar_of_the_brood() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours),
            effect: mill(Selector::Player(PlayerRef::EachOpponent), 1),
        }],
        ..artifact("Altar of the Brood", cost(&[generic(1)]))
    }
}

/// Exploration Broodship — {G} Artifact — Spacecraft. Station. {3+}: you may
/// play an additional land each turn (granted at the start of each of your
/// turns and when first stationed past 3). {8+}: once during each of your
/// turns, you may cast a permanent spell from your graveyard by sacrificing a
/// land in addition to paying its other costs; 4/4 flying.
/// The land is sacrificed right after the cast is made (no priority passes in
/// between), and you pick it when you control more than one.
pub fn exploration_broodship() -> CardDefinition {
    let stationed = || Predicate::SourceHasCountersAtLeast { counter: CounterType::Charge, n: 3 };
    let extra_land = || Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Spacecraft],
            ..Default::default()
        },
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 8,
            keywords: vec![Keyword::Flying],
            pt: Some((4, 4)),
            statics: vec![StaticEffect::GraveyardCastBySacrificingOncePerTurn {
                filter: R::Permanent,
                sacrifice: R::Land,
            }],
            ..Default::default()
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                    .with_filter(stationed()),
                effect: extra_land(),
            },
            // Crossing the {3+} threshold mid-turn opens that turn's drop too.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Charge), EventScope::SelfSource)
                    .with_filter(stationed())
                    .once_per_turn(),
                effect: Effect::If {
                    cond: Predicate::IsTurnOf(PlayerRef::You),
                    then: Box::new(extra_land()),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..artifact("Exploration Broodship", cost(&[g()]))
    }
}

/// Firdoch Core — {3} Kindred Artifact — Shapeshifter. Changeling. {T}: add one
/// mana of any color. {4}: becomes a 4/4 artifact creature until end of turn.
pub fn firdoch_core() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Artifact],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        keywords: vec![Keyword::Changeling],
        activated_abilities: vec![
            tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..artifact("Firdoch Core", cost(&[generic(3)]))
    }
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Black Market Connections — {2}{B} Enchantment. At the beginning of your first
/// main phase, choose one or more: Treasure and lose 1; draw and lose 2; a 3/2
/// colorless changeling Shapeshifter and lose 3. (Default picks: the first two.)
pub fn black_market_connections() -> CardDefinition {
    let lose = |n: i32| Effect::LoseLife { who: Selector::You, amount: Value::Const(n) };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::ChooseN {
                picks: vec![0, 1],
                modes: vec![
                    Effect::Seq(vec![
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(treasure_token()),
                        },
                        lose(1),
                    ]),
                    Effect::Seq(vec![draw(1), lose(2)]),
                    Effect::Seq(vec![
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(shapeshifter_token(3, 2, vec![])),
                        },
                        lose(3),
                    ]),
                ],
            },
        }],
        ..enchantment("Black Market Connections", cost(&[generic(2), b()]))
    }
}

/// Arcane Adaptation — {2}{U} Enchantment. As it enters, choose a creature type.
/// Creatures you control, creature spells you control, and creature cards you
/// own off the battlefield are that type in addition to their other types.
pub fn arcane_adaptation() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control are the chosen type in addition to their \
                              other types.",
                effect: StaticEffect::MatchingAreChosenTypeToo {
                    filter: R::Creature.and(R::ControlledByYou),
                },
            },
            StaticAbility {
                description: "The same is true for creature spells you control and creature \
                              cards you own that aren't on the battlefield.",
                effect: StaticEffect::OwnedCardsOffBattlefieldAreChosenTypeToo {
                    filter: R::Creature,
                },
            },
        ],
        ..enchantment("Arcane Adaptation", cost(&[generic(2), u()]))
    }
}

/// Fraying Sanity — {2}{U} Enchantment — Aura Curse. Enchant player. At the
/// beginning of each end step, enchanted player mills X cards, X = cards put
/// into their graveyard from anywhere this turn.
pub fn fraying_sanity() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EnchantedPlayer),
                amount: Value::CardsPutIntoGraveyardThisTurn(PlayerRef::EnchantedPlayer),
            },
        }],
        ..enchantment("Fraying Sanity", cost(&[generic(2), u()]))
    }
}

/// Druid Class — {1}{G} Enchantment — Class. L1: landfall, gain 1 life. L2
/// ({2}{G}): you may play an additional land on each of your turns (granted per
/// turn). L3 ({4}{G}): target land you control becomes a creature with haste
/// whose P/T equal the lands you control (set on resolution); still a land.
pub fn druid_class() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let extra_land = || Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Class],
            ..Default::default()
        },
        triggered_abilities: vec![
            landfall(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: extra_land(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: extra_land(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(3)),
                effect: Effect::BecomeCreature {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    power: lands_you_control(),
                    toughness: lands_you_control(),
                    creature_types: vec![],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::Permanent,
                },
            },
        ],
        activated_abilities: vec![
            level_up(cost(&[generic(2), g()]), 1),
            level_up(cost(&[generic(4), g()]), 2),
        ],
        ..enchantment("Druid Class", cost(&[generic(1), g()]))
    }
}

/// Virtue of Knowledge // Vantress Visions — {4}{U} Enchantment // {1}{U}
/// Instant — Adventure. If a permanent entering causes a triggered ability of a
/// permanent you control to trigger, it triggers an additional time. Adventure:
/// copy target activated or triggered ability you control (new targets are
/// auto-kept).
pub fn virtue_of_knowledge() -> CardDefinition {
    CardDefinition {
        adventure: Some(Box::new(Adventure {
            name: "Vantress Visions",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou)),
                times: Value::ONE,
            },
        })),
        static_abilities: vec![StaticAbility {
            description: "If a permanent entering causes a triggered ability of a permanent you \
                          control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerEtbTriggers,
        }],
        ..enchantment("Virtue of Knowledge", cost(&[generic(4), u()]))
    }
}

/// Memory Erosion — {1}{U}{U} Enchantment. Whenever an opponent casts a spell,
/// that player mills two cards.
pub fn memory_erosion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: mill(Selector::Player(PlayerRef::Triggerer), 2),
        }],
        ..enchantment("Memory Erosion", cost(&[generic(1), u(), u()]))
    }
}

/// Rites of Flourishing — {2}{G} Enchantment. Each player draws an additional
/// card in their draw step and may play an additional land each turn (the land
/// grant lands at each upkeep, and for the current turn as it enters).
pub fn rites_of_flourishing() -> CardDefinition {
    let active_extra_land =
        || Effect::GrantExtraLandPlay { who: PlayerRef::ActivePlayer, count: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::AnyPlayer),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: active_extra_land(),
            },
            etb(active_extra_land()),
        ],
        ..enchantment("Rites of Flourishing", cost(&[generic(2), g()]))
    }
}

// ── Lands ────────────────────────────────────────────────────────────────────

/// Hobbit Hole — Land. {T}, Sacrifice: search for a basic land card, put it
/// onto the battlefield tapped, then shuffle. Halflingcycling {4}.
pub fn hobbit_hole() -> CardDefinition {
    CardDefinition {
        name: "Hobbit Hole",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Typecycling(Box::new((
            cost(&[generic(4)]),
            R::HasCreatureType(CreatureType::Halfling),
        )))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drownyard Temple — Land. {T}: Add {C}. {3}: return this card from your
/// graveyard to the battlefield tapped.
pub fn drownyard_temple() -> CardDefinition {
    CardDefinition {
        name: "Drownyard Temple",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                from_graveyard: true,
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rejuvenating Springs — Land. Enters tapped unless you have two or more
/// opponents. {T}: Add {G} or {U}.
pub fn rejuvenating_springs() -> CardDefinition {
    pod_dual("Rejuvenating Springs", Color::Green, Color::Blue)
}

/// Undergrowth Stadium — Land. Enters tapped unless you have two or more
/// opponents. {T}: Add {B} or {G}.
pub fn undergrowth_stadium() -> CardDefinition {
    pod_dual("Undergrowth Stadium", Color::Black, Color::Green)
}

/// Morphic Pool — Land. Enters tapped unless you have two or more opponents.
/// {T}: Add {U} or {B}.
pub fn morphic_pool() -> CardDefinition {
    pod_dual("Morphic Pool", Color::Blue, Color::Black)
}

/// Riveteers Overlook — Land. When it enters, sacrifice it. When you do, search
/// for a basic Swamp, Mountain, or Forest card, put it onto the battlefield
/// tapped, then shuffle and you gain 1 life.
pub fn riveteers_overlook() -> CardDefinition {
    CardDefinition {
        name: "Riveteers Overlook",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::SacrificeSource,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(
                    R::HasLandType(LandType::Swamp)
                        .or(R::HasLandType(LandType::Mountain))
                        .or(R::HasLandType(LandType::Forest)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}
