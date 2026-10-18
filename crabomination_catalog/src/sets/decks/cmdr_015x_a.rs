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
                return_to_hand_eot: false,
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
