//! OTJ plot batch exercising the new `EventKind::BecomesPlotted` self-trigger
//! (CR 702.170): Aloe Alchemist (pump on plot) and Longhorn Sharpshooter (burn
//! on plot). Tests in `crabomination/src/tests/recent191.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Duration, Effect, Selector};
use crate::mana::{cost, g, generic, r};

/// Aloe Alchemist — {1}{G} 3/2 Plant Warlock with trample. Plot {1}{G}. When it
/// becomes plotted, target creature gets +3/+2 and gains trample until end of
/// turn.
pub fn aloe_alchemist() -> CardDefinition {
    CardDefinition {
        name: "Aloe Alchemist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        plot_cost: Some(cost(&[generic(1), g()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesPlotted, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Longhorn Sharpshooter — {2}{R} 3/3 Minotaur Rogue with reach. Plot {3}{R}.
/// When it becomes plotted, it deals 2 damage to any target.
pub fn longhorn_sharpshooter() -> CardDefinition {
    CardDefinition {
        name: "Longhorn Sharpshooter",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        plot_cost: Some(cost(&[generic(3), r()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesPlotted, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}
