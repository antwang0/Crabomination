//! The bot's in-sim answer to a pending ask, with the sims' own CR 732.2 stop.

use crate::decision::{AmountKind, Decision, DecisionAnswer};
use crate::game::GameState;

use super::bot::{EvalWeights, decide_pending_policy};

/// "You may" yeses one sim run takes before it declines. The live bot stops at
/// `OPTIONAL_LOOP_CAP` a step; a sim is an estimate, and Delina copying Wyll
/// makes every further "roll again" dearer (pod seed 2947889, a 20-minute
/// attack sim).
const SIM_OPTIONAL_LOOP_CAP: u16 = 16;

/// [`decide_pending_policy`] for the state's pending ask (`false` eval-modes,
/// the in-sim convention), declining an optional loop past the cap. `yes`
/// counts the run's yeses; one counter per sim loop.
pub(crate) fn sim_pending_answer(g: &GameState, w: &EvalWeights, yes: &mut u16) -> Option<DecisionAnswer> {
    let pending = g.pending_decision.as_ref()?;
    let answer = decide_pending_policy(g, pending.acting_player(), w, &pending.decision, false);
    let upside = match (&pending.decision, &answer) {
        (Decision::OptionalTrigger { .. }, DecisionAnswer::Bool(true)) => Some(DecisionAnswer::Bool(false)),
        (Decision::ChooseAmount { kind: AmountKind::Upside, .. }, DecisionAnswer::Amount(n)) if *n > 0 => {
            Some(DecisionAnswer::Amount(0))
        }
        _ => None,
    };
    match upside {
        Some(stop) if *yes >= SIM_OPTIONAL_LOOP_CAP => Some(stop),
        Some(_) => {
            *yes += 1;
            Some(answer)
        }
        None => Some(answer),
    }
}
