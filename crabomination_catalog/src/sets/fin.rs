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

/// Locke Cole — {1}{U}{B} 2/3 Human Rogue with deathtouch and lifelink.
/// Whenever he deals combat damage to a player, draw a card, then discard a card.
pub fn locke_cole() -> CardDefinition {
    CardDefinition {
        name: "Locke Cole",
        cost: cost(&[generic(1), crate::mana::u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..Default::default()
    }
}

/// Ultima Weapon — {7} Legendary Equipment. Equipped creature gets +7/+7; when
/// it attacks, destroy target creature an opponent controls. Equip {7}.
pub fn ultima_weapon() -> CardDefinition {
    CardDefinition {
        name: "Ultima Weapon",
        cost: cost(&[generic(7)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(7)]))],
        equipped_bonus: Some(EquipBonus {
            power: 7,
            toughness: 7,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Cloud, Midgar Mercenary — {W}{W} 2/1 Human Soldier Mercenary. ETB: search
/// your library for an Equipment card and put it into your hand. (The "Cloud's
/// triggered abilities trigger an additional time" rider is omitted.)
pub fn cloud_midgar_mercenary() -> CardDefinition {
    CardDefinition {
        name: "Cloud, Midgar Mercenary",
        cost: cost(&[w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Aerith, Last Ancient — {2}{G}{W} 3/5 Human Cleric Druid with lifelink. At
/// your end step, if you gained life this turn, return a creature card from your
/// graveyard to your hand — or to the battlefield if you gained 7 or more life.
pub fn aerith_last_ancient() -> CardDefinition {
    CardDefinition {
        name: "Aerith, Last Ancient",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                }),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(7),
                },
                then: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Barret, Avalanche Leader — {2}{R}{G} 4/4 Human Rebel with reach. Whenever an
/// Equipment you control enters, create a 2/2 red Rebel token. (The begin-combat
/// auto-attach is omitted.)
pub fn barret_avalanche_leader() -> CardDefinition {
    let rebel = TokenDefinition {
        name: "Rebel".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rebel], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Barret, Avalanche Leader",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rebel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rebel },
        }],
        ..Default::default()
    }
}

/// Edgar, King of Figaro — {4}{U}{U} 4/5 Human Artificer Noble. ETB: draw a card
/// for each artifact you control. (The two-headed-coin rider is omitted.)
pub fn edgar_king_of_figaro() -> CardDefinition {
    CardDefinition {
        name: "Edgar, King of Figaro",
        cost: cost(&[generic(4), crate::mana::u(), crate::mana::u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                }),
                filter: SelectionRequirement::Any,
            },
        })],
        ..Default::default()
    }
}

/// A 2/2 green Bird token that gets +1/+0 until end of turn on landfall (the
/// Chocobo token shared by Gysahl Greens / Chocobo Racetrack).
fn racetrack_bird_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bird".into(),
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
    }
}

/// Eject — {3}{U} Instant that can't be countered. Return target nonland
/// permanent to its owner's hand, then draw a card.
pub fn eject() -> CardDefinition {
    CardDefinition {
        name: "Eject",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Land.negate()),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Deadly Embrace — {3}{B}{B} Sorcery. Destroy target creature an opponent
/// controls, then draw a card for each creature that died this turn.
pub fn deadly_embrace() -> CardDefinition {
    CardDefinition {
        name: "Deadly Embrace",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::Draw { who: Selector::You, amount: Value::CreaturesDiedThisTurnTotal },
        ]),
        ..Default::default()
    }
}

/// Airship Crash — {2}{G} Instant. Destroy target artifact, enchantment, or
/// creature with flying. Cycling {2}.
pub fn airship_crash() -> CardDefinition {
    CardDefinition {
        name: "Airship Crash",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying))),
            ),
        },
        ..Default::default()
    }
}

