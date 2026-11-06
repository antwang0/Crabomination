//! `deck_policy_train` — step 2 of the deck-building net (ML_NOTES "A deck
//! net that builds decks"): the add-one-card builder, by imitation of the
//! rated builder.
//!
//! Rolls `--pools` sealed pools per set (FRA and SOS boosters, seeds disjoint
//! from `fra_card_eval`'s gate seeds), builds each with the rated builder as
//! it builds the bot's opponents today (`rated_sealed_build`, best of 16,
//! polished), and trains the policy `crabomination::deck_policy` reads: from
//! the pool and a partial deck, which pool card next (or stop). A target deck
//! is a set, so each training state is a random-order prefix of its spells,
//! and any remaining target card — or stop, once complete — counts as right.
//! One pool in twenty is held out.
//!
//! Reports held-out loss and next-step accuracy as it trains; at the end
//! exports the best policy (`--export`, checked against the engine's reading
//! of it) and reads, on the held-out pools, the greedy build's spell overlap
//! with the teacher's and how often the land step reproduces the teacher's
//! lands from the teacher's own spells.
//!
//! ```text
//! deck_policy_train --pools 40000 --steps 30000 --export .ladder/deckpol/policy.json
//! ```

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use candle_core::{D, DType, Device, Result as CResult, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use crabomination::cube::CardFactory;
use crabomination::deck_policy::{DeckPolicy, MAX_SPELLS, MIN_SPELLS};
use crabomination::deck_value::Layer;
use crabomination::fra_mode::SealedSet;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

const FEATS: usize = 59;
const EMB: usize = 32;
const H: usize = 64;
const STATE_IN: usize = 2 * H + 16 + 1;

struct Args {
    pools: usize,
    steps: usize,
    batch: usize,
    lr: f64,
    seed: u64,
    threads: usize,
    export: std::path::PathBuf,
    /// Skip training: read this policy and run the offline read on it.
    load: Option<std::path::PathBuf>,
    /// Step 2b: after `warmup` steps, every `dagger_every` the policy builds
    /// `dagger_pools` training pools and trains on the states it reaches.
    dagger: bool,
    warmup: usize,
    dagger_every: usize,
    dagger_pools: usize,
    /// Step 3: generations of expert iteration after the step-2b recipe.
    expert_iter: usize,
    race_fra: usize,
    race_sos: usize,
    gen_steps: usize,
    /// Step 3b: the final confirmation a replacement must also pass.
    final_pairs: usize,
    final_accept: f64,
    /// Step 3c: also race the greedy build held to each colour pair, and
    /// confirm this many screen leaders.
    pair_cands: bool,
    confirm_top: usize,
    /// Start step 3 from these raw weights (`*_gen0.safetensors`), skipping
    /// the step-2b recipe.
    init_vars: Option<std::path::PathBuf>,
}

fn parse_args() -> Args {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1).cloned());
    let num = |k: &str, d: usize| get(k).map_or(d, |v| v.parse().expect(k));
    Args {
        pools: num("--pools", 40_000),
        steps: num("--steps", 30_000),
        batch: num("--batch", 256),
        lr: get("--lr").map_or(1e-3, |v| v.parse().expect("--lr")),
        seed: num("--seed", 1) as u64,
        threads: num("--threads", std::thread::available_parallelism().map_or(8, |n| n.get())),
        export: get("--export").expect("--export FILE.json").into(),
        load: get("--load").map(Into::into),
        dagger: a.iter().any(|x| x == "--dagger"),
        warmup: num("--warmup", 5_000),
        dagger_every: num("--dagger-every", 5_000),
        dagger_pools: num("--dagger-pools", 20_000),
        expert_iter: num("--expert-iter", 0),
        race_fra: num("--race-fra", 8_000),
        race_sos: num("--race-sos", 3_000),
        gen_steps: num("--gen-steps", 10_000),
        final_pairs: num("--final-pairs", 0),
        final_accept: get("--final-accept").map_or(56.0, |v| v.parse().expect("--final-accept")),
        init_vars: get("--init-vars").map(Into::into),
        pair_cands: a.iter().any(|x| x == "--pair-cands"),
        confirm_top: num("--confirm-top", 2),
    }
}

/// What `encode_deck` reads of a card, so a partial deck's features need no
/// definition built per state.
#[derive(Clone, Copy)]
struct Stats {
    land: bool,
    creature: bool,
    mv: u32,
    pips: [u32; 5],
}

struct Cards {
    names: Vec<String>,
    feats: Vec<f32>,
    stats: Vec<Stats>,
    index: HashMap<&'static str, usize>,
}

fn cards() -> Cards {
    use crabomination::mana::Color;
    let fra = crabomination::fra_mode::FraPacks::get();
    let sheet: Vec<CardFactory> = fra
        .common
        .iter()
        .chain(&fra.uncommon)
        .chain(&fra.rare)
        .chain(&fra.mythic)
        .chain(crabomination::draft::sos_draft_pool_ref())
        .copied()
        .collect();
    let vocab = crabomination::server::net_eval::vocab();
    let mut c = Cards { names: Vec::new(), feats: Vec::new(), stats: Vec::new(), index: HashMap::new() };
    for f in sheet {
        let def = f();
        if c.index.contains_key(def.name) {
            continue;
        }
        c.index.insert(def.name, c.names.len());
        c.names.push(def.name.to_string());
        let mut pips = [0u32; 5];
        for col in def.cost.colored_symbols() {
            pips[match col {
                Color::White => 0,
                Color::Blue => 1,
                Color::Black => 2,
                Color::Red => 3,
                Color::Green => 4,
            }] += 1;
        }
        c.stats.push(Stats { land: def.is_land(), creature: def.is_creature(), mv: def.cost.cmc(), pips });
        c.feats.extend_from_slice(&crabomination::server::encode::encode_deck_card(def, vocab).feats);
    }
    c
}

