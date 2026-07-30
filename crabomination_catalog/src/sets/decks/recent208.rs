//! Foundations (FDN) gap batch 7 — more commons/uncommons on existing
//! primitives: french-vanilla bodies, a Defender death-Treasure wall, a Raid
//! ETB, a begin-combat pump, an activated overrun, and a color-exile + scry.
//! Tests in `tests/recent208.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, EventKind, EventScope, EventSpec, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Highborn Vampire — {3}{B} 4/3 vanilla Vampire Warrior.
pub fn highborn_vampire() -> CardDefinition {
    CardDefinition {
        name: "Highborn Vampire",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        ..Default::default()
    }
}

/// Swab Goblin — {1}{R} 2/2 vanilla Goblin Pirate.
pub fn swab_goblin() -> CardDefinition {
    CardDefinition {
        name: "Swab Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Gleaming Barrier — {2} 0/4 Artifact Creature — Wall. Defender; when it dies,
/// create a Treasure token.
pub fn gleaming_barrier() -> CardDefinition {
    CardDefinition {
        name: "Gleaming Barrier",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Storm Fleet Spy — {2}{U} 2/2. Raid — when it enters, if you attacked this
/// turn, draw a card.
pub fn storm_fleet_spy() -> CardDefinition {
    CardDefinition {
        name: "Storm Fleet Spy",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Battle-Rattle Shaman — {3}{R} 2/2. At the beginning of combat on your turn,
/// you may have target creature get +2/+0 until end of turn.
pub fn battle_rattle_shaman() -> CardDefinition {
    CardDefinition {
        name: "Battle-Rattle Shaman",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayDo {
                description: "Target creature gets +2/+0 until end of turn.".into(),
                body: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Wildheart Invoker — {2}{G}{G} 4/3. {8}: Target creature gets +5/+5 and gains
/// trample until end of turn.
pub fn wildheart_invoker() -> CardDefinition {
    CardDefinition {
        name: "Wildheart Invoker",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Devout Decree — {1}{W} Sorcery. Exile target black or red creature or
/// planeswalker. Scry 1.
pub fn devout_decree() -> CardDefinition {
    CardDefinition {
        name: "Devout Decree",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    R::Creature
                        .or(R::Planeswalker)
                        .and(R::HasColor(Color::Black).or(R::HasColor(Color::Red))),
                ),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
