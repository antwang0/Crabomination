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

/// Wall-clock split of where search time goes, for the `PERF.md`
/// "MCTS leaf-evaluation throughput" candidate. Free unless
/// `CRAB_MCTS_TIMING` is set (the check is one cached bool); the ladder
/// prints the table after a run. Wall clock rather than instruction
/// counts because the routine image ships neither valgrind nor perf,
/// and the question is a coarse four-way split, not a per-line
/// attribution.
pub mod timing {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    use std::time::Instant;

    /// Cloning the root state, once per iteration.
    pub static CLONE_NS: AtomicU64 = AtomicU64::new(0);
    /// Applying the candidate action to the clone.
    pub static ROOT_NS: AtomicU64 = AtomicU64::new(0);
    /// Library shuffle + hidden-zone redeal at the top of each rollout.
    pub static DET_NS: AtomicU64 = AtomicU64::new(0);
    /// The rollout's play-forward loop (policy + engine actions).
    pub static SIM_NS: AtomicU64 = AtomicU64::new(0);
    /// Scoring the horizon state (net encode + forward, or the
    /// heuristic material eval).
    pub static LEAF_NS: AtomicU64 = AtomicU64::new(0);
    /// Inside the leaf: building the encoder's feature vectors
    /// ([`super::super::encode::encode_state`]).
    pub static ENC_NS: AtomicU64 = AtomicU64::new(0);
    /// Inside the leaf: the net's forward pass over the encoded state.
    pub static FWD_NS: AtomicU64 = AtomicU64::new(0);
    pub static ROLLOUTS: AtomicU64 = AtomicU64::new(0);
    pub static SIM_ACTIONS: AtomicU64 = AtomicU64::new(0);
    pub static DECISIONS: AtomicU64 = AtomicU64::new(0);

    pub fn enabled() -> bool {
        static ON: OnceLock<bool> = OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("CRAB_MCTS_TIMING").is_some())
    }

    /// Scope guard: accumulates elapsed nanos into `slot` on drop.
    /// Inactive (and costless past the flag check) when timing is off.
    pub struct Lap(Option<(&'static AtomicU64, Instant)>);

    pub fn lap(slot: &'static AtomicU64) -> Lap {
        Lap(enabled().then(|| (slot, Instant::now())))
    }

    impl Drop for Lap {
        fn drop(&mut self) {
            if let Some((slot, t0)) = self.0 {
                slot.fetch_add(t0.elapsed().as_nanos() as u64, Relaxed);
            }
        }
    }

    pub fn count(slot: &'static AtomicU64, n: u64) {
        if enabled() {
            slot.fetch_add(n, Relaxed);
        }
    }

    /// The table, or `None` when the flag is off or nothing ran.
    pub fn report() -> Option<String> {
        if !enabled() {
            return None;
        }
        let rollouts = ROLLOUTS.load(Relaxed);
        if rollouts == 0 {
            return None;
        }
        let rows = [
            ("clone", CLONE_NS.load(Relaxed)),
            ("root action", ROOT_NS.load(Relaxed)),
            ("determinize", DET_NS.load(Relaxed)),
            ("rollout sim", SIM_NS.load(Relaxed)),
            ("leaf eval", LEAF_NS.load(Relaxed)),
        ];
        let total: u64 = rows.iter().map(|&(_, ns)| ns).sum();
        // The two leaf sub-segments overlap "leaf eval" and are excluded
        // from the percentage base.
        let subs = [("  \u{21b3} encode", ENC_NS.load(Relaxed)), ("  \u{21b3} forward", FWD_NS.load(Relaxed))];
        let mut out = format!(
            "mcts timing: {} decisions, {} rollouts, {} sim actions ({:.1} per rollout)\n",
            DECISIONS.load(Relaxed),
            rollouts,
            SIM_ACTIONS.load(Relaxed),
            SIM_ACTIONS.load(Relaxed) as f64 / rollouts as f64,
        );
        for (name, ns) in rows.iter().copied().chain(subs) {
            out.push_str(&format!(
                "  {name:<12} {:>9.2} ms  {:>5.1} %  ({:.0} µs/rollout)\n",
                ns as f64 / 1e6,
                100.0 * ns as f64 / total.max(1) as f64,
                ns as f64 / 1e3 / rollouts as f64,
            ));
        }
        out.push_str(&format!("  {:<12} {:>9.2} ms", "total", total as f64 / 1e6));
        Some(out)
    }
}

use crate::decision::{AutoDecider, Decider};
use crate::game::{GameAction, GameState, TurnStep};
use crate::recommend::STALE_ROUNDS;

use super::bot::{Bot, EvalWeights, HeuristicBot};

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
    /// Search combat declarations too (round 31). Off, the bot searches
    /// only main-phase plays and combat falls through to the heuristic —
    /// the shape every MCTS result through round 29 was measured on. On,
    /// attack and block declarations are searched over the same candidate
    /// menus the sim searches score (`attack_candidates_for_mcts` /
    /// `block_candidates_for_mcts`), with rollouts in place of the
    /// one-turn simulations — which sees through *sampled* opposing
    /// combat instead of the greedy declarations the sims assume.
    pub search_combat: bool,
    /// Gumbel root search (round 37, Danihelka et al. ICLR 2022):
    /// replace UCB1 allocation with Sequential Halving over
    /// Gumbel-perturbed prior logits, arms scored by
    /// `g + logit + σ(q̂)`. Priors come from the net's policy head over
    /// each candidate's successor state when the profile's net carries
    /// one, else from the candidate generator's scores — the same source
    /// round 29's P-UCT negative used, but consumed by an allocator with
    /// a policy-improvement guarantee at small budgets instead of by a
    /// visit bonus that starves arms.
    ///
    /// Why this and not another UCB knob: round 29 found every selection
    /// lever null-to-negative and only iterations paying. Sequential
    /// Halving is not a tuning of UCB1 — it is the allocator built for
    /// exactly this regime (few arms, tiny budget, pick the argmax at
    /// the end), and its improved policy `softmax(logits + σ(q̂))` is the
    /// distillation target with the guarantee, where round 35 softmaxed
    /// raw arm means at a hand-picked temperature.
    ///
    /// Ignores `exploration`, `prior_weight`, `early_stop` and
    /// `extend_close` — the phase plan is the budget policy.
    pub gumbel: bool,
    /// σ(q̂) = (c_visit + max_arm_visits) · c_scale · q̂ — the transform
    /// that puts observed rewards on the logit scale. Defaults are the
    /// reference implementation's (mctx: `maxvisit_init` 50, value scale
    /// 0.1); rewards here are already calibrated win probabilities, the
    /// scale mctx assumes.
    pub gumbel_c_visit: f64,
    pub gumbel_c_scale: f64,
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
            search_combat: false,
            gumbel: false,
            gumbel_c_visit: 50.0,
            gumbel_c_scale: 0.1,
        }
    }
}

