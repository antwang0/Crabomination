use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, SelectionRequirement, Subtypes};
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, r, w};

/// Wrath of God — {2}{W}{W} Sorcery: destroy all creatures
pub fn wrath_of_god() -> CardDefinition {
    CardDefinition {
        name: "Wrath of God",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(SelectionRequirement::Creature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Armageddon — {2}{W}{W} Sorcery: destroy all lands
pub fn armageddon() -> CardDefinition {
    CardDefinition {
        name: "Armageddon",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(SelectionRequirement::Land),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Demonic Tutor — {1}{B} Sorcery: search your library for any card, put it into your hand
pub fn demonic_tutor() -> CardDefinition {
    CardDefinition {
        name: "Demonic Tutor",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Wheel of Fortune — {2}{R} Sorcery: each player draws seven cards
pub fn wheel_of_fortune() -> CardDefinition {
    CardDefinition {
        name: "Wheel of Fortune",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(7),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
