//! Final Fantasy (FIN) — a first wave of cards from the Universes Beyond set.
//! Each card has a functionality test in `crabomination/src/tests/fin.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::card::WardCost;
use crate::effect::shortcut::{
    etb, etb_mint_token, etb_surveil, grant_tap_for_any_color, mint_treasures, target_filtered,
};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticEffect, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, SpendRestriction};

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

/// The Gold Saucer — Land — Town. {T}: Add {C}. {2}, {T}: Flip a coin; on a win,
/// create a Treasure token. {3}, {T}, Sacrifice two artifacts: Draw a card.
pub fn the_gold_saucer() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "The Gold Saucer",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::FlipCoin {
                    count: Value::ONE,
                    on_heads: Box::new(crate::effect::shortcut::mint_treasures(1)),
                    on_tails: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sac_other_filter: Some((SelectionRequirement::Artifact, 2)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

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

/// Yuna, Hope of Spira — {3}{G}{W} 3/5 Human Cleric. During your turn, Yuna and
/// enchantment creatures you control have trample, lifelink, and ward {2}. At
/// the beginning of your end step, return up to one target enchantment card from
/// your graveyard to the battlefield with a finality counter on it.
pub fn yuna_hope_of_spira() -> CardDefinition {
    let ward2 = || Keyword::Ward(WardCost::generic(2));
    let self_kw = |kw: Keyword| StaticAbility {
        description: "During your turn, Yuna has this keyword.",
        effect: StaticEffect::SelfHasKeywordIf {
            keyword: kw,
            condition: Predicate::IsTurnOf(PlayerRef::You),
        },
    };
    CardDefinition {
        name: "Yuna, Hope of Spira",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        static_abilities: vec![
            // Yuna herself (she isn't an enchantment creature, so the anthem
            // filter below wouldn't otherwise reach her).
            self_kw(Keyword::Trample),
            self_kw(Keyword::Lifelink),
            self_kw(ward2()),
            // Enchantment creatures you control, only on your turn.
            StaticAbility {
                description: "During your turn, enchantment creatures you control have trample, lifelink, and ward {2}.",
                effect: StaticEffect::AnthemForFilter {
                    filter: SelectionRequirement::HasCardType(CardType::Enchantment)
                        .and(SelectionRequirement::Creature),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Trample, Keyword::Lifelink, ward2()],
                    opponents: false,
                    only_your_turn: true,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::HasCardType(CardType::Enchantment)
                            .and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Elixir — {1} Artifact. Enters tapped. {5}, {T}, Exile this artifact: Shuffle
/// all nonland cards from your graveyard into your library. You gain life equal
/// to the number of cards shuffled into your library this way.
pub fn elixir() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Elixir",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::ShuffleFilteredGraveyardIntoLibraryGainLife {
                who: PlayerRef::You,
                filter: SelectionRequirement::Nonland,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Clive's Hideaway — Land — Town. Hideaway 4. {T}: Add {C}. {2}, {T}: You may
/// play the exiled card without paying its mana cost if you control four or more
/// legendary creatures.
pub fn clives_hideaway() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "Clive's Hideaway",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasSupertype(Supertype::Legendary))
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(4),
                }),
                effect: Effect::CastWithoutPayingImmediate {
                    what: Selector::CardExiledWithSource,
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Hideaway { count: Value::Const(4) },
        }],
        ..Default::default()
    }
}

/// Starting Town — Land — Town. {T}: Add {C}. {T}, Pay 1 life: Add one mana of
/// any color. (Approximation: the "enters tapped unless it's your first,
/// second, or third turn" clause is modeled as always entering tapped.)
pub fn starting_town() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "Starting Town",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![super::etb_tap()],
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
            override_colors: None,
            enters_tapped: false,
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

/// A Realm Reborn — {4}{G}{G} Enchantment. Other permanents you control have
/// "{T}: Add one mana of any color."
pub fn a_realm_reborn() -> CardDefinition {
    CardDefinition {
        name: "A Realm Reborn",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![grant_tap_for_any_color(
            SelectionRequirement::ControlledByYou.and(SelectionRequirement::OtherThanSource),
        )],
        ..Default::default()
    }
}

/// Combat Tutorial — {2}{U} Sorcery. Target player draws two cards. Put a
/// +1/+1 counter on up to one target creature you control. (The creature slot
/// is optional: with no legal creature the spell just draws.)
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

/// Seymour Flux — {4}{B} 5/5 Spirit Avatar. At the beginning of your upkeep,
/// you may pay 1 life. If you do, draw a card and put a +1/+1 counter on it.
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
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayPayLife {
                description: "Pay 1 life: draw a card and grow Seymour Flux?".into(),
                amount: Value::ONE,
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Cloud of Darkness — {2}{B}{G}{G} 3/3 Avatar with flying. Particle Beam: when
/// it enters, target creature an opponent controls gets -X/-X until end of turn,
/// where X is the number of permanent cards in your graveyard.
pub fn cloud_of_darkness() -> CardDefinition {
    CardDefinition {
        name: "Cloud of Darkness",
        cost: cost(&[generic(2), b(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Diff(
                Box::new(Value::ZERO),
                Box::new(Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::PermanentCard,
                }),
            ),
            toughness: Value::Diff(
                Box::new(Value::ZERO),
                Box::new(Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::PermanentCard,
                }),
            ),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Cargo Ship — {1}{U} 2/3 Vehicle with flying and vigilance. {T}: Add {C},
/// spendable only on artifact spells/abilities. Crew 1.
pub fn cargo_ship() -> CardDefinition {
    CardDefinition {
        name: "Cargo Ship",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Crew(1)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ONE)),
                    crate::mana::SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Wind Crystal — {2}{W}{W} Legendary Artifact. White spells you cast cost
/// {1} less. If you would gain life, you gain twice that much instead.
/// {4}{W}{W}, {T}: Creatures you control gain flying and lifelink until EOT.
pub fn the_wind_crystal() -> CardDefinition {
    use crate::effect::PlayerStaticTarget;
    CardDefinition {
        name: "The Wind Crystal",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "White spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::HasColor(Color::White),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "If you would gain life, you gain twice that much instead.",
                effect: StaticEffect::LifeGainMultiplier {
                    target: PlayerStaticTarget::Controller,
                    factor: 2,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4), w(), w()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Fire Crystal — {2}{R}{R} Legendary Artifact. Red spells you cast cost {1}
/// less. Creatures you control have haste. {4}{R}{R}, {T}: Create a token that's
/// a copy of target creature you control; sacrifice it at the next end step.
pub fn the_fire_crystal() -> CardDefinition {
    CardDefinition {
        name: "The Fire Crystal",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Red spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::HasColor(Color::Red),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Creatures you control have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4), r(), r()]),
            effect: Effect::CreateTokenCopiesHasteSac {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ancient Adamantoise — {5}{G}{G}{G} 8/20 Turtle. Vigilance, ward {3}. All
/// damage that would be dealt to you and other permanents you control is dealt
/// to it instead. When it dies, exile it and create ten Treasure tokens.
/// (The cleanup-damage-retention rider is omitted.)
pub fn ancient_adamantoise() -> CardDefinition {
    CardDefinition {
        name: "Ancient Adamantoise",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Turtle], ..Default::default() },
        power: 8,
        toughness: 20,
        keywords: vec![Keyword::Vigilance, Keyword::Ward(WardCost::generic(3))],
        static_abilities: vec![StaticAbility {
            description: "All damage that would be dealt to you and other permanents you control is dealt to this creature instead.",
            effect: StaticEffect::RedirectDamageToSelf,
        }],
        dies_to_exile: true,
        triggered_abilities: vec![crate::effect::shortcut::on_dies(mint_treasures(10))],
        ..Default::default()
    }
}

/// Poison the Waters — {1}{B} Sorcery. Choose one — all creatures get -1/-1
/// until end of turn; or target player reveals their hand, you choose an
/// artifact or creature card from it, and that player discards it.
pub fn poison_the_waters() -> CardDefinition {
    CardDefinition {
        name: "Poison the Waters",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            },
        ]),
        ..Default::default()
    }
}

/// Valkyrie Aerial Unit — {5}{U}{U} 5/4 Construct artifact creature. Affinity
/// for artifacts, flying. When it enters, surveil 2.
pub fn valkyrie_aerial_unit() -> CardDefinition {
    CardDefinition {
        name: "Valkyrie Aerial Unit",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(SelectionRequirement::Artifact),
        triggered_abilities: vec![etb_surveil(2)],
        ..Default::default()
    }
}

/// Ice Flan — {4}{U}{U} 5/4 Elemental Ooze. When it enters, tap target artifact
/// or creature an opponent controls and put a stun counter on it.
/// Islandcycling {2}.
pub fn ice_flan() -> CardDefinition {
    CardDefinition {
        name: "Ice Flan",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Ooze],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), crate::card::LandType::Island)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Namazu Trader — {3}{B} 3/4 Fish Citizen. When it enters, you lose 1 life and
