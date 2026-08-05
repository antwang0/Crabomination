//! Dependency-light neural-net inference and training-row plumbing.
//!
//! This crate is the seam between the engine and the ML training stack:
//!
//! * The engine (`crabomination`) encodes a [`GameState`] into an
//!   [`EncodedState`] and calls [`PlayNet::forward`] during play. It must
//!   stay free of any ML framework — the engine compiles for wasm32 (the
//!   browser client links it) and its debug builds are opt-level 0, so
//!   heavy dependencies are off the table.
//! * The trainer (`crabomination_ml`, candle-based) produces weights as a
//!   safetensors file that [`PlayNet::load`] reads. Tensor names and the
//!   architecture below are the contract between the two; a parity test in
//!   `crabomination_ml` holds both sides to it.
//!
//! This crate lives outside the engine so the workspace can give it
//! `opt-level = 3` in debug builds (see the root `Cargo.toml`): the forward
//! pass runs inside the bot's simulation loops, where the workspace's
//! opt-level-0 engine code would make a ~130k-parameter net cost more than
//! the game logic around it.
//!
//! ## Architecture (v1)
//!
//! A deep-sets value net over the observable state, seat-relative:
//!
//! * An embedding row per card name ([`EncodedObject::card`]; index 0 is
//!   the unknown/token fallback).
//! * Per object: `relu(W_obj · [embedding ⊕ object-state features])`,
//!   shared weights across all groups, then mean- and max-pooled within
//!   each of the [`NUM_GROUPS`] zone groups (group identity is positional
//!   in the trunk input).
//! * Trunk: two relu layers over `[pooled groups ⊕ global features]`,
//!   then a scalar win-probability head (sigmoid). The trainer adds
//!   auxiliary heads (final life difference, game length); inference
//!   ignores any tensor it doesn't need.
//!
//! Hidden sizes are read from the tensor shapes, not hardcoded — the
//! constants below describe the *standard* configuration the trainer uses
//! and the parts the encoder bakes into the data (feature counts, group
//! count), which genuinely are fixed per format version.

use std::collections::BTreeMap;

/// Zone groups, in trunk-input order. Seat-relative: "self" is the seat
/// being evaluated.
pub const NUM_GROUPS: usize = 5;
/// Group indices into [`EncodedState::groups`].
pub const G_BF_SELF: usize = 0;
pub const G_BF_OPP: usize = 1;
pub const G_HAND_SELF: usize = 2;
pub const G_GY_SELF: usize = 3;
pub const G_GY_OPP: usize = 4;

/// Per-object feature count. Baked into encoded rows — bump
/// [`SHARD_VERSION`] if it changes. Feats 12..=19 are the evasion/combat
/// keyword flags added in round 4 (flying, reach, menace, deathtouch,
/// lifelink, trample, first-or-double strike, vigilance): the pooled
/// encoder previously saw a Serra Angel and a Hill Giant as the same
/// 4-mana 4/4-ish body.
pub const OBJ_FEATS: usize = 20;
/// Global scalar feature count. Baked into encoded rows likewise.
pub const GLOBAL_FEATS: usize = 24;

/// Standard trainer configuration (the file's shapes win at load time).
/// Sizes quadrupled in round 4 (~600 k parameters, from ~125 k): four
/// gate rounds measured the small net flat at 42–45 % as a replacement
/// judge across data-volume and distribution fixes, leaving capacity the
/// first untested lever.
pub const EMB_DIM: usize = 32;
pub const OBJ_HIDDEN: usize = 64;
pub const TRUNK_H1: usize = 512;
pub const TRUNK_H2: usize = 256;

/// One card object as the net sees it: a vocabulary index plus a small
/// dense feature vector (effective P/T, tapped, counters, ...). Index 0 is
/// reserved for unknown names — tokens and anything outside the vocab.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedObject {
    pub card: u16,
    pub feats: [f32; OBJ_FEATS],
}

/// A full observable position from one seat's perspective.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EncodedState {
    pub global: [f32; GLOBAL_FEATS],
    pub groups: [Vec<EncodedObject>; NUM_GROUPS],
}

