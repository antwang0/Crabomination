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
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

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

// ── modern_decks FIN batch 2 ─────────────────────────────────────────────

/// A 0/1 black Wizard token that pings each opponent when you cast a
/// noncreature spell (Queen Brahne, Cornered by Black Mages, Mysidian Elder).
fn black_wizard_ping_token() -> TokenDefinition {
    TokenDefinition {
        name: "Wizard".into(),
        power: 0,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Queen Brahne — {2}{R} 2/1 Legendary Human Noble with prowess. Whenever she
/// attacks, create a 0/1 black Wizard token that pings on your noncreature casts.
pub fn queen_brahne() -> CardDefinition {
    CardDefinition {
        name: "Queen Brahne",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: crate::effect::shortcut::mint_token(black_wizard_ping_token(), 1),
        }],
        ..Default::default()
    }
}

/// Rosa, Resolute White Mage — {3}{W} 2/3 Legendary Human Noble Cleric with
/// reach. At the beginning of combat on your turn, put a +1/+1 counter on
/// target creature you control; it gains lifelink until end of turn.
pub fn rosa_resolute_white_mage() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Rosa, Resolute White Mage",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Slash of Light — {1}{W} Instant. Deals damage equal to the number of
/// creatures you control plus the number of Equipment you control to target
/// creature.
pub fn slash_of_light() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Slash of Light",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Sum(vec![
                Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    )),
                    filter: SelectionRequirement::Any,
                },
                Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                    filter: SelectionRequirement::Any,
                },
            ]),
        },
        ..Default::default()
    }
}

/// Rydia's Return — {3}{G}{G} Sorcery. Choose one — creatures you control get
/// +3/+3 until end of turn; or return up to two target permanent cards from
/// your graveyard to your hand.
pub fn rydias_return() -> CardDefinition {
    CardDefinition {
        name: "Rydia's Return",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::ReturnGraveyardCardsToHand {
                filter: SelectionRequirement::Permanent,
                max: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// The Crystal's Chosen — {5}{W}{W} Sorcery. Create four 1/1 colorless Hero
/// tokens, then put a +1/+1 counter on each creature you control.
pub fn the_crystals_chosen() -> CardDefinition {
    let hero = TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Crystal's Chosen",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            crate::effect::shortcut::mint_token(hero, 4),
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Commune with Beavers — {G} Sorcery. Look at the top three cards; put an
/// artifact, creature, or land card among them into your hand and the rest on
/// the bottom.
pub fn commune_with_beavers() -> CardDefinition {
    CardDefinition {
        name: "Commune with Beavers",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Land),
            ),
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Prishe's Wanderings — {2}{G} Instant. Search your library for a basic land
/// card, put it onto the battlefield tapped, then shuffle. (Town lands are
/// approximated by the basic-land search.)
pub fn prishes_wanderings() -> CardDefinition {
    CardDefinition {
        name: "Prishe's Wanderings",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Laughing Mad — {2}{R} Instant with Flashback {3}{R}. As an additional cost,
/// discard a card. Draw two cards.
pub fn laughing_mad() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Laughing Mad",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        additional_cast_cost: vec![AdditionalCastCost::Discard { count: 1, filter: None }],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// White Auracite — {2}{W}{W} Artifact. When it enters, exile target nonland
/// permanent an opponent controls until it leaves the battlefield.
pub fn white_auracite() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "White Auracite",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::ControlledByOpponent.and(SelectionRequirement::Nonland),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Ride the Shoopuf — {1}{G} Enchantment. Landfall — Whenever a land you
/// control enters, put a +1/+1 counter on target creature you control.
pub fn ride_the_shoopuf() -> CardDefinition {
    CardDefinition {
        name: "Ride the Shoopuf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Cornered by Black Mages — {1}{B}{B} Sorcery. Target opponent sacrifices a
/// creature of their choice; create a 0/1 black Wizard token.
pub fn cornered_by_black_mages() -> CardDefinition {
    CardDefinition {
        name: "Cornered by Black Mages",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Creature,
            },
            crate::effect::shortcut::mint_token(black_wizard_ping_token(), 1),
        ]),
        ..Default::default()
    }
}

/// Sleep Magic — {U} Aura. Enchant creature; tap it on enter. It doesn't untap
/// during its controller's untap step. When it's dealt damage, sacrifice this.
pub fn sleep_magic() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Sleep Magic",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
                effect: Effect::SacrificePermanent { what: Selector::This },
            },
        ],
        ..Default::default()
    }
}

/// Choco-Comet — {X}{R}{R} Sorcery. Deals X damage to any target; create a 2/2
/// green Bird token that gets +1/+0 until end of turn on each of your landfalls.
pub fn choco_comet() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Choco-Comet",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Target(0), amount: Value::XFromCost },
            crate::effect::shortcut::mint_token(bird, 1),
        ]),
        ..Default::default()
    }
}

