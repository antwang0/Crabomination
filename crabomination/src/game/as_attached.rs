//! "As this becomes attached to a creature, choose a color" (Sanctuary
//! Blade). Every attach path reports `AttachmentMoved` (a cast Aura's own
//! attach reports `AuraAttached`), and the dispatch
//! funnel makes the choice before any trigger from the batch goes on the
//! stack — so no player has priority between the attach and the choice, as
//! a replacement-style "as" requires, where the old trigger left a window.

use super::{GameState, TempCopy};
use crate::card::CardId;
use super::types::GameEvent;
use crate::effect::{Effect, StaticEffect};
use crate::game::effects::EffectContext;

impl GameState {
    pub(crate) fn apply_as_attached_choices(&mut self, events: &[GameEvent]) {
        for ev in events {
            // A cast Aura reports `AuraAttached` instead (the resolution's own
            // attach, `stack.rs`); only the copy reads it.
            if let GameEvent::AuraAttached { aura, attached_to } = *ev {
                if self.battlefield_find(aura).is_some_and(|c| {
                    c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::AttachedIsCopyOfChosen))
                }) {
                    self.copy_chosen_onto_host(aura, attached_to);
                }
                continue;
            }
            let GameEvent::AttachmentMoved { attachment, attached_to: Some(host) } = *ev else { continue };
            let Some(c) = self.battlefield_find(attachment) else { continue };
            if c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::AttachedTakesChosenNameAndType)) {
                self.rename_attached_host(attachment, host);
                continue;
            }
            if c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::AttachedIsCopyOfChosen)) {
                self.copy_chosen_onto_host(attachment, host);
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

    /// `StaticEffect::AttachedIsCopyOfChosen` (Metamorphic Alteration): `host`
    /// becomes a copy of the model the Aura locked when its copy first applied
    /// (CR 707.2; its rulings), for as long as the Aura stays attached to it.
    fn copy_chosen_onto_host(&mut self, aura: CardId, host: CardId) {
        let model = match self.attached_copy_models.iter().find(|(a, _)| *a == aura) {
            Some((_, d)) => d.clone(),
            None => {
                let Some(chosen) = self.battlefield_find(aura).and_then(|c| c.chosen_permanent) else { return };
                let Some(d) = self.battlefield_find(chosen).map(|c| c.copiable_definition()) else { return };
                self.attached_copy_models.push((aura, d.clone()));
                d
            }
        };
        // A re-attach to the same host replaces the earlier copy.
        if let Some(pos) = self.temporary_copies.iter().position(|tc| tc.card == host && tc.source == Some(aura))
            && let Some(def) = self.temporary_copies[pos].original_def()
        {
            self.temporary_copies.remove(pos);
            if let Some(h) = self.battlefield.find_by_id_mut(host) {
                h.set_copiable_definition(def);
            }
        }
        let Some(h) = self.battlefield.find_by_id_mut(host) else { return };
        let original = h.copiable_definition();
        h.set_copiable_definition(model);
        self.temporary_copies.push(TempCopy {
            card: host,
            original_name: original.name.to_string(),
            original: Some(original),
            duration: crate::effect::Duration::WhileSourceAttached,
            source: Some(aura),
            shapeshifter: false,
            until_turn_of: None,
        });
    }
}
