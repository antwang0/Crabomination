use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement, Selector, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{forecast, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Azorius First-Wing — {1}{W}{U} 2/2 Bird Soldier Flying
pub fn azorius_first_wing() -> CardDefinition {
    CardDefinition {
        name: "Azorius First-Wing",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Aquastrand Spider — {1}{G/U} 0/0 Spider, Reach, Graft 2.
pub fn aquastrand_spider() -> CardDefinition {
    CardDefinition {
        name: "Aquastrand Spider",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        keywords: vec![Keyword::Reach],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Plaxcaster Frogling — {2}{G/U} 0/0 Frog Beast, Graft 3.
pub fn plaxcaster_frogling() -> CardDefinition {
    CardDefinition {
        name: "Plaxcaster Frogling",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Cytoplast Root-Kin — {2}{G}{G} 0/0 Mutant, Graft 4. ETB puts a +1/+1
/// counter on each other creature you control that already has one.
pub fn cytoplast_root_kin() -> CardDefinition {
    CardDefinition {
        name: "Cytoplast Root-Kin",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::WithCounter(
                            CounterType::PlusOnePlusOne,
                        )),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            }),
            crate::effect::shortcut::graft(),
        ],
        ..Default::default()
    }
}

/// Simic Initiate — {G/U} 0/0 Merfolk Wizard, Graft 1.
pub fn simic_initiate() -> CardDefinition {
    CardDefinition {
        name: "Simic Initiate",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Vigean Graftmage — {1}{G/U} 0/0 Vedalken Wizard, Graft 2.
/// "{1}{U}: Untap target creature with a +1/+1 counter on it."
pub fn vigean_graftmage() -> CardDefinition {
    CardDefinition {
        name: "Vigean Graftmage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Untap {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                )),
                up_to: None,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Helium Squirter — {4}{G/U} 0/0 Mutant, Graft 3.
/// "{1}: Target creature with a +1/+1 counter on it gains flying until end
/// of turn."
pub fn helium_squirter() -> CardDefinition {
    CardDefinition {
        name: "Helium Squirter",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                )),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Assault Zeppelid — {2}{G}{U} 3/3 Beast with flying and trample.
pub fn assault_zeppelid() -> CardDefinition {
    CardDefinition {
        name: "Assault Zeppelid",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        ..Default::default()
    }
}

/// Sky Hussar — {3}{W}{U} 4/3 Human Knight with flying. When it enters, untap
/// all creatures you control. Forecast — Tap two untapped white and/or blue
/// creatures you control: Draw a card.
pub fn sky_hussar() -> CardDefinition {
    let wu_creature = SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(
            SelectionRequirement::HasColor(Color::White)
                .or(SelectionRequirement::HasColor(Color::Blue)),
        );
    CardDefinition {
        name: "Sky Hussar",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((wu_creature, 2)),
            ..forecast(
                cost(&[]),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            )
        }],
        ..Default::default()
    }
}

/// Stalking Vengeance — {5}{R}{R} 5/5 Avatar with haste. Whenever another
/// creature you control dies, it deals damage equal to its power to any target.
/// (Modeled as each opponent — faithful in 1v1; the dead creature's power
/// carries via its die snapshot, CR 603.10.)
pub fn stalking_vengeance() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Stalking Vengeance",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::DealDamageEqualToPower {
                source: Selector::TriggerSource,
                target: Selector::Player(PlayerRef::EachOpponent),
            },
        }],
        ..Default::default()
    }
}

/// Kill-Suit Cultist — {R} 1/1 Goblin Berserker. Attacks each combat if able.
/// `{B}, Sacrifice this creature: The next time damage would be dealt to target
/// creature this turn, destroy that creature instead.`
pub fn kill_suit_cultist() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::b;
    CardDefinition {
        name: "Kill-Suit Cultist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::MustAttack],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::ReplaceNextDamageWithDestroy {
                target: target_filtered(SelectionRequirement::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Azorius Herald — {2}{W} 2/1 Spirit. Can't be blocked. When it enters, gain 4
/// life; and sacrifice it unless {U} was spent to cast it.
pub fn azorius_herald() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    CardDefinition {
        name: "Azorius Herald",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::SourceCastWithColorSpent {
                        color: Color::Blue,
                        at_least: 1,
                    },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::SacrificeSource),
                },
            },
        ],
        ..Default::default()
    }
}