/// create a Treasure token. Whenever it attacks, you may sacrifice another
/// creature or artifact. If you do, surveil 2.
pub fn namazu_trader() -> CardDefinition {
    CardDefinition {
        name: "Namazu Trader",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                mint_treasures(1),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice another creature or artifact to surveil 2?".into(),
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Artifact)
                        .and(SelectionRequirement::OtherThanSource),
                    count: Value::ONE,
                    then: Box::new(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Ultros, Obnoxious Octopus — {1}{U} 2/1 Octopus. Whenever you cast a
/// noncreature spell with at least four mana spent, tap target creature an
/// opponent controls and put a stun counter on it. With at least eight mana
/// spent, put eight +1/+1 counters on Ultros instead.
pub fn ultros_obnoxious_octopus() -> CardDefinition {
    use crate::effect::shortcut::cast_is_noncreature;
    CardDefinition {
        name: "Ultros, Obnoxious Octopus",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Octopus], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        cast_is_noncreature(),
                        Predicate::CastSpellManaSpentAtLeast(4),
                    ]),
                ),
                effect: Effect::Seq(vec![
                    Effect::Tap {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByOpponent),
                        ),
                    },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::ONE,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        cast_is_noncreature(),
                        Predicate::CastSpellManaSpentAtLeast(8),
                    ]),
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(8),
                },
            },
        ],
        ..Default::default()
    }
}

/// Aerith Rescue Mission — {3}{W} Sorcery. Choose one — create three 1/1
/// colorless Hero creature tokens; or tap up to three target creatures and put
/// a stun counter on one of them.
pub fn aerith_rescue_mission() -> CardDefinition {
    let hero = TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Aerith Rescue Mission",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: hero },
            Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 3,
                    filter: SelectionRequirement::Creature,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Zack Fair — {W} 0/1 Soldier. Enters with a +1/+1 counter. {1}, Sacrifice
/// Zack Fair: Target creature you control gains indestructible until end of
/// turn. (The counter- and Equipment-transfer riders are omitted.)
pub fn zack_fair() -> CardDefinition {
    CardDefinition {
        name: "Zack Fair",
        cost: cost(&[w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 0,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Final Days — {2}{B}{B} Sorcery. Create two 2/2 black Horror creature
/// tokens. If this spell was cast from a graveyard, instead create one for each
/// creature card in your graveyard. Flashback {4}{B}{B}. (Tokens enter untapped.)
pub fn the_final_days() -> CardDefinition {
    let horror = TokenDefinition {
        name: "Horror".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Final Days",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), b(), b()]))],
        effect: Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                },
                definition: horror.clone(),
            }),
            else_: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: horror,
            }),
        },
        ..Default::default()
    }
}

/// From Father to Son — {1}{W} Sorcery. Search your library for a Vehicle card
/// and put it into your hand — or onto the battlefield if this spell was cast
/// from a graveyard — then shuffle. Flashback {4}{W}{W}{W}.
pub fn from_father_to_son() -> CardDefinition {
    CardDefinition {
        name: "From Father to Son",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), w(), w(), w()]))],
        effect: Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Call the Mountain Chocobo — {3}{R} Sorcery. Search your library for a
/// Mountain, put it into your hand, then shuffle. Create a 2/2 green Bird token
/// that gets +1/+0 until end of turn whenever a land you control enters.
/// Flashback {5}{R}.
pub fn call_the_mountain_chocobo() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Chocobo".into(),
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
        name: "Call the Mountain Chocobo",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r()]))],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasLandType(crate::card::LandType::Mountain),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: bird },
        ]),
        ..Default::default()
    }
}

/// Traveling Chocobo — {2}{G} 3/2 Bird. You may play lands and cast Bird spells
/// from the top of your library. (The extra-trigger "additional time" rider is
/// omitted.)
pub fn traveling_chocobo() -> CardDefinition {
    CardDefinition {
        name: "Traveling Chocobo",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "You may play lands and cast Bird spells from the top of your library.",
            effect: StaticEffect::PlayFromLibraryTop {
                filter: SelectionRequirement::Land
                    .or(SelectionRequirement::HasCreatureType(CreatureType::Bird)),
            },
        }],
        ..Default::default()
    }
}

/// The Earth Crystal — {2}{G}{G} Legendary Artifact. Green spells you cast cost
/// {1} less. If one or more +1/+1 counters would be put on a creature you
/// control, twice that many are put on it instead. {4}{G}{G}, {T}: Distribute
/// two +1/+1 counters among one or two target creatures you control.
pub fn the_earth_crystal() -> CardDefinition {
    CardDefinition {
        name: "The Earth Crystal",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Green spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::HasColor(Color::Green),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "If one or more +1/+1 counters would be put on a creature you control, twice that many are put on it instead.",
                effect: StaticEffect::DoublePlusOneCounters,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4), g(), g()]),
            effect: Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::PlusOnePlusOne,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                max_targets: 2,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Prima Vista — {4}{U} 5/3 Vehicle with flying. Whenever you cast a
/// noncreature spell with at least four mana spent, it becomes an artifact
/// creature until end of turn. Crew 2.
pub fn the_prima_vista() -> CardDefinition {
    use crate::effect::shortcut::cast_is_noncreature;
    CardDefinition {
        name: "The Prima Vista",
        cost: cost(&[generic(4), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    cast_is_noncreature(),
                    Predicate::CastSpellManaSpentAtLeast(4),
                ]),
            ),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(3),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Quistis Trepe — {2}{U} 2/2 Wizard. Blue Magic: when it enters, you may cast
/// target instant or sorcery card from a graveyard; if that spell would go to a
/// graveyard, exile it instead. (Any mana can pay for it.)
pub fn quistis_trepe() -> CardDefinition {
    CardDefinition {
        name: "Quistis Trepe",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CastWithoutPayingImmediate {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
            source_zone: crate::card::Zone::Graveyard,
            exile_after: true,
        })],
        ..Default::default()
    }
}

/// Town Greeter — {1}{G} 1/1 Citizen. When it enters, mill four cards, then you
/// may put a land card from among them into your hand. (The "if it's a Town,
/// gain 2 life" rider is omitted.)
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

/// Giott, King of the Dwarves — {R}{W} 1/1 Dwarf Noble with double strike.
/// Whenever Giott or another Dwarf you control enters, and whenever an Equipment
/// you control enters, you may discard a card. If you do, draw a card.
pub fn giott_king_of_the_dwarves() -> CardDefinition {
    let loot = || Effect::MayDiscard {
        description: "Discard a card to draw a card?".into(),
        count: Value::ONE,
        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: None,
    };
    CardDefinition {
        name: "Giott, King of the Dwarves",
        cost: cost(&[r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Dwarf),
                    }),
                effect: loot(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                    }),
                effect: loot(),
            },
        ],
        ..Default::default()
    }
}

/// Freya Crescent — {R} 1/1 Rat Knight. Jump — during your turn she has flying.
/// {T}: Add {R}, spendable only on Equipment spells or equip abilities.
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
        static_abilities: vec![StaticAbility {
            description: "Jump — During your turn, Freya Crescent has flying.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Flying,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Red, Value::ONE)),
                    SpendRestriction::EquipmentOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Balthier and Fran — {1}{R}{G} 4/3 Human Rabbit with reach. Vehicles you
/// control get +1/+1 and have vigilance and reach. (The extra-combat-phase rider
/// when a Vehicle it crewed attacks is omitted — no crew-source attribution.)
pub fn balthier_and_fran() -> CardDefinition {
    CardDefinition {
        name: "Balthier and Fran",
        cost: cost(&[generic(1), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rabbit],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Vehicles you control get +1/+1 and have vigilance and reach.",
            effect: StaticEffect::AnthemForFilter {
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Vigilance, Keyword::Reach],
                opponents: false,
                only_your_turn: false,
            },
        }],
        ..Default::default()
    }
}

/// The 1/1 colorless Hero token minted by Job select (CR 702.182).
fn hero_token() -> TokenDefinition {
    TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    }
}

/// Job select ETB: mint a Hero and attach this Equipment to it (CR 702.182).
fn job_select_etb() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: hero_token() },
        Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
    ]))
}

/// Trigger: "Whenever you cast a noncreature spell, [effect]" fired off the host.
fn on_cast_noncreature(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
        effect,
    }
}

