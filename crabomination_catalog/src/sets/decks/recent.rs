//! Recent-set staples (MH3 / BLB / DSK / OTJ / FDN / DFT / TDM …) that fill
//! gaps in the Modern-playable pool. Each card has at least one functionality
//! test in `crabomination/src/tests/recent.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, Predicate, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, recover, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, colorless, cost, g, generic, r, u, w};

/// Questing Beast — {2}{G}{G} 4/4 Legendary Beast. Vigilance, deathtouch,
/// haste; can't be blocked by creatures with power 2 or less; combat damage
/// dealt by creatures you control can't be prevented. (The planeswalker-redirect
/// rider is omitted — planeswalkers aren't attack targets yet.)
pub fn questing_beast() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Questing Beast",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Deathtouch,
            Keyword::Haste,
            Keyword::CantBeBlockedByPowerAtMost(2),
        ],
        static_abilities: vec![StaticAbility {
            description: "Combat damage that would be dealt by creatures you control can't be prevented.",
            effect: StaticEffect::ControllerCreaturesCombatDamageCantBePrevented,
        }],
        ..Default::default()
    }
}

/// Cackling Slasher — {3}{B} 3/3 Human Assassin. Deathtouch; enters with a
/// +1/+1 counter if a creature died this turn.
pub fn cackling_slasher() -> CardDefinition {
    CardDefinition {
        name: "Cackling Slasher",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Vaultborn Tyrant — {5}{G}{G} 6/6 Dinosaur. Trample. Whenever this or
/// another creature you control with power 4+ enters, gain 3 life and draw.
/// When it dies (if not a token), create a token copy of it.
pub fn vaultborn_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Vaultborn Tyrant",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::PowerOf(Box::new(Selector::TriggerSource)),
                        Value::Const(4),
                    )),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::NotToken,
                    }),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    // The token "is an artifact in addition to its other types".
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: None,
                    non_legendary: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Star Charter — {3}{W} 3/1 Bat Cleric. Flying. At your end step, if you
/// gained or lost life this turn, look at the top four cards; you may reveal a
/// creature card with power 3 or less and put it into your hand.
pub fn star_charter() -> CardDefinition {
    CardDefinition {
        name: "Star Charter",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::Any(vec![
                Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(1) },
                Predicate::PlayerLostLifeThisTurn { who: PlayerRef::You },
            ])),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: false,
                pick_filter: Some(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
                ),
                take: None,
                to_battlefield: false,
            },
        }],
        ..Default::default()
    }
}

/// Dour Port-Mage — {1}{U} 1/3 Frog Wizard. Whenever one or more other
/// creatures you control leave the battlefield without dying, draw a card
/// (modeled per-creature). {1}{U}, {T}: Return another target creature you
/// control to its owner's hand.
pub fn dour_port_mage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Dour Port-Mage",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureLeavesBattlefieldNotDying,
                EventScope::AnotherOfYours,
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Three Tree Scribe — {1}{G} 2/3 Frog Druid. Whenever this or another creature
/// you control leaves the battlefield without dying, put a +1/+1 counter on
/// target creature you control. (The self-leave half is approximate — the
/// "another creature you control" case is the one that fires.)
pub fn three_tree_scribe() -> CardDefinition {
    CardDefinition {
        name: "Three Tree Scribe",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CreatureLeavesBattlefieldNotDying,
                EventScope::YourControl,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Krydle of Baldur's Gate — {U}{B} 1/3 Legendary Human Elf Rogue. Whenever it
/// deals combat damage to a player, that player loses 1 life and mills a card,
/// then you gain 1 life and scry 1. (The attack pay-{2} unblockable rider is
/// omitted.)
pub fn krydle_of_baldurs_gate() -> CardDefinition {
    CardDefinition {
        name: "Krydle of Baldur's Gate",
        cost: cost(&[u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Elf, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(1) },
                Effect::Mill { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(1) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Wary Watchdog — {1}{G} 3/1 Dog. When it enters or dies, surveil 1.
pub fn wary_watchdog() -> CardDefinition {
    CardDefinition {
        name: "Wary Watchdog",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
            on_dies(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
        ],
        ..Default::default()
    }
}

/// Hunted Bonebrute — {2}{B} 6/2 Skeleton Beast. Menace; when it enters, target
/// opponent creates two 1/1 white Dog tokens; when it dies, each opponent loses
/// 3 life. Disguise {1}{B}.
pub fn hunted_bonebrute() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Hunted Bonebrute",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 2,
        keywords: vec![Keyword::Menace, Keyword::Disguise(cost(&[generic(1), b()]))],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::EachOpponent,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Dog".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Dog],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            on_dies(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            }),
        ],
        ..Default::default()
    }
}

/// Trumpeting Herd — {2}{G}{G} Sorcery. Create a 3/3 green Elephant token.
/// Rebound.
pub fn trumpeting_herd() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Trumpeting Herd",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Elephant".into(),
                power: 3,
                toughness: 3,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elephant],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Festergloom — {2}{B} Sorcery. Nonblack creatures get -1/-1 until end of turn.
pub fn festergloom() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Festergloom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(Color::Black).negate()),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Intrepid Rabbit — {2}{W} 3/2 Rabbit Soldier. Offspring {1}. When it enters,
/// target creature you control gets +1/+1 until end of turn.
pub fn intrepid_rabbit() -> CardDefinition {
    CardDefinition {
        name: "Intrepid Rabbit",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Offspring(cost(&[generic(1)]))],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Marauding Brinefang — {5}{U}{U} 6/7 Dinosaur. Ward {3}. Islandcycling {2}.
pub fn marauding_brinefang() -> CardDefinition {
    use crate::card::{LandType, WardCost};
    CardDefinition {
        name: "Marauding Brinefang",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 7,
        keywords: vec![
            Keyword::Ward(WardCost::Mana(cost(&[generic(3)]))),
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                SelectionRequirement::HasLandType(LandType::Island),
            ))),
        ],
        ..Default::default()
    }
}

/// Crystal Barricade — {1}{W} 0/4 Wall. Defender; you have hexproof. (The
/// prevent-noncombat-damage-to-your-other-creatures rider is omitted.)
pub fn crystal_barricade() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Crystal Barricade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        ..Default::default()
    }
}

/// Persistent Marshstalker — {1}{B} 3/1 Rat Berserker. Gets +1/+0 for each
/// other Rat you control. (Its threshold attack-recursion is omitted.)
pub fn persistent_marshstalker() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Persistent Marshstalker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each other Rat you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Rat)
                    .and(SelectionRequirement::OtherThanSource),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Druid of the Spade — {2}{G} 2/3 Rabbit Druid. As long as you control a
/// token, it gets +2/+0 and has trample.
pub fn druid_of_the_spade() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Druid of the Spade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "As long as you control a token, this creature gets +2/+0 and has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::IsToken.and(SelectionRequirement::ControlledByYou),
                )),
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

/// Nightbird's Clutches — {1}{R} Sorcery. Up to two target creatures can't
/// block this turn. Flashback {3}{R}.
pub fn nightbirds_clutches() -> CardDefinition {
    CardDefinition {
        name: "Nightbird's Clutches",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Get Out — {U}{U} Instant. Choose one — counter target creature or enchantment
/// spell; or return one or two target creatures and/or enchantments you own to
/// your hand.
pub fn get_out() -> CardDefinition {
    CardDefinition {
        name: "Get Out",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                    ),
                },
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: (SelectionRequirement::Creature.or(SelectionRequirement::Enchantment))
                    .and(SelectionRequirement::ControlledByYou),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Helpful Hunter — {1}{W} 1/1 Cat. When it enters, draw a card.
pub fn helpful_hunter() -> CardDefinition {
    CardDefinition {
        name: "Helpful Hunter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

/// Sunshower Druid — {G} 0/2 Frog Druid. When it enters, put a +1/+1 counter on
/// target creature and you gain 1 life.
pub fn sunshower_druid() -> CardDefinition {
    CardDefinition {
        name: "Sunshower Druid",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Treetop Snarespinner — {3}{G} 1/4 Spider. Reach, deathtouch. {2}{G}: Put a
/// +1/+1 counter on target creature you control. Activate only as a sorcery.
pub fn treetop_snarespinner() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Treetop Snarespinner",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Coruscation Mage — {1}{R} 2/2 Otter Wizard. Offspring {2}. Whenever you cast
/// a noncreature spell, this deals 1 damage to each opponent.
pub fn coruscation_mage() -> CardDefinition {
    CardDefinition {
        name: "Coruscation Mage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Thornplate Intimidator — {3}{B} 4/3 Rat Rogue. Offspring {3}. When it
/// enters, each opponent loses 3 life unless they sacrifice a nonland permanent
/// or discard a card. (Modeled as each opponent; printed targets one.)
pub fn thornplate_intimidator() -> CardDefinition {
    CardDefinition {
        name: "Thornplate Intimidator",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Offspring(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Punisher {
            chooser: Selector::Player(PlayerRef::EachOpponent),
            options: vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::You),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Nonland,
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(1),
                    random: false,
                },
            ],
            otherwise: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            }),
        })],
        ..Default::default()
    }
}

/// Repeating Barrage — {1}{R}{R} Sorcery. Deals 3 damage to any target. Raid —
/// {3}{R}{R}: Return this from your graveyard to your hand if you attacked.
pub fn repeating_barrage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::{deal, target_any};
    CardDefinition {
        name: "Repeating Barrage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: deal(3, target_any()),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), r()]),
            from_graveyard: true,
            condition: Some(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fountainport Bell — {1} Artifact. When it enters, you may search your library
/// for a basic land and put it on top. {1}, Sacrifice: Draw a card.
pub fn fountainport_bell() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Fountainport Bell",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Overprotect — {1}{G} Instant. Target creature you control gets +3/+3 and
/// gains trample, hexproof, and indestructible until end of turn.
pub fn overprotect() -> CardDefinition {
    let grant = |kw| Effect::GrantKeyword { what: Selector::Target(0), keyword: kw, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Overprotect",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            grant(Keyword::Trample),
            grant(Keyword::Hexproof),
            grant(Keyword::Indestructible),
        ]),
        ..Default::default()
    }
}

/// Plumecreed Escort — {1}{U} 2/1 Bird Scout. Flash, flying. When it enters,
/// target creature you control gains hexproof until end of turn.
pub fn plumecreed_escort() -> CardDefinition {
    CardDefinition {
        name: "Plumecreed Escort",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Banishing Slash — {W}{W} Sorcery. Destroy up to one target artifact,
/// enchantment, or tapped creature. Then if you control an artifact and an
/// enchantment, create a 2/2 white Samurai with vigilance.
pub fn banishing_slash() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Banishing Slash",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Creature.and(SelectionRequirement::Tapped)),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    )),
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::Enchantment.and(SelectionRequirement::ControlledByYou),
                    )),
                ]),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Samurai".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Samurai],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Vigilance],
                        ..Default::default()
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Marsh Hulk — {4}{B}{B} 4/6 Zombie Ogre. Megamorph {6}{B}.
pub fn marsh_hulk() -> CardDefinition {
    CardDefinition {
        name: "Marsh Hulk",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Ogre],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Megamorph(cost(&[generic(6), b()]))],
        ..Default::default()
    }
}

