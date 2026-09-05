//! In-process bots for server-hosted matches.
//!
//! Unlike networked clients, a bot reads the full authoritative [`GameState`]
//! each tick and returns the next [`GameAction`] it wants the server to
//! perform. The match actor polls every bot seat to a fixed point after every
//! state change, so a bot just needs to make *some* forward-progressing
//! decision (including `PassPriority`) whenever it holds priority.

use crate::game::KeywordSlice;

use rand::{RngExt, SeedableRng, rng};
use rand::rngs::StdRng;

use crate::card::{CardDefinition, CardId};
use crate::decision::{AutoDecider, Decider};
use crate::effect::{ActivatedAbility, Effect, ManaPayload};
use crate::game::actions::AbilityRef;
use crate::game::{Attack, AttackTarget, GameAction, GameState, Target, TurnStep};
use crate::mana::{ManaCost, ManaPool};

thread_local! {
    /// Per-thread source for the scored bot's tie-break jitter, when a
    /// caller has asked for a reproducible one. `None` = draw from the
    /// thread RNG, which is the behaviour in a real server match.
    static JITTER: std::cell::RefCell<Option<StdRng>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Seed (or clear) this thread's tie-break jitter.
///
/// [`main_phase_action_with`] breaks exact score ties with a small random
/// nudge, so two runs of the "same" game diverge even under a fixed
/// shuffle seed. That is fine in a real match and actively unhelpful in
/// measurement: it means `--seed` never made a ladder run reproducible,
/// and — more expensively — it is the *only* thing that can decide a
/// paired game under a true null, where both seats pilot the same
/// profile. Measured on 2400 sealed pairs, that residual accounted for
/// every one of the 368 non-split pairs and held the within-pair
/// correlation at −0.69 instead of −1.
///
/// Seeding it identically for both games of an antithetic pair makes the
/// two replays differ only where the *profiles* differ, which is the
/// whole point of common random numbers. Real matches leave it `None`.
pub fn set_jitter_seed(seed: Option<u64>) {
    JITTER.with(|j| {
        *j.borrow_mut() = seed.map(StdRng::seed_from_u64);
    });
}

/// `CRAB_NO_JITTER=1` pins every tie-break draw to 0, read once.
///
/// A measurement switch, not a play mode. The scored pickers draw one
/// `jitter_below` per *candidate*, so any refactor that changes how many
/// candidates reach a picker re-aligns the stream for the rest of the game and
/// the run diverges even where the policy is identical — which is invisible to
/// the golden traces and to `--bench` (the `fixed` pool reaches none of
/// `cast_candidates`' specialty blocks). With the draws pinned, two builds on
/// one seed play the same game or they do not, and `cg_edges.py --callers
/// next_action_settled` is the count that says which. See PERF's "How to
/// measure".
fn no_jitter() -> bool {
    static NO_JITTER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *NO_JITTER.get_or_init(|| std::env::var_os("CRAB_NO_JITTER").is_some())
}

/// A jitter draw in `0..n`, from the seeded stream when one is installed.
fn jitter_below(n: usize) -> usize {
    if n <= 1 || no_jitter() {
        return 0;
    }
    JITTER.with(|j| match &mut *j.borrow_mut() {
        Some(r) => r.random_range(0..n),
        None => rng().random_range(0..n),
    })
}

/// A uniform draw in [0, 1) from the jitter stream.
fn jitter_f64() -> f64 {
    JITTER.with(|j| match &mut *j.borrow_mut() {
        Some(r) => r.random::<f64>(),
        None => rng().random::<f64>(),
    })
}

thread_local! {
    /// Per-thread softmax sampling over the LIVE scored pickers' candidates
    /// (attacks, blocks, main-phase finalists): `(temperature in eval
    /// units, last turn to sample on)`. Installed by self-play actors to
    /// diversify training data — the AlphaZero opening-temperature idea
    /// applied to this bot's decomposed searches. `None` — the default,
    /// and what gates, ladders, and real matches run — keeps every picker
    /// argmax. Thread-local rather than an `EvalWeights` field so the
    /// simulations' inner policies (same weights, same thread) can never
    /// see it: the only call sites are the three live pickers.
    static SAMPLING: std::cell::Cell<Option<(i32, u32)>> =
        const { std::cell::Cell::new(None) };
}

/// Install (or clear) live-action sampling for this thread:
/// `Some((temp, turns))` softmax-samples candidate picks with temperature
/// `temp` (eval units) through turn `turns`, argmax after.
pub fn set_action_sampling(cfg: Option<(i32, u32)>) {
    SAMPLING.with(|s| s.set(cfg));
}

/// The sampling temperature in force for a decision on `turn`, if any.
fn sampling_temp(turn: u32) -> Option<f64> {
    SAMPLING
        .with(|s| s.get())
        .and_then(|(t, turns)| (t > 0 && turn <= turns).then_some(t as f64))
}

/// Softmax-sample an index from `scores` at temperature `temp`, drawing
/// from the jitter stream (seeded ⇒ reproducible). Max-subtracted so the
/// net profile's ±10 000-scale scores can't overflow the exp.
/// Total on an empty slice (index 0): the sampling branch of
/// `main_phase_action_with` reaches this with a candidate list that the
/// argmax branch beside it handles as "no action", and a panic on the
/// actor path kills a training run mid-flight.
fn sample_scored_index(scores: &[i32], temp: f64) -> usize {
    let Some(&max) = scores.iter().max() else { return 0 };
    let ws: Vec<f64> = scores.iter().map(|&s| (((s - max) as f64) / temp).exp()).collect();
    let total: f64 = ws.iter().sum();
    let mut u = jitter_f64() * total;
    for (i, w) in ws.iter().enumerate() {
        u -= w;
        if u <= 0.0 {
            return i;
        }
    }
    scores.len().saturating_sub(1)
}

/// Choose among `(candidate index, score)` pairs: softmax-sampled when
/// this thread has sampling installed and the turn qualifies, otherwise
/// the first-wins-ties argmax every ladder number was measured under.
fn choose_scored(turn: u32, scored: &[(usize, i32)]) -> Option<usize> {
    if scored.is_empty() {
        return None;
    }
    if scored.len() > 1
        && let Some(t) = sampling_temp(turn)
    {
        let ws: Vec<i32> = scored.iter().map(|&(_, s)| s).collect();
        return Some(scored[sample_scored_index(&ws, t)].0);
    }
    let mut best = scored[0];
    for &c in &scored[1..] {
        if c.1 > best.1 {
            best = c;
        }
    }
    Some(best.0)
}

/// Drives one seat without a human client. Implementations see the full
/// `GameState` and return the single next action they'd like to submit.
pub trait Bot: Send {
    /// Return `Some(action)` to submit, or `None` if it's not this bot's turn
    /// to act right now (no priority, waiting on an opponent decision, game
    /// already over, etc.).
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction>;

    /// Extended [`next_action`] that MAY hand the caller the state the action
    /// would produce, in addition to the action itself. A picker that
    /// dry-ran the action already *is* the action (see
    /// [`GameState::accept_on`](crate::game::GameState::accept_on)), and the
    /// pattern the driver runs — "call `bot.next_action(&g, s)`, then
    /// `g.perform_action(a)`" — costs one execution of the same action every
    /// time the picker probed it. A driver that owns its state adopts
    /// [`BotStep::settled`] instead of re-running.
    ///
    /// Default: no settled state; the caller must run the action itself.
    /// Bots that produce a settled state under `main_phase_action_with`'s
    /// finalist path (see [`HeuristicBot`]) override to hand it out.
    ///
    /// **Server / interactive callers** should NOT adopt: `perform_action`
    /// returns the event list the server broadcasts, and adopting `settled`
    /// hands back no events. Self-play drivers (which discard events) are
    /// the intended audience.
    fn next_action_settled(&mut self, state: &GameState, seat: usize) -> Option<BotStep> {
        self.next_action(state, seat).map(BotStep::plain)
    }
}

/// A bot's step, with an optional settled state — the pre-committed result of
/// dry-running the action. See [`Bot::next_action_settled`].
pub struct BotStep {
    pub action: GameAction,
    pub settled: Option<Box<GameState>>,
}

impl BotStep {
    /// No settled state — the caller must run the action itself.
    pub fn plain(action: GameAction) -> Self {
        Self { action, settled: None }
    }

    /// Discard any settled state.
    pub fn into_action(self) -> GameAction {
        self.action
    }
}

/// Tunable weights for the bot's board evaluation, so a change to how the
/// bot *values* things can be A/B-laddered against the previous numbers
/// instead of argued about. Every profile is internally consistent: all
/// non-permanent terms are expressed in multiples of [`unit`], so raising
/// `unit` buys arithmetic resolution without moving any relative weight.
///
/// [`unit`]: EvalWeights::unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalWeights {
    /// Value of one "point" on this profile's scale. Flat terms (hand
    /// cards, life, the legendary premium, ...) are written as `n * unit`.
    /// The baseline uses 1, which keeps the historical integer scores;
    /// richer profiles use a larger unit so sub-point terms -- a keyword
    /// worth two thirds of a power point -- survive integer division.
    pub unit: i32,
    /// Per-mana-value weight of a permanent.
    pub cmc: i32,
    /// Flat value of simply *being* a creature, before size. Forge's
    /// evaluator opens at a constant 100 and adds power/toughness on top;
    /// the historical weights here open at zero, which makes every other
    /// creature term -- keywords especially -- proportionally far louder
    /// than the reference they were calibrated against.
    pub creature_base: i32,
    /// Per-point weight of a creature's power and toughness.
    pub power: i32,
    pub toughness: i32,
    /// Keyword scoring strength as a percentage (see [`keyword_value`]).
    /// 0 disables it, which is the baseline: a 1/1 flying lifelinker reads
    /// as a vanilla 1/1.
    pub keyword_pct: i32,
    /// Use the concave life curve (see [`life_value`]) instead of counting
    /// life linearly. Life near zero is worth far more per point than life
    /// near the starting total; a linear term prices them the same.
    pub concave_life: bool,
    /// Hold a play whose only gain this turn is a summoning-sick body until
    /// the postcombat main (see [`eval_material_summon_sick_blind`]).
    pub hold_sick: bool,
    /// Hold an instant-speed line that achieves nothing this turn, so it is
    /// cast on the opponent's turn instead — with a turn more information,
    /// and with the mana up in the meantime. The instant-speed sibling of
    /// [`hold_sick`](Self::hold_sick), and the cheap form of Forge's
    /// "formulate a plan restricted to instant-speed lines and wait if it
    /// scores as well" (`SpellAbilityPicker::createNewPlan`).
    pub hold_instants: bool,
    /// How many *extra* plays a candidate's evaluation may look ahead. 0
    /// scores the board right after the candidate resolves (the historical
    /// behavior); 1 asks "and what would I do next?" once. See
    /// [`evaluate_action_sequence`].
    pub lookahead: u8,
    /// Score a candidate on the board *after* this turn's combat rather
    /// than the instant it resolves (see [`simulate_through_combat`]).
    pub combat_aware: bool,
    /// Search the attack declaration instead of taking the greedy one:
    /// simulate each candidate attack through the opponent's crack-back and
    /// keep the best (see [`pick_attacks_scored`]). 0 disables the search;
    /// higher values allow more candidates, and the cost is roughly linear
    /// in it because each candidate is a full simulated turn cycle.
    ///
    /// The measurement this exists to settle: `bot_probe` shows the bot
    /// declaring every eligible creature as an attacker in 73 % of its
    /// combats, and 41 % of its creatures tapped when blocks are declared
    /// as a direct result. Greedy attacking is *why* it can't block — but
    /// whether restraint is worth the tempo is a ladder question, not an
    /// argument, so this is a flag rather than a rewrite.
    pub attack_search: u8,
    /// Grow the attack declaration one creature at a time instead of only
    /// trimming the greedy one (see [`attack_chain_candidate`]). 0
    /// disables it; higher values cap how many attackers the chain may
    /// add. Each step simulates "the set so far plus one more eligible
    /// creature" for every remaining creature, with "finalize" — the set
    /// so far — as candidate 0 so a tie stops the chain; the finished set
    /// then joins [`attack_candidates_for_mcts`]'s menu for the one argmax
    /// [`pick_attacks_scored`] already takes, where greedy still wins ties.
    ///
    /// The holdback menu can only *drop* attackers, so a declaration
    /// smaller than greedy-minus-one, or one carrying a creature the
    /// greedy filters refused, is unreachable at any valuation — the
    /// menu-hole shape behind both blocking adoptions. Forward growth has
    /// the mirror blind spot (two attackers that only pay together are
    /// each bad alone, so the chain stops at nobody), which is why it
    /// extends the menu rather than replacing it. Cost is up to
    /// `attack_chain × eligible` simulated turn cycles per declaration on
    /// top of the menu's `2 + attack_search`. Needs `attack_search > 0`:
    /// a one-candidate menu returns before any sim runs.
    ///
    /// **Adopted 2026-09-04 (round 55)** at 6 on the default and the
    /// client pilot — see `impl Default` and
    /// [`client_pilot`](Self::client_pilot) for the numbers;
    /// `.ladder/run_r55_atkchain.sh` is the gate.
    pub attack_chain: u8,
    /// The wide chain (round 56). Two gaps in the round-55 shape: the
    /// chain never ran when greedy declared *nobody* (a one-candidate
    /// menu returned before any sim), which is exactly the board where
    /// every creature was refused by a per-creature rule and none was
    /// ever priced by the sim; and forward growth is blind to the
    /// overload — two attackers into one blocker connect where each alone
    /// is blocked and traded, so the first step ties and the chain stops.
    /// On, the chain also runs from an empty greedy, and its first step
    /// offers every *pair* of remaining creatures beside the singles
    /// (`C(n, 2)` sims once; later steps grow singly). Needs
    /// `attack_chain > 0`.
    ///
    /// **Adopted 2026-09-05 (round 56)** on the default only: 50.4 / 50.7
    /// / 50.4 / 50.4 vs the r55 default, every interval clear of 50; the
    /// net leg straddled (50.2 / 50.7). See `default_const`. Round 58
    /// then restricted the pair move (the two flags below, both adopted).
    pub attack_chain_wide: bool,
    /// The wide chain's pair move only when greedy declared *nobody* —
    /// the board it was built for. Off, the `C(n, 2)` pairs are priced at
    /// every chain's first step, including the 55 % that start beside a
    /// non-empty greedy declaration the menu already prices. A throughput
    /// device; needs `attack_chain_wide`. **Adopted 2026-09-05 (round
    /// 58)**, see `default_const`.
    pub attack_pairs_empty_only: bool,
    /// The wide chain's pair move only after the singles step ties: the
    /// overload it exists to find is exactly the board where every single
    /// addition is a trade with staying home, so the pairs are priced only
    /// when single growth would have stopped the chain at nobody. A
    /// throughput device; needs `attack_chain_wide`. **Adopted 2026-09-05
    /// (round 58)**, see `default_const`.
    pub attack_pairs_lazy: bool,
    /// Skip the empty-greedy chain when the defender's untapped blockers
    /// are at least as many as this seat's untapped creatures: greedy held
    /// every creature behind a blocker that dominates it, and with one
    /// blocker per attacker there is no overload for the chain to find.
    /// A throughput device; needs `attack_chain_wide`. **REFUTED 2026-09-05
    /// (round 59)** off its own census: the boards it covers hold 85 % of
    /// the empty-greedy chain's wins (752 of 886) and 15 % of its
    /// searches; kept off as the `empty-gate` control.
    pub attack_empty_gate: bool,
    /// Search the block assignment instead of taking the greedy one:
    /// simulate each candidate through combat damage and keep the best (see
    /// [`pick_blocks_scored`]). 0 disables it; higher values allow more
    /// candidate assignments.
    ///
    /// The block sibling of [`attack_search`](Self::attack_search), and a
    /// cheaper search than it: a block candidate only has to be simulated
    /// through this turn's combat damage, not through a full turn cycle,
    /// because the payoff of a block — who dies, how much life is saved —
    /// is settled inside the same combat.
    pub block_search: u8,
    /// Grow the block assignment one move at a time (see
    /// [`block_chain_candidate`]), the block twin of
    /// [`attack_chain`](Self::attack_chain). 0 disables it; higher values
    /// cap the moves. Each step offers "finalize" as candidate 0, one
    /// (blocker, attacker) pair for every free blocker and every attacker
    /// it may legally block — a second blocker on a blocked attacker is a
    /// gang, so gangs grow naturally — and, per attacker, a *gang move*:
    /// the cheapest free blockers that together kill it, the pair-level
    /// step forward growth cannot take alone (each gang member is a
    /// chump on its own). Every step is priced by the block sim, and the
    /// finished plan joins [`block_candidates_for_mcts`]'s menu for the
    /// picker's one argmax, greedy keeping index 0 and every tie.
    ///
    /// It runs on a one-candidate menu too: `block_candidates_for_mcts`
    /// returns bare "no blocks" whenever greedy found nothing profitable
    /// and no chump was warranted — and never generates gang candidates
    /// there, so a gang that only pays as a pair was unreachable exactly
    /// when greedy had nothing to seed it with.
    ///
    /// **Adopted 2026-09-05 (round 56)** at 4 on the default and the
    /// client pilot: 56.8 / 55.0 / 55.6 / 55.7 vs the r55 default and
    /// 57.7 / 55.7 under the net — the largest reading in the program's
    /// record. See `default_const`; `.ladder/run_r56_chains.sh` is the gate.
    pub block_chain: u8,
    /// Restore the pre-fix mana behavior: tap every land before deciding
    /// anything, and size affordability off the floating pool.
    ///
    /// Not a weight — a behavioral control, kept for the same reason
    /// [`HeuristicBot::uniform_baseline`] is, so the tap-out fix stays
    /// measurable on the ladder instead of being asserted. Approximates
    /// the old pass with its land-tap half, which is the part the
    /// measurement in `main_phase_action_with` was of.
    pub legacy_pretap: bool,
    /// Let the combat simulations cast spells: whichever seat holds
    /// priority inside [`simulate_attack_outcome_from`] /
    /// [`simulate_block_outcome_from`] fires the response layer, the
    /// combat-trick window, and — inside the attack sim's one-turn
    /// horizon — a static-ranked main-phase cast (see
    /// [`sim_spell_action`]). Off, the sims are pure priority passes and
    /// "an opponent holding removal, or ourselves holding a trick, are
    /// invisible" — the documented blindness behind the over-attack the
    /// SOS college probes measured.
    pub attack_sim_spells: bool,
    /// Take the greedy declaration without simulating when no opposing
    /// seat controls a creature, planeswalker or battle — nothing to block
    /// with, nothing to crack back with, nothing to attack instead of the
    /// face. `CRAB_ATTACK_CENSUS` reads that board as 9-16 % of searched
    /// declarations on every pool and greedy winning 94-100 % of them (the
    /// rest a lone Goblin Guide or mana dork, where the sim prices the
    /// trigger's land or the tapped mana above the damage); see
    /// [`attack_candidates_for_mcts`]. **Adopted 2026-09-05 (round 60)** on
    /// the default, not the client pilot — see `default_const`.
    pub attack_skip_open: bool,
    /// Extend the attack simulation one extra turn cycle when it ends
    /// with either life total at 10 or below. The one-cycle horizon can
    /// see "this creature survives to block" but not "this is the race I
    /// need to win" — the roadmap's race-math gap. An extension only
    /// when the sim ends inside burn range keeps the cost bounded to the
    /// positions where the extra cycle can actually reach a result.
    pub attack_race_horizon: bool,
    /// Evaluate undecided positions with the learned value net registered
    /// in this [`net_eval`](crate::server::net_eval) slot instead of the
    /// material heuristic; 0 (default) is off. The net returns a win
    /// probability scaled to 0..10 000, so a decided game's heuristic
    /// ±100 000·unit still dominates every comparison, and an empty slot
    /// falls back to the heuristic — a weights file is a runtime input,
    /// never a build requirement.
    pub net_slot: u8,
    /// With [`net_slot`](Self::net_slot) set: 0 replaces the heuristic
    /// evaluation with the net's win probability outright; a positive
    /// value blends instead — heuristic plus a `(p − 0.5) · scale · unit`
    /// bias, so full confidence is worth ±scale/2 units. The heuristic
    /// stays sharp on small material deltas the net can't resolve; the
    /// net weighs in proportionally to how far from a coin flip it judges
    /// the position. A knob rather than a constant because the right
    /// loudness is a measurement (see the `net_eval_blend*` profiles).
    pub net_blend_scale: i32,
    /// With [`net_blend_scale`](Self::net_blend_scale): scale the bias by
    /// game phase — full weight through turn 5, linear to zero at turn
    /// 12. The stratified calibration has shown three rounds running that
    /// the net's edge over the heuristic peaks at ply 8–11 and decays to
    /// nearly nothing by ply 32+ (both evaluations converge once counting
    /// settles the board), so a constant blend spends the net's voice
    /// exactly where it knows the least. The taper shape is fixed rather
    /// than a knob: the schedule is the hypothesis under test, and knobs
    /// multiply gate rounds.
    pub net_blend_ply: bool,
    /// The saturation fallback (replay diagnostic, 2026-08-31): when the
    /// net scores the *current* state outside the calibrate histogram's
    /// rankable band [0.05, 0.95] — that tool's own printout: "the search
    /// cannot rank lines inside a saturated band" — the scored combat
    /// pickers silence the net for that one decision and score every
    /// candidate with the material eval instead. A sigmoid's sensitivity
    /// is p·(1−p): at 3 life against 23 every settled leaf reads ≈0.01,
    /// the candidate deltas truncate below the tie-break jitter, and
    /// first-wins-ties hands the decision back to the eval-free menu
    /// default — a flying 5/5 the defender could not block was held for
    /// two turns of a lost race (game 5 of the 2026-08-30 replays,
    /// tap-verified). Keyed on the pre-decision state so all of one
    /// argmax's candidates score in the same currency; mid-band
    /// decisions keep the net untouched, where its measured edge lives.
    /// The decided-game clamp (±100 000·unit dominating the net's
    /// 0..10 000) is this same principle at p ∈ {0, 1}; the guard
    /// extends it to "effectively decided". Off by default until
    /// laddered ([`net_tail_guard_on`](Self::net_tail_guard_on), profile
    /// `net-guard`), with the caveat pre-registered in the gate script:
    /// mirror seats saturate together, so the ladder may under-read a
    /// change whose client-facing case stands on its own.
    pub net_tail_guard: bool,
    /// Sequence the land drop instead of taking the first land that
    /// covers the most missing colors. Two additions:
    ///
    /// * **Urgency** — a missing color is worth more when the hand cards
    ///   demanding it are cheap. Covering the color of a two-drop that
    ///   could be cast next turn beats covering the color of a six-drop.
    /// * **Tapped-land timing** — an enters-tapped land is free on a
    ///   turn with nothing to cast and costs a whole turn's play
    ///   otherwise, so it is preferred early and penalized when the
    ///   untapped mana would actually be spent.
    pub land_urgency: bool,
    /// Judge opening hands by what is *in* them, not just how many lands
    /// and whether one spell is castable. The shipped rule keeps every
    /// 2–5-land hand with a single cheap spell and ships every 6-land
    /// hand, so "two lands, one two-drop, four six-drops" is a keep and
    /// "six lands and a bomb" is a mulligan. This adds a card-quality
    /// sum ([`crate::draft::card_quality`]), a redundancy requirement at
    /// two lands, and an on-the-draw allowance — the extra card is
    /// exactly what makes a marginal hand keepable.
    pub mull_quality: bool,
    /// Decide the mulligan by *simulating both branches* instead of by a
    /// hand-written predicate.
    ///
    /// Mulligan is 25 % of every decision the bot is asked (`bot_probe
    /// --deck sos`, 300 games) — more than double the next kind, and it
    /// sets up the whole game. It is also the only high-volume decision
    /// still answered by a rule that never looks past the opening hand:
    /// modes, optional triggers and sacrifice-for-value are all judged by
    /// playing the state forward and scoring the settled result, while
    /// mulligan counts lands and asks whether one spell is castable.
    ///
    /// The predicate refinement was already tried and is a well-powered
    /// null (`mull_quality`: 50.2 % [49.6, 50.8] over 28 800 games). This
    /// is a different mechanism rather than a retune of that one: keep
    /// and mulligan are each played forward and scored, so the cost of
    /// going down a card is *measured* in the sim instead of being priced
    /// by a hand-tuned threshold. Off by default until laddered
    /// ([`mull_sim_on`](Self::mull_sim_on), profile `mullsim`).
    pub mull_sim: bool,
    /// Restore the pre-2026-08-23 planeswalker cash-out read: raw enemy
    /// creature power against *current* loyalty, no blockers, no
    /// attack-capability filter. The control for the fix, kept so the
    /// change is measurable rather than asserted (profile
    /// `walkerlegacy`); see the guard in `pick_loyalty_ability`.
    pub legacy_cashout: bool,
    /// Offer gang-blocks *for value* to the block search.
    ///
    /// The greedy pass already piles blockers onto an attacker, but only
    /// when `life_threatened` — that is, only to survive lethal. Off
    /// that trigger it blocks an attacker solely when one creature can
    /// kill it alone, so two 2/2s never eat a 4/4 however good that
    /// trade is. And [`block_search`](Self::block_search) can only ever
    /// *remove* blockers from the greedy set; adding one was outside the
    /// space it explored, which is why its documented null result says
    /// nothing about this.
    ///
    /// The gangs are candidates, not decisions:
    /// [`simulate_block_outcome_from`] prices the blockers that die against
    /// the attacker that dies, and ties keep the greedy assignment.
    pub block_gang: bool,
    /// Redeal the hidden zones before an attack/block simulation, and
    /// average this many redeals; 0 (the historical behaviour) searches
    /// the true state.
    ///
    /// The combat sims clone the real [`GameState`], so the rollout
    /// opponent casts the cards they are actually holding and both seats
    /// draw the real top of their library. The bot is therefore searching
    /// with perfect information: it can decline an attack because it has
    /// *seen* the trick, which is not a read, it is looking at the hand.
    ///
    /// Two separate reasons that matters, worth keeping apart:
    ///
    /// * Against a human in the client it is simply cheating, whatever it
    ///   does to the win rate.
    /// * The mirror ladder is structurally incapable of detecting it,
    ///   because both seats cheat identically. No measurement this
    ///   harness has ever run could have caught this, which is why it is
    ///   a knob with a documented default rather than a silent fix.
    ///
    /// Averaging several redeals is the honest version: one sample
    /// replaces perfect information with a *wrong* hand, which is its own
    /// bias, while the mean over redeals approximates playing against the
    /// distribution of hands consistent with what the seat can see.
    pub determinize: u8,
    /// Redeal hidden hands from the net's opponent-hand belief head
    /// instead of uniformly (round 39, `determinize_hidden_belief`).
    /// Only meaningful with `determinize > 0` and a `net_slot` whose net
    /// carries `head_opp.*`; anything else silently keeps the uniform
    /// redeal, which is why the profile that turns this on reports its
    /// belief source at startup. Off by default — the uniform redeal is
    /// the historical behaviour and every adopted profile.
    pub belief_determinize: bool,
    /// Copied onto this seat's [`Player::smart_tap`] before the game
    /// starts: spend the most replaceable mana source for each pip
    /// instead of the first in battlefield order.
    ///
    /// The behaviour lives in the engine's auto-tap, so this field exists
    /// only so the ladder can put one seat on each side of it. **Off by
    /// default** — it measured null over 28 800 paired games (see
    /// [`Player::smart_tap`]).
    ///
    /// [`Player::smart_tap`]: crate::player::PlayerData::smart_tap
    pub smart_tap: bool,
    /// Answer an opponent's aura or pump spell aimed at their own
    /// creature by killing the creature in response — the buff fizzles
    /// on a vanished target and one card trades for two. Off by default
    /// until laddered ([`buff_2for1_on`](Self::buff_2for1_on), profile
    /// `buff2for1`).
    pub buff_2for1: bool,
    /// Converge-aware land drops: when the seat holds a card whose text
    /// computes `ConvergedValue` (hand or battlefield), a land producing
    /// a color the mana base doesn't yet make earns a bonus even when no
    /// pip demands that color — a converge deck's first game recorded
    /// the bot playing its third Plains where the human diversified.
    /// Off by default until laddered
    /// ([`converge_lands_on`](Self::converge_lands_on), profile
    /// `convlands`).
    pub converge_lands: bool,
    /// Desperation chump blocks: when unblocked attackers threaten
    /// lethal within two swings, add chump candidates to the block menu
    /// and let the simulations judge them. Without this the menu is
    /// built from *profitable* blocks only — a greedy pass that finds
    /// none returns a bare "no blocks" and the sims never run, so the
    /// bot at 5 life takes 4 to the face holding a chump (the first
    /// recorded human game, turns 12/14, bot seat). Off by default
    /// until laddered ([`chump_blocks_on`](Self::chump_blocks_on),
    /// profile `chumpblocks`).
    pub chump_blocks: bool,
    /// Value-ordered combat damage (CR 510.1c): order a multi-blocked
    /// attacker's blockers (or a multi-blocking creature's attackers, the
    /// 510.1e mirror) so the engine's lethal-to-each split kills what the
    /// assigner most wants dead. `Decision::CombatDamageOrder` has no
    /// policy arm at all — every gang block in this program's history
    /// resolved in declaration (CardId) order, so "kill the better
    /// blocker" was not expressible at any valuation: the same
    /// missing-candidate shape as chump blocks (round 43, +0.9), one
    /// step later in the same combat. The policy simulates the engine's
    /// own default split per candidate order and keeps the best signed
    /// outcome for the deciding seat — banding and Defensive Formation
    /// hand the choice to the victims' controller, whose best order is
    /// the reverse preference, and the sign handles both chairs. An
    /// order whose outcome matches the default's answers empty, so games
    /// where the choice cannot matter play (and pair) exactly as before.
    ///
    /// **Adopted 2026-08-30 (round 52)**: 50.4 % [50.2, 50.6] and
    /// 50.3 % [50.1, 50.5] vs `gang`, seeds 43/97, 24 000 paired sealed
    /// games, both intervals clear of 50 — the round-50 rare-class rule
    /// (incidence ~0.12/game, `.ladder/r52_probe.txt`). The exact gate
    /// re-runs as `dmgorder` vs `gang`: neither profile carries the
    /// default's determinize/chump layers, so adoption did not consume
    /// the control ([`damage_order_on`](Self::damage_order_on),
    /// `.ladder/run_r52_dmgorder.sh`).
    pub damage_order: bool,
    /// Mid-resolution targets judged by settled outcome (round 53):
    /// `Decision::ChooseTarget` on the suspending path — trigger target
    /// picks (`drain_trigger_queue`) and the cast-slot / off-board picks
    /// in `actions.rs` — is answered by `decide_choose_target`, a
    /// polarity guess that hard-codes "hit the opponent's biggest, else
    /// give up our cheapest" and can never decline an optional "up to
    /// one" target. Correct for removal, backwards for every beneficial
    /// resolution effect whose legal set spans both sides — the same
    /// classifier gap `target_arms` (round 46, +0.95) closed for
    /// cast-time slot 0, at the raise sites that flag never touched.
    /// Under this flag the corner candidates (biggest/smallest permanent
    /// per side, each legal player, the decline when optional) are
    /// settled by `settle_answer` and the heuristic's pick is replaced
    /// only on strict improvement, so games where the guess is right
    /// play (and antithetically pair) exactly as before. Real decisions
    /// only (`eval_modes`); sims keep the cheap guess. The seven inline
    /// `self.decider.decide` sites in `effects/mod.rs` (votes, copy
    /// retargeting) never reach any policy and are NOT covered — a
    /// separate hole needing per-site suspend plumbing.
    ///
    /// **Measured round 53 and left off**: 50.1 % on both ladder seeds
    /// (intervals straddling 50 — the ladder's own verdict line), sweep
    /// asymmetry same-signed on both (37/24, 43/32). A positive lean
    /// the r50 rule does not adopt. The diagnosis is a pool property:
    /// 5,925+ of 6,000 pairs were exact mirrors, i.e. the polarity
    /// guess is almost always already right in SOS sealed, where
    /// mid-resolution targets are mostly removal-shaped. Re-run on a
    /// trigger-denser pool (modern/cube) before re-judging
    /// ([`target_eval_on`](Self::target_eval_on), profile `targeteval`,
    /// `.ladder/run_r53_targeteval.sh`).
    pub target_eval: bool,
    /// Walker chip attacks: the greedy pass attacks a planeswalker only
    /// when it can finish it, so a healthy walker sits unpressured to
    /// its ultimate (recorded: ten turns, a lost game). The flag adds
    /// one attack candidate — the smallest face attacker with power ≥ 2
    /// redirected at the lowest-loyalty walker — and the simulations
    /// judge the trade. Off by default until laddered
    /// ([`walker_chip_on`](Self::walker_chip_on), profile `walkerchip`).
    pub walker_chip: bool,
    /// Non-mana activated abilities as main-phase candidates: ability
    /// usage was a handful of hand-written classes, so Sundering
    /// Archaic's {2} exile could never be chosen at ANY valuation —
    /// nothing enumerated it. Two cheapest affordable activations join
    /// the candidate list, lazily validated like the main cast block.
    /// Off by default until laddered
    /// ([`ability_arms_on`](Self::ability_arms_on), profile `abilarms`).
    pub ability_arms: bool,
    /// Activate "impulse draw" abilities — mill or exile off the top and
    /// gain permission to play the card (`Effect::GrantMayPlay`). The
    /// ability generators are a whitelist of effect *shapes* and this one
    /// was on none of them, so Ark of Hunger sat unused for five turns of
    /// a recorded game while the bot topdecked with an empty hand. Only
    /// fires with a short hand and mana to spare, so it cannot mill the
    /// bot out for nothing. Off by default until laddered
    /// ([`impulse_draw_on`](Self::impulse_draw_on), profile `impulse`).
    pub impulse_draw: bool,
    /// Offer the search *alternative targetings* of the same spell, not
    /// just the one the auto-targeter picked.
    ///
    /// Every cast candidate bakes in a single assignment from
    /// `auto_targets_for_effect_all_slots`, so the search can only accept
    /// or reject that package — the correct targeting of a spell it is
    /// already considering is not on the menu at any valuation. That is
    /// the same structural hole the chump-block menu had (round 43,
    /// +0.9): not a valuation failure, a missing candidate. Recorded
    /// games reached it twice (Proctor's Gaze bouncing our own body,
    /// Homesickness stunning our own board); both were patched in the
    /// heuristic classifier, but mixed-polarity cards are unbounded and
    /// the classifier will keep having gaps the search could judge for
    /// itself. Off by default until laddered
    /// ([`target_arms_on`](Self::target_arms_on), profile
    /// `mcts-net-targetarms`).
    pub target_arms: bool,
    /// Reserve an MCTS root arm for a prepared-creature cast: the menu
    /// caps at six arms by heuristic score, and the banked inset spell
    /// is a rare, high-value class the cap can crowd out (two prepared
    /// Ancestral Recalls sat unfired through a recorded loss). Off by
    /// default until laddered (profile `mcts-net-prep`).
    pub prepare_arm: bool,
    /// Search the library-search decision instead of answering it with a
    /// fixed heuristic. `Decision::SearchLibrary` never reaches the MCTS
    /// menu at all: `MctsBot::next_action` falls through to the heuristic
    /// whenever a decision is pending, so every tutor and every fetchland
    /// is resolved by `decide_library_search`'s hardcoded read — supply-only
    /// for basics, and "biggest mana value" for anything else. That is the
    /// same missing-candidate shape as chump blocks (round 43, +0.9) and
    /// target arms (round 46, +0.95): the right fetch is not a low-scoring
    /// arm the search rejects, it is absent. Off by default until laddered
    /// ([`fetch_arms_on`](Self::fetch_arms_on), profile `mcts-net-fetcharms`).
    pub fetch_arms: bool,
    /// The round-51 reproduction control: keep the pre-fix library-search
    /// ranking (supply-only for basics, biggest mana value for tutor hits)
    /// so the demand-aware read can be gated against the read it replaces.
    pub legacy_fetch: bool,

    /// Quantize the net's win probability onto a grid of this many
    /// levels before the search consumes it; 0 is off (continuous).
    ///
    /// The point is to let the net *tie*. `--pairwise` measured the two
    /// evaluators on adjacent same-game snapshots: both are barely above
    /// chance at ordering them (net 54.3 %, heuristic 51.7 %), but
    /// `eval_material` declines to separate 46.9 % of such pairs while
    /// the net separates 100 % of them, asserting a mean 5.5 points of
    /// win probability between boards one turn apart.
    ///
    /// In an argmax search over candidate lines that are genuinely
    /// near-equal, an evaluator that always has an opinion follows its
    /// own noise; one that ties falls through to criteria that carry
    /// real information. The heuristic's coarseness looks like a
    /// limitation and behaves like a feature. Rounding the net onto a
    /// grid buys the same property: two positions within one grid step
    /// score identically, so the net only overrides when it has a large
    /// opinion rather than a noisy one.
    ///
    /// With mean adjacent separation 0.055, a 10-level grid (0.1 steps)
    /// ties most adjacent pairs and a 20-level grid (0.05) ties roughly
    /// half — about where the heuristic sits.
    pub net_quantize: u32,
}

impl EvalWeights {
    /// The historical weights: mana value + power + toughness, one point
    /// each, no keyword term, linear life. Kept exactly as-is so it stays
    /// a valid ladder control — it is what every run measures against, not
    /// what the bot plays (see [`Default`](EvalWeights::default)).
    /// See [`v2`](Self::v2).
    pub const fn baseline() -> Self {
        Self {
            unit: 1,
            cmc: 1,
            creature_base: 0,
            power: 1,
            toughness: 1,
            keyword_pct: 0,
            concave_life: false,
            hold_sick: false,
            hold_instants: false,
            lookahead: 0,
            combat_aware: false,
            attack_search: 0,
            attack_chain: 0,
            attack_chain_wide: false,
            attack_pairs_empty_only: false,
            attack_pairs_lazy: false,
            attack_empty_gate: false,
            block_search: 0,
            block_chain: 0,
            legacy_pretap: false,
            attack_sim_spells: false,
            attack_skip_open: false,
            attack_race_horizon: false,
            net_slot: 0,
            net_blend_scale: 0,
            net_blend_ply: false,
            land_urgency: false,
            mull_quality: false,
            mull_sim: false,
            legacy_cashout: false,
            block_gang: false,
            determinize: 0,
            belief_determinize: false,
            smart_tap: false,
            buff_2for1: false,
            converge_lands: false,
            chump_blocks: false,
            damage_order: false,
            target_eval: false,
            net_tail_guard: false,
            walker_chip: false,
            ability_arms: false,
            impulse_draw: false,
            target_arms: false,
            prepare_arm: false,
            fetch_arms: false,
            legacy_fetch: false,
            net_quantize: 0,
        }
    }

    /// Candidate weights ported from the reference AIs: body ratios and
    /// power-scaled keyword terms from Forge's `CreatureEvaluator`, the
    /// life curve from XMage's `ArtificialScoringSystem::LIFE_SCORES`,
    /// `unit = 10` so those ratios survive integer division.
    ///
    /// **Measured worse than [`baseline`], and not adopted.** Over 12 000
    /// laddered games it lands at 49.4 % (baseline 50.6 %, CI straddling
    /// 50 %), and the [`keywords_only`] decomposition shows the keyword
    /// term is the part that costs: pooled over 20 000 games the baseline
    /// beats it 51.1 % [50.4 %, 51.8 %].
    ///
    /// The first explanation offered for this was *depth*: that a richer
    /// evaluation of a position the bot can only see one action deep gives
    /// a greedy step more confidence without more foresight, and that these
    /// weights are calibrated for the real search their sources run (Forge
    /// fast-forwards to combat damage and plans three plies; XMage runs
    /// depth-4 alpha-beta). **That hypothesis was tested and is wrong.**
    /// [`v2_combat`] — the same weights with the combat-aware evaluator —
    /// measures 53.1 % to the baseline, i.e. *worse* than v2 alone. Extra
    /// depth does not rescue them.
    ///
    /// So the honest reading is that these numbers are simply wrong for
    /// this engine's surrounding balance, not merely premature. They are
    /// kept as a documented dead end: a future attempt should re-derive
    /// weights against *this* evaluator's creature-to-card and
    /// board-to-life ratios rather than port another engine's, and can use
    /// the decomposition profiles below to do it one term at a time.
    ///
    /// [`baseline`]: Self::baseline
    /// [`keywords_only`]: Self::keywords_only
    /// [`v2_combat`]: Self::v2_combat
    pub const fn v2() -> Self {
        Self {
            unit: 10,
            cmc: 10,
            creature_base: 100,
            power: 15,
            toughness: 10,
            keyword_pct: 100,
            concave_life: true,
            hold_sick: false,
            hold_instants: false,
            lookahead: 0,
            combat_aware: false,
            attack_search: 0,
            attack_chain: 0,
            attack_chain_wide: false,
            attack_pairs_empty_only: false,
            attack_pairs_lazy: false,
            attack_empty_gate: false,
            block_search: 0,
            block_chain: 0,
            legacy_pretap: false,
            attack_sim_spells: false,
            attack_skip_open: false,
            attack_race_horizon: false,
            net_slot: 0,
            net_blend_scale: 0,
            net_blend_ply: false,
            land_urgency: false,
            mull_quality: false,
            mull_sim: false,
            legacy_cashout: false,
            block_gang: false,
            determinize: 0,
            belief_determinize: false,
            smart_tap: false,
            buff_2for1: false,
            converge_lands: false,
            chump_blocks: false,
            damage_order: false,
            target_eval: false,
            net_tail_guard: false,
            walker_chip: false,
            ability_arms: false,
            impulse_draw: false,
            target_arms: false,
            prepare_arm: false,
            fetch_arms: false,
            legacy_fetch: false,
            net_quantize: 0,
        }
    }

    // -- Ladder decomposition profiles ---------------------------------
    //
    // A profile that bundles several changes can only ever be laddered as
    // a bundle, and a bundle that loses tells you nothing about which part
    // lost. These turn on one change at a time against a common scale.

    /// Pure control: the baseline ratios at `unit = 10`. Every term is
    /// exactly ten times the baseline's, so this *should* pick the same
    /// actions and ladder at 50 % -- it measures 50.9 % [49.4 %, 52.5 %],
    /// i.e. indistinguishable, which confirms the remaining scale-dependent
    /// behavior (integer truncation in `score_candidate`, and the
    /// fixed-size tie-break jitter, a tenth as influential at this scale)
    /// costs nothing measurable. Run this before attributing a ladder
    /// result to any of the weights themselves.
    pub const fn scaled_control() -> Self {
        Self {
            unit: 10,
            cmc: 10,
            creature_base: 0,
            power: 10,
            toughness: 10,
            keyword_pct: 0,
            concave_life: false,
            hold_sick: false,
            hold_instants: false,
            lookahead: 0,
            combat_aware: false,
            attack_search: 0,
            attack_chain: 0,
            attack_chain_wide: false,
            attack_pairs_empty_only: false,
            attack_pairs_lazy: false,
            attack_empty_gate: false,
            block_search: 0,
            block_chain: 0,
            legacy_pretap: false,
            attack_sim_spells: false,
            attack_skip_open: false,
            attack_race_horizon: false,
            net_slot: 0,
            net_blend_scale: 0,
            net_blend_ply: false,
            land_urgency: false,
            mull_quality: false,
            mull_sim: false,
            legacy_cashout: false,
            block_gang: false,
            determinize: 0,
            belief_determinize: false,
            smart_tap: false,
            buff_2for1: false,
            converge_lands: false,
            chump_blocks: false,
            damage_order: false,
            target_eval: false,
            net_tail_guard: false,
            walker_chip: false,
            ability_arms: false,
            impulse_draw: false,
            target_arms: false,
            prepare_arm: false,
            fetch_arms: false,
            legacy_fetch: false,
            net_quantize: 0,
        }
    }

    /// Baseline + Forge's summon-sick gate, for laddering it on its own.
    ///
    /// **Adopted — this is [`EvalWeights::default`].** Measured 51.5 %
    /// [50.8 %, 52.3 %] over 16 000 games, after two 4000-game runs at
    /// 50.8 % and 50.9 % pointed the same way. Worth roughly +1.5 points.
    ///
    /// Its behavioral effect is large and verifiable: casts in the
    /// precombat main go from 91.9 % to 25.3 %, with 66.2 % moving to the
    /// second main (`bot_probe`, land drops excluded — those are
    /// sorcery-speed by rule and can never be held). That is simply what
    /// correct sequencing looks like. It costs ~30 % more CPU per decision,
    /// since the gate resolves the winning line a second time.
    pub const fn hold_sick() -> Self {
        Self { hold_sick: true, ..Self::baseline() }
    }

    /// Baseline + the instant-speed hold. Needs
    /// [`combat_aware`](Self::combat_aware) to be much use: without it the
    /// gate cannot tell "kill the blocker before I attack" (worth doing
    /// now) from "kill it at end of turn" (worth the same, later).
    pub const fn hold_instants() -> Self {
        Self { hold_instants: true, combat_aware: true, ..Self::baseline() }
    }

    /// The adopted default plus one ply of sequence lookahead.
    ///
    /// **Measured neutral and not adopted**: 50.2 % [49.1 %, 51.3 %]
    /// against the default over 8000 games, with no consistent direction
    /// across archetypes (mono-red and dimir favour it, skies and golgari
    /// don't), at roughly 2.4x the CPU per decision.
    ///
    /// The likely reason is that the summon-sick gate already banked most
    /// of the available sequencing value. The bot was never unable to cast
    /// several spells in a turn — the main-phase loop runs every tick — it
    /// was unable to *compare combinations*, and once plays are deferred to
    /// the second main the greedy loop deploys them anyway. What is left is
    /// the narrower case where the first pick is wrong *given* what follows,
    /// which one ply and two continuations apparently doesn't catch often
    /// enough to measure.
    ///
    /// Forge searches three plies rather than one. Going deeper here costs
    /// proportionally more and, on this series' base rate, isn't worth
    /// betting on without evidence — but the machinery is in place if
    /// someone wants to try `lookahead: 2` and measure it.
    pub const fn lookahead1() -> Self {
        Self { lookahead: 1, ..Self::hold_sick_combat() }
    }

    /// **The adopted default.** The summon-sick gate plus the combat-aware
    /// evaluation, with the instant-speed hold left off.
    ///
    /// This is the decomposition of [`planner`](Self::planner), and it is
    /// why the bundle is not what shipped. Against [`hold_sick`] alone this
    /// measures 51.3 % [50.4 %, 52.2 %] over 12 000 games, while the full
    /// planner measures 51.0 % [50.2 %, 51.8 %] over 16 000 — the same
    /// within error. So `combat_aware` carries the gain and
    /// `hold_instants` adds nothing detectable on top of it.
    ///
    /// The interesting part is that `combat_aware` measured *exactly*
    /// neutral on its own (50.0 % over 12 000 games, 6002-5998). It was
    /// never a bad idea, it just had no consumer: within a single main
    /// phase this turn's combat is nearly identical whichever candidate is
    /// picked, so the term cancelled. Give the bot a reason to ask "is this
    /// worth the same later?" and the same signal is worth +1.3 points.
    ///
    /// [`hold_sick`]: Self::hold_sick
    pub const fn hold_sick_combat() -> Self {
        Self { combat_aware: true, ..Self::hold_sick() }
    }

    /// The adopted default: [`hold_sick_combat`](Self::hold_sick_combat)
    /// plus the searched attack declaration.
    ///
    /// **Measured, and the largest gain since the tap-out fix**: 52.4 %
    /// [51.3 %, 53.5 %] over 8 000 fixed-deck games, and 53.8 %
    /// [53.0 %, 54.6 %] over 13 695 decided cube games, at about +36 %
    /// wall clock.
    ///
    /// The fixed-deck aggregate badly understates how *deck-dependent* it
    /// is, which is the more useful finding:
    ///
    /// | mirror | searched attacks win % |
    /// |---|---|
    /// | mono-red aggro | 59.6 % |
    /// | azorius skies | 56.7 % |
    /// | golgari midrange | 48.5 % |
    /// | dimir control | 44.8 % |
    ///
    /// Restraint is worth nearly ten points in the aggro mirror and *costs*
    /// five in the control mirror, where somebody has to actually close the
    /// game and the passive side doesn't. The search has a one-turn-cycle
    /// horizon, so it can see "this creature survives to block" and cannot
    /// see "this is the race I need to win"; a deck whose plan is inevitable
    /// card advantage is exactly where that blind spot bites. Adopted on the
    /// aggregate, but a profile that scales restraint to the board — or a
    /// horizon that reaches a win — is the obvious next thing to measure.
    pub const fn attack_search() -> Self {
        Self { attack_search: 6, ..Self::hold_sick_combat() }
    }

    /// The adopted default plus [`hold_instants`](Self::hold_instants).
    ///
    /// The hypothesis this existed to test, straight out of the SOS
    /// college probes (`bot_probe --deck sos:prismari --vs baseline`):
    /// in the instant-speed college the default profile cast exactly ONE
    /// spell at instant timing across 60 games, main-phased its instants
    /// proactively, tapped out, and pitched 42 hands' worth of reactive
    /// spells to cleanup — while the ladder read Prismari ≈ 49 % against
    /// the control. `hold_instants` had measured neutral on the four
    /// constructed decks (see [`hold_instants`]), but none of those decks
    /// was built from a pool where half the playables are instants, so
    /// this re-asked the question where it should have mattered most.
    ///
    /// **Measured, and not adopted**: 49.4 % [46.3 %, 52.5 %] against
    /// `atk` over 1000 SOS college-mirror games (seed 11) — statistically
    /// indistinguishable from the atk-vs-atk control at the same seed
    /// (48.9 %), i.e. holding bought nothing, at +65 % wall clock for the
    /// extra `improves_this_turn` simulations. (An earlier reading of the
    /// per-college rows as "Prismari got worse" was noise: the identical-
    /// profile control swings its own college rows to 44.5 % at 200
    /// games. Only the pooled total is a result.) The probe's real
    /// Prismari signal is elsewhere: reactive spells rot because the
    /// response layer under-fires, and the attack search over-swings on
    /// small boards (82 % of eligible, 78 % all-in) — restraint, not
    /// timing, is the open lead; see [`attack_search_sim`].
    pub const fn attack_search_hold() -> Self {
        Self { hold_instants: true, ..Self::attack_search() }
    }

    /// The adopted default with spell-casting combat simulations.
    ///
    /// The hypothesis, out of the SOS college diagnosis (per-college
    /// probes plus the `atk-hold` and `blk` dead ends): the attack search
    /// over-swings on small boards — 82 % of eligible declared, 78 %
    /// all-in in Prismari, 41 % of creatures tapped when blocks come —
    /// because its simulation casts nothing for either side, so a swing
    /// into open mana and a hand full of removal sims as free. With
    /// [`attack_sim_spells`](Self::attack_sim_spells) the crack-back is
    /// visible at declaration time.
    ///
    /// **Measured, and adopted as the default.** Three runs against
    /// `atk`, all positive, plus an identical-profile control:
    ///
    /// | field | result | games |
    /// |---|---|---|
    /// | SOS colleges, seed 11 | 51.7 % [48.6 %, 54.8 %] | 1 000 |
    /// | SOS colleges, seed 7 | 53.2 % [50.1 %, 56.3 %] | 1 000 |
    /// | fixed + cube, seed 11 | 54.4 % [53.0 %, 55.8 %] | 4 794 |
    /// | control (atk vs atk, SOS) | 48.9 % [45.8 %, 52.0 %] | 1 000 |
    ///
    /// No archetype below 50 % on the deciding run, and the largest gain
    /// is dimir control at 61.3 % — the archetype where the blind search
    /// measured 44.8 % and the "restraint costs five points in the
    /// control mirror" caveat was written. Cost: roughly 2-4× the ladder
    /// wall clock of `atk`, all of it on DeclareAttackers/blocks ticks.
    pub const fn attack_search_sim() -> Self {
        Self { attack_sim_spells: true, ..Self::attack_search() }
    }

    /// The adopted default plus the race horizon
    /// ([`attack_race_horizon`](Self::attack_race_horizon)) — the
    /// roadmap's "race math" hypothesis: an attack sim that ends inside
    /// burn range keeps going one cycle so a winning (or losing) race is
    /// scored as such instead of mid-sprint.
    ///
    /// **Measured, and not adopted**: the pre-registered 4× decision run
    /// (1 600 games/archetype, seed 12) read 50.2 % [49.5 %, 51.0 %]
    /// over 19 200 fixed+cube games vs `atk-sim` — the interval
    /// straddles 50 % and the edge is a fifth of the MARGINAL bar.
    ///
    /// The first decider (4 796 games, seed 11) had read 51.2 %
    /// [49.8 %, 52.6 %] with mono-red — the archetype the horizon
    /// exists for — at 54.8 % over 400 games. At 4× the sample the
    /// pooled edge collapsed +1.2 → +0.2 and mono-red reverted to
    /// 49.9 %: the same replication failure
    /// [`block_search`](Self::block_search) documents, reproduced on a
    /// different hypothesis. Whatever the extended horizon sees in the
    /// last burn-range turn, the default profile's one-cycle sim
    /// already prices well enough that the extra cycle (and its extra
    /// fuel) buys nothing measurable. Kept as a profile because the
    /// negative result is worth more than the code.
    pub const fn attack_search_race() -> Self {
        Self { attack_race_horizon: true, ..Self::attack_search_sim() }
    }

    /// The adopted default piloted by the learned SOS-sealed value net
    /// ([`net_slot`](Self::net_slot) = the registry's best slot):
    /// `eval_material` returns the net's win probability instead of the
    /// material count, so every outcome-eval'd decision — casts, blocks,
    /// modes, scries, sacrifices, the combat sims — optimizes the learned
    /// value. Candidate *scoring* and the rest of the decision table stay
    /// heuristic; the net replaces the judge, not the shortlist.
    ///
    /// Requires a net in slot 1 (`CRAB_NET` on the ladder, the training
    /// loop's promotion in `selfplay_train`); with the slot empty this is
    /// exactly `attack_search_sim`. Gate on sealed mirrors (`bot_ladder
    /// --decks sealed`) before any adoption claim.
    ///
    /// **Measured across three checkpoints, and not adopted** (1 200
    /// sealed-mirror games vs `atk-sim` each): 43.6 % [40.8, 46.4] on the
    /// round-1 net (25 k games), 42.3 % [39.6, 45.1] on the round-2 net
    /// (4× the data), 43.4 % [40.6, 46.2] after round 2's over-reused
    /// training tail, 44.7 % [41.9, 47.5] on the round-3 net (mid-turn
    /// snapshot cadence, 10.5 M rows), 43.8 % [41.0, 46.6] on the
    /// round-4 net (5× capacity + keyword object features — but only
    /// 0.4 learner visits per row: at 600 k parameters the CPU learner,
    /// not generation, is the bottleneck, so round 4 tested capacity at
    /// half an epoch; a fair capacity test needs the GPU learner).
    /// Better than the MCTS attempt's 41.5 %, worse than the tuned
    /// heuristic, and *flat-to-marginal across a 4× data jump and the
    /// distribution fix*: neither data volume nor snapshot coverage is
    /// the binding constraint at small capacity. Worth naming what the net is actually up against:
    /// `eval_material` scores the *outcomes of resolved simulations* — a
    /// one-ply search with a perfect forward model — so a value net only
    /// helps where it carries long-horizon signal the material count
    /// misses, and a ~125 k-parameter pooled encoder evidently carries
    /// little yet. Next levers, in order: capacity, richer object
    /// features, search-improved training targets.
    /// The adopted default plus sequenced land drops
    /// ([`land_urgency`](Self::land_urgency)).
    ///
    /// **Measured, and not adopted** — but the route there is worth more
    /// than the result:
    ///
    /// | field | result | games |
    /// |---|---|---|
    /// | fixed + cube, seed 23 | 49.4 % [47.9 %, 50.8 %] | 4 800 |
    /// | sealed, seed 23 | 51.4 % [50.0 %, 52.8 %] | 4 800 |
    /// | **sealed, seed 29 (decider)** | **50.3 % [49.6 %, 51.0 %]** | **19 200** |
    ///
    /// The first row could not have read anything else: the fixed and
    /// cube archetypes play basics almost exclusively, so the
    /// tapland-timing half of this profile never fires there. A profile
    /// can only be measured on decks containing the cards it reasons
    /// about, and running it on the default field first was a wasted
    /// 4 800 games.
    ///
    /// Moving to sealed — where the builder actually produces school
    /// lands — read +1.4 with the lower bound exactly on 50.0, so the
    /// 4× run was pre-registered as the decision rather than reported.
    /// It came back +0.3. That is the third time this harness has seen
    /// a promising sub-5 000-game result evaporate at 4× the sample
    /// (see [`block_search`](Self::block_search) and
    /// [`attack_search_race`](Self::attack_search_race)); the pattern is
    /// now reliable enough to treat any 400-games-per-archetype edge as
    /// a hypothesis, never a finding.
    ///
    /// Why it plausibly does nothing: the sealed builder gives most
    /// decks two colors and a handful of duals, so the tapland decision
    /// arises a few times a game and usually has one obvious answer the
    /// old first-playable rule already stumbled into.
    pub const fn land_sequencing() -> Self {
        Self { land_urgency: true, ..Self::attack_search_sim() }
    }

    /// The adopted default plus quality-aware mulligans
    /// ([`mull_quality`](Self::mull_quality)).
    ///
    /// **Measured and not adopted**: 50.7 % [49.7 %, 51.7 %] over 9 600
    /// sealed games, 50.2 % [49.6 %, 50.8 %] over 28 800 on the
    /// pre-registered decider. The fourth consecutive evaporation of a
    /// sub-10 000-game edge in this harness (after
    /// [`block_search`](Self::block_search),
    /// [`attack_search_race`](Self::attack_search_race) and
    /// [`land_sequencing`](Self::land_sequencing)) — at this point any
    /// result here under ~20 000 games should be read as a hypothesis
    /// however clean its interval looks.
    ///
    /// The rule changes are still the right *shape* — its tests pin two
    /// hands the shipped heuristic reads backwards — so the likely
    /// reading is that opening-hand quality matters less than it feels
    /// like it should when both seats mulligan by the same rule in a
    /// mirror: the edge cancels.
    pub const fn mulligan_quality() -> Self {
        Self { mull_quality: true, ..Self::attack_search_sim() }
    }

    /// Value gang-blocks ([`block_gang`](Self::block_gang)) plus the
    /// `block_search` that scores them — with the search at 0 the gang
    /// candidates are never evaluated, so the two ship together.
    ///
    /// **Adopted — this is [`EvalWeights::default`].** 51.3 %
    /// [50.7 %, 51.9 %] (seed 43) and 51.1 % [50.5 %, 51.7 %] (seed 97),
    /// 28 800 sealed games each vs `atk-sim`, after a 9 600-game
    /// screening read 51.0 %. Unlike the four other play-side profiles
    /// tried alongside it, the edge did not shrink at 3× the sample.
    ///
    /// What it adds: at a healthy life total the greedy pass blocks an
    /// attacker only when one creature kills it alone, so two 2/2s never
    /// ate a 4/4 however good the trade. Gangs are now offered as
    /// candidates and [`simulate_block_outcome_from`] prices the dead
    /// blockers against the dead attacker.
    ///
    /// The bundle caveat, stated plainly: `block_search` alone measured
    /// null (50.4 % over 30 000 games) and is switched on here. That is
    /// not evidence the earlier rejection was wrong — the search had
    /// nothing to find while its only candidates were "block with one
    /// fewer creature".
    pub const fn block_gang_search() -> Self {
        Self { block_gang: true, block_search: 2, ..Self::attack_search_sim() }
    }

    // ── Re-measurement profiles ───────────────────────────────────────
    //
    // Four ideas were measured against `attack_search_sim` and dropped
    // for reading ~50 %, and one (`lookahead1`) for reading 50.2 % over
    // 8 000 games. Every one of those runs was unpaired, and the paired
    // ladder puts the realized within-pair correlation at −0.74 on this
    // field: those game counts carried roughly a quarter of the
    // precision they appeared to. A null at that resolution is not
    // evidence of a null, so each idea gets one honest re-test.
    //
    // They are rebased onto the *current* default rather than reusing
    // the originals: `land_sequencing` and friends branch from
    // `attack_search_sim`, and gang-blocking has been adopted since, so
    // laddering them as written would measure "the idea, minus
    // gang-blocking" and charge the difference to the idea.

    /// [`land_sequencing`](Self::land_sequencing) rebased onto the
    /// adopted default, for the paired re-test.
    pub const fn land_sequencing_default() -> Self {
        Self { land_urgency: true, ..Self::block_gang_search() }
    }

    /// [`mulligan_quality`](Self::mulligan_quality) rebased onto the
    /// adopted default, for the paired re-test.
    pub const fn mulligan_quality_default() -> Self {
        Self { mull_quality: true, ..Self::block_gang_search() }
    }

    /// [`attack_search_race`](Self::attack_search_race) rebased onto the
    /// adopted default, for the paired re-test.
    pub const fn attack_race_default() -> Self {
        Self { attack_race_horizon: true, ..Self::block_gang_search() }
    }

    /// [`lookahead1`](Self::lookahead1) rebased onto the adopted
    /// default, for the paired re-test.
    pub const fn lookahead1_default() -> Self {
        Self { lookahead: 1, ..Self::block_gang_search() }
    }

    /// Two plies of sequence lookahead — the depth `lookahead1`'s doc
    /// comment invites someone to measure. Forge searches three.
    pub const fn lookahead2_default() -> Self {
        Self { lookahead: 2, ..Self::block_gang_search() }
    }

    /// The default, searching a single redeal of the hidden zones
    /// instead of the true state — see
    /// [`determinize`](Self::determinize).
    pub const fn determinized() -> Self {
        Self { determinize: 1, ..Self::block_gang_search() }
    }

    /// [`net_eval`](Self::net_eval) on a 10-level grid — see
    /// [`net_quantize`](Self::net_quantize).
    pub const fn net_eval_q10() -> Self {
        Self { net_quantize: 10, ..Self::net_eval() }
    }

    /// [`net_eval`](Self::net_eval) on a 20-level grid.
    pub const fn net_eval_q20() -> Self {
        Self { net_quantize: 20, ..Self::net_eval() }
    }

    /// [`net_eval_blend`](Self::net_eval_blend) on a 10-level grid.
    pub const fn net_blend_q10() -> Self {
        Self { net_quantize: 10, ..Self::net_eval_blend() }
    }

    /// [`net_eval_blend`](Self::net_eval_blend) on a 20-level grid.
    pub const fn net_blend_q20() -> Self {
        Self { net_quantize: 20, ..Self::net_eval_blend() }
    }

    /// The default plus replaceability-aware mana tapping — the opt-in
    /// for [`smart_tap`](Self::smart_tap), which is off by default after
    /// measuring null. Ladder this as A against the default.
    pub const fn smart_tap_on() -> Self {
        Self { smart_tap: true, ..Self::block_gang_search() }
    }

    /// The default plus the stack 2-for-1 — kill the creature under the
    /// opponent's own buff spell so the buff fizzles. The opt-in for
    /// [`buff_2for1`](Self::buff_2for1); ladder as A against the default.
    pub const fn buff_2for1_on() -> Self {
        Self { buff_2for1: true, ..Self::block_gang_search() }
    }

    /// The default plus the open-board shortcut. The opt-in for
    /// [`attack_skip_open`](Self::attack_skip_open); ladder as A against
    /// the default. A throughput device, so the gate it must pass is "no
    /// loss" rather than "a win".
    pub const fn attack_skip_open_on() -> Self {
        Self { attack_skip_open: true, ..Self::block_gang_search() }
    }

    /// The default plus converge-aware land drops. The opt-in for
    /// [`converge_lands`](Self::converge_lands); ladder as A against the
    /// default.
    pub const fn converge_lands_on() -> Self {
        Self { converge_lands: true, ..Self::block_gang_search() }
    }

    /// The default plus desperation chump blocks. The opt-in for
    /// [`chump_blocks`](Self::chump_blocks); ladder as A against the
    /// default.
    pub const fn chump_blocks_on() -> Self {
        Self { chump_blocks: true, ..Self::block_gang_search() }
    }

    /// The default plus value-ordered combat damage. The opt-in for
    /// [`damage_order`](Self::damage_order); ladder as A against the
    /// default (profile `dmgorder` vs `gang`).
    pub const fn damage_order_on() -> Self {
        Self { damage_order: true, ..Self::block_gang_search() }
    }

    /// The attack chain (round 55) on the `gang` base: ladder `atk-chain`
    /// as A against `gang`. See [`attack_chain`](Self::attack_chain).
    pub const fn attack_chain_on() -> Self {
        Self { attack_chain: 6, ..Self::block_gang_search() }
    }

    /// The wide attack chain (round 56) on the round-55 default: ladder
    /// `atk-chain-wide` as A against `dflt55`.
    pub const fn attack_chain_wide_on() -> Self {
        Self { attack_chain_wide: true, ..Self::round55_default() }
    }

    /// The block chain (round 56) on the round-55 default: ladder
    /// `blk-chain` as A against `dflt55`.
    pub const fn block_chain_on() -> Self {
        Self { block_chain: 4, ..Self::round55_default() }
    }

    /// The default plus outcome-judged mid-resolution targets. The
    /// opt-in for [`target_eval`](Self::target_eval); ladder as A
    /// against the default (profile `targeteval` vs `gang`).
    pub const fn target_eval_on() -> Self {
        Self { target_eval: true, ..Self::block_gang_search() }
    }

    /// The client pilot plus the saturation fallback. The opt-in for
    /// [`net_tail_guard`](Self::net_tail_guard); ladder as A against
    /// `net-det1` (the same weights minus the flag).
    pub const fn net_tail_guard_on() -> Self {
        Self { net_tail_guard: true, ..Self::net_eval_det1() }
    }

    /// The attack chain under the net pilot: ladder `net-chain` as A
    /// against `net-det1`. See [`attack_chain`](Self::attack_chain).
    pub const fn net_attack_chain_on() -> Self {
        Self { attack_chain: 6, ..Self::net_eval_det1() }
    }

    /// The wide attack chain under the net pilot: ladder `net-chain-wide`
    /// as A against `net-chain`.
    pub const fn net_attack_chain_wide_on() -> Self {
        Self { attack_chain_wide: true, ..Self::net_attack_chain_on() }
    }

    /// The block chain under the net pilot: ladder `net-bchain` as A
    /// against `net-chain`.
    pub const fn net_block_chain_on() -> Self {
        Self { block_chain: 4, ..Self::net_attack_chain_on() }
    }

    /// The client's adopted net pilot, composed from the ladder references
    /// rather than folded into them so `net-det1` / `net-guard` stay the
    /// flagless controls every recorded net number was read against:
    /// [`net_eval_det1`](Self::net_eval_det1) plus the saturation fallback
    /// (round 54, client-adopted on replay evidence) plus the attack chain
    /// (round 55, 51.2 / 51.0 over `net-det1` on seeds 43/97) plus the
    /// block chain (round 56, 57.7 / 55.7 over `net-chain`). Not the wide
    /// attack chain: its net leg straddled 50 (50.2 / 50.7).
    pub const fn client_pilot() -> Self {
        Self { attack_chain: 6, block_chain: 4, ..Self::net_tail_guard_on() }
    }

    /// The attack-search profile plus the walker chip candidate. The
    /// opt-in for [`walker_chip`](Self::walker_chip); ladder as A
    /// against `atk-sim` (the same weights minus the flag).
    pub const fn walker_chip_on() -> Self {
        Self { walker_chip: true, ..Self::attack_search_sim() }
    }

    /// The default plus activated-ability candidates. The opt-in for
    /// [`ability_arms`](Self::ability_arms); ladder as A against the
    /// default.
    pub const fn ability_arms_on() -> Self {
        Self { ability_arms: true, ..Self::block_gang_search() }
    }

    /// [`impulse_draw`](Self::impulse_draw); ladder as A against the
    /// `gang` control (profile `impulse`).
    pub const fn impulse_draw_on() -> Self {
        Self { impulse_draw: true, ..Self::block_gang_search() }
    }

    /// [`mull_sim`](Self::mull_sim); ladder as A against `gang`.
    pub const fn mull_sim_on() -> Self {
        Self { mull_sim: true, ..Self::block_gang_search() }
    }

    /// [`legacy_cashout`](Self::legacy_cashout) — the pre-fix control.
    pub const fn legacy_cashout_on() -> Self {
        Self { legacy_cashout: true, ..Self::block_gang_search() }
    }

    /// [`target_arms`](Self::target_arms). Search-only, so it is gated as
    /// an MCTS profile (`mcts-net-targetarms` vs `mcts-net-deep`) rather
    /// than against a scored control.
    pub const fn target_arms_on() -> Self {
        Self { target_arms: true, ..Self::net_eval_det1() }
    }

    /// [`legacy_fetch`](Self::legacy_fetch) — the pre-fix ranking, as a
    /// heuristic-profile control (`legacyfetch`) against `gang`.
    pub const fn legacy_fetch_on() -> Self {
        Self { legacy_fetch: true, ..Self::block_gang_search() }
    }

    /// [`fetch_arms`](Self::fetch_arms). Search-only, so it gates as an
    /// MCTS profile (`mcts-net-fetcharms` vs `mcts-net-deep`).
    pub const fn fetch_arms_on() -> Self {
        Self { fetch_arms: true, ..Self::net_eval_det1() }
    }

    /// The round-46 reproduction control: `net_eval_det1` with the
    /// adopted [`target_arms`](Self::target_arms) switched back off, so
    /// the gate that adopted it can be re-run against the same baseline
    /// it was measured against (profile `mcts-net-noarms`).
    pub const fn target_arms_off() -> Self {
        Self { target_arms: false, ..Self::net_eval_det1() }
    }

    /// The default, averaging three redeals per candidate. Three times
    /// the simulation cost, and the version that actually approximates
    /// "play against the hands consistent with what I can see" rather
    /// than "play against one specific wrong hand".
    pub const fn determinized3() -> Self {
        Self { determinize: 3, ..Self::block_gang_search() }
    }

    pub const fn net_eval() -> Self {
        // Derives from `block_gang_search` + `chump_blocks`, i.e. the
        // blocking the heuristic bot actually plays, NOT from
        // `attack_search_sim`.
        //
        // It branched off `attack_search_sim` until 2026-08-22, which
        // predates both blocking adoptions, so the net profiles never
        // received value gang-blocks (51.3 % over 28 800 games) or
        // desperation chump blocks (51.0 / 50.8 over 24 000) — the client
        // piloted without them, and every `net` vs `gang` gate since
        // round 26 differed in *two* ways rather than one: evaluator and
        // blocking. Those numbers understate the net.
        //
        // `determinize` is deliberately NOT inherited from `Default`: the
        // `net` vs `net-det1` ladder is the measurement of what reading
        // the opponent's hand was worth, and folding the redeal in here
        // would collapse that pair. `net-preblocks` keeps the old shape
        // for the gate.
        Self {
            net_slot: super::net_eval::SLOT_BEST,
            chump_blocks: true,
            ..Self::block_gang_search()
        }
    }

    /// [`net_eval`](Self::net_eval) as it stood before 2026-08-22 —
    /// branched off `attack_search_sim`, so without either adopted
    /// blocking layer. The control for the gate that changed it
    /// (profiles `net-preblocks` / `mcts-net-preblocks`), and the shape
    /// every net gate from round 26 to 46 was measured in.
    pub const fn net_eval_preblocks() -> Self {
        Self { net_slot: super::net_eval::SLOT_BEST, ..Self::attack_search_sim() }
    }

    /// [`net_eval`](Self::net_eval) with honest search: hidden zones are
    /// redealt before every sim and planner dry-run, so the champion's
    /// searches stop reading the opponent's hand and the real library
    /// order. The `net` vs `net-det1` ladder is the asymmetric
    /// measurement of what the peek was worth (mirrors can't see it —
    /// both seats cheat identically).
    pub const fn net_eval_det1() -> Self {
        // `target_arms` ADOPTED 2026-08-22 (round 46, +0.95 over two
        // ladder seeds, 24 000 games). It lives here rather than in
        // `Default` because it is a *search* flag — only
        // `main_phase_candidates_for_mcts` reads it, so it is inert for
        // every `Pilot::Scored` control — and because this is the exact
        // base the gate measured on top of, and the profile the client's
        // `local_bot` pilots with. `mcts-net-noarms` is the reproduction
        // control.
        Self { determinize: 1, target_arms: true, ..Self::net_eval() }
    }

    /// [`net_eval_det1`](Self::net_eval_det1) averaging three redeals per
    /// combat sim — the lower-variance honest search, at 3× sim cost.
    pub const fn net_eval_det3() -> Self {
        Self { determinize: 3, ..Self::net_eval() }
    }

    /// [`net_eval`](Self::net_eval), blended instead of replaced: the
    /// heuristic evaluation plus a ±50·unit net bias. The division of
    /// labor: the heuristic resolves small material deltas exactly, the
    /// net tilts close calls it has an opinion about.
    ///
    /// **Measured four times**: 49.3 % [46.5, 52.2] (round-1 net),
    /// 49.2 % [46.4, 52.1] (round-2 net, 4× the data), 50.7 %
    /// [47.9, 53.6] (round 2 after its over-reused tail), 48.8 %
    /// [45.9, 51.6] (round-4 capacity net, undertrained) over 1 200
    /// sealed-mirror games each vs `atk-sim` — stable statistical parity
    /// while the same nets score 42–45 % as full replacements. The
    /// stability says the ±50-unit bias is mostly inert (the net's
    /// probability hovers near 0.5 in balanced positions, so the bias
    /// rarely clears a decision margin) — hence the louder
    /// [`net_eval_blend300`](Self::net_eval_blend300). The tail
    /// comparison also priced tail over-reuse: loss EMA fell 0.30 → 0.14
    /// during it with no strength change — pure window memorization,
    /// which is why the trainer now caps the tail.
    pub const fn net_eval_blend() -> Self {
        Self { net_blend_scale: 100, ..Self::net_eval() }
    }

    /// [`net_eval_blend`](Self::net_eval_blend) at 3× loudness — full
    /// confidence worth ±150 units, enough to outvote a mid-size body.
    /// Exists because the 100-scale blend measured as inert; where the
    /// right loudness lies is a ladder question, not an argument.
    ///
    /// **Measured, and the answer is "quieter"**: 45.9 % [43.1 %, 48.7 %]
    /// over 1 200 sealed-mirror games with the round-3 net (47.1 %
    /// [44.3, 49.9] with round 4's), vs 49.3 % / 48.8 % for the
    /// 100-scale blend on the same weights. Amplifying the net's
    /// opinion hurts — where it disagrees with the heuristic it is wrong
    /// more often than right, which bounds what any loudness of this
    /// net's bias can contribute.
    pub const fn net_eval_blend300() -> Self {
        Self { net_blend_scale: 300, ..Self::net_eval() }
    }

    /// [`net_eval_blend300`](Self::net_eval_blend300) with the phase
    /// taper: the net speaks loudly in the opening it wins and goes
    /// silent in the endgame it doesn't. Gate against `gang` for
    /// adoption and against `net-blend300` to isolate the taper.
    pub const fn net_eval_blend_ply() -> Self {
        Self { net_blend_ply: true, ..Self::net_eval_blend300() }
    }

    /// The adopted default plus the searched block assignment.
    ///
    /// **Measured, and not adopted**: 50.4 % [49.8 %, 51.0 %] over 30 000
    /// games across all twelve archetypes — the interval straddles 50 %.
    ///
    /// Kept because the negative result is worth more than the code. Two
    /// earlier runs each read about +0.8 (50.9 % over 8 000 fixed-deck
    /// games, 50.7 % over 15 998 cube games), with a dramatic per-deck
    /// split: −3.2 in the mono-red mirror against +6.7 in golgari. Pooling
    /// those two runs would have cleared 50 % and looked like an adoption.
    /// The 30 000-game run was pre-registered as the decision instead, and
    /// the effect halved to +0.4 while the split mostly evaporated —
    /// mono-red came back to 49.4 %. Only golgari survived, at +4.1.
    ///
    /// So the interesting finding is methodological: at 2 000 games per
    /// archetype this ladder produces per-deck swings of five points that
    /// are not there at 2 500. A per-archetype number is roughly a tenth of
    /// the total sample and should be read as a hint about *where* to look,
    /// never as a result on its own.
    ///
    /// Why it might genuinely not help: [`attack_search`](Self::attack_search)
    /// already delivers about twice the untapped board to `DeclareBlockers`,
    /// and the greedy assignment in [`pick_blocks_inner`] is a far more
    /// developed heuristic than `pick_attacks` ever was — it already folds in
    /// first strike, deathtouch, trample, protection, indestructible,
    /// rampage, planeswalker defense and poison. There was much less room
    /// above it than there was above the alpha strike.
    ///
    /// Re-measured on the SOS college mirrors (where the probes show the
    /// default profile leaving 72-78 % of attackers unblocked in the
    /// spell-heavy colleges): 50.1 % [47.0 %, 53.2 %] against `atk` over
    /// 1000 games — neutral there too. The under-block is not an
    /// assignment problem — the blockers are TAPPED (41-42 % of creatures
    /// at DeclareBlockers, vs 27 % in the healthy Witherbloom row), which
    /// points back at the over-attack, not at the block search; see
    /// [`attack_search_sim`].
    pub const fn block_search() -> Self {
        Self { block_search: 6, ..Self::attack_search() }
    }

    /// Searched attacks with life priced on the concave curve.
    ///
    /// The hypothesis this exists to test. `eval_material` prices a
    /// permanent at `3 * (cmc + power + toughness)` but life at one point
    /// per life, so a Grizzly Bears is worth 18 and the 2 damage it deals is
    /// worth 2 — the attack search has to see a 9:1 payoff before swinging
    /// beats staying home. In an aggro mirror that is roughly right, because
    /// the body you keep really does block something. In the dimir control
    /// mirror it is fatal, and that is exactly where
    /// [`attack_search`](Self::attack_search) loses 5.2 points: damage is the
    /// only win route there, and the evaluator has priced it at a ninth of
    /// its worth.
    ///
    /// [`concave_life`](Self::concave_life) is the existing knob for this —
    /// it prices life steeply near zero and flatly near twenty, which is
    /// what makes "this is the race I need to win" visible to a search whose
    /// horizon is one turn cycle.
    pub const fn attack_search_life() -> Self {
        Self { concave_life: true, ..Self::attack_search() }
    }

    /// The adopted default with the concave life curve and *no* attack
    /// search — the control for [`attack_search_life`](Self::attack_search_life).
    /// Without it a win by that profile can't be attributed: the curve might
    /// simply be better everywhere rather than specifically correcting the
    /// search's bias.
    pub const fn hold_sick_combat_life() -> Self {
        Self { concave_life: true, ..Self::hold_sick_combat() }
    }

    /// Searched attacks with the candidate set cut to the two extremes —
    /// the greedy alpha strike and no attack at all. If the cheap version
    /// captures most of the gain, the per-attacker drops aren't paying for
    /// their simulations and the search can stay nearly free.
    pub const fn attack_search_cheap() -> Self {
        Self { attack_search: 1, ..Self::hold_sick_combat() }
    }

    /// Everything the planner work produced: hold summoning-sick bodies,
    /// hold instant-speed lines that do nothing yet, and evaluate both
    /// through this turn's combat so the gate can tell the difference.
    pub const fn planner() -> Self {
        Self { hold_sick: true, hold_instants: true, combat_aware: true, ..Self::baseline() }
    }

    /// Baseline + the combat-aware evaluation.
    ///
    /// **Measured exactly neutral**: 50.0 % [49.1 %, 50.9 %] over 12 000
    /// games (6002-5998). Not adopted as the default, but kept and worth
    /// revisiting, because the reason it does nothing here is structural
    /// rather than a flaw in the simulation: within a single precombat
    /// main phase, *this turn's combat is very nearly the same whichever
    /// candidate the bot picks*, so the term is shared between candidates
    /// and cancels in the comparison. It only starts to pay when the bot
    /// can also choose *when* to act — which is what a turn planner adds
    /// (Forge's `formulatePlanWithPhase(COMBAT_DECLARE_BLOCKERS)` and its
    /// summon-sick gating both consume exactly this signal).
    pub const fn combat_aware() -> Self {
        Self { combat_aware: true, ..Self::baseline() }
    }

    /// [`v2`](Self::v2) weights *plus* the combat-aware evaluation — the
    /// direct test of why v2 lost. The hypothesis was depth: that a richer
    /// evaluation only pays once the evaluator can see past the current
    /// action. This is the cheapest available "more depth".
    ///
    /// **The hypothesis is refuted.** This measures 53.1 % to the baseline
    /// — worse than [`v2`](Self::v2) alone at 51.1 % — so the extra depth
    /// does not rescue the ported weights, it compounds them.
    pub const fn v2_combat() -> Self {
        Self { combat_aware: true, ..Self::v2() }
    }

    /// The historical mana behavior, for laddering the tap-out fix.
    /// See [`legacy_pretap`](Self::legacy_pretap).
    pub const fn legacy_mana() -> Self {
        Self { legacy_pretap: true, ..Self::baseline() }
    }

    /// Scaled control + the keyword term only. **Measured worse than the
    /// baseline**: 51.1 % to the baseline over 20 000 pooled games
    /// ([50.4 %, 51.8 %]). See [`v2`](Self::v2) for why it is kept.
    pub const fn keywords_only() -> Self {
        Self { keyword_pct: 100, ..Self::scaled_control() }
    }

    /// Scaled control + a quarter-strength keyword term. Separates
    /// "keywords are weighted too heavily" from "keyword scoring is the
    /// wrong thing to feed this bot at all": if even a gentle version
    /// loses, the problem is directional, not a magnitude to tune.
    /// Measured neutral (50.1 % to the baseline).
    pub const fn keywords_quarter() -> Self {
        Self { keyword_pct: 25, ..Self::scaled_control() }
    }

    /// Scaled control + Forge's flat creature base only.
    ///
    /// This was the hypothesis for why the keyword port lost -- Forge's
    /// keyword magnitudes are calibrated against a body term that opens at
    /// a flat 100, where this evaluator opens at zero, so the same bonuses
    /// land proportionally ~2.7x louder here. Adding the constant did not
    /// help ([`base_and_keywords`] measured 52.4 % *to the baseline*, worse
    /// than keywords alone), because the constant also shifts the
    /// creature-to-card ratio from 4.5:1 to 12:1. Forge runs that ratio at
    /// roughly 32:1; matching one term without the surrounding balance
    /// moves everything.
    ///
    /// [`base_and_keywords`]: Self::base_and_keywords
    pub const fn creature_base_only() -> Self {
        Self { creature_base: 100, ..Self::scaled_control() }
    }

    /// The creature base plus keywords. **Measured worst of the lot**:
    /// 52.4 % to the baseline. Retained as the record of a tested and
    /// rejected hypothesis -- see [`creature_base_only`].
    ///
    /// [`creature_base_only`]: Self::creature_base_only
    pub const fn base_and_keywords() -> Self {
        Self { keyword_pct: 100, ..Self::creature_base_only() }
    }

    /// Scaled control + the concave life curve only. Measured neutral
    /// (51.1 % to the baseline, CI straddling 50 %).
    pub const fn life_only() -> Self {
        Self { concave_life: true, ..Self::scaled_control() }
    }

    /// Scaled control + Forge's power-over-toughness emphasis only.
    /// Measured neutral (50.4 % to the baseline, CI straddling 50 %).
    pub const fn power_emphasis_only() -> Self {
        Self { power: 15, ..Self::scaled_control() }
    }
}

impl Default for EvalWeights {
    /// The adopted profile. [`baseline`](EvalWeights::baseline) stays the
    /// historical *control* — it is what ladder runs measure against — but
    /// it is no longer what the bot plays. Each layer had to beat the
    /// previous one on the ladder before it was added:
    ///
    /// | layer | vs. the one before | games |
    /// |---|---|---|
    /// | summon-sick gate | 51.5 % [50.8 %, 52.3 %] | 16 000 |
    /// | combat-aware evaluation | 51.3 % [50.4 %, 52.2 %] | 12 000 |
    /// | searched attacks | 52.4 % [51.3 %, 53.5 %] | 8 000 |
    /// | spell-casting combat sims | 54.4 % [53.0 %, 55.8 %] | 4 794 |
    /// | value gang-blocks | 51.3 % [50.7 %, 51.9 %] | 28 800 |
    /// | desperation chump blocks | 51.0 % / 50.8 % (two ladder seeds) | 24 000 |
    /// | value-ordered combat damage | 50.4 % / 50.3 % (two ladder seeds, both intervals clear 50) | 24 000 |
    ///
    /// One adopted layer is *not* in this table because it is not in this
    /// profile: `target_arms` (round 46, 50.7 % / 51.2 %, 24 000 games)
    /// is a search-only flag and lives on
    /// [`net_eval_det1`](Self::net_eval_det1), the base its gate measured
    /// against and the one the client pilots with. Nothing here reads it —
    /// `main_phase_candidates_for_mcts` is the only consumer — so adding
    /// it to this table would advertise a change the heuristic bot does
    /// not make.
    ///
    /// The attack search was additionally confirmed on cube decks —
    /// 53.8 % [53.0 %, 54.6 %] over 13 695 decided games — where the fixed
    /// four archetypes could not have shown its deck-dependence. The
    /// spell-casting sims (see [`attack_search_sim`](Self::attack_search_sim))
    /// were adopted on the fixed+cube row above plus two 1000-game SOS
    /// college runs (51.7 % / 53.2 %, the second clearing 50 % alone),
    /// against an identical-profile control at 48.9 % — and the largest
    /// per-deck gain landed exactly where the blind attack search had its
    /// documented regression: dimir control, 44.8 % blind → 61.3 % seeing.
    /// Gang-blocking (see [`block_gang_search`](Self::block_gang_search))
    /// was adopted last, on two independent 28 800-game sealed runs —
    /// 51.3 % [50.7 %, 51.9 %] and 51.1 % [50.5 %, 51.7 %]. It is the
    /// only one of five play-side heuristics tried in that push to
    /// survive its decider, and the difference is instructive: the
    /// other four refined decisions the bot already made competently,
    /// while this one added a line it could not previously express at
    /// all (the greedy pass gangs only under lethal threat, and
    /// [`block_search`](Self::block_search) could only ever *remove*
    /// blockers). Note that adopting it also turns `block_search` on,
    /// which measured null by itself — the search was never the
    /// problem; it had nothing worth searching.
    /// Determinized search was adopted 2026-08-09 (task #25): the
    /// default bot no longer reads hidden zones in its sims and planner
    /// dry-runs. Priced by asymmetric gates — the peek was worth ~1.4
    /// pts to this profile (`det1` vs `gang` 49.3 %/48.0 %) and ~0.5 to
    /// the net champion — and those cells faced *still-cheating*
    /// opponents, so they overstate the cost against honest opposition
    /// (a human). Ladder control profiles keep `determinize: 0`
    /// explicitly; measurement tools built on this default (deck_duel,
    /// recommend_pool sims) are honest from this date and their numbers
    /// are not margin-comparable to earlier cheating-sim runs.
    fn default() -> Self {
        // `chump_blocks` adopted 2026-08-19 (round 43): 51.0 / 50.8 on
        // two ladder seeds against `gang` over 24 000 mirror sealed
        // games, both pool-level intervals clear of 50. Found by the
        // decision-log/replay review of recorded human games (the bot at
        // 5 life took 4 to the face holding a chump, because the
        // profitable-blocks-only menu never offered one). Measured on
        // the `gang` base; determinization rides on top exactly as the
        // adoption table's earlier layers do.
        // `damage_order` adopted 2026-08-30 (round 52): 50.4 % [50.2,
        // 50.6] and 50.3 % [50.1, 50.5] against `gang` on ladder seeds
        // 43/97, 24 000 paired sealed games total, both cell intervals
        // clear of 50 (the round-50 rare-class rule — 5 817 of 6 000
        // pairs split as exact mirrors, so the ~0.12-per-game decision
        // measures at ±0.2). Found by the state/decision representation
        // audit: `Decision::CombatDamageOrder` had no policy arm, so
        // every gang block in the program's history dealt lethal in
        // declaration order. Measured on the `gang` base like the layers
        // above; the exact gate re-runs as `dmgorder` vs `gang`
        // (.ladder/run_r52_dmgorder.sh).
        // `attack_chain` adopted 2026-09-04 (round 55): 53.1 / 51.7 /
        // 52.1 / 52.3 % against `gang` on ladder seeds 43/97/151/199,
        // 48 000 paired sealed games, every cell's interval clear of
        // 51.4 — pooled +2.3, the largest menu-hole reading on record
        // (gang blocks +1.3, chump blocks +0.9). Replicated under the net
        // pilot at 51.2 / 51.0 (`net-chain` vs `net-det1`). The holdback
        // menu could only drop attackers; growing a declaration from
        // nobody one sim-priced creature at a time reaches the smaller
        // sets and the greedy-refused bodies it never could (a new set
        // in 9.7 % of searched declarations, winning in 6.0 %). Cost:
        // sealed mirror wall-clock 4.7 s -> 6.6 s per 12 000 games
        // (+40 %) with the chain on both seats. Measured on the `gang`
        // base like the layers above; the exact gate re-runs as
        // `atk-chain` vs `gang` (.ladder/run_r55_atkchain.sh).
        Self::default_const()
    }
}

impl EvalWeights {
    /// The default as it stood after round 55 (determinize, chump blocks,
    /// damage order, the attack chain), frozen: the base the round-56
    /// gates were read on (`dflt55`), kept so adoption does not consume
    /// its own control — the r52 precedent.
    pub const fn round55_default() -> Self {
        Self {
            determinize: 1,
            chump_blocks: true,
            damage_order: true,
            attack_chain: 6,
            ..Self::block_gang_search()
        }
    }

    /// [`Default::default`] as a `const fn`, so a profile can be built on
    /// the adopted default at compile time; the two must stay identical.
    ///
    /// `block_chain` and `attack_chain_wide` adopted 2026-09-05 (round
    /// 56, `.ladder/run_r56_chains.sh`, 12 000 paired sealed games a
    /// cell): the block chain read **56.8 / 55.0 / 55.6 / 55.7** against
    /// the r55 default on seeds 43/97/151/199 (pooled +5.8; 57.7 / 55.7
    /// under the net; 59.0 vs `gang`, 60.3 vs `atk-sim`, and 54.5 / 54.5
    /// on `--decks cube`), every interval clear of 54.6 — the block menu
    /// had been bare "no blocks" whenever greedy found nothing profitable,
    /// and never generated a gang there. The wide attack chain read 50.4
    /// / 50.7 / 50.4 / 50.4 with every cell's interval clear of 50 (the
    /// r50 replicated-small rule) and 50.2 / 50.7 under the net, one cell
    /// straddling — adopted here, not in [`client_pilot`](Self::client_pilot).
    ///
    /// `attack_pairs_empty_only` and `attack_pairs_lazy` adopted 2026-09-05
    /// (round 58, `.ladder/run_r58_pairs.sh`, a no-loss throughput gate):
    /// `pairs-both` read 50.1 / 50.1 / 50.0 / 50.2 against the round-56
    /// default on seeds 43/97/151/199 (pooled +0.10, every interval
    /// touching 50; `pairs-empty` 50.08, `pairs-lazy` 50.00) for -14.9 %
    /// of the sealed mirror's wall clock — the pair move beside a
    /// non-empty greedy declaration, and ahead of a single that wins, was
    /// buying nothing.
    ///
    /// `attack_skip_open` adopted 2026-09-05 (round 60,
    /// `.ladder/run_r60_open.sh`, a no-loss throughput gate on the round-58
    /// default): 50.0 / 50.0 / 50.0 / 49.9 on seeds 43/97/151/199 (pooled
    /// -0.02, every cell within ±0.08 and touching 50) for -4.1 % of the
    /// sealed mirror's wall clock — greedy won 1,814 of the 1,824
    /// creatureless-defender searches the sim priced. The `gang`-base
    /// reading at `e725e5c2` (-0.1 on 96 k games) was the same flag on a
    /// search a third the size. Default only: the client pilot is built
    /// on `block_gang_search` and keeps the sim's hold-backs.
    pub const fn default_const() -> Self {
        Self {
            attack_pairs_empty_only: true,
            attack_pairs_lazy: true,
            attack_skip_open: true,
            ..Self::round56_default()
        }
    }

    /// The default as it stood after round 58 (the round-56 default plus
    /// the pair-move restrictions), frozen as the base round 59 and round
    /// 60 read on (`dflt58`).
    pub const fn round58_default() -> Self {
        Self { attack_pairs_empty_only: true, attack_pairs_lazy: true, ..Self::round56_default() }
    }

    /// The default as it stood after round 56 (the round-55 default plus
    /// the wide attack chain and the block chain), frozen as the base the
    /// pair-move throughput gates read on (`dflt56`).
    pub const fn round56_default() -> Self {
        Self { attack_chain_wide: true, block_chain: 4, ..Self::round55_default() }
    }

    /// The wide chain's pair move on the empty-greedy board only: ladder
    /// `pairs-empty` as A against `dflt56`, gated for no loss. See
    /// [`attack_pairs_empty_only`](Self::attack_pairs_empty_only).
    pub const fn attack_pairs_empty_only_on() -> Self {
        Self { attack_pairs_empty_only: true, ..Self::round56_default() }
    }

    /// The wide chain's pair move only after the singles tie: ladder
    /// `pairs-lazy` as A against `dflt56`, gated for no loss. See
    /// [`attack_pairs_lazy`](Self::attack_pairs_lazy).
    pub const fn attack_pairs_lazy_on() -> Self {
        Self { attack_pairs_lazy: true, ..Self::round56_default() }
    }

    /// Both pair-move restrictions: ladder `pairs-both` as A against
    /// `dflt56`.
    pub const fn attack_pairs_both_on() -> Self {
        Self { attack_pairs_empty_only: true, attack_pairs_lazy: true, ..Self::round56_default() }
    }

    /// The empty-greedy blocker gate on the round-58 default: ladder
    /// `empty-gate` as A against `dflt58`, gated for no loss. See
    /// [`attack_empty_gate`](Self::attack_empty_gate).
    pub const fn attack_empty_gate_on() -> Self {
        Self { attack_empty_gate: true, ..Self::round58_default() }
    }

    /// The holdback menu capped at three on the round-60 default (round
    /// 61): ladder `dflt-as3` as A against `dflt`, gated for no loss. See
    /// [`attack_search`](Self::attack_search) — the default's cap is 6.
    pub const fn attack_search_default3() -> Self {
        Self { attack_search: 3, ..Self::default_const() }
    }

    /// The open-board shortcut on the round-58 default (round 60): ladder
    /// `dflt-open` as A against `dflt58`, gated for no loss. See
    /// [`attack_skip_open`](Self::attack_skip_open); `atk-open` is the same
    /// flag on the `gang` base, the round-50-era reading.
    pub const fn attack_skip_open_default() -> Self {
        Self { attack_skip_open: true, ..Self::round58_default() }
    }
}

/// The engine's heuristic player, and the one every real run uses: the
/// client's opponent, the ladder's profiles, and every self-play training
/// actor. Was called `RandomBot` until the name outlived the behaviour —
/// only [`uniform_baseline`](Self::uniform_baseline) is random now, and it
/// exists solely as the ladder's control arm.
///
/// Main phase: enumerates castable candidates and ranks them by
/// [`score_candidate`]. Combat: it attacks with creatures that swing safely
/// or profitably (evasion / first-strike / deathtouch / menace / lifelink /
/// trample / indestructible awareness, plus a suicide filter and
/// planeswalker redirection) and assigns blockers to maximize value trades
/// and survive lethal (see `pick_attacks_scored` / `pick_blocks_scored`).
/// How it values a board is [`EvalWeights`], which is what a ladder profile
/// selects. Decisions are auto-answered with [`AutoDecider`].
///
/// The bot keeps a little internal flag state so it only submits
/// `DeclareAttackers`/`DeclareBlockers` once per combat phase — the match
/// actor polls it repeatedly, so without these flags it would re-submit every
/// tick.
pub struct HeuristicBot {
    last_step_key: Option<(u32, TurnStep, usize)>,
    attackers_declared: bool,
    blocks_declared: bool,
    /// `true` (the default) ranks castable candidates via
    /// [`score_candidate`]; `false` keeps the legacy uniform-random pick.
    /// The baseline exists so bot changes can be A/B-laddered against the
    /// previous behavior.
    scored: bool,
    /// Ad Nauseam-style reveal series: the asks all happen before any life
    /// is lost, so the bot tracks what it has already committed to across
    /// consecutive prompts from the same source. `(source, cards, life)`.
    reveal_commit: Option<(CardId, usize, i32)>,
    /// How this bot values the board. Ladder-selectable -- see
    /// [`EvalWeights`].
    weights: EvalWeights,
}

impl HeuristicBot {
    pub fn new() -> Self {
        Self {
            last_step_key: None,
            attackers_declared: false,
            blocks_declared: false,
            scored: true,
            reveal_commit: None,
            weights: EvalWeights::default(),
        }
    }

    /// The scored bot piloted with a specific evaluation profile.
    pub fn with_weights(weights: EvalWeights) -> Self {
        Self { weights, ..Self::new() }
    }

    /// The pre-scoring reference bot: identical candidate enumeration and
    /// combat, but the castable pick is uniform-random. Kept as the ladder
    /// baseline for measuring bot improvements.
    pub fn uniform_baseline() -> Self {
        Self { scored: false, ..Self::new() }
    }

    fn sync_step(&mut self, state: &GameState) {
        let key = (state.turn_number, state.step, state.active_player_idx);
        if self.last_step_key != Some(key) {
            self.last_step_key = Some(key);
            self.attackers_declared = false;
            self.blocks_declared = false;
        }
    }

    /// Is this combat step's declaration still to be made? For the Monte
    /// Carlo layer, which searches the declaration itself but leaves
    /// every other tick of the phase (tricks, removal, passes) to this
    /// bot — the two share one latch or they double-declare.
    pub(crate) fn declaration_pending(&mut self, state: &GameState, attacks: bool) -> bool {
        self.sync_step(state);
        if attacks { !self.attackers_declared } else { !self.blocks_declared }
    }

    /// Record that the declaration for the current step was made on this
    /// bot's behalf, so its own combat arms pass priority instead of
    /// re-declaring.
    pub(crate) fn note_external_declaration(&mut self, state: &GameState, attacks: bool) {
        self.sync_step(state);
        if attacks {
            self.attackers_declared = true;
        } else {
            self.blocks_declared = true;
        }
    }
}

impl Default for HeuristicBot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot for HeuristicBot {
    /// The whole tick runs inside one `with_frozen_layers` scope. Sound by
    /// construction — a bot only ever receives `&GameState`, so nothing it
    /// does here can invalidate the gathered continuous-effect set — and it
    /// turns a bot tick's many `computed_permanent` reads into one gather
    /// instead of one gather each. The scope does NOT reach the dry-run
    /// probes and combat sims: those clone the state, and `LayerFreeze`
    /// clones as unfrozen precisely because the clone gets mutated.
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        self.next_action_settled(state, seat).map(BotStep::into_action)
    }

    /// Return the finalist's dry-run settled state to the driver, so a
    /// self-play caller adopts it instead of paying for a second execution
    /// of the same action (see [`Bot::next_action_settled`]).
    fn next_action_settled(&mut self, state: &GameState, seat: usize) -> Option<BotStep> {
        state.with_frozen_layers(|state| self.next_action_inner(state, seat))
    }
}

impl HeuristicBot {
    fn next_action_inner(&mut self, state: &GameState, seat: usize) -> Option<BotStep> {
        if state.is_game_over() {
            return None;
        }
        self.sync_step(state);

        // Any pending decision addressed to us: auto-answer it.
        if let Some(pending) = &state.pending_decision {
            if pending.acting_player() == seat {
                // Ad Nauseam's per-reveal prompt is the one STATEFUL
                // policy (it tracks committed reveals on the bot struct
                // across the series — see `RevealTopToHandLoseLifeRepeat`),
                // so it answers here; every other decision goes through
                // [`decide_pending_policy`], the same table simulations
                // use.
                if let crate::decision::Decision::OptionalTrigger { source, description } =
                    &pending.decision
                    && description.starts_with("Reveal the top card (")
                {
                    let (cards, life_committed) = match &self.reveal_commit {
                        Some((s, c, l)) if *s == *source => (*c, *l),
                        _ => (0, 0),
                    };
                    let mv = state.players[seat]
                        .library
                        .get(cards)
                        .map(|c| c.definition.cost.cmc() as i32)
                        .unwrap_or(0);
                    let yes = state.effective_life(seat) - life_committed - mv > 10;
                    self.reveal_commit =
                        if yes { Some((*source, cards + 1, life_committed + mv)) } else { None };
                    return Some(BotStep::plain(GameAction::SubmitDecision(
                        crate::decision::DecisionAnswer::Bool(yes),
                    )));
                }
                let answer =
                    decide_pending_policy(state, seat, &self.weights, &pending.decision, true);
                return Some(BotStep::plain(GameAction::SubmitDecision(answer)));
            }
            return None;
        }

        if state.player_with_priority() != seat {
            return None;
        }

        let is_active = state.active_player_idx == seat;

        match state.step {
            TurnStep::DeclareBlockers if state.may_declare_blocks(seat) => {
                if !self.blocks_declared && !state.attacking().is_empty() {
                    // Kill the biggest attacker BEFORE committing blocks —
                    // removal cast here shrinks the combat the blocks then
                    // answer. Validated actions only, so a resolved kill
                    // falls through to the block declaration next tick.
                    if !is_active
                        && let Some(a) = pick_defensive_removal(state, seat, &self.weights)
                    {
                        return Some(BotStep::plain(a));
                    }
                    self.blocks_declared = true;
                    // On our own turn we're choosing the *defender's* blocks
                    // (Master Warcraft, Invasion Plans), so submit only what
                    // CR 509.1c forces — and aim each forced blocker at the
                    // attacker most likely to kill it.
                    let blocks = if is_active {
                        forced_blocks(state)
                    } else {
                        pick_blocks_scored(state, seat, &self.weights)
                    };
                    Some(BotStep::plain(GameAction::DeclareBlockers(blocks)))
                } else if state.blockers_declared() && state.stack.is_empty() {
                    // Post-block priority: a held pump trick that flips a
                    // fight one of our blockers is losing. `pick_combat_trick`
                    // dry-runs the trick to gate it, so `Probed` carries the
                    // state the driver would otherwise re-run.
                    Some(
                        pick_combat_trick(state, seat, &self.weights)
                            .map(Picked::into_step)
                            .unwrap_or_else(|| BotStep::plain(GameAction::PassPriority)),
                    )
                } else {
                    Some(BotStep::plain(GameAction::PassPriority))
                }
            }
            // Active side of the same window: blocks are in, stack is
            // empty — the classic trick timing for a blocked attacker.
            TurnStep::DeclareBlockers
                if is_active && state.blockers_declared() && state.stack.is_empty() =>
            {
                Some(
                    pick_combat_trick(state, seat, &self.weights)
                        .map(Picked::into_step)
                        .unwrap_or_else(|| BotStep::plain(GameAction::PassPriority)),
                )
            }
            // Master Warcraft on an opponent's turn: we choose *their*
            // attackers, so declare only the creatures that must attack.
            TurnStep::DeclareAttackers if !is_active && state.attack_declarer() == seat => {
                if !self.attackers_declared {
                    self.attackers_declared = true;
                    Some(BotStep::plain(GameAction::DeclareAttackers(forced_attacks(state))))
                } else {
                    Some(BotStep::plain(GameAction::PassPriority))
                }
            }
            TurnStep::DeclareAttackers if is_active && state.attack_declarer() == seat => {
                if !self.attackers_declared {
                    self.attackers_declared = true;
                    Some(BotStep::plain(GameAction::DeclareAttackers(pick_attacks_scored(
                        state,
                        seat,
                        &self.weights,
                    ))))
                } else {
                    Some(BotStep::plain(GameAction::PassPriority))
                }
            }
            TurnStep::PreCombatMain | TurnStep::PostCombatMain if is_active => {
                // A non-empty stack in our own main is response timing,
                // not sorcery timing: an opponent's answer is resolving,
                // and the response layer sees threats the main-phase
                // enumerator can't (counter it, fire a prepared inset
                // spell before its body dies). Both pickers ignore the
                // bot's own spells-in-flight, so a stack we put there
                // falls through to the enumerator as before.
                // `pick_stack_response` is the one branch that dry-runs its
                // pick, so its `Probed` state is what the driver would
                // re-run. The remaining branches (`pick_ability_counter_response`,
                // `pick_prepare_response`, `pick_buff_response`) return only
                // an action, so their outputs stay `Plain`. Try the settled
                // path first; fall through to the plain chain otherwise.
                if !state.stack.is_empty() {
                    if let Some(picked) = pick_stack_response(state, seat, &self.weights) {
                        return Some(picked.into_step());
                    }
                    if let Some(a) = pick_ability_counter_response(state, seat, &self.weights)
                        .or_else(|| pick_prepare_response(state, seat, &self.weights))
                        .or_else(|| pick_buff_response(state, seat, &self.weights))
                    {
                        return Some(BotStep::plain(a));
                    }
                }
                // CR 116.2j — the agenda is already named, so turning it face
                // up is pure upside; do it at the first opportunity.
                if let Some(c) = state.players[seat]
                    .command
                    .iter()
                    .find(|c| c.face_down && c.definition.is_conspiracy())
                {
                    return Some(BotStep::plain(GameAction::RevealConspiracy { card_id: c.id }));
                }
                // CR 901.9 — take the turn's one free planar-die roll before
                // spending mana. Later rolls cost {N} and compete with real
                // plays, so the bot stops after the free one. Inert outside a
                // Planechase game (no planar deck, nothing face up).
                if state.stack.is_empty()
                    && state.players[seat].planar_die_rolls_this_turn == 0
                    && !state.players[seat].planar_deck.is_empty()
                    && !state.face_up_planes().is_empty()
                {
                    return Some(BotStep::plain(GameAction::RollPlanarDie));
                }
                Some(main_phase_action_with(state, seat, self.scored, &self.weights))
            }
            // Opponent's end step with an empty stack — the bot's canonical
            // off-turn window. Reuse the whole scored main-phase enumeration:
            // `would_accept` filters it down to instant-legal lines (removal,
            // tricks, flash creatures, EOT draw, mana-sink abilities), while
            // land drops, sorcery-speed casts, and loyalty/crew lines simply
            // drop out of the candidate set. Without this, every non-counter
            // instant was dead in hand until the bot's own main phase.
            TurnStep::End if !is_active && state.stack.is_empty() => {
                Some(main_phase_action_with(state, seat, self.scored, &self.weights))
            }
            _ => {
                // Same shape as the main-phase stack window above: only
                // `pick_stack_response` carries a dry-run state; the rest
                // fall back through the `Plain` chain to a plain action.
                if let Some(picked) = pick_stack_response(state, seat, &self.weights) {
                    return Some(picked.into_step());
                }
                let action = pick_ability_counter_response(state, seat, &self.weights)
                    .or_else(|| pick_prepare_response(state, seat, &self.weights))
                    .or_else(|| pick_buff_response(state, seat, &self.weights))
                    // Defender windows in the attack steps (the picker
                    // no-ops unless declared attackers are coming at us).
                    .or_else(|| {
                        if state.stack.is_empty() {
                            pick_defensive_removal(state, seat, &self.weights)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(GameAction::PassPriority);
                Some(BotStep::plain(action))
            }
        }
    }
}

/// The bot's answer to a pending `decision` for `seat` — the policy table
/// behind `next_action`, extracted so SIMULATIONS answer with it too.
/// Every lookahead used to answer internal decisions with `AutoDecider`,
/// which meant a line was scored as if the bot's future self would scry
/// badly, decline its tutors, dump the head of its hand to a discard, and
/// take mode 0 — and the opponent would as well. Now both seats inside a
/// sim play by this table (`pending.acting_player()` picks whose view).
///
/// `eval_modes: false` disables the clone-and-resolve answers — mode
/// picks fall back to mode 0 and self-costly optional triggers to the
/// introspection screen's decline — because inside a sim that recursion
/// would multiply whole-state clones for marginal fidelity, and both
/// fallbacks are the pre-policy floor. The stateful Ad Nauseam reveal
/// family (a running life commitment on the bot struct) is handled by
/// `next_action` before this table; a sim reaching it here declines, the
/// conservative read.
pub(crate) fn decide_pending_policy(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    decision: &crate::decision::Decision,
    eval_modes: bool,
) -> crate::decision::DecisionAnswer {
    // Read-only, and reached once per pending decision inside the attack /
    // block sims on a cloned (unfrozen) state — share one gather.
    state.with_frozen_layers(|s| decide_pending_policy_inner(s, seat, w, decision, eval_modes))
}

fn decide_pending_policy_inner(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    decision: &crate::decision::Decision,
    eval_modes: bool,
) -> crate::decision::DecisionAnswer {
    match decision {
        // Smarter mulligan than AutoDecider's blanket Keep:
        // ship hands that are flooded or screwed on lands.
        crate::decision::Decision::Mulligan { mulligans_taken, .. } => {
            // `eval_modes` off means we are inside somebody else's
            // simulation; a nested mulligan sim there would multiply the
            // cost of every enclosing rollout for a decision that branch
            // barely depends on.
            if w.mull_sim && eval_modes {
                decide_mulligan_by_sim(state, seat, *mulligans_taken, w)
            } else {
                decide_mulligan(state, seat, *mulligans_taken, w)
            }
        }
        // Unlike AutoDecider (which declines every tutor), the
        // bot actually fetches — preferring a basic land toward
        // its weakest color so singleplayer tutors fix mana.
        crate::decision::Decision::SearchLibrary { candidates, eligible, .. } => {
            // Only consider picks the engine will accept.
            let pickable: Vec<(crate::card::CardId, String)> = match eligible {
                Some(ok) => {
                    candidates.iter().filter(|(id, _)| ok.contains(id)).cloned().collect()
                }
                None => candidates.clone(),
            };
            decide_library_search(state, seat, &pickable, w)
        }
        // Unlike AutoDecider (which declines *every* "you may"
        // trigger), the bot takes an optional trigger whose body
        // is pure upside — so Provoke's "you may", Boast token
        // riders, etc. actually fire under bot play. It still
        // declines bodies that impose a self-cost (lose life /
        // sacrifice / discard). Engine-authored prompt families the
        // generic screen can't introspect (no MayDo body) each get a
        // real policy instead of the blanket-yes fallback.
        crate::decision::Decision::OptionalTrigger { source, description } => {
            let take = if description.starts_with("Pay ")
                && description.contains(" life to deny ")
            {
                // Rhystic-style life tax: pay only with a healthy
                // buffer. Parse the printed amount.
                let n: i32 = description
                    .split_whitespace()
                    .nth(1)
                    .and_then(|w| w.parse().ok())
                    .unwrap_or(2);
                state.effective_life(seat) - n > 10
            } else if description.starts_with("Accept the tempting offer") {
                // Tempting offers reward the caster; decline.
                false
            } else if description.starts_with("Pay echo ")
                || description.starts_with("Pay cumulative upkeep ")
                || description.starts_with("Discard a card for ")
            {
                // Pay while the permanent is worth keeping; let
                // cheap chaff die to its own upkeep.
                state
                    .battlefield_find(*source)
                    .map(|c| permanent_value(state, c.id, w) >= 4 * w.unit)
                    .unwrap_or(false)
            } else if description.starts_with("Cast a copy of ") {
                // Paradigm recurrence (SOS): a free copy is pure
                // upside unless the spell's own body drains life
                // the bot can't spare — Decorum Dissertation's
                // draw-2-lose-2 recurs every main phase, and the
                // blanket yes played it straight into the
                // state-based loss.
                let loss = state
                    .exile
                    .iter()
                    .find(|c| c.id == *source)
                    .map(|c| self_life_loss(&c.definition.effect))
                    .unwrap_or(0);
                state.effective_life(seat) - loss > 5
            } else if description.starts_with("Reveal the top card (") {
                // Stateful family (see `next_action`); in a sim, decline.
                false
            } else {
                // Introspection screen first (pure upside → yes, self-cost
                // → no). A "no" from the self-cost rule gets a second
                // opinion by OUTCOME at the real decision (`eval_modes`
                // gates it off inside sims): sacrifice-for-value and
                // pay-for-payoff bodies are exactly the trades a blanket
                // decline can't judge. Strictly-better-or-keep-declining.
                let take = optional_trigger_beneficial(state, *source, description);
                if !take && eval_modes {
                    decide_optional_by_outcome(state, seat, w).unwrap_or(false)
                } else {
                    take
                }
            };
            crate::decision::DecisionAnswer::Bool(take)
        }
        // AutoDecider always names Demon; the bot instead names the
        // creature type it has the most of across its battlefield +
        // hand, so tribal payoffs (Cavern of Souls, Kindred
        // Discovery, Door of Destinies, the chosen-type lords) land
        // on a type it can actually exploit.
        crate::decision::Decision::ChooseCreatureType { suggestions, .. } => {
            decide_creature_type(state, seat, suggestions)
        }
        // AutoDecider chooses nothing; the bot exiles opponents'
        // graveyard cards (deny graveyard value) up to the cap.
        crate::decision::Decision::ChooseCards { prompt, candidates, min, max, .. } => {
            decide_choose_cards(w, state, seat, prompt, candidates, *min, *max)
        }
        // London mulligan bottoming (CR 103.5) and "put N cards
        // from your hand on top/bottom" effects. `AutoDecider`
        // takes the first N cards of the hand, so a bot that
        // mulliganed bottomed whichever cards happened to sit at
        // the front — routinely its business spells. Rank them
        // the same way a discard is ranked: surplus lands first,
        // then the priciest spells.
        crate::decision::Decision::PutOnLibrary { player, count, hand } if *player == seat => {
            let order = hand_worst_first(state, seat, hand);
            crate::decision::DecisionAnswer::PutOnLibrary(
                order.into_iter().take(*count).collect(),
            )
        }
        // A self-discard (cleanup over max hand size, rummaging, a
        // discard cost): every offered card is in our own hand and
        // we're the one choosing. Unlike AutoDecider (which dumps
        // the head of the hand — possibly our best spell), shed the
        // least useful cards. Inquisition-style "choose from an
        // opponent's hand" Discards fail the own-hand guard and
        // fall through to AutoDecider unchanged.
        crate::decision::Decision::Discard { player, count, hand }
            if *player == seat
                && hand
                    .iter()
                    .all(|(id, _)| state.players[seat].hand.iter().any(|c| c.id == *id)) =>
        {
            decide_self_discard(state, seat, hand, *count)
        }
        // AutoDecider blindly picks the first legal target. For
        // votes (Council's Judgment), edicts, and removal the bot
        // should instead hit the opponent's *most* valuable
        // permanent — or, when forced to choose among its own
        // permanents, give up the *least* valuable.
        crate::decision::Decision::ChooseTarget { legal, optional, .. } if !legal.is_empty() => {
            // Round 53: judge the corner candidates by settled outcome at
            // the real decision. Inside a sim (`eval_modes` off) the
            // clone-and-resolve would multiply whole-state clones, so the
            // polarity guess below stands there — the same split every
            // other outcome policy in this table uses.
            let by_outcome = (w.target_eval && eval_modes)
                .then(|| decide_target_by_outcome(state, seat, legal, *optional, w))
                .flatten();
            by_outcome.unwrap_or_else(|| decide_choose_target(state, seat, legal, w))
        }
        // AutoDecider answers every amount with 0, which turns
        // "choose up to X" payoffs into no-ops and (worse) reads
        // as "power ≥ 0" on destroy-cutoff wraths. Default to
        // the max for generic upside prompts; prompt families
        // with a real downside get their own rule.
        crate::decision::Decision::ChooseAmount { prompt, max, .. } => {
            let amount = if prompt.contains("destroy all creatures with power") {
                best_destroy_power_cutoff(state, seat, *max, w)
            } else if prompt.to_lowercase().contains("life") {
                // Life payments: keep a buffer, never sink deep.
                let spare = (state.effective_life(seat) - 10).max(0) as u32;
                spare.min(*max).min(3)
            } else {
                *max
            };
            crate::decision::DecisionAnswer::Amount(amount)
        }
        // AutoDecider keeps every scried card on top — a no-op
        // that wastes every scry and surveil under bot play.
        // Bottom flood and unplayable spells, draw wants first.
        crate::decision::Decision::Scry { player, cards, mode } if *player == seat => {
            decide_scry(state, seat, cards, *mode)
        }
        // AutoDecider takes the first legal color (usually White). Pick
        // the color the bot's HAND actually demands — the most colored
        // pips across held cards — so mana-fixing sources (any-color
        // ramp, the Quandrix Fractal fixers) fix toward castability.
        // The Quandrix probe showed this fall-through at 11 % of all
        // decisions in that college. Ties keep the engine's order.
        crate::decision::Decision::ChooseColor { legal, .. } if !legal.is_empty() => {
            let pips = |color: crate::mana::Color| {
                state.players[seat]
                    .hand
                    .iter()
                    .flat_map(|c| c.definition.cost.symbols.iter())
                    .filter(|s| matches!(s, crate::mana::ManaSymbol::Colored(c) if *c == color))
                    .count()
            };
            let mut best = legal[0];
            for &c in &legal[1..] {
                if pips(c) > pips(best) {
                    best = c;
                }
            }
            crate::decision::DecisionAnswer::Color(best)
        }
        // AutoDecider answers every mid-resolution modal with
        // mode 0. Evaluate each mode's settled outcome instead.
        crate::decision::Decision::ChooseMode { num_modes, .. } if eval_modes => {
            crate::decision::DecisionAnswer::Mode(decide_mode_by_outcome(
                state, seat, *num_modes, w,
            ))
        }
        // CR 510.1c / 510.1e — combat-damage order for a multi-blocked
        // attacker (or a creature blocking several attackers). AutoDecider
        // keeps declaration order; under the flag the policy picks the
        // order whose engine-default split kills the most value from the
        // deciding seat's side of the table. The assignment sibling
        // (`AssignCombatDamage`) deliberately stays on the engine default:
        // lethal-to-each-then-trample is already the assigner's optimum in
        // everything but the banding deny-trample corner, not worth an arm
        // in a pool with no banding.
        crate::decision::Decision::CombatDamageOrder { attacker, blockers }
            if w.damage_order =>
        {
            decide_combat_damage_order(state, seat, *attacker, blockers, w)
        }
        other => AutoDecider.decide(other),
    }
}

/// CR 510.1c — choose the damage order for `dealer`'s combat damage over
/// `victims` (its blockers, or the attackers it blocks in the 510.1e
/// mirror). The engine's split assigns each victim its lethal in this
/// order until the power runs out, so the order alone decides who dies.
/// Candidate orders are scored by simulating exactly that split and
/// summing `permanent_value` over the dead — positive for the opponent's
/// creatures, negative for `seat`'s own, which is what makes the same
/// policy correct from either chair (banding and Defensive Formation
/// hand the choice to the victims' controller). Exhaustive to five
/// victims (120 walks of a five-entry list); a wider band falls back to
/// one greedy order by value per point of lethal. Improvement is strict:
/// an order that only ties the default answers empty, so a game where
/// the choice cannot matter plays (and antithetically pairs) exactly as
/// before the flag.
fn decide_combat_damage_order(
    state: &GameState,
    seat: usize,
    dealer: crate::card::CardId,
    victims: &[(crate::card::CardId, String)],
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::card::{Keyword, KeywordSlice};
    let empty = crate::decision::DecisionAnswer::DamageOrder(vec![]);
    let Some(d) = state.computed_permanent(dealer) else { return empty };
    let power = d.power.max(0) as u32;
    // Deathtouch makes any nonzero assignment lethal (CR 702.2c), which
    // is how `combat_assignment_plan` prices it too.
    let deathtouch = d.keywords().has_kw(&Keyword::Deathtouch);
    // (id, lethal, value signed for `seat`), in the engine's default order.
    let entries: Vec<(crate::card::CardId, u32, i32)> = victims
        .iter()
        .filter_map(|(id, _)| {
            let inst = state.battlefield_find(*id)?;
            let cp = state.computed_permanent_on(inst)?;
            let lethal =
                if deathtouch { 1 } else { (cp.toughness - inst.damage as i32).max(1) as u32 };
            let v = permanent_value_with(state, *id, Some(inst), w);
            Some((*id, lethal, if inst.controller == seat { -v } else { v }))
        })
        .collect();
    let n = entries.len();
    if n < 2 || power == 0 {
        return empty;
    }
    // The engine's `default_damage_split`, in miniature: lethal to each in
    // order while the power lasts; a victim dies exactly when its full
    // lethal fit. (The no-trample excess dump lands on a victim that is
    // already dead, so it never changes the outcome scored here.)
    let dead_value = |order: &[usize]| -> i32 {
        let mut remaining = power;
        let mut total = 0i32;
        for &i in order {
            let (_, lethal, v) = entries[i];
            let assigned = lethal.min(remaining);
            remaining -= assigned;
            if assigned >= lethal {
                total += v;
            }
        }
        total
    };
    let default_idx: Vec<usize> = (0..n).collect();
    let default_score = dead_value(&default_idx);
    let mut best_idx = default_idx.clone();
    let mut best_score = default_score;
    if n <= 5 {
        fn perms(k: usize, idx: &mut Vec<usize>, visit: &mut dyn FnMut(&[usize])) {
            if k == idx.len() {
                visit(idx);
                return;
            }
            for i in k..idx.len() {
                idx.swap(k, i);
                perms(k + 1, idx, visit);
                idx.swap(k, i);
            }
        }
        let mut idx = default_idx;
        perms(0, &mut idx, &mut |order| {
            let s = dead_value(order);
            if s > best_score {
                best_score = s;
                best_idx = order.to_vec();
            }
        });
    } else {
        // A six-way-plus gang block: one greedy order, opponents' best
        // trades first, own creatures last. Not optimal, but the case is
        // rare enough that the exhaustive walk's cost isn't owed.
        let mut idx = default_idx;
        idx.sort_by_key(|&i| {
            let (_, lethal, v) = entries[i];
            std::cmp::Reverse(v.saturating_mul(64) / lethal.max(1) as i32)
        });
        let s = dead_value(&idx);
        if s > best_score {
            best_score = s;
            best_idx = idx;
        }
    }
    if best_score <= default_score {
        return empty;
    }
    crate::decision::DecisionAnswer::DamageOrder(
        best_idx.iter().map(|&i| entries[i].0).collect(),
    )
}

/// The minimum legal attack declaration for the active player: only the
/// creatures that "attack each combat if able" (CR 508.1d — `MustAttack` or
/// goaded). Master Warcraft's outside chooser declares this and nothing else.
fn forced_attacks(state: &GameState) -> Vec<Attack> {
    use crate::card::Keyword;
    let active = state.active_player_idx;
    let computed = state.compute_battlefield();
    let mut out = Vec::new();
    for c in state.battlefield.iter().filter(|c| c.controller == active) {
        let kws = computed
            .iter()
            .find(|p| p.id == c.id)
            .map(|p| p.keywords())
            .unwrap_or(&[]);
        if !kws.has_kw(&Keyword::MustAttack) && c.goaded_by.is_empty() {
            continue;
        }
        let able = c.definition.is_creature()
            && !c.tapped
            && (!kws.has_kw(&Keyword::Defender) || state.ignores_defender_for_attack(c))
            && !kws.has_kw(&Keyword::CantAttack)
            && (!c.summoning_sick || kws.has_kw(&Keyword::Haste));
        if !able {
            continue;
        }
        let opponents = || {
            (0..state.players.len())
                .filter(|&q| !state.same_team(active, q) && state.players[q].is_alive())
        };
        let Some(target) = opponents()
            .find(|q| !c.goaded_by.contains(q))
            .or_else(|| opponents().next())
        else {
            continue;
        };
        out.push(Attack { attacker: c.id, target: AttackTarget::Player(target) });
    }
    out
}

/// The block declaration to submit when the *attacking* seat is the block
/// chooser (Invasion Plans): satisfy only what CR 509.1c forces — every
/// `MustBlock`/`MustAttackOrBlock` defender, and enough blockers for each
/// `AllMustBlock` attacker — and send each forced blocker into the attacker
/// most likely to eat it. Anything not required stays home.
fn forced_blocks(state: &GameState) -> Vec<(CardId, CardId)> {
    use crate::card::Keyword;
    let computed = state.compute_battlefield();
    let kws = |id: CardId| {
        computed.iter().find(|p| p.id == id).map(|p| p.keywords()).unwrap_or(&[])
    };
    let mut out: Vec<(CardId, CardId)> = Vec::new();
    let mut used: Vec<CardId> = state.block_map.keys().copied().collect();
    // Best attacker for `blocker` to run into: the one that kills it and
    // survives, else the biggest.
    let best_attacker = |blocker: &crate::card::CardInstance| {
        let mut cands: Vec<(CardId, i32, bool)> = state
            .attacking
            .iter()
            .filter(|atk| {
                state.defender_for(atk.target).is_some_and(|d| state.same_team(blocker.controller, d))
                    && state.blocker_can_block_attacker(blocker.id, atk.attacker)
            })
            .map(|atk| {
                let dmg = attacker_damage_value(state, atk.attacker);
                let lethal = state
                    .computed_permanent(blocker.id)
                    .is_some_and(|b| dmg >= b.toughness);
                (atk.attacker, dmg, lethal)
            })
            .collect();
        cands.sort_by_key(|(_, dmg, lethal)| (!*lethal, -*dmg));
        cands.first().map(|(id, ..)| *id)
    };
    let force = |blocker_id: CardId, out: &mut Vec<(CardId, CardId)>, used: &mut Vec<CardId>| {
        if used.contains(&blocker_id) {
            return;
        }
        let Some(b) = state.battlefield_find(blocker_id) else { return };
        if b.tapped || kws(blocker_id).has_kw(&Keyword::CantBlock) {
            return;
        }
        if let Some(atk) = best_attacker(b) {
            used.push(blocker_id);
            out.push((blocker_id, atk));
        }
    };
    // CR 509.1c — "blocks each combat if able".
    let must: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| {
            kws(c.id).has_kw(&Keyword::MustBlock) || kws(c.id).has_kw(&Keyword::MustAttackOrBlock)
        })
        .map(|c| c.id)
        .collect();
    for id in must {
        force(id, &mut out, &mut used);
    }
    // CR 509.1c — "all creatures able to block this creature do so" / "must be
    // blocked if able": every idle defender that can block such an attacker.
    for atk in &state.attacking {
        let a_kws = kws(atk.attacker);
        let all = a_kws.has_kw(&Keyword::AllMustBlock);
        if !all && !a_kws.has_kw(&Keyword::MustBeBlocked) {
            continue;
        }
        let candidates: Vec<CardId> = state
            .battlefield
            .iter()
            .filter(|c| {
                state.defender_for(atk.target).is_some_and(|d| state.same_team(c.controller, d))
                    && !used.contains(&c.id)
                    && state.blocker_can_block_attacker(c.id, atk.attacker)
            })
            .map(|c| c.id)
            .collect();
        for id in candidates.into_iter().take(if all { usize::MAX } else { 1 }) {
            used.push(id);
            out.push((id, atk.attacker));
        }
    }
    out
}

/// The combat-damage value an attacker on the battlefield actually assigns:
/// its computed toughness when it has `AssignsCombatDamageByToughness` (Doran,
/// the Siege Tower; CR 510.1c), otherwise its computed power. Falls back to the
/// raw `CardInstance` value when no computed view is available. Used by the
/// block planner so a Doran board's high-toughness attackers are scored at
/// their real threat.
fn attacker_damage_value(state: &GameState, id: CardId) -> i32 {
    attacker_damage_value_hinted(state, id, None)
}

/// [`attacker_damage_value`] for a caller that already holds the attacker —
/// see `GameState::computed_permanent_on`.
fn attacker_damage_value_on(state: &GameState, card: &crate::card::CardInstance) -> i32 {
    attacker_damage_value_hinted(state, card.id, Some(card))
}

fn attacker_damage_value_hinted(
    state: &GameState,
    id: CardId,
    hint: Option<&crate::card::CardInstance>,
) -> i32 {
    use crate::card::Keyword;
    let computed = match hint {
        Some(c) => state.computed_permanent_on(c),
        None => state.computed_permanent(id),
    };
    if let Some(cp) = computed {
        let mut base = if cp.keywords().has_kw(&Keyword::AssignsCombatDamageByToughness) {
            cp.toughness
        } else {
            cp.power
        };
        // CR 702.121 — Melee grows the attacker +1/+1 per opponent it attacks
        // this combat. In a duel that's a guaranteed +1 the moment it's
        // declared, so the planner should weigh it in.
        if cp.keywords().has_kw(&Keyword::Melee) {
            base += 1;
        }
        base
    } else {
        state.battlefield_find(id).map(|c| c.power()).unwrap_or(0)
    }
}

/// Instant-speed response layer: when an opponent's spell sits on top of
/// the stack and it's worth answering (it targets the bot's stuff / the
/// bot itself, or it's expensive), cast a counterspell from hand at it.
/// The `would_accept` dry-run is the final gate (timing, mana via
/// auto-tap, per-counter target filters like Spell Snare's MV gate).
fn pick_stack_response(state: &GameState, seat: usize, w: &EvalWeights) -> Option<Picked> {
    use crate::game::types::StackItem;
    let (spell_id, threat) = state.stack.iter().rev().find_map(|si| {
        let StackItem::Spell { card, caster, target, uncounterable, .. } = si else {
            return None;
        };
        if *caster == seat || *uncounterable {
            return None;
        }
        // Score the spell like a candidate play of the caster's: mana
        // investment + body + what it's aimed at. Replaces the old
        // "anything ≥ 3 cmc or pointed at us" gate, which burned
        // Counterspell on 3-mana value creatures and face burn at 20 life.
        let def = &card.definition;
        // Raw card stats lifted onto the profile's scale so the
        // `permanent_value` term below and the bar at the bottom agree.
        let mut threat = def.cost.cmc() as i32 * w.unit;
        if def.card_types.contains(&crate::card::CardType::Creature) {
            threat += (def.power.max(0) + def.toughness.max(0)) * w.unit;
            threat += (def.keywords.len() as i32).min(3) * w.unit;
        }
        match target {
            // Aimed at one of our permanents: the spell is worth what
            // we'd lose.
            Some(crate::game::Target::Permanent(id))
                if state.battlefield_find(*id).is_some_and(|c| c.controller == seat) =>
            {
                threat += permanent_value(state, *id, w);
            }
            // Aimed at our face: mildly threatening, urgent when low.
            Some(crate::game::Target::Player(p)) if *p == seat => {
                threat += 6 * w.unit;
                if state.effective_life(seat) <= 10 {
                    threat += 8 * w.unit;
                }
            }
            _ => {}
        }
        Some((card.id, threat))
    })?;
    // Hold the counter below this bar — a vanilla two-drop or an early
    // cantrip isn't worth the bot's only interaction. The bar drops as
    // the hand clogs: the Prismari probe measured reactive spells
    // rotting to cleanup (42 discards in 60 games) while a full-height
    // bar held them for a threat that never came — a counter pitched at
    // end of turn answered nothing at all.
    let bar = if state.players[seat].hand.len() >= 6 { 5 } else { 10 };
    if threat < bar * w.unit {
        return None;
    }
    let mut counters: Vec<&crate::card::CardInstance> = state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.card_types.contains(&crate::card::CardType::Instant)
                && effect_counters_spells(&c.definition.effect)
        })
        .collect();
    // Cheapest answer first — hold the expensive counter for later.
    counters.sort_by_key(|c| c.definition.cost.cmc());
    // The same affordability pre-filter the main hand sweep runs, for the
    // same reason and now with a number: this path had none, so a counter the
    // board cannot pay for was probed in full — a `GameState` clone, a
    // `mana_source_table` build, the taps it could reach, and a rollback — on
    // every threatening spell, every time. `CRAB_PAY_FAILS` measured **700 of
    // `fixed`'s 704 failed payments as one cost, `{U}{U}`**: this deck's
    // Counterspell, asked and re-asked against two Islands that were not
    // there. The dry run is still the final gate; this only skips the ones
    // the cheap test already answers.
    let sweep = SweepMana::new(state, seat);
    for c in counters {
        if !can_afford_in_state_with(state, seat, c, w, &sweep) {
            continue;
        }
        let action = GameAction::CastSpell {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(spell_id)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if let Some(next) = state.accept(action.clone()) {
            return Some(Picked::Probed(action, Box::new(next)));
        }
    }
    None
}

/// React to a threatening opponent ability on the stack with a dedicated
/// ability-counter card (Stifle / Disallow). The ability's source is the
/// target slot. Held separate from `pick_stack_response`'s spell logic so a
/// counter that can only hit abilities still gets used.
fn pick_ability_counter_response(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
) -> Option<GameAction> {
    use crate::game::types::StackItem;
    // Topmost opponent ability on the stack — counter the most recent one.
    let source = state.stack.iter().rev().find_map(|si| match si {
        StackItem::Trigger { source, controller, .. } if *controller != seat => Some(*source),
        _ => None,
    })?;
    let mut counters: Vec<&crate::card::CardInstance> = state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.card_types.contains(&crate::card::CardType::Instant)
                && effect_counters_abilities(&c.definition.effect)
        })
        .collect();
    counters.sort_by_key(|c| c.definition.cost.cmc());
    // The affordability pre-filter every hand-probing response path now
    // runs — see `pick_stack_response`. Without it a spell the board cannot
    // pay for costs a full probe (clone, `mana_source_table`, taps, rollback)
    // to learn what five adds and five compares already knew, and
    // `CRAB_PAY_FAILS` caught these paths probing with **zero untapped
    // sources and an empty pool**.
    let sweep = SweepMana::new(state, seat);
    for c in counters {
        if !can_afford_in_state_with(state, seat, c, w, &sweep) {
            continue;
        }
        let action = GameAction::CastSpell {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(source)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// The stack 2-for-1: an opponent's aura or targeted pump aimed at their
/// OWN creature is an invitation — instant removal on that creature in
/// response resolves first, the buff fizzles on a vanished target, and
/// one card trades for two. The response chain previously knew only
/// counters here, so the bot watched Auras resolve onto creatures it was
/// holding the answer for. Lethality is judged against the creature's
/// CURRENT toughness — the pump never resolves. Behind
/// [`EvalWeights::buff_2for1`] (ladder profile `buff2for1`) until gated.
fn pick_buff_response(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::card::CardType;
    use crate::effect::{Selector, Value};
    use crate::game::types::StackItem;
    if !w.buff_2for1 {
        return None;
    }
    // Topmost opponent spell aimed at a creature the opponent controls,
    // and shaped like a buff: an Aura, or a first-leaf pump.
    fn pump_leaf(e: &Effect) -> bool {
        match e {
            Effect::PumpPT { what, .. } => {
                matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
            }
            Effect::Seq(v) => v.first().is_some_and(pump_leaf),
            _ => false,
        }
    }
    let (buff_cmc, victim) = state.stack.iter().rev().find_map(|si| {
        let StackItem::Spell { card, caster, target, .. } = si else {
            return None;
        };
        if *caster == seat {
            return None;
        }
        let Some(Target::Permanent(id)) = target else {
            return None;
        };
        let theirs = state.battlefield_find(*id).is_some_and(|c| c.controller != seat);
        if !theirs {
            return None;
        }
        let def = &card.definition;
        (def.is_aura() || pump_leaf(&def.effect))
            .then(|| (def.cost.cmc() as i32, *id))
    })?;
    // Worth a card: the creature plus the spell fizzling on it, together.
    if permanent_value(state, victim, w) + buff_cmc * w.unit < 6 * w.unit {
        return None;
    }
    // The affordability pre-filter every hand-probing response path now
    // runs — see `pick_stack_response`. Without it a spell the board cannot
    // pay for costs a full probe (clone, `mana_source_table`, taps, rollback)
    // to learn what five adds and five compares already knew, and
    // `CRAB_PAY_FAILS` caught these paths probing with **zero untapped
    // sources and an empty pool**.
    let sweep = SweepMana::new(state, seat);
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.card_types.contains(&CardType::Instant))
        .filter(|c| can_afford_in_state_with(state, seat, c, w, &sweep))
    {
        // Same first-leaf removal shapes as `pick_defensive_removal`.
        fn removal_leaf(e: &Effect) -> Option<&Effect> {
            match e {
                Effect::Destroy { .. }
                | Effect::DestroyNoRegen { .. }
                | Effect::DealDamage { .. } => Some(e),
                Effect::Seq(v) => v.first().and_then(removal_leaf),
                _ => None,
            }
        }
        let Some(leaf) = removal_leaf(&c.definition.effect) else { continue };
        let answers = match leaf {
            Effect::Destroy { what } | Effect::DestroyNoRegen { what } => {
                matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
            }
            Effect::DealDamage { to, amount } => {
                matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. })
                    && match amount {
                        Value::Const(n) => state.computed_permanent(victim).is_some_and(|cp| {
                            let marked = state
                                .battlefield_find(victim)
                                .map(|c| c.damage as i32)
                                .unwrap_or(0);
                            *n >= cp.toughness - marked
                        }),
                        _ => false,
                    }
            }
            _ => false,
        };
        if !answers {
            continue;
        }
        let action = GameAction::CastSpell {
            card_id: c.id,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if !ward_gate_ok(state, seat, &action) {
            continue;
        }
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// Instant removal at a declared attacker, from the DEFENDER's side of
/// combat. The response chain only ever countered spells, so a hand full
/// of kill spells watched every alpha strike connect — the SOS college
/// probes measured 68-78 % of attackers unblocked while removal rotted
/// to cleanup discards. Aim at the most valuable attacker the spell
/// actually answers, before blocks commit; `would_accept` gates instant
/// timing and the ward gate keeps taxes payable.
fn pick_defensive_removal(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::card::CardType;
    use crate::effect::{Selector, Value};
    let mut attackers: Vec<CardId> = state
        .attacking()
        .iter()
        .filter(|a| state.defender_for(a.target) == Some(seat))
        .map(|a| a.attacker)
        .collect();
    if attackers.is_empty() {
        return None;
    }
    attackers.sort_by_cached_key(|id| std::cmp::Reverse(permanent_value(state, *id, w)));
    // First-leaf removal shapes, the same convention the counter scan
    // uses: a dedicated kill spell, not a buried rider.
    fn removal_leaf(e: &Effect) -> Option<&Effect> {
        match e {
            Effect::Destroy { .. } | Effect::DestroyNoRegen { .. } | Effect::DealDamage { .. } => {
                Some(e)
            }
            Effect::Seq(v) => v.first().and_then(removal_leaf),
            _ => None,
        }
    }
    let sweep = SweepMana::new(state, seat);
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.card_types.contains(&CardType::Instant))
        .filter(|c| can_afford_in_state_with(state, seat, c, w, &sweep))
    {
        let Some(leaf) = removal_leaf(&c.definition.effect) else { continue };
        for &atk in &attackers {
            // Worth a card: skip chaff attackers.
            if permanent_value(state, atk, w) < 6 * w.unit {
                continue;
            }
            let answers = match leaf {
                Effect::Destroy { what } | Effect::DestroyNoRegen { what } => {
                    matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
                }
                Effect::DealDamage { to, amount } => {
                    matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. })
                        && match amount {
                            Value::Const(n) => state
                                .computed_permanent(atk)
                                .is_some_and(|cp| {
                                    let marked = state
                                        .battlefield_find(atk)
                                        .map(|c| c.damage as i32)
                                        .unwrap_or(0);
                                    *n >= cp.toughness - marked
                                }),
                            _ => false,
                        }
                }
                _ => false,
            };
            if !answers {
                continue;
            }
            let action = GameAction::CastSpell {
                card_id: c.id,
                target: Some(Target::Permanent(atk)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            };
            if !ward_gate_ok(state, seat, &action) {
                continue;
            }
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// SOS Prepare — the inset spell is a one-shot resource carried by a
/// fragile body. When an opponent's spell on the stack targets one of the
/// bot's prepared creatures, cast the inset spell in response, so the
/// resource is spent before the body (and the Prepared counter with it)
/// is answered. `would_accept` gates timing: only an instant-speed inset
/// spell actually fires here, a sorcery copy is simply rejected.
fn pick_prepare_response(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::game::types::StackItem;
    let threatened: Vec<CardId> = state.stack.iter().rev().find_map(|si| {
        let StackItem::Spell { caster, target, additional_targets, .. } = si else {
            return None;
        };
        if *caster == seat {
            return None;
        }
        let hits: Vec<CardId> = target
            .iter()
            .chain(additional_targets.iter())
            .filter_map(|t| match t {
                Target::Permanent(id) => state
                    .battlefield_find(*id)
                    .filter(|c| {
                        c.controller == seat
                            && c.definition.prepare_spell.is_some()
                            && c.counter_count(crate::card::CounterType::Prepared) > 0
                    })
                    .map(|c| c.id),
                _ => None,
            })
            .collect();
        if hits.is_empty() { None } else { Some(hits) }
    })?;
    for creature_id in threatened {
        let Some(c) = state.battlefield_find(creature_id) else { continue };
        let Some(spell) = c.definition.prepare_spell.as_deref() else { continue };
        // Same construction as the main-phase candidate block.
        let (target, additional_targets) = if spell.effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(&spell.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let x_value = if x_relevant(spell) {
            Some(max_affordable_x_for_def(state, seat, spell, 0, w))
        } else {
            None
        };
        let action = GameAction::CastPrepareSpell {
            creature_id,
            target,
            additional_targets,
            mode: None,
            x_value,
        };
        if !ward_gate_ok(state, seat, &action) {
            continue;
        }
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// True when the effect tree's primary action counters a spell (the shapes
/// a dedicated counterspell card uses — not buried `MayDo` riders).
fn effect_counters_spells(eff: &Effect) -> bool {
    match eff {
        Effect::CounterSpell { .. }
        | Effect::CounterSpellExileSameNamed { .. }
        | Effect::CounterSpellToZone { .. }
        | Effect::CounterUnlessPaid { .. }
        | Effect::CounterUnless { .. } => true,
        Effect::Seq(v) => v.first().is_some_and(effect_counters_spells),
        _ => false,
    }
}

/// True when the effect can counter an activated/triggered ability (Stifle's
/// `CounterAbility`, or a modal counter like Disallow whose `ChooseN`/
/// `ChooseMode` carries a `CounterAbility` arm).
fn effect_counters_abilities(eff: &Effect) -> bool {
    match eff {
        Effect::CounterAbility { .. } => true,
        Effect::Seq(v) => v.first().is_some_and(effect_counters_abilities),
        Effect::ChooseMode(modes) => modes.iter().any(effect_counters_abilities),
        Effect::ChooseN { modes, .. } => modes.iter().any(effect_counters_abilities),
        _ => false,
    }
}

/// Land-count mulligan heuristic. A keepable opening hand wants roughly
/// 2–5 lands out of seven; 0–1 (screw) or 6–7 (flood) are shipped. We stop
/// digging after two mulligans (a London mulligan past that bottoms too
/// many cards to be worth chasing a perfect curve) and always keep a hand
/// of three or fewer cards. Reads land counts off the live hand zone since
/// the `Decision::Mulligan` payload only carries names.
/// Colors a land card could tap for, for mulligan color-screw checks.
/// Reads basic land types (Plains→W, …) plus `AddMana` effects on its
/// activated abilities; "any color" payloads yield the full WUBRG set.
fn land_color_output(card: &CardDefinition) -> crate::mana::ColorSet {
    use crate::card::LandType;
    use crate::mana::{Color, ColorSet};
    let mut set = ColorSet::empty();
    for lt in &card.subtypes.land_types {
        match lt {
            LandType::Plains => set.insert(Color::White),
            LandType::Island => set.insert(Color::Blue),
            LandType::Swamp => set.insert(Color::Black),
            LandType::Mountain => set.insert(Color::Red),
            LandType::Forest => set.insert(Color::Green),
            _ => {}
        }
    }
    for ab in &card.activated_abilities {
        accumulate_mana_colors(&ab.effect, &mut set);
    }
    set
}

/// Choose which land to play this turn. Among the lands the engine would
/// accept, prefer the one that covers the most colors the bot's hand wants
/// but can't yet produce from the lands it already controls — basic
/// mana-fixing so a green hand doesn't strand its spells behind a Mountain.
/// Falls back to the first playable land when nothing improves color
/// coverage (or no land needs fixing).
/// Does this land's own printed static say it enters tapped? (A
/// `StaticEffect::EntersTapped` on the card itself — school lands, guild
/// gates. Statics granted by *other* permanents aren't the land's
/// property and aren't consulted.)
fn land_enters_tapped(def: &crate::card::CardDefinition) -> bool {
    def.static_abilities
        .iter()
        .any(|s| matches!(s.effect, crate::card::StaticEffect::EntersTapped { .. }))
}

fn pick_land_to_play(state: &GameState, seat: usize, w: &EvalWeights) -> Option<CardId> {
    use crate::mana::{Color, ColorSet};
    const WUBRG: [Color; 5] =
        [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];

    // Both scorers below loop over the hand's lands; with none there is
    // nothing to score, and the mana-base walks that follow are all for the
    // scoring. One hand walk disqualifies three board-and-hand walks.
    if !state.players[seat].hand.iter().any(|c| c.definition.is_land()) {
        return None;
    }

    // The two card-independent halves of the engine's own land-drop validator,
    // asked once instead of once per hand land. Both loops below filter with
    // `would_accept(PlayLand(id))` — a full engine dry-run on a `GameState`
    // clone, ~11,600 Ir apiece and 82 % of this function's cost — while
    // `play_land_with_face` opens with exactly these two checks, against
    // exactly this player, before it looks at the card at all.
    //
    // **The caller already gates most of this, which is the measurement worth
    // keeping.** The obvious model — "the bot keeps taking main-phase actions
    // after spending its land drop, so most probes here are foregone" — is
    // wrong: `main_phase_action_with` reaches this function only when a drop is
    // plausible, so the gate catches the residue, **934 -> 856 probes on
    // `fixed` (-8.4 %) and 2,496 -> 2,080 on `sealed` (-16.7 %)**, worth
    // `sealed` -0.066 % / `cube` -0.031 % / `fixed` -0.003 % of the program.
    // Do not re-derive the larger figure from the 82 % above.
    //
    // Equivalent rather than conservative — not a heuristic pre-filter of the
    // kind PERF (-51) warns about, where an over-tight test makes a legal line
    // permanently invisible. It is the engine's first two gates, hoisted, and
    // it reads the priority holder for the same reason the engine does.
    let p = state.priority.player_with_priority;
    if !state.can_cast_sorcery_speed(p) || !state.can_player_play_land(p) {
        return None;
    }

    // Colors already producible from battlefield lands the bot controls.
    let have = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .fold(ColorSet::empty(), |acc, c| acc.union(land_color_output(&c.definition)));
    // Colors the bot's nonland hand cards want to be cast.
    let mut want = ColorSet::empty();
    for c in state.players[seat].hand.iter().filter(|c| !c.definition.is_land()) {
        for col in c.definition.cost.colors() {
            want.insert(col);
        }
    }
    // The colors still missing from the bot's mana base.
    let needed: Vec<Color> =
        WUBRG.into_iter().filter(|&col| want.contains(col) && !have.contains(col)).collect();

    // Converge decks value a NEW color even when no pip demands it —
    // the payment repair maximizes colors spent, but only among the
    // colors the mana base makes, and this chooser decides those.
    let converge_want = w.converge_lands
        && (state.players[seat].hand.iter().any(|c| c.definition.wants_converge())
            || state
                .battlefield
                .iter()
                .any(|c| c.controller == seat && c.definition.wants_converge()));
    let fresh_color =
        |out: ColorSet| WUBRG.into_iter().any(|col| out.contains(col) && !have.contains(col));

    if !w.land_urgency {
        let mut best: Option<(CardId, usize)> = None;
        for c in state.players[seat].hand.iter().filter(|c| c.definition.is_land()) {
            if !state.would_accept(GameAction::PlayLand(c.id)) {
                continue;
            }
            let out = land_color_output(&c.definition);
            // Doubled so pip coverage still dominates; the converge bonus
            // only breaks ties between otherwise-equal drops. Flag off,
            // the scale change preserves the ordering exactly.
            let coverage = 2 * needed.iter().filter(|&&col| out.contains(col)).count()
                + usize::from(converge_want && fresh_color(out));
            // Higher coverage wins; the first playable land is the fallback (so a
            // colorless/utility land still gets played when nothing needs fixing).
            if best.is_none_or(|(_, s)| coverage > s) {
                best = Some((c.id, coverage));
            }
        }
        return best.map(|(id, _)| id);
    }

    // Per-color urgency: the cheapest hand card wanting that color sets
    // how soon a source is needed. A {B} two-drop scores 6, a {B}
    // six-drop 2 — both "missing", not equally missing.
    let urgency = |col: Color| -> usize {
        state.players[seat]
            .hand
            .iter()
            .filter(|c| !c.definition.is_land() && c.definition.cost.colors().contains(&col))
            .map(|c| 8usize.saturating_sub(c.definition.cost.cmc() as usize).max(1))
            .max()
            .unwrap_or(0)
    };

    // Whether a land buys a cast *this turn* is a property of that land,
    // not of the turn: an untapped source adds mana and a color now, a
    // tapped one adds neither until next turn. So the question is asked
    // per candidate rather than once.
    let untapped_now = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land() && !c.tapped)
        .count();
    let enables_a_cast = |out: ColorSet, tapped: bool| -> bool {
        let mana = untapped_now + usize::from(!tapped);
        let colors = if tapped { have } else { have.union(out) };
        state.players[seat].hand.iter().any(|c| {
            !c.definition.is_land()
                && c.definition.cost.cmc() as usize <= mana
                && c.definition.cost.colors().iter().all(|col| colors.contains(*col))
        })
    };

    let mut best: Option<(CardId, i32)> = None;
    for c in state.players[seat].hand.iter().filter(|c| c.definition.is_land()) {
        if !state.would_accept(GameAction::PlayLand(c.id)) {
            continue;
        }
        let out = land_color_output(&c.definition);
        let mut score: i32 =
            needed.iter().filter(|&&col| out.contains(col)).map(|&col| urgency(col) as i32).sum();
        // A fresh color for converge sits below "enables a cast this
        // turn" (4) and above the tapland penalty: fixing text beats
        // fixing nothing, but never costs a curve play.
        if converge_want && fresh_color(out) {
            score += 3;
        }
        // Untapped sources the bot already has are worth a little on
        // their own, so a second Forest still beats a dead utility land.
        if !needed.is_empty() || out != ColorSet::empty() {
            score += 1;
        }
        let tapped = land_enters_tapped(&c.definition);
        // A land that turns on a spell this turn is worth more than the
        // fixing it promises for later; a tapland that promises fixing
        // costs almost nothing on a turn with no play.
        if enables_a_cast(out, tapped) {
            score += 4;
        }
        if tapped {
            score -= 1;
        }
        if best.is_none_or(|(_, s)| score > s) {
            best = Some((c.id, score));
        }
    }
    best.map(|(id, _)| id)
}

/// Bot policy for `Decision::OptionalTrigger`: take the trigger unless its
/// matching `MayDo` body imposes a clear self-cost (lose life / sacrifice /
/// discard on the bot). `AutoDecider` declines *every* optional trigger,
/// which means a bot would never take a beneficial "you may" (Provoke's
/// "you may", Boast token riders, etc.); this makes those fire.
pub fn optional_trigger_beneficial(state: &GameState, source: CardId, description: &str) -> bool {
    // Locate the source card's definition in any zone the bot can see.
    let def = state
        .battlefield
        .iter()
        .find(|c| c.id == source)
        .map(|c| &c.definition)
        .or_else(|| {
            state
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter().chain(p.hand.iter()))
                .find(|c| c.id == source)
                .map(|c| &c.definition)
        })
        // A resolving SPELL lives on the stack — without this, any
        // instant/sorcery's self-costly MayDo fell through to the
        // blanket-true fallback below.
        .or_else(|| {
            state.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Spell { card, .. } if card.id == source => {
                    Some(&card.definition)
                }
                _ => None,
            })
        })
        // A Paradigm card prompts from EXILE (`CastFreeParadigmCopy`),
        // as do other exile-resident recurrences.
        .or_else(|| state.exile.iter().find(|c| c.id == source).map(|c| &c.definition));
    let Some(def) = def else { return true };
    // Find the `MayDo` body whose description matches the prompt. Scan the
    // card's spell effect, its triggered abilities, and any static-ability
    // reflexive (`when_you_do`) — the prompt can originate from any of these
    // (e.g. Valentin's exile-replacement reflexive lives on a static).
    let mut body = find_maydo_body(&def.effect, description);
    if body.is_none() {
        for t in &def.triggered_abilities {
            if let Some(b) = find_maydo_body(&t.effect, description) {
                body = Some(b);
                break;
            }
        }
    }
    if body.is_none() {
        for sa in &def.static_abilities {
            if let crate::effect::StaticEffect::ExileDyingOpponentCreatures {
                when_you_do: Some(eff),
            } = &sa.effect
                && let Some(b) = find_maydo_body(eff, description)
            {
                body = Some(b);
                break;
            }
        }
    }
    // Exploit (CR 702.105 — "Exploit: sacrifice a creature?"): the body is a
    // Sacrifice that the generic self-cost screen would always decline. Accept
    // it when the controller has a spare creature to feed it — a token, or the
    // exploiter is one of several creatures so it can sacrifice the weakest (or
    // itself for a strong ETB payoff). Card advantage off a token is a clean win.
    if description.starts_with("Exploit") {
        let ctrl = state.battlefield.find_by_id(source).map(|c| c.controller);
        if let Some(seat) = ctrl {
            let creatures: Vec<&crate::card::CardInstance> = state
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && c.definition.is_creature())
                .collect();
            let has_token = creatures.iter().any(|c| c.is_token);
            // Accept with a sacrificial token, or when there's more than one
            // creature so we don't have to give up the exploiter itself.
            return has_token || creatures.len() > 1;
        }
        return false;
    }
    // Take it unless the body is self-costly; default to taking when the
    // body can't be introspected (most "you may" on your own permanents is
    // upside).
    body.map(|b| !effect_imposes_self_cost(b)).unwrap_or(true)
}

/// Recursively find the optional-effect body whose prompt is `desc`. Both
/// `Effect::MayDo` and `Effect::MayPay` surface as a `Decision::OptionalTrigger`
/// keyed on their description, so the bot's self-cost screen (e.g. a "you may
/// pay {2}: each player loses 3 life" body it shouldn't auto-accept) applies to
/// both shapes.
fn find_maydo_body<'a>(eff: &'a Effect, desc: &str) -> Option<&'a Effect> {
    match eff {
        Effect::MayDo { description, body } | Effect::MayPay { description, body, .. }
            if description == desc =>
        {
            Some(body)
        }
        // A reflexive tap-cost (Caparocti Sunborn) or discard-cost (Toph,
        // Hardheaded Teacher) trigger surfaces the same `OptionalTrigger`
        // prompt; its `then` payoff is what the bot screens.
        Effect::MayTap { description, then, .. }
        | Effect::MayDiscard { description, then, .. }
            if description == desc =>
        {
            Some(then)
        }
        Effect::MayDo { body, .. }
        | Effect::MayPay { body, .. }
        | Effect::MayTap { then: body, .. }
        | Effect::MayDiscard { then: body, .. }
        | Effect::ForEach { body, .. } => find_maydo_body(body, desc),
        Effect::Seq(v) => v.iter().find_map(|e| find_maydo_body(e, desc)),
        Effect::ChooseMode(v)
        | Effect::ChooseN { modes: v, .. }
        | Effect::Escalate { modes: v, .. }
        | Effect::EscalatingThisTurn { modes: v } => {
            v.iter().find_map(|e| find_maydo_body(e, desc))
        }
        Effect::If { then, else_, .. } => {
            find_maydo_body(then, desc).or_else(|| find_maydo_body(else_, desc))
        }
        _ => None,
    }
}

/// Whether `eff` (a "you may" body) imposes a clear cost on its controller —
/// losing life, sacrificing, or discarding. Conservative: the bot declines
/// such triggers rather than paying for an effect it can't value-judge.
fn effect_imposes_self_cost(eff: &Effect) -> bool {
    use crate::effect::{PlayerRef, Selector};
    let hits_self = |sel: &Selector| {
        matches!(sel, Selector::You | Selector::This)
            || matches!(sel, Selector::Player(PlayerRef::You))
    };
    match eff {
        Effect::LoseLife { who, .. }
        | Effect::Discard { who, .. }
        | Effect::Mill { who, .. }
        | Effect::LoseHalfLife { who, .. }
        | Effect::MillHalf { who, .. }
        | Effect::DiscardHalf { who, .. } => hits_self(who),
        // Self-directed damage (a "you may have ~ deal N damage to you" rider).
        Effect::DealDamage { to, .. } => hits_self(to),
        // Drain *out of* the bot is a cost; drain *into* the bot is upside.
        Effect::Drain { from, .. } => hits_self(from),
        Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => hits_self(who),
        Effect::SacrificeAndRemember { .. } => true,
        Effect::SacrificeAnyNumber { who, .. } => matches!(who, PlayerRef::You),
        Effect::PayLifeLookTake { who } => matches!(who, PlayerRef::You),
        Effect::Seq(v) => v.iter().any(effect_imposes_self_cost),
        Effect::ChooseMode(v)
        | Effect::ChooseN { modes: v, .. }
        | Effect::Escalate { modes: v, .. }
        | Effect::EscalatingThisTurn { modes: v } => {
            v.iter().any(effect_imposes_self_cost)
        }
        Effect::If { then, else_, .. } => {
            effect_imposes_self_cost(then) || effect_imposes_self_cost(else_)
        }
        Effect::ForEach { body, .. } | Effect::MayDo { body, .. } => effect_imposes_self_cost(body),
        // Mana/energy "pay or else" wrap a fallback (usually SacrificeSource);
        // the bot reads the fallback to decide whether declining is costly.
        Effect::PayManaOrElse { otherwise, .. } | Effect::PayEnergyOrElse { otherwise, .. } => {
            effect_imposes_self_cost(otherwise)
        }
        // Blight (CR 701.68) puts -1/-1 counters on a creature you control —
        // a clear self-cost, so the bot declines "may blight N" upside riders
        // rather than shrinking (or killing) its own board.
        Effect::Blight { .. } => true,
        // "You may sacrifice/exile this" riders are a clear self-cost.
        Effect::SacrificeSource => true,
        Effect::Exile { what } => hits_self(what),
        // "You may put this into exile / your graveyard / your library" is a
        // self-cost too (returning it to *hand* is upside, so that's excluded).
        Effect::Move { what, to } => {
            hits_self(what)
                && matches!(
                    to,
                    crate::effect::ZoneDest::Exile
                        | crate::effect::ZoneDest::Graveyard
                        | crate::effect::ZoneDest::Library { .. }
                )
        }
        Effect::PayOrLoseGame { .. } => true,
        _ => false,
    }
}

/// Constant life the bot itself would lose to `eff` resolving on its own
/// spell — the amount behind the Paradigm copy guard. Counts `You`-directed
/// life loss AND `Target`-directed loss: a draw-plus-lose body (Decorum
/// Dissertation's "target player draws two and loses 2") auto-targets the
/// caster, so its Target(0) loss lands on the bot. That over-counts a
/// drain the bot would aim at the opponent, which errs toward declining a
/// free cast at low life — the cheap direction. Non-constant amounts count
/// as zero (can't be sized without resolving).
fn self_life_loss(eff: &Effect) -> i32 {
    use crate::effect::{Selector, Value};
    let hits = |sel: &Selector| {
        matches!(sel, Selector::You | Selector::This | Selector::Target(_))
    };
    match eff {
        Effect::LoseLife { who, amount: Value::Const(n) } if hits(who) => (*n).max(0),
        Effect::Drain { from, amount: Value::Const(n), .. } if hits(from) => (*n).max(0),
        Effect::Seq(v) => v.iter().map(self_life_loss).sum(),
        Effect::If { then, else_, .. } => self_life_loss(then).max(self_life_loss(else_)),
        Effect::ChooseMode(v)
        | Effect::ChooseN { modes: v, .. }
        | Effect::Escalate { modes: v, .. }
        | Effect::EscalatingThisTurn { modes: v } => {
            v.iter().map(self_life_loss).max().unwrap_or(0)
        }
        Effect::ForEach { body, .. } | Effect::MayDo { body, .. } => self_life_loss(body),
        _ => 0,
    }
}

/// Bot heuristic for `Decision::SearchLibrary`: pick a basic land that
/// adds the bot's least-covered color, else (no basic land among the
/// candidates) grab the highest-mana-value candidate — a creature/spell
/// tutor (Fauna Shaman, Imperial Recruiter, Spellseeker) should fetch its
/// most impactful hit, not the first one, and certainly not fizzle like the
/// stock `AutoDecider`.
/// Rough board value of a permanent for target selection: mana value + size,
/// plus a loyalty term for planeswalkers and a small legendary premium. When
/// the profile enables it, a keyword term (see [`keyword_value`]) too.
fn permanent_value(state: &GameState, id: crate::card::CardId, w: &EvalWeights) -> i32 {
    permanent_value_with(state, id, state.battlefield_find(id), w)
}

/// [`permanent_value`] with the permanent already in hand — the caller that
/// dominates it (`eval_material_inner`) is walking the battlefield when it
/// asks, and the `id` form re-finds the card with a linear scan. Candidate
/// (11)'s shape; `battlefield_find` is 4.03 % of the simulator across its
/// call sites and this is the largest of the bot's.
fn permanent_value_with(
    state: &GameState,
    id: crate::card::CardId,
    inst: Option<&crate::card::CardInstance>,
    w: &EvalWeights,
) -> i32 {
    use crate::card::{CardType, CounterType, Supertype};
    // `inst` is this state's battlefield permanent at both callers
    // (`permanent_value` hands over `battlefield_find`, `eval_material_inner`
    // the card it is walking), so the `_on` form skips the find.
    let cp = match inst {
        Some(c) => state.computed_permanent_on(c),
        None => state.computed_permanent(id),
    };
    let Some(c) = cp else { return 0 };
    let mut v = inst.map(|c| c.definition.cost.cmc() as i32).unwrap_or(0) * w.cmc;
    if c.card_types().contains(&CardType::Creature) {
        v += w.creature_base + c.power.max(0) * w.power + c.toughness.max(0) * w.toughness;
        if w.keyword_pct != 0 {
            v += keyword_value(c.keywords(), c.power, w) * w.keyword_pct / 100;
        }
    }
    if c.card_types().contains(&CardType::Planeswalker) {
        v += inst.map(|c| c.counter_count(CounterType::Loyalty) as i32).unwrap_or(0) * w.unit;
    }
    if c.supertypes().contains(&Supertype::Legendary) {
        v += 2 * w.unit;
    }
    // A Prepared counter on a prepare-spell body is a castable spell in
    // waiting (SOS): worth about the inset spell's mana value. Gives the
    // eval, the mid-resolution mode picker (Biblioplex Tomekeeper's
    // prepare-vs-unprepare), the attack simulation (attack-trigger
    // preparers gain the counter mid-sim), and removal targeting a live
    // read on the resource — an opponent's prepared creature IS the
    // better kill at equal stats.
    if let Some(inst) = inst
        && inst.counter_count(CounterType::Prepared) > 0
        && let Some(spell) = inst.definition.prepare_spell.as_deref()
    {
        v += (1 + spell.cost.cmc() as i32) * w.unit;
    }
    v
}

/// Keyword contribution to a creature's board value, in `w.unit`-scaled
/// points. Ported from Forge's `CreatureEvaluator`, whose central idea is
/// that keywords split into two families:
///
/// * **Offensive** -- evasion and damage riders are worth what they let the
///   body actually deal, so they scale with power. Flying on a 5/5 is a
///   five-point-per-turn clock; flying on a 1/1 is a chump-blocker that
///   dodges. Pricing both at a flat bonus is the mistake this fixes.
/// * **Defensive** -- protection and resilience are worth roughly the same
///   whatever the body, so they're flat. Hexproof on a 1/1 and on a 5/5
///   both buy exactly "removal doesn't answer this".
///
/// Bad keywords subtract, and a creature that can neither attack nor block
/// collapses to a token value regardless of its printed size.
fn keyword_value(keywords: &[crate::card::Keyword], power: i32, w: &EvalWeights) -> i32 {
    use crate::card::Keyword;
    let p = power.max(0);
    let has = |k: &Keyword| keywords.contains(k);
    // A body that can't attack or block is nearly inert: no size term
    // survives, only the mana it represents. Checked first so the
    // offensive terms below can't rescue a Pacifism'd fatty.
    let inert = (has(&Keyword::CantAttack) || has(&Keyword::Defender))
        && (has(&Keyword::CantBlock) || has(&Keyword::Decayed));
    if inert {
        return -(p * w.power + w.unit);
    }
    let mut v = 0;
    // -- Offensive: scaled by power ------------------------------------
    if has(&Keyword::Flying) || has(&Keyword::Horsemanship) || has(&Keyword::Shadow) {
        v += p * 2 * w.unit / 3;
    }
    if has(&Keyword::Fear) || has(&Keyword::Intimidate) {
        v += p * 2 * w.unit / 5;
    }
    if has(&Keyword::Menace) {
        v += p * w.unit / 4;
    }
    if has(&Keyword::DoubleStrike) {
        v += w.unit + p * w.unit;
    } else if has(&Keyword::FirstStrike) {
        v += w.unit + p * w.unit / 3;
    }
    if has(&Keyword::Lifelink) {
        v += p * 2 * w.unit / 3;
    }
    if has(&Keyword::Infect) {
        v += p * w.unit;
    } else if has(&Keyword::Wither) {
        v += p * 2 * w.unit / 3;
    }
    if p > 1 && has(&Keyword::Trample) {
        v += (p - 1) * w.unit / 3;
    }
    if has(&Keyword::Vigilance) {
        v += p * w.unit / 3;
    }
    for k in keywords {
        match k {
            Keyword::Toxic(n) | Keyword::Poisonous(n) => v += *n as i32 * w.unit / 3,
            Keyword::Annihilator(n) => v += *n as i32 * 3 * w.unit,
            Keyword::Rampage(n) | Keyword::Bushido(n) => v += *n as i32 * w.unit,
            _ => {}
        }
    }
    // -- Defensive: flat -----------------------------------------------
    if has(&Keyword::Indestructible) {
        v += 5 * w.unit;
    }
    if has(&Keyword::Deathtouch) {
        v += 2 * w.unit;
    }
    if has(&Keyword::Hexproof) {
        v += 2 * w.unit;
    } else if has(&Keyword::Shroud) {
        // Shroud is strictly worse than hexproof for its controller: it
        // blocks our own auras, equipment and pump spells too.
        v += 3 * w.unit / 2;
    }
    if has(&Keyword::Reach) && !has(&Keyword::Flying) {
        v += w.unit / 2;
    }
    // -- Bad -----------------------------------------------------------
    if has(&Keyword::Defender) || has(&Keyword::CantAttack) {
        v -= p * w.power * 2 / 3 + w.unit;
    }
    if has(&Keyword::CantBlock) || has(&Keyword::Decayed) {
        v -= w.unit;
    }
    v
}

/// Value of a life total, in `w.unit`-scaled points.
///
/// Life is not linear: the first few points are the difference between
/// losing and not, while points near the starting total are close to
/// worthless. A linear term prices "gain 3" the same at 3 life and at 20,
/// so the bot over-values incidental lifegain when healthy and under-values
/// it when it's actually dying. The curve is XMage's `LIFE_SCORES` shape
/// (`ArtificialScoringSystem`), rescaled so that 20 life is worth the same
/// 20 points it was under the linear term -- only the shape changes, which
/// keeps this comparable against the baseline on the ladder without a
/// wholesale re-tune of every other weight.
///
/// Expressed in tenths of a point (then scaled by `unit`) so the curve stays
/// strictly increasing under integer arithmetic -- a flat spot would make
/// "gain 1 life" evaluate to exactly zero.
fn life_value(life: i32, w: &EvalWeights) -> i32 {
    // ⚠ **Clamp first.** A life total is not bounded by anything: Beacon of
    // Immortality doubles it every other turn until it saturates at `i32::MAX`
    // (ENGINE_BACKLOG's closed stall lead — a correct card doing what it
    // prints, seen in 4 of 183,600 sweep games). Every path below *multiplies*
    // it — `life * w.unit`, `(life - MAX) * 4`, `tenths * w.unit / 10` — so an
    // unclamped total wraps, and in release it wraps silently into a large
    // negative score: the seat with unbounded life evaluates as the one that is
    // losing. Caught by the `debug-assertions` sweep at seeds 53 and 73 of
    // `--decks all`. Ten thousand is far past any total the evaluator has to
    // tell apart, and it keeps every product below in `i32` for the profiles
    // this ships (`unit` 1 and 10).
    const LIFE_CEILING: i32 = 10_000;
    let life = life.min(LIFE_CEILING);
    if !w.concave_life {
        return life * w.unit;
    }
    /// Tenths of a point per life total, index = life, 0..=20.
    const LIFE_TENTHS: [i32; 21] = [
        0, 20, 40, 60, 80, 90, 100, 110, 120, 130, 140, 148, 156, 164, 172, 180, 184, 188, 192,
        196, 200,
    ];
    const MAX: i32 = LIFE_TENTHS.len() as i32 - 1;
    let tenths = if life <= 0 {
        0
    } else if life <= MAX {
        LIFE_TENTHS[life as usize]
    } else {
        // Past the starting total each extra point is worth the same as the
        // shallowest part of the curve (0.4), not nothing -- Ad Nauseam and
        // friends do care about a big buffer.
        LIFE_TENTHS[MAX as usize] + (life - MAX) * 4
    };
    tenths * w.unit / 10
}

/// Keep-value for deciding which of the bot's *own* permanents to give up (to an
/// edict, a "sacrifice a creature" cost, or a self-vote). Distinct from
/// `permanent_value`, which ranks removal targets: here a token is the ideal
/// thing to lose (it can't be recast and vanishes on leaving), so it sorts
/// strictly below every real card, even a bare land of `permanent_value` 0.
fn sacrifice_keep_value(state: &GameState, id: crate::card::CardId, w: &EvalWeights) -> i32 {
    if state.battlefield_find(id).is_some_and(|c| c.is_token) {
        return -1;
    }
    permanent_value(state, id, w)
}

/// Bot heuristic for `Decision::ChooseTarget` (votes, edicts, free-floating
/// removal). Prefer destroying/exiling an opponent's **most** valuable
/// permanent; if every legal permanent is our own (a "sacrifice/vote your own"
/// choice), give up the **least** valuable. Player targets fall back to the
/// **lowest-life** opponent (most progress toward a kill), then to the first
/// legal option.
fn decide_choose_target(
    state: &GameState,
    seat: usize,
    legal: &[crate::game::types::Target],
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    use crate::game::types::Target;
    let owner = |id: crate::card::CardId| state.battlefield_find(id).map(|c| c.controller);
    // Opponent permanents — hit the biggest.
    let best_opp = legal
        .iter()
        .filter_map(|t| match t {
            Target::Permanent(id) if owner(*id).is_some_and(|o| o != seat) => Some(*id),
            _ => None,
        })
        .max_by_key(|id| permanent_value(state, *id, w));
    if let Some(id) = best_opp {
        return DecisionAnswer::Target(Target::Permanent(id));
    }
    // Only our own permanents are legal — give up the least valuable to keep
    // (tokens first, then lowest-value real cards).
    let worst_own = legal
        .iter()
        .filter_map(|t| match t {
            Target::Permanent(id) if owner(*id) == Some(seat) => Some(*id),
            _ => None,
        })
        .min_by_key(|id| sacrifice_keep_value(state, *id, w));
    if let Some(id) = worst_own {
        return DecisionAnswer::Target(Target::Permanent(id));
    }
    // Player targets: prefer the lowest-life opponent (closest to death, so a
    // "deal damage / lose life" effect makes the most progress toward a kill).
    let best_player = legal
        .iter()
        .filter_map(|t| match t {
            Target::Player(p) if *p != seat => Some(*p),
            _ => None,
        })
        .min_by_key(|p| state.players[*p].life);
    if let Some(p) = best_player {
        return DecisionAnswer::Target(Target::Player(p));
    }
    DecisionAnswer::Target(legal[0].clone())
}

/// Bot heuristic for `Decision::ChooseCreatureType` (Cavern of Souls, the
/// chosen-type tribal payoffs). Name the creature type the bot controls / holds
/// the most of — counting battlefield creatures first (already in play, so the
/// payoff is live) and hand creatures as a tiebreak. A Changeling counts for
/// every type. Falls back to the first suggestion, then Demon, when the bot has
/// no creatures at all.
fn decide_creature_type(
    state: &GameState,
    seat: usize,
    suggestions: &[crate::card::CreatureType],
) -> crate::decision::DecisionAnswer {
    use crate::card::{CreatureType, Keyword};
    use crate::fxhash::HashMap;
    // Weight battlefield presence over hand presence (2:1).
    let mut tally: HashMap<CreatureType, i32> = HashMap::default();
    let mut count = |types: &[CreatureType], changeling: bool, weight: i32| {
        if changeling {
            // A Changeling bumps every type it could enable; give the current
            // leaders a small nudge rather than flooding the tally.
            for t in tally.clone().keys() {
                *tally.entry(*t).or_insert(0) += weight;
            }
        }
        for t in types {
            *tally.entry(*t).or_insert(0) += weight;
        }
    };
    for c in state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_creature()) {
        count(&c.definition.subtypes.creature_types,
            c.definition.keywords.has_kw(&Keyword::Changeling), 2);
    }
    for c in state.players[seat].hand.iter().filter(|c| c.definition.is_creature()) {
        count(&c.definition.subtypes.creature_types,
            c.definition.keywords.has_kw(&Keyword::Changeling), 1);
    }
    let best = tally.into_iter().max_by_key(|(_, n)| *n).map(|(t, _)| t);
    let choice = best
        .or_else(|| suggestions.first().copied())
        .unwrap_or(CreatureType::Demon);
    crate::decision::DecisionAnswer::CreatureType(choice)
}

fn decide_library_search(
    state: &GameState,
    seat: usize,
    candidates: &[(crate::card::CardId, String)],
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    DecisionAnswer::Search(rank_library_search(state, seat, candidates, w).first().copied())
}

/// The library-search picks, best first. Split out of
/// [`decide_library_search`] so the MCTS fetch menu
/// ([`EvalWeights::fetch_arms`]) can offer the runners-up as arms instead
/// of only ever seeing this function's first choice.
///
/// Basics are ranked by **unmet demand**, not by supply alone. The old read
/// scored a basic by "fewest existing sources among the colors it makes"
/// and stopped there, so with a hand full of red spells and one stray green
/// one it fetched the Forest — the colour we own least of is not the colour
/// we most need. Its sibling `ChooseColor` has counted pips in hand since it
/// was written; this now uses the same signal, with supply as the
/// tiebreaker so an already-covered colour still loses to an uncovered one.
///
/// Non-basics (a creature/spell tutor: Fauna Shaman, Imperial Recruiter,
/// Spellseeker) were picked by raw mana value — the biggest hit in the
/// library regardless of whether we can ever cast it. Castability now
/// leads: a hit inside our current land count outranks one we cannot pay
/// for, and mana value only sorts within those groups.
pub(crate) fn rank_library_search(
    state: &GameState,
    seat: usize,
    candidates: &[(crate::card::CardId, String)],
    w: &EvalWeights,
) -> Vec<crate::card::CardId> {
    use crate::mana::{Color, ManaSymbol};
    if candidates.is_empty() {
        return Vec::new();
    }
    const COLORS: [Color; 5] =
        [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    // How many of our lands already tap for each color.
    let mut sources: crate::fxhash::HashMap<Color, usize> = crate::fxhash::HashMap::default();
    for c in state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
    {
        let out = land_color_output(&c.definition);
        for col in COLORS {
            if out.contains(col) {
                *sources.entry(col).or_insert(0) += 1;
            }
        }
    }
    // What our hand is actually asking for, by colored pip.
    let mut demand: crate::fxhash::HashMap<Color, usize> = crate::fxhash::HashMap::default();
    for c in state.players[seat].hand.iter() {
        for sym in c.definition.cost.symbols.iter() {
            if let ManaSymbol::Colored(col) = sym {
                *demand.entry(*col).or_insert(0) += 1;
            }
        }
    }
    let lands = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count() as u32;

    let lib = &state.players[seat].library;
    let mut basics: Vec<(i64, u32, crate::card::CardId)> = Vec::new();
    let mut others: Vec<(u8, std::cmp::Reverse<u32>, crate::card::CardId)> = Vec::new();
    for (id, _) in candidates {
        let Some(card) = lib.iter().find(|c| c.id == *id) else { continue };
        let cmc = card.definition.cost.cmc();
        if card.definition.is_basic() && card.definition.is_land() {
            let out = land_color_output(&card.definition);
            // Best over the colors this land makes: most-wanted first, with
            // the supply count as the tiebreaker (negated so `sort` puts the
            // scarcer source in front).
            let score = COLORS
                .iter()
                .filter(|col| out.contains(**col))
                .map(|col| {
                    let want = demand.get(col).copied().unwrap_or(0) as i64;
                    let have = sources.get(col).copied().unwrap_or(0) as i64;
                    // Pips we cannot currently produce are the whole point;
                    // an uncovered color is worth more than a deep one.
                    let unmet = if have == 0 { want * 2 } else { want };
                    if w.legacy_fetch { (0, -have) } else { (unmet, -have) }
                })
                .max()
                .unwrap_or((0, 0));
            basics.push((-score.0, (-score.1) as u32, *id));
        } else {
            // 0 = castable off the lands we already have, 1 = not yet.
            let castable = if w.legacy_fetch { 0 } else { u8::from(cmc > lands) };
            others.push((castable, std::cmp::Reverse(cmc), *id));
        }
    }
    basics.sort();
    others.sort();
    // A basic land beats a spell hit when both are offered, preserving the
    // long-standing "singleplayer tutors fix mana" behaviour.
    basics
        .into_iter()
        .map(|(_, _, id)| id)
        .chain(others.into_iter().map(|(_, _, id)| id))
        .collect()
}

/// Bot heuristic for `Decision::ChooseCards`. Two cases:
/// - **Put-onto-battlefield from hand** (Sneak Attack / Elvish Piper / Goblin
///   Lackey): every candidate is in the bot's own hand. Cheat in the single
///   biggest creature (highest mana value, then power) — that's the whole point
///   of the effect. Without this the AutoDecider min-0 default declines and the
///   bot never uses the card.
/// - **Exile from graveyards** (Collect Evidence / Fateseal-style): exile every
///   offered card an opponent owns, up to `max`, skipping the bot's own.
fn decide_choose_cards(
    w: &EvalWeights,
    state: &GameState,
    seat: usize,
    prompt: &str,
    candidates: &[(crate::card::CardId, String)],
    min: u32,
    max: u32,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    // **An answer shorter than `min` is rejected, and a rejected answer ends
    // the match**: `drive_bots` counts only accepted actions as progress, so a
    // seat that can only propose one illegal answer stops proposing anything.
    // Each branch below fills `min` from the pile it understands — the hand,
    // the board, our own graveyard — and each has a shape it does not: a
    // candidate in exile, or in a graveyard the owner lookup does not resolve,
    // left the answer empty. Three shipped cards did exactly that at the
    // eighty-fifth pass (see ENGINE_BACKLOG). The engine no longer offers
    // those modals, so this has no live reproducer; it is here because a
    // *well-formed* answer is always available — the candidate list — and
    // answering with nothing is never better than answering with it.
    let fill_to_min = |mut chosen: Vec<crate::card::CardId>| -> DecisionAnswer {
        for (id, _) in candidates {
            if chosen.len() >= min as usize {
                break;
            }
            if !chosen.contains(id) {
                chosen.push(*id);
            }
        }
        DecisionAnswer::Cards(chosen)
    };
    // A sacrifice/discard prompt is a COST — the pick should minimize what
    // we give up, not maximize it. Everything else (draft into hand, tap
    // opposing creatures, exile from graveyards) is upside and keeps the
    // biggest-first / most-hostile-first behavior below.
    let prompt_lc = prompt.to_lowercase();
    let detrimental = prompt_lc.contains("sacrifice") || prompt_lc.contains("discard");
    // Hand-source pick.
    let all_in_hand = !candidates.is_empty()
        && candidates
            .iter()
            .all(|(id, _)| state.players[seat].hand.iter().any(|c| c.id == *id));
    if all_in_hand {
        if detrimental {
            // Shed the least useful cards, and only as many as forced.
            let chosen: Vec<_> = hand_worst_first(state, seat, candidates)
                .into_iter()
                .take(min as usize)
                .collect();
            return fill_to_min(chosen);
        }
        // Beneficial: take the biggest card(s) we can.
        let mut ranked: Vec<(crate::card::CardId, i32, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.players[seat].hand.iter().find(|c| c.id == *id)?;
                Some((*id, c.definition.cost.cmc() as i32, c.definition.power))
            })
            .collect();
        // Biggest first: highest mana value, then highest power.
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        let chosen: Vec<_> = ranked.into_iter().take(max as usize).map(|(id, ..)| id).collect();
        return fill_to_min(chosen);
    }
    // Battlefield-source pick (Archipelagore's "tap up to X target creatures",
    // and similar resolution-time multi-target taps): the AutoDecider declines,
    // so the bot would tap nothing. Prefer opponents' untapped creatures — the
    // biggest threats first — up to the cap. A sacrifice prompt (or a forced
    // pick over only our own permanents) instead gives up the least valuable.
    let all_on_battlefield = candidates
        .iter()
        .all(|(id, _)| state.battlefield.iter().any(|c| c.id == *id));
    if all_on_battlefield {
        let own_least_valuable_first = || -> Vec<crate::card::CardId> {
            let mut own: Vec<(crate::card::CardId, i32)> = candidates
                .iter()
                .filter_map(|(id, _)| {
                    let c = state.battlefield.iter().find(|c| c.id == *id)?;
                    (c.controller == seat).then(|| (*id, sacrifice_keep_value(state, c.id, w)))
                })
                .collect();
            own.sort_by_key(|(_, v)| *v);
            own.into_iter().map(|(id, _)| id).collect()
        };
        if detrimental {
            let chosen: Vec<_> =
                own_least_valuable_first().into_iter().take(min as usize).collect();
            return fill_to_min(chosen);
        }
        let mut ranked: Vec<(crate::card::CardId, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.battlefield.iter().find(|c| c.id == *id)?;
                // Only enemy creatures; prefer untapped (tapping a tapped
                // creature is wasted) and higher power.
                (!state.same_team(c.controller, seat)).then_some((*id, c.power() + if c.tapped { -100 } else { 0 }))
            })
            .collect();
        ranked.sort_by_key(|b| std::cmp::Reverse(b.1));
        let mut chosen: Vec<_> = ranked.into_iter().take(max as usize).map(|(id, _)| id).collect();
        // A forced pick (min ≥ 1) with no enemy candidates — an own-board
        // choice the enemy-first logic can't fill. Give up the least
        // valuable of ours rather than answer empty (which the engine
        // rejects, deadlocking the match on a re-ask loop).
        if chosen.len() < min as usize {
            for id in own_least_valuable_first() {
                if chosen.len() >= min as usize {
                    break;
                }
                if !chosen.contains(&id) {
                    chosen.push(id);
                }
            }
        }
        return fill_to_min(chosen);
    }
    let owner_of = |id: crate::card::CardId| -> Option<usize> {
        state
            .players
            .iter()
            .position(|p| p.graveyard.iter().any(|c| c.id == id))
    };
    let mut chosen: Vec<_> = candidates
        .iter()
        .filter(|(id, _)| owner_of(*id).is_some_and(|o| !state.same_team(o, seat)))
        .map(|(id, _)| *id)
        .take(max as usize)
        .collect();
    // A mandatory pick (min ≥ 1) over our own graveyard — Cache Grab's "put a
    // permanent card milled this way into your hand". Keep the biggest one.
    if chosen.len() < min as usize {
        let mut own: Vec<(crate::card::CardId, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.players[seat].graveyard.iter().find(|c| c.id == *id)?;
                Some((*id, c.definition.cost.cmc() as i32))
            })
            .collect();
        own.sort_by_key(|b| std::cmp::Reverse(b.1));
        chosen = own.into_iter().take((min as usize).max(1)).map(|(id, _)| id).collect();
    }
    fill_to_min(chosen)
}

/// Bot heuristic for a self-discard (cleanup discard-to-hand-size, rummaging,
/// a discard cost): shed the `count` least useful cards so the bot keeps its
/// cheap, castable spells. Surplus lands go first once the bot is no longer
/// mana-light; otherwise the most expensive spells (least likely to be cast
/// soon) are pitched. Ties keep hand order.
fn decide_self_discard(
    state: &GameState,
    seat: usize,
    hand: &[(crate::card::CardId, String)],
    count: u32,
) -> crate::decision::DecisionAnswer {
    crate::decision::DecisionAnswer::Discard(
        hand_worst_first(state, seat, hand).into_iter().take(count as usize).collect(),
    )
}

/// Ascending-usefulness ranking of `offered` hand cards (worst first) —
/// the shed order shared by self-discards and sacrifice/discard-cost
/// `ChooseCards` prompts. Surplus lands go first once the bot is no
/// longer mana-light; otherwise the most expensive spells (least likely
/// to be cast soon) are pitched. Ties keep hand order.
fn hand_worst_first(
    state: &GameState,
    seat: usize,
    offered: &[(crate::card::CardId, String)],
) -> Vec<crate::card::CardId> {
    // Lands already in play: once we have plenty, extra lands in hand are the
    // first thing to pitch; while still mana-light, keep them.
    let lands_in_play = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    // We want about five total land sources; only that many lands in hand are
    // "needed". Excess lands are pitched before spells even while mana-light —
    // holding a fistful of duplicate lands shouldn't cost us our spells, and a
    // flooded bot (≥5 in play) pitches every spare land first.
    let mut lands_still_wanted = 5usize.saturating_sub(lands_in_play);
    // Score each offered card — LOWER is pitched sooner.
    let mut scored: Vec<(i64, crate::card::CardId)> = offered
        .iter()
        .filter_map(|(id, _)| {
            let card = state.players[seat].hand.iter().find(|c| c.id == *id)?;
            let score = if card.definition.is_land() {
                // Keep lands up to the buffer; surplus lands are worth the
                // least so they're pitched first.
                if lands_still_wanted > 0 {
                    lands_still_wanted -= 1;
                    1_000
                } else {
                    -100
                }
            } else {
                // Among spells, keep the cheap (castable) ones; pitch the
                // most expensive first.
                -(card.definition.cost.cmc() as i64)
            };
            Some((score, *id))
        })
        .collect();
    scored.sort_by_key(|(s, _)| *s);
    scored.into_iter().map(|(_, id)| id).collect()
}

/// Order a Scry / Surveil / Rearrange window. `AutoDecider` keeps every
/// card on top — a no-op that wastes every scry in the catalog (the SOS
/// school lands alone surveil in every college deck). The land logic is
/// the discard ranker's (see [`hand_worst_first`]): a land is wanted
/// while total sources — in play plus in hand — run below the
/// five-source buffer, surplus past it; a spell is kept unless its cost
/// sits more than two land drops beyond what the bot can see. Kept cards
/// are ordered most-wanted first (index 0 is the next draw). Cards not
/// in the bot's own library (an opponent-library scry) score neutral and
/// keep the engine's order.
fn decide_scry(
    state: &GameState,
    seat: usize,
    cards: &[(crate::card::CardId, String)],
    mode: crate::decision::ScryMode,
) -> crate::decision::DecisionAnswer {
    use crate::decision::ScryMode;
    let lands_in_play = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    let lands_in_hand =
        state.players[seat].hand.iter().filter(|c| c.definition.is_land()).count();
    let sources = lands_in_play + lands_in_hand;
    // Higher draws sooner; below zero means "don't draw this at all".
    let mut scored: Vec<(i64, crate::card::CardId)> = cards
        .iter()
        .map(|(id, _)| {
            let def = state.players[seat]
                .library
                .iter()
                .find(|c| c.id == *id)
                .map(|c| &c.definition);
            let score = match def {
                None => 0,
                Some(d) if d.is_land() => {
                    if sources < 5 {
                        500
                    } else {
                        -100
                    }
                }
                Some(d) => {
                    let cmc = d.cost.cmc() as i64;
                    if cmc > sources as i64 + 2 { -50 } else { 100 - cmc }
                }
            };
            (score, *id)
        })
        .collect();
    // Stable sort: equal scores keep the engine's order.
    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    match mode {
        // Rearrange has no second bucket — everything stays on top,
        // wanted cards first.
        ScryMode::Rearrange => crate::decision::DecisionAnswer::ScryOrder {
            kept_top: scored.into_iter().map(|(_, id)| id).collect(),
            bottom: vec![],
        },
        ScryMode::Scry | ScryMode::Surveil => {
            let (keep, bin): (Vec<_>, Vec<_>) = scored.into_iter().partition(|(s, _)| *s >= 0);
            crate::decision::DecisionAnswer::ScryOrder {
                kept_top: keep.into_iter().map(|(_, id)| id).collect(),
                bottom: bin.into_iter().map(|(_, id)| id).collect(),
            }
        }
    }
}

/// Pick a mid-resolution mode (`Decision::ChooseMode` — Charm modes, ETB
/// choices like Biblioplex Tomekeeper's prepare/unprepare) by outcome
/// instead of `AutoDecider`'s blanket mode 0: submit each candidate on a
/// clone, resolve to quiescence the same way [`evaluate_action_sequence`]
/// does (`AutoDecider` answers any nested decision), and keep the best
/// material eval. Ties and unevaluable modes keep the lowest index, so
/// the old mode-0 behavior is the floor, never regressed below.
/// Round 53 — judge a mid-resolution target by settled outcome instead
/// of the polarity guess. `decide_choose_target` hard-codes "hit the
/// opponent's biggest, else give up our cheapest": right for removal,
/// backwards for every beneficial resolution effect whose legal set
/// spans both sides, and structurally unable to decline an optional
/// "up to one" target. Rather than settling every legal target (a big
/// board is a big list), the candidates are the corners the polarity
/// question turns on — the biggest and smallest permanent on each side,
/// every legal player, and the decline when the pick is optional —
/// scored by [`settle_answer`], which prices the actual effect with no
/// classifier in between. The heuristic's own pick anchors the
/// comparison and is replaced only on **strict** improvement, so a
/// position where the guess is right (or the difference doesn't settle)
/// returns `None` and plays exactly as before the flag.
fn decide_target_by_outcome(
    state: &GameState,
    seat: usize,
    legal: &[crate::game::types::Target],
    optional: bool,
    w: &EvalWeights,
) -> Option<crate::decision::DecisionAnswer> {
    use crate::decision::DecisionAnswer;
    use crate::game::types::Target;
    let default_pick = match decide_choose_target(state, seat, legal, w) {
        DecisionAnswer::Target(t) => t,
        // The heuristic always answers `Target` on a non-empty list;
        // anything else means the contract changed under us — bail.
        _ => return None,
    };
    // `None` in a candidate slot is the decline.
    let mut picks: Vec<Option<Target>> = vec![Some(default_pick.clone())];
    let mut own_perms: Vec<(crate::card::CardId, i32)> = Vec::new();
    let mut opp_perms: Vec<(crate::card::CardId, i32)> = Vec::new();
    for t in legal {
        match t {
            Target::Permanent(id) => {
                if let Some(c) = state.battlefield_find(*id) {
                    let v = permanent_value(state, *id, w);
                    if c.controller == seat {
                        own_perms.push((*id, v));
                    } else {
                        opp_perms.push((*id, v));
                    }
                }
            }
            Target::Player(_) => picks.push(Some(t.clone())),
        }
    }
    for list in [&opp_perms, &own_perms] {
        for &(id, _) in [
            list.iter().max_by_key(|(_, v)| *v),
            list.iter().min_by_key(|(_, v)| *v),
        ]
        .into_iter()
        .flatten()
        {
            picks.push(Some(Target::Permanent(id)));
        }
    }
    if optional {
        picks.push(None);
    }
    let mut seen: Vec<Option<Target>> = Vec::new();
    picks.retain(|p| {
        if seen.contains(p) {
            false
        } else {
            seen.push(p.clone());
            true
        }
    });
    let score = |p: &Option<Target>| -> Option<i32> {
        let answer = match p {
            Some(t) => DecisionAnswer::Target(t.clone()),
            None => DecisionAnswer::DeclineTarget,
        };
        settle_answer(state, seat, w, answer)
    };
    // No settled baseline, no comparison: keep the heuristic.
    let default_score = score(&picks[0])?;
    let mut best: (i32, &Option<Target>) = (default_score, &picks[0]);
    for p in picks.iter().skip(1) {
        if let Some(s) = score(p)
            && s > best.0
        {
            best = (s, p);
        }
    }
    match best.1 {
        Some(t) if *t == default_pick => None,
        Some(t) => Some(DecisionAnswer::Target(t.clone())),
        None => Some(DecisionAnswer::DeclineTarget),
    }
}

fn decide_mode_by_outcome(
    state: &GameState,
    seat: usize,
    num_modes: usize,
    w: &EvalWeights,
) -> usize {
    let mut best: Option<(i32, usize)> = None;
    for m in 0..num_modes {
        let Some(score) =
            settle_answer(state, seat, w, crate::decision::DecisionAnswer::Mode(m))
        else {
            continue;
        };
        if best.is_none_or(|(b, _)| score > b) {
            best = Some((score, m));
        }
    }
    best.map(|(_, m)| m).unwrap_or(0)
}

/// Submit `answer` to the state's pending decision on a clone, resolve to
/// quiescence (nested decisions answered by the policy table, no
/// expensive re-evaluation), and return the settled material eval for
/// `seat`. `None` when the answer is rejected or resolution won't settle.
/// The shared engine behind every answer-by-outcome policy.
fn settle_answer(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    answer: crate::decision::DecisionAnswer,
) -> Option<i32> {
    let mut g = state.clone();
    dry_run(&mut g, GameAction::SubmitDecision(answer)).ok()?;
    let mut fuel = 64u32;
    loop {
        if g.is_game_over() {
            break;
        }
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            dry_run(&mut g, GameAction::SubmitDecision(answer)).ok()?;
        } else if g.stack.is_empty() {
            break;
        } else {
            dry_run(&mut g, GameAction::PassPriority).ok()?;
        }
        fuel = fuel.checked_sub(1)?;
    }
    let v = eval_material(&g, seat, w);
    super::leaf_capture::maybe(&g, seat, v);
    Some(v)
}

/// Judge a self-costly optional trigger by outcome: settle "yes" and "no"
/// on clones and take the trigger only when accepting evals strictly
/// better. This turns "you may sacrifice a Pest: [payoff]" from a blanket
/// decline (the introspection screen's rule for any self-cost) into a
/// judged trade. `None` when either branch won't settle — the caller
/// keeps the conservative decline.
fn decide_optional_by_outcome(state: &GameState, seat: usize, w: &EvalWeights) -> Option<bool> {
    use crate::decision::DecisionAnswer;
    let yes = settle_answer(state, seat, w, DecisionAnswer::Bool(true))?;
    let no = settle_answer(state, seat, w, DecisionAnswer::Bool(false))?;
    Some(yes > no)
}

fn accumulate_mana_colors(eff: &Effect, set: &mut crate::mana::ColorSet) {
    match eff {
        Effect::AddMana { pool, .. } => accumulate_payload_colors(pool, set),
        // The recursion is the only `call` site and it fires on 1 % of asks;
        // out of line the rest pay no frame. See `(-136)`.
        Effect::Seq(v) => accumulate_mana_colors_seq(v, set),
        _ => {}
    }
}

#[inline(never)]
fn accumulate_mana_colors_seq(v: &[Effect], set: &mut crate::mana::ColorSet) {
    v.iter().for_each(|e| accumulate_mana_colors(e, set));
}

fn accumulate_payload_colors(pool: &ManaPayload, set: &mut crate::mana::ColorSet) {
    match pool {
        ManaPayload::Colors(cs) | ManaPayload::OfColors(cs, _) => {
            cs.iter().for_each(|c| set.insert(*c))
        }
        ManaPayload::OfColor(c, _) => set.insert(*c),
        ManaPayload::AnyOneColor(_)
        | ManaPayload::AnyColors(_)
        | ManaPayload::AnyColorOpponentCouldProduce
        | ManaPayload::AnyColorYouCouldProduce
        | ManaPayload::AnyTypeTriggerSourceProduces
        | ManaPayload::AnyTypeSacrificedLandProduces
        | ManaPayload::DevotionOfChosenColor => *set = crate::mana::ColorSet::all(),
        ManaPayload::Colorless(_) => {}
        // Could produce any single color the rock was set to — treat as
        // potentially any color for the bot's mana-base reasoning.
        ManaPayload::ChosenColorOfSource
        | ManaPayload::DraftNotedColorOfSource
        | ManaPayload::ImprintedCardColor
        | ManaPayload::AnyColorAmongLegendaries
        | ManaPayload::AnyColorAmongExiledWithSource
        | ManaPayload::AnyColorAmongYourPermanents => *set = crate::mana::ColorSet::all(),
        ManaPayload::Restricted(inner, _) | ManaPayload::RestrictedToChosenType(inner)
                    | ManaPayload::RestrictedToChosenTypePlain(inner) => {
            accumulate_payload_colors(inner, set)
        }
    }
}

/// Play `g` forward `turns` turns with both seats on the heuristic policy,
/// then stop. Shared by the mulligan simulation; deliberately small — the
/// MCTS rollout in `super::mcts` does the same job with its own budget and
/// determinisation, and duplicating its knobs here would be two policies
/// to keep in step.
fn play_forward(g: &mut GameState, turns: u32, w: &EvalWeights) {
    let stop_turn = g.turn_number + turns;
    let mut policy: Vec<HeuristicBot> =
        (0..g.players.len()).map(|_| HeuristicBot::with_weights(*w)).collect();
    let mut fuel = 400u32;
    let mut stale = 0u32;
    while !g.is_game_over() && g.turn_number < stop_turn && fuel > 0 && stale < 8 {
        fuel -= 1;
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(g, pending.acting_player(), w, &pending.decision, false)
            };
            if g.perform_action(GameAction::SubmitDecision(answer)).is_err() {
                break;
            }
            continue;
        }
        let mut acted = false;
        for (seat, p) in policy.iter_mut().enumerate() {
            let Some(a) = p.next_action(g, seat) else { continue };
            // Events discarded — recycle the buffer (`recycle_events`).
            if let Ok(events) = g.perform_action(a) {
                g.recycle_events(events);
                acted = true;
                if g.is_game_over() {
                    break;
                }
            }
        }
        if acted { stale = 0 } else { stale += 1 }
    }
}

/// Average settled value of answering the pending mulligan decision with
/// `answer`, over `samples` redeals.
///
/// Every sample reshuffles both libraries and redeals the opponent's
/// hidden zones first, so the sim cannot read the real top of the deck —
/// the same determinisation rule the MCTS rollouts follow. Without it a
/// "keep" that happens to be followed by the perfect draw would look
/// wonderful for reasons the bot is not allowed to know.
///
/// The inner policy runs with `mull_sim` cleared: simulating the mulligan
/// branch reaches another mulligan decision inside the sim, and letting
/// that recurse would nest simulations per level.
fn mulligan_branch_value(
    state: &GameState,
    seat: usize,
    answer: &crate::decision::DecisionAnswer,
    w: &EvalWeights,
    samples: u32,
    horizon: u32,
) -> Option<i64> {
    use rand::seq::SliceRandom;
    let inner = EvalWeights { mull_sim: false, ..*w };
    let mut total: i64 = 0;
    let mut taken = 0u32;
    for s in 0..samples {
        let mut g = state.clone();
        let mut rng = StdRng::seed_from_u64(0x4D55_4C4C ^ ((seat as u64) << 24) ^ s as u64);
        for p in &mut g.players {
            let mut lib = std::mem::take(&mut p.library);
            lib.shuffle(&mut rng);
            p.library = lib;
        }
        determinize_hidden(&mut g, seat, 0x4D55_0000 ^ s as u64);
        if g.perform_action(GameAction::SubmitDecision(answer.clone())).is_err() {
            continue;
        }
        play_forward(&mut g, horizon, &inner);
        total += eval_material(&g, seat, &inner) as i64;
        taken += 1;
    }
    (taken > 0).then(|| total / taken as i64)
}

/// The simulation-based mulligan ([`EvalWeights::mull_sim`]): play both
/// branches forward and keep the better one.
///
/// The London mulligan makes this a fair comparison rather than a
/// threshold guess — the mulligan branch redraws seven and bottoms one,
/// and both costs (a card down, a fresh look) land inside the same sim
/// and the same evaluator, so nothing has to be priced by hand.
fn decide_mulligan_by_sim(
    state: &GameState,
    seat: usize,
    mulligans_taken: usize,
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    // Hard floor: below this a hand is too small to be worth another look
    // whatever the sim says, and it bounds the recursion the engine could
    // otherwise be walked down.
    if mulligans_taken >= 3 || state.players[seat].hand.len() <= 3 {
        return DecisionAnswer::Keep;
    }
    const SAMPLES: u32 = 6;
    const HORIZON: u32 = 4;
    let keep = mulligan_branch_value(state, seat, &DecisionAnswer::Keep, w, SAMPLES, HORIZON);
    let mull =
        mulligan_branch_value(state, seat, &DecisionAnswer::TakeMulligan, w, SAMPLES, HORIZON);
    match (keep, mull) {
        // Ties keep: the mulligan branch has to actually beat the hand in
        // front of us, not merely match it.
        (Some(k), Some(m)) => {
            if m > k { DecisionAnswer::TakeMulligan } else { DecisionAnswer::Keep }
        }
        // A branch that could not be simulated is not evidence; fall back
        // to the shipped predicate rather than guessing.
        _ => decide_mulligan(state, seat, mulligans_taken, &EvalWeights { mull_sim: false, ..*w }),
    }
}

fn decide_mulligan(
    state: &GameState,
    seat: usize,
    mulligans_taken: usize,
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    let hand = &state.players[seat].hand;
    let lands = hand.iter().filter(|c| c.definition.is_land()).count();
    // Curve check: a 2–5-land hand is only worth keeping if it has at least
    // one nonland spell cheap enough to cast in the first few turns — three
    // lands plus four 7-drops is a screwed keep. "Castable early" means a
    // spell whose mana value is within `lands + 1` (a generous early-curve
    // window that still trusts a couple of draws).
    // Color-screw awareness: an early play only counts if the hand's lands
    // can actually produce its colored pips. Three Forests + a hand of blue
    // spells is a screwed keep even though the curve looks fine.
    let producible = hand
        .iter()
        .filter(|c| c.definition.is_land())
        .fold(crate::mana::ColorSet::empty(), |acc, c| {
            acc.union(land_color_output(&c.definition))
        });
    let has_early_play = hand.iter().any(|c| {
        if c.definition.is_land() || c.definition.cost.cmc() as usize > lands + 1 {
            return false;
        }
        let mut need = crate::mana::ColorSet::empty();
        for col in c.definition.cost.colors() {
            need.insert(col);
        }
        need.is_subset_of(producible)
    });
    if !w.mull_quality {
        let keepable = ((2..=5).contains(&lands) && has_early_play) || hand.len() <= 3;
        return if keepable || mulligans_taken >= 2 {
            DecisionAnswer::Keep
        } else {
            DecisionAnswer::TakeMulligan
        };
    }

    // How many spells this hand can actually deploy in the early turns,
    // not merely whether one exists: a two-lander living off a single
    // two-drop is a hand that does nothing from turn three.
    let castable_soon = |extra_lands: usize| -> usize {
        hand.iter()
            .filter(|c| {
                if c.definition.is_land() || c.definition.cost.cmc() as usize > lands + extra_lands
                {
                    return false;
                }
                let mut need = crate::mana::ColorSet::empty();
                for col in c.definition.cost.colors() {
                    need.insert(col);
                }
                need.is_subset_of(producible)
            })
            .count()
    };
    let early_plays = castable_soon(1);
    // What the hand is worth if it does get to cast its spells. Uses the
    // sealed builder's card scorer, which prices bodies, evasion and
    // preparation spells — the same blindness that made the builder pick
    // filler over bombs would otherwise make the mulligan ship bombs.
    let quality: i32 = hand
        .iter()
        .filter(|c| !c.definition.is_land())
        .map(|c| crate::draft::card_quality(&c.definition))
        .sum();
    // The player who isn't the starting player sees one more card before
    // their first real turn, which is what rescues a marginal hand.
    let on_draw = state.active_player_idx != seat;

    let keepable = if hand.len() <= 3 {
        // Below four cards the next mulligan costs more than the hand.
        true
    } else {
        match lands {
            0 | 1 => false,
            2 => early_plays >= 2 || (on_draw && has_early_play),
            3..=5 => has_early_play,
            // Flood is a keep only when the spells justify the risk.
            // Calibrated against concrete cards rather than a round
            // number: a 4/4 flier scores 7 and clears this, three
            // vanilla bears score 6 and don't.
            6 => quality >= 7,
            _ => false,
        }
    };
    if keepable || mulligans_taken >= 2 {
        DecisionAnswer::Keep
    } else {
        DecisionAnswer::TakeMulligan
    }
}

#[cfg(test)]
#[cfg(test)]
fn main_phase_action(state: &GameState, seat: usize) -> GameAction {
    main_phase_action_with(state, seat, true, &EvalWeights::default()).action
}

/// Every cast / activation the bot would consider from `state` this tick,
/// as `(already validated, action)`.
///
/// Extracted from `main_phase_action_with` so a sequence search can ask
/// "and what would I do next?" about a hypothetical state. That question
/// is the whole point of looking more than one play ahead: with four mana
/// the bot could never see that two two-drops beat one four-drop, because
/// it only ever scored a single action against the board.
///
/// The `bool` is whether the candidate has already been through the engine
/// dry-run. Specialty shapes (delve, convoke, kicker, spree, ...) are
/// probed eagerly because building them needs the accept/reject signal —
/// how many cards to delve, how few helpers to tap, the biggest affordable
/// kick. Plain casts are left unvalidated for the caller to probe lazily in
/// score order, which is what keeps a typical tick down to one or two
/// engine probes instead of the whole hand.
/// Which specialty blocks of [`cast_candidates`] a zone can still produce.
///
/// Each block used to take its own cold walk of the hand (or graveyard) to
/// ask "is there a card here for me", and the answer is no for nearly all
/// of them on nearly every board. One warm walk answers all of them at
/// once; a clear bit skips the block outright. Bits over-approximate —
/// each block still applies its own filter — so a set bit costs only the
/// walk it always paid.
mod spec {
    // hand
    pub const DELVE: u32 = 1 << 0;
    pub const CONVOKE: u32 = 1 << 1;
    pub const GIFT: u32 = 1 << 2;
    pub const SPREE: u32 = 1 << 3;
    pub const CONSPIRE: u32 = 1 << 4;
    pub const KICKER: u32 = 1 << 5;
    pub const KICKERS: u32 = 1 << 6;
    pub const MULTIKICKER: u32 = 1 << 7;
    pub const BESTOW: u32 = 1 << 8;
    pub const ADVENTURE: u32 = 1 << 9;
    pub const OMEN: u32 = 1 << 10;
    pub const PROTOTYPE: u32 = 1 << 11;
    pub const SPLIT: u32 = 1 << 12;
    pub const BACK: u32 = 1 << 13;
    pub const ALT_COST: u32 = 1 << 14;
    pub const SPLICE: u32 = 1 << 19;
    // battlefield
    pub const PREPARED: u32 = 1 << 18;
    // graveyard
    pub const GY_AFTERMATH: u32 = 1 << 15;
    pub const GY_RECAST: u32 = 1 << 16;
    pub const GY_BACK: u32 = 1 << 17;
    /// The one graveyard loop that carries flashback, disturb, mayhem,
    /// harmonize and the `from_graveyard` activated abilities.
    pub const GY_LOOP: u32 = GY_RECAST;
}

// **There is no dry-run template here, and there must not be one.** The probes
// below take `state` itself: `GameState::accept_on` clones whatever it is
// handed, and `affordance_probe_template` is `self.clone()` since the library
// strip came off it (see its doc), so a cached template was a clone of `state`
// that every probe then cloned again — one extra `GameState` clone and drop
// per sweep, for a value equal to the one it was made from. `Clone` is
// idempotent here by construction: the CoW zones share either way, the
// `LayerFreeze` resets to unfrozen either way, and the decider round-trips
// through `DeciderKind`. PERF (-67).

/// Run one gated specialty block. Release skips it when its bit is clear;
/// debug runs it anyway and asserts the gate against what it actually
/// emitted, so the whole suite audits the mask on real boards rather than
/// against a re-derived list.
macro_rules! gated_block {
    ($mask:expr, $bit:expr, $out:expr, $body:block) => {{
        let gate = $mask & $bit != 0;
        if gate || cfg!(debug_assertions) {
            let before = $out.len();
            $body
            debug_assert!(
                gate || $out.len() == before,
                concat!("cast_candidates gate ", stringify!($bit), " skipped a real candidate"),
            );
        }
    }};
}

/// The three board facts the candidate blocks used to take a walk each for.
struct BoardFacts {
    /// SOS Repartee: steers the plain-cast block toward creature-aimed
    /// sibling candidates.
    repartee: bool,
    /// A `GrantConvokeToSpells` static is on the board, so the convoke
    /// block's per-card grant test can find something.
    grants_convoke: bool,
    /// Some permanent carries a prepared inset spell.
    prepared: bool,
}

impl BoardFacts {
    fn gather(state: &GameState, seat: usize) -> Self {
        let mut f = BoardFacts { repartee: false, grants_convoke: false, prepared: false };
        for c in state.battlefield.iter() {
            if c.controller != seat {
                continue;
            }
            f.repartee = f.repartee
                || c.definition.triggered_abilities.iter().any(is_repartee_trigger);
            f.grants_convoke = f.grants_convoke
                || c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::GrantConvokeToSpells { .. })
                });
            f.prepared = f.prepared
                || (c.definition.prepare_spell.is_some()
                    && c.counter_count(crate::card::CounterType::Prepared) > 0);
        }
        f
    }
}

/// One walk of `seat`'s hand for every specialty block's entry predicate.
fn hand_specialties(state: &GameState, seat: usize, facts: &BoardFacts) -> u32 {
    use crate::card::Keyword;
    let mut m = 0;
    for c in state.players[seat].hand.iter() {
        let def = &c.definition;
        for kw in &def.keywords {
            m |= match kw {
                Keyword::Delve => spec::DELVE,
                Keyword::Convoke | Keyword::Improvise => spec::CONVOKE,
                Keyword::Conspire => spec::CONSPIRE,
                Keyword::Kicker(_) | Keyword::Offspring(_) => spec::KICKER,
                Keyword::Multikicker(_) => spec::MULTIKICKER,
                Keyword::Splice(..) => spec::SPLICE,
                _ => 0,
            };
        }
        if def.gift.is_some() {
            m |= spec::GIFT;
        }
        if matches!(
            def.effect,
            Effect::Spree { .. }
                | Effect::Tiered { .. }
                | Effect::ChooseModesCast { .. }
                | Effect::ChooseModesByPoints { .. }
        ) {
            m |= spec::SPREE;
        }
        if !def.kicker_options.is_empty() {
            m |= spec::KICKERS;
        }
        if def.bestow.is_some() {
            m |= spec::BESTOW;
        }
        if def.adventure.is_some() {
            m |= spec::ADVENTURE;
        }
        if def.omen.is_some() {
            m |= spec::OMEN;
        }
        if def.prototype.is_some() {
            m |= spec::PROTOTYPE;
        }
        if def.split.as_deref().is_some_and(|s| !s.aftermath) {
            m |= spec::SPLIT;
        }
        if def.back_face.is_some() {
            m |= spec::BACK;
        }
        if def.alternative_cost.is_some() {
            m |= spec::ALT_COST;
        }
    }
    // A convoke *grant* is a board property; the block tested it per hand
    // card, which ran a whole-battlefield walk once per card per tick.
    if facts.grants_convoke {
        m |= spec::CONVOKE;
    }
    m
}

/// One walk of `seat`'s graveyard for the graveyard blocks' predicates.
fn graveyard_specialties(state: &GameState, seat: usize) -> u32 {
    use crate::card::Keyword;
    let mut m = 0;
    for c in state.players[seat].graveyard.iter() {
        let def = &c.definition;
        for kw in &def.keywords {
            m |= match kw {
                Keyword::Flashback(_)
                | Keyword::JumpStart
                | Keyword::GraveyardCast
                | Keyword::Disturb(_)
                | Keyword::Mayhem(_)
                | Keyword::Harmonize(_) => spec::GY_RECAST,
                _ => 0,
            };
        }
        if c.granted_flashback_eot.is_some() || c.granted_harmonize_eot.is_some() {
            m |= spec::GY_RECAST;
        }
        if def.activated_abilities.iter().any(|ab| ab.from_graveyard) {
            m |= spec::GY_RECAST;
        }
        if def.split.as_deref().is_some_and(|s| s.aftermath) {
            m |= spec::GY_AFTERMATH;
        }
        if c.may_cast_back_from_graveyard && def.back_face.is_some() {
            m |= spec::GY_BACK;
        }
    }
    m
}

// `SweepMana<'a>` is invariant over `'a` (its `OnceCell` holds borrows of the
// board), so the shared handle's lifetime has to be named rather than elided.
fn cast_candidates<'a>(
    state: &'a GameState,
    seat: usize,
    w: &EvalWeights,
    // A [`SweepMana`] the caller already owns, so one tick's affordability
    // reads — this function's *and* `sink_facts`' — share one
    // `available_mana` and one `CostStaticSources`. `None` builds a private
    // one.
    shared: Option<&SweepMana<'a>>,
) -> Vec<(GameAction, bool)> {
    // Build list of castable non-land spells. Affordability + target
    // pre-filters reduce the candidate set; the FINAL gate is still the
    // engine dry-run, which discards anything the engine would reject
    // (sorcery timing under Teferi, Damping Sphere mana tax, hexproof
    // targets, stolen permanents, etc.) — but for this main block it runs
    // *lazily* at the pick site below, in descending score order, so a
    // typical tick probes one or two candidates instead of the whole hand.
    //
    // One board walk and one hand walk for every block below — see
    // `BoardFacts` / `spec`. SOS Repartee (`facts.repartee`) steers the
    // plain-cast block toward offering creature-aimed sibling candidates.
    let facts = BoardFacts::gather(state, seat);
    let mask = hand_specialties(state, seat, &facts)
        | graveyard_specialties(state, seat)
        | if facts.prepared { spec::PREPARED } else { 0 };
    let has_repartee = facts.repartee;
    // One producible-mana read for every affordability filter in this
    // function — see `SweepMana`.
    let owned_mana;
    let have_mana: &SweepMana<'_> = match shared {
        Some(h) => h,
        None => {
            owned_mana = SweepMana::new(state, seat);
            &owned_mana
        }
    };
    // Plain loops rather than five `filter`s and two nested `flat_map`s: the
    // bot's hand sweep was the largest single source of `FlatMap::next` in the
    // program (30,578 of its 57,600 calling contexts on a `cube` run — see
    // PERF (-78)), and the adapter machinery is paid per hand card per tick
    // whether or not a candidate comes out.
    let mut unvalidated: Vec<GameAction> = Vec::new();
    for c in state.players[seat].hand.iter() {
        if c.definition.is_land() {
            continue;
        }
        // Pure temp-pump instants are combat tricks: held for the fight
        // window (`pick_combat_trick`), not main-phased where the buff
        // telegraphs and fizzles at cleanup.
        if is_combat_trick(&c.definition) {
            continue;
        }
        // Spree spells need `CastSpellSpree` with chosen modes — a plain
        // `CastSpell` resolves them as a no-op. They get their own candidate
        // block below.
        if matches!(c.definition.effect, Effect::Spree { .. }) {
            continue;
        }
        // A gift card whose base effect is empty (a permanent gift — the payoff
        // is a `SourceGiftPromised`-gated ETB) is wasted by a plain cast; it
        // gets a `CastGift` candidate in the gift block below instead.
        if c.definition.gift.is_some() && matches!(c.definition.effect, Effect::Noop) {
            continue;
        }
        if !can_afford_in_state_with(state, seat, c, w, have_mana) {
            continue;
        }
        // For modal effects (ChooseMode), enumerate each mode so the bot can
        // pick (e.g.) Drown in the Loch's mode 1 (destroy creature) when no
        // opp spell is on the stack to counter. Falls back to `mode: None`
        // (engine defaults to mode 0) for non-modal spells.
        let modes = modal_mode_count(&c.definition.effect);
        let x_value = if x_relevant(&c.definition) {
            Some(max_affordable_x(state, seat, c, w))
        } else {
            None
        };
        for i in 0..modes.unwrap_or(1) {
            let mode = modes.map(|_| i);
            // Pick a target appropriate to the chosen mode (ChooseMode
            // mode-aware filter check happens in the cast paths).
            // Multi-target shapes (Snow Day, Homesickness, Cost of
            // Brilliance, Render Speechless, Vibrant Outburst, …) ask
            // the picker for every slot index used by the effect tree;
            // slots that find no legal target are skipped, matching
            // "up to N target" semantics.
            let mode_effect = mode_branch(&c.definition.effect, mode);
            // Beneficial Auras pick their host explicitly: `Effect::Attach`
            // isn't classified friendly by the generic auto-targeter, so
            // without this a Rancor walks the OPPONENT's creatures first.
            // No friendly host at all → skip the candidate rather than
            // let the fallback pump an opposing creature.
            let (target, additional_targets) = if is_beneficial_aura(&c.definition) {
                match beneficial_aura_host(state, seat, c, w) {
                    Some(t) => (Some(t), Vec::new()),
                    None => continue,
                }
            } else if mode_effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(mode_effect, seat, mode);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, Vec::new())
            };
            // SOS Repartee: with a controlled payoff that wants an
            // instant/sorcery to target a CREATURE, an "any target"
            // spell the auto-targeter aimed at a player also gets a
            // creature-aimed sibling candidate. The outcome eval sees
            // the extra triggers fire when it resolves the sibling, so
            // the swap is judged, not assumed. Decided before the
            // primary is built so `additional_targets` is cloned only
            // when there really are two candidates to hand it to.
            let swap = if has_repartee
                && matches!(target, Some(Target::Player(_)))
                && {
                    use crate::card::CardType;
                    c.definition.card_types.contains(&CardType::Instant)
                        || c.definition.card_types.contains(&CardType::Sorcery)
                } {
                best_hostile_creature_target(state, seat, mode_effect, w)
            } else {
                None
            };
            let sibling = swap.map(|t| GameAction::CastSpell {
                card_id: c.id,
                target: Some(t),
                additional_targets: additional_targets.clone(),
                mode,
                x_value,
            });
            let primary = GameAction::CastSpell {
                card_id: c.id,
                target,
                additional_targets,
                mode,
                // For X-cost spells (Banefire, Earthquake, Wrath of the
                // Skies, Mind Twist, Repeal, …), pump as much generic
                // mana as the pool can spare into X. Casting at X=0
                // was a known dead end — Banefire dealt 0 damage, Mind
                // Twist discarded nothing, Earthquake was a no-op.
                x_value,
            };
            unvalidated.push(primary);
            if let Some(sibling) = sibling {
                unvalidated.push(sibling);
            }
        }
    }

    // Specialty candidates below are probed eagerly (their construction
    // loops need the accept/reject signal — max delve size, biggest
    // affordable kick count, conspire-over-plain preference), so they land
    // in `castable` already validated.
    // `(action, pre-validated)`. A block that probes to *decide* what to emit
    // (convoke's fewest-helpers walk, the kicker subsets, the two that drop the
    // plain cast of the same card) keeps its `would_accept_on` and pushes
    // `true`; the nineteen that used it only as a filter push `false` and let
    // the pick sites validate lazily, in score order, the way the main block
    // already does. Same candidates in the same order — and the winner's probe
    // is now the one the caller adopts, instead of a run thrown away ahead of
    // a second identical one. See PERF's seventy-second pass.
    let mut castable: Vec<(GameAction, bool)> = Vec::new();

    // Delve (CR 702.66): for any hand card with `Keyword::Delve` that the
    // bot can't (yet) afford, try exiling graveyard cards to pay the
    // generic portion. Delve the maximum available (capped at the generic
    // pip total), then let `would_accept` confirm the reduced cost is
    // payable. Appended to the candidate set so the bot actually leverages
    // Treasure Cruise / Dig Through Time / Gurmag Angler off a full bin.
    gated_block!(mask, spec::DELVE, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.keywords.has_kw(&crate::card::Keyword::Delve))
    {
        let generic_pips: u32 = c
            .definition
            .cost
            .symbols
            .iter()
            .filter_map(|s| match s {
                crate::mana::ManaSymbol::Generic(n) => Some(*n),
                _ => None,
            })
            .sum();
        let gy_ids: Vec<CardId> = state.players[seat].graveyard.iter().map(|g| g.id).collect();
        let take = (generic_pips as usize).min(gy_ids.len());
        if take == 0 {
            continue;
        }
        let delve_cards: Vec<CardId> = gy_ids.into_iter().take(take).collect();
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellDelve {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
            delve_cards,
        };
        castable.push((action, false));
    }
    });

    // Convoke / Improvise (CR 702.51 / 702.126): tap untapped creatures
    // (or artifacts) to pay {1} each. Without this the bot never taps a
    // helper, so every convoke card sat in hand at full price. Helpers are
    // capped at the spell's generic pips and drawn from creatures that
    // aren't already committed to combat; `would_accept` is the final gate,
    // so an unaffordable-even-with-help spell just doesn't make the list.
    gated_block!(mask, spec::CONVOKE, castable, {
    for c in state.players[seat].hand.iter() {
        let convoke = c.definition.keywords.has_kw(&crate::card::Keyword::Convoke)
            || (facts.grants_convoke && state.spell_granted_convoke(seat, c));
        let improvise = c.definition.keywords.has_kw(&crate::card::Keyword::Improvise);
        if !convoke && !improvise {
            continue;
        }
        // CR 702.51 — convoke pays colored pips too, so the cap is the whole
        // mana value, not just the generic part. Rank candidates so the least
        // useful bodies tap first: summoning-sick creatures (which can't attack
        // anyway) before healthy ones, then by ascending power.
        let cap = c.definition.cost.cmc() as usize;
        let mut candidates: Vec<(bool, i32, CardId)> = state
            .battlefield
            .iter()
            .filter(|h| {
                h.controller == seat
                    && !h.tapped
                    && ((convoke && h.definition.is_creature())
                        || (improvise && h.definition.is_artifact()))
            })
            .map(|h| (!h.summoning_sick, h.power(), h.id))
            .collect();
        candidates.sort_by_key(|(healthy, pow, _)| (*healthy, *pow));
        candidates.truncate(cap);
        let ranked: Vec<CardId> = candidates.into_iter().map(|(_, _, id)| id).collect();
        if ranked.is_empty() {
            continue;
        }
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        // Tap the fewest helpers that make the cast legal — over-tapping throws
        // away blockers for nothing.
        for n in 1..=ranked.len() {
            let action = GameAction::CastSpellConvoke {
                card_id: c.id,
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: None,
                x_value: None,
                convoke_creatures: ranked[..n].to_vec(),
            };
            if GameState::would_accept_on(state, action.clone()) {
                castable.push((action, true));
                break;
            }
        }
    }
    });

    // Gift (CR 702.165): a spell/permanent with a gift can be cast via
    // `CastGift`, promising the gift to resolve its enhanced `gifted_effect`
    // (or, for permanent gifts, unlock a `SourceGiftPromised`-gated ETB). A
    // plain `CastSpell` only ever gets the base effect, so gift-payoff cards
    // (Scrapshooter, Starfall Invocation) would otherwise be wasted. Offer the
    // promised variant alongside; the gifted effect's target slots are picked
    // from `gifted_effect`, and `would_accept` is the final gate.
    gated_block!(mask, spec::GIFT, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.gift.is_some())
        .filter(|c| can_afford_in_state_with(state, seat, c, w, have_mana))
    {
        let gifted = &c.definition.gift.as_ref().unwrap().gifted_effect;
        // The ETB payoff of a permanent gift lives on the creature, not the
        // gifted_effect, so target off the base effect there; for spell gifts
        // the gifted_effect carries the (possibly broader) target.
        let target_effect =
            if gifted.requires_target() { gifted } else { &c.definition.effect };
        let (target, additional_targets) = if target_effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(target_effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastGift {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Spree (CR 702.172) / Tiered / ChooseModesCast: these must be cast via
    // `CastSpellSpree` with the chosen modes stamped — a plain `CastSpell`
    // skips the modes' additional costs. Offer each single mode, plus the
    // every-mode combination for Spree so a bot with mana up can escalate
    // rather than always firing the cheapest tier; `would_accept` gates
    // affordability, so unpayable combinations drop out on their own.
    gated_block!(mask, spec::SPREE, castable, {
    for c in state.players[seat].hand.iter() {
        let (modes, combo): (Vec<&Effect>, bool) = match &c.definition.effect {
            Effect::Spree { modes } => (modes.iter().map(|m| &m.effect).collect(), true),
            Effect::Tiered { modes } => (modes.iter().map(|m| &m.effect).collect(), false),
            Effect::ChooseModesCast { modes, .. } => (modes.iter().collect(), false),
            // The Season cycle: the budget makes "all modes once" a legal
            // combination whenever the prices fit, so offer it too.
            Effect::ChooseModesByPoints { modes, points, budget } => {
                (modes.iter().collect(), points.iter().map(|p| *p as u32).sum::<u32>() <= *budget as u32)
            }
            _ => continue,
        };
        // Each target-bearing mode consumes exactly one target slot at
        // resolution, in printed order.
        let pick = |picks: Vec<u8>| -> Option<GameAction> {
            let mut slots: Vec<crate::game::types::Target> = Vec::new();
            for &i in &picks {
                let eff = modes[i as usize];
                if eff.requires_target() {
                    let (t, _) = state.auto_targets_for_effect_all_slots(eff, seat, None);
                    slots.push(t?);
                }
            }
            let mut slots = slots.into_iter();
            Some(GameAction::CastSpellSpree {
                card_id: c.id,
                spree_modes: picks,
                target: slots.next(),
                additional_targets: slots.collect(),
                x_value: None,
            })
        };
        let mut candidates: Vec<Vec<u8>> = (0..modes.len() as u8).map(|i| vec![i]).collect();
        if combo && modes.len() > 1 {
            candidates.push((0..modes.len() as u8).collect());
        }
        for picks in candidates {
            let Some(action) = pick(picks) else { continue };
            castable.push((action, false));
        }
    }
    });

    // SOS Prepare — a prepared creature's inset spell is a castable
    // resource: offer `CastPrepareSpell` whenever the cost is payable and
    // the spell has a legal target (`would_accept` gates timing/cost).
    // Casting unprepares the creature; enters-prepared bodies were
    // previously dead weight under bot control.
    gated_block!(mask, spec::PREPARED, castable, {
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        let Some(spell) = c.definition.prepare_spell.as_deref() else { continue };
        if c.counter_count(crate::card::CounterType::Prepared) == 0 {
            continue;
        }
        let (target, additional_targets) = if spell.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&spell.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        // X-cost inset spells (Jadzi's Oracle's Gift, {X}{X}{U}) size X
        // like a hand cast would; at `None` the engine casts them at X=0.
        let x_value = if x_relevant(spell) {
            Some(max_affordable_x_for_def(state, seat, spell, 0, w))
        } else {
            None
        };
        let action = GameAction::CastPrepareSpell {
            creature_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value,
        };
        castable.push((action, false));
    }
    });

    // Splice onto Arcane (CR 702.47): splice every affordable partner onto an
    // Arcane spell the bot is casting anyway. `spliceable` already dry-ran the
    // one-splicer case; `would_accept` re-checks the combined cost, and the
    // spliced clauses' targets are auto-aimed inside `cast_spell_spliced`.
    // Ask for the one category we read. `compute_hand_affordances` would run
    // ~40 categories of per-card dry-runs (each a full state clone) and throw
    // 39 of them away — and `cast_candidates` runs inside the attack
    // simulations, so that waste was multiplied per simulated priority pass.
    // Same result: the sweep's `spliceable` field *is* this call, and it is
    // empty off-priority. Gated like every other specialty block: the sweep it
    // calls is a hand walk that returns empty unless a card in hand has
    // Splice.
    gated_block!(mask, spec::SPLICE, castable, {
    let spliceable = if state.player_with_priority() == seat {
        state.spliceable_hand_cards_on(state, seat)
    } else {
        Vec::new()
    };
    for (host, splicers) in spliceable {
        let (target, additional_targets) = {
            let eff = state.players[seat]
                .hand
                .iter()
                .find(|c| c.id == host)
                .map(|c| c.definition.effect.clone());
            match eff {
                Some(e) if e.requires_target() => {
                    let (t, extras) = state.auto_targets_for_effect_all_slots(&e, seat, None);
                    if t.is_none() {
                        continue;
                    }
                    (t, extras)
                }
                _ => (None, vec![]),
            }
        };
        let action = GameAction::CastSpellSpliced {
            card_id: host,
            splice_cards: splicers,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Conspire (CR 702.78): for any hand card with `Keyword::Conspire`, tap
    // the first two untapped creatures sharing a color with it to copy the
    // spell. The bot conspires whenever it can — the copy is strictly upside
    // for the targeted/value spells it appears on. `would_accept` confirms the
    // base cost is still payable after the (free, tap-only) conspire cost.
    gated_block!(mask, spec::CONSPIRE, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.keywords.has_kw(&crate::card::Keyword::Conspire))
    {
        let spell_colors = c.definition.printed_colors();
        let pair: Vec<CardId> = state
            .battlefield
            .iter()
            .filter(|p| {
                p.controller == seat
                    && !p.tapped
                    && p.definition.is_creature()
                    && state
                        .computed_permanent(p.id)
                        .map(|cp| cp.colors.iter().any(|col| spell_colors.contains(&col)))
                        .unwrap_or(false)
            })
            .map(|p| p.id)
            .take(2)
            .collect();
        if pair.len() < 2 {
            continue;
        }
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellConspire {
            card_id: c.id,
            conspire_creatures: [pair[0], pair[1]],
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(state, action.clone()) {
            // Prefer conspiring over the plain cast of the same card — the
            // extra copy is value the bot's spell eval doesn't otherwise see.
            let cid = c.id;
            unvalidated
                .retain(|a| !matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == cid));
            castable.push((action, true));
        }
    }
    });

    // Kicker / Offspring (CR 702.32 / 702.166): for any hand card with the
    // optional additional cost, offer a `CastSpellKicked` candidate. Targets
    // come from the effect tree, whose slot-0 filter resolves to the kicked
    // (typically broader) branch, so a kicked Tear Asunder can aim at a
    // creature. `would_accept` validates the full base+kicker cost, so this is
    // only added when affordable.
    gated_block!(mask, spec::KICKER, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.has_kicker().is_some())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots_kicked(effect, seat, None, true, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellKicked {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(state, action.clone()) {
            // Offspring (CR 702.166) is pure upside — a free 1/1 token copy
            // with no downside beyond the mana. When affordable, prefer it
            // over the plain cast of the same card (mirrors Conspire above).
            if c.definition.has_offspring().is_some() {
                let cid = c.id;
                unvalidated.retain(
                    |a| !matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == cid),
                );
            }
            castable.push((action, true));
        }
    }
    });

    // CR 702.32b — "Kicker {A} and/or {B}": offer the largest affordable
    // subset (both halves before either alone; each rider is pure upside).
    gated_block!(mask, spec::KICKERS, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| !c.definition.kicker_options.is_empty())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots_kicked(effect, seat, None, true, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let n = c.definition.kicker_options.len() as u8;
        let mut best: Option<GameAction> = None;
        for mask in (1u32..(1 << n)).rev() {
            let kickers: Vec<u8> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
            let action = GameAction::CastSpellKickers {
                card_id: c.id,
                kickers,
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: None,
                x_value: None,
            };
            if GameState::would_accept_on(state, action.clone()) {
                best = Some(action);
                break;
            }
        }
        if let Some(action) = best {
            let cid = c.id;
            unvalidated
                .retain(|a| !matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == cid));
            castable.push((action, true));
        }
    }
    });

    // Multikicker (CR 702.33c): offer the *biggest affordable* kick count
    // (probed 4 → 1 via `would_accept`, which validates base + N×kick).
    gated_block!(mask, spec::MULTIKICKER, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.has_multikicker().is_some())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        for times in (1..=4u32).rev() {
            let action = GameAction::CastSpellMultikicked {
                card_id: c.id,
                times,
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: None,
                x_value: None,
            };
            if GameState::would_accept_on(state, action.clone()) {
                castable.push((action, true));
                break;
            }
        }
    }
    });

    // Bestow (CR 702.103): for any hand card with a bestow cost, offer a
    // `CastBestow` candidate that enchants the bot's sturdiest creature (the
    // host most likely to stick, so the Aura keeps its value). `would_accept`
    // validates the full bestow cost, so this is only added when affordable.
    gated_block!(mask, spec::BESTOW, castable, {
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.bestow.is_some())
    {
        // Prefer the controller's highest-toughness creature as the host.
        let host = state
            .battlefield
            .iter()
            .filter(|b| b.controller == seat && b.definition.is_creature())
            .max_by_key(|b| state.computed_permanent(b.id).map(|cp| cp.toughness).unwrap_or(0))
            .map(|b| b.id);
        let Some(host) = host else { continue };
        let action = GameAction::CastBestow {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(host)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Adventure (CR 715): for any hand card with an adventure half that
    // *targets* something (removal / bounce / pump — Stomp, Petty Theft,
    // Swift End, Boulder Rush), offer a `CastAdventure` candidate. Token /
    // card-draw adventures are skipped here so the bot still prefers playing
    // those cards as creatures; the interactive halves are pure tempo wins.
    gated_block!(mask, spec::ADVENTURE, castable, {
    for c in state.players[seat].hand.iter() {
        let Some(adv) = c.definition.has_adventure() else { continue };
        if !adv.effect.requires_target() {
            continue;
        }
        let (target, additional_targets) =
            state.auto_targets_for_effect_all_slots(&adv.effect, seat, None);
        if target.is_none() {
            continue;
        }
        let action = GameAction::CastAdventure {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Omen (CR 702.183): for any hand card with an Omen half that *targets*
    // something, offer a `CastOmen` candidate (the card shuffles back into the
    // library on resolution, so the creature is still drawable later).
    gated_block!(mask, spec::OMEN, castable, {
    for c in state.players[seat].hand.iter() {
        let Some(omen) = c.definition.has_omen() else { continue };
        if !omen.effect.requires_target() {
            continue;
        }
        let (target, additional_targets) =
            state.auto_targets_for_effect_all_slots(&omen.effect, seat, None);
        if target.is_none() {
            continue;
        }
        let action = GameAction::CastOmen {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Prototype (CR 702.160): for any hand card with a prototype face, offer
    // a `CastPrototype` candidate. The smaller colored cost is often the only
    // affordable line early; the body's ETB auto-targets through the cast path.
    gated_block!(mask, spec::PROTOTYPE, castable, {
    for c in state.players[seat].hand.iter() {
        if c.definition.has_prototype().is_none() {
            continue;
        }
        let action = GameAction::CastPrototype {
            card_id: c.id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Split cards (CR 709): for any hand card with a non-aftermath split,
    // offer a `CastSplitRight` candidate (the left half is already covered by
    // the plain `CastSpell` path). Auto-target the right half's effect.
    gated_block!(mask, spec::SPLIT, castable, {
    for c in state.players[seat].hand.iter() {
        let Some(split) = c.definition.has_split() else { continue };
        if split.aftermath {
            continue;
        }
        let (target, additional_targets) = if split.right.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&split.right.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSplitRight {
            card_id: c.id, target, additional_targets, mode: None, x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Aftermath (CR 702.127): cast the right half of a split card from the
    // graveyard. `would_accept` enforces the graveyard-only + timing rules.
    gated_block!(mask, spec::GY_AFTERMATH, castable, {
    for c in state.players[seat].graveyard.iter() {
        let Some(split) = c.definition.has_split().filter(|s| s.aftermath) else { continue };
        let (target, additional_targets) = if split.right.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&split.right.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastAftermath {
            card_id: c.id, target, additional_targets, mode: None, x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Flashback / Jump-start (CR 702.34/702.103) and Disturb (CR 702.146):
    // recast graveyard cards. `would_accept` enforces zone, timing, and an
    // affordable cost, so these only surface when actually castable.
    gated_block!(mask, spec::GY_LOOP, castable, {
    for c in state.players[seat].graveyard.iter() {
        use crate::card::Keyword;
        let recastable = c.effective_flashback().is_some()
            || c.definition.keywords.has_kw(&Keyword::JumpStart)
            || c.definition.keywords.has_kw(&Keyword::GraveyardCast);
        if recastable {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::CastFlashback {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            };
            // 67 % of the simulator's flashback casts failed their payment
            // (170 of 252 at the seventy-first tip). `flashback_cost_shift`
            // and the graveyard-cast reductions are generic-only, so the
            // colour half of the printed flashback cost is exact.
            let fb = c.effective_flashback();
            let fb_cost = fb.unwrap_or(&c.definition.cost);
            if !colors_coverable(fb_cost, have_mana.get()) {
                continue;
            }
            castable.push((action, false));
        }
        if c.definition.keywords.iter().any(|k| matches!(k, Keyword::Disturb(_))) {
            // The back face goes on the stack; an Aura back needs an enchant
            // target (creature backs need none).
            let back = c.definition.back_face.as_deref();
            let (target, additional_targets) = match back {
                Some(b) if b.effect.requires_target() => {
                    let (t, extras) =
                        state.auto_targets_for_effect_all_slots(&b.effect, seat, None);
                    (t, extras)
                }
                _ => (None, vec![]),
            };
            let needs_target = back.is_some_and(|b| b.effect.requires_target());
            if !(needs_target && target.is_none()) {
                let action = GameAction::CastDisturb {
                    card_id: c.id, target, additional_targets,
                };
                castable.push((action, false));
            }
        }
        // Mayhem (CR 702.187): if the card was discarded this turn and has a
        // mayhem cost, offer a graveyard cast for it. `would_accept` enforces
        // the discarded-this-turn gate, cost, and timing.
        if c.definition.mayhem_cost().is_some() {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::CastMayhem {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            };
            castable.push((action, false));
        }
        // Harmonize (CR 702.180): cast from the graveyard for the harmonize
        // cost. The bot doesn't tap a creature to discount (a value call it
        // can't weigh well); `would_accept` enforces cost / timing.
        if c.effective_harmonize().is_some() {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::CastHarmonize {
                card_id: c.id, tap_creature: None, target, additional_targets, mode: None, x_value: None,
            };
            castable.push((action, false));
        }
        // Graveyard-activated abilities (CR 702.84 Unearth, and the SOS
        // "return this from your graveyard" cycle): offer each `from_graveyard`
        // activated ability. `would_accept` enforces zone / cost / sorcery
        // timing, so these only surface when actually activatable.
        for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
            if !ab.from_graveyard {
                continue;
            }
            let (target, additional_targets) = if ab.effect.requires_target() {
                let (t, extras) = state.auto_targets_for_effect_all_slots(&ab.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::ActivateAbility {
                card_id: c.id, ability_index: idx, target, additional_targets, x_value: None, mode: None,
            };
            castable.push((action, false));
        }
    }
    });

    // MDFC back faces (CR 712): cast the back of a hand MDFC, or the back of a
    // graveyard MDFC carrying the one-shot `may_cast_back_from_graveyard`
    // permission (Pestilent Cauldron's "cast it transformed"). Targets come
    // from the BACK face's effect; `would_accept` enforces cost / timing /
    // zone, so these only surface when actually castable. (Land backs are
    // played via PlayLandBack, handled by the land logic, so they're skipped
    // here.)
    gated_block!(mask, spec::BACK | spec::GY_BACK, castable, {
    let back_sources = state.players[seat].hand.iter().chain(
        state.players[seat]
            .graveyard
            .iter()
            .filter(|c| c.may_cast_back_from_graveyard),
    );
    for c in back_sources {
        let Some(back) = c.definition.back_face.as_deref() else { continue };
        if back.is_land() {
            continue;
        }
        let (target, additional_targets) = if back.effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(&back.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellBack {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Adventure creature (CR 715) and plotted cards (CR 702.170d): cast the
    // creature half / a plotted card from exile. `would_accept` enforces the
    // later-turn + sorcery-speed timing, so this is only offered when legal.
    for c in state.exile.iter().filter(|c| c.owner == seat) {
        let action = if c.on_adventure && c.definition.is_land() {
            // CR 715.3d — a land half is played, not cast (FIN's Town cycle).
            GameAction::PlayLand(c.id)
        } else if c.on_adventure {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None)
            } else {
                (None, vec![])
            };
            GameAction::CastAdventureCreature {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            }
        } else if state.plotted_cards.contains(&c.id) {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None)
            } else {
                (None, vec![])
            };
            GameAction::CastPlotted {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            }
        } else {
            continue;
        };
        castable.push((action, false));
    }

    // Mana-only alternative costs (Dash CR 702.110, Blitz 702.152,
    // Spectacle 702.111): for any hand card whose `alternative_cost` is paid
    // purely with mana (no pitch/sacrifice/graveyard/life rider), offer a
    // `CastSpellAlternative` candidate. `would_accept` validates the alt cost
    // and its `condition` gate (e.g. Spectacle's opponent-lost-life), so a
    // Skewer the Critics is only offered for {R} once an opponent has bled.
    gated_block!(mask, spec::ALT_COST, castable, {
    for c in state.players[seat].hand.iter().filter(|c| {
        c.definition.alternative_cost.as_ref().is_some_and(|a| {
            a.exile_filter.is_none()
                && a.sacrifice_permanents.is_none()
                && a.exile_from_graveyard_count == 0
                && a.life_cost == 0
                && !a.evoke_sacrifice
                // Offering (CR 702.48) sacrifices one of the bot's own
                // creatures for a tempo cut it rarely wants — cast normally.
                && a.offering.is_none()
        })
    }) {
        let effect = c
            .definition
            .alternative_cost
            .as_ref()
            .and_then(|a| a.effect_override.as_ref())
            .unwrap_or(&c.definition.effect);
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellAlternative {
            card_id: c.id,
            pitch_card: None,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        castable.push((action, false));
    }
    });

    // Non-mana activated abilities as candidates (flag): without this,
    // ability usage is a handful of hand-written classes and everything
    // else — Sundering Archaic's {2} exile in the recorded games — can
    // never be chosen at any valuation. Cheap prefilter (printed,
    // non-mana, non-X, mana value within pool + untapped lands),
    // auto-aimed targets, lazily validated like the main block, capped
    // at the two cheapest: this fn runs inside the attack simulations.
    if w.ability_arms {
        let available = state.players[seat].mana_pool.total()
            + state
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && c.definition.is_land() && !c.tapped)
                .count() as u32;
        let mut ability_cands: Vec<(u32, GameAction)> = Vec::new();
        for c in state.battlefield.iter().filter(|c| c.controller == seat) {
            for (i, ab) in c.definition.activated_abilities.iter().enumerate() {
                if crate::game::actions::is_mana_ability_public(&ab.effect)
                    || ab.mana_cost.has_x()
                {
                    continue;
                }
                let cmc = ab.mana_cost.cmc();
                if cmc > available {
                    continue;
                }
                // `available` is a count of untapped lands: it says nothing
                // about *which* colours they make, so a `{1}{B}` ability was
                // offered off two Mountains. 59 % of the simulator's non-mana
                // activations failed their payment before this line
                // (734 `restore_payment_state` against 1,242 snapshots at the
                // seventy-first tip), and unlike a cast nothing pre-filtered
                // them. Sound against the *printed* cost: every adjustment
                // `activate_ability_inner` applies is generic-only.
                if !colors_coverable(&ab.mana_cost, have_mana.get()) {
                    continue;
                }
                let (target, additional_targets) = if ab.effect.requires_target() {
                    let (t, extras) =
                        state.auto_targets_for_effect_all_slots(&ab.effect, seat, None);
                    if t.is_none() {
                        continue;
                    }
                    (t, extras)
                } else {
                    (None, vec![])
                };
                ability_cands.push((
                    cmc,
                    GameAction::ActivateAbility {
                        card_id: c.id,
                        ability_index: i,
                        target,
                        additional_targets,
                        mode: None,
                        x_value: None,
                    },
                ));
            }
        }
        ability_cands.sort_by_key(|(cmc, _)| *cmc);
        unvalidated.extend(ability_cands.into_iter().take(2).map(|(_, a)| a));
    }

    let mut out: Vec<(GameAction, bool)> = Vec::with_capacity(castable.len() + unvalidated.len());
    out.extend(castable);
    out.extend(unvalidated.into_iter().map(|a| (a, false)));
    // Ward gate, applied once for every candidate block above: a cast
    // aimed at a warded permanent whose tax the bot can't pay after the
    // spell's own cost resolves as a counter, not a cast (the engine
    // auto-pays ward and `would_accept` can't see the trigger fail).
    out.retain(|(a, _)| ward_gate_ok(state, seat, a));
    out
}

/// Which of [`main_phase_action_with`]'s fallback generators a board can still
/// produce an action for.
///
/// The tail below the cast block is two hand loops and twenty-two `pick_*`
/// generators, and every one of them took its own walk of the seat's
/// battlefield — three of them the whole library, every graveyard card, or
/// every opposing creature's *computed* power — to ask "is there anything here
/// for me". The answer is no for nearly all of them on nearly every tick. One
/// walk answers all of them at once; a clear bit skips the generator outright.
/// Bits over-approximate — each generator still applies its own filter — so a
/// set bit costs only the walk it always paid. Same device as [`spec`] for
/// `cast_candidates`, and [`gated_pick`] audits it the same way.
mod sink {
    // hand
    pub const MORPH: u32 = 1 << 0;
    pub const DISCARD_ACT: u32 = 1 << 1;
    // battlefield permanents
    pub const LOYALTY: u32 = 1 << 2;
    pub const CREW: u32 = 1 << 3;
    pub const SADDLE: u32 = 1 << 4;
    pub const EQUIP: u32 = 1 << 5;
    pub const LANDER: u32 = 1 << 6;
    pub const FACE_DOWN: u32 = 1 << 7;
    // graveyard
    pub const GY_RECUR: u32 = 1 << 8;
    // shapes of the activated abilities the seat can use right now, printed
    // and granted (see `usable_abilities`)
    pub const AB_ATTACH: u32 = 1 << 9;
    pub const AB_REANIMATE: u32 = 1 << 10;
    pub const AB_DAMAGE: u32 = 1 << 11;
    pub const AB_REACH: u32 = 1 << 12;
    pub const AB_DESTROY: u32 = 1 << 13;
    pub const AB_SAC_DESTROY: u32 = 1 << 14;
    pub const AB_TEAM_PUMP: u32 = 1 << 15;
    pub const AB_DRAW: u32 = 1 << 16;
    pub const AB_GRANT_PLAY: u32 = 1 << 17;
    pub const AB_PREPARES: u32 = 1 << 18;
    pub const AB_SAC: u32 = 1 << 19;
    pub const AB_SELF_COUNTER: u32 = 1 << 20;
    pub const AB_TOKEN: u32 = 1 << 21;
    pub const AB_ENERGY: u32 = 1 << 22;
}

/// Run one gated fallback generator, returning its action when it has one.
/// Release skips it when its bit is clear; debug runs it anyway and asserts the
/// gate against what it actually returned, so the whole suite audits the mask
/// on real boards rather than against a re-derived list.
macro_rules! gated_pick {
    ($state:expr, $mask:expr, $bit:expr, $call:expr) => {{
        let gate = $mask & $bit != 0;
        if gate || cfg!(debug_assertions) {
            let picked = $call;
            debug_assert!(
                gate || picked.is_none(),
                concat!("main_phase gate ", stringify!($bit), " skipped a real action: {}"),
                gate_blame($state, picked.as_ref()),
            );
            if gate && let Some(action) = picked {
                return BotStep::plain(action);
            }
        }
    }};
}

/// Name what a `gated_pick!` gate skipped, for the assertion above.
///
/// A gate audit that can only say *which bit* was clear costs a rebuild every
/// time it fires — the same argument `CRAB_CAP_DIAG` is built on. This names the
/// source card, its ability index and the printed cost, which is what the
/// soundness question is actually about: `sink_facts` skips an ability whose
/// coloured pips `available_mana` cannot cover, and `would_accept` is the
/// authority on whether that was right.
#[cfg(debug_assertions)]
fn gate_blame(state: &GameState, action: Option<&GameAction>) -> String {
    let Some(action) = action else { return String::from("(none)") };
    let GameAction::ActivateAbility { card_id, ability_index, .. } = action else {
        return format!("{action:?}");
    };
    let Some(card) = state.find_card_anywhere(*card_id) else {
        return format!("{action:?} (card gone)");
    };
    let cost = card
        .definition
        .activated_abilities
        .get(*ability_index)
        .map(|ab| format!("{:?} gy={} hand={} exile={}", ab.mana_cost.symbols, ab.from_graveyard, ab.from_hand, ab.from_exile))
        .unwrap_or_else(|| String::from("(granted)"));
    let zone = if state.battlefield.iter().any(|c| c.id == *card_id) { "battlefield" } else { "elsewhere" };
    let have = available_mana(state, card.controller);
    format!(
        "{} ability {ability_index} in {zone}, printed cost {cost}, available total {} by_color {:?}",
        card.definition.name, have.total, have.by_color
    )
}

/// Release build: the assertion is compiled out, so the blame string is never
/// built.
///
/// ⚠ **It must still be `Display`.** `debug_assert!` expands to
/// `if cfg!(debug_assertions) { assert!(..) }` — the body is *dead* in
/// release, not *absent*, so its format arguments are still type-checked and
/// a `()`-returning stub is `error[E0277]` on every profile with
/// `debug-assertions` off. `cargo check` and the suite both run with them on,
/// so nothing but a release build catches it.
#[cfg(not(debug_assertions))]
fn gate_blame(_state: &GameState, _action: Option<&GameAction>) -> &'static str {
    ""
}

/// The shape bits one activated ability contributes to [`sink_facts`]. Each
/// arm calls the same predicate its generator does, so the two cannot drift.
fn ability_sink_bits(ab: &crate::effect::ActivatedAbility) -> u32 {
    use crate::effect::Selector;
    let mut m = 0;
    if ab.sac_cost || ab.sac_other_filter.is_some() {
        m |= sink::AB_SAC;
    }
    if ab.energy_cost > 0 || matches!(ab.effect, Effect::PayEnergy { .. }) {
        m |= sink::AB_ENERGY;
    }
    match &ab.effect {
        Effect::Destroy { .. } | Effect::DestroyNoRegen { .. } => {
            m |= if ab.sac_cost { sink::AB_SAC_DESTROY } else { sink::AB_DESTROY };
        }
        Effect::DealDamage { .. } => m |= sink::AB_DAMAGE,
        Effect::Draw { who: Selector::You, .. } => m |= sink::AB_DRAW,
        Effect::PumpPT { what: Selector::EachPermanent(_), .. } => m |= sink::AB_TEAM_PUMP,
        Effect::AddCounter { what: Selector::This, .. } => m |= sink::AB_SELF_COUNTER,
        Effect::Attach { .. } => m |= sink::AB_ATTACH,
        _ => {}
    }
    if ab.effect.is_adapt() {
        m |= sink::AB_SELF_COUNTER;
    }
    if ability_makes_token(&ab.effect) {
        m |= sink::AB_TOKEN;
    }
    if ability_grants_play(&ab.effect) {
        m |= sink::AB_GRANT_PLAY;
    }
    if ability_prepares_target(&ab.effect) {
        m |= sink::AB_PREPARES;
    }
    if ability_reach_amount(&ab.effect).is_some() {
        m |= sink::AB_REACH;
    }
    if effect_reanimates_from_graveyard(&ab.effect) {
        m |= sink::AB_REANIMATE;
    }
    m
}

/// One walk of the seat's hand, battlefield and graveyard for every fallback
/// generator's entry predicate — see [`sink`].
fn sink_facts(state: &GameState, seat: usize, have: &SweepMana<'_>) -> u32 {
    use crate::card::{ArtifactSubtype, Keyword};
    let mut m = 0;
    for c in state.players[seat].hand.iter() {
        for kw in &c.definition.keywords {
            if matches!(
                kw,
                Keyword::Morph(_)
                    | Keyword::MorphCost(_)
                    | Keyword::Megamorph(_)
                    | Keyword::Disguise(_)
            ) {
                m |= sink::MORPH;
            }
        }
        if c.definition.discard_activated.is_some() {
            m |= sink::DISCARD_ACT;
        }
    }
    // One grant scan for the whole tail. Six generators built their own; the
    // gates now skip all six on a board with nothing for them.
    let scan = state.grant_scan();
    let mut scavenge_grant = false;
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        let def = &c.definition;
        if def.is_planeswalker() {
            m |= sink::LOYALTY;
        }
        if def.crew_cost().is_some() {
            m |= sink::CREW;
        }
        if def.saddle_cost().is_some() {
            m |= sink::SADDLE;
        }
        if def.is_equipment() && def.has_equip().is_some() {
            m |= sink::EQUIP;
        }
        if def.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Lander) {
            m |= sink::LANDER;
        }
        if c.face_down && c.face_up_def.is_some() {
            m |= sink::FACE_DOWN;
        }
        scavenge_grant = scavenge_grant
            || def.static_abilities.iter().any(|sa| {
                matches!(
                    sa.effect,
                    crate::effect::StaticEffect::GraveyardCreaturesHaveScavenge
                )
            });
        for (_, ab) in usable_abilities(state, c, &scan) {
            // An ability whose *colour* pips this board cannot cover can never
            // be activated, so it must not light its sink bit: the gate is
            // what keeps the whole `gated_pick!` chain below from walking the
            // battlefield, building an action and paying for a ~50 k-Ir
            // dry-run probe on it. 59 % of the simulator's non-mana
            // activations failed their payment before this line. Sound
            // against the printed cost — see `colors_coverable`.
            // The coloured-pip test first, off the printed cost alone: it is
            // what decides whether the sweep's `available_mana` has to be
            // forced at all, and on an archetype pool whose abilities are
            // `{T}` or generic that read is pure cost (measured +0.292 % of
            // `fixed` when this gate was unconditional, against -0.562 % of
            // `cube`).
            if ab.mana_cost.symbols.iter().any(|sym| matches!(sym, crate::mana::ManaSymbol::Colored(_)))
                && !colors_coverable(&ab.mana_cost, have.get())
            {
                continue;
            }
            m |= ability_sink_bits(&ab);
        }
    }
    if state.players[seat].energy == 0 {
        m &= !sink::AB_ENERGY;
    }
    for c in state.players[seat].graveyard.iter() {
        if c.definition.activated_abilities.iter().any(|ab| ab.from_graveyard)
            || !c.granted_activated_abilities.is_empty()
            || !c.granted_activated_eot.is_empty()
            || (scavenge_grant && c.definition.is_creature())
        {
            m |= sink::GY_RECUR;
            break;
        }
    }
    m
}

/// Cast a hand card face down for {3} (CR 702.36 Morph / 702.166 Disguise).
fn pick_face_down_cast(state: &GameState, seat: usize, probe: &GameState) -> Option<GameAction> {
    use crate::card::Keyword;
    state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.keywords.iter().any(|k| {
                matches!(
                    k,
                    Keyword::Morph(_)
                        | Keyword::MorphCost(_)
                        | Keyword::Megamorph(_)
                        | Keyword::Disguise(_)
                )
            })
        })
        .map(|c| GameAction::CastFaceDown { card_id: c.id })
        .find(|a| GameState::would_accept_on(probe, a.clone()))
}

/// Fire a discard-activated hand ability (Magma Opus's `{U/R}{U/R}, Discard`).
fn pick_discard_ability(state: &GameState, seat: usize, probe: &GameState) -> Option<GameAction> {
    state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.discard_activated.is_some())
        .map(|c| GameAction::ActivateDiscardAbility { card_id: c.id })
        .find(|a| GameState::would_accept_on(probe, a.clone()))
}

fn main_phase_action_with(
    state: &GameState,
    seat: usize,
    scored: bool,
    w: &EvalWeights,
) -> BotStep {
    // One producible-mana read per tick, shared by `cast_candidates`' hand
    // filter and `sink_facts`' ability gate. Lazy (`SweepMana`), so a tick
    // that reaches neither pays nothing — pass 40 measured +0.35 % for an
    // eager read here.
    let have_mana = SweepMana::new(state, seat);

    // NOTE: the bot deliberately does *not* pre-tap its mana sources here.
    //
    // It used to: one untapped land per tick until the board was empty,
    // which is what made `can_afford_in_state` work off the floating pool.
    // The cost was severe and invisible to the unit tests (which all
    // pre-fill `mana_pool` by hand). Pools empty at every step and phase
    // boundary (CR 500.4), so tapping out in the precombat main left the
    // bot with nothing for its own postcombat main and nothing at all on
    // the opponent's turn: measured over 20 bot-vs-bot games, zero of 1366
    // opponent-turn priority windows had a single untapped land, and 100 %
    // of spells were cast in the precombat main. `pick_stack_response`,
    // `pick_ability_counter_response`, `pick_combat_trick` and the
    // end-of-turn instant window below were all dead code in real play.
    //
    // Now the engine's auto-tap pays each cast from only the sources it
    // needs (`try_pay_with_auto_tap`, which `would_accept_on` already runs
    // for every candidate), so leftover mana simply stays untapped and is
    // still there at instant speed.
    if w.legacy_pretap
        && let Some(id) = state
            .battlefield
            .iter()
            .find(|c| c.controller == seat && c.definition.is_land() && !c.tapped)
            .map(|c| c.id)
    {
        let action = GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        };
        if let Some(g) = GameState::accept_on(state, action.clone()) {
            return BotStep { action, settled: Some(Box::new(g)) };
        }
    }

    // Everything castable this tick — see `cast_candidates`.
    let pool = cast_candidates(state, seat, w, Some(&have_mana));

    // Play a land if possible — gated through `would_accept` for
    // the same reason (the engine enforces sorcery timing, lands-
    // played-this-turn, etc.). Use the game-level helper so an
    // Exploration / Azusa-style ExtraLandPerTurn static lets the bot
    // play a second land in the same turn (CR 305.2). Asked once for all
    // three land blocks: `state` is `&GameState` and each block returns
    // when it fires, so the answer cannot change between them.
    // The land-play probes below are `would_accept_on` — the dry-run's state
    // is the same one a successful `perform_action(action)` on the driver
    // produces. `accept_on` keeps it; the driver adopts it and skips its own
    // execution (see `Bot::next_action_settled`).
    let can_play_land = state.can_player_play_land(seat);
    if can_play_land
        && let Some(land_id) = pick_land_to_play(state, seat, w)
    {
        let action = GameAction::PlayLand(land_id);
        if let Some(g) = GameState::accept_on(state, action.clone()) {
            return BotStep { action, settled: Some(Box::new(g)) };
        }
    }

    // Crucible of Worlds / Ramunap Excavator: replay a land from the
    // graveyard if no hand land was played (CR 305 land-from-gy permission).
    if can_play_land
        && state.player_may_play_lands_from_graveyard(seat)
        && let Some(land) =
            state.players[seat].graveyard.iter().find(|c| c.definition.is_land())
    {
        let action = GameAction::PlayLandFromGraveyard(land.id);
        if let Some(g) = GameState::accept_on(state, action.clone()) {
            return BotStep { action, settled: Some(Box::new(g)) };
        }
    }

    // Impulse exile (Light Up the Stage, Gonti Night Minister): a land the
    // seat has a may-play grant on is played from exile before it expires.
    if can_play_land
        && let Some(land) = state.exile.iter().find(|c| {
            c.definition.is_land() && c.may_play_until.is_some_and(|perm| perm.player == seat)
        })
    {
        let action = GameAction::PlayLand(land.id);
        if let Some(g) = GameState::accept_on(state, action.clone()) {
            return BotStep { action, settled: Some(Box::new(g)) };
        }
    }

    if !pool.is_empty() {
        // Magecraft-aware bias: if the bot controls a permanent with a
        // magecraft trigger, prefer instants/sorceries so the trigger
        // fires — IS candidates sort first, and finalist collection stops
        // at the IS/non-IS boundary once an IS line has validated (the
        // lazy-probe equivalent of the old only-IS pool restriction).
        // Push (claude/modern_decks batch 202).
        let has_magecraft = state.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.triggered_abilities.iter().any(is_magecraft_trigger)
        });
        let is_is_spell = |a: &GameAction| {
            matches!(a, GameAction::CastSpell { card_id, .. } if is_instant_or_sorcery_in_hand(state, seat, *card_id))
        };
        if !scored {
            // Uniform baseline: validate everything (the historical
            // behavior) and sample.
            let valid: Vec<GameAction> = pool
                .into_iter()
                .filter(|(a, ok)| *ok || GameState::would_accept_on(state, a.clone()))
                .map(|(a, _)| a)
                .collect();
            if !valid.is_empty() {
                let only_is: Vec<GameAction> = if has_magecraft {
                    valid.iter().filter(|a| is_is_spell(a)).cloned().collect()
                } else {
                    Vec::new()
                };
                let pick = if only_is.is_empty() { &valid } else { &only_is };
                return BotStep::plain(pick[jitter_below(pick.len())].clone());
            }
        } else {
            // Scored pick: rank by static score (+ jitter so exact ties
            // don't collapse into one deterministic line — see
            // `score_candidate`), walk in rank order probing unvalidated
            // candidates lazily, and hand the top few survivors to the
            // outcome evaluation for the final call. Most ticks this
            // probes 1-3 candidates instead of the whole hand.
            //
            // SOS on-cast payoff nudges — score-shaped siblings of the
            // magecraft partition (nudges compose; partitions don't):
            // * Opus: a controlled Opus permanent upgrades its trigger
            //   when 5+ mana was spent on the cast.
            // * Increment: a controlled Increment body grows when the
            //   cast's mana spent clears its smaller stat.
            // * Infusion: an Infusion-gated card in hand unlocks on any
            //   lifegain, so lifegain casts go first while none has been
            //   gained this turn.
            // Each nudge is 8 = two score points ≈ one mana of cast
            // value on `score_candidate`'s ×4 scale — a tiebreaker, not
            // an override.
            let has_opus = state.battlefield.iter().any(|c| {
                c.controller == seat
                    && c.definition.triggered_abilities.iter().any(is_opus_trigger)
            });
            let increment_bar = increment_threshold(state, seat);
            let wants_lifegain = state.players[seat].life_gained_this_turn == 0
                && state.players[seat]
                    .hand
                    .iter()
                    .any(|c| card_infusion_gated(&c.definition));
            let mut ranked: Vec<(i32, GameAction, bool)> = pool
                .into_iter()
                .map(|(a, ok)| {
                    let mut s =
                        score_candidate(state, seat, &a, w) * 4 + jitter_below(4) as i32;
                    let spent = if has_opus || increment_bar.is_some() {
                        cast_mana_spent(state, seat, &a)
                    } else {
                        0
                    };
                    if has_opus && spent >= 5 {
                        s += 8 * w.unit;
                    }
                    if increment_bar.is_some_and(|bar| spent >= bar) {
                        s += 8 * w.unit;
                    }
                    if wants_lifegain && cast_gains_life(state, seat, &a) {
                        s += 8 * w.unit;
                    }
                    (s, a, ok)
                })
                .collect();
            if has_magecraft {
                ranked.sort_by_key(|&(s, ref a, _)| (!is_is_spell(a), std::cmp::Reverse(s)));
            } else {
                ranked.sort_by_key(|&(s, _, _)| std::cmp::Reverse(s));
            }
            const EVAL_TOP: usize = 3;
            let mut finalists: Vec<Finalist> = Vec::new();
            for (s, a, ok) in ranked {
                if finalists.len() >= EVAL_TOP {
                    break;
                }
                if has_magecraft && !finalists.is_empty() && !is_is_spell(&a) {
                    break;
                }
                if ok {
                    finalists.push(Finalist { score: s, action: a, settled: None });
                } else if let Some(g) = GameState::accept_on(state, a.clone()) {
                    finalists.push(Finalist {
                        score: s,
                        action: a,
                        settled: Some(Box::new(g)),
                    });
                }
            }
            if let Some(best) = pick_by_outcome(state, seat, finalists, w) {
                // Forge's summon-sick gate (`SpellAbilityPicker`): if the
                // winning line's only gain this turn is a body that can't
                // attack, it is worth exactly as much after combat — and
                // played then it costs the opponent a turn of information
                // and leaves the mana up in between. Hold it.
                //
                // Applied to the *winner* only, deliberately. Screening
                // every candidate this way would have the bot pick some
                // lesser non-creature line now and then have no mana left
                // for the creature it actually wanted in the second main.
                let own_main = state.active_player_idx == seat
                    && matches!(
                        state.step,
                        TurnStep::PreCombatMain | TurnStep::PostCombatMain
                    );
                // Hold a creature that can't attack yet until the second
                // main; hold an instant-speed line until the opponent's
                // turn. Both ask the same question -- "is this worth the
                // same later?" -- and both only fire on our own main phase,
                // where there is a later to wait for.
                let gate = own_main
                    && ((w.hold_sick && state.step == TurnStep::PreCombatMain)
                        || (w.hold_instants
                            && castable_at_instant_speed(state, seat, &best.action)));
                if gate
                    && !improves_this_turn(
                        state,
                        seat,
                        &best.action,
                        best.settled.as_deref(),
                        w,
                    )
                {
                    return BotStep::plain(GameAction::PassPriority);
                }
                // SOS Converge: float a missing color first so the cast
                // counts it — see `pick_converge_prefloat`. A prefloat tap
                // must run before the cast, so we drop the settled state
                // (the tap invalidates the cast's dry-run).
                if let Some(tap) = pick_converge_prefloat(state, seat, &best.action) {
                    return BotStep::plain(tap);
                }
                // The bargain the whole enum is here for: the finalist was
                // validated by an `accept_on` dry-run, whose *state* is what
                // a successful `perform_action(best.action)` on the driver
                // would produce. A driver that owns its state adopts
                // `settled` and skips running the same action a second time.
                // When the finalist arrived pre-validated (no dry-run),
                // `settled` is `None` and the driver runs the action
                // normally.
                return BotStep {
                    action: best.action,
                    settled: best.settled,
                };
            }
        }
    }

    // Below here the bot has no cast and no land, and every generator that
    // follows used to take its own walk of the seat's board to ask "is there
    // anything here for me". One walk answers all of them — see `sink`.
    let sinks = sink_facts(state, seat, &have_mana);

    // Morph / Disguise (CR 702.36 / 702.166): cast a hand card face down for
    // {3} as a 2/2 (with ward {2} for Disguise). Reached only when no normal
    // spell candidate validated, so the bot still prefers casting cards face
    // up; `would_accept` enforces sorcery timing and the {3} payment.
    gated_pick!(state, sinks, sink::MORPH, pick_face_down_cast(state, seat, state));

    // Discard-activated hand abilities (Magma Opus's {U/R}{U/R}, Discard:
    // create a Treasure) — a fallback value play, reached only when the bot
    // has no spell/face-down line so it never pitches a castable card.
    gated_pick!(state, sinks, sink::DISCARD_ACT, pick_discard_ability(state, seat, state));

    // Activate planeswalker loyalty abilities the bot controls. Pick the
    // first usable ability per walker (engine enforces sorcery timing and
    // once-per-turn). The candidate set is dry-run-gated so failed targets
    // / over-spent loyalty / opp-controlled-walker rejections drop out.
    gated_pick!(state, sinks, sink::LOYALTY, pick_loyalty_ability(state, seat, w));

    // Crew (CR 702.122): turn an uncrewed Vehicle into an attacker by tapping
    // the bot's least-valuable untapped creatures. Dry-run-gated.
    gated_pick!(state, sinks, sink::CREW, pick_crew(state, seat));

    // Saddle (CR 702.171): tap the bot's least-valuable untapped creatures to
    // saddle a Mount that's about to attack, so its "attacks while saddled"
    // riders fire. Dry-run-gated.
    gated_pick!(state, sinks, sink::SADDLE, pick_saddle(state, seat));

    // Equip (CR 702.6): if the bot controls an Equipment that isn't yet
    // attached to one of its creatures, and it controls a creature to wear
    // it, move the Equipment onto the biggest such creature. Dry-run-gated
    // so the equip cost / sorcery timing / target legality all bottom out
    // in `would_accept`.
    gated_pick!(state, sinks, sink::EQUIP, pick_equip(state, seat));

    // Activated two-slot attach (Brass Squire's "{T}: attach target Equipment
    // you control to target creature you control"). The native-equip pass
    // above only covers `Keyword::Equip`; this drives the Equipment-mover
    // creatures so the AI plays them.
    gated_pick!(state, sinks, sink::AB_ATTACH, pick_attach_ability(state, seat));

    // Spend surplus energy on beneficial energy-payoff abilities (Bristling
    // Hydra's grow, Longtusk Cub's +1/+1, Aetherstream Leopard's
    // unblockable, …). Only pure "Pay {E}: do X" abilities with no other
    // cost are considered, so the bot can't bankrupt mana or sacrifice
    // anything. Dry-run-gated like everything else.
    gated_pick!(state, sinks, sink::AB_ENERGY, pick_energy_payoff(state, seat));

    // Recur value from the graveyard (Embalm CR 702.88 / Eternalize CR 702.91
    // and any "Exile this from your graveyard: …" ability) when there's spare
    // mana and nothing better to do. Dry-run-gated so cost / sorcery timing
    // bottom out in `would_accept`.
    gated_pick!(state, sinks, sink::GY_RECUR, pick_graveyard_recursion(state, seat));

    // Reanimate a creature from the graveyard via a battlefield permanent's
    // activated ability (Seedship Broodtender's sac-to-return) when there's a
    // worthwhile target. Dry-run-gated so cost / sorcery-speed timing bottom
    // out in `would_accept`.
    gated_pick!(state, sinks, sink::AB_REANIMATE, pick_battlefield_reanimate(state, seat));

    // Crack a Lander token (CR — `{2}, {T}, Sacrifice: fetch a basic land
    // tapped`) for ramp when there's a basic still in the library and spare
    // mana. Sequenced after spell-casting so the bot only ramps when it has
    // nothing better to spend mana on. Dry-run-gated.
    gated_pick!(state, sinks, sink::LANDER, pick_crack_lander(state, seat));

    // Fire a "{cost}: deal damage to any target" value ability (Frostwielder's
    // {T} ping, Kiku's tap-and-burn, Pain Kami-style sac burn) when it kills an
    // opposing creature outright. Dry-run-gated so cost / timing / target
    // legality bottom out in `would_accept`.
    gated_pick!(state, sinks, sink::AB_DAMAGE, pick_removal_ping(state, seat));

    // Close the game: fire a "deal N to each opponent" / "drain N" / "each
    // opponent loses N" ability when it's lethal to a living opponent
    // (Hazoret's discard-burn, drain pingers). Lethal-only, so the bot never
    // wastes the resource. Dry-run-gated via `would_accept`.
    gated_pick!(state, sinks, sink::AB_REACH, pick_reach_burn(state, seat));

    // Fire a "Sacrifice this: destroy target creature" ability (Pus Kami,
    // Nezumi Bone-Reader-style sac-removal) on a favorable trade — only when
    // the destroyed foe is at least as big as the creature being sacrificed.
    gated_pick!(state, sinks, sink::AB_SAC_DESTROY, pick_removal_sacrifice(state, seat));

    // Fire a repeatable "{cost}: Destroy target creature" (the Torment
    // Possessed cycle's Threshold ability, Royal Assassin-style tappers) on
    // the biggest legal foe. No trade math — the source survives.
    gated_pick!(state, sinks, sink::AB_DESTROY, pick_removal_destroy(state, seat));

    // Unmask a face-down threat (Morph / Megamorph / Disguise / a cloaked or
    // manifested creature card) when the turn-up cost is affordable. Dry-run-
    // gated, so the cost / timing / "manifested noncreature can't turn up"
    // rules all bottom out in `would_accept`.
    gated_pick!(state, sinks, sink::FACE_DOWN, pick_turn_face_up(state, seat));

    // Pump the whole team before combat damage (Bearer of Glory's
    // "{4}{W}: creatures you control get +1/+1") when the bot has two or more
    // attacking creatures — the pump pays off on the swing. Dry-run-gated.
    gated_pick!(state, sinks, sink::AB_TEAM_PUMP, pick_team_pump(state, seat));

    // As a last resort before passing, sink spare mana into a "{cost}: draw a
    // card" ability when card-starved (Bonders' Enclave, Arch of Orazca-style
    // engines). Dry-run-gated, so cost / activation conditions bottom out in
    // `would_accept`.
    gated_pick!(state, sinks, sink::AB_DRAW, pick_card_draw_ability(state, seat));

    // Same slot, the other route to a card: impulse-draw engines that mill
    // or exile off the top and grant permission to play it (Ark of Hunger).
    // Flag-gated until laddered.
    if w.impulse_draw {
        gated_pick!(state, sinks, sink::AB_GRANT_PLAY, pick_impulse_draw_ability(state, seat));
    }

    // Re-arm an unprepared prepare-spell creature via an off-card "target
    // creature becomes prepared" ability (SOS: Skycoach Waypoint). The
    // counter is worth about the inset spell — see the `permanent_value`
    // term — so this banks value on par with the draw sink above.
    gated_pick!(state, sinks, sink::AB_PREPARES, pick_reprepare(state, seat));

    // Sacrifice-for-value engines (sac a Pest: payoff), judged by the
    // resolved outcome rather than skipped for carrying a sac cost.
    gated_pick!(state, sinks, sink::AB_SAC, pick_sacrifice_value(state, seat, w));

    // Crew an uncrewed Vehicle so it can attack this turn (Vehicles are dead
    // cards to the bot otherwise). Dry-run-gated.
    gated_pick!(state, sinks, sink::CREW, pick_crew_vehicle(state, seat));

    // Sink leftover mana into a repeatable "{cost}: +1/+1 counter on this"
    // ability to grow the board (Fire Sages, Water Tribe Captain). Last resort,
    // so it never pre-empts a spell or land. Dry-run-gated.
    gated_pick!(state, sinks, sink::AB_SELF_COUNTER, pick_self_pump_counter(state, seat));

    // Sink leftover mana into a "{cost}: create a token" ability to grow the
    // board (Sun Warriors' {5}: 1/1 Ally, Realm of Koh's Spirit, Jasmine Dragon).
    // Last resort, dry-run-gated.
    gated_pick!(state, sinks, sink::AB_TOKEN, pick_token_maker(state, seat));

    BotStep::plain(GameAction::PassPriority)
}

/// Activate a sacrifice-cost ability when the RESOLVED outcome beats
/// doing nothing. The generic ability pickers skip sac costs outright and
/// `pick_removal_sacrifice` only knows the destroy-for-trade shape — the
/// value shapes (sacrifice a token: draw / drain / counters) had no
/// judge at all. The clone-and-resolve eval prices both sides of the
/// exchange: the permanent given up AND what its death buys, triggers
/// included. Strictly-better-than-passing or nothing.
fn pick_sacrifice_value(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    let baseline = eval_material(state, seat, w);
    let scan = state.grant_scan();
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card, &scan) {
            if !ab.sac_cost && ab.sac_other_filter.is_none() {
                continue;
            }
            // Destroy-shaped sac removal keeps its dedicated trade math
            // (`pick_removal_sacrifice`, earlier in the chain).
            if matches!(&ab.effect, Effect::Destroy { .. } | Effect::DestroyNoRegen { .. }) {
                continue;
            }
            let target = if ab.effect.requires_target() {
                match state.auto_target_for_effect(&ab.effect, seat) {
                    Some(t) => Some(t),
                    None => continue,
                }
            } else {
                None
            };
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if !ward_gate_ok(state, seat, &action) {
                continue;
            }
            let Some(settled) = state.accept(action.clone()) else { continue };
            if let Some(ev) = evaluate_action_outcome(state, seat, &action, Some(&settled), w)
                && ev > baseline
            {
                return Some(action);
            }
        }
    }
    None
}

/// SOS Prepare mana sink: aim an off-card "target creature becomes
/// prepared" ability (Skycoach Waypoint's `{3},{T}`) at the bot's best
/// unprepared prepare-spell creature — biggest inset spell first, since
/// that's what the counter is worth. Dry-run-gated through `would_accept`.
fn ability_prepares_target(e: &Effect) -> bool {
    use crate::card::CounterType;
    use crate::effect::Selector;
    match e {
        Effect::AddCounter { what, kind: CounterType::Prepared, .. } => {
            matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
        }
        Effect::Seq(v) => v.iter().any(ability_prepares_target),
        _ => false,
    }
}

fn pick_reprepare(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::CounterType;
    let mut targets: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.definition.prepare_spell.is_some()
                && c.counter_count(CounterType::Prepared) == 0
        })
        .collect();
    if targets.is_empty() {
        return None;
    }
    targets.sort_by_key(|c| {
        std::cmp::Reverse(c.definition.prepare_spell.as_deref().map(|s| s.cost.cmc()).unwrap_or(0))
    });
    let scan = state.grant_scan();
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card, &scan) {
            if !ability_prepares_target(&ab.effect) {
                continue;
            }
            for t in &targets {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(t.id)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Activate a non-sacrifice "{cost}: create a token" ability as a last-resort
/// mana sink — grows the board when the bot has nothing better to do. Skips
/// sacrifice-cost and once-per-game (Exhaust) abilities. Dry-run-gated through
/// `would_accept`, so cost/timing legality bottoms out there.
fn ability_makes_token(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. } | Effect::CreateTokenAttacking { .. } => true,
        Effect::Seq(steps) => steps.iter().any(ability_makes_token),
        _ => false,
    }
}

fn pick_token_maker(state: &GameState, seat: usize) -> Option<GameAction> {
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost || ab.exhaust || !ability_makes_token(&ab.effect) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Activate a repeatable "{cost}: put a +1/+1 counter on this creature" ability
/// as a last-resort mana sink — grows the board when the bot has nothing better
/// to do. Skips sacrifice-cost and once-per-game (Exhaust) abilities so it never
/// throws away a permanent or a one-shot. Dry-run-gated through `would_accept`.
fn pick_self_pump_counter(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::CounterType;
    use crate::effect::Selector;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost || ab.exhaust {
                continue;
            }
            // Adapt abilities (CR 702.108) put +1/+1 counters on a creature with
            // none — recognize the `If`-wrapped counter shape and fire only when
            // the creature isn't already adapted (else it's a mana-wasting no-op).
            let useful = if let Effect::AddCounter { what: Selector::This, kind, .. } = &ab.effect {
                // Always sink into +1/+1 self-pumps; otherwise only into a counter
                // that still progresses an unmet "becomes a creature at N counters"
                // static (War Balloon's fire counters), so the bot animates it
                // instead of stalling and doesn't dump mana past the threshold.
                *kind == CounterType::PlusOnePlusOne
                    || card.definition.static_abilities.iter().any(|sa| {
                        matches!(&sa.effect,
                            crate::effect::StaticEffect::SelfIsCreatureWhileCountersAtLeast { kind: k, n }
                            if k == kind && card.counter_count(*kind) < *n)
                    })
            } else if ab.effect.is_adapt() {
                card.counter_count(CounterType::PlusOnePlusOne) == 0
            } else {
                false
            };
            if !useful {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Crew an uncrewed Vehicle the bot controls, paying with the smallest
/// untapped creatures whose total power covers the crew cost — but only when
/// the Vehicle is at least as big as the power tapped to crew it (a net combat
/// gain). Dry-run-gated, so crew legality (CR 702.122) bottoms out in
/// `would_accept`.
fn pick_crew_vehicle(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::CardType;
    let mut crewers: Vec<(CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && !c.tapped)
        .filter_map(|c| {
            let cp = state.computed_permanent(c.id)?;
            cp.card_types().contains(&CardType::Creature).then_some((c.id, cp.power.max(0)))
        })
        .collect();
    crewers.sort_by_key(|&(_, p)| p);

    for v in state.battlefield.iter().filter(|c| c.controller == seat) {
        let Some(cost) = v.definition.crew_cost() else { continue };
        let Some(cp) = state.computed_permanent_on(v) else { continue };
        // Already a creature (crewed/animated this turn) → nothing to do.
        if cp.card_types().contains(&CardType::Creature) {
            continue;
        }
        let mut chosen = Vec::new();
        let mut total = 0i32;
        for &(id, p) in crewers.iter().filter(|&&(id, _)| id != v.id) {
            if total >= cost as i32 {
                break;
            }
            chosen.push(id);
            total += p;
        }
        // Worth it only if the cost is fully paid and the Vehicle is at least
        // as big as the creatures tapped to crew it.
        if chosen.is_empty() || total < cost as i32 || cp.power < total {
            continue;
        }
        let action = GameAction::Crew { vehicle: v.id, crew_creatures: chosen };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// Fire a "deal N damage to each opponent" / "drain N" / "each opponent loses
/// N" activated ability when it's lethal to a living opponent. Only fixed
/// (`Value::Const`) amounts are considered, and only when some opponent's life
/// is at or below the amount, so the bot spends the resource (mana / a discard
/// / a tap) exclusively to win — never to chip. Dry-run-gated via
/// `would_accept`.
/// Amount an ability's effect would subtract from each opponent's life, if
/// it's an each-opponent reach effect with a fixed amount.
fn ability_reach_amount(effect: &Effect) -> Option<i32> {
    use crate::effect::{PlayerRef, Selector, Value};
    match effect {
        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) }
        | Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) }
        | Effect::Drain { from: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n), .. } => {
            Some(*n)
        }
        // Compound abilities (e.g. "do X; each opponent loses N") still
        // count their each-opponent reach: sum a Seq's components, take the
        // best mode of a modal. `would_accept` still gates legality, so a
        // wrapped component that demands a target this call can't supply
        // keeps the whole activation from firing.
        Effect::Seq(parts) => {
            let total: i32 = parts.iter().filter_map(ability_reach_amount).sum();
            (total > 0).then_some(total)
        }
        Effect::ChooseMode(modes) => modes.iter().filter_map(ability_reach_amount).max(),
        _ => None,
    }
}

fn pick_reach_burn(state: &GameState, seat: usize) -> Option<GameAction> {
    let lethal_threshold = state
        .players
        .iter()
        .enumerate()
        .filter(|(p, pl)| !state.same_team(*p, seat) && pl.is_alive())
        .map(|(_, pl)| pl.life)
        .min()?;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Some(amount) = ability_reach_amount(&ab.effect) else { continue };
            if amount < lethal_threshold {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Activate a team-wide "creatures you control get +N/+N until end of turn"
/// ability while the bot has two or more attacking creatures, so the pump
/// connects on the swing. Only positive, no-target, non-sacrifice pumps are
/// considered; dry-run-gated so cost / timing bottom out in `would_accept`.
/// True when `req` constrains its subjects to creatures the controller owns
/// (a `ControlledByYou` clause anywhere in its And/Or tree).
fn requirement_restricts_to_your_creatures(req: &crate::card::SelectionRequirement) -> bool {
    use crate::card::SelectionRequirement as R;
    match req {
        R::ControlledByYou => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_restricts_to_your_creatures(a) || requirement_restricts_to_your_creatures(b)
        }
        _ => false,
    }
}

fn pick_team_pump(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::{Selector, Value};
    let attackers = state
        .attacking
        .iter()
        .filter(|a| state.battlefield_find(a.attacker).is_some_and(|c| c.controller == seat))
        .count();
    if attackers < 2 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost {
                continue;
            }
            let Effect::PumpPT { what: Selector::EachPermanent(req), power: Value::Const(p), .. } =
                &ab.effect
            else {
                continue;
            };
            // Only a friendly-team pump (filter restricts to your creatures)
            // with a real power boost.
            if *p <= 0 || !requirement_restricts_to_your_creatures(req) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Activate a bare "{cost}: draw a card" ability (no target, doesn't sacrifice
/// the source) when the bot is card-starved (≤2 cards in hand) and can afford
/// it. Fired last, as a mana sink, so it never pre-empts casting spells or
/// playing lands. Dry-run-gated through `would_accept`.
fn pick_card_draw_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::Selector;
    if state.players[seat].hand.len() > 2 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::Draw { who: Selector::You, .. } = &ab.effect else { continue };
            if ab.sac_cost {
                continue; // don't sacrifice the source just to draw
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

fn ability_grants_play(eff: &Effect) -> bool {
    match eff {
        Effect::GrantMayPlay { .. } | Effect::ExileTopAndGrantMayPlay { .. } => true,
        Effect::Seq(v) => v.iter().any(ability_grants_play),
        Effect::ChooseMode(m) | Effect::ChooseN { modes: m, .. } => {
            m.iter().any(ability_grants_play)
        }
        Effect::MayDo { body, .. } | Effect::MayDoBy { body, .. } => ability_grants_play(body),
        _ => false,
    }
}

/// Activate an "impulse draw" ability: one that puts a card where the bot can
/// then play it (`GrantMayPlay` / `ExileTopAndGrantMayPlay`), typically after
/// milling or exiling off the top. Ark of Hunger's `{T}: mill 1, you may play
/// that card this turn` is the shape.
///
/// This class was invisible to the bot. [`pick_card_draw_ability`] matches a
/// literal `Effect::Draw` and every other generator matches its own narrow
/// shape, so an ability that manufactures card advantage by any other route
/// was never a candidate at any valuation. Recorded game: Ark of Hunger cast
/// on turn 19, never activated across five turns while the bot topdecked with
/// an empty hand, then exiled.
///
/// Gated on having a *use* for the card — a short hand plus untapped mana —
/// because the mill is a real cost: firing this every turn with no mana to
/// cast what it finds just self-mills. Dry-run-gated like every other
/// generator, so tap/mana/timing legality bottoms out in `would_accept`.
fn pick_impulse_draw_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].hand.len() > 2 {
        return None;
    }
    // Something to cast the flipped card with. Not a guarantee (the card
    // may cost more), just a filter against milling for nothing.
    if available_mana(state, seat).total == 0 {
        return None;
    }
    let scan = state.grant_scan();
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card, &scan) {
            if ab.sac_cost || ab.exile_self_cost {
                continue; // the engine is worth more than one card
            }
            if !ability_grants_play(&ab.effect) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Offer a `TurnFaceUp` for the first affordable face-down permanent the bot
/// controls. The cost is the real card's Morph/Megamorph/Disguise cost, or its
/// mana cost for a manifested/cloaked creature card; `would_accept` enforces it.
fn pick_turn_face_up(state: &GameState, seat: usize) -> Option<GameAction> {
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.face_down && c.face_up_def.is_some())
        .map(|c| GameAction::TurnFaceUp { card_id: c.id })
        .find(|a| state.would_accept(a.clone()))
}

/// Fire a single-target "deal damage to any target" activated ability that
/// kills an opposing creature outright. Handles a constant damage amount
/// (Frostwielder, Pain Kami at fixed X) and the "damage equal to its own power"
/// shape (Kiku, Night's Flower). Targets the highest-power killable opponent
/// creature; dry-run-gated so cost / sorcery timing / target legality all
/// bottom out in `would_accept`. Points the ability at an opponent's face only
/// when the hit is exactly lethal (reach for the win); otherwise never chips a
/// player and never targets the bot's own creatures.
/// Every activated ability a permanent can use right now, paired with the
/// index `GameAction::ActivateAbility` expects: printed abilities first, then
/// the statically granted ones at their virtual indices (CR 611.2 — the
/// Threshold-granted removal on the Torment Possessed cycle, Cryptolith Rite).
/// The bot's ability generators walk this instead of `definition
/// .activated_abilities`, which silently skipped every grant.
///
/// Takes a prebuilt [`GameState::grant_scan`] because every caller runs it in
/// a per-permanent loop; without it each card re-walked the whole board.
///
/// The printed half is *borrowed* from `card.definition` — every caller only
/// reads `.effect` and the cost fields, and deep-cloning the printed list per
/// permanent per generator was 1.71 % of the program on its own. Only the
/// grants, which are synthesized, are owned, and they are boxed: see
/// [`AbilityRef`].
///
/// Yields rather than collects. Every caller is a `for` loop that usually
/// breaks on the first ability it likes, and the collected `Vec` was one
/// allocation per permanent per generator — six generators over the same
/// battlefield.
fn usable_abilities<'a>(
    state: &'a GameState,
    card: &'a crate::card::CardInstance,
    scan: &crate::game::actions::GrantScan<'a>,
) -> impl Iterator<Item = (usize, AbilityRef<'a>)> {
    let printed = &card.definition.activated_abilities;
    let n = printed.len();
    printed
        .iter()
        .map(AbilityRef::Printed)
        .enumerate()
        .chain(
            state
                .granted_abilities_of(card, scan)
                .into_iter()
                .enumerate()
                .map(move |(i, ab)| (n + i, AbilityRef::Printed(ab))),
        )
}

/// "{cost}: Destroy target creature" on a permanent that survives the
/// activation — the untargeted-at-self sibling of `pick_removal_sacrifice`.
/// Fires on the biggest legal opposing creature.
fn pick_removal_destroy(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::Selector;
    let mut foes: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| (c.id, cp.power)))
        .collect();
    foes.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
    let scan = state.grant_scan();
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card, &scan) {
            if ab.sac_cost {
                continue; // `pick_removal_sacrifice` owns the trade math.
            }
            let (Effect::Destroy { what } | Effect::DestroyNoRegen { what }) = &ab.effect else {
                continue;
            };
            if !matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. }) {
                continue;
            }
            for (foe, _) in &foes {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*foe)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                // Unpayable ward tax → the activation would be countered;
                // fall through to the next-biggest foe instead.
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

fn pick_removal_ping(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::{Selector, Value};
    // Reach for the win first: if a constant-damage "any target" ability is
    // lethal to an opponent, point it at their face. Only fires when the hit
    // is actually lethal (life ≤ amount), so it's never a wasted chip ping.
    let scan = state.grant_scan();
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card, &scan) {
            let Effect::DealDamage { to, amount: Value::Const(n) } = &ab.effect else { continue };
            // Must be an untyped "any target" slot (a creature-only filter
            // can't be pointed at a player).
            if !matches!(to, Selector::Target(_)) {
                continue;
            }
            for opp in 0..state.players.len() {
                if state.same_team(opp, seat) || state.players[opp].life > *n {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Player(opp)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    // Opposing creatures, highest computed power first (best removal value).
    let mut foes: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent_on(c).map(|cp| (c.id, cp.power)))
        .collect();
    foes.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
    // Reuses the scan built above: `state` is `&GameState` and nothing
    // between the two loops mutates it, so a second `grant_scan` was a
    // second walk of every static ability in play for the same answer.
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card, &scan) {
            // The effect must be a bare single-target DealDamage whose target
            // can be a creature (not a self/own-board selector).
            let Effect::DealDamage { to, amount } = &ab.effect else { continue };
            if !matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. }) {
                continue;
            }
            for (foe, foe_pow) in &foes {
                let Some(cp) = state.computed_permanent(*foe) else { continue };
                // Remaining toughness after damage already marked this turn
                // (CR 120.6) — a ping that wouldn't kill a fresh creature can
                // still finish one that's been chipped in combat.
                let marked = state.battlefield_find(*foe).map(|c| c.damage as i32).unwrap_or(0);
                let remaining = cp.toughness - marked;
                // Lethal check: a constant amount, or "equal to its own power"
                // (Kiku) where the creature dies if power ≥ remaining toughness.
                let lethal = match amount {
                    Value::Const(n) => *n >= remaining,
                    Value::PowerOf(s) if matches!(**s, Selector::Target(_)) => {
                        *foe_pow >= remaining
                    }
                    // "Deals damage equal to its own power" pingers (firebreather-
                    // style {T} abilities) read the source's computed power.
                    Value::PowerOf(s) if matches!(**s, Selector::This) => state
                        .computed_permanent_on(card)
                        .is_some_and(|p| p.power >= remaining),
                    _ => false,
                };
                if !lethal {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*foe)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    // Opposing planeswalkers, highest loyalty first — a constant-damage "any
    // target" ability that's lethal to the loyalty removes the threat.
    let mut walkers: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_planeswalker())
        .map(|c| (c.id, c.counter_count(crate::card::CounterType::Loyalty) as i32))
        .collect();
    walkers.sort_by_key(|(_, loy)| std::cmp::Reverse(*loy));
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::DealDamage { to, amount: Value::Const(n) } = &ab.effect else { continue };
            // Any target slot that could point at a planeswalker (would_accept
            // re-checks the filter).
            if !matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. }) {
                continue;
            }
            for (walker, loy) in &walkers {
                if *n < *loy {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*walker)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Activate a "Sacrifice this creature: Destroy target creature" ability
/// (Pus Kami, Scuttling Death-style sac removal) on a *favorable* trade: the
/// destroyed opposing creature must be at least as powerful as the creature
/// being sacrificed, so the bot won't pitch a 3/3 to kill a 1/1. Targets the
/// biggest qualifying foe. Dry-run-gated through `would_accept`.
fn pick_removal_sacrifice(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::Selector;
    let mut foes: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| (c.id, cp.power)))
        .collect();
    foes.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
    let scan = state.grant_scan();
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        let src_power = state.computed_permanent_on(card).map(|cp| cp.power).unwrap_or(0);
        for (idx, ab) in usable_abilities(state, card, &scan) {
            if !ab.sac_cost {
                continue;
            }
            let target_is_creature = match &ab.effect {
                Effect::Destroy { what } | Effect::DestroyNoRegen { what } => {
                    matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
                }
                _ => false,
            };
            if !target_is_creature {
                continue;
            }
            for (foe, foe_pow) in &foes {
                // Only a favorable/even trade.
                if *foe_pow < src_power {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*foe)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Find an affordable graveyard-activated ability whose cost exiles the source
/// (Embalm / Eternalize, Stone Docent-style recursion). Returns the activation
/// for the first such card the bot can pay for.
fn pick_graveyard_recursion(state: &GameState, seat: usize) -> Option<GameAction> {
    // The bot's own creatures, highest-power first — candidate targets for
    // abilities that need one (Scavenge's +1/+1 counters, Daring Fiendbonder's
    // indestructible counter). For no-target recursion (Embalm / Eternalize /
    // Stone Docent) we pass `None`.
    let mut own: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .collect();
    own.sort_by_key(|c| std::cmp::Reverse(c.power()));
    for card in state.players[seat].graveyard.iter() {
        // Printed graveyard abilities plus static-granted ones (Varolz's
        // scavenge) at indices ≥ the printed count.
        let printed = card.definition.activated_abilities.clone();
        let granted = state.graveyard_granted_abilities(seat, card);
        for (idx, ab) in printed.iter().chain(granted.iter()).enumerate() {
            // Graveyard-activated abilities worth firing: an exile-self payoff
            // (Embalm-style value) or a self-return that replays the creature
            // (Llanowar Greenwidow's "{7}{G}: return this from your graveyard").
            if !(ab.from_graveyard
                && (ab.exile_self_cost || effect_returns_self_to_battlefield(&ab.effect)))
            {
                continue;
            }
            // Only try a no-target activation when the effect needs none —
            // otherwise `would_accept` (which doesn't re-derive targets) would
            // wave through a wasted target-less activation.
            let candidates: Vec<Option<crate::game::Target>> = if ab.effect.requires_target() {
                own.iter().map(|c| Some(crate::game::Target::Permanent(c.id))).collect()
            } else {
                vec![None]
            };
            for target in candidates {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target,
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// True if `eff` returns its own source to the battlefield (a self-reanimating
/// graveyard ability — Llanowar Greenwidow). Recurses into `Seq`.
fn effect_returns_self_to_battlefield(eff: &Effect) -> bool {
    use crate::effect::ZoneDest;
    match eff {
        Effect::Move { what: crate::card::Selector::This, to: ZoneDest::Battlefield { .. } } => true,
        // "Return this card from your graveyard to the battlefield transformed"
        // (Garland, Knight of Cornelia) — a self-reanimation like the plain
        // Move, just landing on the back face.
        Effect::ExileSelfReturnTransformed | Effect::ExileSelfReturnFrontFace => true,
        Effect::Seq(v) => v.iter().any(effect_returns_self_to_battlefield),
        _ => false,
    }
}

/// True if a `SelectionRequirement` tree constrains its target to a card in a
/// graveyard (`InYourGraveyard` / `InGraveyard`).
fn filter_targets_graveyard(req: &crate::card::SelectionRequirement) -> bool {
    use crate::card::SelectionRequirement as R;
    match req {
        R::InYourGraveyard | R::InGraveyard => true,
        R::And(a, b) | R::Or(a, b) => filter_targets_graveyard(a) || filter_targets_graveyard(b),
        _ => false,
    }
}

/// True if `eff` moves a graveyard-targeted card onto the battlefield (a
/// reanimation effect — Seedship Broodtender's sac-to-return). Recurses `Seq`.
fn effect_reanimates_from_graveyard(eff: &Effect) -> bool {
    use crate::effect::ZoneDest;
    match eff {
        Effect::Move {
            what: crate::card::Selector::TargetFiltered { filter, .. },
            to: ZoneDest::Battlefield { .. },
        } => filter_targets_graveyard(filter),
        Effect::Seq(v) => v.iter().any(effect_reanimates_from_graveyard),
        _ => false,
    }
}

/// Activate a battlefield permanent's ability that reanimates a card from the
/// graveyard (Seedship Broodtender's "{cost}, Sacrifice this: return target
/// creature/Spacecraft from your graveyard to the battlefield"), aimed at the
/// engine's auto-picked best graveyard target. Skips when nothing legal exists.
/// Dry-run-gated so cost / sorcery-speed timing bottom out in `would_accept`.
fn pick_battlefield_reanimate(state: &GameState, seat: usize) -> Option<GameAction> {
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !effect_reanimates_from_graveyard(&ab.effect) {
                continue;
            }
            let target = state.auto_target_for_effect(&ab.effect, seat);
            if target.is_none() {
                continue; // no graveyard creature worth returning
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Crack a Lander token for ramp: a `{2}, {T}, Sacrifice: search a basic land
/// onto the battlefield tapped` ability. Only fires when the controller still
/// has a basic land in their library (so the fetch isn't wasted) and the
/// engine accepts the activation (mana/timing). Targets nothing — the fetch
/// resolves via the library-search decider.
fn pick_crack_lander(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::{ArtifactSubtype, SelectionRequirement};
    let has_basic = state.players[seat]
        .library
        .iter()
        .any(|c| state.evaluate_requirement_on_card(&SelectionRequirement::IsBasicLand, c, seat));
    if !has_basic {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        let is_lander = card.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Lander);
        if !is_lander {
            continue;
        }
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !ab.sac_cost || !matches!(ab.effect, Effect::Search { .. }) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Find a beneficial energy-only activated ability the bot can pay for: an
/// `Effect::PayEnergy { amount, .. }` ability with no mana/tap/sac cost,
/// where the bot controls the source and has at least `amount` energy.
fn pick_energy_payoff(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].energy == 0 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // The energy can be modeled either as a real activation cost
            // (`ActivatedAbility.energy_cost`, the up-front-gated form) or as a
            // resolve-time `Effect::PayEnergy` rider. Match either so the bot
            // fires Longtusk Cub-style `{E}{E}{E}: +1/+1` payoffs regardless of
            // which shape the card uses.
            let amount = if ab.energy_cost > 0 {
                ab.energy_cost
            } else if let Effect::PayEnergy { amount, .. } = &ab.effect {
                *amount
            } else {
                continue;
            };
            let is_pure = !ab.tap_cost
                && !ab.sac_cost
                && ab.mana_cost.symbols.is_empty()
                && ab.life_cost == 0;
            if !is_pure || state.players[seat].energy < amount {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Pick an equip activation: the first controlled Equipment that's either
/// unattached or attached to a permanent the bot doesn't control, paired
/// with the highest-power creature the bot controls. Returns `None` when
/// there's nothing worth equipping. Dry-run gated by the caller's
/// `would_accept` is bypassed here (we gate inline) so the bot doesn't
/// thrash re-equipping the same creature.
/// Crew an uncrewed Vehicle (CR 702.122) the bot controls, tapping the
/// smallest untapped creatures that together meet the crew cost. Skipped
/// unless the Vehicle's power is worth more than the creatures spent on it
/// (so the bot never taps a bigger attacker to animate a smaller Vehicle).
fn pick_crew(state: &GameState, seat: usize) -> Option<GameAction> {
    for vehicle in &state.battlefield {
        if vehicle.controller != seat {
            continue;
        }
        let Some(crew_n) = vehicle.definition.crew_cost() else { continue };
        // Already a creature this turn (crewed / animated)? Don't re-crew.
        if state
            .computed_permanent_on(vehicle)
            .is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Creature))
        {
            continue;
        }
        // Candidate crew members: the bot's untapped creatures, smallest first.
        let mut crew: Vec<(CardId, u32)> = state
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && c.id != vehicle.id
                    && c.definition.is_creature()
                    && !c.tapped
            })
            // CR 702.122e/702.171 — count the crew-power rider (Cloudspire
            // Captain / Deathless Pilot crew "as though power N greater").
            .map(|c| (c.id, (c.power() + state.crew_saddle_power_bonus(c.id)).max(0) as u32))
            .collect();
        crew.sort_by_key(|&(_, p)| p);
        let mut picked = Vec::new();
        let mut total = 0u32;
        for (id, p) in &crew {
            if total >= crew_n {
                break;
            }
            picked.push(*id);
            total += p;
        }
        if total < crew_n {
            continue;
        }
        // Don't spend more board power than the Vehicle is worth.
        if total > vehicle.power().max(0) as u32 {
            continue;
        }
        let action = GameAction::Crew { vehicle: vehicle.id, crew_creatures: picked };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// CR 702.171 — saddle a Mount the bot is about to attack with by tapping its
/// least-valuable other untapped creatures (smallest power first). Only saddles
/// a Mount that can attack this turn and isn't already saddled, and never spends
/// more board power than the Mount itself is worth.
fn pick_saddle(state: &GameState, seat: usize) -> Option<GameAction> {
    // Saddled is "until end of turn" (CR 702.171e), so only pay the tap cost
    // when a combat phase still follows — i.e. precombat main. Saddling in
    // postcombat main just wastes the saddlers before the buff can matter.
    if state.step != TurnStep::PreCombatMain {
        return None;
    }
    for mount in &state.battlefield {
        if mount.controller != seat || mount.saddled || mount.tapped {
            continue;
        }
        let Some(saddle_n) = mount.definition.saddle_cost() else { continue };
        if !mount.can_attack() {
            continue;
        }
        // Candidate saddlers: the bot's other untapped creatures. Tap the ones
        // that can't attack this turn (summoning-sick / Defender) *first* — they
        // are "free" since they'd sit idle anyway — then fall back to would-be
        // attackers, smallest power first (the crew-power rider counts, CR
        // 702.171). Track how much *attacker* power we spend so the overspend
        // guard below doesn't fault free saddlers.
        let mut riders: Vec<(CardId, u32, bool)> = state
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && c.id != mount.id
                    && c.definition.is_creature()
                    && !c.tapped
            })
            .map(|c| {
                (c.id, (c.power() + state.crew_saddle_power_bonus(c.id)).max(0) as u32, c.can_attack())
            })
            .collect();
        // (can-attack ascending, then power ascending): free saddlers first.
        riders.sort_by_key(|&(_, p, can_attack)| (can_attack, p));
        let mut picked = Vec::new();
        let mut total = 0u32;
        let mut attacker_power = 0u32;
        for (id, p, can_attack) in &riders {
            if total >= saddle_n {
                break;
            }
            picked.push(*id);
            total += p;
            if *can_attack {
                attacker_power += p;
            }
        }
        if total < saddle_n {
            continue;
        }
        // Don't tap real attackers worth more board power than the Mount is
        // worth. Idle (can't-attack) saddlers are free and don't count.
        if attacker_power > mount.power().max(0) as u32 {
            continue;
        }
        let action = GameAction::Saddle { mount: mount.id, creatures: picked };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

fn pick_equip(state: &GameState, seat: usize) -> Option<GameAction> {
    // Best creature to wear an Equipment: highest current power, but skip
    // attack-locked bodies (Defender / CantAttack) — an Equipment's combat
    // bonus is wasted on them. Fall back to any creature only if every
    // candidate is attack-locked (a board of Walls still wants the
    // deathtouch/keyword grant for blocking).
    use crate::card::Keyword;
    let can_attack = |c: &crate::card::CardInstance| {
        state
            .computed_permanent(c.id)
            .map(|cp| {
                (!cp.keywords().has_kw(&Keyword::Defender)
                    || state.ignores_defender_for_attack(c))
                    && !cp.keywords().has_kw(&Keyword::CantAttack)
            })
            .unwrap_or(true)
    };
    let mine = || {
        state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature())
    };
    // Rank by *computed* power so anthems / lords / conditional pumps count
    // (a small body under a big anthem is a better Voltron target than a
    // vanilla bigger base body).
    let cpow = |c: &crate::card::CardInstance| {
        state.computed_permanent(c.id).map(|cp| cp.power).unwrap_or_else(|| c.power())
    };
    let target = mine()
        .filter(|c| can_attack(c))
        .max_by_key(|c| cpow(c))
        .or_else(|| mine().max_by_key(|c| cpow(c)))
        .map(|c| c.id)?;
    for eq in &state.battlefield {
        if eq.controller != seat || !eq.definition.is_equipment() {
            continue;
        }
        if eq.definition.has_equip().is_none() {
            continue;
        }
        // Skip if already on the chosen target (no point re-equipping).
        if eq.attached_to == Some(target) {
            continue;
        }
        let action = GameAction::Equip { equipment: eq.id, target };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// Drive a "{cost}: attach target Equipment you control to target creature you
/// control" activated ability (Brass Squire). Picks an Equipment not already on
/// the chosen wearer for slot 0 and the highest-power creature for slot 1. The
/// dry-run gate enforces the activation cost / target legality.
fn pick_attach_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::Selector;
    use crate::effect::Effect;
    let wearer = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .max_by_key(|c| c.power())
        .map(|c| c.id)?;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // Two distinct target slots: `what` (slot 0) and `to` (slot 1).
            let Effect::Attach {
                what: Selector::TargetFiltered { slot: 0, .. },
                to: Selector::TargetFiltered { slot: 1, .. },
            } = &ab.effect
            else {
                continue;
            };
            let Some(equip) = state.battlefield.iter().find(|e| {
                e.controller == seat
                    && e.definition.is_equipment()
                    && e.attached_to != Some(wearer)
            }) else {
                continue;
            };
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: Some(crate::game::Target::Permanent(equip.id)),
                additional_targets: vec![crate::game::Target::Permanent(wearer)],
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Walk every planeswalker the bot controls and pick the first activatable
/// loyalty ability. Auto-target via `auto_target_for_effect` for abilities
/// that require a target. Prefers a +loyalty ability when available
/// (preserves the walker for next turn), falling back to the ability with
/// the smallest absolute loyalty cost so we don't suicide-ult immediately.
fn pick_loyalty_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    for card in &state.battlefield {
        if card.controller != seat {
            continue;
        }
        if !card.definition.is_planeswalker() {
            continue;
        }
        let allowed = if card.definition.loyalty_twice_each_turn { 2 } else { 1 };
        if card.loyalty_uses_this_turn >= allowed {
            continue;
        }
        // Gather every affordable ability and pick by OUTCOME, not by
        // loyalty-cost order. The old plus-first walk meant a walker with
        // a strong minus never used it — Professor Dellian Fel spent whole
        // games on "+2: gain 3 life" while "−3: destroy target creature"
        // sat unused (its attribution read neutral for a bomb). Use the
        // *effective* list (printed + statically-granted, e.g. Kasmina
        // Enigma Sage / Ichormoon Gauntlet) so the bot can activate granted
        // loyalty abilities too — the engine indexes the same list.
        // Ultimates whose payoff the material eval can't see (emblems)
        // still lose to a plus — a known limitation.
        let current_loyalty =
            card.counter_count(crate::card::CounterType::Loyalty) as i32;
        let effective = crate::game::effective_loyalty_abilities(card, &state.battlefield);
        let mut finalists: Vec<Finalist> = Vec::new();
        for (idx, ability) in effective.iter().enumerate() {
            if current_loyalty + ability.loyalty_cost < 0 {
                continue;
            }
            let target = if ability.effect.requires_target() {
                // No legal target for *this* ability — skip it and try the
                // next (formerly `?`-returned, which abandoned every other
                // ability and planeswalker the bot controls).
                match state.auto_target_for_effect(&ability.effect, seat) {
                    Some(t) => Some(t),
                    None => continue,
                }
            } else {
                None
            };
            // Variable-X (`-X`) ability: commit all current loyalty.
            let x_value = ability.x_cost.then_some(current_loyalty.max(0) as u32);
            let action = GameAction::ActivateLoyaltyAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                x_value,
            };
            if let Some(g) = state.accept(action.clone()) {
                finalists.push(Finalist {
                    score: score_candidate(state, seat, &action, w),
                    action,
                    settled: Some(Box::new(g)),
                });
            }
        }
        // A walker the board kills before its next activation banks
        // nothing by plussing — the loyalty it gains is removed by
        // attackers at zero cost to the opponent, a future the outcome
        // eval's one-combat horizon cannot see. When the enemy board's
        // creature power already covers current loyalty (a crude read
        // on next combat), cash out: restrict to loyalty-SPENDING
        // abilities whenever any is affordable, and let the outcome
        // eval pick among those.
        //
        // The read has to be about the loyalty the walker would *have*
        // and the damage that would actually *reach* it. Comparing raw
        // enemy power against `current_loyalty` made any single 1/1 force
        // a cash-out at 1 loyalty, counted creatures that cannot attack,
        // and ignored our own blockers entirely — so past the opening
        // turns the condition was always true and the bot never plussed a
        // walker at all. Recorded games (2026-08-23): **zero** plus
        // activations against eight minuses, including Ral Zarek spending
        // its last loyalty to strip one card and die.
        let mut incoming: Vec<i32> = state
            .battlefield
            .iter()
            .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
            // Tapped and summoning-sick both wear off before their next
            // attack; Defender does not.
            .filter(|c| !c.has_keyword(&crate::card::Keyword::Defender))
            .filter_map(|c| state.computed_permanent(c.id).map(|cp| cp.power.max(0)))
            .collect();
        incoming.sort_unstable_by(|a, b| b.cmp(a));
        // Each untapped body of ours eats one attacker, biggest first.
        // Creatures we later choose to attack with will not be back to
        // block, so this is an optimistic read — but the term it replaces
        // assumed *no* defenders at all.
        let blockers = state
            .battlefield
            .iter()
            .filter(|c| state.same_team(c.controller, seat) && c.definition.is_creature() && !c.tapped)
            .count();
        let threat: i32 = if w.legacy_cashout {
            incoming.iter().sum()
        } else {
            incoming.iter().skip(blockers).sum()
        };
        // Plussing is only pointless if the walker dies *even after* the
        // loyalty it would gain.
        let best_plus = effective
            .iter()
            .map(|ab| ab.loyalty_cost)
            .filter(|c| *c > 0)
            .max()
            .unwrap_or(0);
        let bar = if w.legacy_cashout { current_loyalty } else { current_loyalty + best_plus };
        // One predicate, asked twice: "is there a spending line at all" and
        // then "keep only those". Written out once so the two cannot drift.
        let spends = |f: &Finalist| {
            matches!(&f.action, GameAction::ActivateLoyaltyAbility { ability_index, .. }
                if effective.get(*ability_index).is_some_and(|ab| ab.loyalty_cost < 0))
        };
        if threat >= bar && finalists.iter().any(spends) {
            finalists.retain(spends);
        }
        if let Some(best) = pick_by_outcome(state, seat, finalists, w) {
            return Some(best.action);
        }
    }
    None
}

/// Test-visible wrapper for `forced_blocks` — the declaration an attacking
/// block chooser (Invasion Plans) submits.
pub fn forced_blocks_for_test(state: &GameState) -> Vec<(CardId, CardId)> {
    forced_blocks(state)
}

/// Test-visible wrapper for `pick_blocks` so external tests can exercise
/// the blocker heuristic in isolation.
pub fn pick_blocks_for_test(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    pick_blocks(state, seat)
}

/// The bot's attack declaration for `seat`: which creatures swing and at
/// what. Extracted from `next_action` so the combat-aware evaluation can
/// replay the same choice inside a simulation (see
/// [`simulate_through_combat`]) rather than re-deriving it.
pub fn pick_attacks(state: &GameState, seat: usize) -> Vec<Attack> {
    // Layer-aware per-creature checks (Defender/Flying grants, Propaganda,
    // computed P/T) run once per candidate attacker — share one gather, the
    // same way `pick_blocks` does. Matters most inside the attack/block sims,
    // which call this on a freshly cloned (and therefore unfrozen) state.
    state.with_frozen_layers(|state| pick_attacks_inner(state, seat))
}

/// Turns for `clock` damage a turn to finish `life`, without the overflow the
/// plain `(life + clock - 1) / clock` form has.
///
/// `effective_life` reads **`i32::MAX`** on a Beacon of Immortality board —
/// ENGINE_BACKLOG's closed stall lead, a correct card doing what it prints —
/// and `i32::MAX + clock - 1` wraps there. In release that wrap is silent and
/// turns the race check inside out: a negative `their_turns` reads as "the
/// opponent kills us before our next untap", so the bot attacks all-out into a
/// board it cannot race. Caught by the `debug-assertions` sweep at seeds 53 and
/// 73 of `--decks all`.
fn turns_to_lethal(life: i32, clock: i32) -> i32 {
    debug_assert!(clock > 0, "callers gate on a positive clock");
    let life = life.max(1);
    life / clock + i32::from(life % clock != 0)
}

/// The player an attack is aimed at by default: an opposing monarch (CR
/// 724 — stealing the crown denies their end-step card and hands it to
/// us), otherwise the next alive opponent.
fn attack_target_player(state: &GameState, seat: usize) -> usize {
    match state.monarch {
        Some(m) if m != seat && state.players.get(m).map(|p| p.is_alive()).unwrap_or(false) => m,
        _ => state.next_alive_seat(seat),
    }
}

fn pick_attacks_inner(state: &GameState, seat: usize) -> Vec<Attack> {
    use crate::card::Keyword;
    let target_player = attack_target_player(state, seat);
    // Filter on `controller`, not `owner`: cards that have
    // changed control (Threaten / Mind Control / etc.) are
    // attacked WITH by the new controller, not the original
    // owner.
    //
    // Bot AI improvement (push XXV): hold back attackers
    // that would suicide into deathtouch blockers when
    // there's no upside. The heuristic computes:
    //   * lethal_swing: whether sum of attackers' powers
    //     already meets opponent's life total.
    // When NOT lethal:
    //   * skip attackers whose toughness is <= the maximum
    //     opponent blocker power AND there's at least one
    //     opponent blocker with deathtouch + reach/flying
    //     parity (i.e. a blocker can be assigned).
    // This keeps small attackers from auto-dying to
    // Witherbloom Crawler / Sapworm / Toxicultivator and
    // similar deathtouch defenders.
    let opp_seat = target_player;
    let opp_life = state.players[opp_seat].life;
    // One battlefield walk for both prohibitions this function has to model,
    // the same bitmask `declare_attackers_banded` gates its own blocks on.
    let statics = crate::game::combat::attack_static_scan(state);
    // CR 508.1a — the picker asks the *engine* which creatures may be
    // declared rather than re-deriving the gate. Its own copy answered nine
    // of the ~26 restriction families, drifted on the printed-vs-computed
    // reads twice (PERF (-55)), and had just been hand-patched a tenth time
    // for CR 613's power cap; a batch the engine rejects costs the bot its
    // whole combat, so a filter that is merely *close* is worse than one that
    // cannot drift. Batch-level rules (attacks-alone, the participation cap,
    // the attack tax) stay below, where the batch exists.
    let attack_power_caps = state.attack_power_caps(statics);
    // A plain loop, not `filter().collect()`: the battlefield is walked per
    // element on every call, and `Vec::from_iter` forwards the predicate
    // through `&mut F::call_mut` once per permanent — PERF (-78)'s test 1,
    // and this function was 41 % of the program's whole adapter-forwarding
    // tax between this site and `opp_blockers` below.
    // Inline storage: this and `opp_blockers` below are the two board-walk
    // locals `pick_attacks_inner` fills on every call, and between them they
    // are 12,996 `grow_one` calls a six-game `cube` run (2.87 a call). A board
    // rarely puts sixteen candidates on either side, and when it does the
    // spill is one allocation instead of the 0->4->8->16 ladder.
    let mut raw_attackers: smallvec::SmallVec<[&crate::card::CardInstance; 16]> =
        smallvec::SmallVec::new();
    for c in state.battlefield.iter() {
        // Two instance reads before the layer view: `computed_permanent`
        // is ~1.5 k Ir on a first read and asking it about every land and
        // enchantment the seat controls read `fixed` +0.62 %. Both gates
        // are the ones this filter already made, so the candidate set is
        // unchanged — including its one limitation, that a permanent
        // *animated* into a creature is never considered.
        if c.controller == seat
            && !c.tapped
            && c.definition.is_creature()
            && state.may_declare_attacker(
                seat,
                c,
                state.computed_permanent_on(c).as_deref(),
                &attack_power_caps,
                Some(target_player),
            )
        {
            raw_attackers.push(c);
        }
    }
    // Use the damage-aware value so toughness-attackers (Doran,
    // High Alert) are weighed by what they actually deal.
    let total_raw_power: i32 =
        raw_attackers.iter().map(|c| attacker_damage_value(state, c.id)).sum();
    let lethal_swing = total_raw_power >= opp_life;
    // Race math: compare full-out clocks. We strike first
    // (it's our combat), so strictly fewer turns-to-lethal
    // than the opponent's counter-clock — inside a short
    // horizon — means holding back only concedes the race;
    // attack like it's lethal-in-N. Defenders and can't-
    // attack bodies add nothing to their clock.
    let opp_clock: i32 = state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == opp_seat
                && c.definition.is_creature()
                && !c.has_keyword(&Keyword::Defender)
                && !c.has_keyword(&Keyword::CantAttack)
        })
        .map(|c| c.power().max(0))
        .sum();
    let racing = total_raw_power > 0 && opp_clock > 0 && {
        let our_turns = turns_to_lethal(opp_life, total_raw_power);
        let their_turns = turns_to_lethal(state.effective_life(seat), opp_clock);
        our_turns < their_turns && our_turns <= 5
    };
    let lethal_swing = lethal_swing || racing;
    // The second whole-board walk, and a plain loop for the same reason as
    // `raw_attackers` above.
    let mut opp_blockers: smallvec::SmallVec<[&crate::card::CardInstance; 16]> =
        smallvec::SmallVec::new();
    // The computed `CantBlock` read below was a layer-memo *miss* per
    // untapped opposing creature on every attack pick — this scope's first
    // question about each of them, and most of them are asked nothing else
    // here (the legality gate short-circuits on the first blocker that can
    // block). The presence gate's `false` is authoritative for every
    // computed keyword set, so on the ordinary board the loop reads two
    // instance fields per permanent and no view at all.
    let cant_block_in_scope = state.board_keyword_in_scope(&[Keyword::CantBlock]);
    for c in state.battlefield.iter() {
        // A creature that's tapped, not a creature, or has a
        // computed `CantBlock` (Sandstorm Verge, pacifism-
        // style effects) can't block — don't let the bot hold
        // attackers back for a blocker that can't legally block.
        if c.controller == opp_seat
            && c.can_block()
            && !(cant_block_in_scope
                && state
                    .computed_permanent_on(c)
                    .is_some_and(|cp| cp.keywords().has_kw(&Keyword::CantBlock)))
        {
            opp_blockers.push(c);
        }
    }
    let has_ground_deathtouch = opp_blockers
        .iter()
        .any(|b| b.has_keyword(&Keyword::Deathtouch) && !b.has_keyword(&Keyword::Flying));
    let max_ground_blocker_power: i32 = opp_blockers
        .iter()
        .filter(|b| !b.has_keyword(&Keyword::Flying))
        .map(|b| b.power())
        .max()
        .unwrap_or(0);
    let mut attackers: Vec<crate::card::CardId> = raw_attackers
        .into_iter()
        .filter(|c| {
            // CR 508.1d's must-attack creatures are NOT force-included
            // here: `restore_forced_attackers` below re-adds every one
            // the *computed* set obliges, which is the same membership
            // and one predicate instead of two that drifted.
            // Always attack on lethal swings — the bot
            // would rather suicide than miss a kill.
            if lethal_swing {
                return true;
            }
            // CR 615.1 — don't swing with a creature whose
            // combat damage is prevented this turn (Fog /
            // Inspire Awe's exception); attacking only risks it
            // for no damage.
            if state.combat_damage_prevented_for_dealer(c.id) {
                return false;
            }
            // Unblockable by the current board: if the
            // opponent has creatures but none can legally
            // block this attacker (Unblockable, "can't be
            // blocked by/except by" restrictions the board
            // can't satisfy), it's a free swing. Generalizes
            // the Flying/Menace evasion checks below.
            if !opp_blockers.is_empty()
                && opp_blockers
                    .iter()
                    .all(|b| !state.blocker_can_block_attacker(b.id, c.id))
            {
                return true;
            }
            let flying = c.has_keyword(&Keyword::Flying);
            // Evasive attackers (flying) — only block-
            // worried if there's a flying opp blocker.
            // Skip the deathtouch / ground-power filter
            // for them; assume they're safe.
            if flying {
                let opp_has_flying_blocker = opp_blockers.iter()
                    .any(|b| b.has_keyword(&Keyword::Flying)
                          || b.has_keyword(&Keyword::Reach));
                if !opp_has_flying_blocker {
                    return true; // free swing
                }
            }
            // Trample: tougher creatures still come in
            // (we'll get some damage through chumps).
            if c.has_keyword(&Keyword::Trample) {
                return true;
            }
            // Indestructible: safe to swing (won't die).
            if c.has_keyword(&Keyword::Indestructible) {
                return true;
            }
            // Shield counter on the attacker — the first
            // damage is prevented, so a basic ground-trade
            // is safe (push XXVI bot improvement).
            if c.counter_count(crate::card::CounterType::Shield) > 0 {
                return true;
            }
            // Lifelink: even if we trade, we gain life —
            // worth swinging when we can race.
            if c.has_keyword(&Keyword::Lifelink) {
                return true;
            }
            // Deathtouch attacker: any blocker that deals
            // with it dies (CR 702.2), so blocking is at
            // best an even trade for the opponent — swinging
            // is always at least fine.
            if c.has_keyword(&Keyword::Deathtouch) && c.power() >= 1 {
                return true;
            }
            // Menace (CR 702.111): needs two+ blockers. If
            // the opponent has fewer than two creatures that
            // can legally block this attacker, it gets
            // through unblocked — safe to swing.
            if c.has_keyword(&Keyword::Menace) {
                let able = opp_blockers
                    .iter()
                    .filter(|b| {
                        !flying
                            || b.has_keyword(&Keyword::Flying)
                            || b.has_keyword(&Keyword::Reach)
                    })
                    .count();
                if able < 2 {
                    return true;
                }
            }
            // First strike + bigger power than blockers'
            // toughness — we kill the blocker before it
            // strikes back. Safe attack (push XXVI).
            if c.has_keyword(&Keyword::FirstStrike)
                || c.has_keyword(&Keyword::DoubleStrike)
            {
                let max_blocker_toughness: i32 = opp_blockers
                    .iter()
                    .filter(|b| !b.has_keyword(&Keyword::Flying) || flying)
                    .map(|b| b.toughness())
                    .max()
                    .unwrap_or(0);
                if c.power() > max_blocker_toughness {
                    return true;
                }
            }
            // Hold back if a deathtouch blocker exists
            // and we don't outsize the biggest blocker.
            if has_ground_deathtouch && !flying {
                return false;
            }
            // Finality counter on the attacker — if it
            // dies it'll exile instead of returning to
            // the graveyard (CR 122.1h). Don't suicide
            // a finality-counter creature into ground
            // blockers that can kill it.
            // Push (claude/modern_decks, batches 192-197).
            if c.counter_count(crate::card::CounterType::Finality) > 0
                && !flying
                && max_ground_blocker_power >= c.toughness()
            {
                return false;
            }
            // Hold back if our toughness is <= biggest
            // blocker power and we wouldn't kill them
            // (basic suicide filter).
            if !flying
                && max_ground_blocker_power >= c.toughness()
                && c.power() <= max_ground_blocker_power
            {
                return false;
            }
            true
        })
        .map(|c| c.id)
        .collect();
    // CR 508.1d — before the cap, because a creature the rules oblige to
    // attack makes the whole declaration illegal by its absence.
    restore_forced_attackers(state, seat, &attack_power_caps, &mut attackers);
    // CR 506.2 — Silent Arbiter caps the whole combat. An
    // over-sized batch is rejected outright, so trim to the
    // cap keeping the biggest attackers.
    if let Some(cap) = state.combat_participation_cap(false)
        && attackers.len() > cap as usize
    {
        attackers.sort_by_cached_key(|id| {
            -state.computed_permanent(*id).map(|cp| cp.power).unwrap_or(0)
        });
        attackers.truncate(cap as usize);
    }
    // CR 508.0 — drop a lone attacker that can't attack alone
    // (Militia Rallier): a single-attacker batch with
    // CantAttackAlone would be rejected, costing the bot its
    // whole combat. Only matters when it's the sole attacker.
    if attackers.len() == 1
        && state
            .computed_permanent(attackers[0])
            .is_some_and(|cp| cp.keywords().has_kw(&Keyword::CantAttackAlone))
    {
        attackers.clear();
    }
    // CR 508.0, the other half — `AttacksAlone` (Aisling Leprechaun's
    // cousins): a creature that *attacks alone* makes any batch with a
    // second attacker illegal, and the engine rejects the batch rather than
    // the pair. Drop those creatures from a multi-attacker declaration; a
    // lone one is legal and stays. Keeping the loner instead would trade
    // every other attacker for one body.
    if attackers.len() > 1 {
        attackers.retain(|id| {
            !state
                .computed_permanent(*id)
                .is_some_and(|cp| cp.keywords().has_kw(&Keyword::AttacksAlone))
        });
    }
    // Find opponent planeswalkers in loyalty-ascending
    // order. The bot will redirect attacks at PWs whose
    // current loyalty is at-or-below our total attacking
    // power — finishing off the walker. Each PW consumes
    // up to its loyalty worth of attackers; the rest
    // attack the player.
    let mut walker_targets: Vec<(crate::card::CardId, u32)> = state
        .battlefield
        .iter()
        .filter(|c| {
            c.definition.is_planeswalker()
                && c.controller != seat
                && state.players[c.controller].is_alive()
                // CR 506.2 — The Aetherspark while attached.
                && !state.permanent_cant_be_attacked(c.id)
        })
        .map(|c| {
            let loyalty = c
                .counters
                .iter()
                .find_map(|(k, v)| {
                    matches!(k, crate::card::CounterType::Loyalty)
                        .then_some(*v)
                })
                .unwrap_or(0);
            (c.id, loyalty)
        })
        .collect();
    walker_targets.sort_by_key(|(_, l)| *l);
    let total_power: i32 = attackers
        .iter()
        .filter_map(|id| {
            state.battlefield.iter().find(|c| c.id == *id).map(|c| c.power())
        })
        .sum();
    let mut attacks: Vec<Attack> = Vec::with_capacity(attackers.len());
    for (pw_id, loyalty) in walker_targets {
        // Only redirect when we can plausibly finish it
        // off (total attacking power >= loyalty). Avoids
        // throwing 1-power chumps at a 5-loyalty walker.
        if (total_power as u32) < loyalty || loyalty == 0 {
            continue;
        }
        // Pull as many attackers as the walker's loyalty
        // for this redirect, picking smallest-power
        // first so we keep beefy beaters for the player
        // when possible. (Suicide-by-blocker is still
        // not modeled here.)
        let mut budget = loyalty as i32;
        attackers.sort_by_cached_key(|id| {
            state
                .battlefield
                .iter()
                .find(|c| c.id == *id)
                .map(|c| c.power())
                .unwrap_or(0)
        });
        let mut remaining: Vec<crate::card::CardId> = Vec::with_capacity(attackers.len());
        for id in attackers.drain(..) {
            let pow = state
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.power())
                .unwrap_or(0);
            if budget > 0 && pow > 0 {
                attacks.push(Attack {
                    attacker: id,
                    target: AttackTarget::Planeswalker(pw_id),
                });
                budget -= pow;
            } else {
                remaining.push(id);
            }
        }
        attackers = remaining;
    }
    // Remaining attackers go at the player.
    for id in attackers {
        attacks.push(Attack {
            attacker: id,
            target: AttackTarget::Player(target_player),
        });
    }
    // Last, because the tax depends on what each attacker is aimed at.
    trim_attacks_to_payable_tax(state, seat, statics, &mut attacks);
    attacks
}

/// CR 508.1d — a creature the rules oblige to attack, read off the
/// **computed** keyword set because that is where the engine reads it.
///
/// `others_attacking` is `MustAttackIfAnotherAttacks`'s condition (Ekundu
/// Cyclops is obliged only once somebody else has been declared), which is
/// what makes the obligation set-dependent and the repair below a loop.
fn must_attack(
    c: &crate::card::CardInstance,
    kws: &[crate::card::Keyword],
    others_attacking: bool,
) -> bool {
    use crate::card::Keyword;
    kws.has_kw(&Keyword::MustAttack)
        || kws.has_kw(&Keyword::MustAttackOrBlock)
        || (kws.has_kw(&Keyword::MustAttackIfAnotherAttacks) && others_attacking)
        || !c.goaded_by.is_empty()
}

/// CR 508.1d — re-add every creature the rules oblige to attack that the
/// planner's heuristics dropped.
///
/// A missing must-attacker makes the **whole** declaration illegal, not that
/// one attacker, so this costs a combat rather than a body. The picker used
/// to force-include them from its own filter, off the *instance* view and
/// naming one of the three keywords the engine names — so a granted
/// `MustAttackOrBlock`, or an Ekundu Cyclops that had to join once another
/// attacker was declared, was left home and the batch was rejected (22 of
/// the residual attack rejections on `cube --seed 11`; PERF (-55)).
///
/// A repair pass rather than a filter predicate because the obligation is
/// **set-dependent**: adding one attacker can oblige another, so it loops to
/// a fixed point. Gated on the same board presence question the engine uses
/// to decide whether to compute the whole battlefield at all.
fn restore_forced_attackers(
    state: &GameState,
    seat: usize,
    power_caps: &[usize],
    attackers: &mut Vec<CardId>,
) {
    if !attack_requirement_present(state) {
        return;
    }
    let statics = crate::game::combat::attack_static_scan(state);
    restore_forced_attackers_unchecked(state, seat, power_caps, statics, attackers);
}

/// Can any permanent on this board carry an attack requirement at all? The
/// same question `declare_attackers_banded` asks before deciding whether to
/// compute the whole battlefield, hoisted so a caller with several
/// declarations to repair pays it once.
fn attack_requirement_present(state: &GameState) -> bool {
    use crate::card::Keyword;
    state.battlefield.iter().any(|c| !c.goaded_by.is_empty())
        || state.board_keyword_in_scope(&[
            Keyword::MustAttack,
            Keyword::MustAttackOrBlock,
            Keyword::MustAttackIfAnotherAttacks,
        ])
}

/// [`restore_forced_attackers`] with the presence gate already answered.
fn restore_forced_attackers_unchecked(
    state: &GameState,
    seat: usize,
    power_caps: &[usize],
    // `attack_static_scan`, for the CR 508.1g cost gate inside
    // `attacker_is_able` — a whole-board walk, so the caller hoists it.
    statics: u32,
    attackers: &mut Vec<CardId>,
) {
    loop {
        let mut added = false;
        for c in state.battlefield.iter() {
            if c.controller != seat || attackers.contains(&c.id) {
                continue;
            }
            let Some(cp) = state.computed_permanent_on(c) else { continue };
            // `attackers` never holds `c.id` here, so "another attacker
            // exists" is just a non-empty batch; spelled as the engine
            // spells it so the two read alike.
            let others = attackers.iter().any(|id| *id != c.id);
            if !must_attack(c, cp.keywords(), others)
                || !state.attacker_is_able(seat, c, Some(&cp), power_caps, statics)
            {
                continue;
            }
            attackers.push(c.id);
            added = true;
        }
        if !added {
            break;
        }
    }
}

/// CR 508.1g — drop attackers until the declaration's tax is payable.
///
/// The tax is charged per attacker and the engine rejects the declaration
/// **whole** when the auto-tap can't cover it, so an unaffordable batch
/// costs the bot its entire combat rather than one attacker. Against a
/// Propaganda that meant it never attacked at all: 718 of the 740 attack
/// rejections the `CRAB_SIM_REJECTS` census counted on `cube --seed 11` are
/// this one gate, and in a simulation the rollback fallback then passes
/// priority, so the *modelled* opponent declared nothing either.
///
/// [`GameState::attack_tax_for`] is the engine's own walker, so the two
/// can't disagree about the amount. What the picker supplies is the budget,
/// and `available_mana` is deliberately optimistic — so this drops the
/// batches that are clearly unaffordable and leaves the engine to reject
/// what is left, which is the safe direction: an over-tight trim would
/// silently decline legal attacks.
///
/// Untaxed attackers are never dropped, and neither is a must-attack one —
/// CR 508.1d would reject the batch for its absence instead.
/// The largest generic mana `seat` can **actually** pay right now, measured
/// against the engine's own auto-tap rather than estimated, capped at `want`.
///
/// [`available_mana`]'s `total` is an estimate, and every other bias in it is
/// downward on purpose — sac costs and dynamic amounts are left out so the bot
/// does not commit to a line it can only pay by spending something it would
/// rather keep. **A trim uses it as a *budget*, where an upward bias is a
/// different animal entirely: it does not make the bot optimistic, it makes
/// the engine reject the whole declaration**, and the batch rejection costs
/// the bot its entire combat. On `cube` seed 2 the two disagreed by one or two
/// mana on ordinary board shapes — three lands and a Coalition Relic reading
/// as three where auto-tap could reach two — and those disagreements were
/// every attack rejection left on six `cube` seeds.
///
/// It runs in both directions. A lone Lotus Petal reads as 0 to the estimate
/// (right for a *casting* decision, and pinned by a test) where auto-tap will
/// sacrifice it to pay a tax the rules have already forced on the bot; the
/// trim was dropping swings for mana that was there.
///
/// `could_pay_generic` is the engine's own dry run (a clone plus an auto-tap),
/// so this cannot drift from what `declare_attackers_banded` will do. The
/// predicate is monotone in `n`, so a binary search bounds the probes at
/// `1 + log2(want)` — and the first probe is the whole answer whenever the
/// declaration is affordable, which is the common case. Reached only on a
/// board that charges a tax at all.
fn payable_generic_budget(state: &GameState, seat: usize, want: u32) -> u32 {
    if want == 0 || state.could_pay_generic(seat, want) {
        return want;
    }
    // `lo` is known payable, `hi` known not.
    let (mut lo, mut hi) = (0u32, want);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if state.could_pay_generic(seat, mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

fn trim_attacks_to_payable_tax(
    state: &GameState,
    seat: usize,
    statics: u32,
    attacks: &mut Vec<Attack>,
) {
    if attacks.is_empty() {
        return;
    }
    let keyword_tax = |id: CardId| {
        state
            .computed_permanent(id)
            .map(|cp| state.attack_block_keyword_tax(id, cp.keywords(), true))
            .unwrap_or(0)
    };
    let total = state.attack_tax_for(attacks, statics, keyword_tax);
    if total == 0 {
        return;
    }
    // Measured against the engine's auto-tap, not estimated — see
    // `payable_generic_budget` for why an over-counted budget is a rejected
    // declaration rather than an optimistic bot.
    let budget = payable_generic_budget(state, seat, total);
    if total <= budget {
        return;
    }
    // Per-attacker taxes, reached only on a taxed board the bot can't pay
    // for outright. The tax is additive per attacker with no cross terms
    // (see `attack_tax_for`), so the parts sum to the whole.
    let mut spend = 0u32;
    let mut taxed: Vec<(usize, u32, i32)> = Vec::new();
    for (i, a) in attacks.iter().enumerate() {
        let t = state.attack_tax_for(std::slice::from_ref(a), statics, keyword_tax);
        if t == 0 {
            continue;
        }
        // A must-attacker is never the one dropped: CR 508.1d would reject
        // the batch for its absence instead. `attacks` still holds it, so
        // "another attacker exists" is a batch of more than one.
        let forced = state.battlefield_find(a.attacker).is_some_and(|c| {
            state
                .computed_permanent(a.attacker)
                .is_some_and(|cp| must_attack(c, cp.keywords(), attacks.len() > 1))
        });
        // CR 508.1a/508.1g — a forced attacker is kept only while its own tax
        // fits the budget, which is exactly the question `attacker_is_able`
        // now asks: a creature whose attack cost cannot be paid is not *able*,
        // so CR 508.1d does not require it and dropping it is legal. Keeping
        // it unconditionally is what cost seat 0 its whole combat on
        // `build_cube_state_seeded(3637)` — a Juggernaut behind a two-mana tax
        // with no mana, required to attack and rejected for attacking.
        if forced && t <= budget {
            spend += t;
            continue;
        }
        taxed.push((i, t, attacker_damage_value(state, a.attacker)));
    }
    // Damage per mana, descending: the trim keeps the swings that pay for
    // themselves rather than whichever came first in board order.
    taxed.sort_by(|x, y| (y.2 as i64 * x.1 as i64).cmp(&(x.2 as i64 * y.1 as i64)));
    let mut keep = vec![true; attacks.len()];
    for (i, t, _) in taxed {
        match spend.checked_add(t).filter(|s| *s <= budget) {
            Some(s) => spend = s,
            None => keep[i] = false,
        }
    }
    let mut i = 0;
    attacks.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
    // CR 508.0 — the trim must not manufacture the rejection the picker
    // already guards against one step up: a lone `CantAttackAlone` attacker.
    if attacks.len() == 1
        && state
            .computed_permanent(attacks[0].attacker)
            .is_some_and(|cp| cp.keywords().has_kw(&crate::card::Keyword::CantAttackAlone))
    {
        attacks.clear();
    }
}

/// The attack declaration, chosen by search rather than by rule.
///
/// [`pick_attacks`] is a greedy accretion: a pile of individually sensible
/// exclusions (don't suicide into deathtouch, respect Propaganda, honor
/// layer-granted Defender) applied to "swing with everything". Each of
/// those rules is right about the case it names. What none of them can see
/// is the *cost of tapping the team* — that a creature which attacks is
/// not available to block next turn — because that cost is only paid a
/// turn later, on a board the greedy rule never looks at.
///
/// `bot_probe` measures the consequence: the bot declares every eligible
/// creature in 73 % of its combats, and 41 % of its creatures are tapped
/// when it is asked to block. Half of the combats where it has no blocker
/// at all are tapped-out boards rather than empty ones.
///
/// So this searches instead. The greedy declaration seeds the candidate
/// set; the alternatives are "attack with nobody" and the greedy set minus
/// one attacker each. Every candidate is played forward through our combat
/// damage, the rest of our turn, and the opponent's crack-back, then scored
/// with the same evaluator everything else uses — which already prices both
/// the life we took and the creatures we kept.
///
/// Restricted to *dropping* attackers on purpose: greedy already attacks
/// with 77 % of eligible creatures, so its error is over-attacking, and a
/// one-step hill climb toward restraint targets that error directly
/// without paying for a search over subsets the bot will never want.
///
/// **Tried and reverted**: forcing unblockable attackers into every
/// candidate, on the theory that free damage should never be declined.
/// It measured *worse* — 51.4 % [50.3 %, 52.4 %] against this version's
/// 52.4 % [51.3 %, 53.5 %] on the same seed and sample — and did nothing
/// for the dimir mirror it was aimed at (44.0 % against 44.8 %). The
/// reasoning was simply wrong: evasion is about being *blocked*, not about
/// *blocking*, so a 2/3 flier that no ground board can stop on offense is
/// still a perfectly good blocker on defense. Forcing it to attack deletes
/// a real option. See `EvalWeights::attack_search` for the dimir deficit,
/// which remains open.
///
/// Ties go to the greedy set, so the search only ever departs from the
/// current behavior for a strict improvement.
/// The attack declarations worth scoring: greedy first (index 0 wins
/// every tie), all-home, then greedy-minus-one holdbacks ordered by
/// toughness ascending — the cheapest body to keep home is also the one
/// most likely to die attacking, so the front of the order is where
/// both halves of the trade are largest. Shared by the sim search below
/// and the Monte Carlo bot, so the two search identical menus.
pub(crate) fn attack_candidates_for_mcts(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
) -> Vec<Vec<Attack>> {
    let greedy = pick_attacks(state, seat);
    if w.attack_search == 0 || greedy.is_empty() {
        return vec![greedy];
    }
    if w.attack_skip_open && board_open_for_attack(state, seat) {
        return vec![greedy];
    }
    let mut candidates: Vec<Vec<Attack>> = vec![greedy.clone(), Vec::new()];
    if greedy.len() > 1 {
        let mut order: Vec<usize> = (0..greedy.len()).collect();
        order.sort_by_cached_key(|&i| {
            state.battlefield_find(greedy[i].attacker).map(|c| c.toughness()).unwrap_or(0)
        });
        for &i in order.iter().take(w.attack_search as usize) {
            let mut alt = greedy.clone();
            alt.remove(i);
            candidates.push(alt);
        }
    }
    // CR 508.1d — every holdback here is a *subset*, and a subset that
    // leaves an obliged attacker home is rejected whole: the candidate's
    // opening `dry_run` fails, it scores `None`, and the search silently
    // spends a slot on a declaration it could never make. Twenty-two of the
    // residual attack rejections on `cube --seed 11` are exactly this, and
    // "attack with nobody" is one of them whenever anything is goaded.
    //
    // Repair rather than discard: an obliged attacker is put back with the
    // target the greedy declaration gave it, so "all home" becomes "only the
    // obliged ones" — a legal alternative the menu could not express before
    // — and a holdback that repairs back into greedy is deduped away rather
    // than scored twice.
    repair_attack_subsets(state, seat, &greedy, &mut candidates);
    // Walker chip candidate (flag): the greedy pass only attacks a
    // walker it can finish, so a healthy one sits unpressured to its
    // ultimate. One extra declaration — the smallest attacker with
    // power ≥ 2 redirected at the lowest-loyalty opposing walker — and
    // the simulations judge whether taxing its loyalty beats the face
    // damage given up.
    if w.walker_chip
        && !greedy.iter().any(|a| matches!(a.target, AttackTarget::Planeswalker(_)))
        && let Some(walker) = state
            .battlefield
            .iter()
            .filter(|c| {
                c.definition.is_planeswalker()
                    && c.controller != seat
                    && state.players[c.controller].is_alive()
                    && !state.permanent_cant_be_attacked(c.id)
            })
            .min_by_key(|c| c.counter_count(crate::card::CounterType::Loyalty))
        && let Some(i) = greedy
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                state.battlefield_find(a.attacker).map(|c| c.power()).unwrap_or(0) >= 2
            })
            .min_by_key(|(_, a)| {
                state.battlefield_find(a.attacker).map(|c| c.power()).unwrap_or(0)
            })
            .map(|(i, _)| i)
    {
        let mut alt = greedy.clone();
        alt[i].target = AttackTarget::Planeswalker(walker.id);
        candidates.push(alt);
    }
    candidates
}

/// CR 508.1d repair for a menu of attack subsets (see the note in
/// [`attack_candidates_for_mcts`]): an obliged attacker a subset leaves
/// home is put back with the target the greedy declaration gave it, and a
/// subset that repairs into an earlier one is deduped away rather than
/// scored twice. One freeze scope for the sweep: the repair reads
/// `computed_permanent` per own creature per candidate, and outside a
/// scope each read rebuilds the layer gather. No-op on a board with no
/// attack requirement in scope.
fn repair_attack_subsets(
    state: &GameState,
    seat: usize,
    greedy: &[Attack],
    candidates: &mut Vec<Vec<Attack>>,
) {
    if !state.with_frozen_layers(attack_requirement_present) {
        return;
    }
    let statics = crate::game::combat::attack_static_scan(state);
    let power_caps = state.attack_power_caps(statics);
    state.with_frozen_layers(|st| {
        for cand in candidates.iter_mut() {
            let mut ids: Vec<CardId> = cand.iter().map(|a| a.attacker).collect();
            let before = ids.len();
            restore_forced_attackers_unchecked(st, seat, &power_caps, statics, &mut ids);
            for id in ids.into_iter().skip(before) {
                if let Some(a) = greedy.iter().find(|a| a.attacker == id) {
                    cand.push(*a);
                }
            }
        }
    });
    let mut seen: Vec<Vec<u32>> = Vec::new();
    candidates.retain(|c| {
        let ids = attack_set_key(c);
        let fresh = !seen.contains(&ids);
        if fresh {
            seen.push(ids);
        }
        fresh
    });
}

/// A declaration's identity for menu dedupe: its attacker ids, sorted.
/// Targets are ignored on purpose — the walker-chip candidate is the one
/// same-set-different-target declaration, and it is pushed after dedupe.
fn attack_set_key(c: &[Attack]) -> Vec<u32> {
    let mut ids: Vec<u32> = c.iter().map(|a| a.attacker.0).collect();
    ids.sort_unstable();
    ids
}

/// The attack chain's pool: every creature the engine will accept as an
/// attacker — `may_declare_attacker`, the greedy picker's own gate — not
/// the greedy set, so a creature the greedy filters refused is on offer
/// and priced by the sim rather than by the rule that refused it.
/// Attackers keep the target greedy gave them; the rest aim at the
/// default face. Resolved by [`pick_attacks_scored`] ahead of the sims:
/// an empty pool under an empty greedy is a one-candidate menu with
/// nothing to add, and it used to pay one full turn-cycle sim of "nobody"
/// to feed an argmax of one (18.7 k of 22.2 k empty-greedy searches,
/// sealed, 2,400 games, round 59's census).
struct AttackPool {
    attackers: Vec<Attack>,
    /// Greedy declared nobody (the wide flag's board).
    from_empty: bool,
}

fn attack_chain_pool(state: &GameState, seat: usize, greedy: &[Attack]) -> AttackPool {
    let face_player = attack_target_player(state, seat);
    let face = AttackTarget::Player(face_player);
    let statics = crate::game::combat::attack_static_scan(state);
    let power_caps = state.attack_power_caps(statics);
    let attackers: Vec<Attack> = state.with_frozen_layers(|st| {
        st.battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && !c.tapped
                    && c.definition.is_creature()
                    && st.may_declare_attacker(
                        seat,
                        c,
                        st.computed_permanent_on(c).as_deref(),
                        &power_caps,
                        Some(face_player),
                    )
            })
            .map(|c| Attack {
                attacker: c.id,
                target: greedy
                    .iter()
                    .find(|a| a.attacker == c.id)
                    .map(|a| a.target)
                    .unwrap_or(face),
            })
            .collect()
    });
    AttackPool { attackers, from_empty: greedy.is_empty() }
}

/// The attack chain (see [`EvalWeights::attack_chain`]): grow a
/// declaration from the obliged-only set one creature at a time, keeping
/// an addition only when its simulated turn cycle scores strictly above
/// finalizing the set so far. Returns the finished set and its score, or
/// `None` when the start set could not be scored.
///
/// `pool` is [`attack_chain_pool`]'s; `None` when it is empty.
///
/// `menu` / `menu_scores` are the declarations [`pick_attacks_scored`] has
/// already simulated (index 0 greedy): the chain's start set is the menu's
/// repaired "all home" whenever that survived dedupe, so its score is
/// reused instead of re-simulated. `pool.from_empty` says greedy declared
/// nobody, which is what [`EvalWeights::attack_pairs_empty_only`] gates on.
fn attack_chain_candidate(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    menu: &[Vec<Attack>],
    menu_scores: &[(usize, i32)],
    pool: AttackPool,
    starts: &SimStarts,
) -> Option<(Vec<Attack>, i32)> {
    let greedy: &[Attack] = menu.first().map(Vec::as_slice).unwrap_or(&[]);
    let AttackPool { attackers: pool, from_empty } = pool;
    if pool.is_empty() {
        return None;
    }
    // Start from "nobody", repaired: the obliged attackers only.
    let mut start = vec![Vec::new()];
    repair_attack_subsets(state, seat, greedy, &mut start);
    let mut current = start.swap_remove(0);
    let start_key = attack_set_key(&current);
    let mut current_score = match menu_scores
        .iter()
        .find(|(i, _)| menu.get(*i).is_some_and(|c| attack_set_key(c) == start_key))
    {
        Some(&(_, s)) => {
            attack_census::add(11, 1);
            s
        }
        None => simulate_attack_outcome_from(starts, seat, &current, w)?,
    };
    let mut remaining: Vec<Attack> =
        pool.into_iter().filter(|a| !current.iter().any(|c| c.attacker == a.attacker)).collect();
    let mut sims = 0u64;
    // Candidate 0 is always "finalize": the set so far, at its known score.
    // First-wins-ties in `choose_scored` makes a tie stop the chain.
    let push_pairs = |cands: &mut Vec<Vec<Attack>>, current: &[Attack], remaining: &[Attack]| {
        for i in 0..remaining.len() {
            for j in (i + 1)..remaining.len() {
                let mut c = current.to_vec();
                c.push(remaining[i]);
                c.push(remaining[j]);
                cands.push(c);
            }
        }
    };
    let mut price = |cands: &mut Vec<Vec<Attack>>, current_score: i32| -> (usize, Vec<(usize, i32)>) {
        repair_attack_subsets(state, seat, greedy, cands);
        let mut scored: Vec<(usize, i32)> = vec![(0, current_score)];
        for (i, c) in cands.iter().enumerate().skip(1) {
            sims += 1;
            if let Some(s) = simulate_attack_outcome_from(starts, seat, c, w) {
                scored.push((i, s));
            }
        }
        (choose_scored(state.turn_number, &scored).unwrap_or(0), scored)
    };
    for step in 0..w.attack_chain {
        if remaining.is_empty() {
            break;
        }
        // The wide chain's first step also offers every pair: two
        // attackers into one blocker connect where each alone is blocked
        // and traded, so a single step ties and the chain would stop.
        // `attack_pairs_empty_only` keeps that to the empty-greedy board it
        // was built for; `attack_pairs_lazy` prices the pairs only once the
        // singles have tied.
        let pairs = step == 0
            && w.attack_chain_wide
            && remaining.len() >= 2
            && (from_empty || !w.attack_pairs_empty_only);
        let mut cands: Vec<Vec<Attack>> = Vec::with_capacity(remaining.len() + 1);
        cands.push(current.clone());
        for a in &remaining {
            let mut c = current.clone();
            c.push(*a);
            cands.push(c);
        }
        if pairs && !w.attack_pairs_lazy {
            push_pairs(&mut cands, &current, &remaining);
        }
        let (mut chosen, mut scored) = price(&mut cands, current_score);
        if chosen == 0 && pairs && w.attack_pairs_lazy {
            cands = vec![current.clone()];
            push_pairs(&mut cands, &current, &remaining);
            (chosen, scored) = price(&mut cands, current_score);
        }
        if chosen == 0 {
            break;
        }
        current_score = scored.iter().find(|(i, _)| *i == chosen).map(|&(_, s)| s).unwrap_or(current_score);
        current = cands.swap_remove(chosen);
        remaining.retain(|a| !current.iter().any(|c| c.attacker == a.attacker));
    }
    attack_census::add(10, sims);
    Some((current, current_score))
}

/// No opposing seat controls a creature, planeswalker or battle — the board
/// [`EvalWeights::attack_skip_open`] takes the greedy declaration on.
///
/// Printed types first, which settles the common board with no layer
/// gather; a non-creature permanent is then asked for its computed types
/// inside one freeze scope, so an animated land or a Vehicle under a
/// "becomes a creature" effect still counts as a blocker. A land that
/// *could* animate at instant speed does not: the search's own sims never
/// activate one either, so the shortcut loses nothing the search had.
fn board_open_for_attack(state: &GameState, seat: usize) -> bool {
    use crate::card::{CardInstance, CardType};
    let opposing = |c: &CardInstance| {
        c.controller != seat
            && state.players[c.controller].is_alive()
            && !state.same_team(seat, c.controller)
    };
    if state.battlefield.iter().any(|c| {
        opposing(c)
            && (c.definition.is_creature()
                || c.definition.is_planeswalker()
                || c.definition.is_battle())
    }) {
        return false;
    }
    state.with_frozen_layers(|st| {
        !st.battlefield.iter().filter(|c| opposing(c)).any(|c| {
            st.computed_permanent_on(c).is_some_and(|cp| {
                cp.card_types().iter().any(|t| {
                    matches!(t, CardType::Creature | CardType::Planeswalker | CardType::Battle)
                })
            })
        })
    })
}

/// The saturation fallback's switch (see [`EvalWeights::net_tail_guard`]):
/// weights for scoring ONE decision's candidates. When the flag is on and
/// the net reads the current state outside the rankable band
/// [0.05, 0.95] (the `--calibrate` histogram's saturation definition),
/// the returned copy has `net_slot` zeroed, so every leaf this decision
/// settles is scored by the material eval — linear, and therefore still
/// able to order "attack for five free damage" above "hold" when the
/// win probability of both rounds to the same integer. Keyed on the
/// pre-decision state, never per leaf: candidates of one argmax must
/// share a currency, or the comparison at the band's edge is between a
/// probability and an unbounded material score.
fn tail_guarded(state: &GameState, seat: usize, w: &EvalWeights) -> EvalWeights {
    if !w.net_tail_guard || w.net_slot == 0 {
        return *w;
    }
    match super::net_eval::win_prob(state, seat, w.net_slot) {
        Some(p) if !(0.05..=0.95).contains(&p) => EvalWeights { net_slot: 0, ..*w },
        _ => *w,
    }
}

fn pick_attacks_scored(state: &GameState, seat: usize, w: &EvalWeights) -> Vec<Attack> {
    let w = &tail_guarded(state, seat, w);
    let mut candidates = attack_candidates_for_mcts(state, seat, w);
    // A one-candidate menu is greedy alone (search off, open board) or
    // "nobody" (greedy refused everything). The wide chain prices the
    // second case instead of returning it (`EvalWeights::attack_chain_wide`).
    let chain_from_empty = w.attack_chain > 0
        && w.attack_chain_wide
        && w.attack_search > 0
        && candidates.len() == 1
        && candidates[0].is_empty();
    if candidates.len() == 1 && !chain_from_empty {
        return candidates.swap_remove(0);
    }
    // The chain's pool, ahead of the sims: an empty greedy with nothing
    // eligible to attack (every creature sick, tapped or barred) is a
    // one-candidate menu the chain cannot extend, so it is returned as it
    // stands rather than simulated for an argmax of one.
    let mut pool = if w.attack_chain > 0 {
        attack_chain_pool(state, seat, candidates.first().map(Vec::as_slice).unwrap_or(&[]))
    } else {
        AttackPool { attackers: Vec::new(), from_empty: false }
    };
    // The picker's own gate, not the greedy set's emptiness: identical
    // whenever the chain runs, and this is the one the census counts.
    pool.from_empty = chain_from_empty;
    if chain_from_empty && pool.attackers.is_empty() {
        return candidates.swap_remove(0);
    }
    // The blocker gate (`EvalWeights::attack_empty_gate`, REFUTED in
    // round 59 and kept as the ladder's control): one untapped defender
    // per untapped creature of ours. Its walk is paid only when the flag
    // or the census asks.
    let gate_covers = chain_from_empty
        && (w.attack_empty_gate || attack_census::on())
        && empty_greedy_gate_covers(state, seat);
    if chain_from_empty {
        attack_census::add(12, 1);
        if gate_covers && w.attack_empty_gate {
            attack_census::add(15, 1);
            return candidates.swap_remove(0);
        }
    }

    // First-wins-ties in `choose_scored`: index 0 is greedy, so equal
    // scores keep it (unless this thread is sampling — actors only).
    let starts = SimStarts::new(state, seat, w);
    let mut scored: Vec<(usize, i32)> = Vec::new();
    for (i, cand) in candidates.iter().enumerate() {
        let Some(score) = simulate_attack_outcome_from(&starts, seat, cand, w) else { continue };
        scored.push((i, score));
    }
    // The chain's finished set is one more candidate in the same argmax
    // (see `EvalWeights::attack_chain`) — appended, so greedy keeps index
    // 0 and every tie, and skipped when the menu already holds that set.
    let menu_len = candidates.len();
    if w.attack_chain > 0
        && let Some((chain, score)) =
            attack_chain_candidate(state, seat, w, &candidates, &scored, pool, &starts)
        && !candidates.iter().any(|c| attack_set_key(c) == attack_set_key(&chain))
    {
        candidates.push(chain);
        scored.push((menu_len, score));
    }
    let chosen = choose_scored(state.turn_number, &scored).unwrap_or(0);
    if attack_census::on() {
        attack_census::tick(
            state,
            seat,
            menu_len,
            chosen,
            &scored,
            attack_census::Chain {
                novel: candidates.len() > menu_len,
                from_empty: chain_from_empty,
                gate_covers,
            },
        );
    }
    candidates.swap_remove(chosen)
}

/// [`EvalWeights::attack_empty_gate`]'s board test: the defending seats'
/// untapped creatures that may block, against this seat's untapped
/// creatures — instance reads only, plus the computed `CantBlock` read the
/// greedy picker makes behind the same presence gate. `true` says every
/// attacker the chain could offer can be met by its own blocker.
fn empty_greedy_gate_covers(state: &GameState, seat: usize) -> bool {
    use crate::card::Keyword;
    let cant_block_in_scope = state.board_keyword_in_scope(&[Keyword::CantBlock]);
    let mut ours = 0usize;
    let mut theirs = 0usize;
    for c in state.battlefield.iter() {
        if !c.definition.is_creature() || c.tapped {
            continue;
        }
        if c.controller == seat {
            ours += 1;
        } else if state.players[c.controller].is_alive()
            && !state.same_team(seat, c.controller)
            && c.can_block()
            && !(cant_block_in_scope
                && state
                    .computed_permanent_on(c)
                    .is_some_and(|cp| cp.keywords().has_kw(&Keyword::CantBlock)))
        {
            theirs += 1;
        }
    }
    ours > 0 && theirs >= ours
}

/// `CRAB_ATTACK_CENSUS` — what the attack search decides, counted.
///
/// `pick_attacks_scored` is ~60 % of a `cube` game (PERF `(-21)`), and "the
/// candidate count is a search-quality decision" has been re-asserted since
/// the forty-ninth pass without anyone reading what the search *chooses*.
/// Per searched declaration: the candidates simulated, which index won
/// (greedy / no attack / a holdback), how many candidates tied the winner,
/// and whether the defending seats had a creature at all. Off unless the
/// variable is set; one `OnceLock` read per searched declaration.
///
/// ```text
/// CRAB_ATTACK_CENSUS=1 bot_ladder --a gang --b gang --games 6 --threads 1 --decks cube
/// ```
pub mod attack_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    use crate::game::GameState;

    /// `[calls, candidates, won greedy, won none, won holdback, tied with
    /// the winner, defender had no creature, ... and greedy won there,
    /// the attack chain proposed a set the menu lacked, ... and it won,
    /// sims the chain ran, chain start scores reused from the menu, chains
    /// run from an empty greedy (the wide flag), ... of which the chain
    /// proposed a set, ... and it won, empty-greedy searches the blocker
    /// gate covers (`attack_empty_gate`, counted whether or not it is on),
    /// ... of which the chain won, then six slots of holdback wins by menu
    /// index (greedy-minus-one #1..#6, the last slot folding the rest) and
    /// six of holdbacks OFFERED at that index]`.
    pub static N: [AtomicU64; 29] = [const { AtomicU64::new(0) }; 29];

    /// Bump counter `i` by `n` when the census is on.
    pub fn add(i: usize, n: u64) {
        if on() && n > 0 {
            N[i].fetch_add(n, Relaxed);
        }
    }

    /// 0 = off, 1 = count, 2 = count and name each creatureless-defender
    /// search the greedy declaration did not win.
    pub fn level() -> u8 {
        static LEVEL: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
        *LEVEL.get_or_init(|| match std::env::var("CRAB_ATTACK_CENSUS") {
            Ok(v) if v == "2" => 2,
            Ok(v) => u8::from(!v.is_empty() && v != "0"),
            _ => 0,
        })
    }

    pub fn on() -> bool {
        level() > 0
    }

    /// What the chain did on one search, for [`tick`].
    #[derive(Clone, Copy, Default)]
    pub(super) struct Chain {
        /// The chain proposed a set the menu lacked.
        pub novel: bool,
        /// Greedy declared nobody (the wide flag's board).
        pub from_empty: bool,
        /// The round-59 blocker gate covered the board.
        pub gate_covers: bool,
    }

    /// `candidates` is the menu size; `chosen >= candidates` is the chain's
    /// appended set, counted in N[9] rather than as a holdback.
    pub(super) fn tick(
        state: &GameState,
        seat: usize,
        candidates: usize,
        chosen: usize,
        scored: &[(usize, i32)],
        chain: Chain,
    ) {
        let Chain { novel: chain_novel, from_empty, gate_covers } = chain;
        N[0].fetch_add(1, Relaxed);
        N[1].fetch_add(candidates as u64, Relaxed);
        if chosen >= candidates {
            N[9].fetch_add(1, Relaxed);
            if from_empty {
                N[14].fetch_add(1, Relaxed);
            }
            if gate_covers {
                N[16].fetch_add(1, Relaxed);
            }
        } else {
            N[2 + chosen.min(2)].fetch_add(1, Relaxed);
            if chosen >= 2 {
                N[17 + (chosen - 2).min(5)].fetch_add(1, Relaxed);
            }
        }
        for idx in 0..candidates.saturating_sub(2).min(6) {
            N[23 + idx].fetch_add(1, Relaxed);
        }
        if chain_novel {
            N[8].fetch_add(1, Relaxed);
            if from_empty {
                N[13].fetch_add(1, Relaxed);
            }
        }
        if gate_covers {
            N[15].fetch_add(1, Relaxed);
        }
        if let Some(&(_, best)) = scored.iter().find(|(i, _)| *i == chosen) {
            let tied = scored.iter().filter(|(i, s)| *i != chosen && *s == best).count();
            N[5].fetch_add(tied as u64, Relaxed);
        }
        let defender_has_creature = state.battlefield.iter().any(|c| {
            c.controller != seat
                && c.definition.is_creature()
                && state.players[c.controller].is_alive()
        });
        if !defender_has_creature {
            N[6].fetch_add(1, Relaxed);
            if chosen == 0 {
                N[7].fetch_add(1, Relaxed);
            } else if level() >= 2 {
                let mine: Vec<&str> = state
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == seat && c.definition.is_creature())
                    .map(|c| c.definition.name)
                    .collect();
                let theirs: Vec<String> = state
                    .battlefield
                    .iter()
                    .filter(|c| c.controller != seat)
                    .map(|c| format!("{}{}", c.definition.name, if c.tapped { "(T)" } else { "" }))
                    .collect();
                let opp: Vec<String> = state
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(s, p)| *s != seat && p.is_alive())
                    .map(|(_, p)| format!("life {} hand {}", p.life, p.hand.len()))
                    .collect();
                eprintln!(
                    "attack_census turn {} seat {seat} chose {chosen} scores {scored:?} mine {mine:?} \
                     theirs {theirs:?} opp {opp:?}",
                    state.turn_number,
                );
            }
        }
    }

    pub fn snapshot() -> [u64; 29] {
        std::array::from_fn(|i| N[i].load(Relaxed))
    }
}

/// The block side of [`attack_census`], on the same `CRAB_ATTACK_CENSUS`
/// switch: what the block search decides, and what the block chain adds.
pub mod block_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// `[calls, menu candidates, sims the chain ran, the chain proposed a
    /// plan the menu lacked, ... and it won, chain start scores reused,
    /// chains that reached their start (the reuse denominator: a search
    /// with no free blocker or no legal pair returns before it)]`.
    pub static N: [AtomicU64; 7] = [const { AtomicU64::new(0) }; 7];

    pub fn on() -> bool {
        super::attack_census::on()
    }

    pub fn add(i: usize, n: u64) {
        if on() && n > 0 {
            N[i].fetch_add(n, Relaxed);
        }
    }

    pub(super) fn tick(menu: usize, chosen: usize, chain_novel: bool) {
        N[0].fetch_add(1, Relaxed);
        N[1].fetch_add(menu as u64, Relaxed);
        if chain_novel {
            N[3].fetch_add(1, Relaxed);
            if chosen >= menu {
                N[4].fetch_add(1, Relaxed);
            }
        }
    }

    pub fn snapshot() -> [u64; 7] {
        std::array::from_fn(|i| N[i].load(Relaxed))
    }
}

/// Declare `attacks`, play the turn out through the opponent's counter-
/// attack, and score the board for `seat`.
///
/// The horizon matters more than the fidelity here. Scoring right after our
/// own combat damage would make holding a creature back *strictly* worse —
/// we dealt less damage and gained nothing measurable — because the entire
/// payoff of restraint is that the creature is untapped on the opponent's
/// turn. So the simulation has to reach their combat damage or it cannot
/// see the thing it exists to weigh.
///
/// By default neither side casts anything during the simulation; both take
/// the greedy combat declarations. That's a real simplification — an
/// opponent holding removal, or ourselves holding a trick, are invisible —
/// but it keeps the cost to one turn cycle of priority passes per
/// candidate, and the greedy declarations are exactly the policy this
/// search is trying to beat, which makes the comparison conservative
/// rather than flattering. Under
/// [`attack_sim_spells`](EvalWeights::attack_sim_spells) both seats cast
/// via [`sim_spell_action`], which is what makes the crack-back visible.
///
/// `None` when the declaration is rejected (a "must attack" creature we
/// tried to hold back) or the simulation runs out of fuel — an unfinished
/// turn is scored not at all rather than scored wrong, the same rule
/// [`simulate_through_combat`] settled on.
#[cfg(test)]
fn simulate_attack_outcome(
    state: &GameState,
    seat: usize,
    attacks: &[Attack],
    w: &EvalWeights,
) -> Option<i32> {
    simulate_attack_outcome_from(&SimStarts::new(state, seat, w), seat, attacks, w)
}

/// [`simulate_attack_outcome_from`] from a decision's prepared start states.
fn simulate_attack_outcome_from(
    starts: &SimStarts,
    seat: usize,
    attacks: &[Attack],
    w: &EvalWeights,
) -> Option<i32> {
    if w.determinize > 1 {
        let mut total = 0i64;
        let mut n = 0i64;
        for k in 0..w.determinize {
            if let Some(v) = simulate_attack_outcome_once(starts.base(k), seat, attacks, w) {
                total += v as i64;
                n += 1;
            }
        }
        return (n > 0).then(|| (total / n) as i32);
    }
    simulate_attack_outcome_once(starts.base(0), seat, attacks, w)
}

/// One action on a throwaway dry-run clone.
///
/// `perform_action`'s transaction checkpoint clones the whole state so a
/// *rejected* action can be restored — and every caller of this helper throws
/// the clone away on `Err`, so the restore is never read. The checkpoint is
/// not free twice over: it also shares every CoW zone, so the simulation's
/// next write deep-copies one. Use this wherever an `Err` ends the dry run.
#[inline]
fn dry_run(
    g: &mut GameState,
    action: GameAction,
) -> Result<Vec<crate::game::GameEvent>, crate::game::GameError> {
    g.perform_action_inner(action)
}

/// The simulation loops' action step. A rejected action abandons the
/// simulation; `false` says so.
///
/// **This used to take `perform_action`'s transaction checkpoint so a rejected
/// declaration could be rolled back and retried as a priority pass, and that
/// checkpoint was ~0.9 % of a `cube` run for a path that never runs.** The
/// census below is what settled it: `CRAB_SIM_REJECTS` reads **0 in 73
/// configurations** across five pools, and the sim owns `g` outright — a
/// caller that gets `false` drops it unread, so a partially-mutated state is
/// never observed. That is also the semantics
/// [`simulate_attack_outcome_from`]'s own doc already promises for the *opening*
/// declaration ("an unfinished turn is scored not at all rather than scored
/// wrong"); this makes the loop's later declarations agree with it.
///
/// The one error kept: `ManualTapRequired` is a suspension dressed as an
/// `Err`, and `perform_action` deliberately does not roll it back because it
/// leaves exactly the state a retry would have seen — so the retry is kept and
/// reads the same, here as on the pass branch.
///
/// [`simulate_attack_outcome_from`]: fn@simulate_attack_outcome
/// The simulation's own declaration pickers against the engine, counted.
///
/// A picker that proposes an illegal attack or block shows up nowhere on its
/// own — not in the suite, not in the traces, not in `--bench`. PERF's (-54)
/// asked for the failure count, (-55) is what it found, and it is now the
/// guard that keeps `sim_step`'s abandon-on-`Err` honest: **run it before and
/// after anything that touches a picker or a combat check, and sweep seeds
/// rather than sampling three.** Off by default and gated by one `OnceLock`
/// read, so the hot path pays an atomic load and a branch.
///
/// ```text
/// CRAB_SIM_REJECTS=1     bot_ladder … --decks cube    # the counts
/// CRAB_SIM_REJECTS=names bot_ladder … --threads 1     # + card, sickness, computed keywords
/// ```
pub mod sim_rejects {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// `[attack, block, other]` — proposals and rejections per action kind.
    pub static CALLS: [AtomicU64; 3] =
        [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
    pub static ERRS: [AtomicU64; 3] =
        [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];

    /// 0 = off, 1 = count, 2 = count and name each rejection. One reader,
    /// shared with the engine-side site tags — see
    /// [`crate::game::reject_trace_level`].
    pub fn level() -> u8 {
        crate::game::reject_trace_level()
    }

    /// `(proposals, rejections)` per kind, for a caller that wants to print.
    pub fn snapshot() -> [(u64, u64); 3] {
        std::array::from_fn(|i| (CALLS[i].load(Relaxed), ERRS[i].load(Relaxed)))
    }
}

fn sim_step(g: &mut GameState, action: GameAction) -> bool {
    if matches!(action, GameAction::PassPriority) {
        // The events are read by nobody here, so the buffer goes back for its
        // capacity — see `GameState::recycle_events`. This is the largest of
        // the four such sites: 34,298 of the 66,612 `perform_action_inner`
        // calls on a six-game `fixed` run come through here, on a simulation
        // clone that lives for the whole simulation.
        return match g.perform_action_inner(GameAction::PassPriority) {
            Ok(events) => {
                g.recycle_events(events);
                true
            }
            Err(crate::game::GameError::ManualTapRequired { .. }) => {
                match g.perform_action_inner(GameAction::PassPriority) {
                    Ok(events) => {
                        g.recycle_events(events);
                        true
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        };
    }
    if sim_rejects::level() == 0 {
        let r = dry_run(g, action);
        return sim_outcome(g, r);
    }
    let kind = match &action {
        GameAction::DeclareAttackers(_) => 0usize,
        GameAction::DeclareBlockers(_) => 1,
        _ => 2,
    };
    sim_rejects::CALLS[kind].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let r = dry_run(g, action);
    if let Err(e) = &r {
        sim_rejects::ERRS[kind].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if sim_rejects::level() >= 2 {
            let id = match e {
                crate::game::GameError::CannotAttack(i)
                | crate::game::GameError::SummoningSickness(i)
                | crate::game::GameError::CannotBlock(i)
                | crate::game::GameError::MustBeBlockedIfAble(i) => Some(*i),
                _ => None,
            };
            let card = id.and_then(|i| g.find_card_anywhere(i));
            eprintln!(
                "sim_reject kind={kind} {e:?} name={} sick={:?} computed_kw={}",
                card.map(|c| c.definition.name).unwrap_or("?"),
                card.map(|c| c.summoning_sick),
                id.and_then(|i| g.computed_permanent(i))
                    .map(|cp| format!("{:?}", cp.keywords()))
                    .unwrap_or_else(|| "-".into()),
            );
        }
    }
    sim_outcome(g, r)
}

/// `sim_step`'s verdict on one engine call: keep going, or abandon.
///
/// The `ManualTapRequired` retry is the whole reason this is a function and
/// not `is_ok()` — see [`sim_step`].
fn sim_outcome(
    g: &mut GameState,
    r: Result<Vec<crate::game::GameEvent>, crate::game::GameError>,
) -> bool {
    match r {
        Ok(_) => true,
        Err(crate::game::GameError::ManualTapRequired { .. }) => {
            dry_run(g, GameAction::PassPriority).is_ok()
        }
        Err(_) => false,
    }
}

fn simulate_attack_outcome_once(
    base: &GameState,
    seat: usize,
    attacks: &[Attack],
    w: &EvalWeights,
) -> Option<i32> {
    let mut g = base.clone();
    dry_run(&mut g, GameAction::DeclareAttackers(attacks.to_vec())).ok()?;
    let start_turn = g.turn_number;
    // One turn cycle of pure priority passes is on the order of fifty
    // actions; the rest is headroom for triggers and decisions.
    let mut fuel = 400u32;
    // Break when this turn's opponent combat has resolved; the race
    // horizon can push the stop out one more cycle (see below).
    let mut stop_turn = start_turn;
    let mut extended = false;
    let mut declared: crate::game::types::SmallIdSet<(u32, TurnStep)> = Default::default();
    // *This* turn's attack declaration is the candidate, already submitted.
    // Without this the loop's own DeclareAttackers arm fires on the same
    // turn and re-declares the greedy set over the top of it — which the
    // engine happily accepts, so every candidate silently collapses back to
    // the alpha strike and the whole search scores one line N times.
    declared.insert((g.turn_number, TurnStep::DeclareAttackers));
    while !g.is_game_over() {
        // Stop once the opponent's combat is resolved — the first board on
        // which a creature held back has actually done anything. Under
        // `attack_race_horizon`, a sim ending with either life total in
        // burn range runs one more full cycle instead, so the race this
        // attack started is scored at its result, not mid-sprint.
        if g.turn_number > stop_turn && g.step >= TurnStep::EndCombat {
            if w.attack_race_horizon
                && !extended
                && g.players.iter().any(|p| p.is_alive() && p.life <= 10)
            {
                extended = true;
                stop_turn = g.turn_number;
                fuel = fuel.saturating_add(400);
            } else {
                break;
            }
        }
        fuel = fuel.checked_sub(1)?;
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            dry_run(&mut g, GameAction::SubmitDecision(answer)).ok()?;
            continue;
        }
        // Declarations are one-shot per step per turn; the marker keeps a
        // rejected declaration from being retried forever.
        let key = (g.turn_number, g.step);
        let action = match g.step {
            TurnStep::DeclareAttackers if !declared.contains(&key) => {
                declared.insert(key);
                let declarer = g.attack_declarer();
                // Greedy, deliberately: calling the search here would
                // recurse a turn deeper on every candidate.
                GameAction::DeclareAttackers(pick_attacks(&g, declarer))
            }
            TurnStep::DeclareBlockers if !declared.contains(&key) && !g.attacking().is_empty() => {
                match (0..g.players.len()).find(|&s| g.may_declare_blocks(s)) {
                    Some(defender) => {
                        declared.insert(key);
                        GameAction::DeclareBlockers(pick_blocks(&g, defender))
                    }
                    None => GameAction::PassPriority,
                }
            }
            _ if w.attack_sim_spells => match sim_spell_action(&g, w) {
                // The spell layer's dry run already resolved this cast on a
                // clone of exactly this state; adopt it rather than run the
                // same cast a second time through `sim_step`.
                Some(Picked::Probed(_, next)) => {
                    g = *next;
                    continue;
                }
                Some(Picked::Plain(a)) => a,
                None => GameAction::PassPriority,
            },
            _ => GameAction::PassPriority,
        };
        if !sim_step(&mut g, action) {
            return None;
        }
    }
    let v = eval_material(&g, seat, w);
    super::leaf_capture::maybe(&g, seat, v);
    Some(v)
}

/// The spell a combat simulation lets the current priority holder cast —
/// see [`EvalWeights::attack_sim_spells`]. The response layer and the
/// post-block trick window mirror the real dispatch; a main phase inside
/// the sim's horizon takes the best STATIC-ranked candidate. No outcome
/// eval, no hold gates, no jitter: nesting the full pick inside a sim
/// would multiply clone-and-resolve work per candidate, and a
/// deterministic greedy stand-in carries exactly the information the sim
/// is missing — "that mana will be spent on something".
/// An action a picker chose, and — when a dry run validated it — the state
/// that run produced.
///
/// A validating picker's dry run *is* the action: it clones the state and runs
/// the action to completion, then throws the result away so somebody can run
/// the identical action a second time on a state equal to the one the probe
/// started from. A caller that **owns** its state (the attack and block
/// simulations, on their own throwaway clone) adopts `Probed`'s state and pays
/// for one execution. A caller whose action crosses the
/// [`Bot::next_action`](Bot::next_action) boundary drops it and returns the
/// action: the driver's state is the authoritative one, and its decider is
/// live where a clone's is fresh-by-kind.
enum Picked {
    /// [`GameState::accept`] / [`accept_on`](GameState::accept_on) ran this
    /// action on a clone, and this is what it produced. Boxed so the enum
    /// stays small on the far commoner other arm.
    Probed(GameAction, Box<GameState>),
    /// Chosen without a dry run of its own — pre-validated by
    /// `cast_candidates`, or from a picker that does not probe.
    Plain(GameAction),
}

impl Picked {
    /// The action, discarding any state its probe produced.
    #[cfg(test)]
    fn action(self) -> GameAction {
        match self {
            Picked::Probed(a, _) | Picked::Plain(a) => a,
        }
    }

    /// The picker's action together with its dry-run state (`Probed`) or
    /// none (`Plain`), packaged for [`Bot::next_action_settled`].
    fn into_step(self) -> BotStep {
        match self {
            Picked::Probed(action, settled) => BotStep { action, settled: Some(settled) },
            Picked::Plain(action) => BotStep::plain(action),
        }
    }
}

/// A validated candidate on its way to [`pick_by_outcome`], carrying the state
/// its validating dry run produced.
///
/// [`Picked`]'s bargain one level up. Every consumer of a finalist —
/// [`evaluate_action_sequence`] and [`improves_this_turn`] — opens by cloning
/// the *pre-action* state and running the action again, which is the run the
/// validating probe already did. `settled` is that run's result, so those two
/// clone it instead — a clone was ~2,200 Ir against a cast's ~46,000 when this
/// was measured (the fiftieth pass's (C)). `None`
/// means the candidate reached here without a probe of its own — a
/// `cast_candidates` block that validated it eagerly, or a caller with no
/// state to give — and the consumer does its own run as before.
struct Finalist {
    score: i32,
    action: GameAction,
    settled: Option<Box<GameState>>,
}

fn sim_spell_action(g: &GameState, w: &EvalWeights) -> Option<Picked> {
    // Called once per sim-loop iteration on a cloned (unfrozen) state; every
    // candidate it ranks runs layer-aware checks — so the body wants a freeze
    // scope, but the question of whether there is a window to act in does not.
    // `sim_spell_action_inner`'s three entry tests are plain field reads, and
    // on two thirds of the iterations all three miss and the closure returns
    // `None` having read nothing. Opening and closing a scope is not free (the
    // `Unfreeze` drop is most of it), so the window test runs out here.
    let p = g.player_with_priority();
    let window = !g.stack.is_empty()
        || (g.step == TurnStep::DeclareBlockers && g.blockers_declared())
        || (matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
            && g.active_player_idx == p);
    if !window {
        debug_assert!(
            g.with_frozen_layers(|g| sim_spell_action_inner(g, w)).is_none(),
            "sim_spell_action's window gate skipped a real action",
        );
        return None;
    }
    g.with_frozen_layers(|g| sim_spell_action_inner(g, w))
}

fn sim_spell_action_inner(g: &GameState, w: &EvalWeights) -> Option<Picked> {
    let p = g.player_with_priority();
    if !g.stack.is_empty() {
        return pick_stack_response(g, p, w)
            .or_else(|| pick_ability_counter_response(g, p, w).map(Picked::Plain))
            .or_else(|| pick_prepare_response(g, p, w).map(Picked::Plain))
            .or_else(|| pick_buff_response(g, p, w).map(Picked::Plain));
    }
    if g.step == TurnStep::DeclareBlockers && g.blockers_declared() {
        return pick_combat_trick(g, p, w);
    }
    if matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        && g.active_player_idx == p
    {
        let mut ranked: Vec<(i32, GameAction, bool)> = cast_candidates(g, p, w, None)
            .into_iter()
            .map(|(a, ok)| (score_candidate(g, p, &a, w), a, ok))
            .collect();
        ranked.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
        for (_, a, ok) in ranked {
            // A pre-validated candidate never ran, so there is nothing to
            // adopt. Everything else is dry-run here; keeping that run's state
            // is what stops the sim casting the same spell twice.
            if ok {
                return Some(Picked::Plain(a));
            }
            if let Some(next) = GameState::accept_on(g, a.clone()) {
                return Some(Picked::Probed(a, Box::new(next)));
            }
        }
    }
    None
}

/// The block assignment, chosen by search rather than by rule.
///
/// [`pick_blocks`] assigns blockers greedily, one at a time, in ascending
/// power order, each taking the best attacker it can find *given the
/// assignments already made*. That ordering is a heuristic standing in for
/// the thing it can't do: score the whole assignment. A first-fit choice
/// that looks locally best can spend the one blocker a later, bigger
/// attacker needed — and the greedy pass has no way to notice, because it
/// never looks at the board the block produces.
///
/// So this scores whole assignments. The greedy block seeds the candidate
/// set; the alternatives are "block with nobody" and the greedy assignment
/// minus one blocker each. Each is played through combat damage and scored
/// with the same evaluator, which already prices both the creatures that
/// died and the life that got through.
///
/// Cheaper than [`pick_attacks_scored`], and deliberately so: a block's
/// consequences are settled inside this combat, so the simulation stops at
/// end of combat instead of running a full turn cycle.
///
/// Ties go to the greedy assignment, so the search only ever departs from
/// current behavior for a strict improvement.
/// The block assignments worth scoring: greedy first (index 0 wins every
/// tie), no-blocks, greedy-minus-one releases (cheapest bodies first —
/// those are the chump-blocks the greedy pass throws in to save life,
/// and the ones most likely to be worth more alive than the damage they
/// absorb), then the gang candidates. Shared by the sim search below and
/// the Monte Carlo bot, so the two search identical menus.
pub(crate) fn block_candidates_for_mcts(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
) -> Vec<Vec<(CardId, CardId)>> {
    let greedy = pick_blocks(state, seat);
    let chumps =
        if w.chump_blocks { chump_block_candidates(state, seat, &greedy, w) } else { Vec::new() };
    // See `repair_block_candidate`: every candidate below `greedy` is a
    // subset, and a subset can release a provoked creature or strip the
    // second blocker off a Menace attacker.
    // Trim first, then repair: a requirement wins over the cost, so a batch
    // the rules force and the pool cannot pay is left for the engine to
    // reject rather than quietly made illegal a second way.
    let repair = |c: &mut Vec<(CardId, CardId)>| {
        trim_blocks_to_payable_tax(state, seat, c);
        repair_block_candidate(state, seat, c);
    };
    if (w.block_search == 0 || greedy.is_empty()) && chumps.is_empty() {
        return vec![greedy];
    }
    let mut candidates: Vec<Vec<(CardId, CardId)>> = vec![greedy.clone()];
    if !greedy.is_empty() {
        candidates.push(Vec::new());
    }
    candidates.extend(chumps);
    if greedy.len() > 1 {
        let mut order: Vec<usize> = (0..greedy.len()).collect();
        order.sort_by_cached_key(|&i| {
            state.battlefield_find(greedy[i].0).map(|c| c.toughness()).unwrap_or(0)
        });
        for &i in order.iter().take(w.block_search as usize) {
            let mut alt = greedy.clone();
            alt.remove(i);
            candidates.push(alt);
        }
    }

    if w.block_gang {
        candidates.extend(gang_block_candidates(state, seat, &greedy, w));
    }
    // One freeze scope for the whole sweep: every candidate's requirement and
    // cost read goes through `computed_permanent`, and outside a scope each
    // of those rebuilds the layer gather.
    state.with_frozen_layers(|st| {
        if !block_requirement_present(st) && !st.block_tax_present() {
            return;
        }
        for c in candidates.iter_mut() {
            repair(c);
        }
        // Repairing collapses candidates onto each other; index 0 stays
        // greedy, which is the tie-break the search depends on.
        let mut seen: Vec<Vec<(u32, u32)>> = Vec::new();
        candidates.retain(|c| {
            let mut key: Vec<(u32, u32)> = c.iter().map(|(b, a)| (b.0, a.0)).collect();
            key.sort_unstable();
            let fresh = !seen.contains(&key);
            if fresh {
                seen.push(key);
            }
            fresh
        });
    });
    candidates
}

/// Is any permanent on this board subject to a requirement a *subset* of a
/// legal block assignment can violate? One walk, so the repair below is only
/// paid on the boards that need it.
///
/// **A gate written as an enumeration of what it gates is a second copy of
/// that list, and this one was already a stale copy.** It named Provoke and
/// the Menace minimum and stopped there, so on a Lure board the repair never
/// ran at all and every subset went out unrepaired — 14 of `cube` seed 15's
/// block rejections, still there after `enforce_block_requirements` was
/// written, because nothing called it. The keyword leg is now
/// `board_keyword_in_scope` over the same five keywords
/// [`GameState::declare_blockers`] gates its own requirement loops on, so
/// adding a sixth requirement there is what widens this.
fn block_requirement_present(state: &GameState) -> bool {
    use crate::card::Keyword;
    state.battlefield.iter().any(|c| c.must_block.is_some())
        || state.board_keyword_in_scope(&[
            Keyword::MustBlock,
            Keyword::MustAttackOrBlock,
            Keyword::MustBeBlocked,
            Keyword::AllMustBlock,
            Keyword::CantBeBlockedUnlessAllBlock,
            // CR 509.1g's ceiling: a *superset* candidate (the gang and
            // spare-capacity passes both build one) violates it the way a
            // subset violates the floor.
            Keyword::CantBeBlockedByMoreThanOne,
        ])
        || state.attacking().iter().any(|a| {
            state
                .computed_permanent(a.attacker)
                .is_some_and(|cp| min_blockers_required_kws(cp.keywords()) > 1)
        })
}

/// CR 509.1b / 702.39 — make a block assignment legal in the two ways the
/// search's edits break it.
///
/// Every candidate below greedy is a **subset** — "block with nobody",
/// greedy minus one — and a subset can release a provoked creature or strip
/// the second blocker off a Menace attacker. Either makes the *whole*
/// declaration illegal, so the candidate's opening dry run fails, it scores
/// `None`, and the menu silently shrinks to whichever subsets happened to
/// stay legal. Same shape as the attack search's CR 508.1d repair.
///
/// Provoke runs first because it can add a block the count rule then sees;
/// the count rule in turn refuses to release a provoked assignment, so the
/// two cannot oscillate. A board where a provoked creature is the only
/// blocker a Menace attacker can have is unsatisfiable either way, and this
/// leaves it to the engine to say so.
/// The CR 509.1 block **requirements**, forced into a plan whose value passes
/// were built without them. Shared by the greedy planner and the search
/// menu's repair, because a rule written twice is a rule that drifts once.
///
/// Four requirements, and every one of them makes the *whole* declaration
/// illegal rather than the one pair:
///
/// * CR 702.39 Provoke — the provoked creature blocks its provoker.
/// * CR 509.1c true Lure (`AllMustBlock`) — **every** able defender blocks it,
///   and nothing else: the engine asks the question per creature without
///   caring what else that creature was assigned to, so the only satisfiable
///   declaration puts the whole able set on the Lure attacker.
/// * CR 509.1c `MustBlock` / `MustAttackOrBlock` — the creature blocks
///   *something* it is able to block.
/// * CR 509.1c `MustBeBlocked` is handled by the caller, which has the value
///   information to pick which body to spend.
///
/// Ability is [`GameState::block_requirement_able`] throughout — the engine's
/// own predicate, so the two cannot disagree about who is obliged. Returns the
/// pairs a requirement pinned: the CR 509.1b minimum-count repair must not
/// release them.
///
/// Order matters and it is Provoke first. A creature two requirements both
/// claim is an unsatisfiable board either way, so the pins are first-come and
/// nothing here overwrites one; the engine says so rather than this looping.
fn enforce_block_requirements(
    state: &GameState,
    seat: usize,
    blocks: &mut Vec<(CardId, CardId)>,
) -> Vec<(CardId, CardId)> {
    use crate::card::Keyword;
    let mut forced: Vec<(CardId, CardId)> = Vec::new();
    let mut pin = |blocks: &mut Vec<(CardId, CardId)>, b: CardId, a: CardId| {
        if forced.iter().any(|(bid, _)| *bid == b) {
            return;
        }
        blocks.retain(|(bid, aid)| *bid != b || *aid == a);
        if !blocks.iter().any(|(bid, aid)| *bid == b && *aid == a) {
            blocks.push((b, a));
        }
        forced.push((b, a));
    };

    // CR 702.39 — Provoke.
    for b in state.battlefield.iter() {
        let Some(required) = b.must_block else { continue };
        if b.controller != seat
            || !state.attacking().iter().any(|a| a.attacker == required)
            || !state.block_requirement_binds(required)
            || !state.block_requirement_able(b, required)
        {
            continue;
        }
        pin(blocks, b.id, required);
    }

    // The two keyword requirements below walk the board, so gate them on
    // whether the board can carry the keyword at all. `board_keyword_in_scope`
    // is authoritative on `false` and it is `false` on an ordinary board, so
    // this costs a keyword scan and no computed reads.
    if !state.board_keyword_in_scope(&[
        Keyword::AllMustBlock,
        Keyword::MustBlock,
        Keyword::MustAttackOrBlock,
    ]) {
        return forced;
    }

    // CR 509.1c — true Lure. Every able defender, displacing whatever else it
    // was doing; see the note above on why that is the only legal shape.
    let lured: Vec<CardId> = state
        .attacking()
        .iter()
        .filter(|a| state.defender_for(a.target) == Some(seat))
        .map(|a| a.attacker)
        .filter(|a| {
            state
                .computed_permanent(*a)
                .is_some_and(|cp| cp.keywords().has_kw(&Keyword::AllMustBlock))
                && state.block_requirement_binds(*a)
        })
        .collect();
    for a_id in lured {
        for b in state.battlefield.iter() {
            if b.controller != seat
                || state.blocks(b.id, a_id)
                || !state.block_requirement_able(b, a_id)
            {
                continue;
            }
            pin(blocks, b.id, a_id);
        }
    }

    // CR 509.1c — "blocks each combat if able". Into the attacker that hits
    // softest: the body is being spent either way, so spend it where it is
    // likeliest to come back.
    for b in state.battlefield.iter() {
        if b.controller != seat
            || state.is_blocking(b.id)
            || blocks.iter().any(|(bid, _)| *bid == b.id)
        {
            continue;
        }
        let obliged = state.computed_permanent_on(b).is_some_and(|cp| {
            cp.keywords().has_kw(&Keyword::MustBlock)
                || cp.keywords().has_kw(&Keyword::MustAttackOrBlock)
        });
        if !obliged {
            continue;
        }
        let into = state
            .attacking()
            .iter()
            .filter(|a| state.defender_for(a.target) == Some(seat))
            .map(|a| a.attacker)
            // Both gates on purpose. `block_requirement_able` is what obliges
            // the creature; `blocker_can_block_attacker` is what makes the
            // pair legal to declare. Where they disagree the requirement has
            // no legal form and no choice here helps — that is an engine
            // defect, filed in ENGINE_BACKLOG P3, not one to paper over.
            .filter(|a| {
                state.block_requirement_binds(*a)
                    && state.block_requirement_able(b, *a)
                    && state.blocker_can_block_attacker(b.id, *a)
            })
            // **CR 509.1b is a hard filter here, not a preference.** A lone
            // body on a Menace attacker is an under-filled multi-block, so
            // satisfying this requirement there does not save the
            // declaration — it moves the rejection from the requirement to
            // the count rule, and the count rule cannot repair it either
            // because releasing the block would put the requirement back.
            // The first cut of this pass preferred a legal attacker and took
            // an illegal one when that was all there was; it turned 92 of
            // `cube` seed 15's rejections into 20 of a different kind. When
            // no attacker can take this body legally, leave it idle: the
            // requirement fires either way, and this keeps the residual
            // attributable to the rule that actually cannot be satisfied.
            .filter(|a| {
                let min_b = state
                    .computed_permanent(*a)
                    .map_or(1, |cp| min_blockers_required_kws(cp.keywords()));
                min_b <= 1
                    || blocks.iter().filter(|(_, aid)| aid == a).count()
                        + state.blocker_count_of(*a)
                        + 1
                        >= min_b
            })
            .min_by_key(|a| state.computed_permanent(*a).map_or(0, |cp| cp.power));
        if let Some(a_id) = into {
            pin(blocks, b.id, a_id);
        }
    }
    forced
}

fn repair_block_candidate(state: &GameState, seat: usize, blocks: &mut Vec<(CardId, CardId)>) {
    let provoked = enforce_block_requirements(state, seat, blocks);
    let attackers: Vec<CardId> = state.attacking().iter().map(|a| a.attacker).collect();
    for a_id in attackers {
        let min_b = state
            .computed_permanent(a_id)
            .map(|cp| min_blockers_required_kws(cp.keywords()))
            .unwrap_or(1);
        if min_b <= 1 {
            continue;
        }
        let count =
            blocks.iter().filter(|(_, aid)| *aid == a_id).count() + state.blocker_count_of(a_id);
        if count == 0 || count >= min_b {
            continue;
        }
        blocks.retain(|(bid, aid)| *aid != a_id || provoked.contains(&(*bid, *aid)));
    }
    enforce_block_caps(state, blocks, &provoked);
}

/// A block plan's identity for menu dedupe: its (blocker, attacker) pairs,
/// sorted.
fn block_set_key(c: &[(CardId, CardId)]) -> Vec<(u32, u32)> {
    let mut key: Vec<(u32, u32)> = c.iter().map(|(b, a)| (b.0, a.0)).collect();
    key.sort_unstable();
    key
}

/// The repair [`block_candidates_for_mcts`] applies to its menu — the
/// payable-tax trim and the requirement/menace repair, inside one freeze
/// scope, only when a requirement or tax is in scope — followed by dedupe,
/// so a plan that repairs into an earlier one is not scored twice.
fn repair_block_plans(state: &GameState, seat: usize, candidates: &mut Vec<Vec<(CardId, CardId)>>) {
    state.with_frozen_layers(|st| {
        if !block_requirement_present(st) && !st.block_tax_present() {
            return;
        }
        for c in candidates.iter_mut() {
            trim_blocks_to_payable_tax(state, seat, c);
            repair_block_candidate(state, seat, c);
        }
    });
    let mut seen: Vec<Vec<(u32, u32)>> = Vec::new();
    candidates.retain(|c| {
        let key = block_set_key(c);
        let fresh = !seen.contains(&key);
        if fresh {
            seen.push(key);
        }
        fresh
    });
}

/// What the block chain needs before it can offer a move, resolved once
/// per decision: this seat's legal blockers, the attackers pointed at it,
/// and the legal pairs. `None` when there is nothing to add — no free
/// blocker, no attacker, no legal pair — which is what lets
/// [`pick_blocks_scored`] hand back a one-candidate menu without
/// simulating it: the chain ran on 65.6 % of block searches under the
/// round-56 default (`CRAB_ATTACK_CENSUS`, sealed, 1,200 games) and the
/// other third paid one combat sim of bare "no blocks" for an argmax of
/// one.
struct BlockChainSetup<'a> {
    blockers: Vec<(&'a crate::card::CardInstance, std::sync::Arc<crate::game::layers::ComputedPermanent>)>,
    attackers: Vec<(&'a crate::card::CardInstance, Option<std::sync::Arc<crate::game::layers::ComputedPermanent>>)>,
    can: Vec<Vec<bool>>,
}

impl<'a> BlockChainSetup<'a> {
    fn new(state: &'a GameState, seat: usize) -> Option<Self> {
        let blockers = legal_blockers(state, seat);
        if blockers.is_empty() {
            return None;
        }
        let attackers: Vec<(&crate::card::CardInstance, Option<_>)> = state
            .attacking
            .iter()
            .filter(|a| state.defender_for(a.target) == Some(seat))
            .filter_map(|a| state.battlefield_find(a.attacker))
            .map(|c| (c, state.computed_permanent(c.id)))
            .collect();
        if attackers.is_empty() {
            return None;
        }
        // The legal pairs, resolved once per decision.
        let can: Vec<Vec<bool>> = blockers
            .iter()
            .map(|(b, bcp)| {
                attackers
                    .iter()
                    .map(|(a, acp)| state.blocker_can_block_attacker_pair(b, bcp, a, acp.as_deref()))
                    .collect()
            })
            .collect();
        if !can.iter().flatten().any(|&x| x) {
            return None;
        }
        Some(Self { blockers, attackers, can })
    }
}

/// The block chain (see [`EvalWeights::block_chain`]): grow a block plan
/// from the repaired "no blocks" one move at a time, keeping a move only
/// when its simulated combat scores strictly above finalizing the plan so
/// far. Returns the finished plan and its score, or `None` when nothing
/// this seat controls may block anything attacking it.
///
/// Moves per step: one (blocker, attacker) pair for every free blocker
/// and every attacker it may legally block (`blocker_can_block_attacker_pair`,
/// the engine's own gate, resolved once), plus per attacker not yet dealt
/// lethal damage a *gang move* — the cheapest free blockers, by
/// `permanent_value`, that together kill it, [`gang_block_candidates`]'
/// own arithmetic relative to the plan so far. Legality beyond the pair
/// gate is left to the sim's opening dry run, as the menu's gangs leave it
/// to the engine.
///
/// `menu` / `menu_scores` are what [`pick_blocks_scored`] already
/// simulated (index 0 greedy); the start plan's score is reused when the
/// menu holds that plan.
fn block_chain_candidate(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    menu: &[Vec<(CardId, CardId)>],
    menu_scores: &[(usize, i32)],
    setup: BlockChainSetup<'_>,
    starts: &SimStarts,
) -> Option<(Vec<(CardId, CardId)>, i32)> {
    use crate::card::Keyword;
    let BlockChainSetup { blockers, attackers, can } = setup;
    // Cheapest blockers first, for the gang move and for candidate order.
    let mut order: Vec<usize> = (0..blockers.len()).collect();
    order.sort_by_cached_key(|&i| permanent_value(state, blockers[i].0.id, w));

    let mut start = vec![Vec::new()];
    repair_block_plans(state, seat, &mut start);
    let mut current = start.swap_remove(0);
    let start_key = block_set_key(&current);
    block_census::add(6, 1);
    let mut current_score = match menu_scores
        .iter()
        .find(|(i, _)| menu.get(*i).is_some_and(|c| block_set_key(c) == start_key))
    {
        Some(&(_, s)) => {
            block_census::add(5, 1);
            s
        }
        None => simulate_block_outcome_from(starts, seat, &current, w)?,
    };
    let mut sims = 0u64;
    for _ in 0..w.block_chain {
        let free: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| !current.iter().any(|(b, _)| *b == blockers[i].0.id))
            .collect();
        if free.is_empty() {
            break;
        }
        // Candidate 0 is "finalize": the plan so far, at its known score.
        let mut cands: Vec<Vec<(CardId, CardId)>> = vec![current.clone()];
        for &i in &free {
            for (j, (a, _)) in attackers.iter().enumerate() {
                if can[i][j] {
                    let mut c = current.clone();
                    c.push((blockers[i].0.id, a.id));
                    cands.push(c);
                }
            }
        }
        for (j, (a, _)) in attackers.iter().enumerate() {
            let a_tough = a.toughness() - a.damage as i32;
            let already: i32 = current
                .iter()
                .filter(|(_, aid)| *aid == a.id)
                .filter_map(|(bid, _)| state.battlefield_find(*bid))
                .map(|b| b.power().max(0))
                .sum();
            if already >= a_tough {
                continue;
            }
            let mut gang: Vec<CardId> = Vec::new();
            let mut dmg = already;
            for &i in &free {
                if !can[i][j] {
                    continue;
                }
                gang.push(blockers[i].0.id);
                dmg += blockers[i].0.power().max(0);
                if blockers[i].1.keywords().has_kw(&Keyword::Deathtouch) || dmg >= a_tough {
                    break;
                }
            }
            if gang.len() < 2 || dmg < a_tough {
                continue;
            }
            let mut c = current.clone();
            for b in gang {
                c.push((b, a.id));
            }
            cands.push(c);
        }
        repair_block_plans(state, seat, &mut cands);
        if cands.len() < 2 {
            break;
        }
        let mut scored: Vec<(usize, i32)> = vec![(0, current_score)];
        for (i, c) in cands.iter().enumerate().skip(1) {
            sims += 1;
            if let Some(s) = simulate_block_outcome_from(starts, seat, c, w) {
                scored.push((i, s));
            }
        }
        let chosen = choose_scored(state.turn_number, &scored).unwrap_or(0);
        if chosen == 0 {
            break;
        }
        current_score = scored.iter().find(|(i, _)| *i == chosen).map(|&(_, s)| s).unwrap_or(current_score);
        current = cands.swap_remove(chosen);
    }
    block_census::add(2, sims);
    Some((current, current_score))
}

fn pick_blocks_scored(state: &GameState, seat: usize, w: &EvalWeights) -> Vec<(CardId, CardId)> {
    // Same saturation fallback as the attack picker: a flat net can't
    // rank block plans either, and the tie falls to the greedy menu.
    let w = &tail_guarded(state, seat, w);
    let mut candidates = block_candidates_for_mcts(state, seat, w);
    // A one-candidate menu is bare "no blocks" (or greedy with the search
    // off); the block chain prices it rather than returning it — when it
    // can run at all. With nothing to add, the one candidate is the
    // answer and its sim would only feed an argmax of one.
    let setup = if w.block_chain > 0 { BlockChainSetup::new(state, seat) } else { None };
    if candidates.len() == 1 && setup.is_none() {
        return candidates.swap_remove(0);
    }

    let starts = SimStarts::new(state, seat, w);
    let mut scored: Vec<(usize, i32)> = Vec::new();
    for (i, cand) in candidates.iter().enumerate() {
        let Some(score) = simulate_block_outcome_from(&starts, seat, cand, w) else { continue };
        scored.push((i, score));
    }
    // The chain's finished plan is one more candidate in the same argmax
    // (see `EvalWeights::block_chain`), appended so greedy keeps index 0
    // and every tie, and skipped when the menu already holds that plan.
    let menu_len = candidates.len();
    let mut chain_novel = false;
    if let Some(setup) = setup
        && let Some((chain, score)) =
            block_chain_candidate(state, seat, w, &candidates, &scored, setup, &starts)
        && !candidates.iter().any(|c| block_set_key(c) == block_set_key(&chain))
    {
        candidates.push(chain);
        scored.push((menu_len, score));
        chain_novel = true;
    }
    let chosen = choose_scored(state.turn_number, &scored).unwrap_or(0);
    if block_census::on() {
        block_census::tick(menu_len, chosen, chain_novel);
    }
    candidates.swap_remove(chosen)
}

/// Desperation chumps: candidates the profitable-blocks-only greedy
/// pass will never emit, generated only when the unblocked attackers
/// pointed at this seat could kill it within two swings. The cheapest
/// idle body chumps the biggest unblocked attacker; with two idle
/// bodies a two-chump variant covers the two biggest. The simulations
/// price whether the turn bought beats the cards lost — this fn only
/// puts the option on the menu.
fn chump_block_candidates(
    state: &GameState,
    seat: usize,
    greedy: &[(CardId, CardId)],
    w: &EvalWeights,
) -> Vec<Vec<(CardId, CardId)>> {
    let blocked: crate::fxhash::HashSet<CardId> = greedy.iter().map(|(_, a)| *a).collect();
    let used: crate::fxhash::HashSet<CardId> = greedy.iter().map(|(b, _)| *b).collect();
    let mut incoming: Vec<(CardId, i32)> = state
        .attacking
        .iter()
        .filter(|a| state.defender_for(a.target) == Some(seat) && !blocked.contains(&a.attacker))
        .filter_map(|a| {
            state.computed_permanent(a.attacker).map(|cp| (a.attacker, cp.power.max(0)))
        })
        .collect();
    let total: i32 = incoming.iter().map(|(_, p)| p).sum();
    // Two clean swings from dead is where a chump starts buying the
    // turn that matters; above that, the card is worth more.
    if total <= 0 || total * 2 < state.effective_life(seat) {
        return Vec::new();
    }
    incoming.sort_by_key(|&(_, p)| std::cmp::Reverse(p));
    let mut idle: Vec<CardId> = legal_blockers(state, seat)
        .into_iter()
        .filter(|(c, _)| !used.contains(&c.id))
        .map(|(c, _)| c.id)
        .collect();
    idle.sort_by_cached_key(|&id| permanent_value(state, id, w));
    let mut out = Vec::new();
    if let (Some(&blocker), Some(&(atk, _))) = (idle.first(), incoming.first()) {
        let mut cand = greedy.to_vec();
        cand.push((blocker, atk));
        out.push(cand);
    }
    if idle.len() >= 2 && incoming.len() >= 2 {
        let mut cand = greedy.to_vec();
        cand.push((idle[0], incoming[0].0));
        cand.push((idle[1], incoming[1].0));
        out.push(cand);
    }
    out
}

/// Block assignments that add a gang onto an attacker the greedy pass
/// left alone, one candidate per attacker worth ganging.
///
/// Only attackers nobody is already blocking are considered: piling onto
/// an existing block changes a trade the greedy pass already reasoned
/// about, while an unblocked attacker is one it decided it *couldn't*
/// profitably block alone — exactly the case a gang exists for. Blockers
/// are taken cheapest-first so the gang spends the least material that
/// still kills, and a candidate is only emitted when the gang actually
/// kills (an assignment that merely chumps harder is strictly worse than
/// the greedy one and would only waste a simulation).
///
/// Illegal declarations (menace needing two, a "must be blocked"
/// attacker left uncovered) are not filtered here: the engine rejects
/// them and [`simulate_block_outcome_from`] returns `None`, which drops the
/// candidate. Legality is the engine's job, not this heuristic's.
fn gang_block_candidates(
    state: &GameState,
    seat: usize,
    greedy: &[(CardId, CardId)],
    w: &EvalWeights,
) -> Vec<Vec<(CardId, CardId)>> {
    use crate::card::Keyword;
    const MAX_CANDIDATES: usize = 3;

    let blocked: crate::fxhash::HashSet<CardId> =
        greedy.iter().map(|(_, a)| *a).collect();
    let used: crate::fxhash::HashSet<CardId> = greedy.iter().map(|(b, _)| *b).collect();

    // Idle bodies, cheapest first: the gang should cost as little as it
    // can and still kill.
    let mut idle: Vec<(&crate::card::CardInstance, _)> = legal_blockers(state, seat)
        .into_iter()
        .filter(|(c, _)| !used.contains(&c.id))
        .collect();
    idle.sort_by_cached_key(|(c, _)| permanent_value(state, c.id, w));
    if idle.len() < 2 {
        return Vec::new();
    }

    // Unblocked attackers, most valuable first — the gang is only worth
    // its losses against a real threat.
    let mut targets: Vec<&crate::card::CardInstance> = state
        .attacking
        .iter()
        .filter(|a| !blocked.contains(&a.attacker))
        .filter_map(|a| state.battlefield_find(a.attacker))
        .filter(|c| c.controller != seat)
        .collect();
    targets.sort_by_cached_key(|c| -permanent_value(state, c.id, w));

    let mut out = Vec::new();
    for atk in targets.into_iter().take(MAX_CANDIDATES) {
        // CR 613 — the computed set on both sides, as `declare_blockers`
        // reads it: a granted Flying is invisible to the printed list, and so
        // is the granted Reach that answers it. See `evasion_bars_block`.
        let atk_cp = state.computed_permanent(atk.id);
        let a_flying = match &atk_cp {
            Some(cp) => cp.keywords().has_kw(&Keyword::Flying),
            None => atk.has_keyword(&Keyword::Flying),
        };
        let a_tough = atk.toughness() - atk.damage as i32;
        let mut gang: Vec<CardId> = Vec::new();
        let mut dmg = 0i32;
        for (b, bcp) in &idle {
            if evasion_bars_block(
                a_flying,
                bcp.keywords().has_kw(&Keyword::Flying),
                bcp.keywords().has_kw(&Keyword::Reach),
            ) {
                continue;
            }
            gang.push(b.id);
            dmg += b.power().max(0);
            if bcp.keywords().has_kw(&Keyword::Deathtouch) || dmg >= a_tough {
                break;
            }
        }
        // A single blocker is the greedy pass's own decision, already
        // scored; two or more that kill is the new option.
        if gang.len() < 2 || dmg < a_tough {
            continue;
        }
        let mut cand = greedy.to_vec();
        for b in gang {
            cand.push((b, atk.id));
        }
        out.push(cand);
    }
    out
}

/// Declare `blocks`, run combat damage, and score the board for `seat`.
///
/// `None` on a rejected declaration (a must-block creature we tried to hold
/// back, an over-cap batch) or a combat that won't settle — an unfinished
/// combat is scored not at all rather than scored wrong.
/// Redeal everything `seat` cannot legitimately see: each opponent's hand
/// goes back into their library, every library is shuffled, and the
/// opponent redraws the same number of cards.
///
/// This is what turns the combat sims from perfect-information search
/// into search under uncertainty — see
/// [`determinize`](EvalWeights::determinize) for why that matters.
/// `pub(crate)` for the MCTS rollouts, which share the same obligation.
///
/// Two honest approximations, both in the direction of forgetting more
/// than a real player would:
///
/// * Cards the seat has legitimately *seen* (a Duress reveal, a card
///   played and bounced) are re-hidden. Modelling that properly needs a
///   per-seat knowledge log the engine does not keep.
/// * Face-down permanents keep their real identity. They are already on
///   the battlefield and the sim reads them there.
///
/// Zones are permuted directly rather than through the engine's move
/// paths deliberately: this is a redeal of hidden information before the
/// simulation starts, not a game action, and routing it through
/// `move_card` would fire zone-change triggers that never happened.
/// The premise `determinize_hidden`'s canonicalising sort rests on: `CardId`s
/// come off a global monotonic counter, so sorting a hidden zone by id is a
/// *total* order with no ties. A duplicate would leave the order between the
/// duplicates unspecified, and an unspecified order inside the
/// canonicalisation is exactly the hidden-information leak the sort exists to
/// prevent — the redeal would depend on how the zone happened to arrive.
///
/// **`#[cfg(debug_assertions)]`, so this function is not in a release binary
/// at all.** That is not tidiness: the first version of this change put the
/// sort itself behind a named helper, and adding one function to `bot.rs`
/// moved `compute_permanent_pass` and a `FilterMap` by +3.6 M Ir through
/// inlining alone — **+0.18 % of the actor, three times what the change
/// saved** (PERF `(-123)`). A release-side helper next to four hot call sites
/// is not free here.
#[cfg(debug_assertions)]
fn assert_canonical_by_id(cards: &[crate::card::CardInstance]) {
    assert!(
        cards.windows(2).all(|w| w[0].id.0 < w[1].id.0),
        "determinize canonicalisation needs unique CardIds — a tie makes the redeal \
         depend on the order the hidden zone arrived in",
    );
}

pub(crate) fn determinize_hidden(g: &mut GameState, seat: usize, salt: u64) {
    use rand::seq::SliceRandom;
    let mut rng = StdRng::seed_from_u64(
        salt ^ ((g.turn_number as u64) << 32) ^ ((seat as u64) << 16) ^ g.step as u64,
    );
    for p in 0..g.players.len() {
        if p == seat {
            // Our own library order is unknown to us too — a search that
            // plans around the card it is about to draw is cheating just
            // as much as one that reads the opponent's hand.
            g.players[p].library.sort_unstable_by_key(|c| c.id.0);
            #[cfg(debug_assertions)]
            assert_canonical_by_id(&g.players[p].library);
            g.players[p].library.shuffle(&mut rng);
            continue;
        }
        let n = g.players[p].hand.len();
        let returned: Vec<_> = g.players[p].hand.drain(..).collect();
        g.players[p].library.extend(returned);
        // Canonicalise before shuffling, and this is the whole point
        // rather than tidiness. `shuffle` permutes the vector it is
        // given, so its output depends on the order that vector arrived
        // in — which is hidden information. Without the sort, redealing
        // a position whose hidden zones were arranged differently
        // produces a *different* guess, so "the search does not read
        // hidden state" could not even be stated as an invariant, let
        // alone tested. Sorting by card id makes the redeal a function
        // of the information set: which cards are unseen, and how many
        // of them are in hand (a public number), and nothing else.
        g.players[p].library.sort_unstable_by_key(|c| c.id.0);
            #[cfg(debug_assertions)]
            assert_canonical_by_id(&g.players[p].library);
        g.players[p].library.shuffle(&mut rng);
        let split = g.players[p].library.len().saturating_sub(n);
        let redrawn: Vec<_> = g.players[p].library.split_off(split);
        g.players[p].hand.extend(redrawn);
    }
}

/// [`determinize_hidden`] with the opponent's hand drawn from a learned
/// belief instead of uniformly (round 39). A hidden hand is not uniform
/// given observed play — held-back mana, declined blocks, and what was
/// *not* cast are all evidence — and the belief head predicts per-name
/// hold probabilities from exactly the observable state the encoder
/// carries, so the redeal stays a function of the information set (plus
/// the salt and the belief, which is itself observable-determined).
///
/// A separate function rather than a parameter on [`determinize_hidden`]
/// on purpose: the uniform path is exercised by every adopted profile
/// and the golden traces, and must stay byte-identical.
///
/// The sampler is Efraimidis–Spirakis weighted sampling without
/// replacement: each unseen card draws key `ln(u)/w` in canonical
/// (sorted) order and the `hand_size` largest keys become the hand.
/// Weights are hold-odds `p/(1−p)` with `p` clamped to [0.02, 0.98], so
/// no card is unreachable however confident the head gets; out-of-vocab
/// cards are neutral (p = 0.5).
pub(crate) fn determinize_hidden_belief(
    g: &mut GameState,
    seat: usize,
    salt: u64,
    belief: &[f32],
) {
    use rand::seq::SliceRandom;
    let mut rng = StdRng::seed_from_u64(
        salt ^ ((g.turn_number as u64) << 32) ^ ((seat as u64) << 16) ^ g.step as u64,
    );
    let vocab = super::net_eval::vocab();
    for p in 0..g.players.len() {
        if p == seat {
            g.players[p].library.sort_unstable_by_key(|c| c.id.0);
            #[cfg(debug_assertions)]
            assert_canonical_by_id(&g.players[p].library);
            g.players[p].library.shuffle(&mut rng);
            continue;
        }
        let n = g.players[p].hand.len();
        let returned: Vec<_> = g.players[p].hand.drain(..).collect();
        g.players[p].library.extend(returned);
        g.players[p].library.sort_unstable_by_key(|c| c.id.0);
            #[cfg(debug_assertions)]
            assert_canonical_by_id(&g.players[p].library);
        let mut keyed: Vec<(f64, usize)> = g.players[p]
            .library
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let idx = vocab.index_of(c.definition.name) as usize;
                let prob = if idx == 0 {
                    0.5
                } else {
                    belief.get(idx).copied().unwrap_or(0.5).clamp(0.02, 0.98) as f64
                };
                let w = prob / (1.0 - prob);
                let u: f64 = rng.random_range(f64::EPSILON..1.0);
                (u.ln() / w, i)
            })
            .collect();
        keyed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let hand_idx: std::collections::HashSet<usize> =
            keyed.iter().take(n).map(|&(_, i)| i).collect();
        let lib: Vec<_> = g.players[p].library.drain(..).collect();
        for (i, c) in lib.into_iter().enumerate() {
            if hand_idx.contains(&i) {
                g.players[p].hand.push(c);
            } else {
                g.players[p].library.push(c);
            }
        }
        g.players[p].library.shuffle(&mut rng);
    }
}

/// The belief a redeal should use, when the profile asks for one and the
/// slot's net can answer. `None` — flag off, empty slot, or a net with
/// no belief head — falls back to the uniform redeal, the historical
/// path. Computed from the pre-redeal state, which encodes identically
/// to the post-redeal one (the encoder is observable-only), so callers
/// may evaluate it wherever is convenient.
pub(crate) fn hand_belief(g: &GameState, seat: usize, w: &EvalWeights) -> Option<Vec<f32>> {
    if !w.belief_determinize || w.net_slot == 0 {
        return None;
    }
    super::net_eval::opp_hand_probs(g, seat, w.net_slot)
}

/// The state a simulation should start from: the real one, or a redeal of
/// its hidden zones. `k` indexes the redeal so an averaging caller gets
/// different hands each time.
fn sim_start_state(state: &GameState, seat: usize, w: &EvalWeights, k: u8) -> GameState {
    let mut g = state.clone();
    if w.determinize > 0 {
        if let Some(b) = hand_belief(&g, seat, w) {
            determinize_hidden_belief(&mut g, seat, 0x5EED_0000 ^ k as u64, &b);
        } else {
            determinize_hidden(&mut g, seat, 0x5EED_0000 ^ k as u64);
        }
    }
    g
}

/// The start states of one decision's simulations, one per redeal index
/// `k`. A redeal is a function of the pre-decision state, the seat and
/// `k` (the seed is fixed), so every candidate of one argmax starts from
/// the same board; building it here once instead of inside every sim
/// pays the redeal — the library sort, the shuffle, the CoW unshare of
/// two zones, ~10 k Ir — per decision rather than per candidate (PERF
/// `(-256)`: 10,334 redeals against ~1,300 decisions on a six-game
/// sealed run). Each sim still clones its base, as it cloned the real
/// state before.
struct SimStarts {
    bases: Vec<GameState>,
}

impl SimStarts {
    fn new(state: &GameState, seat: usize, w: &EvalWeights) -> Self {
        let n = w.determinize.max(1);
        Self { bases: (0..n).map(|k| sim_start_state(state, seat, w, k)).collect() }
    }

    fn base(&self, k: u8) -> &GameState {
        &self.bases[k as usize]
    }
}

fn simulate_block_outcome_from(
    starts: &SimStarts,
    seat: usize,
    blocks: &[(CardId, CardId)],
    w: &EvalWeights,
) -> Option<i32> {
    if w.determinize > 1 {
        // Mean over redeals: one redeal only swaps perfect information
        // for a specific wrong guess.
        let mut total = 0i64;
        let mut n = 0i64;
        for k in 0..w.determinize {
            if let Some(v) = simulate_block_outcome_once(starts.base(k), seat, blocks, w) {
                total += v as i64;
                n += 1;
            }
        }
        // Every redeal failing means the assignment is illegal, not
        // merely unlucky — propagate that as before.
        return (n > 0).then(|| (total / n) as i32);
    }
    simulate_block_outcome_once(starts.base(0), seat, blocks, w)
}

fn simulate_block_outcome_once(
    base: &GameState,
    seat: usize,
    blocks: &[(CardId, CardId)],
    w: &EvalWeights,
) -> Option<i32> {
    let mut g = base.clone();
    dry_run(&mut g, GameAction::DeclareBlockers(blocks.to_vec())).ok()?;
    let turn = g.turn_number;
    let mut fuel = 200u32;
    while !g.is_game_over() && g.turn_number == turn && g.step < TurnStep::EndCombat {
        fuel = fuel.checked_sub(1)?;
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            dry_run(&mut g, GameAction::SubmitDecision(answer)).ok()?;
            continue;
        }
        // Under `attack_sim_spells` the combat window is live: tricks and
        // responses fire for whichever seat holds priority, so a block
        // that only works until the attacker pumps is scored as such.
        let action = match w.attack_sim_spells.then(|| sim_spell_action(&g, w)).flatten() {
            // See the same arm in `simulate_attack_outcome_once`: the dry run
            // that validated this cast already produced the state it leads to.
            Some(Picked::Probed(_, next)) => {
                g = *next;
                continue;
            }
            Some(Picked::Plain(a)) => a,
            None => GameAction::PassPriority,
        };
        if !sim_step(&mut g, action) {
            return None;
        }
    }
    let v = eval_material(&g, seat, w);
    super::leaf_capture::maybe(&g, seat, v);
    Some(v)
}

fn pick_blocks(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // The heuristic probes block legality per blocker×attacker pair, each a
    // layer-aware check — share one gather across the whole scan, the cap
    // and the tax trim. A second scope would gather a second time.
    state.with_frozen_layers(|state| {
        let mut blocks = pick_blocks_inner(state, seat);
        // CR 509.1b — Silent Arbiter caps the distinct blockers for the whole
        // combat; an over-sized batch is rejected outright, so keep only the
        // first `cap` blockers (the heuristic already ordered them best-first).
        if let Some(cap) = state.combat_participation_cap(true) {
            let mut kept: Vec<CardId> = state.block_map.keys().copied().collect();
            blocks.retain(|(blocker, _)| {
                if kept.contains(blocker) {
                    return true;
                }
                if kept.len() >= cap as usize {
                    return false;
                }
                kept.push(*blocker);
                true
            });
        }
        trim_blocks_to_payable_tax(state, seat, &mut blocks);
        blocks
    })
}

/// CR 509.1d / 509.1b — drop blockers until the declaration's cost is
/// payable.
///
/// The block twin of [`trim_attacks_to_payable_tax`], and the same
/// consequence: the engine charges the tax after every legality check has
/// passed and rejects the declaration **whole** when a player cannot cover
/// it, so an unaffordable batch costs the bot its entire block step rather
/// than one blocker. Two families, both per blocker and additive —
/// `GameState::block_tax_for` (Archangel of Tithes, Heat Wave, and the
/// turn-scoped tax; charges life as well as mana) and the blocker's own
/// "can't block unless you pay {N}" keyword, which is the same
/// `attack_block_keyword_tax` the engine reads.
///
/// Keeps the blocks that stop the most damage: sorted by the attacker's
/// damage value descending, kept while the running cost fits. `available_mana`
/// is deliberately optimistic, so this drops the batches that are clearly
/// unaffordable and leaves the engine to reject the rest.
fn trim_blocks_to_payable_tax(
    state: &GameState,
    seat: usize,
    blocks: &mut Vec<(CardId, CardId)>,
) {
    if blocks.is_empty() {
        return;
    }
    let keyword_tax = |id: CardId| {
        state
            .computed_permanent(id)
            .map(|cp| state.attack_block_keyword_tax(id, cp.keywords(), false))
            .unwrap_or(0)
    };
    let taxed_board = state.block_tax_present();
    let cost = |blocker: CardId| -> (u32, u32) {
        let (mana, life) = if taxed_board { state.block_tax_for(blocker) } else { (0, 0) };
        (mana + keyword_tax(blocker), life)
    };
    // A blocker declared twice (Guardian of the Gateless) is charged once,
    // the way the engine charges it: the tax is per declared blocker.
    let mut distinct: Vec<CardId> = Vec::new();
    for (b, _) in blocks.iter() {
        if !distinct.contains(b) {
            distinct.push(*b);
        }
    }
    let mut owed = (0u32, 0u32);
    for b in &distinct {
        let (m, l) = cost(*b);
        owed = (owed.0 + m, owed.1 + l);
    }
    if owed == (0, 0) {
        return;
    }
    // **Deliberately the estimate, where the attack trim measures.** This
    // trim runs once per *candidate* in `block_candidates_for_mcts`, not once
    // per declaration, and measuring it there read +2.11 % wall on `cube`
    // (`ab_wall`, 6 blocks, CI +0.99 .. +3.24 %, against a null that resolves
    // +/-1.47 %) for a defect the census never observed: the block half was
    // already 0 rejections on every pool and seed swept. Measure the budget
    // where it is wrong, not everywhere it is used.
    let budget = available_mana(state, seat).total;
    let life = state.players[seat].life.max(0) as u32;
    if owed.0 <= budget && owed.1 <= life {
        return;
    }
    // Best damage stopped first. A blocker's value is the largest attacker it
    // is assigned to, which is the damage the block actually prevents.
    let mut order: Vec<(CardId, i32)> = distinct
        .iter()
        .map(|b| {
            let v = blocks
                .iter()
                .filter(|(bid, _)| bid == b)
                .map(|(_, a)| attacker_damage_value(state, *a))
                .max()
                .unwrap_or(0);
            (*b, v)
        })
        .collect();
    order.sort_by_key(|(_, v)| -*v);
    let (mut spent_mana, mut spent_life) = (0u32, 0u32);
    let mut keep: Vec<CardId> = Vec::new();
    for (b, _) in order {
        let (m, l) = cost(b);
        if spent_mana + m <= budget && spent_life + l <= life {
            spent_mana += m;
            spent_life += l;
            keep.push(b);
        }
    }
    blocks.retain(|(b, _)| keep.contains(b));
}

/// CR 509.1b — flying/reach evasion, off the **computed** keyword sets on
/// both sides. `true` bars the block.
///
/// A pre-filter, not the gate: `blocker_can_block_attacker_pair` is
/// authoritative and every planner pass that can afford it per pair calls it.
/// This exists for the passes that cannot, and it is **one function** because
/// four hand-written copies of it read the *printed* keyword lists — so a
/// granted Reach could not block a flier in any plan the bot made, and a
/// granted Flying was pre-filtered as blockable. The second is caught by the
/// authoritative gate one line later; the first is a legal line made
/// permanently invisible, which no rejection counter can ever see, and it is
/// what `bot_block_plan_sees_a_granted_reach` pins.
fn evasion_bars_block(attacker_flying: bool, blocker_flying: bool, blocker_reach: bool) -> bool {
    attacker_flying && !blocker_flying && !blocker_reach
}

/// The seat's legal blockers and their computed views, resolved once
/// (CR 509.1a).
///
/// A creature the bot may legally declare as a blocker is the engine's own
/// `blocker_self_block` answer, so the planner cannot offer a blocker the
/// declaration gate then rejects — and cannot decline one a CR 509.1b
/// requirement obliges either, which is the direction that deadlocks. It used
/// to be a hand-written subset read off the *printed* view; see
/// `GameState::blocker_self_block` for what each of the four copies missed.
///
/// The answer costs a `computed_permanent` per own permanent — a layer-memo
/// probe on a hit and a whole `apply_layers_one` on a miss — and the planner
/// asks it over the whole battlefield five times per declaration, plus once
/// more from each candidate helper. At `be4a9987` that was **117,028 calls /
/// 65,311,295 Ir / 1.83 % of `cube`** and 2.03 % of `fixed`, for an answer
/// that cannot change inside one declaration: nothing in the planner mutates
/// `state`.
///
/// **The computed view comes back with it**, because it had to be built
/// anyway and every caller then reads a keyword off it. Reading the *printed*
/// list instead is the drift `evasion_bars_block` names: it cost a granted
/// Reach every block it could have made.
///
/// Returns the instances, not ids: the five passes below then iterate the
/// handful of legal blockers directly instead of re-walking the whole
/// battlefield and re-finding each one.
fn legal_blockers(
    state: &GameState,
    seat: usize,
) -> Vec<(&crate::card::CardInstance, std::sync::Arc<crate::game::layers::ComputedPermanent>)> {
    // A `filter().filter_map().collect()` is two `&mut F::call_mut` forwards
    // per battlefield permanent, and the second closure captures `state` — the
    // fat-capture shape PERF's `call_mut` census says to fix. The loop is the
    // same walk with no forwards; `Vec::from_iter` off a `filter_map` starts
    // empty too, so the allocation behaviour is unchanged.
    let mut out = Vec::new();
    for c in &state.battlefield {
        if c.controller != seat {
            continue;
        }
        let Some(cp) = state.computed_permanent_on(c) else { continue };
        if state.blocker_can_block_anything(c, &cp) {
            out.push((c, cp));
        }
    }
    out
}

/// Everything the block planner wants to know about one declared attacker.
///
/// Each field is a property of the *attacker alone*, and the planner's inner
/// loop runs once per (blocker x attacker) pair — so every one of these was a
/// whole-battlefield `find` plus a keyword walk per pair before it was a
/// field. Built once per attacker in [`pick_blocks_inner`].
struct AttackerFacts<'a> {
    id: CardId,
    /// The attacker itself and its computed view, resolved once so the pair
    /// check doesn't re-resolve them per candidate blocker.
    card: &'a crate::card::CardInstance,
    cp: Option<std::sync::Arc<crate::game::layers::ComputedPermanent>>,
    target: AttackTarget,
    /// [`attacker_damage_value`] — combat damage this attacker assigns.
    power: i32,
    toughness: i32,
    flying: bool,
    deathtouch: bool,
    /// First or double strike: it damages the blocker before the blocker
    /// strikes back (CR 702.7).
    first_strike: bool,
    trample: bool,
    indestructible: bool,
    /// CR 509.1c — "must be blocked if able".
    must_be_blocked: bool,
    /// CR 702.23 — Rampage N, the per-extra-blocker pump.
    rampage: i32,
    /// CR 509.1b — Menace / `CantBeBlockedExceptByN`, else 1.
    min_blockers: usize,
    /// Poison this attacker would add on damage: Infect (CR 702.90) deals its
    /// power as poison, Toxic/Poisonous N add N.
    poison: u32,
}

/// One candidate blocker in [`pick_blocks_inner`]'s greedy pass: the card and
/// the view `legal_blockers` built for it, plus the printed P/T and the three
/// computed evasion keywords the trade math reads per attacker.
struct BlockerFacts<'a> {
    card: &'a crate::card::CardInstance,
    view: &'a std::sync::Arc<crate::game::layers::ComputedPermanent>,
    power: i32,
    toughness: i32,
    flying: bool,
    reach: bool,
    deathtouch: bool,
}

fn pick_blocks_inner(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // Improved blocker heuristic (push claude/modern_decks):
    //   1. Build the candidate set of (attacker, attacker_power,
    //      attacker_toughness, has_flying) attacking us.
    //   2. Sort blockers by ascending power so cheap chumps get
    //      assigned first; bigger blockers stay free for must-block
    //      situations.
    //   3. For each blocker, pick the **best** attacker it can block:
    //      - Prefer attackers it can kill outright (blocker_power >=
    //        attacker_toughness, with deathtouch granting kill on any
    //        damage).
    //      - Among kill-able attackers, prefer one that won't kill the
    //        blocker (blocker_toughness > attacker_power); ties broken
    //        by highest attacker_power (biggest value trade).
    //      - If no clean kill exists, fall back to a chump-block to
    //        save us from lethal damage when our life total is low
    //        (< current incoming damage).
    //   4. Each attacker can be assigned multiple blockers if a single
    //      blocker can't kill it — the loop falls through to try the
    //      next blocker.
    use crate::card::Keyword;
    // Deathtouch makes the attacker lethal to any blocker it damages
    // regardless of power, so the bot must treat a block against it as a
    // likely loss of the blocker when scoring trades.
    let attacker_info: Vec<AttackerFacts> = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| {
            // The `find` first, then the `_on` form off its result: the layer
            // view was asked for by id and then the same card was found
            // anyway, so the miss paid two whole-battlefield walks for one
            // permanent.
            let a = state.battlefield.find_by_id(atk.attacker)?;
            let cp = state.computed_permanent_on(a);
            Some(AttackerFacts {
                id: atk.attacker,
                card: a,
                cp: cp.clone(),
                target: atk.target,
                power: attacker_damage_value_on(state, a),
                toughness: a.toughness(),
                flying: a.has_keyword(&Keyword::Flying),
                deathtouch: a.has_keyword(&Keyword::Deathtouch),
                first_strike: a.has_keyword(&Keyword::FirstStrike)
                    || a.has_keyword(&Keyword::DoubleStrike),
                trample: a.has_keyword(&Keyword::Trample),
                indestructible: a.is_indestructible(),
                // CR 509.1c — the *computed* set, for the same reason as
                // `min_blockers` below: `declare_blockers` reads the computed
                // keyword and a granted `MustBeBlocked` (Nemesis Mask and the
                // Lure family are auras and equipment, so the keyword is
                // almost always a grant) is invisible to the instance walk.
                must_be_blocked: match &cp {
                    Some(c) => c.keywords().has_kw(&Keyword::MustBeBlocked),
                    None => a.has_keyword(&Keyword::MustBeBlocked),
                },
                rampage: a
                    .definition
                    .keywords
                    .iter()
                    .chain(a.granted_keywords_eot.iter())
                    .filter_map(|k| match k {
                        Keyword::Rampage(n) => Some(*n as i32),
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0),
                // CR 509.1b — the *computed* set, not the printed one: the
                // engine's menace check reads it, and a granted Menace (a
                // layer-6 `AddKeyword`, a keyword counter, CR 701.60 Suspect)
                // is invisible to the instance walk. An under-filled
                // multi-block is rejected as a whole batch.
                min_blockers: match &cp {
                    Some(c) => min_blockers_required_kws(c.keywords()),
                    None => min_blockers_required(a),
                },
                poison: {
                    let mut p = 0u32;
                    if a.has_keyword(&Keyword::Infect) {
                        p += a.power().max(0) as u32;
                    }
                    p += a
                        .definition
                        .keywords
                        .iter()
                        .filter_map(|k| match k {
                            Keyword::Toxic(n) | Keyword::Poisonous(n) => Some(*n),
                            _ => None,
                        })
                        .sum::<u32>();
                    p
                },
            })
        })
        .collect();
    // Only attackers aimed at the *player* threaten our life total — damage
    // to a planeswalker we control hits its loyalty, not our face. Summing
    // every attacker here would over-state the life threat and trigger
    // needless chump-blocks. Read off `attacker_info`: an attacker aimed at
    // us is always in it (`defender_for(Player(seat))` is `Some(seat)`), and
    // one that has left the battlefield scores 0 either way.
    let total_incoming: i32 = attacker_info
        .iter()
        .filter(|a| a.target == AttackTarget::Player(seat))
        .map(|a| a.power)
        .sum();
    // Planeswalker defense (CR 306.7): for each planeswalker we control that
    // is being attacked, if the attackers aimed at it would deal lethal
    // (total power ≥ its loyalty), mark those attackers so the chump-block
    // pass will trade idle blockers to save the walker.
    let defend_attackers: crate::game::types::SmallIdSet<CardId> = {
        use crate::card::CounterType;
        let mut pw_attackers: crate::game::types::SmallIdMap<CardId, (u32, Vec<CardId>)> =
            Default::default();
        for atk in state.attacking() {
            if let AttackTarget::Planeswalker(pw) = atk.target
                && state.battlefield_find(pw).map(|c| c.controller) == Some(seat)
                && let Some(a) = state.battlefield.find_by_id(atk.attacker)
            {
                let e = pw_attackers.entry_or_default(pw);
                e.0 += a.power().max(0) as u32;
                e.1.push(atk.attacker);
            }
        }
        let mut set: crate::game::types::SmallIdSet<CardId> = Default::default();
        for (pw, (incoming, atkrs)) in pw_attackers {
            let loyalty = state
                .battlefield_find(pw)
                .and_then(|c| c.counters.iter().find_map(|(k, v)| {
                    matches!(k, CounterType::Loyalty).then_some(*v)
                }))
                .unwrap_or(0);
            if loyalty > 0 && incoming >= loyalty {
                set.extend(atkrs);
            }
        }
        set
    };
    // Infect (CR 702.90) / Toxic (CR 702.180) make poison the lethal clock,
    // not life: a player with 10+ poison counters loses (CR 104.3d). The bot
    // must chump an infect/toxic attacker to avoid a poison-out even at a
    // healthy life total. Infect deals its power as poison; Toxic N adds N on
    // top of normal combat damage.
    let incoming_poison: u32 = attacker_info.iter().map(|a| a.poison).sum();
    let poison_threatened =
        incoming_poison > 0 && state.players[seat].poison_counters + incoming_poison >= 10;
    let life_threatened = state.players[seat].life <= total_incoming || poison_threatened;

    // CR 509.1a — one resolution for the five passes below. `can_block()`
    // only checks creature-ness + untapped; `legal_blockers` also excludes
    // creatures that genuinely can't block (Decayed CR 702.147, or a granted
    // "can't block") so the bot never submits an illegal block — and it costs
    // a `computed_permanent` apiece, which is why it is asked once and why the
    // view comes back with the card.
    let may_block = legal_blockers(state, seat);
    // CR 613 — flying / reach / deathtouch off the **computed** set, which is
    // what `declare_blockers` and the combat damage step read. The computed
    // view is the one `legal_blockers` already had to build, so this is free;
    // reading the printed list here is what made a granted Reach unable to
    // block a flier in any plan the bot made. See `evasion_bars_block`.
    // The card and its view ride along: the loop below used to re-find the
    // card by id and re-ask the memo for the view `legal_blockers` had just
    // handed back, then re-run `blocker_can_block_anything` on the pair —
    // 21,408 memo hits a six-game `cube` run at `0e9bdaa4` for answers it
    // already held.
    let mut blockers: Vec<BlockerFacts> = may_block
        .iter()
        .map(|(c, cp)| BlockerFacts {
            card: c,
            view: cp,
            power: c.power(),
            toughness: c.toughness(),
            flying: cp.keywords().has_kw(&Keyword::Flying),
            reach: cp.keywords().has_kw(&Keyword::Reach),
            deathtouch: cp.keywords().has_kw(&Keyword::Deathtouch),
        })
        .collect();
    blockers.sort_by_key(|b| b.power);

    // Track which attackers have already been damage-saturated by
    // assigned blockers — if blocker total toughness >= attacker
    // power, additional blockers on the same attacker are wasteful
    // unless they bring deathtouch / first strike.
    let mut attacker_damage_taken: crate::game::types::SmallIdMap<CardId, i32> =
        Default::default();
    // Blockers already committed to each attacker — folds Rampage (CR 702.23)
    // into the trade math for the second-and-later blocker.
    let mut attacker_block_count: crate::game::types::SmallIdMap<CardId, i32> =
        Default::default();
    let mut assignments: Vec<(CardId, CardId)> = Vec::new();

    for BlockerFacts {
        card: blk_card,
        view: blk_view,
        power: b_pow,
        toughness: b_tough,
        flying: b_flying,
        reach: b_reach,
        deathtouch: b_dt,
    } in blockers
    {
        let b_id = blk_card.id;
        let blk_view: &crate::game::layers::ComputedPermanent = blk_view;
        // Pick the best attacker for this blocker.
        let mut best: Option<(CardId, i32, bool)> = None; // (attacker, score, was_kill)
        // Blocker-side facts, read once instead of once per attacker: the
        // trade math below asks both on every pair. The attacker-independent
        // half of block legality (CR 509.1a/b) is `legal_blockers`' filter,
        // already applied to every entry here.
        let blk_first_strike = blk_card.has_keyword(&Keyword::FirstStrike)
            || blk_card.has_keyword(&Keyword::DoubleStrike);
        let blocker_indestructible = blk_card.is_indestructible();
        for a in &attacker_info {
            let (a_id, a_pow, a_tough, a_dt) = (&a.id, &a.power, &a.toughness, &a.deathtouch);
            if evasion_bars_block(a.flying, b_flying, b_reach) {
                continue;
            }
            // Authoritative legality gate (CR 509.1b): also honors
            // "can't be blocked except by …" / "… by …" restrictions,
            // protection, shadow, etc. Skip attackers this blocker can't
            // legally be assigned to, so the bot never submits a block batch
            // the engine will reject.
            if !state.blocker_can_block_attacker_pair(blk_card, blk_view, a.card, a.cp.as_deref())
            {
                continue;
            }
            // Skip attackers that already have at least their toughness
            // worth of damage queued unless we have deathtouch.
            let queued = attacker_damage_taken.get(a_id).copied().unwrap_or(0);
            // Rampage N (CR 702.23): every blocker beyond the first pumps the
            // attacker +N/+N. When this would be an additional blocker, fold
            // that bonus into the effective P/T so the bot doesn't gang-block
            // into a pump that saves the attacker and kills the extra blocker.
            let bcount = attacker_block_count.get(a_id).copied().unwrap_or(0);
            let ramp_bonus = a.rampage * bcount;
            let eff_a_tough = *a_tough + ramp_bonus;
            let eff_a_pow = *a_pow + ramp_bonus;
            if !b_dt && queued >= eff_a_tough {
                continue;
            }
            // First-strike awareness (CR 702.7): if the attacker strikes
            // first (and the blocker doesn't strike back first) and its
            // first-strike damage is already lethal to the blocker, the
            // blocker dies *before* dealing any damage — so it never trades
            // up. Such a "kill" is illusory; downgrade it to a chump.
            let atk_first_strike = a.first_strike;
            // CR 702.16e — protection prevents combat damage either way:
            // a blocker protected from the attacker's color takes none (won't
            // die), and an attacker protected from the blocker's color takes
            // none (won't be killed). Factor both into the trade math.
            // Both views are in hand, so the `_views` form: the by-id form
            // re-asks the scope memo for each, twice a pair.
            let blocker_takes_no_dmg =
                state.protection_prevents_views(*a_id, a.cp.as_deref(), blk_view);
            let attacker_takes_no_dmg = match a.cp.as_deref() {
                Some(acp) => state.protection_prevents_views(b_id, Some(blk_view), acp),
                None => state.damage_prevented_by_protection(b_id, *a_id),
            };
            // CR 702.12 — an indestructible permanent isn't destroyed by lethal
            // damage (or deathtouch), so it never dies in a trade and can't be
            // killed by a blocker. Block freely behind an indestructible body.
            let attacker_indestructible = a.indestructible;
            let dies_before_striking = atk_first_strike
                && !blk_first_strike
                && !blocker_takes_no_dmg
                && !blocker_indestructible
                && (eff_a_pow >= b_tough || (*a_dt && eff_a_pow >= 1));
            let kills_attacker = !attacker_takes_no_dmg
                && !attacker_indestructible
                && !dies_before_striking
                && (b_dt || b_pow >= (eff_a_tough - queued));
            // A deathtouch attacker kills the blocker on any damage.
            let dies_to_attacker = !blocker_takes_no_dmg
                && !blocker_indestructible
                && (eff_a_pow >= b_tough || (*a_dt && eff_a_pow >= 1));
            // Scoring: clean trade (kill, don't die) > kill-and-die >
            // chump (don't kill, die). Higher attacker power adds value.
            let score = if kills_attacker && !dies_to_attacker {
                1000 + *a_pow
            } else if kills_attacker && dies_to_attacker {
                // Even trade (both die). Prefer trading up: score by the
                // stat delta (proxy = power + toughness). Don't sacrifice a
                // much bigger creature for a small attacker unless we're
                // under pressure — keep the body and take the hit.
                let delta = (*a_pow + *a_tough) - (b_pow + b_tough);
                if !life_threatened && delta < -2 {
                    continue;
                }
                500 + delta
            } else if blocker_indestructible && !dies_to_attacker && *a_pow >= 1 {
                // An indestructible wall absorbs the attacker's damage at no
                // cost (it survives and isn't tapped). Free value even with no
                // life pressure — block the biggest attacker it can.
                200 + *a_pow
            } else if life_threatened || defend_attackers.contains(a_id) {
                // Chump-block to stop lethal damage (or to save a doomed
                // planeswalker). A trampler tramples over a chump
                // (CR 702.19e), so a lone chump only stops `blocker_toughness`
                // of its damage — score by the actual damage saved so the bot
                // prefers fully blocking a non-trampler over partially
                // blocking a trampler.
                let saved = if a.trample { b_tough.min(*a_pow) } else { *a_pow };
                100 + saved
            } else {
                continue;
            };
            if best.map(|(_, s, _)| s < score).unwrap_or(true) {
                best = Some((*a_id, score, kills_attacker));
            }
        }
        if let Some((a_id, _score, _kill)) = best {
            assignments.push((b_id, a_id));
            // Mark the damage queued so subsequent blockers can pile on
            // attackers that aren't fully covered yet.
            *attacker_damage_taken.entry_or_default(a_id) += b_tough;
            *attacker_block_count.entry_or_default(a_id) += 1;
        }
    }
    // Gang-block-to-kill when our life is threatened. The greedy single-
    // blocker pass above only starts blocking an attacker when one blocker
    // alone can kill it (or we chump). When we're facing lethal, trading
    // several spare creatures to *remove* a big attacker permanently beats
    // scattering chumps that die for nothing. For each still-unblocked
    // attacker (largest power first), pile idle blockers on until their
    // combined power reaches the attacker's toughness, then commit only if
    // the gang actually kills it.
    if life_threatened {
        let mut used: crate::game::types::IdSet<CardId> =
            assignments.iter().map(|(b, _)| *b).collect();
        // Same computed reads as the main pass's `blockers`, same reason.
        let mut idle: Vec<(CardId, i32, i32, bool, bool, bool)> = may_block
            .iter()
            .filter(|(c, _)| !used.contains(&c.id))
            .map(|(c, cp)| {
                (
                    c.id,
                    c.power(),
                    c.toughness(),
                    cp.keywords().has_kw(&Keyword::Flying),
                    cp.keywords().has_kw(&Keyword::Reach),
                    cp.keywords().has_kw(&Keyword::Deathtouch),
                )
            })
            .collect();
        let mut uncovered: Vec<&AttackerFacts> = attacker_info
            .iter()
            .filter(|a| !assignments.iter().any(|(_, aid)| *aid == a.id))
            .collect();
        uncovered.sort_by_key(|a| -a.power);
        for atk in uncovered {
            let (a_id, a_tough, a_flying) = (atk.id, atk.toughness, atk.flying);
            // Rampage N (CR 702.23): each blocker beyond the first raises the
            // attacker's toughness by N, so a gang must out-damage the pumped
            // total — otherwise the chumps die and the attacker survives.
            let rampage = atk.rampage;
            // Collect a gang of legal idle blockers that together kill it.
            let mut gang: Vec<CardId> = Vec::new();
            let mut dmg = 0i32;
            let mut kills = false;
            for (b_id, b_pow, _bt, b_fly, b_reach, b_dt) in &idle {
                if evasion_bars_block(a_flying, *b_fly, *b_reach) {
                    continue;
                }
                // The same legality gate the greedy pass above uses. Without
                // it this pass assembled pairs the engine rejects — and it
                // rejects the whole *batch*, so one illegal gang member cost
                // every block the planner had made. `legal_blockers` answers
                // only the attacker-independent half; the pair half is where a
                // granted `CantBlock` and the evasion keywords are read.
                if !state.blocker_can_block_attacker(*b_id, a_id) {
                    continue;
                }
                gang.push(*b_id);
                dmg += *b_pow;
                let eff_tough = a_tough + rampage * (gang.len() as i32 - 1);
                if *b_dt || dmg >= eff_tough {
                    kills = true;
                    break;
                }
            }
            if kills {
                for b_id in &gang {
                    assignments.push((*b_id, a_id));
                    used.insert(*b_id);
                }
                idle.retain(|(id, ..)| !gang.contains(id));
            }
        }
    }

    // CR 509.1c — satisfy "must be blocked if able" (Academic Dispute /
    // Lure). The engine rejects a declaration that leaves such an attacker
    // unblocked while an idle able blocker exists, so the bot must assign
    // one or it would deadlock the combat step. Pull any unused creature
    // that can legally block (respecting flying/reach) onto each
    // must-be-blocked attacker still missing a blocker.
    for atk in &attacker_info {
        let a_id = &atk.id;
        if !atk.must_be_blocked
            || !state.block_requirement_binds(*a_id)
            || assignments.iter().any(|(_, aid)| aid == a_id)
        {
            continue;
        }
        // Pick the cheapest (lowest-power) legal idle blocker so a forced block
        // doesn't throw away the bot's best body.
        if let Some((idle, _)) = may_block
            .iter()
            .filter(|(c, _)| {
                !assignments.iter().any(|(bid, _)| *bid == c.id)
                    // No flying/reach pre-filter: `blocker_can_block_attacker`
                    // below answers it off the computed set, and a second
                    // hand-written copy off the printed one is exactly the
                    // drift `evasion_bars_block` exists to stop.
                    //
                    // Both gates, as in `enforce_block_requirements`: one says
                    // the engine counts this body as able and so would demand
                    // it, the other says the pair is legal to declare.
                    && state.block_requirement_able(c, *a_id)
                    && state.blocker_can_block_attacker(c.id, *a_id)
            })
            .min_by_key(|(c, _)| c.power())
        {
            assignments.push((idle.id, *a_id));
        }
    }

    // CR 702.39 / CR 509.1c — the block requirements, forced in ahead of the
    // minimum-count repair and displacing whatever the scoring loops gave
    // those bodies. One function, shared with `repair_block_candidate`, so
    // the greedy plan and the search menu cannot answer differently.
    let forced = enforce_block_requirements(state, seat, &mut assignments);

    // CR 509.1b / 702.110b — Menace (≥2 blockers) and "can't be blocked
    // except by N or more creatures" (CantBeBlockedExceptByN — Pathrazer of
    // Ulamog) impose a minimum block count: an attacker so keyworded must be
    // blocked by 0 or ≥N, never 1..N-1. The greedy passes assign one at a
    // time, so an under-filled multi-block is illegal and the engine rejects
    // the whole declaration. For each such attacker, top the block up to the
    // minimum with legal idle blockers; if the minimum can't be reached,
    // drop every block on it (better unblocked than an illegal batch).
    for atk in &attacker_info {
        let (a_id, min_blockers) = (&atk.id, atk.min_blockers);
        if min_blockers <= 1 {
            continue;
        }
        let mut count = assignments.iter().filter(|(_, aid)| aid == a_id).count();
        if count == 0 || count >= min_blockers {
            continue;
        }
        while count < min_blockers {
            // Cheapest legal idle blocker first — minimise value lost to the
            // forced multi-block.
            let extra = may_block
                .iter()
                .filter(|(c, _)| {
                    // Same as above: the authoritative gate is the only
                    // flying/reach reading here.
                    !assignments.iter().any(|(bid, _)| *bid == c.id)
                        && state.blocker_can_block_attacker(c.id, *a_id)
                })
                .min_by_key(|(c, _)| c.power());
            match extra {
                Some((c, _)) => {
                    assignments.push((c.id, *a_id));
                    count += 1;
                }
                // Can't reach the minimum — drop all blocks on this attacker,
                // except the ones a CR 509.1 requirement pinned there. Those
                // are not the planner's to release: dropping one trades a
                // rejection at the count rule for a rejection at the
                // requirement, and the requirement is the harder one to see.
                None => {
                    assignments
                        .retain(|(bid, aid)| aid != a_id || forced.contains(&(*bid, *aid)));
                    break;
                }
            }
        }
    }

    // CR 509.1b — spend spare block capacity (Guardian of the Gateless and
    // friends). A blocker that can block extra attackers soaks additional
    // ones for free as long as the total damage it would take stays under its
    // toughness and no extra attacker has deathtouch.
    let extra_capacity = |id: CardId| -> usize {
        let Some(c) = state.battlefield_find(id) else { return 0 };
        if c.has_keyword(&Keyword::CanBlockAnyNumber) {
            return usize::MAX;
        }
        state.computed_permanent(id).map_or(0, |cp| {
            cp.keywords()
                .iter()
                .filter_map(|k| match k {
                    Keyword::CanBlockAdditional(n) => Some(*n as usize),
                    _ => None,
                })
                .sum()
        })
    };
    // Seed from every legal blocker with spare capacity, not just the ones the
    // scoring loop already assigned: a 0/N `CanBlockAnyNumber` wall kills
    // nothing and isn't needed against lethal, so it never gets picked up
    // there — but it can still soak the whole swing for free.
    let mut multi: Vec<CardId> = Vec::new();
    let seeds = assignments
        .iter()
        .map(|(b, _)| *b)
        .chain(may_block.iter().map(|(c, _)| c.id));
    for id in seeds {
        if !multi.contains(&id) && extra_capacity(id) > 0 {
            multi.push(id);
        }
    }
    for b_id in multi {
        let Some(b) = state.battlefield_find(b_id) else { continue };
        let (b_tough, b_flying, b_reach) = (
            b.toughness(),
            b.has_keyword(&Keyword::Flying),
            b.has_keyword(&Keyword::Reach),
        );
        let mut taken: i32 = assignments
            .iter()
            .filter(|(bid, _)| *bid == b_id)
            .filter_map(|(_, aid)| attacker_info.iter().find(|a| a.id == *aid))
            .map(|a| a.power)
            .sum();
        let mut spare = extra_capacity(b_id);
        for atk in &attacker_info {
            let a_id = &atk.id;
            if spare == 0 {
                break;
            }
            if atk.deathtouch
                || taken + atk.power >= b_tough
                || assignments.iter().any(|(bid, aid)| *bid == b_id && aid == a_id)
                || assignments.iter().any(|(_, aid)| aid == a_id)
                || (atk.flying && !b_flying && !b_reach)
                || atk.min_blockers > 1
                || !state.blocker_can_block_attacker(b_id, *a_id)
            {
                continue;
            }
            assignments.push((b_id, *a_id));
            taken += atk.power;
            spare = spare.saturating_sub(1);
        }
    }

    enforce_block_caps(state, &mut assignments, &forced);
    assignments
}

/// CR 509.1g — "can't be blocked by more than one creature" (Charging Rhino).
/// The inverse of Menace, and the half of the count rule nothing modelled:
/// `min_blockers` gives the planner a floor and there was no ceiling, so the
/// gang pass and the spare-capacity pass both piled a second body onto an
/// attacker that permits exactly one and the engine threw the batch out.
/// **48 of `cube` seed 23's 48 block rejections.**
///
/// A trim rather than a filter, because it has to run after every pass that
/// can add: the gang pass, the minimum-count top-up, and the spare-capacity
/// pass each append independently. Keeps a requirement pin if one is on the
/// attacker, else the biggest body, which is the one the value passes chose
/// first.
fn enforce_block_caps(
    state: &GameState,
    blocks: &mut Vec<(CardId, CardId)>,
    forced: &[(CardId, CardId)],
) {
    use crate::card::Keyword;
    if !state.board_keyword_in_scope(&[Keyword::CantBeBlockedByMoreThanOne]) {
        return;
    }
    let capped: Vec<CardId> = state
        .attacking()
        .iter()
        .map(|a| a.attacker)
        .filter(|a| {
            state
                .computed_permanent(*a)
                .is_some_and(|cp| cp.keywords().has_kw(&Keyword::CantBeBlockedByMoreThanOne))
        })
        .collect();
    for a_id in capped {
        // The engine counts blocks already declared too, so an attacker that
        // is already spoken for outside this batch takes nothing from it.
        if state.blocker_count_of(a_id) > 0 {
            blocks.retain(|(_, aid)| *aid != a_id);
            continue;
        }
        let keep = blocks
            .iter()
            .filter(|(_, aid)| *aid == a_id)
            .max_by_key(|(bid, aid)| {
                let pinned = forced.contains(&(*bid, *aid));
                let power = state.battlefield_find(*bid).map(|c| c.power()).unwrap_or(0);
                (pinned, power)
            })
            .map(|(bid, _)| *bid);
        blocks.retain(|(bid, aid)| *aid != a_id || Some(*bid) == keep);
    }
}

/// Minimum number of creatures legally required to block `attacker` (CR
/// 509.1b): 2 for Menace, N for `CantBeBlockedExceptByN(N)`, the max of any
/// such requirement, else 1. Reads printed + EOT-granted keywords (the same
/// set [`CardInstance::has_keyword`] consults).
fn min_blockers_required(attacker: &crate::card::CardInstance) -> usize {
    let mut min = min_blockers_required_kws(&attacker.definition.keywords);
    min = min.max(min_blockers_required_kws(&attacker.granted_keywords_eot));
    min
}

/// [`min_blockers_required`] over an already-resolved keyword set — the form
/// a caller holding the *computed* view uses. The engine's menace / "except
/// by N" check reads the computed set, so a planner that reads the printed
/// one under-fills a multi-block and the engine rejects the whole batch.
fn min_blockers_required_kws(kws: &[crate::card::Keyword]) -> usize {
    use crate::card::Keyword;
    let mut min = 1usize;
    for kw in kws {
        match kw {
            Keyword::Menace => min = min.max(2),
            Keyword::CantBeBlockedExceptByN(n) => min = min.max(*n as usize),
            _ => {}
        }
    }
    min
}


/// True if the player can pay the card's mana cost from their current
/// pool **including** static-ability cost increases (Damping Sphere's
/// post-first-spell tax, Chancellor of the Annex's first-spell tax).
///
/// The state-aware overload `can_afford_in_state` is what the bot's
/// main_phase_action uses; the simpler signature is kept for
/// existing callers that don't have a `GameState` handy.
pub fn can_afford(def: &CardDefinition, pool: &ManaPool) -> bool {
    can_afford_with_extra(&def.cost, pool, 0, 0)
}

/// What `seat` could still pay with this phase: mana already floating,
/// plus the most each *untapped* source they control could add.
///
/// The bot used to answer "can I afford this?" against the floating pool
/// alone, which only worked because it tapped every land before deciding
/// anything. That made the pool an accurate picture of its mana -- and
/// left it with none for the rest of the turn (CR 500.4 empties the pool
/// at every step boundary), so counterspells, flash creatures, instant
/// removal and combat tricks were unplayable in practice. Sizing against
/// untapped sources instead lets the engine's auto-tap pay each cast from
/// only what it needs, and whatever is left over survives into the
/// opponent's turn.
#[derive(Debug, Default, Clone, Copy)]
struct AvailableMana {
    /// Upper bound on the number of mana that could be produced.
    total: u32,
    /// Colors at least one source could produce.
    colors: crate::mana::ColorSet,
    /// Per-colour upper bound, indexed by
    /// [`crate::game::actions::color_index`] — the pool's mana of that colour
    /// plus, for every untapped countable source that could make it, that
    /// source's best single-activation amount. Deliberately over-counted (a
    /// source that makes *one* of `{W}` or `{U}` adds its amount to both, and
    /// a "{T}: add {C}{C}" / "{T}: add {G}" pair adds 2 to green), so a
    /// shortfall here is a *proof* the colour can't be covered.
    ///
    /// `colors` alone answers "is there any producer", which passes
    /// **{G}{G} against a lone Forest** — the largest class of cast attempt
    /// the engine then rejects at payment (PERF (-51): 31.9 % of payments are
    /// rolled back). This is the singleton case of Hall's condition; the
    /// multi-colour subsets stay unasked, so the estimate is still an
    /// over-approximation, just a tighter one.
    by_color: [u32; 5],
    /// Whether true colorless ({C}) is producible.
    colorless: bool,
}

/// Estimate [`AvailableMana`] for `seat`.
///
/// Deliberately **optimistic**: it ignores the assignment problem (which
/// source pays which pip), counts every color a choice-source could make,
/// and rounds dynamic amounts down to one rather than giving up. That is
/// the right bias, because this is only a pre-filter -- the authoritative
/// gate on every candidate is still `would_accept_on`, which runs the
/// engine's real auto-tap. An over-permissive estimate costs a few extra
/// dry-run probes; an under-permissive one silently makes castable spells
/// invisible to the bot, which is exactly the failure being fixed here.
fn available_mana(state: &GameState, seat: usize) -> AvailableMana {
    use crate::mana::{Color, ColorSet};
    let pool = &state.players[seat].mana_pool;
    use crate::game::actions::color_index;
    let mut out = AvailableMana {
        total: pool.total(),
        colors: ColorSet::empty(),
        by_color: [0; 5],
        colorless: pool.colorless_amount() > 0,
    };
    for c in Color::ALL {
        if pool.amount(c) > 0 {
            out.colors.insert(c);
            out.by_color[color_index(c)] += pool.amount(c);
        }
    }
    // One board-level grant scan for the whole sweep instead of one per
    // untapped permanent (see `GameState::grant_scan`).
    let scan = state.grant_scan();
    // Two facts that invalidate a per-colour budget, folded into the walk
    // below rather than paid as two more battlefield passes: a CR 609.4b
    // spend-as-any-colour permission that could reach some spell of `seat`'s,
    // and a mana-production multiplier (one source then covers two pips of
    // its colour, which `mana_ability_output` does not model). The
    // `debug_assert!` under the loop ties this copy of the variant list to
    // the two engine functions it fuses — same device as `dispatch_board_scan`.
    let mut fused_relax = state.players[seat].may_spend_any_color_this_turn
        || !state.colored_mana_becomes_this_turn.is_empty()
        || state.players[seat].command.iter().any(|c| {
            state.card_may_grant_any_color_spend(c, seat)
        });
    // An untapped source whose mana shape this estimate cannot cost — see the
    // `is_countable_mana_ability` arm in the loop.
    let mut opaque_source = false;
    for p in state.battlefield.iter() {
        if !fused_relax {
            use crate::effect::StaticEffect;
            let mine = p.controller == seat;
            fused_relax |= p.definition.static_abilities.iter().any(|sa| match sa.effect {
                StaticEffect::PlayersMaySpendManaAsAnyColor => true,
                StaticEffect::MaySpendManaAsAnyColorForNamedSpells
                | StaticEffect::MaySpendManaAsAnyColorForCreaturesWithChosenMv => {
                    mine && !p.face_down
                }
                StaticEffect::ManaProductionDoubled | StaticEffect::ManaProductionTripled => mine,
                _ => false,
            });
        }
        if p.controller != seat || p.tapped {
            continue;
        }
        // CR 602.5g/h — every ability this loop counts has a `{T}` cost
        // (`is_countable_mana_ability` requires it), so a summoning-sick
        // creature contributes nothing: `try_pay_with_auto_tap` refuses it.
        // This estimate had no sickness gate, and a summoning-sick Llanowar
        // Elves read as one mana. **Every other bias here is downward on
        // purpose** — sac costs and dynamic amounts are left out so the bot
        // does not commit to a line it can only pay by spending something —
        // and an upward one is a different animal: the attack tax's trim
        // treats `total` as a *budget*, so over-counting it does not make the
        // bot optimistic, it makes the engine reject the whole declaration.
        // One method with the engine, so the two cannot drift.
        if p.summoning_sick && state.tap_ability_summoning_sick(p, seat) {
            continue;
        }
        // Printed abilities plus anything granted to it (Cryptolith Rite
        // turning creatures into mana sources, Urza's Saga chapters), so a
        // granted mana ability doesn't read as "no mana here".
        let granted = state.granted_abilities_of(p, &scan);
        let mut best = 0u32;
        let mut mine = ColorSet::empty();
        for a in p.definition.activated_abilities.iter().chain(granted.iter().copied()) {
            if !is_countable_mana_ability(a) {
                // Auto-tap reads its table through `effect_produced_colors`,
                // which sees mana shapes this estimate does not: a filter
                // land's `{R}, {T}: add {R}{R}` (non-empty `mana_cost`), a
                // Lotus Petal (`sac_cost`), Crystalline Crawler (no `{T}` at
                // all — a counter cost, so it can even fire twice). `total`
                // and `colors` have always under-counted those on purpose —
                // the bot would rather not commit to a line it can only pay
                // by spending the source — but a *budget* that under-counts
                // becomes a rejection, so a source with one of these gets no
                // per-colour budget at all.
                opaque_source |=
                    !crate::game::actions::effect_produced_colors(&a.effect).is_empty();
                continue;
            }
            let (amount, colors, colorless) = mana_ability_output(&a.effect);
            // A dynamic amount ("add {G} for each creature you control") is
            // rounded down to one by `mana_ability_output`; the engine gets
            // however many it really is. Same reasoning as the arm above.
            opaque_source |= mana_amount_is_dynamic(&a.effect);
            best = best.max(amount);
            mine = mine.union(colors);
            out.colors = out.colors.union(colors);
            out.colorless |= colorless;
        }
        out.total += best;
        // The source's whole budget against each colour it could make — see
        // `by_color`'s doc for why over-counting here is the sound direction.
        for c in mine.iter() {
            out.by_color[color_index(c)] += best;
        }
    }
    debug_assert_eq!(
        fused_relax,
        state.spend_mana_as_any_color_possible_for(seat)
            || state.mana_production_multiplier_for(seat) > 1,
        "available_mana's fused budget scan drifted from the two walks it fuses",
    );
    // CR 609.4b — under a spend-as-any-colour permission a colour's own
    // producers stop bounding what can pay its pips, and a doubler makes one
    // source cover two of them. `total` already under-counts a doubler (a
    // pre-existing, deliberate bias); don't let the per-colour budget turn
    // that into a *new* rejection. `relax_cost_colors` is asked here with
    // `seat: None` and so misses the seat-scoped permissions (North Star,
    // Unexpected Potential, Emissary's Ploy); the fused scan does not.
    // CR 305.6 — a land-type rewrite (Dryad of the Ilysian Grove, Urborg,
    // Blood Moon) changes what a land taps for. `mana_source_table` reads it
    // through `scan_land_type_rewrites`; `granted_abilities_of` alone does
    // not, so this estimate sees a Mountain where auto-tap sees all five
    // colours. `false` from the gate is authoritative.
    let land_type = state.land_type_change_in_scope();
    crate::game::pay_census::record_budget(fused_relax, opaque_source, land_type);
    if fused_relax || opaque_source || land_type {
        // `u32::MAX`, not `total`: widening to `total` is only as good as
        // `total`, and `total` deliberately under-counts the same sources
        // that force the widening. Two Treasures and nothing else read
        // `total = 0`, so `[total; 5]` still rejected every coloured pip
        // while the engine sacrificed a Treasure and paid — the whole point
        // of the widening is that this estimate cannot bound the colour, and
        // `cmc <= total` remains the separate test it always was.
        out.by_color = [u32::MAX; 5];
    }
    out
}

/// A mana ability the bot is willing to count toward affordability: it
/// costs a tap and nothing the bot would regret.
///
/// We only need to know the mana *could* be paid, so color-choice sources
/// (dual lands, Birds of Paradise) and painland-style life costs count --
/// the engine's auto-tap will happily use them. Sources that consume a
/// real resource to fire (sacrifice, discard, exile, energy) are excluded:
/// counting them would have the bot commit to lines it can only pay for by
/// spending something it would rather keep.
fn is_countable_mana_ability(a: &ActivatedAbility) -> bool {
    a.tap_cost
        && a.mana_cost.symbols.is_empty()
        && !a.sac_cost
        && a.sac_other_filter.is_none()
        && a.bounce_other_filter.is_none()
        && a.tap_other_filter.is_none()
        && a.tap_n_filter.is_none()
        && a.exile_other_filter.is_none()
        && a.discard_cost.is_none()
        && !a.exile_self_cost
        && a.energy_cost == 0
        && a.collect_evidence_cost.is_none()
        && a.condition.is_none()
        && !a.from_graveyard
        && !a.from_hand
        && matches!(a.effect, Effect::AddMana { .. })
}

/// Whether a mana ability's amount is a runtime `Value` rather than a
/// constant — [`mana_ability_output`] reports one for those, which is a
/// *lower* bound and so cannot be spent as a per-colour budget.
fn mana_amount_is_dynamic(eff: &Effect) -> bool {
    use crate::effect::Value;
    let Effect::AddMana { pool, .. } = eff else { return false };
    let dynamic = |v: &Value| !matches!(v, Value::Const(_));
    match pool {
        ManaPayload::Colorless(v)
        | ManaPayload::OfColor(_, v)
        | ManaPayload::OfColors(_, v)
        | ManaPayload::AnyOneColor(v)
        | ManaPayload::AnyColors(v) => dynamic(v),
        ManaPayload::Colors(_) => false,
        // The rest are board-dependent palettes `mana_ability_output` answers
        // with a flat one — never a bound.
        _ => true,
    }
}

/// `(most mana produced, colors it could be, produces true colorless)` for
/// a mana ability's effect. Dynamic amounts (`{T}: add {G} equal to this
/// creature's power`) count as one -- enough to keep the source visible
/// without inventing a board state to measure it against.
fn mana_ability_output(eff: &Effect) -> (u32, crate::mana::ColorSet, bool) {
    use crate::effect::Value;
    use crate::mana::{Color, ColorSet};
    let mut colors = ColorSet::empty();
    accumulate_mana_colors(eff, &mut colors);
    let amount_of = |v: &Value| match v {
        Value::Const(n) => (*n).max(0) as u32,
        _ => 1,
    };
    let Effect::AddMana { pool, .. } = eff else { return (0, colors, false) };
    let (amount, colorless) = match pool {
        ManaPayload::Colors(cs) => (cs.len() as u32, false),
        ManaPayload::Colorless(v) => (amount_of(v), true),
        ManaPayload::OfColor(_, v) | ManaPayload::OfColors(_, v) => (amount_of(v), false),
        ManaPayload::AnyOneColor(v) | ManaPayload::AnyColors(v) => {
            for c in Color::ALL {
                colors.insert(c);
            }
            (amount_of(v), false)
        }
        // "Any color an opponent's land could produce" and friends: the
        // exact palette depends on a board read this estimate doesn't do,
        // so assume the source is live for any color.
        _ => {
            for c in Color::ALL {
                colors.insert(c);
            }
            (1, true)
        }
    };
    (amount, colors, colorless)
}

/// Whether `seat` can produce enough of each *colour* `cost` demands.
///
/// The colour pips are the one part of a cost nothing in the engine's
/// adjustment machinery moves: every activation and alternative-cost
/// adjustment in `activate_ability_inner` / `cast_flashback` is
/// `reduce_generic` / `add_generic`, and an `{X}` binding only *adds* pips.
/// So this is sound against a **printed** cost with no effective-cost
/// computation — and deliberately says nothing about the generic half, which
/// a reduction really can move.
///
/// [`AvailableMana::by_color`] is already widened to `total` wherever a
/// colour cannot be bounded (see `available_mana`), so this answers `true`
/// there.
fn colors_coverable(cost: &ManaCost, have: &AvailableMana) -> bool {
    use crate::mana::ManaSymbol;
    let mut need = [0u32; 5];
    for s in cost.symbols.iter() {
        if let ManaSymbol::Colored(c) = s {
            need[crate::game::actions::color_index(*c)] += 1;
        }
    }
    need.iter().zip(have.by_color.iter()).all(|(n, have)| n <= have)
}

/// State-aware affordability check: queries the engine for any
/// per-spell tax that would apply (Damping Sphere etc.) and folds it
/// into the cost before testing what `seat` can produce. Used by the bot to
/// avoid submitting `CastSpell` actions that the engine will reject
/// with a mana shortfall — repeated rejections are what deadlocked
/// `debug/deadlock-t8-1777411577-473115700.json` (Damping Sphere on
/// the board, bot casting its second spell of the turn).
pub fn can_afford_in_state(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
    w: &EvalWeights,
) -> bool {
    can_afford_in_state_with(state, seat, card, w, &SweepMana::new(state, seat))
}

/// One [`available_mana`] read shared across a hand sweep, paid at most once
/// and only if some card actually reaches the affordability test.
///
/// The walk is the whole battlefield and its answer is the same for every
/// card in hand, so a sweep that asks per card pays it per card. It has to
/// stay *lazy*: `pick_combat_trick` runs on every tick and usually filters
/// its hand down to nothing first, and an eager read there costs more than
/// the per-card reads it saves (measured +0.35 % Ir, PERF.md pass 40).
struct SweepMana<'a> {
    state: &'a GameState,
    seat: usize,
    cell: std::cell::OnceCell<AvailableMana>,
    /// The board's cost-static sources, shared by the same argument and paid
    /// under the same laziness — see
    /// [`CostStaticSources`](crate::game::actions::CostStaticSources).
    srcs: std::cell::OnceCell<crate::game::actions::CostStaticSources<'a>>,
}

impl<'a> SweepMana<'a> {
    fn new(state: &'a GameState, seat: usize) -> Self {
        Self {
            state,
            seat,
            cell: std::cell::OnceCell::new(),
            srcs: std::cell::OnceCell::new(),
        }
    }

    fn get(&self) -> &AvailableMana {
        self.cell.get_or_init(|| available_mana(self.state, self.seat))
    }

    fn cost_sources(&self) -> &crate::game::actions::CostStaticSources<'a> {
        self.srcs
            .get_or_init(|| crate::game::actions::CostStaticSources::gather(self.state))
    }
}

/// `can_afford_in_state` against a sweep-shared producible-mana read.
///
/// `card` is a card in `seat`'s hand: every caller walks
/// `state.players[seat].hand`, and the additional-cost read below is the
/// from-hand one. The `debug_assert!` is the audit — the suite exercises
/// every sweep this has.
fn can_afford_in_state_with(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
    w: &EvalWeights,
    have: &SweepMana<'_>,
) -> bool {
    debug_assert!(
        state.players[seat].hand.iter().any(|c| c.id == card.id),
        "can_afford_in_state_with wants a card in the seat's hand",
    );
    // Three whole-board static walks per hand card, over one board — see
    // `CostStaticSources`. The list is lazy for the same reason
    // `SweepMana::get` is: `pick_combat_trick` usually filters its hand to
    // nothing before any card reaches here.
    let srcs = have.cost_sources();
    let extra = crate::game::actions::extra_cost_for_spell_over(
        state,
        seat,
        card,
        None,
        srcs.battlefield(),
    );
    // Fold in generic cost *reductions* (Affinity, CostReduction statics,
    // graveyard-affinity) the same way the real cast path does — otherwise the
    // bot overestimates the cost of e.g. Tolarian Terror with a full graveyard
    // and never casts it. Target-dependent reductions are skipped (no target
    // chosen yet), so this stays conservative.
    let reduction = crate::game::actions::cost_reduction_for_spell_full_over(
        state, seat, card, None, false, false, srcs.all(),
    );
    // Coloured surcharges (the Leech cycle) can't ride the generic `extra`
    // channel, so they join the printed cost before relaxation. Borrowed when
    // there is no surcharge, which is every board without a Leech: the clone
    // only existed so the `extend` had somewhere to write.
    let tax = crate::game::actions::colored_spell_tax_for_spell_over(
        state,
        seat,
        card,
        srcs.battlefield(),
    );
    let printed: std::borrow::Cow<'_, crate::mana::ManaCost> = if tax.symbols.is_empty() {
        std::borrow::Cow::Borrowed(&card.definition.cost)
    } else {
        let mut p = card.definition.cost.clone();
        p.symbols.extend(tax.symbols);
        std::borrow::Cow::Owned(p)
    };
    // Mirror the payment funnel's Lattice relaxation so the bot doesn't
    // pass on a spell whose coloured pips any mana can now cover.
    let cost = state.relax_cost_colors(&printed);
    if w.legacy_pretap {
        return can_afford_with_extra(&cost, &state.players[seat].mana_pool, extra, reduction);
    }
    can_afford_from(&cost, have.get(), extra, reduction)
}

/// Could `printed` be paid from `have`? Three independent tests: enough
/// total mana for the (taxed, reduced) mana value, a producible source for
/// every coloured pip, and enough *of* each colour to cover its pips.
///
/// Hybrid pips pass if *either* half is producible and Phyrexian pips
/// always pass (life is a legal payment), matching what the real payment
/// funnel will accept. Neither is counted against a single colour's budget:
/// a hybrid can go to whichever half is free and Phyrexian to life, so
/// charging them to one colour would be an *under*-estimate, and this filter
/// is only allowed to err the other way.
fn can_afford_from(
    printed: &ManaCost,
    have: &AvailableMana,
    extra_generic: u32,
    reduction: u32,
) -> bool {
    use crate::mana::ManaSymbol;
    use std::borrow::Cow;
    // Borrowed on the common path: the clone only exists so `reduce_generic`
    // can mutate, and most costs have neither an {X} nor a reduction. This ran
    // 12,986 times over six bench games and allocated every time.
    let mut cost: Cow<'_, ManaCost> = if printed.has_x() {
        Cow::Owned(printed.with_x_value(0))
    } else {
        Cow::Borrowed(printed)
    };
    if reduction > 0 {
        cost.to_mut().reduce_generic(reduction);
    }
    if cost.cmc() + extra_generic > have.total {
        return false;
    }
    // Hall's condition on the singleton colour sets: `{G}{G}` off a lone
    // Forest has a producer for green and still cannot be paid. Counted in one
    // pass and compared against the board-derived budget, so this is five
    // adds and five compares per hand card. See `AvailableMana::by_color`.
    let mut need = [0u32; 5];
    for s in cost.symbols.iter() {
        if let ManaSymbol::Colored(c) = s {
            need[crate::game::actions::color_index(*c)] += 1;
        }
    }
    if need.iter().zip(have.by_color.iter()).any(|(n, have)| n > have) {
        return false;
    }
    cost.symbols.iter().all(|s| match s {
        ManaSymbol::Colored(c) => have.colors.contains(*c),
        ManaSymbol::Hybrid(a, b) => have.colors.contains(*a) || have.colors.contains(*b),
        // Phyrexian pips are payable with 2 life, so they never gate.
        ManaSymbol::Phyrexian(_) | ManaSymbol::PhyrexianHybrid(_, _) => true,
        ManaSymbol::Colorless(_) => have.colorless,
        _ => true,
    })
}

/// CR 702.21 — the tax `actor` would owe for aiming a spell or ability at
/// `id`: the permanent's computed Ward cost when it is hostile and
/// non-trivial, `None` when targeting it is tax-free. The engine's
/// auto-targeter already *prefers* un-warded candidates
/// (`auto_target_for_effect_avoiding_set_xc`); this helper exists for the
/// fallback case where every candidate is warded and the bot has to judge
/// whether the tax is survivable at all.
fn ward_tax(state: &GameState, id: CardId, actor: usize) -> Option<crate::card::WardCost> {
    use crate::card::Keyword;
    let c = state.battlefield_find(id)?;
    if state.same_team(c.controller, actor) {
        return None;
    }
    let cp = state.computed_permanent(id);
    let kws: &[Keyword] = match &cp {
        Some(cp) => cp.keywords(),
        None => &c.definition.keywords,
    };
    kws.iter().find_map(|k| match k {
        Keyword::Ward(w) if !crate::game::actions::ward_cost_is_trivial(w) => Some(w.clone()),
        _ => None,
    })
}

/// Whether the bot could actually pay `tax` on top of `besides` — the
/// mana the cast or activation itself is about to consume. The engine
/// auto-pays ward taxes when the trigger resolves (`try_pay_ward_cost`);
/// a payment that fails there gets the bot's spell countered, which is
/// strictly worse than never casting it, and a life payment the engine
/// *can* make is still refused here when it would spend the bot's whole
/// life total into the state-based loss. Variants with no cheap
/// payability read default to `true`: a wrong `true` costs one card and
/// shows up on the ladder, a wrong `false` makes a legal line permanently
/// invisible.
fn ward_tax_payable(
    state: &GameState,
    seat: usize,
    tax: &crate::card::WardCost,
    besides: &ManaCost,
) -> bool {
    use crate::card::WardCost;
    let mana_ok = |mc: &ManaCost| {
        let mut combined = besides.clone();
        combined.symbols.extend(mc.symbols.iter().cloned());
        can_afford_from(&combined, &available_mana(state, seat), 0, 0)
    };
    let life_ok = |n: u32| (n as i32) < state.effective_life(seat);
    let gy = &state.players[seat].graveyard;
    match tax {
        WardCost::Mana(mc) => mana_ok(mc),
        WardCost::Life(n) => life_ok(*n),
        WardCost::ManaAndLife(mc, n) => mana_ok(mc) && life_ok(*n),
        WardCost::Discard(n) | WardCost::DiscardMatching(_, n) => {
            state.players[seat].hand.len() >= *n as usize
        }
        WardCost::DiscardHand => true,
        WardCost::ExileFromGraveyard(n) | WardCost::BottomFromGraveyard(n) => {
            gy.len() >= *n as usize
        }
        WardCost::CollectEvidence(n) => {
            gy.iter().map(|c| c.definition.cost.cmc()).sum::<u32>() >= *n
        }
        WardCost::SacrificeCreature => state
            .battlefield
            .iter()
            .any(|c| c.controller == seat && c.definition.is_creature()),
        WardCost::SacrificePermanents(n) => {
            state.battlefield.iter().filter(|c| c.controller == seat).count() >= *n as usize
        }
        // Dynamic and niche shapes (source-power costs, attached-cost,
        // counter removal, X reads): defer to the engine's auto-pay.
        _ => true,
    }
}

/// Rough mana-equivalent weight of a ward tax, for ranking candidates
/// that survived [`ward_tax_payable`]. Life prices at two per mana (the
/// Phyrexian rate), a discarded card at two mana; shapes with no cheap
/// read get a nominal two. Precision is not the point — the term only
/// has to make an un-warded target of equal value, or a different spell
/// entirely, win the tie.
fn ward_tax_burden(tax: &crate::card::WardCost) -> i32 {
    use crate::card::WardCost;
    match tax {
        WardCost::Mana(mc) => mc.cmc() as i32,
        WardCost::Life(n) => (*n as i32 + 1) / 2,
        WardCost::ManaAndLife(mc, n) => mc.cmc() as i32 + (*n as i32 + 1) / 2,
        WardCost::Discard(n) | WardCost::DiscardMatching(_, n) => 2 * *n as i32,
        WardCost::DiscardHand => 3,
        _ => 2,
    }
}

/// `false` when `action` aims at a warded hostile permanent whose tax the
/// bot could not pay after the action's own mana cost. Such a candidate
/// is a dead card, not an expensive one — the resolution path auto-pays
/// or counters (see [`ward_tax_payable`]) — so it is dropped from the
/// pool entirely rather than merely down-ranked. The printed cost stands
/// in for alternative-cost casts (flashback, delve, …); that
/// over-estimates what some casts consume, which errs toward holding a
/// spell, never toward blanking one. Actions with no recognized target
/// shape pass.
/// Which cost [`ward_gate_ok`] would have to cover — *named* rather than
/// read, because reading it is a `find_card_anywhere` walk plus a `ManaCost`
/// clone (one allocation) and only the warded branch wants either.
enum WardedCost {
    Free,
    /// The card's printed cost, wherever the card is.
    Spell(CardId),
    /// The back face's cost (transform / disturb casts).
    BackFace(CardId),
    /// The prepare-cast inset spell's cost, off a battlefield creature.
    PrepareInset(CardId),
    /// One activated ability's mana cost. Granted abilities index past the
    /// printed list; a missing one falls back to free, which errs permissive —
    /// the gate still sees the tax itself.
    Ability(CardId, usize),
}

impl WardedCost {
    fn resolve(&self, state: &GameState) -> ManaCost {
        let empty = || ManaCost::new(Vec::new());
        match *self {
            Self::Free => empty(),
            Self::Spell(id) => state
                .find_card_anywhere(id)
                .map(|c| c.definition.cost.clone())
                .unwrap_or_else(empty),
            Self::BackFace(id) => state
                .find_card_anywhere(id)
                .and_then(|c| c.definition.back_face.as_deref().map(|b| b.cost.clone()))
                .unwrap_or_else(empty),
            Self::PrepareInset(id) => state
                .battlefield_find(id)
                .and_then(|c| c.definition.prepare_spell.as_deref().map(|s| s.cost.clone()))
                .unwrap_or_else(empty),
            Self::Ability(id, i) => state
                .battlefield_find(id)
                .and_then(|c| c.definition.activated_abilities.get(i))
                .map(|a| a.mana_cost.clone())
                .unwrap_or_else(empty),
        }
    }
}

fn ward_gate_ok(state: &GameState, seat: usize, action: &GameAction) -> bool {
    let (which, target, additional): (WardedCost, &Option<Target>, &[Target]) = match action {
        GameAction::CastSpell { card_id, target, additional_targets, .. }
        | GameAction::CastSpellDelve { card_id, target, additional_targets, .. }
        | GameAction::CastGift { card_id, target, additional_targets, .. }
        | GameAction::CastSpellSpree { card_id, target, additional_targets, .. }
        | GameAction::CastSpellConspire { card_id, target, additional_targets, .. }
        | GameAction::CastSpellKicked { card_id, target, additional_targets, .. }
        | GameAction::CastSpellKickers { card_id, target, additional_targets, .. }
        | GameAction::CastSpellMultikicked { card_id, target, additional_targets, .. }
        | GameAction::CastBestow { card_id, target, additional_targets, .. }
        | GameAction::CastAdventure { card_id, target, additional_targets, .. }
        | GameAction::CastOmen { card_id, target, additional_targets, .. }
        | GameAction::CastPrototype { card_id, target, additional_targets, .. }
        | GameAction::CastSplitRight { card_id, target, additional_targets, .. }
        | GameAction::CastAftermath { card_id, target, additional_targets, .. }
        | GameAction::CastFlashback { card_id, target, additional_targets, .. }
        | GameAction::CastMayhem { card_id, target, additional_targets, .. }
        | GameAction::CastHarmonize { card_id, target, additional_targets, .. }
        | GameAction::CastSpellAlternative { card_id, target, additional_targets, .. }
        | GameAction::CastAdventureCreature { card_id, target, additional_targets, .. }
        | GameAction::CastPlotted { card_id, target, additional_targets, .. } => {
            (WardedCost::Spell(*card_id), target, additional_targets.as_slice())
        }
        // Back-face casts pay the back's cost.
        GameAction::CastSpellBack { card_id, target, additional_targets, .. }
        | GameAction::CastDisturb { card_id, target, additional_targets, .. } => {
            (WardedCost::BackFace(*card_id), target, additional_targets.as_slice())
        }
        // Prepare-casts pay the inset spell's cost.
        GameAction::CastPrepareSpell { creature_id, target, additional_targets, .. } => {
            (WardedCost::PrepareInset(*creature_id), target, additional_targets.as_slice())
        }
        GameAction::ActivateAbility { card_id, ability_index, target, additional_targets, .. } => {
            (WardedCost::Ability(*card_id, *ability_index), target, additional_targets.as_slice())
        }
        GameAction::ActivateLoyaltyAbility { target, .. } => (WardedCost::Free, target, &[]),
        _ => return true,
    };
    // The gate only has an opinion when a chosen target is a permanent
    // carrying a real ward cost, and most candidates carry no target at all:
    // `cast_candidates`' `retain` alone asks this ~12,500 times a six-game
    // `cube` run and the cost read was 10,736 allocations and 10,834
    // `find_card_anywhere` walks of them. Collect the taxes first — `Vec::new`
    // does not allocate until one is pushed — and read the cost only if there
    // is one. `ward_tax` is pure, so asking it about every target rather than
    // stopping at the first unpayable one is a reordering, not a behaviour
    // change; the payability decision below is still the same `all`.
    let mut taxes: Vec<crate::card::WardCost> = Vec::new();
    for t in target.iter().chain(additional.iter()) {
        if let Target::Permanent(id) = t
            && let Some(tax) = ward_tax(state, *id, seat)
        {
            taxes.push(tax);
        }
    }
    if taxes.is_empty() {
        return true;
    }
    let mut cost = which.resolve(state);
    // The one candidate shape that sinks *extra* mana into the cast: a
    // plain CastSpell with a chosen X (`max_affordable_x` dumps the whole
    // spare pool into it). Price the X into the gate, or a max-X spell
    // aimed at a warded target taps the bot out of the tax it then owes.
    if let GameAction::CastSpell { x_value: Some(x), .. } = action {
        cost.symbols.push(crate::mana::ManaSymbol::Generic(*x));
    }
    taxes.iter().all(|tax| ward_tax_payable(state, seat, tax, &cost))
}

/// For an X-cost spell (or a spell whose effect reads
/// `Value::XFromCost`), return the largest non-negative X the caster can
/// pay given their current mana pool — leftover generic mana after the
/// fixed (non-X) portion of the cost is what fuels X. Static cost taxes
/// (Damping Sphere etc.) are folded in via
/// `extra_cost_for_card_in_hand`. Returns 0 when the fixed cost itself
/// is more than the available pool — the caller relies on `would_accept`
/// to reject the unaffordable cast.
///
/// We detect X-relevance via either the cost's explicit `{X}` pip
/// (Wrath of the Skies) **or** an `XFromCost` reference inside the
/// effect tree (Banefire / Earthquake / Mind Twist — these have flat
/// fixed costs in the catalog because the engine had no Value::XFromCost
/// wiring at the time they were added; the X mana goes straight into
/// the pool and the bot pumps the spell at its full pool size).
pub fn max_affordable_x(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
    w: &EvalWeights,
) -> u32 {
    let extra = state.extra_cost_for_card_in_hand(seat, card.id)
        + crate::game::actions::colored_spell_tax_for_spell(state, seat, card).cmc();
    max_affordable_x_for_def(state, seat, &card.definition, extra, w)
}

/// [`max_affordable_x`] for a definition that isn't a hand card — the
/// prepare-cast inset spell. `extra` carries any surcharges the caller
/// can compute (hand casts pass their static taxes; prepare copies pass
/// 0, erring optimistic — `would_accept` re-validates the real payment).
pub fn max_affordable_x_for_def(
    state: &GameState,
    seat: usize,
    def: &CardDefinition,
    extra: u32,
    w: &EvalWeights,
) -> u32 {
    if !x_relevant(def) {
        return 0;
    }
    // Everything the seat could still produce, not just what's floating --
    // see `available_mana`. Sizing X off the floating pool alone only
    // worked back when the bot tapped out before deciding anything.
    let pool_total = if w.legacy_pretap {
        state.players[seat].mana_pool.total()
    } else {
        available_mana(state, seat).total
    };
    let fixed_cmc = def.cost.with_x_value(0).cmc();
    let affordable = pool_total.saturating_sub(fixed_cmc + extra);
    // `with_x_value` replaces EVERY X pip, so an {X}{X} cost (Oracle's
    // Gift) pays 2X total — divide the spare mana across the pips or the
    // declared X overshoots what the payment funnel will accept.
    let x_pips = def
        .cost
        .symbols
        .iter()
        .filter(|s| matches!(s, crate::mana::ManaSymbol::X))
        .count()
        .max(1) as u32;
    let affordable = affordable / x_pips;
    // Don't overkill: an `{X}: deal X damage to target creature` spell
    // (creature-only target — can't go to the face) never needs more X
    // than the toughest opposing creature's toughness. Capping here frees
    // the leftover mana for the rest of the turn instead of vanishing it
    // into a 6-damage Disfigure on a 2/2.
    if let Some(cap) = creature_only_x_damage_cap(state, seat, def) {
        return affordable.min(cap);
    }
    affordable
}

/// For a single-target, creature-only `DealDamage` whose amount scales with
/// X, the most X the bot ever needs: the greatest toughness among opposing
/// creatures (so any legal target still dies). `None` for any other shape —
/// player-targetable burn (Banefire) keeps dumping its whole pool into X.
fn creature_only_x_damage_cap(state: &GameState, seat: usize, def: &CardDefinition) -> Option<u32> {
    use crate::effect::Value;
    use crate::effect::Selector;
    let Effect::DealDamage { to, amount } = &def.effect else { return None };
    if !matches!(amount, Value::XFromCost) || !matches!(to, Selector::TargetFiltered { .. }) {
        return None;
    }
    // Must be a creature target that can't be redirected to a player.
    let filter = def.effect.target_filter_for_slot(0)?;
    if filter.can_match_player() {
        return None;
    }
    state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .map(|c| c.toughness().max(0) as u32)
        .max()
}

/// True if X matters for this spell — either the cost has an `{X}` pip
/// or the effect tree mentions `Value::XFromCost`. The latter catches
/// catalog cards (Banefire, Mind Twist, …) whose costs predate the
/// engine's proper X-pip wiring.
pub fn x_relevant(def: &CardDefinition) -> bool {
    def.cost.has_x() || effect_uses_x(&def.effect)
}

fn effect_uses_x(eff: &Effect) -> bool {
    use crate::effect::Value;
    fn value_uses_x(v: &Value) -> bool {
        match v {
            Value::XFromCost => true,
            Value::Sum(parts) => parts.iter().any(value_uses_x),
            Value::Diff(a, b)
            | Value::Times(a, b)
            | Value::Min(a, b)
            | Value::Max(a, b) => value_uses_x(a) || value_uses_x(b),
            Value::NonNeg(inner) => value_uses_x(inner),
            Value::CountOf(_) | Value::PowerOf(_) | Value::ToughnessOf(_)
            | Value::CountersOn { .. } | Value::ManaValueOf(_)
            | Value::DistinctTypesInTopOfLibrary { .. }
            | Value::DistinctTypesInGraveyard { .. } => false,
            _ => false,
        }
    }
    fn predicate_uses_x(p: &crate::effect::Predicate) -> bool {
        use crate::effect::Predicate as P;
        match p {
            P::ValueAtLeast(a, b) | P::ValueAtMost(a, b) | P::ValueEquals(a, b) => {
                value_uses_x(a) || value_uses_x(b)
            }
            P::Not(inner) => predicate_uses_x(inner),
            P::All(parts) | P::Any(parts) => parts.iter().any(predicate_uses_x),
            P::SelectorCountAtLeast { n, .. } => value_uses_x(n),
            _ => false,
        }
    }
    match eff {
        Effect::Seq(steps) => steps.iter().any(effect_uses_x),
        Effect::If { cond, then, else_ } => {
            predicate_uses_x(cond) || effect_uses_x(then) || effect_uses_x(else_)
        }
        Effect::ChooseMode(modes) => modes.iter().any(effect_uses_x),
        Effect::ForEach { body, .. }
        | Effect::Repeat { body, .. }
        | Effect::DelayUntil { body, .. } => effect_uses_x(body),
        Effect::DealDamage { amount, .. }
        | Effect::GainLife { amount, .. }
        | Effect::LoseLife { amount, .. }
        | Effect::Drain { amount, .. }
        | Effect::Draw { amount, .. }
        | Effect::Mill { amount, .. }
        | Effect::Scry { amount, .. }
        | Effect::Surveil { amount, .. }
        | Effect::LookAtTop { amount, .. }
        | Effect::AddCounter { amount, .. }
        | Effect::RemoveCounter { amount, .. }
        | Effect::AddPoison { amount, .. } => value_uses_x(amount),
        Effect::Discard { amount, .. } => value_uses_x(amount),
        Effect::PumpPT { power, toughness, .. } => {
            value_uses_x(power) || value_uses_x(toughness)
        }
        Effect::Sacrifice { count, .. } | Effect::DiscardChosen { count, .. } => {
            value_uses_x(count)
        }
        Effect::CreateToken { count, .. }
        | Effect::CreateTokenCopyOf { count, .. }
        | Effect::CreateTokenCopiesHasteSac { count, .. }
        | Effect::CopySpell { count, .. }
        | Effect::CopySpellWithRiders { count, .. }
        | Effect::CopySpellMayChooseTargets { count, .. } => value_uses_x(count),
        Effect::RevealUntilFind { cap, .. } => value_uses_x(cap),
        Effect::AddFirstSpellTax { count, .. } => value_uses_x(count),
        _ => false,
    }
}

/// If `eff` is (or wraps via `Seq`) a top-level `ChooseMode`, return the
/// number of modes. Otherwise `None`. The bot uses this to enumerate each
/// mode separately when generating castable actions, so a card whose
/// default mode (mode 0) is dead in the current board state (e.g. Drown
/// in the Loch's "counter target spell" with no opp spell on the stack)
/// still surfaces a viable alternate (mode 1: destroy creature).
fn modal_mode_count(eff: &Effect) -> Option<usize> {
    match eff {
        Effect::ChooseMode(modes) => Some(modes.len()),
        // Cast-time multi-mode spells (Choreographed Sparks, Moment of
        // Reckoning): the bot casts them single-mode via the plain
        // `CastSpell { mode }` back-compat path.
        Effect::ChooseModesCast { modes, .. } | Effect::ChooseModesByPoints { modes, .. } => {
            Some(modes.len())
        }
        Effect::Seq(steps) => steps.iter().find_map(modal_mode_count),
        _ => None,
    }
}

/// Resolve the effect branch for a chosen mode. For non-modal effects
/// (or `mode == None`), returns the original effect. For modal effects,
/// returns the chosen mode's body so the auto-target heuristic uses the
/// correct filter for that mode.
fn mode_branch(eff: &Effect, mode: Option<usize>) -> &Effect {
    match (eff, mode) {
        (Effect::ChooseMode(modes), Some(m)) if m < modes.len() => &modes[m],
        (Effect::ChooseModesCast { modes, .. } | Effect::ChooseModesByPoints { modes, .. }, Some(m))
            if m < modes.len() =>
        {
            &modes[m]
        }
        (Effect::Seq(steps), Some(_)) => steps
            .iter()
            .find(|s| matches!(s, Effect::ChooseMode(_)))
            .map(|s| mode_branch(s, mode))
            .unwrap_or(eff),
        _ => eff,
    }
}

fn can_afford_with_extra(
    printed: &ManaCost,
    pool: &ManaPool,
    extra_generic: u32,
    reduction: u32,
) -> bool {
    let mut cost = if printed.has_x() { printed.with_x_value(0) } else { printed.clone() };
    if reduction > 0 {
        cost.reduce_generic(reduction);
    }
    if extra_generic > 0 {
        cost.symbols.push(crate::mana::ManaSymbol::Generic(extra_generic));
    }
    pool.clone().pay(&cost).is_ok()
}

/// Pick a sensible auto-target for a spell cast by `caster` using the
/// engine's shared targeting heuristic.
pub fn choose_target(state: &GameState, def: &CardDefinition, caster: usize) -> Option<Target> {
    state.auto_target_for_effect(&def.effect, caster)
}

/// True when `ta` is the canonical Strixhaven magecraft trigger:
/// SpellCast scope=YourControl with the IS-only predicate. Used by
/// the bot's spell-bias heuristic so a controlled magecraft permanent
/// nudges the bot toward casting an IS spell to fire the trigger.
fn is_magecraft_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::{EventKind, EventScope};
    matches!(ta.event.kind, EventKind::SpellCast)
        && matches!(ta.event.scope, EventScope::YourControl)
        && ta.event.filter.is_some()
}

/// True when `ta` is an Opus-style rider (SOS): an on-cast trigger whose
/// body branches on `Predicate::CastSpellManaSpentAtLeast` — "if five or
/// more mana was spent to cast that spell, [big] instead". See
/// `shortcut::opus_trigger`.
fn is_opus_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::EventKind;
    fn branches(e: &Effect) -> bool {
        match e {
            Effect::If { cond, then, else_ } => {
                matches!(cond, crate::effect::Predicate::CastSpellManaSpentAtLeast(_))
                    || branches(then)
                    || branches(else_)
            }
            Effect::Seq(v) => v.iter().any(branches),
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => branches(body),
            _ => false,
        }
    }
    matches!(ta.event.kind, EventKind::SpellCast) && branches(&ta.effect)
}

/// True when `ta` is an Increment rider (SOS): an on-cast trigger gated
/// on `Predicate::IncrementSatisfied` — "if the amount of mana spent is
/// greater than this creature's power or toughness, put a +1/+1 counter
/// on it". See `shortcut::increment_trigger`.
fn is_increment_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::EventKind;
    fn branches(e: &Effect) -> bool {
        match e {
            Effect::If { cond, then, else_ } => {
                matches!(cond, crate::effect::Predicate::IncrementSatisfied)
                    || branches(then)
                    || branches(else_)
            }
            Effect::Seq(v) => v.iter().any(branches),
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => branches(body),
            _ => false,
        }
    }
    matches!(ta.event.kind, EventKind::SpellCast) && branches(&ta.effect)
}

/// The smallest mana-spent total that grows at least one of the bot's
/// Increment bodies: `min(power, toughness) + 1` over them (the gate is
/// "spent > power OR toughness", so clearing the smaller stat suffices).
/// `None` with no Increment body out. Computed stats, so the threshold
/// climbs as counters land — exactly the printed escalation.
fn increment_threshold(state: &GameState, seat: usize) -> Option<u32> {
    state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.definition.triggered_abilities.iter().any(is_increment_trigger)
        })
        .filter_map(|c| state.computed_permanent(c.id))
        .map(|cp| (cp.power.min(cp.toughness).max(0) + 1) as u32)
        .min()
}

/// True when `ta` is a Repartee trigger (SOS): an on-cast event filter
/// that requires the spell to target a creature. See `shortcut::repartee`.
fn is_repartee_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::EventKind;
    use crate::effect::Predicate;
    fn wants_creature_target(p: &Predicate) -> bool {
        match p {
            Predicate::CastSpellTargetsMatch(_) => true,
            Predicate::All(v) => v.iter().any(wants_creature_target),
            _ => false,
        }
    }
    matches!(ta.event.kind, EventKind::SpellCast)
        && ta.event.filter.as_ref().is_some_and(wants_creature_target)
}

/// True when `eff` carries a this-turn-lifegain gate (SOS Infusion) —
/// the shape whose payoff a pre-gain cast wastes.
fn effect_infusion_gated(eff: &Effect) -> bool {
    use crate::effect::Predicate;
    fn gated(p: &Predicate) -> bool {
        match p {
            Predicate::LifeGainedThisTurnAtLeast { .. }
            | Predicate::FirstLifeGainThisTurn { .. } => true,
            Predicate::All(v) => v.iter().any(gated),
            _ => false,
        }
    }
    match eff {
        Effect::If { cond, then, else_ } => {
            gated(cond) || effect_infusion_gated(then) || effect_infusion_gated(else_)
        }
        Effect::Seq(v) => v.iter().any(effect_infusion_gated),
        Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => effect_infusion_gated(body),
        _ => false,
    }
}

/// Whether any face of `def` is Infusion-gated — spell body or a
/// triggered rider (the ETB Infusion shape).
fn card_infusion_gated(def: &CardDefinition) -> bool {
    effect_infusion_gated(&def.effect)
        || def.triggered_abilities.iter().any(|t| effect_infusion_gated(&t.effect))
}

/// Mana the bot would spend casting `a`: printed cost plus the chosen X.
/// Only the plain-cast shape is priced — it is the one that carries a
/// live `x_value` — which is all the Opus nudge needs.
fn cast_mana_spent(state: &GameState, seat: usize, a: &GameAction) -> u32 {
    match a {
        GameAction::CastSpell { card_id, x_value, .. } => state.players[seat]
            .hand
            .iter()
            .find(|c| c.id == *card_id)
            .map(|c| c.definition.cost.cmc() + x_value.unwrap_or(0))
            .unwrap_or(0),
        _ => 0,
    }
}

/// True when resolving `a` gains the caster life — the Infusion unlock.
/// Lifelink creatures count: cast precombat, they gain before a
/// postcombat Infusion payoff checks the turn's total.
fn cast_gains_life(state: &GameState, seat: usize, a: &GameAction) -> bool {
    use crate::effect::{PlayerRef, Selector};
    let GameAction::CastSpell { card_id, .. } = a else { return false };
    let Some(c) = state.players[seat].hand.iter().find(|c| c.id == *card_id) else {
        return false;
    };
    fn gains(e: &Effect) -> bool {
        let hits_self = |s: &Selector| {
            matches!(s, Selector::You | Selector::This)
                || matches!(s, Selector::Player(PlayerRef::You))
        };
        match e {
            Effect::GainLife { who, .. } => hits_self(who),
            Effect::Drain { to, .. } => hits_self(to),
            Effect::Seq(v) => v.iter().any(gains),
            Effect::If { then, else_, .. } => gains(then) || gains(else_),
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => gains(body),
            _ => false,
        }
    }
    gains(&c.definition.effect)
        || c.definition.keywords.has_kw(&crate::card::Keyword::Lifelink)
}

/// Best hostile creature the effect's primary slot accepts — the
/// Repartee swap-in for an IS cast the auto-targeter aimed at a player.
/// Highest board value first; `would_accept` re-checks full legality
/// (hexproof, protection) at the probe site.
fn best_hostile_creature_target(
    state: &GameState,
    seat: usize,
    eff: &Effect,
    w: &EvalWeights,
) -> Option<Target> {
    let filter = eff.primary_target_filter();
    let mut foes: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .collect();
    foes.sort_by_cached_key(|c| std::cmp::Reverse(permanent_value(state, c.id, w)));
    foes.into_iter().map(|c| Target::Permanent(c.id)).find(|t| match &filter {
        Some(f) => state.evaluate_requirement_static(f, t, seat, None),
        None => true,
    })
}

/// True when casting `def` reads the converge count — the distinct colors of
/// mana spent. One oracle, [`CardDefinition::wants_converge`], shared with the
/// payment path.
///
/// This used to be a second, hand-written walk of the effect tree, and the two
/// disagreed in both directions: the walker enumerated fifteen `Effect` arms,
/// so converge in any other arm — or in an activated or triggered ability —
/// was invisible to it, while it was the only side that knew about
/// `SelectionRequirement::ManaValueAtMostConverged`. The oracle now covers
/// both spellings, and reading the whole definition rather than the cast's own
/// effect only over-approximates here: the pre-float below is bounded on every
/// other side, so a spare tap is the worst a false positive costs.
fn card_reads_converge(def: &CardDefinition) -> bool {
    def.wants_converge()
}

/// SOS Converge pre-float: when the bot's chosen play scales with the
/// distinct colors of mana spent, tap one plain source of a color the
/// pool doesn't hold yet and cast NEXT tick — the payment funnel spends
/// pool mana first, so every floated color is a drained (counted) color
/// when the cast goes off. Bounded on every side: only fires while the
/// float is smaller than the cost's mana value (excess would strand and
/// vanish at end of phase), only from single-fixed-color tap-only
/// sources with no life cost (no ChooseColor prompt, no pain), and each
/// firing adds a color the pool lacked, so at most four taps precede the
/// cast.
fn pick_converge_prefloat(
    state: &GameState,
    seat: usize,
    action: &GameAction,
) -> Option<GameAction> {
    use crate::mana::Color;
    let def: &CardDefinition = match action {
        GameAction::CastSpell { card_id, .. } => {
            &state.players[seat].hand.iter().find(|c| c.id == *card_id)?.definition
        }
        GameAction::CastPrepareSpell { creature_id, .. } => {
            state.battlefield_find(*creature_id)?.definition.prepare_spell.as_deref()?
        }
        _ => return None,
    };
    if !card_reads_converge(def) {
        return None;
    }
    let pool = &state.players[seat].mana_pool;
    if pool.total() >= def.cost.cmc() {
        return None;
    }
    for c in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (idx, a) in c.definition.activated_abilities.iter().enumerate() {
            if !is_countable_mana_ability(a) || a.life_cost > 0 {
                continue;
            }
            let (amount, colors, colorless) = mana_ability_output(&a.effect);
            if amount == 0 || colorless || colors.len() != 1 {
                continue;
            }
            let Some(color) = Color::ALL.into_iter().find(|c| colors.contains(*c)) else {
                continue;
            };
            if pool.amount(color) > 0 {
                continue;
            }
            let tap = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(tap.clone()) {
                return Some(tap);
            }
        }
    }
    None
}

/// True when the card with id `cid` in `seat`'s hand is an instant or
/// sorcery. Cheap helper for the magecraft-bias path; falls back to
/// false on missing cards.
fn is_instant_or_sorcery_in_hand(state: &GameState, seat: usize, cid: CardId) -> bool {
    use crate::card::CardType;
    state.players[seat]
        .hand
        .iter()
        .find(|c| c.id == cid)
        .map(|c| {
            c.definition.card_types.contains(&CardType::Instant)
                || c.definition.card_types.contains(&CardType::Sorcery)
        })
        .unwrap_or(false)
}

/// For a *beneficial* Aura in hand (positive `equipped_bonus` stats or a
/// granted keyword), pick the bot's most valuable creature that satisfies
/// the enchant filter as the host. Returns `None` for non-Auras and for
/// debuff Auras (negative stats — Pacifism-style restrictions live in
/// other def fields and keep the hostile auto-target walk). Without this,
/// `Effect::Attach` falls into the auto-targeter's hostile branch and a
/// Rancor prefers the opponent's creatures.
fn is_beneficial_aura(def: &CardDefinition) -> bool {
    use crate::card::EnchantmentSubtype;
    if !def.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura) {
        return false;
    }
    def.equipped_bonus.as_ref().is_some_and(|bonus| {
        bonus.power + bonus.toughness > 0
            || (bonus.power + bonus.toughness == 0 && !bonus.keywords.is_empty())
    })
}

fn beneficial_aura_host(
    state: &GameState,
    seat: usize,
    aura: &crate::card::CardInstance,
    w: &EvalWeights,
) -> Option<crate::game::Target> {
    let def = &aura.definition;
    if !is_beneficial_aura(def) {
        return None;
    }
    let filter = def.effect.primary_target_filter();
    let mut hosts: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .collect();
    hosts.sort_by_cached_key(|c| std::cmp::Reverse(permanent_value(state, c.id, w)));
    hosts
        .into_iter()
        .map(|c| crate::game::Target::Permanent(c.id))
        .find(|t| match &filter {
            Some(f) => state.evaluate_requirement_static(f, t, seat, Some(aura.id)),
            None => true,
        })
}

/// Best cutoff for "choose a number; destroy all creatures with power ≥
/// it": maximize destroyed enemy value minus destroyed own value,
/// breaking ties upward (spare more of everyone's board when equal).
fn best_destroy_power_cutoff(state: &GameState, seat: usize, max: u32, w: &EvalWeights) -> u32 {
    let mut best = (i32::MIN, 0u32);
    for n in 0..=max {
        let mut score = 0i32;
        for c in state.battlefield.iter().filter(|c| c.definition.is_creature()) {
            let power = state.computed_permanent_on(c).map(|cp| cp.power).unwrap_or(c.power());
            if power >= n as i32 {
                let v = permanent_value(state, c.id, w);
                score += if c.controller == seat { -v } else { v };
            }
        }
        if score > best.0 || (score == best.0 && n > best.1) {
            best = (score, n);
        }
    }
    best.1
}

/// True when `def` carries a static that keys off the Prepared counter
/// (SOS "prepared creatures you control get …" payoffs). Matched
/// structurally on the pump/keyword-grant shapes those payoffs use.
fn static_rewards_prepared(def: &CardDefinition) -> bool {
    use crate::card::{SelectionRequirement as R, Selector};
    use crate::effect::StaticEffect;
    fn req_mentions_prepared(r: &R) -> bool {
        match r {
            R::WithCounter(crate::card::CounterType::Prepared) => true,
            R::And(a, b) | R::Or(a, b) => req_mentions_prepared(a) || req_mentions_prepared(b),
            _ => false,
        }
    }
    let sel_mentions = |s: &Selector| match s {
        Selector::EachPermanent(r) => req_mentions_prepared(r),
        _ => false,
    };
    def.static_abilities.iter().any(|sa| match &sa.effect {
        StaticEffect::PumpPT { applies_to, .. }
        | StaticEffect::GrantKeyword { applies_to, .. } => sel_mentions(applies_to),
        _ => false,
    })
}

/// First damage amount a spell's effect tree deals (walking `Seq`), with
/// `{X}` resolved to the candidate's chosen X. `None` when the effect deals
/// no (statically knowable) damage — non-Const amounts are treated as
/// unknown rather than guessed.
fn first_damage_amount(effect: &Effect, x: u32) -> Option<i32> {
    use crate::effect::Value;
    match effect {
        Effect::DealDamage { amount, .. } => match amount {
            Value::Const(n) => Some(*n),
            Value::XFromCost => Some(x as i32),
            _ => None,
        },
        Effect::Seq(steps) => steps.iter().find_map(|e| first_damage_amount(e, x)),
        _ => None,
    }
}

/// Heuristic rank for one candidate play. Rough scale:
///
/// * mana investment counts double (printed cmc + chosen X + kick count) —
///   the bot leads with its biggest affordable play and spends its pool;
/// * a creature body adds its printed stats plus a small keyword nod, so
///   on-curve bodies outrank cantrip filler;
/// * a targeted effect adds the value of what it hits — an opponent's
///   permanent contributes its full `permanent_value`, so removal chases
///   the biggest threat and a Bolt at a 1/1 loses to a Bolt at a dragon
///   (or to just deploying a bomb instead);
/// * enhanced cast variants (kicker, delve, gift, bestow, conspire, …) get
///   a flat edge over the plain cast of the same card, so the upside line
///   wins whenever both are affordable.
///
/// The caller adds jitter for tie-breaks; scores only need to be
/// *relatively* right within one candidate pool.
/// Material evaluation of a state from `seat`'s perspective: a decided
/// game dominates everything, then board presence (`permanent_value` ×3
/// per permanent, opponents' counted against), hand size (×2), and life.
/// Deliberately coarse — it's compared between candidate *outcomes* of the
/// same tick, so shared terms cancel and only the action's delta matters.
/// Phase weight for the ply-scheduled blend: 1.0 through turn 5, linear
/// to 0.0 at turn 12, matching where the stratified calibrations put the
/// net's edge (peak ply 8–11 ≈ turns 3–4; gone by ply 32+ ≈ turn 11).
fn ply_blend_factor(turn: u32) -> f32 {
    const FULL_UNTIL: f32 = 5.0;
    const ZERO_AT: f32 = 12.0;
    ((ZERO_AT - turn as f32) / (ZERO_AT - FULL_UNTIL)).clamp(0.0, 1.0)
}

fn eval_material(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    // Scores a whole board, so it reads every permanent's computed state —
    // and the sims call it on a cloned (unfrozen) state once per candidate.
    state.with_frozen_layers(|state| eval_material_frozen(state, seat, w))
}

fn eval_material_frozen(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    // The learned value net, when a profile asks for it and a net is
    // loaded. Undecided positions only: the heuristic's ±100 000·unit for
    // a decided game must keep dominating the net's 0..10 000 range so
    // "actually winning" always outranks "the net likes it".
    if w.net_slot != 0
        && state.game_over.is_none()
        && let Some(p) = super::net_eval::win_prob(state, seat, w.net_slot)
    {
        // Snap onto the grid *before* anything downstream sees it, so
        // both the replacement and blend paths inherit the tie.
        let p = if w.net_quantize > 0 {
            let q = w.net_quantize as f32;
            (p * q).round() / q
        } else {
            p
        };
        if w.net_blend_scale > 0 {
            let mut bias = ((p - 0.5) * (w.net_blend_scale * w.unit) as f32) as i32;
            if w.net_blend_ply {
                bias = (bias as f32 * ply_blend_factor(state.turn_number)) as i32;
            }
            return eval_material_inner(state, seat, w, false) + bias;
        }
        return (p * 10_000.0) as i32;
    }
    eval_material_inner(state, seat, w, false)
}

/// [`eval_material`] with `seat`'s own summoning-sick creatures counted as
/// worth nothing.
///
/// Forge's `GameStateEvaluator` carries this alongside the real score as
/// `summonSickValue`, and uses it to answer one question: does this line
/// achieve anything *this turn*, or does it only add a body that can't
/// attack yet? A creature deployed in the precombat main and a creature
/// deployed after combat are worth the same at end of turn, but the second
/// one was played with a turn's more information and left the mana up in
/// between. Only the first reads as progress to a greedy evaluator, which
/// is why this bot puts 95 % of its plays in the precombat main.
fn eval_material_summon_sick_blind(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    state.with_frozen_layers(|state| eval_material_inner(state, seat, w, true))
}

fn eval_material_inner(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    blind_to_sick: bool,
) -> i32 {
    if let Some(over) = state.game_over {
        return match over {
            Some(winner) if winner == seat => 100_000 * w.unit,
            Some(_) => -100_000 * w.unit,
            None => 0,
        };
    }
    let mut v = 0i32;
    // `same_team` is a function of the *seat*, not of the permanent, and both
    // loops below asked it once per element — 55,664 calls a six-game `cube`
    // run for at most `players.len()` distinct answers. Resolve them once, as
    // a bitmask rather than a collection: a `SmallVec<[bool; 4]>` here cost
    // most of what it saved, all of it the collect and the per-element bounds
    // check. Past seat 63 the mask runs out and the question is asked as
    // before, so this is exact at any table size.
    let mut hostile = 0u64;
    for s in 0..state.players.len().min(64) {
        if !state.same_team(s, seat) {
            hostile |= 1 << s;
        }
    }
    let is_hostile = |s: usize| {
        if s < 64 { hostile & (1 << s) != 0 } else { !state.same_team(s, seat) }
    };
    for c in &state.battlefield {
        // Lands are worth a small flat amount — enough that ramp/fetch
        // registers and land destruction isn't free, without a flooded
        // board dominating the material count.
        let pv = if c.definition.is_land() {
            2 * w.unit
        } else {
            let mut pv = permanent_value_with(state, c.id, Some(c), w);
            // Loyalty is a spendable RESOURCE, not material: counting it
            // here made every plus ability self-rewarding (+2 loyalty
            // read as +6 material for free) and every ultimate
            // self-punishing (−6 loyalty read as −18), so walkers ticked
            // up forever. `permanent_value` keeps the loyalty term for
            // removal targeting — a fat walker is still the best target.
            if c.definition.is_planeswalker() {
                pv -= c.counter_count(crate::card::CounterType::Loyalty) as i32 * w.unit;
            }
            3 * pv
        };
        // A body that can't attack yet isn't this turn's progress -- see
        // `eval_material_summon_sick_blind`.
        let sick = blind_to_sick
            && c.controller == seat
            && c.definition.is_creature()
            && c.summoning_sick
            && !c.has_keyword(&crate::card::Keyword::Haste);
        let pv = if sick { 0 } else { pv };
        if c.controller == seat {
            v += pv;
        } else if is_hostile(c.controller) {
            v -= pv;
        }
    }
    for (i, p) in state.players.iter().enumerate() {
        if !p.is_alive() {
            continue;
        }
        // A hand card at 4 ≈ half an average permanent — enough that
        // "draw a card" beats "gain 3 life" (a card is a future play;
        // three life at a healthy total is nearly nothing).
        let emblems: i32 = p.emblems.iter().map(|e| emblem_value(state, i, e)).sum();
        // CR 725 / 726 — the crown and the initiative are recurring resources,
        // not one-shots: the monarch draws at each of their end steps and the
        // initiative-holder ventures on top of that. Priced above a single
        // hand card (4) because they keep paying until someone takes them.
        let crown = i32::from(state.monarch == Some(i)) * 7
            + i32::from(state.initiative == Some(i)) * 9;
        let material = (4 * p.hand.len() as i32 + emblems + crown) * w.unit
            + life_value(state.effective_life(i), w);
        if i == seat {
            v += material;
        } else if is_hostile(i) {
            v -= material;
        }
    }
    v
}

/// Material value of one emblem for `seat`. Emblems are ultimates and
/// usually game-bending — but a CONDITIONAL emblem is only worth what
/// the deck can feed it. A lifegain-triggered emblem (Professor Dellian
/// Fel's "whenever you gain life, target opponent loses that much") is
/// priced by the seat's visible lifegain sources: with none it's nearly
/// dead (2 — below a +2 ability's gain-3, so the walker holds the fort
/// instead of ulting into nothing), and each source adds 6, capped at
/// 32. A flat price made the bot ult indiscriminately and Fel's fleet
/// attribution DROPPED — the build-around emblem needs the build.
/// Unconditional emblems keep the flat 25.
fn emblem_value(state: &GameState, seat: usize, emblem: &crate::player::Emblem) -> i32 {
    use crate::effect::{EventKind, Value};
    // Ajani-style "whenever you gain life" emblems are worth what the
    // board can feed them — the original special case, kept as-is.
    let lifegain_triggered =
        emblem.triggered.iter().any(|t| matches!(t.event.kind, EventKind::LifeGained));
    if lifegain_triggered {
        return 2 + 6 * lifegain_sources(state, seat).min(5);
    }
    // Everything else used to be a flat 25, which made a game-winning
    // draw engine and a minor rider read the same — the "ultimates the
    // eval can't see" limitation was really "ultimates the eval can't
    // tell apart". Price the recurring payoff by shape instead: card
    // advantage highest (an emblem draw repeats every turn, unanswerable
    // by design), damage and tokens next, anthem statics per body they
    // could pump. Floor near the old constant so unrecognized shapes
    // aren't suddenly worthless; cap so no emblem reads as strictly
    // game-over while the game is still being played.
    let amount = |v: &Value| match v {
        Value::Const(n) => (*n).max(1),
        _ => 2,
    };
    fn shape_value(e: &Effect, amount: &dyn Fn(&Value) -> i32) -> i32 {
        match e {
            Effect::Draw { amount: a, .. } => 12 * amount(a),
            Effect::DealDamage { amount: a, .. } | Effect::Drain { amount: a, .. } => {
                6 * amount(a)
            }
            Effect::CreateToken { count, .. } => 10 * amount(count),
            Effect::GainLife { amount: a, .. } => 2 * amount(a),
            Effect::Seq(v) => v.iter().map(|e| shape_value(e, amount)).sum(),
            Effect::If { then, else_, .. } => {
                shape_value(then, amount).max(shape_value(else_, amount))
            }
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => {
                shape_value(body, amount)
            }
            _ => 8,
        }
    }
    let triggered: i32 =
        emblem.triggered.iter().map(|t| shape_value(&t.effect, &amount)).sum();
    let statics = 12 * emblem.statics.len() as i32;
    (triggered + statics).clamp(20, 60)
}

/// Visible lifegain sources for `seat`: battlefield lifelink bodies, and
/// battlefield/hand cards whose effect trees gain the controller life
/// (GainLife, Drain). Loyalty abilities are deliberately NOT scanned —
/// the emblem-maker mustn't count itself as its own payoff.
fn lifegain_sources(state: &GameState, seat: usize) -> i32 {
    fn gains_life(e: &Effect) -> bool {
        match e {
            Effect::GainLife { .. } | Effect::Drain { .. } => true,
            Effect::Seq(v) => v.iter().any(gains_life),
            Effect::If { then, else_, .. } => gains_life(then) || gains_life(else_),
            Effect::MayDo { body, .. } => gains_life(body),
            Effect::ChooseMode(modes) => modes.iter().any(gains_life),
            Effect::ApplyToTargets { effect, .. } => gains_life(effect),
            _ => false,
        }
    }
    fn card_gains_life(def: &CardDefinition) -> bool {
        def.keywords.has_kw(&crate::card::Keyword::Lifelink)
            || gains_life(&def.effect)
            || def.triggered_abilities.iter().any(|t| gains_life(&t.effect))
            || def.activated_abilities.iter().any(|a| gains_life(&a.effect))
    }
    let battlefield = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && card_gains_life(&c.definition))
        .count();
    let hand = state.players[seat]
        .hand
        .iter()
        .filter(|c| card_gains_life(&c.definition))
        .count();
    (battlefield + hand) as i32
}

/// Advance `g` through this turn's combat, so a candidate line can be
/// scored on the board it actually produces rather than the board that
/// exists the instant it resolves.
///
/// This is the single biggest gap between this evaluator and the reference
/// AIs. `evaluate_action_outcome` snapshots immediately after resolution,
/// which cannot see that the creature just cast dies on the crack-back,
/// that the removal spell opened a lethal swing, or that the 2/2 deployed
/// into an empty board is about to trade with a 4/4. Forge scores nothing
/// without first fast-forwarding a copy to combat damage
/// (`GameStateEvaluator.simulateUpcomingCombatThisTurn`); this is that,
/// driven by the bot's own `pick_attacks` / `pick_blocks` so the simulated
/// combat is the combat this bot would actually play.
///
/// What a combat simulation did.
///
/// Previously a `bool`, which conflated "there was no combat to look at"
/// with "the simulation ran out of fuel partway". Callers treat those
/// oppositely — the first means score the state as-is, the second means
/// refuse to score a board where attackers are tapped but damage was never
/// dealt — and collapsing them made every evaluation on a board with no
/// possible attackers unscoreable, silently dropping the whole position
/// back to the static rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatSim {
    /// Nothing to simulate; `g` is untouched.
    Skipped,
    /// Ran through combat damage.
    Completed,
    /// Started and could not finish; `g` is now a torn state.
    Incomplete,
}

/// Bails cheaply — without touching `g` — when there is no combat to look
/// at: the game is over, the turn is already past combat damage, or the
/// active player has no creature that could attack. Forge guards the same
/// way, because the state copy is the expensive part.
/// The `Skipped` precondition of [`simulate_through_combat`], hoisted so a
/// caller can ask it *before* cloning the state it would hand over.
///
/// A `Skipped` walk leaves `g` byte-identical, so a caller that clones only to
/// simulate and then score can score the original instead. The two callers
/// that do this ([`score_settled_state`] and [`improves_this_turn`]'s idle
/// baseline) ask through here rather than restating the test, so the
/// precondition cannot drift from the walk it guards.
fn combat_sim_skips(g: &GameState) -> bool {
    if g.is_game_over() || g.step >= TurnStep::CombatDamage {
        return true;
    }
    let attacker_seat = g.active_player_idx;
    !g.battlefield.iter().any(|c| {
        c.controller == attacker_seat
            && c.definition.is_creature()
            && !c.tapped
            && (!c.summoning_sick || c.has_keyword(&crate::card::Keyword::Haste))
    })
}

fn simulate_through_combat(g: &mut GameState, fuel: &mut u32, w: &EvalWeights) -> CombatSim {
    if combat_sim_skips(g) {
        return CombatSim::Skipped;
    }
    let turn = g.turn_number;
    let mut attacks_submitted = false;
    let mut blocks_submitted = false;
    while g.step < TurnStep::CombatDamage && g.turn_number == turn && !g.is_game_over() {
        *fuel = match fuel.checked_sub(1) {
            Some(f) => f,
            None => return CombatSim::Incomplete,
        };
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(g, pending.acting_player(), w, &pending.decision, false)
            };
            // Not `dry_run`: `combat_aware`'s `before` probe scores this
            // state even when the walk comes back `Incomplete`, so a
            // rejected action here must still be rolled back.
            if g.perform_action(GameAction::SubmitDecision(answer)).is_err() {
                return CombatSim::Incomplete;
            }
            continue;
        }
        let action = match g.step {
            TurnStep::DeclareAttackers if !attacks_submitted => {
                attacks_submitted = true;
                let declarer = g.attack_declarer();
                GameAction::DeclareAttackers(pick_attacks(g, declarer))
            }
            TurnStep::DeclareBlockers if !blocks_submitted && !g.attacking().is_empty() => {
                // The defender is not the priority holder at this point, so
                // ask the engine which seat is actually owed the
                // declaration. Getting this wrong silently leaves every
                // attacker unblocked, which flatters the attack.
                match (0..g.players.len()).find(|&s| g.may_declare_blocks(s)) {
                    Some(defender) => {
                        blocks_submitted = true;
                        GameAction::DeclareBlockers(pick_blocks(g, defender))
                    }
                    None => GameAction::PassPriority,
                }
            }
            _ => GameAction::PassPriority,
        };
        // Checkpointed for the same reason as the decision above: an
        // abandoned walk's state is read, so it has to be the rolled-back
        // one. A rejected declaration would spin forever, hence the pass.
        // Both results are discarded; hand the buffer back (`recycle_events`).
        match g.perform_action(action) {
            Ok(events) => g.recycle_events(events),
            Err(_) => match g.perform_action(GameAction::PassPriority) {
                Ok(events) => g.recycle_events(events),
                Err(_) => return CombatSim::Incomplete,
            },
        }
    }
    CombatSim::Completed
}

/// Dry-run `action` to quiescence on a full-state clone (libraries kept —
/// resolution may draw) and score the result for `seat`: the cast is
/// applied, then priority passes with [`AutoDecider`] answers for any
/// decision that surfaces until the stack empties. `None` on rejection or
/// a resolution that won't settle — callers fall back to the static rank.
fn evaluate_action_outcome(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    settled: Option<&GameState>,
    w: &EvalWeights,
) -> Option<i32> {
    // Under determinization the planner's dry-runs must not read the true
    // hidden zones either: a resolution that draws would otherwise plan
    // around the exact card coming. One redeal (k = 0), not an average —
    // the planner evaluates many candidates per decision and the salt is
    // turn/step-keyed, so every finalist is judged against the *same*
    // redeal, which keeps the comparison among candidates consistent.
    // The recursion inside `evaluate_action_sequence` must NOT re-redeal:
    // it continues a line through cards the sim already drew.
    if w.determinize > 0 {
        // The redeal makes `settled` the wrong state: it was produced from the
        // true hidden zones, and this line has to be judged against the same
        // redeal as every other finalist.
        let g = sim_start_state(state, seat, w, 0);
        return evaluate_action_sequence(&g, seat, action, None, w, w.lookahead);
    }
    evaluate_action_sequence(state, seat, action, settled, w, w.lookahead)
}

/// Score of the best *sequence* of up to `depth + 1` plays that starts with
/// `action`, rather than the score the moment `action` resolves.
///
/// This is the gap a one-action-at-a-time evaluator cannot close: with four
/// mana available, "cast the four-drop" and "cast a two-drop" are compared
/// as single plays, so the bot never sees that the second line continues
/// into *another* two-drop and ends the turn ahead. Forge searches
/// sequences to three plies for exactly this reason
/// (`SpellAbilityPicker` recursing through `SimulationController`).
///
/// Stopping is always one of the options considered, so a sequence is never
/// forced to spend everything — dumping the hand is a line, not an
/// obligation.
fn evaluate_action_sequence(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    settled: Option<&GameState>,
    w: &EvalWeights,
    depth: u8,
) -> Option<i32> {
    // `settled` is `state` with `action` already run on it, handed over by the
    // probe that validated the candidate — see [`Finalist`]. Cloning it is the
    // same two lines below minus the cast.
    let mut g = match settled {
        Some(g) => g.clone(),
        None => {
            let mut g = state.clone();
            dry_run(&mut g, action.clone()).ok()?;
            g
        }
    };
    let mut fuel = 64u32;
    loop {
        if g.is_game_over() {
            break;
        }
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            dry_run(&mut g, GameAction::SubmitDecision(answer)).ok()?;
        } else if g.stack.is_empty() {
            break;
        } else {
            dry_run(&mut g, GameAction::PassPriority).ok()?;
        }
        fuel = fuel.checked_sub(1)?;
    }
    // The value of stopping here.
    let mut best = score_settled_state(&g, seat, w)?;
    if depth > 0 {
        for follow in follow_up_candidates(&g, seat, w) {
            if let Some(v) = evaluate_action_sequence(&g, seat, &follow, None, w, depth - 1) {
                best = best.max(v);
            }
        }
    }
    Some(best)
}

/// Score a state that has resolved to quiescence, running it through this
/// turn's combat first when the profile asks for it. `None` when the combat
/// simulation can't complete — see `simulate_through_combat`.
fn score_settled_state(g: &GameState, seat: usize, w: &EvalWeights) -> Option<i32> {
    if !w.combat_aware {
        return Some(eval_material(g, seat, w));
    }
    // Score the board this line actually leads to, not the one that exists
    // the moment it resolves -- see `simulate_through_combat`. Its own fuel
    // budget: combat is a long way through the step machine (two
    // declarations plus a priority round per step, before triggers).
    // A skipped walk leaves the clone byte-identical to `g`, so there was
    // nothing to clone for: score `g` itself. Most settled states reach here
    // with no untapped unsick creature, or already past combat damage.
    if combat_sim_skips(g) {
        let v = eval_material(g, seat, w);
        super::leaf_capture::maybe(g, seat, v);
        return Some(v);
    }
    let mut sim = g.clone();
    let mut combat_fuel = 256u32;
    match simulate_through_combat(&mut sim, &mut combat_fuel, w) {
        // A half-simulated combat is worse than none: attackers are
        // declared and tapped but damage was never dealt, so the line reads
        // as pure downside. Refuse to score a torn state -- the caller
        // falls back to the static rank.
        CombatSim::Incomplete => None,
        // Skipped leaves `sim` untouched, so scoring it is just scoring `g`.
        CombatSim::Skipped | CombatSim::Completed => {
            let v = eval_material(&sim, seat, w);
            super::leaf_capture::maybe(&sim, seat, v);
            Some(v)
        }
    }
}

/// The few plays worth considering as a continuation of a sequence:
/// the best-scoring validated candidates from `g`, capped hard.
///
/// The cap is the whole reason this is affordable. Enumerating candidates
/// runs an engine dry-run per specialty card shape, so a wide branching
/// factor at every ply would cost far more than the extra ply is worth;
/// two continuations is enough to catch the case this exists for (a second
/// cheap spell the greedy pick would have priced out).
fn follow_up_candidates(g: &GameState, seat: usize, w: &EvalWeights) -> Vec<GameAction> {
    const MAX_FOLLOW_UPS: usize = 2;
    // Only when the bot could actually take another play right now: still
    // its own main phase, holding priority, nothing on the stack.
    if g.is_game_over()
        || g.pending_decision.is_some()
        || !g.stack.is_empty()
        || g.active_player_idx != seat
        || g.player_with_priority() != seat
        || !matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
    {
        return Vec::new();
    }
    let mut ranked: Vec<(i32, GameAction, bool)> = cast_candidates(g, seat, w, None)
        .into_iter()
        .map(|(a, ok)| (score_candidate(g, seat, &a, w), a, ok))
        .collect();
    ranked.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
    let mut out = Vec::with_capacity(MAX_FOLLOW_UPS);
    for (_, a, ok) in ranked {
        if out.len() >= MAX_FOLLOW_UPS {
            break;
        }
        if ok || GameState::would_accept_on(g, a.clone()) {
            out.push(a);
        }
    }
    out
}

/// Could `action` still be taken later this turn cycle at instant speed?
///
/// Only spells whose card is an Instant or has Flash: everything else is
/// sorcery-timed, so "wait" means "wait a whole turn", which is a very
/// different trade from "wait until their end step". Deliberately narrow —
/// it gates *not acting*, and a false positive there costs a real play.
fn castable_at_instant_speed(state: &GameState, seat: usize, action: &GameAction) -> bool {
    use crate::card::{CardType, Keyword};
    let card_id = match action {
        GameAction::CastSpell { card_id, .. } => *card_id,
        _ => return false,
    };
    let Some(card) = state.players[seat].hand.iter().find(|c| c.id == card_id) else {
        return false;
    };
    card.definition.card_types.contains(&CardType::Instant)
        || card.definition.keywords.has_kw(&Keyword::Flash)
}

/// Does `action` achieve anything *this turn*, ignoring bodies that can't
/// attack yet? See [`eval_material_summon_sick_blind`]. `true` when the
/// question can't be answered (the outcome probe bailed), so an
/// unevaluable line is never held back.
///
/// Under [`EvalWeights::combat_aware`] the comparison runs *through this
/// turn's combat*, which is what makes the question meaningful for
/// interaction rather than just for creatures: killing a blocker before
/// attacking is worth something now, killing it at the opponent's end step
/// is worth the same and costs less information. Only a simulation that
/// reaches combat damage can tell those apart — and this is the consumer
/// the combat-aware evaluator was missing when it measured neutral on its
/// own, because within a single main phase combat is otherwise identical
/// across every candidate and cancels out.
fn improves_this_turn(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    settled: Option<&GameState>,
    w: &EvalWeights,
) -> bool {
    // The baseline has to be measured the same way the outcome is. With
    // `combat_aware` the outcome runs through combat, so a raw pre-combat
    // baseline would be compared against a post-combat score and every
    // connecting attack would read as "this action improved things" —
    // making the gate fire almost never. Forge avoids this by routing both
    // sides through the same `getScoreForGameState`, which fast-forwards
    // combat itself.
    // Same clone-for-nothing as `score_settled_state`: a walk that skips leaves
    // `idle` byte-identical to `state`. An *incomplete* walk does not — the
    // probe deliberately scores the torn state — so only the skip case can
    // take the shortcut, which is what `combat_sim_skips` answers.
    let before = if w.combat_aware && !combat_sim_skips(state) {
        let mut idle = state.clone();
        let mut idle_fuel = 256u32;
        let _ = simulate_through_combat(&mut idle, &mut idle_fuel, w);
        eval_material_summon_sick_blind(&idle, seat, w)
    } else {
        eval_material_summon_sick_blind(state, seat, w)
    };
    // Same reuse as `evaluate_action_sequence`: the probe that validated this
    // action already ran it on a clone of `state`.
    let mut g = match settled {
        Some(g) => g.clone(),
        None => {
            let mut g = state.clone();
            if dry_run(&mut g, action.clone()).is_err() {
                return true;
            }
            g
        }
    };
    let mut fuel = 64u32;
    while !g.is_game_over() {
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            if dry_run(&mut g, GameAction::SubmitDecision(answer)).is_err() {
                return true;
            }
        } else if g.stack.is_empty() {
            break;
        } else if dry_run(&mut g, GameAction::PassPriority).is_err() {
            return true;
        }
        fuel = match fuel.checked_sub(1) {
            Some(f) => f,
            None => return true,
        };
    }
    if w.combat_aware {
        let mut combat_fuel = 256u32;
        // A torn simulation can't answer the question; don't hold on it.
        if simulate_through_combat(&mut g, &mut combat_fuel, w) == CombatSim::Incomplete {
            return true;
        }
    }
    eval_material_summon_sick_blind(&g, seat, w) > before
}

/// Final pick among the validated [`Finalist`]s: resolve each candidate's
/// outcome and take the best resulting state, static score breaking ties and
/// ordering candidates whose outcome probe bailed. A lone finalist skips the
/// outcome clones entirely. The winner comes back whole, so the summon-sick
/// gate downstream inherits its `settled` state too.
fn pick_by_outcome(
    state: &GameState,
    seat: usize,
    finalists: Vec<Finalist>,
    w: &EvalWeights,
) -> Option<Finalist> {
    if finalists.len() <= 1 {
        return finalists.into_iter().next();
    }
    let baseline = eval_material(state, seat, w);
    let evd: Vec<(i32, Finalist)> = finalists
        .into_iter()
        .map(|f| {
            // Known-temporary casts (bounce, until-EOT stat changes) are
            // pinned to the baseline: the post-resolution snapshot can't
            // see the effect reversing, so evaluating it would sell a
            // bounce as removal. They win only on static score against
            // other no-eval-gain lines.
            let ev = if action_outcome_is_temporary(state, &f.action) {
                baseline
            } else {
                evaluate_action_outcome(state, seat, &f.action, f.settled.as_deref(), w)
                    .unwrap_or(baseline)
            };
            (ev, f)
        })
        .collect();
    // Actor-side exploration: sample finalists by outcome score. The
    // static score stays out of the softmax — it's a tiebreak, not a
    // second opinion at temperature.
    if let Some(t) = sampling_temp(state.turn_number) {
        let ws: Vec<i32> = evd.iter().map(|e| e.0).collect();
        let i = sample_scored_index(&ws, t);
        capture_decision(state, seat, &evd, i);
        return evd.into_iter().nth(i).map(|(_, f)| f);
    }
    let best = evd
        .iter()
        .enumerate()
        .max_by_key(|(_, (ev, f))| (*ev, f.score))
        .map(|(i, _)| i)?;
    capture_decision(state, seat, &evd, best);
    evd.into_iter().nth(best).map(|(_, f)| f)
}

/// Feed the finalist set and the pick to [`decision_capture`].
///
/// Hooked here rather than at the enumerator because this is the point
/// where a *choice* is actually made: everything upstream is filtering,
/// and the finalists are the set the bot genuinely weighed. That does
/// mean the recorded candidate set is the shortlist (`EVAL_TOP`) rather
/// than every legal action — which is the same convention AlphaZero uses
/// when it records the search's action set rather than the rules'.
fn capture_decision(state: &GameState, seat: usize, evd: &[(i32, Finalist)], chosen: usize) {
    if !super::decision_capture::enabled() {
        return;
    }
    let actions: Vec<GameAction> = evd.iter().map(|(_, f)| f.action.clone()).collect();
    super::decision_capture::maybe(state, seat, &actions, chosen);
}

/// True when `e`'s tree contains a leaf whose apparent value REVERSES
/// after the turn: an until-end-of-turn stat change, or a bounce of a
/// battlefield permanent to hand (the permanent comes back next turn).
/// The outcome evaluation snapshots the state right after resolution, so
/// these leaves read as permanent gains — a bounced 4-drop looked like
/// Doom Blade (+3×value) and a "base P/T 5/5 until end of turn" like a
/// real +18 material swing, and the bot burned Proctor's Gaze / Quandrix
/// Charm on them at sorcery speed for nothing. Graveyard/exile-to-hand
/// moves (Regrowth) are real card advantage and are NOT temporary.
fn contains_temporary_leaf(e: &Effect) -> bool {
    use crate::effect::{Duration, ZoneDest};
    match e {
        Effect::PumpPT { duration: Duration::EndOfTurn | Duration::EndOfCombat, .. }
        | Effect::SetBasePT { duration: Duration::EndOfTurn | Duration::EndOfCombat, .. }
        | Effect::SwitchPT { duration: Duration::EndOfTurn | Duration::EndOfCombat, .. } => true,
        Effect::Move { what, to: ZoneDest::Hand(_) } => {
            // A bounce of a battlefield object; an off-board (graveyard /
            // exile) retrieval keeps the card — permanent value.
            match what {
                crate::effect::Selector::TargetFiltered { filter, .. } => {
                    !filter.mentions_offboard_zone()
                }
                _ => true,
            }
        }
        Effect::Seq(v) => v.iter().any(contains_temporary_leaf),
        Effect::If { then, else_, .. } => {
            contains_temporary_leaf(then) || contains_temporary_leaf(else_)
        }
        Effect::MayDo { body, .. } => contains_temporary_leaf(body),
        Effect::ApplyToTargets { effect, .. } => contains_temporary_leaf(effect),
        _ => false,
    }
}

/// True when `action` is a cast whose (mode-resolved) effect contains a
/// temporary leaf — such candidates skip the outcome evaluation (see
/// [`contains_temporary_leaf`]) and compete on static score alone.
fn action_outcome_is_temporary(state: &GameState, action: &GameAction) -> bool {
    let (card_id, mode) = match action {
        GameAction::CastSpell { card_id, mode, .. } => (*card_id, *mode),
        _ => return false,
    };
    let Some(card) = state.find_card_anywhere(card_id) else { return false };
    contains_temporary_leaf(mode_branch(&card.definition.effect, mode))
}

/// A pure temporary-pump instant aimed at a target creature (Giant
/// Growth, Infuriate): the whole effect tree is target pumps with an
/// end-of-turn/combat duration. Anything with riders (draw, damage,
/// counters, keyword grants) stays castable on the normal schedule.
fn is_combat_trick(def: &CardDefinition) -> bool {
    use crate::card::CardType;
    use crate::effect::{Duration, Selector};
    if !def.card_types.contains(&CardType::Instant) {
        return false;
    }
    fn all_temp_pumps(e: &Effect) -> bool {
        match e {
            Effect::PumpPT {
                what: Selector::Target(_) | Selector::TargetFiltered { .. },
                duration: Duration::EndOfTurn | Duration::EndOfCombat,
                ..
            } => true,
            Effect::Seq(v) => !v.is_empty() && v.iter().all(all_temp_pumps),
            _ => false,
        }
    }
    all_temp_pumps(&def.effect)
}

/// After blocks are in: cast a held pump trick when it flips a fight our
/// creature is currently losing — it dies to its opposite number and the
/// pump saves it, or it fails to kill and the pump finishes the job.
/// Covers both sides of combat (our blocked attacker on our turn, our
/// blocker on theirs). Constant pumps only; dynamic amounts are skipped
/// rather than mis-valued.
fn pick_combat_trick(state: &GameState, seat: usize, w: &EvalWeights) -> Option<Picked> {
    use crate::effect::{Duration, Selector, Value};
    fn pump_amounts(e: &Effect) -> Option<(i32, i32)> {
        match e {
            Effect::PumpPT {
                what: Selector::Target(_) | Selector::TargetFiltered { .. },
                power: Value::Const(p),
                toughness: Value::Const(t),
                duration: Duration::EndOfTurn | Duration::EndOfCombat,
            } => Some((*p, *t)),
            Effect::Seq(v) => {
                let mut acc: Option<(i32, i32)> = None;
                for e in v {
                    let (p, t) = pump_amounts(e)?;
                    let (ap, at) = acc.unwrap_or((0, 0));
                    acc = Some((ap + p, at + t));
                }
                acc
            }
            _ => None,
        }
    }
    let have_mana = SweepMana::new(state, seat);
    let tricks: Vec<(CardId, i32, i32)> = state.players[seat]
        .hand
        .iter()
        .filter(|c| is_combat_trick(&c.definition))
        .filter(|c| can_afford_in_state_with(state, seat, c, w, &have_mana))
        .filter_map(|c| pump_amounts(&c.definition.effect).map(|(p, t)| (c.id, p, t)))
        .collect();
    if tricks.is_empty() {
        return None;
    }
    let computed_pt = |id: CardId| -> Option<(i32, i32)> {
        let cp = state.computed_permanent(id);
        let raw = state.battlefield_find(id)?;
        Some(match cp {
            Some(cp) => (cp.power, cp.toughness),
            None => (raw.power(), raw.toughness()),
        })
    };
    for (blocker, attacker) in state.block_map_snapshot() {
        let (Some(b), Some(a)) =
            (state.battlefield_find(blocker), state.battlefield_find(attacker))
        else {
            continue;
        };
        let (our_id, their_id) = if a.controller == seat && !state.same_team(b.controller, seat) {
            (attacker, blocker)
        } else if b.controller == seat && !state.same_team(a.controller, seat) {
            (blocker, attacker)
        } else {
            continue;
        };
        let (Some((op, ot)), Some((tp, tt))) = (computed_pt(our_id), computed_pt(their_id))
        else {
            continue;
        };
        let dies = tp >= ot;
        let kills = op >= tt;
        if !dies && kills {
            continue; // already winning this fight
        }
        for &(cid, p, t) in &tricks {
            let saves = dies && ot + t > tp;
            let now_kills = !kills && op + p >= tt;
            if !(saves || now_kills) {
                continue;
            }
            let action = GameAction::CastSpell {
                card_id: cid,
                target: Some(Target::Permanent(our_id)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            };
            if let Some(next) = state.accept(action.clone()) {
                return Some(Picked::Probed(action, Box::new(next)));
            }
        }
    }
    None
}

fn score_candidate(state: &GameState, seat: usize, action: &GameAction, w: &EvalWeights) -> i32 {
    use crate::card::CardType;
    // (source card, slot-0 target, variant bonus, extra mana sunk in).
    let (card_id, target, variant_bonus, extra_mana) = match action {
        GameAction::CastSpell { card_id, target, x_value, .. } => {
            (*card_id, target.clone(), 0, x_value.unwrap_or(0))
        }
        GameAction::CastSpellBack { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastSpellDelve { card_id, target, x_value, .. } => {
            (*card_id, target.clone(), 3, x_value.unwrap_or(0))
        }
        GameAction::CastGift { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellSpree { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastSpellConspire { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellKicked { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellKickers { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellMultikicked { card_id, target, times, .. } => {
            (*card_id, target.clone(), 3, *times)
        }
        GameAction::CastBestow { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastAdventure { card_id, target, .. }
        | GameAction::CastOmen { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastPrototype { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastSplitRight { card_id, target, .. }
        | GameAction::CastAftermath { card_id, target, .. }
        | GameAction::CastFlashback { card_id, target, .. }
        | GameAction::CastMayhem { card_id, target, .. }
        | GameAction::CastHarmonize { card_id, target, .. }
        | GameAction::CastDisturb { card_id, target, .. }
        | GameAction::CastSpellAlternative { card_id, target, .. } => {
            (*card_id, target.clone(), 0, 0)
        }
        GameAction::CastAdventureCreature { card_id, target, .. }
        | GameAction::CastPlotted { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::ActivateAbility { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        // Loyalty activations: the target term is what differentiates them
        // (a −3 destroy at a 5-drop should out-score "+2: gain 3"); the
        // outcome eval in `pick_loyalty_ability` is the primary judge.
        GameAction::ActivateLoyaltyAbility { card_id, target, .. } => {
            (*card_id, target.clone(), 0, 0)
        }
        GameAction::CastPrepareSpell { creature_id, target, .. } => {
            (*creature_id, target.clone(), 0, 0)
        }
        // Fallback lines (face-down morphs, discard-activated) only appear
        // when nothing else is castable, so their exact rank is moot.
        _ => return 0,
    };

    let mut score = 0i32;
    let mut damage: Option<i32> = None;
    if let Some(card) = state.find_card_anywhere(card_id) {
        // Score the face actually being cast when it isn't the front:
        // MDFC backs for back-face casts, and the inset spell for
        // prepare-casts — scoring the latter by the CREATURE valued
        // "cast draw-3 for {U}" like deploying a 5/5 body, so the bot
        // fired every inset spell at the first opportunity.
        let def = match (action, card.definition.back_face.as_deref()) {
            (GameAction::CastSpellBack { .. } | GameAction::CastDisturb { .. }, Some(back)) => back,
            _ => &card.definition,
        };
        let def = match (action, card.definition.prepare_spell.as_deref()) {
            (GameAction::CastPrepareSpell { .. }, Some(spell)) => spell,
            _ => def,
        };
        // These terms are raw card stats; `permanent_value` below is on the
        // profile's scale, so lift them into the same units or a scaled
        // profile would drown the cast's own merits in the target's value.
        score += 2 * (def.cost.cmc() as i32 + extra_mana as i32) * w.unit;
        if def.card_types.contains(&CardType::Creature) {
            score += (def.power.max(0) + def.toughness.max(0)) * w.unit;
            score += (def.keywords.len() as i32).min(3) * w.unit;
        }
        damage = first_damage_amount(&def.effect, extra_mana);
    }

    // Unpreparing forfeits any "prepared creatures you control …" static
    // the bot has out (SOS Top of the Class); charge the cast for the
    // rider it strips.
    if matches!(action, GameAction::CastPrepareSpell { .. })
        && state.battlefield.iter().any(|c| {
            c.controller == seat && static_rewards_prepared(&c.definition)
        })
    {
        score -= 4 * w.unit;
    }

    match target {
        // Aimed at an opponent's permanent: removal / theft / lockdown —
        // worth what the target is worth. Aimed at our own: pump / aura /
        // equip, a small flat gain.
        Some(Target::Permanent(id)) => {
            match state.battlefield_find(id).map(|c| c.controller) {
                Some(ctrl) if ctrl != seat => {
                    let mut v = permanent_value(state, id, w);
                    // CR 702.21 — a ward tax is mana/life the cast sinks
                    // with no effect of its own; price it at the same
                    // 2-per-mana rate the cast's own cost earns above so
                    // an un-warded target of equal value, or a different
                    // spell entirely, wins the tie. Payability is the
                    // candidate gate's job (`ward_gate_ok`); this term
                    // only ranks survivors.
                    if let Some(tax) = ward_tax(state, id, seat) {
                        v -= 2 * ward_tax_burden(&tax) * w.unit;
                    }
                    // Damage spells only count as removal when they kill:
                    // chip damage at a too-big creature keeps a quarter of
                    // the value, and overkill (a huge X at a small body)
                    // pays back the wasted points so Shock-the-2/2 beats
                    // Fireball-for-8-the-2/2.
                    if let (Some(dmg), Some(cp)) = (damage, state.computed_permanent(id))
                        && cp.card_types().contains(&CardType::Creature)
                    {
                        if dmg < cp.toughness {
                            v /= 4;
                        } else {
                            // Overkill is charged in scaled points -- `dmg` and
                            // `toughness` are raw, `v` is not.
                            v -= (dmg - cp.toughness).max(0) * w.unit;
                        }
                    }
                    // A bounce is tempo, not removal — the permanent comes
                    // back next turn. A third of the value keeps target
                    // selection sane without treating it as a kill.
                    if let GameAction::CastSpell { mode, .. } = action
                        && let Some(card) = state.find_card_anywhere(card_id)
                        && contains_temporary_leaf(mode_branch(&card.definition.effect, *mode))
                    {
                        v /= 3;
                    }
                    score += v;
                }
                Some(_) => score += 2 * w.unit,
                None => {}
            }
        }
        // Face damage / discard at an opponent beats a self-aimed cantrip.
        Some(Target::Player(p)) => score += if p != seat { 4 * w.unit } else { w.unit },
        _ => {}
    }

    score + variant_bonus * w.unit
}


// ── Accessors for the Monte Carlo bot ────────────────────────────────────
//
// `mcts` needs the same candidate enumeration and leaf evaluation the
// heuristic bot uses, so the two are compared on identical inputs and any
// ladder difference is the *search*, not a second opinion about what is
// castable or what a board is worth.

/// Alternative primary targets for an already-built cast arm — the
/// `target_arms` menu (see [`EvalWeights::target_arms`]).
///
/// The candidate generators call `auto_targets_for_effect_all_slots` once
/// and bake its answer into the action, so the search's menu contains one
/// targeting per spell. When the auto-targeter is wrong the correct play
/// is not a low-scoring arm the search rejects — it is absent, and no
/// valuation can reach it. This re-enumerates slot 0's legal candidates
/// and returns up to `max` variants of the same cast.
///
/// Only candidates the *effect* wants are offered (per-slot polarity, so
/// a hostile slot considers the opponent's permanents and not ours),
/// biggest body first. An alternate on the wrong side is an arm the
/// search must spend rejecting, and arms are its scarcest resource.
///
/// This ranked "the opposite side from whatever was chosen" until
/// 2026-08-22, generalising from two failures where the baked-in pick was
/// our own permanent. When the baked-in pick is already correct — the
/// common case — "opposite side" means *our own board*, so the first of
/// two arms was spent offering the search a self-target. Observed in a
/// recorded game where the base pick was right but aimed at the smallest
/// enemy creature: the better target was the arm that got crowded out.
/// Only slot 0 is varied —
/// additional slots are the polarity classifier's job
/// (`prefers_friendly_target_for_slot`) and varying them combinatorially
/// would blow the arm budget the search is trying to protect.
fn target_arm_variants(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    max: usize,
) -> Vec<GameAction> {
    let GameAction::CastSpell { card_id, target: Some(Target::Permanent(chosen)), mode, .. } =
        action
    else {
        return Vec::new();
    };
    let Some(card) = state.players[seat].hand.iter().find(|c| c.id == *card_id) else {
        return Vec::new();
    };
    let eff = &card.definition.effect;
    let Some(req) = eff.target_filter_for_slot_in_mode_kicked(0, *mode, false) else {
        return Vec::new();
    };
    let prefer_friendly = eff.prefers_friendly_target_for_slot(0, *mode);
    let mut alts: Vec<(u8, i32, CardId)> = state
        .battlefield
        .iter()
        .filter(|c| c.id != *chosen)
        .filter(|c| {
            let t = Target::Permanent(c.id);
            state.evaluate_requirement_static(req, &t, seat, None)
                && state.check_target_legality(&t, seat).is_ok()
        })
        // Only the side this slot actually wants. An alternate the effect
        // does not want is an arm the search has to spend rejecting it,
        // and the arm budget is the scarcest thing the search has.
        .filter(|c| (c.controller == seat) == prefer_friendly)
        .map(|c| {
            let value = state
                .computed_permanent(c.id)
                .map(|cp| cp.power + cp.toughness)
                .unwrap_or(c.definition.power + c.definition.toughness);
            (0u8, -value, c.id)
        })
        .collect();
    alts.sort();
    alts.truncate(max);
    alts.into_iter()
        .map(|(_, _, id)| {
            let mut v = action.clone();
            if let GameAction::CastSpell { target, .. } = &mut v {
                *target = Some(Target::Permanent(id));
            }
            v
        })
        .collect()
}

/// The main-phase plays worth searching from `state`, validated, each
/// with its heuristic score (in `w.unit`s). The scores were always
/// computed here to rank the arm cap; returning them lets the search
/// seed root priors from the same opinion instead of starting uniform.
pub(crate) fn main_phase_candidates_for_mcts(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
) -> Vec<(GameAction, i32)> {
    let mut ranked: Vec<(i32, GameAction, bool)> = cast_candidates(state, seat, w, None)
        .into_iter()
        .map(|(a, ok)| (score_candidate(state, seat, &a, w), a, ok))
        .collect();
    ranked.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
    // Cap the arms. Every candidate costs at least one rollout to seed, so
    // a wide root eats the whole budget before UCB1 gets to allocate any of
    // it; better to search the plausible plays properly than every play
    // badly.
    const MAX_ARMS: usize = 6;
    // Reserve material for the prepared cast before the cap consumes the
    // ranking — see `prepare_arm` below.
    let best_prepared: Option<(GameAction, i32)> = w
        .prepare_arm
        .then(|| {
            ranked
                .iter()
                .find(|(_, a, _)| matches!(a, GameAction::CastPrepareSpell { .. }))
                .map(|(s, a, _)| (a.clone(), *s))
        })
        .flatten();
    let mut out = Vec::with_capacity(MAX_ARMS);
    for (s, a, ok) in ranked {
        if out.len() >= MAX_ARMS {
            break;
        }
        if ok || GameState::would_accept_on(state, a.clone()) {
            out.push((a, s));
        }
    }
    // Reserve an arm for the banked inset spell (flag): a rare,
    // high-value class the six-arm cap can crowd out — two prepared
    // Ancestral Recalls sat unfired through a recorded loss. Costs the
    // weakest arm, and only when the class exists and missed the cut.
    if let Some((a, sc)) = best_prepared
        && !out.iter().any(|(c, _)| matches!(c, GameAction::CastPrepareSpell { .. }))
        && GameState::would_accept_on(state, a.clone())
    {
        if out.len() >= MAX_ARMS {
            out.pop();
        }
        out.push((a, sc));
    }
    // Alternative targetings of the best targeted cast (flag). Two extra
    // arms at most, and only for the single highest-scoring targeted
    // cast: the arm budget is the search's scarcest resource (round 42 —
    // iterations are the only lever that pays), so this buys menu
    // coverage of the one decision most likely to be miscast rather than
    // fanning every spell out over its target set. Scored a hair under
    // the arm they vary, so a tie leaves the heuristic's pick in front
    // and the sims have to earn the swap.
    if w.target_arms
        && let Some((base, base_score)) = out
            .iter()
            .find(|(a, _)| {
                matches!(a, GameAction::CastSpell { target: Some(Target::Permanent(_)), .. })
            })
            .map(|(a, s)| (a.clone(), *s))
    {
        for v in target_arm_variants(state, seat, &base, 2) {
            if out.len() >= MAX_ARMS + 2 {
                break;
            }
            if GameState::would_accept_on(state, v.clone()) {
                out.push((v, base_score - 1));
            }
        }
    }

    // A land drop is a real option and is enumerated separately.
    // `score_candidate` has no opinion on lands; two units — a solid
    // default play, ahead of a marginal cast, behind a strong one.
    if state.can_player_play_land(seat)
        && let Some(land) = pick_land_to_play(state, seat, w)
    {
        let action = GameAction::PlayLand(land);
        if GameState::would_accept_on(state, action.clone()) {
            out.push((action, 2 * w.unit));
        }
    }
    out
}

/// The heuristic bot's board evaluation, for scoring a rollout leaf.
pub(crate) fn eval_material_for_mcts(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    eval_material(state, seat, w)
}

/// Kill-the-biggest-attacker-first, for the Monte Carlo bot's block arm:
/// removal cast before blocks shrinks the combat the blocks then answer,
/// and the searched declaration must see the same pre-shrunk board the
/// heuristic's does.
pub(crate) fn defensive_removal_for_mcts(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
) -> Option<GameAction> {
    pick_defensive_removal(state, seat, w)
}

/// The heuristic board evaluation, exposed for measurement.
///
/// The value net is only worth its inference cost if it predicts the
/// winner *better than this does*. Four gate rounds compared the two by
/// playing thousands of games, which answers "is the bot stronger"
/// expensively and says nothing about why; comparing their predictions on
/// the same positions is minutes of compute and separates "the net has
/// not learned" from "the net has learned and the integration wastes it".
/// See `selfplay_train --calibrate`.
pub fn eval_material_public(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    eval_material(state, seat, w)
}

#[cfg(test)]
mod tests {
    use super::ply_blend_factor;

    /// The taper is the hypothesis: full net voice through the opening
    /// where its measured edge lives, silence by the turn the stratified
    /// calibrations say both evaluations converge.
    #[test]
    fn ply_blend_factor_is_full_early_and_zero_late() {
        assert_eq!(ply_blend_factor(0), 1.0);
        assert_eq!(ply_blend_factor(5), 1.0);
        let mid = ply_blend_factor(8);
        assert!(mid > 0.0 && mid < 1.0, "{mid}");
        assert_eq!(ply_blend_factor(12), 0.0);
        assert_eq!(ply_blend_factor(30), 0.0);
    }

    /// End-to-end smoke test of the property determinization is for: a
    /// determinized search must not depend on the hidden arrangement,
    /// and permuting a library changes nothing any player is allowed to
    /// know.
    ///
    /// **This test does not discriminate on its own, and the honest note
    /// matters more than the assertion.** Removing the canonicalising
    /// sort from `determinize_hidden` leaves it passing: on a position
    /// this simple the bot reaches the same action whatever the redeal
    /// guesses, so the decision is not sensitive to the thing under
    /// test. Kept as a cheap guard against gross regressions; the actual
    /// guard is `redeal_depends_only_on_the_information_set`, which was
    /// verified to fail when the sort is removed.
    #[test]
    fn determinized_decisions_ignore_library_order() {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        let build = || {
            let mut g = two_player_game();
            for seat in 0..2 {
                for _ in 0..12 {
                    let id = g.add_card_to_hand(seat, catalog::grizzly_bears());
                    if let Some(pos) = g.players[seat].hand.iter().position(|c| c.id == id) {
                        let card = g.players[seat].hand.remove(pos);
                        g.players[seat].library.push(card);
                    }
                }
                for _ in 0..3 {
                    g.add_card_to_hand(seat, catalog::grizzly_bears());
                }
            }
            let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(id);
            g
        };

        let decide = |g: GameState| {
            let mut bot = HeuristicBot::with_weights(EvalWeights::determinized());
            format!("{:?}", bot.next_action(&g, 0))
        };

        let base = build();
        let first = decide(base.clone());

        let mut permuted = base.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(99);
        for p in 0..2 {
            permuted.players[p].library.shuffle(&mut rng);
        }
        let second = decide(permuted);

        assert_eq!(
            first, second,
            "a determinized search changed its mind when only hidden order moved"
        );
    }

    /// And the redeal itself is a function of the information set, not of
    /// how the hidden cards happened to be arranged — the mechanism the
    /// test above rests on, checked directly so a regression names its
    /// own cause rather than surfacing as a mysterious decision flip.
    #[test]
    fn redeal_depends_only_on_the_information_set() {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        let mut g = two_player_game();
        for _ in 0..10 {
            let id = g.add_card_to_hand(1, catalog::grizzly_bears());
            if let Some(pos) = g.players[1].hand.iter().position(|c| c.id == id) {
                let card = g.players[1].hand.remove(pos);
                g.players[1].library.push(card);
            }
        }
        for _ in 0..3 {
            g.add_card_to_hand(1, catalog::grizzly_bears());
        }

        let mut a = g.clone();
        determinize_hidden(&mut a, 0, 0);

        let mut b = g.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(5);
        b.players[1].library.shuffle(&mut rng);
        determinize_hidden(&mut b, 0, 0);

        let ids = |g: &GameState| -> Vec<u32> {
            g.players[1].library.iter().map(|c| c.id.0).collect()
        };
        assert_eq!(ids(&a), ids(&b), "the redeal read the hidden arrangement");
        assert_eq!(
            a.players[1].hand.len(),
            b.players[1].hand.len(),
            "hand size is public and must be preserved"
        );
    }

    use super::*;
    use crate::catalog;
    use crate::game::GameState;
    use crate::game::TriggerPush;
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    fn body_card(name: &'static str, body: Effect) -> CardDefinition {
        use crate::card::{CardType, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec};
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "you may".to_string(),
                    body: Box::new(body),
                },
            }],
            ..Default::default()
        }
    }

    /// A redeal must preserve everything the searching seat can legally
    /// see — hand sizes, the battlefield, both graveyards, and each
    /// player's total card count — while replacing what it can't.
    #[test]
    fn determinize_preserves_public_information() {
        let mut g = two_player_game();
        for _ in 0..30 {
            g.add_card_to_library(1, catalog::forest());
        }
        for _ in 0..5 {
            g.add_card_to_library(1, catalog::shivan_dragon());
        }
        for _ in 0..20 {
            g.add_card_to_library(0, catalog::island());
        }
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::lightning_bolt());
        }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::serra_angel());
        let before = (
            g.players[1].hand.len(),
            g.players[1].library.len() + g.players[1].hand.len(),
            g.players[0].library.len() + g.players[0].hand.len(),
            g.battlefield.len(),
        );

        let mut d = g.clone();
        determinize_hidden(&mut d, 0, 1);

        assert_eq!(d.players[1].hand.len(), before.0, "opponent hand size is public");
        assert_eq!(d.players[1].library.len() + d.players[1].hand.len(), before.1);
        assert_eq!(d.players[0].library.len() + d.players[0].hand.len(), before.2);
        assert_eq!(d.battlefield.len(), before.3, "the battlefield is public");
        // Our own hand is ours to see and must survive the redeal intact.
        assert_eq!(d.players[0].hand.len(), 1);
        assert_eq!(d.players[0].hand[0].definition.name, "Grizzly Bears");
    }

    /// The belief-weighted redeal (round 39): a strong hold-belief must
    /// dominate which cards land in the redealt hand, public information
    /// must survive exactly as in the uniform path, and the redeal must
    /// stay a function of the information set — same salt and belief,
    /// same hand.
    #[test]
    fn belief_redeal_respects_the_belief_and_preserves_public_information() {
        let mut g = two_player_game();
        for _ in 0..30 {
            g.add_card_to_library(1, catalog::forest());
        }
        for _ in 0..5 {
            g.add_card_to_library(1, catalog::island());
        }
        for _ in 0..5 {
            g.add_card_to_hand(1, catalog::forest());
        }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        let vocab = crate::server::net_eval::vocab();
        let island = vocab.index_of("Island") as usize;
        let forest = vocab.index_of("Forest") as usize;
        assert!(island != 0 && forest != 0, "the test needs in-vocab names");
        let mut belief = vec![0.5f32; vocab.size()];
        belief[island] = 0.95;
        belief[forest] = 0.05;

        let mut islands = 0;
        for salt in 0..10u64 {
            let mut d = g.clone();
            determinize_hidden_belief(&mut d, 0, salt, &belief);
            assert_eq!(d.players[1].hand.len(), 5, "hand size is public");
            assert_eq!(d.players[1].hand.len() + d.players[1].library.len(), 40);
            assert_eq!(d.players[0].hand[0].definition.name, "Grizzly Bears");
            islands +=
                d.players[1].hand.iter().filter(|c| c.definition.name == "Island").count();
        }
        // All five Islands carry 19:1 hold-odds against 0.05-odds
        // Forests; the redealt five-card hand should be nearly all of
        // them, every time.
        assert!(islands >= 35, "belief must dominate the redeal: {islands}/50 islands");

        let mut a = g.clone();
        let mut b = g.clone();
        determinize_hidden_belief(&mut a, 0, 7, &belief);
        determinize_hidden_belief(&mut b, 0, 7, &belief);
        let names = |g: &GameState| {
            g.players[1].hand.iter().map(|c| c.definition.name).collect::<Vec<_>>()
        };
        assert_eq!(names(&a), names(&b), "same information set, same redeal");

        // A neutral belief must still resample rather than freeze: the
        // sampler with uniform odds is a (differently keyed) shuffle.
        let mut n = g.clone();
        determinize_hidden_belief(&mut n, 0, 11, &vec![0.5f32; vocab.size()]);
        assert_eq!(n.players[1].hand.len(), 5);
    }

    /// The point of the redeal: the opponent's hand is *resampled* from
    /// their unseen cards, so a search can no longer plan around the
    /// specific card they are holding. With four Bolts in hand and 35
    /// other cards behind them, keeping all four is vanishingly unlikely.
    #[test]
    fn determinize_resamples_the_opponent_hand() {
        let mut g = two_player_game();
        for _ in 0..35 {
            g.add_card_to_library(1, catalog::forest());
        }
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::lightning_bolt());
        }
        let mut changed = 0;
        for salt in 0..8 {
            let mut d = g.clone();
            determinize_hidden(&mut d, 0, salt);
            let bolts = d.players[1]
                .hand
                .iter()
                .filter(|c| c.definition.name == "Lightning Bolt")
                .count();
            if bolts < 4 {
                changed += 1;
            }
        }
        assert!(changed >= 7, "expected the redeal to move the hand, changed {changed}/8");
    }

    /// The bot names the creature type it controls the most of (not the stock
    /// AutoDecider "Demon"), so tribal chosen-type payoffs are useful under bot
    /// play.
    #[test]
    fn bot_names_its_most_common_creature_type() {
        use crate::card::CreatureType;
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // Llanowar Elves and Elvish Clancaller are both printed *Elf Druid*,
        // so a board of those two alone ties Elf against Druid. The third
        // Elf breaks it — and the tie is the reason this setup is spelled
        // out rather than "two Elves and a Bear".
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // Bear
        g.add_card_to_battlefield(0, catalog::llanowar_elves()); // Elf Druid
        g.add_card_to_battlefield(0, catalog::elvish_clancaller()); // Elf Druid
        g.add_card_to_battlefield(0, catalog::elvish_mystic()); // Elf Druid
        g.add_card_to_battlefield(0, catalog::devoted_hero()); // Elf Soldier
        let ans = decide_creature_type(&g, 0, &[]);
        assert!(matches!(ans, DecisionAnswer::CreatureType(CreatureType::Elf)),
            "four Elves against three Druids and a Bear → names Elf, got {ans:?}");
    }

    #[test]
    fn bot_takes_beneficial_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Upside", Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        );
        assert!(optional_trigger_beneficial(&g, id, "you may"),
            "a pure-upside 'you may draw' is taken by the bot");
    }

    /// The bot pays Offspring (CR 702.166) when it can afford it — the chosen
    /// main-phase cast is the kicked variant, not the plain cast.
    #[test]
    fn bot_pays_offspring_when_affordable() {
        use crate::mana::Color;
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let recruit = g.add_card_to_hand(0, catalog::pawpatch_recruit()); // {G}, Offspring {2}
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(action, GameAction::CastSpellKicked { card_id, .. } if card_id == recruit),
            "bot cast Pawpatch Recruit with Offspring paid, got {action:?}"
        );
    }

    /// The bot promises a gift (CR 702.165) when the gifted line is the point
    /// of the card — Scrapshooter's ETB destroy only fires on a promised gift,
    /// so the chosen cast must be `CastGift`, not a plain `CastSpell`.
    #[test]
    fn bot_promises_gift_for_scrapshooter() {
        use crate::mana::Color;
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let scrap = g.add_card_to_hand(0, catalog::scrapshooter()); // {1}{G}{G}
        g.add_card_to_battlefield(1, catalog::sol_ring()); // a legal ETB destroy target
        g.add_card_to_library(1, catalog::forest()); // the gift draw
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(action, GameAction::CastGift { card_id, .. } if card_id == scrap),
            "bot promised Scrapshooter's gift, got {action:?}"
        );
    }

    #[test]
    fn bot_declines_optional_trigger_that_sacrifices_itself() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Downside", Effect::SacrificeSource),
        );
        assert!(!optional_trigger_beneficial(&g, id, "you may"),
            "a 'you may sacrifice this' rider is a self-cost the bot declines");
    }

    /// A planeswalker whose highest-loyalty ability needs a target that
    /// doesn't exist must not stop the bot from activating a lower targetless
    /// ability (regression: the `?` on `auto_target_for_effect` used to bail
    /// out of every ability and planeswalker).
    #[test]
    fn bot_skips_untargetable_loyalty_ability_for_a_usable_one() {
        use crate::card::{CardType, LoyaltyAbility};
        use crate::effect::shortcut::target_filtered;
        use crate::card::SelectionRequirement;
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let pw = CardDefinition {
            name: "Test Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 3,
            loyalty_abilities: vec![
                // Highest loyalty, but needs a creature target (none exist).
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 2,
                    effect: Effect::DealDamage {
                        to: target_filtered(SelectionRequirement::Creature),
                        amount: Value::Const(2),
                    },
                },
                // Lower loyalty, no target — the bot should fall through here.
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 1,
                    effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                },
            ],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, pw);
        g.add_card_to_library(0, catalog::island());
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("bot finds the targetless +1");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(ability_index, 1, "picked the targetless draw, not the dead burn");
            }
            _ => panic!("expected a loyalty activation"),
        }
    }

    /// Loyalty abilities are picked by OUTCOME, not plus-first: Professor
    /// Dellian Fel with an opposing 5/5 on the board fires "−3: destroy
    /// target creature" instead of "+2: you gain 3 life" (the old
    /// cost-ordered walk never pressed a minus, piloting the pool's best
    /// bomb as a lifegain trinket).
    #[test]
    fn bot_walker_presses_removal_over_lifegain() {
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("walker activates something");
        match action {
            GameAction::ActivateLoyaltyAbility {
                card_id, ability_index, target, ..
            } => {
                assert_eq!(card_id, pw);
                assert_eq!(ability_index, 2, "the −3 destroy, not the +2 lifegain");
                assert_eq!(
                    target,
                    Some(crate::game::Target::Permanent(dragon)),
                    "aimed at the opposing dragon",
                );
            }
            other => panic!("expected a loyalty activation, got {other:?}"),
        }
    }

    /// Known-temporary casts skip the outcome eval: with Quandrix Charm
    /// (whose mode 2 is "base P/T 5/5 until end of turn") and a real
    /// creature both castable, the bot develops instead of burning the
    /// Charm as a fake main-phase pump.
    #[test]
    fn bot_prefers_development_over_temp_buff() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // pump target
        let charm = g.add_card_to_hand(0, catalog::quandrix_charm());
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        // Second main: this tests which candidate wins the ranking, not
        // when it is cast. The default profile's summon-sick gate defers a
        // first-main creature, which is orthogonal to the point here.
        g.step = TurnStep::PostCombatMain;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(action, GameAction::CastSpell { card_id, .. } if card_id == bears),
            "cast the creature, not the Charm's temp buff, got {action:?}",
        );
        let _ = charm;
    }

    /// With the ultimate affordable AND lifegain to feed the emblem,
    /// the eval presses it: Dellian Fel at 7 loyalty with a Melancholic
    /// Poet and a lifelink body on board fires −6 (emblem priced by
    /// visible lifegain sources; loyalty spent is a resource, not a
    /// material loss).
    #[test]
    fn bot_walker_ults_when_the_deck_feeds_the_emblem() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == pw)
            .unwrap()
            .counters
            .insert(CounterType::Loyalty, 7);
        // Three visible lifegain sources: emblem value 2 + 6×3 = 20.
        g.add_card_to_battlefield(0, catalog::melancholic_poet());
        g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
        g.add_card_to_hand(0, catalog::melancholic_poet());
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("walker activates something");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, pw);
                assert_eq!(ability_index, 3, "the −6 emblem ultimate, not the +2 lifegain");
            }
            other => panic!("expected a loyalty activation, got {other:?}"),
        }
    }

    /// …and WITHOUT lifegain sources the emblem is nearly dead (2 < the
    /// +2's gain-3), so the walker holds the fort instead of ulting into
    /// nothing — the indiscriminate flat-price ult measurably HURT Fel.
    #[test]
    fn bot_walker_holds_ult_without_lifegain() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == pw)
            .unwrap()
            .counters
            .insert(CounterType::Loyalty, 7);
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("walker activates something");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, pw);
                assert_ne!(
                    ability_index, 3,
                    "no lifegain to feed the emblem — don't ult into nothing"
                );
            }
            other => panic!("expected a loyalty activation, got {other:?}"),
        }
    }

    /// The bot can activate a *statically-granted* loyalty ability (one the
    /// walker doesn't print itself), matching the engine's effective-list
    /// activation path.
    #[test]
    fn bot_activates_granted_loyalty_ability() {
        use crate::card::{CardType, LoyaltyAbility, StaticAbility};
        use crate::effect::{Selector, StaticEffect, Value};
        let mut g = two_player_game();
        // A walker with NO printed loyalty abilities.
        let pw = CardDefinition {
            name: "Blank Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 3,
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, pw);
        // A permanent that grants every planeswalker you control a +1 draw.
        let granter = CardDefinition {
            name: "Loyalty Font",
            card_types: vec![CardType::Artifact],
            static_abilities: vec![StaticAbility {
                description: "Planeswalkers you control have +1: draw a card.",
                effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                    abilities: vec![LoyaltyAbility {
                        x_cost: false,
                        loyalty_cost: 1,
                        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    }],
                },
            }],
            ..Default::default()
        };
        g.add_card_to_battlefield(0, granter);
        g.add_card_to_library(0, catalog::island());
        match pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("bot finds the granted ability") {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, id, "activated on the blank walker");
                assert_eq!(ability_index, 0, "the granted +1 is index 0");
            }
            _ => panic!("expected a loyalty activation"),
        }
    }

    /// The Wandering Emperor's +1 (a friendly +1/+1 buff) auto-targets the
    /// bot's OWN creature, never the opponent's — the targeting regression
    /// this test originally caught. (Which ability the walker fires is the
    /// outcome eval's call and pinned elsewhere; here we probe the +1's
    /// target choice directly.)
    #[test]
    fn bot_wandering_emperor_plus_one_targets_own_creature() {
        let mut g = two_player_game();
        let emp = g.add_card_to_battlefield(0, catalog::the_wandering_emperor());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let plus_one = &catalog::the_wandering_emperor().loyalty_abilities[0];
        let picked = g.auto_target_for_effect(&plus_one.effect, 0);
        assert_eq!(
            picked,
            Some(Target::Permanent(mine)),
            "the +1 buffs its own creature, not {theirs:?}",
        );
        let _ = emp;
    }

    #[test]
    fn bot_declines_self_costly_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Downside", Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
        );
        assert!(!optional_trigger_beneficial(&g, id, "you may"),
            "a 'you may lose 3 life' optional trigger is declined");
    }

    /// Self-directed damage / mill bodies are costs too — the bot declines a
    /// "you may have this deal 4 damage to you" optional trigger.
    #[test]
    fn bot_declines_self_damage_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let dmg = g.add_card_to_battlefield(
            0,
            body_card("SelfBurn", Effect::DealDamage { to: Selector::You, amount: Value::Const(4) }),
        );
        assert!(!optional_trigger_beneficial(&g, dmg, "you may"),
            "a 'you may deal 4 to you' optional trigger is declined");
        let mill = g.add_card_to_battlefield(
            0,
            body_card("SelfMill", Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
        );
        assert!(!optional_trigger_beneficial(&g, mill, "you may"),
            "a 'you may mill yourself 3' optional trigger is declined");
    }

    /// Blight (CR 701.68) shrinks the bot's own board, so a "may blight N for
    /// upside" optional trigger is declined.
    /// A `MayDiscard` reflexive whose payoff isn't self-costly (Toph's
    /// return-a-spell) is accepted by the bot — card filtering is upside.
    #[test]
    fn bot_takes_beneficial_maydiscard() {
        use crate::card::{CardType, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "Rummager",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::MayDiscard {
                    description: "discard to draw?".to_string(),
                    count: Value::ONE,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: None,
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        assert!(optional_trigger_beneficial(&g, id, "discard to draw?"),
            "a MayDiscard whose payoff is a draw is accepted");
    }

    #[test]
    fn bot_declines_blight_optional_trigger() {
        use crate::effect::Value;
        let mut g = two_player_game();
        let blighter = g.add_card_to_battlefield(
            0,
            body_card("Blighter", Effect::Blight { n: Value::Const(2) }),
        );
        assert!(!optional_trigger_beneficial(&g, blighter, "you may"),
            "a 'you may blight 2' optional trigger is declined");
    }

    /// `MayPay` shares the `OptionalTrigger` decision shape with `MayDo`, so
    /// the bot's self-cost screen must introspect it too: a "pay {1}: you lose
    /// 3 life" body is declined even though it's reachable only via MayPay.
    #[test]
    fn bot_declines_self_costly_maypay() {
        use crate::card::{CardType, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "PayDownside",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "you may pay".to_string(),
                    mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
                    body: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
                    else_: None,
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        assert!(!optional_trigger_beneficial(&g, id, "you may pay"),
            "a MayPay whose body costs the bot 3 life is declined");
    }

    /// Moving the source to exile/graveyard is a self-cost (decline); returning
    /// it to hand (Recover-style upside) is accepted.
    #[test]
    fn bot_screens_self_move_bodies() {
        use crate::effect::{PlayerRef, Selector, ZoneDest};
        let mut g = two_player_game();
        let exile_self = g.add_card_to_battlefield(
            0,
            body_card("ExileSelf", Effect::Move { what: Selector::This, to: ZoneDest::Exile }),
        );
        assert!(!optional_trigger_beneficial(&g, exile_self, "you may"),
            "'you may exile this' reads as a self-cost");
        let to_hand = g.add_card_to_battlefield(
            0,
            body_card("ToHand", Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        );
        assert!(optional_trigger_beneficial(&g, to_hand, "you may"),
            "returning self to hand is upside");
    }

    fn generic_spell(name: &'static str, cmc: u32) -> CardDefinition {
        use crate::card::CardType;
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            cost: crate::mana::cost(&[crate::mana::generic(cmc)]),
            ..Default::default()
        }
    }

    /// Self-discard heuristic pitches the priciest spell (least likely to be
    /// cast soon), not the head of the hand, when the bot isn't flooded.
    #[test]
    fn bot_self_discard_pitches_priciest_spell() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let pricey = g.add_card_to_hand(0, generic_spell("Pricey", 6));
        let cheap = g.add_card_to_hand(0, generic_spell("Cheap", 1));
        // Offer both; head dump would take `pricey` (first), but so should the
        // heuristic here — make the cheap card the head to prove it's a real
        // choice rather than a head dump.
        let hand = vec![
            (cheap, "Cheap".to_string()),
            (pricey, "Pricey".to_string()),
        ];
        let DecisionAnswer::Discard(ids) = decide_self_discard(&g, 0, &hand, 1) else {
            panic!("expected a Discard answer");
        };
        assert_eq!(ids, vec![pricey], "the most expensive spell is pitched");
    }

    /// When flooded (≥5 lands in play), a surplus land is pitched before a
    /// keepable cheap spell.
    #[test]
    fn bot_self_discard_pitches_surplus_land_when_flooded() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_battlefield(0, catalog::island());
        }
        let land = g.add_card_to_hand(0, catalog::island());
        let spell = g.add_card_to_hand(0, generic_spell("Cheap", 1));
        let hand = vec![
            (spell, "Cheap".to_string()),
            (land, "Island".to_string()),
        ];
        let DecisionAnswer::Discard(ids) = decide_self_discard(&g, 0, &hand, 1) else {
            panic!("expected a Discard answer");
        };
        assert_eq!(ids, vec![land], "a flooded bot pitches the surplus land");
    }

    /// A lethal constant-damage ping ability aims at an opposing planeswalker
    /// whose loyalty it can finish off.
    #[test]
    fn bot_pings_lethal_opposing_planeswalker() {
        let mut g = two_player_game();
        let tim = g.add_card_to_battlefield(0, catalog::prodigal_pyromancer()); // {T}: 1 dmg any target
        g.clear_sickness(tim);
        let walker = g.add_card_to_battlefield(1, catalog::vivien_reid());
        // Knock the walker down to 1 loyalty so a 1-damage ping is lethal.
        let inst = g.battlefield_find_mut(walker).unwrap();
        inst.counters.insert(crate::card::CounterType::Loyalty, 1);
        let action = pick_removal_ping(&g, 0).expect("bot should ping the walker");
        match action {
            GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. } => {
                assert_eq!(card_id, tim);
                assert_eq!(t, walker, "aimed at the 1-loyalty planeswalker");
            }
            other => panic!("unexpected action: {other:?}"),
        }
    }

    /// A mana rock's output has to count toward what the bot can cast.
    ///
    /// This used to assert that the bot *tapped* Sol Ring as its own
    /// action, back when it pre-tapped every source before deciding
    /// anything. It no longer does that (see the note in
    /// `main_phase_action_with`), so the assertion is now on the outcome
    /// that mattered all along: the rock's mana is what makes the spell
    /// affordable, and the engine's auto-tap spends it.
    #[test]
    fn bot_spends_mana_rock_output_on_a_spell() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let sol = g.add_card_to_battlefield(0, catalog::sol_ring());
        g.clear_sickness(sol);
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.clear_sickness(forest);
        let have = available_mana(&g, 0);
        assert_eq!(have.total, 3, "Sol Ring's two plus the Forest's one");
        assert!(have.colors.contains(crate::mana::Color::Green));

        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        assert!(
            matches!(action, GameAction::CastSpell { card_id, .. } if card_id == bear),
            "bot should cast the bear rather than pre-tapping anything, got {action:?}",
        );
    }

    /// The tap-out regression guard. The bot must not spend mana it has no
    /// use for: with an uncastable hand it should pass, leaving its lands
    /// untapped so they survive into the opponent's turn for instant-speed
    /// plays. Before this fix it tapped every land unconditionally and the
    /// pool was emptied at the phase boundary (CR 500.4).
    #[test]
    fn bot_leaves_mana_untapped_when_it_has_nothing_to_cast() {
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        // A hand card it cannot cast: wrong color, and no black source.
        g.add_card_to_hand(0, catalog::doom_blade());
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        assert!(
            matches!(action, GameAction::PassPriority),
            "bot should pass, not burn mana, got {action:?}",
        );
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && !c.tapped).count(),
            3,
            "all three lands stay untapped and available at instant speed",
        );
    }


    /// With spare mana and nothing better to do, the bot sinks it into War
    /// Balloon's fire-counter ability to progress toward animating it.
    #[test]
    fn bot_feeds_fire_counters_to_animate_war_balloon() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let wb = g.add_card_to_battlefield(0, catalog::war_balloon());
        // A Mountain pays the {1} fire-counter cost; nothing else to do.
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        g.clear_sickness(mtn);
        let mut bot = HeuristicBot::new();
        // Drive a few actions: tap the land for mana, then sink into the counter.
        let mut animated = false;
        for _ in 0..6 {
            let Some(action) = bot.next_action(&g, 0) else { break };
            if let GameAction::ActivateAbility { card_id, ability_index, .. } = &action
                && *card_id == wb
            {
                assert_eq!(*ability_index, 0, "the fire-counter ability");
                animated = true;
            }
            if g.perform_action(action).is_err() { break }
            crate::game::drain_stack(&mut g);
        }
        assert!(animated, "bot activated War Balloon's fire-counter sink");
        assert!(g.battlefield_find(wb).unwrap().counter_count(CounterType::Fire) >= 1,
            "a fire counter was added");
    }

    /// The bot spends surplus energy on a beneficial energy-payoff ability
    /// (Longtusk Cub's `{E}{E}{E}: +1/+1 counter`) once nothing better to do.
    #[test]
    fn bot_spends_energy_on_payoff_ability() {
        let mut g = two_player_game();
        let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
        g.clear_sickness(cub);
        g.players[0].energy = 3;
        let action = pick_energy_payoff(&g, 0).expect("bot should pay energy for the counter");
        match action {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, cub),
            _ => panic!("expected an activate-ability action"),
        }
        // With too little energy the bot leaves it alone.
        g.players[0].energy = 1;
        assert!(pick_energy_payoff(&g, 0).is_none(), "won't activate without enough energy");
    }

    /// When card-starved, the bot sinks spare mana into Bonders' Enclave's
    /// "{3}, {T}: Draw a card" — but only once its activation condition (a
    /// 4-power creature) is met.
    #[test]
    fn bot_draws_with_value_ability_when_card_starved() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::bonders_enclave());
        g.clear_sickness(land);
        g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
        g.players[0].mana_pool.add_colorless(3);
        // No 4-power creature → the draw ability's condition fails.
        assert!(pick_card_draw_ability(&g, 0).is_none(),
            "no draw without a 4-power creature");
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        match pick_card_draw_ability(&g, 0).expect("bot draws when card-starved") {
            GameAction::ActivateAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, land);
                assert_eq!(ability_index, 1, "the draw ability, not the mana ability");
            }
            _ => panic!("expected an activate-ability action"),
        }
        // A full hand → don't bother drawing.
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        assert!(pick_card_draw_ability(&g, 0).is_none(), "won't draw with a full hand");
    }

    /// The bot fires Frostwielder's `{T}: 1 damage` ping to kill a 1/1, but
    /// won't waste it when no opposing creature dies to it.
    #[test]
    fn bot_pings_a_killable_creature() {
        let mut g = two_player_game();
        let fw = g.add_card_to_battlefield(0, catalog::frostwielder());
        g.clear_sickness(fw);
        let frostling = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
        let action = pick_removal_ping(&g, 0).expect("bot pings the 1/1");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, fw);
                assert_eq!(target, Some(Target::Permanent(frostling)));
            }
            _ => panic!("expected an activate-ability action"),
        }
        // A 2/2 survives a 1-damage ping → the bot holds the ability.
        g.battlefield.retain(|c| c.id != frostling);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        assert!(pick_removal_ping(&g, 0).is_none(), "won't waste a ping on a survivor");
    }

    /// The bot fires a self-power ping ("{T}: deals damage equal to its power to
    /// target creature") to kill a foe whose toughness it can beat.
    #[test]
    fn bot_pings_with_self_power() {
        use crate::card::{ActivatedAbility, CardType};
        use crate::effect::{Selector, Value};
        let pinger = CardDefinition {
            name: "Self-Power Pinger",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: crate::card::SelectionRequirement::Creature },
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut g = two_player_game();
        let p = g.add_card_to_battlefield(0, pinger);
        g.clear_sickness(p);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, dies to 3
        match pick_removal_ping(&g, 0).expect("bot pings with its own power") {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, p);
                assert_eq!(target, Some(Target::Permanent(foe)));
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot points a ping at the opponent's face when it's exactly lethal
    /// (reach for the win), not at a creature.
    #[test]
    fn bot_pings_face_for_lethal() {
        let mut g = two_player_game();
        let fw = g.add_card_to_battlefield(0, catalog::frostwielder()); // {T}: 1 dmg any target
        g.clear_sickness(fw);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a 2/2 it can't kill
        g.players[1].life = 1; // lethal to a 1-damage ping
        let action = pick_removal_ping(&g, 0).expect("bot reaches for the win");
        match action {
            GameAction::ActivateAbility { target, .. } => {
                assert_eq!(target, Some(Target::Player(1)), "ping aimed at the face");
            }
            _ => panic!("expected an activate-ability action"),
        }
        // Above 1 life it isn't lethal and there's no killable creature → hold.
        g.players[1].life = 5;
        assert!(pick_removal_ping(&g, 0).is_none(), "won't chip a non-lethal face");
    }

    /// The bot fires a team-pump ability (Bearer of Glory's {4}{W}) once it has
    /// two attackers, but holds it with only one.
    #[test]
    fn bot_team_pumps_with_multiple_attackers() {
        let mut g = two_player_game();
        let bearer = g.add_card_to_battlefield(0, catalog::bearer_of_glory());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bearer);
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        // One attacker: not worth the pump.
        g.attacking = vec![Attack { attacker: bearer, target: AttackTarget::Player(1) }];
        assert!(pick_team_pump(&g, 0).is_none(), "holds the pump with one attacker");
        // Two attackers: fire it.
        g.attacking.push(Attack { attacker: bear, target: AttackTarget::Player(1) });
        match pick_team_pump(&g, 0).expect("bot pumps the team") {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, bearer),
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot crews a Vehicle with a spare small creature, but won't tap a
    /// creature bigger than the Vehicle to do it.
    #[test]
    fn bot_crews_a_vehicle_with_a_small_creature() {
        let mut g = two_player_game();
        let chariot = g.add_card_to_battlefield(0, catalog::thundering_chariot()); // 3/3, Crew 1
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(bear);
        match pick_crew(&g, 0) {
            Some(GameAction::Crew { vehicle, crew_creatures }) => {
                assert_eq!(vehicle, chariot);
                assert_eq!(crew_creatures, vec![bear]);
            }
            other => panic!("expected a crew action, got {other:?}"),
        }
        // Swap the bear for a 5/5: tapping it to animate a 3/3 isn't worth it.
        g.battlefield.retain(|c| c.id != bear);
        let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
        g.clear_sickness(dragon);
        assert!(pick_crew(&g, 0).is_none(), "won't tap a bigger body to crew a smaller Vehicle");
    }

    /// The bot saddles a Mount it can attack with using a spare small creature,
    /// but won't tap a creature bigger than the Mount to do it.
    #[test]
    fn bot_saddles_a_mount_with_a_small_creature() {
        let mut g = two_player_game();
        let ghoda = g.add_card_to_battlefield(0, catalog::gilded_ghoda()); // 2/2, Saddle 1
        g.clear_sickness(ghoda);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(bear);
        match pick_saddle(&g, 0) {
            Some(GameAction::Saddle { mount, creatures }) => {
                assert_eq!(mount, ghoda);
                assert_eq!(creatures, vec![bear]);
            }
            other => panic!("expected a saddle action, got {other:?}"),
        }
        // A summoning-sick Mount can't attack → don't waste a saddler on it.
        g.battlefield_find_mut(ghoda).unwrap().summoning_sick = true;
        assert!(pick_saddle(&g, 0).is_none(), "won't saddle a Mount that can't attack");
    }

    /// The bot only saddles in precombat main — in postcombat main the "until
    /// end of turn" buff would wear off before any attack could use it.
    #[test]
    fn bot_does_not_saddle_in_postcombat_main() {
        let mut g = two_player_game();
        let ghoda = g.add_card_to_battlefield(0, catalog::gilded_ghoda()); // Saddle 1
        g.clear_sickness(ghoda);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        assert!(pick_saddle(&g, 0).is_some(), "saddles in precombat main");
        g.step = TurnStep::PostCombatMain;
        assert!(pick_saddle(&g, 0).is_none(), "no saddle after combat is over");
    }

    /// Saddle 3 on a 2-power Mount (Caustic Bronco) still gets saddled when the
    /// only saddlers are idle (summoning-sick) creatures: they can't attack, so
    /// their power isn't "wasted" against the overspend guard.
    #[test]
    fn bot_saddles_high_cost_mount_with_idle_creatures() {
        let mut g = two_player_game();
        let bronco = g.add_card_to_battlefield(0, catalog::caustic_bronco()); // 2/2, Saddle 3
        g.clear_sickness(bronco);
        // Two summoning-sick 2/2s — idle this turn, so free to tap.
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        match pick_saddle(&g, 0) {
            Some(GameAction::Saddle { mount, creatures }) => {
                assert_eq!(mount, bronco);
                assert_eq!(creatures.len(), 2, "taps both idle bears to reach Saddle 3");
                assert!(creatures.contains(&a) && creatures.contains(&b));
            }
            other => panic!("expected a saddle action, got {other:?}"),
        }
        // If the same bears could attack, don't overspend real attacker power.
        g.clear_sickness(a);
        g.clear_sickness(b);
        assert!(
            pick_saddle(&g, 0).is_none(),
            "won't tap 4 attacker-power to saddle a 2-power Mount"
        );
    }

    /// The bot sacrifices Pus Kami to destroy a bigger opposing creature, but
    /// not to kill something smaller than the creature it would pitch.
    #[test]
    fn bot_sacs_to_destroy_a_favorable_trade() {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::pus_kami()); // 3/3
        g.clear_sickness(kami);
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        // A 5/5-equivalent opposing threat (nonblack) → favorable sac.
        let dreadmaw = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6 green
        let action = pick_removal_sacrifice(&g, 0).expect("bot sacs to kill the big threat");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, kami);
                assert_eq!(target, Some(Target::Permanent(dreadmaw)));
            }
            _ => panic!("expected an activate-ability action"),
        }
        // Replace with a 1/1 — sacrificing a 3/3 for it is a bad trade.
        g.battlefield.retain(|c| c.id != dreadmaw);
        g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
        assert!(pick_removal_sacrifice(&g, 0).is_none(), "won't sac a 3/3 to kill a 1/1");
    }

    /// The bot recurs a creature from the graveyard via Embalm when it can
    /// afford the cost.
    #[test]
    fn bot_embalms_from_graveyard_with_spare_mana() {
        use crate::TurnStep;
        let mut g = two_player_game();
        let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Embalm Sacred Cat");
        match action {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, cat),
            _ => panic!("expected an activate-ability action"),
        }
        // With no mana it leaves the card alone.
        g.players[0].mana_pool.empty();
        assert!(pick_graveyard_recursion(&g, 0).is_none(), "won't Embalm without mana");
    }

    /// The bot reanimates a graveyard creature with a battlefield permanent's
    /// sac-to-return ability (Seedship Broodtender), aimed at the dead creature.
    #[test]
    fn bot_reanimates_from_graveyard_via_battlefield_ability() {
        use crate::TurnStep;
        use crate::mana::Color;
        let mut g = two_player_game();
        let brood = g.add_card_to_battlefield(0, catalog::seedship_broodtender());
        let dead = g.add_card_to_graveyard(0, catalog::colossal_dreadmaw()); // a worthy target
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_battlefield_reanimate(&g, 0).expect("bot reanimates from graveyard");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, brood);
                assert_eq!(target, Some(Target::Permanent(dead)));
            }
            _ => panic!("expected an activate-ability action"),
        }
        // Empty graveyard → nothing to do.
        g.players[0].graveyard.clear();
        assert!(pick_battlefield_reanimate(&g, 0).is_none(), "no target → no activation");
    }

    /// The bot uses a *targeted* graveyard-activated ability (Scavenge),
    /// auto-picking its own creature as the target.
    #[test]
    fn bot_scavenges_onto_own_creature() {
        use crate::TurnStep;
        let mut g = two_player_game();
        let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mangler = g.add_card_to_graveyard(0, catalog::dreg_mangler());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Scavenge Dreg Mangler");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, mangler);
                assert_eq!(target, Some(crate::game::Target::Permanent(beater)),
                    "auto-targets the bot's own creature");
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot activates Varolz's *granted* scavenge (a virtual graveyard
    /// ability at index ≥ printed_count), not just printed scavenge cards.
    #[test]
    fn bot_scavenges_via_varolz_grant() {
        use crate::TurnStep;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::varolz_the_scar_striped());
        let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Scavenge via Varolz");
        match action {
            GameAction::ActivateAbility { card_id, ability_index, target, .. } => {
                assert_eq!(card_id, dead);
                assert_eq!(ability_index, 0, "granted scavenge at index 0 (no printed abilities)");
                // Auto-targets one of the bot's own creatures.
                let t = matches!(target, Some(crate::game::Target::Permanent(id))
                    if id == beater || g.battlefield_find(id).is_some_and(|c| c.controller == 0));
                assert!(t, "scavenge targets an own creature");
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot also recognises the real-cost energy form
    /// (`ActivatedAbility.energy_cost`), not just resolve-time `PayEnergy`.
    #[test]
    fn bot_spends_energy_on_real_cost_form() {
        use crate::card::{ActivatedAbility, CardDefinition, CardType, CounterType};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "Energy Engine",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                energy_cost: 2,
                discard_cost: None,
                effect: Effect::AddCounter {
                    what: crate::effect::Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: crate::effect::Value::Const(1),
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.players[0].energy = 2;
        assert!(pick_energy_payoff(&g, 0).is_some(), "bot fires the energy_cost-gated payoff");
        g.players[0].energy = 1;
        assert!(pick_energy_payoff(&g, 0).is_none(), "and only when it can afford it");
    }

    /// Mulligan heuristic: ship a 1-land seven, keep a 3-land seven, and
    /// stop digging once two mulligans have been taken.
    #[test]
    fn bot_mulligans_land_light_hands_but_keeps_balanced_ones() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // 1 land + 6 spells → mulligan.
        g.add_card_to_hand(0, catalog::island());
        for _ in 0..6 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan));
        // Stop digging after two mulligans even on a bad hand.
        assert!(matches!(decide_mulligan(&g, 0, 2, &EvalWeights::default()), DecisionAnswer::Keep));

        // 3 lands + 4 spells, colors aligned (Forests for green bears) → keep.
        let mut g2 = two_player_game();
        for _ in 0..3 { g2.add_card_to_hand(0, catalog::forest()); }
        for _ in 0..4 { g2.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g2, 0, 0, &EvalWeights::default()), DecisionAnswer::Keep));
    }

    /// Two 2/2s eat a 4/4 when life isn't threatened. The greedy pass
    /// only gangs under lethal pressure, and `block_search` can only
    /// remove blockers, so at a healthy life total this attacker used to
    /// get through untouched — trading two bears for a bomb is a fine
    /// deal the bot simply never considered.
    #[test]
    fn gang_blocks_for_value_not_only_for_survival() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 1;
        // A 4/4 attacking a comfortable life total.
        let big = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.clear_sickness(big);
        let bear_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bear_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear_a);
        g.clear_sickness(bear_b);
        g.players[0].life = 20;
        g.set_attacking(vec![Attack { attacker: big, target: crate::game::types::AttackTarget::Player(0) }]);

        // Serra Angel flies; ground bears can't block it at all, so the
        // gang must be legal to be offered. Swap to a ground fatty.
        let mut g2 = two_player_game();
        g2.step = TurnStep::DeclareBlockers;
        g2.active_player_idx = 1;
        let fatty = g2.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
        g2.clear_sickness(fatty);
        let b1 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b2 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b3 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        for b in [b1, b2, b3] { g2.clear_sickness(b); }
        g2.players[0].life = 20;
        g2.set_attacking(vec![Attack { attacker: fatty, target: crate::game::types::AttackTarget::Player(0) }]);

        let greedy = pick_blocks(&g2, 0);
        assert!(greedy.iter().filter(|(_, a)| *a == fatty).count() <= 1,
            "greedy blocks the 6/4 with at most one body at 20 life: {greedy:?}");

        let gangs = gang_block_candidates(&g2, 0, &greedy, &EvalWeights::block_gang_search());
        assert!(!gangs.is_empty(), "a gang candidate is offered");
        let gang = &gangs[0];
        let on_fatty = gang.iter().filter(|(_, a)| *a == fatty).count();
        assert!(on_fatty >= 2, "the gang puts two or more blockers on it: {gang:?}");
    }

    /// `mull_quality` fixes the two hands the shipped rule reads
    /// backwards: a two-lander whose only play is one two-drop (kept
    /// today, does nothing from turn three) and a six-land hand holding
    /// a bomb (shipped today, a fine limited keep).
    #[test]
    fn mull_quality_judges_the_hand_not_just_the_land_count() {
        use crate::decision::DecisionAnswer;
        let w = EvalWeights::mulligan_quality();

        // Two Forests, one castable bear, four uncastable six-drops.
        let mut thin = two_player_game();
        for _ in 0..2 { thin.add_card_to_hand(0, catalog::forest()); }
        thin.add_card_to_hand(0, catalog::grizzly_bears());
        for _ in 0..4 { thin.add_card_to_hand(0, catalog::craw_wurm()); }
        assert!(matches!(decide_mulligan(&thin, 0, 0, &EvalWeights::default()), DecisionAnswer::Keep),
            "the shipped rule keeps this on the strength of one two-drop");
        assert!(matches!(decide_mulligan(&thin, 0, 0, &w), DecisionAnswer::TakeMulligan),
            "one play is not a keep at two lands");

        // Six Plains and Serra Angel: flooded, but the payoff is real.
        let mut flooded = two_player_game();
        for _ in 0..6 { flooded.add_card_to_hand(0, catalog::plains()); }
        flooded.add_card_to_hand(0, catalog::serra_angel());
        assert!(matches!(decide_mulligan(&flooded, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan),
            "the shipped rule ships every six-land hand");
        assert!(matches!(decide_mulligan(&flooded, 0, 0, &w), DecisionAnswer::Keep),
            "a bomb carries the flood");
    }

    /// Color-screw: enough lands and a fine curve, but the lands can't make
    /// the spells' colors (3 Islands + green {1}{G} Grizzly Bears) → ship it.
    #[test]
    fn bot_mulligans_color_screwed_hands() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        for _ in 0..4 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan),
            "no green source for the green spells → color screw → mulligan");
    }

    /// Curve screen: a hand with enough lands but only spells too expensive
    /// to cast early is a screwed keep — ship it on the first mulligan.
    #[test]
    fn bot_mulligans_lands_with_no_early_play() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // 3 lands + four {6} Obsianus Golems → no spell castable by turn ~4.
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        for _ in 0..4 { g.add_card_to_hand(0, catalog::obsianus_golem()); }
        assert!(matches!(decide_mulligan(&g, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan),
            "no early play despite enough lands → mulligan");
    }

    /// Sac-cost mana abilities (Lotus Petal) are NOT auto-activated — they
    /// destroy the source on activation, which the random bot can't reason
    /// about.
    #[test]
    fn bot_does_not_tap_sac_cost_mana_source() {
        let mut g = two_player_game();
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        // Should not activate Lotus Petal's sac-cost ability.
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, petal, "bot must NOT auto-tap a sac-cost mana source");
        }
    }

    /// Bot activates a planeswalker's loyalty ability when one is
    /// available, picking by OUTCOME: on an empty board Karn's -2
    /// Construct token (a real body that also protects the walker)
    /// out-values the +1's slow card. Karn at 5 loyalty afterward sits
    /// at a healthy 3 — this is development, not a suicide-ult.
    #[test]
    fn bot_activates_planeswalker_loyalty_ability() {
        let mut g = two_player_game();
        // Karn: +1 (reveal two, opponent picks one for your hand) at
        // index 0, a -1 at index 1, and a -2 (Construct token) at index 2.
        let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
        g.clear_sickness(karn);
        // Stock the library so the +1 has cards to reveal.
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());

        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, karn, "bot should target the Karn it controls");
                assert_eq!(ability_index, 2,
                    "the -2 Construct (board presence) out-values the +1's slow card");
            }
            other => panic!("expected ActivateLoyaltyAbility, got {:?}", other),
        }
    }

    /// The attack search must actually *reach* the opponent's crack-back.
    /// A simulation that bails — on fuel, a rejected declaration, or a step
    /// it can't advance past — silently degrades the whole search to the
    /// greedy declaration it was meant to second-guess, and nothing else in
    /// the suite would notice, because falling back is not an error.
    #[test]
    fn attack_simulation_reaches_the_crack_back() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(c);
        let w = EvalWeights::attack_search();
        let greedy = pick_attacks(&g, 0);
        assert_eq!(greedy.len(), 2, "both bears are eligible attackers");
        assert!(
            simulate_attack_outcome(&g, 0, &greedy, &w).is_some(),
            "the alpha strike must simulate to a score"
        );
        assert!(
            simulate_attack_outcome(&g, 0, &[], &w).is_some(),
            "declining to attack must simulate to a score"
        );
    }

    /// Holding a blocker back is only ever *worth* anything a turn later, so
    /// the search has to price it there: two bears into an empty board is a
    /// free swing, but with a 3/3 staring back, keeping one home to block is
    /// the better board once the crack-back is resolved.
    #[test]
    fn attack_search_holds_a_blocker_against_a_bigger_board() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        // A 3/3 that eats a 2/2 for free if we have nothing back.
        let big = g.add_card_to_battlefield(1, catalog::hill_giant());
        g.clear_sickness(big);
        // Both players need something to draw: the simulation runs a full
        // turn cycle, and an empty library decks whoever draws first, which
        // pins every candidate to the same "we won" score.
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        let w = EvalWeights::attack_search();
        let all_in = simulate_attack_outcome(&g, 0, &pick_attacks(&g, 0), &w);
        let none = simulate_attack_outcome(&g, 0, &[], &w);
        assert!(all_in.is_some() && none.is_some(), "both lines must simulate");
        assert_ne!(all_in, none, "the two lines must not score identically \
             — if they do, the simulation is not reaching the crack-back");
    }

    /// Helper: a 1/1 creature with one extra keyword for attack-filter tests.
    fn one_one_with(name: &'static str, kw: crate::card::Keyword) -> CardDefinition {
        let mut d = catalog::grizzly_bears();
        d.name = name;
        d.power = 1;
        d.toughness = 1;
        d.keywords.push(kw);
        d
    }

    /// A menace attacker swings even into a single bigger blocker — menace
    /// needs two blockers, so it gets through (the suicide filter must not
    /// hold it back when the opponent has fewer than two blockers).
    #[test]
    fn bot_attacks_with_menace_into_lone_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, one_one_with("Sneak", crate::card::Keyword::Menace));
        g.clear_sickness(atk);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // lone 2/2 blocker
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|atk_decl| atk_decl.attacker == atk),
                    "menace attacker should swing past a lone blocker");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// CR 506.2 — under Silent Arbiter the bot declares exactly one attacker
    /// (the engine rejects any bigger batch outright).
    #[test]
    fn bot_respects_the_silent_arbiter_attack_cap() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::silent_arbiter());
        for _ in 0..3 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.len() <= 1, "batch trimmed to the cap, got {}", a.len());
                let mut g2 = g.clone();
                g2.declare_attackers(a).expect("the trimmed batch is legal");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// The combat planners and `declare_attackers` / `declare_blockers` are
    /// two hand-written readings of the same rules, and these four cases are
    /// where they disagreed. Each one ends by handing the plan to the engine:
    /// the engine is the oracle, and it rejects the whole **batch**, so one
    /// illegal pair used to cost the bot every block or attack it had made.
    ///
    /// Found by counting `sim_step`'s checkpointed rollbacks, which is the
    /// only place a rejected declaration leaves a trace — 82 in a twenty-game
    /// `cube` run, 0 after these fixes. See ENGINE_BACKLOG P3.
    #[test]
    fn bot_block_plan_honours_landwalk() {
        use crate::card::LandType;
        let mut g = two_player_game();
        // Defender controls a Mountain, so the mountainwalker is unblockable.
        g.add_card_to_battlefield(1, catalog::mountain());
        let walker = g.add_card_to_battlefield(0, catalog::sokenzan_bruiser());
        g.clear_sickness(walker);
        for _ in 0..2 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        // Low enough that the planner wants to chump — without the pressure it
        // declines the block for value reasons and the test cannot see the
        // legality gate at all.
        g.players[1].life = 2;
        assert!(
            g.defender_controls_land_type(1, &LandType::Mountain),
            "the defender has the land the walk names",
        );
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: walker,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.is_empty(), "nothing may block a mountainwalker here: {blocks:?}");
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 509.1b — menace read off the *computed* set. A granted Menace is
    /// invisible to the printed keyword list, so the planner assigned one
    /// blocker and the engine rejected the batch with
    /// `MenaceRequiresTwoBlockers`.
    #[test]
    fn bot_block_plan_honours_granted_menace() {
        use crate::card::{CardType, StaticAbility};
        use crate::effect::{Selector, StaticEffect};
        let anthem = CardDefinition {
            name: "Menace Anthem",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        crate::card::SelectionRequirement::Creature
                            .and(crate::card::SelectionRequirement::ControlledByYou),
                    ),
                    keyword: crate::card::Keyword::Menace,
                },
                description: "Creatures you control have menace.",
            }],
            ..Default::default()
        };
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, anthem);
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        // Two possible blockers: the plan must use both or neither.
        for _ in 0..2 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.len() != 1, "menace takes 0 or 2+ blockers, got {blocks:?}");
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// A creature-type-free anthem granting `keyword` to its controller's
    /// creatures — the shortest way to put a keyword in the *computed* set and
    /// nowhere else. Both guards below need one; the granted-Menace guard above
    /// wrote it out by hand first.
    #[cfg(test)]
    fn granting_anthem(name: &'static str, keyword: crate::card::Keyword) -> CardDefinition {
        use crate::card::{CardType, SelectionRequirement, StaticAbility};
        use crate::effect::{Selector, StaticEffect};
        CardDefinition {
            name,
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword,
                },
                description: "Creatures you control have the granted keyword.",
            }],
            ..Default::default()
        }
    }

    /// CR 509.1b — **the direction `CRAB_SIM_REJECTS` cannot see.** A granted
    /// Reach is invisible to the printed keyword list, so a planner reading the
    /// instance view declines a block the engine would have accepted: a legal
    /// line made permanently invisible. Nothing illegal is ever proposed, so no
    /// rejection counter reports it and only a test in this direction can.
    ///
    /// The defender is at 2 against a 2/2 flier, so the block is wanted — a
    /// planner test that does not make the planner *want* the move proves
    /// nothing (the landwalk guard above learned that the hard way).
    #[test]
    fn bot_block_plan_sees_a_granted_reach() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, granting_anthem("Reach Anthem", Keyword::Reach));
        let flyer = g.add_card_to_battlefield(0, catalog::wind_drake());
        g.clear_sickness(flyer);
        let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        assert!(
            !g.battlefield_find(ground).unwrap().has_keyword(&Keyword::Reach),
            "the reach is a grant, not a printed keyword — otherwise this guards nothing",
        );
        g.players[1].life = 2;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: flyer,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(
            blocks,
            vec![(ground, flyer)],
            "a granted Reach blocks a flier at 2 life; got {blocks:?}",
        );
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 509.1b, the other direction: a granted **Flying** on the attacker is
    /// invisible to `AttackerFacts`' printed keyword reads, and a ground block
    /// against it is what `declare_blockers` rejects — the whole batch, not the
    /// pair. The engine is the oracle here, as in every guard in this family:
    /// the plan is handed to it.
    #[test]
    fn bot_block_plan_honours_a_granted_flying() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, granting_anthem("Flying Anthem", Keyword::Flying));
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        assert!(
            !g.battlefield_find(atk).unwrap().has_keyword(&Keyword::Flying),
            "the flying is a grant, not a printed keyword",
        );
        for _ in 0..2 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        // Low enough that the planner wants to chump, so the legality gate is
        // the only thing that can stop it.
        g.players[1].life = 2;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.is_empty(), "no ground creature may block a granted flier: {blocks:?}");
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 509.1d — the block tax is charged per declared blocker and the
    /// engine rejects the declaration *whole*, so a planner that ignores it
    /// loses its block step, not a blocker. The block twin of the attack
    /// tax; PERF (-55), `combat.rs:2418`.
    #[test]
    fn bot_block_plan_trims_to_the_payable_block_tax() {
        use crate::card::{CardType, StaticAbility};
        use crate::effect::{StaticEffect, Value};
        let toll = || CardDefinition {
            name: "Blockade Toll",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                effect: StaticEffect::BlockTaxToController {
                    amount: Value::Const(2),
                    only_while_attacking: false,
                    filter: None,
                    life: false,
                },
                description: "Creatures can't block unless their controller pays {2} for each.",
            }],
            ..Default::default()
        };
        let setup = |lands: usize| {
            let mut g = two_player_game();
            g.add_card_to_battlefield(0, toll());
            let attackers: Vec<_> = (0..2)
                .map(|_| {
                    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
                    g.clear_sickness(a);
                    a
                })
                .collect();
            for _ in 0..2 {
                g.add_card_to_battlefield(1, catalog::hill_giant());
            }
            for _ in 0..lands {
                g.add_card_to_battlefield(1, catalog::plains());
            }
            g.step = TurnStep::DeclareAttackers;
            g.active_player_idx = 0;
            g.priority.player_with_priority = 0;
            g.declare_attackers(
                attackers
                    .iter()
                    .map(|a| crate::game::Attack {
                        attacker: *a,
                        target: crate::game::AttackTarget::Player(1),
                    })
                    .collect(),
            )
            .expect("attack");
            g.step = TurnStep::DeclareBlockers;
            g.priority.player_with_priority = 1;
            g
        };
        // {2} a blocker, so the plan is the budget / 2.
        for (lands, want) in [(0usize, 0usize), (2, 1), (4, 2)] {
            let g = setup(lands);
            let blocks = pick_blocks_for_test(&g, 1);
            assert_eq!(
                blocks.len(),
                want,
                "{lands} untapped lands fund {want} blockers: {blocks:?}",
            );
            g.clone().declare_blockers(blocks).expect("the plan the planner made is payable");
        }
    }

    /// CR 702.39 — a provoked creature must be assigned to block its
    /// provoker if able, and the engine rejects the whole declaration when
    /// it is not: 82 of the block rejections on `cube --seed 11`.
    #[test]
    fn bot_block_plan_honours_provoke() {
        let mut g = two_player_game();
        let provoker = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        g.clear_sickness(provoker);
        // A body the greedy planner would rather keep home than trade away.
        let provoked = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: provoker,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.battlefield_find_mut(provoked).expect("on the battlefield").must_block =
            Some(provoker);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(
            blocks.contains(&(provoked, provoker)),
            "the provoked creature has to block its provoker: {blocks:?}",
        );
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// A vanilla creature definition, for the boards below where the rule is
    /// a keyword the catalog only ever grants.
    fn vanilla_creature(name: &'static str, power: i32, toughness: i32) -> CardDefinition {
        CardDefinition {
            name,
            card_types: vec![crate::card::CardType::Creature],
            power,
            toughness,
            ..Default::default()
        }
    }

    /// CR 509.1c, true Lure (`AllMustBlock`) — *every* defender able to block
    /// such an attacker must be assigned to it, and `declare_blockers` asks
    /// the question per creature without caring what else that creature was
    /// doing. So the only satisfiable declaration puts the whole able set on
    /// the Lure attacker and nothing anywhere else. 30 of `cube` seed 5's and
    /// 34 of seed 15's block rejections.
    #[test]
    fn bot_block_plan_honours_true_lure() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        let mut lure = vanilla_creature("Lure Beast", 1, 6);
        lure.keywords.push(Keyword::AllMustBlock);
        let lured = g.add_card_to_battlefield(0, lure);
        g.clear_sickness(lured);
        // A second attacker the value pass would much rather block: a 4/4 at
        // a defender on 5 life is the block the planner wants to make, and
        // making it is exactly what the Lure forbids.
        let decoy = g.add_card_to_battlefield(0, vanilla_creature("Decoy", 4, 4));
        g.clear_sickness(decoy);
        let a = g.add_card_to_battlefield(1, vanilla_creature("Guard A", 2, 2));
        let b = g.add_card_to_battlefield(1, vanilla_creature("Guard B", 3, 3));
        g.players[1].life = 5;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![
            crate::game::Attack { attacker: lured, target: crate::game::AttackTarget::Player(1) },
            crate::game::Attack { attacker: decoy, target: crate::game::AttackTarget::Player(1) },
        ])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(
            blocks.contains(&(a, lured)) && blocks.contains(&(b, lured)),
            "every able defender blocks the Lure attacker: {blocks:?}",
        );
        assert!(
            !blocks.iter().any(|(_, aid)| *aid == decoy),
            "and none of them is anywhere else: {blocks:?}",
        );
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 509.1b outranks CR 509.1c: a restriction the defender cannot
    /// satisfy un-binds the requirement rather than making the combat
    /// undeclarable. A Lure attacker that also has Menace, facing exactly one
    /// able blocker, had **no legal declaration at all** — block with nobody
    /// and the Lure rule rejects it, block with the one body and the count
    /// rule does. Reachable in the routine pools (an aura granting
    /// `AllMustBlock` onto a Menace creature, `cube` seed 15), and it is the
    /// engine that has to give: no planner can plan around a contradiction.
    #[test]
    fn a_count_restriction_unbinds_a_block_requirement() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        let mut both = vanilla_creature("Lure Menace", 3, 3);
        both.keywords.push(Keyword::AllMustBlock);
        both.keywords.push(Keyword::Menace);
        let atk = g.add_card_to_battlefield(0, both);
        g.clear_sickness(atk);
        let only = g.add_card_to_battlefield(1, vanilla_creature("Only", 2, 2));
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        // Blocking with nobody is now legal: the Lure requirement does not
        // bind, because no legal declaration could have satisfied it.
        g.clone().declare_blockers(vec![]).expect("no block is legal");
        // And the one-body block is still illegal, on the count rule.
        assert!(
            g.clone().declare_blockers(vec![(only, atk)]).is_err(),
            "one blocker on a Menace attacker stays illegal",
        );
        let picked = pick_blocks_for_test(&g, 1);
        g.declare_blockers(picked).expect("and the planner picks a legal one");
    }

    /// CR 509.1c — "the **maximum number** of requirements". Two requirements
    /// that both name the same creature can never both be satisfied, because
    /// it blocks one attacker; asking each in isolation made every
    /// declaration illegal.
    ///
    /// A Lure attacker, a provoker, and one able defender: block nobody and
    /// the Lure rejects it, block the Lure and Provoke rejects it, block the
    /// provoker and the Lure rejects it. All three, on a board reachable in
    /// `cube` seed 15 — where it was the entire remaining rejection census.
    /// A creature blocking *something that obliges it* is the most any
    /// declaration gets out of it, so both single-block plans are maximal and
    /// legal, and "block with nobody" is still not.
    #[test]
    fn two_block_requirements_on_one_creature_are_both_satisfiable() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        let mut lure = vanilla_creature("Lure Beast", 3, 3);
        lure.keywords.push(Keyword::AllMustBlock);
        let lured = g.add_card_to_battlefield(0, lure);
        g.clear_sickness(lured);
        let provoker = g.add_card_to_battlefield(0, vanilla_creature("Provoker", 2, 2));
        g.clear_sickness(provoker);
        let only = g.add_card_to_battlefield(1, vanilla_creature("Only", 2, 2));
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![
            crate::game::Attack { attacker: lured, target: crate::game::AttackTarget::Player(1) },
            crate::game::Attack {
                attacker: provoker,
                target: crate::game::AttackTarget::Player(1),
            },
        ])
        .expect("attack");
        g.battlefield_find_mut(only).expect("on the battlefield").must_block = Some(provoker);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        g.clone().declare_blockers(vec![(only, lured)]).expect("satisfying the Lure is legal");
        g.clone()
            .declare_blockers(vec![(only, provoker)])
            .expect("satisfying the Provoke is legal");
        assert!(
            g.clone().declare_blockers(vec![]).is_err(),
            "and satisfying neither is still illegal",
        );
        let picked = pick_blocks_for_test(&g, 1);
        assert!(!picked.is_empty(), "the planner picks one of them: {picked:?}");
        g.declare_blockers(picked).expect("and it is legal");
    }

    /// CR 509.1c, `MustBlock` — "blocks each combat if able", asked of the
    /// *blocker*. The planner had no pass for it at all: 8 of `cube` seed
    /// 42's block rejections, and the only site the census reached on a seed
    /// outside 1-24.
    #[test]
    fn bot_block_plan_honours_blocks_each_combat_if_able() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        // A 4/4 attacker into a 1/1 that must block: the trade is pure loss,
        // so nothing but the requirement puts the body in front of it.
        let atk = g.add_card_to_battlefield(0, vanilla_creature("Big", 4, 4));
        g.clear_sickness(atk);
        let mut obliged = vanilla_creature("Obliged", 1, 1);
        obliged.keywords.push(Keyword::MustBlock);
        let blocker = g.add_card_to_battlefield(1, obliged);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(
            blocks.contains(&(blocker, atk)),
            "a creature that blocks each combat if able has to be somewhere: {blocks:?}",
        );
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 509.1g — `CantBeBlockedByMoreThanOne` is the *ceiling* the count
    /// rule never had. The gang pass builds exactly the superset that breaks
    /// it: 48 of `cube` seed 23's 48 block rejections.
    #[test]
    fn bot_block_plan_honours_the_single_blocker_cap() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        // A 5/5 two 3/3s would gang down happily, and may not: the greedy
        // pass wants the gang, so only the cap can stop it.
        let mut rhino = vanilla_creature("Charging Rhino", 6, 6);
        rhino.keywords.push(Keyword::CantBeBlockedByMoreThanOne);
        let atk = g.add_card_to_battlefield(0, rhino);
        g.clear_sickness(atk);
        let a = g.add_card_to_battlefield(1, vanilla_creature("Guard A", 4, 4));
        let b = g.add_card_to_battlefield(1, vanilla_creature("Guard B", 4, 4));
        g.players[1].life = 6;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        let on_rhino = blocks.iter().filter(|(_, aid)| *aid == atk).count();
        assert!(on_rhino <= 1, "at most one blocker on it: {blocks:?}");
        let _ = (a, b);
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 509.1b / 702.39 — the block search's candidates are subsets, and a
    /// subset can release a provoked creature or strip the second blocker off
    /// a Menace attacker. Every candidate on the menu has to be a declaration
    /// the engine accepts.
    #[test]
    fn block_search_candidates_are_all_legal_declarations() {
        use crate::card::{CardType, Keyword};
        let menacing = CardDefinition {
            name: "Skulking Brute",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Menace],
            ..Default::default()
        };
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, menacing);
        g.clear_sickness(atk);
        for _ in 0..3 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let w = EvalWeights { block_search: 3, ..EvalWeights::default() };
        let candidates = block_candidates_for_mcts(&g, 1, &w);
        for c in &candidates {
            assert_ne!(
                c.iter().filter(|(_, a)| *a == atk).count(),
                1,
                "menace takes 0 or 2+ blockers, never 1: {c:?}",
            );
            g.clone().declare_blockers(c.clone()).expect("every candidate is legal");
        }
    }

    /// A board with attackers declared at seat 1 by seat 0's creatures,
    /// ready for seat 1's block step.
    fn attacked_board(attackers: &[CardDefinition], blockers: usize) -> (GameState, Vec<CardId>) {
        let mut g = two_player_game();
        let mut ids = Vec::new();
        for d in attackers {
            let a = g.add_card_to_battlefield(0, d.clone());
            g.clear_sickness(a);
            ids.push(a);
        }
        for _ in 0..blockers {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(
            ids.iter()
                .map(|&a| crate::game::Attack { attacker: a, target: crate::game::AttackTarget::Player(1) })
                .collect(),
        )
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        (g, ids)
    }

    /// The block chain's finished plan is a declaration the engine accepts,
    /// and it never leaves a Menace attacker with exactly one blocker (CR
    /// 702.110b): the pair gate and the repair run at every step.
    #[test]
    fn block_chain_plan_is_legal_and_never_single_blocks_menace() {
        use crate::card::{CardType, Keyword};
        let menacing = CardDefinition {
            name: "Skulking Brute",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Menace],
            ..Default::default()
        };
        let (g, atk) = attacked_board(&[menacing], 3);
        let w = EvalWeights::block_chain_on();
        let menu = block_candidates_for_mcts(&g, 1, &w);
        let (chain, _) = block_chain_candidate(&g, 1, &w, &menu, &[], BlockChainSetup::new(&g, 1).expect("the chain can run"), &SimStarts::new(&g, 1, &w)).expect("scored");
        assert_ne!(chain.iter().filter(|(_, a)| *a == atk[0]).count(), 1, "{chain:?}");
        g.clone().declare_blockers(chain).expect("the chained plan is legal");
        let picked = pick_blocks_scored(&g, 1, &w);
        assert_ne!(picked.iter().filter(|(_, a)| *a == atk[0]).count(), 1, "{picked:?}");
        g.clone().declare_blockers(picked).expect("the picked plan is legal");
    }

    /// The hole the chain closes: a 3/3 into two bears at twenty life.
    /// Greedy finds no profitable single block and no chump is warranted,
    /// so the menu is bare "no blocks" and the gang generator never runs —
    /// the double block that trades a bear for the giant was unreachable.
    /// The chain's gang move finds it from nothing.
    #[test]
    fn block_chain_finds_the_gang_block_the_bare_menu_cannot() {
        let (g, atk) = attacked_board(&[catalog::hill_giant()], 2);
        let w = EvalWeights::block_chain_on();
        let menu = block_candidates_for_mcts(&g, 1, &w);
        assert_eq!(menu, vec![Vec::new()], "the menu is bare: {menu:?}");
        let (chain, _) = block_chain_candidate(&g, 1, &w, &menu, &[], BlockChainSetup::new(&g, 1).expect("the chain can run"), &SimStarts::new(&g, 1, &w)).expect("scored");
        assert_eq!(chain.len(), 2, "both bears on the giant: {chain:?}");
        assert!(chain.iter().all(|(_, a)| *a == atk[0]), "{chain:?}");
        let picked = pick_blocks_scored(&g, 1, &w);
        assert_eq!(picked.len(), 2, "the picker takes the gang: {picked:?}");
        g.clone().declare_blockers(picked).expect("legal");
    }

    /// Two gangs at once: two giants into four bears. The gang generator
    /// emits one candidate per attacker and never both, so the menu can
    /// express "gang A" or "gang B", never "gang A and gang B"; the chain
    /// takes one gang move, then the other.
    #[test]
    fn block_chain_reaches_a_double_gang_the_menu_cannot() {
        let (g, atk) = attacked_board(&[catalog::hill_giant(), catalog::hill_giant()], 4);
        let w = EvalWeights::block_chain_on();
        let menu = block_candidates_for_mcts(&g, 1, &w);
        assert!(
            !menu.iter().any(|c| c.len() == 4),
            "the menu never holds the double gang: {menu:?}"
        );
        let (chain, _) = block_chain_candidate(&g, 1, &w, &menu, &[], BlockChainSetup::new(&g, 1).expect("the chain can run"), &SimStarts::new(&g, 1, &w)).expect("scored");
        assert_eq!(chain.len(), 4, "two bears on each giant: {chain:?}");
        for a in &atk {
            assert_eq!(chain.iter().filter(|(_, x)| x == a).count(), 2, "{chain:?}");
        }
        let picked = pick_blocks_scored(&g, 1, &w);
        assert_eq!(picked.len(), 4, "the picker takes the double gang: {picked:?}");
        g.clone().declare_blockers(picked).expect("legal");
    }

    /// The wide attack chain's two additions, on one board: two bears
    /// against a lone untapped bear, with a tapped 6/6 behind it so the
    /// race math does not fire (their clock beats ours, so greedy is not
    /// "racing"). Greedy's suicide filter then holds both bears (toughness
    /// 2 against a power-2 blocker), so the menu is bare "nobody" and the
    /// round-55 chain never ran; and a single bear into that blocker is a
    /// straight trade, a tie with staying home, so single growth stops.
    /// The pair move connects: one trades, the other deals two.
    #[test]
    fn attack_chain_wide_overloads_the_lone_blocker_greedy_holds_against() {
        use crate::card::CardType;
        let mut g = two_player_game();
        let mut bears = Vec::new();
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
            bears.push(c);
        }
        let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(wall);
        let fatty = g.add_card_to_battlefield(
            1,
            CardDefinition {
                name: "Colossus Test",
                card_types: vec![CardType::Creature],
                power: 6,
                toughness: 6,
                ..Default::default()
            },
        );
        g.clear_sickness(fatty);
        g.battlefield.iter_mut().find(|c| c.id == fatty).expect("on the battlefield").tapped = true;
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_attacks(&g, 0).is_empty(), "greedy holds both bears");
        let narrow = pick_attacks_scored(&g, 0, &EvalWeights::round55_default());
        assert!(narrow.is_empty(), "the round-55 chain never runs from nobody: {narrow:?}");
        let wide = pick_attacks_scored(&g, 0, &EvalWeights::attack_chain_wide_on());
        assert_eq!(wide.len(), 2, "the pair move overloads the blocker: {wide:?}");
        g.clone().declare_attackers(wide).expect("legal");
        // The pair-move throughput restrictions keep exactly this board:
        // greedy declared nobody and every single addition ties.
        for (name, w) in [
            ("pairs-empty", EvalWeights::attack_pairs_empty_only_on()),
            ("pairs-lazy", EvalWeights::attack_pairs_lazy_on()),
            ("pairs-both", EvalWeights::attack_pairs_both_on()),
        ] {
            let picked = pick_attacks_scored(&g, 0, &w);
            assert_eq!(picked.len(), 2, "{name} still overloads the blocker: {picked:?}");
        }
    }

    /// CR 509.1a — `CantBlock` is enforced from the *computed* set, so a
    /// grant bars a blocker whose printed keywords say nothing. Offering one
    /// gets the whole declaration rejected, which is the bot's entire block
    /// step: four a run on the bench pool before this (PERF (-55),
    /// `combat.rs:1762`).
    #[test]
    fn bot_block_plan_honours_a_granted_cant_block() {
        use crate::card::{CardType, Keyword, StaticAbility};
        use crate::effect::{Selector, StaticEffect};
        let curse = CardDefinition {
            name: "Chill of Fear",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        crate::card::SelectionRequirement::Creature
                            .and(crate::card::SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::CantBlock,
                },
                description: "Creatures you control can't block.",
            }],
            ..Default::default()
        };
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::hill_giant());
        g.clear_sickness(atk);
        g.add_card_to_battlefield(1, curse);
        for _ in 0..2 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.is_empty(), "every blocker is barred by the grant: {blocks:?}");
        g.declare_blockers(blocks).expect("the plan is legal");
    }

    /// CR 508.1d — "attacks each combat if able" is judged against the
    /// *computed* set on both sides. A must-attack creature that is
    /// summoning-sick but hasted by a grant is able, and leaving it home made
    /// the whole declaration illegal.
    #[test]
    fn bot_attack_plan_includes_a_granted_haste_must_attacker() {
        use crate::card::{CardType, Keyword, StaticAbility};
        use crate::effect::{Selector, StaticEffect};
        let haste_anthem = CardDefinition {
            name: "Haste Anthem",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        crate::card::SelectionRequirement::Creature
                            .and(crate::card::SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                },
                description: "Creatures you control have haste.",
            }],
            ..Default::default()
        };
        let forced = CardDefinition {
            name: "Forced Charger",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::MustAttack],
            ..Default::default()
        };
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, haste_anthem);
        let must = g.add_card_to_battlefield(0, forced);
        // Deliberately left summoning-sick: the grant is what makes it able.
        assert!(g.battlefield_find(must).is_some_and(|c| c.summoning_sick));
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let attacks = pick_attacks(&g, 0);
        assert!(
            attacks.iter().any(|a| a.attacker == must),
            "a hasted must-attacker has to be in the batch: {attacks:?}",
        );
        let mut g2 = g.clone();
        g2.declare_attackers(attacks).expect("the plan is legal");
    }

    /// CR 508.1a — creature-ness off the *computed* type line. A bestowed
    /// creature is an Aura while it is attached; the attack planner read the
    /// printed line, declared it as an attacker, and `declare_attackers`
    /// rejected the batch and reported `SummoningSickness` on a card whose
    /// flag was clear — the contradiction that made the lead unreadable for
    /// two sessions. Both halves shipped at `d0d1162d` with no test on the
    /// planner side, so this is a pin, not a failing case.
    #[test]
    fn bot_attack_plan_leaves_a_bestowed_aura_home() {
        let mut g = two_player_game();
        let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(host);
        let aura = g.add_card_to_battlefield(0, catalog::kestia_the_cultivator());
        g.clear_sickness(aura);
        // Bestowed: attached to the host, and an Aura until it comes loose.
        {
            let c = g.battlefield_find_mut(aura).expect("on the battlefield");
            c.bestowed = true;
            c.attached_to = Some(host);
        }
        assert!(
            !g.computed_permanent(aura)
                .is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Creature)),
            "a bestowed permanent is not a creature",
        );
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let attacks = pick_attacks(&g, 0);
        assert!(
            !attacks.iter().any(|a| a.attacker == aura),
            "a bestowed Aura cannot attack: {attacks:?}",
        );
        let mut g2 = g.clone();
        g2.declare_attackers(attacks).expect("the plan is legal");
    }

    /// CR 508.0 — `AttacksAlone`. A creature that attacks alone makes any
    /// batch with a second attacker illegal, so the planner drops it rather
    /// than losing the combat.
    #[test]
    fn bot_attack_plan_honours_attacks_alone() {
        let mut g = two_player_game();
        let loner = g.add_card_to_battlefield(0, catalog::master_of_cruelties());
        g.clear_sickness(loner);
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let attacks = pick_attacks(&g, 0);
        assert!(attacks.len() > 1, "the two bears want to swing: {attacks:?}");
        assert!(
            !attacks.iter().any(|a| a.attacker == loner),
            "an attacks-alone creature cannot ride along: {attacks:?}",
        );
        let mut g2 = g.clone();
        g2.declare_attackers(attacks).expect("the plan is legal");
    }

    /// CR 508.1g — the attack tax is charged per attacker and the engine
    /// rejects the declaration *whole*, so a planner that ignores it loses
    /// its whole combat, not one attacker. PERF (-55): 718 of the 740 attack
    /// rejections on `cube --seed 11` were this gate, and against a
    /// Propaganda the bot never attacked at all.
    #[test]
    fn bot_attack_plan_trims_to_the_payable_attack_tax() {
        let setup = |lands: usize| {
            let mut g = two_player_game();
            g.add_card_to_battlefield(1, catalog::propaganda());
            for _ in 0..2 {
                let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
                g.clear_sickness(c);
            }
            for _ in 0..lands {
                g.add_card_to_battlefield(0, catalog::plains());
            }
            g.step = TurnStep::DeclareAttackers;
            g.active_player_idx = 0;
            g.priority.player_with_priority = 0;
            g
        };
        // Propaganda is {2} per attacker, so the plan is the budget / 2.
        for (lands, want) in [(0usize, 0usize), (2, 1), (4, 2)] {
            let g = setup(lands);
            let attacks = pick_attacks(&g, 0);
            assert_eq!(
                attacks.len(),
                want,
                "{lands} untapped lands fund {want} attackers: {attacks:?}",
            );
            let mut g2 = g.clone();
            g2.declare_attackers(attacks).expect("the plan the picker made is payable");
        }
    }

    /// CR 508.1d — the requirement is three keywords read off the *computed*
    /// set, not one read off the instance. A `MustAttackOrBlock` creature the
    /// suicide filter holds back makes the whole declaration illegal by its
    /// absence, so the repair pass has to put it back.
    #[test]
    fn bot_attack_plan_restores_a_must_attack_or_block_creature() {
        use crate::card::{CardType, Keyword};
        let obliged = CardDefinition {
            name: "Obliged Skirmisher",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::MustAttackOrBlock],
            ..Default::default()
        };
        let mut g = two_player_game();
        let me = g.add_card_to_battlefield(0, obliged);
        g.clear_sickness(me);
        // A blocker that eats it, so the suicide filter drops it first.
        let wall = g.add_card_to_battlefield(1, catalog::hill_giant());
        g.clear_sickness(wall);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let attacks = pick_attacks(&g, 0);
        assert!(
            attacks.iter().any(|a| a.attacker == me),
            "a must-attack-or-block creature is obliged: {attacks:?}",
        );
        let mut g2 = g.clone();
        g2.declare_attackers(attacks).expect("the plan is legal");
    }

    /// CR 508.1d — Ekundu Cyclops joins an attack somebody else started and
    /// is free to stay home otherwise, so the obligation is set-dependent and
    /// the repair has to be a pass rather than a filter.
    #[test]
    fn bot_attack_plan_joins_a_conditional_must_attacker_only_with_company() {
        use crate::card::{CardType, Keyword};
        let conditional = || CardDefinition {
            name: "Reluctant Cyclops",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::MustAttackIfAnotherAttacks],
            ..Default::default()
        };
        let flier = || CardDefinition {
            name: "Free Swinger",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        };
        let setup = |with_company: bool| {
            let mut g = two_player_game();
            let me = g.add_card_to_battlefield(0, conditional());
            g.clear_sickness(me);
            if with_company {
                let f = g.add_card_to_battlefield(0, flier());
                g.clear_sickness(f);
            }
            // Ground-only, so the flier swings free and the 1/1 does not.
            let wall = g.add_card_to_battlefield(1, catalog::hill_giant());
            g.clear_sickness(wall);
            g.step = TurnStep::DeclareAttackers;
            g.active_player_idx = 0;
            g.priority.player_with_priority = 0;
            (g, me)
        };
        let (g, me) = setup(false);
        let alone = pick_attacks(&g, 0);
        assert!(
            !alone.iter().any(|a| a.attacker == me),
            "nobody else is attacking, so it is not obliged: {alone:?}",
        );
        g.clone().declare_attackers(alone).expect("staying home is legal");

        let (g, me) = setup(true);
        let with_company = pick_attacks(&g, 0);
        assert!(
            with_company.len() == 2 && with_company.iter().any(|a| a.attacker == me),
            "the flier's swing obliges it to join: {with_company:?}",
        );
        g.clone().declare_attackers(with_company).expect("the plan is legal");
    }

    /// CR 613 — Ensnaring Bridge caps attacking power at the *controller's*
    /// hand size and is symmetric, so it bites the bot's own board. An
    /// over-cap attacker gets the whole declaration rejected, so the planner
    /// has to leave it home rather than lose the combat.
    #[test]
    fn bot_attack_plan_honours_the_ensnaring_bridge_power_cap() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::ensnaring_bridge());
        let big = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(big);
        g.clear_sickness(small);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Two cards in the Bridge controller's hand: power 3 is barred,
        // power 2 is not.
        for _ in 0..2 {
            g.add_card_to_hand(1, catalog::plains());
        }
        let attacks = pick_attacks(&g, 0);
        assert!(
            attacks.iter().any(|a| a.attacker == small)
                && !attacks.iter().any(|a| a.attacker == big),
            "only the creature under the cap may attack: {attacks:?}",
        );
        g.clone().declare_attackers(attacks).expect("the plan is legal");
        // Empty the hand and nothing may attack at all.
        g.players[1].hand.clear();
        assert!(pick_attacks(&g, 0).is_empty(), "a zero cap bars every attacker");
    }

    /// CR 508.1a — the picker asks the engine which creatures may be
    /// declared, so a restriction it never modelled by hand (Goblin Cohort's
    /// "unless you cast a creature this turn") keeps its creature out of the
    /// batch instead of costing the whole combat. One of the ~16 families the
    /// filter's own copy did not read.
    #[test]
    fn bot_attack_plan_honours_a_restriction_it_never_modelled() {
        use crate::card::{CardType, Keyword};
        let cohort = CardDefinition {
            name: "Cohort Test",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 1,
            keywords: vec![Keyword::CantAttackUnlessCastCreatureThisTurn],
            ..Default::default()
        };
        let mut g = two_player_game();
        let locked = g.add_card_to_battlefield(0, cohort);
        g.clear_sickness(locked);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert_eq!(g.players[0].creatures_cast_this_turn, 0);
        let attacks = pick_attacks(&g, 0);
        assert!(attacks.is_empty(), "the cohort is locked: {attacks:?}");
        g.clone().declare_attackers(attacks).expect("the plan is legal");
    }

    /// CR 508.1d — the attack search's holdbacks are subsets, and a subset
    /// that leaves an obliged attacker home is rejected *whole*: the
    /// candidate's opening dry run fails and it scores nothing, so the menu
    /// silently shrinks. Every candidate has to be a declaration the engine
    /// will accept.
    #[test]
    fn attack_search_candidates_are_all_legal_declarations() {
        use crate::card::{CardType, Keyword};
        let obliged = CardDefinition {
            name: "Rolling Juggernaut",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::MustAttack],
            ..Default::default()
        };
        let mut g = two_player_game();
        let must = g.add_card_to_battlefield(0, obliged);
        g.clear_sickness(must);
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // The open-board shortcut (round 60) would take greedy alone on
        // this blockerless board; the menu's repair is what is under test.
        let w = EvalWeights { attack_search: 3, attack_skip_open: false, ..EvalWeights::default() };
        let candidates = attack_candidates_for_mcts(&g, 0, &w);
        assert!(candidates.len() > 1, "the search has a menu: {candidates:?}");
        for c in &candidates {
            assert!(
                c.iter().any(|a| a.attacker == must),
                "no candidate may leave the must-attacker home: {c:?}",
            );
            g.clone()
                .declare_attackers(c.clone())
                .expect("every candidate is a legal declaration");
        }
    }

    /// The attack chain's finished declaration is one the engine accepts,
    /// obliged attackers included (CR 508.1d): the chain starts from the
    /// repaired "nobody" and every growth step is repaired again.
    #[test]
    fn attack_chain_declaration_is_legal_and_keeps_the_obliged_attacker() {
        use crate::card::{CardType, Keyword};
        let obliged = CardDefinition {
            name: "Rolling Juggernaut",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::MustAttack],
            ..Default::default()
        };
        let mut g = two_player_game();
        let must = g.add_card_to_battlefield(0, obliged);
        g.clear_sickness(must);
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(blocker);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Both seats need something to draw: an empty library decks
        // whoever draws first and pins every candidate to "we won".
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        let w = EvalWeights::attack_chain_on();
        let menu = attack_candidates_for_mcts(&g, 0, &w);
        let (chain, _) = attack_chain_candidate(&g, 0, &w, &menu, &[], attack_chain_pool(&g, 0, &menu[0]), &SimStarts::new(&g, 0, &w)).expect("the start set scores");
        assert!(chain.iter().any(|a| a.attacker == must), "the chain keeps the must-attacker: {chain:?}");
        g.clone().declare_attackers(chain).expect("the chained declaration is legal");
        let picked = pick_attacks_scored(&g, 0, &w);
        assert!(picked.iter().any(|a| a.attacker == must), "so does the picker: {picked:?}");
        g.clone().declare_attackers(picked).expect("the picked declaration is legal");
    }

    /// The chain grows when growing pays: a lone bear into an empty board
    /// is two free damage, so "add it" beats "finalize nobody".
    #[test]
    fn attack_chain_adds_a_free_attacker() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Both seats need something to draw: an empty library decks
        // whoever draws first and pins every candidate to "we won".
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        let w = EvalWeights::attack_chain_on();
        let menu = attack_candidates_for_mcts(&g, 0, &w);
        let (chain, _) = attack_chain_candidate(&g, 0, &w, &menu, &[], attack_chain_pool(&g, 0, &menu[0]), &SimStarts::new(&g, 0, &w)).expect("scored");
        assert_eq!(chain.iter().map(|a| a.attacker).collect::<Vec<_>>(), vec![bear], "{chain:?}");
    }

    /// The cap bounds the chain, and the chain never displaces a better
    /// menu entry: with one addition allowed, three free bears chain to
    /// one attacker, and the picker's argmax still takes greedy's three.
    #[test]
    fn attack_chain_respects_its_cap_and_greedy_keeps_the_argmax() {
        let mut g = two_player_game();
        let mut bears = Vec::new();
        for _ in 0..3 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
            bears.push(c);
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Both seats need something to draw: an empty library decks
        // whoever draws first and pins every candidate to "we won".
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        let w = EvalWeights { attack_chain: 1, ..EvalWeights::attack_chain_on() };
        let menu = attack_candidates_for_mcts(&g, 0, &w);
        let (chain, _) = attack_chain_candidate(&g, 0, &w, &menu, &[], attack_chain_pool(&g, 0, &menu[0]), &SimStarts::new(&g, 0, &w)).expect("scored");
        assert_eq!(chain.len(), 1, "one addition allowed: {chain:?}");
        let picked = pick_attacks_scored(&g, 0, &w);
        assert_eq!(picked.len(), 3, "three free bears: the alpha strike wins the argmax: {picked:?}");
    }

    /// The forward blind spot, pinned so nobody re-derives it: two bears
    /// into one 3/3 at two life are lethal *together* — one is blocked, the
    /// other connects — and each alone just dies. The chain prices each
    /// addition alone, so it finalizes at nobody; the menu's greedy alpha
    /// strike is why the chain extends the menu instead of replacing it.
    #[test]
    fn attack_chain_stops_at_nobody_where_only_the_pair_is_lethal() {
        use crate::card::CardType;
        let wall = CardDefinition {
            name: "Hill Giant Test",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            ..Default::default()
        };
        let mut g = two_player_game();
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        let giant = g.add_card_to_battlefield(1, wall);
        g.clear_sickness(giant);
        g.players[1].life = 2;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Both seats need something to draw: an empty library decks
        // whoever draws first and pins every candidate to "we won".
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        let w = EvalWeights::attack_chain_on();
        let menu = attack_candidates_for_mcts(&g, 0, &w);
        assert_eq!(menu[0].len(), 2, "greedy sees the lethal swing: {menu:?}");
        let (chain, _) = attack_chain_candidate(&g, 0, &w, &menu, &[], attack_chain_pool(&g, 0, &menu[0]), &SimStarts::new(&g, 0, &w)).expect("scored");
        assert!(chain.is_empty(), "each bear alone is a dead bear: {chain:?}");
        let picked = pick_attacks_scored(&g, 0, &w);
        assert_eq!(picked.len(), 2, "the menu still finds lethal: {picked:?}");
    }

    /// The same trim must not fire when there is no tax: `available_mana` is
    /// deliberately optimistic, and a trim that ran on an untaxed board
    /// would decline legal attacks for nothing.
    #[test]
    fn bot_attack_plan_is_untouched_without_a_tax() {
        let mut g = two_player_game();
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert_eq!(pick_attacks(&g, 0).len(), 2, "no tax, no trim");
    }

    /// CR 509.1b — the block planner honours the same cap.
    #[test]
    fn bot_respects_the_silent_arbiter_block_cap() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::silent_arbiter());
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        for _ in 0..3 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        let distinct: crate::fxhash::HashSet<_> = blocks.iter().map(|(b, _)| *b).collect();
        assert!(distinct.len() <= 1, "block plan trimmed to the cap");
        g.declare_blockers(blocks).expect("the trimmed plan is legal");
    }

    /// Under High Alert (team "attack as though no defender"), the bot declares
    /// a Wall as an attacker instead of leaving it home.
    #[test]
    fn bot_attacks_with_wall_under_high_alert() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::high_alert());
        let wall = g.add_card_to_battlefield(0, catalog::wall_of_lost_thoughts()); // 0/4 Defender
        g.clear_sickness(wall);
        g.players[1].life = 3; // the Wall's 4 toughness-damage is lethal
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|d| d.attacker == wall),
                    "Wall should attack (deals its toughness) under High Alert");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A blocker with a computed `CantBlock` (Sandstorm Verge, pacifism) isn't
    /// counted as a threat — the bot swings its 2/2 past a can't-block
    /// deathtouch creature that would otherwise scare it off.
    #[test]
    fn bot_ignores_cant_block_opponents_when_attacking() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        let mut deadly = catalog::grizzly_bears();
        deadly.name = "Pacified Deathtoucher";
        deadly.keywords.push(crate::card::Keyword::Deathtouch);
        deadly.keywords.push(crate::card::Keyword::CantBlock);
        g.add_card_to_battlefield(1, deadly);
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|d| d.attacker == atk),
                    "should swing past a can't-block deathtouch creature");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// Under a global fog (CR 615.1), the bot holds back a non-lethal
    /// attacker whose combat damage would be prevented.
    #[test]
    fn bot_holds_back_attackers_under_fog() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        g.prevent_combat_damage_this_turn = true; // a Fog is active
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.is_empty(), "fogged attacker stays home");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A forced block (MustBeBlocked) with no profitable trade uses the
    /// cheapest legal body, not the bot's best creature.
    #[test]
    fn bot_forced_block_uses_cheapest_body() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        // Seat 0 attacks with a 5/5 that must be blocked.
        let mut atk_def = catalog::grizzly_bears();
        atk_def.name = "Provoker";
        atk_def.power = 5;
        atk_def.toughness = 5;
        atk_def.keywords.push(crate::card::Keyword::MustBeBlocked);
        let atk = g.add_card_to_battlefield(0, atk_def);
        g.clear_sickness(atk);
        // Seat 1 (bot) has a 1/1 chump and a 3/3 — neither can kill the 5/5.
        let mut chump = catalog::grizzly_bears();
        chump.name = "Chump"; chump.power = 1; chump.toughness = 1;
        let chump = g.add_card_to_battlefield(1, chump);
        let mut big = catalog::grizzly_bears();
        big.name = "Big"; big.power = 3; big.toughness = 3;
        let big = g.add_card_to_battlefield(1, big);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(chump, atk)], "forced block uses the 1/1, sparing the 3/3");
        assert!(!blocks.iter().any(|(b, _)| *b == big), "the 3/3 is not thrown away");
    }

    /// CR 702.147 — a Decayed creature can't block, so the bot must not pull
    /// one into a gang block even when its life is on the line.
    #[test]
    fn bot_never_gang_blocks_with_decayed_creature() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let mut big = catalog::grizzly_bears();
        big.name = "Bruiser"; big.power = 6; big.toughness = 6;
        let atk = g.add_card_to_battlefield(0, big);
        g.clear_sickness(atk);
        g.players[1].life = 5; // lethal is on the table → life_threatened
        // Two Decayed 3/3s: enough raw power to "kill" the 6/6 on paper, but
        // they can't legally block.
        let mut zombie = catalog::grizzly_bears();
        zombie.name = "Rotter"; zombie.power = 3; zombie.toughness = 3;
        zombie.keywords.push(crate::card::Keyword::Decayed);
        let z1 = g.add_card_to_battlefield(1, zombie.clone());
        let z2 = g.add_card_to_battlefield(1, zombie);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == z1 || *b == z2),
            "Decayed creatures are never assigned as blockers");
    }

    /// An indestructible blocker walls a big attacker for free (CR 702.12) —
    /// it survives, so the bot blocks even with no life pressure.
    #[test]
    fn bot_walls_with_indestructible_blocker() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let mut atk_def = catalog::grizzly_bears();
        atk_def.name = "Bruiser"; atk_def.power = 5; atk_def.toughness = 5;
        let atk = g.add_card_to_battlefield(0, atk_def);
        g.clear_sickness(atk);
        let mut wall = catalog::grizzly_bears();
        wall.name = "Indestructo"; wall.power = 1; wall.toughness = 1;
        wall.keywords.push(crate::card::Keyword::Indestructible);
        let wall = g.add_card_to_battlefield(1, wall);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, atk)], "indestructible 1/1 walls the 5/5 for free");
    }

    /// CR 702.23 — the bot won't pile a second blocker onto a Rampage attacker:
    /// the +N/+N pump means the extra body dies without helping kill it. A lone
    /// deathtouch blocker already kills it, so the 3/3 stays home.
    #[test]
    fn bot_wont_gang_block_a_rampage_attacker() {
        let mut g = two_player_game();
        let giant = g.add_card_to_battlefield(0, catalog::frost_giant()); // 4/4 rampage 2
        g.clear_sickness(giant);
        let rats = g.add_card_to_battlefield(1, catalog::typhoid_rats()); // 1/1 deathtouch
        let mut big = catalog::grizzly_bears();
        big.name = "Ogre"; big.power = 3; big.toughness = 3;
        g.add_card_to_battlefield(1, big);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: giant, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(rats, giant)],
            "deathtouch alone kills it; no second blocker into the Rampage pump");
    }

    /// The bot won't declare a CanAttackOnlyIfDefenderControls attacker
    /// (Dandân) into a defender whose board fails the filter — doing so
    /// would get the whole batch rejected by the engine.
    #[test]
    fn bot_holds_back_dandan_when_defender_has_no_island() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let dd = g.add_card_to_battlefield(0, catalog::dandan());
        g.clear_sickness(dd);
        g.add_card_to_battlefield(0, catalog::island()); // your Island, not the defender's
        let mut bot = HeuristicBot::new();
        if let Some(GameAction::DeclareAttackers(a)) = bot.next_action(&g, 0) {
            assert!(!a.iter().any(|x| x.attacker == dd),
                "Dandân must not be declared when the defender controls no Island");
        } // declaring no attackers is also fine
        // Now give the defender an Island — Dandân becomes a legal attacker.
        g.add_card_to_battlefield(1, catalog::island());
        let mut bot2 = HeuristicBot::new();
        match bot2.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|x| x.attacker == dd),
                    "Dandân should attack once the defender controls an Island");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A deathtouch attacker swings even when smaller than every blocker —
    /// any block trades the opponent's creature for ours.
    #[test]
    fn bot_attacks_with_deathtouch_into_bigger_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, one_one_with("Stinger", crate::card::Keyword::Deathtouch));
        g.clear_sickness(atk);
        // Two 3/3s — without deathtouch awareness the suicide filter would
        // hold the 1/1 back.
        g.add_card_to_battlefield(1, catalog::hill_giant());
        g.add_card_to_battlefield(1, catalog::hill_giant());
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|atk_decl| atk_decl.attacker == atk),
                    "deathtouch attacker should swing into bigger blockers");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// Magecraft-aware spell bias: when the bot controls a magecraft
    /// permanent and has both an IS spell and a creature spell in hand,
    /// it should prefer the IS spell to fire the magecraft trigger.
    /// Push (claude/modern_decks batch 202).
    #[test]
    fn bot_prefers_is_spell_when_magecraft_in_play() {
        let mut g = two_player_game();
        // Drop Witherbloom Apprentice (a magecraft permanent) on board.
        g.add_card_to_battlefield(0, catalog::witherbloom_apprentice());
        // Hand has both Lightning Bolt (instant) and Grizzly Bears
        // (creature). The bot must prefer the bolt.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let _bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = HeuristicBot::new();
        // Drive the bot until it produces a CastSpell — could pass
        // through PlayLand / mana abilities first if seeded with hand-
        // played lands, but in this synthetic state the next non-mana
        // action is the spell.
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastSpell { card_id, .. } = action {
                assert_eq!(card_id, bolt,
                    "magecraft-bias should pick the instant over the creature");
                return;
            }
            // Drive the engine forward so non-cast actions don't loop.
            let _ = g.perform_action(action);
        }
        panic!("bot never produced a CastSpell action");
    }

    /// The bot casts an Adventure half (Stomp) as removal when it can afford
    /// the adventure but not the creature (CR 715).
    #[test]
    fn bot_casts_adventure_half_as_removal() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::bonecrusher_giant());
        // {1}{R}: enough for Stomp, not the {2}{R} creature.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = HeuristicBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastAdventure { card_id, .. } = action {
                assert_eq!(card_id, id, "bot Stomps with the adventure half");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the adventure half");
    }

    /// CR 702.187 — the bot recasts a card discarded this turn from its
    /// graveyard for the mayhem cost (Electro's Bolt as removal).
    #[test]
    fn bot_casts_mayhem_spell_from_graveyard() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::electros_bolt());
        // Discard the Bolt this turn so its Mayhem cast is legal.
        let mut events = Vec::new();
        g.discard_card(0, bolt, &mut events);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = HeuristicBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastMayhem { card_id, .. } = action {
                assert_eq!(card_id, bolt, "bot recasts Electro's Bolt via Mayhem");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the Mayhem spell");
    }

    /// CR 702.183 — the bot casts an Omen half as removal (Petty Revenge on
    /// Disruptive Stormbrood) when it can't yet afford the Dragon.
    #[test]
    fn bot_casts_omen_half_as_removal() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::disruptive_stormbrood());
        // {1}{B}: enough for Petty Revenge, not the {4}{G} creature.
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = HeuristicBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastOmen { card_id, .. } = action {
                assert_eq!(card_id, id, "bot casts Petty Revenge as removal");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the Omen half");
    }

    /// CR 702.78 — the bot conspires Burn Trail when it controls two untapped
    /// creatures sharing its color, tapping them to copy the spell.
    #[test]
    fn bot_conspires_burn_trail_when_able() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::burn_trail());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.add_card_to_battlefield(0, catalog::goblin_guide());
        g.add_card_to_battlefield(0, catalog::goblin_guide());
        // Second main. Conspire taps the two Goblin Guides, which costs
        // their attack — so casting it *before* combat is genuinely worse,
        // and the default profile's gate correctly declines to. This tests
        // that the bot finds the conspire cast, not that it fires it at the
        // worst possible moment.
        g.step = TurnStep::PostCombatMain;
        let mut bot = HeuristicBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastSpellConspire { card_id, .. } = action {
                assert_eq!(card_id, id, "bot conspires Burn Trail");
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never conspired");
    }

    /// When forced to chump (life threatened, no clean kill), the bot
    /// prefers fully blocking a non-trampler over a trampler — a chump
    /// against a trampler only stops `blocker_toughness` of its damage
    /// (CR 702.19e). Push (claude/modern_decks).
    #[test]
    fn bot_chumps_non_trampler_over_trampler_when_threatened() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        fn beater(name: &'static str, kws: Vec<Keyword>) -> CardDefinition {
            CardDefinition {
                name,
                card_types: vec![CardType::Creature],
                power: 4,
                toughness: 4,
                keywords: kws,
                ..Default::default()
            }
        }
        let mut g = two_player_game();
        let vanilla = g.add_card_to_battlefield(0, beater("Brute", vec![]));
        let trampler = g.add_card_to_battlefield(0, beater("Stomper", vec![Keyword::Trample]));
        // One 0/3 wall that can't kill either — only a chump is possible.
        let wall = g.add_card_to_battlefield(1, beater("Wall", vec![]));
        if let Some(w) = g.battlefield_find_mut(wall) { w.set_definition(std::sync::Arc::new(
            CardDefinition {
                name: "Wall",
                card_types: vec![CardType::Creature],
                toughness: 3,
                ..Default::default()
            })); }
        g.players[1].life = 3; // 8 incoming ≫ 3 → life threatened
        g.attacking = vec![
            Attack { attacker: vanilla, target: AttackTarget::Player(1) },
            Attack { attacker: trampler, target: AttackTarget::Player(1) },
        ];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, vanilla)],
            "chump the non-trampler (saves 4) over the trampler (saves only 3)");
    }

    /// CR 306.7 — the bot chump-blocks to save a planeswalker it controls
    /// when the attackers aimed at it are lethal to its loyalty, even at a
    /// healthy life total. (Push claude/modern_decks.)
    #[test]
    fn bot_chumps_to_save_a_doomed_planeswalker() {
        use crate::card::{CardDefinition, CardType, CounterType};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Raider", card_types: vec![CardType::Creature], power: 3, toughness: 3,
            ..Default::default()
        });
        // The bot (seat 1) controls a low-loyalty planeswalker and a 0/3 wall.
        let pw = g.add_card_to_battlefield(1, CardDefinition {
            name: "Walker", card_types: vec![CardType::Planeswalker], base_loyalty: 2,
            ..Default::default()
        });
        if let Some(c) = g.battlefield_find_mut(pw) {
            c.counters.insert(CounterType::Loyalty, 2);
        }
        let wall = g.add_card_to_battlefield(1, CardDefinition {
            name: "Wall", card_types: vec![CardType::Creature], power: 0, toughness: 3,
            ..Default::default()
        });
        g.players[1].life = 20; // NOT life-threatened — only the walker is at risk.
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Planeswalker(pw) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, atk)],
            "the wall chumps to keep the 3-power attacker off the 2-loyalty walker");
    }

    /// The flip side of the above: when the planeswalker would survive the
    /// swing (loyalty > incoming), the bot doesn't waste a blocker on it.
    #[test]
    fn bot_does_not_chump_for_a_safe_planeswalker() {
        use crate::card::{CardDefinition, CardType, CounterType};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Raider", card_types: vec![CardType::Creature], power: 3, toughness: 3,
            ..Default::default()
        });
        let pw = g.add_card_to_battlefield(1, CardDefinition {
            name: "Walker", card_types: vec![CardType::Planeswalker], base_loyalty: 5,
            ..Default::default()
        });
        if let Some(c) = g.battlefield_find_mut(pw) {
            c.counters.insert(CounterType::Loyalty, 5);
        }
        g.add_card_to_battlefield(1, CardDefinition {
            name: "Wall", card_types: vec![CardType::Creature], power: 0, toughness: 3,
            ..Default::default()
        });
        g.players[1].life = 20;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Planeswalker(pw) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.is_empty(), "3 damage to a 5-loyalty walker isn't worth a chump");
    }

    /// CR 702.147 — a Decayed creature can't block, so the bot must never
    /// offer it as a blocker even when life-threatened (an illegal block
    /// would get the whole DeclareBlockers batch rejected).
    #[test]
    fn bot_never_blocks_with_a_decayed_creature() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Beater",
            card_types: vec![CardType::Creature],
            power: 4,
            toughness: 4,
            ..Default::default()
        });
        let zombie = g.add_card_to_battlefield(1, CardDefinition {
            name: "Decayed Zombie",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Decayed],
            ..Default::default()
        });
        g.players[1].life = 1; // life-threatened → the bot would chump if it could
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == zombie), "decayed creature is never declared as a blocker");
    }

    /// CR 509.1b — facing a "can't be blocked except by three or more" lethal
    /// attacker, the bot either commits ≥3 blockers or none. With exactly three
    /// idle bodies and lethal incoming, it gangs all three (never an illegal
    /// 1–2 block).
    #[test]
    fn bot_meets_min_block_count_for_cant_be_blocked_except_by_n() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Ulamog Spawn",
            card_types: vec![CardType::Creature],
            power: 6,
            toughness: 6,
            keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
            ..Default::default()
        });
        let chumps: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, CardDefinition {
            name: "Chump",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            ..Default::default()
        })).collect();
        g.players[1].life = 1; // lethal incoming
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        let on_atk = blocks.iter().filter(|(_, a)| *a == atk).count();
        assert_eq!(on_atk, 3, "gangs all three to satisfy the 3-blocker minimum");
        assert!(chumps.iter().all(|c| blocks.iter().any(|(b, _)| b == c)));
    }

    /// With only two bodies against the same "≥3 blockers" attacker, the bot
    /// drops the block entirely rather than submit an illegal 2-creature batch.
    #[test]
    fn bot_drops_block_when_min_count_unreachable() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Ulamog Spawn",
            card_types: vec![CardType::Creature],
            power: 6,
            toughness: 6,
            keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
            ..Default::default()
        });
        for _ in 0..2 {
            g.add_card_to_battlefield(1, CardDefinition {
                name: "Chump",
                card_types: vec![CardType::Creature],
                power: 1,
                toughness: 1,
                ..Default::default()
            });
        }
        g.players[1].life = 1;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks.iter().filter(|(_, a)| *a == atk).count(), 0,
            "two blockers can't legally block a ≥3 attacker — declares none");
    }

    /// CR 509.1b — a `CanBlockAnyNumber` wall that kills nothing and isn't
    /// needed against lethal still soaks the whole swing for free: the
    /// spare-capacity pass seeds from every legal blocker, not just the ones
    /// the scoring loop already assigned.
    #[test]
    fn bot_soaks_the_swing_with_an_idle_wall() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let bear = |g: &mut GameState| {
            g.add_card_to_battlefield(
                0,
                CardDefinition {
                    name: "Bear",
                    card_types: vec![CardType::Creature],
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
            )
        };
        let a1 = bear(&mut g);
        let a2 = bear(&mut g);
        let wall = g.add_card_to_battlefield(
            1,
            CardDefinition {
                name: "Big Wall",
                card_types: vec![CardType::Creature],
                power: 0,
                toughness: 9,
                keywords: vec![Keyword::Defender, Keyword::CanBlockAnyNumber],
                ..Default::default()
            },
        );
        g.attacking = vec![
            Attack { attacker: a1, target: AttackTarget::Player(1) },
            Attack { attacker: a2, target: AttackTarget::Player(1) },
        ];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(
            blocks.iter().filter(|(b, _)| *b == wall).count(),
            2,
            "the 0/9 wall eats both attackers"
        );
    }

    /// CR 702.16e — the bot treats a block by a protection-from-the-attacker's
    /// -color creature as a clean kill (it survives + kills) rather than a
    /// suicidal trade, so it blocks even at full life.
    #[test]
    fn bot_blocks_freely_with_protected_creature() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        use crate::mana::{cost, r, Color};
        let mut g = two_player_game();
        let mut red_atk = CardDefinition {
            name: "Red Beater",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            ..Default::default()
        };
        red_atk.cost = cost(&[r()]);
        let atk = g.add_card_to_battlefield(0, red_atk);
        let prot = CardDefinition {
            name: "Warded Blocker",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Protection(Color::Red)],
            ..Default::default()
        };
        let blk = g.add_card_to_battlefield(1, prot);
        // Not life-threatened (only a chump would otherwise be declined).
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(blk, atk)], "protected 3/3 kills the red 3/3 and takes no damage");
    }

    /// The bot won't throw a much bigger creature into an even trade with a
    /// small attacker when it isn't under pressure (keeps the body, takes the
    /// hit). A 5/5 should not block a 5/1 at healthy life.
    #[test]
    fn bot_keeps_big_body_over_bad_even_trade() {
        use crate::card::{CardDefinition, CardType};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let glass = CardDefinition {
            name: "Glass Cannon",
            card_types: vec![CardType::Creature],
            power: 5,
            toughness: 1,
            ..Default::default()
        };
        let atk = g.add_card_to_battlefield(0, glass);
        let beater = CardDefinition {
            name: "Big Beater",
            card_types: vec![CardType::Creature],
            power: 5,
            toughness: 5,
            ..Default::default()
        };
        let big = g.add_card_to_battlefield(1, beater);
        g.players[1].life = 20; // not threatened by 5 damage
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == big),
            "won't trade a 5/5 to kill a 5/1 when healthy");
    }

    /// CR 509.1b — the bot must not assign a power-2 blocker to a Steel Leaf
    /// Champion ("can't be blocked by creatures with power 2 or less"), even
    /// when life-threatened; the legality gate keeps the block batch legal.
    #[test]
    fn bot_skips_illegal_block_against_steel_leaf_champion() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::steel_leaf_champion());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — illegal
        g.players[1].life = 1; // life-threatened, so it would chump if it could
        g.attacking = vec![Attack { attacker: champ, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == bear),
            "power-2 blocker can't be assigned to Steel Leaf Champion");
    }

    /// CR 702.90 / 104.3d — the bot chumps an infect attacker that would
    /// reach 10 poison even at a healthy life total (poison, not life, is the
    /// lethal clock).
    #[test]
    fn bot_chumps_infect_attacker_to_avoid_poison_out() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let infect = CardDefinition {
            name: "Plague Beast",
            card_types: vec![CardType::Creature],
            power: 9,
            toughness: 9,
            keywords: vec![Keyword::Infect],
            ..Default::default()
        };
        let atk = g.add_card_to_battlefield(0, infect);
        let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Healthy life (20) but already 1 poison → 9 incoming poison = 10 → lethal.
        g.players[1].poison_counters = 1;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.iter().any(|(b, _)| *b == chump),
            "bot chumps the infect attacker to avoid a poison-out");
    }

    /// Color-choice mana abilities (Ornithopter of Paradise's `{T}: Add one
    /// mana of any color`) require an interactive `ChooseColor` decision,
    /// which the bot's main loop doesn't supply at activation time. The bot
    /// must never volunteer one as a standalone action.
    #[test]
    fn bot_does_not_tap_color_choice_mana_source() {
        let mut g = two_player_game();
        let bird = g.add_card_to_battlefield(0, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, bird,
                "bot must NOT auto-tap a color-choice mana source (would block on ChooseColor)");
        }
    }

    /// The concern that used to live in `is_free_mana_ability`: a generic
    /// pip must not eat a one-shot artifact or a chunk of life while an
    /// ordinary land sits untapped.
    ///
    /// The bot no longer picks its own mana sources -- it stopped pre-tapping
    /// (see `main_phase_action_with`), so the engine's auto-tap chooses, and
    /// this is the guard on its ordering. Lotus Petal sacrifices itself for
    /// mana; with Forests available to pay the same pips, the Petal survives.
    #[test]
    fn auto_tap_spends_a_land_before_sacrificing_a_mana_source() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        // Two Forests cover the bear's {1}{G} on their own. The Petal sits
        // earlier in the battlefield, so a first-match source pick would
        // sacrifice it for the generic pip anyway.
        let forests: Vec<_> = (0..2)
            .map(|_| {
                let f = g.add_card_to_battlefield(0, catalog::forest());
                g.clear_sickness(f);
                f
            })
            .collect();
        g.add_card_to_hand(0, catalog::grizzly_bears());
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        assert!(
            matches!(action, GameAction::CastSpell { .. }),
            "the bear is affordable off the two Forests, got {action:?}",
        );
        g.perform_action(action).expect("the bear should be castable");
        assert!(
            g.battlefield_find(petal).is_some(),
            "Lotus Petal must survive when lands could pay instead",
        );
        assert_eq!(
            forests.iter().filter(|f| g.battlefield_find(**f).is_some_and(|c| c.tapped)).count(),
            2,
            "both Forests are what should have been tapped",
        );
    }

    /// A trim's budget is measured, not estimated, and the two differ in
    /// **both** directions — which is why `payable_generic_budget` exists
    /// rather than a one-sided correction to [`available_mana`].
    ///
    /// Downward, deliberately: a Lotus Petal is not spare mana to the
    /// estimate (the test below pins that, and it is right for *casting*
    /// decisions), but the engine's auto-tap will sacrifice it to pay a tax
    /// the rules already forced on the bot. Trimming an attacker for mana
    /// that is there loses a swing for nothing.
    ///
    /// Upward, accidentally: that direction is the one with teeth, because
    /// the engine rejects the declaration **whole** — see
    /// `payable_generic_budget`'s doc for the `cube` seed 2 shape.
    #[test]
    fn a_trims_budget_is_measured_not_estimated() {
        let mut g = two_player_game();
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        assert_eq!(available_mana(&g, 0).total, 0, "the estimate does not count a Petal");
        assert!(g.could_pay_generic(0, 1), "but the engine's auto-tap sacrifices it");
        assert_eq!(
            payable_generic_budget(&g, 0, 1),
            1,
            "so a trim that used the estimate would drop a swing for mana that is there",
        );
        // And the cap holds: the budget never exceeds what was asked for, so
        // the binary search cannot run away on a board with no tax.
        assert_eq!(payable_generic_budget(&g, 0, 0), 0);
        let empty = two_player_game();
        assert_eq!(payable_generic_budget(&empty, 0, 4), 0, "nothing on the board pays nothing");
    }

    /// `(life + clock - 1) / clock` overflows on an `i32::MAX` life total,
    /// which a Beacon of Immortality board reaches — ENGINE_BACKLOG's closed
    /// stall lead, a correct card doing what it prints. In release the wrap is
    /// silent and negative, which reads as "we lose next turn" and turns the
    /// race check inside out. Caught by the `debug-assertions` sweep at seeds
    /// 53 and 73 of `--decks all`.
    #[test]
    fn turns_to_lethal_does_not_overflow_on_an_unbounded_life_total() {
        // The shape that wrapped, at every clock a board can present.
        for clock in [1, 2, 7, 4_091] {
            let t = super::turns_to_lethal(i32::MAX, clock);
            assert!(t > 0, "clock {clock} gave {t}");
        }
        // ...and it still agrees with the arithmetic it replaces everywhere
        // that arithmetic was defined.
        for life in [1, 2, 3, 19, 20, 21, 100] {
            for clock in [1, 2, 3, 7] {
                assert_eq!(
                    super::turns_to_lethal(life, clock),
                    (life + clock - 1) / clock,
                    "life {life} clock {clock}",
                );
            }
        }
        // A dead-or-negative life total is one swing, not zero or negative.
        assert_eq!(super::turns_to_lethal(0, 3), 1);
        assert_eq!(super::turns_to_lethal(-7, 3), 1);
    }

    /// Sac-cost sources are deliberately *not* counted toward what the bot
    /// can afford: it would be committing to lines it can only pay for by
    /// spending something it would rather keep. A Lotus Petal on its own
    /// does not make a two-drop look castable.
    #[test]
    fn available_mana_ignores_self_consuming_sources() {
        let mut g = two_player_game();
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        assert_eq!(available_mana(&g, 0).total, 0, "a Lotus Petal is not spare mana");
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.clear_sickness(forest);
        assert_eq!(available_mana(&g, 0).total, 1, "only the Forest counts");
    }

    /// Reproducer for the "Vandalblast freeze" bug. The bot is in its main
    /// phase with a Mountain (already tapped or untapped) and Vandalblast in
    /// hand; the human opponent has only an Ornithopter of Paradise on the
    /// battlefield. The bot must pick that artifact as the target and the
    /// match must drive to completion without spinning the bot loop.
    #[test]
    fn bot_vs_bot_vandalblast_against_lone_artifact_resolves() {
        use crate::server::{run_match, SeatOccupant};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;
        let mut g = two_player_game();
        // Bot owns a Mountain so it can pay {R} and Vandalblast in hand.
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        g.clear_sickness(mtn);
        g.add_card_to_hand(0, catalog::vandalblast());
        // Opponent has only Ornithopter of Paradise on the battlefield.
        let bird = g.add_card_to_battlefield(1, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        // Both bots; expect the match to terminate within a short window.
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                g,
                vec![
                    SeatOccupant::Bot(Box::new(HeuristicBot::new())),
                    SeatOccupant::Bot(Box::new(HeuristicBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(15))
            .expect("bot-vs-bot match must terminate (Vandalblast freeze regression)");
        handle.join().unwrap();
    }

    /// Direct (non-server) regression: the bot's main-phase action loop
    /// picks the opponent's Ornithopter as the legal Vandalblast target
    /// when no other artifact is in play. The Mountain has already been
    /// tapped (we seed the pool with {R} and pre-tap the land) so the
    /// bot proceeds straight to the spell-cast step.
    #[test]
    fn bot_main_phase_emits_vandalblast_action() {
        let mut g = two_player_game();
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        if let Some(c) = g.battlefield_find_mut(mtn) {
            c.tapped = true;
        }
        let vandal = g.add_card_to_hand(0, catalog::vandalblast());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        let bird = g.add_card_to_battlefield(1, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        match action {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, vandal, "expected the bot to cast Vandalblast");
                assert_eq!(
                    target,
                    Some(Target::Permanent(bird)),
                    "Vandalblast must target the lone artifact opp controls",
                );
            }
            other => panic!("expected CastSpell(Vandalblast), got {other:?}"),
        }
    }

    /// The bot uses Magma Opus's discard-a-card-for-a-Treasure mode as a
    /// fallback value play when the full {6}{U}{R} spell is unaffordable.
    #[test]
    fn bot_uses_discard_activated_ability_as_fallback() {
        let mut g = two_player_game();
        let opus = g.add_card_to_hand(0, catalog::magma_opus());
        // Only {U/R}{U/R} worth of mana — can't cast the {8} spell.
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        match action {
            GameAction::ActivateDiscardAbility { card_id } => assert_eq!(card_id, opus),
            other => panic!("expected ActivateDiscardAbility(Magma Opus), got {other:?}"),
        }
    }

    /// End-to-end deadlock regression for spectate-mode bot-vs-bot:
    /// load a hand-crafted state that mirrors the captured cube debug
    /// export (own-stack trigger + sorcery-speed castables + a played
    /// land already) and assert the match drives forward instead of
    /// hanging on `merged_rx.recv()`. Pre-fix this would have hung on
    /// any RNG that picked Tireless Tracker before Lightning Bolt.
    #[test]
    fn spectate_match_does_not_deadlock_with_own_trigger_on_stack() {
        use crate::effect::Effect;
        use crate::game::TurnStep;
        use crate::server::{run_match, SeatOccupant};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
        g.clear_sickness(tracker);
        g.stack.push(TriggerPush::new(tracker, 0, Effect::Noop).build());
        g.add_card_to_hand(0, catalog::tireless_tracker());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 5);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
        g.players[0].lands_played_this_turn = 1;
        // Both players at 1 life so combat damage ends the match
        // quickly once a creature attacks.
        g.players[0].life = 1;
        g.players[1].life = 1;

        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                g,
                vec![
                    SeatOccupant::Bot(Box::new(HeuristicBot::new())),
                    SeatOccupant::Bot(Box::new(HeuristicBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(15))
            .expect("bot-vs-bot match must terminate (own-stack-trigger deadlock regression)");
        handle.join().unwrap();
    }

    /// Regression for the Spectate Bot vs Bot deadlock observed in
    /// `debug/state-t11-precombatmain-1777409468-338551100.json`.
    ///
    /// Setup: bot 0 has its own Tireless Tracker trigger sitting on the
    /// stack (no target), all its lands are tapped and one was already
    /// played this turn, and its hand has both sorcery- and instant-
    /// speed castables. Pre-fix, `main_phase_action` sometimes picked a
    /// sorcery to cast — the engine rejected it with `SorcerySpeedOnly`
    /// (stack non-empty), `drive_bots` saw no progress, the actor blocked
    /// on `merged_rx.recv()`, and a spectator-only match froze.
    ///
    /// Post-fix the bot must either pass priority or cast an instant —
    /// never a sorcery — when the stack is non-empty.
    #[test]
    fn bot_does_not_attempt_sorcery_when_stack_nonempty() {
        use crate::effect::Effect;
        let mut g = two_player_game();
        // Bot 0 has a tracker on the battlefield as the trigger source.
        let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
        g.clear_sickness(tracker);
        // Stack: Tireless Tracker trigger (Clue creation), no target.
        g.stack.push(TriggerPush::new(tracker, 0, Effect::Noop).build());
        // Hand: a mix of sorcery- and instant-speed castables. Pyrokinesis
        // (instant) is the only legal cast right now.
        g.add_card_to_hand(0, catalog::tireless_tracker());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        // Mana pool topped up so `can_afford` accepts both.
        g.players[0].mana_pool.add(crate::mana::Color::Green, 5);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
        // Pretend a land was played already so PlayLand is also blocked.
        g.players[0].lands_played_this_turn = 1;

        let mut bot = HeuristicBot::new();
        // Drive a few action picks; none of them may be a sorcery-speed
        // CastSpell (Tireless Tracker). PassPriority and instant casts
        // (Lightning Bolt) are both fine.
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { card_id, .. } = action {
                let def = g.players[0].hand.iter().find(|c| c.id == card_id)
                    .map(|c| &c.definition);
                if let Some(d) = def {
                    assert!(
                        d.is_instant_speed(),
                        "bot tried to cast sorcery-speed {} while stack was non-empty",
                        d.name,
                    );
                }
            }
        }
    }

    /// Regression for the Teferi sorcery-lock deadlock. With Teferi,
    /// Time Raveler on the opponent's side, our **instants** are
    /// timing-locked to sorcery speed. The bot's pre-fix filter
    /// allowed instant casts whenever `is_instant_speed()` was true,
    /// regardless of `OpponentsSorceryTimingOnly`; the engine then
    /// rejected with `SorcerySpeedOnly` and the match deadlocked.
    /// Post-fix, `would_accept` dry-runs the cast and rejects it,
    /// so the bot picks a different action (or passes priority).
    #[test]
    fn bot_respects_teferi_sorcery_lock_on_instants() {
        let mut g = two_player_game();
        // Opponent's Teferi imposes `OpponentsSorceryTimingOnly`.
        let teferi = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
        g.clear_sickness(teferi);
        // Stack non-empty so sorcery-speed timing fails for the bot.
        g.spells_cast_this_turn = 0;
        // Put a dummy spell on the stack to break sorcery timing
        // even on the bot's main phase.
        let dummy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.retain(|c| c.id != dummy);
        let card = crate::card::CardInstance::new(dummy, catalog::grizzly_bears(), 1);
        g.stack.push(crate::game::StackItem::Spell {
            card: Box::new(card),
            caster: 1,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            uncounterable: false,
        });
        // Bot 0 has Lightning Bolt (instant) in hand and a Mountain.
        let _bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        let mut bot = HeuristicBot::new();
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { .. } = action {
                panic!(
                    "bot tried to cast at instant speed under Teferi's lock — \
                     would_accept must filter this out: {action:?}",
                );
            }
        }
    }

    /// Regression for the deadlock at `debug/deadlock-t8-1777411577-473115700.json`.
    /// Damping Sphere on the battlefield + bot has already cast one spell this
    /// turn + a second affordable-by-printed-cost spell in hand whose real cost
    /// (printed + Damping Sphere's `+1` tax) overflows the pool. Pre-fix the
    /// bot's `can_afford` checked only the printed cost; cast was rejected with
    /// `Mana: Need N generic mana but only have N-1 total`; spectate-mode actor
    /// deadlocked. Post-fix `can_afford_in_state` folds the static-ability tax
    /// into the cost so the bot doesn't pick the unaffordable spell.
    #[test]
    fn bot_respects_damping_sphere_tax() {
        let mut g = two_player_game();
        // Opponent's Damping Sphere on the battlefield.
        let sphere = g.add_card_to_battlefield(1, catalog::damping_sphere());
        g.clear_sickness(sphere);
        // Bot 0 has cast one spell already this turn.
        g.players[0].spells_cast_this_turn = 1;
        g.spells_cast_this_turn = 1;
        // Bot 0 has Frantic Search ({2}{U}) in hand and exactly 3 mana
        // (1U + 2C). Without the Damping Sphere tax the bot could
        // pay {2}{U}; with the +1 tax it can't (needs {3}{U} total).
        let _frantic = g.add_card_to_hand(0, catalog::frantic_search());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);

        let mut bot = HeuristicBot::new();
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { card_id, .. } = action {
                let name = g
                    .players[0]
                    .hand
                    .iter()
                    .find(|c| c.id == card_id)
                    .map(|c| c.definition.name);
                assert_ne!(
                    name,
                    Some("Frantic Search"),
                    "bot must respect Damping Sphere's +1 tax — pool can't pay {{3}}{{U}}",
                );
            }
        }
    }

    /// The bot's affordability check folds in generic cost reductions:
    /// Tolarian Terror ({6}{U}) is castable on {3}{U} with three instants/
    /// sorceries in the graveyard.
    #[test]
    fn bot_affordability_honors_graveyard_affinity() {
        let mut g = two_player_game();
        let terror = g.add_card_to_hand(0, catalog::tolarian_terror());
        let card = g.players[0].hand.iter().find(|c| c.id == terror).unwrap().clone();
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3); // {3}{U} only
        assert!(!can_afford_in_state(&g, 0, &card, &EvalWeights::default()), "no discount yet → unaffordable");
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
        let card = g.players[0].hand.iter().find(|c| c.id == terror).unwrap().clone();
        assert!(can_afford_in_state(&g, 0, &card, &EvalWeights::default()), "−{{3}} discount → now affordable");
    }

    /// Regression for the second deadlock observed at
    /// `debug/deadlock-t15-1777411082-269586900.json`. Setup mirrors
    /// the captured cube state: P0 owns a Swamp whose `controller` has
    /// flipped to P1 (Threaten / Mind Control style), all of P0's own
    /// lands are tapped. Pre-fix the bot's main_phase_action filter
    /// (`c.owner == seat`) picked the stolen Swamp, `activate_ability`
    /// rejected with `NotYourPriority`, no progress was made, and the
    /// wall-clock watchdog tripped. Post-fix the filter is keyed on
    /// `c.controller`, so the stolen land is invisible to bot 0 and
    /// the bot falls through to its castable-spell branch (or
    /// `PassPriority`).
    #[test]
    fn max_affordable_x_returns_zero_for_non_x_spells() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 0,
            "Non-X spell yields 0 — caller should pass x_value=None");
    }

    #[test]
    fn max_affordable_x_pumps_remaining_mana_into_x() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire()); // {X}{R}
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Pool: 1 red + 4 colorless. Fixed cost = {R} (1 mana). X = 4.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 4,
            "X soaks up the remaining {{4}} after the fixed {{R}} pip");
    }

    #[test]
    fn max_affordable_x_is_zero_if_only_fixed_cost_can_be_paid() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire());
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Only enough mana for the {R} pip — X must collapse to 0.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 0);
    }

    #[test]
    fn bot_casts_x_cost_burn_at_max_x() {
        // Banefire's catalog cost is just `{R}` (X is read at resolution
        // from `Value::XFromCost`), so x_relevant() picks it up via the
        // effect-tree XFromCost reference and the bot pumps the rest of
        // its pool into X.
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Verify the helper independently first — the bot's `next_action`
        // gates on lots of other state (priority, lands, mana rocks) so
        // a direct call to the helper is the most reliable assertion.
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 3,
            "{{R}} + {{3}} in pool, fixed cost {{R}} => X = 3");
    }

    /// CR 702.51 — the bot taps creatures for convoke when the pool alone
    /// can't cover the spell.
    #[test]
    fn bot_taps_creatures_for_convoke() {
        // Triplicate Spirits ({4}{W}{W}, convoke) with only {W}{W} floating:
        // unaffordable outright, castable by tapping four creatures.
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let id = g.add_card_to_hand(0, catalog::triplicate_spirits());
        g.players[0].mana_pool.add(crate::mana::Color::White, 2);
        for _ in 0..4 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(c).unwrap().summoning_sick = false;
        }
        match main_phase_action(&g, 0) {
            GameAction::CastSpellConvoke { card_id, convoke_creatures, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(convoke_creatures.len(), 4);
            }
            other => panic!("expected a convoke cast, got {other:?}"),
        }
    }

    /// The convoke planner taps the fewest (and least useful) helpers it needs.
    #[test]
    fn bot_taps_the_minimum_number_of_convoke_helpers() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let id = g.add_card_to_hand(0, catalog::triplicate_spirits()); // {4}{W}{W}
        g.players[0].mana_pool.add(crate::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        // Six bodies available but only {2} of the generic is unpaid, and the
        // summoning-sick ones should be spent first.
        let mut sick = Vec::new();
        for i in 0..6 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            if i < 2 {
                sick.push(c);
            } else {
                g.battlefield_find_mut(c).unwrap().summoning_sick = false;
            }
        }
        match main_phase_action(&g, 0) {
            GameAction::CastSpellConvoke { card_id, convoke_creatures, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(convoke_creatures.len(), 2, "only the unpaid {{2}} needs help");
                assert!(
                    convoke_creatures.iter().all(|c| sick.contains(c)),
                    "the summoning-sick bodies tap first",
                );
            }
            other => panic!("expected a convoke cast, got {other:?}"),
        }
    }

    /// Chief Engineer's granted convoke reaches the bot's planner too.
    #[test]
    fn bot_taps_creatures_for_granted_convoke() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::perilous_vault()); // {4} artifact
        g.add_card_to_battlefield(0, catalog::chief_engineer());
        for _ in 0..4 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(c).unwrap().summoning_sick = false;
        }
        match main_phase_action(&g, 0) {
            GameAction::CastSpellConvoke { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a granted-convoke cast, got {other:?}"),
        }
    }

    #[test]
    fn bot_casts_spectacle_when_opponent_bled() {
        // Skewer the Critics ({2}{R}, Spectacle {R}) with only {R} in the pool:
        // unaffordable at its printed cost, but castable for Spectacle once an
        // opponent has lost life this turn.
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let id = g.add_card_to_hand(0, catalog::skewer_the_critics());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.adjust_life(1, -1); // opponent bleeds → Spectacle online
        match main_phase_action(&g, 0) {
            GameAction::CastSpellAlternative { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a Spectacle alternative cast, got {other:?}"),
        }
    }

    /// The bot casts an MDFC's back face from hand when the front is
    /// unaffordable: Wandering Archaic ({5} creature) // Explore the Vastlands
    /// ({4} sorcery), with only {4} in the pool.
    #[test]
    fn bot_casts_mdfc_back_face_from_hand() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        g.players[0].hand.clear();
        let id = g.add_card_to_hand(0, catalog::wandering_archaic());
        g.players[0].mana_pool.add_colorless(4); // affords the {4} back, not the {5} front
        match main_phase_action(&g, 0) {
            GameAction::CastSpellBack { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a back-face cast, got {other:?}"),
        }
    }

    /// The bot casts an MDFC's back face from the graveyard when it carries the
    /// `may_cast_back_from_graveyard` permission (Pestilent Cauldron after its
    /// sacrifice → Restorative Burst).
    #[test]
    fn bot_casts_mdfc_back_face_from_graveyard() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        g.players[0].hand.clear();
        let pc = g.add_card_to_graveyard(0, catalog::pestilent_cauldron());
        g.players[0]
            .graveyard
            .iter_mut()
            .find(|c| c.id == pc)
            .unwrap()
            .may_cast_back_from_graveyard = true;
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3); // {3}{G}{G} for Restorative Burst
        match main_phase_action(&g, 0) {
            GameAction::CastSpellBack { card_id, .. } => assert_eq!(card_id, pc),
            other => panic!("expected a graveyard back-face cast, got {other:?}"),
        }
    }

    /// The bot activates an Unearth ability (CR 702.84) from its graveyard when
    /// it can afford it (a `from_graveyard` activated ability).
    #[test]
    fn bot_unearths_from_graveyard() {
        let mut g = two_player_game();
        g.players[0].hand.clear();
        let dragger = g.add_card_to_graveyard(0, catalog::viscera_dragger());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1); // {1}{B} unearth cost
        match main_phase_action(&g, 0) {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, dragger),
            other => panic!("expected an unearth activation, got {other:?}"),
        }
    }

    #[test]
    fn bot_does_not_try_to_tap_stolen_land() {
        let mut g = two_player_game();
        // P0's own Swamp: tapped (already used this turn).
        let own = g.add_card_to_battlefield(0, catalog::swamp());
        if let Some(c) = g.battlefield_find_mut(own) {
            c.tapped = true;
        }
        // P0-owned Swamp now controlled by P1 (the deadlock state).
        let stolen = g.add_card_to_battlefield(0, catalog::swamp());
        if let Some(c) = g.battlefield_find_mut(stolen) {
            c.controller = 1;
            c.tapped = false;
        }

        let mut bot = HeuristicBot::new();
        // 50 trials; if the bot ever returns ActivateAbility on the
        // stolen card it would deadlock. PassPriority and any action
        // on a card the bot actually controls are both fine.
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::ActivateAbility { card_id, .. } = action {
                assert_ne!(
                    card_id, stolen,
                    "bot must not try to activate a stolen permanent",
                );
            }
        }
    }

    /// Modal spells: when the default mode is dead in the current state
    /// (e.g. Drown in the Loch's mode 0 "counter target spell" with no
    /// opp spell on the stack), the bot picks an alternate mode that
    /// has a legal target. Pre-fix the bot always passed `mode: None`
    /// → engine defaulted to mode 0 → cast was rejected at target
    /// validation, and Drown in the Loch was never cast.
    #[test]
    fn bot_picks_alternate_mode_for_modal_spell() {
        let mut g = two_player_game();
        // Opp creature for mode-1 (destroy creature) to target. Drown's
        // MV gate needs MV(bear=2) ≤ cards in its controller's graveyard.
        g.add_card_to_graveyard(1, catalog::forest());
        g.add_card_to_graveyard(1, catalog::forest());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // Tap an Island/Swamp so {U}{B} is in the pool — bot's land-tap
        // branch otherwise fires first.
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.add_card_to_hand(0, catalog::drown_in_the_loch());
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        // The bot should cast Drown in the Loch with mode = Some(1)
        // (destroy mode). Mode 0 (counter spell) has no spell on the
        // stack, so it's pruned from the candidate set.
        match action {
            GameAction::CastSpell { mode, target, .. } => {
                assert_eq!(mode, Some(1),
                    "Bot should pick mode 1 when mode 0 has no legal target");
                assert_eq!(target, Some(crate::game::Target::Permanent(bear)),
                    "Mode 1's target should be the opp creature");
            }
            other => panic!(
                "expected Drown in the Loch cast with mode 1, got {:?}", other),
        }
    }

    /// `modal_mode_count`: returns the mode count for ChooseMode and
    /// None for non-modal effects.
    #[test]
    fn modal_mode_count_helper() {
        let drown = catalog::drown_in_the_loch();
        assert_eq!(modal_mode_count(&drown.effect), Some(2),
            "Drown in the Loch has 2 modes");
        let bolt = catalog::lightning_bolt();
        assert_eq!(modal_mode_count(&bolt.effect), None,
            "Lightning Bolt is not modal");
    }

    /// The bot delves a stocked graveyard to cast a spell it couldn't afford
    /// at full cost (CR 702.66). Treasure Cruise ({7}{U}) with only one blue
    /// mana but seven graveyard cards must surface as a `CastSpellDelve`.
    #[test]
    fn bot_delves_to_afford_treasure_cruise() {
        let mut g = two_player_game();
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::island()); }
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_hand(0, catalog::treasure_cruise());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        // Drive the bot until it produces the delve cast (it may tap/scan
        // first, but with no lands and one floating U the delve is the only
        // castable line).
        let mut bot = HeuristicBot::new();
        let mut found = false;
        for _ in 0..6 {
            match bot.next_action(&g, 0) {
                Some(GameAction::CastSpellDelve { delve_cards, .. }) => {
                    assert!(!delve_cards.is_empty(), "delved at least one card");
                    found = true;
                    break;
                }
                Some(other) => { g.perform_action(other).ok(); }
                None => break,
            }
        }
        assert!(found, "bot should delve to cast Treasure Cruise");
    }

    /// The bot fetches toward its weakest color: with two Forests already
    /// down and a Forest + Island in the library, it grabs the Island.
    #[test]
    fn bot_search_fetches_weakest_color_basic() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::forest());
        let extra_forest = g.add_card_to_library(0, catalog::forest());
        let island = g.add_card_to_library(0, catalog::island());
        let candidates = vec![(extra_forest, "Forest".into()), (island, "Island".into())];
        let ans = decide_library_search(&g, 0, &candidates, &EvalWeights::block_gang_search());
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == island),
            "bot fetches the Island (Blue uncovered) over a third Forest");
    }

    /// Round 51: the fetch reads *demand*, not supply alone. Two Mountains
    /// and one Forest are down and the hand is three red spells, so the
    /// colour we own least of (green) is not the colour we need. The
    /// pre-fix ranking took the Forest on the supply count alone; both
    /// sides are pinned here so `legacyfetch` stays a faithful control.
    #[test]
    fn fetch_prefers_the_color_the_hand_is_asking_for() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(0, catalog::forest());
        for _ in 0..3 {
            g.add_card_to_hand(0, catalog::lightning_bolt()); // {R}
        }
        let forest = g.add_card_to_library(0, catalog::forest());
        let mountain = g.add_card_to_library(0, catalog::mountain());
        let candidates = vec![(forest, "Forest".into()), (mountain, "Mountain".into())];

        let fixed = decide_library_search(&g, 0, &candidates, &EvalWeights::block_gang_search());
        assert!(matches!(fixed, DecisionAnswer::Search(Some(id)) if id == mountain),
            "demand-aware: three red pips in hand outrank the thin green source");

        let legacy = decide_library_search(&g, 0, &candidates, &EvalWeights::legacy_fetch_on());
        assert!(matches!(legacy, DecisionAnswer::Search(Some(id)) if id == forest),
            "legacy control: supply-only, so it still takes the scarcer Forest");
    }

    /// Round 51: a tutor fetches something it can actually cast. With two
    /// lands down, the MV-5 Angel is a card that sits in hand; the MV-2
    /// body is a play. The pre-fix ranking took the biggest hit regardless.
    #[test]
    fn tutor_prefers_a_castable_hit_over_the_biggest_one() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::plains());
        g.add_card_to_battlefield(0, catalog::plains());
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
        let angel = g.add_card_to_library(0, catalog::serra_angel());   // MV 5
        let candidates =
            vec![(bears, "Grizzly Bears".into()), (angel, "Serra Angel".into())];

        let fixed = decide_library_search(&g, 0, &candidates, &EvalWeights::block_gang_search());
        assert!(matches!(fixed, DecisionAnswer::Search(Some(id)) if id == bears),
            "castability leads: the Angel is uncastable off two lands");

        let legacy = decide_library_search(&g, 0, &candidates, &EvalWeights::legacy_fetch_on());
        assert!(matches!(legacy, DecisionAnswer::Search(Some(id)) if id == angel),
            "legacy control: biggest mana value regardless of castability");
    }

    /// Round 51: the runners-up reach the search. `rank_library_search`
    /// exists so `fetch_arms` can offer more than the heuristic's single
    /// pick — a ranking that collapsed to one entry would make the flag a
    /// no-op, which is exactly how the first mulligan-sim test passed
    /// vacuously.
    #[test]
    fn fetch_ranking_offers_every_legal_hit() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mountain());
        let forest = g.add_card_to_library(0, catalog::forest());
        let island = g.add_card_to_library(0, catalog::island());
        let mountain = g.add_card_to_library(0, catalog::mountain());
        let candidates = vec![
            (forest, "Forest".into()),
            (island, "Island".into()),
            (mountain, "Mountain".into()),
        ];
        let ranked =
            rank_library_search(&g, 0, &candidates, &EvalWeights::block_gang_search());
        assert_eq!(ranked.len(), 3, "all three basics are arms, not just the best one");
        assert!(ranked.contains(&mountain) && ranked.contains(&forest));
    }

    /// The bot's ChooseTarget heuristic votes/targets the opponent's biggest
    /// permanent, not the first legal one.
    #[test]
    fn bot_choose_target_hits_opponents_biggest() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut dino = catalog::grizzly_bears();
        dino.name = "Dino"; dino.power = 6; dino.toughness = 6;
        let big = g.add_card_to_battlefield(1, dino);
        let legal = vec![Target::Permanent(small), Target::Permanent(big)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Permanent(id)) => {
                assert_eq!(id, big, "bot targets the 6/6 over the 2/2");
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// Among player targets the bot picks the lowest-life opponent.
    #[test]
    fn bot_choose_target_hits_lowest_life_opponent() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = crate::game::multi_player_game(3);
        g.players[1].life = 15;
        g.players[2].life = 6;
        let legal = vec![Target::Player(1), Target::Player(2)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Player(p)) => {
                assert_eq!(p, 2, "targets the 6-life opponent over the 15-life one");
            }
            other => panic!("expected a player target, got {other:?}"),
        }
    }

    /// Forced to choose among its own permanents, the bot gives up the smallest.
    #[test]
    fn bot_choose_target_sacrifices_own_smallest() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let mut dino = catalog::grizzly_bears();
        dino.name = "Dino"; dino.power = 6; dino.toughness = 6;
        let big = g.add_card_to_battlefield(0, dino);
        let legal = vec![Target::Permanent(big), Target::Permanent(small)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Permanent(id)) => {
                assert_eq!(id, small, "bot gives up its 2/2, keeps the 6/6");
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// A forced sacrifice gives up a spare token before a real land, even
    /// though the token's raw power/toughness makes it "bigger."
    #[test]
    fn bot_sacrifices_token_before_a_land() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let mut tok = catalog::grizzly_bears(); // a 2/2 body...
        tok.name = "Bear Token";
        let token = g.add_card_to_battlefield(0, tok);
        g.battlefield_find_mut(token).unwrap().is_token = true; // ...but a token
        let legal = vec![Target::Permanent(land), Target::Permanent(token)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Permanent(id)) => {
                assert_eq!(id, token, "bot sacrifices the token, keeps the land");
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// With no basic land among the candidates the bot still fetches the
    /// first option rather than fizzling like AutoDecider.
    #[test]
    fn bot_search_fetches_nonland_when_no_basic_offered() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        let candidates = vec![(bolt, "Lightning Bolt".into())];
        let ans = decide_library_search(&g, 0, &candidates, &EvalWeights::block_gang_search());
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == bolt),
            "bot fetches the only candidate");
    }

    /// A non-land tutor (e.g. Fauna Shaman) fetches the highest-mana-value
    /// hit — the most impactful card — not just the first candidate offered.
    #[test]
    fn bot_search_fetches_highest_mv_nonland() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
        let angel = g.add_card_to_library(0, catalog::serra_angel());   // MV 5
        let candidates = vec![
            (bears, "Grizzly Bears".into()),
            (angel, "Serra Angel".into()),
        ];
        let ans = decide_library_search(&g, 0, &candidates, &EvalWeights::block_gang_search());
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == angel),
            "bot fetches the higher-MV creature");
    }

    /// The bot offers a Bestow cast (enchanting its own creature) when it's
    /// mana-flush, instead of only ever casting the base creature.
    #[test]
    fn bot_considers_bestow_when_mana_flush() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::hopeful_eidolon());
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        // The bot can cast Hopeful Eidolon normally *or* bestow it; the
        // scored pick prefers the bestow line (variant bonus + own-target
        // gain), so it must win outright, not merely appear.
        let bestowed = (0..10).all(|_| {
            matches!(main_phase_action(&g, 0),
                GameAction::CastBestow { target: Some(crate::game::Target::Permanent(t)), .. } if t == host)
        });
        assert!(bestowed, "scored pick prefers the Bestow line enchanting its creature");
    }

    /// `decide_choose_cards` over the bot's own hand (Sneak Attack / Elvish
    /// Piper) cheats in the biggest creature it can.
    #[test]
    fn bot_choose_cards_cheats_in_biggest_creature() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // cmc 2
        let big = g.add_card_to_hand(0, catalog::shivan_dragon());   // cmc 6
        let candidates = vec![
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&EvalWeights::default(), &g, 0, "Put a creature onto the battlefield?", &candidates, 0, 1) {
            DecisionAnswer::Cards(v) => assert_eq!(v, vec![big],
                "bot picks the highest-cmc creature to cheat in"),
            other => panic!("expected Cards, got {other:?}"),
        }
    }

    /// `decide_choose_cards` over battlefield creatures (Archipelagore's tap)
    /// targets opponents' biggest creature, never the bot's own.
    #[test]
    fn bot_choose_cards_taps_enemy_creatures() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::shivan_dragon());   // 5/5
        let candidates = vec![
            (mine, "Grizzly Bears".to_string()),
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&EvalWeights::default(), &g, 0, "Tap which creatures?", &candidates, 0, 1) {
            DecisionAnswer::Cards(v) => assert_eq!(v, vec![big],
                "bot taps the opponent's biggest creature, not its own"),
            other => panic!("expected Cards, got {other:?}"),
        }
    }

    /// A sacrifice `ChooseCards` prompt is a cost: give up the least
    /// valuable permanent, and only as many as forced.
    #[test]
    fn bot_choose_cards_sacrifices_the_worst() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
        let candidates = vec![
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&EvalWeights::default(), &g, 0, "Sacrifice a creature", &candidates, 1, 1) {
            DecisionAnswer::Cards(v) => {
                assert_eq!(v, vec![small], "bot sacrifices the smaller creature")
            }
            other => panic!("expected Cards, got {other:?}"),
        }
    }

    /// Pure temp-pump instants are combat tricks; burn and creatures are not.
    #[test]
    fn combat_trick_classifier() {
        assert!(is_combat_trick(&catalog::giant_growth()));
        assert!(!is_combat_trick(&catalog::lightning_bolt()), "burn is not a trick");
        assert!(!is_combat_trick(&catalog::grizzly_bears()));
    }

    /// The bot holds Giant Growth in its main phase (a sorcery-speed pump
    /// telegraphs and buffs nothing that matters) …
    #[test]
    fn bot_holds_pump_trick_in_main_phase() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let growth = g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = main_phase_action(&g, 0);
        assert!(
            !matches!(action, GameAction::CastSpell { card_id, .. } if card_id == growth),
            "pump trick is held for combat, got {action:?}",
        );
    }

    /// … and fires it after blocks when it flips a fight its attacker is
    /// losing (2/2 blocked by a 5/5: +3/+3 trades instead of chumping).
    #[test]
    fn bot_casts_trick_on_blocked_attacker() {
        let mut g = two_player_game();
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        let growth = g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareBlockers;
        g.set_attacking(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]);
        g.set_block_map([(dragon, bears)]);
        g.set_blockers_declared(true);
        let action = HeuristicBot::new().next_action(&g, 0);
        assert!(
            matches!(
                action,
                Some(GameAction::CastSpell {
                    card_id,
                    target: Some(crate::game::Target::Permanent(t)),
                    ..
                }) if card_id == growth && t == bears
            ),
            "trick targets the blocked attacker, got {action:?}",
        );
    }

    /// No trick when the fight is already won (2/2 blocked by a 1/1).
    #[test]
    fn bot_holds_trick_when_fight_already_won() {
        let mut g = two_player_game();
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
        g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareBlockers;
        g.set_attacking(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]);
        g.set_block_map([(elf, bears)]);
        g.set_blockers_declared(true);
        let action = HeuristicBot::new().next_action(&g, 0);
        assert!(
            matches!(action, Some(GameAction::PassPriority)),
            "no trick needed on a won fight, got {action:?}",
        );
    }

    /// Material eval: a board and full hand beat an empty seat, and a
    /// decided game dominates everything.
    #[test]
    fn eval_material_prefers_board_and_cards() {
        let mut g = two_player_game();
        assert_eq!(
            eval_material(&g, 0, &EvalWeights::default()),
            -eval_material(&g, 1, &EvalWeights::default()),
            "the two-player eval is symmetric",
        );
        g.add_card_to_battlefield(0, catalog::shivan_dragon());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        assert!(eval_material(&g, 0, &EvalWeights::default()) > 0, "board + hand is a material lead");
        assert!(eval_material(&g, 1, &EvalWeights::default()) < 0);
        g.game_over = Some(Some(1));
        assert!(eval_material(&g, 1, &EvalWeights::default()) > eval_material(&g, 0, &EvalWeights::default()), "a won game beats any material");
    }

    /// The gap one-action-at-a-time scoring cannot close: with four mana,
    /// two two-drops beat one four-drop, but a greedy pick compares each
    /// cast against the board *once* and takes the biggest single body.
    #[test]
    fn lookahead_prefers_two_cheap_spells_over_one_expensive_one() {
        let w = EvalWeights::lookahead1();
        let mut g = two_player_game();
        // Second main so the summon-sick gate (on in both profiles) isn't
        // what decides this.
        g.step = TurnStep::PostCombatMain;
        for _ in 0..4 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        // One four-mana 4/5 versus two two-mana 2/2s. Two bears are 4/4
        // across two bodies for the same mana — the greedy pick can't see
        // the second one because it never asks what comes next.
        let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
        let bear_a = g.add_card_to_hand(0, catalog::grizzly_bears());
        let bear_b = g.add_card_to_hand(0, catalog::grizzly_bears());
        let cast = |id| GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        // Craw Wurm costs {4}{G}{G} — too much here; use the mana we have.
        let _ = wurm;
        let one_bear = evaluate_action_sequence(&g, 0, &cast(bear_a), None, &w, 0)
            .expect("single-play score");
        let bear_then_bear = evaluate_action_sequence(&g, 0, &cast(bear_a), None, &w, 1)
            .expect("two-play score");
        assert!(
            bear_then_bear > one_bear,
            "looking one play ahead must see the second bear ({bear_then_bear} vs {one_bear})",
        );
        let _ = bear_b;
    }

    /// Lookahead must not invent plays that aren't legal yet: once the
    /// bot no longer holds priority in its own main phase, there is no
    /// continuation to search.
    #[test]
    fn follow_up_candidates_are_empty_outside_our_main_phase() {
        let w = EvalWeights::lookahead1();
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        assert!(
            !follow_up_candidates(&g, 0, &w).is_empty(),
            "our own main phase offers continuations",
        );
        g.step = TurnStep::DeclareBlockers;
        assert!(
            follow_up_candidates(&g, 0, &w).is_empty(),
            "no sorcery-speed continuation mid-combat",
        );
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        assert!(
            follow_up_candidates(&g, 0, &w).is_empty(),
            "not our turn, no continuation",
        );
    }

    /// Forge's summon-sick gate: a creature that can't attack this turn is
    /// worth the same after combat, so the bot should deploy it in the
    /// second main and keep the mana up in between. Measured by `bot_probe`
    /// to move plays in the postcombat main from 0.5 % to 37.7 %.
    #[test]
    fn hold_sick_gate_defers_a_vanilla_creature_to_the_second_main() {
        let w = EvalWeights::hold_sick();
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let cast = GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        // The body is all it does, and it can't attack -- no progress today.
        assert!(
            !improves_this_turn(&g, 0, &cast, None, &w),
            "a vanilla creature achieves nothing on the turn it lands",
        );
        // So the gated bot passes in the first main...
        let mut bot = HeuristicBot::with_weights(w);
        assert!(
            matches!(bot.next_action(&g, 0), Some(GameAction::PassPriority)),
            "gated bot holds the creature in the precombat main",
        );
        // ...and deploys it in the second, where holding costs nothing.
        g.step = TurnStep::PostCombatMain;
        let mut bot2 = HeuristicBot::with_weights(w);
        assert!(
            matches!(bot2.next_action(&g, 0), Some(GameAction::CastSpell { card_id, .. }) if card_id == bear),
            "gated bot casts it postcombat",
        );
        // The historical profile casts it immediately, which is the
        // behavior the gate exists to change.
        let mut plain = HeuristicBot::with_weights(EvalWeights::baseline());
        let mut pre = g.clone();
        pre.step = TurnStep::PreCombatMain;
        assert!(
            matches!(plain.next_action(&pre, 0), Some(GameAction::CastSpell { .. })),
            "the historical profile still front-loads",
        );
    }

    /// The gate must not hold a line that *does* something now: a hasty
    /// body can attack, so deploying it precombat is real progress.
    #[test]
    fn hold_sick_gate_lets_through_a_play_that_matters_now() {
        let w = EvalWeights::hold_sick();
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::mountain());
            g.clear_sickness(land);
        }
        let guide = g.add_card_to_hand(0, catalog::goblin_guide());
        let cast = GameAction::CastSpell {
            card_id: guide,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        assert!(
            improves_this_turn(&g, 0, &cast, None, &w),
            "a haste creature is progress the turn it lands",
        );
    }

    /// After a London mulligan the bot puts cards back on the library.
    /// `AutoDecider` bottoms the first N cards of the hand, which routinely
    /// meant shipping the business spells and keeping a fistful of lands.
    /// Found by `bot_probe`: `PutOnLibrary` was 9 % of all decisions the bot
    /// faced and every one of them fell through to that default.
    #[test]
    fn bot_bottoms_surplus_lands_not_the_front_of_its_hand() {
        use crate::decision::{Decision, DecisionAnswer};
        let mut g = two_player_game();
        // Front of hand: the good cheap spell. Then a pile of lands.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let lands: Vec<_> =
            (0..4).map(|_| g.add_card_to_hand(0, catalog::mountain())).collect();
        // Already flooded, so every hand land is surplus.
        for _ in 0..5 {
            g.add_card_to_battlefield(0, catalog::mountain());
        }
        let hand: Vec<(crate::card::CardId, String)> = std::iter::once(bolt)
            .chain(lands.iter().copied())
            .map(|id| (id, String::new()))
            .collect();
        g.pending_decision = Some(Box::new(crate::game::types::PendingDecision {
            decision: Decision::PutOnLibrary { player: 0, count: 2, hand },
            resume: crate::game::types::ResumeContext::Mulligan {
                player: 0,
                mulligans_taken: 1,
                next_player: None,
            },
        }));
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot answers the decision");
        let GameAction::SubmitDecision(DecisionAnswer::PutOnLibrary(put)) = action else {
            panic!("expected a PutOnLibrary answer, got {action:?}");
        };
        assert_eq!(put.len(), 2, "bottoms exactly the requested count");
        assert!(!put.contains(&bolt), "the spell must not be bottomed: {put:?}");
        assert!(put.iter().all(|id| lands.contains(id)), "only surplus lands go back");
    }

    /// The combat-aware evaluator has to actually reach combat damage and
    /// come back, or it silently degrades to the old snapshot behavior.
    #[test]
    fn simulate_through_combat_advances_past_combat_damage() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // Empty board opposite, so the swing is free and unambiguous.
        let life_before = g.players[1].life;
        let mut fuel = 200u32;
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Completed,
            "combat should be simulated",
        );
        assert!(g.step >= TurnStep::CombatDamage, "advanced to combat damage, got {:?}", g.step);
        assert_eq!(g.players[1].life, life_before - 2, "the bear connected for 2");
    }

    /// The cheap bail-outs matter: the state clone is the expensive part,
    /// so a position with no combat to look at must cost nothing.
    #[test]
    fn simulate_through_combat_bails_without_attackers() {
        let mut g = two_player_game();
        let mut fuel = 200u32;
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Skipped,
            "no creatures, no combat",
        );
        assert_eq!(fuel, 200, "bailing must not burn fuel");
        // A summoning-sick creature can't attack, so still nothing to see.
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Skipped,
            "sick creature can't attack",
        );
        g.step = TurnStep::PostCombatMain;
        g.clear_sickness(g.battlefield[0].id);
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Skipped,
            "combat is already over",
        );
    }

    /// The payoff. A creature that is *forced* to attack into a blocker
    /// that eats it is not the material the board says it is. The snapshot
    /// evaluator counts the body and never sees it die; the combat-aware
    /// one plays the turn out and prices it correctly.
    #[test]
    fn combat_aware_eval_sees_a_forced_attacker_die() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        let doomed = g.add_card_to_battlefield(
            0,
            weights_test_creature("Doomed Charger", 2, 2, 2, &[Keyword::MustAttack]),
        );
        g.clear_sickness(doomed);
        // A 4/4 blocks it, kills it, and survives.
        g.add_card_to_battlefield(1, catalog::craw_wurm());
        let w = EvalWeights::combat_aware();
        let snapshot = eval_material(&g, 0, &w);
        let mut sim = g.clone();
        let mut fuel = 200u32;
        assert_eq!(simulate_through_combat(&mut sim, &mut fuel, &EvalWeights::default()), CombatSim::Completed);
        assert!(sim.battlefield_find(doomed).is_none(), "the forced attacker died");
        assert!(
            eval_material(&sim, 0, &w) < snapshot,
            "losing the body must score worse than the board that still has it",
        );
    }

    /// `attack_skip_open` collapses the search to the greedy declaration
    /// exactly when no opposing seat controls a blocker, and leaves the
    /// search intact the moment one appears. The throughput device measured
    /// on PERF `(-21)`: an open board is 9-16 % of searched declarations.
    #[test]
    fn attack_skip_open_only_shortcuts_a_blockerless_board() {
        let w = EvalWeights::attack_skip_open_on();
        // Two attackers vs an empty opposing board — the shortcut fires and
        // the candidate list is the single greedy declaration.
        let mut open = two_player_game();
        for n in 0..2 {
            let a = open.add_card_to_battlefield(
                0,
                weights_test_creature(if n == 0 { "Runner A" } else { "Runner B" }, 2, 3, 3, &[]),
            );
            open.clear_sickness(a);
        }
        assert!(board_open_for_attack(&open, 0), "no opposing creature — board is open");
        assert_eq!(
            attack_candidates_for_mcts(&open, 0, &w).len(),
            1,
            "the open-board shortcut takes greedy without simulating",
        );
        // Drop a small blocker the attackers still profitably run past onto
        // the opposing board: the shortcut no longer fires, so the search
        // runs and offers holdback candidates.
        let mut guarded = open.clone();
        guarded.add_card_to_battlefield(1, weights_test_creature("Wall", 1, 0, 3, &[]));
        assert!(!board_open_for_attack(&guarded, 0), "an opposing creature closes the board");
        assert!(
            attack_candidates_for_mcts(&guarded, 0, &w).len() > 1,
            "a blocker restores the full attack search",
        );
    }

    /// The baseline profile must stay a byte-for-byte control for the
    /// ladder: life counted linearly, no keyword term, scale 1.
    #[test]
    fn baseline_profile_is_the_historical_evaluation() {
        let base = EvalWeights::baseline();
        for life in [-3, 0, 1, 7, 20, 41] {
            assert_eq!(life_value(life, &base), life, "baseline life is linear");
        }
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, weights_test_creature("Baseline Body", 4, 3, 3, &[]));
        let body = g.battlefield[0].id;
        // Baseline is exactly mana value + power + toughness, nothing else.
        assert_eq!(permanent_value(&g, body, &base), 4 + 3 + 3);
    }

    /// A creature for the weighting tests: `cost` generic mana, `power`/
    /// `toughness`, and whatever keywords the case needs.
    fn weights_test_creature(
        name: &'static str,
        cost: u32,
        power: i32,
        toughness: i32,
        keywords: &[crate::card::Keyword],
    ) -> CardDefinition {
        use crate::card::CardType;
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            cost: crate::mana::cost(&[crate::mana::generic(cost)]),
            power,
            toughness,
            keywords: keywords.to_vec(),
            ..Default::default()
        }
    }

    /// Life is worth more per point the closer to zero it is. A linear term
    /// prices "gain 3" identically at 3 life and at 20 -- this is the whole
    /// reason for the curve, so assert the shape, not just the endpoints.
    #[test]
    fn concave_life_curve_is_monotone_with_diminishing_returns() {
        let w = EvalWeights::v2();
        let at = |l: i32| life_value(l, &w);
        // Anchored to the linear term it replaces: 20 life is still 20 points.
        assert_eq!(at(20), 20 * w.unit);
        assert_eq!(at(0), 0);
        for l in 1..=40 {
            assert!(at(l) > at(l - 1), "life {l} must beat life {}", l - 1);
        }
        // Marginal value never rises as life goes up.
        for l in 2..=39 {
            let lower = at(l) - at(l - 1);
            let upper = at(l + 1) - at(l);
            assert!(upper <= lower, "marginal life at {l} rose ({lower} -> {upper})");
        }
        // And the low end is dramatically steeper than the high end: the
        // point that saves us from dying is worth several near the top.
        assert!(
            at(1) - at(0) >= 4 * (at(20) - at(19)),
            "the first point of life should dwarf the twentieth",
        );
    }

    /// Evasion scales with power (it's worth what it lets the body deal);
    /// protection is flat (it buys the same thing on any body). Getting
    /// this backwards is the mistake a flat keyword table makes.
    #[test]
    fn keyword_value_scales_evasion_but_not_protection() {
        use crate::card::Keyword;
        let w = EvalWeights::v2();
        let flying = [Keyword::Flying];
        let hexproof = [Keyword::Hexproof];
        assert!(
            keyword_value(&flying, 5, &w) > keyword_value(&flying, 1, &w),
            "flying is worth more on a bigger body",
        );
        assert_eq!(
            keyword_value(&hexproof, 5, &w),
            keyword_value(&hexproof, 1, &w),
            "hexproof buys the same thing regardless of size",
        );
        // Bad keywords are negative, and a body that can neither attack nor
        // block is worth less than its printed size suggests.
        assert!(keyword_value(&[Keyword::Defender], 4, &w) < 0);
        let pacified = keyword_value(&[Keyword::CantAttack, Keyword::CantBlock], 6, &w);
        assert!(
            pacified < keyword_value(&[Keyword::Defender], 6, &w),
            "a fully locked-down creature is the worst case",
        );
    }

    /// The payoff: two bodies the baseline scores as *identical* -- same
    /// cost, same stats -- are correctly separated by v2, which sees that
    /// one of them flies and drains. This is the behavioral difference the
    /// ladder is measuring; removal targeting and cast ranking both read
    /// `permanent_value`, so a tie here is a coin flip on the baseline.
    #[test]
    fn v2_breaks_a_baseline_tie_toward_the_creature_that_does_something() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        g.add_card_to_battlefield(
            0,
            weights_test_creature("Test Flier", 4, 3, 3, &[Keyword::Flying, Keyword::Lifelink]),
        );
        g.add_card_to_battlefield(0, weights_test_creature("Test Lump", 4, 3, 3, &[]));
        let (f, l) = (g.battlefield[0].id, g.battlefield[1].id);
        let base = EvalWeights::baseline();
        let v2 = EvalWeights::v2();
        assert_eq!(
            permanent_value(&g, f, &base),
            permanent_value(&g, l, &base),
            "the baseline can't tell these apart at all",
        );
        assert!(
            permanent_value(&g, f, &v2) > permanent_value(&g, l, &v2),
            "v2 sees that the flier actually does something",
        );
    }
}

#[cfg(test)]
mod monarch_tests {
    use super::*;
    use crate::catalog;
    use crate::player::Player;

    /// CR 725/726 — the crown and the initiative are recurring resources, so
    /// the material eval prices holding them (and an opponent holding them).
    #[test]
    fn eval_material_prices_the_crown() {
        let mut g = crate::game::two_player_game();
        let w = EvalWeights::baseline();
        let before = eval_material(&g, 0, &w);
        g.monarch = Some(0);
        assert!(eval_material(&g, 0, &w) > before);
        g.monarch = Some(1);
        assert!(eval_material(&g, 0, &w) < before);
    }

    #[test]
    fn bot_attacks_the_monarch_over_the_next_seat() {
        // 3 players: next_alive_seat(0) is 1, but seat 2 is the monarch, so
        // the bot should swing at seat 2 to steal the crown.
        let players = vec![Player::new(0, "A"), Player::new(1, "B"), Player::new(2, "C")];
        let mut g = GameState::new(players);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.monarch = Some(2);
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        // Every seat needs a library. The bot now simulates the attack
        // forward before committing to it, and on an empty library taking
        // the crown is *lethal* — the monarch draws at their end step (CR
        // 724) and decks out. Declining would be the right play; the test
        // means to check target selection, not deck-out.
        for seat in 0..3 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }

        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("an action") {
            GameAction::DeclareAttackers(attacks) => {
                assert!(
                    attacks.iter().any(|a| matches!(a.target, AttackTarget::Player(2))),
                    "bot swings at the monarch (seat 2), not the next seat"
                );
            }
            other => panic!("expected DeclareAttackers, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod self_cost_tests {
    use super::*;
    use crate::effect::{Effect, PlayerRef, Selector, Value};

    #[test]
    fn self_cost_seen_through_modal_and_pay_or_else() {
        // A self-cost mode nested inside ChooseMode is recognized.
        let modal = Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]);
        assert!(effect_imposes_self_cost(&modal), "lose-life mode is a self cost");

        // PayManaOrElse → SacrificeSource fallback is a self cost.
        let tax = Effect::PayManaOrElse {
            mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
            otherwise: Box::new(Effect::SacrificeSource),
        };
        assert!(effect_imposes_self_cost(&tax), "sac-unless-pay fallback is a self cost");

        // A purely beneficial modal is not flagged.
        let upside = Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]);
        assert!(!effect_imposes_self_cost(&upside));

        // find_maydo_body reaches into a mode by its prompt.
        let nested = Effect::ChooseMode(vec![Effect::MayDo {
            description: "Pay the price.".into(),
            body: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            }),
        }]);
        assert!(find_maydo_body(&nested, "Pay the price.").is_some());
    }
}

#[cfg(test)]
mod stack_response_tests {
    use super::*;
    use crate::catalog;
    use crate::game::{GameAction, GameState, Target, TurnStep};
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    #[test]
    fn bot_counters_a_big_opponent_spell() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // P0 casts a 7-drop.
        let wurm = g.add_card_to_hand(0, catalog::pelakka_wurm());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 3);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        // Bot (seat 1) holds Counterspell + two untapped Islands.
        let cs = g.add_card_to_hand(1, catalog::counterspell());
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        g.priority.player_with_priority = 1;
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 1).expect("bot acts");
        match action {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, cs, "casts the counterspell");
                assert_eq!(target, Some(Target::Permanent(wurm)), "targets the 7-drop");
            }
            other => panic!("expected a counterspell cast, got {other:?}"),
        }
    }

    #[test]
    fn bot_holds_counter_against_cheap_nonthreatening_spell() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // A cheap spell that doesn't touch the bot's board.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        g.add_card_to_hand(1, catalog::counterspell());
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        g.priority.player_with_priority = 1;
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 1).expect("bot acts");
        assert!(matches!(action, GameAction::PassPriority),
            "a 2-drop bear isn't worth the counter: {action:?}");
    }

    /// The prepared-cast pipeline end to end: with a Prepared counter,
    /// blue up, and cards to draw, the bot casts the banked Ancestral
    /// Recall, and the outcome eval prices the line above passing (it
    /// already values a hand card at 4 — the "eval can't see card
    /// advantage" theory died against this probe).
    #[test]
    fn bot_casts_the_banked_prepared_recall() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_ideation());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == em)
            .unwrap()
            .add_counters(crate::card::CounterType::Prepared, 1);
        g.add_card_to_battlefield(0, catalog::island());
        for _ in 0..5 {
            let id = g.add_card_to_hand(0, catalog::grizzly_bears());
            let idx = g.players[0].hand.iter().position(|c| c.id == id).unwrap();
            let card = g.players[0].hand.remove(idx);
            g.players[0].library.push(card);
        }
        let mut bot = HeuristicBot::new();
        let a = bot.next_action(&g, 0);
        assert!(
            matches!(a, Some(GameAction::CastPrepareSpell { creature_id, .. }) if creature_id == em),
            "the banked Recall fires: {a:?}"
        );
        let w = EvalWeights::default();
        let cast = GameAction::CastPrepareSpell {
            creature_id: em,
            target: Some(crate::game::Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let cast_v = evaluate_action_outcome(&g, 0, &cast, None, &w).unwrap();
        let pass_v = evaluate_action_outcome(&g, 0, &GameAction::PassPriority, None, &w).unwrap();
        assert!(cast_v > pass_v, "drawing three outprices passing: {cast_v} vs {pass_v}");
    }

    /// Walker chip (flag): a 6-loyalty walker against 2+3 power can't be
    /// finished, so the flag-off menu never aims at it; flag-on adds one
    /// declaration redirecting the smallest attacker for the sims to
    /// judge. The recorded loss was ten unpressured turns into an
    /// ultimate.
    #[test]
    fn walker_chip_candidate_joins_the_attack_menu() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
        for c in [bears, angel] {
            g.battlefield.iter_mut().find(|x| x.id == c).unwrap().summoning_sick = false;
        }
        let walker = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == walker)
            .unwrap()
            .add_counters(crate::card::CounterType::Loyalty, 6);
        let chips = |w: &EvalWeights| {
            attack_candidates_for_mcts(&g, 0, w).iter().any(|cand| {
                cand.iter().any(|a| matches!(a.target, AttackTarget::Planeswalker(_)))
            })
        };
        assert!(!chips(&EvalWeights::attack_search_sim()), "flag off: finish-only rule holds");
        assert!(chips(&EvalWeights::walker_chip_on()), "flag on: the chip is on the menu");
    }

    /// The `target_arms` menu: the search is offered the same spell aimed
    /// somewhere else, so a mis-aimed auto-target is a *choice* it can
    /// reject rather than the only option on the menu.
    ///
    /// Built as the recorded failures were shaped — a hostile spell whose
    /// baked-in pick is the caster's own creature — and asserts the
    /// opposite-side variant is present and ordered first among the
    /// alternates.
    #[test]
    fn target_arms_offer_the_other_side() {
        let mut g = two_player_game();
        let own = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, crate::catalog::hill_giant());
        // A hostile cast whose primary target has been aimed at our own
        // creature — the shape the auto-targeter used to produce.
        let bolt = g.add_card_to_hand(0, crate::catalog::doom_blade());
        let mis_aimed = GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(own)),
            additional_targets: Vec::new(),
            mode: None,
            x_value: None,
        };
        let variants = target_arm_variants(&g, 0, &mis_aimed, 2);
        assert!(!variants.is_empty(), "an alternative targeting exists and must be offered");
        assert!(
            matches!(
                &variants[0],
                GameAction::CastSpell { target: Some(Target::Permanent(id)), .. } if *id == theirs
            ),
            "the side the slot wants is the first alternate, got {:?}",
            variants[0]
        );

        // And when the baked-in pick is ALREADY on the right side but
        // aimed at the smaller body, the alternates must offer the bigger
        // enemy creature — not our own. Ranking alternates as "opposite of
        // whatever was chosen" spent the first arm on a self-target here
        // (recorded game, 2026-08-22: Grapple with Death on a 2/2 with a
        // 3/3 beside it).
        // Non-black: Doom Blade's filter excludes black creatures.
        let bigger = g.add_card_to_battlefield(1, crate::catalog::shivan_dragon());
        let correct_but_small = GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(theirs)),
            additional_targets: Vec::new(),
            mode: None,
            x_value: None,
        };
        let v2 = target_arm_variants(&g, 0, &correct_but_small, 2);
        assert!(
            matches!(
                &v2[0],
                GameAction::CastSpell { target: Some(Target::Permanent(id)), .. } if *id == bigger
            ),
            "the bigger enemy body is the first alternate, got {:?}",
            v2[0]
        );
        assert!(
            !v2.iter().any(|a| matches!(a,
                GameAction::CastSpell { target: Some(Target::Permanent(id)), .. } if *id == own)),
            "our own creature must not be offered for a hostile slot: {v2:?}"
        );
        // The flag is what puts them on the menu. Doom Blade is {1}{B}, so
        // the cast has to be affordable before the menu can carry it.
        for _ in 0..2 {
            g.add_card_to_battlefield(0, crate::catalog::swamp());
        }
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // `net_eval_det1` carries the adopted flag since round 46, so the
        // off-side control is the explicit pre-adoption profile.
        let off = main_phase_candidates_for_mcts(&g, 0, &EvalWeights::target_arms_off());
        let on = main_phase_candidates_for_mcts(&g, 0, &EvalWeights::net_eval_det1());
        let casts = |v: &Vec<(GameAction, i32)>| {
            v.iter().filter(|(a, _)| matches!(a, GameAction::CastSpell { .. })).count()
        };
        assert!(casts(&on) > casts(&off), "flag on widens the cast menu: {} vs {}", casts(&on), casts(&off));
    }


    /// The simulation mulligan discriminates: three lands and a curve play
    /// out better than seven six-drops, measured by playing both forward
    /// rather than by counting lands.
    ///
    /// Built through `start_mulligan_phase` so the pending decision is the
    /// real one. `mulligan_branch_value` answers it with `perform_action`
    /// and returns `None` when no mulligan is pending, which is how the
    /// first version of this test passed vacuously.
    #[test]
    fn mulligan_sim_prefers_the_functional_hand() {
        use crate::decision::DecisionAnswer;
        let w = EvalWeights::mull_sim_on();
        let stacked = |spell: fn() -> crate::card::CardDefinition, lands: usize| {
            let mut g = two_player_game();
            for seat in 0..2 {
                for _ in 0..lands {
                    g.add_card_to_library(seat, crate::catalog::forest());
                }
                for _ in 0..(7 - lands) {
                    g.add_card_to_library(seat, spell());
                }
                for _ in 0..20 {
                    g.add_card_to_library(seat, crate::catalog::forest());
                }
            }
            g.start_mulligan_phase();
            g
        };
        let good = stacked(crate::catalog::grizzly_bears, 3);
        let bad = stacked(crate::catalog::craw_wurm, 0);
        let g_keep = mulligan_branch_value(&good, 0, &DecisionAnswer::Keep, &w, 4, 4);
        let b_keep = mulligan_branch_value(&bad, 0, &DecisionAnswer::Keep, &w, 4, 4);
        assert!(
            g_keep.is_some() && b_keep.is_some(),
            "both branches must actually simulate (is a mulligan pending?)"
        );
        assert!(
            g_keep > b_keep,
            "lands-and-a-curve must outplay seven six-drops: {g_keep:?} vs {b_keep:?}"
        );
    }

    /// Ark of Hunger's `{T}: mill 1, you may play that card` — the impulse
    /// draw shape. Recorded game: cast turn 19, never activated across five
    /// turns while the bot topdecked with an empty hand, then exiled. The
    /// ability generators are a whitelist of effect shapes and this one was
    /// on none of them, so no valuation could have chosen it.
    #[test]
    fn impulse_draw_activates_the_ark() {
        let mut g = two_player_game();
        let ark = g.add_card_to_battlefield(0, crate::catalog::ark_of_hunger());
        g.add_card_to_battlefield(0, crate::catalog::mountain());
        for c in g.battlefield.iter_mut() {
            c.summoning_sick = false;
        }
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Empty hand: exactly the spot the replay was in.
        assert!(g.players[0].hand.is_empty());

        let fires = |w: &EvalWeights| {
            let mut bot = HeuristicBot::with_weights(*w);
            matches!(
                bot.next_action(&g, 0),
                Some(GameAction::ActivateAbility { card_id, .. }) if card_id == ark
            )
        };
        assert!(!fires(&EvalWeights::block_gang_search()), "flag off: unchanged");
        assert!(fires(&EvalWeights::impulse_draw_on()), "flag on: the Ark is activated");
    }

    /// auto-aimed target.
    #[test]
    fn ability_arms_enumerate_the_archaic_activation() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let archaic = g.add_card_to_battlefield(0, catalog::sundering_archaic());
        g.add_card_to_battlefield(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::forest());
        // A graveyard card for the ability to aim at.
        let dead = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let idx = g.battlefield.iter().position(|c| c.id == dead).unwrap();
        let card = g.battlefield.remove(idx);
        g.players[1].graveyard.push(card);
        let has_arm = |w: &EvalWeights| {
            cast_candidates(&g, 0, w, None).iter().any(|(a, _)| {
                matches!(a, GameAction::ActivateAbility { card_id, .. } if *card_id == archaic)
            })
        };
        assert!(!has_arm(&EvalWeights::default()), "flag off: the class is invisible");
        assert!(has_arm(&EvalWeights::ability_arms_on()), "flag on: the activation is a candidate");
    }

    #[test]
    fn desperate_board_offers_the_chump_to_the_sims() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.attacking = vec![crate::game::types::Attack {
            attacker: angel,
            target: crate::game::types::AttackTarget::Player(0),
        }];
        g.players[0].life = 5;
        let off = block_candidates_for_mcts(&g, 0, &EvalWeights::block_gang_search());
        assert_eq!(off, vec![Vec::new()], "flag off: no profitable block, no menu");
        let on = block_candidates_for_mcts(&g, 0, &EvalWeights::chump_blocks_on());
        assert!(
            on.contains(&vec![(bears, angel)]),
            "flag on at 5 life: the chump is on the menu: {on:?}"
        );
        assert!(on.contains(&Vec::new()), "not blocking stays on the menu too");
        g.players[0].life = 15;
        let calm = block_candidates_for_mcts(&g, 0, &EvalWeights::chump_blocks_on());
        assert_eq!(calm, vec![Vec::new()], "at 15 life the desperation gate holds");
    }

    /// Converge-aware land drops: holding a converge card, a land of a
    /// color the mana base doesn't make yet beats a duplicate — even
    /// though no pip in hand demands it. Flag off, the duplicate ties
    /// and hand order wins, which is the recorded pre-fix behavior
    /// (the bot playing its third Plains where the human diversified).
    #[test]
    fn converge_hand_prefers_a_fresh_land_color() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.add_card_to_battlefield(0, catalog::plains());
        let plains = g.add_card_to_hand(0, catalog::plains());
        let swamp = g.add_card_to_hand(0, catalog::swamp());
        g.add_card_to_hand(0, catalog::rancorous_archaic()); // {5}: no pip wants Swamp
        assert_eq!(
            pick_land_to_play(&g, 0, &EvalWeights::default()),
            Some(plains),
            "flag off: no pip needs anything, first playable land wins"
        );
        assert_eq!(
            pick_land_to_play(&g, 0, &EvalWeights::converge_lands_on()),
            Some(swamp),
            "flag on: the converge card in hand makes the fresh color worth more"
        );
    }

    /// The bot plays a color-fixing land over an off-color one: with a green
    /// spell in hand and no green source, it plays the Forest, not the Mountain.
    #[test]
    fn bot_plays_color_fixing_land() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let _mountain = g.add_card_to_hand(0, catalog::mountain());
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // wants green
        assert_eq!(pick_land_to_play(&g, 0, &EvalWeights::default()), Some(forest),
            "fixes the missing green over the off-color Mountain");
    }

    /// `land_urgency` sequences the tapland: with nothing castable it is
    /// the free drop, but once the untapped mana would actually be spent
    /// this turn the basic wins.
    #[test]
    fn land_urgency_times_the_tapland() {
        let w = EvalWeights::land_sequencing();
        // Nothing to cast: the school land (enters tapped, fixes W and B)
        // costs nothing now and fixes two colors later.
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let tapland = g.add_card_to_hand(0, catalog::forum_of_amity());
        let _plains = g.add_card_to_hand(0, catalog::plains());
        g.add_card_to_hand(0, catalog::serra_angel()); // {3}{W}{W}, uncastable now
        g.add_card_to_hand(0, catalog::doom_blade()); // {1}{B} — the second color
        assert_eq!(pick_land_to_play(&g, 0, &w), Some(tapland),
            "no play this turn — take the tapped dual for the two colors it fixes");

        // Same hand plus a one-drop the untapped land would actually
        // cast: entering tapped now costs a real play.
        let mut g2 = two_player_game();
        g2.priority.player_with_priority = 0;
        g2.active_player_idx = 0;
        let _tap2 = g2.add_card_to_hand(0, catalog::forum_of_amity());
        let plains2 = g2.add_card_to_hand(0, catalog::plains());
        g2.add_card_to_hand(0, catalog::savannah_lions()); // {W}, castable off one Plains
        assert_eq!(pick_land_to_play(&g2, 0, &w), Some(plains2),
            "the untapped source buys a play this turn; the tapland doesn't");
    }

    /// A creature-only `{X}: deal X damage to target creature` spell caps X at
    /// the toughest opposing creature — the bot doesn't overkill a 2/2.
    #[test]
    fn max_affordable_x_caps_creature_only_burn_at_lethal() {
        use crate::card::{CardDefinition, CardType};
        use crate::effect::shortcut::target_filtered;
        use crate::effect::{Effect, Value};
        use crate::card::SelectionRequirement;
        let mut g = two_player_game();
        let zap = CardDefinition {
            name: "Test Creature Zap",
            cost: crate::mana::cost(&[crate::mana::x(), crate::mana::r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::XFromCost,
            },
            ..Default::default()
        };
        let id = g.add_card_to_hand(0, zap);
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — toughest opp
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(6);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 2,
            "X capped at the 2/2's toughness, not the full {{6}} pool");
    }

    /// Player-targetable burn (Banefire) is not capped — the bot still dumps
    /// its whole pool into X.
    #[test]
    fn max_affordable_x_does_not_cap_any_target_burn() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire()); // any target
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(6);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 6, "Banefire keeps the full X");
    }

    /// An Unblockable attacker swings even into a bigger blocker — no opposing
    /// creature can legally block it, so the suicide filter doesn't hold it
    /// back (generalized evasion check).
    #[test]
    fn bot_attacks_with_unblockable_into_bigger_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let mut ghost = catalog::grizzly_bears();
        ghost.name = "Ghost";
        ghost.power = 1;
        ghost.toughness = 1;
        ghost.keywords.push(crate::card::Keyword::Unblockable);
        let atk = g.add_card_to_battlefield(0, ghost);
        g.clear_sickness(atk);
        // A lone 5/5 that would trade up against a naive ground attacker.
        let mut big = catalog::grizzly_bears();
        big.name = "Wall"; big.power = 5; big.toughness = 5;
        g.add_card_to_battlefield(1, big);
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|d| d.attacker == atk),
                    "unblockable attacker swings past a bigger blocker");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// With only a couple of lands in play but a fistful of duplicate lands in
    /// hand, a forced discard pitches a surplus land rather than a real spell.
    #[test]
    fn bot_discard_pitches_surplus_land_not_a_spell() {
        let mut g = two_player_game();
        // 2 lands in play → wants ~4 more; a 5th land in hand is surplus.
        for _ in 0..2 { g.add_card_to_battlefield(0, catalog::forest()); }
        let mut hand: Vec<(crate::card::CardId, String)> = Vec::new();
        for _ in 0..5 {
            let id = g.add_card_to_hand(0, catalog::forest());
            hand.push((id, "Forest".to_string()));
        }
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        hand.push((bear, "Grizzly Bears".to_string()));
        let ans = decide_self_discard(&g, 0, &hand, 1);
        match ans {
            crate::decision::DecisionAnswer::Discard(ids) => {
                assert_eq!(ids.len(), 1);
                let pitched = g.players[0].hand.iter().find(|c| c.id == ids[0]).unwrap();
                assert!(pitched.definition.is_land(), "pitched a surplus land, kept the spell");
            }
            other => panic!("expected Discard, got {:?}", other),
        }
    }

    /// The bot accepts an exploit trigger when it has a spare creature (here a
    /// second body), instead of always declining the sacrifice.
    #[test]
    fn bot_takes_exploit_with_a_spare_creature() {
        let mut g = two_player_game();
        let drowner = g.add_card_to_battlefield(0, catalog::gurmag_drowner());
        // No other creature → keep it (would have to sacrifice itself; allowed
        // only by a >1 count, so a lone exploiter declines).
        assert!(!optional_trigger_beneficial(&g, drowner, "Exploit — sacrifice a creature?"),
            "lone exploiter with nothing to spare declines");
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a spare body
        assert!(optional_trigger_beneficial(&g, drowner, "Exploit — sacrifice a creature?"),
            "with a spare creature the bot exploits for value");
    }

    /// The bot crews an uncrewed Vehicle with a spare creature so it can swing.
    #[test]
    fn bot_crews_a_vehicle() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::broadcast_rambler()); // Crew 1, 5/4
        // No creatures yet → nothing to crew with.
        assert!(pick_crew_vehicle(&g, 0).is_none(), "no crewers, no crew");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 ≥ 1
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        assert!(
            matches!(pick_crew_vehicle(&g, 0),
                Some(GameAction::Crew { vehicle, .. }) if vehicle == veh),
            "crews the Vehicle with the spare creature",
        );
    }

    /// The bot fires a "deal N to each opponent" ability for lethal, and only
    /// then (not to chip).
    #[test]
    fn bot_reach_burn_only_for_lethal() {
        let mut g = two_player_game();
        let haz = g.add_card_to_battlefield(0, catalog::hazoret_the_fervent());
        g.clear_sickness(haz);
        g.add_card_to_hand(0, catalog::mountain()); // discard fodder
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        // Opponent at 5: the 2-damage burn isn't lethal, so the bot holds it.
        g.players[1].life = 5;
        assert!(pick_reach_burn(&g, 0).is_none(), "won't chip with a non-lethal burn");
        // Opponent at 2: now it's lethal, so the bot fires it.
        g.players[1].life = 2;
        assert!(matches!(pick_reach_burn(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == haz),
            "fires the burn for lethal");
    }

    /// The bot replays a self-returning graveyard creature (Llanowar Greenwidow)
    /// even though its ability has no exile-self cost.
    #[test]
    fn bot_replays_self_returning_graveyard_creature() {
        let mut g = two_player_game();
        let id = g.add_card_to_graveyard(0, catalog::llanowar_greenwidow());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(7);
        assert!(matches!(pick_graveyard_recursion(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == id),
            "bot activates the graveyard self-return");
    }

    /// The bot drives Brass Squire's two-slot attach ability: an Equipment onto
    /// the biggest creature.
    #[test]
    fn bot_activates_brass_squire_attach() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let squire = g.add_card_to_battlefield(0, catalog::brass_squire());
        g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.clear_sickness(squire);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let action = pick_attach_ability(&g, 0).expect("bot drives the attach ability");
        // Slot 1 (the wearer) is the highest-power creature — the bear, not the
        // 1/3 Squire.
        assert!(matches!(action,
            GameAction::ActivateAbility { card_id, ref additional_targets, .. }
                if card_id == squire && additional_targets == &vec![crate::game::Target::Permanent(bear)]));
    }

    /// The bot cracks a Lander token for ramp when it has spare mana and a basic
    /// still in the library — but not when the library has no basic to fetch.
    #[test]
    fn bot_cracks_lander_for_ramp() {
        let mut g = two_player_game();
        let lander = g.add_token_to_battlefield(0, &crabomination_base::tokens::lander_token());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        // No basic in library → don't waste the Lander.
        assert!(pick_crack_lander(&g, 0).is_none(), "no basic to fetch → hold the Lander");
        g.add_card_to_library(0, catalog::forest());
        assert!(matches!(pick_crack_lander(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == lander),
            "with a basic in library and spare mana, the bot ramps");
    }

    /// With spare mana and no better play, the bot sinks it into a repeatable
    /// self-+1/+1 ability (Fire Sages) — but never burns a once-per-game Exhaust.
    #[test]
    fn bot_sinks_spare_mana_into_self_pump() {
        let mut g = two_player_game();
        let sages = g.add_card_to_battlefield(0, catalog::fire_sages());
        g.clear_sickness(sages);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        assert!(matches!(pick_self_pump_counter(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == sages),
            "bot grows Fire Sages with leftover mana");

        // An Exhaust pump (Mai) is left alone even with mana to spare.
        let mut g = two_player_game();
        let mai = g.add_card_to_battlefield(0, catalog::mai_jaded_edge());
        g.clear_sickness(mai);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        assert!(pick_self_pump_counter(&g, 0).is_none(), "won't spend a once-per-game Exhaust as a mana sink");
    }

    /// With spare mana and nothing better, the bot sinks it into a
    /// "{cost}: create a token" ability (Sun Warriors' {5}: 1/1 Ally).
    #[test]
    fn bot_sinks_spare_mana_into_token_maker() {
        let mut g = two_player_game();
        let sw = g.add_card_to_battlefield(0, catalog::sun_warriors());
        g.clear_sickness(sw);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_token_maker(&g, 0).is_none(), "no mana → no token");
        g.players[0].mana_pool.add_colorless(5);
        assert!(matches!(pick_token_maker(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == sw),
            "bot makes an Ally token with leftover mana");
    }

    /// The bot casts a Spree spell via `CastSpellSpree` (not a no-op plain
    /// cast), choosing an affordable mode with a legal target.
    #[test]
    fn bot_casts_spree_spell() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // A juicy opposing creature for Explosive Derailment's +{2} "deal 4" mode.
        g.add_card_to_battlefield(1, catalog::serra_angel());
        let spell = g.add_card_to_hand(0, catalog::explosive_derailment());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2); // base {R} + mode {2}
        match main_phase_action(&g, 0) {
            GameAction::CastSpellSpree { card_id, spree_modes, target, .. } => {
                assert_eq!(card_id, spell, "cast the Spree spell");
                assert!(!spree_modes.is_empty(), "chose at least one mode");
                assert!(target.is_some(), "aimed the damage mode at a target");
            }
            other => panic!("expected a Spree cast, got {other:?}"),
        }
    }

    /// Off-turn window: at the opponent's end step with an empty stack the
    /// bot casts instant-speed spells (EOT removal) but not sorcery-speed
    /// cards, which `would_accept` filters out.
    #[test]
    fn bot_casts_instant_at_opponents_end_step() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::End;
        g.active_player_idx = 1; // opponent's turn
        g.priority.player_with_priority = 0;
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let _bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);

        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, bolt, "only the instant is castable off-turn");
                // "Any target" burn defaults to the face per the engine's
                // auto-targeter; either opponent-side target is fine here —
                // the point is that the instant is cast off-turn at all.
                let opponent_side = matches!(target, Some(Target::Player(1)))
                    || matches!(target, Some(Target::Permanent(id)) if id == angel);
                assert!(opponent_side, "aimed at the opponent's side: {target:?}");
            }
            other => panic!("expected an EOT Bolt, got {other:?}"),
        }
    }

    /// Overkill/chip awareness: with a 5/5 and a 2/2 on the other side and
    /// only Shock in hand, the scorer must not value Shock-at-the-5/5 as
    /// removal — the kill (2/2) outranks the chip (5/5) despite the 5/5's
    /// higher `permanent_value`.
    #[test]
    fn scorer_prefers_killing_over_chipping() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let shock = g.add_card_to_hand(0, catalog::shock());

        let kill = GameAction::CastSpell {
            card_id: shock, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        };
        let chip = GameAction::CastSpell {
            card_id: shock, target: Some(Target::Permanent(dragon)),
            additional_targets: vec![], mode: None, x_value: None,
        };
        assert!(
            score_candidate(&g, 0, &kill, &EvalWeights::default()) > score_candidate(&g, 0, &chip, &EvalWeights::default()),
            "killing the 2/2 must outscore chipping the 5/5",
        );
    }

    /// The counter gate scores the stack spell instead of the old cmc>=3
    /// rule: a removal spell aimed at the bot's best creature is counter-
    /// worthy even at 2 cmc.
    #[test]
    fn bot_counters_cheap_removal_aimed_at_its_bomb() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        // P0 bolts the bot's dragon (cmc 1 — held under the old gate).
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(dragon)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        let cs = g.add_card_to_hand(1, catalog::counterspell());
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        g.priority.player_with_priority = 1;
        let mut bot = HeuristicBot::new();
        match bot.next_action(&g, 1).expect("bot acts") {
            GameAction::CastSpell { card_id, .. } => {
                assert_eq!(card_id, cs, "counters the removal aimed at its best creature");
            }
            other => panic!("expected a counterspell, got {other:?}"),
        }
    }

    /// A beneficial Aura (Rancor) is cast on the bot's own best creature,
    /// never on an opposing one (Effect::Attach isn't classified friendly
    /// by the generic auto-targeter).
    #[test]
    fn bot_puts_beneficial_aura_on_own_best_creature() {
        use crate::mana::Color;
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let _opp = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5, tempting
        g.add_card_to_hand(0, catalog::rancor());
        g.players[0].mana_pool.add(Color::Green, 1);

        match main_phase_action(&g, 0) {
            GameAction::CastSpell { target: Some(Target::Permanent(t)), .. } => {
                assert_ne!(t, small, "picks the better of its own creatures");
                assert_eq!(t, big, "Rancor goes on the bot's best creature, not the opponent's");
            }
            other => panic!("expected a Rancor cast on own creature, got {other:?}"),
        }
    }

    /// Prepare-cast valuation: the inset spell is scored as itself (a
    /// {U} instant ≈ 2 points), not as the 5/5 creature carrying it
    /// (≈ 22) — and a controlled "prepared matters" static (Top of the
    /// Class) charges the cast for the rider it strips.
    #[test]
    fn prepare_cast_scored_by_inset_spell_not_creature() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_ideation());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == em)
            .unwrap()
            .add_counters(CounterType::Prepared, 1);
        let cast = GameAction::CastPrepareSpell {
            creature_id: em,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let plain = score_candidate(&g, 0, &cast, &EvalWeights::default());
        assert!(
            plain <= 8,
            "inset {{U}} draw spell must score as a cheap spell, got {plain}",
        );
        g.add_card_to_battlefield(0, catalog::top_of_the_class());
        let with_anthem = score_candidate(&g, 0, &cast, &EvalWeights::default());
        assert!(
            with_anthem < plain,
            "unpreparing under a prepared-matters anthem must score lower \
             ({with_anthem} !< {plain})",
        );
    }

    /// A 3/3 for the ward tests: identical body with and without Ward, so
    /// a score comparison isolates the ward term instead of conflating it
    /// with mana-value or keyword differences.
    fn test_bear(ward: Option<crate::card::WardCost>) -> CardDefinition {
        use crate::card::{CardType, Keyword};
        CardDefinition {
            name: "Ward Test Bear",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: ward.map(|w| vec![Keyword::Ward(w)]).unwrap_or_default(),
            ..Default::default()
        }
    }

    /// CR 702.21 under bot play: casting removal at a warded creature with
    /// no mana left for the tax gets the spell countered by the ward
    /// trigger's auto-pay failing — strictly worse than holding it. The
    /// ward gate drops the candidate until the tax is payable *on top of*
    /// the spell's own cost.
    #[test]
    fn bot_wont_cast_removal_into_unpayable_ward_mana() {
        use crate::card::WardCost;
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let bear = g.add_card_to_battlefield(1, test_bear(Some(WardCost::generic(2))));
        let blade = g.add_card_to_hand(0, catalog::doom_blade()); // {1}{B}
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            !matches!(action, GameAction::CastSpell { .. }),
            "exactly {{1}}{{B}} up: Doom Blade into Ward {{2}} would be countered, got {action:?}"
        );
        g.players[0].mana_pool.add_colorless(2);
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == blade && t == bear
            ),
            "with the tax affordable the same cast goes through, got {action:?}"
        );
    }

    /// Ward—Pay N life at N ≥ our life total: the engine's auto-pay would
    /// spend the bot's whole life into the state-based loss, so the gate
    /// refuses the target outright; with a live total it is just a tax.
    #[test]
    fn bot_wont_pay_lethal_ward_life() {
        use crate::card::WardCost;
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let bear = g.add_card_to_battlefield(1, test_bear(Some(WardCost::Life(5))));
        let blade = g.add_card_to_hand(0, catalog::doom_blade());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.players[0].life = 4;
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            !matches!(action, GameAction::CastSpell { .. }),
            "paying Ward—5 life at 4 life is suicide, got {action:?}"
        );
        g.players[0].life = 20;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == blade && t == bear
            ),
            "at 20 life the ward is a payable tax, got {action:?}"
        );
    }

    /// Two identical 3/3s, one warded: the cast aimed at the warded twin
    /// scores lower, so the un-warded target (or a different spell) wins
    /// the tie even when both taxes are payable.
    #[test]
    fn warded_target_scores_below_unwarded_twin() {
        use crate::card::WardCost;
        let mut g = two_player_game();
        let warded = g.add_card_to_battlefield(1, test_bear(Some(WardCost::generic(2))));
        let plain = g.add_card_to_battlefield(1, test_bear(None));
        let blade = g.add_card_to_hand(0, catalog::doom_blade());
        let cast_at = |t: CardId| GameAction::CastSpell {
            card_id: blade,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let w = EvalWeights::default();
        let s_warded = score_candidate(&g, 0, &cast_at(warded), &w);
        let s_plain = score_candidate(&g, 0, &cast_at(plain), &w);
        assert!(
            s_warded < s_plain,
            "identical bodies, one warded: {s_warded} !< {s_plain}"
        );
    }

    /// SOS Repartee: with a payoff out that wants instants/sorceries to
    /// target a creature, an "any target" burn spell gets a
    /// creature-aimed sibling candidate, and the outcome eval takes the
    /// creature kill over the face ping.
    #[test]
    fn repartee_offers_creature_target_for_any_target_burn() {
        use crate::card::CardType;
        use crate::effect::shortcut;
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let payoff = CardDefinition {
            name: "Repartee Payoff",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 3,
            triggered_abilities: vec![shortcut::repartee(Effect::Noop)],
            ..Default::default()
        };
        g.add_card_to_battlefield(0, payoff);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let shock = g.add_card_to_hand(0, catalog::shock());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == shock && t == bear
            ),
            "with a Repartee payoff out, Shock kills the bear instead of pinging face, \
             got {action:?}"
        );
    }

    /// Under `attack_sim_spells` the attack simulation sees the
    /// crack-back: an attacker that survives combat but dies to the
    /// removal in the opponent's hand scores the line lower than the
    /// spell-blind sim does.
    #[test]
    fn spell_sim_sees_crackback_removal() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // The opponent holds real interaction: Doom Blade plus the mana
        // to cast it on their turn.
        g.add_card_to_battlefield(1, catalog::swamp());
        g.add_card_to_battlefield(1, catalog::swamp());
        g.add_card_to_hand(1, catalog::doom_blade());
        // Both libraries stocked so the sim's draw steps don't deck anyone.
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
            g.add_card_to_library(1, catalog::swamp());
        }
        let atk = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
        let blind = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search())
            .expect("spell-blind sim completes");
        let seeing = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search_sim())
            .expect("spell-casting sim completes");
        assert!(
            seeing < blind,
            "the sim that lets the opponent Doom Blade must score lower \
             ({seeing} !< {blind})"
        );
    }

    /// Emblems price by shape now: a recurring draw engine out-values a
    /// recurring trickle of life, where the old flat constant read them
    /// the same.
    #[test]
    fn emblem_value_prices_shapes() {
        use crate::card::TriggeredAbility;
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let g = two_player_game();
        // The event kind is irrelevant to the shape pricing (any
        // non-LifeGained trigger walks the body); Attacks is just a
        // parameterless stand-in.
        let emblem = |body: Effect| crate::player::Emblem {
            name: "Test".into(),
            triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
                effect: body,
            }],
            statics: vec![],
        };
        let draw = emblem(Effect::Draw { who: Selector::You, amount: Value::Const(2) });
        let life = emblem(Effect::GainLife { who: Selector::You, amount: Value::Const(1) });
        assert!(
            emblem_value(&g, 0, &draw) > emblem_value(&g, 0, &life),
            "a draw-two-per-turn emblem must out-value gain-one-per-turn"
        );
    }

    /// A walker that survives the swing banks the plus instead of cashing
    /// out — the case the old guard could not express.
    ///
    /// Regression, 2026-08-23, from `replay-1787448562-3` turn 19: Ral
    /// Zarek at 1 loyalty spent its last point to strip one card and
    /// died, with a `+1: Surveil 2` available. The guard compared raw
    /// enemy power against *current* loyalty, counted creatures that
    /// could not attack, and ignored our blockers, so any 1/1 forced a
    /// cash-out. Across every recorded game the bot made **zero** plus
    /// activations against eight minuses.
    #[test]
    fn defended_walker_banks_the_plus() {
        use crate::card::{CardType, CounterType, LoyaltyAbility};
        use crate::effect::shortcut::target_any;
        use crate::effect::{Selector, Value};
        let walker = || CardDefinition {
            name: "Test Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 1,
            loyalty_abilities: vec![
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 1,
                    effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                },
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: -1,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
                },
            ],
            ..Default::default()
        };
        let w = EvalWeights::default();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, walker());
        // One loyalty: the old guard cashed out against any body at all.
        let pw = g.battlefield.iter_mut().find(|c| c.id == id).unwrap();
        let extra = 1 - pw.counter_count(CounterType::Loyalty) as i32;
        if extra > 0 {
            pw.add_counters(CounterType::Loyalty, extra as u32);
        }
        // A lone 2/2 across the table, and two untapped bodies to eat it.
        g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
        g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.add_card_to_battlefield(0, crate::catalog::hill_giant());
        let action = pick_loyalty_ability(&g, 0, &w).expect("walker activates");
        assert!(
            matches!(action, GameAction::ActivateLoyaltyAbility { ability_index: 0, .. }),
            "a defended walker banks the plus rather than dying for one ping, got {action:?}"
        );
    }

    /// A walker the enemy board kills before its next activation cashes
    /// out: with lethal power across the table it spends loyalty on the
    /// minus; with an empty board it banks the plus.
    #[test]
    fn doomed_walker_cashes_out() {
        use crate::card::{CardType, CounterType, LoyaltyAbility};
        use crate::effect::shortcut::target_any;
        use crate::effect::{Selector, Value};
        let walker = || CardDefinition {
            name: "Test Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 2,
            loyalty_abilities: vec![
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 1,
                    effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                },
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: -2,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
                },
            ],
            ..Default::default()
        };
        let w = EvalWeights::default();
        let mut safe = two_player_game();
        let id = safe.add_card_to_battlefield(0, walker());
        safe.battlefield.iter_mut().find(|c| c.id == id).unwrap()
            .add_counters(CounterType::Loyalty, 2);
        let action = pick_loyalty_ability(&safe, 0, &w).expect("walker activates");
        assert!(
            matches!(action, GameAction::ActivateLoyaltyAbility { ability_index: 0, .. }),
            "empty enemy board: bank the plus, got {action:?}"
        );
        // Two 3/3s: power 6 covers the loyalty however the engine seeded
        // it (base loyalty plus the counters added above).
        let mut doomed = safe.clone();
        doomed.add_card_to_battlefield(1, test_bear(None));
        doomed.add_card_to_battlefield(1, test_bear(None));
        let action = pick_loyalty_ability(&doomed, 0, &w).expect("walker activates");
        assert!(
            matches!(action, GameAction::ActivateLoyaltyAbility { ability_index: 1, .. }),
            "enemy power covers the loyalty: spend it down, got {action:?}"
        );
    }

    /// The counter bar drops when the hand clogs: a mid-size threat that a
    /// comfortable hand lets resolve gets countered once the counter would
    /// otherwise rot toward a cleanup discard.
    #[test]
    fn clogged_hand_lowers_counter_bar() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.active_player_idx = 1; // the ogre is a sorcery-speed cast
        let counter = g.add_card_to_hand(0, catalog::counterspell());
        g.players[0].mana_pool.add(Color::Blue, 2);
        // Opponent casts a 7-unit threat (Gray Ogre: 3 cmc + 2 + 2).
        let ogre = g.add_card_to_hand(1, catalog::gray_ogre());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.players[1].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: ogre,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts");
        while g.player_with_priority() != 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let w = EvalWeights::default();
        assert!(
            pick_stack_response(&g, 0, &w).is_none(),
            "a two-card hand holds the counter for something bigger"
        );
        for _ in 0..5 {
            g.add_card_to_hand(0, catalog::forest());
        }
        let action =
            pick_stack_response(&g, 0, &w).expect("clogged hand counters").action();
        assert!(
            matches!(action, GameAction::CastSpell { card_id, .. } if card_id == counter),
            "got {action:?}"
        );
    }

    /// The stack 2-for-1: the opponent's Giant Growth on their own bear
    /// invites the Bolt in response — lethal against CURRENT toughness,
    /// since the pump will fizzle. Off by default; the flag arms it.
    #[test]
    fn buff_response_kills_the_creature_under_the_pump() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.active_player_idx = 1;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let growth = g.add_card_to_hand(1, catalog::giant_growth());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: growth,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent pumps their own bear");
        while g.player_with_priority() != 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        assert!(
            pick_buff_response(&g, 0, &EvalWeights::default()).is_none(),
            "off by default until laddered"
        );
        let action = pick_buff_response(&g, 0, &EvalWeights::buff_2for1_on())
            .expect("the Bolt answers the bear under the Growth");
        assert!(
            matches!(action, GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                if card_id == bolt && t == bears),
            "got {action:?}"
        );
    }

    /// The defender kills the biggest declared attacker before committing
    /// blocks: instant removal is a combat response now, not just an
    /// end-step afterthought.
    #[test]
    fn defensive_removal_kills_declared_attacker() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.clear_sickness(serra);
        let blade = g.add_card_to_hand(0, catalog::doom_blade());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serra,
            target: AttackTarget::Player(0),
        }]))
        .expect("opponent attacks");
        let mut fuel = 8;
        while g.player_with_priority() != 0 && fuel > 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
            fuel -= 1;
        }
        let action = HeuristicBot::new().next_action(&g, 0).expect("defender acts");
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == blade && t == serra
            ),
            "Doom Blade answers the attacker before blocks, got {action:?}"
        );
    }

    /// Sacrifice-for-value is judged by the resolved exchange: a
    /// sac-for-four-cards engine fires, a sac-for-one does not.
    #[test]
    fn sacrifice_value_judged_by_outcome() {
        use crate::card::CardType;
        use crate::effect::{ActivatedAbility, Selector, Value};
        let sac_drawer = |n: i32| CardDefinition {
            name: "Sac Engine",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(n) },
                ..Default::default()
            }],
            ..Default::default()
        };
        let w = EvalWeights::default();
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        for _ in 0..6 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.add_card_to_battlefield(0, sac_drawer(4));
        g.priority.player_with_priority = 0;
        assert!(
            pick_sacrifice_value(&g, 0, &w).is_some(),
            "a 1/1 into four cards is a trade worth making"
        );
        let mut weak = two_player_game();
        weak.step = TurnStep::PostCombatMain;
        for _ in 0..6 {
            weak.add_card_to_library(0, catalog::forest());
        }
        weak.add_card_to_battlefield(0, sac_drawer(1));
        assert!(
            pick_sacrifice_value(&weak, 0, &w).is_none(),
            "a 1/1 into one card is not"
        );
    }

    /// A self-costly optional trigger is judged by outcome at the real
    /// decision: pay 2 life for three cards, decline 8 life for one.
    #[test]
    fn optional_self_cost_taken_when_outcome_wins() {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{Selector, Value};
        use crate::game::TriggerPush;
        let run = |loss: i32, draw: i32| -> GameAction {
            use crate::card::{CardType, TriggeredAbility};
            use crate::effect::{EventKind, EventScope, EventSpec};
            let mut g = two_player_game();
            g.players[0].wants_ui = true;
            let body = Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(loss) },
                Effect::Draw { who: Selector::You, amount: Value::Const(draw) },
            ]);
            let maydo =
                Effect::MayDo { description: "you may".to_string(), body: Box::new(body) };
            // The prompt introspection reads the SOURCE's printed
            // definition, so the trigger must live on the card, exactly
            // as a real fired trigger does.
            let src_def = CardDefinition {
                name: "Optional Source",
                card_types: vec![CardType::Creature],
                power: 2,
                toughness: 2,
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: maydo.clone(),
                }],
                ..Default::default()
            };
            let src = g.add_card_to_battlefield(0, src_def);
            for _ in 0..5 {
                g.add_card_to_library(0, catalog::forest());
            }
            g.stack.push(TriggerPush::new(src, 0, maydo).build());
            let mut fuel = 20;
            while g.pending_decision.is_none() && fuel > 0 {
                g.perform_action(GameAction::PassPriority).unwrap();
                fuel -= 1;
            }
            assert!(matches!(
                g.pending_decision.as_ref().map(|p| &p.decision),
                Some(Decision::OptionalTrigger { .. })
            ));
            HeuristicBot::new().next_action(&g, 0).expect("bot answers")
        };
        assert!(
            matches!(run(2, 3), GameAction::SubmitDecision(DecisionAnswer::Bool(true))),
            "two life for three cards is taken"
        );
        assert!(
            matches!(run(8, 1), GameAction::SubmitDecision(DecisionAnswer::Bool(false))),
            "eight life for one card is declined"
        );
    }

    /// The race horizon scores the win: an attack that puts the opponent
    /// in range reads as the kill it sets up, not as a mid-race board
    /// snapshot.
    #[test]
    fn race_horizon_scores_the_win() {
        use crate::card::CardType;
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let brute = CardDefinition {
            name: "Race Brute",
            card_types: vec![CardType::Creature],
            power: 4,
            toughness: 4,
            ..Default::default()
        };
        let atk_id = g.add_card_to_battlefield(0, brute);
        g.clear_sickness(atk_id);
        g.players[1].life = 8;
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::forest());
            g.add_card_to_library(1, catalog::swamp());
        }
        let atk = vec![Attack { attacker: atk_id, target: AttackTarget::Player(1) }];
        let blind = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search_sim())
            .expect("one-cycle sim completes");
        let race = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search_race())
            .expect("race sim completes");
        assert!(
            race > blind && race >= 90_000,
            "the extended horizon reaches the win ({race} !> {blind} or short of decided)"
        );
    }

    /// A mana-fixing color choice reads the hand: holding double-green
    /// spells, "add one mana of any color" picks Green, not
    /// AutoDecider's first-legal White.
    #[test]
    fn choose_color_follows_hand_demand() {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::mana::Color;
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        g.add_card_to_hand(0, catalog::giant_growth()); // {G}
        g.add_card_to_hand(0, catalog::craw_wurm()); // {4}{G}{G}
        let d = Decision::ChooseColor {
            source: src,
            legal: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        };
        let ans = decide_pending_policy(&g, 0, &EvalWeights::default(), &d, false);
        assert!(
            matches!(ans, DecisionAnswer::Color(Color::Green)),
            "three green pips in hand → Green, got {ans:?}"
        );
    }

    /// SOS Converge: before casting a spell that scales with distinct
    /// colors of mana spent, the bot floats a color the pool lacks —
    /// tapping one source per tick — and only then casts, so the payment
    /// drains every college color instead of whatever the auto-tap
    /// grabbed first.
    #[test]
    fn converge_cast_prefloats_missing_colors() {
        use crate::card::CardType;
        use crate::effect::{Selector, Value};
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let mountain = g.add_card_to_battlefield(0, catalog::mountain());
        let island = g.add_card_to_battlefield(0, catalog::island());
        // {1}{R} "draw cards equal to converge" stand-in.
        let spell = CardDefinition {
            name: "Converge Test",
            cost: crate::mana::ManaCost::new(vec![
                crate::mana::ManaSymbol::Generic(1),
                crate::mana::ManaSymbol::Colored(Color::Red),
            ]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Draw { who: Selector::You, amount: Value::ConvergedValue },
            ..Default::default()
        };
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
        }
        let card = g.add_card_to_hand(0, spell);
        g.priority.player_with_priority = 0;
        // Tick 1: the bot taps a source instead of casting — floating a
        // color toward the converge count.
        let first = main_phase_action(&g, 0);
        assert!(
            matches!(
                first,
                GameAction::ActivateAbility { card_id, .. }
                    if card_id == mountain || card_id == island
            ),
            "first tick floats a color for the converge cast, got {first:?}"
        );
        g.perform_action(first).unwrap();
        // Tick 2: one color floated; with pool ≥ another new color still
        // missing but no room left (cmc 2, 1 floated) — the second tick
        // floats the second color or casts; drive to the cast and check
        // it happened with both colors drained.
        let mut fuel = 4;
        loop {
            let a = main_phase_action(&g, 0);
            let done = matches!(a, GameAction::CastSpell { card_id, .. } if card_id == card);
            g.perform_action(a).expect("bot line applies");
            if done {
                break;
            }
            fuel -= 1;
            assert!(fuel > 0, "prefloat must terminate in a cast");
        }
        // Both sources went into the payment: the cast drained R and U.
        assert!(
            g.battlefield.iter().filter(|c| c.controller == 0).all(|c| c.tapped),
            "both colors were tapped into the converge cast"
        );
    }

    /// Simulations answer decisions with the bot's own policy table:
    /// a pure-upside "you may draw" is TAKEN under
    /// `decide_pending_policy` where `AutoDecider` (the old sim
    /// decider) declines every optional trigger — the difference that
    /// made lookaheads undervalue every line with a beneficial rider.
    #[test]
    fn sim_policy_takes_beneficial_triggers() {
        use crate::card::{CardType, TriggeredAbility};
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let upside = CardDefinition {
            name: "Upside Rider",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "you may".to_string(),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, upside);
        let d = Decision::OptionalTrigger { source: id, description: "you may".to_string() };
        assert!(
            matches!(AutoDecider.decide(&d), DecisionAnswer::Bool(false)),
            "AutoDecider declines every optional trigger"
        );
        let ans = decide_pending_policy(&g, 0, &EvalWeights::default(), &d, false);
        assert!(
            matches!(ans, DecisionAnswer::Bool(true)),
            "the sim policy takes the pure-upside draw, got {ans:?}"
        );
    }

    /// X sizing honors multi-X costs: `{X}{X}{U}` with five mana up
    /// declares X=2 (pays {2}{2}{U}), not X=4 — and the same helper
    /// sizes prepare-cast inset spells that used to be stuck at X=0.
    #[test]
    fn multi_x_pip_costs_split_the_spare_mana() {
        use crate::mana::{Color, ManaCost, ManaSymbol};
        let mut g = two_player_game();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        let def = CardDefinition {
            name: "Double X Test",
            cost: ManaCost::new(vec![
                ManaSymbol::X,
                ManaSymbol::X,
                ManaSymbol::Colored(Color::Blue),
            ]),
            ..Default::default()
        };
        let x = max_affordable_x_for_def(&g, 0, &def, 0, &EvalWeights::default());
        assert_eq!(x, 2, "five mana into {{X}}{{X}}{{U}} is X=2");
    }

    /// The Increment threshold reads the smaller stat: a 2/3 Increment
    /// body wants casts spending 3+ mana, and the bar climbs as the
    /// body grows.
    #[test]
    fn increment_threshold_reads_smaller_stat() {
        use crate::card::CardType;
        use crate::effect::shortcut;
        let mut g = two_player_game();
        assert_eq!(increment_threshold(&g, 0), None, "no body, no bar");
        let body = CardDefinition {
            name: "Increment Test",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 3,
            triggered_abilities: vec![shortcut::increment_trigger(Effect::Noop)],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, body);
        assert!(is_increment_trigger(
            &g.battlefield_find(id).unwrap().definition.triggered_abilities[0]
        ));
        assert_eq!(increment_threshold(&g, 0), Some(3), "min(2,3)+1");
    }

    /// The on-cast family detectors tell the SOS trigger shapes apart:
    /// an Opus rider is not just magecraft, and an Infusion gate is
    /// found on spell bodies and triggered riders alike.
    #[test]
    fn on_cast_family_detectors() {
        use crate::effect::shortcut;
        use crate::effect::{Predicate, Selector, Value};
        let opus = shortcut::opus_trigger(Effect::Noop, Effect::Noop);
        assert!(is_opus_trigger(&opus), "opus_trigger shape detected");
        let mage = shortcut::magecraft(Effect::Noop);
        assert!(!is_opus_trigger(&mage), "plain magecraft is not Opus");
        assert!(is_repartee_trigger(&shortcut::repartee(Effect::Noop)));
        assert!(!is_repartee_trigger(&mage), "plain magecraft is not Repartee");
        let infusion = CardDefinition {
            name: "Infusion Test",
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: crate::effect::PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        };
        assert!(card_infusion_gated(&infusion));
        assert!(!card_infusion_gated(&catalog::shock()));
    }

    /// SOS Prepare — the inset spell is a one-shot resource on a fragile
    /// body: with opponent removal on the stack aimed at the prepared
    /// creature, the bot casts the inset instant in response instead of
    /// letting the counter die with the body.
    #[test]
    fn prepare_inset_instant_fires_in_response_to_removal() {
        use crate::card::CounterType;
        use crate::mana::Color;
        let mut g = two_player_game();
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_conflict());
        g.battlefield.iter_mut().find(|c| c.id == em).unwrap()
            .add_counters(CounterType::Prepared, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        // Opponent Doom Blades the prepared body, then passes priority.
        let blade = g.add_card_to_hand(1, catalog::doom_blade());
        g.players[1].mana_pool.add(Color::Black, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: blade,
            target: Some(Target::Permanent(em)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts removal");
        while g.player_with_priority() != 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let action = HeuristicBot::new().next_action(&g, 0).expect("bot holds priority");
        assert!(
            matches!(action, GameAction::CastPrepareSpell { creature_id, .. } if creature_id == em),
            "inset Lightning Bolt fires before the body dies, got {action:?}"
        );
    }

    /// The re-prepare mana sink: with spare mana and nothing better to do,
    /// Skycoach Waypoint's `{3},{T}` re-arms an unprepared prepare-spell
    /// creature.
    #[test]
    fn reprepare_sink_rearms_prepare_creature() {
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_conflict());
        let waypoint = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == waypoint && t == em
            ),
            "spare mana re-arms the prepare creature, got {action:?}"
        );
    }

    /// A Prepared counter on a prepare-spell body reads as material: the
    /// same creature is worth more prepared than not, so removal aims at
    /// it and the eval charges lines that waste the counter.
    #[test]
    fn prepared_counter_adds_permanent_value() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_conflict());
        let w = EvalWeights::default();
        let unprepared = permanent_value(&g, em, &w);
        g.battlefield.iter_mut().find(|c| c.id == em).unwrap()
            .add_counters(CounterType::Prepared, 1);
        let prepared = permanent_value(&g, em, &w);
        assert!(
            prepared > unprepared,
            "prepared must out-value unprepared ({prepared} !> {unprepared})"
        );
    }

    /// The Paradigm recurrence is a real choice under bot play: a free
    /// Decorum Dissertation copy (draw 2, lose 2 — the loss rides the
    /// auto-self-target) is taken at a healthy total and declined at a
    /// low one, instead of the old unconditional engine-side yes that
    /// drained the bot into the state-based loss two life at a time.
    #[test]
    fn paradigm_copy_declined_at_low_life() {
        use crate::card::CardInstance;
        use crate::decision::{Decision, DecisionAnswer};
        use crate::game::TriggerPush;
        let run_at = |life: i32| -> GameAction {
            let mut g = two_player_game();
            g.players[0].wants_ui = true;
            g.players[0].life = life;
            for _ in 0..4 {
                g.add_card_to_library(0, catalog::forest());
            }
            let id = g.next_id();
            g.exile.push(CardInstance::new(id, catalog::decorum_dissertation(), 0));
            g.stack.push(TriggerPush::new(id, 0, Effect::CastFreeParadigmCopy).build());
            let mut fuel = 20;
            while g.pending_decision.is_none() && fuel > 0 {
                g.perform_action(GameAction::PassPriority).unwrap();
                fuel -= 1;
            }
            assert!(
                matches!(
                    g.pending_decision.as_ref().map(|p| &p.decision),
                    Some(Decision::OptionalTrigger { .. })
                ),
                "paradigm copy must suspend as a real prompt, got {:?}",
                g.pending_decision
            );
            HeuristicBot::new().next_action(&g, 0).expect("bot answers")
        };
        let at_low = run_at(4);
        assert!(
            matches!(at_low, GameAction::SubmitDecision(DecisionAnswer::Bool(false))),
            "at 4 life the draw-2-lose-2 copy is declined, got {at_low:?}"
        );
        let at_healthy = run_at(20);
        assert!(
            matches!(at_healthy, GameAction::SubmitDecision(DecisionAnswer::Bool(true))),
            "at 20 life the free copy is taken, got {at_healthy:?}"
        );
    }

    /// Scry under bot play is no longer a no-op: with plenty of land
    /// sources a scried land goes to the bottom; while mana-light the
    /// same land stays on top, and an uncastable haymaker gets bottomed
    /// in favor of a cheap spell.
    #[test]
    fn scry_bottoms_flood_and_bricks() {
        use crate::decision::{DecisionAnswer, ScryMode};
        let id_of = |g: &GameState, name: &str| {
            g.players[0].library.iter().find(|c| c.definition.name == name).unwrap().id
        };
        // Flooded: six sources in play, a seventh on top → bottom it.
        let mut g = two_player_game();
        for _ in 0..6 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        g.add_card_to_library(0, catalog::forest());
        let land = id_of(&g, "Forest");
        let ans = decide_scry(&g, 0, &[(land, "Forest".into())], ScryMode::Scry);
        match ans {
            DecisionAnswer::ScryOrder { kept_top, bottom } => {
                assert!(kept_top.is_empty() && bottom == vec![land],
                    "at six sources a scried land is flood");
            }
            other => panic!("expected ScryOrder, got {other:?}"),
        }
        // Mana-light: one source, scrying land + 6-drop + Shock. Keep the
        // land (first) and the Shock; bottom the uncastable wurm.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::craw_wurm());
        g.add_card_to_library(0, catalog::shock());
        let (land, wurm, shock) =
            (id_of(&g, "Forest"), id_of(&g, "Craw Wurm"), id_of(&g, "Shock"));
        let cards = vec![
            (wurm, "Craw Wurm".into()),
            (land, "Forest".into()),
            (shock, "Shock".into()),
        ];
        let ans = decide_scry(&g, 0, &cards, ScryMode::Scry);
        match ans {
            DecisionAnswer::ScryOrder { kept_top, bottom } => {
                assert_eq!(kept_top, vec![land, shock],
                    "wanted land first, then the castable spell");
                assert_eq!(bottom, vec![wurm], "a 6-drop at one source is a brick");
            }
            other => panic!("expected ScryOrder, got {other:?}"),
        }
    }

    /// A mid-resolution modal is picked by outcome, not AutoDecider's
    /// blanket mode 0: a trigger offering [draw 1, draw 3] must answer
    /// mode 1.
    #[test]
    fn mode_decision_picked_by_outcome() {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{Selector, Value};
        use crate::game::TriggerPush;
        let mut g = two_player_game();
        g.players[0].wants_ui = true;
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::forest());
        }
        let draw = |n: i32| Effect::Draw { who: Selector::You, amount: Value::Const(n) };
        // `MODE_PICK_DEFERRED` is what `pick_trigger_mode` stamps on a
        // wants_ui controller's modal trigger so the pick suspends at
        // resolution instead of being answered inline by the decider.
        g.stack.push(
            TriggerPush::new(src, 0, Effect::ChooseMode(vec![draw(1), draw(3)]))
                .mode(Some(crate::game::types::MODE_PICK_DEFERRED))
                .build(),
        );
        // Pass priority until the modal suspends for the wants_ui seat.
        let mut fuel = 20;
        while g.pending_decision.is_none() && fuel > 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
            fuel -= 1;
        }
        assert!(
            matches!(
                g.pending_decision.as_ref().map(|p| &p.decision),
                Some(Decision::ChooseMode { .. })
            ),
            "trigger resolution must suspend on the modal, got {:?}",
            g.pending_decision
        );
        let mut bot = HeuristicBot::new();
        let action = bot.next_action(&g, 0).expect("bot answers its pending decision");
        assert!(
            matches!(
                action,
                GameAction::SubmitDecision(DecisionAnswer::Mode(1))
            ),
            "draw 3 beats draw 1, got {action:?}"
        );
    }

    /// Ad Nauseam under bot play: the per-reveal prompt suspends for the
    /// wants_ui seat and the bot keeps revealing only while the next card
    /// leaves a life buffer — it neither declines everything (the old
    /// AutoDecider path: zero cards) nor draws itself to death (the
    /// generic "can't introspect → yes" fallback).
    #[test]
    fn bot_pilots_ad_nauseam_with_life_buffer() {
        use crate::decision::Decision;
        let mut g = two_player_game();
        g.players[0].wants_ui = true;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Library of 3-mana cards: reveals at 20→17→14→11 life, then the
        // next reveal (11 - 3 = 8 ≤ 10 buffer) is declined.
        for _ in 0..10 {
            g.add_card_to_library(0, catalog::gray_ogre());
        }
        let nauseam = g.add_card_to_hand(0, catalog::ad_nauseam());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: nauseam, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .unwrap();
        // Resolve; answer each suspended reveal prompt with the bot.
        let mut bot = HeuristicBot::new();
        let mut guard = 0;
        loop {
            while g.pending_decision.is_none() && !g.stack.is_empty() {
                g.perform_action(GameAction::PassPriority).ok();
                let _ = g.perform_action(GameAction::PassPriority);
            }
            let Some(pd) = &g.pending_decision else { break };
            assert!(matches!(pd.decision, Decision::OptionalTrigger { .. }));
            let action = bot.next_action(&g, 0).expect("bot answers the reveal prompt");
            g.perform_action(action).unwrap();
            guard += 1;
            assert!(guard < 20, "reveal loop must terminate");
        }
        // 3 cards revealed (life 20 → 11), then stopped; -1 for the cast.
        assert_eq!(g.players[0].life, 11, "stopped with a life buffer");
        assert_eq!(
            g.players[0].hand.len(),
            hand_before - 1 + 3,
            "took exactly the comfortable reveals",
        );
    }

    /// A/B ladder: the scored candidate pick vs the legacy uniform-random
    /// pick, mirror decks, seats swapped every game. Expensive (full games),
    /// so `#[ignore]` — run manually:
    ///
    /// ```text
    /// cargo test -p crabomination --lib scored_pick_beats_uniform_baseline -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "expensive A/B ladder; run manually with --ignored"]
    fn scored_pick_beats_uniform_baseline() {
        use crate::player::Player;

        // Mirror match on the same limited-style 40-card creature deck so
        // deck strength cancels out and only the pilots differ. A fair
        // curve-and-removal deck (not the BRG combo deck, whose games are
        // decided by drawing the combo, drowning out play-skill signal).
        fn mirror_game() -> GameState {
            use rand::seq::SliceRandom;
            let deck: Vec<fn() -> CardDefinition> = {
                let mut d: Vec<fn() -> CardDefinition> = Vec::new();
                let mut push = |f: fn() -> CardDefinition, n: usize| {
                    for _ in 0..n {
                        d.push(f);
                    }
                };
                push(catalog::mountain, 17);
                push(catalog::lightning_bolt, 4);
                push(catalog::shock, 3);
                push(catalog::goblin_guide, 4);
                push(catalog::monastery_swiftspear, 3);
                push(catalog::gray_ogre, 3);
                push(catalog::hill_giant, 3);
                push(catalog::fire_elemental, 2);
                push(catalog::shivan_dragon, 1);
                d
            };
            let mut g = GameState::new(vec![Player::new(0, "Scored"), Player::new(1, "Uniform")]);
            let mut r = rng();
            for seat in 0..2 {
                for &f in &deck {
                    g.add_card_to_library(seat, f());
                }
                g.players[seat].library.shuffle(&mut r);
                g.players[seat].wants_ui = true;
            }
            g.start_mulligan_phase();
            g
        }

        const GAMES: usize = 300;
        let (mut scored_wins, mut uniform_wins, mut other) = (0u32, 0u32, 0u32);
        for i in 0..GAMES {
            let scored_seat = i % 2;
            let mut g = mirror_game();
            let mut bots: Vec<Box<dyn Bot>> = (0..2)
                .map(|s| -> Box<dyn Bot> {
                    if s == scored_seat {
                        Box::new(HeuristicBot::new())
                    } else {
                        Box::new(HeuristicBot::uniform_baseline())
                    }
                })
                .collect();
            // Poll both seats to a fixed point, server-actor style. `stale`
            // guards against a state where neither bot volunteers an
            // accepted action (counted as a draw below).
            let (mut actions, mut stale) = (0usize, 0usize);
            while !g.is_game_over() && actions < 50_000 && stale < crate::recommend::STALE_ROUNDS {
                let mut any = false;
                for (s, bot) in bots.iter_mut().enumerate() {
                    let Some(a) = bot.next_action(&g, s) else { continue };
                    if g.perform_action(a).is_ok() {
                        any = true;
                        actions += 1;
                        if g.is_game_over() {
                            break;
                        }
                    }
                }
                if any { stale = 0 } else { stale += 1 }
            }
            let _ = actions;
            match g.game_over {
                Some(Some(w)) if w == scored_seat => scored_wins += 1,
                Some(Some(_)) => uniform_wins += 1,
                _ => other += 1,
            }
        }
        let decided = scored_wins + uniform_wins;
        let pct = 100.0 * scored_wins as f64 / decided.max(1) as f64;
        println!(
            "scored {scored_wins} – uniform {uniform_wins} (draw/stall {other}): scored win rate {pct:.1}%",
        );
        assert!(
            decided >= (GAMES as u32) / 2,
            "too many undecided games ({other}/{GAMES}) — harness stalled, results meaningless",
        );
        assert!(
            pct >= 55.0,
            "scored pick should clearly beat the uniform baseline, got {pct:.1}%",
        );
    }
}

#[cfg(test)]
mod action_sampling_tests {
    use super::*;

    /// With no sampling installed (every gate, ladder, and real match),
    /// `choose_scored` is exactly the historical argmax: first index wins
    /// ties, best score wins otherwise.
    #[test]
    fn sampling_off_is_first_wins_ties_argmax() {
        set_action_sampling(None);
        assert_eq!(choose_scored(3, &[]), None);
        assert_eq!(choose_scored(3, &[(7, 100)]), Some(7));
        assert_eq!(choose_scored(3, &[(0, 100), (1, 100)]), Some(0), "ties keep index 0");
        assert_eq!(choose_scored(3, &[(0, 100), (1, 101)]), Some(1));
    }

    /// The turn cutoff is the schedule: sampling through `turns`, argmax
    /// after — and clearing the config clears it.
    #[test]
    fn sampling_respects_the_turn_cutoff() {
        set_action_sampling(Some((150, 5)));
        assert!(sampling_temp(1).is_some());
        assert!(sampling_temp(5).is_some());
        assert!(sampling_temp(6).is_none(), "past the cutoff is argmax");
        set_action_sampling(Some((0, 5)));
        assert!(sampling_temp(1).is_none(), "temp 0 is off");
        set_action_sampling(None);
        assert!(sampling_temp(1).is_none());
    }

    /// The softmax explores in proportion to score: a one-temperature gap
    /// is sampled but minority; a huge gap is effectively argmax; equal
    /// scores split. Seeded jitter makes the counts deterministic.
    #[test]
    fn softmax_explores_proportionally() {
        set_jitter_seed(Some(7));
        set_action_sampling(Some((150, 20)));

        let draws = |scored: &[(usize, i32)]| -> [usize; 2] {
            let mut n = [0usize; 2];
            for _ in 0..300 {
                n[choose_scored(1, scored).unwrap()] += 1;
            }
            n
        };
        // Gap of one temperature: better line dominates, worse line still
        // gets real visits (softmax weight ratio e ≈ 2.72 : 1).
        let n = draws(&[(0, 0), (1, 150)]);
        assert!(n[1] > n[0], "better line must dominate: {n:?}");
        assert!(n[0] > 30, "worse line must still be explored: {n:?}");
        // Gap of many temperatures: exploration vanishes.
        let n = draws(&[(0, 0), (1, 3000)]);
        assert!(n[0] == 0, "a 20-temperature gap should never sample the loser: {n:?}");
        // Equal scores: roughly even split.
        let n = draws(&[(0, 500), (1, 500)]);
        assert!(n[0] > 100 && n[1] > 100, "equal scores should split: {n:?}");

        set_action_sampling(None);
        set_jitter_seed(None);
    }

    /// Robustness: the sampler is total. `main_phase_action_with`'s
    /// sampling branch reaches it with the same candidate list the argmax
    /// branch beside it treats as "no action", and `scores.len() - 1` on an
    /// empty slice underflowed (debug) / returned a huge index that then
    /// indexed a slice (release). A panic on the actor path kills a
    /// training run at whatever game it happens to reach.
    #[test]
    fn sample_scored_index_is_total_on_an_empty_candidate_list() {
        set_jitter_seed(Some(11));
        assert_eq!(sample_scored_index(&[], 150.0), 0);
        assert_eq!(sample_scored_index(&[7], 150.0), 0);
        for _ in 0..50 {
            assert!(sample_scored_index(&[3, 1, 2], 150.0) < 3);
        }
        set_jitter_seed(None);
    }

}

#[cfg(test)]
mod damage_order_tests {
    use super::*;
    use crate::catalog;
    use crate::game::{GameState, TurnStep};
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    fn order_of(a: crate::decision::DecisionAnswer) -> Vec<crate::card::CardId> {
        match a {
            crate::decision::DecisionAnswer::DamageOrder(ids) => ids,
            other => panic!("expected DamageOrder, got {other:?}"),
        }
    }

    /// CR 510.1c — the order policy kills the most valuable victims the
    /// power can pay for, and the signed value makes the same policy
    /// correct from the defender's chair (banding / Defensive Formation
    /// hand the decision to the victims' controller).
    #[test]
    fn damage_order_kills_by_value_from_either_chair() {
        let mut g = two_player_game();
        let dealer = g.add_card_to_battlefield(1, catalog::hill_giant());
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
        let w = EvalWeights::damage_order_on();
        let victims = vec![(bears, String::new()), (giant, String::new())];

        // The attacker's chair: three power pays for the 3/3, or for the
        // 2/2 with a point to spare. Declaration order killed the 2/2;
        // the policy takes the 3/3.
        let ids = order_of(decide_combat_damage_order(&g, 1, dealer, &victims, &w));
        assert_eq!(ids.first(), Some(&giant), "attacker kills the better blocker");

        // The defender's chair: same decision, victims' controller
        // deciding — feed the 2/2, keep the 3/3.
        let victims_rev = vec![(giant, String::new()), (bears, String::new())];
        let ids = order_of(decide_combat_damage_order(&g, 0, dealer, &victims_rev, &w));
        assert_eq!(ids.first(), Some(&bears), "defender protects the better creature");
    }

    /// The strict-improvement rule: interchangeable victims mean no order
    /// beats the default, the answer is empty, the engine keeps
    /// declaration order — and an antithetic mirror pair stays a mirror.
    #[test]
    fn damage_order_answers_empty_without_a_strict_improvement() {
        let mut g = two_player_game();
        let dealer = g.add_card_to_battlefield(1, catalog::hill_giant());
        let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let w = EvalWeights::damage_order_on();
        let victims = vec![(b1, String::new()), (b2, String::new())];
        let a = decide_combat_damage_order(&g, 1, dealer, &victims, &w);
        assert!(order_of(a).is_empty(), "identical victims: keep the default");
    }

    /// CR 702.2c — deathtouch makes one point lethal, so a two-power
    /// dealer picks WHICH two die rather than paying full toughness.
    /// Discriminating: without the deathtouch lethal, two power could
    /// never put the 3/3 first.
    #[test]
    fn damage_order_prices_deathtouch_lethal() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        let dealer = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let di = g.battlefield.iter().position(|c| c.id == dealer).unwrap();
        g.battlefield[di].granted_keywords_eot.push(Keyword::Deathtouch);
        let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
        let w = EvalWeights::damage_order_on();
        let victims = vec![(b1, String::new()), (b2, String::new()), (giant, String::new())];
        let ids = order_of(decide_combat_damage_order(&g, 1, dealer, &victims, &w));
        // Two points, one lethal each: the first two in the order die.
        // Any optimum kills the 3/3 plus one bear — the 3/3's position
        // within the pair is a tie the policy may break either way.
        assert!(
            ids.iter().take(2).any(|id| *id == giant),
            "deathtouch: the 3/3 is among the two that die, got {ids:?}"
        );
    }
}

#[cfg(test)]
mod target_eval_tests {
    use super::*;
    use crate::catalog;
    use crate::effect::{Effect, Selector, Value};
    use crate::game::types::PendingTriggerPush;
    use crate::game::{GameState, TurnStep};
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g.players[0].wants_ui = true;
        g
    }

    /// Fire `effect` as seat 0's trigger through the real suspending path
    /// (`drain_trigger_queue`) and return the pending ChooseTarget.
    fn suspend_on_target(g: &mut GameState, source: crate::card::CardId, effect: Effect) {
        g.drain_trigger_queue(vec![PendingTriggerPush {
            actor: Some(0),
            source,
            controller: 0,
            effect,
            subject: None,
            event_amount: 0,
            mode: None,
            intervening_if: None,
            from_mana_ability: false,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
        }]);
        assert!(
            matches!(
                g.pending_decision.as_ref().map(|p| &p.decision),
                Some(crate::decision::Decision::ChooseTarget { .. })
            ),
            "fixture: the trigger must suspend on a target pick"
        );
    }

    fn answer(g: &GameState, w: &EvalWeights) -> crate::decision::DecisionAnswer {
        let pending = g.pending_decision.as_ref().unwrap();
        decide_pending_policy(g, pending.acting_player(), w, &pending.decision, true)
    }

    /// A beneficial trigger whose legal set spans both sides: the polarity
    /// guess buffs the opponent's biggest creature; the settled outcome
    /// puts the counters on our own. This is the round-46 classifier gap
    /// at the raise site `target_arms` never touched.
    #[test]
    fn a_beneficial_trigger_targets_our_own_side() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
        suspend_on_target(
            &mut g,
            mine,
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        );
        // The guess, documented: biggest opposing permanent.
        match answer(&g, &EvalWeights::default()) {
            crate::decision::DecisionAnswer::Target(crate::game::types::Target::Permanent(id)) => {
                assert_eq!(id, theirs, "the polarity guess buffs THEIR creature")
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
        // The settled outcome: our own creature.
        match answer(&g, &EvalWeights::target_eval_on()) {
            crate::decision::DecisionAnswer::Target(crate::game::types::Target::Permanent(id)) => {
                assert_eq!(id, mine, "the settled outcome buffs OURS")
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// Undersized removal: two damage kills their 2/2 outright but only
    /// marks their 3/3. The guess targets the biggest; the outcome takes
    /// the kill. Also the mirror-discipline half: the improvement is
    /// strict, so when the guess and the outcome agree the flag answers
    /// exactly as the guess did.
    #[test]
    fn undersized_removal_takes_the_kill_over_the_mark() {
        let mut g = two_player_game();
        let source = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let big = g.add_card_to_battlefield(1, catalog::hill_giant());
        suspend_on_target(
            &mut g,
            source,
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        );
        match answer(&g, &EvalWeights::default()) {
            crate::decision::DecisionAnswer::Target(crate::game::types::Target::Permanent(id)) => {
                assert_eq!(id, big, "the guess aims at the biggest")
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
        match answer(&g, &EvalWeights::target_eval_on()) {
            crate::decision::DecisionAnswer::Target(crate::game::types::Target::Permanent(id)) => {
                assert_eq!(id, small, "the outcome takes the kill: {small:?} dies, {big:?} heals")
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod tail_guard_tests {
    use super::*;
    use crate::catalog;
    use crate::game::{GameState, TurnStep};
    use crate::player::Player;
    use std::sync::Arc;

    /// A net that answers every state with one constant probability —
    /// exactly what a saturated head looks like to the decision loop.
    struct ConstNet(f32);
    impl crabomination_nn::NetEvaluator for ConstNet {
        fn eval(&self, _s: crabomination_nn::EncodedState) -> f32 {
            self.0
        }
    }

    /// The net slots are process-global; serialize the tests that
    /// install one, and always uninstall on the way out.
    static SLOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_const_net<R>(p: f32, f: impl FnOnce(u8) -> R) -> R {
        let _guard = SLOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let slot = super::super::net_eval::SLOT_CANDIDATE;
        super::super::net_eval::set_slot(slot, Some(Arc::new(ConstNet(p))));
        let out = f(slot);
        super::super::net_eval::set_slot(slot, None);
        out
    }

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 1;
        g
    }

    /// The switch itself: a saturated read silences the net for the
    /// decision, a mid-band read leaves it alone, and the flag gates it.
    #[test]
    fn the_guard_trips_exactly_in_the_saturated_band() {
        let g = two_player_game();
        let mut w = EvalWeights::net_tail_guard_on();
        with_const_net(0.01, |slot| {
            w.net_slot = slot;
            assert_eq!(tail_guarded(&g, 1, &w).net_slot, 0, "p=0.01 is unrankable");
        });
        with_const_net(0.5, |slot| {
            w.net_slot = slot;
            assert_eq!(tail_guarded(&g, 1, &w).net_slot, slot, "mid-band keeps the net");
        });
        with_const_net(0.96, |slot| {
            w.net_slot = slot;
            assert_eq!(tail_guarded(&g, 1, &w).net_slot, 0, "the winning tail saturates too");
        });
        with_const_net(0.01, |slot| {
            let mut off = EvalWeights::net_eval_det1();
            off.net_slot = slot;
            assert_eq!(tail_guarded(&g, 1, &off).net_slot, slot, "flag off: unchanged");
        });
    }

    /// The contract on a game-5-shaped board (flying 5/5 the defender
    /// cannot block, attacker far behind on life): under a saturated
    /// net, the guarded picker declares the attack the material eval
    /// finds — the same answer the pure material weights give — instead
    /// of whatever the flat landscape's tie-break falls into.
    #[test]
    fn a_saturated_net_falls_back_to_the_material_attack() {
        let mut g = two_player_game();
        g.turn_number = 20;
        g.players[1].life = 3;
        g.players[0].life = 23;
        let flyer = g.add_card_to_battlefield(1, catalog::emeritus_of_ideation());
        g.clear_sickness(flyer);
        let wall = g.add_card_to_battlefield(0, catalog::pensive_professor());
        g.clear_sickness(wall);

        let material_choice = {
            let w = EvalWeights::net_eval_det1();
            // net_slot stays SLOT_BEST but nothing is loaded there in
            // tests, so this IS the material path.
            pick_attacks_scored(&g, 1, &w)
        };
        assert!(
            material_choice.iter().any(|a| a.attacker == flyer),
            "fixture: the material eval must want the free flying attack"
        );

        let guarded_choice = with_const_net(0.01, |slot| {
            let mut w = EvalWeights::net_tail_guard_on();
            w.net_slot = slot;
            pick_attacks_scored(&g, 1, &w)
        });
        assert_eq!(
            guarded_choice, material_choice,
            "saturated read: the guarded picker gives the material answer"
        );
    }
}
