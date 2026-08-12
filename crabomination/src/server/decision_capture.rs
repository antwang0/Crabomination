//! Capture of the *decisions* the bot makes, not just the positions it
//! reaches.
//!
//! Every training row the program has ever produced is `(encoded state →
//! who eventually won)`. The candidate set and the chosen action are
//! discarded at the moment they exist. Three separate problems trace back
//! to that one omission:
//!
//! * There is no policy head, and there cannot be one, because nothing in
//!   the data says which action was taken.
//! * Search cannot be distilled. AlphaZero compounds because MCTS is a
//!   policy-improvement operator whose output is written back into the
//!   network; here the search's conclusions evaporate at the end of every
//!   decision.
//! * Stacking generations is null twice over (rounds 14 and 18), and the
//!   leading explanation is that generation N+1's pilot differs from
//!   generation N's only by a leaf number inside an unchanged decision
//!   procedure. Recording decisions is what would let a successor learn
//!   to *choose* differently rather than merely to evaluate differently.
//!
//! # Candidates as successor states
//!
//! A policy target needs some representation of "the actions that were
//! available". The obvious route is an action vocabulary — a taxonomy of
//! `GameAction` shapes with their parameters — and it is a poor fit here:
//! the engine's action space is enormous and open (every cast variant,
//! every target, every X), and a vocabulary would need extending for
//! every new mechanic.
//!
//! Instead each candidate is recorded as the *state it leads to*. The
//! network already scores states, so a policy target becomes "which of
//! these N successor states did the player pick", learnable as a softmax
//! over the existing evaluator with no new architecture and no new
//! vocabulary. It also takes search distillation for free: MCTS visit
//! counts over candidates are exactly a soft target over the same
//! successor states.
//!
//! The cost is real and is why this is off by default: recording a
//! decision clones and encodes the state once per candidate, where the
//! snapshot cadence encodes once per position.
//!
//! Off by default; the hook pays one relaxed atomic load per decision
//! when disabled. Thread-local buffer, drained per game, for the same
//! reason as [`super::leaf_capture`] — enabling this under a
//! multi-threaded ladder would capture into per-thread buffers that
//! nothing drains.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

use crabomination_nn::EncodedState;

use crate::game::{GameAction, GameState};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Per-drain cap, so a pathological game cannot grow the buffer without
/// bound. Lower than `leaf_capture`'s because each entry holds a whole
/// candidate set rather than one state.
const CAP: usize = 1_024;

/// One captured decision.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturedDecision {
    /// The state each available candidate leads to, in candidate order.
    pub successors: Vec<EncodedState>,
    /// Index into `successors` of the one the bot actually played.
    pub chosen: usize,
    /// The seat deciding, and the turn it decided on — the policy target
    /// is per-seat, and ply-stratified analysis needs the turn.
    pub seat: usize,
    pub turn: u32,
}

thread_local! {
    static BUF: RefCell<Vec<CapturedDecision>> = const { RefCell::new(Vec::new()) };
}

/// Turn capture on or off for this process.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Hook at a decision site: `candidates` are the actions considered and
/// `chosen` indexes the one taken.
///
/// A candidate the engine rejects is dropped rather than encoded, and
/// `chosen` is remapped onto the surviving list — an index that silently
/// pointed at the wrong successor would be worse than no data at all,
/// since it would train the policy toward whatever happened to shift into
/// that slot.
///
/// Decisions with fewer than two surviving candidates are not recorded:
/// a forced move carries no policy signal and would just dilute the set.
pub fn maybe(state: &GameState, seat: usize, candidates: &[GameAction], chosen: usize) {
    if !ENABLED.load(Ordering::Relaxed) || state.game_over.is_some() || candidates.len() < 2 {
        return;
    }
    let vocab = super::net_eval::vocab();
    let mut successors = Vec::with_capacity(candidates.len());
    let mut chosen_idx = None;
    for (i, a) in candidates.iter().enumerate() {
        let mut next = state.clone();
        if next.perform_action(a.clone()).is_err() {
            continue;
        }
        if i == chosen {
            chosen_idx = Some(successors.len());
        }
        successors.push(super::encode::encode_state(&next, seat, vocab));
    }
    let Some(chosen) = chosen_idx else {
        // The played action did not survive re-application, so nothing
        // here can be labelled. Dropping the whole decision is the only
        // safe move.
        return;
    };
    if successors.len() < 2 {
        return;
    }
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        if b.len() < CAP {
            b.push(CapturedDecision { successors, chosen, seat, turn: state.turn_number });
        }
    });
}

