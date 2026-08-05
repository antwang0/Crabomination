//! Candle-based training for the SOS sealed value net.
//!
//! The inference half — the forward pass the bot actually runs, plus the
//! encoded-row and shard formats — lives in `crabomination_nn`, which this
//! crate mirrors architecture-for-architecture and tensor-name-for-name
//! (`emb.weight`, `obj.{weight,bias}`, `trunk1.*`, `trunk2.*`,
//! `head_win.*`, and the training-only auxiliary heads `head_life.*` /
//! `head_len.*`). The `exported_weights_match_engine_inference` test holds
//! the two implementations to the same numbers; break either side and it
//! fails.
//!
//! Training targets: `win` (MSE on the sigmoid output, the AlphaZero
//! choice) plus two lightly-weighted auxiliary regressions — final life
//! difference and remaining game length — which exist for credit
//! assignment, not play: a bare win bit can't tell the net *which part* of
//! a position lost the game (the KataGo ownership-target lesson at MTG
//! scale).

use std::collections::{HashMap, VecDeque};

use candle_core::{DType, Device, Result as CResult, Tensor};
use candle_nn::{
    AdamW, Embedding, Linear, Module, Optimizer, ParamsAdamW, VarBuilder, VarMap, embedding,
    linear,
};
use crabomination_nn::{
    DeckRow, EMB_DIM, EncodedState, GLOBAL_FEATS, NUM_GROUPS, OBJ_FEATS, OBJ_HIDDEN, TRUNK_H1,
    TRUNK_H2, TrainRow,
};
use rand::{Rng, RngExt};

/// Net dimensions. The engine's loader reads sizes from the tensor shapes,
/// so these are free to change between runs — vocab excepted, which must
/// match the encoder's [`crabomination::server::encode::Vocab::size`].
#[derive(Debug, Clone, Copy)]
pub struct NetConfig {
    pub vocab: usize,
    pub emb_dim: usize,
    pub obj_hidden: usize,
    pub h1: usize,
    pub h2: usize,
}

impl NetConfig {
    /// The standard configuration (`crabomination_nn` constants).
    pub fn standard(vocab: usize) -> Self {
        Self { vocab, emb_dim: EMB_DIM, obj_hidden: OBJ_HIDDEN, h1: TRUNK_H1, h2: TRUNK_H2 }
    }

    fn trunk_in(&self) -> usize {
        NUM_GROUPS * 2 * self.obj_hidden + GLOBAL_FEATS
    }
}

/// The candle mirror of `crabomination_nn::PlayNet`, plus auxiliary heads.
pub struct PlayModel {
    emb: Embedding,
    obj: Linear,
    trunk1: Linear,
    trunk2: Linear,
    head_win: Linear,
    head_life: Linear,
    head_len: Linear,
}

pub fn build_model(cfg: &NetConfig, vb: VarBuilder) -> CResult<PlayModel> {
    Ok(PlayModel {
        emb: embedding(cfg.vocab, cfg.emb_dim, vb.pp("emb"))?,
        obj: linear(cfg.emb_dim + OBJ_FEATS, cfg.obj_hidden, vb.pp("obj"))?,
        trunk1: linear(cfg.trunk_in(), cfg.h1, vb.pp("trunk1"))?,
        trunk2: linear(cfg.h1, cfg.h2, vb.pp("trunk2"))?,
        head_win: linear(cfg.h2, 1, vb.pp("head_win"))?,
        head_life: linear(cfg.h2, 1, vb.pp("head_life"))?,
        head_len: linear(cfg.h2, 1, vb.pp("head_len"))?,
    })
}

/// One training batch: per-group padded id/feature/mask tensors plus
/// globals and labels. Group tensors are padded to the longest instance in
/// the batch (minimum 1 so shapes stay rank-3) with mask 0 on padding.
pub struct Batch {
    ids: Vec<Tensor>,   // per group: [B, N] u32
    feats: Vec<Tensor>, // per group: [B, N, OBJ_FEATS]
    mask: Vec<Tensor>,  // per group: [B, N, 1]
    global: Tensor,     // [B, GLOBAL_FEATS]
    pub win: Tensor,    // [B, 1]
    pub life: Tensor,   // [B, 1]
    pub len_t: Tensor,  // [B, 1]
}

