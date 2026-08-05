//! Conspiracy (CNS) — the in-game half of the remaining set. Tests in
//! `classic_sets/cns`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EntersAsCopy, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, VoteOption, VoteTally, ZoneDest,
    shortcut::{dethrone, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn artifact_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, types, p, t)
    }
}

/// Bite of the Black Rose — will of the council: shrink them, or strip them.
pub fn bite_of_the_black_rose() -> CardDefinition {
    CardDefinition {
        name: "Bite of the Black Rose",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Vote {
            tally: VoteTally::Majority,
            options: vec![
                VoteOption::new(
                    "sickness",
                    Effect::PumpPT {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                        power: Value::Const(-2),
                        toughness: Value::Const(-2),
                        duration: Duration::EndOfTurn,
                    },
                ),
                VoteOption::new(
                    "psychosis",
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                        random: false,
                    },
                ),
            ],
        },
        ..Default::default()
    }
}

/// Brago's Representative — a second vote on every ballot.
pub fn bragos_representative() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While voting, you get an additional vote.",
            effect: StaticEffect::AdditionalVotes(1),
        }],
        ..creature(
            "Brago's Representative",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            4,
        )
    }
}

/// Council Guardian — will of the council: protection from every winning color.
pub fn council_guardian() -> CardDefinition {
    let ballot = |label: &str, color: Color| {
        VoteOption::new(
            label,
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Protection(color),
                duration: Duration::Permanent,
            },
        )
    };
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Vote {
            tally: VoteTally::AllTied,
            options: vec![
                ballot("blue", Color::Blue),
                ballot("black", Color::Black),
                ballot("red", Color::Red),
                ballot("green", Color::Green),
            ],
        })],
        ..creature(
            "Council Guardian",
            cost(&[generic(5), w()]),
            vec![CreatureType::Giant, CreatureType::Soldier],
            5,
            5,
        )
    }
}

/// Custodi Squire — will of the council over your own graveyard.
pub fn custodi_squire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::WillOfTheCouncilOnCards {
                candidates: Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::Artifact.or(R::Creature).or(R::Enchantment),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )],
        ..creature(
            "Custodi Squire",
            cost(&[generic(4), w()]),
            vec![CreatureType::Spirit, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Dack Fayden — loot, steal an artifact, then steal everything you target.
pub fn dack_fayden() -> CardDefinition {
    CardDefinition {
        name: "Dack Fayden",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Planeswalker],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Dack],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(2),
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(2),
                        random: false,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::GainControl {
                    what: target_filtered(R::Artifact),
                    to: None,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Dack Fayden".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                        effect: Effect::GainControl {
                            what: Selector::AllCastSpellTargets,
                            to: None,
                            duration: Duration::Permanent,
                        },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dack's Duplicate — a clone that comes in hasty and hungry for the crown.
pub fn dacks_duplicate() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_keywords: vec![Keyword::Haste],
            extra_triggered: vec![dethrone()],
            ..Default::default()
        }),
        ..creature(
            "Dack's Duplicate",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Extract from Darkness — mill everyone, then reanimate the best body.
pub fn extract_from_darkness() -> CardDefinition {
    CardDefinition {
        name: "Extract from Darkness",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                        filter: R::Creature,
                    }),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]),
        ..Default::default()
    }
}

/// Flamewright — mints Constructs and fires them off one at a time.
pub fn flamewright() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Construct".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        keywords: vec![Keyword::Defender],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Construct],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::Creature.and(R::HasKeyword(Keyword::Defender)), 1)),
                effect: Effect::DealDamage {
                    to: crate::effect::shortcut::target_any(),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Flamewright",
            cost(&[r(), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Grenzo, Dungeon Warden — digs up creatures from the bottom of the library.
pub fn grenzo_dungeon_warden() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BottomCardToGraveyardThenDeploy {
                max_power: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..creature(
            "Grenzo, Dungeon Warden",
            cost(&[x(), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Grenzo's Rebuttal — an Ogre, and everyone strips their left-hand neighbor.
pub fn grenzos_rebuttal() -> CardDefinition {
    CardDefinition {
        name: "Grenzo's Rebuttal",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Ogre".into(),
                    power: 4,
                    toughness: 4,
                    colors: vec![Color::Red],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Ogre],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::EachPlayerDestroysChosenFromLeftNeighbor {
                filters: vec![R::Artifact, R::Creature, R::Land],
            },
        ]),
        ..Default::default()
    }
}

/// Grudge Keeper — punishes everyone who voted against you.
pub fn grudge_keeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::VotingFinished, EventScope::AnyPlayer),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::OpponentsWhoVotedDifferently),
                amount: Value::Const(2),
            },
        }],
        ..creature(
            "Grudge Keeper",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Ignition Team — grows off tapped lands and turns one into a 4/4.
pub fn ignition_team() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::EachPermanent(R::Land.and(R::Tapped)))),
        )),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Land),
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![CreatureType::Elemental],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Ignition Team",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            0,
            0,
        )
    }
}