/// Astrologian's Planisphere — {1}{U} Equipment. Job select. Equipped creature is
/// a Wizard and has "Whenever you cast a noncreature spell, put a +1/+1 counter on
/// this creature." Equip {2}. (The "draw your third card each turn" counter half
/// is omitted — no drew-Nth-card-this-turn trigger yet.)
pub fn astrologians_planisphere() -> CardDefinition {
    CardDefinition {
        name: "Astrologian's Planisphere",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            add_creature_types: vec![CreatureType::Wizard],
            triggered_abilities: vec![on_cast_noncreature(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Samurai's Katana — {2}{R} Equipment. Job select. Equipped creature gets +2/+2,
/// has trample and haste, and is a Samurai. Equip {5}.
pub fn samurais_katana() -> CardDefinition {
    CardDefinition {
        name: "Samurai's Katana",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample, Keyword::Haste],
            add_creature_types: vec![CreatureType::Samurai],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Red Mage's Rapier — {1}{R} Equipment. Job select. Equipped creature is a
/// Wizard and has "Whenever you cast a noncreature spell, this creature gets
/// +2/+0 until end of turn." Equip {3}.
pub fn red_mages_rapier() -> CardDefinition {
    CardDefinition {
        name: "Red Mage's Rapier",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            add_creature_types: vec![CreatureType::Wizard],
            triggered_abilities: vec![on_cast_noncreature(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Black Mage's Rod — {1}{B} Equipment. Job select. Equipped creature gets +1/+0,
/// is a Wizard, and has "Whenever you cast a noncreature spell, this creature
/// deals 1 damage to each opponent." Equip {3}.
pub fn black_mages_rod() -> CardDefinition {
    CardDefinition {
        name: "Black Mage's Rod",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            add_creature_types: vec![CreatureType::Wizard],
            triggered_abilities: vec![on_cast_noncreature(Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Relentless X-ATM092 — {6} 6/5 Robot Spider artifact creature. Can't be blocked
/// except by three or more creatures. {8}: return it from your graveyard to the
/// battlefield tapped with a finality counter on it.
pub fn relentless_x_atm092() -> CardDefinition {
    CardDefinition {
        name: "Relentless X-ATM092",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Spider],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Qutrub Forayer — {2}{B} 3/2 Zombie Horror. When it enters, choose one —
/// destroy target creature that was dealt damage this turn; or exile up to two
/// target cards from graveyards. (The "single graveyard" clause is approximated
/// as any graveyards.)
pub fn qutrub_forayer() -> CardDefinition {
    CardDefinition {
        name: "Qutrub Forayer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::DealtDamageThisTurn),
                ),
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::InGraveyard,
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
            },
        ]))],
        ..Default::default()
    }
}

/// Ninja's Blades — {2}{B} Equipment. Job select. Equipped creature gets +1/+1,
/// is a Ninja, and has "Whenever this creature deals combat damage to a player,
/// draw a card, then discard a card. That player loses life equal to the
/// discarded card's mana value." Equip {2}.
pub fn ninjas_blades() -> CardDefinition {
    CardDefinition {
        name: "Ninja's Blades",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            add_creature_types: vec![CreatureType::Ninja],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::ManaValueOf(Box::new(Selector::DiscardedThisResolution {
                            filter: SelectionRequirement::Any,
                        })),
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Machinist's Arsenal — {4}{W} Equipment. Job select. Equipped creature gets
/// +2/+2 for each artifact you control and is an Artificer. Equip {4}.
pub fn machinists_arsenal() -> CardDefinition {
    CardDefinition {
        name: "Machinist's Arsenal",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            add_creature_types: vec![CreatureType::Artificer],
            scale: Some(crate::card::EquipScale {
                filter: SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                per_power: 2,
                per_toughness: 2,
                count_self_counters: None,
                count_graveyard: None,
                count_all_graveyards: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sage's Nouliths — {1}{U} Equipment. Job select. Equipped creature gets +1/+0,
/// is a Cleric, and has "Whenever this creature attacks, untap target attacking
/// creature." Equip {3}.
pub fn sages_nouliths() -> CardDefinition {
    CardDefinition {
        name: "Sage's Nouliths",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            add_creature_types: vec![CreatureType::Cleric],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Untap {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
                    ),
                    up_to: None,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Dragoon's Lance — {1}{W} Equipment. Job select. Equipped creature gets +1/+0,
/// is a Knight, and has flying during your turn. Equip {4}.
pub fn dragoons_lance() -> CardDefinition {
    CardDefinition {
        name: "Dragoon's Lance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            add_creature_types: vec![CreatureType::Knight],
            during_your_turn_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Cloud, Planet's Champion — {3}{R}{W} 4/4 Human Soldier Mercenary. During your
/// turn, while equipped, Cloud has double strike and indestructible. Equip
/// abilities you activate cost {2} less. (The reduction is modeled as applying to
/// all your equips, not just those targeting Cloud.)
pub fn cloud_planets_champion() -> CardDefinition {
    let equipped_this_turn = || Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::SourceIsEquipped,
    ]);
    CardDefinition {
        name: "Cloud, Planet's Champion",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human, CreatureType::Soldier, CreatureType::Mercenary,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, while equipped, Cloud has double strike.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::DoubleStrike,
                    condition: equipped_this_turn(),
                },
            },
            StaticAbility {
                description: "During your turn, while equipped, Cloud has indestructible.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Indestructible,
                    condition: equipped_this_turn(),
                },
            },
            StaticAbility {
                description: "Equip abilities you activate that target Cloud cost {2} less.",
                effect: StaticEffect::EquipCostReduction { amount: 2 },
            },
        ],
        ..Default::default()
    }
}

/// Opera Love Song — {1}{R} Instant. Choose one — exile the top two cards of your
/// library, you may play them until your next end step; or one or two target
/// creatures each get +2/+0 until end of turn.
pub fn opera_love_song() -> CardDefinition {
    CardDefinition {
        name: "Opera Love Song",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Blitzball — {3} Artifact. {T}: Add one mana of any color. "GOOOOAAAALLL!" —
/// {T}, Sacrifice this artifact: draw two cards, if you dealt combat damage to a
/// player this turn. (The printed "an opponent was dealt combat damage by a
/// legendary creature this turn" is approximated as your own combat hit.)
pub fn blitzball() -> CardDefinition {
    CardDefinition {
        name: "Blitzball",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                condition: Some(Predicate::DealtCombatDamageToPlayerThisTurn { who: PlayerRef::You }),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Seifer Almasy — {3}{R} 3/4 Human Knight. Whenever a creature you control
/// attacks alone, it gains double strike. Fire Cross — whenever Seifer deals
/// combat damage to a player, you may cast target instant or sorcery card with
/// mana value 3 or less from your graveyard without paying its mana cost (exiled
/// if it would leave the stack).
pub fn seifer_almasy() -> CardDefinition {
    CardDefinition {
        name: "Seifer Almasy",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
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
                effect: Effect::CastWithoutPayingImmediate {
                    what: target_filtered(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                            .and(SelectionRequirement::InYourGraveyard)
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    ),
                    source_zone: crate::card::Zone::Graveyard,
                    exile_after: true,
                },
            },
        ],
        ..Default::default()
    }
}

/// Raubahn, Bull of Ala Mhigo — {1}{R} 2/2 Human Warrior. Ward—pay life equal to
/// Raubahn's power. Whenever Raubahn attacks, attach up to one target Equipment
/// you control to it. (The printed "target attacking creature" is approximated
/// as Raubahn itself; the "up to one" is a required target.)
pub fn raubahn_bull_of_ala_mhigo() -> CardDefinition {
    CardDefinition {
        name: "Raubahn, Bull of Ala Mhigo",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::LifeSourcePower)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Attach {
                what: target_filtered(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                to: Selector::This,
            },
        }],
        ..Default::default()
    }
}

/// Golbez, Crystal Collector — {U}{B} 1/4 Human Wizard. Whenever an artifact you
/// control enters, surveil 1. At your end step, if you control four or more
/// artifacts, return target creature card from your graveyard to your hand; then
/// if you control eight or more, each opponent loses life equal to its power.
pub fn golbez_crystal_collector() -> CardDefinition {
    let artifacts_at_least = |n: i32| Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(
            SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
        ),
        n: Value::Const(n),
    };
    CardDefinition {
        name: "Golbez, Crystal Collector",
        cost: cost(&[u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Artifact,
                    }),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                    .with_filter(artifacts_at_least(4)),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(
                            SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::If {
                        cond: artifacts_at_least(8),
                        then: Box::new(Effect::LoseLife {
                            who: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::PowerOf(Box::new(Selector::Target(0))),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Jenova, Ancient Calamity — {2}{B}{G} 1/5 Alien. At the beginning of combat on
/// your turn, put +1/+1 counters equal to Jenova's power on up to one other
/// target creature; it becomes a Mutant. Whenever a Mutant you control dies
/// during your turn, draw cards equal to its power. (The "up to one" is modeled
/// as a required target; the granted Mutant type is preserved in the death LKI
/// snapshot, so the dies-draw fires for creatures Jenova turned into Mutants.)
pub fn jenova_ancient_calamity() -> CardDefinition {
    CardDefinition {
        name: "Jenova, Ancient Calamity",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Alien], ..Default::default() },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::OtherThanSource),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::PowerOf(Box::new(Selector::This)),
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::Target(0),
                        creature_types: vec![CreatureType::Mutant],
                        duration: Duration::Permanent,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::All(vec![
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: SelectionRequirement::HasCreatureType(CreatureType::Mutant),
                        },
                        Predicate::IsTurnOf(PlayerRef::You),
                    ])),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Ardyn, the Usurper — {5}{B}{B}{B} 4/4 Elder Human Noble. Demons you control
/// have menace, lifelink, and haste. Starscourge — at the beginning of combat on
/// your turn, exile up to one target creature card from a graveyard; if you do,
/// create a token copy of it that's a 5/5 black Demon. (The "up to one" is
/// modeled as a required target — no-op with no graveyard creature.)
pub fn ardyn_the_usurper() -> CardDefinition {
    CardDefinition {
        name: "Ardyn, the Usurper",
        cost: cost(&[generic(5), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Demons you control have menace, lifelink, and haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Demon),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Menace, Keyword::Lifelink, Keyword::Haste],
                opponents: false,
                only_your_turn: false,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
                    ),
                    to: ZoneDest::Exile,
                },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![CreatureType::Demon],
                    extra_card_types: vec![],
                    override_pt: Some((5, 5)),
                    override_colors: Some(vec![Color::Black]),
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Lightning, Security Sergeant — {2}{R} 2/3 Human Soldier with menace. Whenever
/// she deals combat damage to a player, exile the top card of your library; you
/// may play it (modeled as "for as long as it remains exiled").
pub fn lightning_security_sergeant() -> CardDefinition {
    CardDefinition {
        name: "Lightning, Security Sergeant",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::WhileExiled,
                pay_any_color: false,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Bartz and Boko — {3}{G}{G} 4/3 Human Bird with Affinity for Birds. When it
/// enters, each other Bird you control deals damage equal to its power to target
/// creature an opponent controls.
pub fn bartz_and_boko() -> CardDefinition {
    CardDefinition {
        name: "Bartz and Boko",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Bird],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Bird)
                .and(SelectionRequirement::ControlledByYou),
        ),
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Bird)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            }),
        })],
        ..Default::default()
    }
}

/// Magitek Scythe — {4} Equipment. When it enters, attach it to target creature
/// you control; that creature gains first strike and must be blocked this turn.
/// Equipped creature gets +2/+1. Equip {2}. (The "you may" attach is modeled as
/// a required target — no-op if you control no creature.)
pub fn magitek_scythe() -> CardDefinition {
    CardDefinition {
        name: "Magitek Scythe",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
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
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::MustBeBlocked,
                duration: Duration::EndOfTurn,
            },
        ]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Self-Destruct — {1}{R} Instant. Target creature you control deals X damage to
/// any other target and X damage to itself, where X is its power.
pub fn self_destruct() -> CardDefinition {
    CardDefinition {
        name: "Self-Destruct",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Any },
                amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                })),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Excalibur II — {1} Legendary Equipment. Whenever you gain life, put a charge
/// counter on Excalibur II. Equipped creature gets +1/+1 for each charge counter
/// on Excalibur II. Equip {3}.
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
            scale: Some(EquipScale {
                filter: SelectionRequirement::Any,
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::Charge),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Summon: Choco/Mog — {2}{W} Enchantment Creature — Saga Bird Moogle 3/3.
/// Chapters I–IV each: Stampede! — other creatures you control get +1/+0 until
/// end of turn. Sacrificed after IV (CR 714 saga rule, applied to a creature).
pub fn summon_choco_mog() -> CardDefinition {
    let stampede = || Effect::PumpPT {
        what: Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        ),
        power: Value::ONE,
        toughness: Value::ZERO,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Summon: Choco/Mog",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Bird, CreatureType::Moogle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        saga_chapters: vec![
            (1, stampede()),
            (2, stampede()),
            (3, stampede()),
            (4, stampede()),
        ],
        ..Default::default()
    }
}

/// Summon: Bahamut — {9} Enchantment Creature — Saga Dragon 9/9 with flying.
/// I, II — destroy up to one target nonland permanent. III — draw two cards.
/// IV — Mega Flare — deals damage equal to the total mana value of other
/// permanents you control to each opponent. ("Up to one" is modeled as a
/// required target, the codebase-wide convention for the printed shape.)
pub fn summon_bahamut() -> CardDefinition {
    let destroy_nonland = || Effect::Destroy {
        what: target_filtered(
            SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
        ),
    };
    CardDefinition {
        name: "Summon: Bahamut",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        keywords: vec![Keyword::Flying],
        saga_chapters: vec![
            (1, destroy_nonland()),
            (2, destroy_nonland()),
            (3, Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            (4, Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::TotalManaValueOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::OtherThanSource),
                ))),
            }),
        ],
        ..Default::default()
    }
}

