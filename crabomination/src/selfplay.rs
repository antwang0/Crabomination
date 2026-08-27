//! Self-play game generation for the ML training loop.
//!
//! The `selfplay_train` binary (in `crabomination_ml`) owns the threads,
//! the sliding window, and the learner; this module owns everything that
//! needs the engine — building a sealed match and playing it out while
//! recording encoded snapshots. Keeping the game loop here means the bin
//! never touches `GameState` internals, and the snapshot cadence is
//! defined once, next to the encoder it feeds.
//!
//! Determinism: pools, builds, and shuffles are derived from the seeds
//! the caller passes, matching the ladder/recommender convention. The
//! games themselves are not replayable — [`HeuristicBot`]'s candidate jitter
//! draws from the thread RNG by design — which the training loop doesn't
//! need; the jitter is the exploration noise that diversifies the data.

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crabomination_nn::TrainRow;

use crate::cube::CardFactory;
use crate::draft::SosPacks;
use crate::game::GameState;
use crate::recommend::{
    STALE_ROUNDS, SimConfig, StopReason, build_match_template, build_random_deck,
};
use crate::server::bot::{Bot, EvalWeights, HeuristicBot};
use crate::server::encode::{Vocab, encode_state};

/// Whether the snapshot cadence includes combat steps.
///
/// Default off, which is what every run through round 39 recorded: a
/// snapshot at each new turn, at post-combat main, and at end step. The
/// feature census (`selfplay_train --feature-census`) showed what that
/// costs — across 29 696 recorded positions the entire round-28 combat
/// block (globals 36..=40, object feats 37..=39), the coarse combat
/// one-hot (global 11), the attacking count (global 19) and the
/// attacking/blocking flags (object feats 10, 28, 29) are non-zero in
/// **exactly zero** rows. Combat is over by post-combat main and hasn't
/// started at end step, so no training row has ever been a combat row.
///
/// Those features are live at inference time — the attack and block
/// sims evaluate mid-combat positions — so the columns the search feeds
/// them into never received a gradient. Turning this on adds a snapshot
/// on entry to declare-blockers (attacks declared, blocks pending: the
/// attack sim's leaf) and to end-of-combat (blocks declared and damage
/// marked: the block sim's leaf), which is the only way those features
/// can ever be trained.
///
/// Process-global for the reason the encoder ablation is: the recorder
/// runs on every actor thread and the flag is set once at startup.
static COMBAT_SNAPSHOTS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Set the combat-snapshot cadence. See [`COMBAT_SNAPSHOTS`].
pub fn set_combat_snapshots(on: bool) {
    COMBAT_SNAPSHOTS.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// True when the recorder is capturing combat steps.
pub fn combat_snapshots() -> bool {
    COMBAT_SNAPSHOTS.load(std::sync::atomic::Ordering::Relaxed)
}

/// A 6-pack SOS sealed pool, fully determined by `seed`.
pub fn sealed_pool(seed: u64) -> Vec<CardFactory> {
    sealed_pool_packs(seed, SEALED_PACKS)
}

/// The packs a standard sealed pool is opened from (CR 100.2b).
pub const SEALED_PACKS: usize = 6;

/// [`sealed_pool`] with the pack count chosen — a deeper pool builds a
/// stronger deck, which is how a bot opponent is given a handicap edge
/// without touching the builder.
pub fn sealed_pool_packs(seed: u64, packs: usize) -> Vec<CardFactory> {
    let mut rng = StdRng::seed_from_u64(seed);
    // Both the pool and its bucketing are process constants; a training actor
    // opens two pools a game and rebuilt them each time (PERF (-66)).
    let rolls = SosPacks::sos_default();
    (0..packs.max(1)).flat_map(|_| rolls.roll(&mut rng)).collect()
}

/// The bootstrap deckbuilder: the recommender's noisy-greedy sealed build
/// (color-pair shapes, jittered picks). This is what generates training
/// decks until the Phase C build net earns the job.
pub fn heuristic_sealed_build(pool: &[CardFactory], seed: u64) -> Vec<CardFactory> {
    heuristic_sealed_build_with(pool, seed, true)
}

/// [`heuristic_sealed_build`] with the builder generation selectable:
/// `false` is the pre-`builder_v2` scorer/splash/mana model, kept so the
/// repaired builder can be raced against what it replaced.
pub fn heuristic_sealed_build_with(
    pool: &[CardFactory],
    seed: u64,
    builder_v2: bool,
) -> Vec<CardFactory> {
    let cfg = SimConfig { builder_v2, ..SimConfig::default() };
    let mut rng = StdRng::seed_from_u64(seed);
    build_random_deck(pool, &cfg, &mut rng).cards
}

/// One noisy build under the v3 shape ranker (quality- and curve-aware
/// [`crate::recommend::static_build_score_v3`]); the v2 sample above is
/// its measurement control, raced by `selfplay_train --gate-builder-v3`.
pub fn heuristic_sealed_build_v3(pool: &[CardFactory], seed: u64) -> Vec<CardFactory> {
    let cfg = SimConfig { builder_v3: true, ..SimConfig::default() };
    let mut rng = StdRng::seed_from_u64(seed);
    build_random_deck(pool, &cfg, &mut rng).cards
}

/// A random sealed opponent: a fresh 6-pack pool, built by the same
/// builder family the training loop and recommender use — but as its
/// best-of-16 pick, not a single noisy sample (see
/// [`random_sealed_opponent_packs`] for why). Returns the 40-card deck
/// and a label naming the seed, so a match can be reproduced
/// ("Sealed #12345").
pub fn random_sealed_opponent(seed: u64) -> (Vec<CardFactory>, String) {
    random_sealed_opponent_packs(seed, SEALED_PACKS)
}

/// [`random_sealed_opponent`] opening `packs` boosters instead of the
/// standard six. The extra cards go through the same builder, so the
/// opponent is simply choosing 40 from a deeper pool — a smooth
/// difficulty dial rather than a different kind of deck. The label names
/// the handicap so a match is still reproducible from what it says.
///
/// The deck is [`best_build_v3`] — best-of-16 noisy builds under the
/// quality/curve judge — not the single sample the training field uses.
/// A field member is deliberately mediocre (a room of humans); a named
/// opponent sitting across from *you* should be that room's best
/// builder, and one sample handed the player a free handicap on top of
/// whatever `packs` says. Still deterministic in `seed`.
pub fn random_sealed_opponent_packs(seed: u64, packs: usize) -> (Vec<CardFactory>, String) {
    let packs = packs.max(1);
    let pool = sealed_pool_packs(seed, packs);
    let deck = best_build_v3(&pool, 16, seed ^ 0x005E_A1ED);
    let label = if packs == SEALED_PACKS {
        format!("Sealed #{seed}")
    } else {
        format!("Sealed #{seed} ({packs} packs)")
    };
    (deck, label)
}

/// Unshuffled two-seat template with both libraries loaded — clone per
/// game.
pub fn sealed_game_template(seat0: &[CardFactory], seat1: &[CardFactory]) -> GameState {
    build_match_template(seat0, seat1)
}

/// `n` distinct noisy-greedy builds of the same pool — the candidate set
/// a build judge picks from. Deterministic in `seed`.
pub fn build_candidates(pool: &[CardFactory], n: usize, seed: u64) -> Vec<Vec<CardFactory>> {
    build_candidates_cfg(pool, n, seed, &SimConfig::default())
}

/// [`build_candidates`] under an explicit config (the v3 candidate set
/// differs from the default in which shapes the softmax favors).
pub fn build_candidates_cfg(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    cfg: &SimConfig,
) -> Vec<Vec<CardFactory>> {
    // The shape lattice is deterministic in `(pool, cfg)` — see
    // `build_random_deck_from` — so enumerate it once for all `n`
    // candidates instead of per candidate. At `best_build_by`'s n = 32
    // (what `selfplay_train --use-deck-best` runs per side per game) that
    // is 32 x ~26 `build_shape` calls replaced by ~26 + 32. The pool's
    // `PoolScores` is invariant across both loops for the same reason.
    let scores = crate::recommend::PoolScores::new(pool, cfg.builder_v2);
    let shapes = crate::recommend::enumerate_shapes(&scores, cfg);
    let out: Vec<Vec<CardFactory>> = (0..n as u64)
        .map(|i| {
            let mut rng =
                StdRng::seed_from_u64(seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7));
            crate::recommend::build_random_deck_from(&shapes, &scores, cfg, &mut rng).cards
        })
        .collect();
    // The hoist is sound only while `enumerate_candidates` is a pure
    // function of `(pool, cfg)`. Re-derived after the loop and compared, in
    // debug builds only: a later change that gives it hidden state — or that
    // lets a candidate build perturb it — then fails the suite
    // (`best_build_v3_is_the_argmax_of_its_candidates` runs this at n = 16)
    // instead of silently changing every judged build in training.
    debug_assert!(
        {
            let again = crate::recommend::enumerate_candidates(pool, cfg);
            again.len() == shapes.len()
                && again.iter().zip(&shapes).all(|(a, b)| {
                    a.static_score == b.static_score
                        && a.main.len() == b.main.len()
                        && a.main.iter().zip(&b.main).all(|(x, y)| *x as usize == *y as usize)
                })
        },
        "enumerate_candidates is no longer a pure function of (pool, cfg); \
         build_candidates_cfg hoists it out of the candidate loop",
    );
    out
}

