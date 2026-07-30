//! Engine-primitive-unblocked batch: distinct-mana-values-in-graveyard
//! (Aven Heartstabber), greatest-power-including-graveyard (Ambitious
//! Dragonborn), the `GiftGiven` event (Jolly Gerbils), and an Enlist Orc
//! (Argivian Cavalier). Tests in `tests/recent_b/recent283.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{enlist, etb, on_dies};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility, Value,
};
use crate::mana::{Color, b, cost, generic, u, w};

/// Aven Heartstabber — {U}{B} 1/1 Bird Assassin. Flying. Gets +2/+2 and has
/// deathtouch while there are 5+ mana values among cards in your graveyard.
/// When it dies, mill two, then draw a card.
pub fn aven_heartstabber() -> CardDefinition {
    CardDefinition {
        name: "Aven Heartstabber",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Assassin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "+2/+2 and deathtouch while 5+ mana values in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::DistinctManaValuesInGraveyard(PlayerRef::You),
                    Value::Const(5),
                ),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Deathtouch],
            },
        }],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Ambitious Dragonborn — {3}{G} Dragon Barbarian. Enters with X +1/+1
/// counters, X = greatest power among creatures you control and creature cards
/// in your graveyard.
pub fn ambitious_dragonborn() -> CardDefinition {
    CardDefinition {
        name: "Ambitious Dragonborn",
        cost: cost(&[generic(3), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Barbarian],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::GreatestPowerControlledAndGraveyard,
        )),
        ..Default::default()
    }
}

/// Jolly Gerbils — {1}{W} 2/3 Hamster Citizen. Whenever you give a gift, draw
/// a card.
pub fn jolly_gerbils() -> CardDefinition {
    CardDefinition {
        name: "Jolly Gerbils",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hamster, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::GiftGiven, EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Argivian Cavalier — {2}{W} 2/2 Orc Knight. Enlist. When it enters, create a
/// 1/1 white Soldier creature token.
pub fn argivian_cavalier() -> CardDefinition {
    CardDefinition {
        name: "Argivian Cavalier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            enlist(),
            etb(Effect::CreateToken {
                who: PlayerRef::You,
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
            }),
        ],
        ..Default::default()
    }
}
