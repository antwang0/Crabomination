//! Aboleth Spawn's Probing Telepathy — "Whenever a creature entering under an
//! opponent's control causes a triggered ability of that creature to trigger,
//! you may copy that ability." The copier's own trigger goes on the stack on
//! top of the ability that caused it (2022-06-10 ruling) and, resolving,
//! copies *that* ability by its stack id (CR 115.1, `game/stack_ability.rs`) —
//! so the copy lands on top of the original and resolves first, and nothing is
//! copied if the original has already left the stack.

use super::GameState;
use super::types::{StackItem, Target};
use crate::card::CardId;
use crate::effect::{Effect, Selector, Value};

/// The copier's trigger body: "you may copy that ability" — the ability is
/// bound into slot 0 as the trigger goes on the stack.
pub(crate) fn probing_telepathy_trigger() -> Effect {
    Effect::MayDo {
        description: "Probing Telepathy — copy that triggered ability?".into(),
        body: Box::new(Effect::CopyAbility { what: Selector::Target(0), times: Value::ONE }),
    }
}

impl GameState {
    /// The ability a Probing Telepathy trigger copies: the topmost ability
    /// of `entering` on the stack that no other such trigger has claimed
    /// (one per fire when a doubler fires it twice).
    pub(crate) fn probing_telepathy_target(&self, entering: CardId) -> Option<Target> {
        let marker = probing_telepathy_trigger();
        let claimed: Vec<u32> = self
            .stack
            .iter()
            .filter_map(|si| match si {
                StackItem::Trigger { effect, target: Some(Target::Permanent(id)), .. } if **effect == marker => {
                    Some(id.0)
                }
                _ => None,
            })
            .collect();
        self.stack.iter().rev().find_map(|si| match si {
            StackItem::Trigger { source, ability_id, effect, .. }
                if *source == entering && *ability_id != 0 && !claimed.contains(ability_id) && **effect != marker =>
            {
                Some(Target::Permanent(CardId(*ability_id)))
            }
            _ => None,
        })
    }

    /// Is `effect` a Probing Telepathy trigger body (bound at push time)?
    pub(crate) fn is_probing_telepathy(effect: &Effect) -> bool {
        *effect == probing_telepathy_trigger()
    }
}