/// `encode_deck`'s 16 features plus the size term, from the cards' stats.
fn deck_feats(deck: &[usize], st: &[Stats]) -> [f32; 17] {
    let mut f = [0f32; 17];
    let (mut lands, mut creatures, mut mv_sum, mut spells) = (0u32, 0u32, 0u32, 0u32);
    let mut pips = [0u32; 5];
    for &i in deck {
        let s = st[i];
        if s.land {
            lands += 1;
            continue;
        }
        spells += 1;
        mv_sum += s.mv;
        f[(s.mv.clamp(1, 7) - 1) as usize] += 1.0 / 8.0;
        creatures += u32::from(s.creature);
        for (p, q) in pips.iter_mut().zip(s.pips) {
            *p += q;
        }
    }
    f[7] = lands as f32 / 17.0;
    f[8] = creatures as f32 / 23.0;
    for (k, p) in pips.iter().enumerate() {
        f[9 + k] = *p as f32 / 20.0;
    }
    f[14] = pips.iter().filter(|&&p| p > 0).count() as f32 / 3.0;
    f[15] = if spells > 0 { mv_sum as f32 / spells as f32 / 4.0 } else { 0.0 };
    f[16] = deck.len() as f32 / 23.0;
    f
}

struct PoolRow {
    set: SealedSet,
    pool: Vec<CardFactory>,
    /// Distinct non-land known pool cards and their copies, first-seen order.
    avail: Vec<(usize, u32)>,
    /// The expert's spells, as card indices — the teacher's, until step 3's
    /// races replace them — and its deck as built.
    target: std::sync::RwLock<Vec<usize>>,
    best: std::sync::RwLock<Vec<CardFactory>>,
    /// The rated builder's deck, as built.
    teacher: Vec<CardFactory>,
    held_out: bool,
    index: usize,
}

fn roll(set: SealedSet, i: usize, cards: &Cards) -> PoolRow {
    let seed = 0x00DE_C4B0_0000_0000 ^ ((set == SealedSet::Fra) as u64) << 40 ^ i as u64;
    let pool = crabomination::selfplay::sealed_pool_of(set, seed, crabomination::selfplay::SEALED_PACKS);
    let teacher = crabomination::selfplay::rated_sealed_build(&pool, 16, seed ^ 1);
    let mut avail: Vec<(usize, u32)> = Vec::new();
    for f in &pool {
        let def = crabomination::cube::card_def(*f);
        if def.is_land() {
            continue;
        }
        let Some(&k) = cards.index.get(def.name) else { continue };
        match avail.iter_mut().find(|a| a.0 == k) {
            Some(a) => a.1 += 1,
            None => avail.push((k, 1)),
        }
    }
    let target = teacher
        .iter()
        .filter(|f| !crabomination::cube::card_def(**f).is_land())
        .filter_map(|f| cards.index.get(crabomination::cube::card_def(*f).name).copied())
        .collect();
    PoolRow {
        set,
        pool,
        avail,
        target: std::sync::RwLock::new(target),
        best: std::sync::RwLock::new(teacher.clone()),
        teacher,
        held_out: i.is_multiple_of(20),
        index: i,
    }
}

impl PoolRow {
    /// The rated builder's spells as card indices (what the offline reads
    /// compare against, whatever step 3 has relabelled).
    fn teacher_target(&self, cards: &Cards) -> Vec<usize> {
        let spells: Vec<CardFactory> = self.teacher.iter().copied().filter(|f| !crabomination::cube::card_def(*f).is_land()).collect();
        indices(&spells, cards)
    }
}

/// The pilot `fra_card_eval`'s gates play each set with.
fn pilot_for(set: SealedSet) -> crabomination::recommend::Pilot {
    use crabomination::server::EvalWeights;
    crabomination::recommend::Pilot::Scored(match set {
        SealedSet::Fra => EvalWeights::fra_fixes_on(),
        _ => EvalWeights::default(),
    })
}

fn share(t: &crabomination::recommend::PairedTally) -> Option<f64> {
    let n = t.wins_a + t.wins_b;
    (n > 0).then(|| 100.0 * t.wins_a as f64 / n as f64)
}

fn deck_key(d: &[CardFactory]) -> Vec<usize> {
    let mut v: Vec<usize> = d.iter().map(|f| *f as usize).collect();
    v.sort_unstable();
    v
}

/// One raced pool: its set, the replacement's confirmed share, the
/// leader's final share, and whether the replacement was a pair build.
type RaceRow = (SealedSet, Option<f64>, Option<f64>, bool);

