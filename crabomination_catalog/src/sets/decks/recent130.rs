//! A third WOE wave: Adventure, modal charms, and Food/Prowess payoffs. Reuses
//! existing primitives. Tests in `crabomination/src/tests/recent130.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, TriggeredAbility, ZoneDest,
};
use crate::game::effects::food_token;
use crate::mana::{Color, b, cost, g, generic, u};

fn white_human_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Scream Puff — {4}{B} 4/5 Horror with deathtouch. Whenever it deals combat
/// damage to a player, create a Food token.
pub fn scream_puff() -> CardDefinition {
    CardDefinition {
        name: "Scream Puff",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            },
        }],
        ..Default::default()
    }
}

/// Beanstalk Wurm // Plant Beans — {4}{G} 5/4 Plant Wurm with reach; Adventure
/// {1}{G} Sorcery lets you play an additional land this turn.
pub fn beanstalk_wurm() -> CardDefinition {
    CardDefinition {
        name: "Beanstalk Wurm",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        adventure: Some(Box::new(Adventure {
            name: "Plant Beans",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::GrantExtraLandPlay {
                who: PlayerRef::You,
                count: Value::ONE,
            },
        })),
        ..Default::default()
    }
}

/// Return from the Wilds — {2}{G} Sorcery. Choose two — search a basic land onto
/// the battlefield tapped; create a 1/1 white Human; create a Food.
pub fn return_from_the_wilds() -> CardDefinition {
    CardDefinition {
        name: "Return from the Wilds",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![2],
            modes: vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: white_human_token(),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: food_token(),
                },
            ],
        },
        ..Default::default()
    }
}

/// Stockpiling Celebrant — {2}{W} 3/2 Dwarf Knight. ETB: you may return another
/// target nonland permanent you control to hand; if you do, scry 2.
pub fn stockpiling_celebrant() -> CardDefinition {
    use crate::mana::w;
    CardDefinition {
        name: "Stockpiling Celebrant",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "return a nonland permanent you control to hand, then scry 2".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Nonland.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Elusive Otter // Grove's Bounty — {U} 1/1 Otter with prowess that can't be
/// blocked by lower-power creatures; Adventure {X}{G} distributes X +1/+1
/// counters among creatures you control.
pub fn elusive_otter() -> CardDefinition {
    CardDefinition {
        name: "Elusive Otter",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Prowess, Keyword::CantBeBlockedByPowerLess],
        adventure: Some(Box::new(Adventure {
            name: "Grove's Bounty",
            cost: cost(&[crate::mana::x(), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DistributeCounters {
                total: Value::XFromCost,
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature.and(R::ControlledByYou),
                max_targets: 8,
            },
        })),
        ..Default::default()
    }
}
