//! Lifegain-matters and Angel tribal: gain-life triggers, life-threshold
//! statics, Angel/Horse payoffs, and aristocrat drains. Tests in
//! `tests/recent56.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_noncreature, etb, target_filtered};
use crate::effect::{Duration, PlayerRef, PlayerStaticTarget};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, w};

fn plus_one_counter(what: Selector, n: i32) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    }
}

/// A 4/4 white Angel with flying (Speaker of the Heavens, Valkyrie Harbinger).
fn angel_token(vigilance: bool) -> TokenDefinition {
    let mut keywords = vec![Keyword::Flying];
    if vigilance {
        keywords.push(Keyword::Vigilance);
    }
    TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Bishop of Wings — {W}{W} 1/4 Human Cleric. Angel you control enters → gain 4
/// life; Angel you control dies → make a 1/1 white flying Spirit.
pub fn bishop_of_wings() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    let angel_filter = || Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: R::HasCreatureType(CreatureType::Angel),
    };
    CardDefinition {
        name: "Bishop of Wings",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(angel_filter()),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(angel_filter()),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: spirit,
                },
            },
        ],
        ..Default::default()
    }
}

/// Youthful Valkyrie — {1}{W} 1/3 Angel with flying. Another Angel you control
/// enters → +1/+1 counter on this.
pub fn youthful_valkyrie() -> CardDefinition {
    CardDefinition {
        name: "Youthful Valkyrie",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Angel),
                }),
            effect: plus_one_counter(Selector::This, 1),
        }],
        ..Default::default()
    }
}

/// Righteous Valkyrie — {2}{W} 2/4 Angel Cleric with flying. Angel/Cleric you
/// control enters → gain life = its toughness. Life ≥ starting+7 → team +2/+2.
pub fn righteous_valkyrie() -> CardDefinition {
    CardDefinition {
        name: "Righteous Valkyrie",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Angel)
                        .or(R::HasCreatureType(CreatureType::Cleric)),
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "While you have 7+ life above your starting total, creatures you control get +2/+2.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::PlayerLifeAtLeastAboveStarting {
                    who: PlayerRef::You,
                    delta: 7,
                },
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Twinblade Paladin — {3}{W} 3/3 Human Knight. Gain life → +1/+1 counter;
/// 25+ life → double strike.
pub fn twinblade_paladin() -> CardDefinition {
    CardDefinition {
        name: "Twinblade Paladin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: plus_one_counter(Selector::This, 1),
        }],
        static_abilities: vec![StaticAbility {
            description: "While you have 25 or more life, this creature has double strike.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::PlayerLifeAtLeast {
                    who: PlayerRef::You,
                    life: 25,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
            },
        }],
        ..Default::default()
    }
}

