//! Candle training for the table net — the mirror of
//! [`crabomination_nn::table`], tensor name for tensor name.
//!
//! One model for every seat count: a batch mixes duels and pods, padded to
//! its widest table with a seat mask, and the value head fits each row's
//! winner with a cross-entropy over the living slots. The parity tests below
//! hold this model and the engine's dependency-free forward to the same
//! numbers across mixed seat counts in one batch — padding is where two
//! implementations of the same net most easily disagree.

use std::collections::HashMap;

use candle_core::{D, DType, Device, Result as CResult, Tensor};
use candle_nn::{
    AdamW, Embedding, Linear, Module, Optimizer, ParamsAdamW, VarBuilder, VarMap, embedding, linear,
};
use crabomination_nn::table::{
    BELIEF_HIDDEN, DESC_BUCKETS, NO_DESC, SEAT_FEATS, SEAT_HIDDEN, SLOT_TAGS, TABLE_GLOBAL_FEATS,
    TABLE_OBJ_FEATS, TABLE_ZONES, TablePool, TableRow, TableState, VALUE_HIDDEN, slot_tag,
};
use crabomination_nn::{EMB_DIM, OBJ_HIDDEN, TRUNK_H1, TRUNK_H2};

use crate::{AUX_WEIGHT, TBlockLayer};

/// Table-net dimensions. The engine's loader reads sizes from the file, so
/// all of these are free between runs except `vocab`, which must match the
/// encoder's name vocabulary.
#[derive(Debug, Clone, Copy)]
pub struct TableConfig {
    pub vocab: usize,
    pub emb_dim: usize,
    pub obj_hidden: usize,
    pub seat_hidden: usize,
    pub h1: usize,
    pub h2: usize,
    pub value_hidden: usize,
    /// Pre-LN transformer blocks over every object at the table (with zone
    /// and slot tags). 0 is the pure pooled net.
    pub blocks: usize,
    /// Read the game's card pool: each pool card through the object layer
    /// (features zero), its mean and max beside the trunk's input. Off is
    /// step 1's net, unchanged.
    pub pool: bool,
    /// The belief head (`belief.*`, needs `pool`): per opponent slot, its
    /// hand's and library's distributions over the pool.
    pub belief: bool,
}

impl TableConfig {
    pub fn standard(vocab: usize) -> Self {
        Self {
            vocab,
            emb_dim: EMB_DIM,
            obj_hidden: OBJ_HIDDEN,
            seat_hidden: SEAT_HIDDEN,
            h1: TRUNK_H1,
            h2: TRUNK_H2,
            value_hidden: VALUE_HIDDEN,
            blocks: 0,
            pool: false,
            belief: false,
        }
    }
}

/// The candle mirror of `crabomination_nn::table::TableNet`, plus the
/// training-only heads (`head_life.*` per slot, `head_len.*`).
pub struct TableModel {
    name_emb: Embedding,
    tok_emb: Embedding,
    obj: Linear,
    /// `(tag_zone, tag_slot)`, present exactly when `blocks` is non-empty.
    tags: Option<(Embedding, Embedding)>,
    blocks: Vec<TBlockLayer>,
    seat: Linear,
    seat_slot: Embedding,
    trunk1: Linear,
    trunk2: Linear,
    value1: Linear,
    value2: Linear,
    head_life: Linear,
    head_len: Linear,
    pool: bool,
    belief: Option<BeliefModel>,
}

/// `belief.*`: `u = relu(b1 · [trunk ⊕ seat])` per slot; the hand's and the
/// library's logit for pool card `c` are `q_hand(u) · key(h_c)` and
/// `q_lib(u) · key(h_c)`, `h_c` the card through the object layer.
struct BeliefModel {
    b1: Linear,
    q_hand: Linear,
    q_lib: Linear,
    key: Linear,
}

pub fn build_table_model(cfg: &TableConfig, vb: VarBuilder) -> CResult<TableModel> {
    let (e, h, sh) = (cfg.emb_dim, cfg.obj_hidden, cfg.seat_hidden);
    let mut blocks = Vec::with_capacity(cfg.blocks);
    for i in 0..cfg.blocks {
        blocks.push(TBlockLayer::build(h, vb.pp("tblocks").pp(i.to_string()))?);
    }
    let tags = if cfg.blocks > 0 {
        Some((
            embedding(TABLE_ZONES, h, vb.pp("tag_zone"))?,
            embedding(SLOT_TAGS, h, vb.pp("tag_slot"))?,
        ))
    } else {
        None
    };
    Ok(TableModel {
        name_emb: embedding(cfg.vocab, e, vb.pp("name_emb"))?,
        tok_emb: embedding(DESC_BUCKETS, e, vb.pp("tok_emb"))?,
        obj: linear(2 * e + TABLE_OBJ_FEATS, h, vb.pp("obj"))?,
        tags,
        blocks,
        seat: linear(TABLE_ZONES * 2 * h + SEAT_FEATS, sh, vb.pp("seat"))?,
        seat_slot: embedding(SLOT_TAGS, sh, vb.pp("seat_slot"))?,
        trunk1: linear(3 * sh + TABLE_GLOBAL_FEATS + if cfg.pool { 2 * h } else { 0 }, cfg.h1, vb.pp("trunk1"))?,
        trunk2: linear(cfg.h1, cfg.h2, vb.pp("trunk2"))?,
        value1: linear(cfg.h2 + sh, cfg.value_hidden, vb.pp("value1"))?,
        value2: linear(cfg.value_hidden, 1, vb.pp("value2"))?,
        head_life: linear(cfg.value_hidden, 1, vb.pp("head_life"))?,
        head_len: linear(cfg.h2, 1, vb.pp("head_len"))?,
        pool: cfg.pool,
        belief: if cfg.belief {
            assert!(cfg.pool, "the belief head scores the pool");
            let v = vb.pp("belief");
            Some(BeliefModel {
                b1: linear(cfg.h2 + sh, BELIEF_HIDDEN, v.pp("b1"))?,
                q_hand: linear(BELIEF_HIDDEN, BELIEF_HIDDEN, v.pp("q_hand"))?,
                q_lib: linear(BELIEF_HIDDEN, BELIEF_HIDDEN, v.pp("q_lib"))?,
                key: linear(h, BELIEF_HIDDEN, v.pp("key"))?,
            })
        } else {
            None
        },
    })
}