/// Brave-Kin Duo — {W} 1/1 Rabbit Mouse. {1}, {T}: Target creature gets +1/+1
/// until end of turn. Activate only as a sorcery.
pub fn brave_kin_duo() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Brave-Kin Duo",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Mouse],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightshield Parry — {W} Instant. Target creature gets +2/+2 until end of
/// turn. Cycling {2}.
pub fn lightshield_parry() -> CardDefinition {
    CardDefinition {
        name: "Lightshield Parry",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Hard-Hitting Question — {G} Sorcery. Target creature you control deals damage
/// equal to its power to target creature or planeswalker you don't control.
pub fn hard_hitting_question() -> CardDefinition {
    CardDefinition {
        name: "Hard-Hitting Question",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            })),
        },
        ..Default::default()
    }
}

/// Refurbished Familiar — {3}{B} 2/1 Zombie Rat. Affinity for artifacts, flying.
/// When it enters, each opponent discards a card. (The draw-per-opponent-who-
/// can't rider is omitted.)
pub fn refurbished_familiar() -> CardDefinition {
    CardDefinition {
        name: "Refurbished Familiar",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(SelectionRequirement::Artifact),
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Galvanic Discharge — {R} Instant. Choose target creature or planeswalker.
/// You get {E}{E}{E}, then you may pay any amount of {E}; deal that much damage.
pub fn galvanic_discharge() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Discharge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(3)),
            Effect::PayAnyEnergyDealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// This Town Ain't Big Enough — {4}{U} Instant. Costs {3} less if it targets a
/// permanent you control. Return up to two target nonland permanents to hand.
pub fn this_town_aint_big_enough() -> CardDefinition {
    CardDefinition {
        name: "This Town Ain't Big Enough",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::ControlledByYou, 3)),
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Nonland,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
        ..Default::default()
    }
}

/// Hardened Scales — {G} enchantment. If one or more +1/+1 counters would be
/// put on a creature you control, that many plus one are put on it instead.
pub fn hardened_scales() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Hardened Scales",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature \
                          you control, that many plus one +1/+1 counters are put \
                          on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        ..Default::default()
    }
}

/// Highspire Bell-Ringer — {2}{U} 1/4 Djinn Monk. Flying; the second spell you
/// cast each turn costs {1} less.
pub fn highspire_bell_ringer() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Highspire Bell-Ringer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "The second spell you cast each turn costs {1} less to cast.",
            effect: StaticEffect::CostReductionNthSpell {
                filter: SelectionRequirement::Any,
                nth: 2,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Emberheart Challenger — {1}{R} 2/2 Mouse Warrior. Haste, prowess; Valiant
/// — the first time it becomes the target of your spell/ability each turn,
/// exile the top card of your library; you may play it this turn.
pub fn emberheart_challenger() -> CardDefinition {
    CardDefinition {
        name: "Emberheart Challenger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            crate::effect::shortcut::prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl)
                    .once_per_turn(),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Eldrazi Linebreaker — {1}{C}{R} 3/3 Eldrazi. Devoid, trample. At the
/// beginning of combat on your turn, target creature you control gains haste
/// and gets +X/+0 until end of turn, where X is the number of Eldrazi you
/// control.
pub fn eldrazi_linebreaker() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Linebreaker",
        cost: cost(&[generic(1), colorless(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::ControlledByYou,
                        )),
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Eldrazi),
                    },
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// No More Lies — {W}{U} Instant. Counter target spell unless its controller
/// pays {3}. If countered this way, exile it instead.
pub fn no_more_lies() -> CardDefinition {
    CardDefinition {
        name: "No More Lies",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
            exile: true,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Unstoppable Slasher — {2}{B} 2/3 Zombie Assassin. Deathtouch; when it deals
/// combat damage to a player, they lose half their life, rounded up. When it
/// dies, return it tapped with two stun counters under its owner's control.
pub fn unstoppable_slasher() -> CardDefinition {
    CardDefinition {
        name: "Unstoppable Slasher",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    rounded_up: true,
                },
            },
            on_dies(Effect::ReturnSelfTappedWithCounters {
                kind: CounterType::Stun,
                amount: 2,
            }),
        ],
        ..Default::default()
    }
}

/// Enduring Curiosity — {2}{U}{U} 4/3 Cat Glimmer enchantment creature. Flash;
/// whenever a creature you control deals combat damage to a player, draw a
/// card. When it dies, return it to the battlefield as an enchantment.
pub fn enduring_curiosity() -> CardDefinition {
    CardDefinition {
        name: "Enduring Curiosity",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
            on_dies(Effect::ReturnSelfAsEnchantment),
        ],
        ..Default::default()
    }
}

/// Galvanic Relay — {2}{R} Sorcery. Exile the top card of your library; you
/// may play it during your next turn. Storm.
pub fn galvanic_relay() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Relay",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(1),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        },
        ..Default::default()
    }
}

/// The Necrobloom — {1}{W}{B}{G} 2/7 Legendary Plant. Landfall — whenever a
/// land you control enters, create a 0/1 green Plant token; if you control 7+
/// lands with different names, a 2/2 Zombie instead. (The "lands in your
/// graveyard have dredge 2" static is omitted.)
pub fn the_necrobloom() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Necrobloom",
        cost: cost(&[generic(1), w(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: plant },
        }],
        ..Default::default()
    }
}

/// Tyvar's Stand — {X}{G} Instant. Target creature you control gets +X/+X and
/// gains hexproof and indestructible until end of turn.
pub fn tyvars_stand() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Tyvar's Stand",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Gird for Battle — {W} Sorcery. Put a +1/+1 counter on each of up to two
/// target creatures.
pub fn gird_for_battle() -> CardDefinition {
    CardDefinition {
        name: "Gird for Battle",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}

/// Stock Up — {2}{U} Sorcery. Look at the top five cards of your library, put
/// two into your hand and the rest on the bottom.
pub fn stock_up() -> CardDefinition {
    CardDefinition {
        name: "Stock Up",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: None,
            take: Some(Value::Const(2)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Shelter — {1}{W} Instant. Target creature you control gains protection from
/// the color of your choice until end of turn. Draw a card.
pub fn shelter() -> CardDefinition {
    CardDefinition {
        name: "Shelter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Pick Your Poison — {G} Sorcery. Choose one — each opponent sacrifices an
/// artifact / an enchantment / a creature with flying, their choice.
pub fn pick_your_poison() -> CardDefinition {
    let edict = |filter: SelectionRequirement| Effect::Sacrifice {
        who: Selector::Player(PlayerRef::EachOpponent),
        count: Value::Const(1),
        filter,
    };
    CardDefinition {
        name: "Pick Your Poison",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            edict(SelectionRequirement::Artifact),
            edict(SelectionRequirement::Enchantment),
            edict(SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying))),
        ]),
        ..Default::default()
    }
}

/// Tail Swipe — {G} Instant. Target creature you control fights target creature
/// you don't control; if cast in your main phase, yours gets +1/+1 until end of
/// turn first.
pub fn tail_swipe() -> CardDefinition {
    use crate::effect::Predicate;
    use crate::game::TurnStep;
    let attacker = Selector::TargetFiltered {
        slot: 0,
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Tail Swipe",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::Any(vec![
                        Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                        Predicate::CurrentStepIs(TurnStep::PostCombatMain),
                    ]),
                ]),
                then: Box::new(Effect::PumpPT {
                    what: attacker.clone(),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker,
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Lightning Axe — {R} Instant. (As an additional cost, discard a card.)
/// Deals 5 damage to target creature. (The "or pay {5}" alternative is
/// omitted — the discard is taken at resolution, Deadly-Dispute style.)
pub fn lightning_axe() -> CardDefinition {
    CardDefinition {
        name: "Lightning Axe",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(5),
            },
        ]),
        ..Default::default()
    }
}

/// Stormsplitter — {3}{R} 1/4 Otter Wizard. Haste. Whenever you cast an instant
/// or sorcery spell, create a token copy of this creature; exile it at the
/// beginning of the next end step.
pub fn stormsplitter() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        name: "Stormsplitter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    non_legendary: false,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Unburden — {1}{B}{B} Sorcery. Target player discards two cards. Cycling {2}.
pub fn unburden() -> CardDefinition {
    CardDefinition {
        name: "Unburden",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
            random: false,
        },
        ..Default::default()
    }
}

/// Goblin Anarchomancer — {R}{G} 2/2 Goblin Shaman. Each red or green spell you
/// cast costs {1} less to cast.
pub fn goblin_anarchomancer() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::mana::Color;
    CardDefinition {
        name: "Goblin Anarchomancer",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Red or green spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasColor(Color::Red)
                    .or(SelectionRequirement::HasColor(Color::Green)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Beza, the Bounding Spring — {2}{W}{W} 4/5 Legendary Elemental Elk. ETB: a
/// Treasure if an opponent has more lands; gain 4 if an opponent has more life;
/// two 1/1 Fish if an opponent has more creatures; draw if an opponent has more
/// cards in hand.
pub fn beza_the_bounding_spring() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let if_then = |cond: Predicate, then: Effect| Effect::If {
        cond,
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    };
    let fish = TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Beza, the Bounding Spring",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Elk],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            if_then(
                Predicate::OpponentControlsMoreLandsThanYou,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            ),
            if_then(
                Predicate::AnOpponentHasMoreLife,
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ),
            if_then(
                Predicate::AnOpponentControlsMoreCreatures,
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: fish },
            ),
            if_then(
                Predicate::AnOpponentHasMoreCardsInHand,
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ),
        ]))],
        ..Default::default()
    }
}