/// Ether — {3}{U} Artifact. {T}, Exile this artifact: Add {U}. When you next
/// cast an instant or sorcery spell this turn, copy that spell (you may choose
/// new targets for the copy).
pub fn ether() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ether",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                crate::effect::shortcut::add_mana(vec![Color::Blue]),
                Effect::OnYourNextSpellCastThisTurn {
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: SelectionRequirement::HasCardType(CardType::Instant)
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        },
                        then: Box::new(Effect::CopySpellMayChooseTargets {
                            what: Selector::TriggerSource,
                            count: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Summon: Fat Chocobo — {4}{G} Enchantment Creature — Saga Bird 4/4. I — create
/// a 2/2 green Bird token with a landfall +1/+0. II, III, IV — creatures you
/// control gain trample until end of turn.
pub fn summon_fat_chocobo() -> CardDefinition {
    let bird = TokenDefinition {
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
    };
    let trample = || Effect::GrantKeyword {
        what: Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        keyword: Keyword::Trample,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Summon: Fat Chocobo",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        saga_chapters: vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: bird }),
            (2, trample()),
            (3, trample()),
            (4, trample()),
        ],
        ..Default::default()
    }
}

/// Summon: G.F. Cerberus — {2}{R}{R} Enchantment Creature — Saga Dog 3/3.
/// I — Surveil 1. II — when you next cast an instant or sorcery this turn, copy
/// it (you may choose new targets). III — copy it twice.
pub fn summon_gf_cerberus() -> CardDefinition {
    let copy_next = |count: i32| Effect::OnYourNextSpellCastThisTurn {
        body: Box::new(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
            then: Box::new(Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::Const(count),
            }),
            else_: Box::new(Effect::Noop),
        }),
    };
    CardDefinition {
        name: "Summon: G.F. Cerberus",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        saga_chapters: vec![
            (1, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }),
            (2, copy_next(1)),
            (3, copy_next(2)),
        ],
        ..Default::default()
    }
}

/// Summon: Esper Ramuh — {2}{R}{R} Enchantment Creature — Saga Wizard 3/3.
/// I — Judgment Bolt — deals damage equal to the noncreature, nonland cards in
/// your graveyard to target creature an opponent controls. II, III — Wizards you
/// control get +1/+0 until end of turn.
pub fn summon_esper_ramuh() -> CardDefinition {
    let wizards_pump = || Effect::PumpPT {
        what: Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Wizard)
                .and(SelectionRequirement::ControlledByYou),
        ),
        power: Value::ONE,
        toughness: Value::ZERO,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Summon: Esper Ramuh",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        saga_chapters: vec![
            (1, Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Noncreature.and(SelectionRequirement::Nonland),
                }),
            }),
            (2, wizards_pump()),
            (3, wizards_pump()),
        ],
        ..Default::default()
    }
}

/// Summon: G.F. Ifrit — {2}{R} Enchantment Creature — Saga Demon 3/2. I, II — you
/// may discard a card; if you do, draw a card. III, IV — add {R}. Sacrificed
/// after IV.
pub fn summon_gf_ifrit() -> CardDefinition {
    let loot = || Effect::MayDiscard {
        description: "Discard a card to draw a card".into(),
        count: Value::ONE,
        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: None,
    };
    let add_r = || crate::effect::shortcut::add_mana(vec![Color::Red]);
    CardDefinition {
        name: "Summon: G.F. Ifrit",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        saga_chapters: vec![(1, loot()), (2, loot()), (3, add_r()), (4, add_r())],
        ..Default::default()
    }
}

/// Summon: Anima — {4}{B}{B} Enchantment Creature — Saga Horror 4/4 with menace.
/// I, II, III — Pain — you draw a card and lose 1 life. IV — Oblivion — each
/// opponent sacrifices a creature of their choice and loses 3 life.
pub fn summon_anima() -> CardDefinition {
    let pain = || Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
    ]);
    CardDefinition {
        name: "Summon: Anima",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        saga_chapters: vec![
            (1, pain()),
            (2, pain()),
            (3, pain()),
            (4, Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: SelectionRequirement::Creature,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
            ])),
        ],
        ..Default::default()
    }
}

/// Haste Magic — {1}{R} Instant. Target creature gets +3/+1 and gains haste
/// until end of turn. Exile the top card of your library; you may play it until
/// end of turn. (The "until your next end step" window is modeled as end of the
/// current turn — the intended attack-this-turn use.)
pub fn haste_magic() -> CardDefinition {
    CardDefinition {
        name: "Haste Magic",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
        ]),
        ..Default::default()
    }
}

/// Delivery Moogle — {3}{W} 3/2 Moogle with flying. When it enters, search your
/// library and/or graveyard for an artifact card with mana value 2 or less and
/// put it into your hand (shuffle if you searched your library).
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
            filter: SelectionRequirement::Artifact.and(SelectionRequirement::ManaValueAtMost(2)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Aettir and Priwen — {6} Legendary Equipment. Equipped creature has base power
/// and toughness X/X, where X is your life total. Equip {5}.
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
            set_base_pt_controller_life: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Summon: Shiva — {3}{U}{U} Enchantment Creature — Saga Elemental 4/5.
/// I, II — tap target creature an opponent controls and put a stun counter on
/// it. III — draw a card for each tapped creature your opponents control.
pub fn summon_shiva() -> CardDefinition {
    let heavenly_strike = || Effect::Seq(vec![
        Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        },
        Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
    ]);
    CardDefinition {
        name: "Summon: Shiva",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        saga_chapters: vec![
            (1, heavenly_strike()),
            (2, heavenly_strike()),
            (3, Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Tapped)
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
            }),
        ],
        ..Default::default()
    }
}

/// Summon: Titan — {3}{G}{G} Enchantment Creature — Saga Giant 7/7, reach and
/// trample. I — mill five. II — return all land cards from your graveyard to
/// the battlefield tapped. III — another target creature you control gains
/// trample and gets +X/+X, where X is the number of lands you control.
pub fn summon_titan() -> CardDefinition {
    let lands_you_control = || Value::count(Selector::EachPermanent(
        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
    ));
    CardDefinition {
        name: "Summon: Titan",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        saga_chapters: vec![
            (1, Effect::Mill { who: Selector::You, amount: Value::Const(5) }),
            (2, Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Land,
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            (3, Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: lands_you_control(),
                    toughness: lands_you_control(),
                    duration: Duration::EndOfTurn,
                },
            ])),
        ],
        ..Default::default()
    }
}