/// [`make_batch_with_targets`] fitting each row's own game result — the
/// Monte Carlo target, and what every pre-λ gate round trained on.
pub fn make_batch(rows: &[&TrainRow], dev: &Device) -> CResult<Batch> {
    let with: Vec<(&TrainRow, f32)> = rows.iter().map(|r| (*r, r.win)).collect();
    make_batch_with_targets(&with, dev)
}

/// Pack rows into tensors, fitting the supplied win target rather than the
/// row's raw result — see [`SampleWindow::relabel_lambda`].
pub fn make_batch_with_targets(rows: &[(&TrainRow, f32)], dev: &Device) -> CResult<Batch> {
    let b = rows.len();
    let (mut ids, mut feats, mut mask) = (Vec::new(), Vec::new(), Vec::new());
    for g in 0..NUM_GROUPS {
        let n = rows.iter().map(|(r, _)| r.state.groups[g].len()).max().unwrap_or(0).max(1);
        let mut id_v = vec![0u32; b * n];
        let mut ft_v = vec![0f32; b * n * OBJ_FEATS];
        let mut mk_v = vec![0f32; b * n];
        for (ri, (row, _)) in rows.iter().enumerate() {
            for (oi, o) in row.state.groups[g].iter().enumerate() {
                id_v[ri * n + oi] = o.card as u32;
                ft_v[(ri * n + oi) * OBJ_FEATS..][..OBJ_FEATS].copy_from_slice(&o.feats);
                mk_v[ri * n + oi] = 1.0;
            }
        }
        ids.push(Tensor::from_vec(id_v, (b, n), dev)?);
        feats.push(Tensor::from_vec(ft_v, (b, n, OBJ_FEATS), dev)?);
        mask.push(Tensor::from_vec(mk_v, (b, n, 1), dev)?);
    }
    let mut gl = vec![0f32; b * GLOBAL_FEATS];
    let (mut win, mut life, mut len_t) = (vec![0f32; b], vec![0f32; b], vec![0f32; b]);
    for (ri, (row, target)) in rows.iter().enumerate() {
        gl[ri * GLOBAL_FEATS..][..GLOBAL_FEATS].copy_from_slice(&row.state.global);
        win[ri] = *target;
        life[ri] = row.life_diff;
        len_t[ri] = row.game_len;
    }
    Ok(Batch {
        ids,
        feats,
        mask,
        global: Tensor::from_vec(gl, (b, GLOBAL_FEATS), dev)?,
        win: Tensor::from_vec(win, (b, 1), dev)?,
        life: Tensor::from_vec(life, (b, 1), dev)?,
        len_t: Tensor::from_vec(len_t, (b, 1), dev)?,
    })
}

impl PlayModel {
    /// Returns `(win probability, life-diff, game-len)` predictions,
    /// each `[B, 1]`.
    pub fn forward(&self, batch: &Batch) -> CResult<(Tensor, Tensor, Tensor)> {
        let mut pooled: Vec<Tensor> = Vec::with_capacity(2 * NUM_GROUPS + 1);
        for g in 0..NUM_GROUPS {
            let e = self.emb.forward(&batch.ids[g])?; // [B,N,E]
            let x = Tensor::cat(&[&e, &batch.feats[g]], 2)?;
            let h = self.obj.forward(&x)?.relu()?; // [B,N,H]
            let hm = h.broadcast_mul(&batch.mask[g])?;
            let count = batch.mask[g].sum(1)?; // [B,1]
            // Mean over the real (unmasked) objects; zero for empty groups,
            // matching the engine forward's "empty slot stays zero".
            let mean = hm.sum(1)?.broadcast_div(&count.maximum(1f64)?)?;
            // Max over real objects: padding is pushed to -1e9 before the
            // reduction, then empty groups are zeroed via the 0/1 presence.
            let sink = batch.mask[g].affine(1e9, -1e9)?;
            let mx = hm.broadcast_add(&sink)?.max(1)?;
            let mx = mx.broadcast_mul(&count.minimum(1f64)?)?;
            pooled.push(mean);
            pooled.push(mx);
        }
        pooled.push(batch.global.clone());
        let refs: Vec<&Tensor> = pooled.iter().collect();
        let t = Tensor::cat(&refs, 1)?;
        let t1 = self.trunk1.forward(&t)?.relu()?;
        let t2 = self.trunk2.forward(&t1)?.relu()?;
        let win = candle_nn::ops::sigmoid(&self.head_win.forward(&t2)?)?;
        let life = self.head_life.forward(&t2)?;
        let len_p = self.head_len.forward(&t2)?;
        Ok((win, life, len_p))
    }
}