// ── modern_decks FIN Town lands ──────────────────────────────────────────

/// A Final Fantasy "Land — Town" dual: enters tapped, taps for either of two
/// colors. Powers the Towns-matter theme (Affinity for Towns, etc.).
fn town_dual(name: &'static str, color_a: Color, color_b: Color) -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        activated_abilities: vec![super::tap_add(color_a), super::tap_add(color_b)],
        triggered_abilities: vec![super::etb_tap()],
        ..Default::default()
    }
}

pub fn baron_airship_kingdom() -> CardDefinition { town_dual("Baron, Airship Kingdom", Color::Blue, Color::Red) }
pub fn gohn_town_of_ruin() -> CardDefinition { town_dual("Gohn, Town of Ruin", Color::Black, Color::Green) }
pub fn gongaga_reactor_town() -> CardDefinition { town_dual("Gongaga, Reactor Town", Color::Red, Color::Green) }
pub fn guadosalam_farplane_gateway() -> CardDefinition { town_dual("Guadosalam, Farplane Gateway", Color::Green, Color::Blue) }
pub fn insomnia_crown_city() -> CardDefinition { town_dual("Insomnia, Crown City", Color::White, Color::Black) }
pub fn rabanastre_royal_city() -> CardDefinition { town_dual("Rabanastre, Royal City", Color::Red, Color::White) }
pub fn sharlayan_nation_of_scholars() -> CardDefinition { town_dual("Sharlayan, Nation of Scholars", Color::White, Color::Blue) }
pub fn treno_dark_city() -> CardDefinition { town_dual("Treno, Dark City", Color::Blue, Color::Black) }
pub fn vector_imperial_capital() -> CardDefinition { town_dual("Vector, Imperial Capital", Color::Black, Color::Red) }
pub fn windurst_federation_center() -> CardDefinition { town_dual("Windurst, Federation Center", Color::Green, Color::White) }

/// Adventurer's Inn — untapped "Land — Town". ETB gain 2 life; {T}: Add {C}.
pub fn adventurers_inn() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Adventurer's Inn",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        activated_abilities: vec![super::tap_add_colorless()],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })],
        ..Default::default()
    }
}

/// Travel the Overworld — {5}{U}{U} Sorcery with Affinity for Towns (costs {1}
/// less to cast for each Town you control). Draw four cards.
pub fn travel_the_overworld() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Travel the Overworld",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Sorcery],
        affinity_filter: Some(
            SelectionRequirement::HasLandType(LandType::Town)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(4) },
        ..Default::default()
    }
}

// ── modern_decks batch: FIN wave (creature-or-artifact death event + 19 more) ──

