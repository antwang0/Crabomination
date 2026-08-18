//! selfplay_train — the concurrent train-and-play loop for the SOS sealed
//! value net.
//!
//! KataGo-shaped, single process (see FEATURE_ROADMAP Tier 13): actor
//! threads play sealed self-play games and push encoded rows into a shared
//! sliding window while the learner thread trains from it, throttled by a
//! sample-reuse cap so it can never grind the recent window into overfit.
//! Checkpoints land in `--out` as `latest.safetensors` (atomic rename), and
//! a `stats.jsonl` line per checkpoint records the loss curve.
//!
//! Bootstrap phase: actors play with the heuristic default bot and the
//! heuristic sealed builder — the net is a passenger until it can pass a
//! gate (the gatekeeper and the net-eval bot profile are the next step;
//! nothing here reads the net back yet).
//!
//! ```text
//! selfplay_train [--actors N] [--games N] [--steps N] [--batch N]
//!                [--lr F] [--reuse F] [--lambda F] [--relabel-every N]
//!                [--window N] [--min-window N]
//!                [--checkpoint-every N] [--out DIR] [--seed N]
//!                [--use-best WEIGHTS.safetensors] [--seed-emb DECK.safetensors]
//!                [--gpu-eval] [--eval-batch N] [--eval-flush-us N]
//!                [--stop-after-stale N] [--relabel-mode full|new]
//!                [--attn] [--blocks N] [--aux] [--ablate lib,cast,rel,combat,kw]
//!                [--muon] [--muon-lr F] [--lr-cosine STEPS]
//!                [--sample-temp N] [--sample-turns N] [--mcts-actors N]
//!                [--record-decisions] [--policy-every N] [--policy-temp F]
//!                [--policy-head] [--mcts-gumbel] [--best-metric auc|policy]
//!                [--mcts-fleet N] [--opp-head] [--search-value-weight F]
//!                [--emb-dim N] [--obj-hidden N] [--h1 N] [--h2 N]
//!                [--gate-builder GAMES_PER_POOL] [--gate-builder-v3 GAMES_PER_POOL]
//!
//! The build net (Phase C) trains alongside the play net from the same
//! games — every decided game labels its two decklists — and checkpoints
//! as `deck-latest.safetensors`. `--gate-builder N` skips training and
//! races net-judged best-of-32 builds against static-judged best-of-32
//! of the *same* candidate sets on paired pools: the judges differ, the
//! candidates and pilots don't, so the result isolates the judge.
//! ```
//!
//! With `--games`/`--steps` unset it runs until killed; the latest
//! checkpoint is at most `--checkpoint-every` steps stale, and a fresh run
//! with the same `--out` resumes from `latest.safetensors` if the shapes
//! still match.
//!
//! Measured on the first smoke run (22 actors, debug build, defaults):
//! ~10 SOS sealed games/s (~36 rows each) with the learner sharing the
//! box, loss EMA 0.371 → 0.216 over 600 steps of batch 256, and the
//! reuse throttle engaging within 3 % of the 6× cap. Generation is the
//! bottleneck by design — the learner spends most wall clock waiting,
//! which is the sample-reuse cap doing its job.

// Allocator swap, on by default. The simulator spends ~16 % of its
// instructions in malloc/free/memcpy and each actor is one thread playing one
// game, so system-malloc contention grows with the actor count: measured
// +18-31 % throughput across 4-24 actors for a flat ~1.5-1.7x RSS (PERF.md).
// `--no-default-features` restores the system allocator. A
// `#[global_allocator]` is a whole-program choice, so it lives in the binary.
#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use crabomination::selfplay::{
    best_build_by, heuristic_sealed_build, hill_climb_build_by, mutate_build,
    play_recorded_game, sealed_game_template, sealed_pool, static_deck_score,
};
use crabomination::recommend::StopReason;
use crabomination::server::bot::EvalWeights;
use crabomination::server::encode::{Vocab, encode_deck};
use crabomination_ml::{DeckNetConfig, DeckTrainer, NetConfig, SampleWindow, Trainer};
use crabomination_nn::{DeckNet, DeckRow, TrainRow};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

