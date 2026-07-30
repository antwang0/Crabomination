//! Kamigawa: Neon Dynasty batch 6 — the "modified matters" green/white shell
//! plus a spread of legends and utility. Rides existing primitives except
//! Golden-Tail Trainer's `CostReductionBySourcePower` (spell discount = this
//! creature's power) and Traproot Kami's `DynamicPt::ForestsInPlay`. Tests in
//! `tests/recent100.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mint_treasures, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, w};

/// A 1/1 colorless Spirit creature token.
fn colorless_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A 2/2 red Spirit creature token with menace.
fn red_menace_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

/// Golden-Tail Trainer — {1}{G}{W} 1/3 Fox Samurai. Aura and Equipment spells you
/// cast cost {X} less, where X is this creature's power. Whenever it attacks,
/// other modified creatures you control get +X/+X until end of turn.
pub fn golden_tail_trainer() -> CardDefinition {
    CardDefinition {
        name: "Golden-Tail Trainer",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox, CreatureType::Samurai],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Aura and Equipment spells you cast cost {X} less, X = this creature's power.",
            effect: StaticEffect::CostReductionBySourcePower {
                filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::IsModified)
                        .and(R::OtherThanSource),
                ),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Kami of Terrible Secrets — {3}{B} 3/4 Spirit. ETB: if you control an artifact
/// and an enchantment, draw a card and gain 1 life.
pub fn kami_of_terrible_secrets() -> CardDefinition {
    CardDefinition {
        name: "Kami of Terrible Secrets",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::All(vec![
                Predicate::SelectorExists(Selector::EachPermanent(
                    R::Artifact.and(R::ControlledByYou),
                )),
                Predicate::SelectorExists(Selector::EachPermanent(
                    R::Enchantment.and(R::ControlledByYou),
                )),
            ]),
            then: Box::new(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Sky-Blessed Samurai — {6}{W} 4/4 enchantment creature Human Samurai. Affinity
/// for enchantments; flying.
pub fn sky_blessed_samurai() -> CardDefinition {
    CardDefinition {
        name: "Sky-Blessed Samurai",
        cost: cost(&[generic(6), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Affinity for enchantments.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: R::Enchantment.and(R::ControlledByYou),
                per: 1,
            },
        }],
        ..Default::default()
    }
}

/// Bamboo Grove Archer — {1}{G} 3/3 enchantment creature Snake Archer. Defender,
/// reach. Channel — {4}{G}, Discard this card: destroy target creature with flying.
pub fn bamboo_grove_archer() -> CardDefinition {
    CardDefinition {
        name: "Bamboo Grove Archer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Archer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Defender, Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Walking Skyscraper — {8} 8/8 Construct. Costs {1} less per modified creature
/// you control. Trample; has hexproof as long as it's untapped.
pub fn walking_skyscraper() -> CardDefinition {
    CardDefinition {
        name: "Walking Skyscraper",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            StaticAbility {
                description: "This spell costs {1} less to cast for each modified creature you control.",
                effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                    filter: R::Creature.and(R::ControlledByYou).and(R::IsModified),
                    per: 1,
                },
            },
            StaticAbility {
                description: "Has hexproof as long as it's untapped.",
                effect: StaticEffect::SelfHasKeywordWhile {
                    keyword: Keyword::Hexproof,
                    condition: R::Untapped,
                },
            },
        ],
        ..Default::default()
    }
}

/// Master's Rebuke — {1}{G} Instant. Target creature you control deals damage
/// equal to its power to target creature or planeswalker you don't control.
pub fn masters_rebuke() -> CardDefinition {
    CardDefinition {
        name: "Master's Rebuke",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamageEqualToPower {
            source: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
            target: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
            },
        },
        ..Default::default()
    }
}

/// Tempered in Solitude — {1}{R} Enchantment. Whenever a creature you control
/// attacks alone, exile the top card of your library; you may play it this turn.
pub fn tempered_in_solitude() -> CardDefinition {
    CardDefinition {
        name: "Tempered in Solitude",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                pay_own_cost: false,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Akki Ember-Keeper — {1}{R} 2/1 enchantment creature Goblin Warrior. Whenever a
/// nontoken modified creature you control dies, create a 1/1 colorless Spirit token.
pub fn akki_ember_keeper() -> CardDefinition {
    CardDefinition {
        name: "Akki Ember-Keeper",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsModified.and(R::NotToken),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: colorless_spirit_token(),
            },
        }],
        ..Default::default()
    }
}

