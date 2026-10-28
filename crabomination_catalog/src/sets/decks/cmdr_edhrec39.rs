//! Commander: the 1500 rescan's one- and two-card gaps for Kwia, Kyodai,
//! Eivor, Ashaya's Enduring Bond, The Mighty Thor (Jane Foster), Azula, King
//! Macar, Hurska, RMS Titanic and Mu Yanling, Wind Rider (`cmdr_edhrec15` is
//! this run's previous batch). Tests in `tests/recent_b/cmdr_edhrec39.rs`.

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LoyaltyAbility, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn up_to_one(body: Effect) -> Effect {
    Effect::OptionalTargets { min: 0, body: Box::new(body) }
}

fn food() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::food_token()),
    }
}

fn you_gain_life(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl), effect }
}

// ── Kwia, Kyodai ─────────────────────────────────────────────────────────────

/// Enlightened Confidant — {1}{W} 2/1 Kor Cleric, lifelink. At the beginning
/// of your end step, if you gained life this turn, surveil 1; a card it put
/// into your graveyard with mana value at most the life you gained this turn
/// goes to your hand. The card surveiled is the top one, so the trigger binds
/// it first and reads where the surveil left it.
pub fn enlightened_confidant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
            ),
            effect: Effect::ForEach {
                selector: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                body: Box::new(Effect::Seq(vec![
                    Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::InYourGraveyard.and(R::ManaValueAtMostLifeGainedThisTurn),
                        },
                        then: Box::new(Effect::Move {
                            what: Selector::TriggerSource,
                            to: ZoneDest::Hand(PlayerRef::You),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..creature("Enlightened Confidant", cost(&[generic(1), w()]), vec![CreatureType::Kor, CreatureType::Cleric], 2, 1)
    }
}

/// Myojin of Blooming Dawn — {5}{W}{W}{W} 4/6 Legendary Spirit. Enters with an
/// indestructible counter if cast from hand (CR 122.1b: the counter is the keyword);
/// remove it: a 1/1 colorless Spirit for each permanent you control.
pub fn myojin_of_blooming_dawn() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches { what: Selector::This, filter: R::WasCastFromHand },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Indestructible,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Indestructible, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(R::ControlledByYou)),
                definition: Arc::new(spirit),
            },
            ..Default::default()
        }],
        ..legend("Myojin of Blooming Dawn", cost(&[generic(5), w(), w(), w()]), vec![CreatureType::Spirit], 4, 6)
    }
}

// ── Eivor ────────────────────────────────────────────────────────────────────

/// Eivor, Wolf-Kissed — {3}{R}{G}{W} 7/6 Legendary Human Assassin Warrior,
/// trample, haste. Combat damage to a player: mill that many; you may put a
/// Saga card and/or a land card from among them onto the battlefield.
pub fn eivor_wolf_kissed() -> CardDefinition {
    let among = |filter: R| Selector::one_of(Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter });
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::TriggerEventAmount },
                may(
                    "Put a Saga card and/or a land card from among them onto the battlefield?",
                    Effect::Move {
                        what: Selector::Both(
                            Box::new(among(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga))),
                            Box::new(among(R::Land)),
                        ),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ),
            ]),
        }],
        ..legend(
            "Eivor, Wolf-Kissed",
            cost(&[generic(3), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Warrior],
            7,
            6,
        )
    }
}

/// Fall of the First Civilization — {2}{W} Saga. I: you and target opponent
/// each draw two. II: exile target artifact an opponent controls. III: each
/// player chooses three nonland permanents they control; destroy the rest.
pub fn fall_of_the_first_civilization() -> CardDefinition {
    saga(
        "Fall of the First Civilization",
        cost(&[generic(2), w()]),
        vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::Draw {
                        who: target_filtered(R::Player.and(R::ControlledByOpponent)),
                        amount: Value::Const(2),
                    },
                ]),
            ),
            (2, Effect::Exile { what: target_filtered(R::Artifact.and(R::ControlledByOpponent)) }),
            (
                3,
                Effect::EachPlayerKeepsNSacrificesRest {
                    keep: Value::Const(3),
                    filter: Some(R::Nonland),
                    destroy: true,
                },
            ),
        ],
    )
}

// ── Ashaya's Enduring Bond ───────────────────────────────────────────────────

/// Ashaya's Enduring Bond — {1}{R}{G} Legendary Sorcery that can be your
/// commander (CR 903.3a). Discover X, X = the mana spent to cast it.
pub fn ashayas_enduring_bond() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        ..spell(
            "Ashaya's Enduring Bond",
            cost(&[generic(1), r(), g()]),
            CardType::Sorcery,
            Effect::Discover { n: Value::CastSpellManaSpent, filter: None },
        )
    }
}