struct Args {
    actors: usize,
    games: Option<u64>,
    steps: Option<u64>,
    batch: usize,
    lr: f64,
    reuse: f64,
    /// TD(lambda) mixing for the win target. 1.0 (default) is the
    /// historical pure Monte Carlo label — the row's own game result —
    /// so every prior gate round is reproducible. Below 1.0 the target
    /// bootstraps through the net's estimate of the next snapshot, which
    /// trades the variance of a twenty-turn outcome for the bias of the
    /// Fit the win head with cross-entropy instead of MSE
    /// (`crabomination_ml::binary_cross_entropy`). MSE through a sigmoid
    /// vanishes the gradient exactly where the net is confidently wrong;
    /// BCE does not. Off by default so the control arm is the historical
    /// behaviour. NOTE: with this on, `loss_win` in stats.jsonl is a
    /// log-loss and is not comparable to an MSE run's number -- the
    /// checkpoint's `train_raw` / `val_win` stay MSE either way and are
    /// what to compare across arms.
    bce: bool,
    /// Record the pilot's decisions (candidate successor states + the
    /// pick) and train the win head to rank them. See
    /// `crabomination::server::decision_capture` and
    /// `Trainer::train_policy_step`. Off by default; recording encodes
    /// once per candidate rather than once per position, so it is not
    /// free.
    record_decisions: bool,
    /// Learner steps between policy steps when `--record-decisions` is
    /// on. 1 = a policy step every value step.
    ///
    /// **0 means record and measure but do not train** — the control arm
    /// this experiment needs. `val_policy` (held-out top-1 agreement) is
    /// meaningless without a baseline from a net that never saw a policy
    /// gradient, and an arm that simply omits `--record-decisions`
    /// cannot supply one: it reports no `val_policy` at all. So the
    /// honest control records the same decisions, scores the same
    /// holdout, and trains on none of it.
    policy_every: u64,
    /// Softmax temperature for the distilled policy target. Arm values
    /// are win probabilities, so their spread is small and this is what
    /// decides sharp-pick vs soft-ranking. Ignored for Gumbel captures,
    /// whose improved-policy logits carry their own scale.
    policy_temp: f32,
    /// Build the separate policy head (`head_policy.*`, round 37) and
    /// aim the ranking objective at it instead of at the win head. The
    /// head is exported and the engine consumes it as the Gumbel
    /// search's prior. Off is the control: the round-33 shared-win-head
    /// path, bit-reachable. Only does anything under
    /// `--record-decisions` with `--policy-every` > 0.
    policy_head: bool,
    /// Pilot the MCTS actors with the Gumbel root search (Sequential
    /// Halving over policy-head priors) instead of UCB1 — the gen-1
    /// loop, once a policy-headed `--use-best` exists to pilot with. On
    /// a headless pilot the search falls back to heuristic-score priors
    /// and says so at startup. Requires `--mcts-actors`.
    mcts_gumbel: bool,
    /// Which holdout metric publishes `best.safetensors`: false = AUC
    /// (the historical rule), true (`--best-metric policy`) = held-out
    /// top-1 policy agreement. Exists because round 34 measured AUC and
    /// ranking moving in *opposite* directions on one seed — selecting a
    /// distillation run's artifact on AUC can publish its worst ranking
    /// checkpoint while the run reports success.
    best_policy: bool,
    /// Build and train the opponent-hand belief head (`head_opp.*`,
    /// round 39): per-name hold probabilities fit against the recorded
    /// truth with multi-label BCE at the auxiliary weight. Engine-
    /// consumed (belief-weighted determinization), so exports carry it.
    /// Off is the control.
    opp_head: bool,
    /// Weight of the search-value regression inside the policy step
    /// (round 39, Reanalyze-lite): the win head pulled toward the
    /// search's per-arm mean rewards on successor states, MSE weighted
    /// by rollout counts. 0 (default) = off, the historical control.
    /// UCB1 captures only — Gumbel improved-policy logits are not win
    /// probabilities. Needs `--record-decisions` and MCTS actors to
    /// have anything to regress against.
    search_value_weight: f32,
    /// current net (see `SampleWindow::relabel_lambda`).
    lambda: f32,
    /// Learner steps between recomputing the lambda-returns. They are
    /// functions of the net, so they drift as it trains; recomputing
    /// costs one forward pass over the window. Ignored at lambda = 1.
    relabel_every: u64,
    window: usize,
    min_window: u64,
    checkpoint_every: u64,
    out: PathBuf,
    seed: u64,
    /// Weights for the actors to *play* with (a gate-passed promotion) —
    /// this is what closes the self-improvement loop. Unset, actors play
    /// the heuristic default (the bootstrap phase).
    use_best: Option<PathBuf>,
    /// Override the heuristic actors' `determinize` (round 28c). The
    /// r25 adoption flipped the *default* pilot to honest sims, which
    /// silently changed the training data distribution too; 0 restores
    /// the pre-adoption generator for attribution.
    actor_det: Option<u8>,
    /// Window-passes of tail training after the actors finish (round
    /// 28d). The old fixed allowance (reuse/2 passes) assumed the
    /// learner streamed most of its budget during generation; a faster
    /// container halved the streaming phase and the allowance silently
    /// became the binding stop — runs died at ~8k steps mid-improvement
    /// with stop-after-stale never firing. Raise it and let stale be
    /// the stopper it was built to be.
    tail_reuse: f64,
    /// Serve `--use-best` evals from a batched device-side collator
    /// instead of per-thread CPU forwards. Pair with a large `--actors`
    /// (hundreds): batch size is bounded by the number of games blocked
    /// on an eval at once, so 22 threads can't fill a 256 batch.
    gpu_eval: bool,
    /// Collator limits: flush a batch at this many states...
    eval_batch: usize,
    /// ...or after this many microseconds, whichever comes first.
    eval_flush_us: u64,
    /// Gate mode: skip training, load `<out>/deck-latest.safetensors`,
    /// and race net-judged builds against static-judged builds of the
    /// same candidate sets on paired pools, N games per pool.
    gate_builder: Option<usize>,
    /// Distillation, generation half: label N decks with a gauntlet win
    /// rate (240 games each vs a fixed 20-deck field) and append to
    /// `<out>/deck_labels.bin`. The deck mix deliberately covers the
    /// space search visits — builder picks, best-of-32 picks, 3/8/15-swap
    /// mutants, and climb trajectories under the current deck net.
    distill_gen: Option<usize>,
    /// Distillation, training half: fit the deck net to the gauntlet
    /// win rates in `<out>/deck_labels.bin`, report holdout pair-order
    /// accuracy, save `<out>/deck-distilled.safetensors`.
    distill_train: bool,
    /// Gate mode: race hill-climbed builds (single-spell-swap search
    /// under the deck-net judge, started from the net's best-of-32 pick)
    /// against the plain best-of-32 pick — same pools, same pilots, same
    /// judge, so the result isolates the *search*. N games per pool.
    gate_builder_hc: Option<usize>,
    /// Gate mode: race the repaired sealed builder (`builder_v2`)
    /// against the one it replaces, same pools, same pilots, N games
    /// per pool. No net involved — this measures the builder alone.
    gate_builder_v2: Option<usize>,
    /// Gate mode: two races against the single-sample v2 build on the
    /// same pools — the v3 shape ranker as a single sample (isolates
    /// the scoring change), then best-of-16 v3 (the client's sealed
    /// opponent recipe, so the combined effect the player meets). No
    /// net involved.
    gate_builder_v3: Option<usize>,
    /// Fraction of trajectories held out of training and scored at every
    /// checkpoint. 0 disables.
    ///
    /// This exists because its absence cost six gate rounds. `stats.jsonl`
    /// reported training loss only, and training loss here is a
    /// memorisation reading: at lambda=1 the net hit 0.017 MSE on the
    /// window while its out-of-sample log-loss was 1.12 — *worse than
    /// predicting 0.5 every time*. Nothing in the loop could see that, so
    /// the overfit was diagnosed only after thousands of gate games.
    ///
    /// Split by trajectory, not by row: consecutive snapshots of one game
    /// are near-duplicates, so a row-level split leaks the answer across
    /// it and the validation number comes back reassuringly good.
    holdout: f64,
    /// Train the play net with the pre-pool attention layer — the
    /// interaction model. Off is the pooled control.
    attn: bool,
    /// Train the short-horizon aux head (next-snapshot life/power/
    /// creature deltas + opponent hand). Off is the control; the engine
    /// ignores the extra tensors either way.
    aux: bool,
    /// Pre-pool transformer blocks (`--blocks N`): pre-LN attention +
    /// 2× FFN per block, group tag in the stream. 0 (default) is off;
    /// mutually exclusive with `--attn`, which it supersedes.
    blocks: usize,
    /// Train with the Muon hybrid optimizer (orthogonalized updates on
    /// hidden matrices; AdamW at `--lr` for embeddings/heads/biases).
    muon: bool,
    /// Muon step size — RMS-1 update semantics, ~100× an AdamW lr.
    muon_lr: f64,
    /// Cosine-decay the lr from its base to a 10 % floor over this many
    /// steps (`cosine_lr_factor`). 0 (default) = constant lr. The
    /// horizon should approximate the *expected* run length under early
    /// stop, not the `--steps` cap.
    lr_cosine: u64,
    /// Actor-side softmax action sampling: temperature in eval units
    /// (0 = off). ~120 is one mid-size creature on the heuristic scale;
    /// ~300 ≈ 3 % win probability on the net's ±10 000 scale.
    sample_temp: i32,
    /// Sample through this turn, argmax after (AlphaZero-style cutoff).
    sample_turns: u32,
    /// Pilot actors with net-evaluated MCTS at this many rollouts per
    /// decision (0 = the plain scored bot). Requires `--use-best`; pair
    /// with `--gpu-eval` — rollout rewards go through the same slot, so
    /// they batch on the collator automatically. Generation is ~50–100×
    /// slower per game than heuristic actors: size `--games` from a
    /// measured probe, not hope.
    mcts_actors: u32,
    /// Mixed actor fleet (round 37 — the KataGo playout-cap-
    /// randomization idea translated to this loop's shape): only the
    /// first N actor threads pilot with MCTS; the rest play the plain
    /// net pilot and record no decisions (thread-scoped capture
    /// override). 0 (default) = every actor is MCTS when
    /// `--mcts-actors` is set, the historical all-or-nothing.
    ///
    /// The point is that the two data streams have different budgets:
    /// policy targets need *searched* decisions (~1.5 games/s at
    /// MCTS-64), while the value window needs volume (~50–100× that) —
    /// and an all-MCTS fleet leaves the learner asleep ~85 % of wall
    /// clock on the reuse throttle (r35/r37 measured). A mixed fleet
    /// feeds both at once. Pair with a large `--actors` (hundreds):
    /// every fleet member spends most of its time blocked in the
    /// batched evaluator, so thread count, not core count, is what
    /// fills batches and cores alike.
    mcts_fleet: usize,
    /// Width overrides — the engine reads sizes from the tensor shapes,
    /// so capacity is a flag, not a format change. `--seed-emb` requires
    /// the deck net's width, so it refuses a changed `--emb-dim`.
    emb_dim: Option<usize>,
    obj_hidden: Option<usize>,
    h1: Option<usize>,
    h2: Option<usize>,
    /// Stop the whole run — actors included — once the holdout AUC has
    /// gone this many checkpoints without a new best. 0 (default) never
    /// stops, the historical behaviour.
    ///
    /// This is the wall-clock lever: on the round-11 `full` run the
    /// holdout AUC peaked at step 4 000, 112 s into a 1 538 s run, and
    /// fell ~0.07 over the rest. `best.safetensors` rescues the artifact
    /// but not the compute. Needs a holdout to see (`--holdout` > 0);
    /// checkpoints without a scored holdout don't count either way. Note
    /// stopping early also truncates *generation*, so the peak is being
    /// compared against a run with more unique data — A/B against a
    /// no-stop control before trusting a tuned value.
    stop_after_stale: u64,
    /// `full` (default) recomputes every window row's λ-return each
    /// relabel tick — bit-identical to all prior runs. `new` relabels
    /// only rows pushed since the last tick
    /// (`SampleWindow::relabel_lambda_new_rows`): on the round-11 runs
    /// the full pass spent 66 M forward-rows against 13.5 M trained and
    /// held the learner to 1.56× of its 6× reuse cap. Ignored at λ = 1.
    relabel_new: bool,
    /// Build training decks with the gate-passed deck net as the judge
    /// (best-of-32 over the same noisy-greedy candidates the heuristic
    /// picks from) instead of taking the heuristic builder's own pick.
    ///
    /// The deck net is the one learned component that has cleared the
    /// house bar — twice, at 61.7 % and 60.7 % against the static judge —
    /// so the decks the actors play should be its picks. Until this flag
    /// the gate result had no consumer: every training game was still
    /// played with heuristic builds.
    use_deck_best: Option<PathBuf>,
    /// Initialize the play net's card embeddings from a trained deck net
    /// rather than from noise. See
    /// `Trainer::seed_embeddings_from_deck_net`. Ignored when the run
    /// resumes from an existing `latest.safetensors`.
    seed_emb: Option<PathBuf>,
    /// Ablation control: comma-separated feature blocks to switch *off*
    /// in the encoder. See
    /// `crabomination::server::encode::ABLATION_BLOCKS` for the names;
    /// an unrecognised one exits rather than silently ablating nothing.
    ablate: Vec<String>,
    /// Diagnostic mode: local-discrimination test over N games. See
    /// `pairwise`.
    pairwise: Option<usize>,
    /// Diagnostic mode: play N self-play games and score the value net
    /// and the heuristic evaluation as *predictors of the winner* on the
    /// same positions. Requires `--use-best`.
    calibrate: Option<usize>,
    /// Diagnostic mode: like `--calibrate`, but scored on the *simulated
    /// leaves* the searches actually evaluate (captured via
    /// `server::leaf_capture`) alongside the snapshot positions from the
    /// same games. This is the direct test of the last standing
    /// explanation for "better predictor, worse pilot": a net that holds
    /// its edge on snapshots and loses it on sim leaves would produce
    /// exactly that pattern while every other instrument reads fine.
    calibrate_leaves: Option<usize>,
    /// Diagnostic mode: play N games and report how often each encoder
    /// feature is non-zero in the resulting positions. See
    /// [`feature_census`].
    feature_census: Option<usize>,
    /// Record snapshots at declare-blockers and end-of-combat as well as
    /// at the turn/main/end points every run through round 39 used. See
    /// `crabomination::selfplay::set_combat_snapshots` — without it no
    /// training row is ever a combat row, and the combat features the
    /// search feeds at inference time have never had a gradient.
    record_combat: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        actors: std::thread::available_parallelism().map(|n| n.get().saturating_sub(2)).unwrap_or(4).max(1),
        games: None,
        steps: None,
        batch: 256,
        lr: 1e-3,
        reuse: 6.0,
        bce: false,
        record_decisions: false,
        policy_every: 4,
        policy_temp: 0.1,
        policy_head: false,
        mcts_gumbel: false,
        best_policy: false,
        opp_head: false,
        search_value_weight: 0.0,
        lambda: 1.0,
        relabel_every: 200,
        window: 250_000,
        min_window: 20_000,
        checkpoint_every: 2_000,
        out: PathBuf::from("nets"),
        seed: 0x0505_ACAD,
        use_best: None,
        actor_det: None,
        tail_reuse: 3.0,
        gpu_eval: false,
        eval_batch: 256,
        eval_flush_us: 1000,
        gate_builder: None,
        gate_builder_hc: None,
        distill_gen: None,
        distill_train: false,
        gate_builder_v2: None,
        gate_builder_v3: None,
        calibrate: None,
        calibrate_leaves: None,
        feature_census: None,
        record_combat: false,
        pairwise: None,
        use_deck_best: None,
        seed_emb: None,
        ablate: Vec::new(),
        attn: false,
        aux: false,
        blocks: 0,
        muon: false,
        muon_lr: 0.02,
        lr_cosine: 0,
        sample_temp: 0,
        sample_turns: 6,
        mcts_actors: 0,
        mcts_fleet: 0,
        emb_dim: None,
        obj_hidden: None,
        h1: None,
        h2: None,
        holdout: 0.05,
        stop_after_stale: 0,
        relabel_new: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--actors" => a.actors = val().parse().expect("--actors"),
            "--games" => a.games = Some(val().parse().expect("--games")),
            "--steps" => a.steps = Some(val().parse().expect("--steps")),
            "--batch" => a.batch = val().parse().expect("--batch"),
            "--lr" => a.lr = val().parse().expect("--lr"),
            "--reuse" => a.reuse = val().parse().expect("--reuse"),
            "--bce" => a.bce = true,
            "--record-decisions" => a.record_decisions = true,
            "--record-combat" => a.record_combat = true,
            "--policy-temp" => a.policy_temp = val().parse().expect("--policy-temp"),
            "--policy-head" => {
                a.policy_head = true;
                continue; // bare flag, consumes no value
            }
            "--mcts-gumbel" => {
                a.mcts_gumbel = true;
                continue; // bare flag, consumes no value
            }
            "--best-metric" => {
                let m = val();
                a.best_policy = match m.as_str() {
                    "policy" => true,
                    "auc" => false,
                    other => panic!("--best-metric: {other:?} (expected auc or policy)"),
                };
            }
            "--opp-head" => {
                a.opp_head = true;
                continue; // bare flag, consumes no value
            }
            "--search-value-weight" => {
                a.search_value_weight = val().parse().expect("--search-value-weight")
            }
            "--policy-every" => {
                a.policy_every = val().parse::<u64>().expect("--policy-every")
            }
            "--lambda" => a.lambda = val().parse().expect("--lambda"),
            "--relabel-every" => {
                a.relabel_every = val().parse::<u64>().expect("--relabel-every").max(1)
            }
            "--window" => a.window = val().parse().expect("--window"),
            "--min-window" => a.min_window = val().parse().expect("--min-window"),
            "--checkpoint-every" => a.checkpoint_every = val().parse().expect("--checkpoint-every"),
            "--out" => a.out = PathBuf::from(val()),
            "--seed" => a.seed = val().parse().expect("--seed"),
            "--use-best" => a.use_best = Some(PathBuf::from(val())),
            "--actor-det" => a.actor_det = Some(val().parse().expect("--actor-det")),
            "--tail-reuse" => a.tail_reuse = val().parse().expect("--tail-reuse"),
            "--gpu-eval" => {
                a.gpu_eval = true;
                continue; // bare flag, consumes no value
            }
            "--eval-batch" => a.eval_batch = val().parse().expect("--eval-batch"),
            "--eval-flush-us" => a.eval_flush_us = val().parse().expect("--eval-flush-us"),
            "--calibrate" => a.calibrate = Some(val().parse().expect("--calibrate")),
            "--calibrate-leaves" => {
                a.calibrate_leaves = Some(val().parse().expect("--calibrate-leaves"))
            }
            "--feature-census" => {
                a.feature_census = Some(val().parse().expect("--feature-census"))
            }
            "--pairwise" => a.pairwise = Some(val().parse().expect("--pairwise")),
            "--use-deck-best" => a.use_deck_best = Some(PathBuf::from(val())),
            "--seed-emb" => a.seed_emb = Some(PathBuf::from(val())),
            // Names are validated once, by the encoder, when the mask is
            // set in `main` — not here against a second copy of the list.
            // The copy that used to live here went stale the moment the
            // v7 blocks landed and would have rejected `hist`.
            "--ablate" => {
                a.ablate = val().split(',').map(|s| s.trim().to_string()).collect();
            }
            "--holdout" => a.holdout = val().parse().expect("--holdout"),
            "--stop-after-stale" => {
                a.stop_after_stale = val().parse().expect("--stop-after-stale")
            }
            "--relabel-mode" => {
                let mode = val();
                a.relabel_new = match mode.as_str() {
                    "new" => true,
                    "full" => false,
                    other => panic!("--relabel-mode: {other:?} (expected full or new)"),
                };
            }
            "--attn" => {
                a.attn = true;
                continue; // bare flag, consumes no value
            }
            "--aux" => {
                a.aux = true;
                continue; // bare flag, consumes no value
            }
            "--blocks" => a.blocks = val().parse().expect("--blocks"),
            "--muon" => {
                a.muon = true;
                continue; // bare flag, consumes no value
            }
            "--muon-lr" => a.muon_lr = val().parse().expect("--muon-lr"),
            "--lr-cosine" => a.lr_cosine = val().parse().expect("--lr-cosine"),
            "--sample-temp" => a.sample_temp = val().parse().expect("--sample-temp"),
            "--sample-turns" => a.sample_turns = val().parse().expect("--sample-turns"),
            "--mcts-actors" => a.mcts_actors = val().parse().expect("--mcts-actors"),
            "--mcts-fleet" => a.mcts_fleet = val().parse().expect("--mcts-fleet"),
            "--emb-dim" => a.emb_dim = Some(val().parse().expect("--emb-dim")),
            "--obj-hidden" => a.obj_hidden = Some(val().parse().expect("--obj-hidden")),
            "--h1" => a.h1 = Some(val().parse().expect("--h1")),
            "--h2" => a.h2 = Some(val().parse().expect("--h2")),
            "--gate-builder" => a.gate_builder = Some(val().parse().expect("--gate-builder")),
            "--gate-builder-hc" => {
                a.gate_builder_hc = Some(val().parse().expect("--gate-builder-hc"))
            }
            "--distill-gen" => a.distill_gen = Some(val().parse().expect("--distill-gen")),
            "--distill-train" => {
                a.distill_train = true;
                continue;
            }
            "--gate-builder-v2" => {
                a.gate_builder_v2 = Some(val().parse().expect("--gate-builder-v2"))
            }
            "--gate-builder-v3" => {
                a.gate_builder_v3 = Some(val().parse().expect("--gate-builder-v3"))
            }
            other => panic!("unknown flag {other} (see the module doc for usage)"),
        }
    }
    a
}

/// The play-net configuration this invocation asks for — one place, so
/// the training loop and every diagnostic that reloads a checkpoint
/// build the same architecture from the same flags.
fn net_config(args: &Args, vocab: usize) -> crabomination_ml::NetConfig {
    let mut cfg = if args.attn {
        NetConfig::with_attention(vocab)
    } else {
        NetConfig::standard(vocab)
    };
    cfg.aux = args.aux;
    cfg.blocks = args.blocks;
    cfg.policy = args.policy_head;
    cfg.opp = args.opp_head;
    assert!(
        !(cfg.attn && cfg.blocks > 0),
        "--attn and --blocks are mutually exclusive architectures"
    );
    if let Some(v) = args.emb_dim {
        cfg.emb_dim = v;
    }
    if let Some(v) = args.obj_hidden {
        cfg.obj_hidden = v;
    }
    if let Some(v) = args.h1 {
        cfg.h1 = v;
    }
    if let Some(v) = args.h2 {
        cfg.h2 = v;
    }
    if cfg.attn || cfg.blocks > 0 {
        assert!(
            cfg.obj_hidden.is_multiple_of(crabomination_nn::ATTN_HEADS),
            "--obj-hidden {} must be divisible by {} attention heads",
            cfg.obj_hidden,
            crabomination_nn::ATTN_HEADS
        );
    }
    cfg
}

/// Whether a checkpoint carries the policy head. Drives the model shape
/// used to reload other runs' artifacts (the frozen pilot, the batch
/// eval server): candle's `VarMap::load` errors on a var the file lacks
/// and silently ignores file tensors the model lacks, so the shape has
/// to follow the *file*, not this run's flags — a `--policy-head` run
/// piloted by a headless champion-class net is the normal gen-0 case.
fn file_heads(path: &std::path::Path) -> (bool, bool) {
    std::fs::read(path)
        .ok()
        .and_then(|b| crabomination_nn::PlayNet::load(&b).ok())
        .map(|n| (n.has_policy_head(), n.has_opp_head()))
        .unwrap_or((false, false))
}

