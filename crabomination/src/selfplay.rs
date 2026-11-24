//! Self-play game generation for the ML training loop.
//!
//! `selfplay_train` (in `crabomination_ml`) owns threads, window and
//! learner; this module owns everything that touches the engine — building a
//! sealed match and playing it while recording encoded snapshots — so the bin
//! never sees `GameState` internals and the snapshot cadence lives next to the
//! encoder it feeds.
//!
//! Determinism: pools, builds, shuffles and [`HeuristicBot`]'s jitter (pinned
//! per game by [`play_recorded_game_mcts`]) all derive from the caller's
//! seeds, so a recorded game re-runs from its seed.

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crabomination_nn::TrainRow;

use crate::cube::CardFactory;
use crate::draft::SosPacks;
use crate::game::GameState;
use crate::mana::ColorSet;
use crate::recommend::{
    SimConfig, StopReason, build_match_template, build_random_deck, stop_reason,
};
use crate::sos_mode::College;
use crate::server::bot::{Bot, EvalWeights, HeuristicBot};
use crate::server::encode::{Vocab, encode_state_pair};

/// Whether the snapshot cadence includes combat steps.
///
/// Default off (every run through round 39): snapshots at turn start,
/// post-combat main and end step — none of which is mid-combat, so the
/// feature census (`selfplay_train --feature-census`) found every combat
/// feature zero in every row. The attack/block sims do evaluate mid-combat
/// positions, so those columns never got a gradient. On adds snapshots at
/// declare-blockers entry (the attack sim's leaf) and end-of-combat (the
/// block sim's leaf).
///
/// Process-global like the encoder ablation: set once at startup, read on
/// every actor thread.
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
    // `CRAB_SEALED_SET=fra` opens Reality Fracture boosters instead
    // (`fra_mode`); unset, the pool is SOS's as before.
    sealed_pool_of(crate::fra_mode::SealedSet::current(), seed, packs)
}