/// Model + optimizer + the varmap that owns the weights.
pub struct Trainer {
    pub varmap: VarMap,
    pub model: PlayModel,
    opt: AdamW,
    dev: Device,
}

/// Loss weights: the auxiliary targets regularize, they don't compete.
const AUX_WEIGHT: f64 = 0.25;

/// One training step's loss, whole and by head. `total` is what the
/// gradient descends (`win + 0.25·(life + len)`); the parts are raw MSEs.
#[derive(Debug, Clone, Copy)]
pub struct LossParts {
    pub total: f32,
    pub win: f32,
    pub life: f32,
    pub len: f32,
}

impl Trainer {
    pub fn new(cfg: &NetConfig, lr: f64) -> CResult<Trainer> {
        // CPU unless the crate was built with the `cuda` feature AND a
        // device is present — then this silently returns the GPU.
        let dev = Device::cuda_if_available(0)?;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &dev);
        let model = build_model(cfg, vb)?;
        let opt = AdamW::new(varmap.all_vars(), ParamsAdamW { lr, ..Default::default() })?;
        Ok(Trainer { varmap, model, opt, dev })
    }

    /// One SGD step over `rows`; returns the (pre-step) loss, decomposed.
    /// The components exist because a single number hid a real regime
    /// change once already: the round-1/round-2 EMAs (0.22 vs 0.30) were
    /// incomparable — different effective sample reuse — and there was no
    /// way to see whether the win head or the aux heads moved.
    pub fn train_step(&mut self, rows: &[&TrainRow]) -> CResult<LossParts> {
        let with: Vec<(&TrainRow, f32)> = rows.iter().map(|r| (*r, r.win)).collect();
        self.train_step_with_targets(&with)
    }

    /// [`train_step`](Self::train_step) fitting supplied win targets — the
    /// λ-return path (see [`SampleWindow::relabel_lambda`]).
    pub fn train_step_with_targets(&mut self, rows: &[(&TrainRow, f32)]) -> CResult<LossParts> {
        let batch = make_batch_with_targets(rows, &self.dev)?;
        let (win, life, len_p) = self.model.forward(&batch)?;
        let loss_win = candle_nn::loss::mse(&win, &batch.win)?;
        let loss_life = candle_nn::loss::mse(&life, &batch.life)?;
        let loss_len = candle_nn::loss::mse(&len_p, &batch.len_t)?;
        let loss = loss_win
            .add(&loss_life.affine(AUX_WEIGHT, 0.0)?)?
            .add(&loss_len.affine(AUX_WEIGHT, 0.0)?)?;
        self.opt.backward_step(&loss)?;
        Ok(LossParts {
            total: loss.to_scalar::<f32>()?,
            win: loss_win.to_scalar::<f32>()?,
            life: loss_life.to_scalar::<f32>()?,
            len: loss_len.to_scalar::<f32>()?,
        })
    }

    /// Win probability for one encoded state — the trainer-side twin of
    /// `crabomination_nn::PlayNet::forward`, used by the parity test and
    /// for quick evaluation during training.
    /// Win probability for a batch of rows in one forward pass — what
    /// [`SampleWindow::relabel_lambda`] bootstraps through. Batched
    /// because relabelling touches the whole window and one row at a time
    /// would dominate the learner's step time.
    pub fn predict_win_batch(&self, rows: &[&TrainRow], chunk: usize) -> CResult<Vec<f32>> {
        let mut out = Vec::with_capacity(rows.len());
        for part in rows.chunks(chunk.max(1)) {
            let batch = make_batch(part, &self.dev)?;
            let (win, _, _) = self.model.forward(&batch)?;
            out.extend(win.flatten_all()?.to_vec1::<f32>()?);
        }
        Ok(out)
    }

    pub fn predict_win(&self, s: &EncodedState) -> CResult<f32> {
        let row =
            TrainRow { state: s.clone(), win: 0.0, life_diff: 0.0, game_len: 0.0, traj: 0, ply: 0 };
        let batch = make_batch(&[&row], &self.dev)?;
        let (win, _, _) = self.model.forward(&batch)?;
        win.flatten_all()?.to_vec1::<f32>().map(|v| v[0])
    }

    /// Export weights as safetensors — the file `PlayNet::load` reads.
    pub fn save(&self, path: &std::path::Path) -> CResult<()> {
        self.varmap.save(path)
    }

    /// Resume from a previous export (shapes must match the config).
    pub fn load(&mut self, path: &std::path::Path) -> CResult<()> {
        self.varmap.load(path)
    }
}

