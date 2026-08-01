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

use rand::seq::SliceRandom;
use rand::rng;

use crate::decision::{AutoDecider, Decider};
use crate::game::{GameAction, GameState, TurnStep};

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
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            iterations: 24,
            horizon_turns: 2,
            exploration: 1.0,
            weights: EvalWeights::default(),
            heuristic_rollouts: false,
        }
    }
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
        while !g.is_game_over() && g.turn_number < stop_turn && fuel > 0 && stale < 8 {
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

    /// Run the search over `candidates` and return the best.
    fn search(&self, state: &GameState, seat: usize, candidates: Vec<GameAction>) -> GameAction {
        let n = candidates.len();
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
        let mut done: u32 = n as u32;
        while done < self.cfg.iterations {
            let parent = done.max(1) as f64;
            let pick = (0..n)
                .filter(|&i| visits[i] != u32::MAX)
                .max_by(|&a, &b| {
                    let ucb = |i: usize| {
                        let v = visits[i].max(1) as f64;
                        total[i] / v + self.cfg.exploration * (parent.ln() / v).sqrt()
                    };
                    ucb(a).partial_cmp(&ucb(b)).unwrap_or(std::cmp::Ordering::Equal)
                });
            let Some(i) = pick else { break };
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
        // will always find a reason to.
        candidates.push(GameAction::PassPriority);
        Some(self.search(state, seat, candidates))
    }
}