/// Judge Magister Gabranth — {W}{B} 2/2 Legendary Human Advisor Knight with
/// menace. Whenever another creature or artifact you control dies, put a +1/+1
/// counter on it.
pub fn judge_magister_gabranth() -> CardDefinition {
    CardDefinition {
        name: "Judge Magister Gabranth",
        cost: cost(&[w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::AnotherOfYours),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// G'raha Tia — {4}{W} 3/5 Legendary Cat Archer with reach. Whenever one or
/// more other creatures and/or artifacts you control die, draw a card. Once
/// each turn.
pub fn graha_tia() -> CardDefinition {
    CardDefinition {
        name: "G'raha Tia",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Archer],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::AnotherOfYours)
                .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Diamond Weapon — {7}{G}{G} 8/8 Legendary Artifact Creature — Elemental.
/// Costs {1} less per permanent card in your graveyard; reach; prevents all
/// combat damage that would be dealt to it (Immune).
pub fn diamond_weapon() -> CardDefinition {
    CardDefinition {
        name: "Diamond Weapon",
        cost: cost(&[generic(7), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Reach],
        affinity_graveyard_filter: Some(SelectionRequirement::PermanentCard),
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to Diamond Weapon.",
            effect: StaticEffect::PreventAllCombatDamageToThis,
        }],
        ..Default::default()
    }
}

/// Light of Judgment — {4}{R} Instant. Deals 6 damage to target creature, then
/// destroys up to one Equipment attached to that creature.
pub fn light_of_judgment() -> CardDefinition {
    CardDefinition {
        name: "Light of Judgment",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(6),
            },
            Effect::Destroy {
                what: Selector::Take {
                    inner: Box::new(Selector::AttachedToMe(Box::new(Selector::Target(0)))),
                    count: Box::new(Value::ONE),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Judgment Bolt — {3}{R} Instant. Deals 5 damage to target creature and X
/// damage to that creature's controller, where X is the number of Equipment
/// you control.
pub fn judgment_bolt() -> CardDefinition {
    CardDefinition {
        name: "Judgment Bolt",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(5),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                        .and(SelectionRequirement::ControlledByYou),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Mysidian Elder — {2}{R} 1/3 Human Wizard. ETB: create a 0/1 black Wizard
/// token with "Whenever you cast a noncreature spell, this deals 1 damage to
/// each opponent."
pub fn mysidian_elder() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Wizard".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Mysidian Elder",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb_mint_token(wizard, 1)],
        ..Default::default()
    }
}

/// Ultimecia, Temporal Threat — {4}{U}{U} 4/4 Legendary Human Warlock. ETB:
/// tap all creatures your opponents control. Whenever a creature you control
/// deals combat damage to a player, draw a card.
pub fn ultimecia_temporal_threat() -> CardDefinition {
    CardDefinition {
        name: "Ultimecia, Temporal Threat",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::Tap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Rook Turret — {3}{U} 3/3 Artifact Creature — Construct with flying. Whenever
/// another artifact you control enters, draw a card, then discard a card.
pub fn rook_turret() -> CardDefinition {
    CardDefinition {
        name: "Rook Turret",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..Default::default()
    }
}

/// Gran Pulse Ochu — {G} 1/1 Plant Beast with deathtouch. {8}: it gets +1/+1
/// until end of turn for each permanent card in your graveyard.
pub fn gran_pulse_ochu() -> CardDefinition {
    CardDefinition {
        name: "Gran Pulse Ochu",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::PermanentCard,
                },
                toughness: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::PermanentCard,
                },
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Item Shopkeep — {1}{R} 2/2 Human Citizen. Whenever you attack, target
/// attacking equipped creature gains menace until end of turn.
pub fn item_shopkeep() -> CardDefinition {
    CardDefinition {
        name: "Item Shopkeep",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::IsAttacking.and(SelectionRequirement::IsEquipped),
                ),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Relm's Sketching — {2}{U}{U} Sorcery. Create a token that's a copy of
/// target artifact, creature, or land.
pub fn relms_sketching() -> CardDefinition {
    CardDefinition {
        name: "Relm's Sketching",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::ONE,
            source: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Land),
            ),
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            non_legendary: false,
            legendary: false,
        },
        ..Default::default()
    }
}

/// Reach the Horizon — {3}{G} Sorcery. Search your library for up to two basic
/// land and/or Town cards, put them onto the battlefield tapped, then shuffle.
pub fn reach_the_horizon() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Reach the Horizon",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand
                .or(SelectionRequirement::HasLandType(LandType::Town)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Fang, Fearless l'Cie — {2}{B} 2/3 Legendary Human Warrior. Whenever one or
/// more cards leave your graveyard, draw a card and lose 1 life. Once each turn.
pub fn fang_fearless_lcie() -> CardDefinition {
    CardDefinition {
        name: "Fang, Fearless l'Cie",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Prompto Argentum — {1}{R} 2/2 Legendary Human Scout with haste. Whenever you
/// cast a noncreature spell, if at least four mana was spent to cast it, create
/// a Treasure token.
pub fn prompto_argentum() -> CardDefinition {
    CardDefinition {
        name: "Prompto Argentum",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastSpellMatches(SelectionRequirement::Noncreature),
                    Predicate::CastSpellManaSpentAtLeast(4),
                ]),
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Shantotto, Tactician Magician — {1}{U}{R} 0/4 Legendary Dwarf Wizard.
/// Whenever you cast a noncreature spell, Shantotto gets +X/+0 until end of
/// turn, where X is the mana spent to cast it; if X is 4 or more, draw a card.
pub fn shantotto_tactician_magician() -> CardDefinition {
    CardDefinition {
        name: "Shantotto, Tactician Magician",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::CastSpellManaSpent,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::CastSpellManaSpentAtLeast(4),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Seq(vec![])),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Rufus Shinra — {1}{W}{B} 2/4 Legendary Human Noble. Whenever he attacks, if
/// you don't control a creature named Darkstar, create Darkstar, a legendary
/// 2/2 white and black Dog creature token.
pub fn rufus_shinra() -> CardDefinition {
    let darkstar = TokenDefinition {
        name: "Darkstar".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Rufus Shinra",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::Not(Box::new(Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::HasName("Darkstar".into())
                                .and(SelectionRequirement::ControlledByYou),
                        )),
                        filter: SelectionRequirement::HasName("Darkstar".into())
                            .and(SelectionRequirement::ControlledByYou),
                    },
                    Value::ONE,
                ))),
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: darkstar,
            },
        }],
        ..Default::default()
    }
}