impl Default for EncodedObject {
    fn default() -> Self {
        Self { card: 0, feats: [0.0; OBJ_FEATS] }
    }
}

/// A labelled training example. Labels are stamped after the game ends:
/// `win` is 1.0 if the encoded seat won (0.0 otherwise, 0.5 for a draw),
/// `life_diff` is the final (self − opp) life clamped to ±20 and scaled by
/// 1/20, `game_len` is the number of turns the game still had to run from
/// this snapshot, scaled by 1/15 — both auxiliary targets exist for credit
/// assignment (the KataGo lesson: a bare win bit can't say *why*), not for
/// play.
#[derive(Debug, Clone, PartialEq)]
pub struct TrainRow {
    pub state: EncodedState,
    /// The game's actual result from this seat: 1 win, 0 loss. This stays
    /// the ground truth even when the trainer fits a bootstrapped target
    /// derived from it (see `SampleWindow::relabel_lambda`).
    pub win: f32,
    pub life_diff: f32,
    pub game_len: f32,
    /// Trajectory this row belongs to — one per (game, seat), since the
    /// two seats see different information and are separate episodes.
    /// Rows can only be bootstrapped against their own successors, so the
    /// trainer needs to be able to reassemble trajectories from a shuffled
    /// window.
    pub traj: u32,
    /// Position within the trajectory, ascending. Recorded rather than
    /// inferred from row order because the window samples and evicts.
    pub ply: u16,
}

// ───────────────────────────── shard format ─────────────────────────────

/// Magic + version guard the on-disk row format. Bump the version whenever
/// `OBJ_FEATS`, `GLOBAL_FEATS`, group order, or the row layout changes —
/// stale shards must fail loudly, not decode as garbage.
pub const SHARD_MAGIC: [u8; 4] = *b"CRML";
pub const SHARD_VERSION: u32 = 3;

/// Serialize rows into a self-describing shard (little-endian throughout).
pub fn write_shard(rows: &[TrainRow]) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 + rows.len() * 256);
    out.extend_from_slice(&SHARD_MAGIC);
    out.extend_from_slice(&SHARD_VERSION.to_le_bytes());
    out.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    for row in rows {
        for v in row.state.global {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for group in &row.state.groups {
            out.extend_from_slice(&(group.len() as u16).to_le_bytes());
            for o in group {
                out.extend_from_slice(&o.card.to_le_bytes());
                for v in o.feats {
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
        for v in [row.win, row.life_diff, row.game_len] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&row.traj.to_le_bytes());
        out.extend_from_slice(&row.ply.to_le_bytes());
    }
    out
}

/// Decode a shard produced by [`write_shard`]. Returns `None` on any
/// magic/version/length mismatch — a torn or stale shard is dropped whole
/// rather than half-read.
pub fn read_shard(bytes: &[u8]) -> Option<Vec<TrainRow>> {
    let mut r = Reader { b: bytes, pos: 0 };
    if r.take(4)? != SHARD_MAGIC {
        return None;
    }
    if r.u32()? != SHARD_VERSION {
        return None;
    }
    let n = r.u32()? as usize;
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let mut state = EncodedState::default();
        for g in state.global.iter_mut() {
            *g = r.f32()?;
        }
        for group in state.groups.iter_mut() {
            let len = r.u16()? as usize;
            group.reserve(len);
            for _ in 0..len {
                let card = r.u16()?;
                let mut feats = [0.0; OBJ_FEATS];
                for f in feats.iter_mut() {
                    *f = r.f32()?;
                }
                group.push(EncodedObject { card, feats });
            }
        }
        let win = r.f32()?;
        let life_diff = r.f32()?;
        let game_len = r.f32()?;
        let traj = r.u32()?;
        let ply = r.u16()?;
        rows.push(TrainRow { state, win, life_diff, game_len, traj, ply });
    }
    // Trailing bytes mean the writer and reader disagree about the layout.
    if r.pos != bytes.len() {
        return None;
    }
    Some(rows)
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.pos..self.pos + n)?;
        self.pos += n;
        Some(s)
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}

// ─────────────────────────────── deck net ───────────────────────────────