/// [`sealed_pool_packs`] for a named set rather than the process's — what a
/// caller mixing sets in one process (the table recorder's mode mixer) needs.
pub fn sealed_pool_of(set: crate::fra_mode::SealedSet, seed: u64, packs: usize) -> Vec<CardFactory> {
    let mut rng = StdRng::seed_from_u64(seed);
    if set == crate::fra_mode::SealedSet::Fra {
        let rolls = crate::fra_mode::FraPacks::get();
        return (0..packs.max(1)).flat_map(|_| rolls.roll(&mut rng)).collect();
    }
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

/// [`heuristic_sealed_build_v3`] scoring cards by their measured value
/// (`SimConfig::rated_cards`, [`crate::card_ratings`]) — the FRA builder
/// `fra_card_eval --ab` races against the v3 sample it replaces.
pub fn heuristic_sealed_build_rated(pool: &[CardFactory], seed: u64) -> Vec<CardFactory> {
    let cfg = SimConfig { builder_v3: true, rated_cards: true, ..SimConfig::default() };
    let mut rng = StdRng::seed_from_u64(seed);
    build_random_deck(pool, &cfg, &mut rng).cards
}

/// The best of `n` rated builds of `pool`, judged by the rated v3 shape
/// score — [`best_build_v3`] with `SimConfig::rated_cards`. The sealed
/// helper `fra_card_eval --build` prints this for a player's own pool.
pub fn rated_sealed_build(pool: &[CardFactory], n: usize, seed: u64) -> Vec<CardFactory> {
    rated_sealed_build_in_colors(pool, n, seed, ColorSet::empty())
}

/// [`rated_sealed_build`] with the candidate shapes confined to `colors`
/// ([`SimConfig::color_restriction`]; `ColorSet::empty()` is unrestricted),
/// so `fra_card_eval --build POOL --colors WG` can put each pair of a pool
/// on the table instead of only the one the shape score likes.
pub fn rated_sealed_build_in_colors(pool: &[CardFactory], n: usize, seed: u64, colors: ColorSet) -> Vec<CardFactory> {
    rated_sealed_build_cfg(pool, n, seed, SimConfig { color_restriction: colors, ..SimConfig::default() })
}

/// [`rated_sealed_build_in_colors`] with the land count fixed at `lands`
/// (and the spells at 40 − `lands`) instead of sampled from 16..=18.
pub fn rated_sealed_build_lands(pool: &[CardFactory], n: usize, seed: u64, colors: ColorSet, lands: u32) -> Vec<CardFactory> {
    let spells = 40 - lands as usize;
    let cfg = SimConfig {
        color_restriction: colors,
        spell_count_range: (spells, spells),
        land_count_range: (lands, lands),
        target_spells: spells,
        total_lands: lands,
        ..SimConfig::default()
    };
    rated_sealed_build_cfg(pool, n, seed, cfg)
}

/// Does the rated judge add [`crate::card_ratings::shape_penalty`]? On: it
/// pays more the more candidates it chooses among (FRA best of 64, seeds
/// 43 / 97: 50.43 / 50.53 %; best of 16 ~50.1; SOS best of 16 50.83 / 50.53).
pub const RATED_SHAPE_JUDGE: bool = true;

/// The best of `n` rated v3 builds under `cfg` (its builder flags forced on).
fn rated_sealed_build_cfg(pool: &[CardFactory], n: usize, seed: u64, cfg: SimConfig) -> Vec<CardFactory> {
    rated_sealed_build_judged(pool, n, seed, cfg, RATED_SHAPE_JUDGE)
}

/// [`rated_sealed_build_in_colors`] with the judge's shape term chosen by
/// the caller — the two arms `fra_card_eval --ab --best N --shape` races.
pub fn rated_sealed_build_shaped(pool: &[CardFactory], n: usize, seed: u64, shape: bool) -> Vec<CardFactory> {
    rated_sealed_build_judged(pool, n, seed, SimConfig::default(), shape)
}

/// Does the rated builder build half its candidates to the measured deck
/// shape (`SimConfig::shape_slots`)? See [`build_candidates_shaped`]. Off:
/// the judge takes shaped decks but they play no better (FRA best of 64
/// 49.50 / 49.89 %) — the curve [`crate::card_ratings::shape_penalty`] fits
/// is partly the pool, so building to it doesn't buy what the term prices.
pub const RATED_SHAPE_SLOTS: bool = false;

fn rated_sealed_build_judged(pool: &[CardFactory], n: usize, seed: u64, cfg: SimConfig, shape: bool) -> Vec<CardFactory> {
    rated_sealed_build_slotted(pool, n, seed, cfg, shape, RATED_SHAPE_SLOTS)
}

/// [`rated_sealed_build_shaped`] (judge term on) with the generator's
/// shape slots chosen by the caller — the two arms `fra_card_eval --ab
/// --best N --slots` races.
pub fn rated_sealed_build_slots(pool: &[CardFactory], n: usize, seed: u64, slots: bool) -> Vec<CardFactory> {
    rated_sealed_build_slotted(pool, n, seed, SimConfig::default(), true, slots)
}

fn rated_sealed_build_slotted(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    cfg: SimConfig,
    shape: bool,
    slots: bool,
) -> Vec<CardFactory> {
    let cfg = SimConfig { builder_v3: true, rated_cards: true, ..cfg };
    let judge = |d: &[CardFactory]| {
        let spells: Vec<CardFactory> = d.iter().copied().filter(|f| !crate::cube::card_def(*f).is_land()).collect();
        let shape_term = if shape { crate::card_ratings::shape_penalty(&spells) } else { 0 };
        crate::recommend::static_build_score_v3(&spells, spells.len(), true) + shape_term
    };
    let cands = if slots { build_candidates_shaped(pool, n, seed, &cfg) } else { build_candidates_cfg(pool, n, seed, &cfg) };
    let best = cands.into_iter().max_by_key(|d| judge(d)).unwrap_or_default();
    if !polish_on() || best.is_empty() {
        return best;
    }
    judge_polish(pool, best, |d| judge(d) as f64)
}

/// Does the rated builder hill-climb its best-of-`n` pick under the same
/// judge? The candidates are jittered builds, so a pick keeps swaps its own
/// judge prefers (Murmuring Volume made 5 of 20 client FRA decks, 2026-10-29).
/// On: polished vs unpolished, best of 16, seeds 43 / 97, FRA 52.64 / 52.66 %,
/// SOS 50.84 / 50.72 %. About 0.3 ms a build.
pub const RATED_JUDGE_POLISH: bool = true;

thread_local! {
    static POLISH: std::cell::Cell<bool> = const { std::cell::Cell::new(RATED_JUDGE_POLISH) };
}

fn polish_on() -> bool {
    POLISH.get()
}

/// Run `f` with [`RATED_JUDGE_POLISH`] set to `on` — the trial arm of
/// `fra_card_eval --ab --trial`.
pub fn with_judge_polish<R>(on: bool, f: impl FnOnce() -> R) -> R {
    let was = POLISH.replace(on);
    let r = f();
    POLISH.set(was);
    r
}

/// [`hill_climb_build_by`] over the pool cards the deck's lands can cast:
/// the judge reads spells only, so a swap that needs a colour the mana base
/// lacks would score as a gain and play as a dead card.
fn judge_polish(pool: &[CardFactory], deck: Vec<CardFactory>, judge: impl FnMut(&[CardFactory]) -> f64) -> Vec<CardFactory> {
    let mana = deck
        .iter()
        .filter(|f| crate::cube::card_def(**f).is_land())
        .fold(ColorSet::empty(), |s, f| ColorSet(s.0 | crate::cube::card_brief(*f).produces.0));
    let castable: Vec<CardFactory> = pool
        .iter()
        .copied()
        .filter(|f| {
            let b = crate::cube::card_brief(*f);
            b.def.is_land() || deck.contains(f) || b.fits(mana)
        })
        .collect();
    hill_climb_build_by(&castable, deck, 3, judge)
}

/// The rated builder's best-of-`n` pick, unpolished, then polished under a
/// learned deck value ([`crate::deck_value`]) by at most `swaps` single-spell
/// swaps over pool cards the lands cast and the value has priced. The step-1
/// play gate's arm (ML_NOTES "A deck net that builds decks", `fra_card_eval
/// --trial deckvalue`). The swap cap is a trust region: the value was trained
/// on decks three swaps from a builder's, and round 15 showed what an
/// unbounded climb does to a learned judge.
///
/// `after_polish` swaps on top of the rated builder's own polish (step 1b)
/// instead of in its place.
pub fn deck_value_build(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    dv: &crate::deck_value::DeckValue,
    swaps: usize,
    after_polish: bool,
) -> Vec<CardFactory> {
    deck_value_build_in_colors(pool, n, seed, ColorSet::empty(), dv, swaps, after_polish)
}

/// [`deck_value_build`] with the rated pick confined to `colors`
/// ([`rated_sealed_build_in_colors`]); the swaps stay castable off the
/// pick's lands, so they keep its colours.
pub fn deck_value_build_in_colors(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    colors: ColorSet,
    dv: &crate::deck_value::DeckValue,
    swaps: usize,
    after_polish: bool,
) -> Vec<CardFactory> {
    let pick = with_judge_polish(after_polish, || rated_sealed_build_in_colors(pool, n, seed, colors));
    let mana = pick
        .iter()
        .filter(|f| crate::cube::card_def(**f).is_land())
        .fold(ColorSet::empty(), |s, f| ColorSet(s.0 | crate::cube::card_brief(*f).produces.0));
    let castable: Vec<CardFactory> = pool
        .iter()
        .copied()
        .filter(|f| {
            let b = crate::cube::card_brief(*f);
            !b.def.is_land() && b.fits(mana) && dv.knows(b.def.name)
        })
        .collect();
    let mut deck = pick;
    for _ in 0..swaps {
        // What the pool still holds, by name: a copy in the deck is a copy
        // out of the pool.
        let mut avail: std::collections::BTreeMap<&'static str, (CardFactory, i32)> = std::collections::BTreeMap::new();
        for f in &castable {
            avail.entry(crate::cube::card_def(*f).name).or_insert((*f, 0)).1 += 1;
        }
        for f in &deck {
            if let Some(e) = avail.get_mut(crate::cube::card_def(*f).name) {
                e.1 -= 1;
            }
        }
        let base = dv.value(&deck);
        let mut best: Option<(f64, usize, CardFactory)> = None;
        for i in 0..deck.len() {
            if crate::cube::card_def(deck[i]).is_land() {
                continue;
            }
            let old = deck[i];
            for (name, (f, k)) in &avail {
                if *k <= 0 || *name == crate::cube::card_def(old).name {
                    continue;
                }
                deck[i] = *f;
                let v = dv.value(&deck);
                if v > base + 1e-9 && best.is_none_or(|b| v > b.0) {
                    best = Some((v, i, *f));
                }
            }
            deck[i] = old;
        }
        let Some((_, i, f)) = best else { break };
        deck[i] = f;
    }
    deck
}

/// [`build_candidates_cfg`] with every odd-numbered candidate built under
/// `SimConfig::shape_slots`: the same lattice, the same seeds, so the even
/// half is exactly the unshaped set's and the judge chooses between a
/// strictly-by-score build and a shaped one of the same colours.
pub fn build_candidates_shaped(pool: &[CardFactory], n: usize, seed: u64, cfg: &SimConfig) -> Vec<Vec<CardFactory>> {
    let scores = crate::recommend::PoolScores::new(pool, cfg.builder_v2, cfg.curve_aggro, cfg.rated_cards);
    let shapes = crate::recommend::enumerate_shapes(&scores, cfg);
    let shaped = SimConfig { shape_slots: true, ..cfg.clone() };
    (0..n as u64)
        .map(|i| {
            let mut rng =
                StdRng::seed_from_u64(seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7));
            let c = if i % 2 == 1 { &shaped } else { cfg };
            crate::recommend::build_random_deck_from(&shapes, &scores, c, &mut rng).cards
        })
        .collect()
}

/// A random sealed opponent: a fresh 6-pack pool built as the builder
/// family's best-of-16 pick (see [`random_sealed_opponent_packs`]). Returns
/// the 40 cards and a reproducible label ("Sealed #12345").
pub fn random_sealed_opponent(seed: u64) -> (Vec<CardFactory>, String) {
    random_sealed_opponent_packs(seed, SEALED_PACKS)
}