/// True when this trajectory belongs to the held-out set. A hash of the
/// id rather than a counter, so actors decide independently and the same
/// game always lands on the same side of the split.
fn is_holdout(traj: u32, frac: f64) -> bool {
    if frac <= 0.0 {
        return false;
    }
    let mut h = (traj as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    ((h >> 11) as f64 / (1u64 << 53) as f64) < frac
}

struct Shared {
    window: Mutex<SampleWindow>,
    /// Held-out rows, never trained on. Capped so a long run can't grow
    /// it without bound.
    val: Mutex<Vec<TrainRow>>,
    /// Every game also labels its two decklists — the build net's stream.
    /// Small (2 rows/game), so a plain capped deque suffices.
    deck_window: Mutex<std::collections::VecDeque<DeckRow>>,
    /// Recorded decisions: candidate successor states plus the pick.
    /// Capped deque; policy batches sample it uniformly. Separate from
    /// `window` because a decision is a different shape and a different
    /// cadence from a labelled position.
    decisions: Mutex<std::collections::VecDeque<crabomination_ml::DecisionRow>>,
    /// Held-out decisions, never trained on — the policy top-1 metric.
    decisions_val: Mutex<Vec<crabomination_ml::DecisionRow>>,
    /// Rows ever pushed (not evicted-adjusted) — the reuse cap's basis.
    rows_pushed: AtomicU64,
    /// Next self-play game index to claim — also the per-game seed salt.
    next_game: AtomicU64,
    games_done: AtomicU64,
    /// Games played by the MCTS fleet specifically. Decisions are the
    /// binding budget of a distillation run, so the mix has to be
    /// legible per checkpoint rather than inferred from the deque.
    games_mcts: AtomicU64,
    stalls: AtomicU64,
    /// The two ways a stall happens, so a moving stall rate names its own
    /// cause instead of only its size. `stalls - capped - stuck` is the
    /// remainder: a game that ended by the rules but produced no rows.
    stalls_capped: AtomicU64,
    stalls_stuck: AtomicU64,
    live_actors: AtomicU64,
    /// Set by the learner when `--stop-after-stale` trips: the holdout
    /// has stopped improving, so further generation is compute spent
    /// making `latest` worse. Actors finish their current game and exit.
    stop: AtomicBool,
}

const DECK_WINDOW_CAP: usize = 200_000;

fn actor_loop(shared: &Shared, args: &Args, vocab: &Vocab, deck_judge: Option<&DeckNet>, mcts: bool) {
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            break;
        }
        let n = shared.next_game.fetch_add(1, Ordering::Relaxed);
        if let Some(max) = args.games
            && n >= max
        {
            break;
        }
        let salt = |k: u64| {
            args.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x9E37_79B9)
        };
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        // Best-of-N under the gate-passed deck net when one is loaded,
        // otherwise the heuristic builder's own pick. Same candidate
        // generator either way, so only the judge differs.
        const DECK_CANDS: usize = 32;
        let build = |pool: &[crabomination::cube::CardFactory], seed: u64| match deck_judge {
            Some(net) => best_build_by(pool, DECK_CANDS, seed, |d| {
                let (cards, feats) = encode_deck(d, vocab);
                net.forward(&cards, &feats) as f64
            }),
            None => heuristic_sealed_build(pool, seed),
        };
        let deck_a = build(&pool_a, salt(3));
        let deck_b = build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        // Honest pilots by default, MCTS or not.
        //
        // Until now this branch handed plain `net_eval()` -- determinize
        // 0 -- to every non-MCTS actor, which is the champion regime. So
        // all 250k champion games were generated by a pilot whose sims
        // and planner dry-runs read the opponent's real hand and the true
        // library order, to train a net deployed behind `net_eval_det1()`
        // (see `lobby.rs`), which cannot. That is a train/deploy
        // mismatch, and a worse shape than uniform cheating: the training
        // distribution was composed of positions reached by a policy that
        // cannot exist at inference time.
        //
        // Round 30 gated actor det0 vs det1 across two seeds and found a
        // null, so this is not expected to move strength. It is not
        // measured by that gate either -- a mirror ladder is close to
        // blind here, because both seats cheat identically. `--actor-det`
        // still forces either value as the control arm.
        let mut pilot = if args.use_best.is_some() {
            EvalWeights::net_eval_det1()
        } else {
            EvalWeights::default()
        };
        if let Some(d) = args.actor_det {
            pilot.determinize = d;
        }
        let rec = crabomination::selfplay::play_recorded_game_mcts(
            &template,
            [pilot, pilot],
            salt(5),
            4_000,
            vocab,
            if mcts { args.mcts_actors } else { 0 },
            args.mcts_gumbel,
        );
        if mcts && args.mcts_actors > 0 {
            shared.games_mcts.fetch_add(1, Ordering::Relaxed);
        }
        // Decisions are captured into a thread-local buffer during the
        // game, so each actor drains its own. Split by the same
        // trajectory-hash rule the value rows use — `traj = (seed << 1) |
        // seat`, hashed — so a seat's decisions land on the same side as
        // that seat's positions. (Until round 37 this hashed the *game
        // index* instead, a third key independent of both row hashes, so
        // a game's positions could train while its decisions validated —
        // exactly the leak the comment here claimed was impossible.)
        if args.record_decisions {
            let captured = crabomination::server::decision_capture::drain();
            if !captured.is_empty() {
                const DECISION_CAP: usize = 200_000;
                const DECISION_VAL_CAP: usize = 4_000;
                let traj_base = (salt(5) as u32) << 1;
                let converted: Vec<(bool, crabomination_ml::DecisionRow)> = captured
                    .into_iter()
                    .map(|d| {
                        (
                            is_holdout(traj_base | d.seat as u32, args.holdout),
                            crabomination_ml::DecisionRow {
                                successors: d.successors,
                                chosen: d.chosen,
                                values: d.values,
                                values_are_logits: d.values_are_logits,
                                visits: d.visits,
                            },
                        )
                    })
                    .collect();
                let mut v = shared.decisions_val.lock().unwrap();
                let mut d = shared.decisions.lock().unwrap();
                for (held, row) in converted {
                    if held {
                        if v.len() < DECISION_VAL_CAP {
                            v.push(row);
                        }
                    } else {
                        if d.len() == DECISION_CAP {
                            d.pop_front();
                        }
                        d.push_back(row);
                    }
                }
            }
        }
        shared.games_done.fetch_add(1, Ordering::Relaxed);
        if rec.rows.is_empty() {
            shared.stalls.fetch_add(1, Ordering::Relaxed);
            match rec.stop {
                StopReason::ActionCap => {
                    shared.stalls_capped.fetch_add(1, Ordering::Relaxed);
                }
                StopReason::NoLegalMove => {
                    shared.stalls_stuck.fetch_add(1, Ordering::Relaxed);
                }
                // Ended by the rules with no winner: a draw, not a stall
                // the simulator can be blamed for. Left out of both splits.
                StopReason::GameOver => {}
            }
            continue;
        }
        // A whole game's rows share a trajectory pair, so the split is
        // decided per row by trajectory id and stays consistent.
        let (val_rows, train_rows): (Vec<TrainRow>, Vec<TrainRow>) =
            rec.rows.into_iter().partition(|r| is_holdout(r.traj, args.holdout));
        if !val_rows.is_empty() {
            const VAL_CAP: usize = 20_000;
            let mut v = shared.val.lock().unwrap();
            for row in val_rows {
                if v.len() < VAL_CAP {
                    v.push(row);
                }
            }
        }
        shared.rows_pushed.fetch_add(train_rows.len() as u64, Ordering::Relaxed);
        {
            let mut w = shared.window.lock().unwrap();
            for row in train_rows {
                w.push(row);
            }
        }
        if let Some(winner) = rec.winner {
            let mut dw = shared.deck_window.lock().unwrap();
            for (seat, deck) in [&deck_a, &deck_b].into_iter().enumerate() {
                let (cards, feats) = encode_deck(deck, vocab);
                if dw.len() == DECK_WINDOW_CAP {
                    dw.pop_front();
                }
                dw.push_back(DeckRow {
                    cards,
                    feats,
                    win: if seat == winner { 1.0 } else { 0.0 },
                });
            }
        }
    }
    shared.live_actors.fetch_sub(1, Ordering::Relaxed);
}

/// Wall-clock decomposition of the learner thread between checkpoints,
/// drained into `stats.jsonl` at each one.
///
/// Exists because the imbalance it exposes had to be reverse-engineered:
/// the round-11 runs reached only 1.56× of their 6× reuse cap, and only
/// arithmetic on the flags (264 relabel passes × a 250 k window = 66 M
/// forward-rows vs 13.5 M trained) said the λ-relabel pass was why. Now
/// the split is a column, not a derivation.
#[derive(Default)]
struct LearnerTiming {
    sample: std::time::Duration,
    step: std::time::Duration,
    relabel: std::time::Duration,
    deck: std::time::Duration,
    sleep: std::time::Duration,
}

impl LearnerTiming {
    /// Milliseconds per bucket since the last drain, zeroing the counters:
    /// `[sample, step, relabel, deck, sleep]`.
    fn take_ms(&mut self) -> [u64; 5] {
        let out = [&self.sample, &self.step, &self.relabel, &self.deck, &self.sleep]
            .map(|d| d.as_millis() as u64);
        *self = Self::default();
        out
    }
}

fn sample_owned(shared: &Shared, n: usize, rng: &mut StdRng) -> Vec<(TrainRow, f32)> {
    let w = shared.window.lock().unwrap();
    w.sample_with_targets(n, rng).into_iter().map(|(r, t)| (r.clone(), t)).collect()
}

/// Wilson score interval (z = 1.96), same construction as bot_ladder's.
fn wilson(wins: u32, n: u32) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let z = 1.96f64;
    let (n, p) = (n as f64, wins as f64 / n as f64);
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    ((center - half).max(0.0), (center + half).min(1.0))
}

/// Two-sided 95% Student-t critical value. Exact table through 30
/// degrees of freedom, then the standard coarse steps — the ±2·se habit
/// at small n is exactly the overstatement round 41 had to walk back.
fn t95(df: usize) -> f64 {
    const TABLE: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::INFINITY,
        1..=30 => TABLE[df - 1],
        31..=40 => 2.021,
        41..=60 => 2.000,
        61..=120 => 1.980,
        _ => 1.960,
    }
}

/// Run `per_pool` for pool indices `0..n` across up to `threads` workers,
/// returning results in pool order. Deck construction is seeded per pool
/// and matches the sequential loop exactly; game tallies drift by a game
/// or two per pool because the bots' candidate jitter draws from the
/// thread RNG by design (measured 30.0% vs 29.9% on a 4,800-game gate) —
/// statistically equivalent, not bit-identical.
/// 32 MB worker stacks, same as every other thread that simulates games.
fn parallel_pools<T: Send>(
    n: u64,
    threads: usize,
    per_pool: impl Fn(u64) -> T + Sync,
) -> Vec<T> {
    let next = AtomicU64::new(0);
    let results: Mutex<Vec<(u64, T)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.clamp(1, n as usize) {
            std::thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        if i >= n {
                            break;
                        }
                        let r = per_pool(i);
                        results.lock().unwrap().push((i, r));
                    }
                })
                .expect("spawn gate worker");
        }
    });
    let mut v = results.into_inner().unwrap();
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, r)| r).collect()
}

/// The builder gate: over paired pools, both judges rank the *same*
/// best-of-N candidate set — net win-prob vs the heuristic's static
/// score — and their picks race with identical pilots. Whatever
/// difference shows up is attributable to the judge alone.
fn gate_builder(args: &Args, vocab: &Vocab, games_per_pool: usize) {
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    let path = args.out.join("deck-latest.safetensors");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (train a deck net first)", path.display()));
    let net = DeckNet::load(&bytes).expect("deck net loads");
    assert_eq!(net.vocab_size(), vocab.size(), "deck net vocab != encoder vocab");
    const POOLS: u64 = 12;
    const CANDS: usize = 32;
    println!(
        "builder gate: net-judged vs static-judged best-of-{CANDS}, {games_per_pool} games x {POOLS} pools, seed {}",
        args.seed
    );
    let per_pool = |i: u64| {
        let salt = args.seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xB111D);
        let pool = sealed_pool(salt);
        let net_deck = best_build_by(&pool, CANDS, salt ^ 1, |d| {
            let (cards, feats) = encode_deck(d, vocab);
            net.forward(&cards, &feats) as f64
        });
        let static_deck =
            best_build_by(&pool, CANDS, salt ^ 1, |d| static_deck_score(d) as f64);
        let tally = simulate_match_games_piloted(
            &net_deck,
            &static_deck,
            games_per_pool,
            [Pilot::default(), Pilot::default()],
            4_000,
            Some(salt ^ 2),
        );
        (
            format!(
                "pool #{i}: net {} - {} static ({} n/d)",
                tally.wins_a, tally.wins_b, tally.undecided
            ),
            tally.wins_a,
            tally.wins_b,
            tally.undecided,
        )
    };
    let (mut wins_net, mut wins_static, mut undecided) = (0u32, 0u32, 0u32);
    for (line, a, b, nd) in parallel_pools(POOLS, args.actors, per_pool) {
        println!("{line}");
        wins_net += a;
        wins_static += b;
        undecided += nd;
    }
    let decided = wins_net + wins_static;
    let pct = 100.0 * wins_net as f64 / decided.max(1) as f64;
    let (lo, hi) = wilson(wins_net, decided);
    println!(
        "TOTAL: net {wins_net} - {wins_static} static ({undecided} n/d) = {pct:.1}% [{:.1}%, {:.1}%]",
        lo * 100.0,
        hi * 100.0
    );
    println!(
        "verdict: {}",
        if lo > 0.5 {
            "net-judged builds are stronger — the interval clears 50%"
        } else if hi < 0.5 {
            "static-judged builds are stronger — the interval is below 50%"
        } else {
            "inconclusive — the interval straddles 50%"
        }
    );
}

/// Generate gauntlet-labelled decks for distillation. Eight variants per
/// pool spanning builder-distribution and search-visited space, each
/// labelled by win rate over 240 games against a fixed 20-deck field,
/// parallelised across pools.
fn distill_gen(args: &Args, vocab: &Vocab, n_decks: usize) {
    use crabomination::cube::CardFactory;
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    let path = args.out.join("deck-latest.safetensors");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (need a deck net for the search variants)", path.display()));
    let net = DeckNet::load(&bytes).expect("deck net loads");
    assert_eq!(net.vocab_size(), vocab.size(), "deck net vocab != encoder vocab");
    const FIELD: u64 = 20;
    const GAMES_PER_FIELD_DECK: usize = 12; // 240 games per labelled deck
    let field: Vec<Vec<CardFactory>> = (0..FIELD)
        .map(|i| {
            let seed = 0xF1E1D ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            heuristic_sealed_build(&sealed_pool(seed), seed ^ 1)
        })
        .collect();

    let pools_n = n_decks.div_ceil(8);
    let mut jobs: Vec<(u64, Vec<CardFactory>)> = Vec::new();
    for pi in 0..pools_n as u64 {
        let salt = args.seed ^ pi.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xD157);
        let pool = sealed_pool(salt);
        let judge = |d: &[CardFactory]| {
            let (cards, feats) = encode_deck(d, vocab);
            net.forward(&cards, &feats) as f64
        };
        let pick32 = best_build_by(&pool, 32, salt ^ 1, judge);
        let variants: Vec<Vec<CardFactory>> = vec![
            heuristic_sealed_build(&pool, salt ^ 2),
            heuristic_sealed_build(&pool, salt ^ 3),
            pick32.clone(),
            mutate_build(&pool, pick32.clone(), 3, salt ^ 4),
            mutate_build(&pool, pick32.clone(), 8, salt ^ 5),
            mutate_build(&pool, pick32.clone(), 15, salt ^ 6),
            hill_climb_build_by(&pool, pick32.clone(), 2, judge),
            hill_climb_build_by(&pool, pick32, 6, judge),
        ];
        for (vi, v) in variants.into_iter().enumerate() {
            jobs.push((salt ^ (0x100 + vi as u64), v));
        }
    }
    jobs.truncate(n_decks);
    println!(
        "distill-gen: {} decks x {} games vs {FIELD} field decks, {} threads",
        jobs.len(),
        GAMES_PER_FIELD_DECK as u64 * FIELD,
        args.actors
    );

    let next = AtomicU64::new(0);
    let done = AtomicU64::new(0);
    let results: Mutex<Vec<DeckRow>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..args.actors {
            std::thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    loop {
                        let j = next.fetch_add(1, Ordering::Relaxed) as usize;
                        let Some((seed, deck)) = jobs.get(j) else { break };
                        let (mut wins, mut decided) = (0u32, 0u32);
                        for (fi, fd) in field.iter().enumerate() {
                            let t = simulate_match_games_piloted(
                                deck,
                                fd,
                                GAMES_PER_FIELD_DECK,
                                [Pilot::default(), Pilot::default()],
                                4_000,
                                Some(seed ^ (fi as u64) << 32),
                            );
                            wins += t.wins_a;
                            decided += t.wins_a + t.wins_b;
                        }
                        let (cards, feats) = encode_deck(deck, vocab);
                        let win = wins as f32 / decided.max(1) as f32;
                        results.lock().unwrap().push(DeckRow { cards, feats, win });
                        let d = done.fetch_add(1, Ordering::Relaxed) + 1;
                        if d.is_multiple_of(50) {
                            eprintln!("  {d}/{} labelled", jobs.len());
                        }
                    }
                })
                .expect("spawn labeller");
        }
    });
    let fresh = results.into_inner().unwrap();
    let labels_path = args.out.join("deck_labels.bin");
    let mut all = std::fs::read(&labels_path)
        .ok()
        .and_then(|b| crabomination_nn::read_deck_shard(&b))
        .unwrap_or_default();
    let (lo, hi) = fresh.iter().fold((1.0f32, 0.0f32), |(l, h), r| (l.min(r.win), h.max(r.win)));
    let mean = fresh.iter().map(|r| r.win).sum::<f32>() / fresh.len().max(1) as f32;
    all.extend(fresh);
    std::fs::write(&labels_path, crabomination_nn::write_deck_shard(&all))
        .expect("write deck_labels.bin");
    println!(
        "labelled win rates: mean {mean:.3}, min {lo:.3}, max {hi:.3}; {} total rows in {}",
        all.len(),
        labels_path.display()
    );
}

