//! Final Fantasy (FIN) — a first wave of cards from the Universes Beyond set.
//! Each card has a functionality test in `crabomination/src/tests/fin.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, etb_mint_token, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticEffect, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Iron Giant — {7} 6/6 artifact creature with vigilance, reach, and trample.
pub fn iron_giant() -> CardDefinition {
    CardDefinition {
        name: "Iron Giant",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Reach, Keyword::Trample],
        ..Default::default()
    }
}

/// Sazh's Chocobo — {G} 0/1 Bird. Landfall: put a +1/+1 counter on it.
pub fn sazhs_chocobo() -> CardDefinition {
    CardDefinition {
        name: "Sazh's Chocobo",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Sephiroth's Intervention — {3}{B} Instant. Destroy target creature; gain 2 life.
pub fn sephiroths_intervention() -> CardDefinition {
    CardDefinition {
        name: "Sephiroth's Intervention",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Cactuar — {G} 3/3 Plant with trample. At the beginning of your end step, if
/// it didn't enter this turn, return it to its owner's hand.
pub fn cactuar() -> CardDefinition {
    CardDefinition {
        name: "Cactuar",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::EnteredThisTurn.negate(),
                }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..Default::default()
    }
}

/// Magitek Armor — {3}{W} 4/4 Vehicle. ETB: make a 1/1 colorless Hero. Crew 1.
pub fn magitek_armor() -> CardDefinition {
    let hero = TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Magitek Armor",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![etb_mint_token(hero, 1)],
        ..Default::default()
    }
}

/// Chocobo Racetrack — {3}{G}{G} Artifact. Landfall: create a 2/2 green Bird
/// token that gets +1/+0 until end of turn whenever a land you control enters.
pub fn chocobo_racetrack() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Racetrack Bird".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Chocobo Racetrack",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: bird,
            },
        }],
        ..Default::default()
    }
}

/// Malboro — {4}{B}{B} 4/4 Plant Horror. Bad Breath ETB: each opponent
/// discards a card, loses 2 life, and exiles the top three of their library.
/// Swampcycling {2}.
pub fn malboro() -> CardDefinition {
    CardDefinition {
        name: "Malboro",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), crate::card::LandType::Swamp)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
                link_to_source: false,
                face_down: false,
            },
        ]))],
        ..Default::default()
    }
}

/// Sephiroth, Planet's Heir — {4}{U}{B} 4/4 Vigilance. ETB: opponents'
/// creatures get -2/-2. Whenever a creature an opponent controls dies, put a
/// +1/+1 counter on Sephiroth.
pub fn sephiroth_planets_heir() -> CardDefinition {
    CardDefinition {
        name: "Sephiroth, Planet's Heir",
        cost: cost(&[generic(4), crate::mana::u(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Avatar,
                CreatureType::Soldier,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Aerith Gainsborough — {2}{W} 2/2 Lifelink. Whenever you gain life, grow.
/// On death, distribute its +1/+1 counters across each legendary creature you
/// control.
pub fn aerith_gainsborough() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Aerith Gainsborough",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasSupertype(
                                crate::card::Supertype::Legendary,
                            ))
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Phoenix Down — {W} Artifact. {1}{W}, {T}, exile this: reanimate a creature
/// card with mana value 4 or less from your graveyard tapped, or exile a target
/// Skeleton, Spirit, or Zombie.
pub fn phoenix_down() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Phoenix Down",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), w()]),
            exile_self_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::InYourGraveyard)
                            .and(SelectionRequirement::ManaValueAtMost(4)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::HasCreatureType(CreatureType::Skeleton)
                            .or(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                            .or(SelectionRequirement::HasCreatureType(CreatureType::Zombie)),
                    ),
                    to: ZoneDest::Exile,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tifa Lockhart — {1}{G} 1/2 Human Monk, trample. Landfall — whenever a land
/// you control enters, double Tifa's power until end of turn.
pub fn tifa_lockhart() -> CardDefinition {
    CardDefinition {
        name: "Tifa Lockhart",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            // Doubling as an EOT self-pump equal to current power (expires,
            // unlike Effect::DoublePower which is permanent).
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Feather of Flight — {1}{W} Aura with flash. Enchant creature. ETB: draw a
/// card. Enchanted creature gets +1/+0 and has flying.
pub fn feather_of_flight() -> CardDefinition {
    CardDefinition {
        name: "Feather of Flight",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Creature },
        },
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Vivi Ornitier — {1}{U}{R} 0/3 Wizard. {0}: add power-worth of {U}/{R} (once
/// each turn, your turn only). Whenever you cast a noncreature spell, put a
/// +1/+1 counter on Vivi and it deals 1 damage to each opponent.
pub fn vivi_ornitier() -> CardDefinition {
    CardDefinition {
        name: "Vivi Ornitier",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 0,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[]),
            once_per_turn: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(
                    vec![Color::Blue, Color::Red],
                    Value::PowerOf(Box::new(Selector::This)),
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Barret Wallace — {3}{R} 4/4 Human Rebel, reach. When it attacks, it deals
/// damage equal to the number of equipped creatures you control to the
/// defending player.
pub fn barret_wallace() -> CardDefinition {
    CardDefinition {
        name: "Barret Wallace",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::IsEquipped),
                    }),
                    filter: SelectionRequirement::Any,
                },
            },
        }],
        ..Default::default()
    }
}

/// Squall, SeeD Mercenary — {2}{W}{B} 3/4 Human Knight Mercenary. When a
/// creature you control attacks alone, it gains double strike. Combat damage to
/// a player → return a permanent card (mana value ≤ 3) from your graveyard to
/// the battlefield.
pub fn squall_seed_mercenary() -> CardDefinition {
    CardDefinition {
        name: "Squall, SeeD Mercenary",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::AttackingAlone),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::InYourGraveyard)
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..Default::default()
    }
}

