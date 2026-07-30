//! An eighth wave of staples drawn from the newest sets — the Avatar: The
//! Last Airbender bending mechanics (**earthbend**, CR 701.66; **airbend**,
//! CR 701.65) and Lorwyn's **blight** (CR 701.68) — plus a handful of clean
//! commons that ride existing primitives. Each card has a functionality test
//! in `crabomination/src/tests/recent8.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement,
    Selector, SpellSubtype, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};
use crabomination_base::tokens::clue_token;

// ── Earthbend (CR 701.66) ──────────────────────────────────────────────────

/// Badgermole Cub — {1}{G} 2/2. When it enters, earthbend 1.
pub fn badgermole_cub() -> CardDefinition {
    CardDefinition {
        name: "Badgermole Cub",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::Const(1) })],
        ..Default::default()
    }
}

/// Badgermole — {4}{G} 4/4. When it enters, earthbend 2.
pub fn badgermole() -> CardDefinition {
    CardDefinition {
        name: "Badgermole",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::Const(2) })],
        ..Default::default()
    }
}

/// Earthbending Student — {2}{G} 1/3. When it enters, earthbend 2.
pub fn earthbending_student() -> CardDefinition {
    CardDefinition {
        name: "Earthbending Student",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::Const(2) })],
        ..Default::default()
    }
}

/// Earth Village Ruffians — {2}{B/G} 3/1. When it dies, earthbend 2.
pub fn earth_village_ruffians() -> CardDefinition {
    CardDefinition {
        name: "Earth Village Ruffians",
        cost: cost(&[generic(2), hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Rogue,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::Earthbend { n: Value::Const(2) })],
        ..Default::default()
    }
}

/// Earthbender Ascension — {2}{G} Enchantment. ETB earthbend 2, then ramp a
/// basic onto the battlefield tapped. (Landfall quest-counter engine omitted.)
pub fn earthbender_ascension() -> CardDefinition {
    CardDefinition {
        name: "Earthbender Ascension",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Earthbend { n: Value::Const(2) },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        ]))],
        ..Default::default()
    }
}

// ── Blight (CR 701.68) ─────────────────────────────────────────────────────

/// Blighted Blackthorn — {4}{B} 3/7. Whenever it enters or attacks, you may
/// blight 2; if you do, draw a card and lose 1 life.
pub fn blighted_blackthorn() -> CardDefinition {
    let body = Effect::MayDo {
        description: "Blight 2 to draw a card and lose 1 life?".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Blight { n: Value::Const(2) },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        ])),
    };
    CardDefinition {
        name: "Blighted Blackthorn",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 7,
        triggered_abilities: vec![
            etb(body.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: body,
            },
        ],
        ..Default::default()
    }
}

/// Chaos Spewer — {2}{B/R} 5/4. When it enters, you may pay {2}; if you
/// don't, blight 2.
pub fn chaos_spewer() -> CardDefinition {
    CardDefinition {
        name: "Chaos Spewer",
        cost: cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warlock],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::PayManaOrElse {
            mana_cost: cost(&[generic(2)]),
            otherwise: Box::new(Effect::Blight { n: Value::Const(2) }),
        })],
        ..Default::default()
    }
}

/// Boggart Mischief — {2}{B} Enchantment. When it enters, you may blight 1; if
/// you do, create two 1/1 black-and-red Goblin tokens. (Goblin-death payoff
/// omitted.)
pub fn boggart_mischief() -> CardDefinition {
    let goblin = crate::card::TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Boggart Mischief",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Blight 1 to create two Goblins?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Blight { n: Value::Const(1) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: goblin,
                },
            ])),
        })],
        ..Default::default()
    }
}

// ── Airbend (CR 701.65) ────────────────────────────────────────────────────