/// Trelasarra, Moon Dancer — {G}{W} 2/2 legendary Elf Cleric. Gain life →
/// +1/+1 counter on Trelasarra and scry 1.
pub fn trelasarra_moon_dancer() -> CardDefinition {
    CardDefinition {
        name: "Trelasarra, Moon Dancer",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Seq(vec![
                plus_one_counter(Selector::This, 1),
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Dauntless Bodyguard — {W} 2/1 Human Knight. As it enters, choose another
/// creature you control. Sacrifice this: the chosen creature gains
/// indestructible until end of turn.
pub fn dauntless_bodyguard() -> CardDefinition {
    CardDefinition {
        name: "Dauntless Bodyguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ChoosePermanentForSource {
            filter: R::Creature.and(R::ControlledByYou),
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::ChosenPermanentOfSource,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bloodthirsty Aerialist — {1}{B}{B} 2/3 Vampire Rogue, flying. Gain life →
/// +1/+1 counter on this.
pub fn bloodthirsty_aerialist() -> CardDefinition {
    CardDefinition {
        name: "Bloodthirsty Aerialist",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: plus_one_counter(Selector::This, 1),
        }],
        ..Default::default()
    }
}

/// Vito, Thorn of the Dusk Rose — {2}{B} 1/3 legendary Vampire Cleric. Gain
/// life → target opponent loses that much life. {3}{B}{B}: creatures you
/// control gain lifelink until end of turn.
pub fn vito_thorn_of_the_dusk_rose() -> CardDefinition {
    CardDefinition {
        name: "Vito, Thorn of the Dusk Rose",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), b()]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rhox Faithmender — {3}{W} 1/5 Rhino Monk with lifelink. If you would gain
/// life, you gain twice that much instead.
pub fn rhox_faithmender() -> CardDefinition {
    CardDefinition {
        name: "Rhox Faithmender",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain twice that much instead.",
            effect: StaticEffect::LifeGainMultiplier {
                target: PlayerStaticTarget::Controller,
                factor: 2,
            },
        }],
        ..Default::default()
    }
}

/// Angelic Chorus — {3}{W}{W} Enchantment. Whenever a creature you control
/// enters, you gain life equal to its toughness.
pub fn angelic_chorus() -> CardDefinition {
    CardDefinition {
        name: "Angelic Chorus",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Exquisite Blood — {4}{B} Enchantment. Whenever an opponent loses life, you
/// gain that much life.
pub fn exquisite_blood() -> CardDefinition {
    CardDefinition {
        name: "Exquisite Blood",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Griffin Aerie — {1}{W} Enchantment. At your end step, if you gained 3+ life
/// this turn, make a 2/2 white flying Griffin.
pub fn griffin_aerie() -> CardDefinition {
    let griffin = TokenDefinition {
        name: "Griffin".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Griffin Aerie",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(3),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: griffin,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Epicure of Blood — {4}{B} 4/4 Vampire. Gain life → each opponent loses 1.
pub fn epicure_of_blood() -> CardDefinition {
    CardDefinition {
        name: "Epicure of Blood",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Crested Sunmare — {3}{W}{W} 5/5 Horse. Other Horses you control have
/// indestructible; at each end step, if you gained life this turn, make a 5/5
/// white Horse.
pub fn crested_sunmare() -> CardDefinition {
    let horse = TokenDefinition {
        name: "Horse".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Crested Sunmare",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Other Horses you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Horse)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                keyword: Keyword::Indestructible,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: horse,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Celestial Unicorn — {2}{W} 3/2 Unicorn. Gain life → +1/+1 counter on this.
pub fn celestial_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Celestial Unicorn",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Unicorn],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: plus_one_counter(Selector::This, 1),
        }],
        ..Default::default()
    }
}

/// Linden, the Steadfast Queen — {W}{W}{W} 3/3 legendary Human Noble with
/// vigilance. Whenever a white creature you control attacks, gain 1 life.
pub fn linden_the_steadfast_queen() -> CardDefinition {
    CardDefinition {
        name: "Linden, the Steadfast Queen",
        cost: cost(&[w(), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasColor(Color::White)),
                },
            ),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Soul's Grace — {1}{W} Instant. You gain life equal to target creature's power.
pub fn souls_grace() -> CardDefinition {
    CardDefinition {
        name: "Soul's Grace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature,
            })),
        },
        ..Default::default()
    }
}

/// Sunscorch Regent — {3}{W}{W} 4/3 Dragon with flying. Whenever an opponent
/// casts a spell, put a +1/+1 counter on this and gain 1 life.
pub fn sunscorch_regent() -> CardDefinition {
    CardDefinition {
        name: "Sunscorch Regent",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                plus_one_counter(Selector::This, 1),
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Kambal, Consul of Allocation — {1}{W}{B} 2/3 legendary Human Advisor.
/// Whenever an opponent casts a noncreature spell, that player loses 2 life and
/// you gain 2 life.
pub fn kambal_consul_of_allocation() -> CardDefinition {
    CardDefinition {
        name: "Kambal, Consul of Allocation",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Valkyrie Harbinger — {4}{W}{W} 4/5 Angel Cleric, flying/lifelink. At each end
/// step, if you gained 4+ life this turn, make a 4/4 vigilant flying Angel.
pub fn valkyrie_harbinger() -> CardDefinition {
    CardDefinition {
        name: "Valkyrie Harbinger",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(4),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: angel_token(true),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Regal Bloodlord — {3}{W}{B} 2/4 Vampire Soldier with flying. At each end
/// step, if you gained life this turn, make a 1/1 black flying Bat.
pub fn regal_bloodlord() -> CardDefinition {
    let bat = TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Regal Bloodlord",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: bat,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Gideon's Company — {3}{W} 3/3 Human Soldier. Gain life → two +1/+1 counters
/// on this. (The Gideon-loyalty activated ability is omitted — fringe.)
pub fn gideons_company() -> CardDefinition {
    CardDefinition {
        name: "Gideon's Company",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: plus_one_counter(Selector::This, 2),
        }],
        ..Default::default()
    }
}
