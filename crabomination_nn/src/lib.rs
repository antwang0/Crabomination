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
pub const NUM_GROUPS: usize = 8;
/// Group indices into [`EncodedState::groups`].
pub const G_BF_SELF: usize = 0;
pub const G_BF_OPP: usize = 1;
pub const G_HAND_SELF: usize = 2;
pub const G_GY_SELF: usize = 3;
pub const G_GY_OPP: usize = 4;
/// The encoded seat's own library, as an unordered multiset of the cards
/// still in it — one object per distinct name, carrying its remaining
/// count in feature 27.
///
/// This is information the seat genuinely has (it is their own decklist)
/// and the net was previously denied: before this group the library was a
/// single scalar, `library.len() / 40`, so "22 cards left, three of them
/// removal and one a bomb" and "22 lands" encoded identically. It is also
/// the one zone where the bag-of-cards prior is unambiguously right — a
/// library really is an unordered set, which is exactly the argument that
/// makes the *deck* net work where the pooled play net did not.
///
/// Order never reaches the net: entries are emitted sorted by vocabulary
/// index, and both pooling and attention are permutation-invariant over a
/// group. The encoder must not leak the shuffle.
pub const G_LIB_SELF: usize = 5;
/// The stack, split by controller like the battlefield. One object per
/// stack item — the spell's card, or a trigger's source card — with its
/// depth from the top of the stack in feature 36. Before these groups the
/// stack was a single count (`global[18]`): "there is a spell on the
/// stack" was representable, "it is their removal spell aimed at my best
/// creature" was not, and the latter is the shape of every
/// instant-speed decision. Shallow stacks make the pooled bag mostly a
/// one-object group, which is exactly when pooling is lossless.
pub const G_STACK_SELF: usize = 6;
pub const G_STACK_OPP: usize = 7;

/// Per-object feature count. Baked into encoded rows — bump
/// [`SHARD_VERSION`] if it changes. Feats 12..=19 are the evasion/combat
/// keyword flags added in round 4 (flying, reach, menace, deathtouch,
/// lifelink, trample, first-or-double strike, vigilance): the pooled
/// encoder previously saw a Serra Angel and a Hill Giant as the same
/// 4-mana 4/4-ish body. Feats 20..=26 are the castability block (colour
/// pips, castable now, castable next turn) and 27 is library
/// multiplicity.
///
/// Feats 28..=36 are the round-12 relation block. Pooling can never
/// represent an *edge* between two objects, but a flag summarising the
/// edge from one endpoint's side survives it — and the attention layer
/// can match flagged endpoints across groups:
///
/// * 28 `is_blocking` / 29 `is_blocked` — combat edges from `block_map`.
/// * 30 `is_attached` — this object is an aura/equipment on a host.
/// * 31 `has_own_attachment` / 32 `has_opp_attachment` — the host's
///   side, split by who controls the attachment: an own aura is a buff,
///   an opposing aura is a Pacifism, and before this the two encoded as
///   the same creature.
/// * 33 `targeted_by_stack` — something on the stack aims at this.
/// * 34 counters other than loyalty/prepared, count / 4 (P/T counters
///   already reach the net through effective P/T; this carries the rest
///   — Glyph, Currency, and whatever lands later).
/// * 35 `is_attachment_type` — aura or equipment by printed type.
/// * 36 stack depth from the top, / 4 (stack groups only).
pub const OBJ_FEATS: usize = 37;
/// Global scalar feature count. Baked into encoded rows likewise.
pub const GLOBAL_FEATS: usize = 36;

/// Standard trainer configuration (the file's shapes win at load time).
/// Sizes quadrupled in round 4: four gate rounds measured the small net
/// flat at 42–45 % as a replacement judge across data-volume and
/// distribution fixes, leaving capacity the first untested lever. (It was
/// not the answer — see [`ATTN_HEADS`].)
///
/// Parameter count is a function of these constants *and* the vocabulary,
/// so it is deliberately not quoted here; a stale figure was carried in
/// this doc for several rounds. The trunk dominates: its input is
/// `NUM_GROUPS · 2 · OBJ_HIDDEN + GLOBAL_FEATS`.
pub const EMB_DIM: usize = 32;
pub const OBJ_HIDDEN: usize = 64;
pub const TRUNK_H1: usize = 512;
pub const TRUNK_H2: usize = 256;