// ─────────────────────────────── deck net ───────────────────────────────

/// Build-net dimensions. Small on purpose: the target is a 40-card
/// multiset, not a game state.
#[derive(Debug, Clone, Copy)]
pub struct DeckNetConfig {
    pub vocab: usize,
    pub emb_dim: usize,
    pub h1: usize,
    pub h2: usize,
}

impl DeckNetConfig {
    pub fn standard(vocab: usize) -> Self {
        Self { vocab, emb_dim: 32, h1: 128, h2: 64 }
    }
}

/// Candle mirror of `crabomination_nn::DeckNet` — pooled card embeddings
/// (sum ×0.1, mean, max) ⊕ deck features → two relu layers → sigmoid.
pub struct DeckModel {
    emb: Embedding,
    trunk1: Linear,
    trunk2: Linear,
    head: Linear,
}

pub fn build_deck_model(cfg: &DeckNetConfig, vb: VarBuilder) -> CResult<DeckModel> {
    Ok(DeckModel {
        emb: embedding(cfg.vocab, cfg.emb_dim, vb.pp("emb"))?,
        trunk1: linear(3 * cfg.emb_dim + crabomination_nn::DECK_FEATS, cfg.h1, vb.pp("trunk1"))?,
        trunk2: linear(cfg.h1, cfg.h2, vb.pp("trunk2"))?,
        head: linear(cfg.h2, 1, vb.pp("head_win"))?,
    })
}

impl DeckModel {
    /// `ids` [B,N] u32 (padded), `mask` [B,N,1], `feats` [B,DECK_FEATS] →
    /// win probability [B,1].
    fn forward(&self, ids: &Tensor, mask: &Tensor, feats: &Tensor) -> CResult<Tensor> {
        let e = self.emb.forward(ids)?; // [B,N,D]
        let em = e.broadcast_mul(mask)?;
        let count = mask.sum(1)?; // [B,1]
        let raw_sum = em.sum(1)?; // [B,D]
        let sum = raw_sum.affine(0.1, 0.0)?;
        let mean = raw_sum.broadcast_div(&count.maximum(1f64)?)?;
        let sink = mask.affine(1e9, -1e9)?;
        let mx = em.broadcast_add(&sink)?.max(1)?.broadcast_mul(&count.minimum(1f64)?)?;
        let t = Tensor::cat(&[&sum, &mean, &mx, feats], 1)?;
        let t1 = self.trunk1.forward(&t)?.relu()?;
        let t2 = self.trunk2.forward(&t1)?.relu()?;
        candle_nn::ops::sigmoid(&self.head.forward(&t2)?)
    }
}

/// Model + optimizer for the build net, mirroring [`Trainer`].
pub struct DeckTrainer {
    pub varmap: VarMap,
    pub model: DeckModel,
    opt: AdamW,
    dev: Device,
}

