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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crabomination_nn::EncodedState;

use crate::game::{GameAction, GameState};

static ENABLED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Thread-scoped override, ANDed with the global flag. A mixed actor
    /// fleet (round 37) records only its *search* pilots' decisions: the
    /// value fleet's picks are one-hot imitation targets that would
    /// flood the distillation stream at ~50–100× the searched games
    /// rate, burying the targets the stream exists to carry.
    static THREAD_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

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
    /// False: `values` are mean rewards in win-probability units (UCB1
    /// roots — the trainer picks the softmax temperature). True: they are
    /// the Gumbel search's improved-policy logits (`logit + σ(q̂)`),
    /// already in log space, softmaxed at temperature 1 downstream.
    pub values_are_logits: bool,
    /// Rollouts behind each entry of `values`, aligned with
    /// `successors` (round 39). A mean over 30 rollouts and a mean over
    /// 2 are very different evidence, and the search-value regression
    /// weights by this rather than pretending they are the same number.
    pub visits: Option<Vec<u32>>,
    /// The behaviour policy's record (round 72): the i32 scores the live
    /// picker fed `choose_scored` / `sample_scored_index`, aligned with
    /// `successors` and filtered alongside them on the reject remap, like
    /// `values`. With `temp` this makes the probability the pilot played
    /// `chosen` exactly recoverable — `softmax(scores / temp)[chosen]` —
    /// which is what an off-policy policy-gradient step needs for its
    /// importance ratio. `None` for search roots (their record is
    /// `values`).
    pub scores: Option<Vec<i32>>,
    /// Sampling temperature in force at the pick, in the scores' units;
    /// 0 = argmax (deterministic behaviour, no ratio to correct).
    pub temp: i32,
    /// True when `scores` are net units (win probability × 10 000,
    /// `eval_material_frozen`), false when they are material units — the
    /// tail guard fired, or the pick came from a net-free profile. The
    /// behaviour probability is exact either way; this is an analysis tag.
    pub net_scored: bool,
    /// The seat's snapshot index at capture time — `TrainRow::ply` of the
    /// latest row of this seat — so a decision can be joined to the value
    /// stream's row and its λ-return. Set by the recorder through
    /// [`set_ply`]; 0 outside a recorded game.
    pub ply: u16,
}

thread_local! {
    static BUF: RefCell<Vec<CapturedDecision>> = const { RefCell::new(Vec::new()) };
    /// The recorder's per-seat snapshot index, stamped onto every capture
    /// as [`CapturedDecision::ply`]. Thread-local because the recorder and
    /// the bot it drives share a thread, and the hook has no view of the
    /// recorder's counters otherwise.
    static PLY: std::cell::Cell<u16> = const { std::cell::Cell::new(0) };
}

/// Decisions the cap discarded, process-wide. The cap used to be a silent
/// drop: the encode work was paid and nothing said the buffer was full.
static DROPPED: AtomicU64 = AtomicU64::new(0);

/// Set the snapshot index every subsequent capture on this thread carries.
pub fn set_ply(p: u16) {
    PLY.with(|c| c.set(p));
}

/// The snapshot index in force on this thread.
pub fn ply() -> u16 {
    PLY.with(|c| c.get())
}

/// Decisions discarded at the cap since process start.
pub fn dropped() -> u64 {
    DROPPED.load(Ordering::Relaxed)
}

/// Everything a capture site knows about *why* the pick was made, beyond
/// the candidates and the index. All optional; `Default` is a bare
/// heuristic pick with no record.
#[derive(Clone, Copy, Default)]
pub struct Provenance<'a> {
    /// Per-candidate search values — see [`CapturedDecision::values`].
    pub values: Option<&'a [f32]>,
    pub values_are_logits: bool,
    /// Rollouts behind each value — see [`CapturedDecision::visits`].
    pub visits: Option<&'a [u32]>,
    /// The picker's own scores — see [`CapturedDecision::scores`].
    pub scores: Option<&'a [i32]>,
    /// Sampling temperature at the pick; 0 = argmax.
    pub temp: i32,
    /// Whether `scores` are net units.
    pub net_scored: bool,
}

/// Turn capture on or off for this process.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Turn capture on or off for *this thread* (default on). Effective only
/// while the process flag is on — see [`THREAD_ENABLED`].
pub fn set_thread_enabled(on: bool) {
    THREAD_ENABLED.with(|t| t.set(on));
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed) && THREAD_ENABLED.with(|t| t.get())
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
    maybe_valued(state, seat, candidates, chosen, None, false, None)
}