/// Attention heads in the optional interaction layer. Must divide
/// [`OBJ_HIDDEN`].
///
/// Why the layer exists: mean/max pooling is permutation invariant *per
/// group*, so the trunk only ever sees "how much is in this zone" and
/// "what is the biggest thing in it". It cannot represent a relation
/// between two objects in different zones — "my flier gets through
/// because their board has no flier and no reach" needs my battlefield
/// compared element-wise against theirs, and pooling has already
/// discarded that by the time the trunk runs.
///
/// The calibration diagnostic made this concrete rather than theoretical:
/// the pooled net scores AUC 0.746 at predicting the winner against
/// `eval_material`'s 0.755 on identical positions. It learned the
/// material heuristic and stopped, which is the ceiling of what the
/// representation can express — and why quadrupling the trunk in round 4
/// could not have helped. Widening a layer cannot recover information
/// discarded before it.
pub const ATTN_HEADS: usize = 4;

/// One card object as the net sees it: a vocabulary index plus a small
/// dense feature vector (effective P/T, tapped, counters, ...). Index 0 is
/// reserved for unknown names — tokens and anything outside the vocab.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedObject {
    pub card: u16,
    pub feats: [f32; OBJ_FEATS],
}

/// A full observable position from one seat's perspective.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedState {
    pub global: [f32; GLOBAL_FEATS],
    pub groups: [Vec<EncodedObject>; NUM_GROUPS],
}

// Hand-written rather than derived: `Default` for arrays stops at length
// 32, and both feature counts have outgrown that.
impl Default for EncodedState {
    fn default() -> Self {
        Self { global: [0.0; GLOBAL_FEATS], groups: std::array::from_fn(|_| Vec::new()) }
    }
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
/// Auxiliary short-horizon targets, labelled from the recorded
/// trajectory: `[Δlife-diff, Δpower-diff, Δcreature-diff, opp hand
/// next]`, each measured to the seat's *next snapshot* and scaled. Dense
/// and near-term where `win` is sparse and twenty turns away — the same
/// credit-assignment argument as the life/length heads, one hop out
/// instead of at the horizon. Terminal rows carry zero deltas.
pub const AUX_FEATS: usize = 4;

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
    /// See [`AUX_FEATS`]. All-zero when the recorder predates them.
    pub aux: [f32; AUX_FEATS],
}

// ───────────────────────────── shard format ─────────────────────────────

/// Magic + version guard the on-disk row format. Bump the version whenever
/// `OBJ_FEATS`, `GLOBAL_FEATS`, group order, or the row layout changes —
/// stale shards must fail loudly, not decode as garbage.
pub const SHARD_MAGIC: [u8; 4] = *b"CRML";
pub const SHARD_VERSION: u32 = 5;

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
        for v in row.aux {
            out.extend_from_slice(&v.to_le_bytes());
        }
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
        let mut aux = [0.0; AUX_FEATS];
        for a in aux.iter_mut() {
            *a = r.f32()?;
        }
        rows.push(TrainRow { state, win, life_diff, game_len, traj, ply, aux });
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

/// Deck-label shard: the distillation training set — decklists labelled
/// with a *gauntlet win rate* rather than a single game's outcome. One
/// 300-game win rate carries the information of hundreds of Bernoulli
/// labels, and the file accumulates across generation passes (read,
/// extend, rewrite — it is tiny).
pub const DECK_SHARD_MAGIC: [u8; 4] = *b"CRDL";
pub const DECK_SHARD_VERSION: u32 = 1;

