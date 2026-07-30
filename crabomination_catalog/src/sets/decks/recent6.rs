//! A sixth wave of staples — cube/EDH/modern cards that filled remaining gaps
//! across all five colors (Elspeth Sun's Champion, Tezzeret the Seeker, the
//! reanimator/edict package, value lands, …). Each card has a functionality
//! test in `crabomination/src/tests/recent6.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, magecraft, on_dies, target_filtered};
use crate::effect::{Duration, LibraryPosition, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::effects::treasure_token;
use crate::mana::{Color, ManaCost, b, cost, g, generic, u, w};

// ── White ────────────────────────────────────────────────────────────────

/// Karmic Guide — {3}{W}{W} 2/2 Angel Spirit. Flying, protection from black,
/// Echo {3}{W}{W}. ETB: return target creature card from your graveyard to the
/// battlefield.
pub fn karmic_guide() -> CardDefinition {
    CardDefinition {
        name: "Karmic Guide",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(Color::Black),
            Keyword::Echo(cost(&[generic(3), w(), w()])),
        ],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
        ..Default::default()
    }
}

/// Elspeth, Sun's Champion — {4}{W}{W} Planeswalker, 4 loyalty.
/// +1: three 1/1 Soldiers. −3: destroy all creatures power ≥ 4. −7: emblem
/// "Creatures you control get +2/+2 and have flying."
pub fn elspeth_suns_champion() -> CardDefinition {
    CardDefinition {
        name: "Elspeth, Sun's Champion",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: soldier_token(),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Elspeth, Sun's Champion".into(),
                    triggered: vec![],
                    statics: vec![
                        StaticAbility {
                            description: "Creatures you control get +2/+2.",
                            effect: StaticEffect::PumpPT {
                                applies_to: Selector::EachPermanent(
                                    SelectionRequirement::Creature
                                        .and(SelectionRequirement::ControlledByYou),
                                ),
                                power: 2,
                                toughness: 2,
                            },
                        },
                        StaticAbility {
                            description: "Creatures you control have flying.",
                            effect: StaticEffect::GrantKeyword {
                                applies_to: Selector::EachPermanent(
                                    SelectionRequirement::Creature
                                        .and(SelectionRequirement::ControlledByYou),
                                ),
                                keyword: Keyword::Flying,
                            },
                        },
                    ],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".to_string(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        ..Default::default()
    }
}

/// Faith's Fetters — {3}{W} Aura. Enchant permanent. ETB: gain 4 life.
/// Enchanted permanent can't attack or block, and its abilities can't be
/// activated unless they're mana abilities.
pub fn faiths_fetters() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Faith's Fetters",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Any),
        },
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(4),
        })],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateAbilities,
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Council's Judgment — {1}{W}{W} Sorcery. Will of the council: each player
/// votes for a nonland permanent you don't control; exile each tied for most
/// votes (no targeting — gets around hexproof/shroud). CR 701.31.
pub fn councils_judgment() -> CardDefinition {
    CardDefinition {
        name: "Council's Judgment",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::WillOfTheCouncilExile {
            filter: SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
        },
        ..Default::default()
    }
}

/// Increasing Devotion — {3}{W}{W} Sorcery. Create five 1/1 Humans; ten if
/// cast from a graveyard. Flashback {7}{W}{W}.
pub fn increasing_devotion() -> CardDefinition {
    CardDefinition {
        name: "Increasing Devotion",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(10),
                definition: human_token(),
            }),
            else_: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(5),
                definition: human_token(),
            }),
        },
        keywords: vec![Keyword::Flashback(cost(&[generic(7), w(), w()]))],
        ..Default::default()
    }
}

fn human_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human".to_string(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        ..Default::default()
    }
}

/// Wing Shards — {1}{W}{W} Instant with Storm. Target player sacrifices an
/// attacking creature of their choice.
pub fn wing_shards() -> CardDefinition {
    CardDefinition {
        name: "Wing Shards",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Sacrifice {
            who: Selector::Target(0),
            count: Value::ONE,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
        },
        keywords: vec![Keyword::Storm],
        ..Default::default()
    }
}

// ── Blue ─────────────────────────────────────────────────────────────────