/// Per-deck global feature count (curve buckets, land/creature counts,
/// color pips — see the engine's `encode_deck`). Baked into deck rows.
pub const DECK_FEATS: usize = 16;

/// A labelled decklist for the build net: the 40 cards as vocab indices
/// plus dense deck-level features, labelled with the game outcome. Every
/// self-play game yields two of these for free — the winner's list and
/// the loser's.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckRow {
    pub cards: Vec<u16>,
    pub feats: [f32; DECK_FEATS],
    pub win: f32,
}

/// The build-evaluation net `D(decklist) → win probability`. Deep-sets
/// over the card multiset: embedding rows summed, meaned and maxed, then
/// a two-layer trunk over `[pools ⊕ deck features]`. Lives in its own
/// safetensors file (same tensor names as the play net's trunk — the
/// files are separate, so there is no collision).
///
/// **First ML component to pass a house gate**: judging best-of-32
/// sealed builds against the heuristic `static_build_score` over the
/// same candidate sets (identical pilots and seeds — only the judge
/// differs), net-judged builds won 61.7 % [58.9, 64.4] and, on a fresh
/// seed, 60.7 % [57.9, 63.4] over 1 200 games each
/// (`selfplay_train --gate-builder`). Trained from ~100 k self-play
/// games' decklist labels riding along with a play-net run.
#[derive(Debug, Clone)]
pub struct DeckNet {
    emb: Tensor2,      // [vocab, emb_dim]
    trunk1_w: Tensor2, // [h1, 3*emb_dim + DECK_FEATS]
    trunk1_b: Vec<f32>,
    trunk2_w: Tensor2, // [h2, h1]
    trunk2_b: Vec<f32>,
    head_w: Tensor2, // [1, h2]
    head_b: Vec<f32>,
}

impl DeckNet {
    pub fn vocab_size(&self) -> usize {
        self.emb.rows
    }

    pub fn load(bytes: &[u8]) -> Result<DeckNet, NnError> {
        let st = safetensors::SafeTensors::deserialize(bytes)
            .map_err(|e| NnError::BadFile(e.to_string()))?;
        let get = |name: &'static str| get_tensor(&st, name);
        let net = DeckNet {
            emb: get("emb.weight")?,
            trunk1_w: get("trunk1.weight")?,
            trunk1_b: get("trunk1.bias")?.data,
            trunk2_w: get("trunk2.weight")?,
            trunk2_b: get("trunk2.bias")?.data,
            head_w: get("head_win.weight")?,
            head_b: get("head_win.bias")?.data,
        };
        if net.trunk1_w.cols != 3 * net.emb.cols + DECK_FEATS {
            return Err(NnError::BadTensor(
                "trunk1.weight",
                format!("cols {} don't match 3×{} + {DECK_FEATS}", net.trunk1_w.cols, net.emb.cols),
            ));
        }
        for (name, ok) in [
            ("trunk1.bias", net.trunk1_b.len() == net.trunk1_w.rows),
            ("trunk2.weight", net.trunk2_w.cols == net.trunk1_w.rows),
            ("trunk2.bias", net.trunk2_b.len() == net.trunk2_w.rows),
            ("head_win.weight", net.head_w.cols == net.trunk2_w.rows && net.head_w.rows == 1),
            ("head_win.bias", net.head_b.len() == 1),
        ] {
            if !ok {
                return Err(NnError::BadTensor(name, "shape inconsistent with the rest of the net".into()));
            }
        }
        Ok(net)
    }