/// Optimistic Scavenger — {W} 1/1 Human Scout. Eerie — whenever an enchantment
/// you control enters, put a +1/+1 counter on target creature. (The
/// fully-unlock-a-Room half is omitted.)
pub fn optimistic_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Optimistic Scavenger",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Frilled Sandwalla — {G} 1/1 Lizard. {1}{G}: +2/+2 until end of turn,
/// once each turn.
pub fn frilled_sandwalla() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Frilled Sandwalla",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lizard], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spell Stutter — {1}{U} Instant. Counter target spell unless its controller
/// pays {2} plus {1} for each Faerie you control.
pub fn spell_stutter() -> CardDefinition {
    CardDefinition {
        name: "Spell Stutter",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
            exile: false,
            extra_generic: Some(Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Faerie),
            }),
        },
        ..Default::default()
    }
}

/// Spectral Interference — {1}{U} Instant. Counter target artifact or creature
/// spell unless its controller pays {4}.
pub fn spectral_interference() -> CardDefinition {
    CardDefinition {
        name: "Spectral Interference",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::HasCardType(CardType::Creature)),
                ),
            ),
            mana_cost: cost(&[generic(4)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Refute — {1}{U}{U} Instant. Counter target spell. Draw a card, then discard.
pub fn refute() -> CardDefinition {
    CardDefinition {
        name: "Refute",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Skullcap Snail — {1}{B} 1/1 Fungus Snail. ETB: target opponent exiles a
/// card from their hand.
pub fn skullcap_snail() -> CardDefinition {
    CardDefinition {
        name: "Skullcap Snail",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Snail],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Aspirant's Ascent — {U} Instant. Target creature gets +1/+3 and gains flying
/// and toxic 1 until end of turn.
pub fn aspirants_ascent() -> CardDefinition {
    CardDefinition {
        name: "Aspirant's Ascent",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Toxic(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Take the Fall — {U} Instant. Target creature gets -1/-0 (or -4/-0 if you
/// control an outlaw) until end of turn. Draw a card.
pub fn take_the_fall() -> CardDefinition {
    let outlaw = SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(
            SelectionRequirement::HasCreatureType(CreatureType::Assassin)
                .or(SelectionRequirement::HasCreatureType(CreatureType::Mercenary))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Pirate))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Rogue))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Warlock)),
        );
    CardDefinition {
        name: "Take the Fall",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(outlaw),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-3),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Hopeful Vigil — {1}{W} Enchantment. ETB: create a 2/2 white Knight with
/// vigilance. When it leaves the battlefield, scry 2. {2}{W}: sacrifice it.
pub fn hopeful_vigil() -> CardDefinition {
    use crate::card::{ActivatedAbility, TokenDefinition};
    use crate::mana::Color;
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Hopeful Vigil",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: knight }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hopeless Nightmare — {B} Enchantment. ETB: each opponent discards a card and
/// loses 2 life. When it leaves the battlefield, scry 2. {2}{B}: sacrifice it.
pub fn hopeless_nightmare() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Hopeless Nightmare",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hangar Scrounger — {2}{R} 2/1 Dwarf Pilot. Backup 1. Whenever it becomes
/// tapped, you may discard a card; if you do, draw a card. (The backup-grant
/// of the loot ability to the backed-up creature is omitted.)
pub fn hangar_scrounger() -> CardDefinition {
    let loot = TriggeredAbility {
        event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "discard a card, then draw a card".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
        },
    };
    CardDefinition {
        name: "Hangar Scrounger",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::backup(1, vec![]), loot],
        ..Default::default()
    }
}

/// Bristlebud Farmer — {2}{G}{G} 5/5 Plant Druid. Trample. ETB: create two
/// Food tokens. (The attack "sac a Food → mill three, grab a permanent" rider
/// is omitted.)
pub fn bristlebud_farmer() -> CardDefinition {
    CardDefinition {
        name: "Bristlebud Farmer",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Outcaster Greenblade — {2}{G} 1/2 Human Mercenary. ETB: search your library
/// for a basic land or Desert card and put it into your hand. Gets +1/+1 for
/// each Desert you control.
pub fn outcaster_greenblade() -> CardDefinition {
    use crate::card::{DynamicPt, LandType};
    CardDefinition {
        name: "Outcaster Greenblade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled {
            land_type: LandType::Desert,
            base_p: 1,
            base_t: 2,
        }),
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand
                .or(SelectionRequirement::HasLandType(LandType::Desert)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Mizzium Skin — {U} Instant. Target creature you control gets +0/+1 and gains
/// hexproof until end of turn. Overload {3}{U}: each creature you control
/// instead.
pub fn mizzium_skin() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Mizzium Skin",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u()]),
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(0),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Demand Answers — {1}{R} Instant. (As an additional cost, discard a card —
/// the "sacrifice an artifact" alternative is omitted.) Draw two cards.
pub fn demand_answers() -> CardDefinition {
    CardDefinition {
        name: "Demand Answers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Boltwave — {R} Sorcery. Deals 3 damage to each opponent.
pub fn boltwave() -> CardDefinition {
    CardDefinition {
        name: "Boltwave",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Strike It Rich — {R} Sorcery. Create a Treasure token. Flashback {2}{R}.
pub fn strike_it_rich() -> CardDefinition {
    CardDefinition {
        name: "Strike It Rich",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::game::effects::treasure_token(),
        },
        ..Default::default()
    }
}

/// Brotherhood's End — {1}{R}{R} Sorcery. Choose one — 3 damage to each
/// creature and planeswalker; or destroy all artifacts with mana value 3 or
/// less.
pub fn brotherhoods_end() -> CardDefinition {
    CardDefinition {
        name: "Brotherhood's End",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::Destroy {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Boon-Bringer Valkyrie — {3}{W}{W} 4/4 Angel Warrior. Flying, first strike,
/// lifelink. Backup 1 (grants those abilities to the backed-up creature).
pub fn boon_bringer_valkyrie() -> CardDefinition {
    CardDefinition {
        name: "Boon-Bringer Valkyrie",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![crate::effect::shortcut::backup(
            1,
            vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        )],
        ..Default::default()
    }
}

/// Inti, Seneschal of the Sun — {1}{R} 2/2 Legendary Human Knight. Whenever you
/// attack, you may discard a card to put a +1/+1 counter on target attacking
/// creature and give it trample. Whenever you discard a card, exile the top of
/// your library; you may play it until your next end step. ("Whenever you
/// attack" is approximated as once per turn.)
pub fn inti_seneschal_of_the_sun() -> CardDefinition {
    CardDefinition {
        name: "Inti, Seneschal of the Sun",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).once_per_turn(),
                effect: Effect::MayDo {
                    description: "Discard a card to grow a target attacking creature".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                        Effect::AddCounter {
                            what: target_filtered(SelectionRequirement::IsAttacking),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(1),
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Trample,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Warren Soultrader — {2}{B} 3/3 Zombie Goblin Wizard. Pay 1 life, Sacrifice
/// another creature: Create a Treasure token.
pub fn warren_soultrader() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Warren Soultrader",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hostile Investigator — {3}{B} 4/3 Ogre Rogue Detective. When it enters,
/// target opponent discards a card. Whenever one or more players discard one or
/// more cards, investigate (once each turn).
pub fn hostile_investigator() -> CardDefinition {
    CardDefinition {
        name: "Hostile Investigator",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rogue, CreatureType::Detective],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::AnyPlayer)
                    .once_per_turn(),
                effect: crate::effect::shortcut::investigate(1),
            },
        ],
        ..Default::default()
    }
}

/// Marshal of Zhalfir — {W}{U} 2/2 Human Knight. Other Knights you control get
/// +1/+1. {W}{U}, {T}: Tap another target creature.
pub fn marshal_of_zhalfir() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Marshal of Zhalfir",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Knights you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Knight)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pawpatch Recruit — {G} 2/1 Rabbit Warrior with trample. Offspring {2}.
/// Whenever a creature you control becomes the target of a spell or ability an
/// opponent controls, put a +1/+1 counter on target creature you control other
/// than that creature.
pub fn pawpatch_recruit() -> CardDefinition {
    CardDefinition {
        name: "Pawpatch Recruit",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![
            // Offspring (CR 702.166): if its cost was paid, mint a 1/1 copy.
            etb(Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    non_legendary: false,
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::BecameTarget,
                    EventScope::YourPermanentTargetedByOpponent,
                ),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Helping Hand — {W} Sorcery. Return target creature card with mana value 3 or
/// less from your graveyard to the battlefield tapped.
pub fn helping_hand() -> CardDefinition {
    CardDefinition {
        name: "Helping Hand",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3))
                    .and(SelectionRequirement::InYourGraveyard),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Diversion Unit — {1}{U} 2/1 Robot artifact creature with flying. {U},
/// Sacrifice this creature: Counter target instant or sorcery spell unless its
/// controller pays {3}.
pub fn diversion_unit() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Diversion Unit",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                ),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Final Vengeance — {B} Sorcery. As an additional cost, sacrifice a creature
/// or enchantment. Exile target creature.
pub fn final_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Final Vengeance",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: (SelectionRequirement::Creature.or(SelectionRequirement::Enchantment))
                .and(SelectionRequirement::ControlledByYou),
            count: 1,
        }],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Roughshod Mentor — {5}{G} 5/4 Giant Warrior. Green creatures you control
/// have trample.
pub fn roughshod_mentor() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::mana::Color;
    CardDefinition {
        name: "Roughshod Mentor",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Green creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasColor(Color::Green)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Innocuous Rat — {1}{B} 1/1 Rat. When it dies, manifest dread.
pub fn innocuous_rat() -> CardDefinition {
    CardDefinition {
        name: "Innocuous Rat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::ManifestDread { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Quaketusk Boar — {3}{R}{R} 5/5 Elemental Boar with reach, trample, haste.
pub fn quaketusk_boar() -> CardDefinition {
    CardDefinition {
        name: "Quaketusk Boar",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Boar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Haste],
        ..Default::default()
    }
}

/// Veteran Guardmouse — {3}{R/W} 3/4 Mouse Soldier. Valiant — the first time it
/// becomes the target of a spell or ability you control each turn, it gets
/// +1/+0 and gains first strike until end of turn, then scry 1.
pub fn veteran_guardmouse() -> CardDefinition {
    use crate::mana::hybrid;
    use crate::mana::Color::{Red, White};
    CardDefinition {
        name: "Veteran Guardmouse",
        cost: cost(&[generic(3), hybrid(Red, White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Polliwallop — {3}{G} Instant. Affinity for Frogs. Target creature you
/// control deals damage equal to twice its power to target creature you don't
/// control. (Damage is dealt by the spell rather than the creature.)
pub fn polliwallop() -> CardDefinition {
    CardDefinition {
        name: "Polliwallop",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Frog)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::Times(
                Box::new(Value::PowerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                }))),
                Box::new(Value::Const(2)),
            ),
        },
        ..Default::default()
    }
}

/// Coiling Rebirth — {3}{B}{B} Sorcery. Gift a card. Return target creature card
/// from your graveyard to the battlefield. If the gift was promised and that
/// creature isn't legendary, also create a 1/1 token copy of it.
pub fn coiling_rebirth() -> CardDefinition {
    use crate::card::Gift;
    let reanimate = Effect::Move {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
        },
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Coiling Rebirth",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: reanimate.clone(),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
                reanimate,
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::Target(0),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    non_legendary: true,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Pearl of Wisdom — {2}{U} Sorcery. Costs {1} less if you control an Otter.
/// Draw two cards.
pub fn pearl_of_wisdom() -> CardDefinition {
    CardDefinition {
        name: "Pearl of Wisdom",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_control: Some((
            SelectionRequirement::HasCreatureType(CreatureType::Otter)
                .and(SelectionRequirement::ControlledByYou),
            1,
        )),
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Ride's End — {4}{W} Instant. Costs {3} less to cast if it targets a tapped
/// permanent. Exile target creature or Vehicle.
pub fn rides_end() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Ride's End",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::Tapped, 3)),
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            ),
        },
        ..Default::default()
    }
}

/// Nurturing Pixie — {W} 1/1 Faerie Rogue with flying. When it enters, return
/// up to one target non-Faerie, nonland permanent you control to its owner's
/// hand; if one was returned, put a +1/+1 counter on this creature.
pub fn nurturing_pixie() -> CardDefinition {
    CardDefinition {
        name: "Nurturing Pixie",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Bounce a nonland permanent you control to grow the Pixie".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Permanent
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::Not(Box::new(
                                SelectionRequirement::HasCreatureType(CreatureType::Faerie),
                            ))),
                    },
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Stab — {B} Instant. Target creature gets -2/-2 until end of turn.
pub fn stab() -> CardDefinition {
    CardDefinition {
        name: "Stab",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Slumbering Keepguard — {W} 1/1 Human Knight. Whenever an enchantment you
/// control enters, scry 1. {2}{W}: This creature gets +1/+1 until end of turn
/// for each enchantment you control.
pub fn slumbering_keepguard() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let enchant_count = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::Enchantment,
    };
    CardDefinition {
        name: "Slumbering Keepguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: enchant_count.clone(),
                toughness: enchant_count,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ruby, Daring Tracker — {R}{G} 1/2 Legendary Human Scout with haste. Whenever
/// Ruby attacks while you control a creature with power 4 or greater, it gets
/// +2/+2 until end of turn. {T}: Add {R} or {G}.
pub fn ruby_daring_tracker() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Ruby, Daring Tracker",
        cost: cost(&[r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(4)),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            crate::sets::tap_add(Color::Green),
        ],
        ..Default::default()
    }
}


/// Anoint with Affliction — {1}{B} Instant. Exile target creature with mana
/// value 3 or less. (The Corrupted "any creature if its controller has 3+
/// poison" rider is dropped; the base mode caps the target at MV 3.)
pub fn anoint_with_affliction() -> CardDefinition {
    CardDefinition {
        name: "Anoint with Affliction",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Wing It — {1}{W} Instant. Target creature gets +2/+2 until end of turn, gets
/// a flying counter, then scry 1.
pub fn wing_it() -> CardDefinition {
    CardDefinition {
        name: "Wing It",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Cackling Prowler — {3}{G} 4/3 Hyena Rogue. Ward {2}. Morbid — at the
/// beginning of your end step, if a creature died this turn, put a +1/+1
/// counter on it.
pub fn cackling_prowler() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Cackling Prowler",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hyena, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Glimmerlight — {2} Equipment. When it enters, create a 1/1 white Glimmer
/// enchantment creature token. Equipped creature gets +1/+1. Equip {1}.
pub fn glimmerlight() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, TokenDefinition};
    use crate::mana::Color;
    CardDefinition {
        name: "Glimmerlight",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Glimmer".into(),
                card_types: vec![CardType::Enchantment, CardType::Creature],
                colors: vec![Color::White],
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Demonic Ruckus — {1}{R} Aura. Enchanted creature gets +1/+1 and has menace
/// and trample. When this Aura is put into a graveyard from the battlefield,
/// draw a card. Plot {R}.
pub fn demonic_ruckus() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Demonic Ruckus",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Menace, Keyword::Trample],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        plot_cost: Some(cost(&[r()])),
        ..Default::default()
    }
}

/// Hugs, Grisly Guardian — {X}{R}{R}{G}{G} 5/5 Legendary Badger Warrior with
/// trample. When it enters, exile the top X cards of your library; you may play
/// them until your next end step. You may play an additional land each turn.
pub fn hugs_grisly_guardian() -> CardDefinition {
    use crate::effect::StaticEffect;
    use crate::mana::x;
    CardDefinition {
        name: "Hugs, Grisly Guardian",
        cost: cost(&[x(), r(), r(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![crate::card::StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::XFromCost,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Gloomfang Mauler — {5}{B}{B} 5/5 Nightmare. Swampcycling {2}. Backup 2.
pub fn gloomfang_mauler() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Gloomfang Mauler",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp)],
        triggered_abilities: vec![crate::effect::shortcut::backup(2, vec![])],
        ..Default::default()
    }
}

/// Audacity — {G} Aura. Enchanted creature gets +2/+0 and has trample. When
/// this Aura is put into a graveyard from the battlefield, draw a card.
pub fn audacity() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Audacity",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            keywords: vec![Keyword::Trample],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Felonious Rage — {R} Instant. Target creature you control gets +2/+0 and
/// gains haste until end of turn. When that creature dies this turn, create a
/// 2/2 white and blue Detective creature token.
pub fn felonious_rage() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Felonious Rage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                slot: 0,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Detective".into(),
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White, Color::Blue],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Detective],
                            ..Default::default()
                        },
                        power: 2,
                        toughness: 2,
                        ..Default::default()
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Razorkin Hordecaller — {4}{R} 4/4 Human Clown Berserker with haste. Whenever
/// you attack, create a 1/1 red Gremlin creature token. (Modeled once per turn.)
pub fn razorkin_hordecaller() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Razorkin Hordecaller",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Clown, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Gremlin".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Gremlin],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Goldvein Pick — {2} Equipment. Equipped creature gets +1/+1 and, whenever it
/// deals combat damage to a player, creates a Treasure token. Equip {1}.
pub fn goldvein_pick() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Goldvein Pick",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![],
            scale: None,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Tarkir: Dragonstorm + recent-set batch (claude/modern_decks) ─────────────

/// Boulderborn Dragon — {5} Artifact Dragon 3/3. Flying, vigilance; attacks →
/// surveil 1.
pub fn boulderborn_dragon() -> CardDefinition {
    CardDefinition {
        name: "Boulderborn Dragon",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Scales of Shale — {2}{B} Instant. Affinity for Lizards. Target creature gets
/// +2/+0 and gains lifelink and indestructible until end of turn.
pub fn scales_of_shale() -> CardDefinition {
    CardDefinition {
        name: "Scales of Shale",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Lizard)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sunset Strikemaster — {1}{R} 3/1 Human Monk. {T}: Add {R}. {2}{R}, {T},
/// Sacrifice this: it deals 6 damage to target creature with flying.
pub fn sunset_strikemaster() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Sunset Strikemaster",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                    ),
                    amount: Value::Const(6),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wardens of the Cycle — {1}{B}{G}{G} 3/4 Elf Warlock. Morbid — at your end
/// step, if a creature died this turn, gain 2 life, or draw a card and lose 1.
pub fn wardens_of_the_cycle() -> CardDefinition {
    CardDefinition {
        name: "Wardens of the Cycle",
        cost: cost(&[generic(1), b(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
                then: Box::new(Effect::ChooseMode(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                    ]),
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Roiling Dragonstorm — {1}{U} Enchantment. ETB: draw two, then discard a
/// card. When a Dragon you control enters, return this to its owner's hand.
pub fn roiling_dragonstorm() -> CardDefinition {
    CardDefinition {
        name: "Roiling Dragonstorm",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Stormcatch Mentor — {U}{R} 1/1 Otter Wizard. Haste, prowess; instant and
/// sorcery spells you cast cost {1} less.
pub fn stormcatch_mentor() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Stormcatch Mentor",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Gurmag Drowner — {3}{U} 2/4 Snake Wizard. Exploit; when it exploits a
/// creature, look at the top four cards, put one into your hand, the rest on
/// the bottom.
pub fn gurmag_drowner() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Drowner",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![crate::effect::shortcut::exploit(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Temur Battlecrier — {G}{U}{R} 4/3 Orc Ranger. Spells you cast cost {1} less
/// for each creature you control with power 4 or greater. (The "during your
/// turn" gate is approximated as always-on.)
pub fn temur_battlecrier() -> CardDefinition {
    CardDefinition {
        name: "Temur Battlecrier",
        cost: cost(&[g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Ranger],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        affinity_filter: Some(
            SelectionRequirement::Creature
                .and(SelectionRequirement::PowerAtMost(3).negate())
                .and(SelectionRequirement::ControlledByYou),
        ),
        ..Default::default()
    }
}

/// Nullpriest of Oblivion — {1}{B} 2/1 Vampire Cleric. Kicker {3}{B}. Lifelink,
/// menace. ETB, if kicked: return target creature card from your graveyard to
/// the battlefield.
pub fn nullpriest_of_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Nullpriest of Oblivion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink, Keyword::Menace, Keyword::Kicker(cost(&[generic(3), b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Ureni, the Song Unending — {5}{G}{U}{R} 10/10 Spirit Dragon. Flying,
/// protection from white and from black. ETB: deal X damage divided as you
/// choose among any number of target creatures/planeswalkers opponents control,
/// where X is the number of lands you control.
pub fn ureni_the_song_unending() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Ureni, the Song Unending",
        cost: cost(&[generic(5), g(), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dragon],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(crate::mana::Color::White),
            Keyword::Protection(crate::mana::Color::Black),
        ],
        triggered_abilities: vec![etb(Effect::DealDamageDivided {
            total: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ))),
            filter: (SelectionRequirement::Creature
                .or(SelectionRequirement::HasCardType(CT::Planeswalker)))
            .and(SelectionRequirement::ControlledByOpponent),
            max_targets: 10,
        })],
        ..Default::default()
    }
}

/// Elspeth, Storm Slayer — {3}{W}{W} Legendary Planeswalker. Tokens you create
/// are doubled. +1: make a 1/1 Soldier. 0: +1/+1 on each creature you control,
/// they gain flying until your next turn. −3: destroy target creature an
/// opponent controls with mana value 3 or greater.
pub fn elspeth_storm_slayer() -> CardDefinition {
    use crate::card::{
        LoyaltyAbility, PlaneswalkerSubtype, StaticAbility, StaticEffect, TokenDefinition,
    };
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Elspeth, Storm Slayer",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        base_loyalty: 5,
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, twice that many are created instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        keyword: Keyword::Flying,
                        duration: Duration::UntilNextTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(SelectionRequirement::ManaValueAtLeast(3)),
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Betor, Kin to All — {2}{W}{B}{G} 5/7 Spirit Dragon. Flying. At your end step:
/// if your creatures' total toughness ≥10 draw a card; then ≥20 untap each
/// creature you control; then ≥40 each opponent loses half their life, rounded
/// up.
pub fn betor_kin_to_all() -> CardDefinition {
    CardDefinition {
        name: "Betor, Kin to All",
        cost: cost(&[generic(2), w(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(10)),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(20)),
                    then: Box::new(Effect::Untap {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        up_to: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(40)),
                    then: Box::new(Effect::LoseHalfLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        rounded_up: true,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mistmoon Griffin — {3}{W} 2/2 Griffin. Flying. When it dies, return the top
/// creature card of your graveyard to the battlefield.
pub fn mistmoon_griffin() -> CardDefinition {
    CardDefinition {
        name: "Mistmoon Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::ReturnTopCreatureFromGraveyard {
            who: PlayerRef::You,
        })],
        ..Default::default()
    }
}

/// Dalek Squadron — {2}{B} 3/3 Artifact Dalek. Menace, myriad.
pub fn dalek_squadron() -> CardDefinition {
    CardDefinition {
        name: "Dalek Squadron",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dalek], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Myriad,
        }],
        ..Default::default()
    }
}

/// Perennation — {3}{W}{B}{G} Sorcery. Return target permanent card from your
/// graveyard to the battlefield with a hexproof counter and an indestructible
/// counter on it.
pub fn perennation() -> CardDefinition {
    CardDefinition {
        name: "Perennation",
        cost: cost(&[generic(3), w(), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Karakyk Guardian — {3}{G}{U}{R} 6/5 Dragon. Flying, vigilance, trample.
/// (Its conditional hexproof-while-it-hasn't-dealt-damage rider is omitted —
/// no lifetime damage-dealt tracking yet.)
pub fn karakyk_guardian() -> CardDefinition {
    CardDefinition {
        name: "Karakyk Guardian",
        cost: cost(&[generic(3), g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        ..Default::default()
    }
}

/// Sarkhan, Soul Aflame — {1}{U}{R} 2/4 Human Shaman. Dragon spells you cast
/// cost {1} less. Whenever a Dragon you control enters, you may have Sarkhan
/// become a copy of it until end of turn. (The copy keeps the Dragon's name —
/// the printed "name stays Sarkhan" override is approximated.)
pub fn sarkhan_soul_aflame() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Sarkhan, Soul Aflame",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Dragon spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                }),
            effect: Effect::MayDo {
                description: "have Sarkhan become a copy of that Dragon until end of turn".into(),
                body: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Recent-set batch 2 (claude/modern_decks) ─────────────────────────────────

/// Skirmish Rhino — {W}{B}{G} 3/4 Rhino. Trample. ETB: each opponent loses 2
/// life and you gain 2 life.
pub fn skirmish_rhino() -> CardDefinition {
    CardDefinition {
        name: "Skirmish Rhino",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rhino], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Rabid Gnaw — {1}{R} Instant. Target creature you control gets +1/+0 until end
/// of turn, then deals damage equal to its power to target creature you don't
/// control.
pub fn rabid_gnaw() -> CardDefinition {
    CardDefinition {
        name: "Rabid Gnaw",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Reckless Lackey — {R} 1/2 Goblin Pirate. First strike, haste. {2}{R},
/// Sacrifice this: draw a card and create a Treasure token.
pub fn reckless_lackey() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Reckless Lackey",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lunar Convocation — {W}{B} Enchantment. At your end step: if you gained life
/// this turn, each opponent loses 1 life; if you also lost life this turn,
/// create a 1/1 black Bat with flying.
pub fn lunar_convocation() -> CardDefinition {
    use crate::card::TokenDefinition;
    let bat = TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Lunar Convocation",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::LifeGainedThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::LifeGainedThisTurnAtLeast {
                            who: PlayerRef::You,
                            at_least: Value::Const(1),
                        },
                        Predicate::PlayerLostLifeThisTurn { who: PlayerRef::You },
                    ]),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: bat,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dazzling Denial — {1}{U} Instant. Counter target spell unless its controller
/// pays {2} — or {4} instead if you control a Bird.
pub fn dazzling_denial() -> CardDefinition {
    CardDefinition {
        name: "Dazzling Denial",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Bird)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[generic(4)]),
                exile: false,
                extra_generic: None,
            }),
            else_: Box::new(Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            }),
        },
        ..Default::default()
    }
}

/// Cori Mountain Monastery — Land. Enters tapped unless you control a Plains or
/// an Island. {T}: Add {R}. {3}{R}, {T}: exile the top card of your library;
/// you may play it until the end of your next turn.
pub fn cori_mountain_monastery() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Cori Mountain Monastery",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![land_tapped_unless(LandType::Plains, LandType::Island)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r()]),
                tap_cost: true,
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mistrise Village — Land. Enters tapped unless you control a Mountain or a
/// Forest. {T}: Add {U}. {U}, {T}: your spells can't be countered this turn.
/// (The printed "next spell" scope is approximated as all your spells.)
pub fn mistrise_village() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Mistrise Village",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![land_tapped_unless(LandType::Mountain, LandType::Forest)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Blue, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::GrantSpellsUncounterableThisTurn { who: Selector::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// "Enters tapped unless you control a land of `type_a` or `type_b`" — the
/// check-land ETB conditional reused by recent dual-color utility lands.
fn land_tapped_unless(type_a: crate::card::LandType, type_b: crate::card::LandType) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::HasLandType(type_a)
                        .or(SelectionRequirement::HasLandType(type_b))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Tap { what: Selector::This }),
        },
    }
}

/// Bloodletter of Aclazotz — {1}{B}{B}{B} 2/4 Vampire Demon. Flying. If an
/// opponent would lose life during your turn, they lose twice that much instead.
pub fn bloodletter_of_aclazotz() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Bloodletter of Aclazotz",
        cost: cost(&[generic(1), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Demon],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would lose life during your turn, they lose twice that much life instead.",
            effect: StaticEffect::OpponentLifeLossDoubledDuringYourTurn,
        }],
        ..Default::default()
    }
}

/// Touch the Spirit Realm — {2}{W} Enchantment. ETB: exile up to one target
/// artifact or creature until this leaves. (The Channel discard-mode is omitted.)
pub fn touch_the_spirit_realm() -> CardDefinition {
    use crate::card::{CardType as CT, ExileReturnZone};
    CardDefinition {
        name: "Touch the Spirit Realm",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::HasCardType(CT::Artifact)),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Sonar Strike — {1}{W} Instant. Deals 4 damage to target attacking, blocking,
/// or tapped creature; gain 3 life if you control a Bat.
pub fn sonar_strike() -> CardDefinition {
    CardDefinition {
        name: "Sonar Strike",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::IsAttacking
                            .or(SelectionRequirement::IsBlocking)
                            .or(SelectionRequirement::Tapped),
                    ),
                ),
                amount: Value::Const(4),
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Bat)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Aerie Auxiliary — {3}{W} 3/3 Bird Soldier. Flying. ETB: support 2 (put a
/// +1/+1 counter on each of up to two other target creatures).
pub fn aerie_auxiliary() -> CardDefinition {
    CardDefinition {
        name: "Aerie Auxiliary",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(crate::effect::shortcut::support(2))],
        ..Default::default()
    }
}

/// Loran's Escape — {W} Instant. Target artifact or creature gains hexproof and
/// indestructible until end of turn. Scry 1.
pub fn lorans_escape() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Loran's Escape",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::HasCardType(CT::Artifact)),
                ),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Dauntless Veteran — {1}{W}{W} 2/2 Human Soldier. Whenever it attacks,
/// creatures you control get +1/+1 until end of turn.
pub fn dauntless_veteran() -> CardDefinition {
    CardDefinition {
        name: "Dauntless Veteran",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Spectral Denial — {X}{U} Instant. Costs {1} less for each creature you
/// control with power 4 or greater. Counter target spell unless its controller
/// pays {X}.
pub fn spectral_denial() -> CardDefinition {
    CardDefinition {
        name: "Spectral Denial",
        cost: cost(&[crate::mana::x(), u()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::Creature
                .and(SelectionRequirement::PowerAtMost(3).negate())
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::CounterUnlessPaid {
            what: crate::effect::shortcut::target(),
            mana_cost: crate::mana::ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::XFromCost),
        },
        ..Default::default()
    }
}

/// Glistener Seer — {U} 0/3 Phyrexian Advisor. Enters with three oil counters.
/// {T}, Remove an oil counter: scry 1.
pub fn glistener_seer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Glistener Seer",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Advisor],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        enters_with_counters: Some((CounterType::Oil, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 1)),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vengeful Bloodwitch — {1}{B} 1/1 Vampire Warlock. Whenever this or another
/// creature you control dies, an opponent loses 1 life and you gain 1.
pub fn vengeful_bloodwitch() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Bloodwitch",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Hulking Raptor — {2}{G}{G} 5/3 Dinosaur. Ward {2}. At your first main phase,
/// add {G}{G}.
pub fn hulking_raptor() -> CardDefinition {
    use crate::card::WardCost;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Hulking Raptor",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::PreCombatMain),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(crate::mana::Color::Green, Value::Const(2)),
            },
        }],
        ..Default::default()
    }
}

// ── DFT "Start your engines!" / speed (CR 702.179) ──────────────────────────

/// Nesting Bot — {W} 1/1 Robot artifact creature. Start your engines! When it
/// dies, create a 1/1 colorless Servo artifact creature token. Max speed — it
/// gets +1/+0.
pub fn nesting_bot() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, TokenDefinition};
    CardDefinition {
        name: "Nesting Bot",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Servo".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Artifact, CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
                ..Default::default()
            },
        })],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Nesting Bot gets +1/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Burnout Bashtronaut — {R} 1/1 Goblin Warrior. Menace, Start your engines!
/// {2}: it gets +1/+0 until end of turn. Max speed — it has double strike.
pub fn burnout_bashtronaut() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Burnout Bashtronaut",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Burnout Bashtronaut has double strike.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
            },
        }],
        ..Default::default()
    }
}

/// Swiftwing Assailant — {3}{W} 3/3 Bird Warrior. Flying, Start your engines!
/// Max speed — it gets +0/+1 and has vigilance.
pub fn swiftwing_assailant() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Swiftwing Assailant",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Swiftwing Assailant gets +0/+1 and has vigilance.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 1,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        ..Default::default()
    }
}

/// Risen Necroregent — {4}{B} 5/4 Zombie Cat Knight. Start your engines! Max
/// speed — at the beginning of your end step, create a 2/2 black Zombie token.
pub fn risen_necroregent() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Risen Necroregent",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cat, CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Embalmed Ascendant — {1}{W}{B} 1/2 Zombie. Start your engines! When it
/// enters, create a 2/2 black Zombie token. Max speed — whenever a creature you
/// control dies, each opponent loses 1 life and you gain 1 life.
pub fn embalmed_ascendant() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Embalmed Ascendant",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
                    ..Default::default()
                },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Walking Sarcophagus — {2} 2/1 Zombie Cat artifact creature. Start your
/// engines! Max speed — it gets +1/+2.
pub fn walking_sarcophagus() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Walking Sarcophagus",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Walking Sarcophagus gets +1/+2.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Streaking Oilgorger — {4}{B} 3/3 Vampire. Flying, haste, Start your engines!
/// Max speed — it has lifelink.
pub fn streaking_oilgorger() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Streaking Oilgorger",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::StartYourEngines],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Streaking Oilgorger has lifelink.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Goblin Surveyor — {2}{R} 3/2 Goblin Scout. Trample, Start your engines! Max
/// speed — {3}, Exile this card from your graveyard: Draw a card.
pub fn goblin_surveyor() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Goblin Surveyor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            from_graveyard: true,
            exile_self_cost: true,
            condition: Some(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gastal Thrillseeker — {B}{R} 2/3 Lizard Berserker. Start your engines! When
/// it enters, deal 1 damage to each opponent (printed "target opponent",
/// 1v1-faithful) and you gain 1 life. Max speed — it has deathtouch and haste.
pub fn gastal_thrillseeker() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Gastal Thrillseeker",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Max speed — Gastal Thrillseeker has deathtouch and haste.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Deathtouch, Keyword::Haste],
            },
        }],
        ..Default::default()
    }
}

// ── Recover (Coldsnap, CR 702.58) ───────────────────────────────────────────

/// Grim Harvest — {1}{B} Instant. Return target creature card from your
/// graveyard to your hand. Recover {2}{B}. (The main effect's graveyard zone
/// filter is dropped per the Disentomb convention.)
pub fn grim_harvest() -> CardDefinition {
    CardDefinition {
        name: "Grim Harvest",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        triggered_abilities: vec![recover(cost(&[generic(2), b()]))],
        ..Default::default()
    }
}

/// Sun's Bounty — {1}{W} Instant. You gain 4 life. Recover {1}{W}.
pub fn suns_bounty() -> CardDefinition {
    CardDefinition {
        name: "Sun's Bounty",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        triggered_abilities: vec![recover(cost(&[generic(1), w()]))],
        ..Default::default()
    }
}

/// Icefall — {2}{R}{R} Sorcery. Destroy target artifact or land. Recover {R}{R}.
pub fn icefall() -> CardDefinition {
    CardDefinition {
        name: "Icefall",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Land),
            ),
        },
        triggered_abilities: vec![recover(cost(&[r(), r()]))],
        ..Default::default()
    }
}

