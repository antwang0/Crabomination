//! Ravnica Allegiance (RNA) wave 2 — the rares/mythics that close the set's
//! `set_gaps.py` list. Tests in `classic_sets/rna`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, StaticEffect};
use crate::mana::{b, cost, r, u, w, x, Color, generic};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

fn token(
    name: &'static str,
    colors: Vec<Color>,
    p: i32,
    t: i32,
    types: Vec<CardType>,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords: kw,
        card_types: types,
        colors,
        subtypes: creatures(ct),
        ..Default::default()
    }
}

/// Dovin, Grand Arbiter — {1}{W}{U} planeswalker, loyalty 3.
/// +1: this turn, your creatures' combat damage to a player grows him.
/// −1: a 1/1 flying Thopter and 1 life. −7: look at ten, take three.
pub fn dovin_grand_arbiter() -> CardDefinition {
    CardDefinition {
        name: "Dovin, Grand Arbiter",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Dovin],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreaturesYouControlDealingCombatDamageThisTurn {
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Loyalty,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: token(
                            "Thopter",
                            vec![],
                            1,
                            1,
                            vec![CardType::Artifact, CardType::Creature],
                            vec![CreatureType::Thopter],
                            vec![Keyword::Flying],
                        ),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(10),
                    rest_to_graveyard: false,
                    pick_filter: None,
                    take: Some(Value::Const(3)),
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: false,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: true,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gideon, Champion of Justice — {2}{W}{W} planeswalker, loyalty 4.
/// +1: a loyalty counter per creature target opponent controls. 0: becomes an
/// indestructible, damage-proof Human Soldier with P/T = his loyalty.
/// −15: exile all other permanents.
pub fn gideon_champion_of_justice() -> CardDefinition {
    CardDefinition {
        name: "Gideon, Champion of Justice",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Gideon],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Loyalty,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByOpponent),
                    ))),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Loyalty,
                        },
                        toughness: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Loyalty,
                        },
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        keywords: vec![Keyword::Indestructible],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PreventAllDamageThisTurn { target: Selector::This },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -15,
                effect: Effect::Move {
                    what: Selector::EachPermanent(R::OtherThanSource),
                    to: crate::effect::ZoneDest::Exile,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Teysa Karlov — {2}{W}{B} 2/4 Human Advisor. Death triggers of permanents
/// you control trigger an additional time; your creature tokens have vigilance
/// and lifelink.
pub fn teysa_karlov() -> CardDefinition {
    let token_anthem = |kw: Keyword, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::IsToken).and(R::ControlledByYou),
            ),
            keyword: kw,
        },
    };
    CardDefinition {
        name: "Teysa Karlov",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Advisor]),
        power: 2,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
                effect: StaticEffect::DoubleControllerDeathTriggers,
            },
            token_anthem(Keyword::Vigilance, "Creature tokens you control have vigilance."),
            token_anthem(Keyword::Lifelink, "Creature tokens you control have lifelink."),
        ],
        ..Default::default()
    }
}

/// Mass Manipulation — {X}{X}{U}{U}{U}{U} Sorcery. Gain control of X target
/// creatures and/or planeswalkers.
pub fn mass_manipulation() -> CardDefinition {
    CardDefinition {
        name: "Mass Manipulation",
        cost: cost(&[x(), x(), u(), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 6,
                min_targets: 0,
                filter: R::Creature.or(R::Planeswalker),
                effect: Box::new(Effect::GainControl {
                    what: Selector::Target(0),
                    to: None,
                    duration: Duration::Permanent,
                }),
            }),
        },
        ..Default::default()
    }
}

/// Lavinia, Azorius Renegade — {W}{U} 2/2 Human Soldier. Opponents can't cast
/// noncreature spells with mana value greater than their land count, and any
/// spell an opponent casts for no mana is countered.
pub fn lavinia_azorius_renegade() -> CardDefinition {
    CardDefinition {
        name: "Lavinia, Azorius Renegade",
        cost: cost(&[w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Each opponent can't cast noncreature spells with mana value greater than the number of lands that player controls.",
            effect: StaticEffect::OpponentsCantCastNoncreatureAboveLandCount,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::ValueAtMost(Value::CastSpellManaSpent, Value::Const(0)),
            ),
            effect: Effect::CounterSpell { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Mirror March — {5}{R} Enchantment. Whenever a nontoken creature you control
/// enters, flip until you lose a flip; make that many hasty token copies,
/// exiled at the beginning of the next end step.
pub fn mirror_march() -> CardDefinition {
    CardDefinition {
        name: "Mirror March",
        cost: cost(&[generic(5), crate::mana::r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken),
                }),
            effect: Effect::FlipUntilLossThenTokenCopies { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Amplifire — {2}{R}{R} 1/1 Elemental. Upkeep: reveal until a creature card;
/// until your next turn its base P/T become twice that card's, and the reveal
/// is bottomed in a random order.
pub fn amplifire() -> CardDefinition {
    CardDefinition {
        name: "Amplifire",
        cost: cost(&[generic(2), crate::mana::r(), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::RevealUntilCreatureDoubleBasePt,
        }],
        ..Default::default()
    }
}

/// Lazav, Dimir Mastermind — {U}{U}{B}{B} 3/3 Shapeshifter with hexproof.
/// Whenever a creature card hits an opponent's graveyard you may have him
/// become a copy of it, keeping his own name, legendary status and hexproof.
pub fn lazav_dimir_mastermind() -> CardDefinition {
    CardDefinition {
        name: "Lazav, Dimir Mastermind",
        cost: cost(&[u(), u(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Shapeshifter]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::MayDo {
                description: "Have Lazav become a copy of that creature card?".into(),
                body: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    duration: Duration::Permanent,
                    non_legendary: false,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Illusionist's Bracers — {2} Equipment. Each nonmana activated ability of
/// equipped creature is copied (new targets allowed). Equip {3}.
pub fn illusionists_bracers() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Illusionist's Bracers",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::AbilityActivated, EventScope::SelfSource),
                effect: Effect::CopyActivatedAbilityMayChooseTargets,
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Captive Audience — {5}{B}{R} Enchantment. Enters under an opponent's
/// control; at their upkeep it picks a punishment it hasn't picked before.
pub fn captive_audience() -> CardDefinition {
    CardDefinition {
        name: "Captive Audience",
        cost: cost(&[generic(5), b(), r()]),
        card_types: vec![CardType::Enchantment],
        enters_under_opponent_control: true,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::SetLifeTotal { who: Selector::You, amount: Value::Const(4) },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                    Effect::CreateToken {
                        who: PlayerRef::EachOpponent,
                        count: Value::Const(5),
                        definition: TokenDefinition {
                            name: "Zombie".into(),
                            power: 2,
                            toughness: 2,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Zombie],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                ],
            },
        }],
        ..Default::default()
    }
}
