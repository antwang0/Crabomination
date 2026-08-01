//! Modern-deck staples batch 111 — Merfolk / Elves tribal pieces, artifact
//! payoffs, and green engines. Tests in `tests/recent111.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u};

fn lord(t: CreatureType) -> StaticAbility {
    StaticAbility {
        description: "Other creatures of this type you control get +1/+1.",
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(t)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            power: 1,
            toughness: 1,
        },
    }
}

/// Silvergill Douser — {1}{U} 1/1. {T}: target creature gets -X/-0, X =
/// Merfolk and Faeries you control.
pub fn silvergill_douser() -> CardDefinition {
    CardDefinition {
        name: "Silvergill Douser",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature,
                },
                power: Value::Times(
                    Box::new(Value::count(Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Merfolk)
                            .or(SelectionRequirement::HasCreatureType(CreatureType::Faerie))
                            .and(SelectionRequirement::ControlledByYou),
                    ))),
                    Box::new(Value::Const(-1)),
                ),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Merfolk Sovereign — {1}{U}{U} 2/2 Merfolk lord; {T}: target Merfolk
/// can't be blocked this turn.
pub fn merfolk_sovereign() -> CardDefinition {
    CardDefinition {
        name: "Merfolk Sovereign",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![lord(CreatureType::Merfolk)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Merfolk),
                },
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tidebinder Mage — {U}{U} 2/2; ETB taps target red/green creature an
/// opponent controls (the printed while-you-control lock is a next-untap skip).
pub fn tidebinder_mage() -> CardDefinition {
    CardDefinition {
        name: "Tidebinder Mage",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(
                                SelectionRequirement::HasColor(Color::Red)
                                    .or(SelectionRequirement::HasColor(Color::Green)),
                            ),
                    },
                },
                Effect::SkipNextUntap {
                    what: Selector::Target(0),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Master of Waves — {3}{U} 2/1, protection from red; Elemental lord; ETB
/// mints 1/0 blue Elementals equal to your devotion to blue.
pub fn master_of_waves() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Master of Waves",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Protection(Color::Red)],
        static_abilities: vec![StaticAbility {
            description: "Elemental creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Elemental)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::DevotionTo(vec![Color::Blue]),
                definition: TokenDefinition {
                    name: "Elemental".into(),
                    power: 1,
                    toughness: 0,
                    colors: vec![Color::Blue],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elemental],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Loaming Shaman — {2}{G} 3/2; ETB: target player shuffles their graveyard
/// into their library (printed "any number of target cards" collapses to all).
pub fn loaming_shaman() -> CardDefinition {
    CardDefinition {
        name: "Loaming Shaman",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ShuffleGraveyardIntoLibrary {
                who: PlayerRef::Target(0),
            },
        }],
        ..Default::default()
    }
}

/// Defense of the Heart — {3}{G} Enchantment. Your upkeep, if an opponent
/// controls 3+ creatures: sacrifice it, tutor two creatures to play.
pub fn defense_of_the_heart() -> CardDefinition {
    CardDefinition {
        name: "Defense of the Heart",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                n: Value::Const(3),
            }),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Leaf-Crowned Visionary — {G}{G} 1/1 Elf lord; casting an Elf spell lets
/// you pay {G} to draw.
pub fn leaf_crowned_visionary() -> CardDefinition {
    CardDefinition {
        name: "Leaf-Crowned Visionary",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![lord(CreatureType::Elf)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Elf),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {G} to draw a card?".into(),
                mana_cost: cost(&[g()]),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Copperhorn Scout — {G} 1/1; attacking untaps each other creature you
/// control.
pub fn copperhorn_scout() -> CardDefinition {
    CardDefinition {
        name: "Copperhorn Scout",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Genesis Wave — {X}{G}{G}{G} Sorcery. Reveal top X; permanents MV ≤ X hit
/// the battlefield, the rest the graveyard.
pub fn genesis_wave() -> CardDefinition {
    use crate::mana::ManaSymbol;
    CardDefinition {
        name: "Genesis Wave",
        cost: crate::mana::ManaCost::new(vec![
            ManaSymbol::X,
            ManaSymbol::Colored(Color::Green),
            ManaSymbol::Colored(Color::Green),
            ManaSymbol::Colored(Color::Green),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GenesisWave,
        ..Default::default()
    }
}

/// Harald, King of Skemfar — {1}{B}{G} 3/2 menace; ETB digs 5 for an Elf or
/// Warrior card (the Tyvar name-pick is folded in), rest bottom.
pub fn harald_king_of_skemfar() -> CardDefinition {
    CardDefinition {
        name: "Harald, King of Skemfar",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(5),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::HasCreatureType(CreatureType::Elf)
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Warrior)),
                ),
                take: Some(Value::Const(1)),
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
        }],
        ..Default::default()
    }
}

/// Skemfar Shadowsage — {3}{B} 2/5; ETB: each opponent loses X or you gain
/// X, X = your largest shared creature-type count.
pub fn skemfar_shadowsage() -> CardDefinition {
    CardDefinition {
        name: "Skemfar Shadowsage",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::LargestCreatureTypeCount,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::LargestCreatureTypeCount,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Canopy Tactician — {3}{G} 3/3 Elf lord; {T}: Add {G}{G}{G}.
pub fn canopy_tactician() -> CardDefinition {
    CardDefinition {
        name: "Canopy Tactician",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![lord(CreatureType::Elf)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::Const(3)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rhonas the Indomitable — {2}{G} 5/5 deathtouch, indestructible; can't
/// attack or block without another power-4+ creature; {2}{G}: pump.
pub fn rhonas_the_indomitable() -> CardDefinition {
    CardDefinition {
        name: "Rhonas the Indomitable",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::God],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Deathtouch,
            Keyword::Indestructible,
            Keyword::CantAttackOrBlockUnlessYouControlCount {
                filter: Box::new(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                ),
                min: 1,
                attack_only: false,
                block_only: false,
                exclude_self: true,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::OtherThanSource),
                    },
                    power: Value::Const(2),
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
        }],
        ..Default::default()
    }
}

/// Oath of Nissa — {G} Legendary Enchantment; ETB digs 3 for a creature,
/// land, or planeswalker. The planeswalker any-color rider is unmodeled.
pub fn oath_of_nissa() -> CardDefinition {
    CardDefinition {
        name: "Oath of Nissa",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Land)
                        .or(SelectionRequirement::Planeswalker),
                ),
                take: Some(Value::Const(1)),
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
        }],
        ..Default::default()
    }
}