/// [`random_sealed_opponent`] opening `packs` boosters: the same builder
/// choosing 40 from a deeper pool, a smooth difficulty dial. The label names
/// the handicap.
///
/// The deck is [`best_build_v3`]'s best-of-16, not the training field's
/// single sample: a field member is deliberately mediocre, but a named
/// opponent should be the room's best builder, not a free extra handicap.
/// Deterministic in `seed`.
pub fn random_sealed_opponent_packs(seed: u64, packs: usize) -> (Vec<CardFactory>, String) {
    random_sealed_opponent_in_colors(seed, packs, ColorSet::empty())
}

/// [`random_sealed_opponent_packs`] with the opponent's colours pinned
/// (`{Red, White}` = Lorehold every game, the client's `--opponent-colors`);
/// `ColorSet::empty()` is unrestricted.
///
/// The restriction chooses the 40 from a plain `packs` pool; it never rerolls
/// for a pool deep in those colours, since that would make the opponent
/// stronger than `packs` says. The filter is mana identity, so a card castable
/// with generic mana (`{2/G}{2/G}`) is fair game. Needs at least two colours
/// (see [`SimConfig::color_restriction`]).
pub fn random_sealed_opponent_in_colors(
    seed: u64,
    packs: usize,
    colors: ColorSet,
) -> (Vec<CardFactory>, String) {
    let packs = packs.max(1);
    let pool = sealed_pool_packs(seed, packs);
    let deck = sealed_opponent_build(&pool, 16, seed ^ 0x005E_A1ED, colors);
    (deck, sealed_opponent_label(seed, packs, colors))
}

/// The opponent's name: the seed, plus whatever was asked for on top of a
/// standard pool. Both handicaps are named so a match stays reproducible
/// from what the client printed.
fn sealed_opponent_label(seed: u64, packs: usize, colors: ColorSet) -> String {
    let mut extras: Vec<String> = Vec::new();
    if packs != SEALED_PACKS {
        extras.push(format!("{packs} packs"));
    }
    if !colors.is_empty() {
        extras.push(color_identity_name(colors));
    }
    if extras.is_empty() {
        format!("Sealed #{seed}")
    } else {
        format!("Sealed #{seed} ({})", extras.join(", "))
    }
}

/// A colour identity's display name: the Strixhaven college when the set is
/// exactly one college's pair, WUBRG letters otherwise. The sealed pool
/// *is* a Strixhaven pool, so the five colleges are the five pairs a player
/// names — and `rw` and `lorehold` then read back the same way.
pub fn color_identity_name(colors: ColorSet) -> String {
    College::ALL
        .into_iter()
        .find(|c| c.colors().iter().collect::<ColorSet>() == colors)
        .map(|c| c.name().to_string())
        .unwrap_or_else(|| colors.iter().map(|c| c.short_name()).collect())
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
    // The shape lattice and `PoolScores` are deterministic in `(pool, cfg)`
    // (see `build_random_deck_from`), so they are computed once for all `n`
    // candidates rather than per candidate.
    let scores = crate::recommend::PoolScores::new(pool, cfg.builder_v2, cfg.curve_aggro, cfg.rated_cards);
    let shapes = crate::recommend::enumerate_shapes(&scores, cfg);
    let out: Vec<Vec<CardFactory>> = (0..n as u64)
        .map(|i| {
            let mut rng =
                StdRng::seed_from_u64(seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7));
            crate::recommend::build_random_deck_from(&shapes, &scores, cfg, &mut rng).cards
        })
        .collect();
    // The hoist is sound only while `enumerate_candidates` is pure in
    // `(pool, cfg)`. Debug builds re-derive and compare, so hidden state or a
    // candidate build perturbing it fails the suite
    // (`best_build_v3_is_the_argmax_of_its_candidates`) instead of silently
    // changing every judged build.
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
    best_build_v3_in_colors(pool, n, seed, ColorSet::empty())
}

/// [`best_build_v3`] with the candidate shapes confined to one colour
/// identity ([`SimConfig::color_restriction`]); `ColorSet::empty()` is the
/// unrestricted call above.
pub fn best_build_v3_in_colors(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    colors: ColorSet,
) -> Vec<CardFactory> {
    best_build_v3_curved(pool, n, seed, colors, false)
}

/// The **client's** sealed-opponent recipe: [`best_build_v3_in_colors`] with
/// `SimConfig::curve_aggro`.
///
/// A separate entry point, not a flag in the shared function: `best_build_v3`
/// is an arm of `selfplay_train --gate-builder-v3`, and flipping the curve
/// there silently changed what that gate measures. Neither touches
/// `SimConfig::default()`, so the training field and ladder gate decks are
/// unaffected. `curve_aggro` adds what `builder_v3`'s shape-level
/// `curve_penalty` lacks: a mana-value pick term (`draft::aggro_curve_delta`)
/// and reserved early-drop slots (`recommend::CurveSlots`).
///
/// A pool the measured ratings cover ([`crate::card_ratings::rates_pool`]:
/// FRA and SOS) goes to [`rated_sealed_build_in_colors`] instead (vs this
/// recipe, 2026-10-26: FRA 69.7 / 70.0 %, SOS 60.9 / 60.6 %). A pool the
/// learned builder covers ([`LEARNED_SEALED_BUILDER`]) is built by it: the
/// policy proposes greedy, `n` temperature-1 and per-colour-pair builds and
/// the same-pool value picks ([`DeckPolicy::build_valued`]). A colour
/// restriction keeps the rated build in those colours.
pub fn sealed_opponent_build(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    colors: ColorSet,
) -> Vec<CardFactory> {
    if crate::card_ratings::rates_pool(pool) {
        if LEARNED_SEALED_BUILDER && colors.is_empty() && sealed_deck_policy().covers(pool) {
            let mut rng = StdRng::seed_from_u64(seed ^ 3);
            let opts = sealed_valued_opts();
            let value = if crate::card_ratings::sos_pool(pool) { sos_deck_value() } else { fra_deck_value() };
            return sealed_deck_policy().build_valued_with(pool, value, n, &mut rng, &opts);
        }
        return rated_sealed_build_in_colors(pool, n, seed, colors);
    }
    v3_opponent_build(pool, n, seed, colors)
}

/// A drafted pile as the client's deckbuilding screen shows it: the main
/// deck's nonbasic cards, the rest of the pile, and the basics to add.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftDeck {
    pub main: Vec<CardFactory>,
    pub sideboard: Vec<CardFactory>,
    /// Basic lands by colour, WUBRG order, counts above zero.
    pub basics: Vec<(crate::mana::Color, u32)>,
    /// The deck's two main colours (most basics first).
    pub colors: [crate::mana::Color; 2],
}

impl DraftDeck {
    /// The playable list: main plus basics.
    pub fn deck(&self) -> Vec<CardFactory> {
        let mut deck = self.main.clone();
        for &(c, n) in &self.basics {
            deck.extend(std::iter::repeat_n(crate::draft::basic_land_factory(c), n as usize));
        }
        deck
    }
}