/// Dreams of Laguna — {1}{U} Instant. Surveil 1, then draw a card.
/// Flashback {3}{U}.
pub fn dreams_of_laguna() -> CardDefinition {
    CardDefinition {
        name: "Dreams of Laguna",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Gysahl Greens — {1}{G} Sorcery. Create a 2/2 green Bird token that grows on
/// landfall. Flashback {6}{G}.
pub fn gysahl_greens() -> CardDefinition {
    CardDefinition {
        name: "Gysahl Greens",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(6), g()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: racetrack_bird_token(),
        },
        ..Default::default()
    }
}

/// Battle Menu — {1}{W} Instant. Choose one — make a 2/2 Knight; give a creature
/// +0/+4; destroy a creature with power 4+; or gain 4 life.
pub fn battle_menu() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Battle Menu",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: knight },
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::ZERO,
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Coral Sword — {R} Equipment with flash. ETB: attach to target creature you
/// control; it gains first strike until end of turn. Equipped creature gets
/// +1/+0. Equip {1}.
pub fn coral_sword() -> CardDefinition {
    CardDefinition {
        name: "Coral Sword",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(1)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]))],
        equipped_bonus: Some(EquipBonus { power: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Adventurer's Airship — {3} 3/2 Vehicle with flying. Whenever it attacks, draw
/// then discard. Crew 2.
pub fn adventurers_airship() -> CardDefinition {
    CardDefinition {
        name: "Adventurer's Airship",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..Default::default()
    }
}

/// Al Bhed Salvagers — {2}{B} 2/3 Human Artificer Warrior. Whenever this or
/// another creature you control dies, target opponent loses 1 life, you gain 1.
/// (Modeled on creature deaths; the printed "or artifact" clause covers
/// artifact creatures.)
pub fn al_bhed_salvagers() -> CardDefinition {
    CardDefinition {
        name: "Al Bhed Salvagers",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: crate::effect::shortcut::drain(1),
        }],
        ..Default::default()
    }
}

/// Demon Wall — {1}{B} 3/3 Artifact Creature with defender and menace. It can
/// attack as though it didn't have defender while it has a +1/+1 counter.
/// {5}{B}: put two +1/+1 counters on it.
pub fn demon_wall() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Demon Wall",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Defender, Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "While it has a counter, it can attack as though it didn't have defender.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ashe, Princess of Dalmasca — {2}{W} 3/2 Legendary Human Rebel Noble. Whenever
/// she attacks, look at the top five cards; you may put an artifact among them
/// into your hand, the rest on the bottom in a random order.
pub fn ashe_princess_of_dalmasca() -> CardDefinition {
    CardDefinition {
        name: "Ashe, Princess of Dalmasca",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::RevealTopTakeMatchingToHand {
                who: PlayerRef::You,
                count: Value::Const(5),
                filter: SelectionRequirement::Artifact,
            },
        }],
        ..Default::default()
    }
}

/// Gladiolus Amicitia — {4}{R}{G} 6/6 Legendary Human Warrior. ETB: search your
/// library for a land, put it onto the battlefield tapped. Landfall: another
/// target creature you control gets +2/+2 and gains trample until end of turn.
pub fn gladiolus_amicitia() -> CardDefinition {
    CardDefinition {
        name: "Gladiolus Amicitia",
        cost: cost(&[generic(4), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![
            etb(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::OtherThanSource),
                        ),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Cloudbound Moogle — {3}{W}{W} 2/3 Moogle with flying. ETB: put a +1/+1
/// counter on target creature. Plainscycling {2}.
pub fn cloudbound_moogle() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Cloudbound Moogle",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moogle], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![
            Keyword::Flying,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Balamb T-Rexaur — {4}{G}{G} 6/6 Dinosaur with trample. ETB: gain 3 life.
/// Forestcycling {2}.
pub fn balamb_t_rexaur() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Balamb T-Rexaur",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![
            Keyword::Trample,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest),
        ],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        ..Default::default()
    }
}

/// Goobbue Gardener — {1}{G} 1/3 Plant Beast. {T}: Add {G}.
pub fn goobbue_gardener() -> CardDefinition {
    CardDefinition {
        name: "Goobbue Gardener",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![super::tap_add(Color::Green)],
        ..Default::default()
    }
}

