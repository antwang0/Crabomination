//! A Monte Carlo bot: pick the play whose random continuations turn out
//! best, rather than the play that scores best right now.
//!
//! The heuristic bot in [`super::bot`] chooses by *evaluating* a position —
//! a hand-written function saying what a board is worth. This one chooses
//! by *playing the game out*: for each candidate it simulates a number of
//! short continuations with a cheap policy, averages how they end up, and
//! takes the best average. The appeal is that it needs no opinion about
//! what a board is worth beyond the final score, so it can find lines
//! nobody encoded a rule for.
//!
//! ## What this is, precisely
//!
//! Root-level Monte Carlo with UCB1 allocation and depth-limited rollouts —
//! not full MCTS. There is one level of tree (the root's candidate plays);
//! below that, rollouts run to a turn horizon and the position is scored
//! with the same evaluator the heuristic bot uses.
//!
//! That is a deliberate choice, not a shortcut taken blindly. Growing a
//! deeper tree only pays when rollouts are cheap enough to afford many of
//! them, and here they are not: this engine runs its bots inside debug
//! builds (see CLAUDE.md), where a single game costs on the order of half a
//! second, so a rollout to game end is thousands of times more expensive
//! than the heuristic bot's entire decision. Depth-limiting the rollout and
//! spending the whole budget at the root is what makes any Monte Carlo
//! search affordable at all here. [`MctsConfig`] exposes the knobs so the
//! trade can be re-measured rather than argued about.
//!
//! ## Measured result
//!
//! **It loses, and it isn't close.** Against the heuristic bot over 1000
//! ladder games it wins 41.5 % [38.5 %, 44.6 %], consistently across all
//! four archetypes, while costing about 33x as much wall clock (1.25
//! games/sec against ~41). Nothing here is adopted; it exists as a
//! measured, reproducible answer to "what about Monte Carlo".
//!
//! `heuristic_rollouts` narrows it — 45.0 % [38.8 %, 51.3 %] over 240
//! games, inconclusive at that size but clearly better than 41.5 % — at
//! ~54x cost. That direction is the diagnosis confirming itself: rollout
//! quality is the binding constraint, not the number of samples.
//!
//! The result is what the theory predicts for this shape of game, and the
//! reasons are worth writing down because they are properties of Magic and
//! of this engine, not bugs to be fixed:
//!
//! * **The rollout policy is the estimator.** Monte Carlo replaces domain
//!   knowledge with sampling, so a uniform-random rollout is only as good
//!   as the law of large numbers makes it — and 24 samples over a two-turn
//!   horizon is a *very* noisy estimate. The heuristic evaluator it is
//!   competing against is not noisy at all; it encodes years of accreted
//!   knowledge about what a board is worth. Beating it by sampling needs
//!   either far more samples or far better rollouts.
//! * **Rollouts can't reach a result.** In games where MCTS shines, a
//!   playout ends in a win or a loss and the reward is ground truth. Here a
//!   playout is truncated after a couple of turns and scored with... the
//!   same heuristic evaluator. So the search is not escaping the
//!   evaluator's opinion, it is averaging noisy samples *of* that opinion.
//! * **Branching is enormous and mostly irrelevant.** Most legal actions in
//!   a Magic priority window are passes and mana taps; the interesting
//!   branching is narrow but deep, which is the worst shape for a search
//!   that spends its budget at the root.
//!
//! The productive directions from here, in the order the evidence supports:
//! give rollouts a strong policy (`heuristic_rollouts`) so each sample is
//! worth more, and get the cost down enough to afford real depth — which in
//! this engine means the release build the project deliberately doesn't
//! use for bots. Both are measurable with the knobs already here.
//!
//! ## Hidden information
//!
//! Rollouts re-shuffle every library first. Without that, a rollout draws
//! the cards that are *actually* on top, so the search optimises against
//! one known future rather than the distribution — the classic
//! determinisation failure. Hands stay as they are: this bot sees the same
//! state the heuristic bot does, so that the ladder compares policies
//! rather than information.

use rand::RngExt;
use rand::seq::SliceRandom;
use rand::rng;

use crate::decision::{AutoDecider, Decider};
use crate::game::{GameAction, GameState, TurnStep};
use crate::recommend::STALE_ROUNDS;

use super::bot::{Bot, EvalWeights, RandomBot};

