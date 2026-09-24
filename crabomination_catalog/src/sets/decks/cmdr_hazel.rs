//! Commander: the cards the **Squirreled Away** precon (BLC, Hazel of the
//! Rootbloom) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_hazel.rs`.
//!
//! Residuals (each also on its card):
//! - **Hazel of the Rootbloom** — "tap X untapped tokens" taps every other
//!   untapped token you control (X is all of them); the mana is any colors.
//! - **Hazel's Brewmaster** — your Foods gain the activated abilities of every
//!   card exiled with it, not only the creature cards.
//! - **Sword of the Squeak** — "base power or toughness 1" reads the printed
//!   power and toughness.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic};
use crabomination_base::tokens::{blood_token, food_token};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn body(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn squirrel() -> Arc<TokenDefinition> {
    Arc::new(body("Squirrel", vec![Color::Green], vec![CreatureType::Squirrel], 1, 1))
}

fn make(count: Value, token: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: token }
}

/// Insects, Rats, Spiders and Squirrels — the Swarmyard family.
fn vermin() -> R {
    R::HasCreatureType(CreatureType::Insect)
        .or(R::HasCreatureType(CreatureType::Rat))
        .or(R::HasCreatureType(CreatureType::Spider))
        .or(R::HasCreatureType(CreatureType::Squirrel))
}

/// Chittering Witch — enters: a 1/1 Rat per opponent; {1}{B}, sacrifice a
/// creature: target creature gets -2/-2 until end of turn.
pub fn chittering_witch() -> CardDefinition {
    let rat = Arc::new(body("Rat", vec![Color::Black], vec![CreatureType::Rat], 1, 1));
    CardDefinition {
        triggered_abilities: vec![etb(make(Value::OpponentCount, rat))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Chittering Witch", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Warlock], 2, 2)
    }
}

/// Deep Forest Hermit — vanishing 3; enters: four Squirrels; Squirrels you
/// control get +1/+1.
pub fn deep_forest_hermit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vanishing(3)],
        triggered_abilities: vec![etb(make(Value::Const(4), squirrel()))],
        static_abilities: vec![static_(
            "Squirrels you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: yours(R::HasCreatureType(CreatureType::Squirrel)),
                power: 1,
                toughness: 1,
            },
        )],
        ..creature("Deep Forest Hermit", cost(&[generic(3), g(), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 1)
    }
}

/// Garruk, Cursed Huntsman — 0: two 2/2 Wolves that feed Garruk loyalty as
/// they die; −3: destroy target creature, draw; −6: +3/+3 and trample emblem.
pub fn garruk_cursed_huntsman() -> CardDefinition {
    let wolf = Arc::new(TokenDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: yours(R::HasPlaneswalkerType(PlaneswalkerSubtype::Garruk)),
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
        }],
        ..body("Wolf", vec![Color::Black, Color::Green], vec![CreatureType::Wolf], 2, 2)
    });
    CardDefinition {
        name: "Garruk, Cursed Huntsman",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Garruk], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility { loyalty_cost: 0, effect: make(Value::Const(2), wolf), x_cost: false },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Destroy { what: target_filtered(R::Creature) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Garruk, Cursed Huntsman".into(),
                    triggered: vec![],
                    statics: vec![
                        static_(
                            "Creatures you control get +3/+3.",
                            StaticEffect::PumpPT { applies_to: yours(R::Creature), power: 3, toughness: 3 },
                        ),
                        static_(
                            "Creatures you control have trample.",
                            StaticEffect::GrantKeyword { applies_to: yours(R::Creature), keyword: Keyword::Trample },
                        ),
                    ],
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}

/// Gourmand's Talent — during your turn your artifacts are Foods with the
/// Food ability; L2: a 3/3 Raccoon on your first life gain each turn; L3: a
/// +1/+1 counter on each creature you control then (CR 716 Class levels).
pub fn gourmands_talent() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let first_gain = |level: u8, effect: Effect| TriggeredAbility {
        event: EventSpec {
            once_per_turn: true,
            ..EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
                .with_filter(Predicate::SourceClassLevelAtLeast(level))
        },
        effect,
    };
    let raccoon = Arc::new(body("Raccoon", vec![Color::Green], vec![CreatureType::Raccoon], 3, 3));
    CardDefinition {
        name: "Gourmand's Talent",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        static_abilities: vec![
            static_(
                "During your turn, artifacts you control are Foods in addition to their other types.",
                StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::AddCardTypeToMatching {
                        applies_to: yours(R::Artifact),
                        card_type: CardType::Artifact,
                        artifact_subtype: Some(ArtifactSubtype::Food),
                    }),
                },
            ),
            static_(
                "During your turn, artifacts you control have \"{2}, {T}, Sacrifice this artifact: You gain 3 life.\"",
                StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GrantActivatedAbility {
                        applies_to: yours(R::Artifact),
                        ability: ActivatedAbility {
                            tap_cost: true,
                            sac_cost: true,
                            mana_cost: cost(&[generic(2)]),
                            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                            ..Default::default()
                        },
                        condition: None,
                    }),
                },
            ),
        ],
        triggered_abilities: vec![
            first_gain(2, make(Value::ONE, raccoon)),
            first_gain(
                3,
                Effect::AddCounter { what: yours(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ),
        ],
        activated_abilities: vec![level_up(cost(&[generic(2), g()]), 1), level_up(cost(&[generic(3), g()]), 2)],
        ..Default::default()
    }
}

