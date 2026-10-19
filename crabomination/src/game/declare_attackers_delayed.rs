//! CR 508.2 — "at the beginning of [player]'s next declare attackers step"
//! (Noctis, Heir Apparent's warp-strike return). The step's own triggers wait
//! for the declaration (CR 508.2), so the hook runs as `declare_attackers_full`
//! finishes — attackers or none (Noctis's ruling: "it will return even if its
//! owner declares no attackers").

use crate::game::types::{DelayedKind, DelayedTrigger};
use crate::game::GameState;

impl GameState {
    /// Fire the delayed triggers waiting on the active player's declare
    /// attackers step. A read before the take: nearly every declaration has
    /// none.
    pub(crate) fn fire_declare_attackers_delayed(&mut self) {
        let active = self.active_player_idx;
        let turn = self.turn_number;
        let matches = |dt: &DelayedTrigger| {
            matches!(dt.kind, DelayedKind::PlayersNextDeclareAttackers { player, after_turn }
                if player == active && turn > after_turn)
        };
        if !self.delayed_triggers.iter().any(matches) {
            return;
        }
        let mut fire = Vec::new();
        let mut keep = Vec::new();
        for dt in std::mem::take(&mut self.delayed_triggers) {
            if matches(&dt) { fire.push(dt) } else { keep.push(dt) }
        }
        self.delayed_triggers = keep;
        for dt in fire {
            let mode = self.pick_trigger_mode(&dt.effect, dt.source, dt.controller);
            self.push_pending_trigger(
                crate::game::types::PendingTriggerPush {
                    from_mana_ability: false,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                    actor: None,
                    source: dt.source,
                    controller: dt.controller,
                    effect: dt.effect,
                    subject: dt.bound_subject,
                    event_amount: 0,
                    mode,
                    intervening_if: None,
                },
                dt.target,
            );
        }
    }
}