/// Build a drafted pile for play: the client's suggestion for the player and
/// the bot opponent's deck.
///
/// A pile the measured ratings cover (SOS) goes through
/// [`sealed_opponent_build`] (learned builder, polish 3) and is split back
/// into main / sideboard / basics; any other pile (Cube) keeps
/// `draft::suggest_main_deck` + `suggest_basic_split`. 2026-11-15
/// (`fra_card_eval --draft-ab`, seeds 43 / 97, basics exempt from
/// [`crate::draft::enforce_copy_cap`]): SOS learned vs suggestion 59.6 / 59.5
/// %, vs rated 52.1 / 52.1 %; Cube `v3` vs suggestion 49.9 / 49.7 %, a tie.
///
/// Copies of a nonbasic past four go to the sideboard, each replaced by a
/// main-colour basic, so the client's copy cap never shortens the deck.
pub fn draft_deck_build(picks: &[CardFactory], seed: u64) -> DraftDeck {
    use crate::draft::{COPY_CAP, basic_land_factory, suggest_basic_split, suggest_main_deck, top_two_colors};
    use crate::mana::Color;
    if !crate::card_ratings::rates_pool(picks) {
        let (main, sideboard) = suggest_main_deck(picks, 23);
        let colors = top_two_colors(picks);
        let split = suggest_basic_split(&main, colors, 17);
        let basics = Color::ALL
            .into_iter()
            .filter_map(|c| split.get(&c).copied().filter(|&n| n > 0).map(|n| (c, n)))
            .collect();
        return DraftDeck { main, sideboard, basics, colors };
    }
    let deck = sealed_opponent_build(picks, 16, seed, ColorSet::empty());
    let basic_color = |f: CardFactory| {
        let def = crate::cube::card_def(f);
        if !def.is_land() || !def.supertypes.contains(&crate::card::Supertype::Basic) {
            return None;
        }
        Color::ALL.into_iter().find(|&c| crate::cube::card_def(basic_land_factory(c)).name == def.name)
    };
    let mut counts = [0u32; 5];
    let mut main = Vec::new();
    let mut over_cap = 0u32;
    let mut copies: crate::fxhash::HashMap<&str, u32> = Default::default();
    let mut sideboard = picks.to_vec();
    for f in deck {
        if let Some(c) = basic_color(f) {
            counts[c as usize] += 1;
            continue;
        }
        let n = copies.entry(crate::cube::card_def(f).name).or_insert(0);
        if *n >= COPY_CAP {
            over_cap += 1;
            continue;
        }
        *n += 1;
        if let Some(i) = sideboard.iter().position(|&g| crate::cube::card_def(g).name == crate::cube::card_def(f).name) {
            sideboard.remove(i);
        }
        main.push(f);
    }
    let mut by_count: Vec<Color> = Color::ALL.into_iter().filter(|&c| counts[c as usize] > 0).collect();
    by_count.sort_by_key(|&c| std::cmp::Reverse(counts[c as usize]));
    for c in top_two_colors(&main) {
        if !by_count.contains(&c) {
            by_count.push(c);
        }
    }
    let colors = [by_count[0], by_count[1]];
    counts[colors[0] as usize] += over_cap;
    let basics = Color::ALL.into_iter().filter(|&c| counts[c as usize] > 0).map(|c| (c, counts[c as usize])).collect();
    DraftDeck { main, sideboard, basics, colors }
}

/// Do FRA and SOS sealed opponents build with the learned policy and
/// same-pool deck value? On, adopted 2026-11-07 at the user's call (ML_NOTES
/// "A deck net that builds decks", step 5; vs rated best-of-16, seeds 43 / 97:
/// FRA 51.41 / 51.84 %, SOS 52.78 / 52.75 %). Replaced step 1b's SOS polish.
pub const LEARNED_SEALED_BUILDER: bool = true;

/// Max value-guided one-for-one swaps an SOS sealed opponent makes after the
/// learned pick (`deck_policy::polish`, kept only if the relanded deck still
/// rates higher). 3, adopted 2026-11-09 at the user's call (ML_NOTES
/// "Research sweep": ~51.5-51.8 % over 4 seeds; 1 and 6 swaps no better). FRA
/// stays unpolished — its value's swap preferences don't survive play (~50 %).
pub const SOS_SEALED_POLISH: usize = 3;

/// The sealed opponent's build-time levers on both sets (round 88, under the
/// step-7 values): [`SOS_SEALED_POLISH`] swaps, now on FRA too (polish 3 vs
/// none, seeds 11 / 29 / 43 / 97: 50.98 / 51.15 / 51.29 / 51.41), plus 32
/// extra candidates at T 1.5 / 2 (`hot_samples`; polish 3 + 48 candidates
/// vs polish 3 alone, seeds 11 / 29: SOS 52.36 vs 51.81 pooled, FRA 51.14 vs
/// 51.07). The FRA null of 2026-11-08 was read under step 5's value, which
/// could not see sources.
pub const SEALED_HOT_SAMPLES: usize = 32;

/// [`sealed_opponent_build`]'s [`crate::deck_policy::ValuedOpts`].
pub fn sealed_valued_opts() -> crate::deck_policy::ValuedOpts {
    crate::deck_policy::ValuedOpts { polish: SOS_SEALED_POLISH, hot_samples: SEALED_HOT_SAMPLES, ..Default::default() }
}

/// The add-one-card deck policy (width 256, DAgger imitation then one
/// generation on race labels — step 4's `cap256_gen1`), compiled in like
/// the rating tables it replaces. It knows both sheets.
pub fn sealed_deck_policy() -> &'static crate::deck_policy::DeckPolicy {
    static P: std::sync::OnceLock<crate::deck_policy::DeckPolicy> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        crate::deck_policy::DeckPolicy::from_json(include_str!("../../nets/deck-policy.json"))
            .expect("the compiled deck policy parses")
    })
}

/// The FRA same-pool deck value (which of one pool's builds wins). Since
/// 2026-11-16 step 7's export (A3, λ 10, seed 1, `--mana`), swapped in at the
/// user's call: under the confirm bar in play (~50.25-50.81 % over 4 seeds)
/// but taken for the mana base — a deck starved of its second colour rates
/// lower in 99.9 % of decks (step 5's value: 87 %).
pub fn fra_deck_value() -> &'static crate::deck_value::DeckValue {
    static DV: std::sync::OnceLock<crate::deck_value::DeckValue> = std::sync::OnceLock::new();
    DV.get_or_init(|| {
        crate::deck_value::DeckValue::from_json(include_str!("../../nets/deck-value-fra.json"))
            .expect("the compiled FRA deck value parses")
    })
}

/// The SOS same-pool deck value. Since 2026-11-16 step 7's export (A3, λ 10,
/// seed 1, `--mana`; see [`fra_deck_value`]), passing the registered bar
/// (~50.4-51.1 % over 4 seeds). [`SOS_SEALED_POLISH`] and
/// [`draft_deck_build`]'s reads were taken under step 5's value.
pub fn sos_deck_value() -> &'static crate::deck_value::DeckValue {
    static DV: std::sync::OnceLock<crate::deck_value::DeckValue> = std::sync::OnceLock::new();
    DV.get_or_init(|| {
        crate::deck_value::DeckValue::from_json(include_str!("../../nets/deck-value-sos.json"))
            .expect("the compiled SOS deck value parses")
    })
}