/// Shambling Cie'th — {2}{B} 3/2 Mutant Horror that enters tapped. Whenever you
/// cast a noncreature spell, you may pay {B}; if you do, return this card from
/// your graveyard to your hand.
pub fn shambling_cieth() -> CardDefinition {
    CardDefinition {
        name: "Shambling Cie'th",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Shambling Cie'th enters the battlefield tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::MayPay {
                description: "Pay {B} to return Shambling Cie'th to your hand".into(),
                mana_cost: cost(&[b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Lion Heart — {4} Artifact — Equipment. ETB: it deals 2 damage to any target.
/// Equipped creature gets +2/+1. Equip {2}.
pub fn lion_heart() -> CardDefinition {
    CardDefinition {
        name: "Lion Heart",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Ring of the Lucii — {4} Legendary Artifact. {T}: Add {C}{C}. {2}, {T}, Pay 1
/// life: Tap target nonland permanent.
pub fn ring_of_the_lucii() -> CardDefinition {
    CardDefinition {
        name: "Ring of the Lucii",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                life_cost: 1,
                effect: Effect::Tap {
                    what: target_filtered(SelectionRequirement::Permanent.and(
                        SelectionRequirement::Land.negate(),
                    )),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sandworm — {4}{R} 5/4 Worm with haste. ETB: destroy target land; its
/// controller may search their library for a basic land, put it onto the
/// battlefield tapped, then shuffle.
pub fn sandworm() -> CardDefinition {
    CardDefinition {
        name: "Sandworm",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Worm], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // Search first so `Target(0)` (the land) is still live for the
            // controller lookup; the net board state matches the printed order.
            Effect::SearchUpToN {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    tapped: true,
                },
                count: Value::ONE,
            },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Land) },
        ]))],
        ..Default::default()
    }
}

// ── modern_decks batch 2: FIN counterspell, Towns, artifacts, mass token ──────

/// Syncopate — {X}{U} Instant. Counter target spell unless its controller pays
/// {X}; if countered this way, exile it instead.
pub fn syncopate() -> CardDefinition {
    CardDefinition {
        name: "Syncopate",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[]),
            exile: true,
            extra_generic: Some(Value::XFromCost),
        },
        ..Default::default()
    }
}

/// Crossroads Village — "Land — Town" that enters tapped; as it enters, choose a
/// color; {T}: Add one mana of the chosen color.
pub fn crossroads_village() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Crossroads Village",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Crossroads Village enters the battlefield tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Capital City — "Land — Town". {T}: Add {C}. {1}, {T}: Add one mana of any
/// color. Cycling {2}.
pub fn capital_city() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Capital City",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lunatic Pandora — {1} Legendary Artifact. {2}, {T}: Surveil 1. {6}, {T},
/// Sacrifice it: Destroy target nonland permanent.
pub fn lunatic_pandora() -> CardDefinition {
    CardDefinition {
        name: "Lunatic Pandora",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(6)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Permanent.and(
                        SelectionRequirement::Land.negate(),
                    )),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// PuPu UFO — {2} 0/4 Artifact Creature — Construct Alien with flying. {T}: put
/// a land from your hand onto the battlefield. {3}: base power becomes the
/// number of Towns you control until end of turn.
pub fn pupu_ufo() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "PuPu UFO",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct, CreatureType::Alien],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::SetBasePower {
                    what: Selector::This,
                    power: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::HasLandType(LandType::Town)
                                .and(SelectionRequirement::ControlledByYou),
                        )),
                        filter: SelectionRequirement::HasLandType(LandType::Town)
                            .and(SelectionRequirement::ControlledByYou),
                    },
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Magitek Infantry — {W} 1/1 Artifact Creature — Robot Soldier. Gets +1/+0
/// while you control another artifact. {2}{W}: search your library for a card
/// named Magitek Infantry, put it onto the battlefield tapped, then shuffle.
pub fn magitek_infantry() -> CardDefinition {
    CardDefinition {
        name: "Magitek Infantry",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Magitek Infantry gets +1/+0 while you control another artifact.",
            effect: StaticEffect::PumpSelfIf {
                // At least two artifacts you control (itself + one other).
                condition: Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                        )),
                        filter: SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    },
                    Value::Const(2),
                ),
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasName("Magitek Infantry".into()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Moogles' Valor — {3}{W}{W} Instant. For each creature you control, create a
/// 1/2 white Moogle with lifelink; then creatures you control gain
/// indestructible until end of turn.
pub fn moogles_valor() -> CardDefinition {
    let moogle = TokenDefinition {
        name: "Moogle".into(),
        power: 1,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moogle], ..Default::default() },
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    };
    CardDefinition {
        name: "Moogles' Valor",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    )),
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                definition: moogle,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── modern_decks batch 3: FIN spells + artifacts ──────────────────────────────