    /// Win probability the net predicts for this decklist.
    pub fn forward(&self, cards: &[u16], feats: &[f32; DECK_FEATS]) -> f32 {
        let d = self.emb.cols;
        let mut trunk_in = vec![0.0f32; self.trunk1_w.cols];
        let (sum, rest) = trunk_in[..3 * d].split_at_mut(d);
        let (mean, max) = rest.split_at_mut(d);
        max.fill(f32::NEG_INFINITY);
        for &card in cards {
            let c = (card as usize).min(self.emb.rows - 1);
            let row = &self.emb.data[c * d..(c + 1) * d];
            for ((s, r), mx) in sum.iter_mut().zip(row).zip(max.iter_mut()) {
                *s += r;
                *mx = mx.max(*r);
            }
        }
        if cards.is_empty() {
            max.fill(0.0);
        } else {
            let inv = 1.0 / cards.len() as f32;
            for (m, s) in mean.iter_mut().zip(sum.iter()) {
                *m = s * inv;
            }
        }
        // Sum pool scaled down so a 40-card sum sits in the same numeric
        // range as the mean/max pools.
        for s in sum.iter_mut() {
            *s *= 0.1;
        }
        trunk_in[3 * d..].copy_from_slice(feats);

        let mut t1 = vec![0.0f32; self.trunk1_w.rows];
        self.trunk1_w.matvec(&trunk_in, &mut t1);
        for (v, b) in t1.iter_mut().zip(&self.trunk1_b) {
            *v = (*v + b).max(0.0);
        }
        let mut t2 = vec![0.0f32; self.trunk2_w.rows];
        self.trunk2_w.matvec(&t1, &mut t2);
        for (v, b) in t2.iter_mut().zip(&self.trunk2_b) {
            *v = (*v + b).max(0.0);
        }
        let mut logit = [0.0f32];
        self.head_w.matvec(&t2, &mut logit);
        1.0 / (1.0 + (-(logit[0] + self.head_b[0])).exp())
    }
}

// ───────────────────────────── inference net ────────────────────────────

/// Row-major dense matrix (`data[r * cols + c]`).
#[derive(Debug, Clone)]
pub struct Tensor2 {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f32>,
}

impl Tensor2 {
    /// `out = self · x` (no accumulation into prior contents).
    fn matvec(&self, x: &[f32], out: &mut [f32]) {
        debug_assert_eq!(x.len(), self.cols);
        debug_assert_eq!(out.len(), self.rows);
        for (r, o) in out.iter_mut().enumerate() {
            let row = &self.data[r * self.cols..(r + 1) * self.cols];
            let mut acc = 0.0f32;
            for (w, v) in row.iter().zip(x) {
                acc += w * v;
            }
            *o = acc;
        }
    }
}

#[derive(Debug)]
pub enum NnError {
    /// The safetensors container failed to parse.
    BadFile(String),
    /// A required tensor is missing.
    Missing(&'static str),
    /// A tensor has the wrong dtype or an inconsistent shape.
    BadTensor(&'static str, String),
}

impl std::fmt::Display for NnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NnError::BadFile(e) => write!(f, "unreadable weights file: {e}"),
            NnError::Missing(name) => write!(f, "weights file lacks tensor {name}"),
            NnError::BadTensor(name, why) => write!(f, "tensor {name}: {why}"),
        }
    }
}

impl std::error::Error for NnError {}

/// Pull one f32 tensor out of a safetensors container as a [`Tensor2`]
/// (rank-1 tensors become a single row).
fn get_tensor(st: &safetensors::SafeTensors<'_>, name: &'static str) -> Result<Tensor2, NnError> {
    let view = st.tensor(name).map_err(|_| NnError::Missing(name))?;
    if view.dtype() != safetensors::Dtype::F32 {
        return Err(NnError::BadTensor(name, format!("dtype {:?}, want F32", view.dtype())));
    }
    let shape = view.shape();
    let (rows, cols) = match *shape {
        [r, c] => (r, c),
        [n] => (1, n),
        _ => return Err(NnError::BadTensor(name, format!("rank {} shape", shape.len()))),
    };
    let raw = view.data();
    if raw.len() != rows * cols * 4 {
        return Err(NnError::BadTensor(name, "data length != shape".into()));
    }
    let data =
        raw.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
    Ok(Tensor2 { rows, cols, data })
}

/// The value net, loaded from a trainer-exported safetensors file.
#[derive(Debug, Clone)]
pub struct PlayNet {
    emb: Tensor2,      // [vocab, emb_dim]
    obj_w: Tensor2,    // [obj_hidden, emb_dim + OBJ_FEATS]
    obj_b: Vec<f32>,   // [obj_hidden]
    trunk1_w: Tensor2, // [h1, NUM_GROUPS * 2*obj_hidden + GLOBAL_FEATS]
    trunk1_b: Vec<f32>,
    trunk2_w: Tensor2, // [h2, h1]
    trunk2_b: Vec<f32>,
    head_win_w: Tensor2, // [1, h2]
    head_win_b: Vec<f32>,
}