/// [`sealed_opponent_build`]'s v3 recipe on any pool — the control
/// `fra_card_eval --ab --best N` races the rated build against.
pub fn v3_opponent_build(pool: &[CardFactory], n: usize, seed: u64, colors: ColorSet) -> Vec<CardFactory> {
    best_build_v3_curved(pool, n, seed, colors, true)
}

fn best_build_v3_curved(
    pool: &[CardFactory],
    n: usize,
    seed: u64,
    colors: ColorSet,
    curve_aggro: bool,
) -> Vec<CardFactory> {
    let cfg = SimConfig {
        builder_v3: true,
        curve_aggro,
        color_restriction: colors,
        ..SimConfig::default()
    };
    // `n == 0`, or a pool with no playable card, has no best build — and an
    // empty deck is the answer, not a panic. This is the actor's deck-building
    // path (`selfplay_train` calls it twice a game); a panic at game 400 k
    // kills a training run where a decked-out game is one counted loss.
    build_candidates_cfg(pool, n, seed, &cfg)
        .into_iter()
        .max_by_key(|d| static_deck_score_v3(d))
        .unwrap_or_default()
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
    // Same contract as [`best_build_v3`]: no candidates means an empty deck,
    // never a panic on the actor's path.
    build_candidates(pool, n, seed)
        .into_iter()
        .map(|d| (judge(&d), d))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, d)| d)
        .unwrap_or_default()
}

/// Greedy hill-climb over single spell swaps under an arbitrary judge
/// ([`best_build_by`] only picks among random candidates). For each non-land
/// slot try every unused non-land pool card, keep the first strict
/// improvement, up to `max_passes` or a pass with none. Cheap for a deck-net
/// judge.
///
/// Lands are never touched — the judge can't see a broken mana base.
/// Deterministic (candidates in name order).
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
    crate::recommend::static_build_score_v3(&spells, spells.len(), false)
}

/// One finished self-play game's worth of labelled rows.
pub struct RecordedGame {
    pub rows: Vec<TrainRow>,
    /// The heuristic evaluation of each row's position from that row's seat,
    /// parallel to `rows`. Not in the shard format: it lets the calibration
    /// diagnostic score net and heuristic as predictors on identical
    /// positions.
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

/// Play one bot game from `template`, snapshotting from both seats'
/// perspectives, and stamp the outcome labels at the end.
///
/// Cadence: turn start, postcombat main entry, end step entry (consecutive
/// duplicates skipped). The mid-turn points are there because nets trained on
/// turn boundaries alone gated at ~43 % — `eval_material` reads post-action
/// and post-combat positions. Both seats' views are pushed together, so win
/// labels are balanced by construction.
pub fn play_recorded_game(
    template: &GameState,
    weights: [EvalWeights; 2],
    seed: u64,
    max_actions: usize,
    vocab: &Vocab,
) -> RecordedGame {
    play_recorded_game_mcts(template, weights, seed, max_actions, vocab, None)
}

/// The MCTS actors' search: `iterations` rollouts, the round-26 3-turn
/// horizon, Gumbel at the root when `gumbel`. 0 iterations is no search.
pub fn actor_mcts(iterations: u32, gumbel: bool) -> Option<crate::server::MctsConfig> {
    (iterations > 0).then(|| crate::server::MctsConfig {
        iterations,
        horizon_turns: 3,
        gumbel,
        ..crate::server::MctsConfig::default()
    })
}

/// [`play_recorded_game`] with net-evaluated MCTS pilots (`mcts` is the
/// search config, each seat's `weights` written into it; `actor_mcts` builds
/// the round-26 shape) — the training side of search amplification. `weights`
/// still parameterizes the reward and exploration openings.
pub fn play_recorded_game_mcts(
    template: &GameState,
    weights: [EvalWeights; 2],
    seed: u64,
    max_actions: usize,
    vocab: &Vocab,
    mcts: Option<crate::server::MctsConfig>,
) -> RecordedGame {
    // Pin the bot's tie-break / sampling stream to this game's seed so the
    // game replays from its seed (cross-process determinism; the actors were
    // the one path that never installed one). Per game, so every game still
    // gets its own stream; a caller that pinned it already (antithetic pairs,
    // `--feature-census`) keeps its own.
    let pinned = crate::server::bot::install_jitter_seed_if_unset(seed ^ 0x11_7737_5EED_0001);
    let out = play_recorded_game_mcts_inner(
        template,
        weights,
        seed,
        max_actions,
        vocab,
        mcts,
    );
    if pinned {
        crate::server::bot::set_jitter_seed(None);
    }
    out
}

fn play_recorded_game_mcts_inner(
    template: &GameState,
    weights: [EvalWeights; 2],
    seed: u64,
    max_actions: usize,
    vocab: &Vocab,
    mcts: Option<crate::server::MctsConfig>,
) -> RecordedGame {
    let mut g = template.clone();
    // Profile settings the ENGINE reads off the seat — the same push
    // `recommend::play_one_game_traced` makes. Missing until round 68, so the
    // self-play stream kept caster-first player slots after round 67 fixed
    // them for the ladder.
    for (seat, w) in weights.iter().enumerate() {
        g.players[seat].smart_tap = w.smart_tap;
        g.players[seat].converge_rarest = w.converge_rarest;
        g.players[seat].hostile_player_targets = w.hostile_player_targets;
        // The two `fra-fixes` seat flags (both off in the default), so an
        // actor on `--actor-fra-fixes` plays what the ladder's pilot plays.
        g.players[seat].duel_sinks = w.duel_sinks;
        g.players[seat].tap_timing = w.tap_timing;
        g.players[seat].color_fixing = w.color_fixing;
    }
    let mut rng = StdRng::seed_from_u64(seed);
    for seat in 0..2 {
        g.players[seat].library.shuffle(&mut rng);
    }
    // In-game rolls (mulligan reshuffles, random discards) key off the same
    // seed, derived rather than drawn so the explore draw keeps its position.
    // With the jitter pin above, a re-run of `seed` plays the same game. A
    // whole net-in-the-loop run is still not reproducible (which actor plays
    // game `n`, and so the learner's weights then, varies). `CRAB_NO_JITTER=1`
    // pins the draws to 0 for a measurement.
    g.rng.reseed(seed ^ 0x5EED_600D_C0DE_1234);
    g.start_mulligan_phase();
    // Opening-move exploration: both seats otherwise play one deterministic
    // policy and the net sees only the positions it reaches (self-play
    // distribution collapse). Confined to the opening so outcomes still
    // reflect competent play.
    let explore_plies = rng.random_range(0..=EXPLORE_PLIES);
    // The move-list recorder's candidate sampling (`policy_pilot::record`),
    // its own stream so the game's draws are untouched.
    let mut menu_rng = StdRng::seed_from_u64(seed ^ 0x3E_7A_11_57);
    let mut bots: Vec<Box<dyn Bot>> = weights
        .into_iter()
        .map(|w| -> Box<dyn Bot> {
            if let Some(cfg) = mcts {
                Box::new(crate::server::MctsBot::new(crate::server::MctsConfig { weights: w, ..cfg }))
            } else {
                Box::new(HeuristicBot::with_weights(w))
            }
        })
        .collect();
    let mut explorers: Vec<Box<dyn Bot>> =
        (0..2).map(|_| Box::new(HeuristicBot::uniform_baseline()) as Box<dyn Bot>).collect();

    // (turn, seat, encoded state) — labelled after the game decides.
    //
    // The four parallel accumulators reach ~99 entries a game; one reserve
    // apiece covers the mean game and leaves a long one a single growth
    // (PERF (-110); `snaps` growth memcpys whole `EncodedState`s).
    const SNAP_HINT: usize = 128;
    let mut snaps: Vec<(u32, usize, crabomination_nn::EncodedState)> =
        Vec::with_capacity(SNAP_HINT);
    let mut heur: Vec<i32> = Vec::with_capacity(SNAP_HINT);
    // Raw per-seat stats at each snapshot, parallel to `snaps` — the aux
    // targets are deltas between a seat's consecutive snapshots, so the
    // raw values have to be captured live and diffed at labelling time.
    let mut raw: Vec<[f32; 4]> = Vec::with_capacity(SNAP_HINT);
    // The opponent's held card names per snapshot, parallel to `snaps` —
    // the belief head's target (round 39). Recorded here because only
    // the recorder may look; the encoder never carries it.
    let mut opp_hands: Vec<Vec<u16>> = Vec::with_capacity(SNAP_HINT);
    let mut last_turn = (0u32, usize::MAX);
    let mut last_step = crate::game::TurnStep::Untap;
    // Index of the last pushed pair's seat-0 entry in `snaps` (the de-dup
    // key), rather than a clone of the pair — which cost two whole
    // `EncodedState` copies per snapshot. Snapshots go in as seat pairs and
    // nothing else pushes, so `i` and `i + 1` are the pair.
    let mut last_pair: Option<usize> = None;
    let (mut actions, mut stale) = (0usize, 0usize);
    // The decision recorder's join key (round 72): every captured decision
    // carries the seat's snapshot index at capture time, which is exactly
    // the `ply` the labelling loop below assigns the latest row. Set here
    // and bumped in the snapshot block, so a decision made in iteration k
    // reads the row pushed in iteration k (or the last one pushed before).
    crate::server::decision_capture::set_ply(0);
    while stop_reason(&g, actions, max_actions, stale).is_none() {
        let new_turn = (g.turn_number, g.active_player_idx) != last_turn;
        // The four shapes a combat passes through, each a bot decision's
        // leaf: declare-attackers (the attack decision), declare-blockers
        // (the attack sim's leaf), first-strike/combat damage (the block
        // sim's leaf — the only point the blocking flags exist, since
        // `block_map`/`attacking` are torn down at damage), end-of-combat.
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
            // One frozen-layer scope for the whole snapshot: the two
            // encodes and the two material evals read the same position,
            // and each opened its own scope and rebuilt every permanent's
            // layer view — four gathers and four view sets per snapshot
            // where nested scopes reuse the first (PERF `(-268)`).
            g.with_frozen_layers(|g| {
                let pair = encode_state_pair(g, vocab);
                let repeat = last_pair
                    .is_some_and(|i| snaps[i].2 == pair[0] && snaps[i + 1].2 == pair[1]);
                if !repeat {
                    last_pair = Some(snaps.len());
                    // Snapshots are pushed in seat pairs, so `len / 2`
                    // before the push is this pair's per-seat `ply`.
                    crate::server::decision_capture::set_ply((snaps.len() / 2) as u16);
                    for (seat, s) in pair.into_iter().enumerate() {
                        snaps.push((g.turn_number, seat, s));
                        heur.push(crate::server::bot::eval_material_public(
                            g,
                            seat,
                            &EvalWeights::default(),
                        ));
                        raw.push(snapshot_stats(g, seat));
                        opp_hands.push(
                            g.players[1 - seat]
                                .hand
                                .iter()
                                .map(|c| vocab.index_of(c.definition.name))
                                .filter(|&i| i != 0)
                                .collect(),
                        );
                    }
                }
            });
        }
        last_step = g.step;
        let mut any = false;
        // Uniform picks for the opening window, then the real policy.
        let acting =
            if actions < explore_plies { &mut explorers } else { &mut bots };
        let record_menu = actions >= explore_plies && crate::server::decision_capture::menu_only();
        for (s, bot) in acting.iter_mut().enumerate() {
            let pushed = crate::server::decision_capture::pushed();
            let Some(a) = bot.next_action(&g, s) else { continue };
            // The move-list recorder (`decision_capture::set_menu_only`):
            // the pre-move state, recorded once the engine has taken the move
            // — unless the bot recorded the decision itself (a searched menu
            // window's improved policy, phase 5).
            let pre = (record_menu
                && crate::server::decision_capture::enabled()
                && crate::server::decision_capture::pushed() == pushed)
                .then(|| g.clone());
            let played = a.clone();
            if g.perform_action(a).is_ok() {
                if let Some(pre) = pre {
                    crate::server::policy_pilot::record(&pre, s, &played, &weights[s], &mut menu_rng);
                }
                any = true;
                actions += 1;
                if g.is_game_over() {
                    break;
                }
            }
        }
        if any { stale = 0 } else { stale += 1 }
    }