/// Step 3's expert: on each row, the policy's greedy build and seven
/// sampled ones (temperatures 1, 1, 1, 1.5, 1.5, 2, 2), deduplicated, race
/// the row's current best deck — four pairs each, then sixteen fresh pairs
/// for the top two — and the confirmed leader replaces it at ≥ `accept` %.
/// Returns, per set, (rows raced, rows replaced, mean confirmed share of
/// the replacements).
/// With `final_pairs > 0` (step 3b) the leader must also clear
/// `final_accept` % over that many fresh pairs; the last tuple element is
/// then the mean final share of every leader that reached it — near 50 if
/// the first two stages pass ties.
/// With `pair_cands` (step 3c) the candidates also include the greedy build
/// held to each colour pair ([`DeckPolicy::build_in`]), and the top
/// `confirm_top` screen leaders are confirmed; the last element counts the
/// replacements that were pair builds.
#[allow(clippy::too_many_arguments)]
fn race_generation(
    policy: &DeckPolicy,
    rows: &[&PoolRow],
    cards: &Cards,
    seed: u64,
    threads: usize,
    accept: f64,
    final_stage: (usize, f64),
    pair_cands: bool,
    confirm_top: usize,
) -> Vec<(SealedSet, usize, usize, f64, usize, f64, usize)> {
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<RaceRow>> = Mutex::new(Vec::new());
    let t0 = std::time::Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::Relaxed);
                if j >= rows.len() {
                    break;
                }
                let row = rows[j];
                let pilot = pilot_for(row.set);
                let salt = seed ^ (row.index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((row.set == SealedSet::Fra) as u64) << 62;
                let mut rng = StdRng::seed_from_u64(salt);
                let reference = row.best.read().unwrap().clone();
                let mut seen = vec![deck_key(&reference)];
                let mut cands: Vec<Vec<CardFactory>> = Vec::new();
                for t in [None, Some(1.0), Some(1.0), Some(1.0), Some(1.5), Some(1.5), Some(2.0), Some(2.0)] {
                    let d = match t {
                        None => policy.build(&row.pool, None),
                        Some(t) => policy.build(&row.pool, Some((&mut rng, t))),
                    };
                    let k = deck_key(&d);
                    if !seen.contains(&k) {
                        seen.push(k);
                        cands.push(d);
                    }
                }
                let sampled = cands.len();
                if pair_cands {
                    use crabomination::mana::Color;
                    for (a, &c1) in Color::ALL.iter().enumerate() {
                        for &c2 in &Color::ALL[a + 1..] {
                            if let Some(d) = policy.build_in(&row.pool, [c1, c2]) {
                                let k = deck_key(&d);
                                if !seen.contains(&k) {
                                    seen.push(k);
                                    cands.push(d);
                                }
                            }
                        }
                    }
                }
                let mut screened: Vec<(f64, usize)> = cands
                    .iter()
                    .enumerate()
                    .filter_map(|(i, d)| {
                        let t = crabomination::recommend::simulate_match_pairs_piloted(d, &reference, 4, [pilot, pilot], 50_000, salt ^ 0x5C);
                        share(&t).map(|x| (x, i))
                    })
                    .collect();
                screened.sort_by(|a, b| b.0.total_cmp(&a.0));
                let mut won: Option<(f64, usize)> = None;
                for &(_, i) in screened.iter().take(confirm_top) {
                    let t = crabomination::recommend::simulate_match_pairs_piloted(&cands[i], &reference, 16, [pilot, pilot], 50_000, salt ^ 0xC0F);
                    if let Some(x) = share(&t)
                        && x >= accept
                        && won.is_none_or(|w| x > w.0)
                    {
                        won = Some((x, i));
                    }
                }
                let mut finalist: Option<f64> = None;
                if let Some((_, i)) = won
                    && final_stage.0 > 0
                {
                    let t = crabomination::recommend::simulate_match_pairs_piloted(&cands[i], &reference, final_stage.0, [pilot, pilot], 50_000, salt ^ 0xF1A1);
                    finalist = share(&t);
                    if finalist.is_none_or(|x| x < final_stage.1) {
                        won = None;
                    }
                }
                if let Some((_, i)) = won {
                    let spells: Vec<CardFactory> = cands[i].iter().copied().filter(|f| !crabomination::cube::card_def(*f).is_land()).collect();
                    *row.target.write().unwrap() = indices(&spells, cards);
                    *row.best.write().unwrap() = cands[i].clone();
                }
                let mut o = out.lock().unwrap();
                o.push((row.set, won.map(|w| w.0), finalist, won.is_some_and(|w| w.1 >= sampled)));
                if o.len().is_multiple_of(1000) {
                    eprintln!("  raced {}/{} pools, {:.0}s", o.len(), rows.len(), t0.elapsed().as_secs_f64());
                }
            });
        }
    });
    let out = out.into_inner().unwrap();
    [SealedSet::Fra, SealedSet::Sos]
        .into_iter()
        .map(|set| {
            let mine: Vec<&RaceRow> = out.iter().filter(|r| r.0 == set).collect();
            let won: Vec<f64> = mine.iter().filter_map(|r| r.1).collect();
            let fin: Vec<f64> = mine.iter().filter_map(|r| r.2).collect();
            let pair_won = mine.iter().filter(|r| r.3).count();
            (set, mine.len(), won.len(), won.iter().sum::<f64>() / won.len().max(1) as f64, fin.len(), fin.iter().sum::<f64>() / fin.len().max(1) as f64, pair_won)
        })
        .collect()
}

/// The held-out play read: the greedy build against the rated builder's
/// deck, four pairs a pool, under the gate's pilot — per set, the mean
/// pool-level share and its 95 % half-width.
fn play_read(policy: &DeckPolicy, rows: &[&PoolRow], threads: usize) -> Vec<(SealedSet, f64, f64)> {
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<(SealedSet, f64)>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::Relaxed);
                if j >= rows.len() {
                    break;
                }
                let row = rows[j];
                let pilot = pilot_for(row.set);
                let d = policy.build(&row.pool, None);
                let t = crabomination::recommend::simulate_match_pairs_piloted(&d, &row.teacher, 4, [pilot, pilot], 50_000, 0x9EAD ^ row.index as u64);
                if let Some(x) = share(&t) {
                    out.lock().unwrap().push((row.set, x));
                }
            });
        }
    });
    let out = out.into_inner().unwrap();
    [SealedSet::Fra, SealedSet::Sos]
        .into_iter()
        .map(|set| {
            let xs: Vec<f64> = out.iter().filter(|r| r.0 == set).map(|r| r.1).collect();
            let n = xs.len().max(2) as f64;
            let m = xs.iter().sum::<f64>() / n;
            let sd = (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
            (set, m, 1.96 * sd / n.sqrt())
        })
        .collect()
}

/// One state: a partial deck in a pool, and the teacher's cards it still
/// lacks (empty: stop is the answer).
#[derive(Clone)]
struct State<'a> {
    row: &'a PoolRow,
    deck: Vec<usize>,
    remaining: Vec<usize>,
}

fn sample_state<'a>(row: &'a PoolRow, rng: &mut StdRng) -> State<'a> {
    let mut order = row.target.read().unwrap().clone();
    order.shuffle(rng);
    let k = rng.random_range(0..=order.len());
    let remaining = order.split_off(k);
    State { row, deck: order, remaining }
}

/// The teacher's cards `deck` has not taken; none once it holds as many
/// spells as the teacher, so stop is the answer there. DAgger's label for a
/// state off the teacher's path: the expert's own completion.
fn remaining_for(target: &[usize], deck: &[usize]) -> Vec<usize> {
    if deck.len() >= target.len() {
        return Vec::new();
    }
    let mut left = target.to_vec();
    for d in deck {
        if let Some(p) = left.iter().position(|x| x == d) {
            left.swap_remove(p);
        }
    }
    left
}

