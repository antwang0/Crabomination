//! Dissension (DIS) gap wave 5. Trial // Error rides the new
//! `Selector::CreaturesInCombatWith`. Tests in `classic_sets/dis`.

use crate::card::{CardDefinition, CardType, Effect, SelectionRequirement as R, Selector, SplitCard, SplitHalf};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{b, cost, u, w};

/// Trial // Error — {W}{U} // {U}{B} Instant // Instant. Trial returns all
/// creatures blocking or blocked by target creature to their owners' hands;
/// Error counters target multicolored spell.
pub fn trial_error() -> CardDefinition {
    CardDefinition {
        name: "Trial // Error",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::CreaturesInCombatWith(Box::new(target_filtered(R::Creature))),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[u(), b()]),
                card_types: vec![CardType::Instant],
                effect: Effect::CounterSpell { what: target_filtered(R::Multicolored) },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}