/// Fit the deck net to the gauntlet labels; holdout is every 10th row.
fn distill_train(args: &Args, vocab: &Vocab) {
    let labels_path = args.out.join("deck_labels.bin");
    let bytes = std::fs::read(&labels_path)
        .unwrap_or_else(|e| panic!("{}: {e} (run --distill-gen first)", labels_path.display()));
    let rows = crabomination_nn::read_deck_shard(&bytes).expect("readable deck_labels.bin");
    let (mut train, mut hold): (Vec<DeckRow>, Vec<DeckRow>) = (Vec::new(), Vec::new());
    for (i, r) in rows.into_iter().enumerate() {
        if i % 10 == 0 { hold.push(r) } else { train.push(r) }
    }
    println!("distill-train: {} train / {} holdout rows", train.len(), hold.len());
    let mut trainer =
        DeckTrainer::new(&DeckNetConfig::standard(vocab.size()), args.lr).expect("deck trainer");
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0xD157);
    // Pair-order accuracy: of random holdout pairs with meaningfully
    // different labels, how often does the net order them the same way?
    let pair_acc = |t: &DeckTrainer| {
        let preds: Vec<f32> =
            hold.iter().map(|r| t.predict(r).unwrap_or(0.5)).collect();
        let (mut ok, mut n) = (0u32, 0u32);
        let mut prng = StdRng::seed_from_u64(7);
        for _ in 0..5_000 {
            let (a, b) =
                (prng.random_range(0..hold.len()), prng.random_range(0..hold.len()));
            if (hold[a].win - hold[b].win).abs() < 0.05 {
                continue;
            }
            n += 1;
            if (preds[a] > preds[b]) == (hold[a].win > hold[b].win) {
                ok += 1;
            }
        }
        (ok, n)
    };
    for step in 1..=4_000u32 {
        let batch: Vec<&DeckRow> =
            (0..64).map(|_| &train[rng.random_range(0..train.len())]).collect();
        let loss = trainer.train_step(&batch).expect("step");
        if step % 1_000 == 0 {
            let (ok, n) = pair_acc(&trainer);
            println!(
                "step {step}: train mse {loss:.4}, holdout pair-order {:.1}% ({n} pairs)",
                100.0 * ok as f64 / n.max(1) as f64
            );
        }
    }
    let out = args.out.join("deck-distilled.safetensors");
    trainer.save(&out).expect("save distilled net");
    println!("saved {}", out.display());
}

/// Race hill-climbed builds against the best-of-32 picks they started
/// from. Judge, pools, and pilots are identical on both sides, so this
/// isolates one thing: whether *searching* the build space under the
/// deck net beats sampling it.
fn gate_builder_hc(args: &Args, vocab: &Vocab, games_per_pool: usize) {
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    let path = args.out.join("deck-latest.safetensors");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (train a deck net first)", path.display()));
    let net = DeckNet::load(&bytes).expect("deck net loads");
    assert_eq!(net.vocab_size(), vocab.size(), "deck net vocab != encoder vocab");
    const POOLS: u64 = 12;
    const CANDS: usize = 32;
    const PASSES: usize = 6;
    println!(
        "builder gate: hill-climbed vs best-of-{CANDS} (same net judge), {games_per_pool} games x {POOLS} pools, seed {}",
        args.seed
    );
    let per_pool = |i: u64| {
        let salt = args.seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xB111DC);
        let pool = sealed_pool(salt);
        let judge = |d: &[crabomination::cube::CardFactory]| {
            let (cards, feats) = encode_deck(d, vocab);
            net.forward(&cards, &feats) as f64
        };
        let pick = best_build_by(&pool, CANDS, salt ^ 1, judge);
        let climbed = hill_climb_build_by(&pool, pick.clone(), PASSES, judge);
        let swaps = climbed
            .iter()
            .zip(&pick)
            .filter(|(a, b)| a() .name != b().name)
            .count();
        let tally = simulate_match_games_piloted(
            &climbed,
            &pick,
            games_per_pool,
            [Pilot::default(), Pilot::default()],
            4_000,
            Some(salt ^ 2),
        );
        (
            format!(
                "pool #{i}: climbed {} - {} pick ({} n/d), {swaps} slots changed, net score {:.4} -> {:.4}",
                tally.wins_a,
                tally.wins_b,
                tally.undecided,
                judge(&pick),
                judge(&climbed),
            ),
            tally.wins_a,
            tally.wins_b,
            tally.undecided,
        )
    };
    let (mut wins_hc, mut wins_pick, mut undecided) = (0u32, 0u32, 0u32);
    for (line, a, b, nd) in parallel_pools(POOLS, args.actors, per_pool) {
        println!("{line}");
        wins_hc += a;
        wins_pick += b;
        undecided += nd;
    }
    let decided = wins_hc + wins_pick;
    let pct = 100.0 * wins_hc as f64 / decided.max(1) as f64;
    let (lo, hi) = wilson(wins_hc, decided);
    println!(
        "TOTAL: climbed {wins_hc} - {wins_pick} pick ({undecided} n/d) = {pct:.1}% [{:.1}%, {:.1}%]",
        lo * 100.0,
        hi * 100.0
    );
    println!(
        "verdict: {}",
        if lo > 0.5 {
            "hill-climbed builds are stronger — the interval clears 50%"
        } else if hi < 0.5 {
            "the plain pick is stronger — the interval is below 50%"
        } else {
            "inconclusive — the interval straddles 50%"
        }
    );
}

/// Race the repaired builder against the one it replaces: same pool,
/// same pilots, same seeds — only the build differs.
fn gate_builder_v2(args: &Args, games_per_pool: usize) {
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    const POOLS: u64 = 12;
    println!(
        "builder gate: builder_v2 vs legacy builder, {games_per_pool} games x {POOLS} pools, seed {}",
        args.seed
    );
    let per_pool = |i: u64| {
        let salt = args.seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xB0111D);
        let pool = sealed_pool(salt);
        let v2 = crabomination::selfplay::heuristic_sealed_build_with(&pool, salt ^ 1, true);
        let old = crabomination::selfplay::heuristic_sealed_build_with(&pool, salt ^ 1, false);
        let tally = simulate_match_games_piloted(
            &v2,
            &old,
            games_per_pool,
            [Pilot::default(), Pilot::default()],
            4_000,
            Some(salt ^ 2),
        );
        (
            format!(
                "pool #{i}: v2 {} - {} legacy ({} n/d)",
                tally.wins_a, tally.wins_b, tally.undecided
            ),
            tally.wins_a,
            tally.wins_b,
            tally.undecided,
        )
    };
    let (mut wins_v2, mut wins_old, mut undecided) = (0u32, 0u32, 0u32);
    for (line, a, b, nd) in parallel_pools(POOLS, args.actors, per_pool) {
        println!("{line}");
        wins_v2 += a;
        wins_old += b;
        undecided += nd;
    }
    let decided = wins_v2 + wins_old;
    let pct = 100.0 * wins_v2 as f64 / decided.max(1) as f64;
    let (lo, hi) = wilson(wins_v2, decided);
    println!(
        "TOTAL: v2 {wins_v2} - {wins_old} legacy ({undecided} n/d) = {pct:.1}% [{:.1}%, {:.1}%]",
        lo * 100.0,
        hi * 100.0
    );
    println!(
        "verdict: {}",
        if lo > 0.5 {
            "builder_v2 is stronger — the interval clears 50%"
        } else if hi < 0.5 {
            "the legacy builder is stronger — the interval is below 50%"
        } else {
            "inconclusive — the interval straddles 50%"
        }
    );
}

/// Two races against the single-sample v2 build (the training field's
/// recipe) on the same pools: the v3 shape ranker as a single sample —
/// the scoring change alone — and best-of-16 v3, the client's sealed
/// opponent recipe. Same pools, same pilots, same seeds; only the build
/// differs.
fn gate_builder_v3(args: &Args, games_per_pool: usize) {
    use crabomination::cube::CardFactory;
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    // Pool count is the axis that buys precision here, not games per
    // pool: each side fields ONE draw from its builder distribution per
    // pool, so pool-level spread (measured sd ~15 points at 800
    // games/pool) dwarfs within-pool sampling noise. Twelve pools at
    // 800 games each gave a ±6.5-point t-interval; the same 9,600 games
    // as 192 pools × 50 give ~±2.3. Default matches the v2 gate;
    // override with CRAB_GATE_POOLS for the wide-flat shape.
    let pools: u64 = std::env::var("CRAB_GATE_POOLS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12);
    type Builder = dyn Fn(&[CardFactory], u64) -> Vec<CardFactory> + Sync;
    let race = |title: &str, name_a: &str, build_a: &Builder| {
        println!(
            "builder gate: {title}, {games_per_pool} games x {pools} pools, seed {}",
            args.seed
        );
        let per_pool = |i: u64| {
            let salt = args.seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xB3111D);
            let pool = sealed_pool(salt);
            let a = build_a(&pool, salt ^ 1);
            let v2 = crabomination::selfplay::heuristic_sealed_build_with(&pool, salt ^ 1, true);
            let tally = simulate_match_games_piloted(
                &a,
                &v2,
                games_per_pool,
                [Pilot::default(), Pilot::default()],
                4_000,
                Some(salt ^ 2),
            );
            (
                format!(
                    "pool #{i}: {name_a} {} - {} v2 ({} n/d)",
                    tally.wins_a, tally.wins_b, tally.undecided
                ),
                tally.wins_a,
                tally.wins_b,
                tally.undecided,
            )
        };
        let (mut wins_a, mut wins_v2, mut undecided) = (0u32, 0u32, 0u32);
        let mut pool_pcts: Vec<f64> = Vec::new();
        for (line, a, b, nd) in parallel_pools(pools, args.actors, per_pool) {
            println!("{line}");
            wins_a += a;
            wins_v2 += b;
            undecided += nd;
            if a + b > 0 {
                pool_pcts.push(100.0 * a as f64 / (a + b) as f64);
            }
        }
        let decided = wins_a + wins_v2;
        let pct = 100.0 * wins_a as f64 / decided.max(1) as f64;
        let (lo, hi) = wilson(wins_a, decided);
        println!(
            "TOTAL: {name_a} {wins_a} - {wins_v2} v2 ({undecided} n/d) = {pct:.1}% [{:.1}%, {:.1}%] (Wilson, games as units)",
            lo * 100.0,
            hi * 100.0
        );
        // The Wilson interval treats games as independent; they are not —
        // each pool is one deck-pair matchup and one DRAW from each
        // builder distribution, and that pool-level spread (sd ~15 pts)
        // is the dominant noise. Pools are the honest unit; the verdict
        // reads this interval, not Wilson.
        let n = pool_pcts.len() as f64;
        let mean = pool_pcts.iter().sum::<f64>() / n.max(1.0);
        let sd = (pool_pcts.iter().map(|p| (p - mean).powi(2)).sum::<f64>()
            / (n - 1.0).max(1.0))
        .sqrt();
        let half = t95(pool_pcts.len().saturating_sub(1)) * sd / n.max(1.0).sqrt();
        println!(
            "POOLS: mean {mean:.2}% over {} pools, sd {sd:.1}, 95% t-interval [{:.2}%, {:.2}%]",
            pool_pcts.len(),
            mean - half,
            mean + half
        );
        println!(
            "verdict: {}",
            if mean - half > 50.0 {
                "the candidate builds are stronger — the pool-level interval clears 50%"
            } else if mean + half < 50.0 {
                "the v2 builds are stronger — the pool-level interval is below 50%"
            } else {
                "inconclusive — the pool-level interval straddles 50%"
            }
        );
    };
    race("builder_v3 vs builder_v2 (single sample each, same seed)", "v3", &|pool, seed| {
        crabomination::selfplay::heuristic_sealed_build_v3(pool, seed)
    });
    race("best-of-16 v3 (client opponent recipe) vs single-sample v2", "b16", &|pool, seed| {
        crabomination::selfplay::best_build_v3(pool, 16, seed)
    });
}

