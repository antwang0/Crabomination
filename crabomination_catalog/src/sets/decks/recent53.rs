//! Monarch payoffs, artifact/enchantment hate, Boros battalion, lifegain, and
//! white/multicolour staples. Tests in `tests/recent53.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, w, x, Color, SpendRestriction};

/// By Force — {X}{R} Sorcery. Destroy X target artifacts.
pub fn by_force() -> CardDefinition {
    CardDefinition {
        name: "By Force",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DestroyTargets { filter: R::Artifact },
        ..Default::default()
    }
}

/// Palace Jailer — {2}{W}{W} 2/2 Human Soldier. ETB: become the monarch, then
/// exile target creature an opponent controls until an opponent becomes the
/// monarch.
pub fn palace_jailer() -> CardDefinition {
    CardDefinition {
        name: "Palace Jailer",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            etb(Effect::ExileUntilOpponentMonarch {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
        ],
        ..Default::default()
    }
}

/// Loxodon Smiter — {1}{G}{W} 4/4 Elephant Soldier. Can't be countered. (The
/// discard→battlefield replacement is approximated to the uncounterable body.)
pub fn loxodon_smiter() -> CardDefinition {
    CardDefinition {
        name: "Loxodon Smiter",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::CantBeCountered],
        ..Default::default()
    }
}

/// Leonin Vanguard — {W} 1/1 Cat Soldier. At the beginning of combat on your
/// turn, if you control three or more creatures, it gets +1/+1 until end of
/// turn and you gain 1 life.
pub fn leonin_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Leonin Vanguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou),
                        )),
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    Value::Const(3),
                ),
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Marchesa's Decree — {3}{B} Enchantment. ETB become the monarch; whenever a
/// creature attacks you or a planeswalker you control, that creature's
/// controller loses 1 life.
pub fn marchesas_decree() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Marchesa's Decree",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Thorn of the Black Rose — {3}{B} 1/3 Human Assassin. Deathtouch; ETB become
/// the monarch.
pub fn thorn_of_the_black_rose() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Thorn of the Black Rose",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::BecomeMonarch { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Throne Warden — {1}{W} 2/2 Human Soldier. At your end step, if you're the
/// monarch, put a +1/+1 counter on it.
pub fn throne_warden() -> CardDefinition {
    CardDefinition {
        name: "Throne Warden",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::IsMonarch { who: PlayerRef::You },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Skyline Despot — {5}{R}{R} 5/5 Dragon. Flying; ETB become the monarch; at
/// your upkeep, if you're the monarch, make a 5/5 red flying Dragon token.
pub fn skyline_despot() -> CardDefinition {
    use crate::card::TokenDefinition;
    let dragon = || TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Skyline Despot",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::IsMonarch { who: PlayerRef::You },
                    then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: dragon() }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Keeper of Keys — {3}{U}{U} 4/4 Human Rogue Mutant. ETB become the monarch;
/// at your upkeep, if you're the monarch, creatures you control can't be
/// blocked this turn.
pub fn keeper_of_keys() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Keeper of Keys",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Mutant],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::IsMonarch { who: PlayerRef::You },
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Judith, the Scourge Diva — {1}{B}{R} 2/2 Human Shaman. Other creatures you
/// control get +1/+0; whenever a nontoken creature you control dies, deal 1
/// damage to any target.
pub fn judith_the_scourge_diva() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Judith, the Scourge Diva",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(R::OtherThanSource),
                }),
            effect: Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Giada, Font of Hope — {1}{W} 2/2 Legendary Angel. Flying, vigilance. Each
/// other Angel you control enters with an additional +1/+1 counter for each
/// Angel you already control. {T}: Add {W}, spend only to cast an Angel spell.
pub fn giada_font_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Giada, Font of Hope",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Each other Angel you control enters with an additional +1/+1 counter on it for each Angel you already control.",
            effect: StaticEffect::TypeEntersWithCountersPerControlled {
                creature_type: CreatureType::Angel,
                kind: CounterType::PlusOnePlusOne,
                per: R::Creature.and(R::HasCreatureType(CreatureType::Angel)).and(R::ControlledByYou),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::White])),
                    SpendRestriction::CreatureOfType(CreatureType::Angel),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hopeful Initiate — {W} 1/2 Human Warlock. Training. {2}{W}, remove two
