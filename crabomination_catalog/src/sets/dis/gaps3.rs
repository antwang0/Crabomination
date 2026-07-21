//! Dissension (DIS) third gap wave. Exercises the new
//! `Effect::PreventCombatDamageByTargetThisTurn` (deal-side combat-damage
//! prevention).

use crate::card::{CardDefinition, CardType, SelectionRequirement as R, Selector};
use crate::effect::Effect;
use crate::mana::{cost, generic, u, w};

/// Azorius Ploy — {1}{W}{W}{U} Instant. Prevent all combat damage target
/// creature would deal this turn; prevent all combat damage that would be dealt
/// to another target creature this turn.
pub fn azorius_ploy() -> CardDefinition {
    CardDefinition {
        name: "Azorius Ploy",
        cost: cost(&[generic(1), w(), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventCombatDamageByTargetThisTurn {
                target: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            },
            Effect::PreventCombatDamageToTargetThisTurn {
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature },
            },
        ]),
        ..Default::default()
    }
}