/// Magister of Worth — will of the council: mass reanimation or a wrath.
pub fn magister_of_worth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Vote {
            tally: VoteTally::Majority,
            options: vec![
                VoteOption::new(
                    "grace",
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                            filter: R::Creature,
                        },
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOfMoved,
                            tapped: false,
                        },
                    },
                ),
                VoteOption::new(
                    "condemnation",
                    Effect::Destroy {
                        what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    },
                ),
            ],
        })],
        ..creature(
            "Magister of Worth",
            cost(&[generic(4), w(), b()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Marchesa, the Black Rose — dethrone for the team, and counters come back.
pub fn marchesa_the_black_rose() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have dethrone.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ability: Box::new(dethrone()),
            },
        }],
        triggered_abilities: vec![dethrone(), TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::WithCounter(CounterType::PlusOnePlusOne),
                }),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Marchesa, the Black Rose",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Marchesa's Smuggler — dethrone, and it slips a creature through.
pub fn marchesas_smuggler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dethrone()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), r()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Marchesa's Smuggler",
            cost(&[u(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Muzzio, Visionary Architect — digs as deep as your biggest artifact.
pub fn muzzio_visionary_architect() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::LookPickToHand {
                then_if_picked: None,
                who: PlayerRef::You,
                count: Value::GreatestManaValueAmongPermanents(PlayerRef::You),
                pick_filter: Some(R::Artifact),
                to_battlefield: true,
                optional: true,
                rest_to_graveyard: false,
                take: None,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Muzzio, Visionary Architect",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Reign of the Pit — everyone gives up a creature; you get the Demon.
pub fn reign_of_the_pit() -> CardDefinition {
    CardDefinition {
        name: "Reign of the Pit",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Creature,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Demon".into(),
                    colors: vec![Color::Black],
                    card_types: vec![CardType::Creature],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Demon],
                        ..Default::default()
                    },
                    dynamic_pt: Some((Value::SacrificedTotalPower, Value::SacrificedTotalPower)),
                    ..Default::default()
                },
            },
        ]),
        ..Default::default()
    }
}

/// Scourge of the Throne — dethrone, and a second combat off the first swing.
pub fn scourge_of_the_throne() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![dethrone(), TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::PlayerHasMostLife { who: PlayerRef::DefendingPlayer },
                )
            },
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                    up_to: None,
                },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..creature(
            "Scourge of the Throne",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Split Decision — will of the council: counter the spell, or copy it.
pub fn split_decision() -> CardDefinition {
    CardDefinition {
        name: "Split Decision",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Vote {
            tally: VoteTally::Majority,
            options: vec![
                VoteOption::new(
                    "denial",
                    Effect::CounterSpell {
                        what: target_filtered(R::IsSpellOnStack.and(
                            R::HasCardType(CardType::Instant)
                                .or(R::HasCardType(CardType::Sorcery)),
                        )),
                    },
                ),
                VoteOption::new(
                    "duplication",
                    Effect::CopySpellMayChooseTargets { what: Selector::Target(0), count: Value::ONE },
                ),
            ],
        },
        ..Default::default()
    }
}

// ── Draft-matters shells (the noted-value halves are draft-time; the
//    battlefield half is exact) ───────────────────────────────────────────────

/// Cogwork Tracker — a 4/4 that must attack every combat.
pub fn cogwork_tracker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttack],
        ..artifact_creature(
            "Cogwork Tracker",
            cost(&[generic(4)]),
            vec![CreatureType::Dog, CreatureType::Construct],
            4,
            4,
        )
    }
}

// ── Conspiracies ────────────────────────────────────────────────────────────

fn conspiracy(
    name: &'static str,
    description: &'static str,
    effect: StaticEffect,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Conspiracy],
        static_abilities: vec![StaticAbility { description, effect }],
        ..Default::default()
    }
}

/// Advantageous Proclamation — your minimum deck size drops by five.
pub fn advantageous_proclamation() -> CardDefinition {
    conspiracy(
        "Advantageous Proclamation",
        "Your minimum deck size is reduced by five.",
        StaticEffect::ReduceMinimumDeckSize(5),
    )
}

/// Backup Plan — an extra opening hand; all but one are shuffled back.
pub fn backup_plan() -> CardDefinition {
    conspiracy(
        "Backup Plan",
        "Draw an additional hand of seven cards as the game begins.",
        StaticEffect::ExtraOpeningHand,
    )
}

/// Unexpected Potential — hidden agenda; the named card casts off any colour.
pub fn unexpected_potential() -> CardDefinition {
    conspiracy(
        "Unexpected Potential",
        "You may spend mana as though it were mana of any color to cast spells \
         with the chosen name.",
        StaticEffect::MaySpendManaAsAnyColorForNamedSpells,
    )
}

/// Deal Broker — the battlefield half; the post-draft trade is draft-time.
pub fn deal_broker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..artifact_creature("Deal Broker", cost(&[generic(3)]), vec![CreatureType::Construct], 2, 3)
    }
}