/// +1/+1 counters from among creatures you control: destroy target artifact or
/// enchantment.
pub fn hopeful_initiate() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::training;
    CardDefinition {
        name: "Hopeful Initiate",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![training()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            remove_counter_among_filter: Some((
                Some(CounterType::PlusOnePlusOne),
                2,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sanctum Prelate — {1}{W}{W} 2/2 Human Cleric. As it enters, choose a number.
/// Noncreature spells with mana value equal to the chosen number can't be cast.
pub fn sanctum_prelate() -> CardDefinition {
    CardDefinition {
        name: "Sanctum Prelate",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseNumberForSource { max: 16 })],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells with mana value equal to the chosen number can't be cast.",
            effect: StaticEffect::NoncreatureSpellsWithChosenManaValueCantBeCast,
        }],
        ..Default::default()
    }
}

/// Old Rutstein — {1}{B}{G} 1/4 Legendary Human Peasant. When it enters and at
/// the beginning of your upkeep, mill a card: land → Treasure, creature → 1/1
/// green Insect, else → Blood.
pub fn old_rutstein() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::b;
    use crabomination_base::tokens::{blood_token, treasure_token};
    let insect = || TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    };
    let mill_branch = move || Effect::MillThenBranchByType {
        land: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure_token() }),
        creature: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: insect() }),
        noncreature: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: blood_token() }),
    };
    CardDefinition {
        name: "Old Rutstein",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            etb(mill_branch()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: mill_branch(),
            },
        ],
        ..Default::default()
    }
}

/// Serra Ascendant — {W} 1/1 Human Monk. Lifelink; while you have 30+ life it
/// gets +5/+5 and has flying.
pub fn serra_ascendant() -> CardDefinition {
    CardDefinition {
        name: "Serra Ascendant",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Gets +5/+5 and has flying as long as you have 30 or more life.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::PlayerLifeAtLeast { who: PlayerRef::You, life: 30 },
                power: 5,
                toughness: 5,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..Default::default()
    }
}

/// Angelic Accord — {3}{W} Enchantment. At each end step, if you gained 4 or
/// more life this turn, make a 4/4 white Angel token with flying.
pub fn angelic_accord() -> CardDefinition {
    use crate::card::TokenDefinition;
    let angel = TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Angelic Accord",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(4) },
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: angel }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Warleader's Helix — {2}{R}{W} Instant. Deal 4 damage to any target and gain
/// 4 life.
pub fn warleaders_helix() -> CardDefinition {
    CardDefinition {
        name: "Warleader's Helix",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(4) },
            Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Wojek Halberdiers — {R}{W} 3/2 Human Soldier. Battalion — first strike until
/// end of turn.
pub fn wojek_halberdiers() -> CardDefinition {
    use crate::effect::shortcut::battalion;
    CardDefinition {
        name: "Wojek Halberdiers",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![battalion(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Firemane Avenger — {2}{R}{W} 3/3 Angel. Flying; Battalion — deal 3 damage to
/// any target and gain 3 life.
pub fn firemane_avenger() -> CardDefinition {
    use crate::effect::shortcut::battalion;
    CardDefinition {
        name: "Firemane Avenger",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![battalion(Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(3) },
            Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(3) },
        ]))],
        ..Default::default()
    }
}

/// Assemble the Legion — {3}{R}{W} Enchantment. At your upkeep, put a muster
/// counter on it, then make a 1/1 red-and-white Soldier with haste for each
/// muster counter on it.
pub fn assemble_the_legion() -> CardDefinition {
    use crate::card::{CounterType, TokenDefinition};
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Assemble the Legion",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Muster, amount: Value::ONE },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TotalCountersOn { what: Box::new(Selector::This) },
                    definition: soldier,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// War Priest of Thune — {1}{W} 2/2 Human Cleric. ETB you may destroy target
/// enchantment.
pub fn war_priest_of_thune() -> CardDefinition {
    CardDefinition {
        name: "War Priest of Thune",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "destroy target enchantment".into(),
            body: Box::new(Effect::Destroy { what: target_filtered(R::Enchantment) }),
        })],
        ..Default::default()
    }
}

/// Goldnight Redeemer — {4}{W}{W} 4/4 Angel. Flying; ETB gain 2 life for each
/// other creature you control.
pub fn goldnight_redeemer() -> CardDefinition {
    CardDefinition {
        name: "Goldnight Redeemer",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::Player(PlayerRef::You),
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    )),
                    filter: R::Creature,
                }),
            ),
        })],
        ..Default::default()
    }
}

/// Kinsbaile Borderguard — {1}{W}{W} 1/1 Kithkin Soldier. Enters with a +1/+1
/// counter for each other Kithkin you control; when it dies, make a 1/1 white
/// Kithkin Soldier token for each counter on it.
pub fn kinsbaile_borderguard() -> CardDefinition {
    use crate::card::{CounterType, TokenDefinition};
    use crate::effect::shortcut::on_dies;
    let kithkin = || TokenDefinition {
        name: "Kithkin Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Kinsbaile Borderguard",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Kithkin))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                )),
                filter: R::Creature,
            },
        )),
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::TotalCountersOn { what: Box::new(Selector::This) },
            definition: kithkin(),
        })],
        ..Default::default()
    }
}