/// World Map — {1} Artifact. {1}, {T}, Sacrifice: search for a basic land to
/// hand. {3}, {T}, Sacrifice: search for any land to hand.
pub fn world_map() -> CardDefinition {
    CardDefinition {
        name: "World Map",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Retrieve the Esper — {3}{U} Sorcery. Create a 3/3 blue Robot Warrior artifact
/// creature token; if this was cast from a graveyard, put two +1/+1 counters on
/// it. Flashback {5}{U}.
pub fn retrieve_the_esper() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Retrieve the Esper",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), u()]))],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: robot },
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Seq(vec![])),
            },
        ]),
        ..Default::default()
    }
}

/// Circle of Power — {3}{B} Sorcery. Draw two cards and lose 2 life; create a
/// 0/1 Wizard token that pings on your noncreature casts; Wizards you control
/// get +1/+0 and gain lifelink until end of turn.
pub fn circle_of_power() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Wizard".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    let wizards = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Wizard)
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Circle of Power",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: wizard },
            Effect::PumpPT {
                what: wizards(),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: wizards(),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Unexpected Request — {2}{R} Sorcery. Gain control of target creature until end
/// of turn, untap it, and it gains haste. (The optional Equipment-attach rider
/// is omitted.)
pub fn unexpected_request() -> CardDefinition {
    CardDefinition {
        name: "Unexpected Request",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Resentful Revelation — {1}{B} Sorcery. Look at the top three cards of your
/// library; put one into your hand and the rest into your graveyard.
/// Flashback {6}{B}.
pub fn resentful_revelation() -> CardDefinition {
    CardDefinition {
        name: "Resentful Revelation",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(6), b()]))],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Gaius van Baelsar — {2}{B}{B} 3/2 Legendary Human Soldier. ETB: choose one —
/// each player sacrifices a creature token, a nontoken creature, or an
/// enchantment (their choice).
pub fn gaius_van_baelsar() -> CardDefinition {
    let edict = |filter: SelectionRequirement| Effect::Sacrifice {
        who: Selector::Player(PlayerRef::EachPlayer),
        count: Value::ONE,
        filter,
    };
    CardDefinition {
        name: "Gaius van Baelsar",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            edict(SelectionRequirement::Creature.and(SelectionRequirement::IsToken)),
            edict(SelectionRequirement::Creature.and(SelectionRequirement::NotToken)),
            edict(SelectionRequirement::Enchantment),
        ]))],
        ..Default::default()
    }
}

// ── modern_decks batch 4: FIN recursion, token lord, vehicle ──────────────────

/// Sorceress's Schemes — {3}{R} Sorcery. Return target instant or sorcery card
/// from your graveyard to your hand; add {R}. Flashback {4}{R}. (The exiled-
/// flashback-card target is omitted.)
pub fn sorceresss_schemes() -> CardDefinition {
    CardDefinition {
        name: "Sorceress's Schemes",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r()]))],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::ONE) },
        ]),
        ..Default::default()
    }
}

/// Rinoa Heartilly — {3}{G}{W} 4/4 Legendary Human Rebel Warlock. ETB: create
/// Angelo, a legendary 1/1 green and white Dog. Whenever Rinoa attacks, another
/// target creature you control gets +1/+1 until end of turn for each creature
/// you control.
pub fn rinoa_heartilly() -> CardDefinition {
    let angelo = TokenDefinition {
        name: "Angelo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Rinoa Heartilly",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: angelo }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        )),
                        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    },
                    toughness: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        )),
                        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    },
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// The Regalia — {4} Legendary Artifact — Vehicle 4/4 with haste. Whenever it
/// attacks, reveal from the top until you reveal a land, put it onto the
/// battlefield tapped, and the rest on the bottom in a random order. Crew 1.
pub fn the_regalia() -> CardDefinition {
    CardDefinition {
        name: "The Regalia",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                cap: Value::Const(60), // "until a land" — capped past any library size
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        }],
        ..Default::default()
    }
}