    crate::server::decision_capture::set_ply(0);
    let turns = g.turn_number;
    // The same `stop_reason` as `recommend::play_one_game_traced`, so the
    // actor and the ladder bound a game identically.
    let stop = stop_reason(&g, actions, max_actions, stale).unwrap_or(StopReason::NoLegalMove);
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
                life_diff: life_diff_label(life[seat], life[1 - seat]),
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

/// The row's life-difference label, in `[-1, 1]`.
///
/// ⚠ `i64`, not `i32`: both totals saturate at the rules layer (`(-291)`'s
/// Beacon board), so their `i32` difference overflows — a panic under the
/// actor's `overflow` profile, a wrong-SIGN label in release.
fn life_diff_label(mine: i32, theirs: i32) -> f32 {
    ((mine as i64 - theirs as i64) as f32 / 20.0).clamp(-1.0, 1.0)
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
            power[side] = power[side].saturating_add(c.power().max(0));
        }
    }
    [
        // The same saturation the encoder guards (`encode::scaled`): a power
        // and a life total are both unbounded rules quantities, so the per-side
        // total and the difference of two of them both saturate here.
        (g.players[seat].life.saturating_sub(g.players[opp].life)) as f32,
        (power[0].saturating_sub(power[1])) as f32,
        (creatures[0] - creatures[1]) as f32,
        g.players[opp].hand.len() as f32,
    ]
}

#[cfg(test)]
mod tests {

    /// A saturated board (Exponential Growth, Beacon of Immortality) goes
    /// through the actor's per-side power sums and life difference without
    /// overflowing; twin of
    /// `encode::tests::a_saturated_power_encodes_as_a_bounded_feature`.
    #[test]
    fn a_saturated_board_does_not_overflow_the_actor_stats() {
        use crate::card::{CardId, CardInstance};
        let mut g = crate::game::two_player_game();
        g.players[0].life = i32::MAX;
        g.players[1].life = i32::MIN + 1;
        for (i, seat) in [(0usize, 0usize), (1, 0), (2, 1)] {
            let mut inst =
                CardInstance::new(CardId(930 + i as u32), crate::catalog::grizzly_bears(), seat);
            inst.controller = seat;
            inst.pump(i32::MAX, i32::MAX);
            inst.pump(i32::MAX, i32::MAX);
            g.battlefield.push(inst);
        }
        for seat in 0..2 {
            let s = super::snapshot_stats(&g, seat);
            assert!(s.iter().all(|x| x.is_finite()), "seat {seat}: {s:?}");
        }
        // And the row label the same board produces stays in range with the
        // sign it should have.
        assert_eq!(super::life_diff_label(i32::MAX, i32::MIN + 1), 1.0);
        assert_eq!(super::life_diff_label(i32::MIN + 1, i32::MAX), -1.0);
    }

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

