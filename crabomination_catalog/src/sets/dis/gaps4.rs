//! Dissension (DIS) gap wave 4: Carom's damage redirect, riding the new
//! `Effect::RedirectNextDamage`. Tests in `classic_sets/dis`.

use crate::card::{CardDefinition, CardType, SelectionRequirement as R, Selector, Value};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{cost, generic, w};

/// Carom — {1}{W} Instant. The next 1 damage that would be dealt to target
/// creature this turn is dealt to another target creature instead. Draw a card.
pub fn carom() -> CardDefinition {
    CardDefinition {
        name: "Carom",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::RedirectNextDamage {
                target: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature,
                },
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature,
                },
                amount: Value::ONE,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::You),
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