/// `table[ids]` (`[N, E]`) through a gather whose backward parallelises.
///
/// candle's `index_select` backward is an `index_add`, and its CUDA kernel
/// runs one thread per embedding lane (E of them) looping serially over
/// every id — at a pod batch's ~100 k object lookups, plus the descriptors'
/// tokens, that serial loop was ~1.5 s of a step with the GPU "100 %" busy.
/// A gather along dim 1 of the table broadcast `K`-fold backpropagates
/// through a `scatter_add` over `K · E` threads (each looping `N / K` ids)
/// and a sum over the copies instead. Same values, same gradient.
pub fn fast_embed(table: &Tensor, ids: &[u32]) -> CResult<Tensor> {
    let (v, e) = table.dims2()?;
    let n = ids.len();
    if n == 0 {
        return Tensor::zeros((0, e), table.dtype(), table.device());
    }
    let k = 256.min(n);
    let m = n.div_ceil(k);
    let mut idx = vec![0u32; k * m * e];
    for (i, &id) in ids.iter().enumerate() {
        idx[i * e..(i + 1) * e].fill(id);
    }
    let idx = Tensor::from_vec(idx, (k, m, e), table.device())?;
    let t3 = table.unsqueeze(0)?.broadcast_as((k, v, e))?.contiguous()?;
    t3.gather(&idx, 1)?.reshape((k * m, e))?.narrow(0, 0, n)
}

/// A batch of tables padded to its widest one. Group `g = slot *
/// TABLE_ZONES + zone` for `slot < slots`; a row with fewer seats has empty
/// groups (mask 0) past its own, and seat mask 0 on those slots. Every
/// group's `[B, n_g]` block is laid out flat, group after group.
pub struct TableBatch {
    b: usize,
    slots: usize,
    /// Each group's padded width `n_g`.
    widths: Vec<usize>,
    /// Per object slot (flat, group-major): its name index, and its row in
    /// the batch's descriptor table (`ds` = none).
    obj_names: Vec<u32>,
    obj_descs: Vec<u32>,
    feats: Tensor, // [Ntot, TABLE_OBJ_FEATS]
    mask: Tensor,  // [Ntot, 1]
    /// The batch's distinct state descriptors (deduplicated by content):
    /// their tokens as indices into `utok` (the batch's distinct tokens),
    /// padded to `desc_t` with `utok.len()` (a zero row), and `1 / tokens`
    /// per descriptor plus the none row (0).
    utok: Tensor,
    n_utok: usize,
    desc_pad: Vec<u32>,
    desc_t: usize,
    ds: usize,
    desc_inv: Tensor, // [ds + 1, 1]
    /// The batch's distinct pools (by `Arc`) and each row's pool.
    pools: Vec<std::sync::Arc<TablePool>>,
    pub row_pool: Vec<Option<usize>>,
    seat_feats: Tensor, // [B, S, SEAT_FEATS]
    seat_mask: Tensor,  // [B, S]
    global: Tensor,     // [B, TABLE_GLOBAL_FEATS]
}

pub fn make_table_batch(states: &[&TableState], dev: &Device) -> CResult<TableBatch> {
    let b = states.len();
    let slots = states.iter().map(|s| s.seat_count()).max().unwrap_or(0).max(1);

    // Distinct descriptors and tokens across the batch.
    let mut desc_of: HashMap<&[u16], u32> = HashMap::new();
    let mut descs: Vec<&[u16]> = Vec::new();
    let mut state_desc: Vec<Vec<u32>> = Vec::with_capacity(b);
    for s in states {
        let mut m = Vec::with_capacity(s.desc_count());
        for d in 0..s.desc_count() {
            let toks = s.desc(d);
            let id = *desc_of.entry(toks).or_insert_with(|| {
                descs.push(toks);
                (descs.len() - 1) as u32
            });
            m.push(id);
        }
        state_desc.push(m);
    }
    let ds = descs.len();
    let mut tok_of: HashMap<u16, u32> = HashMap::new();
    let mut utok: Vec<u32> = Vec::new();
    let desc_t = descs.iter().map(|d| d.len()).max().unwrap_or(0).max(1);
    let mut desc_pad = Vec::with_capacity(ds * desc_t);
    let mut inv = vec![0f32; ds + 1];
    for (i, d) in descs.iter().enumerate() {
        for &t in d.iter() {
            let id = *tok_of.entry(t).or_insert_with(|| {
                utok.push(u32::from(t));
                (utok.len() - 1) as u32
            });
            desc_pad.push(id);
        }
        desc_pad.extend(std::iter::repeat_n(u32::MAX, desc_t - d.len()));
        if !d.is_empty() {
            inv[i] = 1.0 / d.len() as f32;
        }
    }
    let n_utok = utok.len();
    for x in desc_pad.iter_mut() {
        if *x == u32::MAX {
            *x = n_utok as u32;
        }
    }

    let mut pools: Vec<std::sync::Arc<TablePool>> = Vec::new();
    let row_pool: Vec<Option<usize>> = states
        .iter()
        .map(|s| {
            let p = s.pool.as_ref()?;
            Some(match pools.iter().position(|q| std::sync::Arc::ptr_eq(q, p)) {
                Some(i) => i,
                None => {
                    pools.push(p.clone());
                    pools.len() - 1
                }
            })
        })
        .collect();

    // Padding takes no gradient (every pooling is masked); its indices are
    // spread anyway so no table row collects every padded slot.
    let spread_ids = states.iter().flat_map(|s| s.objects().iter().map(|o| o.card as usize)).max().unwrap_or(0) + 1;
    let groups = slots * TABLE_ZONES;
    let mut widths = Vec::with_capacity(groups);
    let (mut obj_names, mut obj_descs, mut ft, mut mk) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for g in 0..groups {
        let (slot, zone) = (g / TABLE_ZONES, g % TABLE_ZONES);
        let n = states
            .iter()
            .filter(|s| slot < s.seat_count())
            .map(|s| s.group_len(slot, zone))
            .max()
            .unwrap_or(0)
            .max(1);
        widths.push(n);
        let base = obj_names.len();
        obj_names.extend((0..b * n).map(|i| ((base + i) % spread_ids) as u32));
        obj_descs.extend((0..b * n).map(|i| ((base + i) % (ds + 1)) as u32));
        ft.extend(std::iter::repeat_n(0f32, b * n * TABLE_OBJ_FEATS));
        mk.extend(std::iter::repeat_n(0f32, b * n));
        for (ri, s) in states.iter().enumerate() {
            if slot >= s.seat_count() {
                continue;
            }
            for (oi, o) in s.group(slot, zone).iter().enumerate() {
                let at = base + ri * n + oi;
                obj_names[at] = u32::from(o.card);
                obj_descs[at] = if o.desc != NO_DESC { state_desc[ri][o.desc as usize] } else { ds as u32 };
                ft[at * TABLE_OBJ_FEATS..][..TABLE_OBJ_FEATS].copy_from_slice(&o.feats);
                mk[at] = 1.0;
            }
        }
    }
    let ntot = obj_names.len();

    let mut sf = vec![0f32; b * slots * SEAT_FEATS];
    let mut sm = vec![0f32; b * slots];
    let mut gl = vec![0f32; b * TABLE_GLOBAL_FEATS];
    for (ri, s) in states.iter().enumerate() {
        for slot in 0..s.seat_count() {
            sf[(ri * slots + slot) * SEAT_FEATS..][..SEAT_FEATS].copy_from_slice(s.seat(slot));
            sm[ri * slots + slot] = 1.0;
        }
        gl[ri * TABLE_GLOBAL_FEATS..][..TABLE_GLOBAL_FEATS].copy_from_slice(&s.global);
    }
    Ok(TableBatch {
        b,
        slots,
        widths,
        obj_names,
        obj_descs,
        feats: Tensor::from_vec(ft, (ntot, TABLE_OBJ_FEATS), dev)?,
        mask: Tensor::from_vec(mk, (ntot, 1), dev)?,
        utok: Tensor::from_vec(utok, n_utok, dev)?,
        n_utok,
        desc_pad,
        desc_t,
        ds,
        desc_inv: Tensor::from_vec(inv, (ds + 1, 1), dev)?,
        pools,
        row_pool,
        seat_feats: Tensor::from_vec(sf, (b, slots, SEAT_FEATS), dev)?,
        seat_mask: Tensor::from_vec(sm, (b, slots), dev)?,
        global: Tensor::from_vec(gl, (b, TABLE_GLOBAL_FEATS), dev)?,
    })
}