/// Thundering Raiju — {2}{R}{R} 3/3 Spirit, haste. Whenever it attacks, put a
/// +1/+1 counter on target creature you control, then deal X damage to each
/// opponent, where X is the number of other modified creatures you control.
pub fn thundering_raiju() -> CardDefinition {
    CardDefinition {
        name: "Thundering Raiju",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::Creature
                                .and(R::ControlledByYou)
                                .and(R::IsModified)
                                .and(R::OtherThanSource),
                        )),
                        filter: R::Any,
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Scrapyard Steelbreaker — {3}{R} 3/4 Human Warrior. {1}, Sacrifice another
/// artifact: this creature gets +2/+1 until end of turn.
pub fn scrapyard_steelbreaker() -> CardDefinition {
    CardDefinition {
        name: "Scrapyard Steelbreaker",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact.and(R::OtherThanSource), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Atsushi, the Blazing Sky — {2}{R}{R} 4/4 Legendary Dragon Spirit, flying,
/// trample. When it dies, choose one — exile the top two cards of your library
/// (play them until your next turn) or create three Treasure tokens.
pub fn atsushi_the_blazing_sky() -> CardDefinition {
    CardDefinition {
        name: "Atsushi, the Blazing Sky",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![on_dies(Effect::ChooseMode(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                pay_own_cost: false,
                uncast_penalty: None,
            },
            mint_treasures(3),
        ]))],
        ..Default::default()
    }
}

/// Junji, the Midnight Sky — {3}{B}{B} 5/5 Legendary Dragon Spirit, flying,
/// menace. When it dies, choose one — each opponent discards two cards and loses
/// 2 life; or reanimate a non-Dragon creature card from a graveyard (lose 2 life).
pub fn junji_the_midnight_sky() -> CardDefinition {
    CardDefinition {
        name: "Junji, the Midnight Sky",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Spirit],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Menace],
        triggered_abilities: vec![on_dies(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                    random: false,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            ]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::InGraveyard)
                            .and(R::HasCreatureType(CreatureType::Dragon).negate()),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        ]))],
        ..Default::default()
    }
}

/// Chishiro, the Shattered Blade — {2}{R}{G} 4/4 Legendary Snake Samurai.
/// Whenever an Aura or Equipment you control enters, create a 2/2 red Spirit with
/// menace. At the beginning of your end step, put a +1/+1 counter on each
/// modified creature you control.
pub fn chishiro_the_shattered_blade() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Chishiro, the Shattered Blade",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Samurai],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                            .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: red_menace_spirit_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::IsModified),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Risona, Asari Commander — {1}{R}{W} 3/3 Legendary Human Samurai, haste.
/// Whenever it deals combat damage to a player, if it has no indestructible
/// counter, put one on it. Whenever combat damage is dealt to you, remove one.
pub fn risona_asari_commander() -> CardDefinition {
    CardDefinition {
        name: "Risona, Asari Commander",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::WithCounter(CounterType::Indestructible),
                    }))),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Indestructible,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::ControllerDealtCombatDamage,
                    EventScope::SelfSource,
                ),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Indestructible,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Traproot Kami — {G} 0/* Spirit. Defender, reach. Its toughness equals the
/// number of Forests on the battlefield.
pub fn traproot_kami() -> CardDefinition {
    CardDefinition {
        name: "Traproot Kami",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Defender, Keyword::Reach],
        dynamic_pt: Some(DynamicPt::ForestsInPlay { base_p: 0 }),
        ..Default::default()
    }
}

/// Unstoppable Ogre — {2}{R} 4/1 Artifact Creature Ogre Warrior. ETB: target
/// creature can't block this turn.
pub fn unstoppable_ogre() -> CardDefinition {
    CardDefinition {
        name: "Unstoppable Ogre",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// You Are Already Dead — {B} Instant. Destroy target creature that was dealt
/// damage this turn. Draw a card.
pub fn you_are_already_dead() -> CardDefinition {
    CardDefinition {
        name: "You Are Already Dead",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}
