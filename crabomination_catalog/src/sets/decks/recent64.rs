//! Blue tempo/value batch: flyers, loot engines, prowess, and card draw.
//! Tests in `tests/recent64.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::PlayerRef;
use crate::effect::shortcut::{etb, prowess};
use crate::mana::{cost, generic, u, w};

/// Peregrine Drake — {4}{U} 2/3 Drake with flying. ETB: untap up to five lands.
pub fn peregrine_drake() -> CardDefinition {
    CardDefinition {
        name: "Peregrine Drake",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Untap {
            what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            up_to: Some(Value::Const(5)),
        })],
        ..Default::default()
    }
}

/// Cloud Elemental — {2}{U} 2/3 Elemental with flying; can block only creatures
/// with flying.
pub fn cloud_elemental() -> CardDefinition {
    CardDefinition {
        name: "Cloud Elemental",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..Default::default()
    }
}

/// Thought Courier — {1}{U} 1/1 Human Wizard. {T}: Draw a card, then discard a
/// card.
pub fn thought_courier() -> CardDefinition {
    CardDefinition {
        name: "Thought Courier",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jhessian Thief — {2}{U} 1/3 Human Rogue with prowess. Whenever it deals
/// combat damage to a player, draw a card.
pub fn jhessian_thief() -> CardDefinition {
    CardDefinition {
        name: "Jhessian Thief",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Sky Spirit — {1}{W}{U} 2/2 Spirit with flying and first strike.
pub fn sky_spirit() -> CardDefinition {
    CardDefinition {
        name: "Sky Spirit",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Cephalid Broker — {3}{U} 2/2 Octopus. {T}: Target player draws two cards,
/// then discards two cards.
pub fn cephalid_broker() -> CardDefinition {
    CardDefinition {
        name: "Cephalid Broker",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Riverwise Augur — {3}{U} 2/2 Merfolk Wizard. ETB: draw three cards, then put
/// two cards from your hand on top of your library in any order.
pub fn riverwise_augur() -> CardDefinition {
    CardDefinition {
        name: "Riverwise Augur",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
                count: Value::Const(2),
            },
        ]))],
        ..Default::default()
    }
}
