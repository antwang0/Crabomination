//! A ninth staples wave — more Avatar/Lorwyn cards reusing the earthbend /
//! airbend / blight primitives from `recent8`, plus Ally-tribal, prowess, and
//! second-draw payoffs. Tests in `crabomination/src/tests/recent9.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{dies_gain_life, etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};
use crabomination_base::tokens::food_token;

/// Trigger filter: another creature you control of `ty` entering.
fn another_kind_enters(ty: CreatureType) -> EventSpec {
    EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
        Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::HasCreatureType(ty)
                .and(SelectionRequirement::OtherThanSource),
        },
    )
}

/// Trigger: your second draw each turn. `once_per_turn` so a multi-card draw
/// (Divination) — which leaves the running count at 2 for *both* CardDrawn
/// events — still fires the payoff exactly once (CR 603.3d).
fn second_draw() -> EventSpec {
    EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
        .with_filter(Predicate::ValueEquals(
            Value::CardsDrawnThisTurn(PlayerRef::You),
            Value::Const(2),
        ))
        .once_per_turn()
}

// ── Earthbend / Ally ───────────────────────────────────────────────────────

/// Haru, Hidden Talent — {1}{G} 1/1. Whenever another Ally you control enters,
/// earthbend 1.
pub fn haru_hidden_talent() -> CardDefinition {
    CardDefinition {
        name: "Haru, Hidden Talent",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Peasant,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: another_kind_enters(CreatureType::Ally),
            effect: Effect::Earthbend { n: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Avatar Enthusiasts — {2}{W} 2/2. Whenever another Ally you control enters,
/// put a +1/+1 counter on this creature.
pub fn avatar_enthusiasts() -> CardDefinition {
    CardDefinition {
        name: "Avatar Enthusiasts",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: another_kind_enters(CreatureType::Ally),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Invasion Reinforcements — {1}{W} 1/1, Flash. When it enters, create a 1/1
/// white Ally.
pub fn invasion_reinforcements() -> CardDefinition {
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
        name: "Invasion Reinforcements",
        cost: cost(&[generic(1), w()]),
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
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: ally,
        })],
        ..Default::default()
    }
}

// ── Airbend ────────────────────────────────────────────────────────────────

/// Aang, Airbending Master — {4}{W} 4/4. When he enters, airbend another
/// target creature.
pub fn aang_airbending_master() -> CardDefinition {
    CardDefinition {
        name: "Aang, Airbending Master",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Avatar,
                CreatureType::Ally,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Airbend {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
            ),
        })],
        ..Default::default()
    }
}

// ── Blight ─────────────────────────────────────────────────────────────────

/// Sinister Gnarlbark — {2}{B} 0/4. At the beginning of your end step, draw a
/// card and blight 1.
pub fn sinister_gnarlbark() -> CardDefinition {
    CardDefinition {
        name: "Sinister Gnarlbark",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Warlock],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Blight { n: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Dream Seizer — {3}{B} 3/2, Flying. When it enters, you may blight 1; if you
/// do, each opponent discards a card.
pub fn dream_seizer() -> CardDefinition {
    CardDefinition {
        name: "Dream Seizer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Blight 1 so each opponent discards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Blight { n: Value::ONE },
                Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: SelectionRequirement::Any,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Sourbread Auntie — {2}{R}{R} 4/3. When it enters, you may blight 2; if you
/// do, create two 1/1 black-and-red Goblins.
pub fn sourbread_auntie() -> CardDefinition {
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
        name: "Sourbread Auntie",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Blight 2 to create two Goblins?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Blight { n: Value::Const(2) },
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

/// Shadow Urchin — {2}{B/R} 3/4. Whenever it attacks, blight 1.
pub fn shadow_urchin() -> CardDefinition {
    CardDefinition {
        name: "Shadow Urchin",
        cost: cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ouphe],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::Blight { n: Value::ONE })],
        ..Default::default()
    }
}

// ── Card-advantage / counters / prowess ────────────────────────────────────

/// Knowledge Seeker — {1}{U} 2/1, Vigilance. Whenever you draw your second
/// card each turn, put a +1/+1 counter on it. When it dies, create a Clue.
pub fn knowledge_seeker() -> CardDefinition {
    CardDefinition {
        name: "Knowledge Seeker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: second_draw(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            on_dies(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::clue_token(),
            }),
        ],
        ..Default::default()
    }
}

/// Otter-Penguin — {1}{U} 2/1. Whenever you draw your second card each turn, it
/// gets +1/+2 until end of turn.
pub fn otter_penguin() -> CardDefinition {
    CardDefinition {
        name: "Otter-Penguin",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: second_draw(),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Master Pakku — {1}{U} 1/3, Prowess.
pub fn master_pakku() -> CardDefinition {
    CardDefinition {
        name: "Master Pakku",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
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
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

// ── ETB value / utility ────────────────────────────────────────────────────

/// Unlucky Cabbage Merchant — {1}{G} 2/2. When it enters, create a Food.
pub fn unlucky_cabbage_merchant() -> CardDefinition {
    CardDefinition {
        name: "Unlucky Cabbage Merchant",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: food_token(),
        })],
        ..Default::default()
    }
}

/// Curious Farm Animals — {W} 1/1. When it dies, you gain 3 life. (Sacrifice
/// ability omitted.)
pub fn curious_farm_animals() -> CardDefinition {
    CardDefinition {
        name: "Curious Farm Animals",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar, CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![dies_gain_life(3)],
        ..Default::default()
    }
}

/// Deserter's Disciple — {1}{R} 2/2. {T}: another target creature you control
/// with power 2 or less can't be blocked this turn.
pub fn deserters_disciple() -> CardDefinition {
    CardDefinition {
        name: "Deserter's Disciple",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::PowerAtMost(2)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Turtle-Duck — {G} 0/4. {3}: until end of turn, it gets +4/+0 and gains
/// trample. (Modeled as a pump rather than a base-power set; same result.)
pub fn turtle_duck() -> CardDefinition {
    CardDefinition {
        name: "Turtle-Duck",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle, CreatureType::Bird],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
