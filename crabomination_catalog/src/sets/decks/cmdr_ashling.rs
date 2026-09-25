//! Commander: the cards the **Dance of the Elements** precon (ECC, Ashling,
//! the Limitless) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_ashling.rs`.
//!
//! Residuals (each also on its card):
//! - **Flamebraider / Smokebraider / Primal Beyond** — the mana spends on
//!   Elemental creature spells and Elemental sources' abilities; a Kindred
//!   Elemental noncreature spell can't use it (the precon has none).
//! - **Haunting Voyage** — unforetold, the two cards returned are the two
//!   with the greatest power, not the caster's pick.
//! - **Horde of Notions** — casts the Elemental; an Elemental land card
//!   can't be played this way.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{encore, etb, evoke, myriad, on_attack, on_dies, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{
    Color, ManaCost, SpendRestriction, b, cost, g, generic, hybrid, r, u, w,
};
use crate::sets::{enters_tapped, tap_add_colorless};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn elemental() -> R {
    R::HasCreatureType(CreatureType::Elemental)
}

fn wubrg() -> ManaCost {
    cost(&[w(), u(), b(), r(), g()])
}

/// "Vivid — … the number of colors among permanents you control."
fn vivid() -> Value {
    Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou)))
}

/// A colorless Shapeshifter creature token with changeling.
fn changeling() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Shapeshifter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Changeling],
        ..Default::default()
    })
}

/// "{T}: Add [pool]. Spend this mana only [restriction]."
fn restricted_tap(pool: ManaPayload, restriction: SpendRestriction) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Restricted(Box::new(pool), restriction) },
        ..Default::default()
    }
}

/// Flamebraider / Smokebraider — two mana in any combination of colors for
/// Elemental spells and Elemental sources' abilities.
fn braider(name: &'static str, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![restricted_tap(
            ManaPayload::AnyColors(Value::Const(2)),
            SpendRestriction::CreatureOfTypeOrItsAbility(CreatureType::Elemental),
        )],
        ..creature(name, cost(&[generic(1), r()]), types, p, t)
    }
}

/// Abundant Countryside — {T}: {C}; {T}: one mana of any color for creature
/// spells; {6},{T}: a 1/1 changeling Shapeshifter.
pub fn abundant_countryside() -> CardDefinition {
    CardDefinition {
        name: "Abundant Countryside",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            restricted_tap(ManaPayload::AnyOneColor(Value::ONE), SpendRestriction::CreatureOnly),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(6)]),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: changeling() },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ashling, the Limitless — Elemental permanent spells you cast from your
/// hand gain evoke {4}. Whenever you sacrifice a nontoken Elemental, create
/// a token copy of it with haste until end of turn; at your next end step
/// sacrifice the token unless you pay {W}{U}{B}{R}{G}.
pub fn ashling_the_limitless() -> CardDefinition {
    let keep_or_sacrifice = Effect::DelayUntilWithCapture {
        kind: DelayedTriggerKind::YourNextEndStep,
        capture: Selector::LastCreatedToken,
        // A token that already left play is gone for good (CR 111.7); only
        // one still on the battlefield asks for the {W}{U}{B}{R}{G}.
        body: Box::new(Effect::If {
            cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::ControlledByYou },
            then: Box::new(Effect::PayManaOrElse {
                mana_cost: wubrg(),
                otherwise: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
            }),
            else_: Box::new(Effect::Noop),
        }),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Elemental permanent spells you cast from your hand gain evoke {4} as you cast them.",
            effect: StaticEffect::GrantEvokeToSpells {
                filter: elemental().and(R::PermanentCard),
                cost: cost(&[generic(4)]),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: elemental().and(R::NotToken) },
            ),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                keep_or_sacrifice,
            ]),
        }],
        ..legend(
            "Ashling, the Limitless",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            2,
            3,
        )
    }
}

/// Belonging — 6/6 Elemental Incarnation; ETB three 1/1 changeling
/// Shapeshifters. Encore {6}{W}{W}.
pub fn belonging() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: changeling(),
        })],
        activated_abilities: vec![encore(cost(&[generic(6), w(), w()]))],
        ..creature(
            "Belonging",
            cost(&[generic(5), w()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

/// Cavalier of Thorns — reach; ETB reveal the top five, a land from among
/// them onto the battlefield, the rest into your graveyard. Dies: you may
/// exile it; if you do, another card from your graveyard goes on top of
/// your library.
pub fn cavalier_of_thorns() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            etb(Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::Const(5),
                filter: R::Land,
                then: None,
                max: Some(1),
                tapped: false,
                exile_rest: false,
                rest_to_graveyard: true,
            }),
            on_dies(Effect::MayExileSelfThen {
                body: Box::new(Effect::Move {
                    what: target_filtered(R::InYourGraveyard.and(R::OtherThanSource)),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                }),
            }),
        ],
        ..creature(
            "Cavalier of Thorns",
            cost(&[generic(2), g(), g(), g()]),
            vec![CreatureType::Elemental, CreatureType::Knight],
            5,
            6,
        )
    }
}

