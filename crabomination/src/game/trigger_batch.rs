//! CR 603.2c / 608.2h — `Effect::WithTriggerBatch`: a once-per-batch trigger
//! that also picks its own targets reads "them" (the batch's subjects) as
//! `Selector::BoundTriggerBatch`. Members are fixed when it fires and read as
//! the trigger resolves; one that left the battlefield meanwhile reads its
//! last-known information (Vulpine Harvester's 2023-04-14 ruling), kept from
//! the dispatcher's departure snapshots before they are cleared.

use super::GameState;
use super::effects::{EffectContext, rewrap_parked};
use super::types::{GameEvent, StackItem};
use crate::card::CardId;
use crate::effect::Effect;

impl GameState {
    /// Run `body` with `ids` bound as the batch, restoring the previous
    /// binding after, and dropping the members' kept LKI once it resolves.
    pub(crate) fn run_with_trigger_batch(
        &mut self,
        ids: &[CardId],
        body: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), crate::game::GameError> {
        let prev = (self.scratch.bound_trigger_batch.as_slice() != ids)
            .then(|| std::mem::replace(&mut self.scratch.bound_trigger_batch, ids.to_vec()));
        let r = self.run_effect(body, ctx, events);
        if let Some(prev) = prev {
            self.scratch.bound_trigger_batch = prev;
        }
        let parked = rewrap_parked(&mut self.suspend_signal, |carried| Effect::WithTriggerBatch {
            ids: ids.to_vec(),
            body: Box::new(carried),
        });
        if !parked {
            for id in ids {
                if self.resolving_lki_source != Some(*id) && self.resolving_lki_subject != Some(*id) {
                    self.leaves_bf_lki.remove(id);
                }
            }
        }
        r
    }

    /// Before the dispatcher clears its departure snapshots: keep those of
    /// members of a batch still waiting on the stack as their LKI.
    pub(crate) fn keep_trigger_batch_lki(&mut self) {
        if self.died_card_snapshots.is_empty() {
            return;
        }
        let members: Vec<CardId> = self
            .stack
            .iter()
            .filter_map(|si| match si {
                StackItem::Trigger { effect, .. } => match effect.as_ref() {
                    Effect::WithTriggerBatch { ids, .. } => Some(ids),
                    _ => None,
                },
                _ => None,
            })
            .flatten()
            .filter(|id| self.died_card_snapshots.contains_key(id) && !self.leaves_bf_lki.contains_key(id))
            .copied()
            .collect();
        for id in members {
            if let Some(snap) = self.died_card_snapshots.get(&id).cloned() {
                self.leaves_bf_lki.insert(id, snap);
            }
        }
    }

    /// Whether `id` is a member of the batch the resolving body bound.
    pub(crate) fn is_bound_batch_member(&self, id: CardId) -> bool {
        self.scratch.bound_trigger_batch.contains(&id)
    }
}