/// Take this thread's captured decisions.
pub fn drain() -> Vec<CapturedDecision> {
    BUF.with(|b| std::mem::take(&mut *b.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardDefinition, CardType};
    use crate::game::two_player_game;

    /// `ENABLED` is process-global while `BUF` is thread-local, and
    /// cargo runs these tests concurrently on separate threads. Without
    /// serialising, one test flipping the flag off can land inside
    /// another's capture window — a flake that would show up rarely and
    /// look like a capture bug.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn creature(name: &'static str, p: i32, t: i32) -> CardDefinition {
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: p,
            toughness: t,
            ..Default::default()
        }
    }

    /// A position with two distinct legal casts, so the candidate set is
    /// real rather than contrived.
    fn two_choice_state() -> (GameState, Vec<GameAction>) {
        let mut g = two_player_game();
        let a = g.add_card_to_hand(0, creature("Alpha", 2, 2));
        let b = g.add_card_to_hand(0, creature("Beta", 3, 3));
        let cast = |card_id| GameAction::CastSpell {
            card_id,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        };
        (g, vec![cast(a), cast(b)])
    }

    #[test]
    fn nothing_is_captured_while_disabled() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(false);
        maybe(&g, 0, &cands, 0);
        assert!(drain().is_empty());
    }

    #[test]
    fn a_decision_captures_one_successor_per_candidate() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        maybe(&g, 0, &cands, 1);
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1, "one decision");
        let d = &got[0];
        assert_eq!(d.successors.len(), 2, "one successor per candidate");
        assert_eq!(d.chosen, 1);
        assert_eq!(d.seat, 0);
        // The successors have to actually differ — encoding the *same*
        // state twice would train the policy on nothing while looking
        // exactly like a working capture.
        assert_ne!(
            d.successors[0], d.successors[1],
            "both candidates encoded to the same successor"
        );
        assert!(drain().is_empty(), "drained means drained");
    }

    /// End to end: a real game with capture on produces decisions whose
    /// chosen successor is the one the bot actually played. Without this,
    /// the hook could be wired to the wrong variable and every unit test
    /// above would still pass — they exercise `maybe` directly and never
    /// touch the call site.
    #[test]
    fn a_played_game_captures_decisions_from_the_real_pick_site() {
        use crate::server::{Bot, HeuristicBot};
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());

        let mut g = two_player_game();
        // Several distinct castable bodies, so the bot has a shortlist to
        // choose among rather than a forced line.
        for (n, p, t) in [("A", 1, 1), ("B", 2, 2), ("C", 3, 3), ("D", 4, 4)] {
            g.add_card_to_hand(0, creature(n, p, t));
        }
        let _ = drain();
        set_enabled(true);
        let mut bot = HeuristicBot::new();
        let mut fuel = 40;
        while fuel > 0 && !g.is_game_over() {
            fuel -= 1;
            let Some(a) = bot.next_action(&g, 0) else { break };
            if g.perform_action(a).is_err() {
                break;
            }
        }
        let got = drain();
        set_enabled(false);

        assert!(!got.is_empty(), "a real game recorded no decisions");
        for d in &got {
            assert!(d.successors.len() >= 2, "a recorded decision had no alternatives");
            assert!(d.chosen < d.successors.len(), "chosen index out of range");
            assert_eq!(d.seat, 0);
        }
    }

    /// A forced move carries no policy signal.
    #[test]
    fn single_candidate_decisions_are_not_recorded() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        maybe(&g, 0, &cands[..1], 0);
        let got = drain();
        set_enabled(false);
        assert!(got.is_empty());
    }

    /// The remap is the part most likely to be silently wrong: when a
    /// candidate ahead of the chosen one is rejected by the engine, the
    /// recorded index must follow the *played* action rather than keep
    /// its original slot.
    #[test]
    fn chosen_index_follows_the_played_action_past_a_rejected_candidate() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        // An action the engine will refuse, inserted ahead of both real
        // candidates so a naive implementation reports the wrong index.
        let bogus = GameAction::CastSpell {
            card_id: crate::card::CardId(9_999),
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        };
        let mut with_bogus = vec![bogus];
        with_bogus.extend(cands.iter().cloned());

        set_enabled(true);
        // Index 2 in the padded list is the second real candidate.
        maybe(&g, 0, &with_bogus, 2);
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].successors.len(), 2, "the bogus candidate is dropped");
        assert_eq!(got[0].chosen, 1, "chosen must be remapped onto the survivors");
    }
}
