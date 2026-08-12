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
    /// Per-candidate value from a *search*, aligned with `successors`,
    /// when the decision came from one. `None` for heuristic picks.
    ///
    /// This is what makes distillation possible rather than mere
    /// imitation. A one-hot target teaches the net to copy the pilot and
    /// therefore caps it at the pilot; a search's per-candidate values
    /// carry how much better each option looked, and the search is
    /// stronger than the evaluator inside it. Stored raw (mean reward per
    /// arm, in win-probability units) so the trainer picks the
    /// temperature rather than baking one in here.
    ///
    /// Note this is *mean reward*, not visit counts. `MctsBot::search`
    /// selects by highest mean, because at 64 iterations visit counts are
    /// dominated by the one-per-arm seeding pass and carry little signal
    /// — so means are the faithful record of what the search preferred.
    pub values: Option<Vec<f32>>,
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
    maybe_valued(state, seat, candidates, chosen, None)
}

/// [`maybe`] with per-candidate search values attached. `values` must be
/// aligned with `candidates`; it is filtered alongside them when the
/// engine rejects one, so the alignment survives the remap.
pub fn maybe_valued(
    state: &GameState,
    seat: usize,
    candidates: &[GameAction],
    chosen: usize,
    values: Option<&[f32]>,
) {
    if !ENABLED.load(Ordering::Relaxed) || state.game_over.is_some() || candidates.len() < 2 {
        return;
    }
    let vocab = super::net_eval::vocab();
    let mut successors = Vec::with_capacity(candidates.len());
    let mut kept_values: Vec<f32> = Vec::new();
    let mut chosen_idx = None;
    for (i, a) in candidates.iter().enumerate() {
        let mut next = state.clone();
        if next.perform_action(a.clone()).is_err() {
            continue;
        }
        if i == chosen {
            chosen_idx = Some(successors.len());
        }
        if let Some(v) = values
            && let Some(x) = v.get(i)
        {
            kept_values.push(*x);
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
            let values = (values.is_some() && kept_values.len() == successors.len())
                .then_some(kept_values);
            b.push(CapturedDecision {
                successors,
                chosen,
                seat,
                turn: state.turn_number,
                values,
            });
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

    /// An MCTS-piloted game must record the *search's* root decisions,
    /// with its per-arm values attached. Before the hook in
    /// `MctsBot::search`, the recorder only ever saw the heuristic
    /// fallback's picks, so an MCTS run would have measured the wrong
    /// policy while looking like it worked.
    #[test]
    fn mcts_root_decisions_are_captured_with_their_values() {
        use crate::server::{Bot, MctsBot, MctsConfig};
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());

        let mut g = two_player_game();
        for (n, p, t) in [("A", 1, 1), ("B", 2, 2), ("C", 3, 3), ("D", 4, 4)] {
            g.add_card_to_hand(0, creature(n, p, t));
            g.add_card_to_hand(1, creature(n, p, t));
        }
        let _ = drain();
        set_enabled(true);
        let mut bot = MctsBot::new(MctsConfig { iterations: 8, horizon_turns: 1, ..Default::default() });
        let mut fuel = 20;
        while fuel > 0 && !g.is_game_over() {
            fuel -= 1;
            let Some(a) = bot.next_action(&g, 0) else { break };
            if g.perform_action(a).is_err() {
                break;
            }
        }
        let got = drain();
        set_enabled(false);

        assert!(!got.is_empty(), "an MCTS-piloted game recorded no decisions");
        let valued = got.iter().filter(|d| d.values.is_some()).count();
        assert!(valued > 0, "no decision carried search values: {} captured", got.len());
        for d in got.iter().filter(|d| d.values.is_some()) {
            let v = d.values.as_ref().expect("checked");
            assert_eq!(
                v.len(),
                d.successors.len(),
                "values must stay aligned with successors after the reject remap"
            );
            assert!(d.chosen < d.successors.len());
        }
    }

    /// Diagnostic: the candidate-count distribution, which sets the
    /// chance rate that `val_policy` has to be read against. Asserting
    /// "chance is somewhere between 0.33 and 0.5" is not good enough
    /// when the measured value falls inside that band.
    #[test]
    #[ignore = "diagnostic"]
    fn print_mcts_candidate_count_distribution() {
        use crate::server::{Bot, MctsBot, MctsConfig};
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let mut hist = std::collections::BTreeMap::<usize, usize>::new();
        for seed in 0..12u64 {
            let mut g = two_player_game();
            for i in 0..8 {
                g.add_card_to_hand(0, creature("X", 1 + (i % 4), 1 + ((i + seed as i32) % 4)));
                g.add_card_to_hand(1, creature("Y", 1 + (i % 3), 2));
            }
            let _ = drain();
            set_enabled(true);
            let mut bot =
                MctsBot::new(MctsConfig { iterations: 8, horizon_turns: 1, ..Default::default() });
            let mut fuel = 40;
            while fuel > 0 && !g.is_game_over() {
                fuel -= 1;
                let Some(a) = bot.next_action(&g, 0) else { break };
                if g.perform_action(a).is_err() {
                    break;
                }
            }
            for d in drain() {
                if d.values.is_some() {
                    *hist.entry(d.successors.len()).or_default() += 1;
                }
            }
            set_enabled(false);
        }
        let total: usize = hist.values().sum();
        let mut chance = 0.0;
        for (k, n) in &hist {
            eprintln!("  MCTS {k} candidates: {n}");
            chance += (*n as f64) / (*k as f64);
        }
        eprintln!("  MCTS total {total}, chance rate {:.4}", chance / total.max(1) as f64);
    }

    #[test]
    #[ignore = "diagnostic"]
    fn print_candidate_count_distribution() {
        use crate::server::{Bot, HeuristicBot};
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let mut hist = std::collections::BTreeMap::<usize, usize>::new();
        for seed in 0..40u64 {
            let mut g = two_player_game();
            for i in 0..8 {
                g.add_card_to_hand(0, creature("X", 1 + (i % 4), 1 + ((i + seed as i32) % 4)));
                g.add_card_to_hand(1, creature("Y", 1 + (i % 3), 2));
            }
            let _ = drain();
            set_enabled(true);
            let mut bots = [HeuristicBot::new(), HeuristicBot::new()];
            let mut fuel = 120;
            while fuel > 0 && !g.is_game_over() {
                fuel -= 1;
                let mut acted = false;
                for seat in 0..2 {
                    if let Some(a) = bots[seat].next_action(&g, seat)
                        && g.perform_action(a).is_ok()
                    {
                        acted = true;
                    }
                }
                if !acted {
                    break;
                }
            }
            for d in drain() {
                *hist.entry(d.successors.len()).or_default() += 1;
            }
            set_enabled(false);
        }
        let total: usize = hist.values().sum();
        let mut chance = 0.0;
        for (k, n) in &hist {
            eprintln!("  {k} candidates: {n}");
            chance += (*n as f64) / (*k as f64);
        }
        eprintln!("  total {total}, chance rate {:.4}", chance / total.max(1) as f64);
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