/// Summon: Primal Garuda — {3}{W} Enchantment Creature — Saga Harpy 3/3, flying.
/// I — deal 4 damage to target tapped creature an opponent controls. II, III —
/// another target creature you control gets +1/+0 and gains flying until end of
/// turn.
pub fn summon_primal_garuda() -> CardDefinition {
    let slipstream = || Effect::Seq(vec![
        Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            power: Value::ONE,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        Effect::GrantKeyword {
            what: Selector::Target(0),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
    ]);
    CardDefinition {
        name: "Summon: Primal Garuda",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Harpy],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        saga_chapters: vec![
            (1, Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Tapped)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(4),
            }),
            (2, slipstream()),
            (3, slipstream()),
        ],
        ..Default::default()
    }
}

/// Summon: Primal Odin — {4}{B}{B} Enchantment Creature — Saga Knight 5/3.
/// I — destroy target creature an opponent controls. II — gains "Whenever this
/// creature deals combat damage to a player, that player loses the game." III —
/// draw two cards; each player loses 2 life.
pub fn summon_primal_odin() -> CardDefinition {
    let zantetsuken = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::LoseGame { who: PlayerRef::Target(0) },
    };
    CardDefinition {
        name: "Summon: Primal Odin",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        saga_chapters: vec![
            (1, Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            }),
            (2, Effect::GrantTriggeredAbility {
                what: Selector::This,
                trigger: Box::new(zantetsuken),
                duration: Duration::Permanent,
            }),
            (3, Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
            ])),
        ],
        ..Default::default()
    }
}

/// Weapons Vendor — {3}{W} 2/2 Human Artificer. When it enters, draw a card. At
/// the beginning of combat on your turn, if you control an Equipment, you may
/// pay {1}. When you do, attach target Equipment you control to target creature
/// you control.
pub fn weapons_vendor() -> CardDefinition {
    CardDefinition {
        name: "Weapons Vendor",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::ONE,
                }),
                effect: Effect::MayPay {
                    description: "Pay {1} to attach an Equipment you control".into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::Attach {
                            what: Selector::TargetFiltered {
                                slot: 0,
                                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                                    .and(SelectionRequirement::ControlledByYou),
                            },
                            to: Selector::TargetFiltered {
                                slot: 1,
                                filter: SelectionRequirement::Creature
                                    .and(SelectionRequirement::ControlledByYou),
                            },
                        }),
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Fire Magic — {R} Instant. Tiered (choose one additional cost): Fire {0} —
/// 1 damage to each creature; Fira {2} — 2 damage; Firaga {5} — 3 damage.
pub fn fire_magic() -> CardDefinition {
    use crate::effect::SpreeMode;
    let dmg_each = |n: i32| Effect::ForEach {
        selector: Selector::EachPermanent(SelectionRequirement::Creature),
        body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(n) }),
    };
    CardDefinition {
        name: "Fire Magic",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode { cost: cost(&[]), effect: dmg_each(1) },
                SpreeMode { cost: cost(&[generic(2)]), effect: dmg_each(2) },
                SpreeMode { cost: cost(&[generic(5)]), effect: dmg_each(3) },
            ],
        },
        ..Default::default()
    }
}

/// Thunder Magic — {R} Instant. Tiered: Thunder {0} — 2 damage to target
/// creature; Thundara {3} — 4 damage; Thundaga {5}{R} — 8 damage.
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

/// Ice Magic — {1}{U} Instant. Tiered: Blizzard {0} — return target creature to
/// its owner's hand; Blizzara {2} — its owner puts it on top or bottom of their
/// library; Blizzaga {5}{U} — its owner shuffles it into their library.
pub fn ice_magic() -> CardDefinition {
    use crate::effect::{LibraryPosition, SpreeMode};
    let bounce_to = |pos: Option<LibraryPosition>| Effect::Move {
        what: target_filtered(SelectionRequirement::Creature),
        to: match pos {
            None => ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            Some(p) => ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: p },
        },
    };
    CardDefinition {
        name: "Ice Magic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode { cost: cost(&[]), effect: bounce_to(None) },
                SpreeMode {
                    cost: cost(&[generic(2)]),
                    effect: bounce_to(Some(LibraryPosition::OwnerChoice)),
                },
                SpreeMode {
                    cost: cost(&[generic(5), u()]),
                    effect: bounce_to(Some(LibraryPosition::Shuffled)),
                },
            ],
        },
        ..Default::default()
    }
}

/// Restoration Magic — {W} Instant. Tiered: Cure {0} — target permanent gains
/// hexproof and indestructible until end of turn; Cura {1} — same, gain 3 life;
/// Curaga {3}{W} — permanents you control gain hexproof and indestructible,
/// gain 6 life.
pub fn restoration_magic() -> CardDefinition {
    use crate::effect::SpreeMode;
    let protect = |what: Selector| Effect::Seq(vec![
        Effect::GrantKeyword { what: what.clone(), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
        Effect::GrantKeyword { what, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
    ]);
    CardDefinition {
        name: "Restoration Magic",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode {
                    cost: cost(&[]),
                    effect: protect(target_filtered(SelectionRequirement::Permanent)),
                },
                SpreeMode {
                    cost: cost(&[generic(1)]),
                    effect: Effect::Seq(vec![
                        protect(target_filtered(SelectionRequirement::Permanent)),
                        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    ]),
                },
                SpreeMode {
                    cost: cost(&[generic(3), w()]),
                    effect: Effect::Seq(vec![
                        protect(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                        Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
                    ]),
                },
            ],
        },
        ..Default::default()
    }
}

/// Warrior's Sword — {3}{R} Equipment. Job select. Equipped creature gets +3/+2
/// and is a Warrior in addition to its other types. Equip {5}.
pub fn warriors_sword() -> CardDefinition {
    CardDefinition {
        name: "Warrior's Sword",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 2,
            add_creature_types: vec![CreatureType::Warrior],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Thief's Knife — {2}{U} Equipment. Job select. Equipped creature gets +1/+1,
/// draws a card on combat damage to a player, and is a Rogue. Equip {4}.
pub fn thiefs_knife() -> CardDefinition {
    CardDefinition {
        name: "Thief's Knife",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            add_creature_types: vec![CreatureType::Rogue],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Suplex — {1}{R} Sorcery. Choose one — deal 3 to target creature and exile it
/// if it would die this turn; or exile target artifact.
pub fn suplex() -> CardDefinition {
    CardDefinition {
        name: "Suplex",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                // Install the exile-instead replacement before the damage lands.
                Effect::ExileIfWouldDieThisTurn {
                    what: target_filtered(SelectionRequirement::Creature),
                },
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
            ]),
            Effect::Exile { what: target_filtered(SelectionRequirement::Artifact) },
        ]),
        ..Default::default()
    }
}

/// Tifa's Limit Break — {G} Instant. Tiered: Somersault {0} — +2/+2; Meteor
/// Strikes {2} — double target creature's power and toughness; Final Heaven
/// {6}{G} — triple them. All until end of turn.
pub fn tifas_limit_break() -> CardDefinition {
    use crate::effect::SpreeMode;
    let pump = |p: Value, t: Value| Effect::PumpPT {
        what: target_filtered(SelectionRequirement::Creature),
        power: p,
        toughness: t,
        duration: Duration::EndOfTurn,
    };
    let pow = || Value::PowerOf(Box::new(Selector::Target(0)));
    let tou = || Value::ToughnessOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Tifa's Limit Break",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                SpreeMode { cost: cost(&[]), effect: pump(Value::Const(2), Value::Const(2)) },
                // Double = add its current P/T.
                SpreeMode { cost: cost(&[generic(2)]), effect: pump(pow(), tou()) },
                // Triple = add twice its current P/T.
                SpreeMode {
                    cost: cost(&[generic(6), g()]),
                    effect: pump(Value::Times(Box::new(pow()), Box::new(Value::Const(2))),
                                 Value::Times(Box::new(tou()), Box::new(Value::Const(2)))),
                },
            ],
        },
        ..Default::default()
    }
}

/// Swallowed by Leviathan — {2}{U} Instant. Surveil 2, then counter target spell
/// unless its controller pays {1} for each card in your graveyard.
pub fn swallowed_by_leviathan() -> CardDefinition {
    CardDefinition {
        name: "Swallowed by Leviathan",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[]),
                exile: false,
                extra_generic: Some(Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                })),
            },
        ]),
        ..Default::default()
    }
}

/// Zodiark, Umbral God — {B}{B}{B}{B}{B} 5/5 Legendary God with indestructible.
/// ETB: each player sacrifices half the non-God creatures they control, rounded
/// down. Whenever a player sacrifices a creature, put a +1/+1 counter on it.
pub fn zodiark_umbral_god() -> CardDefinition {
    CardDefinition {
        name: "Zodiark, Umbral God",
        cost: cost(&[b(), b(), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::God], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            etb(Effect::SacrificeHalf {
                who: Selector::Player(PlayerRef::EachPlayer),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::God).negate()),
                rounded_up: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::AnyPlayer),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Phantom Train — {3}{B} Vehicle 4/4 with trample. Sacrifice another artifact
/// or creature: put a +1/+1 counter on it and it becomes a Spirit artifact
/// creature until end of turn.
pub fn phantom_train() -> CardDefinition {
    CardDefinition {
        name: "Phantom Train",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Spirit],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
            ]),
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                1,
            )),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stuck in Summoner's Sanctum — {2}{U} Aura with flash. Enchant artifact or