/// Hazel of the Rootbloom — {T}, pay 2 life, tap X untapped tokens: X mana in
/// any colors; at your end step, copy target token you control (twice for a
/// Squirrel).
/// Residual: the tap takes every other untapped token you control.
pub fn hazel_of_the_rootbloom() -> CardDefinition {
    let tokens = || yours(R::IsToken.and(R::Untapped).and(R::OtherThanSource));
    let copy = |source: Selector| Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyColors(Value::CountOf(Box::new(tokens()))),
                },
                Effect::Tap { what: tokens() },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                copy(target_filtered(R::IsToken.and(R::ControlledByYou))),
                // "If that token is a Squirrel, instead create two."
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::HasCreatureType(CreatureType::Squirrel),
                    },
                    then: Box::new(copy(Selector::Target(0))),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature(
            "Hazel of the Rootbloom",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Squirrel, CreatureType::Druid],
            3,
            5,
        )
    }
}

/// Hazel's Brewmaster — menace; entering or attacking, exile up to one target
/// card from a graveyard and make a Food; your Foods have the activated
/// abilities of the cards exiled with it.
/// Residual: every exiled card lends its abilities, not only creature cards.
pub fn hazels_brewmaster() -> CardDefinition {
    let body = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::ExileWithSourceStamp },
            make(Value::ONE, Arc::new(food_token())),
        ])),
    };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(body()), on_attack(body())],
        static_abilities: vec![static_(
            "Foods you control have all activated abilities of all creature cards exiled with Hazel's Brewmaster.",
            StaticEffect::ControlledHaveAbilitiesOfExiledWithSource {
                filter: R::HasArtifactSubtype(ArtifactSubtype::Food),
            },
        )],
        ..creature("Hazel's Brewmaster", cost(&[generic(3), b()]), vec![CreatureType::Squirrel, CreatureType::Warlock], 3, 4)
    }
}

/// Insatiable Frugivore — enters: a Food, then you may exile three cards from
/// your graveyard to repeat; {3}{B}, sacrifice X Foods: your creatures get
/// +X/+0 and menace until end of turn.
pub fn insatiable_frugivore() -> CardDefinition {
    let gy = || Selector::EachMatching { zone: ZoneRef::Graveyard(PlayerRef::You), filter: R::Any };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(Value::ONE, Arc::new(food_token())),
            Effect::MayRepeat {
                description: "Exile three cards from your graveyard to make another Food?".into(),
                max: 20,
                body: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CountOf(Box::new(gy())), Value::Const(3)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::MoveChosen { from: gy(), filter: None, count: Value::Const(3), up_to: false, to: ZoneDest::Exile },
                        make(Value::ONE, Arc::new(food_token())),
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            sac_other_x: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: yours(R::Creature),
                    power: Value::XFromCost,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Insatiable Frugivore", cost(&[generic(3), b()]), vec![CreatureType::Rat, CreatureType::Berserker], 2, 4)
    }
}

/// Moonstone Eulogist — flying; an opponent's creature dying makes you a
/// Blood; sacrificing an artifact grows it and gains you 1.
pub fn moonstone_eulogist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: make(Value::ONE, Arc::new(blood_token())),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..creature("Moonstone Eulogist", cost(&[generic(3), b(), b()]), vec![CreatureType::Bat, CreatureType::Warlock], 4, 4)
    }
}

/// Rootcast Apprenticeship — choose three, repeats allowed: two +1/+1
/// counters; copy a token you control; a player makes a Squirrel; an opponent
/// sacrifices a nontoken artifact.
pub fn rootcast_apprenticeship() -> CardDefinition {
    spell(
        "Rootcast Apprenticeship",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            min: 3,
            max: 3,
            allow_repeats: true,
            modes: vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::IsToken.and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(target_filtered(R::Player))),
                    count: Value::ONE,
                    definition: squirrel(),
                },
                Effect::Sacrifice {
                    who: target_filtered(R::OpponentPlayer),
                    count: Value::ONE,
                    filter: R::Artifact.and(R::NotToken),
                },
            ],
        },
    )
}