impl DeckTrainer {
    pub fn new(cfg: &DeckNetConfig, lr: f64) -> CResult<DeckTrainer> {
        let dev = Device::cuda_if_available(0)?;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &dev);
        let model = build_deck_model(cfg, vb)?;
        let opt = AdamW::new(varmap.all_vars(), ParamsAdamW { lr, ..Default::default() })?;
        Ok(DeckTrainer { varmap, model, opt, dev })
    }

    fn batch(&self, rows: &[&DeckRow]) -> CResult<(Tensor, Tensor, Tensor, Tensor)> {
        let b = rows.len();
        let n = rows.iter().map(|r| r.cards.len()).max().unwrap_or(0).max(1);
        let mut ids = vec![0u32; b * n];
        let mut mask = vec![0f32; b * n];
        let mut feats = vec![0f32; b * crabomination_nn::DECK_FEATS];
        let mut win = vec![0f32; b];
        for (ri, row) in rows.iter().enumerate() {
            for (ci, &c) in row.cards.iter().enumerate() {
                ids[ri * n + ci] = c as u32;
                mask[ri * n + ci] = 1.0;
            }
            feats[ri * crabomination_nn::DECK_FEATS..][..crabomination_nn::DECK_FEATS]
                .copy_from_slice(&row.feats);
            win[ri] = row.win;
        }
        Ok((
            Tensor::from_vec(ids, (b, n), &self.dev)?,
            Tensor::from_vec(mask, (b, n, 1), &self.dev)?,
            Tensor::from_vec(feats, (b, crabomination_nn::DECK_FEATS), &self.dev)?,
            Tensor::from_vec(win, (b, 1), &self.dev)?,
        ))
    }

    /// One SGD step; returns the win MSE.
    pub fn train_step(&mut self, rows: &[&DeckRow]) -> CResult<f32> {
        let (ids, mask, feats, win) = self.batch(rows)?;
        let pred = self.model.forward(&ids, &mask, &feats)?;
        let loss = candle_nn::loss::mse(&pred, &win)?;
        self.opt.backward_step(&loss)?;
        loss.to_scalar::<f32>()
    }

    pub fn predict(&self, row: &DeckRow) -> CResult<f32> {
        let (ids, mask, feats, _) = self.batch(&[row])?;
        let pred = self.model.forward(&ids, &mask, &feats)?;
        pred.flatten_all()?.to_vec1::<f32>().map(|v| v[0])
    }

    pub fn save(&self, path: &std::path::Path) -> CResult<()> {
        self.varmap.save(path)
    }

    pub fn load(&mut self, path: &std::path::Path) -> CResult<()> {
        self.varmap.load(path)
    }
}

/// Sliding window over recent training rows — the replay buffer. Uniform
/// sampling, FIFO eviction; the caller grows `cap` over the run (the
/// KataGo schedule) and throttles reads to cap sample reuse.
pub struct SampleWindow {
    rows: VecDeque<TrainRow>,
    /// Fit target per row, parallel to `rows`. Starts as the game result
    /// and is replaced by a λ-return whenever
    /// [`relabel_lambda`](Self::relabel_lambda) runs.
    targets: VecDeque<f32>,
    cap: usize,
}

