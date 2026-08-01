//! Adapter between effect-tree delayed triggers and game-state delayed kinds.

use crate::effect::DelayedTriggerKind;
use crate::game::DelayedKind;

/// Translate an `Effect`-side `DelayedTriggerKind` to its game-state mirror
/// `DelayedKind`. Centralized so adding a new delayed-trigger kind requires
/// only this one pattern match update. `target_player` / `turn` feed the
/// player-scoped kinds; a `TargetsNextEndStep` with no resolved player falls
/// back to the next end step on any turn.
pub(crate) fn delayed_kind_from_effect(
    k: DelayedTriggerKind,
    target_player: Option<usize>,
    turn: u32,
) -> DelayedKind {
    match k {
        DelayedTriggerKind::YourNextUpkeep => DelayedKind::YourNextUpkeep,
        DelayedTriggerKind::NextEndStep => DelayedKind::NextEndStep,
        DelayedTriggerKind::NextCleanupStep => DelayedKind::NextCleanupStep,
        DelayedTriggerKind::YourNextMainPhase => DelayedKind::YourNextMainPhase,
        DelayedTriggerKind::EndOfCombat => DelayedKind::EndOfCombat,
        DelayedTriggerKind::NextCombat => DelayedKind::NextCombat,
        DelayedTriggerKind::CreatureAttacksYouUntilYourNextTurn => {
            DelayedKind::CreatureAttacksYouUntilYourNextTurn
        }
        DelayedTriggerKind::TargetsNextEndStep => match target_player {
            Some(player) => DelayedKind::PlayersNextEndStep { player, after_turn: turn },
            None => DelayedKind::NextEndStep,
        },
    }
}
