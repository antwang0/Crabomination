//! Capture of the positions the search actually evaluates.
//!
//! Every calibration diagnostic so far samples the *snapshot* cadence
//! (turn start / postcombat main / end step), but the searches consume
//! evaluations of **simulated leaves** — post-combat and post-line
//! hypotheticals inside `simulate_attack_outcome` and the cast planner's
//! `score_settled_state`. Those are a distribution the net is neither
//! trained on nor measured at, and "better on snapshots, worse on sim
//! leaves" is the one remaining explanation for a net that outpredicts
//! the heuristic and still loses gates as its replacement. This module
//! makes the leaf distribution observable so `selfplay_train
//! --calibrate-leaves` can score both evaluators on it.
//!
//! Off by default; the two hook sites pay one relaxed atomic load per
//! leaf when disabled. The buffer is thread-local because the diagnostic
//! plays its games single-threaded and drains between games — enabling
//! this under a multi-threaded ladder would capture into per-thread
//! buffers that nothing drains, so don't.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use crabomination_nn::EncodedState;

use crate::game::GameState;

static ENABLED: AtomicBool = AtomicBool::new(false);

/// 1-in-N sampling of leaf evaluations. Denser than the first guess of
/// 31: the layer-memo and candidate-queue perf work collapsed repeated
/// evaluations, so a default-profile game only reaches ~43 distinct leaf
/// evals — at 1-in-31 a 300-game run captured 413 leaves, under the
/// diagnostic's own floor.
const SAMPLE_EVERY: u64 = 3;
/// Per-drain cap, so a pathological game can't grow the buffer without
/// bound.
const CAP: usize = 4_096;

/// One captured leaf: the encoded state, its heuristic score, the seat the
/// score is from, and the turn number.
type Leaf = (EncodedState, i32, usize, u32);

thread_local! {
    /// (leaf evaluations seen, captured leaves).
    static BUF: RefCell<(u64, Vec<Leaf>)> = const { RefCell::new((0, Vec::new())) };
}

/// Turn capture on or off for this process.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Hook called at the search's leaf evaluations with the *simulated*
/// state and the heuristic's score for it. Decided sims are skipped: the
/// search ranks those by the ±100 000 result term, not by evaluation, so
/// they are not part of the question.
pub fn maybe(g: &GameState, seat: usize, heur: i32) {
    if !ENABLED.load(Ordering::Relaxed) || g.game_over.is_some() {
        return;
    }
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        b.0 += 1;
        if b.0 % SAMPLE_EVERY == 0 && b.1.len() < CAP {
            let s = super::encode::encode_state(g, seat, super::net_eval::vocab());
            let turn = g.turn_number;
            b.1.push((s, heur, seat, turn));
        }
    });
}

/// Take this thread's captured leaves and reset the sampling counter.
pub fn drain() -> Vec<Leaf> {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        b.0 = 0;
        std::mem::take(&mut b.1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;

    #[test]
    fn capture_samples_only_when_enabled_and_drains_clean() {
        let players = vec![Player::new(0, "A"), Player::new(1, "B")];
        let g = GameState::new(players);
        // Disabled: nothing lands however many evals happen.
        set_enabled(false);
        for _ in 0..(SAMPLE_EVERY * 3) {
            maybe(&g, 0, 42);
        }
        assert!(drain().is_empty());
        // Enabled: 1-in-SAMPLE_EVERY of evaluations are captured.
        set_enabled(true);
        for _ in 0..(SAMPLE_EVERY * 3) {
            maybe(&g, 0, 42);
        }
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].1, 42);
        // Drained means drained.
        assert!(drain().is_empty());
    }
}