/// Sequential Halving's phase plan: `(surviving arms, rollouts per arm)`
/// per phase, spending at most `budget` rollouts in total.
///
/// Arm counts halve each phase down to a final pair; the per-phase spend
/// is the remaining budget split evenly over the remaining phases, so
/// later (narrower) phases visit each survivor more — with `arms`
/// dividing evenly the whole budget is spent exactly. Every phase visits
/// each survivor at least once when the budget allows; a budget too thin
/// even for that truncates rather than overspends, because a
/// budget-matched gate is only a gate if both sides spend what they say.
fn sequential_halving_plan(arms: usize, budget: u32) -> Vec<(usize, u32)> {
    if arms < 2 || budget == 0 {
        return Vec::new();
    }
    let phases = usize::BITS - (arms - 1).leading_zeros(); // ceil(log2(arms))
    let mut plan = Vec::with_capacity(phases as usize);
    let mut remaining = budget;
    let mut m = arms;
    for p in 0..phases {
        if remaining == 0 {
            break;
        }
        let phases_left = phases - p;
        let mut visits = (remaining / (phases_left * m as u32)).max(1);
        if visits * m as u32 > remaining {
            visits = remaining / m as u32;
            if visits == 0 {
                break;
            }
        }
        plan.push((m, visits));
        remaining -= visits * m as u32;
        m = m.div_ceil(2);
        if m < 2 {
            m = 2;
        }
    }
    // Whatever integer division left over goes to the final pair — the
    // decision the whole search exists to sharpen.
    if remaining >= 2
        && let Some((m, v)) = plan.last_mut()
    {
        *v += remaining / *m as u32;
    }
    plan
}