fn main() {
    let args = parse_args();
    // Before anything encodes: the diagnostics below must see the same
    // features the run being diagnosed was trained on.
    if !args.ablate.is_empty() {
        let off: Vec<&str> = args.ablate.iter().map(String::as_str).collect();
        if let Err(e) = crabomination::server::encode::set_encode_ablation_off(&off) {
            eprintln!("--ablate: {e}");
            std::process::exit(2);
        }
        eprintln!("encoder ablation: {} switched off", args.ablate.join(", "));
    }
    // Same rule as the ablation above: every path that encodes — the
    // diagnostics included — must see the cadence the run being
    // diagnosed recorded with.
    crabomination::selfplay::set_combat_snapshots(args.record_combat);
    eprintln!(
        "snapshot cadence: turn/postcombat/end{}",
        if args.record_combat { " + declare-blockers/end-combat" } else { " (no combat rows)" }
    );
    let vocab = Vocab::sos_sealed();
    if let Some(games) = args.gate_builder_v3 {
        gate_builder_v3(&args, games);
        return;
    }
    if let Some(games) = args.gate_builder_v2 {
        gate_builder_v2(&args, games);
        return;
    }
    if let Some(games) = args.gate_builder {
        gate_builder(&args, &vocab, games);
        return;
    }
    if let Some(games) = args.gate_builder_hc {
        gate_builder_hc(&args, &vocab, games);
        return;
    }
    if let Some(n) = args.distill_gen {
        distill_gen(&args, &vocab, n);
        return;
    }
    if args.distill_train {
        distill_train(&args, &vocab);
        return;
    }
    if let Some(games) = args.feature_census {
        feature_census(&args, &vocab, games);
        return;
    }
    if let Some(games) = args.calibrate {
        calibrate(&args, &vocab, games);
        return;
    }
    if let Some(games) = args.calibrate_leaves {
        calibrate_leaves(&args, &vocab, games);
        return;
    }
    if let Some(games) = args.pairwise {
        pairwise(&args, &vocab, games);
        return;
    }
    let cfg = net_config(&args, vocab.size());
    let mut trainer = if args.muon {
        Trainer::new_muon(
            &cfg,
            crabomination_ml::MuonParams {
                lr: args.muon_lr,
                adamw_lr: args.lr,
                ..Default::default()
            },
        )
        .expect("muon trainer init")
    } else {
        Trainer::new(&cfg, args.lr).expect("trainer init")
    };
    trainer.set_bce(args.bce);
    // A frozen copy of the pilot, for the `pilot_policy` reference.
    // lr 0 and never stepped: this exists only to score.
    let pilot_trainer: Option<Trainer> = args.record_decisions.then(|| args.use_best.as_ref()).flatten().and_then(|p| {
        // The pilot's shape follows the pilot's file, not this run's
        // architecture flags — see `file_has_policy_head`.
        let mut cfg = cfg;
        (cfg.policy, cfg.opp) = file_heads(p);
        let mut t = Trainer::new(&cfg, 0.0).ok()?;
        match t.load(p) {
            Ok(()) => {
                eprintln!("pilot baseline scored from {}", p.display());
                Some(t)
            }
            Err(e) => {
                eprintln!("pilot baseline unavailable ({e}); pilot_policy will be nan");
                None
            }
        }
    });
    let mut deck_trainer =
        DeckTrainer::new(&DeckNetConfig::standard(vocab.size()), args.lr).expect("deck trainer");
    eprintln!(
        "learner device: {} (lambda {}, batch {}, {}, {}, win loss {})",
        trainer.device_label(),
        args.lambda,
        args.batch,
        if args.muon {
            format!("muon lr {} / adamw lr {}", args.muon_lr, args.lr)
        } else {
            format!("lr {}", args.lr)
        },
        if args.blocks > 0 {
            format!("{} transformer blocks", args.blocks)
        } else if args.attn {
            "attention".to_string()
        } else {
            "pooled".to_string()
        },
        if args.bce { "bce" } else { "mse" }
    );
    std::fs::create_dir_all(&args.out).expect("create --out dir");
    let latest = args.out.join("latest.safetensors");
    if latest.exists() {
        trainer.load(&latest).expect("resume from latest.safetensors (delete it to start fresh)");
        eprintln!("resumed weights from {}", latest.display());
    } else if let Some(src) = &args.seed_emb {
        // Only on a fresh run: resuming already has embeddings that have
        // been trained on real positions, and overwriting them with the
        // deck net's would throw that away.
        trainer.seed_embeddings_from_deck_net(src).expect("--seed-emb");
        eprintln!("card embeddings seeded from the deck net at {}", src.display());
    }
    let deck_latest = args.out.join("deck-latest.safetensors");
    if deck_latest.exists() {
        deck_trainer.load(&deck_latest).expect("resume deck-latest.safetensors");
        eprintln!("resumed deck weights from {}", deck_latest.display());
    }
    // The gate-passed build net, if the run wants its decks judged.
    let deck_judge: Option<DeckNet> = args.use_deck_best.as_ref().map(|p| {
        let bytes = std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let net = DeckNet::load(&bytes).expect("deck net loads");
        assert_eq!(net.vocab_size(), vocab.size(), "deck net vocab != encoder vocab");
        eprintln!("actors build decks with the net from {}", p.display());
        net
    });
    if let Some(best) = &args.use_best {
        if args.gpu_eval {
            let mut cfg = net_config(&args, vocab.size());
            (cfg.policy, cfg.opp) = file_heads(best);
            let client = crabomination_ml::BatchEvalServer::start(
                &cfg,
                best,
                args.eval_batch,
                std::time::Duration::from_micros(args.eval_flush_us),
            )
            .expect("--gpu-eval server start");
            crabomination::server::net_eval::set_slot(
                crabomination::server::net_eval::SLOT_BEST,
                Some(client),
            );
            eprintln!(
                "actors play with the net from {} via batched eval (batch {}, flush {}us)",
                best.display(),
                args.eval_batch,
                args.eval_flush_us
            );
        } else {
            crabomination::server::net_eval::load_slot(
                crabomination::server::net_eval::SLOT_BEST,
                best,
            )
            .expect("--use-best weights load");
            eprintln!("actors play with the net from {}", best.display());
        }
    }

    if args.record_decisions {
        crabomination::server::decision_capture::set_enabled(true);
        if args.policy_every == 0 {
            eprintln!("recording decisions, MEASURING ONLY (no policy training)");
        } else {
            eprintln!(
                "recording decisions (policy step every {} value steps, into {})",
                args.policy_every,
                if args.policy_head { "the policy head" } else { "the win head" }
            );
        }
    }
    if args.mcts_gumbel {
        assert!(
            args.mcts_actors > 0,
            "--mcts-gumbel shapes the MCTS actors' search; it needs --mcts-actors > 0"
        );
        // Which prior the Gumbel actors will actually run with, said
        // once at startup rather than discovered from a null result: a
        // headless pilot silently falls back to heuristic-score priors,
        // which is a legitimate control arm but a different experiment.
        let learned = crabomination::server::net_eval::slot_has_policy(
            crabomination::server::net_eval::SLOT_BEST,
        );
        eprintln!(
            "gumbel actors: {}",
            if learned {
                "policy-head priors"
            } else {
                "heuristic-score priors (pilot carries no policy head)"
            }
        );
    }

    let shared = Shared {
        window: Mutex::new(SampleWindow::new(args.window)),
        val: Mutex::new(Vec::new()),
        deck_window: Mutex::new(std::collections::VecDeque::new()),
        decisions: Mutex::new(std::collections::VecDeque::new()),
        decisions_val: Mutex::new(Vec::new()),
        rows_pushed: AtomicU64::new(0),
        next_game: AtomicU64::new(0),
        games_done: AtomicU64::new(0),
        games_mcts: AtomicU64::new(0),
        stalls: AtomicU64::new(0),
        stalls_capped: AtomicU64::new(0),
        stalls_stuck: AtomicU64::new(0),
        live_actors: AtomicU64::new(args.actors as u64),
        stop: AtomicBool::new(false),
    };
    let mcts_all = args.use_best.is_some() && args.mcts_actors > 0;
    let fleet_split = mcts_all && args.mcts_fleet > 0 && args.mcts_fleet < args.actors;
    if fleet_split {
        eprintln!(
            "mixed fleet: {} mcts actors + {} value actors (decisions recorded from the mcts fleet only)",
            args.mcts_fleet,
            args.actors - args.mcts_fleet
        );
    }
    eprintln!(
        "selfplay_train: {} actors, vocab {}, window {}, batch {}, reuse cap {}x",
        args.actors,
        vocab.size(),
        args.window,
        args.batch,
        args.reuse
    );

    let start = Instant::now();
    // The per-thread fleet flag is a Copy the closure must own (a loop
    // local can't outlive the scope), so the shared state is re-borrowed
    // here for the `move` closures to capture by reference.
    let shared_r = &shared;
    let args_r = &args;
    let vocab_r = &vocab;
    let judge_r = deck_judge.as_ref();
    std::thread::scope(|scope| {
        for i in 0..args.actors {
            let mcts_thread = mcts_all && (!fleet_split || i < args.mcts_fleet);
            // SOS games overflow the default stack (see bot_probe) — match
            // the ladder's 32 MB workers.
            std::thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, move || {
                    // Data-generation exploration only: the sampling
                    // config is thread-local, so gates and calibration
                    // (different threads/processes) stay argmax.
                    if args_r.sample_temp > 0 {
                        crabomination::server::bot::set_action_sampling(Some((
                            args_r.sample_temp,
                            args_r.sample_turns,
                        )));
                    }
                    // In a mixed fleet only the search pilots feed the
                    // decision stream: a value actor's picks are one-hot
                    // imitation targets that would bury the distillation
                    // targets at ~50-100x the games rate. Thread-local,
                    // like the sampling config above, so nothing outside
                    // this thread is affected.
                    if fleet_split {
                        crabomination::server::decision_capture::set_thread_enabled(mcts_thread);
                    }
                    actor_loop(shared_r, args_r, vocab_r, judge_r, mcts_thread)
                })
                .expect("spawn actor");
        }

        // Learner (this thread).
        let mut rng = StdRng::seed_from_u64(args.seed ^ 0x1EA4);
        let mut consumed = 0u64;
        let mut step = 0u64;
        // EMAs of [total, win, life, len] — decomposed so a regime change
        // in one head (or in effective sample reuse) is visible.
        let mut loss_ema = [f32::NAN; 5];
        let mut deck_loss_ema = f32::NAN;
        // Best held-out AUC seen so far; drives `best.safetensors`.
        let mut best_auc = f32::NEG_INFINITY;
        // Scored checkpoints since the last new best — `--stop-after-stale`.
        let mut stale = 0u64;
        let mut timing = LearnerTiming::default();
        // (samples consumed when the actors finished, tail allowance).
        let mut policy_ema = f32::NAN;
        let mut policy_loss_ema = f32::NAN;
        let mut policy_sv_ema = f32::NAN;
        let mut tail_budget = None::<(u64, u64)>;
        let stats_path = args.out.join("stats.jsonl");
        let mut prev_interval = Interval::default();
        loop {
            if let Some(max) = args.steps
                && step >= max
            {
                break;
            }
            let pushed = shared.rows_pushed.load(Ordering::Relaxed);
            let actors_live = shared.live_actors.load(Ordering::Relaxed) > 0;
            // Once generation ends, the global reuse budget stops meaning
            // "≤ reuse visits per row": every further sample lands on the
            // final window, concentrating budget/window_len visits there
            // (measured ~14× on the first release run, where generation
            // outpaced the single-threaded learner 5:1). Grant the tail
            // half the nominal reuse on the window it has — the rows
            // already absorbed roughly the other half while streaming —
            // and stop.
            if !actors_live && tail_budget.is_none() {
                let wlen = shared.window.lock().unwrap().len() as u64;
                tail_budget = Some((consumed, (args.tail_reuse * wlen as f64) as u64));
            }
            if let Some((at, budget)) = tail_budget
                && consumed - at + args.batch as u64 > budget
            {
                break;
            }
            let budget = (args.reuse * pushed as f64) as u64;
            if pushed < args.min_window || consumed + args.batch as u64 > budget {
                if !actors_live {
                    break; // no more data coming and the reuse budget is spent
                }
                let t0 = Instant::now();
                std::thread::sleep(std::time::Duration::from_millis(200));
                timing.sleep += t0.elapsed();
                continue;
            }
            // Refresh the λ-returns periodically: they are computed
            // through the net, so they go stale as it learns. Every
            // `relabel_every` steps is a compromise between staleness and
            // the cost of a forward pass over the whole window — with
            // λ = 1 the targets never change, so this is skipped entirely.
            if args.lambda < 1.0
                && step.is_multiple_of(args.relabel_every)
                && shared.window.lock().unwrap().len() as u64 >= args.min_window
            {
                let t0 = Instant::now();
                let mut w = shared.window.lock().unwrap();
                let value =
                    |rows: &[&TrainRow]| trainer.predict_win_batch(rows, 512).unwrap_or_default();
                if args.relabel_new {
                    w.relabel_lambda_new_rows(args.lambda, value);
                } else {
                    w.relabel_lambda(args.lambda, value);
                }
                drop(w);
                timing.relabel += t0.elapsed();
            }
            let t0 = Instant::now();
            let rows = sample_owned(&shared, args.batch, &mut rng);
            timing.sample += t0.elapsed();
            let refs: Vec<(&TrainRow, f32)> = rows.iter().map(|(r, t)| (r, *t)).collect();
            if args.lr_cosine > 0 {
                trainer.set_lr_factor(crabomination_ml::cosine_lr_factor(step, args.lr_cosine));
            }
            let t0 = Instant::now();
            let loss = trainer.train_step_with_targets(&refs).expect("train step");
            timing.step += t0.elapsed();
            consumed += args.batch as u64;
            step += 1;
            for (ema, part) in loss_ema
                .iter_mut()
                .zip([loss.total, loss.win, loss.life, loss.len, loss.opp])
            {
                *ema = if ema.is_nan() { part } else { 0.99 * *ema + 0.01 * part };
            }

            // Policy steps ride alongside the value steps: the same net,
            // taught to rank a decision's candidate successors the way
            // the pilot ranked them. Sampled from a separate window
            // because a decision is a different shape and cadence from a
            // labelled position.
            if args.record_decisions
                && args.policy_every > 0
                && step.is_multiple_of(args.policy_every)
            {
                let sample: Vec<crabomination_ml::DecisionRow> = {
                    let d = shared.decisions.lock().unwrap();
                    if d.len() < 256 {
                        Vec::new()
                    } else {
                        (0..args.batch.min(64))
                            .map(|_| d[rng.random_range(0..d.len())].clone())
                            .collect()
                    }
                };
                if !sample.is_empty() {
                    let refs: Vec<&crabomination_ml::DecisionRow> = sample.iter().collect();
                    if let Ok(st) = trainer.train_policy_step_full(
                        &refs,
                        args.policy_temp,
                        args.search_value_weight,
                    ) {
                        policy_ema = if policy_ema.is_nan() {
                            st.top1
                        } else {
                            0.99 * policy_ema + 0.01 * st.top1
                        };
                        policy_loss_ema = if policy_loss_ema.is_nan() {
                            st.loss
                        } else {
                            0.99 * policy_loss_ema + 0.01 * st.loss
                        };
                        policy_sv_ema = if policy_sv_ema.is_nan() {
                            st.sv
                        } else {
                            0.99 * policy_sv_ema + 0.01 * st.sv
                        };
                    }
                }
            }

            // The deck net rides along at a quarter cadence — its stream
            // is 2 rows/game, so training it every step would just churn
            // the same rows.
            if step.is_multiple_of(4) {
                let t0 = Instant::now();
                let rows: Vec<DeckRow> = {
                    let dw = shared.deck_window.lock().unwrap();
                    if dw.len() < 4_000 {
                        Vec::new()
                    } else {
                        (0..args.batch)
                            .map(|_| dw[rng.random_range(0..dw.len())].clone())
                            .collect()
                    }
                };
                if !rows.is_empty() {
                    let refs: Vec<&DeckRow> = rows.iter().collect();
                    let dl = deck_trainer.train_step(&refs).expect("deck step");
                    deck_loss_ema =
                        if deck_loss_ema.is_nan() { dl } else { 0.99 * deck_loss_ema + 0.01 * dl };
                }
                timing.deck += t0.elapsed();
            }

            if step.is_multiple_of(args.checkpoint_every) {
                let improved = checkpoint(
                    &trainer,
                    &deck_trainer,
                    deck_loss_ema,
                    &args,
                    &shared,
                    step,
                    consumed,
                    loss_ema,
                    start,
                    &stats_path,
                    &mut best_auc,
                    [policy_ema, policy_loss_ema, policy_sv_ema],
                    pilot_trainer.as_ref(),
                    &mut timing,
                    &mut prev_interval,
                );
                match improved {
                    Some(true) => stale = 0,
                    Some(false) => stale += 1,
                    None => {} // no holdout scored: says nothing either way
                }
                if args.stop_after_stale > 0 && stale >= args.stop_after_stale {
                    eprintln!(
                        "early stop at step {step}: no holdout-AUC improvement in {stale} \
                         checkpoints (best {best_auc:.4} is published as best.safetensors)"
                    );
                    shared.stop.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
        if step > 0 && !step.is_multiple_of(args.checkpoint_every) {
            checkpoint(
                &trainer,
                &deck_trainer,
                deck_loss_ema,
                &args,
                &shared,
                step,
                consumed,
                loss_ema,
                start,
                &stats_path,
                &mut best_auc,
                [policy_ema, policy_loss_ema, policy_sv_ema],
                pilot_trainer.as_ref(),
                &mut timing,
                &mut prev_interval,
            );
        }
        eprintln!(
            "learner done: {} steps, {} rows consumed; waiting for actors...",
            step, consumed
        );
        // Scope exit joins the actors; with --games unset they run forever
        // and this process is ended by the operator (checkpoints are
        // already on disk).
    });

    let secs = start.elapsed().as_secs_f64();
    let games = shared.games_done.load(Ordering::Relaxed);
    eprintln!(
        "done: {games} games ({:.1}/s), {} rows, {} stalls, {:.0}s",
        games as f64 / secs.max(0.001),
        shared.rows_pushed.load(Ordering::Relaxed),
        shared.stalls.load(Ordering::Relaxed),
        secs
    );
}

/// Counters as of the previous checkpoint, so every stats line can report
/// the rate over *that interval* and not just the run average. A run whose
/// throughput decays — window growth, allocator churn, actors starving on
/// the learner's lock — shows it here and nowhere else: the cumulative
/// rate is dominated by however the run started.
#[derive(Default)]
struct Interval {
    secs: f64,
    games: u64,
    rows: u64,
    consumed: u64,
    step: u64,
}
/// Returns whether this checkpoint improved the holdout AUC — `None` when
/// no holdout was scored, so `--stop-after-stale` counts only checkpoints
/// that actually said something.
#[allow(clippy::too_many_arguments)]
fn checkpoint(
    trainer: &Trainer,
    deck_trainer: &DeckTrainer,
    deck_loss: f32,
    args: &Args,
    shared: &Shared,
    step: u64,
    consumed: u64,
    loss_ema: [f32; 5],
    start: Instant,
    stats_path: &std::path::Path,
    best_auc: &mut f32,
    policy_ema: [f32; 3],
    pilot: Option<&Trainer>,
    timing: &mut LearnerTiming,
    prev: &mut Interval,
) -> Option<bool> {
    let tmp = args.out.join("latest.safetensors.tmp");
    trainer.save(&tmp).expect("save checkpoint");
    std::fs::rename(&tmp, args.out.join("latest.safetensors")).expect("publish checkpoint");
    let dtmp = args.out.join("deck-latest.safetensors.tmp");
    deck_trainer.save(&dtmp).expect("save deck checkpoint");
    std::fs::rename(&dtmp, args.out.join("deck-latest.safetensors")).expect("publish deck");
    let games = shared.games_done.load(Ordering::Relaxed);
    let games_mcts = shared.games_mcts.load(Ordering::Relaxed);
    let rows = shared.rows_pushed.load(Ordering::Relaxed);
    let stalls = shared.stalls.load(Ordering::Relaxed);
    let stalls_capped = shared.stalls_capped.load(Ordering::Relaxed);
    let stalls_stuck = shared.stalls_stuck.load(Ordering::Relaxed);
    let secs = start.elapsed().as_secs_f64();
    let [total, win, life, len, opp] = loss_ema;
    // Held-out scoring, against BOTH labels a row has.
    //
    // `loss_win` is the training EMA against whatever target the learner
    // fit — the λ-return at λ < 1, the raw result at λ = 1. This comment
    // used to claim `val_win` was "directly comparable to loss_win (same
    // MSE, different rows)", which is true only at λ = 1; every
    // champion-class run since round 27 is λ = 0.7, where the two numbers
    // measure different things and their difference is not the overfit.
    // So each side is now scored against both labels:
    //
    // * `*_raw` — MSE against the actual 0/1 game result. `val_raw` vs
    //   `train_raw` is the honest generalisation gap.
    // * `*_tgt` — MSE against the λ-return. `train_tgt` vs `train_raw` is
    //   the bootstrap divergence: how much better the net fits its own
    //   propagated opinion than it fits reality. At λ = 1 the two
    //   coincide by construction. A run where `train_tgt` collapses
    //   toward zero while `val_raw` and `val_auc` stall is fitting its
    //   own bootstrap, which is the failure mode a λ schedule risks.
    //
    // `val_auc` remains the one that says whether the net knows anything —
    // MSE can improve while ranking does not.
    let (val_n, val_win, val_tgt, val_ll, val_auc) = {
        let v = shared.val.lock().unwrap();
        if v.len() < 200 {
            (0usize, f32::NAN, f32::NAN, f32::NAN, f32::NAN)
        } else {
            let refs: Vec<&TrainRow> = v.iter().collect();
            match trainer.predict_win_batch(&refs, 512) {
                Ok(p) => {
                    let mse = p
                        .iter()
                        .zip(&refs)
                        .map(|(q, r)| (q - r.win) * (q - r.win))
                        .sum::<f32>()
                        / p.len() as f32;
                    // The holdout is never relabelled (it is never trained
                    // on), so its λ-targets are computed here from the same
                    // forward pass the MSE above used — no extra cost.
                    let tgt = crabomination_ml::lambda_targets(&refs, &p, args.lambda);
                    let mse_tgt = p
                        .iter()
                        .zip(&tgt)
                        .map(|(q, t)| (q - t) * (q - t))
                        .sum::<f32>()
                        / p.len() as f32;
                    let pairs: Vec<(f32, f32)> =
                        p.iter().zip(&refs).map(|(q, r)| (*q, r.win)).collect();
                    (
                        v.len(),
                        mse,
                        mse_tgt,
                        log_loss(pairs.iter().copied()),
                        auc(pairs.iter().copied()),
                    )
                }
                Err(_) => (0, f32::NAN, f32::NAN, f32::NAN, f32::NAN),
            }
        }
    };
    // The training-side mirror: a fresh sample of the window scored against
    // its stored λ-targets and against the raw result. Seeded from `step`
    // so a checkpoint's diagnostic is reproducible, and taken under a
    // short lock (owned clones) so the forward pass never holds the window.
    const DIAG_ROWS: usize = 4_096;
    let (train_n, train_tgt, train_raw) = {
        let sample = {
            let w = shared.window.lock().unwrap();
            if w.len() < 200 {
                Vec::new()
            } else {
                let mut drng = StdRng::seed_from_u64(step ^ 0xD1A6_0000);
                w.sample_pairs_owned(DIAG_ROWS.min(w.len()), &mut drng)
            }
        };
        if sample.is_empty() {
            (0usize, f32::NAN, f32::NAN)
        } else {
            let refs: Vec<&TrainRow> = sample.iter().map(|(r, _)| r).collect();
            match trainer.predict_win_batch(&refs, 512) {
                Ok(p) => {
                    let n = p.len() as f32;
                    let tgt = p
                        .iter()
                        .zip(&sample)
                        .map(|(q, (_, t))| (q - t) * (q - t))
                        .sum::<f32>()
                        / n;
                    let raw = p
                        .iter()
                        .zip(&refs)
                        .map(|(q, r)| (q - r.win) * (q - r.win))
                        .sum::<f32>()
                        / n;
                    (p.len(), tgt, raw)
                }
                Err(_) => (0, f32::NAN, f32::NAN),
            }
        }
    };
    // Interval rates: what the simulator and learner did since the last
    // checkpoint. `*_cum` stay alongside them so a decaying run reads as a
    // gap between the two rather than as a single ambiguous number.
    let dt = (secs - prev.secs).max(1e-9);
    let dg = games.saturating_sub(prev.games) as f64 / dt;
    let dr = rows.saturating_sub(prev.rows) as f64 / dt;
    let dc = consumed.saturating_sub(prev.consumed) as f64 / dt;
    let ds = step.saturating_sub(prev.step) as f64 / dt;
    *prev = Interval { secs, games, rows, consumed, step };
    let cum_g = games as f64 / secs.max(1e-9);
    let cum_r = rows as f64 / secs.max(1e-9);
    // Held-out policy agreement: how often the net's top-scoring
    // candidate is the one the pilot actually played, on decisions it was
    // never trained on. This is the first metric in the program that
    // measures *choosing* rather than predicting -- MSE and AUC both
    // score isolated positions, and a search consumes rankings.
    // The pilot's own agreement with the search, measured once and
    // cached. Round 34 could not separate "the search disagrees with its
    // evaluator" from "these are two different nets": MCTS rolls out on
    // the *pilot* net while `val_policy` scores the *training* net. This
    // is the missing reference — the same holdout decisions scored by the
    // very net the search used, so any gap between it and 1.0 is
    // search-vs-its-own-evaluator with no net-vs-net confound.
    let (val_policy, val_policy_chance, val_policy_n) = {
        let v = shared.decisions_val.lock().unwrap();
        if v.len() < 100 {
            (f32::NAN, f32::NAN, 0usize)
        } else {
            let refs: Vec<&crabomination_ml::DecisionRow> = v.iter().collect();
            (
                trainer.policy_top1(&refs).unwrap_or(f32::NAN),
                crabomination_ml::Trainer::policy_chance(&refs),
                refs.len(),
            )
        }
    };
    let [policy_top1, policy_loss, policy_sv] = policy_ema;
    // The pilot's own agreement with the search, on the same holdout.
    // Round 34 could not separate "the search disagrees with its
    // evaluator" from "these are two different nets": MCTS rolls out on
    // the *pilot* net while `val_policy` scores the *training* net. This
    // is the missing reference — scored by the very net the search used,
    // so its distance from 1.0 is search-vs-its-own-evaluator with no
    // net-vs-net confound.
    let pilot_policy = match pilot {
        Some(t) if val_policy_n > 0 => {
            let v = shared.decisions_val.lock().unwrap();
            let refs: Vec<&crabomination_ml::DecisionRow> = v.iter().collect();
            t.policy_top1(&refs).unwrap_or(f32::NAN)
        }
        _ => f32::NAN,
    };
    let [t_sample, t_step, t_relabel, t_deck, t_sleep] = timing.take_ms();
    let line = format!(
        "{{\"step\":{step},\"loss_ema\":{total:.5},\"loss_win\":{win:.5},\"loss_life\":{life:.5},\"loss_len\":{len:.5},\"loss_opp\":{opp:.5},\"loss_deck\":{deck_loss:.5},\"val_n\":{val_n},\"val_win\":{val_win:.5},\"val_tgt\":{val_tgt:.5},\"val_logloss\":{val_ll:.5},\"val_auc\":{val_auc:.5},\"policy_top1\":{policy_top1:.4},\"policy_loss\":{policy_loss:.5},\"policy_sv\":{policy_sv:.5},\"val_policy\":{val_policy:.4},\"val_policy_chance\":{val_policy_chance:.4},\"pilot_policy\":{pilot_policy:.4},\"val_policy_n\":{val_policy_n},\"train_n\":{train_n},\"train_raw\":{train_raw:.5},\"train_tgt\":{train_tgt:.5},\"rows_consumed\":{consumed},\"rows\":{rows},\"games\":{games},\"games_mcts\":{games_mcts},\"stalls\":{stalls},\"stalls_capped\":{stalls_capped},\"stalls_stuck\":{stalls_stuck},\"elapsed_s\":{secs:.0},\"games_per_s\":{dg:.3},\"rows_per_s\":{dr:.1},\"consumed_per_s\":{dc:.1},\"steps_per_s\":{ds:.3},\"games_per_s_cum\":{cum_g:.3},\"rows_per_s_cum\":{cum_r:.1},\"t_sample_ms\":{t_sample},\"t_step_ms\":{t_step},\"t_relabel_ms\":{t_relabel},\"t_deck_ms\":{t_deck},\"t_sleep_ms\":{t_sleep}}}\n"
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stats_path)
        .expect("stats.jsonl");
    f.write_all(line.as_bytes()).expect("stats write");
    // Keep the best net by held-out ranking, not just the last one.
    //
    // `latest.safetensors` is whatever the run happened to end on, and a
    // run that overfits ends on its *worst* net: in the round-11 pair the
    // holdout AUC peaked around step 4–6 k and then fell ~0.07 over the
    // next 45 k steps while the training loss went to 0.001. Every gate
    // and every calibration before this was therefore scored on a
    // memorised checkpoint. Publishing `best.safetensors` costs one file
    // and makes those comparisons mean what they claim to.
    //
    // Which holdout number "best" means is the run's stated objective:
    // AUC by default, `val_policy` under `--best-metric policy`. Round
    // 34 measured the two moving in opposite directions on one seed, so
    // a distillation run selected on AUC can publish its worst *ranking*
    // checkpoint while reporting success.
    let (scored, metric) = if args.best_policy {
        (val_policy_n > 0 && val_policy.is_finite(), val_policy)
    } else {
        (val_n > 0 && val_auc.is_finite(), val_auc)
    };
    let mut improved = false;
    if scored && metric > *best_auc {
        *best_auc = metric;
        let btmp = args.out.join("best.safetensors.tmp");
        trainer.save(&btmp).expect("save best checkpoint");
        std::fs::rename(&btmp, args.out.join("best.safetensors")).expect("publish best");
        improved = true;
    }
    let val_note = if val_n > 0 {
        let best_note = if improved { " *best*" } else { "" };
        format!(" | val win {val_win:.4} auc {val_auc:.4} (n={val_n}){best_note}")
    } else {
        String::new()
    };
    // Learner wall-clock split since the last checkpoint. "train" is
    // sample + step + deck: the time spent actually learning, against the
    // relabel overhead and the reuse-throttle idle.
    let busy = (t_sample + t_step + t_relabel + t_deck + t_sleep).max(1);
    eprintln!(
        "step {step}: loss {total:.4} (win {win:.4}){val_note}, {games} games, {rows} rows \
         ({dg:.1} games/s, {dr:.0} rows/s this interval; {cum_g:.1} games/s cum) [train {}% relabel {}% sleep {}%]",
        100 * (t_sample + t_step + t_deck) / busy,
        100 * t_relabel / busy,
        100 * t_sleep / busy
    );
    if scored { Some(improved) } else { None }
}

// ────────────────────────────── calibration ──────────────────────────────

/// Score the value net and the heuristic evaluation as *predictors of the
/// winner* on identical self-play positions.
///
/// This exists because four gate rounds answered "is the net-piloted bot
/// stronger" (42–45 % as a replacement, ~49 % blended) without ever
/// answering "does the net know more than `eval_material` does". Those are
/// different questions with different fixes:
///
/// * If the net's log-loss is no better than the heuristic's, the net has
///   not learned and no amount of integration work will help.
/// * If it *is* better and the bot still loses, the loss is in the
///   integration — and the output histogram is the first place to look. A
///   sigmoid that saturates near 0 and 1 hands the search a flat
///   landscape in which every candidate line scores the same, which would
///   make a better predictor into a worse player.
///
/// The heuristic is put on the same footing by fitting a one-parameter
/// logistic to its score (`p = sigmoid(score / t)`, `t` chosen by scan),
/// so it is scored as the best probability forecast that evaluation can
/// support rather than penalised for not being calibrated.
/// Per-feature occupancy over a set of encoded states.
struct Occupancy {
    obj_hits: Vec<u64>,
    obj_sum: Vec<f64>,
    gl_hits: Vec<u64>,
    gl_sum: Vec<f64>,
    objects: u64,
    positions: u64,
}

impl Occupancy {
    fn new() -> Self {
        use crabomination_nn::{GLOBAL_FEATS, OBJ_FEATS};
        Self {
            obj_hits: vec![0; OBJ_FEATS],
            obj_sum: vec![0.0; OBJ_FEATS],
            gl_hits: vec![0; GLOBAL_FEATS],
            gl_sum: vec![0.0; GLOBAL_FEATS],
            objects: 0,
            positions: 0,
        }
    }

    fn add(&mut self, s: &crabomination_nn::EncodedState) {
        self.positions += 1;
        for (i, v) in s.global.iter().enumerate() {
            if *v != 0.0 {
                self.gl_hits[i] += 1;
                self.gl_sum[i] += v.abs() as f64;
            }
        }
        for group in &s.groups {
            for o in group {
                self.objects += 1;
                for (i, v) in o.feats.iter().enumerate() {
                    if *v != 0.0 {
                        self.obj_hits[i] += 1;
                        self.obj_sum[i] += v.abs() as f64;
                    }
                }
            }
        }
    }

    /// Share of positions where global `i` is non-zero, as a percentage.
    fn gl_rate(&self, i: usize) -> f64 {
        match self.positions {
            0 => 0.0,
            n => 100.0 * self.gl_hits[i] as f64 / n as f64,
        }
    }

    /// Share of encoded objects where feature `i` is non-zero.
    fn obj_rate(&self, i: usize) -> f64 {
        match self.objects {
            0 => 0.0,
            n => 100.0 * self.obj_hits[i] as f64 / n as f64,
        }
    }

    fn gl_mean(&self, i: usize) -> f64 {
        match self.gl_hits[i] {
            0 => 0.0,
            n => self.gl_sum[i] / n as f64,
        }
    }

    fn obj_mean(&self, i: usize) -> f64 {
        match self.obj_hits[i] {
            0 => 0.0,
            n => self.obj_sum[i] / n as f64,
        }
    }
}

/// Which block each feature slot belongs to, so the report reads as a
/// verdict on blocks rather than a wall of indices.
fn obj_block(i: usize) -> &'static str {
    match i {
        0..=11 => "base",
        12..=19 => "keywords",
        20..=27 => "cast",
        28..=36 => "rel",
        37..=39 => "combat",
        40..=44 => "kw",
        45..=47 => "exp",
        _ => "ctr",
    }
}

fn gl_block(i: usize) -> &'static str {
    match i {
        0..=23 => "base",
        24..=35 => "cast",
        36..=40 => "combat",
        41..=42 => "kw",
        _ => "hist",
    }
}

