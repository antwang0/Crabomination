//! CR 120.3a / 614 — "damage doesn't cause you to lose life" (Archon of
//! Coronation while you're the monarch). Read at the damage-to-player life
//! sites through `clamp_damage_to_life_floor`, so the damage itself is still
//! dealt and every "deals damage" trigger still fires.

use super::GameState;
use crate::effect::StaticEffect;

impl GameState {
    /// True when a static `seat` controls says damage doesn't make them lose
    /// life. Peeled through `active_static`, so a `WhileCondition` gate (the
    /// Archon's "as long as you're the monarch") is honoured.
    pub(crate) fn damage_causes_no_life_loss(&self, seat: usize) -> bool {
        self.battlefield.iter().any(|src| {
            src.controller == seat
                && src.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        self.active_static(&sa.effect, src),
                        Some(StaticEffect::DamageDoesntCauseControllerLifeLoss)
                    )
                })
        })
    }
}