impl SampleWindow {
    pub fn new(cap: usize) -> Self {
        Self {
            rows: VecDeque::with_capacity(cap.min(1 << 20)),
            targets: VecDeque::with_capacity(cap.min(1 << 20)),
            cap,
        }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Grow (or shrink) the window; excess oldest rows fall out.
    pub fn set_cap(&mut self, cap: usize) {
        self.cap = cap;
        while self.rows.len() > self.cap {
            self.rows.pop_front();
            self.targets.pop_front();
        }
    }

    pub fn push(&mut self, row: TrainRow) {
        if self.rows.len() == self.cap {
            self.rows.pop_front();
            self.targets.pop_front();
        }
        self.targets.push_back(row.win);
        self.rows.push_back(row);
    }

    /// `n` rows sampled uniformly with replacement.
    pub fn sample<'a, R: Rng>(&'a self, n: usize, rng: &mut R) -> Vec<&'a TrainRow> {
        (0..n).map(|_| &self.rows[rng.random_range(0..self.rows.len())]).collect()
    }

    /// `n` (row, fit target) pairs sampled uniformly with replacement.
    pub fn sample_with_targets<'a, R: Rng>(
        &'a self,
        n: usize,
        rng: &mut R,
    ) -> Vec<(&'a TrainRow, f32)> {
        (0..n)
            .map(|_| {
                let i = rng.random_range(0..self.rows.len());
                (&self.rows[i], self.targets[i])
            })
            .collect()
    }

    /// Replace every row's fit target with its **λ-return**, bootstrapped
    /// through `value` (the current net's win probability).
    ///
    /// The problem this solves: labelling a turn-2 state with the eventual
    /// winner of a twenty-turn game is mostly labelling noise. The result
    /// is real, but almost none of it was determined at turn 2, so the net
    /// spends capacity fitting variance. That is the standard Monte Carlo
    /// target and it is what every gate round so far has trained on.
    ///
    /// For an episodic task whose only reward is terminal, the forward
    /// view collapses to a backward recursion along each trajectory:
    ///
    /// ```text
    /// G_T = z                                  (the actual result)
    /// G_t = (1 − λ)·V(s_{t+1}) + λ·G_{t+1}
    /// ```
    ///
    /// λ = 1 is exactly the current Monte Carlo behaviour, so this is a
    /// strict generalisation and the old target stays reachable as a
    /// control. λ = 0 is one-step TD. Between them the target trades the
    /// variance of the result for the bias of the net's own estimate —
    /// worth measuring, not assuming.
    ///
    /// Trajectories are reassembled by [`TrainRow::traj`] and ordered by
    /// [`TrainRow::ply`]. A trajectory whose tail has already been evicted
    /// from the window simply bootstraps from its last surviving row,
    /// which is the correct thing to do: that row's own target is
    /// whatever it was last assigned.
    pub fn relabel_lambda(&mut self, lambda: f32, value: impl Fn(&[&TrainRow]) -> Vec<f32>) {
        if self.rows.is_empty() {
            return;
        }
        let preds = value(&self.rows.iter().collect::<Vec<_>>());
        if preds.len() != self.rows.len() {
            return; // a failed forward pass leaves the targets alone
        }
        // Group row indices by trajectory, then walk each backwards.
        let mut by_traj: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, r) in self.rows.iter().enumerate() {
            by_traj.entry(r.traj).or_default().push(i);
        }
        for idxs in by_traj.values_mut() {
            idxs.sort_by_key(|&i| self.rows[i].ply);
            // The last surviving row of the trajectory keeps the result:
            // either it really is terminal, or its successors are gone and
            // there is nothing to bootstrap through.
            let mut carried = self.rows[*idxs.last().unwrap()].win;
            self.targets[*idxs.last().unwrap()] = carried;
            for w in idxs.windows(2).rev() {
                let (t, next) = (w[0], w[1]);
                carried = (1.0 - lambda) * preds[next] + lambda * carried;
                self.targets[t] = carried.clamp(0.0, 1.0);
            }
        }
    }

    /// Read-only view of the current fit targets, for tests and logging.
    pub fn targets(&self) -> &VecDeque<f32> {
        &self.targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination_nn::{EncodedObject, PlayNet};
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn small_cfg() -> NetConfig {
        NetConfig { vocab: 12, emb_dim: 4, obj_hidden: 8, h1: 32, h2: 16 }
    }

    fn random_state<R: Rng>(rng: &mut R, vocab: usize) -> EncodedState {
        let mut s = EncodedState::default();
        for g in s.global.iter_mut() {
            *g = rng.random_range(-1.0..1.0);
        }
        for group in s.groups.iter_mut() {
            let n = rng.random_range(0..5);
            for _ in 0..n {
                let mut feats = [0.0f32; OBJ_FEATS];
                for f in feats.iter_mut() {
                    *f = rng.random_range(0.0..1.0);
                }
                group.push(EncodedObject { card: rng.random_range(0..vocab as u16), feats });
            }
        }
        s
    }

    /// The candle model and the engine's dependency-free forward pass must
    /// produce the same numbers from the same weights — this is the
    /// trainer/inference contract.
    #[test]
    fn exported_weights_match_engine_inference() {
        let cfg = small_cfg();
        let trainer = Trainer::new(&cfg, 1e-3).expect("trainer");
        let path = std::env::temp_dir().join(format!("crab_parity_{}.safetensors", std::process::id()));
        trainer.save(&path).expect("save");
        let bytes = std::fs::read(&path).expect("read");
        let _ = std::fs::remove_file(&path);
        let net = PlayNet::load(&bytes).expect("engine loads trainer export");
        assert_eq!(net.vocab_size(), cfg.vocab);

        let mut rng = StdRng::seed_from_u64(7);
        for i in 0..8 {
            let s = random_state(&mut rng, cfg.vocab);
            let want = trainer.predict_win(&s).expect("candle forward");
            let got = net.forward(&s);
            assert!(
                (got - want).abs() < 1e-4,
                "state {i}: engine {got} vs candle {want}"
            );
        }
    }

    /// End-to-end learning sanity: a synthetic signal (life lead wins,
    /// with the answer also visible through an object feature) must be
    /// learnable to well under coin-flip loss in a few hundred steps.
    #[test]
    fn learns_a_synthetic_signal() {
        let cfg = small_cfg();
        let mut trainer = Trainer::new(&cfg, 3e-3).expect("trainer");
        let mut rng = StdRng::seed_from_u64(11);
        let rows: Vec<TrainRow> = (0..512)
            .map(|_| {
                let s = random_state(&mut rng, cfg.vocab);
                let win = if s.global[0] > s.global[1] { 1.0 } else { 0.0 };
                let life_diff = (s.global[0] - s.global[1]).clamp(-1.0, 1.0);
                TrainRow { state: s, win, life_diff, game_len: 0.5, traj: 0, ply: 0 }
            })
            .collect();

        let first = {
            let batch: Vec<&TrainRow> = rows[..64].iter().collect();
            let mut t2 = Trainer::new(&cfg, 0.0).expect("probe trainer");
            t2.train_step(&batch).expect("loss probe").total
        };
        let mut last = f32::MAX;
        for step in 0..400 {
            let batch: Vec<&TrainRow> =
                (0..64).map(|_| &rows[rng.random_range(0..rows.len())]).collect();
            last = trainer.train_step(&batch).expect("step").total;
            if step % 100 == 0 && last < 0.02 {
                break;
            }
        }
        assert!(
            last < first * 0.5 && last < 0.08,
            "loss failed to fall: first {first}, last {last}"
        );

        // The learned rule generalizes to fresh states.
        let mut correct = 0;
        for _ in 0..100 {
            let s = random_state(&mut rng, cfg.vocab);
            let p = trainer.predict_win(&s).expect("predict");
            if (p > 0.5) == (s.global[0] > s.global[1]) {
                correct += 1;
            }
        }
        assert!(correct >= 85, "only {correct}/100 fresh states classified");
    }

    fn random_deck_row<R: Rng>(rng: &mut R, vocab: usize) -> DeckRow {
        let cards = (0..40).map(|_| rng.random_range(0..vocab as u16)).collect();
        let mut feats = [0.0f32; crabomination_nn::DECK_FEATS];
        for f in feats.iter_mut() {
            *f = rng.random_range(0.0..1.0);
        }
        DeckRow { cards, feats, win: 0.0 }
    }

    /// Deck-net parity: candle export loads into the engine DeckNet and
    /// the two agree on random decklists.
    #[test]
    fn deck_export_matches_engine_inference() {
        let cfg = DeckNetConfig { vocab: 20, emb_dim: 8, h1: 32, h2: 16 };
        let trainer = DeckTrainer::new(&cfg, 1e-3).expect("deck trainer");
        let path =
            std::env::temp_dir().join(format!("crab_deck_parity_{}.safetensors", std::process::id()));
        trainer.save(&path).expect("save");
        let bytes = std::fs::read(&path).expect("read");
        let _ = std::fs::remove_file(&path);
        let net = crabomination_nn::DeckNet::load(&bytes).expect("engine loads deck export");

        let mut rng = StdRng::seed_from_u64(23);
        for i in 0..8 {
            let row = random_deck_row(&mut rng, cfg.vocab);
            let want = trainer.predict(&row).expect("candle forward");
            let got = net.forward(&row.cards, &row.feats);
            assert!((got - want).abs() < 1e-4, "deck {i}: engine {got} vs candle {want}");
        }
    }

    /// The deck net learns a synthetic build rule (low curve wins).
    #[test]
    fn deck_net_learns_a_synthetic_signal() {
        let cfg = DeckNetConfig { vocab: 20, emb_dim: 8, h1: 32, h2: 16 };
        let mut trainer = DeckTrainer::new(&cfg, 3e-3).expect("deck trainer");
        let mut rng = StdRng::seed_from_u64(29);
        let rows: Vec<DeckRow> = (0..512)
            .map(|_| {
                let mut r = random_deck_row(&mut rng, cfg.vocab);
                r.win = if r.feats[0] + r.feats[1] > r.feats[5] + r.feats[6] { 1.0 } else { 0.0 };
                r
            })
            .collect();
        let mut last = f32::MAX;
        for _ in 0..400 {
            let batch: Vec<&DeckRow> =
                (0..64).map(|_| &rows[rng.random_range(0..rows.len())]).collect();
            last = trainer.train_step(&batch).expect("step");
        }
        assert!(last < 0.08, "deck loss failed to fall: {last}");
        let mut correct = 0;
        for _ in 0..100 {
            let mut r = random_deck_row(&mut rng, cfg.vocab);
            r.win = if r.feats[0] + r.feats[1] > r.feats[5] + r.feats[6] { 1.0 } else { 0.0 };
            let p = trainer.predict(&r).expect("predict");
            if (p > 0.5) == (r.win > 0.5) {
                correct += 1;
            }
        }
        assert!(correct >= 85, "only {correct}/100 fresh decks classified");
    }

    #[test]
    fn window_evicts_oldest_and_respects_cap() {
        let mut w = SampleWindow::new(3);
        for i in 0..5 {
            let mut row = TrainRow {
                state: EncodedState::default(),
                win: i as f32,
                life_diff: 0.0,
                game_len: 0.0,
                traj: 0,
                ply: i as u16,
            };
            row.state.global[0] = i as f32;
            w.push(row);
        }
        assert_eq!(w.len(), 3);
        let mut rng = StdRng::seed_from_u64(1);
        assert!(w.sample(10, &mut rng).iter().all(|r| r.win >= 2.0));
        w.set_cap(1);
        assert_eq!(w.len(), 1);
        assert_eq!(w.targets().len(), 1, "targets must track evictions");
    }

    fn traj_row(traj: u32, ply: u16, win: f32) -> TrainRow {
        TrainRow {
            state: EncodedState::default(),
            win,
            life_diff: 0.0,
            game_len: 0.0,
            traj,
            ply,
        }
    }

    /// lambda = 1 is pure Monte Carlo: every row in a trajectory keeps the
    /// game result, whatever the net thinks. This is the pre-lambda
    /// behaviour and has to stay bit-reachable, because every gate round
    /// on record trained on it.
    #[test]
    fn lambda_one_is_the_monte_carlo_target() {
        let mut w = SampleWindow::new(16);
        for ply in 0..4 {
            w.push(traj_row(7, ply, 1.0));
        }
        // A net that is confidently wrong at every step.
        w.relabel_lambda(1.0, |rows| vec![0.0; rows.len()]);
        assert!(w.targets().iter().all(|&t| t == 1.0), "{:?}", w.targets());
    }

    /// lambda = 0 is one-step TD: each row takes the net's estimate of the
    /// *next* state, and only the terminal row keeps the result.
    #[test]
    fn lambda_zero_is_one_step_bootstrapping() {
        let mut w = SampleWindow::new(16);
        for ply in 0..4 {
            w.push(traj_row(7, ply, 1.0));
        }
        // V(s_t) = 0.25 * t, so row t should take 0.25 * (t + 1).
        w.relabel_lambda(0.0, |rows| rows.iter().map(|r| 0.25 * r.ply as f32).collect());
        let got: Vec<f32> = w.targets().iter().copied().collect();
        assert!((got[0] - 0.25).abs() < 1e-6, "{got:?}");
        assert!((got[1] - 0.50).abs() < 1e-6, "{got:?}");
        assert!((got[2] - 0.75).abs() < 1e-6, "{got:?}");
        assert!((got[3] - 1.00).abs() < 1e-6, "terminal keeps the result: {got:?}");
    }

    /// Intermediate lambda interpolates, and the recursion must run along
    /// each trajectory separately — a row can only bootstrap through its
    /// own successors, never the other seat's.
    #[test]
    fn lambda_returns_do_not_cross_trajectories() {
        let mut w = SampleWindow::new(16);
        // Trajectory 1 won; trajectory 2 lost. Interleaved on push, as the
        // recorder emits them.
        for ply in 0..3 {
            w.push(traj_row(1, ply, 1.0));
            w.push(traj_row(2, ply, 0.0));
        }
        w.relabel_lambda(0.5, |rows| vec![0.5; rows.len()]);
        let t: Vec<f32> = w.targets().iter().copied().collect();
        // Winner's trajectory: terminal 1.0, then 0.5*0.5 + 0.5*1.0 = 0.75,
        // then 0.5*0.5 + 0.5*0.75 = 0.625.
        assert!((t[4] - 1.0).abs() < 1e-6, "{t:?}");
        assert!((t[2] - 0.75).abs() < 1e-6, "{t:?}");
        assert!((t[0] - 0.625).abs() < 1e-6, "{t:?}");
        // Loser's: terminal 0.0, then 0.25, then 0.375 — never pulled
        // toward the winner's 1.0.
        assert!((t[5] - 0.0).abs() < 1e-6, "{t:?}");
        assert!((t[3] - 0.25).abs() < 1e-6, "{t:?}");
        assert!((t[1] - 0.375).abs() < 1e-6, "{t:?}");
    }
}