/// How often each encoder feature is actually non-zero — in the rows the
/// trainer fits, and in the leaves the search evaluates.
///
/// The question a new feature block has to answer before it earns a GPU
/// run: a feature that is zero in 99.9 % of positions cannot move a gate
/// no matter how well-motivated it is. Round 28f gated a block that was
/// identically zero in every training row, and round 12's additions were
/// never occupancy-checked either. Cheap — no net, no GPU, one self-play
/// pass — and it reads the encoder as it is currently ablated, so it can
/// also confirm that a control arm really is blanking what it claims to.
///
/// The two columns are the point. **train** is the recorder's
/// distribution, what the weights are fit to. **leaf** is the
/// distribution of simulated positions the attack and cast searches
/// actually evaluate (via `server::leaf_capture`, the same hooks
/// `--calibrate-leaves` uses). A feature that is common in the leaf
/// column and rare in the train column is being fed live values at
/// inference by weights that never received a gradient for it — that is
/// exactly the state round 40 found the whole combat block in, and the
/// `!` marker flags it.
///
/// Round 40's other lesson applies to reading this: matching the two
/// columns is *not* automatically good. `--record-combat` closed the
/// combat gap and cost 3 points of search strength, because what the
/// value head is fit to also determines its calibration. This measures
/// the mismatch; it does not prescribe closing it.
///
/// Object rates are per encoded object over every group; global rates
/// are per position. Both are the denominator the trainer sees, not the
/// denominator of "positions where the feature could apply" — a rate of
/// 2 % on a battlefield-only feature means 2 % of *all* objects, hand
/// and library included.
fn feature_census(args: &Args, vocab: &Vocab, games: usize) {
    use crabomination_nn::{GLOBAL_FEATS, OBJ_FEATS};

    let mut train = Occupancy::new();
    let mut leaf = Occupancy::new();
    // Single-threaded and drained per game, which is the only way
    // `leaf_capture`'s thread-local buffer is safe to read.
    crabomination::server::leaf_capture::set_enabled(true);
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0xFEA7);
    for n in 0..games as u64 {
        let salt =
            |k: u64| args.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0xCA1);
        let deck_a = heuristic_sealed_build(&sealed_pool(salt(1)), salt(3));
        let deck_b = heuristic_sealed_build(&sealed_pool(salt(2)), salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            50_000,
            vocab,
        );
        for (state, _, _, _) in crabomination::server::leaf_capture::drain() {
            leaf.add(&state);
        }
        for row in &rec.rows {
            train.add(&row.state);
        }
    }
    crabomination::server::leaf_capture::set_enabled(false);
    if train.positions == 0 {
        eprintln!("feature census: no positions recorded");
        return;
    }
    if leaf.positions == 0 {
        eprintln!("feature census: no leaves captured — the leaf column will read all zero");
    }

    // A feature the search meets far more often than the trainer does.
    // Absolute floor as well as a ratio, so a jump from 0.001 % to
    // 0.02 % doesn't get flagged as a crisis.
    let gap = |train_pct: f64, leaf_pct: f64| leaf_pct >= 1.0 && leaf_pct > 4.0 * train_pct.max(0.01);

    println!(
        "feature census: train {} positions / {} objects; leaves {} positions / {} objects; \
         from {games} games",
        train.positions, train.objects, leaf.positions, leaf.objects
    );
    println!("  train = the recorder's rows (what the weights are fit to)");
    println!("  leaf  = the simulated positions the search evaluates");
    println!("  !     = live at inference, rare or absent in training");

    println!("\nglobals (share of positions non-zero; mean = mean |value| when non-zero)");
    println!("  {:<5} {:<8} {:>8} {:>8}  {:>6}", "slot", "block", "train", "leaf", "mean");
    for i in 0..GLOBAL_FEATS {
        let (t, l) = (train.gl_rate(i), leaf.gl_rate(i));
        println!(
            "  g{i:<4} {:<8} {t:7.2}% {l:7.2}%  {:6.3} {}",
            gl_block(i),
            train.gl_mean(i),
            if gap(t, l) { "!" } else { "" }
        );
    }
    println!("\nobject feats (share of all encoded objects non-zero)");
    println!("  {:<5} {:<8} {:>8} {:>8}  {:>6}", "slot", "block", "train", "leaf", "mean");
    for i in 0..OBJ_FEATS {
        let (t, l) = (train.obj_rate(i), leaf.obj_rate(i));
        println!(
            "  f{i:<4} {:<8} {t:7.2}% {l:7.2}%  {:6.3} {}",
            obj_block(i),
            train.obj_mean(i),
            if gap(t, l) { "!" } else { "" }
        );
    }

    // The summary the round-40 finding would have surfaced immediately.
    let flagged: Vec<String> = (0..GLOBAL_FEATS)
        .filter(|&i| gap(train.gl_rate(i), leaf.gl_rate(i)))
        .map(|i| format!("g{i}"))
        .chain(
            (0..OBJ_FEATS)
                .filter(|&i| gap(train.obj_rate(i), leaf.obj_rate(i)))
                .map(|i| format!("f{i}")),
        )
        .collect();
    match flagged.is_empty() {
        true => println!("\nno feature is materially more common at inference than in training"),
        false => println!(
            "\n{} features are live at inference and rare in training: {}",
            flagged.len(),
            flagged.join(" ")
        ),
    }

    // The `!` marker only catches never-trained features. A block that is
    // trained but two or three times denser at the leaves is the same
    // phenomenon at lower contrast, and it is what tells you *which way*
    // the two distributions differ — round 40's leaves turned out to be
    // uniformly "later in the turn, after combat" than its training rows,
    // which no single flagged feature would have shown.
    let mut ratios: Vec<(f64, String, f64, f64)> = (0..GLOBAL_FEATS)
        .map(|i| (format!("g{i}"), train.gl_rate(i), leaf.gl_rate(i)))
        .chain((0..OBJ_FEATS).map(|i| (format!("f{i}"), train.obj_rate(i), leaf.obj_rate(i))))
        .filter(|(_, t, l)| *l >= 5.0 && *t > 0.0 && l > t)
        .map(|(name, t, l)| (l / t, name, t, l))
        .collect();
    ratios.sort_by(|a, b| b.0.total_cmp(&a.0));
    if !ratios.is_empty() {
        println!("\ntrained features the search meets most disproportionately (leaf/train):");
        for (r, name, t, l) in ratios.iter().take(10) {
            println!("  {name:<5} {r:5.2}x   train {t:6.2}%  leaf {l:6.2}%");
        }
    }
}