/// Saw in Half — destroy target creature; if it died, its controller gets
/// two token copies with half its power and toughness, rounded up.
pub fn saw_in_half() -> CardDefinition {
    let half_up = |v: Value| Value::Diff(Box::new(v.clone()), Box::new(Value::HalfDown(Box::new(v))));
    spell(
        "Saw in Half",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::CreaturesDiedThisResolution, Value::ONE),
                then: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        count: Value::Const(2),
                        source: Selector::Target(0),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                    Effect::SetBasePT {
                        what: Selector::LastCreatedTokens,
                        power: half_up(Value::PowerOf(Box::new(Selector::Target(0)))),
                        toughness: half_up(Value::ToughnessOf(Box::new(Selector::Target(0)))),
                        duration: Duration::Permanent,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Scurry of Squirrels — myriad, myriad; combat damage to a player puts a
/// +1/+1 counter on target creature you control.
pub fn scurry_of_squirrels() -> CardDefinition {
    let myriad = || TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: Effect::Myriad };
    CardDefinition {
        triggered_abilities: vec![
            myriad(),
            myriad(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature("Scurry of Squirrels", cost(&[generic(2), g()]), vec![CreatureType::Squirrel, CreatureType::Scout], 2, 2)
    }
}

/// Skyfisher Spider — reach; enters: you may sacrifice another creature, and
/// when you do, destroy target nonland permanent; dies: you may gain 1 life
/// per creature card in your graveyard and exile it.
pub fn skyfisher_spider() -> CardDefinition {
    let others = R::Creature.and(R::ControlledByYou).and(R::OtherThanSource);
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountOf(Box::new(Selector::EachPermanent(others.clone()))),
                    Value::ONE,
                ),
                then: Box::new(Effect::MayDo {
                    description: "Sacrifice another creature to destroy target nonland permanent?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: others },
                        Effect::ReflexiveTrigger {
                            body: Box::new(Effect::Destroy { what: target_filtered(R::Permanent.and(R::Nonland)) }),
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Gain 1 life per creature card in your graveyard and exile Skyfisher Spider?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::GainLife {
                            who: Selector::You,
                            amount: Value::CountOf(Box::new(Selector::EachMatching {
                                zone: ZoneRef::Graveyard(PlayerRef::You),
                                filter: R::Creature,
                            })),
                        },
                        Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                    ])),
                },
            },
        ],
        ..creature("Skyfisher Spider", cost(&[generic(2), b(), g()]), vec![CreatureType::Spider], 3, 3)
    }
}

/// Swarmyard — {T}: {C}; {T}: regenerate target Insect, Rat, Spider or
/// Squirrel.
pub fn swarmyard() -> CardDefinition {
    CardDefinition {
        name: "Swarmyard",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Regenerate { what: target_filtered(R::Creature.and(vermin())) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Swarmyard Massacre — two Squirrels, then each creature that isn't an
/// Insect, Rat, Spider or Squirrel gets -1/-1 per such creature you control.
pub fn swarmyard_massacre() -> CardDefinition {
    let n = || Value::Diff(
        Box::new(Value::Const(0)),
        Box::new(Value::CountOf(Box::new(yours(R::Creature.and(vermin()))))),
    );
    spell(
        "Swarmyard Massacre",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            make(Value::Const(2), squirrel()),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(vermin())))),
                power: n(),
                toughness: n(),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Sword of the Squeak — +1/+1 per creature you control with base power or
/// toughness 1; a Hamster, Mouse, Rat or Squirrel of yours entering may pick
/// it up. Equip {2}.
/// Residual: "base power or toughness" reads the printed numbers.
pub fn sword_of_the_squeak() -> CardDefinition {
    let n = || Value::CountOf(Box::new(yours(R::BasePowerOrToughnessIs(1))));
    let small = R::HasCreatureType(CreatureType::Hamster)
        .or(R::HasCreatureType(CreatureType::Mouse))
        .or(R::HasCreatureType(CreatureType::Rat))
        .or(R::HasCreatureType(CreatureType::Squirrel));
    CardDefinition {
        name: "Sword of the Squeak",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        static_abilities: vec![static_(
            "Equipped creature gets +1/+1 for each creature you control with base power or toughness 1.",
            StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: n(),
                toughness: n(),
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(small) }),
            effect: Effect::MayDo {
                description: "Attach Sword of the Squeak to it?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..Default::default()
    }
}

/// The Odd Acorn Gang — reach, menace, trample; your Squirrels have "{T}:
/// target Squirrel gets +2/+2 and trample, sorcery speed"; one or more
/// Squirrels hitting a player draws a card.
pub fn the_odd_acorn_gang() -> CardDefinition {
    let sq = || R::HasCreatureType(CreatureType::Squirrel);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach, Keyword::Menace, Keyword::Trample],
        static_abilities: vec![static_(
            "Squirrels you control have \"{T}: Target Squirrel gets +2/+2 and gains trample until end of turn. Activate only as a sorcery.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: yours(sq()),
                ability: ActivatedAbility {
                    tap_cost: true,
                    sorcery_speed: true,
                    effect: Effect::Seq(vec![
                        Effect::PumpPT {
                            what: target_filtered(R::Creature.and(sq())),
                            power: Value::Const(2),
                            toughness: Value::Const(2),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                    ]),
                    ..Default::default()
                },
                condition: None,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: sq() })
                .once_per_batch(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "The Odd Acorn Gang",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Squirrel, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Windgrace's Judgment — for any number of opponents, destroy target
/// nonland permanent that player controls.
pub fn windgraces_judgment() -> CardDefinition {
    spell(
        "Windgrace's Judgment",
        cost(&[generic(3), b(), g()]),
        CardType::Instant,
        Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            }),
        },
    )
}
