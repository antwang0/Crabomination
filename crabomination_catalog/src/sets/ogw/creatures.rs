use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::effect::shortcut::{ingest, prowess_trigger};
use crate::mana::{cost, r, u};

/// Stormchaser Mage — {1}{U}{R} 1/3 Flying Haste Prowess
pub fn stormchaser_mage() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Mage",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Prowess],
        effect: Effect::Noop,
        triggered_abilities: vec![prowess_trigger()],
        ..Default::default()
    }
}

/// Mist Intruder — {1}{U} 1/2 Eldrazi Drone. Devoid, Flying, Ingest.
pub fn mist_intruder() -> CardDefinition {
    CardDefinition {
        name: "Mist Intruder",
        cost: cost(&[crate::mana::generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![ingest()],
        ..Default::default()
    }
}

/// Sludge Crawler — {B} 1/1 Eldrazi Drone. Devoid, Ingest, {2}: +1/+1 EOT.
pub fn sludge_crawler() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Sludge Crawler",
        cost: cost(&[crate::mana::b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![ingest()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
