//! "As this becomes attached to a creature, choose a color" (Sanctuary
//! Blade). Every attach path reports `AttachmentMoved`, and the dispatch
//! funnel makes the choice before any trigger from the batch goes on the
//! stack — so no player has priority between the attach and the choice, as
//! a replacement-style "as" requires, where the old trigger left a window.

use super::GameState;
use super::types::GameEvent;
use crate::effect::{Effect, StaticEffect};
use crate::game::effects::EffectContext;

impl GameState {
    pub(crate) fn apply_as_attached_choices(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::AttachmentMoved { attachment, attached_to: Some(host) } = *ev else { continue };
            let Some(c) = self.battlefield_find(attachment) else { continue };
            if c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::AttachedTakesChosenNameAndType)) {
                self.rename_attached_host(attachment, host);
                continue;
            }
            let chooses = c
                .definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::ChooseColorAsAttached));
            if !chooses {
                continue;
            }
            let ctx = EffectContext::for_ability(attachment, c.controller, None);
            let _ = self.resolve_as_enters_driven(&Effect::ChooseColorForSelf, &ctx);
        }
    }
}