/// Best of `n` noisy v3 builds under the v3 static judge — the builder's
/// best effort at a pool without simulating a game. Deterministic in
/// `seed`. This is the client's sealed-opponent recipe: the sampled
/// builds are a field of humans, this is the one who kept the best of
/// their sixteen tries.
pub fn best_build_v3(pool: &[CardFactory], n: usize, seed: u64) -> Vec<CardFactory> {
    let cfg = SimConfig { builder_v3: true, ..SimConfig::default() };
    build_candidates_cfg(pool, n, seed, &cfg)
        .into_iter()
        .max_by_key(|d| static_deck_score_v3(d))
        .expect("n > 0 builds")
}

/// Best of `n` candidate builds under an arbitrary judge (higher is
/// better). The judge is a closure so the engine stays ignorant of what
/// scores a deck — the build net, the static score, or anything else.
pub fn best_build_by<F: FnMut(&[CardFactory]) -> f64>(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    mut judge: F,
) -> Vec<CardFactory> {
    build_candidates(pool, n, seed)
        .into_iter()
        .map(|d| (judge(&d), d))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, d)| d)
        .expect("n > 0 builds")
}

/// Greedy hill-climb over single spell swaps, under an arbitrary judge.
///
/// [`best_build_by`] picks the best of N *random* candidates — it never
/// searches. This walks from a starting build: for each non-land slot,
/// try every distinct non-land pool card not already used, keep the
/// first strict improvement, repeat up to `max_passes` passes or until a
/// full pass finds nothing. A deck-net judge scores a 40-card list in
/// microseconds, so the few thousand evaluations a climb costs are free
/// next to a single simulated game.
///
/// Lands are never touched: the mana base came from the builder's
/// colour/count model and the judge has no way to know a swapped basic
/// broke it. Deterministic — candidates are visited in name order — so
/// the same pool and start always climb to the same deck.
pub fn hill_climb_build_by<F: FnMut(&[CardFactory]) -> f64>(
    pool: &[CardFactory],
    mut deck: Vec<CardFactory>,
    max_passes: usize,
    mut judge: F,
) -> Vec<CardFactory> {
    use std::collections::BTreeMap;
    // Pool multiset minus the current deck, spells only, one
    // representative factory per name. Kept incrementally correct as
    // swaps are accepted within a pass.
    let mut avail: BTreeMap<&'static str, (CardFactory, i32)> = BTreeMap::new();
    for f in pool {
        let d = crate::cube::card_def(*f);
        if d.is_land() {
            continue;
        }
        avail.entry(d.name).or_insert((*f, 0)).1 += 1;
    }
    for f in &deck {
        if let Some(e) = avail.get_mut(crate::cube::card_def(*f).name) {
            e.1 -= 1;
        }
    }
    let mut best = judge(&deck);
    for _ in 0..max_passes {
        let mut improved = false;
        for i in 0..deck.len() {
            if deck[i]().is_land() {
                continue;
            }
            let old = deck[i];
            let old_name = old().name;
            let cands: Vec<CardFactory> = avail
                .iter()
                .filter(|(name, (_, n))| *n > 0 && **name != old_name)
                .map(|(_, (f, _))| *f)
                .collect();
            let mut chosen = None;
            for c in cands {
                deck[i] = c;
                let score = judge(&deck);
                if score > best {
                    best = score;
                    chosen = Some(c);
                    break;
                }
            }
            match chosen {
                Some(c) => {
                    improved = true;
                    avail.get_mut(c().name).expect("came from avail").1 -= 1;
                    avail.entry(old_name).or_insert((old, 0)).1 += 1;
                }
                None => deck[i] = old,
            }
        }
        if !improved {
            break;
        }
    }
    deck
}

