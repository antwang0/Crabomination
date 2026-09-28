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
    controller: usize,
    turn: u32,
) -> DelayedKind {
    match k {
        // `after_turn` one back lets this turn's end step count when it is
        // the controller's and hasn't begun yet.
        DelayedTriggerKind::YourNextEndStep => {
            DelayedKind::PlayersNextEndStep { player: controller, after_turn: turn.saturating_sub(1) }
        }
        DelayedTriggerKind::YourNextUpkeep => DelayedKind::YourNextUpkeep,
        DelayedTriggerKind::NextEndStep => DelayedKind::NextEndStep,
        DelayedTriggerKind::OpponentPermanentDamagesYouThisTurn => DelayedKind::OpponentPermanentDamagesYouThisTurn,
        DelayedTriggerKind::NextCleanupStep => DelayedKind::NextCleanupStep,
        DelayedTriggerKind::YourNextMainPhase => DelayedKind::YourNextMainPhase,
        DelayedTriggerKind::YourNextMainPhaseAny | DelayedTriggerKind::NextMainPhaseThisTurn => {
            DelayedKind::YourNextMainPhaseAny
        }
        DelayedTriggerKind::EndOfCombat => DelayedKind::EndOfCombat,
        DelayedTriggerKind::NextCombat => DelayedKind::NextCombat,
        DelayedTriggerKind::CreatureAttacksYouUntilYourNextTurn => {
            DelayedKind::CreatureAttacksYouUntilYourNextTurn
        }
        DelayedTriggerKind::TargetsNextEndStep => match target_player {
            Some(player) => DelayedKind::PlayersNextEndStep { player, after_turn: turn },
            None => DelayedKind::NextEndStep,
        },
        DelayedTriggerKind::TargetsNextDrawStep => match target_player {
            Some(player) => DelayedKind::PlayersNextDrawStep { player, after_turn: turn - 1 },
            None => DelayedKind::NextEndStep,
        },
    }
}

/// The `expires_after_turn` a delayed kind carries: a stated "this turn"
/// duration (CR 603.7b) lapses at this turn's cleanup, fired or not. Every
/// other kind runs on its own clock.
pub(crate) fn delayed_expiry(k: DelayedTriggerKind, turn: u32) -> Option<u32> {
    matches!(k, DelayedTriggerKind::NextMainPhaseThisTurn).then_some(turn)
}
