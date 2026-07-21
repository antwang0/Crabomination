//! Ravnica (RAV) gap wave 5: a Boros combat-damage trick with a spent-{R}
//! rider and a Replicate land-animation. Tests in `classic_sets/rav`.

use crate::card::{CardDefinition, CardType, Keyword, LandType, Predicate, SelectionRequirement as R};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, generic, r, w, Color};

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
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Red, at_least: 1 },
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