/// Warstorm Surge — {5}{R} Enchantment. Whenever a creature you control enters,
/// it deals damage equal to its power to any target.
pub fn warstorm_surge() -> CardDefinition {
    CardDefinition {
        name: "Warstorm Surge",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Terror of the Peaks — {3}{R}{R} 5/4 Dragon. Flying; whenever another creature
/// you control enters, it deals damage equal to that creature's power to any
/// target. (The "opponents' spells targeting this cost 3 life more" rider is
/// dropped — there's no life-tax on targeting yet.)
pub fn terror_of_the_peaks() -> CardDefinition {
    CardDefinition {
        name: "Terror of the Peaks",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Tuktuk the Explorer — {2}{R} 1/1 Goblin. Haste; when it dies, create Tuktuk
/// the Returned, a legendary 5/5 colorless Goblin Golem artifact token.
pub fn tuktuk_the_explorer() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::effect::shortcut::on_dies;
    let returned = TokenDefinition {
        name: "Tuktuk the Returned".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        colors: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Golem],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Tuktuk the Explorer",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: returned,
        })],
        ..Default::default()
    }
}

/// Tine Shrike — {3}{W} 2/1 Phyrexian Bird. Flying, infect.
pub fn tine_shrike() -> CardDefinition {
    CardDefinition {
        name: "Tine Shrike",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Infect],
        ..Default::default()
    }
}

/// Balustrade Spy — {3}{B} 2/3 Vampire Rogue. Flying; ETB target player mills
/// until they reveal a land card.
pub fn balustrade_spy() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Balustrade Spy",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MillUntilLands {
            who: Selector::Player(PlayerRef::Target(0)),
            lands: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Ravos, Soultender — {3}{W}{B} 2/2 Human Cleric. Flying; other creatures you
/// control get +1/+1; at your upkeep you may return a creature card from your
/// graveyard to your hand.
pub fn ravos_soultender() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::b;
    CardDefinition {
        name: "Ravos, Soultender",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "return a creature card from your graveyard to your hand".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Adriana, Captain of the Guard — {3}{R}{W} 4/4 Legendary Human Knight. Melee;
/// other creatures you control have melee. (Melee is the engine's flat +1/+1
/// on-attack approximation.)
pub fn adriana_captain_of_the_guard() -> CardDefinition {
    use crate::effect::shortcut::melee;
    CardDefinition {
        name: "Adriana, Captain of the Guard",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![melee()],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have melee.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ability: Box::new(melee()),
            },
        }],
        ..Default::default()
    }
}

/// Regal Behemoth — {4}{G}{G} 5/5 Dinosaur. Trample; ETB become the monarch;
/// while you're the monarch, a land tapped for mana produces one extra mana.
/// (The printed "one mana of any color" is approximated to the produced type.)
pub fn regal_behemoth() -> CardDefinition {
    use crate::card::StaticAbility as SA;
    use crate::effect::ExtraManaKind;
    use crate::mana::g;
    CardDefinition {
        name: "Regal Behemoth",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::BecomeMonarch { who: PlayerRef::You })],
        static_abilities: vec![SA {
            description: "Whenever you tap a land for mana while you're the monarch, add an additional one mana.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::Any,
                extra: ExtraManaKind::Mirror,
                while_monarch: true,
            },
        }],
        ..Default::default()
    }
}

/// Gallant Cavalry — {3}{W} 2/2 Human Knight. Vigilance; ETB make a 2/2 white
/// Knight token with vigilance.
pub fn gallant_cavalry() -> CardDefinition {
    use crate::card::TokenDefinition;
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Gallant Cavalry",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: knight,
        })],
        ..Default::default()
    }
}

/// Valiant Knight — {3}{W} 3/4 Human Knight. Other Knights you control get
/// +1/+1; {3}{W}{W}: Knights you control gain double strike until end of turn.
pub fn valiant_knight() -> CardDefinition {
    let knights = || Selector::EachPermanent(
        R::Creature.and(R::HasCreatureType(CreatureType::Knight)).and(R::ControlledByYou),
    );
    CardDefinition {
        name: "Valiant Knight",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other Knights you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Knight))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            effect: Effect::GrantKeyword {
                what: knights(),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Custodi Lich — {3}{B}{B} 4/2 Zombie Cleric. ETB become the monarch; each
/// opponent sacrifices a creature. (The printed "whenever you become the
/// monarch, target player sacrifices" is approximated to the ETB edict.)
pub fn custodi_lich() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Custodi Lich",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            etb(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature,
            }),
        ],
        ..Default::default()
    }
}