/// Resize — {1}{G} Instant. Target creature gets +3/+3 until end of turn.
/// Recover {1}{G}.
pub fn resize() -> CardDefinition {
    CardDefinition {
        name: "Resize",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(3),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![recover(cost(&[generic(1), g()]))],
        ..Default::default()
    }
}

// ── More recent-set staples ─────────────────────────────────────────────────

/// Bloodthirsty Conqueror — {3}{B}{B} 5/5 Vampire Knight. Flying, deathtouch.
/// Whenever an opponent loses life, you gain that much life.
pub fn bloodthirsty_conqueror() -> CardDefinition {
    CardDefinition {
        name: "Bloodthirsty Conqueror",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    }
}

/// Razorkin Needlehead — {R}{R} 2/2 Human Assassin. First strike during your
/// turn. Whenever an opponent draws a card, it deals 1 damage to them.
pub fn razorkin_needlehead() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Razorkin Needlehead",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Razorkin Needlehead has first strike during your turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Savor — {1}{B} Instant. Target creature gets -2/-2 until end of turn; create
/// a Food token.
pub fn savor() -> CardDefinition {
    CardDefinition {
        name: "Savor",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::food_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Spinewoods Armadillo — {4}{G}{G} 7/7 Armadillo. Reach, ward {3}. {1}{G},
/// Discard this card: Search your library for a basic land or Desert card, put
/// it into your hand, then shuffle. You gain 3 life.
pub fn spinewoods_armadillo() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType, WardCost};
    CardDefinition {
        name: "Spinewoods Armadillo",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Armadillo], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Reach, Keyword::Ward(WardCost::generic(3))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand
                        .or(SelectionRequirement::HasLandType(LandType::Desert)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Screaming Nemesis — {2}{R} 3/3 Spirit. Haste. Whenever it's dealt damage, it
/// deals that much damage to any target; if a player is dealt damage this way,
/// they can't gain life for the rest of the game.
pub fn screaming_nemesis() -> CardDefinition {
    CardDefinition {
        name: "Screaming Nemesis",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Any),
                    amount: Value::TriggerEventAmount,
                },
                // No-op unless the target was a player (CR 119.7 rest-of-game lock).
                Effect::LifeGainLockGame { who: Selector::Target(0) },
            ]),
        }],
        ..Default::default()
    }
}