/// creature. ETB taps it. Enchanted permanent doesn't untap and its activated
/// abilities can't be activated.
pub fn stuck_in_summoners_sanctum() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Stuck in Summoner's Sanctum",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Artifact.or(SelectionRequirement::Creature)),
        },
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted permanent doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                },
            },
            StaticAbility {
                description: "Enchanted permanent's activated abilities can't be activated.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    keyword: Keyword::CantActivateAbilities,
                },
            },
        ],
        ..Default::default()
    }
}

/// Buster Sword — {3} Equipment. Equipped creature gets +3/+2 and draws a card
/// when it deals combat damage to a player. Equip {2}.
/// (The "then cast a spell with mana value ≤ that damage for free" rider is
/// dropped — no free-cast-from-hand-by-mana-value primitive yet.)
pub fn buster_sword() -> CardDefinition {
    CardDefinition {
        name: "Buster Sword",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Absolute Virtue — {6}{W}{U} 8/8 Legendary Avatar Warrior. Can't be countered,
/// flying, and you have protection from each of your opponents.
/// (Protection is modeled as controller hexproof — the "can't be targeted by
/// opponents" half; the damage-prevention half is approximated.)
pub fn absolute_virtue() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Absolute Virtue",
        cost: cost(&[generic(6), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar, CreatureType::Warrior],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::CantBeCountered, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You have protection from each of your opponents.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        ..Default::default()
    }
}

/// The Masamune — {3} Legendary Equipment. While the equipped creature is
/// attacking it has first strike and must be blocked if able. Equip {2}.
/// ("While attacking" is modeled as "during your turn"; the death-trigger
/// doubler rider is dropped.)
pub fn the_masamune() -> CardDefinition {
    CardDefinition {
        name: "The Masamune",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            during_your_turn_keywords: vec![Keyword::FirstStrike, Keyword::MustBeBlocked],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Dark Knight's Greatsword — {2}{B} Equipment. Job select. Equipped creature
/// gets +3/+0 and is a Knight in addition to its other types. Equip {3}.
/// (The printed "Equip—Pay 3 life, once each turn" is approximated as a {3}
/// generic equip.)
pub fn dark_knights_greatsword() -> CardDefinition {
    CardDefinition {
        name: "Dark Knight's Greatsword",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 0,
            add_creature_types: vec![CreatureType::Knight],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Summoner's Grimoire — {3}{G} Book Equipment. Job select. Equipped creature is
/// a Shaman and, when it attacks, you may put a creature card from your hand
/// onto the battlefield. Equip {3}.
/// (The "if it's an enchantment card it enters tapped and attacking" rider is
/// approximated — the entrant simply enters under your control.)
pub fn summoners_grimoire() -> CardDefinition {
    CardDefinition {
        name: "Summoner's Grimoire",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment, ArtifactSubtype::Book],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![job_select_etb()],
        equipped_bonus: Some(EquipBonus {
            add_creature_types: vec![CreatureType::Shaman],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// The Water Crystal — {2}{U}{U} Legendary Artifact. Blue spells you cast cost
/// {1} less. Opponents mill extra. {4}{U}{U}, {T}: each opponent mills cards
/// equal to the number of cards in your hand.
/// (The printed "mill that many plus four" is approximated as mill-doubling.)
pub fn the_water_crystal() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "The Water Crystal",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Blue spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::HasColor(Color::Blue),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "If an opponent would mill one or more cards, they mill extra.",
                effect: StaticEffect::OpponentMillDoubled,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4), u(), u()]),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Hand,
                    filter: SelectionRequirement::Any,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Wandering Minstrel — {G}{U} 1/3 Legendary Human Bard. Lands you control
/// enter untapped. At the beginning of combat on your turn, if you control five
/// or more Towns, create a 2/2 all-colors Elemental token. {3}{W}{U}{B}{R}{G}:
/// other creatures you control get +X/+X, where X is the number of Towns you
/// control.
pub fn the_wandering_minstrel() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let towns = || Value::count(Selector::EachPermanent(
        SelectionRequirement::HasLandType(crate::card::LandType::Town)
            .and(SelectionRequirement::ControlledByYou),
    ));
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        colors: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        ..Default::default()
    };
    CardDefinition {
        name: "The Wandering Minstrel",
        cost: cost(&[g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Bard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Lands you control enter untapped.",
            effect: StaticEffect::LandsEnterUntapped,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasLandType(crate::card::LandType::Town)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(5),
                }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elemental },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), u(), b(), r(), g()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: towns(),
                toughness: towns(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nibelheim Aflame — {2}{R}{R} sorcery with Flashback {5}{R}{R}.
/// Target creature you control deals damage equal to its power to each other
/// creature; if cast from a graveyard, discard your hand and draw four.
pub fn nibelheim_aflame() -> CardDefinition {
    CardDefinition {
        name: "Nibelheim Aflame",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r(), r()]))],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPowerToEach {
                source: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                targets: Selector::EachPermanent(SelectionRequirement::Creature),
                each_opponent: false,
            },
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(4) },
                ])),
                else_: Box::new(Effect::Seq(vec![])),
            },
        ]),
        ..Default::default()
    }
}

/// Ignis Scientia — {1}{G}{U} 2/2 Legendary Human Advisor. ETB: dig for a land
/// (top six, put a land onto the battlefield tapped, bottom the rest).
/// {1}{G}{U}, {T}: exile target card from a graveyard; if a creature card was
/// exiled this way, create a Food token.
pub fn ignis_scientia() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ignis Scientia",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::DigForLandToBattlefield { count: Value::Const(6) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), g(), u()]),
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::InGraveyard,
                    },
                },
                Effect::If {
                    cond: Predicate::EntityMatchesAny {
                        what: Selector::Target(0),
                        filter: SelectionRequirement::Creature,
                    },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: crabomination_base::tokens::food_token(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Genji Glove — {5} Equipment. Equipped creature has double strike. Whenever
/// it attacks, if it's the first combat phase of the turn, untap it and add an
/// additional combat phase after this one. Equip {3}.
pub fn genji_glove() -> CardDefinition {
    CardDefinition {
        name: "Genji Glove",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::IsFirstCombatPhaseThisTurn,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ultima — {3}{W}{W} sorcery. Destroy all artifacts and creatures, then end
/// the turn (CR 728).
pub fn ultima() -> CardDefinition {
    CardDefinition {
        name: "Ultima",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
            },
            Effect::EndTheTurn,
        ]),
        ..Default::default()
    }
}

/// Summon: Knights of Round — {6}{W}{W} Saga Knight 3/3, indestructible.
/// I–IV: create three 2/2 white Knight tokens. V (Ultimate End): other creatures
/// you control get +2/+2 and gain an indestructible counter.
pub fn summon_knights_of_round() -> CardDefinition {
    let knight = || TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    let make_three = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(3),
        definition: knight(),
    };
    let others = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        )
    };
    CardDefinition {
        name: "Summon: Knights of Round",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Indestructible],
        saga_chapters: vec![
            (1, make_three()),
            (2, make_three()),
            (3, make_three()),
            (4, make_three()),
            (5, Effect::Seq(vec![
                Effect::PumpPT {
                    what: others(),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCounter {
                    what: others(),
                    kind: CounterType::Indestructible,
                    amount: Value::ONE,
                },
            ])),
        ],
        ..Default::default()
    }
}

/// The Lunar Whale — {3}{U} Legendary Vehicle 3/5, flying, crew 1. Whenever it
/// attacks you may play the top card of your library for the rest of the turn.
/// (The always-on "look at your top card" is cosmetic and omitted.)
pub fn the_lunar_whale() -> CardDefinition {
    CardDefinition {
        name: "The Lunar Whale",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantPlayFromTopThisTurn,
        }],
        ..Default::default()
    }
}

/// Tellah, Great Sage — {3}{U}{R} 3/3 Human Wizard. Whenever you cast a
/// noncreature spell, create a 1/1 Hero. If 4+ mana was spent, draw two; if 8+
/// mana was spent, sacrifice Tellah and it deals that much to each opponent.
pub fn tellah_great_sage() -> CardDefinition {
    CardDefinition {
        name: "Tellah, Great Sage",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: hero_token(),
                },
                Effect::If {
                    cond: Predicate::CastSpellManaSpentAtLeast(4),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::CastSpellManaSpentAtLeast(8),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::CastSpellManaSpent,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Ragnarok, Divine Deliverance — the {0} meld back-face, 7/6 Legendary Beast
/// Avatar with vigilance, menace, trample, reach, and haste. When it dies,
/// destroy target permanent and return a target nonlegendary permanent card
/// from your graveyard to the battlefield.
pub fn ragnarok_divine_deliverance() -> CardDefinition {
    CardDefinition {
        name: "Ragnarok, Divine Deliverance",
        cost: cost(&[]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Avatar],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Menace,
            Keyword::Trample,
            Keyword::Reach,
            Keyword::Haste,
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(SelectionRequirement::Permanent) },
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: SelectionRequirement::InYourGraveyard
                            .and(SelectionRequirement::PermanentCard)
                            .and(SelectionRequirement::Not(Box::new(
                                SelectionRequirement::HasSupertype(Supertype::Legendary),
                            ))),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Omega, Heartless Evolution — {5}{G}{U} 8/8 Legendary Artifact Robot. Wave
/// Cannon ETB: tap up to one target nonland permanent an opponent controls, put
/// X stun counters on it, and gain X life, where X is the number of nonbasic
/// lands you control. (Multiplayer "for each opponent" is one target — the
/// codebase convention.)
pub fn omega_heartless_evolution() -> CardDefinition {
    let x = || Value::NonbasicLandCountControlledBy(PlayerRef::You);
    CardDefinition {
        name: "Omega, Heartless Evolution",
        cost: cost(&[generic(5), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: x(),
            },
            Effect::GainLife { who: Selector::You, amount: x() },
        ]))],
        ..Default::default()
    }
}