fn calibrate(args: &Args, vocab: &Vocab, games: usize) {
    let best = args.use_best.as_ref().expect("--calibrate needs --use-best WEIGHTS");
    let cfg = net_config(args, vocab.size());
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    trainer
        .load(best)
        .expect("load weights for calibration (add --attn if these are attention weights)");

    // (net p, heuristic score, actual result, ply)
    let mut obs: Vec<(f32, f32, f32, u16)> = Vec::new();
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0xCA11B);
    for n in 0..games as u64 {
        let salt =
            |k: u64| args.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0xCA1);
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let deck_a = heuristic_sealed_build(&pool_a, salt(3));
        let deck_b = heuristic_sealed_build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            50_000,
            vocab,
        );
        if rec.rows.is_empty() {
            continue;
        }
        let refs: Vec<&TrainRow> = rec.rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("forward");
        for ((row, h), p) in rec.rows.iter().zip(&rec.heur).zip(preds) {
            obs.push((p, *h as f32, row.win, row.ply));
        }
    }
    if obs.len() < 100 {
        eprintln!("calibrate: only {} positions — not enough to say anything", obs.len());
        return;
    }

    // Fit the heuristic's temperature by scanning: one parameter, and a
    // scan cannot land in a bad local optimum the way a gradient step can.
    let mut best_t = 1.0f32;
    let mut best_ll = f32::INFINITY;
    for k in 0..60 {
        let t = 2.0f32.powf(k as f32 / 4.0);
        let ll = log_loss(obs.iter().map(|&(_, h, y, _)| (sigmoid(h / t), y)));
        if ll < best_ll {
            best_ll = ll;
            best_t = t;
        }
    }

    let net_ll = log_loss(obs.iter().map(|&(p, _, y, _)| (p, y)));
    let net_brier = brier(obs.iter().map(|&(p, _, y, _)| (p, y)));
    let heur_brier = brier(obs.iter().map(|&(_, h, y, _)| (sigmoid(h / best_t), y)));
    let net_auc = auc(obs.iter().map(|&(p, _, y, _)| (p, y)));
    let heur_auc = auc(obs.iter().map(|&(_, h, y, _)| (h, y)));
    // The constant predictor: what "knows nothing" scores, so the two
    // numbers above have a floor to be read against.
    let base = obs.iter().map(|&(_, _, y, _)| y).sum::<f32>() / obs.len() as f32;
    let base_ll = log_loss(obs.iter().map(|&(_, _, y, _)| (base, y)));

    println!("calibration on {} positions from {games} games", obs.len());
    println!("  base rate {base:.3}  (log-loss {base_ll:.4} — the score of knowing nothing)");
    println!("  net        log-loss {net_ll:.4}  Brier {net_brier:.4}  AUC {net_auc:.4}");
    println!(
        "  heuristic  log-loss {best_ll:.4}  Brier {heur_brier:.4}  AUC {heur_auc:.4}  (t={best_t:.1})"
    );

    ply_strata(&obs, best_t);

    // Output histogram: the saturation check.
    let mut bins = [0usize; 10];
    for &(p, _, _, _) in &obs {
        bins[((p * 10.0) as usize).min(9)] += 1;
    }
    println!("  net output histogram (0.0..1.0 in tenths):");
    for (i, c) in bins.iter().enumerate() {
        let pct = 100.0 * *c as f64 / obs.len() as f64;
        println!("    {:.1}-{:.1}  {:>6}  {:5.1}%  {}", i as f32 / 10.0, (i + 1) as f32 / 10.0, c, pct, "#".repeat((pct / 2.0) as usize));
    }
    let extreme = obs.iter().filter(|&&(p, _, _, _)| !(0.05..=0.95).contains(&p)).count();
    println!(
        "  {:.1}% of positions score outside [0.05, 0.95] — the search cannot rank lines \
         inside a saturated band",
        100.0 * extreme as f64 / obs.len() as f64
    );
}