impl PlayNet {
    /// Vocabulary size the embedding table was trained with. The encoder's
    /// vocab must match or indices silently mean the wrong cards.
    pub fn vocab_size(&self) -> usize {
        self.emb.rows
    }

    /// Parse a safetensors byte buffer (see the module doc for the tensor
    /// naming contract). Extra tensors — the auxiliary training heads —
    /// are ignored.
    pub fn load(bytes: &[u8]) -> Result<PlayNet, NnError> {
        let st = safetensors::SafeTensors::deserialize(bytes)
            .map_err(|e| NnError::BadFile(e.to_string()))?;
        let get = |name: &'static str| get_tensor(&st, name);

        let net = PlayNet {
            emb: get("emb.weight")?,
            obj_w: get("obj.weight")?,
            obj_b: get("obj.bias")?.data,
            trunk1_w: get("trunk1.weight")?,
            trunk1_b: get("trunk1.bias")?.data,
            trunk2_w: get("trunk2.weight")?,
            trunk2_b: get("trunk2.bias")?.data,
            head_win_w: get("head_win.weight")?,
            head_win_b: get("head_win.bias")?.data,
        };

        // Cross-check the shapes once here so forward() can trust them.
        let h_obj = net.obj_w.rows;
        if net.obj_w.cols != net.emb.cols + OBJ_FEATS {
            return Err(NnError::BadTensor(
                "obj.weight",
                format!("cols {} != emb {} + {OBJ_FEATS}", net.obj_w.cols, net.emb.cols),
            ));
        }
        if net.trunk1_w.cols != NUM_GROUPS * 2 * h_obj + GLOBAL_FEATS {
            return Err(NnError::BadTensor(
                "trunk1.weight",
                format!("cols {} don't match {NUM_GROUPS} groups of 2×{h_obj} + {GLOBAL_FEATS}", net.trunk1_w.cols),
            ));
        }
        for (name, ok) in [
            ("obj.bias", net.obj_b.len() == h_obj),
            ("trunk1.bias", net.trunk1_b.len() == net.trunk1_w.rows),
            ("trunk2.weight", net.trunk2_w.cols == net.trunk1_w.rows),
            ("trunk2.bias", net.trunk2_b.len() == net.trunk2_w.rows),
            ("head_win.weight", net.head_win_w.cols == net.trunk2_w.rows && net.head_win_w.rows == 1),
            ("head_win.bias", net.head_win_b.len() == 1),
        ] {
            if !ok {
                return Err(NnError::BadTensor(name, "shape inconsistent with the rest of the net".into()));
            }
        }
        Ok(net)
    }

    /// Win probability for the seat the state was encoded for, in [0, 1].
    pub fn forward(&self, s: &EncodedState) -> f32 {
        let emb_dim = self.emb.cols;
        let h_obj = self.obj_w.rows;
        let mut trunk_in = vec![0.0f32; self.trunk1_w.cols];

        let mut x = vec![0.0f32; self.obj_w.cols];
        let mut h = vec![0.0f32; h_obj];
        for (gi, group) in s.groups.iter().enumerate() {
            if group.is_empty() {
                continue; // group slot stays zero — the net's "empty" signal
            }
            let base = gi * 2 * h_obj;
            let (mean, rest) = trunk_in[base..base + 2 * h_obj].split_at_mut(h_obj);
            let max = rest;
            max.fill(f32::NEG_INFINITY);
            for o in group {
                let card = (o.card as usize).min(self.emb.rows - 1);
                x[..emb_dim].copy_from_slice(&self.emb.data[card * emb_dim..(card + 1) * emb_dim]);
                x[emb_dim..].copy_from_slice(&o.feats);
                self.obj_w.matvec(&x, &mut h);
                for ((hv, b), (m, mx)) in
                    h.iter_mut().zip(&self.obj_b).zip(mean.iter_mut().zip(max.iter_mut()))
                {
                    let v = (*hv + b).max(0.0);
                    *m += v;
                    *mx = mx.max(v);
                }
            }
            let inv = 1.0 / group.len() as f32;
            for m in mean.iter_mut() {
                *m *= inv;
            }
        }
        let gbase = NUM_GROUPS * 2 * h_obj;
        trunk_in[gbase..].copy_from_slice(&s.global);

        let mut t1 = vec![0.0f32; self.trunk1_w.rows];
        self.trunk1_w.matvec(&trunk_in, &mut t1);
        for (v, b) in t1.iter_mut().zip(&self.trunk1_b) {
            *v = (*v + b).max(0.0);
        }
        let mut t2 = vec![0.0f32; self.trunk2_w.rows];
        self.trunk2_w.matvec(&t1, &mut t2);
        for (v, b) in t2.iter_mut().zip(&self.trunk2_b) {
            *v = (*v + b).max(0.0);
        }
        let mut logit = [0.0f32];
        self.head_win_w.matvec(&t2, &mut logit);
        let z = logit[0] + self.head_win_b[0];
        1.0 / (1.0 + (-z).exp())
    }
}

