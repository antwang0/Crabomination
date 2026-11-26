//! CR 601.2f / 734.1 — a spell that costs more for each target beyond the
//! first (strive, CR 207.2c; Fireball). The "choose an additional target"
//! prompt must not offer a slot the caster can't pay for: picking one rewound
//! the whole cast (a `--a uniform` seat re-cast Launch the Fleet 643 times).

use super::GameState;
use super::types::{GameAction, Target};
use crate::card::CardId;

impl GameState {
    /// Whether casting `card_id` with `extra` appended to its targets is
    /// still payable. Only asked for a per-extra-target-cost spell; the probe
    /// declines any further slot, the same reading `accept_on` gives a bot.
    pub(crate) fn surcharged_slot_affordable(
        &self,
        card_id: CardId,
        target: &Option<Target>,
        additional_targets: &[Target],
        extra: Target,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> bool {
        let mut more = additional_targets.to_vec();
        more.push(extra);
        Self::would_accept_on(
            self,
            GameAction::CastSpell {
                card_id,
                target: target.clone(),
                additional_targets: more,
                mode,
                x_value,
            },
        )
    }
}
