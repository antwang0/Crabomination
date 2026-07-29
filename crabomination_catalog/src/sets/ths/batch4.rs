//! Theros (THS) — batch 4: the rare/mythic tail. Tests in `classic_sets/ths`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, monstrosity, on_becomes_monstrous, target_filtered};
use crate::effect::{Duration, Effect, Predicate, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: ct, ..Default::default() },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, p, t, ct, kw)
    }
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect,
    }
}

/// Abhorrent Overlord — {5}{B}{B} 6/6 Demon with flying. ETB: a 1/1 flying
/// Harpy per point of devotion to black. Upkeep: sacrifice a creature.
pub fn abhorrent_overlord() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::DevotionTo(vec![Color::Black]),
                definition: TokenDefinition {
                    name: "Harpy".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Harpy],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            upkeep(Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            }),
        ],
        ..creature(
            "Abhorrent Overlord",
            cost(&[generic(5), b(), b()]),
            6,
            6,
            vec![CreatureType::Demon],
            vec![Keyword::Flying],
        )
    }
}

/// Akroan Horse — {4} 0/4 Horse artifact creature with defender. ETB: an
/// opponent gains control of it. Upkeep: each opponent creates a 1/1 Soldier.
pub fn akroan_horse() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_under_opponent_control: true,
        triggered_abilities: vec![upkeep(Effect::CreateToken {
            who: PlayerRef::EachOpponent,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Soldier".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Soldier],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..creature(
            "Akroan Horse",
            cost(&[generic(4)]),
            0,
            4,
            vec![CreatureType::Horse],
            vec![Keyword::Defender],
        )
    }
}

/// Anthousa, Setessan Hero — {3}{G}{G} 4/5 legendary Human Warrior. Heroic: up
/// to three target lands you control become 2/2 Warriors until end of turn.
pub fn anthousa_setessan_hero() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Land.and(R::ControlledByYou),
            effect: Box::new(Effect::BecomeCreature {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                creature_types: vec![CreatureType::Warrior],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            }),
        })],
        ..legend(
            "Anthousa, Setessan Hero",
            cost(&[generic(3), g(), g()]),
            4,
            5,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Colossus of Akros — {8} 10/10 Golem with defender and indestructible.
/// {10}: Monstrosity 10; while monstrous it has trample and attacks as though
/// it didn't have defender.
pub fn colossus_of_akros() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![monstrosity(cost(&[generic(10)]), 10)],
        static_abilities: vec![
            StaticAbility {
                description: "As long as this creature is monstrous, it has trample.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SourceIsMonstrous,
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Trample],
                },
            },
            StaticAbility {
                description: "As long as this creature is monstrous, it can attack as though \
                              it didn't have defender.",
                effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                    condition: Predicate::SourceIsMonstrous,
                },
            },
        ],
        ..creature(
            "Colossus of Akros",
            cost(&[generic(8)]),
            10,
            10,
            vec![CreatureType::Golem],
            vec![Keyword::Defender, Keyword::Indestructible],
        )
    }
}

/// Hythonia the Cruel — {4}{B}{B} 4/6 legendary Gorgon with deathtouch.
/// {6}{B}{B}: Monstrosity 3; when it becomes monstrous, destroy all non-Gorgon
/// creatures.
pub fn hythonia_the_cruel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(6), b(), b()]), 3)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::Destroy {
            what: Selector::EachPermanent(
                R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Gorgon)))),
            ),
        })],
        ..legend(
            "Hythonia the Cruel",
            cost(&[generic(4), b(), b()]),
            4,
            6,
            vec![CreatureType::Gorgon],
            vec![Keyword::Deathtouch],
        )
    }
}

/// Medomai the Ageless — {4}{W}{U} 4/4 legendary Sphinx with flying. Combat
/// damage to a player takes an extra turn. (The printed "can't attack during
/// extra turns" rider needs an is-extra-turn predicate — TODO.md.)
pub fn medomai_the_ageless() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
        }],
        ..legend(
            "Medomai the Ageless",
            cost(&[generic(4), w(), u()]),
            4,
            4,
            vec![CreatureType::Sphinx],
            vec![Keyword::Flying],
        )
    }
}