/// Y'shtola Rhul — {4}{U}{U} 3/5 Legendary Cat Druid. At the beginning of your
/// end step, blink target creature you control (exile, then return it). If it's
/// the first end step of the turn, there is an additional end step after this
/// one (CR 500.7).
pub fn yshtola_rhul() -> CardDefinition {
    CardDefinition {
        name: "Y'shtola Rhul",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::If {
                    cond: Predicate::IsFirstEndStepThisTurn,
                    then: Box::new(Effect::AdditionalEndStep { count: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Beatrix, Loyal General — {4}{W}{W} 4/4 Legendary Human Soldier, vigilance.
/// At the beginning of combat on your turn, attach any number of Equipment you
/// control to target creature you control (modeled as "attach all of them").
pub fn beatrix_loyal_general() -> CardDefinition {
    CardDefinition {
        name: "Beatrix, Loyal General",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Attach {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Kain, Traitorous Dragoon — {2}{B} 2/4 Legendary Human Knight. Jump (flying
/// during your turn). Whenever Kain deals combat damage to a player, that player
/// gains control of Kain; you then draw that many cards, make that many tapped
/// Treasures, and lose that much life.
pub fn kain_traitorous_dragoon() -> CardDefinition {
    let treasure = || {
        let mut t = crabomination_base::tokens::treasure_token();
        t.tapped = true;
        t
    };
    CardDefinition {
        name: "Kain, Traitorous Dragoon",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Jump — During your turn, Kain has flying.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Flying,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::Target(0)),
                    duration: Duration::Permanent,
                },
                Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: treasure(),
                },
                Effect::LoseLife { who: Selector::You, amount: Value::TriggerEventAmount },
            ]),
        }],
        ..Default::default()
    }
}

/// Cecil, Dark Knight // Cecil, Redeemed Paladin — {B} 2/3 Human Knight,
/// deathtouch (transform DFC). Whenever Cecil deals combat damage to a player,
/// you lose that much life; then if your life ≤ half your starting life, untap
/// Cecil and transform it. Back: 4/4 lifelink; when it attacks, other attacking
/// creatures you control gain indestructible until end of turn.
/// (Approximation: the front's "deals damage" trigger fires only on combat
/// damage to a player, the dominant play pattern.)
pub fn cecil_dark_knight() -> CardDefinition {
    let paladin = CardDefinition {
        name: "Cecil, Redeemed Paladin",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsAttacking
                        .and(SelectionRequirement::OtherThanSource),
                },
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Cecil, Dark Knight",
        cost: cost(&[b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::TriggerEventAmount },
                Effect::If {
                    cond: Predicate::PlayerLifeAtMostHalfStarting { who: PlayerRef::You },
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::Transform { what: Selector::This },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(paladin)),
        ..Default::default()
    }
}

/// Galuf's Final Act — {1}{G} instant. Until end of turn, target creature gets
/// +1/+0 and gains "When this creature dies, put a number of +1/+1 counters
/// equal to its power on up to one target creature."
pub fn galufs_final_act() -> CardDefinition {
    CardDefinition {
        name: "Galuf's Final Act",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Clash of the Eikons — {G} sorcery. Choose one or more — target creature you
/// control fights target creature an opponent controls; remove a lore counter
/// from target Saga you control; put a lore counter on target Saga you control.
pub fn clash_of_the_eikons() -> CardDefinition {
    CardDefinition {
        name: "Clash of the Eikons",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![1, 2, 3],
            modes: vec![
                Effect::Fight {
                    attacker: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    },
                    defender: Selector::TargetFiltered {
                        slot: 1,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    },
                },
                Effect::RemoveCounter {
                    what: target_filtered(
                        SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Saga)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::Lore,
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Saga)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::Lore,
                    amount: Value::ONE,
                },
            ],
        },
        ..Default::default()
    }
}

/// Louisoix's Sacrifice — {U} instant. As an additional cost, sacrifice a
/// legendary creature or pay {2}. Counter target noncreature spell.
/// (Approximation: the "counter an activated or triggered ability" halves are
/// dropped — the noncreature-spell counter is the dominant mode.)
pub fn louisoixs_sacrifice() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Louisoix's Sacrifice",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay {
            filter: SelectionRequirement::HasSupertype(Supertype::Legendary)
                .and(SelectionRequirement::Creature),
            pay: 2,
        }],
        effect: Effect::CounterSpell {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::Noncreature),
            },
        },
        ..Default::default()
    }
}

/// Kefka, Court Mage // Kefka, Ruler of Ruin — {2}{U}{B}{R} 4/5 Human Wizard
/// (transform DFC). When Kefka enters or attacks, each player discards a card,
/// then you draw a card. {8}: Each opponent sacrifices a permanent, then
/// transform Kefka (sorcery-speed). Back: 5/7 flying; whenever an opponent
/// loses life during your turn, you draw that many cards.
/// (Approximation: the front's "draw a card for each card type among the
/// discarded cards" is modeled as a flat draw of one.)
pub fn kefka_court_mage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let discard_draw = || {
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
                random: false,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ])
    };
    let ruler = CardDefinition {
        name: "Kefka, Ruler of Ruin",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                filter: Some(Predicate::IsTurnOf(PlayerRef::You)),
                ..EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl)
            },
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Kefka, Court Mage",
        cost: cost(&[generic(2), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: discard_draw(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: discard_draw(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: SelectionRequirement::Permanent,
                },
                Effect::Transform { what: Selector::This },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(ruler)),
        ..Default::default()
    }
}

/// Eden, Seat of the Sanctum — Land — Town. {T}: Add {C}. {5}, {T}: Mill two
/// cards. Then you may sacrifice Eden; when you do, return another target
/// permanent card from your graveyard to your hand.
pub fn eden_seat_of_the_sanctum() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    CardDefinition {
        name: "Eden, Seat of the Sanctum",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { land_types: vec![LandType::Town], ..Default::default() },
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(5)]),
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) },
                    Effect::MaySacrificeSource {
                        description: "sacrifice Eden, Seat of the Sanctum".into(),
                        then: Box::new(Effect::Reflexive {
                            body: Box::new(Effect::Move {
                                what: target_filtered(
                                    SelectionRequirement::PermanentCard
                                        .and(SelectionRequirement::InYourGraveyard)
                                        .and(SelectionRequirement::OtherThanSource),
                                ),
                                to: ZoneDest::Hand(PlayerRef::You),
                            }),
                        }),
                        else_: None,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Reno and Rude — {1}{B} 2/1 Human Assassin with menace. Whenever it deals
/// combat damage to a player, you may sacrifice another creature or artifact;
/// if you do, exile the top card of that player's library and you may play it
/// this turn, spending mana of any type to cast it.
/// (Approximation: the printed order exiles first, then gates play on the
/// sacrifice; here the exile+play grant is gated on the sacrifice together.)
pub fn reno_and_rude() -> CardDefinition {
    use crate::card::MayPlayDuration;
    CardDefinition {
        name: "Reno and Rude",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "sacrifice another creature or artifact".into(),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::ControlledByYou),
                count: Value::ONE,
                then: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::Target(0),
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: true,
                    uncast_penalty: None,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Torgal, A Fine Hound — {1}{G} 2/2 Wolf. Whenever you cast your first Human
/// creature spell each turn, that creature enters with an additional +1/+1
/// counter for each Dog and/or Wolf you control. {T}: Add one mana of any color.
pub fn torgal_a_fine_hound() -> CardDefinition {
    CardDefinition {
        name: "Torgal, A Fine Hound",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(SelectionRequirement::HasCreatureType(
                        CreatureType::Human,
                    )),
                )
            },
            // The rider lands on the very spell that triggered it: the trigger
            // resolves above the still-on-the-stack Human creature, so the
            // counter count (Dogs/Wolves out now) applies as that creature ETBs.
            effect: Effect::GrantNextCreatureSpellCounters {
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Dog)
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Wolf))
                        .and(SelectionRequirement::ControlledByYou),
                )),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Garnet, Princess of Alexandria — {G}{W} 2/2 Human Noble Cleric with lifelink.