/// Build a safetensors byte buffer from named f32 tensors — the writer half
/// of [`PlayNet::load`], used by tests and by any tool that wants to emit
/// weights without pulling in an ML framework.
pub fn to_safetensors(tensors: &[(&str, Vec<usize>, Vec<f32>)]) -> Vec<u8> {
    let views: BTreeMap<String, safetensors::tensor::TensorView<'_>> = tensors
        .iter()
        .map(|(name, shape, data)| {
            let bytes: &[u8] = bytemuck_cast(data);
            (
                name.to_string(),
                safetensors::tensor::TensorView::new(safetensors::Dtype::F32, shape.clone(), bytes)
                    .expect("consistent shape/data"),
            )
        })
        .collect();
    safetensors::serialize(&views, None).expect("serializable tensors")
}

/// f32 slice → little-endian bytes without a bytemuck dependency. Only
/// correct on little-endian targets, which is everything this project runs
/// on (x86-64, aarch64, wasm32); a big-endian build fails to compile rather
/// than silently writing byte-swapped weights.
#[cfg(not(target_endian = "little"))]
compile_error!("crabomination_nn's weight serialisation assumes a little-endian target");

fn bytemuck_cast(data: &[f32]) -> &[u8] {
    // SAFETY: f32 has no invalid bit patterns as bytes, alignment of u8 is 1,
    // and the length is the element count times size_of::<f32>.
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_row() -> TrainRow {
        let mut state = EncodedState::default();
        state.global[0] = 0.5;
        state.global[GLOBAL_FEATS - 1] = -1.25;
        let mut feats = [0.0; OBJ_FEATS];
        feats[0] = 0.375;
        feats[OBJ_FEATS - 1] = 1.0;
        state.groups[G_BF_SELF].push(EncodedObject { card: 7, feats });
        state.groups[G_GY_OPP].push(EncodedObject { card: 0, feats: [0.25; OBJ_FEATS] });
        TrainRow { state, win: 1.0, life_diff: 0.55, game_len: 0.2, traj: 42, ply: 3 }
    }

    #[test]
    fn shard_roundtrip_is_exact() {
        let rows =
            vec![tiny_row(), TrainRow { win: 0.0, traj: 42, ply: 4, ..tiny_row() }];
        let bytes = write_shard(&rows);
        assert_eq!(read_shard(&bytes).expect("decodes"), rows);
        // A truncated or version-bumped shard is rejected whole.
        assert!(read_shard(&bytes[..bytes.len() - 1]).is_none());
        let mut stale = bytes.clone();
        stale[4] ^= 0xFF;
        assert!(read_shard(&stale).is_none());
    }

    /// Forward pass against hand-computed values on a minimal net: vocab 2,
    /// embedding 1, one object-hidden unit, trunk 1×1. Every weight is
    /// chosen so the arithmetic is checkable on paper.
    #[test]
    fn forward_matches_hand_computation() {
        let h_obj = 1usize;
        let trunk_in = NUM_GROUPS * 2 * h_obj + GLOBAL_FEATS;
        // obj input = [emb(1), feats(OBJ_FEATS)]; weight 1 on the embedding,
        // 1 on feats[0], 0 elsewhere. Bias 0.
        let mut obj_w = vec![0.0f32; 1 + OBJ_FEATS];
        obj_w[0] = 1.0;
        obj_w[1] = 1.0;
        // trunk1: single unit summing the whole input; trunk2 identity.
        let bytes = to_safetensors(&[
            ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
            ("obj.weight", vec![1, 1 + OBJ_FEATS], obj_w),
            ("obj.bias", vec![1], vec![0.0]),
            ("trunk1.weight", vec![1, trunk_in], vec![1.0; trunk_in]),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![1.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            ("head_win.weight", vec![1, 1], vec![1.0]),
            ("head_win.bias", vec![1], vec![0.0]),
        ]);
        let net = PlayNet::load(&bytes).expect("loads");
        assert_eq!(net.vocab_size(), 2);

        let mut s = EncodedState::default();
        // One object, card 1 (embedding 2.0), feats[0] = 0.5:
        // obj unit = relu(2.0 + 0.5) = 2.5 → group mean 2.5, max 2.5.
        let mut feats = [0.0; OBJ_FEATS];
        feats[0] = 0.5;
        s.groups[G_BF_SELF].push(EncodedObject { card: 1, feats });
        // Global contributes 0.25; trunk sums to 2.5 + 2.5 + 0.25 = 5.25.
        s.global[3] = 0.25;
        let want = 1.0 / (1.0 + (-5.25f32).exp());
        let got = net.forward(&s);
        assert!((got - want).abs() < 1e-6, "got {got}, want {want}");

        // Two objects in a group: mean and max diverge.
        // Second object: card 0 (embedding 0.0), feats[0] = 1.5 → unit 1.5.
        // mean = (2.5 + 1.5)/2 = 2.0, max = 2.5 → trunk 2.0+2.5+0.25 = 4.75.
        let mut feats2 = [0.0; OBJ_FEATS];
        feats2[0] = 1.5;
        s.groups[G_BF_SELF].push(EncodedObject { card: 0, feats: feats2 });
        let want2 = 1.0 / (1.0 + (-4.75f32).exp());
        let got2 = net.forward(&s);
        assert!((got2 - want2).abs() < 1e-6, "got {got2}, want {want2}");
    }

    /// DeckNet forward against hand arithmetic: embedding 1, both trunk
    /// layers 1×1 identity-ish. Cards [1, 1, 0] with emb [0.0, 2.0]:
    /// sum = 4.0 (scaled ×0.1 → 0.4), mean = 4/3, max = 2.0; feats[0] =
    /// 0.5 → trunk input sums to 0.4 + 4/3 + 2.0 + 0.5.
    #[test]
    fn deck_forward_matches_hand_computation() {
        let trunk_in = 3 + DECK_FEATS;
        let bytes = to_safetensors(&[
            ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
            ("trunk1.weight", vec![1, trunk_in], vec![1.0; trunk_in]),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![1.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            ("head_win.weight", vec![1, 1], vec![1.0]),
            ("head_win.bias", vec![1], vec![0.0]),
        ]);
        let net = DeckNet::load(&bytes).expect("loads");
        let mut feats = [0.0f32; DECK_FEATS];
        feats[0] = 0.5;
        let z = 0.4f32 + 4.0 / 3.0 + 2.0 + 0.5;
        let want = 1.0 / (1.0 + (-z).exp());
        let got = net.forward(&[1, 1, 0], &feats);
        assert!((got - want).abs() < 1e-6, "got {got}, want {want}");
    }

    #[test]
    fn load_rejects_inconsistent_shapes() {
        let bytes = to_safetensors(&[
            ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
            // obj.weight cols disagree with emb + OBJ_FEATS.
            ("obj.weight", vec![1, 3], vec![0.0; 3]),
            ("obj.bias", vec![1], vec![0.0]),
            ("trunk1.weight", vec![1, 1], vec![1.0]),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![1.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            ("head_win.weight", vec![1, 1], vec![1.0]),
            ("head_win.bias", vec![1], vec![0.0]),
        ]);
        assert!(PlayNet::load(&bytes).is_err());
    }
}
