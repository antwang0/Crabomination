//! Extra-turn spells — "take an extra turn after this one" (CR 500.7),
//! wired via `Effect::TakeExtraTurn` and the per-player `extra_turns`
//! bank consumed during turn advance.

use crate::card::{CardDefinition, CardType};
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{cost, generic, u};

/// Take an extra turn after this one.
fn extra_turn_body() -> Effect {
    Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::Const(1) }
}

/// Time Walk — {1}{U} Sorcery. "Take an extra turn after this one."
pub fn time_walk() -> CardDefinition {
    CardDefinition {
        name: "Time Walk",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Time Warp — {3}{U}{U} Sorcery. "Take an extra turn after this one."
pub fn time_warp() -> CardDefinition {
    CardDefinition {
        name: "Time Warp",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Temporal Manipulation — {3}{U}{U} Sorcery. "Take an extra turn after
/// this one."
pub fn temporal_manipulation() -> CardDefinition {
    CardDefinition {
        name: "Temporal Manipulation",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Capture of Jingzhou — {3}{U}{U} Sorcery. "Take an extra turn after
/// this one." (Time Warp reprint.)
pub fn capture_of_jingzhou() -> CardDefinition {
    CardDefinition {
        name: "Capture of Jingzhou",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Nexus of Fate — {5}{U}{U} Instant. "Take an extra turn after this
/// one." (The shuffle-instead-of-graveyard rider is omitted — no
/// leaves-graveyard replacement primitive yet.)
pub fn nexus_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Nexus of Fate",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: extra_turn_body(),
        ..Default::default()
    }
}

/// Part the Waterveil — {4}{U}{U} Sorcery. Take an extra turn after this
/// one. Awaken 6—{6}{U}{U} (CR 702.113): cast for the awaken cost to also
/// put six +1/+1 counters on target land you control; it becomes a 0/0
/// Elemental creature with haste. (The self-exile rider is omitted.)
pub fn part_the_waterveil() -> CardDefinition {
    use crate::card::{
        AlternativeCost, CounterType, CreatureType, Keyword, SelectionRequirement, Selector,
        Value,
    };
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    let own_land = SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou);
    CardDefinition {
        name: "Part the Waterveil",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: extra_turn_body(),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(6), u(), u()]),
            target_filter: Some(own_land.clone()),
            effect_override: Some(Effect::Seq(vec![
                extra_turn_body(),
                Effect::AddCounter {
                    what: target_filtered(own_land),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(6),
                },
                Effect::BecomeCreature {
                    what: Selector::Target(0),
                    power: Value::Const(0),
                    toughness: Value::Const(0),
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::Permanent,
                },
            ])),
            ..Default::default()
        }),
        ..Default::default()
    }
}
