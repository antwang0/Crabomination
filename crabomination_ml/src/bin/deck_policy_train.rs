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
    /// The teacher's spells, as card indices, and as built.
    target: Vec<usize>,
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
    PoolRow { set, pool, avail, target, teacher, held_out: i.is_multiple_of(20), index: i }
}

/// One state: a prefix of a pool's target in some order.
struct State<'a> {
    row: &'a PoolRow,
    deck: Vec<usize>,
    remaining: Vec<usize>,
}

fn sample_state<'a>(row: &'a PoolRow, rng: &mut StdRng) -> State<'a> {
    let mut order = row.target.clone();
    order.shuffle(rng);
    let k = rng.random_range(0..=order.len());
    let remaining = order.split_off(k);
    State { row, deck: order, remaining }
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
    let usable = |r: &PoolRow| (MIN_SPELLS..=MAX_SPELLS).contains(&r.target.len());
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
    let t1 = std::time::Instant::now();
    let steps = if args.load.is_some() { 0 } else { args.steps };
    for step in 0..=steps {
        if args.load.is_some() {
            break;
        }
        if step % 500 == 0 || step == steps {
            let (hl, ha) = eval(&model)?;
            eprintln!("step {step}: held-out loss {hl:.4} acc {ha:.4} ({:.0}s)", t1.elapsed().as_secs_f64());
            if hl < best_loss {
                best_loss = hl;
                best_step = step;
                best = Some(export(&model, &cards)?);
            } else if step >= best_step + 5000 {
                break;
            }
        }
        if step == steps {
            break;
        }
        let states: Vec<State> = (0..args.batch).map(|_| sample_state(train[rng.random_range(0..train.len())], &mut rng)).collect();
        let bt = batch(&states, c, &cards.stats, &dev)?;
        let (l, _) = loss(&model, &bt)?;
        opt.step(&l.backward()?)?;
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
    println!("best held-out loss {best_loss:.4} at step {best_step}; exported to {}", args.export.display());
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
