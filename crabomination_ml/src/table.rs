//! Candle training for the table net — the mirror of
//! [`crabomination_nn::table`], tensor name for tensor name.
//!
//! One model for every seat count: a batch mixes duels and pods, padded to
//! its widest table with a seat mask, and the value head fits each row's
//! winner with a cross-entropy over the living slots. The parity tests below
//! hold this model and the engine's dependency-free forward to the same
//! numbers across mixed seat counts in one batch — padding is where two
//! implementations of the same net most easily disagree.

use candle_core::{D, DType, Device, Result as CResult, Tensor};
use candle_nn::{
    AdamW, Embedding, Linear, Module, Optimizer, ParamsAdamW, VarBuilder, VarMap, embedding, linear,
};
use crabomination_nn::table::{
    DESC_BUCKETS, NO_DESC, SEAT_FEATS, SEAT_HIDDEN, SLOT_TAGS, TABLE_GLOBAL_FEATS, TABLE_OBJ_FEATS,
    TABLE_ZONES, TableRow, TableState, VALUE_HIDDEN, slot_tag,
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
        trunk1: linear(3 * sh + TABLE_GLOBAL_FEATS, cfg.h1, vb.pp("trunk1"))?,
        trunk2: linear(cfg.h1, cfg.h2, vb.pp("trunk2"))?,
        value1: linear(cfg.h2 + sh, cfg.value_hidden, vb.pp("value1"))?,
        value2: linear(cfg.value_hidden, 1, vb.pp("value2"))?,
        head_life: linear(cfg.value_hidden, 1, vb.pp("head_life"))?,
        head_len: linear(cfg.h2, 1, vb.pp("head_len"))?,
    })
}

/// A batch of tables padded to its widest one. Group `g = slot *
/// TABLE_ZONES + zone` for `slot < slots`; a row with fewer seats has empty
/// groups (mask 0) past its own, and seat mask 0 on those slots.
pub struct TableBatch {
    b: usize,
    slots: usize,
    ids: Vec<Tensor>,   // per group [B, n] u32
    didx: Vec<Tensor>,  // per group [B * n] u32 — rows of the batch's descriptor table
    feats: Vec<Tensor>, // per group [B, n, TABLE_OBJ_FEATS]
    mask: Vec<Tensor>,  // per group [B, n, 1]
    /// Every descriptor in the batch plus one all-padding row at the end,
    /// which objects without a descriptor (and padding) point at.
    desc_tok: Tensor, // [D + 1, T] u32
    desc_mask: Tensor,  // [D + 1, T, 1]
    seat_feats: Tensor, // [B, S, SEAT_FEATS]
    seat_mask: Tensor,  // [B, S]
    global: Tensor,     // [B, TABLE_GLOBAL_FEATS]
}