/// Masked mean and max over axis 1 of `x` (`[B, N, H]`) with `m`
/// (`[B, N, 1]`, 1 real / 0 padding): both zero where nothing is real,
/// exactly the engine's "an empty group stays zero".
fn masked_mean_max(x: &Tensor, m: &Tensor) -> CResult<(Tensor, Tensor)> {
    let xm = x.broadcast_mul(m)?;
    let count = m.sum(1)?; // [B, 1]
    let mean = xm.sum(1)?.broadcast_div(&count.maximum(1f64)?)?;
    // Padding is pushed to -1e9 before the max, then empty sets are zeroed
    // through the 0/1 presence — the same shape as the play net's pooling.
    let sink = m.affine(1e9, -1e9)?;
    let mx = xm.broadcast_add(&sink)?.max(1)?.broadcast_mul(&count.minimum(1f64)?)?;
    Ok((mean, mx))
}

/// What one forward pass produces, every tensor `[B, S]` except `len`.
pub struct TableOut {
    /// Raw per-slot logits, padding slots pushed to -1e9.
    pub logits: Tensor,
    /// Softmax over the living slots.
    pub probs: Tensor,
    pub life: Tensor,
    /// `[B, 1]`.
    pub len: Tensor,
    /// The belief head's logits, one entry per pool in the batch: the rows
    /// on that pool, and their hand / library logits `[rows, S, P]`.
    pub belief: Vec<BeliefOut>,
}

pub struct BeliefOut {
    pub pool: usize,
    pub rows: Vec<usize>,
    pub hand: Tensor,
    pub lib: Tensor,
}

