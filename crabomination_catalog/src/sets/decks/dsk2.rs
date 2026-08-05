//! Duskmourn gap batch — the legends, the enchantment build-arounds and the
//! delirium spells. Tests in `tests/recent_b/dsk2.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, ZoneDest,
};
use crate::mana::{ManaCost, b, cost, g, generic, u, w, x};

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Let's Play a Game — {3}{B} Sorcery. Delirium turns the single mode into
/// "choose one or more".
pub fn lets_play_a_game() -> CardDefinition {
    let modes = || {
        vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
            },
        ]
    };
    CardDefinition {
        name: "Let's Play a Game",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::DeliriumActive { who: PlayerRef::You },
            then: Box::new(Effect::ChooseN { picks: vec![0, 1, 2], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
        ..Default::default()
    }
}

/// Marina Vendrell — {W}{U}{B}{R}{G} 3/5. Digs seven for enchantments, then
/// works the doors of your Rooms by hand.
pub fn marina_vendrell() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::RevealTopTakeMatchingToHand {
            who: PlayerRef::You,
            count: Value::Const(7),
            filter: R::Enchantment,
            distinct_powers: false,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::LockOrUnlockRoomDoor {
                what: target_filtered(
                    R::HasEnchantmentSubtype(EnchantmentSubtype::Room).and(R::ControlledByYou),
                ),
            },
            ..Default::default()
        }],
        ..legend(
            "Marina Vendrell",
            cost(&[w(), u(), b(), crate::mana::r(), g()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            5,
        )
    }
}

/// Marina Vendrell's Grimoire — {5}{U} Book. Life swings become cards, and an
/// empty hand ends the game.
pub fn marina_vendrells_grimoire() -> CardDefinition {
    CardDefinition {
        name: "Marina Vendrell's Grimoire",
        cost: cost(&[generic(5), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        static_abilities: vec![
            StaticAbility {
                description: "You have no maximum hand size.",
                effect: StaticEffect::NoMaximumHandSize,
            },
            StaticAbility {
                description: "You don't lose the game for having 0 or less life.",
                effect: StaticEffect::ControllerDoesntLoseFromLife,
            },
        ],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::SourceWasCast),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(5) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::TriggerEventAmount,
                        random: false,
                    },
                    Effect::If {
                        cond: Predicate::HellbentActive { who: PlayerRef::You },
                        then: Box::new(Effect::LoseGame { who: PlayerRef::You }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Marvin, Murderous Mimic — {2} 2/2 Toy. Wears every differently-named
/// creature's activated abilities.
pub fn marvin_murderous_mimic() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Marvin has all activated abilities of creatures you control that \
                          don't have the same name as this creature.",
            effect: StaticEffect::HasActivatedAbilitiesOfOtherNamedControlledCreatures,
        }],
        ..legend("Marvin, Murderous Mimic", cost(&[generic(2)]), vec![CreatureType::Toy], 2, 2)
    }
}

/// Meathook Massacre II — {X}{X}{B}{B}{B}{B}. A sweeper that keeps the bodies.
pub fn meathook_massacre_ii() -> CardDefinition {
    let reanimate = || {
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::Finality,
                amount: Value::ONE,
            },
        ])
    };
    CardDefinition {
        name: "Meathook Massacre II",
        cost: cost(&[x(), x(), b(), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::XFromCost,
                filter: R::Creature,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::MayPayLife {
                    description: "Pay 3 life to return that creature?".into(),
                    amount: Value::Const(3),
                    body: Box::new(reanimate()),
                    else_: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::PlayerMayPayLifeElse {
                    who: PlayerRef::TriggerEventPlayer,
                    life: Value::Const(3),
                    else_: Box::new(reanimate()),
                },
            },
        ],
        ..Default::default()
    }
}

/// Nashi, Searcher in the Dark — {U}{B} 2/2 menace Rat Ninja. Connects to mill
/// for legends and enchantments, or grows when it whiffs.
pub fn nashi_searcher_in_the_dark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MillThenToHandN {
                amount: Value::TriggerEventAmount,
                filter: R::HasSupertype(Supertype::Legendary).or(R::Enchantment),
                take: Value::TriggerEventAmount,
                otherwise: Some(Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                })),
            },
        }],
        ..legend(
            "Nashi, Searcher in the Dark",
            cost(&[u(), b()]),
            vec![CreatureType::Rat, CreatureType::Ninja, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// The Mindskinner — {U}{U}{U} 10/1 unblockable. Its damage doesn't hurt; it
/// grinds libraries instead.
pub fn the_mindskinner() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Unblockable],
        static_abilities: vec![StaticAbility {
            description: "If a source you control would deal damage to an opponent, prevent \
                          that damage and each opponent mills that many cards.",
            effect: StaticEffect::YourDamageToOpponentsBecomesMill,
        }],
        ..legend("The Mindskinner", cost(&[u(), u(), u()]), vec![CreatureType::Nightmare], 10, 1)
    }
}

/// The Rollercrusher Ride — {X}{2}{R}. Delirium doubles your noncombat damage;
/// the entry burn spreads X around.
pub fn the_rollercrusher_ride() -> CardDefinition {
    CardDefinition {
        name: "The Rollercrusher Ride",
        cost: cost(&[x(), generic(2), crate::mana::r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Delirium — If a source you control would deal noncombat damage to a \
                          permanent or player while there are four or more card types among \
                          cards in your graveyard, it deals double that damage instead.",
            effect: StaticEffect::DoubleYourNoncombatDamageWhile {
                condition: Predicate::DeliriumActive { who: PlayerRef::You },
            },
        }],
        triggered_abilities: vec![etb(Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                    amount: Value::XFromCost,
                }),
            }),
        })],
        ..Default::default()
    }
}

/// Tyvar, the Pummeler — {1}{G}{G} 3/3 Elf. Taps the team for protection, then
/// swings the whole board up to your biggest body.
pub fn tyvar_the_pummeler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_others_cost: Some((R::Creature.and(R::ControlledByYou), 1)),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), g(), g()]),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::GreatestPowerControlled { who: PlayerRef::You },
                    toughness: Value::GreatestPowerControlled { who: PlayerRef::You },
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Tyvar, the Pummeler",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Rip, Spawn Hunter — {2}{G}{W} 4/4 Survivor. Survival digs its own power
/// deep for bodies with different powers.
pub fn rip_spawn_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crabomination_base::turn_step::TurnStep::PostCombatMain),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Tapped,
            }),
            effect: Effect::RevealTopTakeMatchingToHand {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::This)),
                filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                distinct_powers: true,
            },
        }],
        ..legend(
            "Rip, Spawn Hunter",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Survivor],
            4,
            4,
        )
    }
}