/// White Mage's Staff — {1}{W} Equipment. Job select: mint a 1/1 Hero and equip
/// it. Equipped creature gets +1/+1, is a Cleric, and gains 1 life on attack.
/// Equip {3}.
pub fn white_mages_staff() -> CardDefinition {
    let hero = TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "White Mage's Staff",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: hero },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            add_creature_types: vec![CreatureType::Cleric],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Tidus, Blitzball Star — {1}{W}{U} 2/1 Human Warrior. Whenever an artifact you
/// control enters, put a +1/+1 counter on Tidus. Whenever Tidus attacks, tap
/// target creature an opponent controls.
pub fn tidus_blitzball_star() -> CardDefinition {
    CardDefinition {
        name: "Tidus, Blitzball Star",
        cost: cost(&[generic(1), w(), crate::mana::u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Artifact,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
            },
        ],
        ..Default::default()
    }
}

/// Zidane, Tantalus Thief — {3}{R}{W} 3/3 Human Mutant Scout. ETB: gain control
/// of target creature an opponent controls until end of turn; untap it and it
/// gains lifelink and haste. (The "opponent gains control from you" Treasure
/// rider is omitted.)
pub fn zidane_tantalus_thief() -> CardDefinition {
    CardDefinition {
        name: "Zidane, Tantalus Thief",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mutant, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Snow Villiers — {2}{W} */3 Human Rebel Monk with vigilance. His power equals
/// the number of creatures you control.
pub fn snow_villiers() -> CardDefinition {
    CardDefinition {
        name: "Snow Villiers",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Monk],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        dynamic_pt: Some(crate::card::DynamicPt::CreaturesYouControl { base_t: 3 }),
        ..Default::default()
    }
}

/// Hope Estheim — {W}{U} 2/2 Human Wizard with lifelink. At the beginning of
/// your end step, each opponent mills cards equal to the life you gained this turn.
pub fn hope_estheim() -> CardDefinition {
    CardDefinition {
        name: "Hope Estheim",
        cost: cost(&[w(), crate::mana::u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::LifeGainedThisTurn(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Sazh Katzroy — {3}{G} 3/3 Human Pilot. ETB: you may search for a Bird or
/// basic land card to hand. Whenever it attacks, put a +1/+1 counter on target
/// creature, then double the number of +1/+1 counters on it.
pub fn sazh_katzroy() -> CardDefinition {
    CardDefinition {
        name: "Sazh Katzroy",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "search for a Bird or basic land".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Bird)
                        .or(SelectionRequirement::IsBasicLand),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::DoubleCountersOnEach {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Vanille, Cheerful L'Cie — {3}{G} 3/2 Human Cleric. ETB: mill two, then return
/// a permanent card from your graveyard to your hand. (Meld half omitted.)
pub fn vanille_cheerful_lcie() -> CardDefinition {
    CardDefinition {
        name: "Vanille, Cheerful L'Cie",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]))],
        ..Default::default()
    }
}

/// Y'shtola, Night's Blessed — {1}{W}{U}{B} 2/4 Cat Warlock, vigilance. Whenever
/// you cast a noncreature spell with mana value 3 or greater, deal 2 damage to
/// each opponent and gain 2 life. (The lost-4-life end-step draw is omitted.)
pub fn yshtola_nights_blessed() -> CardDefinition {
    CardDefinition {
        name: "Y'shtola, Night's Blessed",
        cost: cost(&[generic(1), w(), crate::mana::u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(
                    SelectionRequirement::Noncreature
                        .and(SelectionRequirement::ManaValueAtLeast(3)),
                ),
            ),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        ..Default::default()
    }
}

/// Tonberry — {B} 2/1 Salamander Horror. Enters tapped with a stun counter.
/// During your turn it has first strike and deathtouch.
pub fn tonberry() -> CardDefinition {
    CardDefinition {
        name: "Tonberry",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        enters_with_counters: Some((CounterType::Stun, Value::ONE)),
        triggered_abilities: vec![etb(Effect::Tap { what: Selector::This })],
        static_abilities: vec![StaticAbility {
            description: "Tonberry has first strike and deathtouch during your turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike, Keyword::Deathtouch],
            },
        }],
        ..Default::default()
    }
}

/// Zell Dincht — {2}{R} 0/3 Human Monk. Play an extra land each turn; gets +1/+0
/// per land you control; at your end step, return a land you control to hand.
pub fn zell_dincht() -> CardDefinition {
    CardDefinition {
        name: "Zell Dincht",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        dynamic_pt: Some(crate::card::DynamicPt::LandsControlledPower { base_p: 0, base_t: 3 }),
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..Default::default()
    }
}

/// Angel of Mercy — {4}{W} 3/3 Angel with flying. ETB: gain 3 life.
pub fn angel_of_mercy() -> CardDefinition {
    CardDefinition {
        name: "Angel of Mercy",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        ..Default::default()
    }
}

/// Rydia, Summoner of Mist — {R}{G} 1/2 Human Shaman. Landfall — whenever a land
/// you control enters, you may discard a card; if you do, draw a card. (The
/// "Summon" Saga-reanimation activated ability is omitted.)
pub fn rydia_summoner_of_mist() -> CardDefinition {
    CardDefinition {
        name: "Rydia, Summoner of Mist",
        cost: cost(&[r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::MayDiscard {
                description: "Discard a card to draw a card?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