impl TableModel {
    pub fn forward(&self, batch: &TableBatch) -> CResult<TableOut> {
        let (b, slots) = (batch.b, batch.slots);

        // Descriptor embeddings: the masked mean of each descriptor's token
        // rows. The last row is all padding and embeds as zeros.
        let dev0 = batch.global.device();
        let e_cols = self.name_emb.embeddings().dim(1)?;
        // The batch's distinct descriptors: each its tokens' mean, through
        // the batch's distinct tokens (see `fast_embed`), plus a zero none row.
        let tok_sub = if batch.n_utok > 0 {
            self.tok_emb.embeddings().index_select(&batch.utok, 0)?
        } else {
            Tensor::zeros((0, e_cols), DType::F32, dev0)?
        };
        let tok_sub = Tensor::cat(&[&tok_sub, &Tensor::zeros((1, e_cols), DType::F32, dev0)?], 0)?;
        let demb = if batch.ds > 0 {
            fast_embed(&tok_sub, &batch.desc_pad)?.reshape((batch.ds, batch.desc_t, e_cols))?.sum(1)?
        } else {
            Tensor::zeros((0, e_cols), DType::F32, dev0)?
        };
        let demb = Tensor::cat(&[&demb, &Tensor::zeros((1, e_cols), DType::F32, dev0)?], 0)?.broadcast_mul(&batch.desc_inv)?;

        // Each distinct pool's cards through the object layer, features
        // zero. Their name rows and descriptor means are read off the
        // current weights on the host, without gradient: a Commander pool is
        // 22,907 cards and ~700 k tokens a step. The tables still train
        // through the board's objects; the object layer and the belief head
        // through the pool.
        let mut pool_h: Vec<Tensor> = Vec::with_capacity(batch.pools.len());
        if self.pool && !batch.pools.is_empty() {
            let names: Vec<Vec<f32>> = self.name_emb.embeddings().to_vec2()?;
            let toks: Vec<Vec<f32>> = self.tok_emb.embeddings().to_vec2()?;
            for p in &batch.pools {
                let n = p.len();
                let k = 2 * e_cols + TABLE_OBJ_FEATS;
                let mut x = vec![0f32; n * k];
                for i in 0..n {
                    let row = &mut x[i * k..(i + 1) * k];
                    let c = if (p.card(i) as usize) < names.len() { p.card(i) as usize } else { 0 };
                    row[..e_cols].copy_from_slice(&names[c]);
                    let d = p.desc(i);
                    if !d.is_empty() {
                        for &t in d {
                            let t = if (t as usize) < toks.len() { t as usize } else { 0 };
                            for (acc, v) in row[e_cols..2 * e_cols].iter_mut().zip(&toks[t]) {
                                *acc += v;
                            }
                        }
                        let inv = 1.0 / d.len() as f32;
                        for v in row[e_cols..2 * e_cols].iter_mut() {
                            *v *= inv;
                        }
                    }
                }
                let x = Tensor::from_vec(x, (n, k), dev0)?;
                pool_h.push(self.obj.forward(&x)?.relu()?);
            }
        }

        // Every object slot in one pass: names and descriptors gathered,
        // the object layer once, then split back into the groups.
        let e_all = fast_embed(self.name_emb.embeddings(), &batch.obj_names)?;
        let d_all = fast_embed(&demb, &batch.obj_descs)?;
        let h_all = self.obj.forward(&Tensor::cat(&[&e_all, &d_all, &batch.feats], 1)?)?.relu()?;
        let mut hs = Vec::with_capacity(batch.widths.len());
        let mut masks = Vec::with_capacity(batch.widths.len());
        let mut off = 0;
        for &n in &batch.widths {
            hs.push(h_all.narrow(0, off, b * n)?.reshape((b, n, ()))?);
            masks.push(batch.mask.narrow(0, off, b * n)?.reshape((b, n, 1))?);
            off += b * n;
        }

        if let Some((tag_zone, tag_slot)) = &self.tags {
            let dev = hs[0].device();
            let sizes: Vec<usize> = hs.iter().map(|h| h.dim(1)).collect::<CResult<_>>()?;
            let mut tagged = Vec::with_capacity(hs.len());
            for (g, h) in hs.iter().enumerate() {
                let (slot, zone) = (g / TABLE_ZONES, g % TABLE_ZONES);
                let z = tag_zone.forward(&Tensor::new(&[zone as u32], dev)?)?; // [1, H]
                let s = tag_slot.forward(&Tensor::new(&[slot_tag(slot) as u32], dev)?)?;
                tagged.push(h.broadcast_add(&z.add(&s)?.unsqueeze(0)?)?);
            }
            let mut x = Tensor::cat(&tagged.iter().collect::<Vec<_>>(), 1)?;
            let m = Tensor::cat(&masks.iter().collect::<Vec<_>>(), 1)?;
            let n: usize = sizes.iter().sum();
            let keybias = ((m.reshape((b, 1, 1, n))? - 1.0)? * 1e9)?;
            for blk in &self.blocks {
                x = blk.forward(&x, &keybias)?;
            }
            let mut off = 0;
            for (g, sz) in sizes.into_iter().enumerate() {
                hs[g] = x.narrow(1, off, sz)?;
                off += sz;
            }
        }

        // Seat vectors: each slot's zone pools and seat features through the
        // shared seat layer, plus the slot's own row.
        let dev = batch.global.device().clone();
        let mut zs = Vec::with_capacity(slots);
        for slot in 0..slots {
            let mut parts = Vec::with_capacity(2 * TABLE_ZONES + 1);
            for zone in 0..TABLE_ZONES {
                let g = slot * TABLE_ZONES + zone;
                let (mean, mx) = masked_mean_max(&hs[g], &masks[g])?;
                parts.push(mean);
                parts.push(mx);
            }
            parts.push(batch.seat_feats.narrow(1, slot, 1)?.squeeze(1)?);
            let seat_in = Tensor::cat(&parts.iter().collect::<Vec<_>>(), 1)?;
            // [1, SH]
            let tag = self.seat_slot.forward(&Tensor::new(&[slot_tag(slot) as u32], &dev)?)?;
            zs.push(self.seat.forward(&seat_in)?.broadcast_add(&tag)?.relu()?);
        }
        let z = Tensor::stack(&zs.iter().collect::<Vec<_>>(), 1)?; // [B, S, SH]
        let sh = z.dim(2)?;

        let me = z.narrow(1, 0, 1)?.squeeze(1)?;
        let (opp_mean, opp_max) = if slots > 1 {
            let o = z.narrow(1, 1, slots - 1)?;
            let om = batch.seat_mask.narrow(1, 1, slots - 1)?.unsqueeze(2)?;
            masked_mean_max(&o, &om)?
        } else {
            let zero = Tensor::zeros((b, sh), DType::F32, &dev)?;
            (zero.clone(), zero)
        };
        let mut parts = vec![me, opp_mean, opp_max, batch.global.clone()];
        if self.pool {
            // Each row's pool summary (mean ⊕ max over its cards), zeros
            // for a row without one.
            let h = self.obj.weight().dim(0)?;
            let summ: Vec<Tensor> = pool_h
                .iter()
                .map(|ph| Tensor::cat(&[&ph.mean(0)?, &ph.max(0)?], 0))
                .collect::<CResult<_>>()?;
            let zero = Tensor::zeros(2 * h, DType::F32, &dev)?;
            let rows: Vec<&Tensor> = batch.row_pool.iter().map(|p| p.map_or(&zero, |i| &summ[i])).collect();
            parts.push(Tensor::stack(&rows, 0)?);
        }
        let t = Tensor::cat(&parts.iter().collect::<Vec<_>>(), 1)?;
        let t1 = self.trunk1.forward(&t)?.relu()?;
        let t2 = self.trunk2.forward(&t1)?.relu()?; // [B, h2]

        let h2 = t2.dim(1)?;
        let t2s = t2.unsqueeze(1)?.broadcast_as((b, slots, h2))?.contiguous()?;
        let u = self.value1.forward(&Tensor::cat(&[&t2s, &z], 2)?)?.relu()?; // [B, S, VH]
        let raw = self.value2.forward(&u)?.squeeze(2)?; // [B, S]
        let logits = raw.add(&((&batch.seat_mask - 1.0)? * 1e9)?)?;
        let probs = candle_nn::ops::softmax(&logits, D::Minus1)?;
        let life = self.head_life.forward(&u)?.squeeze(2)?;
        let len = self.head_len.forward(&t2)?;
        let mut belief = Vec::new();
        if let Some(bm) = &self.belief {
            let ub = bm.b1.forward(&Tensor::cat(&[&t2s, &z], 2)?)?.relu()?; // [B, S, BH]
            let (qh, ql) = (bm.q_hand.forward(&ub)?, bm.q_lib.forward(&ub)?);
            for (pi, ph) in pool_h.iter().enumerate() {
                let rows: Vec<usize> = (0..b).filter(|&r| batch.row_pool[r] == Some(pi)).collect();
                if rows.is_empty() {
                    continue;
                }
                let idx = Tensor::from_vec(rows.iter().map(|&r| r as u32).collect::<Vec<_>>(), rows.len(), &dev)?;
                let keys = bm.key.forward(ph)?.t()?.contiguous()?; // [BH, P]
                let hand = qh.index_select(&idx, 0)?.broadcast_matmul(&keys)?; // [r, S, P]
                let lib = ql.index_select(&idx, 0)?.broadcast_matmul(&keys)?;
                belief.push(BeliefOut { pool: pi, rows, hand, lib });
            }
        }
        Ok(TableOut { logits, probs, life, len, belief })
    }
}