/// Talrand, Sky Summoner — {2}{U}{U} 2/2 Merfolk Wizard. Whenever you cast an
/// instant or sorcery, create a 2/2 blue Drake with flying.
pub fn talrand_sky_summoner() -> CardDefinition {
    CardDefinition {
        name: "Talrand, Sky Summoner",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Drake".to_string(),
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Drake],
                    ..Default::default()
                },
                power: 2,
                toughness: 2,
                colors: vec![Color::Blue],
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Tezzeret the Seeker — {3}{U}{U} Planeswalker, 4 loyalty.
/// +1: untap up to two target artifacts. −X: search your library for an
/// artifact card with mana value X or less and put it onto the battlefield.
/// (The −5 "artifacts become 5/5 creatures" ultimate is dropped — the modeled
/// abilities capture the tutor + untap play pattern.)
pub fn tezzeret_the_seeker() -> CardDefinition {
    CardDefinition {
        name: "Tezzeret the Seeker",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Tezzeret],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            // +1: untap up to one target artifact. (The printed "up to two
            // target artifacts" is approximated as one — loyalty abilities carry
            // a single target slot.)
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Untap {
                    what: target_filtered(SelectionRequirement::Artifact),
                    up_to: None,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                x_cost: true,
                loyalty_cost: 0,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::ManaValueAtMostXFromCost),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Dream Eater — {4}{U}{U} 4/3 Nightmare Sphinx. Flash, flying. ETB: surveil 4,
/// then you may return target nonland permanent an opponent controls to its
/// owner's hand.
pub fn dream_eater() -> CardDefinition {
    CardDefinition {
        name: "Dream Eater",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Sphinx],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(4),
            },
            Effect::MayDo {
                description: "Return a nonland permanent an opponent controls to its owner's hand?"
                    .into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Nonland
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        ]))],
        ..Default::default()
    }
}

/// Malcolm, Keen-Eyed Navigator — {2}{U} 2/2 Siren Pirate. Flying. Whenever one
/// or more Pirates you control deal combat damage to your opponents, create a
/// Treasure for each opponent dealt damage.
pub fn malcolm_keen_eyed_navigator() -> CardDefinition {
    CardDefinition {
        name: "Malcolm, Keen-Eyed Navigator",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Siren, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Pirate),
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Faerie Mastermind — {1}{U} 2/1 Faerie Rogue. Flash, flying. Whenever an
/// opponent draws their second card each turn, draw a card. {3}{U}: Each player
/// draws a card. CR 121 (per-turn draw count).
pub fn faerie_mastermind() -> CardDefinition {
    CardDefinition {
        name: "Faerie Mastermind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn {
                    who: PlayerRef::Triggerer,
                    n: 2,
                })
                .once_per_turn(),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Black ────────────────────────────────────────────────────────────────

/// Profane Tutor — Sorcery with Suspend 2—{1}{B}. Search your library for a
/// card and put it into your hand.
pub fn profane_tutor() -> CardDefinition {
    CardDefinition {
        name: "Profane Tutor",
        cost: ManaCost::default(),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(2, cost(&[generic(1), b()]))],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Shambling Ghast — {B} 1/1 Zombie. When it dies, choose one — target creature
/// an opponent controls gets -1/-1, or create a Treasure.
pub fn shambling_ghast() -> CardDefinition {
    CardDefinition {
        name: "Shambling Ghast",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: treasure_token(),
            },
        ]))],
        ..Default::default()
    }
}

/// Priest of Forgotten Gods — {1}{B} 1/2 Human Cleric. {T}, Sacrifice two other
/// creatures: any number of target players each lose 2 life and sacrifice a
/// creature; you add {B}{B} and draw a card.
pub fn priest_of_forgotten_gods() -> CardDefinition {
    CardDefinition {
        name: "Priest of Forgotten Gods",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 2)),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Target(0),
                    amount: Value::Const(2),
                },
                Effect::Sacrifice {
                    who: Selector::Target(0),
                    count: Value::ONE,
                    filter: SelectionRequirement::Creature,
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Black, Value::Const(2)),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spawn of Mayhem — {2}{B}{B} 4/4 Demon. Spectacle {1}{B}{B}. Flying, trample.
/// At the beginning of your upkeep, this creature deals 1 damage to each player.
/// Whenever you cast a spell that targets only a single creature, put a +1/+1
/// counter on this creature. (The targets-a-single-creature pump rider is
/// dropped — the headline upkeep ping + Spectacle are modeled.)
pub fn spawn_of_mayhem() -> CardDefinition {
    use crate::card::AlternativeCost;
    use crate::game::TurnStep;
    CardDefinition {
        name: "Spawn of Mayhem",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(1), b(), b()]),
            condition: Some(Predicate::PlayerLostLifeThisTurn {
                who: PlayerRef::EachOpponent,
            }),
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::DealDamage {
                amount: Value::ONE,
                to: Selector::Player(PlayerRef::EachPlayer),
            },
        }],
        ..Default::default()
    }
}

/// Magus of the Coffers — {4}{B} 4/4 Human Wizard. {2}, {T}: Add {B} for each
/// Swamp you control.
pub fn magus_of_the_coffers() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Magus of the Coffers",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Black,
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        SelectionRequirement::HasLandType(LandType::Swamp)
                            .and(SelectionRequirement::ControlledByYou),
                    ))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Plague Engineer — {2}{B} 2/2 Phyrexian. Deathtouch. As it enters, choose a