/// Goblin Boarders — {2}{R} 3/2 Goblin Pirate. Raid — enters with a +1/+1
/// counter if you attacked this turn.
pub fn goblin_boarders() -> CardDefinition {
    CardDefinition {
        name: "Goblin Boarders",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Cogwork Wrestler — {U} 1/2 Gnome artifact creature. Flash. When it enters,
/// target creature an opponent controls gets -2/-0 until end of turn.
pub fn cogwork_wrestler() -> CardDefinition {
    CardDefinition {
        name: "Cogwork Wrestler",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Crocodile of the Crossing — {3}{G} 5/4 Crocodile. Haste. When it enters, put
/// a -1/-1 counter on target creature you control.
pub fn crocodile_of_the_crossing() -> CardDefinition {
    CardDefinition {
        name: "Crocodile of the Crossing",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Crocodile], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::MinusOneMinusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Topiary Stomper — {1}{G}{G} 4/4 Plant Dinosaur. Vigilance. Can't attack or
/// block unless you control seven or more lands. When it enters, search your
/// library for a basic land and put it onto the battlefield tapped.
pub fn topiary_stomper() -> CardDefinition {
    CardDefinition {
        name: "Topiary Stomper",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::CantAttackOrBlockUnlessYouControlCount {
                filter: Box::new(SelectionRequirement::Land),
                min: 7,
            },
        ],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        ..Default::default()
    }
}

/// Cache Grab — {1}{G} Instant. Mill four, then you may put a permanent card
/// milled this way into your hand. If you control a Squirrel, create a Food.
/// (The "returned a Squirrel this way" half of the Food trigger is approximated
/// to controlling one.)
pub fn cache_grab() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Cache Grab",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MillThenToHand {
                amount: Value::Const(4),
                filter: SelectionRequirement::PermanentCard,
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Squirrel),
                }),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::food_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Lumbering Worldwagon — {2}{G} Vehicle `*`/4. Power = lands you control.
/// Whenever it enters or attacks, you may fetch a basic land tapped. Crew 4.
pub fn lumbering_worldwagon() -> CardDefinition {
    use crate::card::{ArtifactSubtype, DynamicPt};
    let fetch = || Effect::MayDo {
        description: "Search for a basic land, put it onto the battlefield tapped".into(),
        body: Box::new(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        }),
    };
    CardDefinition {
        name: "Lumbering Worldwagon",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 0,
        toughness: 4,
        dynamic_pt: Some(DynamicPt::LandsControlledPower { base_p: 0, base_t: 4 }),
        keywords: vec![Keyword::Crew(4)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: fetch(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: fetch(),
            },
        ],
        ..Default::default()
    }
}

