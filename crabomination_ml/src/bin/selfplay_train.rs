//! selfplay_train — the concurrent train-and-play loop for the sealed value
//! net.
//!
//! KataGo-shaped, single process (FEATURE_ROADMAP Tier 13): actor threads
//! play sealed self-play games and push encoded rows into a shared sliding
//! window; the learner trains from it, throttled by a sample-reuse cap so it
//! can't overfit the recent window. Checkpoints land in `--out` as
//! `latest.safetensors` (atomic rename), one `stats.jsonl` line each.
//!
//! Without `--use-best` actors play the heuristic default bot and sealed
//! builder (the bootstrap phase); with it they play the gate-passed net.
//! ```text
//! selfplay_train [--actors N] [--games N] [--steps N] [--batch N]
//!                [--lr F] [--reuse F] [--steps-per-game F]
//!                [--lambda F] [--relabel-every N]
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
//!                [--menu-decisions] [--policy-kinds] [--init-from W.safetensors]
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
//! First smoke run (22 actors, debug build, defaults): ~10 SOS sealed
//! games/s (~36 rows each), and the reuse throttle engaged within 3 % of the
//! 6× cap. Generation is the bottleneck by design.

// Allocator swap, on by default: the simulator spends ~16 % of its
// instructions in malloc/free/memcpy, and system-malloc contention grows with
// the actor count (measured +18-31 % throughput at 4-24 actors for ~1.5-1.7x
// RSS, PERF.md). `--no-default-features` restores the system allocator; a
// `#[global_allocator]` is whole-program, so it lives in the binary.
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
use rand::seq::SliceRandom;

