//! Ravnica (RAV) gap wave 5: a Boros combat-damage trick with a spent-{R}
//! rider and a Replicate land-animation. Tests in `classic_sets/rav`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, LandType, Predicate,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{Color, cost, g, generic, r, u, w};

/// Boros Fury-Shield — {2}{W} Instant. Prevent all combat damage target
/// attacking or blocking creature would deal this turn. If {R} was spent to
/// cast this spell, it deals damage to that creature's controller equal to the
/// creature's power.
pub fn boros_fury_shield() -> CardDefinition {
    CardDefinition {
        name: "Boros Fury-Shield",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast {
                    color: Color::Red,
                    at_least: 1,
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Siege of Towers — {1}{R} Sorcery with Replicate {1}{R}. Target Mountain
/// becomes a 3/1 creature. It's still a land.
pub fn siege_of_towers() -> CardDefinition {
    CardDefinition {
        name: "Siege of Towers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Replicate(cost(&[generic(1), r()]))],
        effect: Effect::BecomeCreature {
            what: target_filtered(R::Land.and(R::HasLandType(LandType::Mountain))),
            power: Value::Const(3),
            toughness: Value::Const(1),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::Permanent,
        },
        ..Default::default()
    }
}

/// Greater Mossdog — {3}{G} 3/3 Plant Dog with dredge 3.
pub fn greater_mossdog() -> CardDefinition {
    CardDefinition {
        name: "Greater Mossdog",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Dredge(3)],
        ..Default::default()
    }
}

/// Flow of Ideas — {5}{U} Sorcery. Draw a card for each Island you control.
pub fn flow_of_ideas() -> CardDefinition {
    CardDefinition {
        name: "Flow of Ideas",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(
                R::HasLandType(LandType::Island).and(R::ControlledByYou),
            )),
        },
        ..Default::default()
    }
}

/// Hour of Reckoning — {4}{W}{W}{W} Sorcery with convoke. Destroy all nontoken
/// creatures.
pub fn hour_of_reckoning() -> CardDefinition {
    CardDefinition {
        name: "Hour of Reckoning",
        cost: cost(&[generic(4), w(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::IsToken)))),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        ..Default::default()
    }
}

/// Guardian of Vitu-Ghazi — {6}{G}{W} 4/7 Elemental with convoke and vigilance.
pub fn guardian_of_vitu_ghazi() -> CardDefinition {
    CardDefinition {
        name: "Guardian of Vitu-Ghazi",
        cost: cost(&[generic(6), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 7,
        keywords: vec![Keyword::Convoke, Keyword::Vigilance],
        ..Default::default()
    }
}