/// Airbending Lesson — {2}{W} Instant — Lesson. Airbend target nonland
/// permanent, then draw a card.
pub fn airbending_lesson() -> CardDefinition {
    CardDefinition {
        name: "Airbending Lesson",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            Effect::Airbend {
                what: target_filtered(SelectionRequirement::Nonland),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Aang, the Last Airbender — {3}{W} 3/2, Flying. When he enters, airbend up
/// to one other target nonland permanent. (Lesson-cast trigger omitted.)
pub fn aang_the_last_airbender() -> CardDefinition {
    CardDefinition {
        name: "Aang, the Last Airbender",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Avatar,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: SelectionRequirement::Nonland.and(SelectionRequirement::OtherThanSource),
            effect: Box::new(Effect::Airbend {
                what: Selector::Target(0),
            }),
        })],
        ..Default::default()
    }
}

/// Airbender Ascension — {1}{W} Enchantment. ETB airbend up to one target
/// creature; a quest counter whenever a creature you control enters; at your
/// end step, if it has four or more, flicker up to one creature you control.
pub fn airbender_ascension() -> CardDefinition {
    CardDefinition {
        name: "Airbender Ascension",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::Airbend {
                    what: Selector::Target(0),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Quest,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Quest,
                    },
                    Value::Const(4),
                )),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Exile {
                            what: Selector::Target(0),
                        },
                        Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                                tapped: false,
                            },
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Whirlwind Technique — {4}{U}{U} Instant — Lesson. Target player draws two
/// cards, then discards a card; airbend up to two target creatures.
pub fn whirlwind_technique() -> CardDefinition {
    CardDefinition {
        name: "Whirlwind Technique",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: false,
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::Airbend {
                    what: Selector::Target(0),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Glider Staff — {2}{W} Equipment. ETB airbend up to one target creature.
/// Equipped creature gets +1/+1 and has flying. Equip {1}.
pub fn glider_staff() -> CardDefinition {
    CardDefinition {
        name: "Glider Staff",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::Airbend {
                what: Selector::Target(0),
            }),
        })],
        ..Default::default()
    }
}

// ── Riders on existing primitives ──────────────────────────────────────────

/// Fire Nation Soldier — {2}{R} 3/2, Haste.
pub fn fire_nation_soldier() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Soldier",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Corrupt Court Official — {1}{B} 1/1. When it enters, target opponent
/// discards a card.
pub fn corrupt_court_official() -> CardDefinition {
    CardDefinition {
        name: "Corrupt Court Official",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        ..Default::default()
    }
}

/// Jeong Jeong's Deserters — {1}{W} 1/2. When it enters, put a +1/+1 counter
/// on target creature.
pub fn jeong_jeongs_deserters() -> CardDefinition {
    CardDefinition {
        name: "Jeong Jeong's Deserters",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Forecasting Fortune Teller — {1}{U} 1/3. When it enters, create a Clue.
pub fn forecasting_fortune_teller() -> CardDefinition {
    CardDefinition {
        name: "Forecasting Fortune Teller",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Advisor,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: clue_token(),
        })],
        ..Default::default()
    }
}

/// Pretending Poxbearers — {1}{W/B} 2/1. When it dies, create a 1/1 white Ally.
pub fn pretending_poxbearers() -> CardDefinition {
    let ally = crate::card::TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Pretending Poxbearers",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: ally,
        })],
        ..Default::default()
    }
}

/// Merchant of Many Hats — {1}{B} 2/2. {2}{B}: Return this card from your
/// graveyard to your hand.
pub fn merchant_of_many_hats() -> CardDefinition {
    CardDefinition {
        name: "Merchant of Many Hats",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Peasant,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Yuyan Archers — {1}{R} 3/1, Reach. When it enters, you may discard a card.
/// If you do, draw a card.
pub fn yuyan_archers() -> CardDefinition {
    CardDefinition {
        name: "Yuyan Archers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Discard a card to draw a card?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Platypus-Bear — {1}{G/U} 2/3, Defender. When it enters, mill two cards.
pub fn platypus_bear() -> CardDefinition {
    CardDefinition {
        name: "Platypus-Bear",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Compassionate Healer — {1}{W} 2/2. Whenever it becomes tapped, you gain 1
/// life and scry 1.
pub fn compassionate_healer() -> CardDefinition {
    CardDefinition {
        name: "Compassionate Healer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Cleric,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}