/// `swaps` random single-spell substitutions from the pool, for
/// distillation coverage: the deck net must be trained on the *space
/// search visits*, not just builder outputs, or search exploits its
/// extrapolation (the round-15 lesson). Deterministic in `seed`.
pub fn mutate_build(
    pool: &[CardFactory],
    mut deck: Vec<CardFactory>,
    swaps: usize,
    seed: u64,
) -> Vec<CardFactory> {
    use std::collections::BTreeMap;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut avail: BTreeMap<&'static str, (CardFactory, i32)> = BTreeMap::new();
    for f in pool {
        let d = crate::cube::card_def(*f);
        if !d.is_land() {
            avail.entry(d.name).or_insert((*f, 0)).1 += 1;
        }
    }
    for f in &deck {
        if let Some(e) = avail.get_mut(crate::cube::card_def(*f).name) {
            e.1 -= 1;
        }
    }
    let spell_slots: Vec<usize> =
        (0..deck.len()).filter(|&i| !deck[i]().is_land()).collect();
    for _ in 0..swaps {
        let free: Vec<CardFactory> =
            avail.values().filter(|(_, n)| *n > 0).map(|(f, _)| *f).collect();
        if free.is_empty() || spell_slots.is_empty() {
            break;
        }
        let slot = spell_slots[rng.random_range(0..spell_slots.len())];
        let inc = free[rng.random_range(0..free.len())];
        let out = deck[slot];
        deck[slot] = inc;
        avail.get_mut(inc().name).expect("from avail").1 -= 1;
        avail.entry(out().name).or_insert((out, 0)).1 += 1;
    }
    deck
}

