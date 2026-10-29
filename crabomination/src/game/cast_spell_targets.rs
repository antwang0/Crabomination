//! "Whenever you cast a spell that targets [filter]": the just-cast spell's
//! targets — slot 0 and every additional one — read off the stack through the
//! trigger source (`fire_spell_cast_triggers` stamps the cast card there).
//! `Predicate::CastSpellTargetsMatch` asks for one; `Value::
//! CastSpellTargetsMatching` counts them, each distinct target once ("put
//! that many +1/+1 counters" — Arcee, Acrobatic Coupe).

use super::GameState;
use super::effects::{EffectContext, EntityRef};
use super::types::{StackItem, Target};
use crate::card::SelectionRequirement;

impl GameState {
    pub(crate) fn cast_spell_targets_matching(&self, filter: &SelectionRequirement, ctx: &EffectContext) -> u32 {
        let Some(EntityRef::Card(cid)) = ctx.trigger_source else { return 0 };
        let Some((target, extra)) = self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, target, additional_targets, .. } if card.id == cid => {
                Some((target.as_ref(), additional_targets))
            }
            _ => None,
        }) else {
            return 0;
        };
        let mut seen: Vec<&Target> = Vec::new();
        for t in target.into_iter().chain(extra.iter()) {
            // CR 109.2: "a spell that targets a creature" means a creature on
            // the battlefield — a creature card in a graveyard is a card.
            // Glorious Decay exiling one fed Repartee (SOS spot check,
            // 2026-10-29). A filter that names another zone keeps it.
            if let Target::Permanent(id) = t
                && self.battlefield_find(*id).is_none()
                && !filter.mentions_offboard_zone()
            {
                continue;
            }
            if !seen.contains(&t) && self.evaluate_requirement_static(filter, t, ctx.controller, ctx.source) {
                seen.push(t);
            }
        }
        seen.len() as u32
    }
}