/// creature type. Creatures of the chosen type your opponents control get -1/-1.
pub fn plague_engineer() -> CardDefinition {
    CardDefinition {
        name: "Plague Engineer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::NameCreatureType {
            what: Selector::This,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures of the chosen type your opponents control get -1/-1.",
            effect: StaticEffect::AnthemForChosenType {
                power: -1,
                toughness: -1,
                exclude_source: false,
                opponents: true,
                per_counter: None,
            },
        }],
        ..Default::default()
    }
}

/// Mukotai Soulripper — {1}{B} 4/3 Vehicle. Crew 2. Whenever it attacks, you may
/// sacrifice another artifact or creature; if you do, put a +1/+1 counter on it
/// and it gains menace until end of turn.
pub fn mukotai_soulripper() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Mukotai Soulripper",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another artifact or creature? (+1/+1 counter + menace)"
                    .into(),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Menace,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── Green ────────────────────────────────────────────────────────────────

/// Dryad Arbor — Land Creature — Forest Dryad. 1/1, intrinsic {T}: Add {G}.
pub fn dryad_arbor() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Dryad Arbor",
        card_types: vec![CardType::Land, CardType::Creature],
        subtypes: Subtypes {
            land_types: vec![LandType::Forest],
            creature_types: vec![CreatureType::Dryad],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Marwyn, the Nurturer — {2}{G} 1/1 Elf Druid. Whenever another Elf you control
/// enters, put a +1/+1 counter on Marwyn. {T}: Add {G} equal to Marwyn's power.
pub fn marwyn_the_nurturer() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Marwyn, the Nurturer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Elf)
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hexdrinker — {G} 2/1 Snake. Level up {1}. LEVEL 3-7: 4/4 protection from
/// instants. LEVEL 8+: 6/6 protection from everything. CR 702.16 / 702.87.
pub fn hexdrinker() -> CardDefinition {
    use crate::card::{CounterType, LevelBand};
    CardDefinition {
        name: "Hexdrinker",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        level_bands: vec![
            LevelBand {
                min: 3,
                max: Some(7),
                power: 4,
                toughness: 4,
                keywords: vec![Keyword::ProtectionFromInstants],
            },
            LevelBand {
                min: 8,
                max: None,
                power: 6,
                toughness: 6,
                keywords: vec![Keyword::ProtectionFromEverything],
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Level,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wolfir Avenger — {1}{G}{G} 3/3 Wolf Warrior. Flash. {1}{G}: Regenerate this
/// creature.
pub fn wolfir_avenger() -> CardDefinition {
    CardDefinition {
        name: "Wolfir Avenger",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wolf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mwonvuli Acid-Moss — {2}{G}{G} Sorcery. Destroy target land. Search your
/// library for a Forest card and put it onto the battlefield tapped.
pub fn mwonvuli_acid_moss() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Mwonvuli Acid-Moss",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Land),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasLandType(LandType::Forest),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        ]),
        ..Default::default()
    }
}

// ── Lands ────────────────────────────────────────────────────────────────

/// Fabled Passage — Land. {T}, Sacrifice this land: Search your library for a
/// basic land and put it onto the battlefield tapped. Then if you control four
/// or more lands, untap that land.
pub fn fabled_passage() -> CardDefinition {
    CardDefinition {
        name: "Fabled Passage",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Graveyard,
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                        ),
                        n: Value::Const(4),
                    },
                    then: Box::new(Effect::Untap {
                        what: Selector::LastMoved,
                        up_to: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mystic Sanctuary — Land — Island. Enters tapped unless you control three or
/// more other Islands. When it enters untapped, you may put target instant or
/// sorcery card from your graveyard on top of your library.
pub fn mystic_sanctuary() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Mystic Sanctuary",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Island],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Blue, Value::ONE),
            },
            ..Default::default()
        }],
        // ETB: if 4+ Islands (i.e. 3+ *other* Islands) it enters untapped and
        // may recur an I/S; otherwise it enters tapped.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasLandType(LandType::Island)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(4),
                },
                then: Box::new(Effect::MayDo {
                    description:
                        "Put an instant or sorcery from your graveyard on top of your library?"
                            .into(),
                    body: Box::new(Effect::Move {
                        what: target_filtered(
                            SelectionRequirement::HasCardType(CardType::Instant)
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        ),
                        to: ZoneDest::Library {
                            who: PlayerRef::You,
                            pos: LibraryPosition::Top,
                        },
                    }),
                }),
                else_: Box::new(Effect::Tap {
                    what: Selector::This,
                }),
            },
        }],
        ..Default::default()
    }
}