/// The Gumbel search's value transform: σ(q̂) = (c_visit + max_visits) ·
/// c_scale · q̂ puts an observed mean reward on the same scale as the
/// prior logits, growing with visit depth so better-estimated rewards
/// override the prior more.
///
/// `q` must be the *normalized* reward — see [`completed_sigma`]. The
/// first gate of round 37 fed raw win probabilities in, and their
/// across-arm spread (~0.05) made σ gaps of ~0.3 logits against Gumbel
/// noise of stddev ~1.28: the final argmax was a noise lottery, and all
/// six cells lost by ~15–20 points.
fn sigma_q(q: f64, max_visits: u32, c_visit: f64, c_scale: f64) -> f64 {
    (c_visit + max_visits as f64) * c_scale * q
}

/// Per-arm σ terms for one decision: observed mean rewards min-max
/// normalized to [0, 1] *across the decision's visited arms* (the
/// reference implementation's `rescale_values`), then scaled by
/// [`sigma_q`]. Unvisited arms get 0 — they compete on their perturbed
/// prior alone, and every live arm is visited from phase 1 whenever the
/// budget allows.
///
/// The normalization is the part that makes the transform work at all:
/// two candidate lines in one position differ by a few points of win
/// probability, and σ has to spread *that* gap over the (c_visit +
/// max_visits)·c_scale ≈ 5–7 logit range the procedure assumes, not the
/// gap's raw size.
fn completed_sigma(
    visits: &[u32],
    totals: &[f64],
    c_visit: f64,
    c_scale: f64,
) -> Vec<f64> {
    let n = visits.len();
    let mut out = vec![0.0f64; n];
    let max_v = visits.iter().copied().filter(|&v| v != u32::MAX).max().unwrap_or(0);
    if max_v == 0 {
        return out;
    }
    let q = |i: usize| totals[i] / visits[i] as f64;
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for (i, &v) in visits.iter().enumerate() {
        if v != u32::MAX && v > 0 {
            lo = lo.min(q(i));
            hi = hi.max(q(i));
        }
    }
    let spread = (hi - lo).max(1e-8);
    for i in 0..n {
        if visits[i] != u32::MAX && visits[i] > 0 {
            out[i] = sigma_q((q(i) - lo) / spread, max_v, c_visit, c_scale);
        }
    }
    out
}

/// Standard-Gumbel noise: `-ln(-ln u)`, u uniform in (0, 1). Added once
/// per arm per decision — sampling without replacement over the prior in
/// disguise, which is what lets a fixed deterministic procedure explore.
fn gumbel_noise<R: RngExt>(r: &mut R) -> f64 {
    let u: f64 = r.random_range(f64::EPSILON..1.0);
    -(-u.ln()).ln()
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
    fallback: HeuristicBot,
}

impl MctsBot {
    pub fn new(cfg: MctsConfig) -> Self {
        Self { cfg, fallback: HeuristicBot::with_weights(cfg.weights) }
    }