    /// `--opponent-colors lorehold`: nothing the opponent plays — spell or
    /// land — sits outside the named pair, and the label names it back so
    /// the match is still reproducible from what the client printed.
    #[test]
    fn a_colour_restricted_opponent_plays_only_that_pair() {
        let lorehold: ColorSet = College::Lorehold.colors().iter().collect();
        let (deck, label) = random_sealed_opponent_in_colors(7, 9, lorehold);
        assert_eq!(deck.len(), 40, "a restricted build is still a legal 40");
        assert_eq!(label, "Sealed #7 (9 packs, Lorehold)");
        for &card in &deck {
            let brief = crate::cube::card_brief(card);
            assert!(
                brief.pip_colors.is_subset_of(lorehold),
                "{} is outside Lorehold",
                brief.def.name,
            );
            // Lands carry no pips, so they are the other way a build can
            // reach outside its colours: an off-colour dual from the pool.
            let produced = crate::recommend::land_produced_colors(brief.def);
            assert!(
                produced.is_subset_of(lorehold),
                "{} taps outside Lorehold",
                brief.def.name,
            );
        }
        // The restriction is what chose those colours, not the pool: the
        // same seed under a different college is a different deck.
        let witherbloom: ColorSet = College::Witherbloom.colors().iter().collect();
        let (other, other_label) = random_sealed_opponent_in_colors(7, 9, witherbloom);
        assert_ne!(other, deck);
        assert_eq!(other_label, "Sealed #7 (9 packs, Witherbloom)");

        // A pair that is nobody's college falls back to WUBRG letters, and
        // an unrestricted opponent's label is unchanged.
        let wg: ColorSet = [crate::mana::Color::White, crate::mana::Color::Green].iter().collect();
        let (_, wg_label) = random_sealed_opponent_in_colors(7, SEALED_PACKS, wg);
        assert_eq!(wg_label, "Sealed #7 (WG)");
        let unrestricted = random_sealed_opponent_in_colors(7, SEALED_PACKS, ColorSet::empty());
        assert_eq!(unrestricted.1, "Sealed #7");
    }
    use super::*;

    /// The actor's deck-building path must not panic on a degenerate input.
    /// A panic at game 400 k kills a training run; an empty deck is one
    /// counted loss. `scripts/audit_panics.py` flagged both `expect("n > 0
    /// builds")` sites as bare and they were.
    #[test]
    fn a_degenerate_build_request_yields_an_empty_deck_rather_than_a_panic() {
        let pool = sealed_pool(7);
        assert!(best_build_v3(&pool, 0, 7).is_empty(), "no candidates, no build");
        assert!(best_build_by(&pool, 0, 7, |_| 0.0).is_empty());
        // An empty pool has no playable card in any colour pair, so every
        // shape in the lattice is hollow — the path the removed
        // `expect("enumerated shape rebuilds")` sat on.
        assert!(best_build_v3(&[], 4, 7).is_empty(), "an empty pool builds nothing");
        assert!(best_build_by(&[], 4, 7, |_| 0.0).is_empty());
        // And the normal call still builds a legal 40, so the guards are
        // additions rather than a change of behaviour.
        assert_eq!(best_build_v3(&pool, 4, 7).len(), 40);
    }

    /// The adopted arm and nothing else: an FRA or SOS opponent is the
    /// policy's valued build under its set's same-pool value, as
    /// `fra_card_eval --trial policy-value` built it; a colour restriction
    /// keeps the rated build in those colours.
    #[test]
    fn the_sealed_opponent_is_the_gated_policy_and_value_build() {
        use crate::fra_mode::SealedSet;
        let mut changed = 0;
        for (set, value) in [(SealedSet::Fra, fra_deck_value()), (SealedSet::Sos, sos_deck_value())] {
            for seed in 0..12u64 {
                let pool = sealed_pool_of(set, seed, SEALED_PACKS);
                assert!(sealed_deck_policy().covers(&pool), "{set:?} pool {seed}");
                let deck = sealed_opponent_build(&pool, 16, seed, ColorSet::empty());
                let opts = sealed_valued_opts();
                assert_eq!(deck, sealed_deck_policy().build_valued_with(&pool, value, 16, &mut StdRng::seed_from_u64(seed ^ 3), &opts));
                assert_eq!(deck.len(), 40);
                changed += usize::from(deck != rated_sealed_build(&pool, 16, seed));
            }
        }
        assert!(changed > 12, "the learned builder moves most decks, not {changed} of 24");
        let pool = sealed_pool_of(SealedSet::Fra, 3, SEALED_PACKS);
        let mut wg = ColorSet::empty();
        wg.insert(crate::mana::Color::White);
        wg.insert(crate::mana::Color::Green);
        assert_eq!(sealed_opponent_build(&pool, 16, 3, wg), rated_sealed_build_in_colors(&pool, 16, 3, wg));
    }

    /// The client's draft deck: an SOS pile is the sealed opponent's build
    /// split back into main + sideboard + basics (the pile, card for card),
    /// and a Cube pile keeps the old suggestion.
    #[test]
    fn a_drafted_sos_pile_is_the_sealed_opponents_build() {
        use crate::fra_mode::SealedSet;
        let name = |f: &CardFactory| crate::cube::card_def(*f).name;
        let sorted = |d: &[CardFactory]| {
            let mut v: Vec<&str> = d.iter().map(name).collect();
            v.sort_unstable();
            v
        };
        for seed in 0..2u64 {
            let pile = sealed_pool_of(SealedSet::Sos, seed, 3);
            let built = draft_deck_build(&pile, seed);
            assert_eq!(sorted(&built.deck()), sorted(&sealed_opponent_build(&pile, 16, seed, ColorSet::empty())));
            assert_eq!(built.deck().len(), 40);
            assert!(built.basics.iter().map(|b| b.1).sum::<u32>() >= 14, "{:?}", built.basics);
            let mut both = built.main.clone();
            both.extend(&built.sideboard);
            assert_eq!(sorted(&both), sorted(&pile), "main + sideboard is the pile");
            assert!(built.basics.iter().any(|b| b.0 == built.colors[0]));
        }
        let cube: Vec<CardFactory> = crate::draft::draft_pool().into_iter().take(45).collect();
        let built = draft_deck_build(&cube, 1);
        assert_eq!(built.main, crate::draft::suggest_main_deck(&cube, 23).0);
    }