/// Bakersbane Duo — {1}{G} 2/2 Squirrel Raccoon. When it enters, create a Food
/// token. Whenever you expend 4, it gets +1/+1 until end of turn.
pub fn bakersbane_duo() -> CardDefinition {
    CardDefinition {
        name: "Bakersbane Duo",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Raccoon],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::food_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                    .with_filter(Predicate::ExpendReached(4)),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Spire Mangler — {2}{B} 2/1 Insect. Flash, flying. When it enters, target
/// creature you control with flying gets +2/+0 until end of turn.
pub fn spire_mangler() -> CardDefinition {
    CardDefinition {
        name: "Spire Mangler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Palace Familiar — {1}{U} 1/1 Bird. Flying; when it dies, draw a card.
pub fn palace_familiar() -> CardDefinition {
    CardDefinition {
        name: "Palace Familiar",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Symbiotic Elf — {3}{G} 2/2 Elf. When it dies, create two 1/1 green Insect
/// creature tokens.
pub fn symbiotic_elf() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let insect = TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Symbiotic Elf",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: insect,
        })],
        ..Default::default()
    }
}

/// Bear's Companion — {2}{G}{U}{R} 2/2 Human Warrior. When it enters, create a
/// 4/4 green Bear creature token.
pub fn bears_companion() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let bear = TokenDefinition {
        name: "Bear".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Bear's Companion",
        cost: cost(&[generic(2), g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: bear,
        })],
        ..Default::default()
    }
}

/// Grasping Thrull — {3}{W}{B} 3/3 Thrull. Flying; when it enters, it deals 2
/// damage to each opponent and you gain 2 life.
pub fn grasping_thrull() -> CardDefinition {
    CardDefinition {
        name: "Grasping Thrull",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thrull], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Hero of Precinct One — {1}{W} 2/2 Human Warrior. Whenever you cast a
/// multicolored spell, create a 1/1 white Human creature token.
pub fn hero_of_precinct_one() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let human = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Hero of Precinct One",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Multicolored,
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: human,
            },
        }],
        ..Default::default()
    }
}