/// Dragoon's Lance — {1}{W} Equipment. Job select. Equipped creature gets +1/+0,
/// is a Knight, and has flying during your turn. Equip {4}.
pub fn dragoons_lance() -> CardDefinition {
    let mut def = crate::sets::decks::job_select_equipment(
        "Dragoon's Lance",
        cost(&[generic(1), w()]),
        cost(&[generic(4)]),
        1,
        0,
        vec![],
        Some(CreatureType::Knight),
    );
    if let Some(bonus) = def.equipped_bonus.as_mut() {
        bonus.conditional.push(crate::card::ConditionalEquipBonus {
            host_filter: SelectionRequirement::Creature,
            power: 0,
            toughness: 0,
            keywords: vec![Keyword::Flying],
            predicate: Some(Predicate::IsTurnOf(PlayerRef::You)),
        });
    }
    def
}

/// Aettir and Priwen — {6} Legendary Equipment. Equipped creature has base
/// power and toughness X/X, where X is your life total. Equip {5}.
pub fn aettir_and_priwen() -> CardDefinition {
    CardDefinition {
        name: "Aettir and Priwen",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equipped_bonus: Some(EquipBonus {
            set_base_pt_dynamic: Some(crate::card::EquipDynamicValue::ControllerLife),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Excalibur II — {1} Legendary Equipment. Whenever you gain life, put a charge
/// counter on it; equipped creature gets +1/+1 per charge counter. Equip {3}.
pub fn excalibur_ii() -> CardDefinition {
    CardDefinition {
        name: "Excalibur II",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        equipped_bonus: Some(EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: SelectionRequirement::Any,
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::Charge),
                count_graveyard: None,
                count_all_graveyards: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// A Realm Reborn — {4}{G}{G} Enchantment. Other permanents you control have
/// "{T}: Add one mana of any color."
pub fn a_realm_reborn() -> CardDefinition {
    CardDefinition {
        name: "A Realm Reborn",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![crate::effect::shortcut::grant_tap_for_any_color(
            SelectionRequirement::ControlledByYou.and(SelectionRequirement::OtherThanSource),
        )],
        ..Default::default()
    }
}

/// Delivery Moogle — {3}{W} 3/2 Moogle with flying. ETB: search your library
/// and/or graveyard for an artifact card with mana value 2 or less and put it
/// into your hand.
pub fn delivery_moogle() -> CardDefinition {
    CardDefinition {
        name: "Delivery Moogle",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moogle], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::SearchLibraryOrGraveyard {
            who: PlayerRef::You,
            filter: SelectionRequirement::Artifact
                .and(SelectionRequirement::ManaValueAtMost(2)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Combat Tutorial — {2}{U} Sorcery. Target player draws two cards. Put a
/// +1/+1 counter on up to one target creature you control.
pub fn combat_tutorial() -> CardDefinition {
    CardDefinition {
        name: "Combat Tutorial",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::Target(0), amount: Value::Const(2) },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

// ── Job select Equipment cycle ────────────────────────────────────────────────

/// Build a Job-select Equipment whose granted `EquipBonus` also carries
/// triggered abilities (the "equipped creature has '…'" clauses).
fn job_equipment_with_triggers(
    name: &'static str,
    mana: crate::mana::ManaCost,
    equip: crate::mana::ManaCost,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
    add_type: CreatureType,
    triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    let mut def = crate::sets::decks::job_select_equipment(
        name, mana, equip, power, toughness, keywords, Some(add_type),
    );
    if let Some(bonus) = def.equipped_bonus.as_mut() {
        bonus.triggered_abilities = triggers;
    }
    def
}

/// Samurai's Katana — {2}{R} Equipment. Job select. +2/+2, trample, haste, and
/// a Samurai. Equip {5}.
pub fn samurais_katana() -> CardDefinition {
    crate::sets::decks::job_select_equipment(
        "Samurai's Katana",
        cost(&[generic(2), r()]),
        cost(&[generic(5)]),
        2,
        2,
        vec![Keyword::Trample, Keyword::Haste],
        Some(CreatureType::Samurai),
    )
}

/// Warrior's Sword — {3}{R} Equipment. Job select. +3/+2 and a Warrior.
/// Equip {5}.
pub fn warriors_sword() -> CardDefinition {
    crate::sets::decks::job_select_equipment(
        "Warrior's Sword",
        cost(&[generic(3), r()]),
        cost(&[generic(5)]),
        3,
        2,
        vec![],
        Some(CreatureType::Warrior),
    )
}

/// Dark Knight's Greatsword — {2}{B} Equipment. Job select. +3/+0 and a Knight.
/// (Printed equip is "Pay 3 life, once each turn"; modeled as Equip {3}.)
pub fn dark_knights_greatsword() -> CardDefinition {
    crate::sets::decks::job_select_equipment(
        "Dark Knight's Greatsword",
        cost(&[generic(2), b()]),
        cost(&[generic(3)]),
        3,
        0,
        vec![],
        Some(CreatureType::Knight),
    )
}

/// Thief's Knife — {2}{U} Equipment. Job select. +1/+1, a Rogue, and "whenever
/// this creature deals combat damage to a player, draw a card." Equip {4}.
pub fn thiefs_knife() -> CardDefinition {
    job_equipment_with_triggers(
        "Thief's Knife",
        cost(&[generic(2), u()]),
        cost(&[generic(4)]),
        1,
        1,
        vec![],
        CreatureType::Rogue,
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
    )
}

/// Ninja's Blades — {2}{B} Equipment. Job select. +1/+1, a Ninja, and "whenever
/// this creature deals combat damage to a player, draw a card, then discard a
/// card. That player loses life equal to the discarded card's mana value."
/// Equip {2}.
pub fn ninjas_blades() -> CardDefinition {
    job_equipment_with_triggers(
        "Ninja's Blades",
        cost(&[generic(2), b()]),
        cost(&[generic(2)]),
        1,
        1,
        vec![],
        CreatureType::Ninja,
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::LoseLife {
                    who: Selector::Target(0),
                    amount: Value::ManaValueOf(Box::new(Selector::DiscardedThisResolution {
                        filter: SelectionRequirement::Any,
                    })),
                },
            ]),
        }],
    )
}

/// Red Mage's Rapier — {1}{R} Equipment. Job select. Equipped creature is a
/// Wizard with "whenever you cast a noncreature spell, this creature gets +2/+0
/// until end of turn." Equip {3}.
pub fn red_mages_rapier() -> CardDefinition {
    job_equipment_with_triggers(
        "Red Mage's Rapier",
        cost(&[generic(1), r()]),
        cost(&[generic(3)]),
        0,
        0,
        vec![],
        CreatureType::Wizard,
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
    )
}

/// Black Mage's Rod — {1}{B} Equipment. Job select. +1/+0, a Wizard, and
/// "whenever you cast a noncreature spell, this creature deals 1 damage to each
/// opponent." Equip {3}.
pub fn black_mages_rod() -> CardDefinition {
    job_equipment_with_triggers(
        "Black Mage's Rod",
        cost(&[generic(1), b()]),
        cost(&[generic(3)]),
        1,
        0,
        vec![],
        CreatureType::Wizard,
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
    )
}

/// Sage's Nouliths — {1}{U} Equipment. Job select. +1/+0, a Cleric, and
/// "whenever this creature attacks, untap target attacking creature."
/// Equip {3}.
pub fn sages_nouliths() -> CardDefinition {
    job_equipment_with_triggers(
        "Sage's Nouliths",
        cost(&[generic(1), u()]),
        cost(&[generic(3)]),
        1,
        0,
        vec![],
        CreatureType::Cleric,
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Untap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
                ),
                up_to: None,
            },
        }],
    )
}

// ── Tiered spells ─────────────────────────────────────────────────────────────

/// Fire Magic — {R} Instant. Tiered: Fire {0} / Fira {2} / Firaga {5} deal
/// 1 / 2 / 3 damage to each creature.
pub fn fire_magic() -> CardDefinition {
    use crate::effect::SpreeMode;
    let sweep = |n: i32| Effect::DealDamage {
        to: Selector::EachPermanent(SelectionRequirement::Creature),
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Fire Magic",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode { cost: cost(&[]), effect: sweep(1) },
                SpreeMode { cost: cost(&[generic(2)]), effect: sweep(2) },
                SpreeMode { cost: cost(&[generic(5)]), effect: sweep(3) },
            ],
        },
        ..Default::default()
    }
}

