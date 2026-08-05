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
//! games themselves are not replayable — [`RandomBot`]'s candidate jitter
//! draws from the thread RNG by design — which the training loop doesn't
//! need; the jitter is the exploration noise that diversifies the data.

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crabomination_nn::TrainRow;

use crate::cube::CardFactory;
use crate::draft::{generate_sos_pack, sos_draft_pool};
use crate::game::GameState;
use crate::recommend::{SimConfig, build_match_template, build_random_deck};
use crate::server::bot::{Bot, EvalWeights, RandomBot};
use crate::server::encode::{Vocab, encode_state};

/// A 6-pack SOS sealed pool, fully determined by `seed`.
pub fn sealed_pool(seed: u64) -> Vec<CardFactory> {
    let pool = sos_draft_pool();
    let mut rng = StdRng::seed_from_u64(seed);
    (0..6).flat_map(|_| generate_sos_pack(&pool, &mut rng)).collect()
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

/// A random sealed opponent: a fresh 6-pack pool, built by the same
/// heuristic builder the training loop and recommender use. Returns the
/// 40-card deck and a label naming the seed, so a match can be
/// reproduced ("Sealed #12345").
///
/// Exists for the client's Sealed game mode — the point of that mode is
/// to play a hand-built deck against the field the recommender races
/// against, so the opponent has to come from the same generator.
pub fn random_sealed_opponent(seed: u64) -> (Vec<CardFactory>, String) {
    let pool = sealed_pool(seed);
    let deck = heuristic_sealed_build(&pool, seed ^ 0x005E_A1ED);
    (deck, format!("Sealed #{seed}"))
}

/// Unshuffled two-seat template with both libraries loaded — clone per
/// game.
pub fn sealed_game_template(seat0: &[CardFactory], seat1: &[CardFactory]) -> GameState {
    build_match_template(seat0, seat1)
}

/// `n` distinct noisy-greedy builds of the same pool — the candidate set
/// a build judge picks from. Deterministic in `seed`.
pub fn build_candidates(pool: &[CardFactory], n: usize, seed: u64) -> Vec<Vec<CardFactory>> {
    let cfg = SimConfig::default();
    (0..n as u64)
        .map(|i| {
            let mut rng =
                StdRng::seed_from_u64(seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7));
            build_random_deck(pool, &cfg, &mut rng).cards
        })
        .collect()
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

/// The heuristic builder's own opinion of a finished deck, exposed so a
/// gate can pit "static-score judge" against a learned judge over the
/// *same* candidate set — the comparison that isolates the judge.
/// (`static_build_score` scores the spell picks; lands are filtered out
/// and no shortfall applies to a completed 40-card build.)
pub fn static_deck_score(deck: &[CardFactory]) -> i32 {
    let spells: Vec<CardFactory> = deck.iter().copied().filter(|f| !f().is_land()).collect();
    crate::recommend::static_build_score(&spells, spells.len())
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
    let mut g = template.clone();
    let mut rng = StdRng::seed_from_u64(seed);
    for seat in 0..2 {
        g.players[seat].library.shuffle(&mut rng);
    }
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
        .map(|w| Box::new(RandomBot::with_weights(w)) as Box<dyn Bot>)
        .collect();
    let mut explorers: Vec<Box<dyn Bot>> =
        (0..2).map(|_| Box::new(RandomBot::uniform_baseline()) as Box<dyn Bot>).collect();

    // (turn, seat, encoded state) — labelled after the game decides.
    let mut snaps = Vec::new();
    let mut heur: Vec<i32> = Vec::new();
    let mut last_turn = (0u32, usize::MAX);
    let mut last_step = crate::game::TurnStep::Untap;
    let mut last_pair: Option<[crabomination_nn::EncodedState; 2]> = None;
    let (mut actions, mut stale) = (0usize, 0usize);
    while !g.is_game_over() && actions < max_actions && stale < 8 {
        let new_turn = (g.turn_number, g.active_player_idx) != last_turn;
        let step_point = g.step != last_step
            && matches!(
                g.step,
                crate::game::TurnStep::PostCombatMain | crate::game::TurnStep::End
            );
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
    let Some(Some(winner)) = g.game_over else {
        return RecordedGame { rows: Vec::new(), heur: Vec::new(), winner: None, turns };
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
        .map(|(turn, seat, state)| {
            let p = ply[seat];
            ply[seat] += 1;
            TrainRow {
                state,
                win: if seat == winner { 1.0 } else { 0.0 },
                life_diff: ((life[seat] - life[1 - seat]) as f32 / 20.0).clamp(-1.0, 1.0),
                game_len: (turns.saturating_sub(turn)) as f32 / 15.0,
                traj: traj_base | seat as u32,
                ply: p,
            }
        })
        .collect();
    RecordedGame { rows, heur, winner: Some(winner), turns }
}

#[cfg(test)]
mod tests {
    use super::*;

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