/// Havoc Devils — {2}{R}{R} 4/3 Devil with trample.
pub fn havoc_devils() -> CardDefinition {
    CardDefinition {
        name: "Havoc Devils",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Hollow Dogs — {4}{B} 3/3 Phyrexian Zombie Dog. Whenever it attacks, it gets
/// +2/+0 until end of turn.
pub fn hollow_dogs() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Hollow Dogs",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Argothian Enchantress — {1}{G} 0/1 Human Druid. Shroud; whenever you cast an
/// enchantment spell, draw a card.
pub fn argothian_enchantress() -> CardDefinition {
    CardDefinition {
        name: "Argothian Enchantress",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Enchantment),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Patrol Hound — {1}{W} 2/2 Dog. "Discard a card: This creature gains first
/// strike until end of turn."
pub fn patrol_hound() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Patrol Hound",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Canyon Wildcat — {1}{R} 2/1 Cat with mountainwalk.
pub fn canyon_wildcat() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Canyon Wildcat",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..Default::default()
    }
}

/// Squirrelanoids — {B} 1/1 Squirrel Mutant with deathtouch.
pub fn squirrelanoids() -> CardDefinition {
    CardDefinition {
        name: "Squirrelanoids",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Mutant],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Vile Deacon — {2}{B}{B} 2/2 Human Cleric. Whenever it attacks, it gets +X/+X
/// until end of turn, where X is the number of Clerics on the battlefield.
pub fn vile_deacon() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    let clerics = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::Creature)),
        filter: SelectionRequirement::HasCreatureType(CreatureType::Cleric),
    };
    CardDefinition {
        name: "Vile Deacon",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: clerics.clone(),
            toughness: clerics,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Mischievous Mystic — {1}{U} 2/1 Human Wizard. Flying; whenever you draw your
/// second card each turn, create a 1/1 blue Faerie creature token with flying.
pub fn mischievous_mystic() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let faerie = TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Mischievous Mystic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(
                Predicate::ValueEquals(
                    Value::CardsDrawnThisTurn(PlayerRef::You),
                    Value::Const(2),
                ),
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: faerie,
            },
        }],
        ..Default::default()
    }
}

/// Dawn's Light Archer — {2}{G} 4/2 Elf Archer with flash and reach.
pub fn dawns_light_archer() -> CardDefinition {
    CardDefinition {
        name: "Dawn's Light Archer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Archer],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Reach],
        ..Default::default()
    }
}

/// Plumeveil — {W/U}{W/U}{W/U} 4/4 Elemental with flash, defender, and flying.
pub fn plumeveil() -> CardDefinition {
    use crate::mana::{hybrid, Color};
    CardDefinition {
        name: "Plumeveil",
        cost: cost(&[
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Defender, Keyword::Flying],
        ..Default::default()
    }
}

/// Rooftop Assassin — {3}{B} 2/2 Vampire Assassin. Flash, flying, lifelink. When
/// it enters, destroy target creature an opponent controls that was dealt
/// damage this turn.
pub fn rooftop_assassin() -> CardDefinition {
    CardDefinition {
        name: "Rooftop Assassin",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::DealtDamageThisTurn),
            ),
        })],
        ..Default::default()
    }
}

/// Spellgorger Barbarian — {3}{R} 3/1 Human Nightmare Barbarian. When it enters,
/// discard a card at random. When it leaves the battlefield, draw a card.
pub fn spellgorger_barbarian() -> CardDefinition {
    CardDefinition {
        name: "Spellgorger Barbarian",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Nightmare,
                CreatureType::Barbarian,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: true,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Bog Gnarr — {4}{G} 2/2 Beast. Whenever a player casts a black spell, it gets
/// +2/+2 until end of turn.
pub fn bog_gnarr() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Bog Gnarr",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasColor(Color::Black),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Elf Replica — {3} 2/2 artifact Elf. "{1}{G}, Sacrifice this creature:
/// Destroy target enchantment."
pub fn elf_replica() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Elf Replica",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasCardType(CardType::Enchantment)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seismic Mage — {3}{R} 1/1 Human Spellshaper. "{2}{R}, {T}, Discard a card:
/// Destroy target land."
pub fn seismic_mage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Seismic Mage",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Spellshaper],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::Land),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Etched Oracle — {4} 0/0 artifact Wizard. Sunburst; "{1}, Remove four +1/+1
/// counters from this creature: Target player draws three cards."
pub fn etched_oracle() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Etched Oracle",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 4)),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skyreach Manta — {5} 0/0 artifact Fish. Sunburst; flying.
pub fn skyreach_manta() -> CardDefinition {
    CardDefinition {
        name: "Skyreach Manta",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        ..Default::default()
    }
}

/// Phyrexian Digester — {3} 2/1 artifact Phyrexian Construct with infect.
pub fn phyrexian_digester() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Digester",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Infect],
        ..Default::default()
    }
}

/// Blackcleave Goblin — {3}{B} 2/1 Phyrexian Goblin Zombie with haste and infect.
pub fn blackcleave_goblin() -> CardDefinition {
    CardDefinition {
        name: "Blackcleave Goblin",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin, CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Infect],
        ..Default::default()
    }
}

/// Essence Depleter — {2}{B} 2/3 Eldrazi Drone. Devoid; "{1}{C}: Target opponent
/// loses 1 life and you gain 1 life."
pub fn essence_depleter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Essence Depleter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), colorless(1)]),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stormclaw Rager — {1}{B}{R} 2/2 Ogre Warrior. "{1}, Sacrifice another creature
/// or artifact: Put a +1/+1 counter on this creature and draw a card. Activate
/// only as a sorcery."
pub fn stormclaw_rager() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Stormclaw Rager",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wave Elemental — {2}{U}{U} 2/3 Elemental. "{U}, {T}, Sacrifice this creature:
/// Tap up to three target creatures without flying."
pub fn wave_elemental() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Wave Elemental",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::TapUpToValue {
                count: Value::Const(3),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasKeyword(
                        Keyword::Flying,
                    )))),
                skip_untap: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shipwreck Moray — {3}{U} 0/5 Fish. When it enters, you get four energy. "Pay
/// {E}: This creature gets +2/-2 until end of turn."
pub fn shipwreck_moray() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Shipwreck Moray",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        power: 0,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(4)))],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 1,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Argothian Sprite — {1}{G} 2/2 Faerie. Can't be blocked by artifact creatures;
/// "{7}: Put two +1/+1 counters on this creature."
pub fn argothian_sprite() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Argothian Sprite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::HasCardType(
            CardType::Artifact,
        )))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
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

/// Nadier's Nightblade — {2}{B} 1/3 Elf Warrior. Whenever a token you control
/// leaves the battlefield, each opponent loses 1 life and you gain 1 life.
pub fn nadiers_nightblade() -> CardDefinition {
    CardDefinition {
        name: "Nadier's Nightblade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::IsToken,
                }),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Gnarlroot Pallbearer — {4}{G}{G} 5/5 Treefolk Druid. Trample; when it enters,
/// target creature gets +X/+X until end of turn, where X is the number of
/// creature cards in your graveyard.
pub fn gnarlroot_pallbearer() -> CardDefinition {
    let gy_creatures = Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: SelectionRequirement::Creature,
    };
    CardDefinition {
        name: "Gnarlroot Pallbearer",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: gy_creatures.clone(),
            toughness: gy_creatures,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Illusionary Servant — {1}{U}{U} 3/4 Illusion. Flying; when it becomes the
/// target of a spell or ability, sacrifice it.
pub fn illusionary_servant() -> CardDefinition {
    CardDefinition {
        name: "Illusionary Servant",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Illusion], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Graveyard },
        }],
        ..Default::default()
    }
}

/// Bounding Wolf — {2}{G} 3/2 Wolf with flash and reach.
pub fn bounding_wolf() -> CardDefinition {
    CardDefinition {
        name: "Bounding Wolf",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Reach],
        ..Default::default()
    }
}

/// Goblin Sky Raider — {2}{R} 1/2 Goblin Warrior with flying.
pub fn goblin_sky_raider() -> CardDefinition {
    CardDefinition {
        name: "Goblin Sky Raider",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Glowing Anemone — {3}{U} 1/3 Jellyfish Beast. When it enters, you may return
/// target land to its owner's hand.
pub fn glowing_anemone() -> CardDefinition {
    CardDefinition {
        name: "Glowing Anemone",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return target land to its owner's hand".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(SelectionRequirement::Land),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..Default::default()
    }
}

/// Contraband Kingpin — {U}{B} 1/4 Aetherborn Rogue. Lifelink; whenever an
/// artifact you control enters, scry 1.
pub fn contraband_kingpin() -> CardDefinition {
    CardDefinition {
        name: "Contraband Kingpin",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Aetherborn, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                },
            ),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Kingpin's Enforcers — {2}{B} 2/3 Human Villain. Lifelink; "{2}{B}, Sacrifice
/// an artifact or creature: Draw a card."
pub fn kingpins_enforcers() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Kingpin's Enforcers",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Villain],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                1,
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goldmaw Champion — {2}{W} 2/3 Dwarf Warrior. Boast — {1}{W}: Tap target
/// creature.
pub fn goldmaw_champion() -> CardDefinition {
    use crate::effect::shortcut::boast;
    CardDefinition {
        name: "Goldmaw Champion",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![boast(
            cost(&[generic(1), w()]),
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
        )],
        ..Default::default()
    }
}

/// Gold Myr — {2} 1/1 artifact Myr. "{T}: Add {W}."
pub fn gold_myr() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Gold Myr",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![crate::sets::tap_add(Color::White)],
        ..Default::default()
    }
}

/// Drumhunter — {3}{G} 2/2 Human Druid Warrior. At the beginning of your end
/// step, if you control a creature with power 5 or greater, you may draw a card.
/// "{T}: Add {C}."
pub fn drumhunter() -> CardDefinition {
    CardDefinition {
        name: "Drumhunter",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(5)),
            })),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}