    /// The client's sealed opponent opts into `curve_aggro`, so it has early
    /// plays: filling strictly by score, a pool of expensive bombs once
    /// produced an opponent that did nothing before turn four.
    #[test]
    fn the_sealed_opponent_builds_to_a_curve() {
        for seed in [1u64, 7, 42, 1234] {
            let (deck, _) = random_sealed_opponent_in_colors(
                seed,
                crate::selfplay::SEALED_PACKS,
                crate::mana::ColorSet::empty(),
            );
            assert_eq!(deck.len(), 40, "seed {seed}: a sealed deck is 40 cards");
            let spells: Vec<u32> = deck
                .iter()
                .map(|&f| crate::cube::card_brief(f))
                .filter(|b| !b.is_land)
                .map(|b| b.cmc)
                .collect();
            let early = spells.iter().filter(|&&c| c <= 2).count();
            let top = spells.iter().filter(|&&c| c >= 5).count();
            // `CurveSlots`' floor scales with the spell count; at the usual
            // 23 it is five. The second pass can leave the reserve short
            // only when the pool genuinely lacks cheap playables, which a
            // full sealed pool does not.
            assert!(
                early >= 4,
                "seed {seed}: {early} early plays in {} spells (top {top})",
                spells.len(),
            );
            assert!(top <= 8, "seed {seed}: {top} five-plus-drops is top-heavy");
        }
    }

    /// The judge polish takes an upgrade the lands can cast and never one
    /// they can't: a Forest deck swaps a Bear for Craw Wurm, not Serra Angel,
    /// though the judge (summed mana value here) likes both.
    #[test]
    fn judge_polish_swaps_only_what_the_lands_cast() {
        use crate::catalog;
        let mut deck: Vec<CardFactory> = vec![catalog::forest; 17];
        deck.extend(std::iter::repeat_n(catalog::grizzly_bears as CardFactory, 23));
        let mut pool = deck.clone();
        pool.push(catalog::craw_wurm);
        pool.push(catalog::serra_angel);
        let judge = |d: &[CardFactory]| d.iter().map(|f| crate::cube::card_def(*f).cost.cmc() as f64).sum::<f64>();
        let out = judge_polish(&pool, deck, judge);
        let n = |f: CardFactory| out.iter().filter(|&&g| g as usize == f as usize).count();
        assert_eq!(out.len(), 40);
        assert_eq!(n(catalog::craw_wurm), 1, "the green upgrade went in");
        assert_eq!(n(catalog::serra_angel), 0, "no white card off seventeen Forests");
        assert_eq!(n(catalog::forest), 17, "lands untouched");
    }

    /// The client's sealed-opponent recipe: best-of-16 under the v3 judge is
    /// a legal 40, deterministic in the seed, and the argmax of its own
    /// candidates — not one more sample.
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
        // Pools and builds are seed-determined, and so is the game now that
        // the wrapper pins the bot's jitter stream per game.
        let pool_again = sealed_pool(0xA11CE);
        let names = |d: &[CardFactory]| d.iter().map(|f| f().name).collect::<Vec<_>>();
        assert_eq!(names(&pool_again), names(&pool_a));
        assert_eq!(
            names(&heuristic_sealed_build(&pool_again, 1)),
            names(&deck_a)
        );
    }

    /// Recording the move list (`decision_capture::set_menu_only`) doesn't
    /// change the game: the recorder runs between moves, on clones. (Written
    /// after a one-off 2026-11-03 failure of
    /// `the_same_seed_records_the_same_game` beside the capture tests; this
    /// rules the recorder out.)
    #[test]
    fn recording_the_move_list_does_not_change_the_game() {
        use crate::server::decision_capture;
        let _serial = decision_capture::TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let vocab = Vocab::sos_sealed();
        let deck_a = heuristic_sealed_build(&sealed_pool(0xA11CE), 1);
        let deck_b = heuristic_sealed_build(&sealed_pool(0xB0B), 2);
        let template = sealed_game_template(&deck_a, &deck_b);
        let play = || play_recorded_game(&template, [EvalWeights::default(), EvalWeights::default()], 78, 4000, &vocab);
        let plain = play();
        decision_capture::set_enabled(true);
        decision_capture::set_menu_only(true);
        let recorded = play();
        let rows = decision_capture::drain();
        decision_capture::set_menu_only(false);
        decision_capture::set_enabled(false);
        assert!(!rows.is_empty(), "the recorder recorded");
        assert_eq!((plain.winner, plain.turns), (recorded.winner, recorded.turns));
        assert!(plain.rows == recorded.rows, "every position replays identically");
    }

    /// A recorded game replays itself: the bot's tie-break and sampling draws
    /// were once unseeded on this path, so a crash at game 400 k couldn't be
    /// re-run. `play_recorded_game_mcts` pins the stream per game.
    #[test]
    fn the_same_seed_records_the_same_game() {
        let vocab = Vocab::sos_sealed();
        let deck_a = heuristic_sealed_build(&sealed_pool(0xA11CE), 1);
        let deck_b = heuristic_sealed_build(&sealed_pool(0xB0B), 2);
        let template = sealed_game_template(&deck_a, &deck_b);
        let play = || {
            play_recorded_game(
                &template,
                [EvalWeights::default(), EvalWeights::default()],
                77,
                4000,
                &vocab,
            )
        };
        let a = play();
        let b = play();
        assert_eq!((a.winner, a.turns, a.rows.len()), (b.winner, b.turns, b.rows.len()));
        assert!(a.rows == b.rows, "every recorded position replays identically");
        assert!(a.heur == b.heur, "so does every heuristic row");
    }

    /// Round 72: every decision the recorder captures carries the ply of
    /// the seat's latest row, so a policy-gradient target can be joined
    /// to the value stream. The join has to be in range, chronological
    /// (decisions drain in play order, rows are pushed in play order),
    /// and the cell has to be reset for the next game.
    #[test]
    fn decision_ply_joins_to_the_seat_rows() {
        use crate::server::decision_capture as cap;
        let _serial = cap::TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let vocab = Vocab::sos_sealed();
        let deck_a = heuristic_sealed_build(&sealed_pool(0xA11CE), 1);
        let deck_b = heuristic_sealed_build(&sealed_pool(0xB0B), 2);
        let template = sealed_game_template(&deck_a, &deck_b);
        let _ = cap::drain();
        cap::set_enabled(true);
        let rec = play_recorded_game(
            &template,
            [EvalWeights::default(), EvalWeights::default()],
            77,
            4000,
            &vocab,
        );
        let decisions = cap::drain();
        cap::set_enabled(false);
        assert!(rec.winner.is_some(), "fixture: the game decides");
        assert!(!decisions.is_empty(), "a recorded game captures decisions");
        let rows_of = |seat: usize| rec.rows.iter().filter(|r| r.traj & 1 == seat as u32).count();
        let mut last_ply = [0u16; 2];
        for d in &decisions {
            assert!(
                (d.ply as usize) < rows_of(d.seat),
                "seat {} decision at ply {} but only {} rows",
                d.seat,
                d.ply,
                rows_of(d.seat)
            );
            assert!(d.ply >= last_ply[d.seat], "plies run forward within a seat");
            last_ply[d.seat] = d.ply;
            assert!(d.scores.is_some(), "the live pickers record their scores");
        }
        assert!(
            decisions.iter().any(|d| d.seat == 0) && decisions.iter().any(|d| d.seat == 1),
            "both seats decide"
        );
        assert_eq!(cap::ply(), 0, "the recorder resets the cell after the game");
    }
}