pub fn make_table_batch(states: &[&TableState], dev: &Device) -> CResult<TableBatch> {
    let b = states.len();
    let slots = states.iter().map(|s| s.seat_count()).max().unwrap_or(0).max(1);

    // The batch's descriptor table: each state's descriptors at an offset.
    let mut offsets = Vec::with_capacity(b);
    let mut total = 0usize;
    let mut t_max = 1usize;
    for s in states {
        offsets.push(total);
        total += s.desc_count();
        for d in 0..s.desc_count() {
            t_max = t_max.max(s.desc(d).len());
        }
    }
    let none_row = total as u32;
    let mut tok = vec![0u32; (total + 1) * t_max];
    let mut tmask = vec![0f32; (total + 1) * t_max];
    for (s, off) in states.iter().zip(&offsets) {
        for d in 0..s.desc_count() {
            let row = off + d;
            for (j, &t) in s.desc(d).iter().enumerate() {
                tok[row * t_max + j] = u32::from(t);
                tmask[row * t_max + j] = 1.0;
            }
        }
    }

    let groups = slots * TABLE_ZONES;
    let (mut ids, mut didx, mut feats, mut mask) = (
        Vec::with_capacity(groups),
        Vec::with_capacity(groups),
        Vec::with_capacity(groups),
        Vec::with_capacity(groups),
    );
    for g in 0..groups {
        let (slot, zone) = (g / TABLE_ZONES, g % TABLE_ZONES);
        let n = states
            .iter()
            .filter(|s| slot < s.seat_count())
            .map(|s| s.group_len(slot, zone))
            .max()
            .unwrap_or(0)
            .max(1);
        let mut id_v = vec![0u32; b * n];
        let mut di_v = vec![none_row; b * n];
        let mut ft_v = vec![0f32; b * n * TABLE_OBJ_FEATS];
        let mut mk_v = vec![0f32; b * n];
        for (ri, s) in states.iter().enumerate() {
            if slot >= s.seat_count() {
                continue;
            }
            for (oi, o) in s.group(slot, zone).iter().enumerate() {
                let at = ri * n + oi;
                id_v[at] = u32::from(o.card);
                if o.desc != NO_DESC {
                    di_v[at] = (offsets[ri] + o.desc as usize) as u32;
                }
                ft_v[at * TABLE_OBJ_FEATS..][..TABLE_OBJ_FEATS].copy_from_slice(&o.feats);
                mk_v[at] = 1.0;
            }
        }
        ids.push(Tensor::from_vec(id_v, (b, n), dev)?);
        didx.push(Tensor::from_vec(di_v, b * n, dev)?);
        feats.push(Tensor::from_vec(ft_v, (b, n, TABLE_OBJ_FEATS), dev)?);
        mask.push(Tensor::from_vec(mk_v, (b, n, 1), dev)?);
    }

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
        ids,
        didx,
        feats,
        mask,
        desc_tok: Tensor::from_vec(tok, (total + 1, t_max), dev)?,
        desc_mask: Tensor::from_vec(tmask, (total + 1, t_max, 1), dev)?,
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
}

impl TableModel {
    pub fn forward(&self, batch: &TableBatch) -> CResult<TableOut> {
        let (b, slots) = (batch.b, batch.slots);

        // Descriptor embeddings: the masked mean of each descriptor's token
        // rows. The last row is all padding and embeds as zeros.
        let te = self.tok_emb.forward(&batch.desc_tok)?; // [D+1, T, E]
        let (demb, _) = masked_mean_max(&te, &batch.desc_mask)?; // [D+1, E]

        let e_dim = demb.dim(1)?;
        let mut hs = Vec::with_capacity(batch.ids.len());
        for g in 0..batch.ids.len() {
            let n = batch.ids[g].dim(1)?;
            let e = self.name_emb.forward(&batch.ids[g])?; // [B, n, E]
            let d = demb.index_select(&batch.didx[g], 0)?.reshape((b, n, e_dim))?;
            let x = Tensor::cat(&[&e, &d, &batch.feats[g]], 2)?;
            hs.push(self.obj.forward(&x)?.relu()?);
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
            let m = Tensor::cat(&batch.mask.iter().collect::<Vec<_>>(), 1)?;
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
                let (mean, mx) = masked_mean_max(&hs[g], &batch.mask[g])?;
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
        let t = Tensor::cat(&[&me, &opp_mean, &opp_max, &batch.global], 1)?;
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
        Ok(TableOut { logits, probs, life, len })
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
        let states: Vec<&TableState> = rows.iter().map(|r| &r.state).collect();
        let batch = make_table_batch(&states, &self.dev)?;
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
        let logp = candle_nn::ops::log_softmax(&out.logits, D::Minus1)?;
        let loss_win = win.mul(&logp)?.sum(1)?.neg()?.mean_all()?;
        let diff = out.life.sub(&life_t)?.mul(&batch.seat_mask)?;
        let loss_life =
            diff.sqr()?.sum_all()?.broadcast_div(&batch.seat_mask.sum_all()?.maximum(1f64)?)?;
        let loss_len = candle_nn::loss::mse(&out.len, &len_t)?;
        let loss = loss_win
            .add(&loss_life.affine(AUX_WEIGHT, 0.0)?)?
            .add(&loss_len.affine(AUX_WEIGHT, 0.0)?)?;
        self.opt.backward_step(&loss)?;
        Ok(TableLoss {
            total: loss.to_scalar::<f32>()?,
            win: loss_win.to_scalar::<f32>()?,
            life: loss_life.to_scalar::<f32>()?,
            len: loss_len.to_scalar::<f32>()?,
        })
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
                TableRow { state: s, win, life, game_len: 0.5, traj: 0, ply: 0 }
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