/// Score net and heuristic on the positions the search *consumes* —
/// simulated leaves — against the snapshot positions from the same games.
///
/// Ground truth for a leaf is the real game's winner from the evaluating
/// seat's perspective: the same label snapshots carry, applied to the
/// hypothetical positions the search ranked while playing that game. The
/// heuristic's temperature is fitted per distribution, so each evaluator
/// is scored as the best forecast it can support on that set.
fn calibrate_leaves(args: &Args, vocab: &Vocab, games: usize) {
    let best = args.use_best.as_ref().expect("--calibrate-leaves needs --use-best WEIGHTS");
    let cfg = net_config(args, vocab.size());
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    trainer.load(best).expect("load weights (match --attn/width flags to the checkpoint)");

    // (net p, heuristic score, result, turn)
    let mut leaves: Vec<(f32, f32, f32, u16)> = Vec::new();
    let mut snaps: Vec<(f32, f32, f32, u16)> = Vec::new();
    crabomination::server::leaf_capture::set_enabled(true);
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0x1EAF);
    for n in 0..games as u64 {
        let salt =
            |k: u64| args.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x1EA);
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let deck_a = heuristic_sealed_build(&pool_a, salt(3));
        let deck_b = heuristic_sealed_build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            50_000,
            vocab,
        );
        let captured = crabomination::server::leaf_capture::drain();
        let Some(winner) = rec.winner else { continue };

        let leaf_rows: Vec<TrainRow> = captured
            .iter()
            .map(|(state, _, _, _)| TrainRow {
                state: state.clone(),
                win: 0.0,
                life_diff: 0.0,
                game_len: 0.0,
                traj: 0,
                ply: 0,
                aux: [0.0; crabomination_nn::AUX_FEATS],
                opp_hand: Vec::new(),
            })
            .collect();
        let refs: Vec<&TrainRow> = leaf_rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("leaf forward");
        for ((_, heur, seat, turn), p) in captured.iter().zip(preds) {
            let y = if *seat == winner { 1.0 } else { 0.0 };
            leaves.push((p, *heur as f32, y, *turn as u16));
        }

        let refs: Vec<&TrainRow> = rec.rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("snap forward");
        for ((row, h), p) in rec.rows.iter().zip(&rec.heur).zip(preds) {
            snaps.push((p, *h as f32, row.win, row.ply));
        }
    }
    crabomination::server::leaf_capture::set_enabled(false);
    if leaves.len() < 500 || snaps.len() < 500 {
        eprintln!(
            "calibrate-leaves: only {} leaves / {} snapshots — not enough to say anything",
            leaves.len(),
            snaps.len()
        );
        return;
    }

    let fit_t = |obs: &[(f32, f32, f32, u16)]| {
        let (mut best_t, mut best_ll) = (1.0f32, f32::INFINITY);
        for k in 0..60 {
            let t = 2.0f32.powf(k as f32 / 4.0);
            let ll = log_loss(obs.iter().map(|&(_, h, y, _)| (sigmoid(h / t), y)));
            if ll < best_ll {
                best_ll = ll;
                best_t = t;
            }
        }
        best_t
    };
    let score = |name: &str, obs: &[(f32, f32, f32, u16)]| -> (f32, f32) {
        let t = fit_t(obs);
        let n_auc = auc(obs.iter().map(|&(p, _, y, _)| (p, y)));
        let h_auc = auc(obs.iter().map(|&(_, h, y, _)| (h, y)));
        let n_ll = log_loss(obs.iter().map(|&(p, _, y, _)| (p, y)));
        let h_ll = log_loss(obs.iter().map(|&(_, h, y, _)| (sigmoid(h / t), y)));
        println!(
            "  {name:<11} {:>7}  net {n_auc:.4} vs heur {h_auc:.4}  delta {:+.4}   LL {n_ll:.4} vs {h_ll:.4}",
            obs.len(),
            n_auc - h_auc,
        );
        (n_auc, h_auc)
    };

    println!("sim-leaf calibration: {games} games, {} leaves, {} snapshots", leaves.len(), snaps.len());
    let (sn, sh) = score("snapshots", &snaps);
    let (ln_, lh) = score("sim leaves", &leaves);
    // The leaf set skews later than snapshots (searches run every turn but
    // leaves pile up on big boards), so also show the leaf strata: a
    // mismatch confined to one phase reads differently from a global one.
    let early: Vec<_> = leaves.iter().copied().filter(|o| o.3 <= 6).collect();
    let late: Vec<_> = leaves.iter().copied().filter(|o| o.3 > 6).collect();
    if early.len() >= 500 {
        score("leaves t1-6", &early);
    }
    if late.len() >= 500 {
        score("leaves t7+", &late);
    }
    let (ds, dl) = (sn - sh, ln_ - lh);
    println!(
        "verdict: net-minus-heuristic AUC is {ds:+.4} on snapshots and {dl:+.4} on sim leaves — {}",
        if dl < ds * 0.5 {
            "the edge shrinks by more than half off-distribution; the mismatch hypothesis is SUPPORTED"
        } else if dl < ds {
            "the edge shrinks but survives; a partial mismatch at most"
        } else {
            "the edge holds on the positions the search consumes; the mismatch hypothesis is REFUTED"
        }
    );
}

/// Lower edge of each ply stratum; the last bucket is open-ended.
const PLY_EDGES: [u16; 6] = [0, 4, 8, 12, 20, 32];

/// Break the calibration observations down by position-in-game.
///
/// The aggregate numbers above pool every snapshot in every game, and the
/// two ends of a game are not the same problem. Late positions are mostly
/// *already decided* — one player is at 3 life facing a board they cannot
/// beat — and any evaluation that can count power and life gets them
/// right. Early positions are the contested ones, and they are the only
/// ones where the search's choices can still change the result.
///
/// So a net that is better than the heuristic late and worse early posts a
/// better aggregate AUC **and plays worse**, because it is winning the
/// pooled comparison on exactly the positions where being right is free.
/// That is a candidate explanation for the play net's standing result (a
/// strictly better predictor that keeps losing gates) and it is invisible
/// in any aggregate metric, which is why this breakdown exists.
///
/// Buckets are directly comparable: both seats are snapshotted at the same
/// instants and carry the same `ply`, so every stratum holds exactly one
/// win per loss and the base rate is 0.5 throughout. The heuristic's
/// temperature is the one fitted globally — refitting per bucket would
/// hand it a free parameter per stratum that it would not have in play.
fn ply_strata(obs: &[(f32, f32, f32, u16)], heur_t: f32) {
    println!("  by ply (snapshots are ~3/turn/seat, so ply 12 is about turn 4):");
    println!("    {:<8} {:>7}  {:>8} {:>8} {:>7}   {:>8} {:>8}", "ply", "n", "net AUC", "heur", "delta", "net LL", "heur LL");
    let mut first: Option<f32> = None;
    let mut last: Option<f32> = None;
    for (i, &lo) in PLY_EDGES.iter().enumerate() {
        let hi = PLY_EDGES.get(i + 1).copied().unwrap_or(u16::MAX);
        let cut: Vec<&(f32, f32, f32, u16)> =
            obs.iter().filter(|o| o.3 >= lo && o.3 < hi).collect();
        // AUC is undefined without both classes and meaningless on a
        // handful of positions; say so rather than printing noise.
        if cut.len() < 200 {
            println!("    {:<8} {:>7}  (too few positions to score)", label(lo, hi), cut.len());
            continue;
        }
        let n_auc = auc(cut.iter().map(|&&(p, _, y, _)| (p, y)));
        let h_auc = auc(cut.iter().map(|&&(_, h, y, _)| (h, y)));
        let n_ll = log_loss(cut.iter().map(|&&(p, _, y, _)| (p, y)));
        let h_ll = log_loss(cut.iter().map(|&&(_, h, y, _)| (sigmoid(h / heur_t), y)));
        println!(
            "    {:<8} {:>7}  {:>8.4} {:>8.4} {:>+7.4}   {:>8.4} {:>8.4}",
            label(lo, hi),
            cut.len(),
            n_auc,
            h_auc,
            n_auc - h_auc,
            n_ll,
            h_ll
        );
        first.get_or_insert(n_auc - h_auc);
        last = Some(n_auc - h_auc);
    }
    if let (Some(f), Some(l)) = (first, last)
        && f < 0.0
        && l > 0.0
    {
        println!(
            "    ^ the net's edge is late-game only ({f:+.4} earliest bucket, {l:+.4} latest) — \
             the aggregate AUC is being carried by positions the search no longer decides"
        );
    }
}

fn label(lo: u16, hi: u16) -> String {
    if hi == u16::MAX { format!("{lo}+") } else { format!("{lo}-{}", hi - 1) }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn log_loss(it: impl Iterator<Item = (f32, f32)>) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0usize);
    for (p, y) in it {
        let p = p.clamp(1e-6, 1.0 - 1e-6);
        sum += -(y * p.ln() + (1.0 - y) * (1.0 - p).ln());
        n += 1;
    }
    sum / n.max(1) as f32
}

fn brier(it: impl Iterator<Item = (f32, f32)>) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0usize);
    for (p, y) in it {
        sum += (p - y) * (p - y);
        n += 1;
    }
    sum / n.max(1) as f32
}

/// Rank-based AUC. Scale-free, so the heuristic's raw score can be scored
/// against the net's probability without fitting anything.
fn auc(it: impl Iterator<Item = (f32, f32)>) -> f32 {
    let mut v: Vec<(f32, f32)> = it.collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (mut pos, mut neg, mut rank_sum) = (0.0f64, 0.0f64, 0.0f64);
    // Ties share the mean rank, which is what keeps a constant predictor
    // at exactly 0.5 instead of whatever the sort order happened to be.
    let mut i = 0;
    while i < v.len() {
        let mut j = i;
        while j < v.len() && v[j].0 == v[i].0 {
            j += 1;
        }
        let mean_rank = (i + j + 1) as f64 / 2.0;
        for item in &v[i..j] {
            if item.1 > 0.5 {
                pos += 1.0;
                rank_sum += mean_rank;
            } else {
                neg += 1.0;
            }
        }
        i = j;
    }
    if pos == 0.0 || neg == 0.0 {
        return 0.5;
    }
    ((rank_sum - pos * (pos + 1.0) / 2.0) / (pos * neg)) as f32
}

// ───────────────────────── local discrimination ──────────────────────────

/// Can the evaluator order two *adjacent* positions from the same game?
///
/// This exists because `--calibrate` measured the wrong thing for the
/// question that matters. AUC is a *global* ranking statistic: it asks
/// whether a winning board outscores a losing one across a diverse pool of
/// positions. The attention net wins that comparison against
/// `eval_material` (0.798 vs 0.760 on seed 43, replicated 0.761 vs 0.747
/// on seed 97) — and then loses the gate outright, 44.8 % as a
/// replacement and 48.8 % blended.
///
/// The reconciliation is that the search never asks the global question.
/// It compares *near-identical* boards — the same position differing by
/// one attack or one block — dozens of times per decision, and picks the
/// argmax. An evaluator can be excellent at "who is winning" and useless
/// at "which of these two almost-identical lines is better", and only the
/// second is consumed inside a resolved simulation.
///
/// Adjacent snapshots of one trajectory are the cheapest available proxy
/// for that: one turn apart, same deck, same seat, overwhelmingly similar
/// boards. Ground truth is the trajectory's direction — a winner's
/// position tends toward 1 and a loser's toward 0. That is imperfect per
/// pair (a winner's board does not improve monotonically) but it is
/// exactly the assumption the value target already makes, and it is
/// applied identically to both evaluators, so the *comparison* is fair
/// even where the label is noisy.
///
/// Also reported: mean separation. An evaluator that orders pairs
/// correctly but by a hair is still useless to a search whose candidates
/// differ by less than its own noise.
fn pairwise(args: &Args, vocab: &Vocab, games: usize) {
    let best = args.use_best.as_ref().expect("--pairwise needs --use-best WEIGHTS");
    let cfg = net_config(args, vocab.size());
    let mut trainer = Trainer::new(&cfg, args.lr).expect("trainer init");
    trainer.load(best).expect("load weights (add --attn for attention weights)");

    let (mut net_ok, mut heur_ok, mut net_tie, mut heur_tie, mut n) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut net_sep, mut heur_sep) = (0.0f64, 0.0f64);
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0x9A1D);
    for g in 0..games as u64 {
        let salt = |k: u64| args.seed ^ g.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x9A1);
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let deck_a = heuristic_sealed_build(&pool_a, salt(3));
        let deck_b = heuristic_sealed_build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            50_000,
            vocab,
        );
        if rec.rows.len() < 4 {
            continue;
        }
        let refs: Vec<&TrainRow> = rec.rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("forward");

        // Group row indices by trajectory, ordered by ply.
        let mut by_traj: std::collections::HashMap<u32, Vec<usize>> = Default::default();
        for (i, r) in rec.rows.iter().enumerate() {
            by_traj.entry(r.traj).or_default().push(i);
        }
        for idxs in by_traj.values_mut() {
            idxs.sort_by_key(|&i| rec.rows[i].ply);
            for w in idxs.windows(2) {
                let (a, b) = (w[0], w[1]);
                // +1 when this seat won: later snapshots should score
                // higher. -1 when it lost.
                let dir = if rec.rows[a].win > 0.5 { 1.0f64 } else { -1.0 };
                let dn = (preds[b] - preds[a]) as f64;
                // Heuristic scores are unbounded ints; normalise by the
                // pair's own scale so "separation" is comparable to the
                // net's [0,1] output rather than a raw material delta.
                let dh = (rec.heur[b] - rec.heur[a]) as f64 / 1000.0;
                n += 1;
                if dn == 0.0 { net_tie += 1 } else if dn * dir > 0.0 { net_ok += 1 }
                if dh == 0.0 { heur_tie += 1 } else if dh * dir > 0.0 { heur_ok += 1 }
                net_sep += dn.abs();
                heur_sep += dh.abs();
            }
        }
    }
    if n == 0 {
        eprintln!("pairwise: no usable adjacent pairs");
        return;
    }
    // Report the correct-rate among pairs the evaluator actually
    // *separates*, alongside the tie rate. Scoring ties as wrong would
    // punish an evaluator for honestly declining to distinguish two
    // equal positions, which is the opposite of what we want to measure —
    // and mixing the two into one percentage hides the tie rate, which is
    // the interesting number here.
    //
    // Separations are NOT compared across evaluators: the heuristic is an
    // unbounded integer score and the net is a probability, so any shared
    // normalisation would be arbitrary. Each is reported against its own
    // scale only.
    let rate = |ok: usize, ties: usize| {
        let decided = n - ties;
        if decided == 0 { f64::NAN } else { 100.0 * ok as f64 / decided as f64 }
    };
    println!("local discrimination on {n} adjacent same-game pairs from {games} games");
    println!("  (chance is 50 %; this is the question the SEARCH asks, unlike AUC)");
    println!(
        "  net        {:.1}% of separated pairs ordered right   ties {:.1}%   mean |delta| {:.4} (prob scale)",
        rate(net_ok, net_tie),
        100.0 * net_tie as f64 / n as f64,
        net_sep / (n - net_tie).max(1) as f64
    );
    println!(
        "  heuristic  {:.1}% of separated pairs ordered right   ties {:.1}%   mean |delta| {:.1} (eval units)",
        rate(heur_ok, heur_tie),
        100.0 * heur_tie as f64 / n as f64,
        heur_sep * 1000.0 / (n - heur_tie).max(1) as f64
    );
}
