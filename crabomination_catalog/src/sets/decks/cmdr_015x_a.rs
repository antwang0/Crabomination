//! Commander: 2026-10-18 (`015XaGkQ`) one-deck closers. Tests in
//! `tests/recent_b/cmdr_015x_a.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::card::MayPlayDuration;
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Effect, Predicate, PlayerRef, Selector, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};
use std::sync::Arc;

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn transform_self(mana: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        effect: Effect::Transform { what: Selector::This },
        ..Default::default()
    }
}

/// April O'Neil, Hacktivist — {3}{U} legendary 1/5 Human Scientist. At the
/// beginning of your end step, draw a card for each card type among spells
/// you've cast this turn.
pub fn april_oneil_hacktivist() -> CardDefinition {
    legend(CardDefinition {
        name: "April O'Neil, Hacktivist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Scientist], ..Default::default() },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::You),
                amount: Value::CardTypesAmongSpellsCastThisTurn,
            },
        }],
        ..Default::default()
    })
}

/// The Incredible Hulk — Bruce Banner's back face, an 8/8 reach, trample
/// Gamma Berserker Hero. Enrage: a +1/+1 counter; if he's attacking, untap
/// him and there is an additional combat phase after this one (CR 500.8).
fn the_incredible_hulk() -> CardDefinition {
    legend(CardDefinition {
        name: "The Incredible Hulk",
        cost: cost(&[generic(2), r(), r(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gamma, CreatureType::Berserker, CreatureType::Hero],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    })
}

/// Bruce Banner // The Incredible Hulk — {U} legendary 1/1 Human Scientist
/// Hero. {X}{X}, {T}: draw X (sorcery speed). {2}{R}{R}{G}{G}: transform
/// (sorcery speed). A modal DFC: the Hulk face may be cast too.
pub fn bruce_banner() -> CardDefinition {
    legend(CardDefinition {
        name: "Bruce Banner",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[x(), x()]),
                sorcery_speed: true,
                effect: Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::XFromCost },
                ..Default::default()
            },
            transform_self(cost(&[generic(2), r(), r(), g(), g()])),
        ],
        back_face: Some(Box::new(the_incredible_hulk())),
        ..Default::default()
    })
}

/// Amazing Spider-Man — Peter Parker's back face, a 4/4 vigilance, reach
/// Spider Human Hero. Each legendary spell you cast that's one or more colors
/// has web-slinging {G}{W}{U} (CR 702.188).
fn amazing_spider_man() -> CardDefinition {
    legend(CardDefinition {
        name: "Amazing Spider-Man",
        cost: cost(&[generic(1), g(), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Each legendary spell you cast that's one or more colors has web-slinging {G}{W}{U}.",
            effect: StaticEffect::GrantWebSlingingToSpells {
                filter: R::HasSupertype(Supertype::Legendary).and(R::Colorless.negate()),
                cost: cost(&[g(), w(), u()]),
            },
        }],
        ..Default::default()
    })
}

/// Peter Parker // Amazing Spider-Man — {1}{W} legendary 0/1 Human Scientist
/// Hero. Enters: a 2/1 green Spider token with reach. {1}{G}{W}{U}:
/// transform (sorcery speed). A modal DFC.
pub fn peter_parker() -> CardDefinition {
    legend(CardDefinition {
        name: "Peter Parker",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(TokenDefinition {
                name: "Spider".into(),
                power: 2,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
                keywords: vec![Keyword::Reach],
                ..Default::default()
            }),
        })],
        activated_abilities: vec![transform_self(cost(&[generic(1), g(), w(), u()]))],
        back_face: Some(Box::new(amazing_spider_man())),
        ..Default::default()
    })
}