/// Tunables for [`MctsBot`]. Every one of these trades search quality for
/// wall clock, and the right setting is a measurement, so none of them are
/// baked in.
#[derive(Debug, Clone, Copy)]
pub struct MctsConfig {
    /// Rollouts per decision, spread across the root's candidates by UCB1.
    pub iterations: u32,
    /// How many turns a rollout plays before the position is scored.
    /// Rollouts are depth-limited rather than run to a result because a
    /// full game is far too expensive to repeat here.
    pub horizon_turns: u32,
    /// UCB1 exploration constant, scaled against the normalised reward.
    pub exploration: f64,
    /// Weights for the leaf evaluation and for the rollout policy.
    pub weights: EvalWeights,
    /// Use the heuristic bot as the rollout policy instead of uniform
    /// random. Better rollouts are better estimates but cost far more per
    /// simulation, which at a fixed time budget buys fewer of them.
    pub heuristic_rollouts: bool,
    /// P-UCT prior strength (round 29). 0 is pure UCB1 — every arm starts
    /// equal and the seeding pass burns one rollout per candidate finding
    /// out what the candidate generator's scores already said. Positive
    /// values steer early visits by a softmax over those scores
    /// (AlphaZero-style `w · p · √parent / (1+n)`); the prior washes out
    /// as visits accumulate, so it biases the budget, not the verdict.
    pub prior_weight: f64,
    /// Softmax temperature for the priors, in `EvalWeights::unit`s of
    /// candidate score: a score gap of this many units is one e-fold of
    /// prior ratio.
    pub prior_temp: f64,
    /// Stop the base budget early once the leader is statistically
    /// decided (its confidence lower bound clears every rival's upper
    /// bound). Spends nothing on forced moves; the saving is the point —
    /// strength should be unchanged.
    pub early_stop: bool,
    /// Budget multiplier for close calls: after the base iterations, keep
    /// searching (up to `iterations · extend_close`) while the top two
    /// means sit within [`Self::close_margin`]. 1.0 disables. This is the
    /// value-of-computation trade: a fixed budget spends the same on a
    /// forced pass as on a razor-thin decision.
    pub extend_close: f64,
    /// How close "close" is, in reward (win-probability) units.
    pub close_margin: f64,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            iterations: 24,
            horizon_turns: 2,
            exploration: 1.0,
            weights: EvalWeights::default(),
            heuristic_rollouts: false,
            prior_weight: 0.0,
            prior_temp: 4.0,
            early_stop: false,
            extend_close: 1.0,
            close_margin: 0.03,
        }
    }
}