/// Eclipsed Flamekin — ETB look at the top four; may take an Elemental,
/// Island, or Mountain card; the rest to the bottom in a random order.
pub fn eclipsed_flamekin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(
                elemental().or(R::HasLandType(LandType::Island)).or(R::HasLandType(LandType::Mountain)),
            ),
            take: Some(Value::ONE),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..creature(
            "Eclipsed Flamekin",
            ManaCost::new(vec![generic(1), hybrid(Color::Blue, Color::Red), hybrid(Color::Blue, Color::Red)]),
            vec![CreatureType::Elemental, CreatureType::Scout],
            1,
            4,
        )
    }
}

/// Elemental Spectacle — Vivid: a 5/5 red and green Elemental per color
/// among your permanents; then gain life per creature you control.
pub fn elemental_spectacle() -> CardDefinition {
    let token = Arc::new(TokenDefinition {
        name: "Elemental".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "Elemental Spectacle",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: vivid(), definition: token },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))),
            },
        ]),
        ..Default::default()
    }
}

/// Flamebraider — {T}: two mana in any combination of colors, Elemental
/// spells and abilities only.
pub fn flamebraider() -> CardDefinition {
    braider("Flamebraider", vec![CreatureType::Elemental, CreatureType::Bard], 2, 2)
}

/// Haunting Voyage — choose a creature type; return up to two creature
/// cards of it from your graveyard to the battlefield, or all of them if
/// this spell was foretold. Foretell {5}{B}{B}.
/// Residual: unforetold, the two returned are the two with the greatest
/// power rather than the caster's pick.
pub fn haunting_voyage() -> CardDefinition {
    let of_type = || R::Creature.and(R::IsSourceChosenCreatureType);
    CardDefinition {
        name: "Haunting Voyage",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        foretell_cost: Some(cost(&[generic(5), b(), b()])),
        effect: Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::If {
                cond: Predicate::CastForetold,
                then: Box::new(Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                    who: PlayerRef::You,
                    filter: of_type(),
                    sacrifice_eot: false,
                }),
                else_: Box::new(Effect::Move {
                    what: Selector::TakeGreatestPower {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: of_type(),
                        }),
                        count: Box::new(Value::Const(2)),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            }),
        },
        ..Default::default()
    }
}

/// Horde of Notions — vigilance, trample, haste; {W}{U}{B}{R}{G}: you may
/// cast target Elemental card from your graveyard without paying its mana
/// cost. Residual: an Elemental land card can't be played this way.
pub fn horde_of_notions() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: wubrg(),
            effect: Effect::CastWithoutPayingImmediate {
                what: target_filtered(elemental().and(R::InYourGraveyard)),
                source_zone: Zone::Graveyard,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            ..Default::default()
        }],
        ..legend("Horde of Notions", wubrg(), vec![CreatureType::Elemental], 5, 5)
    }
}

/// Impulsivity — 7/5; ETB you may cast target instant or sorcery card from
/// a graveyard for free, exiled instead of going to a graveyard. Encore
/// {7}{R}{R}.
pub fn impulsivity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CastWithoutPayingImmediate {
            what: target_filtered(
                (R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))).and(R::InGraveyard),
            ),
            source_zone: Zone::Graveyard,
            exile_after: true,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: false,
        })],
        activated_abilities: vec![encore(cost(&[generic(7), r(), r()]))],
        ..creature(
            "Impulsivity",
            cost(&[generic(6), r()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            7,
            5,
        )
    }
}

/// Incandescent Soulstoke — other Elemental creatures you control get
/// +1/+1; {1}{R},{T}: put an Elemental creature card from your hand onto
/// the battlefield with haste, sacrificed at the next end step.
pub fn incandescent_soulstoke() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Elemental creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    elemental().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: elemental().and(R::Creature),
                count: Value::ONE,
                tapped: false,
                haste: true,
                sacrifice_eot: true,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Incandescent Soulstoke",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Jubilation — 5/5; ETB your creatures get +2/+2 and trample until end of
/// turn. Encore {7}{G}{G}.
pub fn jubilation() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: yours(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]))],
        activated_abilities: vec![encore(cost(&[generic(7), g(), g()]))],
        ..creature(
            "Jubilation",
            cost(&[generic(5), g()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            5,
            5,
        )
    }
}