/// Nico Minoru, Runaway — {3}{R} 2/4 Legendary Human Warlock Hero. Casting a
/// spell from anywhere but your hand deals 2 to each opponent. {2}{R}, {T},
/// discard a card: exile from the top until a nonland card; you may cast it
/// free.
pub fn nico_minoru_runaway() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::CastSpellFromHand))),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::one_of(Selector::ExiledThisResolution { filter: R::Nonland }),
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Nico Minoru, Runaway",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Warlock, CreatureType::Hero],
            2,
            4,
        )
    }
}

// ── The Mighty Thor, Jane Foster ─────────────────────────────────────────────

/// Bespoke Bō — {2}{U} Equipment. Enters: return up to one other target
/// nonland permanent to its owner's hand. Equipped creature gets +2/+1 and has
/// vigilance. Equip {3}.
pub fn bespoke_bo() -> CardDefinition {
    CardDefinition {
        name: "Bespoke Bō",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(up_to_one(Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::OtherThanSource)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        }))],
        ..Default::default()
    }
}

/// Ironheart, Clever Champion — {4}{U} 3/4 Legendary Artifact Creature —
/// Human Hero. Improvise, flying. Noncreature spells you cast have improvise.
pub fn ironheart_clever_champion() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Improvise, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast have improvise.",
            effect: StaticEffect::GrantImproviseToSpells { filter: R::Creature.negate() },
        }],
        ..legend(
            "Ironheart, Clever Champion",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            4,
        )
    }
}

// ── Azula ────────────────────────────────────────────────────────────────────

/// Hama, the Bloodbender — {2}{U/B}{U/B}{U/B} 3/3 Legendary Human Warlock.
/// Enters: target opponent mills three; exile up to one noncreature, nonland
/// card from their graveyard, castable by waterbending {X} (X its mana value).
/// The waterbend is its mana value paid with any mana (CR 701.67 lets your
/// artifacts and creatures help; this grant does not) — INCOMPLETE_CARDS 🟡.
pub fn hama_the_bloodbender() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: target_filtered(R::Player.and(R::ControlledByOpponent)),
                amount: Value::Const(3),
            },
            may(
                "Exile a noncreature, nonland card from their graveyard?",
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::one_of(Selector::CardsInZone {
                            who: PlayerRef::Target(0),
                            zone: Zone::Graveyard,
                            filter: R::Nonland.and(R::Creature.negate()),
                        }),
                        to: ZoneDest::Exile,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::WhileHolderControlsSource {
                            source: crate::card::CardId(0),
                            your_turn: true,
                        },
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: true,
                    },
                ]),
            ),
        ]))],
        ..legend(
            "Hama, the Bloodbender",
            cost(&[generic(2), ub(), ub(), ub()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Roll-Roll-Roll-Roll — {2}{U} Saga, four chapters: each exiles up to one
/// target creature or land you control, returning it at the next end step.
pub fn roll_roll_roll_roll() -> CardDefinition {
    let chapter = || {
        up_to_one(Effect::ExileReturnNextEndStep {
            what: target_filtered(yours(R::Creature.or(R::Land))),
        })
    };
    saga(
        "Roll-Roll-Roll-Roll",
        cost(&[generic(2), u()]),
        vec![(1, chapter()), (2, chapter()), (3, chapter()), (4, chapter())],
    )
}

// ── King Macar ───────────────────────────────────────────────────────────────

/// Mobile Garrison — {3} 3/4 Vehicle, crew 2. Attacks: untap another target
/// artifact or creature you control.
pub fn mobile_garrison() -> CardDefinition {
    CardDefinition {
        name: "Mobile Garrison",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        keywords: vec![Keyword::Crew(2)],
        power: 3,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::Untap {
            up_to: None,
            what: target_filtered(yours(R::Artifact.or(R::Creature)).and(R::OtherThanSource)),
        })],
        ..Default::default()
    }
}

/// Fortuitous Find — {2}{B} sorcery. Choose one or both: return target
/// artifact card / target creature card from your graveyard to your hand.
pub fn fortuitous_find() -> CardDefinition {
    let back = |filter: R| Effect::Move {
        what: target_filtered(filter.and(R::InYourGraveyard)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    spell(
        "Fortuitous Find",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::ChooseModesCast { modes: vec![back(R::Artifact), back(R::Creature)], min: 1, max: 2, allow_repeats: false },
    )
}

// ── Hurska ───────────────────────────────────────────────────────────────────

/// Hurska Sweet-Tooth — {2}{G} 3/3 Legendary Bear. Attacks: a Food. Whenever
/// you gain life, you may pay {G/W}; when you do, target creature gets +X/+X
/// until end of turn, X the life gained.
pub fn hurska_sweet_tooth() -> CardDefinition {
    let x = || Value::TriggerEventAmount;
    CardDefinition {
        triggered_abilities: vec![
            on_attack(food()),
            you_gain_life(Effect::MayPay {
                description: "Pay {G/W} to pump a creature?".into(),
                mana_cost: cost(&[hybrid(Color::Green, Color::White)]),
                // CR 603.12 — "When you do": the target is chosen after the
                // payment; X (the life gained) rides in as the trigger's X.
                body: Box::new(Effect::WithX {
                    x: x(),
                    body: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::PumpPT {
                            what: target_filtered(R::Creature),
                            power: Value::XFromCost,
                            toughness: Value::XFromCost,
                            duration: Duration::EndOfTurn,
                        }),
                    }),
                }),
                else_: None,
            }),
        ],
        ..legend("Hurska Sweet-Tooth", cost(&[generic(2), g()]), vec![CreatureType::Bear], 3, 3)
    }
}

