//! CR 601.2c / 603.3d — the further targets of an "up to N target" triggered
//! ability (Katsumasa's "each of up to three target noncreature artifacts",
//! Depthshaker Titan's "any number of target noncreature artifacts you
//! control") are the controller's to name, slot by slot, not only the first.
//!
//! Distinct per-slot filters past an `OptionalTargets` minimum (Capricious
//! Efreet's "and up to two target nonland permanents you don't control") are
//! asked slot by slot under each slot's own filter.
//!
//! Each ask leads with the engine's own fill (`auto_extra_targets_for`) and
//! stops where that fill does, so a seat answering with the first candidate —
//! the bot policy here, and `AutoDecider` — names exactly the targets it was
//! given before the asks existed.

use super::GameState;
use crate::decision::Decision;
use crate::effect::Effect;
use crate::game::types::{PendingTriggerPush, Target};

/// Is `eff` a same-filter "up to N target" fan-out under its transparent
/// wrappers? Per-opponent / per-player fan-outs keep the engine's fill.
fn same_filter_fan_out(eff: &Effect) -> bool {
    match eff {
        Effect::ApplyToTargets { .. } | Effect::SupportCounters { .. } => true,
        Effect::MayDo { body, .. }
        | Effect::OptionalTargets { body, .. }
        | Effect::TargetsExactlyX { body }
        | Effect::CapTargetsAt { body, .. } => same_filter_fan_out(body),
        Effect::Seq(parts) => {
            let mut targeting = parts.iter().filter(|e| e.requires_target());
            match (targeting.next(), targeting.next()) {
                (Some(only), None) => same_filter_fan_out(only),
                _ => false,
            }
        }
        _ => false,
    }
}

/// Is slot `n` of `eff` an optional slot past an `OptionalTargets` minimum,
/// each slot carrying its own filter?
fn optional_distinct_slot(eff: &Effect, n: usize) -> bool {
    match eff {
        Effect::OptionalTargets { min, .. } => n >= *min as usize,
        _ => false,
    }
}

impl GameState {
    /// The ask for the next target of `pending` after `picked` (slot 0 first),
    /// or `None` when the fill is done: no prompting seat, not a same-filter
    /// fan-out, or as many named as the engine's fill would name.
    pub(crate) fn next_trigger_extra_slot(
        &self,
        pending: &PendingTriggerPush,
        picked: &[Target],
        seat: usize,
    ) -> Option<Decision> {
        if matches!(pending.effect, Effect::ChooseN { .. }) {
            return self.next_modal_slot(pending, picked, seat);
        }
        let first = picked.first()?;
        if !self.seat_prompts(seat) || !matches!(first, Target::Permanent(_)) {
            return None;
        }
        // As `push_pending_trigger` binds it: an event-sized cap is fixed.
        let effect = match &pending.effect {
            Effect::CapTargetsAt { amount: crate::effect::Value::TriggerEventAmount, body } => Effect::CapTargetsAt {
                amount: crate::effect::Value::Const(pending.event_amount as i32),
                body: body.clone(),
            },
            other => other.clone(),
        };
        let same = same_filter_fan_out(&effect);
        if !same && !optional_distinct_slot(&effect, picked.len()) {
            return None;
        }
        let fill = self.auto_extra_targets_for(&effect, pending.source, pending.controller, Some(first.clone()));
        if picked.len() > fill.len() {
            return None;
        }
        let slot = if same { 1 } else { picked.len() };
        let filter = effect.target_filter_for_slot_in_mode(u8::try_from(slot).ok()?, pending.mode)?.clone();
        // A same-filter fill is a pool; a per-slot one is positional.
        let lead = if same {
            fill.iter().find(|t| !picked.contains(t)).cloned()
        } else {
            fill.get(slot - 1).filter(|t| !picked.contains(t)).cloned()
        };
        let mut legal: Vec<Target> = self.with_frozen_layers(|s| {
            s.battlefield
                .iter()
                .map(|c| Target::Permanent(c.id))
                .chain((0..s.players.len()).map(Target::Player))
                .filter(|t| {
                    // CR 601.2c — one "target" word names distinct objects.
                    !picked.contains(t)
                        && s.evaluate_requirement_static(&filter, t, pending.controller, Some(pending.source))
                        && s.check_target_legality(t, pending.controller).is_ok()
                })
                .collect()
        });
        if let Some(lead) = lead {
            legal.retain(|t| *t != lead);
            legal.insert(0, lead);
        } else {
            return None;
        }
        let source_name = self.find_card_anywhere(pending.source).map(|c| c.definition.name.to_string()).unwrap_or_default();
        Some(Decision::ChooseTarget {
            source: pending.source,
            legal,
            source_name,
            description: format!("Choose another target ({} chosen)", picked.len()),
            optional: true,
            extra_cast_slot: false,
        })
    }
}

impl GameState {
    /// CR 700.2c / 603.3c — a modal trigger's per-mode slots ("each mode must
    /// target a different player": Vindictive Lich), asked positionally after
    /// slot 0, the engine's fill leading. A decline leaves that mode's slot a
    /// hole (`target_hole.rs`), so the mode does nothing; no slot reuses a
    /// target an earlier one named, as the fill never does for `ChooseN`.
    fn next_modal_slot(&self, pending: &PendingTriggerPush, picked: &[Target], seat: usize) -> Option<Decision> {
        if !self.seat_prompts(seat) || picked.is_empty() {
            return None;
        }
        // One slot per target-bearing mode, as the `ChooseN` arm binds them.
        let Effect::ChooseN { picks, modes } = &pending.effect else { return None };
        let mut owners: Vec<u8> = picks
            .iter()
            .copied()
            .filter(|&i| modes.get(i as usize).is_some_and(|m| m.requires_target()))
            .collect();
        owners.dedup();
        let slot = picked.len();
        if slot >= owners.len() {
            return None;
        }
        let filter = pending.effect.target_filter_for_slot_in_mode(u8::try_from(slot).ok()?, pending.mode)?.clone();
        let fill = self.auto_extra_targets_for(&pending.effect, pending.source, pending.controller, picked.first().cloned());
        let mut legal: Vec<Target> = self.with_frozen_layers(|s| {
            s.battlefield
                .iter()
                .map(|c| Target::Permanent(c.id))
                .chain((0..s.players.len()).map(Target::Player))
                .filter(|t| {
                    !picked.contains(t)
                        && s.evaluate_requirement_static(&filter, t, pending.controller, Some(pending.source))
                        && s.check_target_legality(t, pending.controller).is_ok()
                })
                .collect()
        });
        if legal.is_empty() {
            return None;
        }
        if let Some(lead) = fill.get(slot - 1).filter(|t| legal.contains(t)).cloned() {
            legal.retain(|t| *t != lead);
            legal.insert(0, lead);
        }
        let source_name = self.find_card_anywhere(pending.source).map(|c| c.definition.name.to_string()).unwrap_or_default();
        Some(Decision::ChooseTarget {
            source: pending.source,
            legal,
            source_name,
            description: format!("Choose the target for mode slot {}", slot + 1),
            optional: true,
            extra_cast_slot: false,
        })
    }
}