/// [`maybe`] with per-candidate search values attached. `values` and
/// `visits` must be aligned with `candidates`; both are filtered
/// alongside them when the engine rejects one, so the alignment survives
/// the remap. `values_are_logits` labels the values' scale — see
/// [`CapturedDecision::values_are_logits`].
pub fn maybe_valued(
    state: &GameState,
    seat: usize,
    candidates: &[GameAction],
    chosen: usize,
    values: Option<&[f32]>,
    values_are_logits: bool,
    visits: Option<&[u32]>,
) {
    maybe_full(
        state,
        seat,
        candidates,
        chosen,
        Provenance { values, values_are_logits, visits, ..Default::default() },
    )
}

/// The general hook: [`maybe`] with the full [`Provenance`]. Every
/// per-candidate slice in `prov` is filtered alongside `candidates` on the
/// reject remap and attached only if it ends up aligned with the survivors.
pub fn maybe_full(
    state: &GameState,
    seat: usize,
    candidates: &[GameAction],
    chosen: usize,
    prov: Provenance<'_>,
) {
    if !enabled() || state.game_over.is_some() || candidates.len() < 2 {
        return;
    }
    let vocab = super::net_eval::vocab();
    let mut successors = Vec::with_capacity(candidates.len());
    let mut kept_values: Vec<f32> = Vec::new();
    let mut kept_visits: Vec<u32> = Vec::new();
    let mut kept_scores: Vec<i32> = Vec::new();
    let mut chosen_idx = None;
    for (i, a) in candidates.iter().enumerate() {
        let mut next = state.clone();
        // A rejected candidate is thrown away, so `perform_action`'s
        // transaction checkpoint (a second clone of the whole state, and
        // one that shares every CoW zone) would never be read. Same
        // reasoning as `bot::dry_run`.
        if next.perform_action_inner(a.clone()).is_err() {
            continue;
        }
        if i == chosen {
            chosen_idx = Some(successors.len());
        }
        if let Some(v) = prov.values
            && let Some(x) = v.get(i)
        {
            kept_values.push(*x);
        }
        if let Some(n) = prov.visits
            && let Some(x) = n.get(i)
        {
            kept_visits.push(*x);
        }
        if let Some(sc) = prov.scores
            && let Some(x) = sc.get(i)
        {
            kept_scores.push(*x);
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
        if b.len() >= CAP {
            DROPPED.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let n = successors.len();
        let values = (prov.values.is_some() && kept_values.len() == n).then_some(kept_values);
        let visits = (prov.visits.is_some() && kept_visits.len() == n).then_some(kept_visits);
        let scores = (prov.scores.is_some() && kept_scores.len() == n).then_some(kept_scores);
        b.push(CapturedDecision {
            successors,
            chosen,
            seat,
            turn: state.turn_number,
            values,
            values_are_logits: prov.values_are_logits,
            visits,
            scores,
            temp: prov.temp,
            net_scored: prov.net_scored,
            ply: ply(),
        });
    });
}

/// `ENABLED` is process-global while `BUF` is thread-local, and cargo runs
/// tests concurrently on separate threads: without serialising, one test
/// flipping the flag off can land inside another's capture window. Every
/// test that flips the flag, in this module or another, takes this lock.
#[cfg(test)]
pub(crate) static TEST_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Take this thread's captured decisions.
pub fn drain() -> Vec<CapturedDecision> {
    BUF.with(|b| std::mem::take(&mut *b.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardDefinition, CardType};
    use crate::game::two_player_game;

    use super::TEST_SERIAL as SERIAL;

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
            // Round 39: the search-value regression needs the evidence
            // count per arm, aligned the same way, and a searched arm
            // always has at least the seeding rollout behind it.
            let counts = d.visits.as_ref().expect("mcts roots carry visit counts");
            assert_eq!(counts.len(), d.successors.len(), "visits must stay aligned");
            assert!(counts.iter().all(|&c| c > 0), "surviving arms were all rolled out");
            assert!(d.chosen < d.successors.len());
        }
    }

    /// A Gumbel-piloted game records improved-policy logits: values
    /// marked as logits, finite for every surviving arm (an unvisited
    /// arm is completed by its prior, never left as a hole), aligned
    /// with the successors.
    #[test]
    fn gumbel_root_decisions_carry_improved_policy_logits() {
        use crate::server::{Bot, MctsBot, MctsConfig};
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());

        let mut g = two_player_game();
        for (n, p, t) in [("A", 1, 1), ("B", 2, 2), ("C", 3, 3), ("D", 4, 4)] {
            g.add_card_to_hand(0, creature(n, p, t));
            g.add_card_to_hand(1, creature(n, p, t));
        }
        let _ = drain();
        set_enabled(true);
        let mut bot = MctsBot::new(MctsConfig {
            iterations: 8,
            horizon_turns: 1,
            gumbel: true,
            ..Default::default()
        });
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

        let valued: Vec<_> = got.iter().filter(|d| d.values.is_some()).collect();
        assert!(!valued.is_empty(), "no Gumbel root decision was captured");
        for d in valued {
            assert!(d.values_are_logits, "gumbel captures must be marked as logits");
            let v = d.values.as_ref().expect("checked");
            assert_eq!(v.len(), d.successors.len(), "values must stay aligned");
            assert!(
                v.iter().all(|x| x.is_finite()),
                "surviving arms must all carry a completed logit: {v:?}"
            );
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
                for (seat, bot) in bots.iter_mut().enumerate() {
                    if let Some(a) = bot.next_action(&g, seat)
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

    /// The thread override suppresses capture while the process flag is
    /// on — the mixed-fleet contract: a value-fleet thread that forgot
    /// to opt out would flood the stream with imitation targets.
    #[test]
    fn thread_override_suppresses_capture_and_restores() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        set_thread_enabled(false);
        maybe(&g, 0, &cands, 0);
        assert!(drain().is_empty(), "thread-disabled capture must record nothing");
        set_thread_enabled(true);
        maybe(&g, 0, &cands, 0);
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1, "re-enabled thread records again");
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

    /// The behaviour record rides along with the pick: the picker's own
    /// scores and the temperature it sampled at, so the probability the
    /// pilot played `chosen` can be recomputed exactly by the trainer.
    #[test]
    fn behaviour_scores_and_temperature_are_stored() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        maybe_full(
            &g,
            0,
            &cands,
            1,
            Provenance { scores: Some(&[10, 20]), temp: 300, net_scored: true, ..Default::default() },
        );
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].scores.as_deref(), Some(&[10, 20][..]));
        assert_eq!(got[0].temp, 300);
        assert!(got[0].net_scored);
        assert!(got[0].values.is_none(), "a heuristic pick carries no search values");
    }

    /// A bare `maybe` records no behaviour scores and temperature 0 —
    /// the deterministic-behaviour default the trainer reads as "no ratio".
    #[test]
    fn a_bare_capture_has_no_behaviour_record() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        maybe(&g, 0, &cands, 0);
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1);
        assert!(got[0].scores.is_none());
        assert_eq!(got[0].temp, 0);
        assert!(!got[0].net_scored);
    }

    /// Scores follow the same remap as the successors: a rejected
    /// candidate's score is dropped with it, so the survivors' scores stay
    /// aligned with their states.
    #[test]
    fn scores_follow_the_reject_remap() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
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
        maybe_full(
            &g,
            0,
            &with_bogus,
            2,
            Provenance { scores: Some(&[99, 10, 20]), temp: 100, ..Default::default() },
        );
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].successors.len(), 2);
        assert_eq!(got[0].chosen, 1);
        assert_eq!(got[0].scores.as_deref(), Some(&[10, 20][..]), "the bogus score is dropped");
    }

    /// A misaligned score slice degrades to `None` rather than pairing a
    /// state with another candidate's score — the same rule `values` has.
    #[test]
    fn misaligned_scores_are_dropped_not_misattributed() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        maybe_full(&g, 0, &cands, 0, Provenance { scores: Some(&[10]), ..Default::default() });
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1);
        assert!(got[0].scores.is_none());
    }

    /// The recorder's snapshot index is stamped from the thread cell, so
    /// a decision can be joined to the row it followed.
    #[test]
    fn ply_is_stamped_from_the_thread_cell() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        set_enabled(true);
        set_ply(7);
        maybe(&g, 0, &cands, 0);
        set_ply(0);
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].ply, 7);
        assert_eq!(ply(), 0, "the cell is reset for the next game");
    }

    /// The cap discards, and says so: the counter is the only way an actor
    /// can tell a quiet game from a game that overflowed the buffer.
    #[test]
    fn the_cap_is_counted_not_silent() {
        let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (g, cands) = two_choice_state();
        let _ = drain();
        set_enabled(true);
        let before = dropped();
        for _ in 0..=CAP {
            maybe(&g, 0, &cands, 0);
        }
        let got = drain();
        set_enabled(false);
        assert_eq!(got.len(), CAP, "the buffer holds exactly the cap");
        assert_eq!(dropped() - before, 1, "one decision past the cap was dropped and counted");
    }
}