/// One step's loss, decomposed (the value term alone is what a gate
/// compares).
#[derive(Debug, Clone, Copy)]
pub struct TableLoss {
    pub total: f32,
    /// Cross-entropy of the per-slot softmax against the result share.
    pub win: f32,
    pub life: f32,
    pub len: f32,
    /// The belief head's cross-entropies (hand, library) over the pool;
    /// 0 without the head or a row carrying the truth.
    pub hand: f32,
    pub lib: f32,
}

/// One belief group's cross-entropy (hand, or `lib` the library) against
/// the recorder's truth, read sparsely: per (row, slot ≥ 1) with a non-empty
/// truth, `−Σ_c t_c · log softmax(logits)_c` over just the true cards (`t`
/// the truth's normalised counts), averaged over those slots. The truth is
/// a handful of cards out of a pool of up to 22,907, so the dense target
/// tensors this replaced were most of a pod step. `None` when no slot
/// carries a truth.
fn belief_ce(rows: &[&TableRow], g: &BeliefOut, logits: &Tensor, slots: usize, lib: bool, dev: &Device) -> CResult<Option<Tensor>> {
    let (n, p) = (g.rows.len(), logits.dim(2)?);
    let mut lists: Vec<Vec<(u32, f32)>> = vec![Vec::new(); n * slots];
    let mut weighted = 0usize;
    for (k, &ri) in g.rows.iter().enumerate() {
        let r = rows[ri];
        for slot in 1..r.state.seat_count().min(slots) {
            let mut l: Vec<(u32, f32)> = Vec::new();
            if lib {
                for &(c, m) in r.library.get(slot).map_or(&[][..], |v| &v[..]) {
                    if (c as usize) < p {
                        l.push((c as u32, m as f32));
                    }
                }
            } else {
                for &c in r.hand.get(slot).map_or(&[][..], |v| &v[..]) {
                    if (c as usize) < p {
                        match l.iter_mut().find(|x| x.0 == c as u32) {
                            Some(x) => x.1 += 1.0,
                            None => l.push((c as u32, 1.0)),
                        }
                    }
                }
            }
            let total: f32 = l.iter().map(|x| x.1).sum();
            if total > 0.0 {
                for x in l.iter_mut() {
                    x.1 /= total;
                }
                weighted += 1;
                lists[k * slots + slot] = l;
            }
        }
    }
    if weighted == 0 {
        return Ok(None);
    }
    let width = lists.iter().map(|l| l.len()).max().unwrap_or(1).max(1);
    let t = n * slots;
    let (mut ids, mut ws) = (vec![0u32; t * width], vec![0f32; t * width]);
    for (i, l) in lists.iter().enumerate() {
        for (j, &(c, w)) in l.iter().enumerate() {
            ids[i * width + j] = c;
            ws[i * width + j] = w;
        }
    }
    let ids = Tensor::from_vec(ids, (t, width), dev)?;
    let ws = Tensor::from_vec(ws, (t, width), dev)?;
    let logp = candle_nn::ops::log_softmax(logits, D::Minus1)?.reshape((t, p))?;
    let picked = logp.gather(&ids, 1)?; // [t, width]
    Ok(Some(picked.mul(&ws)?.sum_all()?.neg()?.affine(1.0 / weighted as f64, 0.0)?))
}

pub struct TableTrainer {
    varmap: VarMap,
    model: TableModel,
    opt: AdamW,
    dev: Device,
}