/// Trash for Treasure — {2}{R} Sorcery; sacrifice an artifact, reanimate an
/// artifact from your graveyard.
pub fn trash_for_treasure() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Trash for Treasure",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact,
            count: 1,
        }],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Artifact.and(SelectionRequirement::InYourGraveyard),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Metalwork Colossus — {11} 10/10; costs {X} less, X = total MV of your
/// noncreature artifacts; sac two artifacts to return it from the graveyard.
pub fn metalwork_colossus() -> CardDefinition {
    CardDefinition {
        name: "Metalwork Colossus",
        cost: cost(&[generic(11)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        static_abilities: vec![StaticAbility {
            description: "Costs {X} less, X = total mana value of noncreature artifacts you control.",
            effect: StaticEffect::SelfCostReducedByNoncreatureArtifactMv,
        }],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sac_other_filter: Some((SelectionRequirement::Artifact, 2)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jhoira's Familiar — {4} 2/2 flying; historic spells you cast cost {1}
/// less.
pub fn jhoiras_familiar() -> CardDefinition {
    CardDefinition {
        name: "Jhoira's Familiar",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Historic spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::Artifact
                    .or(SelectionRequirement::HasSupertype(
                        crate::card::Supertype::Legendary,
                    ))
                    .or(SelectionRequirement::HasEnchantmentSubtype(
                        crate::card::EnchantmentSubtype::Saga,
                    )),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Grand Architect — {1}{U}{U} 1/3 blue-creature lord; {U}: artifact
/// creature becomes blue EOT; tap an untapped blue creature: {C}{C} for
/// artifacts only.
pub fn grand_architect() -> CardDefinition {
    CardDefinition {
        name: "Grand Architect",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other blue creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(Color::Blue))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::BecomeColor {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Artifact.and(SelectionRequirement::Creature),
                    },
                    colors: vec![Color::Blue],
                    duration: Duration::EndOfTurn,
                    additive: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_other_filter: Some(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(Color::Blue))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::Const(2))),
                        SpendRestriction::ArtifactOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