/// Heroic Feast — {2}{G} enchantment. Enters: a Food. Whenever you gain life,
/// put a +1/+1 counter on each of up to that many target creatures you
/// control (CR 601.2c: the cap is read as the trigger resolves).
pub fn heroic_feast() -> CardDefinition {
    CardDefinition {
        name: "Heroic Feast",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(food()),
            you_gain_life(Effect::CapTargetsAt {
                amount: Value::TriggerEventAmount,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 10,
                    min_targets: 0,
                    filter: yours(R::Creature),
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                }),
            }),
        ],
        ..Default::default()
    }
}

// ── RMS Titanic ──────────────────────────────────────────────────────────────

/// Dowsing Device // Geode Grotto — {1}{R} artifact. It or another artifact
/// of yours entering gives up to one creature of yours +1/+0 and haste, then
/// transforms it if you control four or more artifacts. Back: a Cave land,
/// {T}: {R}; {2}{R}, {T}: target creature gains haste and +X/+0 (X = your
/// artifacts), sorcery speed.
pub fn dowsing_device() -> CardDefinition {
    let artifacts = || Value::count(Selector::EachPermanent(yours(R::Artifact)));
    let grotto = CardDefinition {
        name: "Geode Grotto",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: artifacts(),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Dowsing Device",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::Seq(vec![
                up_to_one(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(yours(R::Creature)),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ])),
                Effect::If {
                    cond: Predicate::ValueAtLeast(artifacts(), Value::Const(4)),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(grotto)),
        ..Default::default()
    }
}

/// Fatal Frenzy — {2}{R} instant. Target creature you control gains trample
/// and gets +X/+0 until end of turn (X its power); sacrifice it at the
/// beginning of the next end step.
pub fn fatal_frenzy() -> CardDefinition {
    spell(
        "Fatal Frenzy",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(yours(R::Creature)),
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::SacrificeAtNextEndStep { what: Selector::Target(0) },
        ]),
    )
}

// ── Mu Yanling, Wind Rider ───────────────────────────────────────────────────

/// Mu Yanling, Sky Dancer — {1}{U}{U} planeswalker, loyalty 2. +2: until your
/// next turn, up to one target creature gets -2/-0 and loses flying. −3: a
/// 4/4 blue Elemental Bird with flying. −8: emblem — Islands you control have
/// "{T}: Draw a card."
pub fn mu_yanling_sky_dancer() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Elemental Bird".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental, CreatureType::Bird], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Mu Yanling, Sky Dancer",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Yanling], ..Default::default() },
        base_loyalty: 2,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: up_to_one(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::Const(-2),
                        toughness: Value::Const(0),
                        duration: Duration::UntilNextTurn,
                    },
                    Effect::LoseKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Flying,
                        duration: Duration::UntilNextTurn,
                    },
                ])),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(bird) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Mu Yanling, Sky Dancer".into(),
                    statics: vec![StaticAbility {
                        description: "Islands you control have \"{T}: Draw a card.\"",
                        effect: StaticEffect::GrantActivatedAbility {
                            applies_to: Selector::EachPermanent(yours(R::HasLandType(LandType::Island))),
                            ability: ActivatedAbility {
                                tap_cost: true,
                                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                                ..Default::default()
                            },
                            condition: None,
                        },
                    }],
                    triggered: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Anchor to Reality — {2}{U}{U} sorcery. As an additional cost, sacrifice an
/// artifact or creature. Search for an Equipment or Vehicle card onto the
/// battlefield; if its mana value is less than the sacrificed one's, scry 2.
pub fn anchor_to_reality() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact.or(R::Creature),
            count: 1,
        }],
        ..spell(
            "Anchor to Reality",
            cost(&[generic(2), u(), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment)
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::SacrificedManaValue,
                        Value::Sum(vec![Value::ManaValueOf(Box::new(Selector::LastMoved)), Value::ONE]),
                    ),
                    then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}