impl TableTrainer {
    pub fn new(cfg: &TableConfig, lr: f64) -> CResult<TableTrainer> {
        let dev = Device::cuda_if_available(0)?;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &dev);
        let model = build_table_model(cfg, vb)?;
        let opt = AdamW::new(varmap.all_vars(), ParamsAdamW { lr, ..Default::default() })?;
        Ok(TableTrainer { varmap, model, opt, dev })
    }

    pub fn device(&self) -> &Device {
        &self.dev
    }

    pub fn varmap(&self) -> &VarMap {
        &self.varmap
    }

    /// One AdamW step over `rows`; returns the pre-step loss.
    pub fn train_step(&mut self, rows: &[&TableRow]) -> CResult<TableLoss> {
        let timing = std::env::var_os("CRAB_TABLE_TIMING").is_some();
        let t0 = std::time::Instant::now();
        let states: Vec<&TableState> = rows.iter().map(|r| &r.state).collect();
        let batch = make_table_batch(&states, &self.dev)?;
        let t_batch = t0.elapsed();
        let (b, slots) = (batch.b, batch.slots);
        let mut win = vec![0f32; b * slots];
        let mut life = vec![0f32; b * slots];
        let mut len = vec![0f32; b];
        for (ri, r) in rows.iter().enumerate() {
            debug_assert_eq!(r.win.len(), r.state.seat_count(), "one result share per slot");
            win[ri * slots..][..r.win.len()].copy_from_slice(&r.win);
            life[ri * slots..][..r.life.len()].copy_from_slice(&r.life);
            len[ri] = r.game_len;
        }
        let win = Tensor::from_vec(win, (b, slots), &self.dev)?;
        let life_t = Tensor::from_vec(life, (b, slots), &self.dev)?;
        let len_t = Tensor::from_vec(len, (b, 1), &self.dev)?;

        let out = self.model.forward(&batch)?;
        let t_fwd = t0.elapsed();
        let logp = candle_nn::ops::log_softmax(&out.logits, D::Minus1)?;
        let loss_win = win.mul(&logp)?.sum(1)?.neg()?.mean_all()?;
        let diff = out.life.sub(&life_t)?.mul(&batch.seat_mask)?;
        let loss_life =
            diff.sqr()?.sum_all()?.broadcast_div(&batch.seat_mask.sum_all()?.maximum(1f64)?)?;
        let loss_len = candle_nn::loss::mse(&out.len, &len_t)?;
        let mut loss = loss_win
            .add(&loss_life.affine(AUX_WEIGHT, 0.0)?)?
            .add(&loss_len.affine(AUX_WEIGHT, 0.0)?)?;
        // The belief head's two cross-entropies, at the auxiliary weight,
        // averaged over the batch's pools by the slots each carries.
        let (mut hand_v, mut lib_v, mut groups) = (0f32, 0f32, 0f32);
        for g in &out.belief {
            if let Some(h) = belief_ce(rows, g, &g.hand, slots, false, &self.dev)? {
                hand_v += h.to_scalar::<f32>()?;
                loss = loss.add(&h.affine(AUX_WEIGHT, 0.0)?)?;
            }
            if let Some(l) = belief_ce(rows, g, &g.lib, slots, true, &self.dev)? {
                lib_v += l.to_scalar::<f32>()?;
                loss = loss.add(&l.affine(AUX_WEIGHT, 0.0)?)?;
            }
            groups += 1.0;
        }
        let t_loss = t0.elapsed();
        self.opt.backward_step(&loss)?;
        if timing {
            let _ = loss.to_scalar::<f32>();
            let padded: usize = batch.obj_names.len();
            let real: usize = states.iter().map(|s| s.len()).sum();
            eprintln!(
                "table step: {} slots, {} object slots for {} objects; batch {:?} forward {:?} losses {:?} backward {:?}",
                batch.slots,
                padded,
                real,
                t_batch,
                t_fwd - t_batch,
                t_loss - t_fwd,
                t0.elapsed() - t_loss
            );
        }
        Ok(TableLoss {
            total: loss.to_scalar::<f32>()?,
            win: loss_win.to_scalar::<f32>()?,
            life: loss_life.to_scalar::<f32>()?,
            len: loss_len.to_scalar::<f32>()?,
            hand: if groups > 0.0 { hand_v / groups } else { 0.0 },
            lib: if groups > 0.0 { lib_v / groups } else { 0.0 },
        })
    }

    /// The belief read **B1** on held-out rows: per (row, opponent slot)
    /// with a known hand, the share of the hand's cards among the head's top
    /// `k` pool cards (`min(k, hand)` the denominator), against the uniform
    /// pool's expectation (`k / P`, the pre-registered baseline) and a
    /// base-rate baseline (`base`: pool cards ranked by how often they sit
    /// in a training hand, per pool). `(head, uniform, base_rate, n)`.
    pub fn belief_recall(&self, rows: &[&TableRow], k: usize, base: &std::collections::HashMap<u64, Vec<u16>>) -> CResult<(f32, f32, f32, usize)> {
        let (mut head, mut uni, mut br, mut n) = (0f64, 0f64, 0f64, 0usize);
        for part in rows.chunks(64) {
            let states: Vec<&TableState> = part.iter().map(|r| &r.state).collect();
            let batch = make_table_batch(&states, &self.dev)?;
            let out = self.model.forward(&batch)?;
            for g in &out.belief {
                let logits: Vec<Vec<Vec<f32>>> = g.hand.to_vec3()?;
                for (j, &ri) in g.rows.iter().enumerate() {
                    let r = part[ri];
                    let p = logits[j].first().map_or(0, |x| x.len());
                    let ranked_base = r.state.pool.as_ref().and_then(|pl| base.get(&pl.id()));
                    for (slot, l) in logits[j].iter().enumerate().take(r.state.seat_count()).skip(1) {
                        let hand: std::collections::HashSet<u16> = r.hand[slot].iter().copied().collect();
                        if hand.is_empty() || p == 0 {
                            continue;
                        }
                        let denom = k.min(hand.len()) as f64;
                        let mut order: Vec<usize> = (0..p).collect();
                        order.sort_by(|&a, &b| l[b].partial_cmp(&l[a]).unwrap_or(std::cmp::Ordering::Equal));
                        head += order.iter().take(k).filter(|&&c| hand.contains(&(c as u16))).count() as f64 / denom;
                        uni += (k as f64 / p as f64) * hand.len() as f64 / denom;
                        if let Some(rb) = ranked_base {
                            br += rb.iter().take(k).filter(|c| hand.contains(c)).count() as f64 / denom;
                        }
                        n += 1;
                    }
                }
            }
        }
        let d = n.max(1) as f64;
        Ok(((head / d) as f32, (uni / d) as f32, (br / d) as f32, n))
    }

    /// Each state's per-slot win probabilities, in its own slot order.
    pub fn predict(&self, states: &[&TableState]) -> CResult<Vec<Vec<f32>>> {
        if states.is_empty() {
            return Ok(Vec::new());
        }
        let batch = make_table_batch(states, &self.dev)?;
        let probs: Vec<Vec<f32>> = self.model.forward(&batch)?.probs.to_vec2()?;
        Ok(probs.into_iter().zip(states).map(|(p, s)| p[..s.seat_count()].to_vec()).collect())
    }

    pub fn save(&self, path: &std::path::Path) -> CResult<()> {
        self.varmap.save(path)
    }

    /// Resume from an export of the same shape.
    pub fn load(&mut self, path: &std::path::Path) -> CResult<()> {
        self.varmap.load(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination_nn::table::{TableNet, TableObject, Z_HAND, Z_LIB};
    use rand::rngs::StdRng;
    use rand::{Rng, RngExt, SeedableRng};

    fn small(blocks: usize) -> TableConfig {
        TableConfig {
            vocab: 12,
            emb_dim: 4,
            obj_hidden: 8,
            seat_hidden: 8,
            h1: 16,
            h2: 8,
            value_hidden: 8,
            blocks,
            pool: false,
            belief: false,
        }
    }

    /// A random table: 2..=6 seats, a few objects per (slot, zone) — hand
    /// and library on slot 0 only, as the encoder emits — and a handful of
    /// descriptors, some objects without one.
    fn random_table<R: Rng>(rng: &mut R, vocab: usize) -> TableState {
        let seats = rng.random_range(2..=6);
        let mut s = TableState::with_seats(seats);
        for g in s.global.iter_mut() {
            *g = rng.random_range(-1.0..1.0);
        }
        for slot in 0..seats {
            for f in s.seat_mut(slot).iter_mut() {
                *f = rng.random_range(0.0..1.0);
            }
        }
        let descs = rng.random_range(0..4usize);
        for _ in 0..descs {
            let len = rng.random_range(0..6);
            let toks: Vec<u16> =
                (0..len).map(|_| rng.random_range(1..DESC_BUCKETS as u16)).collect();
            s.add_desc(&toks);
        }
        for slot in 0..seats {
            for zone in 0..TABLE_ZONES {
                if slot > 0 && (zone == Z_HAND || zone == Z_LIB) {
                    continue;
                }
                for _ in 0..rng.random_range(0..4) {
                    let mut feats = [0.0f32; TABLE_OBJ_FEATS];
                    for f in feats.iter_mut() {
                        *f = rng.random_range(0.0..1.0);
                    }
                    let desc = if descs > 0 && rng.random_range(0..3) > 0 {
                        rng.random_range(0..descs) as u16
                    } else {
                        NO_DESC
                    };
                    s.push(
                        slot,
                        zone,
                        TableObject { card: rng.random_range(0..vocab as u16), desc, feats },
                    );
                }
            }
        }
        s
    }

    fn export(t: &TableTrainer, tag: &str) -> TableNet {
        let path = std::env::temp_dir()
            .join(format!("crab_table_{tag}_{}.safetensors", std::process::id()));
        t.save(&path).expect("save");
        let bytes = std::fs::read(&path).expect("read");
        let _ = std::fs::remove_file(&path);
        TableNet::load(&bytes).expect("engine loads the trainer's export")
    }

    /// The trainer/inference contract, over one batch that mixes seat
    /// counts: every row's padding (slots, groups, objects, descriptor
    /// tokens) has to vanish exactly for the two to agree.
    fn parity(blocks: usize, seed: u64) {
        let cfg = small(blocks);
        let trainer = TableTrainer::new(&cfg, 1e-3).expect("trainer");
        crate::tests::reseed_params(&trainer.varmap, &trainer.dev, seed);
        let net = export(&trainer, &format!("parity{blocks}"));
        assert_eq!(net.arch().6, blocks);
        let mut rng = StdRng::seed_from_u64(seed);
        let states: Vec<TableState> = (0..10).map(|_| random_table(&mut rng, cfg.vocab)).collect();
        let refs: Vec<&TableState> = states.iter().collect();
        let want = trainer.predict(&refs).expect("candle forward");
        for (i, (s, w)) in states.iter().zip(&want).enumerate() {
            let got = net.forward(s);
            assert_eq!(got.len(), s.seat_count());
            for (g, w) in got.iter().zip(w) {
                assert!(
                    (g - w).abs() < 1e-4,
                    "state {i} ({} seats): engine {got:?} vs candle {w:?}",
                    s.seat_count()
                );
            }
        }
    }

    #[test]
    fn exported_table_net_matches_engine_inference() {
        parity(0, 31);
    }

    /// The pool input and the belief head: rows on two different pools and
    /// rows on none, in one batch — the engine's per-game pool encoding and
    /// its belief logits reproduce the trainer's.
    #[test]
    fn the_pool_and_belief_head_match_engine_inference() {
        let cfg = TableConfig { pool: true, belief: true, ..small(1) };
        let trainer = TableTrainer::new(&cfg, 1e-3).expect("trainer");
        crate::tests::reseed_params(&trainer.varmap, &trainer.dev, 8);
        let net = export(&trainer, "pool");
        assert!(net.has_pool() && net.has_belief());
        let mut rng = StdRng::seed_from_u64(8);
        let mk_pool = |rng: &mut StdRng, n: usize| {
            let mut p = TablePool::default();
            for _ in 0..n {
                let toks: Vec<u16> = (0..rng.random_range(0..4)).map(|_| rng.random_range(0..200u16)).collect();
                p.push(rng.random_range(0..cfg.vocab as u16), &toks);
            }
            std::sync::Arc::new(p)
        };
        let pools = [mk_pool(&mut rng, 9), mk_pool(&mut rng, 23)];
        let states: Vec<TableState> = (0..9)
            .map(|i| {
                let mut s = random_table(&mut rng, cfg.vocab);
                s.pool = (i % 3 != 2).then(|| pools[i % 2].clone());
                s
            })
            .collect();
        let refs: Vec<&TableState> = states.iter().collect();
        let want = trainer.predict(&refs).expect("candle");
        let batch = make_table_batch(&refs, &trainer.dev).expect("batch");
        let out = trainer.model.forward(&batch).expect("forward");
        for (i, (s, w)) in states.iter().zip(&want).enumerate() {
            for (g, w) in net.forward(s).iter().zip(w) {
                assert!((g - w).abs() < 1e-4, "state {i} value: engine {g} vs candle {w}");
            }
            let got = net.belief(s);
            assert_eq!(got.is_some(), s.pool.is_some(), "state {i}");
            let Some(got) = got else { continue };
            let g = out.belief.iter().find(|g| g.rows.contains(&i)).expect("a belief group");
            let j = g.rows.iter().position(|&r| r == i).unwrap();
            let hand: Vec<Vec<Vec<f32>>> = g.hand.to_vec3().unwrap();
            let lib: Vec<Vec<Vec<f32>>> = g.lib.to_vec3().unwrap();
            for (slot, (gh, gl)) in got.iter().enumerate().map(|(k, x)| (k + 1, x)) {
                for c in 0..gh.len() {
                    assert!((gh[c] - hand[j][slot][c]).abs() < 1e-3, "state {i} slot {slot} hand {c}");
                    assert!((gl[c] - lib[j][slot][c]).abs() < 1e-3, "state {i} slot {slot} lib {c}");
                }
            }
        }
    }

    #[test]
    fn exported_table_net_with_blocks_matches_engine_inference() {
        parity(2, 37);
    }

    /// Parity on what the engine actually emits: every seat's view of a
    /// four-seat Commander pod a few turns in — real descriptors (dozens of
    /// tokens a card), real board sizes, a command zone per slot — through
    /// both forwards with an attention block, in one batch.
    #[test]
    fn engine_encoded_pods_match_across_the_two_forwards() {
        use crabomination::server::bot::{Bot, HeuristicBot};
        use crabomination::server::encode::Vocab;
        use crabomination::server::encode_table::encode_table;
        let vocab = Vocab::sos_sealed();
        let mut g = crabomination::pod::build_pod_template(&crabomination::pod::pod_field(4));
        g.rng.reseed(11);
        g.start_mulligan_phase();
        let mut bots: Vec<HeuristicBot> = (0..4).map(|_| HeuristicBot::new()).collect();
        let mut actions = 0;
        while g.turn_number < 5 && !g.is_game_over() && actions < 4_000 {
            for (seat, bot) in bots.iter_mut().enumerate() {
                if let Some(a) = bot.next_action(&g, seat) {
                    let _ = g.perform_action(a);
                    actions += 1;
                }
            }
        }
        let states: Vec<TableState> = (0..4).map(|seat| encode_table(&g, seat, &vocab)).collect();
        assert!(states.iter().all(|s| s.desc_count() > 10), "real descriptors");

        let cfg = TableConfig { vocab: vocab.size(), ..small(1) };
        let trainer = TableTrainer::new(&cfg, 1e-3).expect("trainer");
        crate::tests::reseed_params(&trainer.varmap, &trainer.dev, 43);
        let net = export(&trainer, "pod");
        let refs: Vec<&TableState> = states.iter().collect();
        let want = trainer.predict(&refs).expect("candle forward");
        for (seat, (s, w)) in states.iter().zip(&want).enumerate() {
            let got = net.forward(s);
            for (g, w) in got.iter().zip(w) {
                assert!((g - w).abs() < 1e-4, "seat {seat}: engine {got:?} vs candle {w:?}");
            }
        }
    }

    /// One trainer learns a rule that only makes sense across seat counts:
    /// the seat with the largest seat feature 0 wins, at 2 to 6 seats in the
    /// same batches — and picks the winner on fresh tables.
    #[test]
    fn learns_the_winner_at_every_seat_count() {
        let _guard = crate::tests::TRAIN_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let cfg = small(0);
        let mut trainer = TableTrainer::new(&cfg, 3e-3).expect("trainer");
        crate::tests::reseed_params(&trainer.varmap, &trainer.dev, 41);
        let mut rng = StdRng::seed_from_u64(41);
        let label = |s: &TableState| -> usize {
            (0..s.seat_count()).max_by(|&a, &b| s.seat(a)[0].total_cmp(&s.seat(b)[0])).unwrap()
        };
        let rows: Vec<TableRow> = (0..3072)
            .map(|_| {
                let s = random_table(&mut rng, cfg.vocab);
                let mut win = vec![0.0; s.seat_count()];
                win[label(&s)] = 1.0;
                let life = vec![0.5; s.seat_count()];
                let seat_ids = (0..s.seat_count() as u8).collect();
                TableRow { state: s, win, life, game_len: 0.5, traj: 0, ply: 0, seat_ids, hand: vec![], library: vec![] }
            })
            .collect();
        let mut first = None;
        let mut last = f32::MAX;
        for _ in 0..500 {
            let batch: Vec<&TableRow> =
                (0..64).map(|_| &rows[rng.random_range(0..rows.len())]).collect();
            let loss = trainer.train_step(&batch).expect("step");
            first.get_or_insert(loss.win);
            last = loss.win;
        }
        let first = first.unwrap();
        assert!(last < first * 0.5, "cross-entropy failed to fall: first {first}, last {last}");

        let fresh: Vec<TableState> = (0..400).map(|_| random_table(&mut rng, cfg.vocab)).collect();
        let refs: Vec<&TableState> = fresh.iter().collect();
        let preds = trainer.predict(&refs).expect("predict");
        let correct = fresh
            .iter()
            .zip(&preds)
            .filter(|(s, p)| {
                let pick = (0..p.len()).max_by(|&a, &b| p[a].total_cmp(&p[b])).unwrap();
                pick == label(s)
            })
            .count();
        // Chance is ~29 % averaged over 2..=6 seats (116 / 400); this seed
        // picks 373. 768 training tables instead of 3,072 reached only 272 —
        // 37 noise features per seat overfit a small set, which is why the
        // set is this size.
        assert!(correct >= 340, "only {correct}/400 fresh winners picked");
    }

    /// The per-slot probabilities are a distribution over exactly the
    /// row's own seats, whatever the batch is padded to.
    /// The belief head learns what an opponent shows: each opponent seat's
    /// first feature names its "archetype", and its hand is drawn from that
    /// archetype's half of a 20-card pool — so top-5 recall should leave the
    /// uniform pool's 0.25 far behind, on rows it never trained on.
    #[test]
    fn the_belief_head_learns_an_opponents_hand_from_what_it_shows() {
        let cfg = TableConfig { pool: true, belief: true, ..small(0) };
        let mut pool = TablePool::default();
        for c in 0..20u16 {
            pool.push(c % 12, &[c, 100 + c]);
        }
        let pool = std::sync::Arc::new(pool);
        let mut rng = StdRng::seed_from_u64(17);
        let make = |rng: &mut StdRng| {
            let mut s = random_table(rng, cfg.vocab);
            s.pool = Some(pool.clone());
            let k = s.seat_count();
            let mut hand = vec![Vec::new()];
            for slot in 1..k {
                let arch = rng.random_range(0..2u16);
                s.seat_mut(slot)[0] = arch as f32;
                hand.push((0..4).map(|_| arch * 10 + rng.random_range(0..10u16)).collect());
            }
            let library = vec![Vec::new(); k];
            let win = {
                let mut w = vec![0.0; k];
                w[0] = 1.0;
                w
            };
            TableRow { state: s, win, life: vec![0.5; k], game_len: 0.5, traj: 0, ply: 0, seat_ids: (0..k as u8).collect(), hand, library }
        };
        let train: Vec<TableRow> = (0..256).map(|_| make(&mut rng)).collect();
        let held: Vec<TableRow> = (0..64).map(|_| make(&mut rng)).collect();
        let mut t = TableTrainer::new(&cfg, 1e-2).expect("trainer");
        let refs: Vec<&TableRow> = train.iter().collect();
        for _ in 0..100 {
            for chunk in refs.chunks(64) {
                t.train_step(chunk).expect("step");
            }
        }
        let hrefs: Vec<&TableRow> = held.iter().collect();
        let (head, uni, _, n) = t.belief_recall(&hrefs, 5, &Default::default()).expect("recall");
        eprintln!("belief recall {head} vs uniform {uni} over {n}");
        assert!(n > 50, "{n} opponent slots");
        assert!((uni - 0.25).abs() < 0.01, "uniform baseline {uni}");
        assert!(head > 0.40, "belief recall {head} vs uniform {uni}");
    }

    #[test]
    fn padded_slots_get_no_probability() {
        let cfg = small(0);
        let trainer = TableTrainer::new(&cfg, 1e-3).expect("trainer");
        let mut rng = StdRng::seed_from_u64(5);
        let mut duel = random_table(&mut rng, cfg.vocab);
        while duel.seat_count() != 2 {
            duel = random_table(&mut rng, cfg.vocab);
        }
        let mut pod = random_table(&mut rng, cfg.vocab);
        while pod.seat_count() != 6 {
            pod = random_table(&mut rng, cfg.vocab);
        }
        let batch = make_table_batch(&[&duel, &pod], trainer.device()).expect("batch");
        let probs: Vec<Vec<f32>> =
            trainer.model.forward(&batch).expect("forward").probs.to_vec2().unwrap();
        assert!(probs[0][2..].iter().all(|p| *p == 0.0), "padding took mass: {:?}", probs[0]);
        assert!((probs[0][..2].iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert!((probs[1].iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }
}
