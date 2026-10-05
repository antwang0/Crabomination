//! CR 601.2c / 603.3d — the further targets of an "up to N target" triggered
//! ability (Katsumasa's "each of up to three target noncreature artifacts",
//! Depthshaker Titan's "any number of target noncreature artifacts you
//! control") are the controller's to name, slot by slot, not only the first.
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
/// wrappers? Per-opponent / per-player fan-outs and distinct per-slot filters
/// keep the engine's fill.
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
        if !same_filter_fan_out(&effect) {
            return None;
        }
        let fill = self.auto_extra_targets_for(&effect, pending.source, pending.controller, Some(first.clone()));
        if picked.len() > fill.len() {
            return None;
        }
        let filter = effect.target_filter_for_slot_in_mode(1, pending.mode)?.clone();
        let lead = fill.iter().find(|t| !picked.contains(t)).cloned();
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