/// Thunder Magic — {R} Instant. Tiered: Thunder {0} / Thundara {3} /
/// Thundaga {5}{R} deal 2 / 4 / 8 damage to target creature.
pub fn thunder_magic() -> CardDefinition {
    use crate::effect::SpreeMode;
    let bolt = |n: i32| Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Thunder Magic",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode { cost: cost(&[]), effect: bolt(2) },
                SpreeMode { cost: cost(&[generic(3)]), effect: bolt(4) },
                SpreeMode { cost: cost(&[generic(5), r()]), effect: bolt(8) },
            ],
        },
        ..Default::default()
    }
}

/// Ice Magic — {1}{U} Instant. Tiered: Blizzard {0} bounces, Blizzara {2} puts
/// on top or bottom, Blizzaga {5}{U} shuffles the creature away.
pub fn ice_magic() -> CardDefinition {
    use crate::effect::{LibraryPosition, SpreeMode};
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::Target(0)));
    let move_to = |to: ZoneDest| Effect::Move {
        what: target_filtered(SelectionRequirement::Creature),
        to,
    };
    CardDefinition {
        name: "Ice Magic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode { cost: cost(&[]), effect: move_to(ZoneDest::Hand(owner())) },
                SpreeMode {
                    cost: cost(&[generic(2)]),
                    effect: move_to(ZoneDest::Library {
                        who: owner(),
                        pos: LibraryPosition::OwnerChoice,
                    }),
                },
                SpreeMode {
                    cost: cost(&[generic(5), u()]),
                    effect: move_to(ZoneDest::Library {
                        who: owner(),
                        pos: LibraryPosition::Shuffled,
                    }),
                },
            ],
        },
        ..Default::default()
    }
}