pub fn write_deck_shard(rows: &[DeckRow]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + rows.len() * 128);
    out.extend_from_slice(&DECK_SHARD_MAGIC);
    out.extend_from_slice(&DECK_SHARD_VERSION.to_le_bytes());
    out.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    for r in rows {
        out.extend_from_slice(&(r.cards.len() as u16).to_le_bytes());
        for c in &r.cards {
            out.extend_from_slice(&c.to_le_bytes());
        }
        for v in r.feats {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&r.win.to_le_bytes());
    }
    out
}

/// `None` on any mismatch — a stale or torn label file is rejected whole.
pub fn read_deck_shard(bytes: &[u8]) -> Option<Vec<DeckRow>> {
    let mut r = Reader { b: bytes, pos: 0 };
    if r.take(4)? != DECK_SHARD_MAGIC || r.u32()? != DECK_SHARD_VERSION {
        return None;
    }
    let n = r.u32()? as usize;
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let k = r.u16()? as usize;
        let mut cards = Vec::with_capacity(k);
        for _ in 0..k {
            cards.push(r.u16()?);
        }
        let mut feats = [0.0; DECK_FEATS];
        for f in feats.iter_mut() {
            *f = r.f32()?;
        }
        let win = r.f32()?;
        rows.push(DeckRow { cards, feats, win });
    }
    (r.pos == bytes.len()).then_some(rows)
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
    /// Present only in weights trained with the interaction layer. Absent
    /// means the pure deep-sets net, which stays loadable unchanged — the
    /// two architectures are distinguished by which tensors the file
    /// carries, not by a flag the caller has to remember.
    attn: Option<Attn>,
}