/// Public Enemy — {2}{U} Aura. All creatures attack enchanted creature's
/// controller each combat if able (CR 508.1d). When enchanted creature dies,
/// draw a card.
pub fn public_enemy() -> CardDefinition {
    CardDefinition {
        name: "Public Enemy",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "All creatures attack enchanted creature's controller each combat if able.",
            effect: StaticEffect::AllCreaturesAttackEnchantedController,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Maraxus of Keld — {4}{R}{R} legendary */* Human Warrior; its power and
/// toughness are each the number of untapped artifacts, creatures, and lands
/// you control (CR 604.3 characteristic-defining).
pub fn maraxus_of_keld() -> CardDefinition {
    let untapped = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::Artifact.or(R::Creature).or(R::Land).and(R::Untapped).and(R::ControlledByYou),
        )))
    };
    legend(CardDefinition {
        name: "Maraxus of Keld",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Warrior], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Maraxus's power and toughness are each equal to the number of untapped artifacts, creatures, and lands you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: untapped(), toughness: untapped() },
        }],
        ..Default::default()
    })
}

/// Prowl, Pursuit Vehicle — Prowl's converted face, a 2/3 living metal
/// Vehicle. Another creature or Vehicle of yours entering puts a +1/+1
/// counter on it; the second resolution in a turn also converts it.
fn prowl_pursuit_vehicle() -> CardDefinition {
    let counter = || Effect::AddCounter {
        what: Selector::This,
        kind: crate::card::CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    legend(CardDefinition {
        name: "Prowl, Pursuit Vehicle",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![Color::White],
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::LivingMetal],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                },
            ),
            effect: Effect::EscalatingThisTurn {
                modes: vec![
                    counter(),
                    Effect::Seq(vec![counter(), Effect::Transform { what: Selector::This }]),
                    counter(),
                ],
            },
        }],
        ..Default::default()
    })
}

/// Prowl, Stoic Strategist // Prowl, Pursuit Vehicle — {3}{W} legendary 3/3
/// artifact creature Robot; More Than Meets the Eye {2}{W}. Attacking, it
/// exiles up to one other tapped creature or Vehicle, whose owner may play it
/// while it stays exiled; a card played from among those draws you a card
/// and converts Prowl (CR 701.28).
pub fn prowl_stoic_strategist() -> CardDefinition {
    let played = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::AnyPlayer).with_filter(Predicate::TriggerCardExiledWithSource),
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
            Effect::Transform { what: Selector::This },
        ]),
    };
    legend(CardDefinition {
        name: "Prowl, Stoic Strategist",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 3,
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), w()]),
            converted: true,
            ..Default::default()
        }),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::ExileWithSource {
                            what: target_filtered(
                                R::Creature
                                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                                    .and(R::Tapped)
                                    .and(R::OtherThanSource),
                            ),
                        },
                        Effect::GrantMayPlay {
                            what: Selector::CardExiledWithSource,
                            duration: MayPlayDuration::WhileExiled,
                            to_owner: true,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        },
                    ])),
                },
            },
            played(EventKind::SpellCast),
            played(EventKind::LandPlayed),
        ],
        back_face: Some(Box::new(prowl_pursuit_vehicle())),
        ..Default::default()
    })
}

/// Codie, Ravenous Codex — {3} legendary 1/4 artifact creature Book
/// Construct. Casting a prepared spell copies it (new targets allowed).
/// {W}{U}{B}{R}{G}, {T}: each creature you control with a prepare spell
/// becomes prepared.
pub fn codie_ravenous_codex() -> CardDefinition {
    legend(CardDefinition {
        name: "Codie, Ravenous Codex",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Book, CreatureType::Construct], ..Default::default() },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsPreparedSpell,
            }),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasPrepareSpell)),
                kind: crate::card::CounterType::Prepared,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    })
}

/// Nick Fury, Agent of S.H.I.E.L.D. — {W} legendary 2/1 Human Spy Hero.
/// Power-up — {W}{U}{B}{R}{G}: two +1/+1 counters, then look at the top
/// seven and put a Hero, Equipment, or Vehicle card from among them onto the
/// battlefield; the rest go to the bottom in a random order (CR 702.193).
pub fn nick_fury_agent_of_shield() -> CardDefinition {
    legend(CardDefinition {
        name: "Nick Fury, Agent of S.H.I.E.L.D.",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::LookTopPutMatchingOntoBattlefield {
                    count: Value::Const(7),
                    filter: R::HasCreatureType(CreatureType::Hero)
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                    then: None,
                    max: Some(1),
                    tapped: false,
                    exile_rest: false,
                    rest_to_graveyard: false,
                    mandatory: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    })
}

/// Ultimate Spider-Man — Miles Morales's back face, a 4/3 first strike,
/// haste Spider Human Hero. Camouflage — {2}: a +1/+1 counter, hexproof and
/// colorless until end of turn. Whenever you attack, double each kind of
/// counter on each Spider and legendary creature you control.
fn ultimate_spider_man() -> CardDefinition {
    legend(CardDefinition {
        name: "Ultimate Spider-Man",
        cost: cost(&[generic(3), r(), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Hexproof,
                    duration: crate::effect::Duration::EndOfTurn,
                },
                Effect::BecomeColor {
                    what: Selector::This,
                    colors: vec![],
                    duration: crate::effect::Duration::EndOfTurn,
                    additive: false,
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::DoubleAllCountersOn {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Spider).or(R::HasSupertype(Supertype::Legendary))),
                ),
            },
        }],
        ..Default::default()
    })
}

/// Miles Morales // Ultimate Spider-Man — {1}{G} legendary 1/2 Human Citizen
/// Hero. Enters: a +1/+1 counter on each of up to two target creatures.
/// {3}{R}{G}{W}: transform (sorcery speed). A modal DFC.
pub fn miles_morales() -> CardDefinition {
    legend(CardDefinition {
        name: "Miles Morales",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen, CreatureType::Hero],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        })],
        activated_abilities: vec![transform_self(cost(&[generic(3), r(), g(), w()]))],
        back_face: Some(Box::new(ultimate_spider_man())),
        ..Default::default()
    })
}

/// Multiversal Passage — land. As it enters, choose a basic land type, then
/// pay 2 life or it enters tapped (CR 614.12). It is the chosen type, and so
/// taps for that type's mana (CR 305.6).
pub fn multiversal_passage() -> CardDefinition {
    CardDefinition {
        name: "Multiversal Passage",
        card_types: vec![CardType::Land],
        as_enters_effect: Some(Effect::Seq(vec![
            Effect::ChooseBasicLandTypeForSource,
            crate::sets::shockland_pay_two_or_tap(),
        ])),
        static_abilities: vec![StaticAbility {
            description: "This land is the chosen type.",
            effect: StaticEffect::SelfIsChosenLandType,
        }],
        ..Default::default()
    }
}

/// Falco Spara, Pactweaver — {1}{G}{W}{U} legendary 3/3 flying, trample Bird
/// Demon; enters with a shield counter. You may look at the top card of your
/// library any time, and cast spells from there by removing a counter from a
/// creature you control in addition to their other costs.
pub fn falco_spara_pactweaver() -> CardDefinition {
    legend(CardDefinition {
        name: "Falco Spara, Pactweaver",
        cost: cost(&[generic(1), g(), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird, CreatureType::Demon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        enters_with_counters: Some((crate::card::CounterType::Shield, Value::ONE)),
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast spells from the top of your library by removing a counter from a creature you control in addition to paying their other costs.",
                effect: StaticEffect::PlayFromLibraryTopByRemovingCounter { filter: R::Any },
            },
        ],
        ..Default::default()
    })
}

/// Evelyn, the Covetous — {2}{U/B}{B}{B/R} legendary 2/5 flash Vampire Rogue.
/// Whenever a Vampire you control enters (Evelyn included), exile the top
/// card of each player's library with a collection counter on it. Once each
/// turn you may play one of them, spending mana as though it were any color.
pub fn evelyn_the_covetous() -> CardDefinition {
    use crate::mana::{Color, hybrid};
    legend(CardDefinition {
        name: "Evelyn, the Covetous",
        cost: cost(&[generic(2), hybrid(Color::Blue, Color::Black), b(), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Vampire),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: false,
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: crate::card::CounterType::Collection,
                    amount: Value::ONE,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::WhileSourceOnBattlefield {
                        source: crate::card::CardId(0),
                        one_spell_a_turn: true,
                        follows_control: false,
                    },
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: true,
                },
            ]),
        }],
        ..Default::default()
    })
}

/// Garza Zol, Plague Queen — {4}{U}{B}{R} legendary 5/5 flying, haste Vampire
/// Noble. A creature it dealt damage to this turn dying puts a +1/+1 counter
/// on it; its combat damage to a player may draw a card.
pub fn garza_zol_plague_queen() -> CardDefinition {
    legend(CardDefinition {
        name: "Garza Zol, Plague Queen",
        cost: cost(&[generic(4), u(), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire, CreatureType::Noble], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::DamagedBySourceThisTurn },
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Draw a card?".into(),
                    body: Box::new(Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE }),
                },
            },
        ],
        ..Default::default()
    })
}

/// A Transformers back face: a legendary artifact Vehicle with living metal
/// and a printed color indicator (CR 105.2c).
fn converted_vehicle(name: &'static str, colors: Vec<Color>, power: i32, toughness: i32) -> CardDefinition {
    legend(CardDefinition {
        name,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: colors,
        power,
        toughness,
        keywords: vec![Keyword::LivingMetal],
        ..Default::default()
    })
}

/// A Transformers front face: a legendary artifact creature Robot cast
/// converted for its More Than Meets the Eye cost (CR 702.162).
fn robot_front(name: &'static str, c: crate::mana::ManaCost, mtmte: crate::mana::ManaCost, pt: (i32, i32), back: CardDefinition) -> CardDefinition {
    legend(CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: pt.0,
        toughness: pt.1,
        alternative_cost: Some(AlternativeCost { mana_cost: mtmte, converted: true, ..Default::default() }),
        back_face: Some(Box::new(back)),
        ..Default::default()
    })
}

/// Optimus Prime, Hero // Optimus Prime, Autobot Leader — {3}{U}{R}{W} 4/8;
/// More Than Meets the Eye {2}{U}{R}{W}. Each end step, bolster 1 (CR
/// 701.39); dying, it returns converted. The 6/8 trample Vehicle: whenever
/// you attack, bolster 2, the chosen creature gains trample, and when it
/// deals combat damage to a player this turn, Optimus converts.
pub fn optimus_prime_hero() -> CardDefinition {
    let mut leader = converted_vehicle("Optimus Prime, Autobot Leader", vec![Color::Blue, Color::Red, Color::White], 6, 8);
    leader.keywords.push(Keyword::Trample);
    leader.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
        effect: Effect::ForEach {
            selector: Selector::LeastToughnessYouControl,
            body: Box::new(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Trample,
                    duration: crate::effect::Duration::EndOfTurn,
                },
                Effect::WhenObjectDealsCombatDamageToPlayerThisTurn {
                    what: Selector::TriggerSource,
                    body: Box::new(Effect::Transform { what: Selector::This }),
                },
            ])),
        },
    });
    let mut hero = robot_front(
        "Optimus Prime, Hero",
        cost(&[generic(3), u(), r(), w()]),
        cost(&[generic(2), u(), r(), w()]),
        (4, 8),
        leader,
    );
    hero.triggered_abilities = vec![
        TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::LeastToughnessYouControl,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ExileSelfReturnTransformed,
        },
    ];
    hero
}

/// Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier — {4}{R}{G}{W}
/// 7/7 ward {2}; More Than Meets the Eye {2}{R}{G}{W}. Attacking, it may put
/// an artifact creature card from hand onto the battlefield tapped and
/// attacking, and then converts at end of combat. The 4/7 haste Vehicle's
/// Formidable: attacking creatures you control gain indestructible, and at
/// total power 8 or more it converts.
pub fn ultra_magnus_tactician() -> CardDefinition {
    let attackers = || Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou));
    let mut carrier = converted_vehicle("Ultra Magnus, Armored Carrier", vec![Color::Red, Color::Green, Color::White], 4, 7);
    carrier.keywords.push(Keyword::Haste);
    carrier.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: attackers(),
                keyword: Keyword::Indestructible,
                duration: crate::effect::Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::PowerOf(Box::new(attackers())), Value::Const(8)),
                then: Box::new(Effect::Transform { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    });
    let mut tactician = robot_front(
        "Ultra Magnus, Tactician",
        cost(&[generic(4), r(), g(), w()]),
        cost(&[generic(2), r(), g(), w()]),
        (7, 7),
        carrier,
    );
    tactician.keywords = vec![Keyword::Ward(crate::card::WardCost::generic(2))];
    tactician.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::DeployCreatureFromHandAttacking {
                filter: R::Artifact.and(R::Creature),
                return_to_hand_eot: false, same_defender: false,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::CountOf(Box::new(Selector::LastMoved)), Value::ONE),
                then: Box::new(Effect::AtEndOfCombat { body: Box::new(Effect::Transform { what: Selector::This }) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    }];
    tactician
}

/// Rootpath Purifier — {3}{G} 3/4 Elf Druid. Lands you control and land
/// cards in your library are basic (no basic land type comes with it).
pub fn rootpath_purifier() -> CardDefinition {
    CardDefinition {
        name: "Rootpath Purifier",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Druid], ..Default::default() },
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Lands you control and land cards in your library are basic.",
            effect: StaticEffect::LandsAndLibraryLandsAreBasic,
        }],
        ..Default::default()
    }
}

/// Boiling Rock Rioter — {2}{B} 3/3 Human Rogue Ally; firebending 1. Tap an
/// untapped Ally you control: exile target card from a graveyard (linked to
/// it). Whenever it attacks, you may cast an Ally spell from among cards you
/// own exiled with it, paying its cost.
pub fn boiling_rock_rioter() -> CardDefinition {
    CardDefinition {
        name: "Boiling Rock Rioter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Firebending(1)],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::HasCreatureType(CreatureType::Ally), 1)),
            effect: Effect::ExileWithSource { what: target_filtered(R::InGraveyard) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CastWithoutPayingImmediate {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::CardExiledWithSource),
                    filter: R::HasCreatureType(CreatureType::Ally).and(R::OwnedByYou),
                },
                source_zone: crate::card::Zone::Exile,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: true,
            },
        }],
        ..Default::default()
    }
}

/// Nahiri's Wrath — {2}{R} sorcery. As an additional cost, discard X cards.
/// It deals damage equal to the discarded cards' total mana value to each of
/// up to X target creatures and/or planeswalkers (CR 107.3d — X is the
/// cast's choice; no `{X}` in the cost).
pub fn nahiris_wrath() -> CardDefinition {
    CardDefinition {
        name: "Nahiri's Wrath",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::DiscardXFromCost],
        effect: Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature.or(R::Planeswalker),
                effect: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::CastDiscardedManaValue,
                }),
            }),
        },
        ..Default::default()
    }
}

/// Nathan Drake, Treasure Hunter — {U}{B}{R} legendary 3/2 first strike
/// Human Rogue. Whenever it attacks, exile the top card of each player's
/// library; you may cast a spell from among them — one you don't own with
/// mana spent as though any color (its static), else your own.
pub fn nathan_drake_treasure_hunter() -> CardDefinition {
    let exiled = |filter: R| Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter };
    legend(CardDefinition {
        name: "Nathan Drake, Treasure Hunter",
        cost: cost(&[u(), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rogue], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                    link_to_source: false,
                    face_down: false,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountOf(Box::new(exiled(R::Nonland.and(R::OwnedByYou.negate())))),
                        Value::ONE,
                    ),
                    then: Box::new(Effect::CastImmediateAnyManaType {
                        what: exiled(R::Nonland.and(R::OwnedByYou.negate())),
                        source_zone: crate::card::Zone::Exile,
                    }),
                    else_: Box::new(Effect::CastWithoutPayingImmediate {
                        what: exiled(R::Nonland.and(R::OwnedByYou)),
                        source_zone: crate::card::Zone::Exile,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: true,
                    }),
                },
            ]),
        }],
        ..Default::default()
    })
}

/// Kotose, the Silent Spider — {3}{U}{B} legendary 4/4 Human Ninja. Entering,
/// it exiles target non-basic-land card from an opponent's graveyard and every
/// card of that name from that player's graveyard, hand, and library; while
/// you control Kotose you may play one of them, mana as though any color.
pub fn kotose_the_silent_spider() -> CardDefinition {
    legend(CardDefinition {
        name: "Kotose, the Silent Spider",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Ninja], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::ExileSameNameAsTarget {
                what: target_filtered(R::InOpponentGraveyard.and(R::IsBasicLand.negate())),
            },
            // Exactly one of them, ever, and only while you control Kotose
            // (rulings 2022-02-18; CR 611.2b).
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileHolderControlsSource { source: crate::card::CardId(0), your_turn: false },
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
            Effect::OneCastAmongGranted { what: Selector::LastMoved },
        ]))],
        ..Default::default()
    })
}

/// Flamewar, Brash Veteran // Flamewar, Streetwise Operative — {1}{B}{R} 3/2
/// legendary artifact creature Robot; More Than Meets the Eye {B}{R}.
/// Sacrifice another artifact: a +1/+1 counter and convert (sorcery speed).
/// {1}, discard your hand: your exiled cards with intel counters go to your
/// hand. The 2/1 menace, deathtouch Vehicle's combat damage to a player
/// exiles that many of your top cards face down with intel counters, then
/// converts it.
pub fn flamewar_brash_veteran() -> CardDefinition {
    let mut operative =
        converted_vehicle("Flamewar, Streetwise Operative", vec![Color::Black, Color::Red], 2, 1);
    operative.keywords.extend([Keyword::Menace, Keyword::Deathtouch]);
    operative.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::You),
                amount: Value::TriggerEventAmount,
                link_to_source: false,
                face_down: true,
            },
            Effect::AddCounter { what: Selector::LastMoved, kind: crate::card::CounterType::Intel, amount: Value::ONE },
            Effect::Transform { what: Selector::This },
        ]),
    });
    let mut veteran = robot_front(
        "Flamewar, Brash Veteran",
        cost(&[generic(1), b(), r()]),
        cost(&[b(), r()]),
        (3, 2),
        operative,
    );
    veteran.activated_abilities = vec![
        ActivatedAbility {
            sac_other_filter: Some((R::Artifact.and(R::OtherThanSource), 1)),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Transform { what: Selector::This },
            ]),
            ..Default::default()
        },
        ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_hand_cost: true,
            effect: Effect::Move {
                what: Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Exile,
                    filter: R::WithCounter(crate::card::CounterType::Intel).and(R::OwnedByYou),
                },
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        },
    ];
    veteran
}

/// A legendary Robot artifact creature token (Ravage, Laserbeak).
fn robot_token(name: &str, color: Color, pt: (i32, i32), keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: pt.0,
        toughness: pt.1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![color],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        keywords,
        ..Default::default()
    })
}

/// Soundwave, Sonic Spy // Soundwave, Superior Captain — {1}{W}{U}{B} 5/4
/// legendary artifact creature Robot; More Than Meets the Eye {2}{W}{U}{B}.
/// When creature tokens you control deal combat damage to a player, exile an
/// instant or sorcery card with that mana value from their graveyard and you
/// may cast a copy free — if you do, it converts. The Captain (an artifact)
/// converts on each spell you cast, making Ravage (odd mana value) or
/// Laserbeak (even).
pub fn soundwave_sonic_spy() -> CardDefinition {
    let mut captain = legend(CardDefinition {
        name: "Soundwave, Superior Captain",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::White, Color::Blue, Color::Black],
        ..Default::default()
    });
    for (odd, token) in [
        (true, robot_token("Ravage", Color::Black, (3, 3), vec![Keyword::Menace, Keyword::Deathtouch])),
        (false, robot_token("Laserbeak", Color::Blue, (2, 2), vec![Keyword::Flying, Keyword::Hexproof])),
    ] {
        captain.triggered_abilities.push(TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::ManaValueParity { odd },
            }),
            effect: Effect::Seq(vec![
                Effect::Transform { what: Selector::This },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: token },
            ]),
        });
    }
    let mut spy = robot_front(
        "Soundwave, Sonic Spy",
        cost(&[generic(1), w(), u(), b()]),
        cost(&[generic(2), w(), u(), b()]),
        (5, 4),
        captain,
    );
    spy.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsToken) })
            .once_per_batch_summing_damage(),
        effect: Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::MoveChosen {
                from: Selector::CardsInZone {
                    who: PlayerRef::TriggerEventPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: (R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                        .and(R::ManaValueEqualsTriggerAmount),
                },
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: crate::effect::ZoneDest::Exile,
            },
            Effect::CastWithoutPayingImmediate {
                what: Selector::LastMoved,
                source_zone: crate::card::Zone::Exile,
                exile_after: false,
                copy: true,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            Effect::If {
                cond: Predicate::CastImmediatelyThisResolution,
                then: Box::new(Effect::Transform { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    }];
    spy
}

/// Cosmic Spider-Man — {W}{U}{B}{R}{G} legendary 5/5 Spider Human Hero with
/// flying, first strike, trample, lifelink, and haste. At the beginning of
/// combat on your turn, your other Spiders gain all five until end of turn.
pub fn cosmic_spider_man() -> CardDefinition {
    let five = [Keyword::Flying, Keyword::FirstStrike, Keyword::Trample, Keyword::Lifelink, Keyword::Haste];
    legend(CardDefinition {
        name: "Cosmic Spider-Man",
        cost: cost(&[w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: five.to_vec(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(
                five.iter()
                    .map(|k| Effect::GrantKeyword {
                        what: Selector::EachPermanent(
                            R::Creature
                                .and(R::HasCreatureType(CreatureType::Spider))
                                .and(R::ControlledByYou)
                                .and(R::OtherThanSource),
                        ),
                        keyword: k.clone(),
                        duration: crate::effect::Duration::EndOfTurn,
                    })
                    .collect(),
            ),
        }],
        ..Default::default()
    })
}

/// Zog, Triceraton Castaway — {4}{R} legendary 5/4 reach, trample Dinosaur
/// Soldier. Entering, target creature can't block this turn.
/// Mountaincycling {2}.
pub fn zog_triceraton_castaway() -> CardDefinition {
    legend(CardDefinition {
        name: "Zog, Triceraton Castaway",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur, CreatureType::Soldier], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![
            Keyword::Reach,
            Keyword::Trample,
            Keyword::Landcycling(cost(&[generic(2)]), crate::card::LandType::Mountain),
        ],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: crate::effect::Duration::EndOfTurn,
        })],
        ..Default::default()
    })
}

/// Casey & Raph, Hotheads — {4}{R} legendary 4/4 Mutant Ninja Human Turtle.
/// Entering, choose one or both, each mode targeting a different player:
/// that player exiles their top card and may play it free until their next
/// end step; that player creates two Treasures. (`ChooseN` gives each mode
/// its own player slot and never reuses a player.)
pub fn casey_and_raph_hotheads() -> CardDefinition {
    legend(CardDefinition {
        name: "Casey & Raph, Hotheads",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Human, CreatureType::Turtle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::ChooseN { up_to: None,
            picks: vec![0, 1],
            modes: vec![
                Effect::AsPlayer {
                    who: PlayerRef::Target(0),
                    body: Box::new(Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: MayPlayDuration::UntilYourNextEndStep,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: false,
                        uncast_penalty: None,
                    }),
                },
                Effect::CreateToken {
                    who: PlayerRef::Target(0),
                    count: Value::Const(2),
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            ],
        })],
        ..Default::default()
    })
}

/// "When the `n`th plan counter is put on this, sacrifice it. When you do,
/// `then`." — the Plan cycle's payoff (CR 603.12: `then` runs only when the
/// sacrifice happened).
pub(super) fn plan_payoff(n: u32, then: Effect) -> TriggeredAbility {
    use crate::card::CounterType;
    TriggeredAbility {
        event: EventSpec::new(EventKind::CounterAdded(CounterType::Plan), EventScope::SelfSource)
            .with_filter(Predicate::SourceHasCountersAtLeast { counter: CounterType::Plan, n }),
        effect: Effect::Seq(vec![
            Effect::SacrificeSource,
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                // CR 603.12 — "when you do" is a reflexive trigger: its own
                // stack object, targeting after the sacrifice.
                then: Box::new(Effect::ReflexiveTrigger { body: Box::new(then) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    }
}

pub(super) fn plan(name: &'static str, c: crate::mana::ManaCost, triggers: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Plan], ..Default::default() },
        triggered_abilities: triggers,
        ..Default::default()
    }
}

pub(super) fn plan_counter() -> Effect {
    Effect::AddCounter { what: Selector::This, kind: crate::card::CounterType::Plan, amount: Value::ONE }
}

/// Doom Reigns Supreme — {1}{B} enchantment — Plan. A Villain of yours
/// entering drains each opponent for 1 and adds a plan counter. At the fifth,
/// sacrifice it: target opponent exiles their top five and you may cast up to
/// two spells from among them free.
pub fn doom_reigns_supreme() -> CardDefinition {
    plan(
        "Doom Reigns Supreme",
        cost(&[generic(1), b()]),
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Villain),
                    },
                ),
                effect: Effect::Seq(vec![
                    Effect::Drain {
                        from: Selector::Player(PlayerRef::EachOpponent),
                        to: Selector::Player(PlayerRef::You),
                        amount: Value::ONE,
                    },
                    plan_counter(),
                ]),
            },
            plan_payoff(
                5,
                Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::ExileTopOfLibrary {
                        who: target_filtered(R::OpponentPlayer),
                        amount: Value::Const(5),
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::CastAnyOrderWithoutPaying {
                        what: Selector::LastMoved,
                        source_zone: crate::card::Zone::Exile,
                        filter: None,
                        cap: Some(Value::Const(2)),
                        total_mana_value: None,
                        exile_after: false,
                    },
                ]),
            ),
        ],
    )
}

/// Construct a Cosmic Cube — {2}{B} enchantment — Plan. Drawing your second
/// card each turn makes a 2/1 black menace Villain and adds a plan counter.
/// At the seventh, sacrifice it: you control target opponent during their
/// next turn (CR 723.1).
pub fn construct_a_cosmic_cube() -> CardDefinition {
    plan(
        "Construct a Cosmic Cube",
        cost(&[generic(2), b()]),
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(TokenDefinition {
                            name: "Villain".into(),
                            power: 2,
                            toughness: 1,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: Subtypes { creature_types: vec![CreatureType::Villain], ..Default::default() },
                            keywords: vec![Keyword::Menace],
                            ..Default::default()
                        }),
                    },
                    plan_counter(),
                ]),
            },
            plan_payoff(
                7,
                Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::ControlPlayerNextTurn { who: PlayerRef::Target(0) }),
                },
            ),
        ],
    )
}

/// Jeweled Amulet — {0} artifact. {1}, {T}: a charge counter, noting the
/// type of mana spent (only with no charge counter on it). {T}, remove a
/// charge counter: add one mana of the last noted type ({C} for colorless).
pub fn jeweled_amulet() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Jeweled Amulet",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                condition: Some(Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::Charge,
                    n: 1,
                }))),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                    Effect::NoteSpentManaTypeOnSource,
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lodestone Bauble — {0} artifact. {1}, {T}, sacrifice it: put up to four
/// target basic land cards from a player's graveyard on top of their library;
/// that player draws a card at the beginning of the next turn's upkeep.
pub fn lodestone_bauble() -> CardDefinition {
    CardDefinition {
        name: "Lodestone Bauble",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 4,
                    min_targets: 0,
                    filter: R::IsBasicLand.and(R::InGraveyard).and(R::SameGraveyardAsTargetSlot(0)),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: crate::effect::ZoneDest::Library {
                            who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            pos: crate::effect::LibraryPosition::Top,
                        },
                    }),
                },
                Effect::AsPlayer {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    body: Box::new(Effect::AtNextTurnsUpkeep {
                        body: Box::new(Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE }),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tazri, Stalwart Survivor — {2}{W} legendary 3/3 Human Warrior. Each
/// creature you control with another activated ability has "{T}: add one
/// mana of any of this creature's colors, spent only on creature abilities".
/// {W}{U}{B}{R}{G}, {T}: mill five; every creature card among them with a
/// non-mana activated ability goes to your hand.
pub fn tazri_stalwart_survivor() -> CardDefinition {
    legend(CardDefinition {
        name: "Tazri, Stalwart Survivor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Warrior], ..Default::default() },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Each creature you control has \"{T}: Add one mana of any of this creature's colors. Spend this mana only to activate an ability of a creature. Activate only if this creature has another activated ability.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasActivatedAbility)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: crate::effect::ManaPayload::Restricted(
                            Box::new(crate::effect::ManaPayload::AnyColorOfSource),
                            crate::mana::SpendRestriction::CreatureAbilitiesOnly,
                        ),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::MillThenToHandN {
                amount: Value::Const(5),
                filter: R::Creature.and(R::HasNonManaActivatedAbility),
                take: Value::Const(5),
                otherwise: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    })
}

/// Sensational Spider-Man — {1}{W}{U} legendary 3/3 Spider Human Hero.
/// Attacking, it taps target creature defending player controls and stuns
/// it; then you may remove up to three stun counters from among all
/// permanents and draw that many.
pub fn sensational_spider_man() -> CardDefinition {
    use crate::card::CounterType;
    legend(CardDefinition {
        name: "Sensational Spider-Man",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                Effect::RemoveCountersOfKindAmongUpTo { kind: CounterType::Stun, max: Value::Const(3), filter: R::Any },
                Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::CountersRemovedThisEffect },
            ]),
        }],
        ..Default::default()
    })
}

/// Involuntary Cooldown — {3}{U} sorcery. Tap up to two target artifacts
/// and/or creatures and put two stun counters on each.
pub fn involuntary_cooldown() -> CardDefinition {
    CardDefinition {
        name: "Involuntary Cooldown",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Artifact.or(R::Creature),
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: crate::card::CounterType::Stun,
                    amount: Value::Const(2),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Delif's Cone — {0} artifact. {T}, sacrifice it: this turn, when target
/// creature you control attacks and isn't blocked, you may gain life equal
/// to its power; if you do, it assigns no combat damage this turn.
pub fn delifs_cone() -> CardDefinition {
    CardDefinition {
        name: "Delif's Cone",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            // "Target creature you control": the slot's filter, declared by
            // the wrapper (the watch itself only reads the slot).
            effect: Effect::TargetPlayerThen {
                filter: R::Creature.and(R::ControlledByYou),
                then: Box::new(Effect::WhenTargetAttacksUnblockedThisTurn {
                slot: 0,
                body: Box::new(Effect::MayDo {
                    description: "Gain life equal to its power (it then deals no combat damage)?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::GainLife {
                            who: Selector::Player(PlayerRef::You),
                            amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                        },
                        Effect::AssignsNoCombatDamageThisTurn { what: Selector::TriggerSource },
                    ])),
                }),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Energy Arc — {W}{U} instant. Untap any number of target creatures; all
/// combat damage to and by them is prevented this turn.
pub fn energy_arc() -> CardDefinition {
    CardDefinition {
        name: "Energy Arc",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::PreventCombatDamageToTargetThisTurn { target: Selector::Target(0) },
                Effect::PreventCombatDamageByTargetThisTurn { target: Selector::Target(0) },
            ])),
        },
        ..Default::default()
    }
}

/// Robaran Mercenaries — {3}{W} 3/4 vigilance Human Mercenary; it has all
/// activated abilities of all legendary creatures you control.
pub fn robaran_mercenaries() -> CardDefinition {
    CardDefinition {
        name: "Robaran Mercenaries",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Mercenary], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "This creature has all activated abilities of all legendary creatures you control.",
            effect: StaticEffect::HasActivatedAbilitiesOfControlledLegendaryCreatures,
        }],
        ..Default::default()
    }
}

/// Mutant Town Musicians — {2}{R} 2/4 trample Mutant Bard Performer.
/// Alliance — another creature of yours entering gives it +1/+0 until end of
/// turn.
pub fn mutant_town_musicians() -> CardDefinition {
    CardDefinition {
        name: "Mutant Town Musicians",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Bard, CreatureType::Performer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Clairvoyance — {U} instant. Look at target player's hand; draw a card at
/// the beginning of the next turn's upkeep.
pub fn clairvoyance() -> CardDefinition {
    CardDefinition {
        name: "Clairvoyance",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LookAtHand { who: target_filtered(R::Player) },
            Effect::AtNextTurnsUpkeep {
                body: Box::new(Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE }),
            },
        ]),
        ..Default::default()
    }
}

/// Sorcerous Sight — {U} sorcery. Look at target opponent's hand; draw a
/// card.
pub fn sorcerous_sight() -> CardDefinition {
    CardDefinition {
        name: "Sorcerous Sight",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LookAtHand { who: target_filtered(R::OpponentPlayer) },
            Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Sylvan Basilisk — {3}{G}{G} 2/4 Basilisk. Whenever it becomes blocked by
/// a creature, destroy that creature (each blocker).
pub fn sylvan_basilisk() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Basilisk",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Basilisk], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::InCombatWithSource).and(R::OtherThanSource)),
            },
        }],
        ..Default::default()
    }
}

/// Kjeldoran Warrior — {W} 1/1 Human Warrior with banding.
pub fn kjeldoran_warrior() -> CardDefinition {
    CardDefinition {
        name: "Kjeldoran Warrior",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Warrior], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Banding],
        ..Default::default()
    }
}

/// Definitely Not a Turtle — {3} 3/2 Mutant Ninja Turtle. Dying, it looks at
/// your top six and may put a land or legendary Turtle card from among them
/// into your hand; the rest go to the bottom in a random order.
pub fn definitely_not_a_turtle() -> CardDefinition {
    CardDefinition {
        name: "Definitely Not a Turtle",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::LookPickToHand(Box::new(
            crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(6),
                pick_filter: Some(
                    R::Land.or(R::HasCreatureType(CreatureType::Turtle).and(R::HasSupertype(Supertype::Legendary))),
                ),
                optional: true,
                ..Default::default()
            },
        )))],
        ..Default::default()
    }
}

/// Needle Specter — {1}{B}{B} 1/1 flying, wither Specter; its combat damage
/// to a player makes them discard that many cards.
pub fn needle_specter() -> CardDefinition {
    CardDefinition {
        name: "Needle Specter",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Specter], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Wither],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::TriggerEventAmount,
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Shimian Specter — {2}{B}{B} 2/2 flying Specter. Its combat damage to a
/// player reveals their hand; you pick a nonland card (the priciest, headless)
/// and every card of that name leaves their graveyard, hand, and library.
pub fn shimian_specter() -> CardDefinition {
    CardDefinition {
        name: "Shimian Specter",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Specter], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RevealHand { who: PlayerRef::TriggerEventPlayer },
                Effect::ExileSameNameAsTarget {
                    what: Selector::TakeGreatestManaValue {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::TriggerEventPlayer,
                            zone: crate::card::Zone::Hand,
                            filter: R::Nonland,
                        }),
                        count: Box::new(Value::ONE),
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}