/// Roc of Kher Ridges — {3}{R} 3/3 Bird with flying.
pub fn roc_of_kher_ridges() -> CardDefinition {
    CardDefinition {
        name: "Roc of Kher Ridges",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Minotaur Aggressor — {6}{R} 6/2 Minotaur Berserker with first strike and haste.
pub fn minotaur_aggressor() -> CardDefinition {
    CardDefinition {
        name: "Minotaur Aggressor",
        cost: cost(&[generic(6), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Berserker],
            ..Default::default()
        },
        power: 6,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        ..Default::default()
    }
}

/// Malakir Familiar — {2}{B} 2/1 Bat. Flying, deathtouch; whenever you gain
/// life, it gets +1/+1 until end of turn.
pub fn malakir_familiar() -> CardDefinition {
    CardDefinition {
        name: "Malakir Familiar",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Mercurial Geists — {2}{U}{R} 1/3 Spirit. Flying; whenever you cast an instant
/// or sorcery spell, it gets +3/+0 until end of turn.
pub fn mercurial_geists() -> CardDefinition {
    CardDefinition {
        name: "Mercurial Geists",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Engine Rat — {B} 1/1 Zombie Rat. Deathtouch; "{5}{B}: Each opponent loses 2
/// life."
pub fn engine_rat() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Engine Rat",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gavony Silversmith — {3}{W} 2/3 Human Soldier. When it enters, put a +1/+1
/// counter on each of up to two target creatures.
pub fn gavony_silversmith() -> CardDefinition {
    CardDefinition {
        name: "Gavony Silversmith",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Reputable Merchant — {2/W}{2/B}{2/G} 2/2 Human Citizen. When it enters or
/// dies, put a +1/+1 counter on target creature you control.
pub fn reputable_merchant() -> CardDefinition {
    use crate::mana::{mono_hybrid, Color};
    let counter = || Effect::AddCounter {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Reputable Merchant",
        cost: cost(&[
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(counter()), on_dies(counter())],
        ..Default::default()
    }
}

/// Withering Torment — {2}{B} Instant. Destroy target creature or enchantment;
/// you lose 2 life.
pub fn withering_torment() -> CardDefinition {
    CardDefinition {
        name: "Withering Torment",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Voltage Surge — {R} Instant. You may sacrifice an artifact as an additional
/// cost; deals 2 damage to target creature or planeswalker, or 4 if you did.
/// (The additional cost is taken at resolution — a `MayDo` sacrifice.)
pub fn voltage_surge() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Voltage Surge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MayDo {
                description: "Sacrifice an artifact for extra damage".into(),
                body: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Artifact,
                }),
            },
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Corpse Appraiser — {U}{B}{R} 3/3 Vampire Rogue. ETB exile a creature card
/// from a graveyard, then dig three (one to hand, the rest to your graveyard).
/// (The "only if a card was exiled" gate is approximated — the dig is
/// unconditional.)
pub fn corpse_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Corpse Appraiser",
        cost: cost(&[u(), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Exile,
            },
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: true,
                pick_filter: None,
                take: None,
                to_battlefield: false,
            },
        ]))],
        ..Default::default()
    }
}

/// The Wandering Rescuer — {3}{W}{W} 3/4 Legendary Human Samurai Noble. Flash,
/// Convoke, Double strike. Other tapped creatures you control have hexproof.
pub fn the_wandering_rescuer() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "The Wandering Rescuer",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Convoke, Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "Other tapped creatures you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Tapped)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Hexproof,
            },
        }],
        ..Default::default()
    }
}

/// Light Up the Night — {X}{R} Sorcery. Deals X damage to any target — X+1 if
/// that target is a creature or planeswalker. (Flashback via remove-loyalty is
/// omitted — no loyalty-removal alt-cost primitive.)
pub fn light_up_the_night() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Light Up the Night",
        cost: cost(&[crate::mana::x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::XFromCost,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Tyrant's Scorn — {U}{B} Instant. Destroy a creature with mana value 3 or
/// less; or return target creature to its owner's hand.
pub fn tyrants_scorn() -> CardDefinition {
    CardDefinition {
        name: "Tyrant's Scorn",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
        ..Default::default()
    }
}

/// Fang of Shigeki — {G} 1/1 Snake Ninja Enchantment Creature with deathtouch.
pub fn fang_of_shigeki() -> CardDefinition {
    CardDefinition {
        name: "Fang of Shigeki",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Ninja],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}




/// Lifecraft Cavalry — {4}{G} 4/4 Elf Warrior with trample. Revolt — enters
/// with two +1/+1 counters if a permanent left the battlefield under your
/// control this turn.
pub fn lifecraft_cavalry() -> CardDefinition {
    CardDefinition {
        name: "Lifecraft Cavalry",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::revolt_etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Workshop Warchief — {3}{G}{G} 5/3 Rhino Warrior with trample. ETB gain 3
/// life; dies → make a 4/4 green Rhino Warrior. Blitz {4}{G}{G}.
pub fn workshop_warchief() -> CardDefinition {
    use crate::card::TokenDefinition;
    let rhino = TokenDefinition {
        name: "Rhino".into(), power: 4, toughness: 4,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Green],
        keywords: vec![Keyword::Trample],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Workshop Warchief",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(crate::effect::shortcut::blitz(cost(&[generic(4), g(), g()]))),
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            on_dies(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: rhino }),
        ],
        ..Default::default()
    }
}

/// Prosperous Innkeeper — {1}{G} 1/1 Halfling Citizen. ETB create a Treasure.
/// Whenever another creature you control enters, gain 1 life.
pub fn prosperous_innkeeper() -> CardDefinition {
    CardDefinition {
        name: "Prosperous Innkeeper",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Halfling, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Jadar, Ghoulcaller of Nephalia — {1}{B} 1/1 Legendary Human Wizard. At your
/// end step, if you control no creature with decayed, create a 2/2 black Zombie
/// with decayed.
pub fn jadar_ghoulcaller_of_nephalia() -> CardDefinition {
    let zombie = crate::card::TokenDefinition {
        name: "Zombie".into(), power: 2, toughness: 2,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Jadar, Ghoulcaller of Nephalia",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasKeyword(Keyword::Decayed),
                },
            )))),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: zombie },
        }],
        ..Default::default()
    }
}

/// The Goose Mother — {X}{G}{U} 2/2 Legendary Bird Hydra. Flying. Enters with X
/// +1/+1 counters and makes half X Food (rounded up). Attack: you may sacrifice
/// a Food to draw a card.
pub fn the_goose_mother() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "The Goose Mother",
        cost: cost(&[crate::mana::x(), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Hydra],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::HalvedRoundUp(Box::new(Value::XFromCost)),
                definition: crabomination_base::tokens::food_token(),
            }),
            crate::effect::shortcut::on_attack(Effect::MayDo {
                description: "Sacrifice a Food to draw a card".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Food),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Archangel of Wrath — {2}{W}{W} 3/4 Angel. Flying, lifelink. Kicker (multi,
/// approximated from the printed {B} and/or {R}): when it enters, deals 2 damage
/// to any target for each time it was kicked (up to twice).
pub fn archangel_of_wrath() -> CardDefinition {
    CardDefinition {
        name: "Archangel of Wrath",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Multikicker(cost(&[r()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::TimesKicked, Value::Const(1))),
                effect: Effect::DealDamage { to: target_filtered(SelectionRequirement::Any), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::TimesKicked, Value::Const(2))),
                effect: Effect::DealDamage { to: target_filtered(SelectionRequirement::Any), amount: Value::Const(2) },
            },
        ],
        ..Default::default()
    }
}

/// Ascendant Packleader — {G} 2/1 Wolf. Enters with a +1/+1 counter if you
/// control a permanent with mana value 4+; gains a counter when you cast a
/// spell with mana value 4 or greater.
pub fn ascendant_packleader() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Ascendant Packleader",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::ManaValueAtLeast(4),
                }),
                then: Box::new(Effect::AddCounter {
                    what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(SelectionRequirement::ManaValueAtLeast(4))),
                effect: Effect::AddCounter {
                    what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Persistent Specimen — {B} 1/1 Skeleton. {2}{B}: Return this from your
/// graveyard to the battlefield tapped.
pub fn persistent_specimen() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Persistent Specimen",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wedding Invitation — {2} Artifact. ETB draw a card. {T}, Sacrifice: target
/// creature can't be blocked this turn; if it's a Vampire it also gains lifelink.
pub fn wedding_invitation() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType};
    use crate::effect::Predicate;
    CardDefinition {
        name: "Wedding Invitation",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Vampire),
                    },
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unlucky Witness — {R} 1/1 Human Citizen. When it dies, exile the top two
/// cards of your library; until your next end step, you may play one of them.
pub fn unlucky_witness() -> CardDefinition {
    CardDefinition {
        name: "Unlucky Witness",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Citizen], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Squee, Dubious Monarch — {2}{R} 2/2 Legendary Goblin Noble. Haste. Attacks →
/// make a tapped, attacking 1/1 Goblin. Escape {3}{R}, exile four other cards.
pub fn squee_dubious_monarch() -> CardDefinition {
    let goblin = crate::card::TokenDefinition {
        name: "Goblin".into(), power: 1, toughness: 1,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Squee, Dubious Monarch",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Noble], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Escape(cost(&[generic(3), r()]), 4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: goblin,
                cleanup: crate::effect::AttackingTokenCleanup::None,
            },
        }],
        ..Default::default()
    }
}

/// A 2/2 black Zombie with decayed (shared by several Innistrad makers).
fn decayed_zombie_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Zombie".into(), power: 2, toughness: 2,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

/// A plain 2/2 black Zombie token.
fn black_zombie_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Zombie".into(), power: 2, toughness: 2,
        card_types: vec![CardType::Creature], colors: vec![crate::mana::Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}



/// Headless Rider — {2}{B} 3/1 Zombie. Whenever this or another nontoken Zombie
/// you control dies, create a 2/2 black Zombie.
pub fn headless_rider() -> CardDefinition {
    let make = || Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: black_zombie_token() };
    CardDefinition {
        name: "Headless Rider",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            on_dies(make()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                            .and(SelectionRequirement::NotToken),
                    }),
                effect: make(),
            },
        ],
        ..Default::default()
    }
}

/// Diregraf Horde — {4}{B} 3/4 Zombie. ETB make two 2/2 decayed Zombies and
/// exile up to two cards from graveyards.
pub fn diregraf_horde() -> CardDefinition {
    CardDefinition {
        name: "Diregraf Horde",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: decayed_zombie_token() },
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    },
                    Value::Const(2),
                ),
                to: ZoneDest::Exile,
            },
        ]))],
        ..Default::default()
    }
}

/// The Meathook Massacre — {X}{B}{B} Legendary Enchantment. ETB each creature
/// gets -X/-X EOT. Your creature dies → each opponent loses 1; an opponent's
/// creature dies → you gain 1.
pub fn the_meathook_massacre() -> CardDefinition {
    CardDefinition {
        name: "The Meathook Massacre",
        cost: cost(&[crate::mana::x(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                    toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                    duration: Duration::EndOfTurn,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}