/// Blazing Bomb — {R} 1/1 Elemental. Whenever you cast an expensive noncreature
/// spell, grow. Blow Up — {T}, Sacrifice this: deal damage equal to its power to
/// target creature. (The "four mana spent" gate is modeled as mana value ≥ 4.)
pub fn blazing_bomb() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Blazing Bomb",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastSpellMatches(SelectionRequirement::Noncreature),
                    Predicate::CastSpellMatches(SelectionRequirement::ManaValueAtLeast(4)),
                ]),
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::DealDamageEqualToPower {
                source: Selector::This,
                target: target_filtered(SelectionRequirement::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Adelbert Steiner — {1}{W} 2/1 Legendary Human Knight with lifelink that gets
/// +1/+1 for each Equipment you control.
pub fn adelbert_steiner() -> CardDefinition {
    CardDefinition {
        name: "Adelbert Steiner",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+1 for each Equipment you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                    .and(SelectionRequirement::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Ambrosia Whiteheart — {1}{W} 2/2 Legendary Bird with flash. ETB: you may
/// return another permanent you control to hand. Landfall: +1/+0 until EOT.
pub fn ambrosia_whiteheart() -> CardDefinition {
    CardDefinition {
        name: "Ambrosia Whiteheart",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "return another permanent you control to hand".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Coliseum Behemoth — {5}{G}{G} 7/7 Beast with trample. ETB: destroy target
/// artifact or enchantment, or draw a card.
pub fn coliseum_behemoth() -> CardDefinition {
    CardDefinition {
        name: "Coliseum Behemoth",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Hill Gigas — {4}{R}{R} 5/4 Giant with trample and haste. Mountaincycling {2}.
pub fn hill_gigas() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Hill Gigas",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![
            Keyword::Trample,
            Keyword::Haste,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain),
        ],
        ..Default::default()
    }
}

/// Cid, Timeless Artificer — {2}{W}{U} 4/4 Legendary Human Artificer. Artifact
/// creatures and Heroes you control get +1/+1 for each Artificer you control
/// and each Artificer card in your graveyard. Cycling {W}{U}. (The "any number
/// of copies in a deck" clause is a deck-construction rule, not modeled.)
pub fn cid_timeless_artificer() -> CardDefinition {
    CardDefinition {
        name: "Cid, Timeless Artificer",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[w(), u()]))],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures and Heroes you control get +1/+1 for each \
                          Artificer you control and each Artificer card in your graveyard.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: SelectionRequirement::Artifact
                    .and(SelectionRequirement::Creature)
                    .or(SelectionRequirement::HasCreatureType(CreatureType::Hero)),
                count_filter: SelectionRequirement::HasCreatureType(CreatureType::Artificer),
                per_power: 1,
                per_toughness: 1,
                count_graveyard: true,
            },
        }],
        ..Default::default()
    }
}