/// Single pre-pool self-attention layer over every object on the board,
/// with a residual. See [`ATTN_HEADS`].
#[derive(Debug, Clone)]
struct Attn {
    /// Learned per-group tag, `[NUM_GROUPS, obj_hidden]`.
    group: Tensor2,
    q_w: Tensor2,
    q_b: Vec<f32>,
    k_w: Tensor2,
    k_b: Vec<f32>,
    v_w: Tensor2,
    v_b: Vec<f32>,
    o_w: Tensor2,
    o_b: Vec<f32>,
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
            attn: None,
        };

        // The interaction layer is all-or-nothing. A file carrying some of
        // its tensors is a trainer/inference mismatch, and silently
        // ignoring the partial set would run the wrong architecture on
        // weights trained for another — the exact class of bug the tensor
        // naming contract exists to prevent.
        const ATTN_NAMES: [&str; 9] = [
            "attn.group.weight",
            "attn.q.weight",
            "attn.q.bias",
            "attn.k.weight",
            "attn.k.bias",
            "attn.v.weight",
            "attn.v.bias",
            "attn.o.weight",
            "attn.o.bias",
        ];
        let present = ATTN_NAMES.iter().filter(|n| st.tensor(n).is_ok()).count();
        let net = if present == ATTN_NAMES.len() {
            PlayNet {
                attn: Some(Attn {
                    group: get("attn.group.weight")?,
                    q_w: get("attn.q.weight")?,
                    q_b: get("attn.q.bias")?.data,
                    k_w: get("attn.k.weight")?,
                    k_b: get("attn.k.bias")?.data,
                    v_w: get("attn.v.weight")?,
                    v_b: get("attn.v.bias")?.data,
                    o_w: get("attn.o.weight")?,
                    o_b: get("attn.o.bias")?.data,
                }),
                ..net
            }
        } else if present == 0 {
            net
        } else {
            return Err(NnError::BadTensor(
                "attn.*",
                format!("{present} of {} attention tensors present", ATTN_NAMES.len()),
            ));
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
        if let Some(a) = &net.attn {
            if h_obj % ATTN_HEADS != 0 {
                return Err(NnError::BadTensor(
                    "attn.q.weight",
                    format!("obj_hidden {h_obj} not divisible by {ATTN_HEADS} heads"),
                ));
            }
            for (name, t) in [
                ("attn.q.weight", &a.q_w),
                ("attn.k.weight", &a.k_w),
                ("attn.v.weight", &a.v_w),
                ("attn.o.weight", &a.o_w),
            ] {
                if t.rows != h_obj || t.cols != h_obj {
                    return Err(NnError::BadTensor(
                        name,
                        format!("{}x{} != {h_obj}x{h_obj}", t.rows, t.cols),
                    ));
                }
            }
            if a.group.rows != NUM_GROUPS || a.group.cols != h_obj {
                return Err(NnError::BadTensor(
                    "attn.group.weight",
                    format!("{}x{} != {NUM_GROUPS}x{h_obj}", a.group.rows, a.group.cols),
                ));
            }
        }
        Ok(net)
    }

    /// Per-object hidden vectors for every object on the board, flattened
    /// across groups: `(hs [n, h_obj], group index per object)`.
    fn encode_objects(&self, s: &EncodedState, h_obj: usize) -> (Vec<f32>, Vec<usize>) {
        let emb_dim = self.emb.cols;
        let n: usize = s.groups.iter().map(|g| g.len()).sum();
        let mut hs = vec![0.0f32; n * h_obj];
        let mut owner = Vec::with_capacity(n);
        let mut x = vec![0.0f32; self.obj_w.cols];
        let mut i = 0;
        for (gi, group) in s.groups.iter().enumerate() {
            for o in group {
                let card = (o.card as usize).min(self.emb.rows - 1);
                x[..emb_dim].copy_from_slice(&self.emb.data[card * emb_dim..(card + 1) * emb_dim]);
                x[emb_dim..].copy_from_slice(&o.feats);
                self.obj_w.matvec(&x, &mut hs[i * h_obj..(i + 1) * h_obj]);
                for (v, b) in hs[i * h_obj..(i + 1) * h_obj].iter_mut().zip(&self.obj_b) {
                    *v = (*v + b).max(0.0);
                }
                owner.push(gi);
                i += 1;
            }
        }
        (hs, owner)
    }

    /// Multi-head self-attention over all objects, in place, with a
    /// residual and a relu.
    ///
    /// Every object attends to every other regardless of zone, which is
    /// the entire point: this is the first place in the network where a
    /// creature on my battlefield can be compared against a creature on
    /// theirs. A learned per-group tag is added to the input so a query
    /// can tell whose object it is looking at.
    ///
    /// No layer norm and a single layer, deliberately. This is an
    /// experiment testing whether interaction modelling buys anything at
    /// all; a full pre-norm transformer is a lot more parity surface
    /// between this hand-rolled forward and the candle mirror, and is
    /// worth paying for only once there is a signal to justify it.
    fn attend(&self, hs: &mut [f32], owner: &[usize], h_obj: usize) {
        let Some(a) = &self.attn else { return };
        let n = owner.len();
        if n == 0 {
            return;
        }
        let hd = h_obj / ATTN_HEADS;
        let scale = 1.0 / (hd as f32).sqrt();

        // Tagged input: h + group_embedding(group_of_object).
        let mut xin = vec![0.0f32; n * h_obj];
        for i in 0..n {
            let tag = &a.group.data[owner[i] * h_obj..(owner[i] + 1) * h_obj];
            for j in 0..h_obj {
                xin[i * h_obj + j] = hs[i * h_obj + j] + tag[j];
            }
        }

        let project = |w: &Tensor2, b: &[f32]| {
            let mut out = vec![0.0f32; n * h_obj];
            for i in 0..n {
                w.matvec(&xin[i * h_obj..(i + 1) * h_obj], &mut out[i * h_obj..(i + 1) * h_obj]);
                for (v, bb) in out[i * h_obj..(i + 1) * h_obj].iter_mut().zip(b) {
                    *v += bb;
                }
            }
            out
        };
        let q = project(&a.q_w, &a.q_b);
        let k = project(&a.k_w, &a.k_b);
        let v = project(&a.v_w, &a.v_b);

        let mut ctx = vec![0.0f32; n * h_obj];
        let mut scores = vec![0.0f32; n];
        for head in 0..ATTN_HEADS {
            let off = head * hd;
            for i in 0..n {
                let qi = &q[i * h_obj + off..i * h_obj + off + hd];
                let mut max = f32::NEG_INFINITY;
                for (j, sc) in scores.iter_mut().enumerate() {
                    let kj = &k[j * h_obj + off..j * h_obj + off + hd];
                    let dot: f32 = qi.iter().zip(kj).map(|(x, y)| x * y).sum();
                    *sc = dot * scale;
                    max = max.max(*sc);
                }
                // Softmax, max-subtracted so a wide score range can't
                // overflow on a big board.
                let mut sum = 0.0f32;
                for sc in scores.iter_mut() {
                    *sc = (*sc - max).exp();
                    sum += *sc;
                }
                let inv = 1.0 / sum;
                for (j, sc) in scores.iter().enumerate() {
                    let w = sc * inv;
                    let vj = &v[j * h_obj + off..j * h_obj + off + hd];
                    let dst = &mut ctx[i * h_obj + off..i * h_obj + off + hd];
                    for (d, sv) in dst.iter_mut().zip(vj) {
                        *d += w * sv;
                    }
                }
            }
        }

        // Output projection, residual against the pre-attention hidden
        // state, relu.
        let mut proj = vec![0.0f32; h_obj];
        for i in 0..n {
            a.o_w.matvec(&ctx[i * h_obj..(i + 1) * h_obj], &mut proj);
            for j in 0..h_obj {
                hs[i * h_obj + j] = (hs[i * h_obj + j] + proj[j] + a.o_b[j]).max(0.0);
            }
        }
    }

    /// Win probability for the seat the state was encoded for, in [0, 1].
    pub fn forward(&self, s: &EncodedState) -> f32 {
        let h_obj = self.obj_w.rows;
        let mut trunk_in = vec![0.0f32; self.trunk1_w.cols];

        // Encode every object, let them interact (no-op without the
        // attention tensors), then pool per group exactly as before —
        // pooling still supplies the fixed-width trunk input, it just
        // pools representations that have already seen the whole board.
        let (mut hs, owner) = self.encode_objects(s, h_obj);
        self.attend(&mut hs, &owner, h_obj);

        let mut counts = [0usize; NUM_GROUPS];
        for gi in 0..NUM_GROUPS {
            let base = gi * 2 * h_obj;
            trunk_in[base + h_obj..base + 2 * h_obj].fill(f32::NEG_INFINITY);
        }
        for (i, &gi) in owner.iter().enumerate() {
            let base = gi * 2 * h_obj;
            counts[gi] += 1;
            for j in 0..h_obj {
                let v = hs[i * h_obj + j];
                trunk_in[base + j] += v;
                let mx = &mut trunk_in[base + h_obj + j];
                *mx = mx.max(v);
            }
        }
        for (gi, &n) in counts.iter().enumerate() {
            let base = gi * 2 * h_obj;
            if n == 0 {
                // Empty group stays all-zero — the net's "nothing here"
                // signal, and what the -inf max slots must collapse to.
                trunk_in[base..base + 2 * h_obj].fill(0.0);
                continue;
            }
            let inv = 1.0 / n as f32;
            for m in trunk_in[base..base + h_obj].iter_mut() {
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
        TrainRow {
            state,
            win: 1.0,
            life_diff: 0.55,
            game_len: 0.2,
            traj: 42,
            ply: 3,
            aux: [0.1, -0.2, 0.3, 0.6],
        }
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

    #[test]
    fn deck_shard_roundtrip_is_exact() {
        let rows = vec![
            DeckRow { cards: vec![1, 5, 5, 9], feats: [0.25; DECK_FEATS], win: 0.62 },
            DeckRow { cards: vec![], feats: [0.0; DECK_FEATS], win: 0.0 },
        ];
        let bytes = write_deck_shard(&rows);
        assert_eq!(read_deck_shard(&bytes).expect("decodes"), rows);
        assert!(read_deck_shard(&bytes[..bytes.len() - 1]).is_none());
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