/// Whether a state's label is reachable: stop allowed when it is the
/// answer, else some target card still in the pool.
fn labelled(s: &State) -> bool {
    if s.remaining.is_empty() {
        return s.deck.len() >= MIN_SPELLS;
    }
    s.remaining.iter().any(|&i| {
        let have = s.row.avail.iter().find(|a| a.0 == i).map_or(0, |a| a.1) as usize;
        have > s.deck.iter().filter(|&&d| d == i).count()
    })
}

fn indices(spells: &[CardFactory], cards: &Cards) -> Vec<usize> {
    spells.iter().filter_map(|f| cards.index.get(crabomination::cube::card_def(*f).name).copied()).collect()
}

/// Every state `policy` passes through building `rows`' pools, once greedy
/// and once at temperature 1, each labelled by [`remaining_for`].
fn roll_states<'a>(policy: &DeckPolicy, rows: &[&'a PoolRow], cards: &Cards, seed: u64, threads: usize) -> Vec<State<'a>> {
    let out = Mutex::new(Vec::new());
    let next = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                let mut mine = Vec::new();
                loop {
                    let j = next.fetch_add(1, Ordering::Relaxed);
                    if j >= rows.len() {
                        break;
                    }
                    let row = rows[j];
                    let mut rng = StdRng::seed_from_u64(seed ^ (row.index as u64) << 1 ^ (row.set == SealedSet::Fra) as u64);
                    for sampled in [false, true] {
                        let spells = policy.spells(&row.pool, sampled.then_some((&mut rng, 1.0)));
                        let idx = indices(&spells, cards);
                        for k in 0..=idx.len().min(MAX_SPELLS - 1) {
                            let deck = idx[..k].to_vec();
                            let st = State { row, remaining: remaining_for(&row.target.read().unwrap(), &deck), deck };
                            if labelled(&st) {
                                mine.push(st);
                            }
                        }
                    }
                }
                out.lock().unwrap().extend(mine);
            });
        }
    });
    out.into_inner().unwrap()
}

/// Mean greedy spell overlap with the teacher over `rows` — the step-2b
/// checkpoint metric.
fn greedy_overlap(policy: &DeckPolicy, rows: &[&PoolRow], cards: &Cards) -> f64 {
    let total: f64 = rows
        .iter()
        .map(|r| {
            let mut left = r.teacher_target(cards);
            let mut hit = 0usize;
            for i in indices(&policy.spells(&r.pool, None), cards) {
                if let Some(p) = left.iter().position(|&x| x == i) {
                    left.swap_remove(p);
                    hit += 1;
                }
            }
            hit as f64 / r.teacher_target(cards).len().max(1) as f64
        })
        .sum();
    total / rows.len().max(1) as f64
}

/// A batch's tensors; candidates padded to the batch's widest pool.
struct Batch {
    pool_counts: Tensor,
    deck_counts: Tensor,
    dfeats: Tensor,
    cand_idx: Tensor,
    cand_extra: Tensor,
    valid: Tensor,
    target: Tensor,
    b: usize,
    k: usize,
    /// Per state, the card index behind each candidate slot.
    slots: Vec<Vec<usize>>,
}

fn batch(states: &[State], c: usize, st: &[Stats], dev: &Device) -> CResult<Batch> {
    let b = states.len();
    let k = states.iter().map(|s| s.row.avail.len()).max().unwrap_or(1).max(1);
    let mut pool_counts = vec![0f32; b * c];
    let mut deck_counts = vec![0f32; b * c];
    let mut dfeats = vec![0f32; b * 17];
    let mut cand_idx = vec![0u32; b * k];
    let mut cand_extra = vec![0f32; b * k * 2];
    let mut valid = vec![0f32; b * (k + 1)];
    let mut target = vec![0f32; b * (k + 1)];
    let mut slots = Vec::with_capacity(b);
    for (r, s) in states.iter().enumerate() {
        for &(i, n) in &s.row.avail {
            pool_counts[r * c + i] += n as f32;
        }
        for &i in &s.deck {
            deck_counts[r * c + i] += 1.0;
        }
        dfeats[r * 17..(r + 1) * 17].copy_from_slice(&deck_feats(&s.deck, st));
        let mut slot = Vec::new();
        for (j, &(i, n)) in s.row.avail.iter().enumerate() {
            let used = s.deck.iter().filter(|&&d| d == i).count() as f32;
            let left = n as f32 - used;
            cand_idx[r * k + j] = i as u32;
            cand_extra[(r * k + j) * 2] = used / 2.0;
            cand_extra[(r * k + j) * 2 + 1] = left / 2.0;
            if left > 0.0 {
                valid[r * (k + 1) + j] = 1.0;
                target[r * (k + 1) + j] = f32::from(s.remaining.contains(&i));
            }
            slot.push(i);
        }
        slots.push(slot);
        let stop_ok = s.deck.len() >= MIN_SPELLS;
        valid[r * (k + 1) + k] = f32::from(stop_ok);
        target[r * (k + 1) + k] = f32::from(s.remaining.is_empty());
    }
    Ok(Batch {
        pool_counts: Tensor::from_vec(pool_counts, (b, c), dev)?,
        deck_counts: Tensor::from_vec(deck_counts, (b, c), dev)?,
        dfeats: Tensor::from_vec(dfeats, (b, 17), dev)?,
        cand_idx: Tensor::from_vec(cand_idx, b * k, dev)?,
        cand_extra: Tensor::from_vec(cand_extra, (b, k, 2), dev)?,
        valid: Tensor::from_vec(valid, (b, k + 1), dev)?,
        target: Tensor::from_vec(target, (b, k + 1), dev)?,
        b,
        k,
        slots,
    })
}

struct Lin {
    w: Var,
    b: Var,
}

