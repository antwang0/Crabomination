//! "That player [does X]" — an effect run with another seat as its controller
//! (`Effect::AsPlayer`).

use super::{EffectContext, rewrap_parked};
use crate::effect::{Effect, PlayerRef};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// Runs `body` as `who`. A body that parks for a prompt resumes under the
    /// stack item's own context, so its continuation is re-wrapped with the
    /// seat pinned (a `ChosenPlayerOfSource` pick is scratch and would be gone).
    pub(super) fn run_as_player(
        &mut self,
        who: &PlayerRef,
        body: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        if !self.players.get(seat).is_some_and(|p| p.is_alive()) {
            return Ok(());
        }
        self.run_effect(body, &EffectContext { controller: seat, ..ctx.clone() }, events)?;
        rewrap_parked(&mut self.suspend_signal, |carried| Effect::AsPlayer {
            who: PlayerRef::Seat(seat),
            body: Box::new(carried),
        });
        Ok(())
    }
}