/// The heuristic builder's own opinion of a finished deck, exposed so a
/// gate can pit "static-score judge" against a learned judge over the
/// *same* candidate set — the comparison that isolates the judge.
/// (`static_build_score` scores the spell picks; lands are filtered out
/// and no shortfall applies to a completed 40-card build.)
pub fn static_deck_score(deck: &[CardFactory]) -> i32 {
    let spells: Vec<CardFactory> = deck.iter().copied().filter(|f| !crate::cube::card_def(*f).is_land()).collect();
    crate::recommend::static_build_score(&spells, spells.len())
}

/// [`static_deck_score`] under the v3 ranker (card quality + curve
/// shape). Kept separate rather than replacing it: the v2 score is the
/// pinned control judge in recorded gates, and a judge that drifts
/// silently invalidates the comparisons built on it.
pub fn static_deck_score_v3(deck: &[CardFactory]) -> i32 {
    let spells: Vec<CardFactory> = deck.iter().copied().filter(|f| !crate::cube::card_def(*f).is_land()).collect();
    crate::recommend::static_build_score_v3(&spells, spells.len())
}

/// One finished self-play game's worth of labelled rows.
pub struct RecordedGame {
    pub rows: Vec<TrainRow>,
    /// The heuristic evaluation of each row's position, from that row's
    /// seat — parallel to `rows` and in the same order.
    ///
    /// Not part of the shard format: this exists so the calibration
    /// diagnostic can score the net and the heuristic as *predictors* on
    /// identical positions, which is the cheap question the expensive
    /// gate rounds were standing in for.
    pub heur: Vec<i32>,
    /// Winning seat; `None` for a stall/draw (rows are empty then — an
    /// unlabelled position teaches nothing).
    pub winner: Option<usize>,
    pub turns: u32,
    /// Which of the loop's three exits ended this game. The training
    /// actor counts a rowless game as a "stall"; this says whether it ran
    /// out of action budget, got stuck with no bot able to move, or drew.
    pub stop: StopReason,
}