impl Lin {
    fn new(i: usize, o: usize, rng: &mut StdRng, dev: &Device) -> CResult<Self> {
        let s = (1.0 / i as f32).sqrt();
        let w = Tensor::from_vec((0..o * i).map(|_| rng.random_range(-s..s)).collect::<Vec<f32>>(), (o, i), dev)?;
        Ok(Self { w: Var::from_tensor(&w)?, b: Var::zeros(o, DType::F32, dev)? })
    }
    fn fwd(&self, x: &Tensor) -> CResult<Tensor> {
        x.matmul(&self.w.as_tensor().t()?)?.broadcast_add(self.b.as_tensor())
    }
    fn export(&self) -> CResult<Layer> {
        Ok(Layer { w: self.w.as_tensor().to_vec2()?, b: self.b.as_tensor().to_vec1()? })
    }
}

struct Model {
    emb: Var,
    feats: Tensor,
    enc: Lin,
    state: Lin,
    cand1: Lin,
    cand2: Lin,
    stop1: Lin,
    stop2: Lin,
}

impl Model {
    fn vars(&self) -> Vec<Var> {
        let mut v = vec![self.emb.clone()];
        for l in [&self.enc, &self.state, &self.cand1, &self.cand2, &self.stop1, &self.stop2] {
            v.push(l.w.clone());
            v.push(l.b.clone());
        }
        v
    }
    fn x(&self) -> CResult<Tensor> {
        self.enc.fwd(&Tensor::cat(&[self.emb.as_tensor(), &self.feats], 1)?)?.relu()
    }
    /// Masked logits `[B, K + 1]` (the last column is stop).
    fn logits(&self, bt: &Batch) -> CResult<Tensor> {
        let x = self.x()?;
        let pool = bt.pool_counts.matmul(&x)?.broadcast_div(&bt.pool_counts.sum_keepdim(1)?.maximum(1f32)?)?;
        let deck = bt.deck_counts.matmul(&x)?.broadcast_div(&bt.deck_counts.sum_keepdim(1)?.maximum(1f32)?)?;
        let u = self.state.fwd(&Tensor::cat(&[&pool, &deck, &bt.dfeats], 1)?)?.relu()?;
        let xc = x.index_select(&bt.cand_idx, 0)?.reshape((bt.b, bt.k, H))?;
        let ub = u.unsqueeze(1)?.broadcast_as((bt.b, bt.k, H))?;
        let cin = Tensor::cat(&[&xc, &ub, &bt.cand_extra], 2)?.reshape((bt.b * bt.k, 2 * H + 2))?;
        let s = self.cand2.fwd(&self.cand1.fwd(&cin)?.relu()?)?.reshape((bt.b, bt.k))?;
        let stop = self.stop2.fwd(&self.stop1.fwd(&u)?.relu()?)?;
        let l = Tensor::cat(&[&s, &stop], 1)?;
        l + ((&bt.valid - 1.0)? * 1e9)?
    }
}

fn logsumexp(x: &Tensor) -> CResult<Tensor> {
    let m = x.max_keepdim(D::Minus1)?;
    (x.broadcast_sub(&m)?.exp()?.sum_keepdim(D::Minus1)?.log()? + m)?.squeeze(D::Minus1)
}

/// `(loss, accuracy)`: −log of the probability on the target set, and how
/// often the argmax is in it.
fn loss(m: &Model, bt: &Batch) -> CResult<(Tensor, f64)> {
    let l = m.logits(bt)?;
    let t = (&l + ((&bt.target - 1.0)? * 1e9)?)?;
    let loss = (logsumexp(&l)? - logsumexp(&t)?)?.mean_all()?;
    let arg: Vec<u32> = l.argmax(D::Minus1)?.to_vec1()?;
    let tgt: Vec<Vec<f32>> = bt.target.to_vec2()?;
    let hit = arg.iter().zip(&tgt).filter(|(a, t)| t[**a as usize] > 0.5).count();
    Ok((loss, hit as f64 / bt.b as f64))
}

fn export(m: &Model, cards: &Cards) -> CResult<DeckPolicy> {
    Ok(DeckPolicy::new(
        cards.names.clone(),
        m.x()?.to_vec2()?,
        m.state.export()?,
        m.cand1.export()?,
        m.cand2.export()?,
        m.stop1.export()?,
        m.stop2.export()?,
    ))
}