    /// Score a finished rollout on a 0..=1 scale.
    ///
    /// A decided game pins to the extremes; anything else is squashed from
    /// the material evaluation. UCB1 assumes bounded rewards, and material
    /// scores are unbounded and scale-dependent, so feeding them in raw
    /// would make the exploration constant meaningless.
    fn reward(&self, g: &GameState, seat: usize) -> f64 {
        let _t = timing::lap(&timing::LEAF_NS);
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
        timing::count(&timing::ROLLOUTS, 1);
        let mut r = rng();
        {
            let _t = timing::lap(&timing::DET_NS);
            // Determinise: the real top of each library is information a
            // search has no business exploiting.
            for p in &mut g.players {
                let mut lib = std::mem::take(&mut p.library);
                lib.shuffle(&mut r);
                p.library = lib;
            }
            // Under a determinizing profile the opponent's *hand* is
            // redealt too (the library shuffle above never covered it —
            // rollouts have been reading the held cards since the first
            // MCTS experiment). Salted per rollout so the bandit averages
            // over imagined hands.
            if self.cfg.weights.determinize > 0 {
                let salt = 0x3C75_0000 ^ r.random::<u32>() as u64;
                if let Some(b) = super::bot::hand_belief(&g, seat, &self.cfg.weights) {
                    super::bot::determinize_hidden_belief(&mut g, seat, salt, &b);
                } else {
                    super::bot::determinize_hidden(&mut g, seat, salt);
                }
            }
        }
        let _t = timing::lap(&timing::SIM_NS);
        let mut actions = 0u64;
        let stop_turn = g.turn_number + self.cfg.horizon_turns;
        let mut policy: Vec<HeuristicBot> = (0..g.players.len())
            .map(|_| {
                if self.cfg.heuristic_rollouts {
                    HeuristicBot::with_weights(self.cfg.weights)
                } else {
                    HeuristicBot::uniform_baseline()
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
                actions += 1;
                if g.perform_action(GameAction::SubmitDecision(answer)).is_err() {
                    break;
                }
                continue;
            }
            let mut acted = false;
            for (s, p) in policy.iter_mut().enumerate() {
                let Some(a) = p.next_action(&g, s) else { continue };
                actions += 1;
                if g.perform_action(a).is_ok() {
                    acted = true;
                    if g.is_game_over() {
                        break;
                    }
                }
            }
            if acted { stale = 0 } else { stale += 1 }
        }
        timing::count(&timing::SIM_ACTIONS, actions);
        drop(_t);
        self.reward(&g, seat)
    }

    /// The Gumbel root search (round 37): Sequential Halving over arms
    /// scored by `g + logit + σ(q̂)`.
    ///
    /// Structure, phase by phase: validate every arm (a rejected action
    /// is parked, as in the UCB1 path), compute one prior logit per
    /// surviving arm, perturb each once with Gumbel noise, then run the
    /// halving plan — every phase rolls out each survivor equally often
    /// and keeps the better-scoring half. The final pick is the argmax of
    /// the same score over the last survivors, which is what makes the
    /// procedure a policy improvement even at tiny budgets: an arm only
    /// wins by out-scoring the prior's favourite *after* its rewards have
    /// been watched.
    ///
    /// Priors: the net's policy head over each candidate's successor
    /// state when the profile's net carries one (`net_eval::policy_logit`
    /// — the state the head was trained on), else the log-softmax of the
    /// candidate generator's scores at `prior_temp`. The fallback keeps
    /// the profile runnable with a headless net as its own control arm.
    fn gumbel_search(
        &self,
        state: &GameState,
        seat: usize,
        candidates: Vec<(GameAction, i32)>,
    ) -> GameAction {
        let n = candidates.len();
        timing::count(&timing::DECISIONS, 1);
        let unit = self.cfg.weights.unit.max(1) as f64;
        let scores: Vec<f64> = candidates.iter().map(|(_, s)| *s as f64 / unit).collect();
        let candidates: Vec<GameAction> = candidates.into_iter().map(|(a, _)| a).collect();

        // Validate once, keeping each successor for the prior pass.
        let succ: Vec<Option<GameState>> = candidates
            .iter()
            .map(|a| {
                let mut g = state.clone();
                g.perform_action(a.clone()).ok().map(|_| g)
            })
            .collect();
        let mut live: Vec<usize> = (0..n).filter(|&i| succ[i].is_some()).collect();
        if live.is_empty() {
            return GameAction::PassPriority;
        }
        if live.len() == 1 {
            // Forced: nothing to search and no policy signal to record.
            return candidates.into_iter().nth(live[0]).unwrap_or(GameAction::PassPriority);
        }

        let slot = self.cfg.weights.net_slot;
        let use_head = slot != 0 && super::net_eval::slot_has_policy(slot);
        let mut logits = vec![0f64; n];
        if use_head {
            for &i in &live {
                if let Some(g) = &succ[i]
                    && let Some(l) = super::net_eval::policy_logit(g, seat, slot)
                {
                    logits[i] = l as f64;
                }
            }
        } else {
            let p = softmax_priors(&scores, self.cfg.prior_temp);
            for &i in &live {
                logits[i] = p[i].max(1e-12).ln();
            }
        }

        let mut r = rng();
        let noise: Vec<f64> = (0..n).map(|_| gumbel_noise(&mut r)).collect();
        let mut visits = vec![0u32; n];
        let mut total = vec![0.0f64; n];
        // Arms are compared on `g + logit + σ(normalized q̂)` — σ terms
        // recomputed per phase from the visits so far ([`completed_sigma`]:
        // rewards min-max normalized across the decision's visited arms,
        // an unvisited arm competing on its perturbed prior alone).
        // Survivors of a phase share a visit count, so the comparison is
        // always on the same footing.
        for (m, per_arm) in sequential_halving_plan(live.len(), self.cfg.iterations) {
            let sig =
                completed_sigma(&visits, &total, self.cfg.gumbel_c_visit, self.cfg.gumbel_c_scale);
            live.sort_by(|&a, &b| {
                (noise[b] + logits[b] + sig[b])
                    .partial_cmp(&(noise[a] + logits[a] + sig[a]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            live.truncate(m);
            for &i in &live {
                for _ in 0..per_arm {
                    let mut g = {
                        let _t = timing::lap(&timing::CLONE_NS);
                        state.clone()
                    };
                    let rooted = {
                        let _t = timing::lap(&timing::ROOT_NS);
                        g.perform_action(candidates[i].clone())
                    };
                    if rooted.is_err() {
                        break;
                    }
                    total[i] += self.rollout(g, seat);
                    visits[i] += 1;
                }
            }
        }
        let sig =
            completed_sigma(&visits, &total, self.cfg.gumbel_c_visit, self.cfg.gumbel_c_scale);
        let best = live
            .iter()
            .copied()
            .max_by(|&a, &b| {
                (noise[a] + logits[a] + sig[a])
                    .partial_cmp(&(noise[b] + logits[b] + sig[b]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(live[0]);

        // The distillation target is the *improved policy* — `logit +
        // σ(normalized q̂)` per arm, no Gumbel noise (the noise is
        // exploration, not belief). Softmaxing these downstream at
        // temperature 1 is Danihelka et al.'s completed-Q construction,
        // with an unvisited arm completed by its prior alone. Parked
        // arms carry -inf and are dropped with their candidates, as in
        // the UCB1 capture.
        if super::decision_capture::enabled() {
            let improved: Vec<f32> = (0..n)
                .map(|i| {
                    if succ[i].is_none() {
                        f32::NEG_INFINITY
                    } else {
                        (logits[i] + sig[i]) as f32
                    }
                })
                .collect();
            super::decision_capture::maybe_valued(
                state,
                seat,
                &candidates,
                best,
                Some(&improved),
                true,
                Some(&visits),
            );
        }
        candidates.into_iter().nth(best).unwrap_or(GameAction::PassPriority)
    }

    /// Run the search over scored `candidates` and return the best.
    fn search(&self, state: &GameState, seat: usize, candidates: Vec<(GameAction, i32)>) -> GameAction {
        if self.cfg.gumbel {
            return self.gumbel_search(state, seat, candidates);
        }
        let n = candidates.len();
        let unit = self.cfg.weights.unit.max(1) as f64;
        let priors = softmax_priors(
            &candidates.iter().map(|(_, s)| *s as f64 / unit).collect::<Vec<_>>(),
            self.cfg.prior_temp,
        );
        let candidates: Vec<GameAction> = candidates.into_iter().map(|(a, _)| a).collect();
        let mut visits = vec![0u32; n];
        let mut total = vec![0.0f64; n];
        timing::count(&timing::DECISIONS, 1);
        // Seed every arm once so UCB1 has a finite term for each.
        for i in 0..n {
            let mut g = {
                let _t = timing::lap(&timing::CLONE_NS);
                state.clone()
            };
            let rooted = {
                let _t = timing::lap(&timing::ROOT_NS);
                g.perform_action(candidates[i].clone())
            };
            if rooted.is_err() {
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
            let mut g = {
                let _t = timing::lap(&timing::CLONE_NS);
                state.clone()
            };
            let rooted = {
                let _t = timing::lap(&timing::ROOT_NS);
                g.perform_action(candidates[i].clone())
            };
            if rooted.is_err() {
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
        // Record the root decision with the search's per-arm means, so a
        // policy target can be distilled from what the *search* concluded
        // rather than from which action it happened to output. Without
        // this hook the recorder only ever sees the heuristic fallback's
        // picks, and an MCTS-piloted run would quietly measure the wrong
        // policy. Arms the engine rejected are parked at NEG_INFINITY and
        // are dropped alongside their candidates downstream.
        if super::decision_capture::enabled() {
            let means: Vec<f32> = (0..n)
                .map(|i| {
                    if visits[i] == u32::MAX {
                        f32::NEG_INFINITY
                    } else {
                        (total[i] / visits[i].max(1) as f64) as f32
                    }
                })
                .collect();
            // Parked arms report zero rollouts; they are dropped with
            // their candidates in the capture remap regardless.
            let counts: Vec<u32> =
                visits.iter().map(|&v| if v == u32::MAX { 0 } else { v }).collect();
            super::decision_capture::maybe_valued(
                state,
                seat,
                &candidates,
                best,
                Some(&means),
                false,
                Some(&counts),
            );
        }
        candidates.into_iter().nth(best).unwrap_or(GameAction::PassPriority)
    }

    /// The round-51 fetch arms: search a pending `SearchLibrary` instead of
    /// letting it fall through to the heuristic. Every other pending
    /// decision still does fall through — this is the one whose answer is a
    /// whole card and whose heuristic is a fixed ranking with no board read
    /// past mana. `None` means "not a searched fetch tick".
    fn fetch_search(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        use crate::decision::{Decision, DecisionAnswer};
        let pending = state.pending_decision.as_ref()?;
        if pending.acting_player() != seat || state.is_game_over() {
            return None;
        }
        let Decision::SearchLibrary { candidates, eligible, .. } = &pending.decision else {
            return None;
        };
        let pickable: Vec<(crate::card::CardId, String)> = match eligible {
            Some(ok) => candidates.iter().filter(|(id, _)| ok.contains(id)).cloned().collect(),
            None => candidates.clone(),
        };
        let ranked = super::bot::rank_library_search(state, seat, &pickable, &self.cfg.weights);
        // One legal hit is not a decision.
        if ranked.len() < 2 {
            return None;
        }
        // Four arms, scored a hair apart in the heuristic's order: a tie
        // leaves its pick in front and the sims have to earn the swap, the
        // same convention `target_arms` uses.
        let arms: Vec<(GameAction, i32)> = ranked
            .into_iter()
            .take(4)
            .enumerate()
            .map(|(i, id)| {
                (GameAction::SubmitDecision(DecisionAnswer::Search(Some(id))), -(i as i32))
            })
            .collect();
        Some(self.search(state, seat, arms))
    }

    /// The round-31 combat arms: search the attack/block declaration when
    /// it is this seat's to make and still pending. `None` means "not a
    /// searched combat tick" and the caller falls through to the normal
    /// path. Master Warcraft-style declarations for the *other* side
    /// (forced-only) stay with the heuristic, as does every non-declaration
    /// tick of the phase — tricks, defensive removal follow-ups, passes —
    /// via the shared latch on the fallback bot.
    fn combat_search(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        if state.pending_decision.is_some()
            || state.player_with_priority() != seat
            || state.is_game_over()
        {
            return None;
        }
        let is_active = state.active_player_idx == seat;
        let w = self.cfg.weights;

        if state.step == TurnStep::DeclareAttackers
            && is_active
            && state.attack_declarer() == seat
            && self.fallback.declaration_pending(state, true)
        {
            let cands = super::bot::attack_candidates_for_mcts(state, seat, &w);
            self.fallback.note_external_declaration(state, true);
            if cands.len() < 2 {
                let only = cands.into_iter().next().unwrap_or_default();
                return Some(GameAction::DeclareAttackers(only));
            }
            // No cheap prior scores exist for declarations (the heuristic's
            // opinion costs a simulation); every arm starts equal.
            let arms = cands
                .into_iter()
                .map(|a| (GameAction::DeclareAttackers(a), 0))
                .collect();
            return Some(self.search(state, seat, arms));
        }

        if state.step == TurnStep::DeclareBlockers
            && !is_active
            && state.may_declare_blocks(seat)
            && !state.attacking().is_empty()
            && self.fallback.declaration_pending(state, false)
        {
            // Removal before blocks, exactly like the heuristic arm: the
            // declaration should answer the combat that remains after the
            // biggest attacker dies. The latch stays unset so this tick
            // repeats until the removal runs dry.
            if let Some(a) = super::bot::defensive_removal_for_mcts(state, seat, &w) {
                return Some(a);
            }
            let cands = super::bot::block_candidates_for_mcts(state, seat, &w);
            self.fallback.note_external_declaration(state, false);
            if cands.len() < 2 {
                let only = cands.into_iter().next().unwrap_or_default();
                return Some(GameAction::DeclareBlockers(only));
            }
            let arms = cands
                .into_iter()
                .map(|b| (GameAction::DeclareBlockers(b), 0))
                .collect();
            return Some(self.search(state, seat, arms));
        }
        None
    }
}

impl Bot for MctsBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        if self.cfg.search_combat
            && let Some(a) = self.combat_search(state, seat)
        {
            return Some(a);
        }
        if self.cfg.weights.fetch_arms
            && let Some(a) = self.fetch_search(state, seat)
        {
            return Some(a);
        }
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

    fn creature(name: &'static str, p: i32, t: i32) -> crate::card::CardDefinition {
        crate::card::CardDefinition {
            name,
            card_types: vec![crate::card::CardType::Creature],
            power: p,
            toughness: t,
            ..Default::default()
        }
    }

    /// The round-31 block arm: a searched declaration comes back from the
    /// candidate menu, and the latch shared with the fallback stops the
    /// next tick of the same step from declaring again.
    #[test]
    fn combat_search_declares_blocks_once() {
        use crate::game::types::{Attack, AttackTarget};
        use crate::player::Player;

        let players = vec![Player::new(0, "A"), Player::new(1, "B")];
        let mut g = GameState::new(players);
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(1, creature("Beater", 3, 3));
        g.add_card_to_battlefield(0, creature("Bear A", 2, 2));
        g.add_card_to_battlefield(0, creature("Bear B", 2, 2));
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(0) }];

        let mut bot = MctsBot::new(MctsConfig {
            iterations: 8,
            search_combat: true,
            ..MctsConfig::default()
        });
        let a = bot.next_action(&g, 0);
        assert!(
            matches!(a, Some(GameAction::DeclareBlockers(_))),
            "searched block declaration, got {a:?}"
        );
        let b = bot.next_action(&g, 0);
        assert!(
            !matches!(b, Some(GameAction::DeclareBlockers(_))),
            "second tick must not re-declare, got {b:?}"
        );
    }

    /// The attack arm, and the off switch: with `search_combat` false the
    /// declaration is the fallback heuristic's exactly as before round 31.
    #[test]
    fn combat_search_declares_attacks_and_defaults_off() {
        use crate::player::Player;

        let players = vec![Player::new(0, "A"), Player::new(1, "B")];
        let mut g = GameState::new(players);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, creature("Bear", 2, 2));
        for c in g.battlefield.iter_mut() {
            c.summoning_sick = false;
        }

        let mut on = MctsBot::new(MctsConfig {
            iterations: 8,
            search_combat: true,
            ..MctsConfig::default()
        });
        let a = on.next_action(&g, 0);
        assert!(
            matches!(a, Some(GameAction::DeclareAttackers(_))),
            "searched attack declaration, got {a:?}"
        );
        let again = on.next_action(&g, 0);
        assert!(
            !matches!(again, Some(GameAction::DeclareAttackers(_))),
            "second tick must not re-declare, got {again:?}"
        );

        // Default config: the combat arm never fires; the fallback's own
        // declaration path (which sets its own latch) still runs.
        let mut off = MctsBot::new(MctsConfig { iterations: 8, ..MctsConfig::default() });
        let d = off.next_action(&g, 0);
        assert!(
            matches!(d, Some(GameAction::DeclareAttackers(_))),
            "fallback declares when search_combat is off, got {d:?}"
        );
    }

    /// The halving plan is the budget policy, so its arithmetic is the
    /// contract: never overspend, halve to a final pair, visit every
    /// survivor of a phase equally and at least once.
    #[test]
    fn sequential_halving_plan_spends_the_budget_and_halves() {
        // The profile shape: 8 arms, 64 rollouts — exact spend, later
        // phases deeper.
        assert_eq!(sequential_halving_plan(8, 64), vec![(8, 2), (4, 6), (2, 12)]);
        assert_eq!(sequential_halving_plan(3, 64), vec![(3, 10), (2, 17)]);
        assert_eq!(sequential_halving_plan(2, 64), vec![(2, 32)]);
        // A budget too thin to halve visits what it can and stops.
        assert_eq!(sequential_halving_plan(8, 8), vec![(8, 1)]);
        // Degenerate inputs are empty plans, not panics.
        assert!(sequential_halving_plan(1, 64).is_empty());
        assert!(sequential_halving_plan(8, 0).is_empty());
        // Properties across a sweep: spend ≤ budget, arm counts start at
        // n and never increase, every phase visits each survivor ≥ once.
        for arms in 2..=9usize {
            for budget in [arms as u32, 24, 64, 100, 256] {
                let plan = sequential_halving_plan(arms, budget);
                let spend: u32 = plan.iter().map(|&(m, v)| m as u32 * v).sum();
                assert!(spend <= budget, "{arms} arms, {budget}: spent {spend}");
                assert_eq!(plan[0].0, arms, "first phase covers every live arm");
                assert!(plan.windows(2).all(|w| w[1].0 < w[0].0), "arms must shrink");
                assert!(plan.iter().all(|&(_, v)| v >= 1));
                // With a real budget, the plan reaches the final pair and
                // leaves at most one rollout unspent.
                if budget >= 8 * arms as u32 {
                    assert_eq!(plan.last().unwrap().0, 2, "{arms} arms, {budget}");
                    assert!(budget - spend <= 1, "{arms} arms, {budget}: left {}", budget - spend);
                }
            }
        }
    }

    /// σ(q̂) is what lets observed rewards override the prior: monotone
    /// in the reward, and growing with visit depth so better-estimated
    /// rewards speak louder.
    #[test]
    fn sigma_q_scales_with_reward_and_visits() {
        assert!(sigma_q(0.8, 8, 50.0, 0.1) > sigma_q(0.4, 8, 50.0, 0.1));
        assert!(sigma_q(0.8, 64, 50.0, 0.1) > sigma_q(0.8, 8, 50.0, 0.1));
        assert_eq!(sigma_q(0.0, 8, 50.0, 0.1), 0.0);
        // At the defaults and one visit, a full win-probability unit is
        // worth ~5 logits — decisive but not prior-erasing.
        let one = sigma_q(1.0, 1, 50.0, 0.1);
        assert!((one - 5.1).abs() < 1e-9, "got {one}");
    }

    /// The round-37 gate-1 lesson, pinned: rewards are min-max
    /// normalized across the decision's arms before σ, so the best and
    /// worst *observed* arms are separated by the full (c_visit +
    /// max_visits)·c_scale range however small their raw win-probability
    /// gap is. Unnormalized, a 0.05 gap was ~0.3 logits against Gumbel
    /// noise of stddev ~1.28 and the final argmax was a noise lottery —
    /// all six gate cells lost by 15–20 points.
    #[test]
    fn completed_sigma_normalizes_rewards_across_arms() {
        // Three visited arms with a tiny raw spread, one unvisited.
        let visits = [4, 4, 4, 0];
        let totals = [4.0 * 0.50, 4.0 * 0.52, 4.0 * 0.55, 0.0];
        let sig = completed_sigma(&visits, &totals, 50.0, 0.1);
        let full = (50.0 + 4.0) * 0.1;
        assert!((sig[2] - full).abs() < 1e-6, "best arm spans the full range: {sig:?}");
        assert!(sig[0].abs() < 1e-6, "worst arm is the floor: {sig:?}");
        assert!(sig[1] > sig[0] && sig[1] < sig[2], "middle arm ordered: {sig:?}");
        assert_eq!(sig[3], 0.0, "unvisited arm competes on its prior alone");
        // All-equal rewards: no arm is favoured, and nothing divides by
        // zero.
        let flat = completed_sigma(&[2, 2], &[1.0, 1.0], 50.0, 0.1);
        assert!(flat.iter().all(|s| s.abs() < 1e-6), "{flat:?}");
        // Nothing visited: all zero.
        assert!(completed_sigma(&[0, 0], &[0.0, 0.0], 50.0, 0.1).iter().all(|s| *s == 0.0));
    }

    /// Gumbel noise is a standard Gumbel: finite, and centred near the
    /// Euler–Mascheroni constant (~0.577) rather than zero.
    #[test]
    fn gumbel_noise_is_finite_with_the_right_mean() {
        use rand::SeedableRng;
        let mut r = rand::rngs::StdRng::seed_from_u64(7);
        let n = 20_000;
        let mut sum = 0.0;
        for _ in 0..n {
            let g = gumbel_noise(&mut r);
            assert!(g.is_finite());
            sum += g;
        }
        let mean = sum / n as f64;
        assert!((mean - 0.5772).abs() < 0.05, "mean {mean}");
    }

    /// The Gumbel arm returns a legal searched action from the same menus
    /// as the UCB1 path, and the config default leaves it off.
    #[test]
    fn gumbel_search_plays_a_legal_main_phase_action() {
        use crate::game::two_player_game;

        let mut g = two_player_game();
        for (n, p, t) in [("A", 1, 1), ("B", 2, 2), ("C", 3, 3)] {
            g.add_card_to_hand(0, creature(n, p, t));
        }
        assert!(!MctsConfig::default().gumbel, "gumbel must be opt-in");
        let mut bot = MctsBot::new(MctsConfig {
            iterations: 8,
            horizon_turns: 1,
            gumbel: true,
            ..MctsConfig::default()
        });
        let mut fuel = 30;
        let mut acted = false;
        while fuel > 0 && !g.is_game_over() {
            fuel -= 1;
            let Some(a) = bot.next_action(&g, 0) else { break };
            if g.perform_action(a).is_ok() {
                acted = true;
            }
        }
        assert!(acted, "gumbel bot never played a legal action");
    }

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
