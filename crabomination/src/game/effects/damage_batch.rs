//! CR 603.2c / 120.3 — the noncombat damage batch. One damage-dealing
//! sentence ("deals 1 damage to each creature and each player") is one damage
//! event per source, however many recipients it hits, so a recipient-agnostic
//! "whenever ~ deals damage" (Spirit Link) fires once with the total. The
//! batch is opened around the outermost such effect; its per-dealer keys ride
//! `combat_trigger_fired_this_step` (no resolution runs inside a combat damage
//! sub-step) so `GameState` doesn't grow — see `BatchSubject::NoncombatDealer`.

use crate::card::CardId;
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::combat::BatchSubject;

/// An effect that does nothing but deal damage: the damage arms themselves,
/// and a `ForEach` / `Seq` built only of them (Pestilence's "each creature and
/// each player" is a `Seq` of two `ForEach` loops).
pub(super) fn is_damage_sentence(effect: &Effect) -> bool {
    match effect {
        Effect::DealDamage { .. }
        | Effect::DealDamageExcessTo { .. }
        | Effect::DealDamageExcessToController { .. }
        | Effect::RadianceDamage { .. }
        | Effect::SameNameDamage { .. }
        | Effect::EachControlledCreatureDealsDamage { .. }
        | Effect::DealDamageDivided { .. }
        | Effect::DealDamageDividedEvenly { .. } => true,
        Effect::ForEach { body, .. } => is_damage_sentence(body),
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_damage_sentence),
        _ => false,
    }
}

impl GameState {
    /// Whether a damage effect is resolving inside an open batch.
    pub(crate) fn noncombat_damage_batch_open(&self) -> bool {
        self.combat_trigger_fired_this_step.iter().any(|k| k.2 == BatchSubject::NoncombatBatch)
    }

    /// Opens the batch; `None` when one is already open (the inner damage
    /// effects of a batched `Seq` share the outer batch).
    pub(super) fn open_noncombat_damage_batch(&mut self) -> Option<usize> {
        if self.noncombat_damage_batch_open() {
            return None;
        }
        let mark = self.combat_trigger_fired_this_step.len();
        self.combat_trigger_fired_this_step.push((CardId(u32::MAX), usize::MAX, BatchSubject::NoncombatBatch));
        Some(mark)
    }

    pub(super) fn close_noncombat_damage_batch(&mut self, mark: usize) {
        self.combat_trigger_fired_this_step.truncate(mark);
    }
}
