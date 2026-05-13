//! Adapter between effect-tree delayed triggers and game-state delayed kinds.

use crate::effect::DelayedTriggerKind;
use crate::game::DelayedKind;

/// Translate an `Effect`-side `DelayedTriggerKind` to its game-state mirror
/// `DelayedKind`. Centralized so adding a new delayed-trigger kind requires
/// only this one pattern match update.
pub(crate) fn delayed_kind_from_effect(k: DelayedTriggerKind) -> DelayedKind {
    match k {
        DelayedTriggerKind::YourNextUpkeep => DelayedKind::YourNextUpkeep,
        DelayedTriggerKind::NextEndStep => DelayedKind::NextEndStep,
        DelayedTriggerKind::YourNextMainPhase => DelayedKind::YourNextMainPhase,
    }
}