/// Lamentation — 5/4; ETB destroy target creature an opponent controls and
/// gain 3 life. Encore {6}{B}{B}.
pub fn lamentation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        activated_abilities: vec![encore(cost(&[generic(6), b(), b()]))],
        ..creature(
            "Lamentation",
            cost(&[generic(5), b()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            5,
            4,
        )
    }
}

/// Mass of Mysteries — first strike, vigilance, trample; at the beginning
/// of combat on your turn, another target Elemental you control gains
/// myriad until end of turn.
pub fn mass_of_mysteries() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::GrantTriggeredAbility {
                what: target_filtered(elemental().and(R::ControlledByYou).and(R::OtherThanSource)),
                trigger: Box::new(myriad()),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend("Mass of Mysteries", wubrg(), vec![CreatureType::Elemental], 5, 5)
    }
}

/// Primal Beyond — enters tapped unless you reveal an Elemental card from
/// your hand; {T}: {C}; {T}: one mana of any color for Elemental spells and
/// abilities.
pub fn primal_beyond() -> CardDefinition {
    CardDefinition {
        name: "Primal Beyond",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "As this land enters, you may reveal an Elemental card from your hand. If you don't, this land enters tapped.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Hand,
                    filter: elemental(),
                }),
            },
        }],
        activated_abilities: vec![
            tap_add_colorless(),
            restricted_tap(
                ManaPayload::AnyOneColor(Value::ONE),
                SpendRestriction::CreatureOfTypeOrItsAbility(CreatureType::Elemental),
            ),
        ],
        ..Default::default()
    }
}

/// Shimmercreep — menace; Vivid — ETB each opponent loses X life and you
/// gain X, X the colors among your permanents.
pub fn shimmercreep() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: vivid(),
        })],
        ..creature("Shimmercreep", cost(&[generic(4), b()]), vec![CreatureType::Elemental], 3, 5)
    }
}

/// Slithermuse — leaves the battlefield: choose an opponent; if they have
/// more cards in hand than you, draw the difference. Evoke {3}{U}. The
/// opponent chosen is the one with the most cards in hand.
pub fn slithermuse() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(evoke(cost(&[generic(3), u()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::MostCardsInHand)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ))),
            },
        }],
        ..creature("Slithermuse", cost(&[generic(2), u(), u()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Smokebraider — {T}: two mana in any combination of colors, Elemental
/// spells and abilities only.
pub fn smokebraider() -> CardDefinition {
    braider("Smokebraider", vec![CreatureType::Elemental, CreatureType::Shaman], 1, 1)
}

/// Subterfuge — 3/5; ETB target creature gains flying and "whenever this
/// creature deals combat damage to a player, draw that many cards" until
/// end of turn. Encore {7}{U}{U}.
pub fn subterfuge() -> CardDefinition {
    let draw_that_many = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(draw_that_many),
                duration: Duration::EndOfTurn,
            },
        ]))],
        activated_abilities: vec![encore(cost(&[generic(7), u(), u()]))],
        ..creature(
            "Subterfuge",
            cost(&[generic(4), u()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            3,
            5,
        )
    }
}

/// Timeless Lotus — legendary artifact; enters tapped; {T}: {W}{U}{B}{R}{G}.
pub fn timeless_lotus() -> CardDefinition {
    CardDefinition {
        name: "Timeless Lotus",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility { description: "Timeless Lotus enters tapped.", ..enters_tapped() }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Titan of Industry — reach, trample; ETB choose two: destroy target
/// artifact or enchantment; target player gains 5 life; a 4/4 Rhino
/// Warrior; a shield counter on a creature you control.
pub fn titan_of_industry() -> CardDefinition {
    let rhino = Arc::new(TokenDefinition {
        name: "Rhino Warrior".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rhino, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![2],
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(5) },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rhino },
                // "a creature you control" — chosen, not targeted.
                Effect::ChooseOneAmong {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::AddCounter {
                        what: Selector::SeparatedPile { chosen: true },
                        kind: CounterType::Shield,
                        amount: Value::ONE,
                    }),
                    other: Box::new(Effect::Noop),
                },
            ],
        })],
        ..creature(
            "Titan of Industry",
            cost(&[generic(4), g(), g(), g()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Vernal Sovereign — enters or attacks: a green and white Elemental token
/// whose power and toughness are each the number of creatures you control.
pub fn vernal_sovereign() -> CardDefinition {
    let token = Arc::new(TokenDefinition {
        name: "Elemental".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token's power and toughness are each equal to the number of creatures you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Creature,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    });
    let make = || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: token.clone() };
    CardDefinition {
        triggered_abilities: vec![etb(make()), on_attack(make())],
        ..creature(
            "Vernal Sovereign",
            cost(&[generic(4), g(), w()]),
            vec![CreatureType::Elemental, CreatureType::Elk],
            4,
            4,
        )
    }
}