/// Flaring Flame-Kin — {2}{R} 2/2 Elemental Warrior. As long as it's enchanted,
/// it gets +2/+2, has trample, and has "{R}: This creature gets +1/+0 until end
/// of turn."
pub fn flaring_flame_kin() -> CardDefinition {
    let enchanted = || Predicate::EntityMatches {
        what: Selector::This,
        filter: SelectionRequirement::IsEnchanted,
    };
    CardDefinition {
        name: "Flaring Flame-Kin",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "As long as this creature is enchanted, it gets +2/+2 and has trample.",
                effect: StaticEffect::PumpTeamIf {
                    condition: enchanted(),
                    applies_to: Selector::This,
                    power: 2,
                    toughness: 2,
                    keywords: vec![Keyword::Trample],
                },
            },
            StaticAbility {
                description: "As long as this creature is enchanted, it has \"{R}: This creature gets +1/+0 until end of turn.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::This,
                    ability: ActivatedAbility {
                        mana_cost: cost(&[r()]),
                        effect: Effect::PumpPT {
                            what: Selector::This,
                            power: Value::Const(1),
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        ..Default::default()
                    },
                    condition: Some(enchanted()),
                },
            },
        ],
        ..Default::default()
    }
}

/// Haazda Shield Mate — {2}{W} 1/1 Human Soldier. At your upkeep, sacrifice it
/// unless you pay {W}{W}. `{W}: The next time a source of your choice would deal
/// damage to you this turn, prevent that damage.`
pub fn haazda_shield_mate() -> CardDefinition {
    CardDefinition {
        name: "Haazda Shield Mate",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayPay {
                description: "Pay {W}{W} or sacrifice Haazda Shield Mate?".into(),
                mana_cost: cost(&[w(), w()]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::SacrificeSource)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: SelectionRequirement::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jagged Poppet — {1}{B}{R} 3/4 Ogre Warrior. Whenever it's dealt damage,
/// discard that many cards. Hellbent — whenever it deals combat damage to a
/// player, if you have no cards in hand, that player discards that many cards.
pub fn jagged_poppet() -> CardDefinition {
    CardDefinition {
        name: "Jagged Poppet",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::Discard {
                    who: Selector::You,
                    amount: Value::TriggerEventAmount,
                    random: false,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::HellbentActive {
                        who: PlayerRef::You,
                    }),
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::DefendingPlayer),
                    amount: Value::TriggerEventAmount,
                    random: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Rakdos Augermage — {B}{B}{R} 3/2 Human Wizard with first strike. `{T}: You
/// discard a card, then target opponent reveals their hand and discards a card
/// of your choice. Activate only as a sorcery.` (The printed "opponent chooses
/// your discard" clause is approximated as your own discard.)
pub fn rakdos_augermage() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Augermage",
        cost: cost(&[b(), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: SelectionRequirement::Any,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drekavac — {1}{B} 3/3 Beast. When it enters, sacrifice it unless you discard
/// a noncreature card.
pub fn drekavac() -> CardDefinition {
    CardDefinition {
        name: "Drekavac",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDiscardMatching {
                description: "Discard a noncreature card to keep Drekavac?".into(),
                count: Value::ONE,
                filter: SelectionRequirement::Creature.negate(),
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::SacrificeSource)),
            },
        }],
        ..Default::default()
    }
}

/// Crypt Champion — {3}{B} 2/2 Zombie with double strike. When it enters, each
/// player puts a creature card with mana value 3 or less from their graveyard
/// onto the battlefield; then sacrifice Crypt Champion unless {R} was spent to
/// cast it.
pub fn crypt_champion() -> CardDefinition {
    CardDefinition {
        name: "Crypt Champion",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::EachPlayerReanimateCreatureMaxMv { max_mv: 3 }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::SourceCastWithColorSpent {
                        color: Color::Red,
                        at_least: 1,
                    },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::SacrificeSource),
                },
            },
        ],
        ..Default::default()
    }
}
