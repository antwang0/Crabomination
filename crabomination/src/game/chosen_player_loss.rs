//! CR 603.2 / 104.2a — "When the chosen player loses the game, you win the
//! game" (Shinryu, Transcendent Rival). `EventKind` has no bit left for a
//! player-loss event, so the SBA pass that eliminates players puts the
//! trigger on the stack itself, the way it does a state trigger.

use super::GameState;
use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, StaticEffect};
use crate::game::types::TriggerPush;

impl GameState {
    /// For each permanent with `YouWinWhenChosenPlayerLoses` whose remembered
    /// player is among `lost`, its controller (still in the game) gets a
    /// "you win the game" trigger.
    pub(crate) fn push_chosen_player_loss_wins(&mut self, lost: &[usize]) {
        let wins: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.chosen_player.is_some_and(|p| lost.contains(&p)))
            .filter(|c| !lost.contains(&c.controller) && !self.players[c.controller].eliminated)
            .filter(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::YouWinWhenChosenPlayerLoses))
            })
            .map(|c| (c.id, c.controller))
            .collect();
        for (id, ctrl) in wins {
            self.push_stack(TriggerPush::new(id, ctrl, Effect::WinGame { who: PlayerRef::You }).build());
        }
    }
}
