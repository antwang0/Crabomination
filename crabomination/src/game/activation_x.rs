//! CR 602.1a / 107.3 — "that ability's activation cost contains {X}"
//! (Unbound Flourishing). The stack item carries the ability's effect but not
//! its index, so the activation is matched back to its printed ability by
//! effect: the source's activated abilities whose effect is the stack item's
//! (or nested inside it, under the activation path's cost-capture wrappers).

use crate::game::GameState;
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::types::StackItem;

impl GameState {
    /// Does the newest activated ability on the stack from the trigger's
    /// source have `{X}` in its mana cost?
    pub(crate) fn activation_cost_has_x(&self, ctx: &EffectContext) -> bool {
        let owner = match ctx.trigger_source {
            Some(EntityRef::Permanent(id)) | Some(EntityRef::Card(id)) => id,
            _ => return false,
        };
        let Some(effect) = self.stack.iter().rev().find_map(|si| match si {
            StackItem::Trigger { source, activated: true, effect, .. } if *source == owner => Some(effect),
            _ => None,
        }) else {
            return false;
        };
        let Some(card) = self.battlefield_find(owner) else { return false };
        card.definition
            .activated_abilities
            .iter()
            .filter(|a| a.mana_cost.has_x())
            .any(|a| **effect == a.effect || effect.any_nested(&|e| *e == a.effect))
    }
}