/// Priest of Iroas — {R} 1/1 Human Cleric. {3}{W}, Sacrifice this: destroy
/// target enchantment.
pub fn priest_of_iroas() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature(
            "Priest of Iroas",
            cost(&[r()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Rageblood Shaman — {1}{R}{R} 2/3 Minotaur Shaman with trample. Other
/// Minotaurs you control get +1/+1 and have trample.
pub fn rageblood_shaman() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Minotaur creatures you control get +1/+1 and have trample.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasCreatureType(CreatureType::Minotaur))
                    .and(R::OtherThanSource),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Trample],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..creature(
            "Rageblood Shaman",
            cost(&[generic(1), r(), r()]),
            2,
            3,
            vec![CreatureType::Minotaur, CreatureType::Shaman],
            vec![Keyword::Trample],
        )
    }
}

/// Reaper of the Wilds — {2}{B}{G} 4/5 Gorgon. Whenever another creature dies,
/// scry 1. {B}: deathtouch until end of turn. {1}{G}: hexproof until end of turn.
pub fn reaper_of_the_wilds() -> CardDefinition {
    let grant = |mana: ManaCost, keyword: Keyword| ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        activated_abilities: vec![
            grant(cost(&[b()]), Keyword::Deathtouch),
            grant(cost(&[generic(1), g()]), Keyword::Hexproof),
        ],
        ..creature(
            "Reaper of the Wilds",
            cost(&[generic(2), b(), g()]),
            4,
            5,
            vec![CreatureType::Gorgon],
            vec![],
        )
    }
}

/// Shipwreck Singer — {U}{B} 1/2 Siren with flying. {1}{U}: target creature an
/// opponent controls attacks this turn if able. {1}{B}, {T}: attacking
/// creatures get -1/-1 until end of turn.
pub fn shipwreck_singer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    keyword: Keyword::MustAttack,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Shipwreck Singer",
            cost(&[u(), b()]),
            1,
            2,
            vec![CreatureType::Siren],
            vec![Keyword::Flying],
        )
    }
}

/// Steam Augury — {2}{U}{R} Instant. Reveal the top five cards, split them into
/// two piles; an opponent chooses one pile for your hand, the rest is milled.
pub fn steam_augury() -> CardDefinition {
    CardDefinition {
        name: "Steam Augury",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::FactOrFiction { count: Value::Const(5) },
        ..Default::default()
    }
}

/// Time to Feed — {2}{G} Sorcery. Your creature fights an opponent's; when that
/// creature dies this turn, you gain 3 life.
pub fn time_to_feed() -> CardDefinition {
    CardDefinition {
        name: "Time to Feed",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                }),
                slot: 0,
                filter: Some(R::Creature.and(R::ControlledByOpponent)),
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                defender: Selector::Target(0),
            },
        ]),
        ..Default::default()
    }
}

/// Tymaret, the Murder King — {B}{R} 2/2 legendary Zombie Warrior. {1}{R},
/// sacrifice another creature: 2 damage to target player or planeswalker.
/// {1}{B}, sacrifice a creature: return Tymaret from your graveyard to hand.
pub fn tymaret_the_murder_king() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                from_graveyard: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Tymaret, the Murder King",
            cost(&[b(), r()]),
            2,
            2,
            vec![CreatureType::Zombie, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Warriors' Lesson — {G} Instant. Until end of turn, up to two target
/// creatures you control each gain "whenever this deals combat damage to a
/// player, draw a card."
pub fn warriors_lesson() -> CardDefinition {
    CardDefinition {
        name: "Warriors' Lesson",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByYou),
            effect: Box::new(Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                }),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Bident of Thassa — {2}{U}{U} legendary enchantment artifact. Your creatures'
/// combat damage to a player may draw a card; {1}{U}, {T}: opponents' creatures
/// attack this turn if able.
pub fn bident_of_thassa() -> CardDefinition {
    CardDefinition {
        name: "Bident of Thassa",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::MayDo {
                description: String::from("Draw a card?"),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bow of Nylea — {1}{G}{G} legendary enchantment artifact. Attacking creatures
/// you control have deathtouch; {1}{G}, {T}: choose one of four modes.
pub fn bow_of_nylea() -> CardDefinition {
    CardDefinition {
        name: "Bow of Nylea",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control have deathtouch.",
            effect: StaticEffect::GrantKeywordToAttackers { keyword: Keyword::Deathtouch },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: Effect::ChooseModesCast {
                modes: vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::ShuffleGraveyardCardsIntoLibrary {
                        who: PlayerRef::You,
                        filter: R::Any,
                        max: Value::Const(4),
                    },
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