/// Tifa's Limit Break — {G} Instant. Tiered: Somersault {0} gives +2/+2,
/// Meteor Strikes {2} doubles P/T, Final Heaven {6}{G} triples it.
pub fn tifas_limit_break() -> CardDefinition {
    use crate::effect::SpreeMode;
    let target = || target_filtered(SelectionRequirement::Creature);
    // "Double" adds the creature's current P/T once; "triple" adds it twice.
    let scale = |mult: i32| Effect::PumpPT {
        what: target(),
        power: Value::Times(
            Box::new(Value::Const(mult)),
            Box::new(Value::PowerOf(Box::new(Selector::Target(0)))),
        ),
        toughness: Value::Times(
            Box::new(Value::Const(mult)),
            Box::new(Value::ToughnessOf(Box::new(Selector::Target(0)))),
        ),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Tifa's Limit Break",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode {
                    cost: cost(&[]),
                    effect: Effect::PumpPT {
                        what: target(),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                },
                SpreeMode { cost: cost(&[generic(2)]), effect: scale(1) },
                SpreeMode { cost: cost(&[generic(6), g()]), effect: scale(2) },
            ],
        },
        ..Default::default()
    }
}

// ── Jump / misc FIN batch ─────────────────────────────────────────────────────

/// Jump (Final Fantasy) — "During your turn, this creature has flying."
fn jump() -> StaticAbility {
    StaticAbility {
        description: "Jump — During your turn, this creature has flying.",
        effect: StaticEffect::SelfHasKeywordWhile {
            keyword: Keyword::Flying,
            condition: SelectionRequirement::ControllersTurn,
        },
    }
}

/// Freya Crescent — {R} 1/1 Rat Knight. Jump; {T}: Add {R}, spendable only on
/// Equipment spells and equip abilities.
pub fn freya_crescent() -> CardDefinition {
    CardDefinition {
        name: "Freya Crescent",
        cost: cost(&[r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![jump()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Red])),
                    crate::mana::SpendRestriction::EquipmentSpellsOrEquip,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seymour Flux — {4}{B} 5/5 Spirit Avatar. Upkeep: you may pay 1 life to draw
/// a card and put a +1/+1 counter on it.
pub fn seymour_flux() -> CardDefinition {
    CardDefinition {
        name: "Seymour Flux",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayPayLife {
                amount: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Self-Destruct — {1}{R} Instant. Target creature you control deals damage
/// equal to its power to any other target and that much to itself.
pub fn self_destruct() -> CardDefinition {
    CardDefinition {
        name: "Self-Destruct",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::Target(1),
            },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::Target(0),
            },
        ]),
        ..Default::default()
    }
}

/// Valkyrie Aerial Unit — {5}{U}{U} 5/4 Construct with affinity for artifacts
/// and flying. ETB: surveil 2.
pub fn valkyrie_aerial_unit() -> CardDefinition {
    CardDefinition {
        name: "Valkyrie Aerial Unit",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(
            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
        ),
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Town Greeter — {1}{G} 1/1 Citizen. ETB: mill four, then you may put a land
/// card from among them into your hand.
pub fn town_greeter() -> CardDefinition {
    CardDefinition {
        name: "Town Greeter",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(4),
            filter: SelectionRequirement::Land,
        })],
        ..Default::default()
    }
}

/// Vayne's Treachery — {1}{B} Instant. Kicker—sacrifice an artifact or
/// creature. Target creature gets -2/-2, or -6/-6 if kicked.
pub fn vaynes_treachery() -> CardDefinition {
    CardDefinition {
        name: "Vayne's Treachery",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[]))],
        kicker_additional_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-6),
                toughness: Value::Const(-6),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}