struct Args {
    actors: usize,
    games: Option<u64>,
    steps: Option<u64>,
    batch: usize,
    lr: f64,
    reuse: f64,
    /// Learner steps per generated game, held exactly (whichever side runs
    /// ahead waits, within a few games). 0 (default) leaves the ratio to the
    /// faster side, as before round 86. Round 84: unpinned, the ratio follows
    /// actor speed (r82's FRA seeds read 0.17-0.27 on a shared box) and
    /// strength tracked it as strongly as any recipe change.
    steps_per_game: f64,
    /// Fit the win head with cross-entropy instead of MSE
    /// (`crabomination_ml::binary_cross_entropy`): MSE through a sigmoid
    /// vanishes the gradient where the net is confidently wrong. Off by
    /// default (the control). NOTE: with it on, `loss_win` in stats.jsonl is
    /// a log-loss, not comparable to an MSE run's; compare arms on the
    /// checkpoint's `train_raw` / `val_win`, which stay MSE.
    bce: bool,
    /// Record the pilot's decisions (candidate successor states + the pick)
    /// and train the win head to rank them
    /// (`crabomination::server::decision_capture`,
    /// `Trainer::train_policy_step`). Off by default: it encodes once per
    /// candidate, not once per position.
    record_decisions: bool,
    /// Record EVERY decision over the move list instead of the pickers'
    /// shortlists (`decision_capture::set_menu_only`): one row per decision
    /// (per step for a combat declaration), tagged with its kind — the
    /// imitation stream for `server::policy_pilot`. Implies `--record-decisions`.
    menu_decisions: bool,
    /// `--menu-search mc`: phase-5 actors. MCTS searches these windows over the
    /// full move list (`MctsConfig::menu_windows`; letters m(ain) i(nstant)
    /// c(ombat) p(rompt)) on `--use-policy`'s priors, drawing arms by
    /// Gumbel-top-k (`menu_sample`). Searched decisions record the improved
    /// policy as target; the rest record imitation rows as under
    /// `--menu-decisions` (implied). Needs `--mcts-actors`.
    menu_search: u8,
    /// Most decision rows the learner keeps (`--decision-cap`, default
    /// 200 000). Each holds every candidate's encoded state, so this is most of
    /// the trainer's memory (a 6,000-game FRA search run peaked at 20.6 GB at
    /// the default and was OOM-killed, 2026-11-03).
    decision_cap: usize,
    /// The policy-headed net giving `--menu-search` its priors, loaded into
    /// the candidate slot (CPU forwards; the value net stays `--use-best`).
    use_policy: Option<PathBuf>,
    /// Build the policy head's decision-kind table (`NetConfig::
    /// policy_kinds`). Implies `--policy-head`.
    policy_kinds: bool,
    /// Build the action head (`NetConfig::policy_action`): the policy logit
    /// gains a term read off the decision state's objects at the ones each
    /// move involves. Implies `--policy-kinds`. On `--init-from` a file
    /// without the head loads whole and the head starts at zero.
    policy_action: bool,
    /// `--init-from W.safetensors`: a fresh run's learner takes W's shape and
    /// every tensor W carries, plus the policy head (and kind table under
    /// `--policy-kinds`) seeded from the win head — `--pg`'s warm start, so an
    /// imitation run needn't relearn the trunk. Ignored on resume.
    init_from: Option<PathBuf>,
    /// Learner steps between policy steps under `--record-decisions`; 1 = a
    /// policy step every value step. 0 = record and score `val_policy` but
    /// never train: the control arm (a run without `--record-decisions`
    /// reports no `val_policy` baseline at all).
    policy_every: u64,
    /// Softmax temperature for the distilled policy target. Arm values
    /// are win probabilities, so their spread is small and this is what
    /// decides sharp-pick vs soft-ranking. Ignored for Gumbel captures,
    /// whose improved-policy logits carry their own scale.
    policy_temp: f32,
    /// Build the separate policy head (`head_policy.*`, round 37) and aim the
    /// ranking objective at it instead of the win head; it is exported as the
    /// Gumbel search's prior. Off = the round-33 shared-win-head path. Only
    /// acts under `--record-decisions` with `--policy-every` > 0.
    policy_head: bool,
    /// Pilot the MCTS actors with the Gumbel root search (Sequential Halving
    /// over policy-head priors) instead of UCB1. A headless pilot falls back
    /// to heuristic-score priors and says so at startup. Needs `--mcts-actors`.
    mcts_gumbel: bool,
    /// Which holdout metric publishes `best.safetensors`: false = AUC, true
    /// (`--best-metric policy`) = held-out top-1 policy agreement. Round 34 saw
    /// AUC and ranking move in opposite directions, so picking a distillation
    /// run's artifact on AUC can publish its worst-ranking checkpoint.
    best_policy: bool,
    /// Build and train the opponent-hand belief head (`head_opp.*`, round
    /// 39): per-name hold probabilities, multi-label BCE at the auxiliary
    /// weight. The engine consumes it (belief-weighted determinization), so
    /// exports carry it. Off is the control.
    opp_head: bool,
    /// Weight of the search-value regression in the policy step (round 39,
    /// Reanalyze-lite): the win head pulled toward the search's per-arm mean
    /// rewards on successors, weighted by rollout counts. 0 (default) = off.
    /// UCB1 captures only (Gumbel logits aren't win probabilities); needs
    /// `--record-decisions` and MCTS actors.
    search_value_weight: f32,
    /// TD(lambda) mixing for the win target. 1.0 (default) is the pure Monte
    /// Carlo label (the row's own game result), reproducing every prior gate
    /// round; below 1.0 the target bootstraps through the net's estimate of
    /// the next snapshot, trading outcome variance for the current net's bias
    /// (see `SampleWindow::relabel_lambda`).
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
    /// Override the heuristic actors' `determinize` (round 28c): r25's switch
    /// of the default pilot to honest sims also changed the training data; 0
    /// restores the pre-adoption generator for attribution.
    actor_det: Option<u8>,
    /// Window-passes of tail training after the actors finish (round 28d). The
    /// old fixed reuse/2 allowance became the binding stop on a faster box
    /// (runs died at ~8k steps mid-improvement); keep it high and let
    /// `--stop-after-stale` stop the run.
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
    /// checkpoint; 0 disables. Training loss here measures memorisation (at
    /// lambda=1: 0.017 window MSE against an out-of-sample log-loss of 1.12,
    /// worse than predicting 0.5), and running without this cost six gate rounds.
    ///
    /// Split by trajectory, not row: consecutive snapshots of a game are
    /// near-duplicates, so a row split leaks the answer into validation.
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
    /// `--lr-cosine auto`: the horizon is `--steps-per-game` x `--games`, so
    /// the schedule anneals over the run actually made. A fixed 60 000
    /// left r81 s43 (30,770 steps) at lr factor 0.53 and spent r87's R40
    /// 40 k steps on the floor (2026-11-23 audit).
    lr_cosine_auto: bool,
    /// Sample only rows a relabel pass has reached (their λ-return, never
    /// the raw result they enter with); `SampleWindow::
    /// sample_labelled_with_targets`.
    sample_labelled: bool,
    /// Save every embedding row no trained row ever named as a copy of the
    /// unknown row 0 (`Trainer::track_seen_cards`).
    reset_unseen: bool,
    /// Re-draw the card embedding and group tags at N(0, std) instead of
    /// candle's N(0, 1) (`Trainer::set_embedding_init_std`). Fresh runs only.
    emb_init_std: Option<f64>,
    /// A frozen holdout (`write_shard` rows, from `--write-holdout`): every
    /// checkpoint's val metrics, `best` and the stale stop read it, and the
    /// actors' own holdout games train instead. Seed-independent, so AUC
    /// compares across seeds and arms.
    holdout_file: Option<PathBuf>,
    /// Diagnostic mode: play `--holdout-games` games on a seed stream of
    /// their own, keep at most 4 rows a game (two snapshots, both seats),
    /// write them to this path and exit.
    write_holdout: Option<PathBuf>,
    holdout_games: usize,
    /// Diagnostic mode: score each net in `--nets a,b,..` on this holdout
    /// file (AUC, log-loss, mean prediction, and the ply 0-11 stratum).
    score_holdout: Option<PathBuf>,
    nets: Vec<PathBuf>,
    /// Actor-side softmax action sampling: temperature in eval units
    /// (0 = off). ~120 is one mid-size creature on the heuristic scale;
    /// ~300 ≈ 3 % win probability on the net's ±10 000 scale.
    sample_temp: i32,
    /// Sample through this turn, argmax after (AlphaZero-style cutoff).
    sample_turns: u32,
    /// Pilot actors with net-evaluated MCTS at this many rollouts per decision
    /// (0 = the plain scored bot). Requires `--use-best`; pair with
    /// `--gpu-eval` so rollout rewards batch on the collator. ~50–100× slower
    /// per game than heuristic actors: size `--games` from a measured probe.
    mcts_actors: u32,
    /// Mixed actor fleet (round 37, KataGo playout-cap randomization): only
    /// the first N actor threads pilot with MCTS; the rest play the plain net
    /// pilot and record no decisions. 0 (default) = every actor is MCTS when
    /// `--mcts-actors` is set.
    ///
    /// Policy targets need searched decisions (~1.5 games/s at MCTS-64) while
    /// the value window needs volume (~50–100× that); an all-MCTS fleet left
    /// the learner asleep ~85 % of wall clock (r35/r37). Pair with hundreds of
    /// `--actors`: members mostly block in the batched evaluator, so thread
    /// count is what fills batches and cores.
    mcts_fleet: usize,
    /// Width overrides — the engine reads sizes from the tensor shapes,
    /// so capacity is a flag, not a format change. `--seed-emb` requires
    /// the deck net's width, so it refuses a changed `--emb-dim`.
    emb_dim: Option<usize>,
    obj_hidden: Option<usize>,
    h1: Option<usize>,
    h2: Option<usize>,
    /// Stop the whole run, actors included, once the holdout AUC has gone
    /// this many checkpoints without a new best; 0 (default) never stops.
    /// Saves the compute `best.safetensors` can't (round 11: AUC peaked 112 s
    /// into a 1 538 s run). Needs `--holdout` > 0. Stopping also truncates
    /// generation, so A/B a tuned value against a no-stop control.
    stop_after_stale: u64,
    /// `full` (default) recomputes every window row's λ-return each relabel
    /// tick, bit-identical to prior runs; `new` relabels only rows pushed
    /// since the last tick (`SampleWindow::relabel_lambda_new_rows`) — round
    /// 11's full pass spent 66 M forward-rows against 13.5 M trained. Ignored
    /// at λ = 1.
    relabel_new: bool,
    /// Round 72: off-policy, advantage-weighted policy-gradient steps on
    /// the decision stream (`Trainer::train_pg_step`). Implies
    /// `--record-decisions --policy-head`; needs `--use-best` (the pilot
    /// whose win head seeds the policy head and whose decisions are the
    /// behaviour) and `--sample-temp > 0` (a deterministic behaviour has
    /// no ratio to correct). The net config follows the pilot's file.
    pg: bool,
    /// `--pg` with no value step, relabel or row throttle: the trunk and
    /// win head stay the pilot's, `--steps` counts PG steps, and the reuse
    /// throttle runs on decisions pushed. The round-72 gate arm — its
    /// control is the same file under `net67`.
    pg_only: bool,
    /// PG step every N value steps (ignored under `--pg-only`).
    pg_every: u64,
    /// Decisions per PG step. Lifts the distillation path's 64 cap.
    pg_batch: usize,
    pg_entropy: f32,
    pg_rho_max: f32,
    pg_lambda: f32,
    /// Let the policy gradient reach the trunk (`PgConfig::trunk`).
    pg_trunk: bool,
    /// PG-only: pack the next batch on a worker thread while this one runs
    /// the step (default). `--pg-no-prefetch` packs inline — the control
    /// arm for the throughput A/B.
    pg_prefetch: bool,
    /// Round 73: record each decision's SETTLED successors (the states the
    /// pickers' scores were read from) instead of the one-action ones —
    /// `decision_capture::set_settled`. Pairs with the `net67-pols` pilot.
    capture_settled: bool,
    /// Actor profile under `--use-best`: `net67` (= `dflt` + the net
    /// leaf, the gate's pilot) instead of the historical `net_eval_det1`,
    /// which chains from a base four adoptions behind the default.
    pilot_net67: bool,
    /// `--actor-fra-fixes`: actors play `EvalWeights::with_fra_fixes` (round
    /// 83: train the FRA net on the pilot the client plays FRA with).
    actor_fra_fixes: bool,
    /// `--actor-rated-decks`: build training decks with the rated builder
    /// (`selfplay::heuristic_sealed_build_rated`, one noisy sample like the
    /// default's) on a pool the ratings cover (`card_ratings::rates_pool`),
    /// so the net learns from the decks the client's opponent now builds
    /// (round 84). Any other pool keeps `heuristic_sealed_build`.
    actor_rated_decks: bool,
    /// Build training decks with the gate-passed deck net as judge
    /// (best-of-32 over the heuristic's noisy-greedy candidates) instead of
    /// the heuristic builder's own pick, so actors play the decks of the one
    /// learned component that has cleared the house bar (61.7 % / 60.7 % vs
    /// the static judge).
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
    /// `--calibrate`'s action cap (default 50,000). The actors cap a game at
    /// 4,000 actions and drop longer ones as stalls, so `--calibrate-cap
    /// 4000` scores the net on the distribution it was trained on.
    calibrate_cap: usize,
    /// Diagnostic mode: `--calibrate`, also scored on the simulated leaves the
    /// searches evaluate (`server::leaf_capture`). Tests "better predictor,
    /// worse pilot": a net that keeps its edge on snapshots but loses it on
    /// sim leaves would show exactly that.
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
        steps_per_game: 0.0,
        bce: false,
        record_decisions: false,
        menu_decisions: false,
        menu_search: 0,
        decision_cap: 200_000,
        use_policy: None,
        policy_kinds: false,
        policy_action: false,
        init_from: None,
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
        calibrate_cap: 50_000,
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
        lr_cosine_auto: false,
        sample_labelled: false,
        reset_unseen: false,
        emb_init_std: None,
        holdout_file: None,
        write_holdout: None,
        holdout_games: 2000,
        score_holdout: None,
        nets: Vec::new(),
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
        pg: false,
        pg_only: false,
        pg_every: 1,
        pg_batch: 256,
        pg_entropy: 0.01,
        pg_rho_max: 2.0,
        pg_lambda: 1.0,
        pg_trunk: false,
        pg_prefetch: true,
        capture_settled: false,
        pilot_net67: false,
        actor_fra_fixes: false,
        actor_rated_decks: false,
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
            "--steps-per-game" => a.steps_per_game = val().parse().expect("--steps-per-game"),
            "--bce" => a.bce = true,
            "--record-decisions" => a.record_decisions = true,
            "--menu-decisions" => {
                a.menu_decisions = true;
                a.record_decisions = true;
                continue; // bare flag, consumes no value
            }
            "--menu-search" => {
                for ch in val().chars() {
                    a.menu_search |= match ch {
                        'm' => crabomination::server::mcts::MENU_MAIN,
                        'i' => crabomination::server::mcts::MENU_INSTANT,
                        'c' => crabomination::server::mcts::MENU_COMBAT,
                        'p' => crabomination::server::mcts::MENU_PROMPT,
                        other => panic!("--menu-search: window {other:?} (expected m, i, c, p)"),
                    };
                }
                a.menu_decisions = true;
                a.record_decisions = true;
            }
            "--use-policy" => a.use_policy = Some(PathBuf::from(val())),
            "--decision-cap" => a.decision_cap = val().parse().expect("--decision-cap"),
            "--init-from" => {
                a.init_from = Some(PathBuf::from(val()));
                a.policy_head = true;
            }
            "--policy-kinds" => {
                a.policy_kinds = true;
                a.policy_head = true;
                continue; // bare flag, consumes no value
            }
            "--policy-action" => {
                a.policy_action = true;
                a.policy_kinds = true;
                a.policy_head = true;
                continue; // bare flag, consumes no value
            }
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
            "--actor-fra-fixes" => a.actor_fra_fixes = true,
            "--actor-rated-decks" => a.actor_rated_decks = true,
            "--tail-reuse" => a.tail_reuse = val().parse().expect("--tail-reuse"),
            "--gpu-eval" => {
                a.gpu_eval = true;
                continue; // bare flag, consumes no value
            }
            "--eval-batch" => a.eval_batch = val().parse().expect("--eval-batch"),
            "--eval-flush-us" => a.eval_flush_us = val().parse().expect("--eval-flush-us"),
            "--calibrate" => a.calibrate = Some(val().parse().expect("--calibrate")),
            "--calibrate-cap" => a.calibrate_cap = val().parse().expect("--calibrate-cap"),
            "--calibrate-leaves" => {
                a.calibrate_leaves = Some(val().parse().expect("--calibrate-leaves"))
            }
            "--feature-census" => {
                a.feature_census = Some(val().parse().expect("--feature-census"))
            }
            "--pairwise" => a.pairwise = Some(val().parse().expect("--pairwise")),
            "--use-deck-best" => a.use_deck_best = Some(PathBuf::from(val())),
            "--seed-emb" => a.seed_emb = Some(PathBuf::from(val())),
            // Names are validated once, by the encoder, when `main` sets the
            // mask; a second copy here went stale when the v7 blocks landed.
            "--ablate" => {
                a.ablate = val().split(',').map(|s| s.trim().to_string()).collect();
            }
            "--holdout" => a.holdout = val().parse().expect("--holdout"),
            "--stop-after-stale" => {
                a.stop_after_stale = val().parse().expect("--stop-after-stale")
            }
            "--pg" => {
                a.pg = true;
                a.record_decisions = true;
                a.policy_head = true;
            }
            "--pg-only" => {
                a.pg = true;
                a.pg_only = true;
                a.record_decisions = true;
                a.policy_head = true;
            }
            "--pg-every" => a.pg_every = val().parse().expect("--pg-every"),
            "--pg-batch" => a.pg_batch = val().parse().expect("--pg-batch"),
            "--pg-entropy" => a.pg_entropy = val().parse().expect("--pg-entropy"),
            "--pg-rho-max" => a.pg_rho_max = val().parse().expect("--pg-rho-max"),
            "--pg-lambda" => a.pg_lambda = val().parse().expect("--pg-lambda"),
            "--pg-trunk" => a.pg_trunk = true,
            "--pg-no-prefetch" => a.pg_prefetch = false,
            "--capture-settled" => a.capture_settled = true,
            "--pilot" => {
                a.pilot_net67 = match val().as_str() {
                    "net67" => true,
                    "det1" => false,
                    other => panic!("--pilot: {other:?} (expected net67 or det1)"),
                }
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
            "--lr-cosine" => {
                let v = val();
                if v == "auto" {
                    a.lr_cosine_auto = true;
                } else {
                    a.lr_cosine = v.parse().expect("--lr-cosine N|auto");
                }
            }
            "--sample-labelled" => {
                a.sample_labelled = true;
                continue; // bare flag, consumes no value
            }
            "--reset-unseen" => {
                a.reset_unseen = true;
                continue; // bare flag, consumes no value
            }
            "--emb-init-std" => a.emb_init_std = Some(val().parse().expect("--emb-init-std")),
            "--holdout-file" => a.holdout_file = Some(PathBuf::from(val())),
            "--write-holdout" => a.write_holdout = Some(PathBuf::from(val())),
            "--holdout-games" => a.holdout_games = val().parse().expect("--holdout-games"),
            "--score-holdout" => a.score_holdout = Some(PathBuf::from(val())),
            "--nets" => a.nets = val().split(',').filter(|x| !x.is_empty()).map(PathBuf::from).collect(),
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
    if a.lr_cosine_auto {
        assert!(a.steps_per_game > 0.0, "--lr-cosine auto needs --steps-per-game (the horizon is steps/game x games)");
        a.lr_cosine = (a.steps_per_game * a.games.expect("--lr-cosine auto needs --games") as f64).round().max(1.0) as u64;
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
    cfg.policy_kinds = args.policy_kinds;
    cfg.policy_action = args.policy_action;
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

/// The config a checkpoint file was saved with (heads included), for
/// reloading other runs' artifacts (frozen pilot, batch eval server). candle's
/// `VarMap::load` errors on a var the file lacks and ignores extra file
/// tensors, so the shape must follow the file, not this run's flags (a
/// `--policy-head` run piloted by a headless net is the normal gen-0 case).
fn file_net_config(
    path: &std::path::Path,
    vocab: usize,
) -> Option<crabomination_ml::NetConfig> {
    let bytes = std::fs::read(path).ok()?;
    let n = crabomination_nn::PlayNet::load(&bytes).ok()?;
    // The choke point for every trainer-side load of an external play net,
    // so the vocabulary rule lives here: `Trainer::load` pads a short
    // embedding table on trust, and a pre-freeze net is exactly the file
    // that must not be padded.
    if let Err(e) = crabomination::server::vocab_snapshot::vocab_fit(n.vocab_size(), vocab) {
        eprintln!("{}: {e}", path.display());
        return None;
    }
    let (emb_dim, obj_hidden, h1, h2, attn, blocks) = n.arch();
    Some(crabomination_ml::NetConfig {
        vocab,
        emb_dim,
        obj_hidden,
        h1,
        h2,
        attn,
        // Training-only head; candle's load ignores file tensors the
        // model lacks, so an aux-trained artifact still loads.
        aux: false,
        blocks,
        policy: n.has_policy_head(),
        policy_kinds: n.has_policy_kinds(),
        policy_action: n.has_action_head(),
        opp: n.has_opp_head(),
    })
}

/// True when this trajectory belongs to the held-out set. A hash rather than
/// a counter, so actors decide independently and a game always lands on the
/// same side.
///
/// Hashes the game (`traj >> 1`), not the seat (fixed 2026-11-09, ML_NOTES
/// "Research sweep"): `traj = (seed << 1) | seat`, and a per-seat split put
/// ~95 % of held-out trajectories' mirrors in training, inflating `val_*`.
/// Holdout numbers across the fix are not comparable.
fn is_holdout(traj: u32, frac: f64) -> bool {
    if frac <= 0.0 {
        return false;
    }
    let mut h = ((traj >> 1) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
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
    /// Recorded decisions (candidate successor states plus the pick); policy
    /// batches sample this capped deque uniformly. Separate from `window`: a
    /// decision has a different shape and cadence.
    decisions: Mutex<std::collections::VecDeque<crabomination_ml::DecisionRow>>,
    /// Held-out decisions, never trained on — the policy top-1 metric.
    decisions_val: Mutex<Vec<crabomination_ml::DecisionRow>>,
    /// Rows ever pushed (not evicted-adjusted) — the reuse cap's basis.
    rows_pushed: AtomicU64,
    /// Decisions pushed to the training deque — the PG-only throttle's
    /// unit, as `rows_pushed` is the value loop's.
    decisions_pushed: AtomicU64,
    /// Of those, the rows a searched menu window recorded (`--menu-search`:
    /// the improved policy, with its rollout counts) — the rest imitate.
    decisions_searched: AtomicU64,
    /// Next self-play game index to claim — also the per-game seed salt.
    next_game: AtomicU64,
    games_done: AtomicU64,
    /// Run start, and how far in the last game finished (ms). A `--steps`-bound
    /// run outlives its actors, so `games / elapsed` divides by the learner's
    /// time (identical actor work read 242.6 vs 92.6 games/s at `--steps 1`
    /// vs `200`); these report the actor window on its own.
    run_start: Instant,
    actor_ms: AtomicU64,
    /// Games played by the MCTS fleet specifically. Decisions are the
    /// binding budget of a distillation run, so the mix has to be
    /// legible per checkpoint rather than inferred from the deque.
    games_mcts: AtomicU64,
    stalls: AtomicU64,
    /// The ways a stall happens, so a moving stall rate names its own cause
    /// instead of only its size. `stalls - capped - board - stuck` is the
    /// remainder: a game that ended by the rules but produced no rows.
    stalls_capped: AtomicU64,
    /// ⚠ A board cap is its own counter because it costs its own amount (it
    /// was once summed into `stalls_capped`): on `cube` 1215 (951 Scute
    /// Swarms, 987 permanents) four undecided games cost ~51 s each against
    /// ~3.3 ms for a normal one on `release-fast` — 0.125 % of the cell's
    /// games, 95 % of its wall clock. Invisible in `games_per_s`, yet decides it.
    stalls_board: AtomicU64,
    stalls_stuck: AtomicU64,
    live_actors: AtomicU64,
    /// Set by the learner when `--stop-after-stale` trips: the holdout
    /// has stopped improving, so further generation is compute spent
    /// making `latest` worse. Actors finish their current game and exit.
    stop: AtomicBool,
    /// `--steps-per-game`'s two sides. The learner publishes its step
    /// count and, at its first step, how many games were done then (the
    /// pin's origin: nothing trains before `--min-window` fills);
    /// `learner_done` releases actors waiting on a learner that has
    /// stopped (a `--steps` cap leaves the actors playing out `--games`).
    learner_step: AtomicU64,
    pin_origin: AtomicU64,
    learner_done: AtomicBool,
}

/// Games either side of `--steps-per-game` may drift before it waits: wide
/// enough that the actors' in-flight games can never hold both sides
/// waiting at once (that needs more than twice this many in flight).
fn pin_slack(args: &Args) -> u64 {
    (args.actors as u64).max(8)
}

const DECK_WINDOW_CAP: usize = 200_000;

/// The one place a stored deck net is read (the play net's `load_slot`
/// counterpart), so call sites can't disagree on the diagnosis. `why` is what
/// the caller wanted the net for; it goes in the missing-file message.
fn load_deck_net(path: &std::path::Path, vocab: &Vocab, why: &str) -> DeckNet {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e} ({why})", path.display()));
    let mut net = DeckNet::load(&bytes).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
    let (have, want) = (net.vocab_size(), vocab.size());
    if let Err(e) = crabomination::server::vocab_snapshot::vocab_fit(have, want) {
        panic!("{}: {e}", path.display());
    }
    if let Err(e) = net.pad_vocab(want) {
        panic!("{}: {e:?}", path.display());
    }
    if have < want {
        eprintln!(
            "{}: deck net trained at vocab {have}, encoder is {want} — padded the {} newer \
             cards to the unknown row (row 0). They score as unknown until the net is retrained.",
            path.display(),
            want - have,
        );
    }
    net
}

fn actor_loop(shared: &Shared, args: &Args, vocab: &Vocab, deck_judge: Option<&DeckNet>, mcts: bool) {
    let actor_search = crabomination::selfplay::actor_mcts(args.mcts_actors, args.mcts_gumbel).map(|c| {
        if args.menu_search == 0 {
            return c;
        }
        crabomination::server::MctsConfig {
            menu_windows: args.menu_search,
            policy_slot: crabomination::server::net_eval::SLOT_CANDIDATE,
            menu_sample: true,
            ..c
        }
    });
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            break;
        }
        // `--steps-per-game`: hold off claiming a game while the learner is
        // behind the pinned ratio.
        if args.steps_per_game > 0.0 {
            let origin = shared.pin_origin.load(Ordering::Relaxed);
            let claimed = shared.next_game.load(Ordering::Relaxed);
            let step = shared.learner_step.load(Ordering::Relaxed);
            if origin != u64::MAX
                && !shared.learner_done.load(Ordering::Relaxed)
                && (step as f64)
                    < args.steps_per_game * claimed.saturating_sub(origin + pin_slack(args)) as f64
            {
                std::thread::sleep(std::time::Duration::from_millis(20));
                continue;
            }
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
            None if args.actor_rated_decks && crabomination::card_ratings::rates_pool(pool) => {
                crabomination::selfplay::heuristic_sealed_build_rated(pool, seed)
            }
            None => heuristic_sealed_build(pool, seed),
        };
        let deck_a = build(&pool_a, salt(3));
        let deck_b = build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        // Honest pilots by default, MCTS or not. Non-MCTS actors once got plain
        // `net_eval()` (determinize 0), whose sims read the opponent's hand and
        // library order, to train a net deployed behind `net_eval_det1()`
        // (`lobby.rs`): a train/deploy mismatch. Round 30 gated det0 vs det1 as
        // a null (a mirror ladder is nearly blind to it: both seats cheat
        // alike). `--actor-det` forces either value as the control arm.
        let mut pilot = if args.use_best.is_some() {
            // `--pilot net67`: the gate's own profile (round 72);
            // `net_eval_det1` chains from a base four adoptions behind `dflt`.
            if args.pilot_net67 { EvalWeights::net_on_default() } else { EvalWeights::net_eval_det1() }
        } else {
            EvalWeights::default()
        };
        if let Some(d) = args.actor_det {
            pilot.determinize = d;
        }
        if args.actor_fra_fixes {
            pilot = pilot.with_fra_fixes();
        }
        let rec = crabomination::selfplay::play_recorded_game_mcts(
            &template,
            [pilot, pilot],
            salt(5),
            4_000,
            vocab,
            if mcts { actor_search } else { None },
        );
        if mcts && args.mcts_actors > 0 {
            shared.games_mcts.fetch_add(1, Ordering::Relaxed);
        }
        // Decisions are captured into a thread-local buffer, so each actor
        // drains its own. Split by the value rows' trajectory hash, so a
        // seat's decisions land on the same side as its positions (before
        // round 37 this hashed the game index, leaking one into the other).
        if args.record_decisions {
            let captured = crabomination::server::decision_capture::drain();
            if !captured.is_empty() {
                let decision_cap = args.decision_cap;
                const DECISION_VAL_CAP: usize = 4_000;
                let traj_base = (salt(5) as u32) << 1;
                // Stamp the return while the result is in hand (round 72): 1
                // for the deciding seat's win, 0 for a loss, NaN for no
                // winner; under `--pg` an unlabelled game's decisions are dropped.
                let converted: Vec<(bool, crabomination_ml::DecisionRow)> = captured
                    .into_iter()
                    .filter(|_| !(args.pg && rec.winner.is_none()))
                    .map(|d| {
                        let ret = match rec.winner {
                            Some(w) if w == d.seat => 1.0,
                            Some(_) => 0.0,
                            None => f32::NAN,
                        };
                        (
                            is_holdout(traj_base | d.seat as u32, args.holdout),
                            crabomination_ml::DecisionRow {
                                successors: d.successors,
                                chosen: d.chosen,
                                values: d.values,
                                values_are_logits: d.values_are_logits,
                                visits: d.visits,
                                scores: d.scores,
                                temp: d.temp,
                                net_scored: d.net_scored,
                                seat: d.seat as u8,
                                turn: d.turn,
                                ply: d.ply,
                                settled: d.settled,
                                kind: d.kind,
                                root: d.root,
                                refs: d.refs,
                                ret,
                            },
                        )
                    })
                    .collect();
                let mut v = shared.decisions_val.lock().unwrap();
                let mut d = shared.decisions.lock().unwrap();
                let mut pushed = 0u64;
                let mut searched = 0u64;
                for (held, row) in converted {
                    if held {
                        if v.len() < DECISION_VAL_CAP {
                            v.push(row);
                        }
                    } else {
                        if d.len() >= decision_cap {
                            d.pop_front();
                        }
                        searched += (row.values_are_logits && row.visits.is_some()) as u64;
                        d.push_back(row);
                        pushed += 1;
                    }
                }
                drop(d);
                drop(v);
                shared.decisions_pushed.fetch_add(pushed, Ordering::Relaxed);
                shared.decisions_searched.fetch_add(searched, Ordering::Relaxed);
            }
        }
        shared.games_done.fetch_add(1, Ordering::Relaxed);
        shared
            .actor_ms
            .fetch_max(shared.run_start.elapsed().as_millis() as u64, Ordering::Relaxed);
        if rec.rows.is_empty() {
            shared.stalls.fetch_add(1, Ordering::Relaxed);
            match rec.stop {
                StopReason::ActionCap => {
                    shared.stalls_capped.fetch_add(1, Ordering::Relaxed);
                }
                // Counted apart (see `stalls_board`): its rate, not the stall
                // total, decides actor throughput.
                StopReason::BoardCap => {
                    shared.stalls_board.fetch_add(1, Ordering::Relaxed);
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
        // A whole game's rows share a trajectory pair and `is_holdout`
        // hashes the game half of the id, so both seats land on one side.
        // A frozen holdout replaces the actors' own: every row trains.
        let frozen = args.holdout_file.is_some();
        let (val_rows, train_rows): (Vec<TrainRow>, Vec<TrainRow>) =
            rec.rows.into_iter().partition(|r| !frozen && is_holdout(r.traj, args.holdout));
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
/// drained into `stats.jsonl` at each one. Added after round 11's runs
/// reached only 1.56× of the 6× reuse cap and only arithmetic on the flags
/// showed the λ-relabel pass was why.
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

fn sample_owned(shared: &Shared, n: usize, rng: &mut StdRng, labelled: bool) -> Vec<(TrainRow, f32)> {
    let w = shared.window.lock().unwrap();
    let draws = if labelled { w.sample_labelled_with_targets(n, rng) } else { w.sample_with_targets(n, rng) };
    draws.into_iter().map(|(r, t)| (r.clone(), t)).collect()
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

/// Two-sided 95% Student-t critical value: exact through 30 degrees of
/// freedom, then coarse steps (±2·se at small n overstated round 41).
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

/// Run `per_pool` for pool indices `0..n` on up to `threads` workers (32 MB
/// stacks, like every game-simulating thread), results in pool order. Deck
/// construction matches the sequential loop exactly; game tallies drift a
/// game or two per pool because the bots' jitter draws from the thread RNG
/// (30.0% vs 29.9% on a 4,800-game gate) — equivalent, not bit-identical.
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
    let net = load_deck_net(&path, vocab, "train a deck net first");
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
    let net = load_deck_net(&path, vocab, "need a deck net for the search variants");
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
    let net = load_deck_net(&path, vocab, "train a deck net first");
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
/// recipe) on the same pools, pilots and seeds: single-sample v3 (the
/// scoring change alone) and best-of-16 v3 (the client's sealed opponent).
fn gate_builder_v3(args: &Args, games_per_pool: usize) {
    use crabomination::cube::CardFactory;
    use crabomination::recommend::{Pilot, simulate_match_games_piloted};
    // Pools, not games per pool, buy precision: each side fields one draw
    // from its builder per pool, so pool-level spread (sd ~15 points)
    // dominates. 12 pools × 800 games gave ±6.5 points; 192 × 50 gave ~±2.3.
    // Default matches the v2 gate; CRAB_GATE_POOLS overrides.
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
        // Wilson treats games as independent, but each pool is one draw from
        // each builder (sd ~15 pts between pools), so the verdict reads this
        // pool-level t-interval instead.
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
    // No seat here renders a prompt; see `game::set_prompt_text`.
    crabomination::game::set_prompt_text(false);
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
    // As with the ablation: every encoding path, diagnostics included, must
    // use the snapshot cadence the diagnosed run recorded with.
    crabomination::selfplay::set_combat_snapshots(args.record_combat);
    eprintln!(
        "snapshot cadence: turn/postcombat/end{}",
        if args.record_combat { " + declare-blockers/end-combat" } else { " (no combat rows)" }
    );
    let vocab = Vocab::sos_sealed();
    // Every trainer built from here on (CUDA) starts from `--seed`'s init.
    crabomination_ml::set_init_seed(args.seed);
    if let Some(path) = &args.write_holdout {
        write_holdout(&args, &vocab, path);
        return;
    }
    if let Some(path) = &args.score_holdout {
        score_holdout(&args, &vocab, path);
        return;
    }
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
    let pg_cfg = crabomination_ml::PgConfig {
        rho_max: args.pg_rho_max,
        entropy: args.pg_entropy,
        lambda: args.pg_lambda,
        trunk: args.pg_trunk,
    };
    if args.pg {
        assert!(
            args.use_best.is_some(),
            "--pg needs --use-best: the pilot's win head seeds the policy head and its \
             decisions are the behaviour"
        );
        assert!(
            args.sample_temp > 0,
            "--pg needs --sample-temp > 0: a deterministic behaviour has no ratio to correct"
        );
    }
    // Under `--pg` the learner's shape follows the pilot's file (widths,
    // attention, belief head), plus the policy head — the run trains on
    // top of that net, not beside it.
    let cfg = match (&args.use_best, &args.init_from) {
        (Some(best), _) if args.pg => {
            let mut c = file_net_config(best, vocab.size())
                .unwrap_or_else(|| panic!("--pg: cannot read the pilot net {}", best.display()));
            c.policy = true;
            c
        }
        (_, Some(init)) => {
            let mut c = file_net_config(init, vocab.size())
                .unwrap_or_else(|| panic!("--init-from: cannot read {}", init.display()));
            c.policy = true;
            c.policy_kinds = args.policy_kinds || c.policy_kinds;
            c.policy_action = args.policy_action || c.policy_action;
            c
        }
        _ => net_config(&args, vocab.size()),
    };
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
    if let Some(std) = args.emb_init_std {
        trainer.set_embedding_init_std(std).expect("--emb-init-std");
        eprintln!("embeddings and group tags re-drawn at N(0, {std})");
    }
    if args.reset_unseen {
        trainer.track_seen_cards(cfg.vocab);
        eprintln!("unseen embedding rows save as the unknown row (--reset-unseen)");
    }
    // A frozen copy of the pilot, for the `pilot_policy` reference.
    // lr 0 and never stepped: this exists only to score.
    let pilot_trainer: Option<Trainer> = args.record_decisions.then_some(args.use_best.as_ref()).flatten().and_then(|p| {
        // The pilot's shape follows the pilot's file, not this run's
        // architecture flags — widths included, or a wide-learner run
        // loses its pilot baseline (see `file_net_config`).
        let cfg = file_net_config(p, vocab.size())?;
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
    } else if let Some(init) = args.init_from.as_ref().filter(|_| !args.pg) {
        // A file that already carries the policy head (a previous imitation
        // run) loads whole; one without it gets the head seeded from the
        // win head.
        const ACTION: [&str; 8] = [
            "head_action.obj.weight",
            "head_action.obj.bias",
            "head_action.role.weight",
            "head_action.ctx.weight",
            "head_action.ctx.bias",
            "head_action.kind.weight",
            "head_action.out.weight",
            "head_action.out.bias",
        ];
        if trainer.load(init).is_ok() {
            eprintln!("init: every tensor from {}", init.display());
        } else if args.policy_action && trainer.load_missing_ok(init, &ACTION).is_ok() {
            // A policy-headed net gaining the action head: everything it
            // has, the new head fresh (its output layer at zero, so the
            // first ranking is the file's own).
            eprintln!("init: every tensor from {}, the action head fresh", init.display());
        } else {
            trainer
                .load_missing_ok(
                    init,
                    &[
                        "head_policy.weight",
                        "head_policy.bias",
                        "head_policy_kind.weight",
                        "head_policy_kind.bias",
                        "head_action.obj.weight",
                        "head_action.obj.bias",
                        "head_action.role.weight",
                        "head_action.ctx.weight",
                        "head_action.ctx.bias",
                        "head_action.kind.weight",
                        "head_action.out.weight",
                        "head_action.out.bias",
                    ],
                )
                .expect("--init-from: load the net into the learner");
            trainer.init_policy_head_from_win().expect("--init-from: seed head_policy from head_win");
            eprintln!("init: weights from {}, head_policy initialised from head_win", init.display());
        }
    } else if args.pg {
        // The warm start (round 72): every tensor from the pilot, and the
        // policy head as a copy of the win head, so the policy's first
        // ranking is exactly the pilot's own.
        let best = args.use_best.as_ref().expect("checked above");
        trainer
            .load_missing_ok(best, &["head_policy.weight", "head_policy.bias"])
            .expect("--pg: load the pilot into the learner");
        trainer.init_policy_head_from_win().expect("--pg: seed head_policy from head_win");
        eprintln!(
            "pg: weights from {}, head_policy initialised from head_win ({}; rho_max {}, entropy {}, lambda {}, batch {})",
            best.display(),
            if args.pg_trunk { "gradient into the trunk" } else { "head only, trunk and win head frozen" },
            args.pg_rho_max,
            args.pg_entropy,
            args.pg_lambda,
            args.pg_batch
        );
    } else if let Some(src) = &args.seed_emb {
        // Fresh runs only: on resume this would overwrite embeddings already
        // trained on real positions.
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
        let net = load_deck_net(p, &vocab, "the deck judge for --use-deck-best");
        eprintln!("actors build decks with the net from {}", p.display());
        net
    });
    if let Some(best) = &args.use_best {
        if args.gpu_eval {
            let cfg = file_net_config(best, vocab.size()).unwrap_or_else(|| {
                panic!("--gpu-eval: cannot read the pilot net {}", best.display())
            });
            // `BatchEvalClient` serves the win and the shared policy logit
            // only; the trait defaults would silently drop a kind table, an
            // action head and a belief head, so the actors' priors would not
            // be the net the lobby plays (2026-11-23 audit).
            let bytes = std::fs::read(best).unwrap_or_else(|e| panic!("{}: {e}", best.display()));
            let net = crabomination_nn::PlayNet::load(&bytes).unwrap_or_else(|e| panic!("{}: {e:?}", best.display()));
            if net.has_policy_kinds() || net.has_action_head() || net.has_opp_head() {
                panic!(
                    "--gpu-eval cannot serve {}: it carries a kind table, action head or belief head \
                     that the batched client would drop; run without --gpu-eval",
                    best.display()
                );
            }
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

    if args.menu_search != 0 {
        assert!(args.mcts_actors > 0, "--menu-search shapes the MCTS actors' search; it needs --mcts-actors > 0");
        let p = args.use_policy.as_ref().expect("--menu-search needs --use-policy <policy-headed weights>");
        crabomination::server::net_eval::load_slot(crabomination::server::net_eval::SLOT_CANDIDATE, p)
            .expect("--use-policy weights load");
        assert!(
            crabomination::server::net_eval::slot_has_policy(crabomination::server::net_eval::SLOT_CANDIDATE),
            "--use-policy {}: no policy head",
            p.display()
        );
        eprintln!("menu-search actors (windows {:#x}) on the policy priors of {}", args.menu_search, p.display());
    }
    if args.record_decisions {
        crabomination::server::decision_capture::set_enabled(true);
        crabomination::server::decision_capture::set_menu_only(args.menu_decisions);
        if args.capture_settled {
            crabomination::server::decision_capture::set_settled(true);
            eprintln!("recording SETTLED successors (round 73)");
        }
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
        // Say once at startup which prior the Gumbel actors run with: a
        // headless pilot silently falls back to heuristic-score priors (a
        // valid control, but a different experiment).
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
        val: Mutex::new(match &args.holdout_file {
            Some(p) => {
                let bytes = std::fs::read(p).unwrap_or_else(|e| panic!("--holdout-file {}: {e}", p.display()));
                let rows = crabomination_nn::read_shard(&bytes)
                    .unwrap_or_else(|| panic!("--holdout-file {}: not a shard", p.display()));
                eprintln!("frozen holdout: {} rows from {}", rows.len(), p.display());
                rows
            }
            None => Vec::new(),
        }),
        deck_window: Mutex::new(std::collections::VecDeque::new()),
        decisions: Mutex::new(std::collections::VecDeque::new()),
        decisions_val: Mutex::new(Vec::new()),
        rows_pushed: AtomicU64::new(0),
        decisions_pushed: AtomicU64::new(0),
        decisions_searched: AtomicU64::new(0),
        next_game: AtomicU64::new(0),
        games_done: AtomicU64::new(0),
        run_start: Instant::now(),
        actor_ms: AtomicU64::new(0),
        games_mcts: AtomicU64::new(0),
        stalls: AtomicU64::new(0),
        stalls_capped: AtomicU64::new(0),
        stalls_board: AtomicU64::new(0),
        stalls_stuck: AtomicU64::new(0),
        live_actors: AtomicU64::new(args.actors as u64),
        stop: AtomicBool::new(false),
        learner_step: AtomicU64::new(0),
        pin_origin: AtomicU64::new(u64::MAX),
        learner_done: AtomicBool::new(false),
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

    // One clock for both, so the actor window and the run's elapsed are
    // measured from the same origin.
    let start = shared.run_start;
    // The per-thread fleet flag is a Copy the closure must own (a loop
    // local can't outlive the scope), so the shared state is re-borrowed
    // here for the `move` closures to capture by reference.
    let shared_r = &shared;
    let args_r = &args;
    let vocab_r = &vocab;
    let judge_r = deck_judge.as_ref();
    // Owned outside the scope so the prefetch worker can borrow it.
    let prefetch_stop_cell = AtomicBool::new(false);
    std::thread::scope(|scope| {
        for i in 0..args.actors {
            let mcts_thread = mcts_all && (!fleet_split || i < args.mcts_fleet);
            // SOS games overflow the default stack — match the ladder's
            // 32 MB workers.
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
                    // In a mixed fleet only search pilots feed the decision
                    // stream: value actors' one-hot imitation picks would bury
                    // the distillation targets at ~50-100x the games rate.
                    // Thread-local, like the sampling config above.
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
        let mut policy_ema = f32::NAN;
        let mut policy_loss_ema = f32::NAN;
        let mut policy_sv_ema = f32::NAN;
        // (samples consumed when the actors finished, tail allowance).
        let mut tail_budget = None::<(u64, u64)>;
        let stats_path = args.out.join("stats.jsonl");
        let mut prev_interval = Interval::default();
        // Round 72: EMAs of [loss, adv_mean, adv_std, entropy, rho_mean,
        // clip_frac, det_frac], the wide-row skip count, and the one-shot
        // temperature sanity.
        let pg_only = args.pg_only;
        let mut pg_ema = [f32::NAN; 7];
        let mut pg_wide = 0u64;
        let mut pg_sanity_done = false;
        // The prefetch pipeline (PG-only): a worker samples and packs the
        // next batch on the CPU while this thread runs the current step on
        // the GPU. Two batches of slack — enough to overlap, too few to run
        // meaningfully past the reuse throttle. Dropping the receiver ends
        // the worker; `prefetch_stop` ends it while it waits for data.
        let (pg_tx, pg_rx) = std::sync::mpsc::sync_channel::<crabomination_ml::PgBatch>(2);
        let prefetch_stop = &prefetch_stop_cell;
        if pg_only && args.pg_prefetch {
            let pg_batch = args.pg_batch;
            let seed = args.seed;
            std::thread::Builder::new()
                .name("pg-prefetch".into())
                .spawn_scoped(scope, move || {
                    let mut prng = StdRng::seed_from_u64(seed ^ 0x9F3E_7C11);
                    while !prefetch_stop.load(Ordering::Relaxed) {
                        let sample: Vec<crabomination_ml::DecisionRow> = {
                            let d = shared_r.decisions.lock().unwrap();
                            if d.len() < pg_batch.max(256) {
                                Vec::new()
                            } else {
                                (0..pg_batch).map(|_| d[prng.random_range(0..d.len())].clone()).collect()
                            }
                        };
                        if sample.is_empty() {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                        let refs: Vec<&crabomination_ml::DecisionRow> = sample.iter().collect();
                        // Host buffers only: the learner uploads them with the
                        // inline path's direct `from_vec`, so the step pays no
                        // device-to-device copy for having been prefetched.
                        if pg_tx.send(crabomination_ml::prepare_pg_batch(&refs)).is_err() {
                            break;
                        }
                    }
                })
                .expect("spawn pg-prefetch");
        } else {
            drop(pg_tx);
        }
        loop {
            if let Some(max) = args.steps
                && step >= max
            {
                break;
            }
            // The throttle's unit: rows and the value batch, or — PG-only —
            // decisions and the PG batch.
            let pushed = if pg_only {
                shared.decisions_pushed.load(Ordering::Relaxed)
            } else {
                shared.rows_pushed.load(Ordering::Relaxed)
            };
            let unit = if pg_only { args.pg_batch } else { args.batch } as u64;
            let actors_live = shared.live_actors.load(Ordering::Relaxed) > 0;
            // Once generation ends, the global reuse budget would pile every
            // further sample onto the final window (~14× visits measured on
            // the first release run). Grant the tail `--tail-reuse` passes
            // over the window it has, then stop.
            if !actors_live && tail_budget.is_none() {
                let wlen = if pg_only {
                    shared.decisions.lock().unwrap().len() as u64
                } else {
                    shared.window.lock().unwrap().len() as u64
                };
                tail_budget = Some((consumed, (args.tail_reuse * wlen as f64) as u64));
            }
            if let Some((at, budget)) = tail_budget
                && consumed - at + unit > budget
            {
                break;
            }
            let budget = (args.reuse * pushed as f64) as u64;
            // `--steps-per-game`, the learner's side: wait while it is
            // ahead of the actors. Off once generation ends (the tail).
            let games_done = shared.games_done.load(Ordering::Relaxed);
            let pin_ahead = args.steps_per_game > 0.0 && actors_live && {
                let origin = shared.pin_origin.load(Ordering::Relaxed);
                origin != u64::MAX
                    && step as f64
                        >= args.steps_per_game * (games_done.saturating_sub(origin) + pin_slack(&args)) as f64
            };
            if pushed < args.min_window || consumed + unit > budget || pin_ahead {
                if !actors_live {
                    break; // no more data coming and the reuse budget is spent
                }
                let t0 = Instant::now();
                std::thread::sleep(std::time::Duration::from_millis(200));
                timing.sleep += t0.elapsed();
                continue;
            }
            if shared.pin_origin.load(Ordering::Relaxed) == u64::MAX {
                shared.pin_origin.store(games_done, Ordering::Relaxed);
            }
            if !pg_only {
                // Refresh the λ-returns every `relabel_every` steps: computed
                // through the net, they go stale as it learns, and each
                // refresh is a forward pass over the window. Skipped at λ = 1.
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
                let rows = sample_owned(&shared, args.batch, &mut rng, args.sample_labelled);
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
                shared.learner_step.store(step, Ordering::Relaxed);
                for (ema, part) in loss_ema
                    .iter_mut()
                    .zip([loss.total, loss.win, loss.life, loss.len, loss.opp])
                {
                    *ema = if ema.is_nan() { part } else { 0.99 * *ema + 0.01 * part };
                }

                // Policy steps ride alongside the value steps: the same net,
                // taught to rank a decision's candidate successors as the
                // pilot did, sampled from the separate decision window.
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

            }

            // Round 72: the policy-gradient step. Alongside the value steps
            // at `--pg-every`, or — PG-only — the whole loop: `step` and
            // `consumed` advance here, on decisions.
            if args.pg && (pg_only || step.is_multiple_of(args.pg_every.max(1))) {
                // PG-only: the next packed batch from the prefetch worker
                // (the wait for it is "sample" time). Alongside value
                // steps: sampled and packed here, synchronously.
                let t0 = Instant::now();
                let prepared: Option<crabomination_ml::PgBatch> = if pg_only && args.pg_prefetch {
                    match pg_rx.recv() {
                        Ok(pb) => Some(pb),
                        Err(_) => break, // the worker died; nothing more will come
                    }
                } else {
                    let sample: Vec<crabomination_ml::DecisionRow> = {
                        let d = shared.decisions.lock().unwrap();
                        if d.len() < args.pg_batch.max(256) {
                            Vec::new()
                        } else {
                            (0..args.pg_batch)
                                .map(|_| d[rng.random_range(0..d.len())].clone())
                                .collect()
                        }
                    };
                    if sample.is_empty() {
                        None
                    } else {
                        let refs: Vec<&crabomination_ml::DecisionRow> = sample.iter().collect();
                        Some(crabomination_ml::prepare_pg_batch(&refs))
                    }
                };
                timing.sample += t0.elapsed();
                if prepared.is_none() && pg_only {
                    // Inline packing found too few decisions: wait for the
                    // actors, as the worker would have.
                    let t0 = Instant::now();
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    timing.sleep += t0.elapsed();
                    continue;
                }
                if let Some(pb) = prepared {
                    if !pg_sanity_done && !pb.is_empty() {
                        pg_sanity_done = true;
                        pg_temperature_sanity(&pb, args.sample_temp);
                    }
                    if pg_only && args.lr_cosine > 0 {
                        trainer.set_lr_factor(crabomination_ml::cosine_lr_factor(step, args.lr_cosine));
                    }
                    let t0 = Instant::now();
                    let st = trainer.train_pg_step_prepared(pb, pg_cfg).expect("pg step");
                    timing.step += t0.elapsed();
                    pg_wide += st.wide_skipped as u64;
                    if st.n > 0 {
                        let parts = [
                            st.loss,
                            st.adv_mean,
                            st.adv_std,
                            st.entropy,
                            st.rho_mean,
                            st.clip_frac,
                            st.det_frac,
                        ];
                        for (ema, part) in pg_ema.iter_mut().zip(parts) {
                            *ema = if ema.is_nan() { part } else { 0.99 * *ema + 0.01 * part };
                        }
                    }
                    if pg_only {
                        consumed += args.pg_batch as u64;
                        step += 1;
                        shared.learner_step.store(step, Ordering::Relaxed);
                    }
                }
            }

            // The deck net trains at a quarter cadence: its stream is only
            // 2 rows/game.
            if !pg_only && step.is_multiple_of(4) {
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
                    pg_ema,
                    args.pg.then_some((pg_cfg, pg_wide)),
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
        // End the prefetch worker: the flag covers its wait-for-data loop,
        // dropping the receiver covers a send blocked on a full channel.
        prefetch_stop.store(true, Ordering::Relaxed);
        drop(pg_rx);
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
                pg_ema,
                args.pg.then_some((pg_cfg, pg_wide)),
                pilot_trainer.as_ref(),
                &mut timing,
                &mut prev_interval,
            );
        }
        shared.learner_done.store(true, Ordering::Relaxed);
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
    // Two rates: `--steps` can outlast the actors, leaving games/elapsed
    // ~3x under the actors' own rate. The gap is the learner.
    let actor_s = shared.actor_ms.load(Ordering::Relaxed) as f64 / 1000.0;
    eprintln!(
        "done: {games} games ({:.1}/s), {} rows, {} stalls, {:.0}s",
        games as f64 / secs.max(0.001),
        shared.rows_pushed.load(Ordering::Relaxed),
        shared.stalls.load(Ordering::Relaxed),
        secs
    );
    if args.record_decisions {
        eprintln!(
            "decisions: {} trained-side rows, {} of them searched",
            shared.decisions_pushed.load(Ordering::Relaxed),
            shared.decisions_searched.load(Ordering::Relaxed)
        );
    }
    if actor_s > 0.0 {
        let share = 100.0 * actor_s / secs.max(0.001);
        eprintln!(
            "actors: {:.1} games/s over {:.1}s ({:.0}% of the run{})",
            games as f64 / actor_s.max(0.001),
            actor_s,
            share,
            if share < 95.0 { "; the rest is the learner outliving them" } else { "" },
        );
    }
}

/// Counters as of the previous checkpoint, so each stats line reports the
/// rate over that interval: a run whose throughput decays (window growth,
/// allocator churn, lock starvation) shows it here, not in the cumulative rate.
#[derive(Default)]
struct Interval {
    secs: f64,
    games: u64,
    rows: u64,
    consumed: u64,
    step: u64,
}
#[allow(clippy::too_many_arguments)]
/// Round 72's one-shot check on the sampling temperature, from the first PG
/// batch: the mean probability the behaviour gave its own pick. Below 0.5 the
/// actors aren't playing the pilot (round 57's caveat); above 0.9 every
/// near-tie ratio clips and the step learns nothing. Either way the
/// temperature is wrong for this net's scale, so the run aborts.
fn pg_temperature_sanity(pb: &crabomination_ml::PgBatch, temp: i32) {
    if pb.is_empty() {
        return;
    }
    let (mean, net_share) = pb.behaviour_summary();
    eprintln!(
        "pg sanity: mean pi_b(chosen) {mean:.3} over {} decisions at --sample-temp {temp} ({:.0}% net-scored)",
        pb.len(),
        100.0 * net_share
    );
    if !(0.5..=0.9).contains(&mean) {
        eprintln!(
            "ABORT: mean pi_b(chosen) {mean:.3} is outside [0.5, 0.9] — the sampling temperature is \
             wrong for this net's score scale (see ML_NOTES round 72)"
        );
        std::process::exit(2);
    }
}

// Seventeen arguments: the one place that writes a checkpoint needs all of the
// run's state, and a struct would be the same fields one indirection away (as
// with the server's wide helpers, `server/mod.rs`, `recommend.rs`).
/// Publishes `latest` / `deck-latest`, appends a `stats.jsonl` line, and
/// publishes `best` on a new holdout best. Returns whether the holdout metric
/// improved — `None` when none was scored, so `--stop-after-stale` counts only
/// checkpoints that actually said something.
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
    pg_ema: [f32; 7],
    pg: Option<(crabomination_ml::PgConfig, u64)>,
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
    let stalls_board = shared.stalls_board.load(Ordering::Relaxed);
    let stalls_stuck = shared.stalls_stuck.load(Ordering::Relaxed);
    let secs = start.elapsed().as_secs_f64();
    let [total, win, life, len, opp] = loss_ema;
    // Held-out scoring against both labels a row has. `loss_win` is the
    // training EMA against whatever target the learner fits (the λ-return at
    // λ < 1), so at λ = 0.7 (every champion-class run since round 27)
    // `val_win - loss_win` is not the overfit:
    //
    // * `*_raw` — MSE against the 0/1 game result; `val_raw` vs `train_raw`
    //   is the honest generalisation gap.
    // * `*_tgt` — MSE against the λ-return; `train_tgt` vs `train_raw` is the
    //   bootstrap divergence (they coincide at λ = 1). `train_tgt` collapsing
    //   while `val_raw` / `val_auc` stall means the net is fitting its own
    //   bootstrap.
    //
    // `val_auc` says whether the net knows anything; MSE can improve while
    // ranking does not.
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
    // The actors' own window. Equal to `secs` while they are still
    // playing; smaller once `--steps` outlives them, and the gap is what
    // separates simulator throughput from run throughput.
    let actor_s = shared.actor_ms.load(Ordering::Relaxed) as f64 / 1000.0;
    let actor_g = games as f64 / actor_s.max(1e-9);
    // Held-out policy agreement: how often the net's top candidate is the one
    // the pilot played, on decisions never trained on — a measure of choosing
    // rather than predicting (MSE and AUC score isolated positions; a search
    // consumes rankings).
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
    // The pilot's own agreement with the search on the same holdout. MCTS
    // rolls out on the pilot net while `val_policy` scores the training net
    // (round 34 couldn't separate the two), so this reference's distance from
    // 1.0 is search-vs-its-own-evaluator with no net-vs-net confound.
    let pilot_policy = match pilot {
        Some(t) if val_policy_n > 0 => {
            let v = shared.decisions_val.lock().unwrap();
            let refs: Vec<&crabomination_ml::DecisionRow> = v.iter().collect();
            t.policy_top1(&refs).unwrap_or(f32::NAN)
        }
        _ => f32::NAN,
    };
    // Round 72: the policy-gradient EMAs, and the same statistics on the
    // held-out decisions without an update (`Trainer::pg_eval`).
    let [pg_loss, pg_adv_mean, pg_adv_std, pg_entropy, pg_rho_mean, pg_clip_frac, pg_det_frac] =
        pg_ema;
    let pg_wide = pg.map_or(0, |(_, w)| w);
    // Capped at 512 decisions: the step pads to one batch (up to 8 states a
    // decision), and 4 000 at once is a single ~30 k-state forward that the
    // round-72 run's first checkpoint failed outright (val_pg_* read NaN).
    let (val_pg_adv, val_pg_entropy, val_pg_clip) = match pg {
        Some((cfg, _)) if val_policy_n > 0 => {
            let v = shared.decisions_val.lock().unwrap();
            let refs: Vec<&crabomination_ml::DecisionRow> = v.iter().take(512).collect();
            match trainer.pg_eval(&refs, cfg) {
                Ok(st) if st.n > 0 => (st.adv_mean, st.entropy, st.clip_frac),
                _ => (f32::NAN, f32::NAN, f32::NAN),
            }
        }
        _ => (f32::NAN, f32::NAN, f32::NAN),
    };
    let decisions_dropped = crabomination::server::decision_capture::dropped();
    let [t_sample, t_step, t_relabel, t_deck, t_sleep] = timing.take_ms();
    let line = format!(
        "{{\"step\":{step},\"loss_ema\":{total:.5},\"loss_win\":{win:.5},\"loss_life\":{life:.5},\"loss_len\":{len:.5},\"loss_opp\":{opp:.5},\"loss_deck\":{deck_loss:.5},\"val_n\":{val_n},\"val_win\":{val_win:.5},\"val_tgt\":{val_tgt:.5},\"val_logloss\":{val_ll:.5},\"val_auc\":{val_auc:.5},\"policy_top1\":{policy_top1:.4},\"policy_loss\":{policy_loss:.5},\"policy_sv\":{policy_sv:.5},\"val_policy\":{val_policy:.4},\"val_policy_chance\":{val_policy_chance:.4},\"pilot_policy\":{pilot_policy:.4},\"val_policy_n\":{val_policy_n},\"pg_loss\":{pg_loss:.5},\"pg_adv_mean\":{pg_adv_mean:.4},\"pg_adv_std\":{pg_adv_std:.4},\"pg_entropy\":{pg_entropy:.4},\"pg_rho_mean\":{pg_rho_mean:.4},\"pg_clip_frac\":{pg_clip_frac:.4},\"pg_det_frac\":{pg_det_frac:.4},\"pg_wide_skipped\":{pg_wide},\"decisions_dropped\":{decisions_dropped},\"val_pg_adv\":{val_pg_adv:.4},\"val_pg_entropy\":{val_pg_entropy:.4},\"val_pg_clip\":{val_pg_clip:.4},\"train_n\":{train_n},\"train_raw\":{train_raw:.5},\"train_tgt\":{train_tgt:.5},\"rows_consumed\":{consumed},\"rows\":{rows},\"games\":{games},\"games_mcts\":{games_mcts},\"stalls\":{stalls},\"stalls_capped\":{stalls_capped},\"stalls_board\":{stalls_board},\"stalls_stuck\":{stalls_stuck},\"elapsed_s\":{secs:.0},\"games_per_s\":{dg:.3},\"rows_per_s\":{dr:.1},\"consumed_per_s\":{dc:.1},\"steps_per_s\":{ds:.3},\"games_per_s_cum\":{cum_g:.3},\"actor_s\":{actor_s:.1},\"actor_games_per_s\":{actor_g:.3},\"rows_per_s_cum\":{cum_r:.1},\"t_sample_ms\":{t_sample},\"t_step_ms\":{t_step},\"t_relabel_ms\":{t_relabel},\"t_deck_ms\":{t_deck},\"t_sleep_ms\":{t_sleep}}}\n"
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stats_path)
        .expect("stats.jsonl");
    f.write_all(line.as_bytes()).expect("stats write");
    // Keep the best net by the holdout, not just the last: an overfitting run
    // ends on its worst net (round 11: AUC peaked at ~4–6 k steps, then fell
    // ~0.07 over 45 k more), and gates scored on `latest` were scoring a
    // memorised checkpoint. "Best" is AUC by default, `val_policy` under
    // `--best-metric policy` (round 34: the two can move in opposite ways).
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
        for o in s.objects() {
            self.objects += 1;
            for (i, v) in o.feats.iter().enumerate() {
                if *v != 0.0 {
                    self.obj_hits[i] += 1;
                    self.obj_sum[i] += v.abs() as f64;
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

/// How often each encoder feature is non-zero — in the rows the trainer fits
/// and in the leaves the search evaluates. A feature zero in 99.9 % of
/// positions can't move a gate (round 28f gated a block that was zero in
/// every training row). Cheap (no net, one self-play pass), and it reads the
/// encoder as currently ablated, so it also confirms a control arm blanks
/// what it claims to.
///
/// **train** is what the weights are fit to; **leaf** is what the attack and
/// cast searches evaluate (`server::leaf_capture`, as `--calibrate-leaves`).
/// `!` flags a feature common at the leaves and rare in training: live inputs
/// to weights that never got a gradient, the state round 40 found the whole
/// combat block in. Matching the columns isn't automatically good —
/// `--record-combat` closed that gap and cost 3 points of search strength —
/// so this measures the mismatch without prescribing a fix.
///
/// Object rates are per encoded object over every group (hand and library
/// included); global rates are per position.
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
        // Pin the tie-break jitter stream per game, or the census isn't
        // reproducible (two same-`--seed` runs disagreed by 12 positions / 515
        // objects, 2026-08-29: `play_recorded_game` leaves `bot::jitter_below`
        // on the unseeded thread RNG). Only the census is pinned; whether the
        // training actors should be is an open TODO.
        let game_seed = rng.random_range(0..u64::MAX);
        crabomination::server::bot::set_jitter_seed(Some(game_seed ^ 0xFEA7_5EED));
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            game_seed,
            50_000,
            vocab,
        );
        crabomination::server::bot::set_jitter_seed(None);
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

    // `!` only catches never-trained features. A trained block two or three
    // times denser at the leaves is the same phenomenon at lower contrast,
    // and shows which way the distributions differ (round 40's leaves were
    // uniformly "later in the turn" than its training rows).
    let mut ratios: Vec<(f64, String, f64, f64)> = (0..GLOBAL_FEATS)
        .map(|i| (format!("g{i}"), train.gl_rate(i), leaf.gl_rate(i)))
        .chain((0..OBJ_FEATS).map(|i| (format!("f{i}"), train.obj_rate(i), leaf.obj_rate(i))))
        .filter(|(_, t, l)| *l >= 5.0 && *t > 0.0 && l > t)
        .map(|(name, t, l)| (l / t, name, t, l))
        .collect();
    ratios.sort_by(|a, b| b.0.total_cmp(&a.0));
    if !ratios.is_empty() {
        // Name the denominator: "top 10 of 47" (concentrated skew) and "top
        // 10 of 10" (skew everywhere) are different diagnoses.
        println!(
            "\ntrained features the search meets most disproportionately \
             (leaf/train) — top {} of {}:",
            ratios.len().min(10),
            ratios.len(),
        );
        for (r, name, t, l) in ratios.iter().take(10) {
            println!("  {name:<5} {r:5.2}x   train {t:6.2}%  leaf {l:6.2}%");
        }
    }
}

/// Score the value net and the heuristic evaluation as *predictors of the
/// winner* on identical self-play positions: "does the net know more than
/// `eval_material`", which four gate rounds of "is the net bot stronger"
/// (42–45 % as a replacement, ~49 % blended) never answered. No better
/// log-loss means the net hasn't learned; better yet losing means the fault
/// is in integration, and the output histogram is the first place to look (a
/// saturated sigmoid hands the search a flat landscape).
///
/// The heuristic gets a fitted one-parameter logistic (`p = sigmoid(score /
/// t)`, `t` by scan), so it is scored as the best forecast it can support.
/// `--write-holdout PATH`: a frozen, seed-independent holdout. Plays
/// `--holdout-games` sealed games on a seed stream of its own (not
/// `--seed`'s, so every run and arm scores the same games), with the actors'
/// deck builder (`--actor-rated-decks` honoured) and the default pilot, and
/// keeps two random snapshots a game, both seats (at most 4 rows), so long
/// games and late plies stop dominating the metric. The actors' first-come
/// 20 k-row holdout was ~165 games, different per seed (2026-11-23 audit).
fn write_holdout(args: &Args, vocab: &Vocab, path: &std::path::Path) {
    const HOLDOUT_SEED: u64 = 0x401D_F0E5_7000_0001;
    let mut rng = StdRng::seed_from_u64(HOLDOUT_SEED ^ 0x5A17);
    let mut rows: Vec<TrainRow> = Vec::new();
    let mut games = 0usize;
    let mut n = 0u64;
    while games < args.holdout_games {
        n += 1;
        let salt = |k: u64| HOLDOUT_SEED ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k * 0x9E37_79B9);
        let pool_a = sealed_pool(salt(1));
        let pool_b = sealed_pool(salt(2));
        let build = |pool: &[crabomination::cube::CardFactory], seed: u64| {
            if args.actor_rated_decks && crabomination::card_ratings::rates_pool(pool) {
                crabomination::selfplay::heuristic_sealed_build_rated(pool, seed)
            } else {
                heuristic_sealed_build(pool, seed)
            }
        };
        let deck_a = build(&pool_a, salt(3));
        let deck_b = build(&pool_b, salt(4));
        let template = sealed_game_template(&deck_a, &deck_b);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            rng.random_range(0..u64::MAX),
            4_000,
            vocab,
        );
        let pairs = rec.rows.len() / 2;
        if pairs == 0 {
            continue;
        }
        let mut picks: Vec<usize> = (0..pairs).collect();
        picks.shuffle(&mut rng);
        for &i in picks.iter().take(2) {
            rows.push(rec.rows[2 * i].clone());
            rows.push(rec.rows[2 * i + 1].clone());
        }
        games += 1;
        if games % 250 == 0 {
            eprintln!("holdout: {games} games, {} rows", rows.len());
        }
    }
    std::fs::write(path, crabomination_nn::write_shard(&rows)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    println!("wrote {} rows from {games} games to {}", rows.len(), path.display());
}

/// `--score-holdout PATH --nets a,b,..`: each net's AUC, log-loss, mean
/// prediction (against the base rate) and early-ply (< 12) AUC on one frozen
/// holdout — the paired comparison the per-seed first-come holdout could not
/// give. Each net's architecture comes from its own file.
fn score_holdout(_args: &Args, vocab: &Vocab, path: &std::path::Path) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let rows = crabomination_nn::read_shard(&bytes).unwrap_or_else(|| panic!("{}: not a shard", path.display()));
    let refs: Vec<&TrainRow> = rows.iter().collect();
    let base = rows.iter().map(|r| r.win).sum::<f32>() / rows.len().max(1) as f32;
    println!("holdout {}: {} rows, base rate {base:.3}", path.display(), rows.len());
    println!("{:<60} {:>7} {:>8} {:>7} {:>9}", "net", "AUC", "logloss", "mean p", "AUC p<12");
    for net in &_args.nets {
        let Some(cfg) = file_net_config(net, vocab.size()) else {
            println!("{:<60} unreadable", net.display());
            continue;
        };
        let mut t = Trainer::new(&cfg, 0.0).expect("trainer");
        if let Err(e) = t.load(net) {
            println!("{:<60} load failed: {e}", net.display());
            continue;
        }
        let p = t.predict_win_batch(&refs, 512).expect("forward");
        let a = auc(p.iter().zip(&rows).map(|(&p, r)| (p, r.win)));
        let ll = log_loss(p.iter().zip(&rows).map(|(&p, r)| (p, r.win)));
        let mean = p.iter().map(|&x| x as f64).sum::<f64>() / p.len().max(1) as f64;
        let early = auc(p.iter().zip(&rows).filter(|(_, r)| r.ply < 12).map(|(&p, r)| (p, r.win)));
        println!("{:<60} {a:>7.4} {ll:>8.4} {mean:>7.3} {early:>9.4}", net.display().to_string());
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
    let mut by_card = CardResiduals::default();
    // Own hand size (8 = 8+) → (positions, Σ residual, Σ prediction).
    let mut by_hand = [(0u64, 0f64, 0f64); 9];
    // Game length in turns (bucketed) → (positions, Σ prediction, Σ result,
    // games): whether the bias lives in the long games the actors drop.
    const LEN_EDGES: [u32; 5] = [10, 14, 18, 24, 32];
    let mut by_len = [(0u64, 0f64, 0f64, 0u64); 6];
    // Rows come in seat pairs (one snapshot, seat 0 then seat 1), so each
    // position can also be scored from both sides: (p0, p1, y0).
    let mut pairs: Vec<(f32, f32, f32)> = Vec::new();
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
            args.calibrate_cap,
            vocab,
        );
        if rec.rows.is_empty() {
            continue;
        }
        let refs: Vec<&TrainRow> = rec.rows.iter().collect();
        let preds = trainer.predict_win_batch(&refs, 512).expect("forward");
        by_card.add_game(&rec.rows, &preds);
        {
            let b = LEN_EDGES.iter().position(|&e| rec.turns < e).unwrap_or(LEN_EDGES.len());
            let e = &mut by_len[b];
            e.0 += preds.len() as u64;
            e.1 += preds.iter().map(|&p| p as f64).sum::<f64>();
            e.2 += rec.rows.iter().map(|r| r.win as f64).sum::<f64>();
            e.3 += 1;
        }
        for ([r0, r1], [p0, p1]) in rec.rows.as_chunks::<2>().0.iter().zip(preds.as_chunks::<2>().0) {
            debug_assert_eq!(r0.traj & 1, 0);
            debug_assert_eq!(r1.traj & 1, 1);
            pairs.push((*p0, *p1, r0.win));
        }
        for (row, &p) in rec.rows.iter().zip(&preds) {
            let h = row.state.group_len(crabomination_nn::G_HAND_SELF).min(8);
            let e = &mut by_hand[h];
            e.0 += 1;
            e.1 += (row.win - p) as f64;
            e.2 += p as f64;
        }
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
    let mean_p = obs.iter().map(|o| o.0 as f64).sum::<f64>() / obs.len() as f64;
    println!("  mean prediction {mean_p:.4} against base rate {base:.4}");
    reliability(&obs.iter().map(|o| (o.0, o.2)).collect::<Vec<_>>());
    println!("  by own hand size: positions, mean prediction, mean residual (result − net)");
    for (h, &(n, r, p)) in by_hand.iter().enumerate().filter(|(_, e)| e.0 > 0) {
        println!("    {h}{}  {n:>7}  {:.3}  {:+.3}", if h == 8 { "+" } else { " " }, p / n as f64, r / n as f64);
    }
    // The two seats' readings of one position, combined: the mean of p and
    // 1 − p' (what a symmetric head would say), and the ratio p / (p + p').
    // Each scored on both seats' rows, so the numbers compare with the net's.
    {
        let both = |f: &dyn Fn(f32, f32) -> f32| -> Vec<(f32, f32)> {
            pairs.iter().flat_map(|&(a, b, y)| [(f(a, b), y), (f(b, a), 1.0 - y)]).collect()
        };
        let sum = pairs.iter().map(|&(a, b, _)| (a + b) as f64).sum::<f64>() / pairs.len() as f64;
        println!("  the two seats' readings of one position: mean p + p' = {sum:.4} ({} positions)", pairs.len());
        for (name, v) in [
            ("as read     ", both(&|a, _| a)),
            ("(p + 1 - p')/2", both(&|a, b| (a + 1.0 - b) / 2.0)),
            ("p / (p + p')", both(&|a, b| if a + b > 0.0 { a / (a + b) } else { 0.5 })),
        ] {
            let ll = log_loss(v.iter().copied());
            let br = brier(v.iter().copied());
            let au = auc(v.iter().copied());
            println!("    {name}  log-loss {ll:.4}  Brier {br:.4}  AUC {au:.4}");
        }
    }
    println!("  by game length (turns): games, positions, mean prediction, mean result");
    for (b, &(n, p, y, g)) in by_len.iter().enumerate().filter(|(_, e)| e.0 > 0) {
        let lo = if b == 0 { 0 } else { LEN_EDGES[b - 1] };
        let hi = LEN_EDGES.get(b).map_or("+".to_string(), |e| format!("-{}", e - 1));
        println!("    {lo:>2}{hi:<4}  {g:>5}  {n:>7}  {:.3}  {:.3}", p / n as f64, y / n as f64);
    }
    by_card.report(vocab, base as f64 - mean_p);
}

/// `--calibrate`'s per-card blind-spot table: per card, the mean of (result −
/// net prediction) over positions where it sits in a zone group. A
/// calibrated net leaves every mean near zero; an undervalued card has a
/// positive mean where its controller holds it.
///
/// The unit is the (game, seat), contributing its mean residual once: a
/// seat's snapshots share one result, so a per-position interval would be
/// several times too narrow.
#[derive(Default)]
struct CardResiduals {
    /// (group, vocab index) → (game seats, Σ seat mean, Σ seat mean², positions).
    cells: std::collections::HashMap<(usize, u16), (u64, f64, f64, u64)>,
    /// The open seat's per-cell (Σ residual, positions), flushed by `end_seat`.
    open: std::collections::HashMap<(usize, u16), (f64, u64)>,
}

impl CardResiduals {
    const GROUPS: [(usize, &'static str); 3] = [
        (crabomination_nn::G_BF_SELF, "own battlefield"),
        (crabomination_nn::G_BF_OPP, "opponent's battlefield"),
        (crabomination_nn::G_HAND_SELF, "own hand"),
    ];

    /// Record one game's rows (both seats, split on `traj`).
    fn add_game(&mut self, rows: &[TrainRow], preds: &[f32]) {
        let mut trajs: Vec<u32> = rows.iter().map(|r| r.traj).collect();
        trajs.sort_unstable();
        trajs.dedup();
        for t in trajs {
            for (row, &p) in rows.iter().zip(preds).filter(|(r, _)| r.traj == t) {
                let r = (row.win - p) as f64;
                for (g, _) in Self::GROUPS {
                    let mut seen: Vec<u16> = row.state.group(g).iter().map(|o| o.card).filter(|&c| c != 0).collect();
                    seen.sort_unstable();
                    seen.dedup();
                    for c in seen {
                        let e = self.open.entry((g, c)).or_default();
                        e.0 += r;
                        e.1 += 1;
                    }
                }
            }
            for (k, (s, n)) in self.open.drain() {
                let m = s / n as f64;
                let e = self.cells.entry(k).or_default();
                e.0 += 1;
                e.1 += m;
                e.2 += m * m;
                e.3 += n;
            }
        }
    }

    /// `bias` is the residual every position carries (base rate − mean
    /// prediction); cards are ranked on their residual net of it.
    fn report(&self, vocab: &Vocab, bias: f64) {
        let names: std::collections::HashMap<u16, &str> = crabomination::server::vocab_snapshot::VOCAB_SNAPSHOT
            .iter()
            .map(|n| (vocab.index_of(n), *n))
            .collect();
        let min_n: u64 = std::env::var("CRAB_CALIB_MIN_N").ok().and_then(|v| v.parse().ok()).unwrap_or(100);
        for (g, label) in Self::GROUPS {
            // (z, mean, game seats, name): z is the mean over its
            // seat-level standard error, ranked by |z| so a common card's
            // small bias and a rare card's large one read on one scale.
            let mut rows: Vec<(f64, f64, u64, &str)> = self
                .cells
                .iter()
                .filter(|((gg, _), v)| *gg == g && v.0 >= min_n)
                .map(|((_, c), &(n, s, s2, _))| {
                    let m = s / n as f64;
                    let var = ((s2 - n as f64 * m * m) / (n as f64 - 1.0)).max(1e-12);
                    ((m - bias) / (var / n as f64).sqrt(), m - bias, n, names.get(c).copied().unwrap_or("?"))
                })
                .collect();
            rows.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()));
            println!("  per-card residual (result − net − {bias:+.3}), {label}, |z| order, n ≥ {min_n} game seats:");
            for (z, m, n, name) in rows.iter().take(30) {
                println!("    {m:+.3}  z {z:+6.1}  seats {n:>6}  {name}");
            }
        }
    }
}

/// Score net and heuristic on the simulated leaves the search consumes,
/// against the snapshot positions of the same games. A leaf's label is the
/// real game's winner from the evaluating seat's side; the heuristic's
/// temperature is fitted per distribution.
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

/// Break the calibration observations down by position-in-game. Late
/// positions are mostly decided, and any evaluation that counts power and
/// life gets them right; early ones are contested, and the only ones the
/// search can still change. A net better late and worse early posts a better
/// aggregate AUC and plays worse — a candidate explanation for "better
/// predictor, keeps losing gates" that no aggregate metric shows.
///
/// Both seats are snapshotted at the same `ply`, so every stratum's base
/// rate is 0.5. The heuristic keeps its global temperature: refitting per
/// bucket would give it a free parameter it doesn't have in play.
/// Reliability bins (mean prediction vs observed rate per tenth) and the
/// Platt fit `sigmoid(a·logit(p) + b)` that minimises log-loss: `a` < 1 says
/// the net is over-confident, > 1 under-confident; `b` is its level offset.
/// The search mixes the leaf with exact 0 / 1 terminal rewards, so either
/// error shifts every arm whose rollouts sometimes end the game (round 88, B6).
fn reliability(obs: &[(f32, f32)]) {
    let mut bins = [(0u64, 0f64, 0f64); 10];
    for &(p, y) in obs {
        let b = &mut bins[((p * 10.0) as usize).min(9)];
        b.0 += 1;
        b.1 += p as f64;
        b.2 += y as f64;
    }
    println!("  reliability: bin, n, mean prediction, observed rate");
    for (i, &(n, sp, sy)) in bins.iter().enumerate().filter(|(_, b)| b.0 > 0) {
        println!("    {:.1}-{:.1}  {n:>7}  {:.3}  {:.3}", i as f32 / 10.0, (i + 1) as f32 / 10.0, sp / n as f64, sy / n as f64);
    }
    let logit = |p: f32| {
        let p = p.clamp(1e-4, 1.0 - 1e-4);
        (p / (1.0 - p)).ln()
    };
    let z: Vec<(f32, f32)> = obs.iter().map(|&(p, y)| (logit(p), y)).collect();
    let (mut best_a, mut best_b, mut best_ll) = (1.0f32, 0.0f32, f32::INFINITY);
    for ai in 0..41 {
        let a = 0.5 + ai as f32 * 0.05;
        for bi in 0..41 {
            let b = -0.5 + bi as f32 * 0.025;
            let ll = log_loss(z.iter().map(|&(x, y)| (1.0 / (1.0 + (-(a * x + b)).exp()), y)));
            if ll < best_ll {
                (best_a, best_b, best_ll) = (a, b, ll);
            }
        }
    }
    let raw = log_loss(obs.iter().copied());
    println!("  Platt fit: a {best_a:.2}  b {best_b:+.3}  log-loss {raw:.4} -> {best_ll:.4}");
}

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
/// AUC is a global ranking statistic, and the attention net wins it against
/// `eval_material` (0.798 vs 0.760 seed 43; 0.761 vs 0.747 seed 97) while
/// losing the gate (44.8 % as a replacement, 48.8 % blended). The search
/// never asks the global question: it compares near-identical boards (one
/// attack or block apart) and takes the argmax.
///
/// Adjacent snapshots of one trajectory are the cheapest proxy: one turn
/// apart, same seat, similar boards. Ground truth is the trajectory's
/// direction (a winner's positions tend toward 1) — noisy per pair, but it
/// is the value target's own assumption, applied identically to both
/// evaluators. Mean separation is reported too: ordering pairs right by a
/// hair is useless to a search whose candidates differ by less than its noise.
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
                // Heuristic scores are unbounded ints; the 1/1000 is undone
                // when printed (eval units), never compared to the net's.
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
    // Correct-rate among pairs the evaluator separates, with the tie rate
    // beside it: scoring ties as wrong would punish honestly declining to
    // distinguish equal positions, and folding them in hides the tie rate.
    // Separations are reported on each evaluator's own scale, never compared
    // (integer score vs probability).
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
