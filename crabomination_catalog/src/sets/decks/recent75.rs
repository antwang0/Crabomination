//! Retro batch — enrage, upkeep self-ping, can't-block bruisers, a loot-ETB,
//! firebreathing Drakes, and vanilla bodies. All ride existing primitives.
//! Tests in `tests/recent75.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{enrage, etb, etb_loot};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w};

/// Fungusaur — {3}{G} 2/2 Fungus Dinosaur. Whenever it's dealt damage, put a
/// +1/+1 counter on it.
pub fn fungusaur() -> CardDefinition {
    CardDefinition {
        name: "Fungusaur",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![enrage(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Serpent Warrior — {2}{B} 3/3 Snake Warrior. When it enters, you lose 3 life.
pub fn serpent_warrior() -> CardDefinition {
    CardDefinition {
        name: "Serpent Warrior",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        ..Default::default()
    }
}

/// Ekundu Griffin — {3}{W} 2/2 Griffin. Flying, first strike.
pub fn ekundu_griffin() -> CardDefinition {
    CardDefinition {
        name: "Ekundu Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Nettletooth Djinn — {3}{G} 4/4 Djinn. At the beginning of your upkeep, it
/// deals 1 damage to you.
pub fn nettletooth_djinn() -> CardDefinition {
    CardDefinition {
        name: "Nettletooth Djinn",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::DealDamage {
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Hulking Cyclops — {3}{R}{R} 5/5 Cyclops. Can't block.
pub fn hulking_cyclops() -> CardDefinition {
    CardDefinition {
        name: "Hulking Cyclops",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

/// Pygmy Pyrosaur — {1}{R} 1/1 Lizard. Can't block. {R}: gets +1/+0 until end
/// of turn.
pub fn pygmy_pyrosaur() -> CardDefinition {
    CardDefinition {
        name: "Pygmy Pyrosaur",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Owl Familiar — {1}{U} 1/1 Bird. Flying. When it enters, draw a card, then
/// discard a card.
pub fn owl_familiar() -> CardDefinition {
    CardDefinition {
        name: "Owl Familiar",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb_loot()],
        ..Default::default()
    }
}

/// Fire Drake — {1}{R}{R} 1/2 Drake. Flying. {R}: gets +1/+0 until end of turn.
/// Activate only once each turn.
pub fn fire_drake() -> CardDefinition {
    CardDefinition {
        name: "Fire Drake",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Muck Rats — {B} 1/1 Rat (vanilla).
pub fn muck_rats() -> CardDefinition {
    CardDefinition {
        name: "Muck Rats",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}