/// Whenever Garnet attacks, remove a lore counter from each Saga you control and
/// put a +1/+1 counter on Garnet for each one removed.
/// (Approximation: the printed "any number" choice is modeled as all your Sagas
/// that carry a lore counter.)
pub fn garnet_princess_of_alexandria() -> CardDefinition {
    let sagas_with_lore = || {
        SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Saga)
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::WithCounter(CounterType::Lore))
    };
    CardDefinition {
        name: "Garnet, Princess of Alexandria",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::count(Selector::EachPermanent(sagas_with_lore())),
                },
                Effect::RemoveCounter {
                    what: Selector::EachPermanent(sagas_with_lore()),
                    kind: CounterType::Lore,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Summon: Fenrir — {2}{G} Enchantment Creature — Saga Wolf 3/2. I — search for
/// a basic land, put it onto the battlefield tapped. II — your next creature
/// spell this turn enters with an extra +1/+1 counter. III — draw a card if you
/// control the creature with the greatest power (or tied).
pub fn summon_fenrir() -> CardDefinition {
    CardDefinition {
        name: "Summon: Fenrir",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Wolf],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        saga_chapters: vec![
            (1, Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            (2, Effect::GrantNextCreatureSpellCounters {
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            (3, Effect::If {
                cond: Predicate::ControlsGreatestPowerCreature { who: PlayerRef::You },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..Default::default()
    }
}

/// Summon: Brynhildr — {1}{R} Enchantment Creature — Saga Knight 2/1.
/// I — Chain — exile the top card of your library; you may play it while it
/// remains exiled. II, III — Gestalt Mode — your next creature spell this turn
/// enters with haste.
/// (Approximation: chapter I's "during any turn you put a lore counter" play
/// window is modeled as the broader `WhileExiled`.)
pub fn summon_brynhildr() -> CardDefinition {
    use crate::card::MayPlayDuration;
    CardDefinition {
        name: "Summon: Brynhildr",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        saga_chapters: vec![
            (1, Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::WhileExiled,
                pay_any_color: false,
                uncast_penalty: None,
            }),
            (2, Effect::GrantNextCreatureSpellKeyword { keyword: Keyword::Haste }),
            (3, Effect::GrantNextCreatureSpellKeyword { keyword: Keyword::Haste }),
        ],
        ..Default::default()
    }
}

/// Stiltzkin, Moogle Merchant — {W} 1/2 Moogle with lifelink. {2}, {T}: Target
/// opponent gains control of another target permanent you control. You draw a
/// card. (The opponent recipient is auto-bound to the lone opponent; the
/// per-opponent choice is a multiplayer follow-up.)
pub fn stiltzkin_moogle_merchant() -> CardDefinition {
    CardDefinition {
        name: "Stiltzkin, Moogle Merchant",
        cost: cost(&[w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moogle], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Permanent
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    },
                    to: Some(PlayerRef::EachOpponent),
                    duration: Duration::Permanent,
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vincent's Limit Break — {1}{B} Instant. Tiered (choose one additional cost).
/// Until end of turn, target creature you control gains "When this creature
/// dies, return it to the battlefield tapped" and gets the chosen base P/T:
/// Galian Beast {0} 3/2, Death Gigas {1} 5/2, Hellmasker {3} 7/2.
pub fn vincents_limit_break() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::SpreeMode;
    let mode = |extra: crate::mana::ManaCost, p: i32, t: i32| {
        let tgt = || Selector::TargetFiltered {
            slot: 0,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        };
        SpreeMode {
            cost: extra,
            effect: Effect::Seq(vec![
                Effect::SetBasePT {
                    what: tgt(),
                    power: Value::Const(p),
                    toughness: Value::Const(t),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantTriggeredAbility {
                    what: tgt(),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                        effect: Effect::Move {
                            what: Selector::This,
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::OwnerOf(Box::new(Selector::This)),
                                tapped: true,
                            },
                        },
                    }),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }
    };
    CardDefinition {
        name: "Vincent's Limit Break",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Tiered {
            modes: vec![
                mode(cost(&[]), 3, 2),
                mode(cost(&[generic(1)]), 5, 2),
                mode(cost(&[generic(3)]), 7, 2),
            ],
        },
        ..Default::default()
    }
}


/// Vayne's Treachery — {1}{B} Instant. Kicker—Sacrifice an artifact or
/// creature. Target creature gets -2/-2 until end of turn; -6/-6 if kicked.
pub fn vaynes_treachery() -> CardDefinition {
    CardDefinition {
        name: "Vayne's Treachery",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        kicker_action_cost: Some(crate::card::AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }),
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
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

/// Chocobo Kick — {1}{G} Sorcery. Kicker—Return a land you control to its
/// owner's hand. Target creature you control deals damage equal to its power
/// (twice its power if kicked) to target creature an opponent controls.
pub fn chocobo_kick() -> CardDefinition {
    let bite = |mult: i32| Effect::DealDamage {
        to: Selector::TargetFiltered {
            slot: 1,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
        },
        amount: Value::Times(
            Box::new(Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            }))),
            Box::new(Value::Const(mult)),
        ),
    };
    CardDefinition {
        name: "Chocobo Kick",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        kicker_action_cost: Some(crate::card::AdditionalCastCost::ReturnToHand {
            filter: SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            count: 1,
        }),
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(bite(2)),
            else_: Box::new(bite(1)),
        },
        ..Default::default()
    }
}

/// Sin, Spira's Punishment — {4}{B}{G}{U} 7/7 Leviathan Avatar with flying.
/// Whenever Sin enters or attacks, exile a permanent card from your graveyard
/// at random, then create a tapped token copy of it; repeat if it was a land.
pub fn sin_spiras_punishment() -> CardDefinition {
    let loop_effect = || Effect::ExileRandomGraveyardCopyTapped { who: PlayerRef::You };
    CardDefinition {
        name: "Sin, Spira's Punishment",
        cost: cost(&[generic(4), b(), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leviathan, CreatureType::Avatar],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(loop_effect()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: loop_effect(),
            },
        ],
        ..Default::default()
    }
}

/// Noctis, Prince of Lucis — {1}{W}{U}{B} 4/3 Human Noble with lifelink. You
/// may cast artifact spells from your graveyard by paying 3 life in addition
/// to their other costs; they enter with a finality counter.
pub fn noctis_prince_of_lucis() -> CardDefinition {
    CardDefinition {
        name: "Noctis, Prince of Lucis",
        cost: cost(&[generic(1), w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "You may cast artifact spells from your graveyard by paying 3 life in addition to paying their other costs. They enter with a finality counter.",
            effect: StaticEffect::GraveyardCastWithLifeSurcharge {
                filter: SelectionRequirement::Artifact,
                life: 3,
            },
        }],
        ..Default::default()
    }
}

/// Vaan, Street Thief — {2}{R} 2/2 Human Scout. Whenever one or more Scouts,
/// Pirates, and/or Rogues you control deal combat damage to a player, exile
/// that player's library top; you may cast it this turn, else mint a Treasure.
/// Whenever you cast a spell you don't own, put a +1/+1 counter on each
/// Scout, Pirate, and Rogue you control.
pub fn vaan_street_thief() -> CardDefinition {
    use crate::card::MayPlayDuration;
    use crate::effect::shortcut::mint_treasures;
    let thief_types = || {
        SelectionRequirement::HasCreatureType(CreatureType::Scout)
            .or(SelectionRequirement::HasCreatureType(CreatureType::Pirate))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Rogue))
    };
    CardDefinition {
        name: "Vaan, Street Thief",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec {
                    once_per_turn: true,
                    ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: thief_types(),
                        })
                },
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::Target(0),
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    uncast_penalty: Some(Box::new(mint_treasures(1))),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(SelectionRequirement::Not(Box::new(
                        SelectionRequirement::OwnedByYou,
                    ))),
                ),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        thief_types().and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// The Darkness Crystal — {2}{B}{B} Legendary Artifact. Black spells cost {1}
/// less. Nontoken opponent creatures that would die are exiled instead (+2
/// life). {4}{B}{B}, {T}: return a creature card exiled with it to the
/// battlefield tapped under your control with two extra +1/+1 counters.
pub fn the_darkness_crystal() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "The Darkness Crystal",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Black spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::HasColor(Color::Black),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "If a nontoken creature an opponent controls would die, instead exile it and you gain 2 life.",
                effect: StaticEffect::ExileDyingOpponentCreatures {
                    when_you_do: Some(Box::new(Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    })),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ExiledWithSource),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Summon: Leviathan — {4}{U}{U} Enchantment Creature — Saga Leviathan 6/6,
/// ward {2}. I — return each creature that isn't a Kraken, Leviathan, Merfolk,
/// Octopus, or Serpent to its owner's hand. II, III — until end of turn,
/// whenever one of those types attacks, draw a card.
pub fn summon_leviathan() -> CardDefinition {
    let sea_types = || {
        SelectionRequirement::HasCreatureType(CreatureType::Kraken)
            .or(SelectionRequirement::HasCreatureType(CreatureType::Leviathan))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Merfolk))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Octopus))
            .or(SelectionRequirement::HasCreatureType(CreatureType::Serpent))
    };
    let chapter23 = || Effect::OnMatchingAttacksThisTurn {
        filter: sea_types(),
        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
    };
    CardDefinition {
        name: "Summon: Leviathan",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Leviathan],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        saga_chapters: vec![
            (1, Effect::Move {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Not(Box::new(sea_types()))),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
            (2, chapter23()),
            (3, chapter23()),
        ],
        ..Default::default()
    }
}
