//! `Effect::PreventAllButOneFromChosenSource` — "the next time a [filter]
//! source of your choice would deal damage to you this turn, prevent all but
//! 1 of that damage" (Forcefield, CR 615.7).

use crate::card::SelectionRequirement;
use crate::game::effects::EffectContext;
use crate::game::types::{PreventionShield, PreventionTarget};
use crate::game::{GameError, GameState};

impl GameState {
    pub(super) fn prevent_all_but_one_from_chosen_source(
        &mut self,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
    ) -> Result<(), GameError> {
        let Some(chosen) = self.choose_damage_prevention_source(filter, ctx) else { return Ok(()) };
        self.turn.prevention_shields.push(PreventionShield {
            target: PreventionTarget::Player(ctx.controller),
            source: Some(chosen),
            one_event: true,
            all_but: 1,
            ..Default::default()
        });
        Ok(())
    }
}