/// Upper bound on the randomised opening window, in actions. Each game
/// draws its own length in `0..=EXPLORE_PLIES`, so a share of games are
/// pure policy play and the distribution is widened rather than shifted.
const EXPLORE_PLIES: usize = 12;

/// Play one bot game from `template`, snapshotting along the way from both
/// seats' perspectives, and stamp the outcome labels at the end.
///
/// Snapshot cadence: at the top of every turn, on entering the postcombat
/// main, and on entering the end step (consecutive duplicates skipped).
/// The mid-turn points exist because of a measured failure, not taste:
/// nets trained on turn boundaries alone gated at 43.6 % / 42.3 % across a
/// 4× data jump — `eval_material` consumes post-action and post-combat
/// positions, which turn-boundary training never shows the net. Both
/// seats' views are always pushed together (two rows per point, one
/// eventual winner and one loser), keeping the win labels balanced by
/// construction.
pub fn play_recorded_game(
    template: &GameState,
    weights: [EvalWeights; 2],
    seed: u64,
    max_actions: usize,
    vocab: &Vocab,
) -> RecordedGame {
    play_recorded_game_mcts(template, weights, seed, max_actions, vocab, 0, false)
}

/// [`play_recorded_game`] with the seats piloted by net-evaluated MCTS
/// (`mcts_iterations` > 0, 3-turn horizon — the round-26 adopted shape)
/// instead of the scored bot. This is the training side of search
/// amplification: labels from games played ~3–5 points stronger than the
/// plain net pilot. `weights` still parameterizes the reward and the
/// exploration openings.
pub fn play_recorded_game_mcts(
    template: &GameState,
    weights: [EvalWeights; 2],
    seed: u64,
    max_actions: usize,
    vocab: &Vocab,
    mcts_iterations: u32,
    mcts_gumbel: bool,
) -> RecordedGame {
    let mut g = template.clone();
    let mut rng = StdRng::seed_from_u64(seed);
    for seat in 0..2 {
        g.players[seat].library.shuffle(&mut rng);
    }
    // The in-game rolls (mulligan reshuffles, random discards) key off the
    // same seed, so a training game that crashes or produces a suspect row
    // can be replayed exactly. Derived rather than drawn, so the explore
    // draw below keeps its old position in the stream.
    g.rng.reseed(seed ^ 0x5EED_600D_C0DE_1234);
    g.start_mulligan_phase();
    // Opening-move exploration. Both seats otherwise play the same
    // deterministic policy, so the net only ever sees the narrow band of
    // positions that policy reaches and learns to evaluate *those* — the
    // classic self-play distribution collapse. Randomising the first few
    // decisions of each game widens the state distribution for free; it
    // is confined to the opening so the *outcome* still reflects
    // competent play and the win labels stay meaningful.
    let explore_plies = rng.random_range(0..=EXPLORE_PLIES);
    let mut bots: Vec<Box<dyn Bot>> = weights
        .into_iter()
        .map(|w| -> Box<dyn Bot> {
            if mcts_iterations > 0 {
                Box::new(crate::server::MctsBot::new(crate::server::MctsConfig {
                    iterations: mcts_iterations,
                    horizon_turns: 3,
                    weights: w,
                    gumbel: mcts_gumbel,
                    ..crate::server::MctsConfig::default()
                }))
            } else {
                Box::new(HeuristicBot::with_weights(w))
            }
        })
        .collect();
    let mut explorers: Vec<Box<dyn Bot>> =
        (0..2).map(|_| Box::new(HeuristicBot::uniform_baseline()) as Box<dyn Bot>).collect();

    // (turn, seat, encoded state) — labelled after the game decides.
    let mut snaps = Vec::new();
    let mut heur: Vec<i32> = Vec::new();
    // Raw per-seat stats at each snapshot, parallel to `snaps` — the aux
    // targets are deltas between a seat's consecutive snapshots, so the
    // raw values have to be captured live and diffed at labelling time.
    let mut raw: Vec<[f32; 4]> = Vec::new();
    // The opponent's held card names per snapshot, parallel to `snaps` —
    // the belief head's target (round 39). Recorded here because only
    // the recorder may look; the encoder never carries it.
    let mut opp_hands: Vec<Vec<u16>> = Vec::new();
    let mut last_turn = (0u32, usize::MAX);
    let mut last_step = crate::game::TurnStep::Untap;
    let mut last_pair: Option<[crabomination_nn::EncodedState; 2]> = None;
    let (mut actions, mut stale) = (0usize, 0usize);
    while !g.is_game_over() && actions < max_actions && stale < STALE_ROUNDS {
        let new_turn = (g.turn_number, g.active_player_idx) != last_turn;
        // The four shapes a combat passes through, each the leaf of a
        // decision the bot actually makes: declare-attackers (nothing
        // declared — the attack decision's own position),
        // declare-blockers (attacks declared, blocks pending — the
        // attack sim's leaf), first-strike/combat damage (blocks
        // declared, damage not yet dealt — the block sim's leaf), and
        // end-of-combat (damage marked on the survivors). `block_map`
        // and `attacking` are torn down when combat damage resolves, so
        // the blocking flags are only reachable at the third of these.
        // See [`COMBAT_SNAPSHOTS`].
        let step_point = g.step != last_step
            && match g.step {
                crate::game::TurnStep::PostCombatMain | crate::game::TurnStep::End => true,
                crate::game::TurnStep::DeclareAttackers
                | crate::game::TurnStep::DeclareBlockers
                | crate::game::TurnStep::FirstStrikeDamage
                | crate::game::TurnStep::CombatDamage
                | crate::game::TurnStep::EndCombat => combat_snapshots(),
                _ => false,
            };
        if new_turn || step_point {
            last_turn = (g.turn_number, g.active_player_idx);
            let pair = [encode_state(&g, 0, vocab), encode_state(&g, 1, vocab)];
            if last_pair.as_ref() != Some(&pair) {
                for (seat, s) in pair.iter().enumerate() {
                    snaps.push((g.turn_number, seat, s.clone()));
                    heur.push(crate::server::bot::eval_material_public(
                        &g,
                        seat,
                        &EvalWeights::default(),
                    ));
                    raw.push(snapshot_stats(&g, seat));
                    opp_hands.push(
                        g.players[1 - seat]
                            .hand
                            .iter()
                            .map(|c| vocab.index_of(c.definition.name))
                            .filter(|&i| i != 0)
                            .collect(),
                    );
                }
                last_pair = Some(pair);
            }
        }
        last_step = g.step;
        let mut any = false;
        // Uniform picks for the opening window, then the real policy.
        let acting =
            if actions < explore_plies { &mut explorers } else { &mut bots };
        for (s, bot) in acting.iter_mut().enumerate() {
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

    let turns = g.turn_number;
    // Same order as `recommend::play_one_game_traced`: `is_game_over`
    // wins over both caps, and the action cap wins over staleness.
    let stop = if g.is_game_over() {
        StopReason::GameOver
    } else if actions >= max_actions {
        StopReason::ActionCap
    } else {
        StopReason::NoLegalMove
    };
    let Some(Some(winner)) = g.game_over else {
        return RecordedGame { rows: Vec::new(), heur: Vec::new(), winner: None, turns, stop };
    };
    let life = [g.players[0].life, g.players[1].life];
    // One trajectory per (game, seat): the two seats see different
    // information and end on opposite results, so bootstrapping a row
    // against the other seat's successor would be meaningless. `seed`
    // identifies the game — actors already draw distinct seeds — and the
    // low bit carries the seat.
    let traj_base = (seed as u32) << 1;
    let mut ply = [0u16, 0u16];
    let rows = snaps
        .into_iter()
        .enumerate()
        .map(|(j, (turn, seat, state))| {
            let p = ply[seat];
            ply[seat] += 1;
            // Aux targets: deltas to this seat's *next* snapshot, which is
            // always at j + 2 because snapshots are pushed in seat pairs.
            // The last snapshot of a trajectory carries zero deltas — the
            // game ends there, so "no further change" is the true label.
            let aux = match raw.get(j + 2) {
                Some(next) => [
                    (next[0] - raw[j][0]) / 10.0,
                    (next[1] - raw[j][1]) / 8.0,
                    (next[2] - raw[j][2]) / 4.0,
                    next[3] / 7.0,
                ],
                None => [0.0; crabomination_nn::AUX_FEATS],
            };
            TrainRow {
                state,
                win: if seat == winner { 1.0 } else { 0.0 },
                life_diff: ((life[seat] - life[1 - seat]) as f32 / 20.0).clamp(-1.0, 1.0),
                game_len: (turns.saturating_sub(turn)) as f32 / 15.0,
                traj: traj_base | seat as u32,
                ply: p,
                aux,
                opp_hand: std::mem::take(&mut opp_hands[j]),
            }
        })
        .collect();
    RecordedGame { rows, heur, winner: Some(winner), turns, stop }
}

/// Raw per-seat stats at snapshot time: `[life diff, board-power diff,
/// creature-count diff, opponent hand size]`, unscaled — the aux labels
/// are diffs of these between a seat's consecutive snapshots.
fn snapshot_stats(g: &GameState, seat: usize) -> [f32; 4] {
    let opp = 1 - seat;
    let (mut power, mut creatures) = ([0i32; 2], [0i32; 2]);
    for c in g.battlefield.iter() {
        if c.definition.is_creature() {
            let side = if c.controller == seat { 0 } else { 1 };
            creatures[side] += 1;
            power[side] += c.power().max(0);
        }
    }
    [
        (g.players[seat].life - g.players[opp].life) as f32,
        (power[0] - power[1]) as f32,
        (creatures[0] - creatures[1]) as f32,
        g.players[opp].hand.len() as f32,
    ]
}

#[cfg(test)]
mod tests {

    #[test]
    fn extra_packs_deepen_the_pool_and_name_the_handicap() {
        // `--opponent-packs N`: the bot chooses its 40 from a deeper pool,
        // which is the difficulty dial. Same builder, more to pick from.
        assert_eq!(sealed_pool_packs(7, SEALED_PACKS).len(), sealed_pool(7).len());
        let standard = sealed_pool_packs(7, SEALED_PACKS).len();
        let deep = sealed_pool_packs(7, SEALED_PACKS * 2).len();
        assert_eq!(deep, standard * 2, "twice the packs is twice the cards");
        // A pack count of zero would build from nothing; clamp to one.
        assert!(!sealed_pool_packs(7, 0).is_empty());

        // The deck is still a legal 40, and the label carries the handicap
        // so a match remains reproducible from what it reports.
        let (deck, label) = random_sealed_opponent_packs(7, 12);
        assert_eq!(deck.len(), 40);
        assert!(label.contains("12 packs"), "label names the handicap: {label}");
        let (_, plain) = random_sealed_opponent_packs(7, SEALED_PACKS);
        assert_eq!(plain, "Sealed #7", "a standard pool reads as before");
    }
    use super::*;

    /// The client's sealed-opponent recipe: best-of-16 under the v3
    /// judge is a legal 40, deterministic in the seed, and actually the
    /// argmax of its own candidate set — not one more sample.
    #[test]
    fn best_build_v3_is_the_argmax_of_its_candidates() {
        let pool = sealed_pool(0xB3);
        let best = best_build_v3(&pool, 16, 9);
        assert_eq!(best.len(), 40);
        let cfg = SimConfig { builder_v3: true, ..SimConfig::default() };
        let top = build_candidates_cfg(&pool, 16, 9, &cfg)
            .iter()
            .map(|d| static_deck_score_v3(d))
            .max()
            .unwrap();
        assert_eq!(static_deck_score_v3(&best), top, "the pick is the judge's argmax");
        let names = |d: &[CardFactory]| d.iter().map(|f| f().name).collect::<Vec<_>>();
        assert_eq!(names(&best), names(&best_build_v3(&pool, 16, 9)), "deterministic");
    }

    /// The climb monotonically improves its judge, keeps the deck at 40
    /// with the mana base untouched, and is deterministic.
    #[test]
    fn hill_climb_improves_the_judge_and_preserves_shape() {
        let pool = sealed_pool(0xC1B);
        let start = heuristic_sealed_build(&pool, 5);
        let lands_before = start.iter().filter(|f| f().is_land()).count();
        // A judge the climb can definitely satisfy: cheap decks win.
        let judge = |d: &[CardFactory]| {
            -(d.iter().map(|f| f().cost.cmc() as i64).sum::<i64>() as f64)
        };
        let before = judge(&start);
        let climbed = hill_climb_build_by(&pool, start.clone(), 6, judge);
        let after = judge(&climbed);
        assert!(after >= before, "climb must never worsen: {before} -> {after}");
        assert!(after > before, "a sealed pool always has a cheaper spell somewhere");
        assert_eq!(climbed.len(), 40);
        assert_eq!(climbed.iter().filter(|f| f().is_land()).count(), lands_before);
        // Deterministic: same pool, same start, same deck.
        let again = hill_climb_build_by(&pool, start, 6, judge);
        let names = |d: &[CardFactory]| d.iter().map(|f| f().name).collect::<Vec<_>>();
        assert_eq!(names(&climbed), names(&again));
    }

    /// End-to-end: sealed pools build 40-card decks, a recorded game
    /// produces per-turn rows from both seats, and the labels agree with
    /// the reported winner.
    #[test]
    fn recorded_sealed_game_yields_labelled_rows() {
        let vocab = Vocab::sos_sealed();
        let pool_a = sealed_pool(0xA11CE);
        let pool_b = sealed_pool(0xB0B);
        assert!(pool_a.len() >= 78, "6 SOS packs, got {}", pool_a.len());
        let deck_a = heuristic_sealed_build(&pool_a, 1);
        let deck_b = heuristic_sealed_build(&pool_b, 2);
        assert_eq!(deck_a.len(), 40);
        assert_eq!(deck_b.len(), 40);

        let template = sealed_game_template(&deck_a, &deck_b);
        // A handful of seeds so a single stalled game can't fail the test.
        let mut decided = None;
        for seed in 0..5u64 {
            let rec = play_recorded_game(
                &template,
                [EvalWeights::default(), EvalWeights::default()],
                seed,
                4000,
                &vocab,
            );
            if rec.winner.is_some() {
                decided = Some(rec);
                break;
            }
        }
        let rec = decided.expect("five seeds, no decided game");
        let winner = rec.winner.unwrap();
        assert!(rec.turns >= 3, "game lasted {} turns", rec.turns);
        // Two rows per recorded turn, labels split by seat.
        assert!(rec.rows.len() >= 6, "only {} rows", rec.rows.len());
        assert_eq!(rec.rows.len() % 2, 0);
        for pair in rec.rows.chunks(2) {
            let wins: Vec<f32> = pair.iter().map(|r| r.win).collect();
            assert_eq!(wins.iter().sum::<f32>(), 1.0, "one winner view per turn");
        }
        let winner_rows = rec.rows.iter().skip(winner).step_by(2);
        assert!(winner_rows.clone().all(|r| r.win == 1.0));
        // Later snapshots are closer to the end.
        let first = &rec.rows[0];
        let last = &rec.rows[rec.rows.len() - 1];
        assert!(first.game_len > last.game_len);
        // Pools and builds ARE seed-determined (games are not — the bot's
        // candidate jitter is thread-RNG exploration noise by design).
        let pool_again = sealed_pool(0xA11CE);
        let names = |d: &[CardFactory]| d.iter().map(|f| f().name).collect::<Vec<_>>();
        assert_eq!(names(&pool_again), names(&pool_a));
        assert_eq!(
            names(&heuristic_sealed_build(&pool_again, 1)),
            names(&deck_a)
        );
    }
}