fn main() -> CResult<()> {
    let args = parse_args();
    let dev = Device::Cpu;
    let cards = cards();
    let c = cards.names.len();
    eprintln!("{c} cards on the two sheets");

    // The teacher's decks, both sets, in parallel.
    let t0 = std::time::Instant::now();
    let jobs: Vec<(SealedSet, usize)> =
        [SealedSet::Fra, SealedSet::Sos].into_iter().flat_map(|s| (0..args.pools).map(move |i| (s, i))).collect();
    let next = AtomicUsize::new(0);
    let rows: Mutex<Vec<(usize, PoolRow)>> = Mutex::new(Vec::with_capacity(jobs.len()));
    std::thread::scope(|s| {
        for _ in 0..args.threads {
            s.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::Relaxed);
                if j >= jobs.len() {
                    break;
                }
                let (set, i) = jobs[j];
                let row = roll(set, i, &cards);
                let mut r = rows.lock().unwrap();
                r.push((j, row));
                if r.len().is_multiple_of(10_000) {
                    eprintln!("{} teacher builds, {:.0}s", r.len(), t0.elapsed().as_secs_f64());
                }
            });
        }
    });
    let mut rows = rows.into_inner().unwrap();
    rows.sort_by_key(|r| r.0);
    let rows: Vec<PoolRow> = rows.into_iter().map(|r| r.1).collect();
    // Teachers outside the policy's stop window cannot be imitated exactly:
    // kept out of training, counted.
    let usable = |r: &PoolRow| (MIN_SPELLS..=MAX_SPELLS).contains(&r.target.read().unwrap().len());
    let train: Vec<&PoolRow> = rows.iter().filter(|r| !r.held_out && usable(r)).collect();
    let hold: Vec<&PoolRow> = rows.iter().filter(|r| r.held_out && usable(r)).collect();
    eprintln!(
        "{} teacher builds in {:.0}s: {} train, {} held out, {} outside {MIN_SPELLS}..={MAX_SPELLS} spells",
        rows.len(),
        t0.elapsed().as_secs_f64(),
        train.len(),
        hold.len(),
        rows.iter().filter(|r| !usable(r)).count()
    );

    let mut rng = StdRng::seed_from_u64(args.seed ^ 0xDEC_B01);
    let model = Model {
        emb: Var::from_tensor(&Tensor::from_vec((0..c * EMB).map(|_| rng.random_range(-0.17f32..0.17)).collect::<Vec<_>>(), (c, EMB), &dev)?)?,
        feats: Tensor::from_vec(cards.feats.clone(), (c, FEATS), &dev)?,
        enc: Lin::new(EMB + FEATS, H, &mut rng, &dev)?,
        state: Lin::new(STATE_IN, H, &mut rng, &dev)?,
        cand1: Lin::new(2 * H + 2, H, &mut rng, &dev)?,
        cand2: Lin::new(H, 1, &mut rng, &dev)?,
        stop1: Lin::new(H, 32, &mut rng, &dev)?,
        stop2: Lin::new(32, 1, &mut rng, &dev)?,
    };
    let mut opt = AdamW::new(model.vars(), ParamsAdamW { lr: args.lr, weight_decay: 1e-4, ..Default::default() })?;
    let mut hrng = StdRng::seed_from_u64(0x401D);
    let hold_states: Vec<State> = (0..4000).map(|i| sample_state(hold[i % hold.len()], &mut hrng)).collect();
    let hold_batches: Vec<Batch> = hold_states.chunks(500).map(|ch| batch(ch, c, &cards.stats, &dev)).collect::<CResult<_>>()?;
    let eval = |m: &Model| -> CResult<(f64, f64)> {
        let (mut l, mut a) = (0.0, 0.0);
        for bt in &hold_batches {
            let (x, acc) = loss(m, bt)?;
            l += x.to_scalar::<f32>()? as f64;
            a += acc;
        }
        Ok((l / hold_batches.len() as f64, a / hold_batches.len() as f64))
    };
    let (mut best_loss, mut best_step) = (f64::INFINITY, 0);
    let mut best: Option<DeckPolicy> = None;
    // Step 2b's checkpoint metric and its policy states.
    let overlap_rows: Vec<&PoolRow> = [SealedSet::Fra, SealedSet::Sos]
        .into_iter()
        .flat_map(|set| hold.iter().copied().filter(move |r| r.set == set).take(500))
        .collect();
    let mut best_overlap = f64::NEG_INFINITY;
    let mut dagger_states: Vec<State> = Vec::new();
    let mut best_vars: Vec<Tensor> = Vec::new();
    let t1 = std::time::Instant::now();
    if let Some(path) = &args.init_vars {
        let saved = candle_core::safetensors::load(path, &dev)?;
        for (k, v) in model.vars().iter().enumerate() {
            v.set(saved.get(&format!("v{k}")).unwrap_or_else(|| panic!("{}: no v{k}", path.display())))?;
        }
        best = Some(export(&model, &cards)?);
        best_vars = model.vars().iter().map(|v| v.as_tensor().copy()).collect::<CResult<_>>()?;
        eprintln!("initialised from {}", path.display());
    }
    let steps = if args.load.is_some() || args.init_vars.is_some() { 0 } else { args.steps };
    for step in 0..=steps {
        if args.load.is_some() || args.init_vars.is_some() {
            break;
        }
        if step % 500 == 0 || step == steps {
            let (hl, ha) = eval(&model)?;
            eprintln!("step {step}: held-out loss {hl:.4} acc {ha:.4} ({:.0}s)", t1.elapsed().as_secs_f64());
            if args.dagger {
                let p = export(&model, &cards)?;
                let o = greedy_overlap(&p, &overlap_rows, &cards);
                eprintln!("step {step}: held-out greedy overlap {o:.4} ({} policy states)", dagger_states.len());
                if o > best_overlap {
                    best_overlap = o;
                    best_loss = hl;
                    best_step = step;
                    best = Some(p);
                    best_vars = model.vars().iter().map(|v| v.as_tensor().copy()).collect::<CResult<_>>()?;
                }
            } else if hl < best_loss {
                best_loss = hl;
                best_step = step;
                best = Some(export(&model, &cards)?);
            } else if !args.dagger && step >= best_step + 5000 {
                break;
            }
        }
        if step == steps {
            break;
        }
        if args.dagger && step >= args.warmup && (step - args.warmup).is_multiple_of(args.dagger_every) {
            let p = export(&model, &cards)?;
            let pick: Vec<&PoolRow> = (0..args.dagger_pools.min(train.len())).map(|_| train[rng.random_range(0..train.len())]).collect();
            let rolled = roll_states(&p, &pick, &cards, args.seed ^ step as u64, args.threads);
            eprintln!("step {step}: rolled {} policy states from {} pools", rolled.len(), pick.len());
            dagger_states.extend(rolled);
        }
        let states: Vec<State> = (0..args.batch)
            .map(|j| {
                if !dagger_states.is_empty() && j % 2 == 1 {
                    dagger_states[rng.random_range(0..dagger_states.len())].clone()
                } else {
                    sample_state(train[rng.random_range(0..train.len())], &mut rng)
                }
            })
            .collect();
        let bt = batch(&states, c, &cards.stats, &dev)?;
        let (l, _) = loss(&model, &bt)?;
        opt.step(&l.backward()?)?;
    }
    // Step 3: expert iteration from the step-2b checkpoint.
    if args.expert_iter > 0 {
        assert!(args.dagger && !best_vars.is_empty(), "--expert-iter runs after the --dagger recipe");
        for (v, t) in model.vars().iter().zip(&best_vars) {
            v.set(t)?;
        }
        opt.set_learning_rate(args.lr * 0.3);
        let play_rows: Vec<&PoolRow> = [SealedSet::Fra, SealedSet::Sos]
            .into_iter()
            .flat_map(|set| hold.iter().copied().filter(move |r| r.set == set).take(1000))
            .collect();
        let report = |g: usize, p: &DeckPolicy| {
            for (set, m, h) in play_read(p, &play_rows, args.threads) {
                println!("gen {g}: {set:?} greedy vs rated, held-out play read {m:.2} ± {h:.2}");
            }
        };
        let p0 = best.clone().expect("evaluated at step 0");
        report(0, &p0);
        let stem = args.export.with_extension("");
        let named: HashMap<String, Tensor> = best_vars.iter().enumerate().map(|(k, t)| (format!("v{k}"), t.clone())).collect();
        candle_core::safetensors::save(&named, format!("{}_gen0.safetensors", stem.display()))?;
        p0.save(&std::path::PathBuf::from(format!("{}_gen0.json", stem.display()))).unwrap_or_else(|e| panic!("{e}"));
        for g in 1..=args.expert_iter {
            let p = export(&model, &cards)?;
            let tg = std::time::Instant::now();
            let mut race: Vec<&PoolRow> = Vec::new();
            for (set, n) in [(SealedSet::Fra, args.race_fra), (SealedSet::Sos, args.race_sos)] {
                let mut mine: Vec<&PoolRow> = train.iter().copied().filter(|r| r.set == set).collect();
                mine.shuffle(&mut rng);
                race.extend(mine.into_iter().take(n));
            }
            for (set, raced, won, mean, fin, fin_mean, pair_won) in race_generation(
                &p,
                &race,
                &cards,
                args.seed ^ (g as u64) << 32,
                args.threads,
                55.0,
                (args.final_pairs, args.final_accept),
                args.pair_cands,
                args.confirm_top,
            ) {
                println!("gen {g}: {set:?} raced {raced} pools, replaced {won} ({:.1} %), replacements' confirmed share {mean:.2} ({:.0}s)", 100.0 * won as f64 / raced.max(1) as f64, tg.elapsed().as_secs_f64());
                if args.final_pairs > 0 {
                    println!("gen {g}: {set:?} {fin} leaders reached the {}-pair final; their mean final share {fin_mean:.2}", args.final_pairs);
                }
                if args.pair_cands {
                    println!("gen {g}: {set:?} {pair_won} of the {won} replacements were colour-pair builds");
                }
            }
            let improved: Vec<&PoolRow> = train.iter().copied().filter(|r| *r.best.read().unwrap() != r.teacher).collect();
            dagger_states.clear();
            for step in 0..args.gen_steps {
                if step.is_multiple_of(args.gen_steps / 2) {
                    let p = export(&model, &cards)?;
                    let pick: Vec<&PoolRow> = (0..args.dagger_pools.min(train.len())).map(|_| train[rng.random_range(0..train.len())]).collect();
                    let rolled = roll_states(&p, &pick, &cards, args.seed ^ (g * 1_000_000 + step) as u64, args.threads);
                    eprintln!("gen {g} step {step}: rolled {} policy states", rolled.len());
                    dagger_states.extend(rolled);
                }
                // Half the batch prefixes (half of those from relabelled
                // pools), half policy states labelled by the current expert.
                let states: Vec<State> = (0..args.batch)
                    .map(|j| match j % 4 {
                        0 if !improved.is_empty() => sample_state(improved[rng.random_range(0..improved.len())], &mut rng),
                        0 | 1 => sample_state(train[rng.random_range(0..train.len())], &mut rng),
                        _ => dagger_states[rng.random_range(0..dagger_states.len())].clone(),
                    })
                    .collect();
                let bt = batch(&states, c, &cards.stats, &dev)?;
                let (l, _) = loss(&model, &bt)?;
                opt.step(&l.backward()?)?;
            }
            let p = export(&model, &cards)?;
            println!("gen {g}: {} train pools carry a replaced deck; held-out greedy overlap with the teacher {:.4}", improved.len(), greedy_overlap(&p, &overlap_rows, &cards));
            report(g, &p);
            p.save(&std::path::PathBuf::from(format!("{}_gen{g}.json", stem.display()))).unwrap_or_else(|e| panic!("{e}"));
        }
    }

    let policy = match &args.load {
        Some(path) => DeckPolicy::load(path).unwrap_or_else(|e| panic!("{e}")),
        None => {
            best.expect("evaluated at step 0").save(&args.export).unwrap_or_else(|e| panic!("{e}"));
            DeckPolicy::load(&args.export).unwrap_or_else(|e| panic!("{e}"))
        }
    };

    // Parity: the engine's scores for held-out states against the export.
    let probe: Vec<State> = hold_states.iter().take(64).map(|s| State { row: s.row, deck: s.deck.clone(), remaining: s.remaining.clone() }).collect();
    parity(&policy, &probe, &cards, &dev)?;

    // The offline read on the held-out pools, per set.
    for set in [SealedSet::Fra, SealedSet::Sos] {
        let (mut overlap, mut lands_same, mut raw_same, mut n) = (0.0, 0usize, 0usize, 0usize);
        // Where the misses are: the overlap's spread, whether the build
        // plays the teacher's colours (each with at least three pips), and
        // its spell count against the teacher's.
        let (mut buckets, mut same_colors, mut count_diff) = ([0usize; 4], 0usize, 0i64);
        let colors = |spells: &[CardFactory]| -> u8 {
            let mut m = 0u8;
            for (k, c) in crabomination::mana::Color::ALL.into_iter().enumerate() {
                if spells.iter().map(|f| crabomination::cube::card_brief(*f).pips.get(c)).sum::<u32>() >= 3 {
                    m |= 1 << k;
                }
            }
            m
        };
        for r in hold.iter().filter(|r| r.set == set) {
            let spells = policy.spells(&r.pool, None);
            let want: Vec<&str> = r
                .teacher
                .iter()
                .map(|f| crabomination::cube::card_def(*f))
                .filter(|d| !d.is_land())
                .map(|d| d.name)
                .collect();
            let mut hit = 0usize;
            let mut left = want.clone();
            for f in &spells {
                let nm = crabomination::cube::card_def(*f).name;
                if let Some(p) = left.iter().position(|w| *w == nm) {
                    left.swap_remove(p);
                    hit += 1;
                }
            }
            let o = hit as f64 / want.len().max(1) as f64;
            overlap += o;
            buckets[if o < 0.5 { 0 } else if o < 0.8 { 1 } else if o < 0.95 { 2 } else { 3 }] += 1;
            let teacher_sp: Vec<CardFactory> = r.teacher.iter().copied().filter(|f| !crabomination::cube::card_def(*f).is_land()).collect();
            same_colors += usize::from(colors(&spells) == colors(&teacher_sp));
            count_diff += spells.len() as i64 - teacher_sp.len() as i64;
            let teacher_spells: Vec<CardFactory> =
                r.teacher.iter().copied().filter(|f| !crabomination::cube::card_def(*f).is_land()).collect();
            let mut ours = crabomination::recommend::lands_for_spells(&r.pool, &teacher_spells, (40 - teacher_spells.len()) as u32);
            let mut theirs: Vec<CardFactory> = r.teacher.iter().copied().filter(|f| crabomination::cube::card_def(*f).is_land()).collect();
            ours.sort_by_key(|f| *f as usize);
            theirs.sort_by_key(|f| *f as usize);
            lands_same += usize::from(ours.iter().map(|f| *f as usize).eq(theirs.iter().map(|f| *f as usize)));
            // The teacher's polish swaps spells after its lands are built, so
            // the same check on its unpolished build is the land step's own
            // agreement.
            let seed = 0x00DE_C4B0_0000_0000 ^ ((set == SealedSet::Fra) as u64) << 40 ^ r.index as u64;
            let raw = crabomination::selfplay::with_judge_polish(false, || crabomination::selfplay::rated_sealed_build(&r.pool, 16, seed ^ 1));
            let raw_spells: Vec<CardFactory> = raw.iter().copied().filter(|f| !crabomination::cube::card_def(*f).is_land()).collect();
            let mut ours = crabomination::recommend::lands_for_spells(&r.pool, &raw_spells, (40 - raw_spells.len()) as u32);
            let mut theirs: Vec<CardFactory> = raw.iter().copied().filter(|f| crabomination::cube::card_def(*f).is_land()).collect();
            ours.sort_by_key(|f| *f as usize);
            theirs.sort_by_key(|f| *f as usize);
            raw_same += usize::from(ours.iter().map(|f| *f as usize).eq(theirs.iter().map(|f| *f as usize)));
            n += 1;
        }
        println!(
            "{set:?}: greedy spell overlap with the teacher {:.3} over {n} held-out pools; land step reproduces the teacher's lands {:.3} (its unpolished build's {:.3})",
            overlap / n.max(1) as f64,
            lands_same as f64 / n.max(1) as f64,
            raw_same as f64 / n.max(1) as f64
        );
        println!(
            "{set:?}: overlap <0.5 / 0.5-0.8 / 0.8-0.95 / >=0.95: {:?}; same colours {:.3}; spells vs teacher {:+.2}",
            buckets,
            same_colors as f64 / n.max(1) as f64,
            count_diff as f64 / n.max(1) as f64
        );
    }
    println!(
        "best checkpoint at step {best_step} (held-out loss {best_loss:.4}{}); exported to {}",
        if args.dagger { format!(", greedy overlap {best_overlap:.4}") } else { String::new() },
        args.export.display()
    );
    Ok(())
}