/// Warrior of Light — {W}{U}{B}{R}{G} 5/5 Legendary Human Wizard. Legendary
/// creatures you control get +X/+X where X is the number of legendary creatures
/// you control. Whenever you cast a legendary spell from your hand, exile from
/// the top of your library until a legendary nonland card of lesser mana value;
/// you may cast it without paying its mana cost, rest to the bottom.
pub fn warrior_of_light() -> CardDefinition {
    use crate::card::MayPlayDuration;
    use crate::effect::RevealMissDest;
    CardDefinition {
        name: "Warrior of Light",
        cost: cost(&[w(), u(), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Legendary creatures you control get +X/+X, where X is the number \
                          of legendary creatures you control.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: SelectionRequirement::HasSupertype(Supertype::Legendary)
                    .and(SelectionRequirement::Creature),
                count_filter: SelectionRequirement::HasSupertype(Supertype::Legendary)
                    .and(SelectionRequirement::Creature),
                per_power: 1,
                per_toughness: 1,
                count_graveyard: false,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastFromHand,
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasSupertype(Supertype::Legendary),
                    },
                ]),
            ),
            effect: Effect::Seq(vec![
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    // "lesser mana value" — `trigger_event_amount_scratch` is the
                    // cast legendary spell's mana value on a SpellCast trigger.
                    find: SelectionRequirement::HasSupertype(Supertype::Legendary)
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ManaValueLessThanEventAmount),
                    to: ZoneDest::Exile,
                    cap: Value::Const(60),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                    any_color: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Cloud, Ex-SOLDIER — {2}{R}{G}{W} 4/4 Legendary Human Soldier Mercenary with
/// haste. When it enters, attach a target Equipment you control to it. Whenever
/// it attacks, draw a card for each equipped attacking creature you control,
/// then create two Treasures if it has power 7 or greater.
pub fn cloud_ex_soldier() -> CardDefinition {
    CardDefinition {
        name: "Cloud, Ex-SOLDIER",
        cost: cost(&[generic(2), r(), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Mercenary,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            // "up to one target Equipment" is modeled as a required target
            // (fizzles if you control no Equipment).
            etb(Effect::Attach {
                what: target_filtered(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                to: Selector::This,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::CountMatching {
                            sel: Box::new(Selector::EachMatching {
                                zone: ZoneRef::Battlefield,
                                filter: SelectionRequirement::Creature
                                    .and(SelectionRequirement::ControlledByYou)
                                    .and(SelectionRequirement::IsAttacking)
                                    .and(SelectionRequirement::IsEquipped),
                            }),
                            filter: SelectionRequirement::Any,
                        },
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::This,
                            filter: SelectionRequirement::PowerAtLeast(7),
                        },
                        then: Box::new(crate::effect::shortcut::mint_treasures(2)),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Undercity Dire Rat — {1}{B} 2/2 Rat. When it dies, create a Treasure.
pub fn undercity_dire_rat() -> CardDefinition {
    CardDefinition {
        name: "Undercity Dire Rat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        ..Default::default()
    }
}

/// Magic Pot — {3} 1/4 Artifact Goblin Construct. When it dies, create a
/// Treasure. {2}, {T}: Exile target card from a graveyard.
pub fn magic_pot() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Magic Pot",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Exile {
                what: target_filtered(SelectionRequirement::InGraveyard),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shinra Reinforcements — {2}{B} 2/3 Human Soldier. When it enters, mill three
/// cards and you gain 3 life.
pub fn shinra_reinforcements() -> CardDefinition {
    CardDefinition {
        name: "Shinra Reinforcements",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..Default::default()
    }
}

/// Minwu, White Mage — {3}{W}{W} 3/3 Legendary Human Cleric with vigilance and
/// lifelink. Whenever you gain life, put a +1/+1 counter on each Cleric you
/// control.
pub fn minwu_white_mage() -> CardDefinition {
    CardDefinition {
        name: "Minwu, White Mage",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Cleric)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Il Mheg Pixie — {1}{U} 2/1 Faerie with flying. Whenever it attacks,
/// surveil 1.
pub fn il_mheg_pixie() -> CardDefinition {
    CardDefinition {
        name: "Il Mheg Pixie",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Sabotender — {1}{R} 2/1 Plant with reach. Landfall — Whenever a land you
/// control enters, it deals 1 damage to each opponent.
pub fn sabotender() -> CardDefinition {
    CardDefinition {
        name: "Sabotender",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                },
        }],
        ..Default::default()
    }
}

/// Black Waltz No. 3 — {2}{B}{R} 2/2 Legendary Wizard with flying and
/// deathtouch. Whenever you cast a noncreature spell, it deals 2 damage to
/// each opponent.
pub fn black_waltz_no_3() -> CardDefinition {
    CardDefinition {
        name: "Black Waltz No. 3",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                },
        }],
        ..Default::default()
    }
}

/// Xande, Dark Mage — {2}{U}{B} 3/3 Legendary Human Wizard with menace. Gets
/// +1/+1 for each noncreature, nonland card in your graveyard.
pub fn xande_dark_mage() -> CardDefinition {
    CardDefinition {
        name: "Xande, Dark Mage",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        dynamic_pt: Some(
            crate::card::DynamicPt::BasePlusNoncreatureNonlandInControllerGraveyard {
                base_p: 3,
                base_t: 3,
            },
        ),
        ..Default::default()
    }
}

/// Overkill — {2}{B} Instant. Target creature gets -0/-9999 until end of turn.
pub fn overkill() -> CardDefinition {
    CardDefinition {
        name: "Overkill",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(0),
            toughness: Value::Const(-9999),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Blitzball Shot — {1}{G} Instant. Target creature gets +3/+3 and gains
/// trample until end of turn.
pub fn blitzball_shot() -> CardDefinition {
    CardDefinition {
        name: "Blitzball Shot",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Fight On! — {2}{B} Instant. Return up to two target creature cards from your
/// graveyard to your hand.
pub fn fight_on() -> CardDefinition {
    CardDefinition {
        name: "Fight On!",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ReturnGraveyardCardsToHand {
            filter: SelectionRequirement::Creature,
            max: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Evil Reawakened — {4}{B} Sorcery. Return target creature card from your
/// graveyard to the battlefield with two additional +1/+1 counters on it.
pub fn evil_reawakened() -> CardDefinition {
    CardDefinition {
        name: "Evil Reawakened",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Fate of the Sun-Cryst — {4}{W} Instant. Costs {2} less to cast if it targets
/// a tapped creature. Destroy target nonland permanent.
pub fn fate_of_the_sun_cryst() -> CardDefinition {
    CardDefinition {
        name: "Fate of the Sun-Cryst",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((
            SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            2,
        )),
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            ),
        },
        ..Default::default()
    }
}

/// You're Not Alone — {W} Instant. Target creature gets +2/+2 until end of turn.
/// If you control three or more creatures, it gets +4/+4 instead.
pub fn youre_not_alone() -> CardDefinition {
    CardDefinition {
        name: "You're Not Alone",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(3),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Auron's Inspiration — {2}{W} Instant with Flashback {2}{W}{W}. Attacking
/// creatures get +2/+0 until end of turn.
pub fn aurons_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Auron's Inspiration",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), w(), w()]))],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(SelectionRequirement::IsAttacking),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Magic Damper — {U} Instant. Target creature you control gets +1/+1 and gains
/// hexproof until end of turn. Untap it.
pub fn magic_damper() -> CardDefinition {
    CardDefinition {
        name: "Magic Damper",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Instant Ramen — {2} Artifact — Food with flash. When it enters, draw a card.
/// {2}, {T}, Sacrifice this artifact: You gain 3 life.
pub fn instant_ramen() -> CardDefinition {
    use crate::card::{ActivatedAbility, ArtifactSubtype};
    CardDefinition {
        name: "Instant Ramen",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sahagin — {1}{U} 1/3 Merfolk Warrior. Whenever you cast a noncreature spell,
/// if at least four mana was spent to cast it, put a +1/+1 counter on it.
pub fn sahagin() -> CardDefinition {
    CardDefinition {
        name: "Sahagin",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastSpellMatches(SelectionRequirement::Noncreature),
                    Predicate::CastSpellMatches(SelectionRequirement::ManaValueAtLeast(4)),
                ]),
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Qiqirn Merchant — {2}{U} 1/4 Beast Citizen. {1}, {T}: Draw a card, then
/// discard a card. {7}, {T}, Sacrifice this creature: Draw three cards.
pub fn qiqirn_merchant() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Qiqirn Merchant",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(7)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Matoya, Archon Elder — {2}{U} 1/4 Legendary Human Warlock. Whenever you scry
/// or surveil, draw a card.
pub fn matoya_archon_elder() -> CardDefinition {
    CardDefinition {
        name: "Matoya, Archon Elder",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}