/// Softmax with max-subtraction. `temp` in the same units as `scores`.
fn softmax_priors(scores: &[f64], temp: f64) -> Vec<f64> {
    let t = temp.max(1e-9);
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = scores.iter().map(|s| ((s - max) / t).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.into_iter().map(|e| e / sum).collect()
}

/// The arm to sample next. Arms parked at `visits == u32::MAX` (rejected
/// at the root) are skipped. With `prior_weight > 0` this is AlphaZero's
/// P-UCT (`Q + w·p·√parent/(1+n)`); otherwise plain UCB1.
fn pick_arm(
    visits: &[u32],
    totals: &[f64],
    priors: &[f64],
    parent: f64,
    exploration: f64,
    prior_weight: f64,
) -> Option<usize> {
    (0..visits.len()).filter(|&i| visits[i] != u32::MAX).max_by(|&a, &b| {
        let score = |i: usize| {
            let v = visits[i].max(1) as f64;
            let q = totals[i] / v;
            if prior_weight > 0.0 {
                q + prior_weight * priors[i] * parent.sqrt() / (1.0 + visits[i] as f64)
            } else {
                q + exploration * (parent.ln() / v).sqrt()
            }
        };
        score(a).partial_cmp(&score(b)).unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Mean rewards of the two best live arms, best first. `None` with fewer
/// than two live arms — with one candidate there is nothing to decide.
fn top_two_means(visits: &[u32], totals: &[f64]) -> Option<(f64, f64)> {
    let mut best = f64::NEG_INFINITY;
    let mut second = f64::NEG_INFINITY;
    let mut live = 0usize;
    for i in 0..visits.len() {
        if visits[i] == u32::MAX || visits[i] == 0 {
            continue;
        }
        live += 1;
        let m = totals[i] / visits[i] as f64;
        if m > best {
            second = best;
            best = m;
        } else if m > second {
            second = m;
        }
    }
    (live >= 2).then_some((best, second))
}

/// Is the best arm's confidence lower bound above every rival's upper
/// bound? Bounds are the UCB1 radius (`c·√(ln parent / n)`), so "decided"
/// is measured on the same scale the search explores with.
fn leader_decided(visits: &[u32], totals: &[f64], parent: f64, exploration: f64) -> bool {
    let radius = |i: usize| {
        let v = visits[i].max(1) as f64;
        exploration * (parent.ln() / v).sqrt()
    };
    let mut best: Option<usize> = None;
    for i in 0..visits.len() {
        if visits[i] == u32::MAX || visits[i] == 0 {
            continue;
        }
        let better = match best {
            Some(b) => totals[i] / visits[i] as f64 > totals[b] / visits[b] as f64,
            None => true,
        };
        if better {
            best = Some(i);
        }
    }
    let Some(b) = best else { return false };
    let lb = totals[b] / visits[b] as f64 - radius(b);
    (0..visits.len())
        .filter(|&i| i != b && visits[i] != u32::MAX && visits[i] > 0)
        .all(|i| totals[i] / visits[i] as f64 + radius(i) < lb)
}

/// Monte Carlo bot. Falls back to the heuristic bot for everything that
/// isn't a main-phase play — combat declarations, decisions, instant-speed
/// responses — so the comparison isolates *which spell to cast* rather than
/// re-deriving all of combat from scratch.
pub struct MctsBot {
    cfg: MctsConfig,
    fallback: RandomBot,
}

impl MctsBot {
    pub fn new(cfg: MctsConfig) -> Self {
        Self { cfg, fallback: RandomBot::with_weights(cfg.weights) }
    }

    /// Score a finished rollout on a 0..=1 scale.
    ///
    /// A decided game pins to the extremes; anything else is squashed from
    /// the material evaluation. UCB1 assumes bounded rewards, and material
    /// scores are unbounded and scale-dependent, so feeding them in raw
    /// would make the exploration constant meaningless.
    fn reward(&self, g: &GameState, seat: usize) -> f64 {
        if let Some(over) = g.game_over {
            return match over {
                Some(w) if w == seat => 1.0,
                Some(_) => 0.0,
                None => 0.5,
            };
        }
        // The value net, when the profile carries one, is a *native* UCB1
        // reward: a calibrated win probability already in [0, 1], with no
        // squash scale to tune. This is a better fit than it ever was as
        // a replacement inside the shallow sims — those needed fine
        // discrimination between near-identical lines, while a bandit
        // over rollout outcomes needs exactly a bounded win estimate.
        // Empty slot falls through to the heuristic, per the EvalWeights
        // convention.
        if self.cfg.weights.net_slot != 0
            && let Some(p) = super::net_eval::win_prob(g, seat, self.cfg.weights.net_slot)
        {
            return p as f64;
        }
        let material = super::bot::eval_material_for_mcts(g, seat, &self.cfg.weights) as f64;
        // Logistic squash. The scale is set so a swing of roughly one
        // creature moves the reward appreciably without saturating.
        0.5 + 0.5 * (material / 60.0).tanh()
    }

    /// Play `g` forward to the horizon and score it.
    fn rollout(&self, mut g: GameState, seat: usize) -> f64 {
        let mut r = rng();
        // Determinise: the real top of each library is information a search
        // has no business exploiting.
        for p in &mut g.players {
            let mut lib = std::mem::take(&mut p.library);
            lib.shuffle(&mut r);
            p.library = lib;
        }
        // Under a determinizing profile the opponent's *hand* is redealt
        // too (the library shuffle above never covered it — rollouts have
        // been reading the held cards since the first MCTS experiment).
        // Salted per rollout so the bandit averages over imagined hands.
        if self.cfg.weights.determinize > 0 {
            super::bot::determinize_hidden(&mut g, seat, 0x3C75_0000 ^ r.random::<u32>() as u64);
        }
        let stop_turn = g.turn_number + self.cfg.horizon_turns;
        let mut policy: Vec<RandomBot> = (0..g.players.len())
            .map(|_| {
                if self.cfg.heuristic_rollouts {
                    RandomBot::with_weights(self.cfg.weights)
                } else {
                    RandomBot::uniform_baseline()
                }
            })
            .collect();
        let mut fuel = 400u32;
        let mut stale = 0u32;
        while !g.is_game_over() && g.turn_number < stop_turn && fuel > 0 && (stale as usize) < STALE_ROUNDS {
            fuel -= 1;
            if g.pending_decision.is_some() {
                let answer = {
                    let pending = g.pending_decision.as_ref().unwrap();
                    AutoDecider.decide(&pending.decision)
                };
                if g.perform_action(GameAction::SubmitDecision(answer)).is_err() {
                    break;
                }
                continue;
            }
            let mut acted = false;
            for (s, p) in policy.iter_mut().enumerate() {
                let Some(a) = p.next_action(&g, s) else { continue };
                if g.perform_action(a).is_ok() {
                    acted = true;
                    if g.is_game_over() {
                        break;
                    }
                }
            }
            if acted { stale = 0 } else { stale += 1 }
        }
        self.reward(&g, seat)
    }

    /// Run the search over scored `candidates` and return the best.
    fn search(&self, state: &GameState, seat: usize, candidates: Vec<(GameAction, i32)>) -> GameAction {
        let n = candidates.len();
        let unit = self.cfg.weights.unit.max(1) as f64;
        let priors = softmax_priors(
            &candidates.iter().map(|(_, s)| *s as f64 / unit).collect::<Vec<_>>(),
            self.cfg.prior_temp,
        );
        let candidates: Vec<GameAction> = candidates.into_iter().map(|(a, _)| a).collect();
        let mut visits = vec![0u32; n];
        let mut total = vec![0.0f64; n];
        // Seed every arm once so UCB1 has a finite term for each.
        for i in 0..n {
            let mut g = state.clone();
            if g.perform_action(candidates[i].clone()).is_err() {
                // Rejected at the root: park it at the bottom.
                visits[i] = u32::MAX;
                total[i] = f64::NEG_INFINITY;
                continue;
            }
            total[i] = self.rollout(g, seat);
            visits[i] = 1;
        }
        let base = self.cfg.iterations;
        let hard_max = (base as f64 * self.cfg.extend_close.max(1.0)).round() as u32;
        let mut done: u32 = n as u32;
        while done < hard_max {
            let parent = done.max(1) as f64;
            if done >= base {
                // Extension phase: only a still-close call earns more
                // budget; separation is the stop.
                match top_two_means(&visits, &total) {
                    Some((m1, m2)) if m1 - m2 < self.cfg.close_margin => {}
                    _ => break,
                }
            } else if self.cfg.early_stop
                && done >= (n as u32).saturating_add(base / 4)
                && leader_decided(&visits, &total, parent, self.cfg.exploration)
            {
                break;
            }
            let Some(i) = pick_arm(
                &visits,
                &total,
                &priors,
                parent,
                self.cfg.exploration,
                self.cfg.prior_weight,
            ) else {
                break;
            };
            let mut g = state.clone();
            if g.perform_action(candidates[i].clone()).is_err() {
                visits[i] = u32::MAX;
                continue;
            }
            total[i] += self.rollout(g, seat);
            visits[i] += 1;
            done += 1;
        }
        // Highest mean reward wins. (Robust-child — most visits — is the
        // usual MCTS choice, but with this few iterations the visit counts
        // are dominated by the seeding pass and carry little signal.)
        let best = (0..n)
            .filter(|&i| visits[i] != u32::MAX)
            .max_by(|&a, &b| {
                let mean = |i: usize| total[i] / visits[i].max(1) as f64;
                mean(a).partial_cmp(&mean(b)).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(0);
        candidates.into_iter().nth(best).unwrap_or(GameAction::PassPriority)
    }
}

impl Bot for MctsBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        // Only the main-phase play choice is searched; everything else is
        // the heuristic bot's, unchanged.
        let searchable = state.pending_decision.is_none()
            && state.player_with_priority() == seat
            && state.active_player_idx == seat
            && state.stack.is_empty()
            && matches!(state.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
            && !state.is_game_over();
        if !searchable {
            return self.fallback.next_action(state, seat);
        }
        let mut candidates = super::bot::main_phase_candidates_for_mcts(state, seat, &self.cfg.weights);
        if candidates.is_empty() {
            return self.fallback.next_action(state, seat);
        }
        // Passing is always an option: a search that must spend something
        // will always find a reason to. Score zero — the do-nothing
        // baseline every candidate's score is implicitly measured against.
        candidates.push((GameAction::PassPriority, 0));
        Some(self.search(state, seat, candidates))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn softmax_priors_are_a_distribution_ordered_by_score() {
        let p = softmax_priors(&[4.0, 0.0, 2.0], 4.0);
        assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(p[0] > p[2] && p[2] > p[1], "monotone in score: {p:?}");
        // One temperature unit of score gap is one e-fold of ratio.
        assert!((p[0] / p[1] - 1.0f64.exp()).abs() < 1e-9);
        // Higher temperature flattens toward uniform.
        let flat = softmax_priors(&[4.0, 0.0, 2.0], 400.0);
        assert!(flat[0] - flat[1] < 0.01, "near-uniform at high temp: {flat:?}");
    }

    #[test]
    fn pick_arm_without_priors_is_ucb1() {
        // Arm 0: mean 0.8 over 2 visits; arm 1: mean 0.4 over 1 visit.
        // At parent 3, c = 1: ucb0 = 0.8 + √(ln3/2) ≈ 1.541,
        // ucb1 = 0.4 + √(ln3/1) ≈ 1.448 → arm 0. At c = 4 the wider bonus
        // flips it: 0.8 + 4·0.741 = 3.764 < 0.4 + 4·1.048 = 4.592.
        let visits = [2, 1];
        let totals = [1.6, 0.4];
        let uniform = [0.5, 0.5];
        assert_eq!(pick_arm(&visits, &totals, &uniform, 3.0, 1.0, 0.0), Some(0));
        assert_eq!(pick_arm(&visits, &totals, &uniform, 3.0, 4.0, 0.0), Some(1));
        // Parked arms are never picked; all parked is None.
        assert_eq!(pick_arm(&[u32::MAX, 1], &[f64::NEG_INFINITY, 0.4], &uniform, 3.0, 1.0, 0.0), Some(1));
        assert_eq!(pick_arm(&[u32::MAX], &[f64::NEG_INFINITY], &[1.0], 3.0, 1.0, 0.0), None);
    }

    #[test]
    fn priors_steer_early_visits_then_wash_out() {
        // Two arms, equal means, one visit each; the prior favours arm 1.
        let totals = [0.5, 0.5];
        let priors = [0.2, 0.8];
        assert_eq!(pick_arm(&[1, 1], &totals, &priors, 2.0, 1.0, 2.0), Some(1));
        // After arm 1 accumulates visits at the same mean, the shrinking
        // `1/(1+n)` hands the pick back to the neglected arm.
        assert_eq!(
            pick_arm(&[1, 9], &[0.5, 4.5], &priors, 10.0, 1.0, 2.0),
            Some(0),
            "prior washes out with visits"
        );
    }

    #[test]
    fn leader_decided_needs_separated_confidence_bounds() {
        // Far apart with plenty of visits: decided.
        let visits = [16, 16];
        assert!(leader_decided(&visits, &[15.2, 3.2], 32.0, 0.3), "0.95 vs 0.2, tight bounds");
        // Same means, exploration wide enough to overlap: not decided.
        assert!(!leader_decided(&visits, &[15.2, 3.2], 32.0, 2.0), "wide bounds overlap");
        // Close means: never decided at sane radii.
        assert!(!leader_decided(&visits, &[8.0, 7.8], 32.0, 0.3));
        // One live arm has no rival to overlap with: trivially decided,
        // which is the right early-stop answer for a forced move.
        assert!(leader_decided(&[4], &[2.0], 4.0, 1.0), "single live arm is decided");
    }

    #[test]
    fn top_two_means_reports_best_first_and_needs_two_live_arms() {
        assert_eq!(top_two_means(&[2, 4, 1], &[1.0, 3.2, 0.9]), Some((0.9, 0.8)));
        assert_eq!(top_two_means(&[1], &[0.5]), None);
        assert_eq!(top_two_means(&[1, u32::MAX], &[0.5, f64::NEG_INFINITY]), None);
        assert_eq!(top_two_means(&[1, 0], &[0.5, 0.0]), None, "unvisited arms are not live");
    }
}
