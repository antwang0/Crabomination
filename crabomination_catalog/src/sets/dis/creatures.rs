use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword, SelectionRequirement,
    Selector, Subtypes, Value,
};
use crate::mana::{Color, cost, g, generic, hybrid, u, w};

/// Azorius First-Wing — {1}{W}{U} 2/2 Bird Soldier Flying
pub fn azorius_first_wing() -> CardDefinition {
    CardDefinition {
        name: "Azorius First-Wing",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Aquastrand Spider — {1}{G/U} 0/0 Spider, Reach, Graft 2.
pub fn aquastrand_spider() -> CardDefinition {
    CardDefinition {
        name: "Aquastrand Spider",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        keywords: vec![Keyword::Reach],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Plaxcaster Frogling — {2}{G/U} 0/0 Frog Beast, Graft 3.
pub fn plaxcaster_frogling() -> CardDefinition {
    CardDefinition {
        name: "Plaxcaster Frogling",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Beast],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Cytoplast Root-Kin — {2}{G}{G} 0/0 Mutant, Graft 4. ETB puts a +1/+1
/// counter on each other creature you control that already has one.
pub fn cytoplast_root_kin() -> CardDefinition {
    CardDefinition {
        name: "Cytoplast Root-Kin",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            }),
            crate::effect::shortcut::graft(),
        ],
        ..Default::default()
    }
}