/// The engine's scores against a model rebuilt from the export: the export
/// is what the gate plays, so the trainer's tensors and the engine's
/// arithmetic must agree on it.
fn parity(policy: &DeckPolicy, probe: &[State], cards: &Cards, dev: &Device) -> CResult<()> {
    let lin = |l: &Layer| -> CResult<Lin> {
        let w: Vec<f32> = l.w.iter().flatten().copied().collect();
        Ok(Lin {
            w: Var::from_tensor(&Tensor::from_vec(w, (l.w.len(), l.w[0].len()), dev)?)?,
            b: Var::from_tensor(&Tensor::new(l.b.clone(), dev)?)?,
        })
    };
    // The exported `x` is the encoder's output: an identity encoder over it.
    let c = policy.cards.len();
    let x: Vec<f32> = policy.x.iter().flatten().copied().collect();
    let ident: Vec<f32> = (0..H * H).map(|i| if i / H == i % H { 1.0 } else { 0.0 }).collect();
    let m = Model {
        emb: Var::from_tensor(&Tensor::from_vec(x, (c, H), dev)?)?,
        feats: Tensor::zeros((c, 0), DType::F32, dev)?,
        enc: Lin { w: Var::from_tensor(&Tensor::from_vec(ident, (H, H), dev)?)?, b: Var::zeros(H, DType::F32, dev)? },
        state: lin(&policy.state)?,
        cand1: lin(&policy.cand1)?,
        cand2: lin(&policy.cand2)?,
        stop1: lin(&policy.stop1)?,
        stop2: lin(&policy.stop2)?,
    };
    let bt = batch(probe, c, &cards.stats, dev)?;
    let l: Vec<Vec<f32>> = m.logits(&bt)?.to_vec2()?;
    let mut worst = 0f32;
    for (r, s) in probe.iter().enumerate() {
        let name_of = |f: &CardFactory| crabomination::cube::card_def(*f).name;
        let by_name: HashMap<&str, CardFactory> = s.row.pool.iter().map(|f| (name_of(f), *f)).collect();
        let deck: Vec<CardFactory> = s.deck.iter().map(|&i| by_name[cards.names[i].as_str()]).collect();
        let eng = policy.step(&s.row.pool, &deck);
        for (f, sc) in &eng.cards {
            let i = cards.index[name_of(f)];
            let j = bt.slots[r].iter().position(|&k| k == i).expect("the engine offers a pool card the batch has");
            worst = worst.max((sc - l[r][j]).abs());
        }
        if let Some(st) = eng.stop {
            worst = worst.max((st - l[r][bt.k]).abs());
        }
    }
    assert!(worst < 1e-3, "engine and trainer disagree on a policy score by {worst}");
    eprintln!("engine parity: {} held-out states, worst |Δ| {worst:.2e}", probe.len());
    Ok(())
}
