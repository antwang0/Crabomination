//! The **table net**: one value net for every seat count (ML_NOTES "One net
//! for every mode", step 1).
//!
//! [`PlayNet`](crate::PlayNet) is two-seat by type — eight zone groups, each
//! either "self" or "opp", and a scalar win head. This net reads a table
//! instead:
//!
//! * **Slots.** Slot 0 is the seat the state was encoded for; slots 1.. are
//!   the other living seats in turn order (slot 1 takes the next turn). A
//!   duel is one opponent slot, a four-seat pod three. Every slot has a row
//!   of [`SEAT_FEATS`] and its objects grouped by [`TABLE_ZONES`] zones;
//!   only slot 0 ever has hand or library objects — the encoder never sees
//!   another seat's hidden cards.
//! * **Objects.** `relu(W_obj · [name ⊕ description ⊕ feats])`, one weight
//!   matrix for every slot and zone. The *name* is a learned row per card
//!   name (the frozen vocabulary, index 0 unknown). The *description* is the
//!   mean of learned rows for the card's descriptor tokens — hashed buckets
//!   the engine derives from the card definition — so a card the name table
//!   never saw still says what it does. Descriptors are stored once per
//!   distinct card in a state ([`TableState::add_desc`]) and objects point at
//!   them.
//! * **Interaction** (optional, `tblocks.*`): the same pre-LN transformer
//!   blocks as the play net, over every object at the table, with a zone tag
//!   and a slot tag added into the stream at entry so attention can tell
//!   whose object it is looking at.
//! * **Seats.** Each (slot, zone) group is mean- and max-pooled; a slot's
//!   pools plus its seat features go through one shared seat layer, plus a
//!   learned per-slot row (turn-order distance, capped at [`SLOT_TAGS`]).
//! * **Trunk.** `[me ⊕ mean(opponents) ⊕ max(opponents) ⊕ globals]` through
//!   two relu layers. Any number of opponents pools to the same width.
//! * **Value.** One logit per slot, `value2 · relu(value1 · [trunk ⊕ seat])`,
//!   softmaxed over the living slots: each seat's chance to win. At two
//!   slots that is `sigmoid(l0 − l1)` — the duel is the N = 2 case, not a
//!   separate head.
//!
//! Tensor names are disjoint from the play net's where the shapes differ
//! (`name_emb`, not `emb`), so neither loader accepts the other's file. The
//! trainer mirrors this module in `crabomination_ml::table`; its parity tests
//! hold the two to the same numbers.

use super::{
    NnError, Reader, TBlock, Tensor2, count_tblocks, get_tensor, load_tblocks, run_tblocks,
};

/// Zones per slot, in group order. Group `g = slot * TABLE_ZONES + zone`.
pub const TABLE_ZONES: usize = 6;
pub const Z_BF: usize = 0;
pub const Z_GY: usize = 1;
/// Stack items the slot's seat controls, depth from the top in feature 36.
pub const Z_STACK: usize = 2;
/// The command zone (CR 408): public, so every slot has one.
pub const Z_COMMAND: usize = 3;
/// Slot 0 only.
pub const Z_HAND: usize = 4;
/// Slot 0 only: the seat's own library as a multiset, one object per name,
/// remaining count in feature 27 (as the play net's library group).
pub const Z_LIB: usize = 5;

/// Per-object features: the play net's [`OBJ_FEATS`](crate::OBJ_FEATS)
/// block, same indices and scales, then four table-only ones:
///
/// * 59 is a commander (CR 903.3).
/// * 60 commander tax while in the command zone: `{2}` per earlier cast
///   (CR 903.8), / 6.
/// * 61 attacking the encoded seat, or a planeswalker it controls. Feature 10
///   says "attacking"; in a pod *whom* is the whole question.
/// * 62 goaded (CR 701.15): must attack, and not its goader.
pub const TABLE_OBJ_FEATS: usize = crate::OBJ_FEATS + 4;

/// Per-seat features. Indices are the encoder's (`server::encode_table`);
/// the count is what the weights are shaped by.
pub const SEAT_FEATS: usize = 38;

/// Game-wide features (turn, step, stack, seats alive, format bits).
pub const TABLE_GLOBAL_FEATS: usize = 16;

/// Distinct slot tags. Slots past the last share its tag — a nine-seat pod
/// still encodes, its farthest seats just read as "far".
pub const SLOT_TAGS: usize = 8;

/// Descriptor token buckets: the rows of `tok_emb`. The engine hashes each
/// token into `1..DESC_BUCKETS`; 0 is never emitted.
pub const DESC_BUCKETS: usize = 16384;

/// [`TableObject::desc`] for an object with no descriptor (a face-down
/// card, or an unknown stack source).
pub const NO_DESC: u16 = u16::MAX;

/// Standard trainer sizes (a file's shapes win at load time).
pub const SEAT_HIDDEN: usize = 128;
pub const VALUE_HIDDEN: usize = 64;

/// The tag row slot `slot` reads.
#[inline]
pub fn slot_tag(slot: usize) -> usize {
    slot.min(SLOT_TAGS - 1)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableObject {
    /// Name-embedding index; 0 is unknown.
    pub card: u16,
    /// Index into the state's descriptor table, or [`NO_DESC`].
    pub desc: u16,
    pub feats: [f32; TABLE_OBJ_FEATS],
}

impl Default for TableObject {
    fn default() -> Self {
        Self { card: 0, desc: NO_DESC, feats: [0.0; TABLE_OBJ_FEATS] }
    }
}

/// A whole table from one seat's point of view. Seats first
/// ([`TableState::with_seats`]), then objects in group order — slot-major,
/// zone-minor — in one buffer, like [`EncodedState`](crate::EncodedState).
#[derive(Debug, Clone, PartialEq)]
pub struct TableState {
    pub global: [f32; TABLE_GLOBAL_FEATS],
    seats: Vec<[f32; SEAT_FEATS]>,
    objs: Vec<TableObject>,
    /// Objects per group, `seats.len() * TABLE_ZONES` entries.
    counts: Vec<u32>,
    /// Every descriptor's tokens, back to back.
    desc_tokens: Vec<u16>,
    /// End offset of each descriptor in `desc_tokens`.
    desc_ends: Vec<u32>,
}

impl Default for TableState {
    fn default() -> Self {
        Self::with_seats(0)
    }
}

impl TableState {
    /// An empty table with `n` zeroed seat rows.
    pub fn with_seats(n: usize) -> Self {
        Self {
            global: [0.0; TABLE_GLOBAL_FEATS],
            seats: vec![[0.0; SEAT_FEATS]; n],
            objs: Vec::new(),
            counts: vec![0; n * TABLE_ZONES],
            desc_tokens: Vec::new(),
            desc_ends: Vec::new(),
        }
    }

    pub fn seat_count(&self) -> usize {
        self.seats.len()
    }

    pub fn seat(&self, slot: usize) -> &[f32; SEAT_FEATS] {
        &self.seats[slot]
    }

    pub fn seat_mut(&mut self, slot: usize) -> &mut [f32; SEAT_FEATS] {
        &mut self.seats[slot]
    }

    pub fn reserve(&mut self, n: usize) {
        self.objs.reserve(n);
    }

    /// Every object, in group order.
    pub fn objects(&self) -> &[TableObject] {
        &self.objs
    }

    pub fn len(&self) -> usize {
        self.objs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objs.is_empty()
    }

    pub fn group_len(&self, slot: usize, zone: usize) -> usize {
        self.counts[slot * TABLE_ZONES + zone] as usize
    }

    pub fn group(&self, slot: usize, zone: usize) -> &[TableObject] {
        let g = slot * TABLE_ZONES + zone;
        let start: usize = self.counts[..g].iter().map(|&c| c as usize).sum();
        &self.objs[start..start + self.counts[g] as usize]
    }

    /// `(slot, zone, objects)` for every group, walking the buffer once.
    pub fn groups(&self) -> impl Iterator<Item = (usize, usize, &[TableObject])> {
        let mut start = 0usize;
        self.counts.iter().enumerate().map(move |(g, &c)| {
            let s = start;
            start += c as usize;
            (g / TABLE_ZONES, g % TABLE_ZONES, &self.objs[s..start])
        })
    }

    /// Append to group (`slot`, `zone`). Groups fill in order.
    pub fn push(&mut self, slot: usize, zone: usize, o: TableObject) {
        let g = slot * TABLE_ZONES + zone;
        debug_assert!(
            self.counts[g + 1..].iter().all(|&c| c == 0),
            "TableState groups must be filled in order (pushing slot {slot} zone {zone})"
        );
        debug_assert!(
            o.desc == NO_DESC || (o.desc as usize) < self.desc_ends.len(),
            "object names descriptor {} of {}",
            o.desc,
            self.desc_ends.len()
        );
        self.objs.push(o);
        self.counts[g] += 1;
    }

    /// Append a zeroed object (no descriptor) and hand back a handle to
    /// fill in place.
    pub fn push_default(&mut self, slot: usize, zone: usize) -> &mut TableObject {
        self.push(slot, zone, TableObject::default());
        self.objs.last_mut().expect("just pushed")
    }

    /// Register a descriptor and return its index. The caller deduplicates
    /// (one descriptor per distinct card is the point of the table).
    pub fn add_desc(&mut self, tokens: &[u16]) -> u16 {
        let i = self.desc_ends.len();
        assert!(i < NO_DESC as usize, "more than {} descriptors in one state", NO_DESC);
        self.desc_tokens.extend_from_slice(tokens);
        self.desc_ends.push(self.desc_tokens.len() as u32);
        i as u16
    }

    pub fn desc_count(&self) -> usize {
        self.desc_ends.len()
    }

    pub fn desc(&self, d: usize) -> &[u16] {
        let start = if d == 0 { 0 } else { self.desc_ends[d - 1] as usize };
        &self.desc_tokens[start..self.desc_ends[d] as usize]
    }
}

/// A labelled table position. Per-slot vectors are in the state's slot
/// order.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub state: TableState,
    /// Each slot's share of the result: one-hot on the winner, split evenly
    /// across the seats a draw leaves standing. The value head's
    /// cross-entropy target.
    pub win: Vec<f32>,
    /// Each slot's final life over its starting life, clamped by the
    /// recorder — a training-only auxiliary target (the KataGo "why", per
    /// seat).
    pub life: Vec<f32>,
    /// Turns the game still ran from this snapshot, scaled — auxiliary.
    pub game_len: f32,
    /// One trajectory per (game, encoded seat), as [`crate::TrainRow::traj`].
    pub traj: u32,
    pub ply: u16,
}

// ───────────────────────────── shard format ─────────────────────────────

/// Table shards carry their own magic: a play-net shard must never decode
/// as one of these, or the reverse.
pub const TABLE_SHARD_MAGIC: [u8; 4] = *b"CRTB";
pub const TABLE_SHARD_VERSION: u32 = 1;

pub fn write_table_shard(rows: &[TableRow]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + rows.len() * 1024);
    out.extend_from_slice(&TABLE_SHARD_MAGIC);
    out.extend_from_slice(&TABLE_SHARD_VERSION.to_le_bytes());
    out.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    let f32s = |out: &mut Vec<u8>, xs: &[f32]| {
        for v in xs {
            out.extend_from_slice(&v.to_le_bytes());
        }
    };
    for row in rows {
        let s = &row.state;
        debug_assert_eq!(row.win.len(), s.seat_count());
        debug_assert_eq!(row.life.len(), s.seat_count());
        f32s(&mut out, &s.global);
        out.extend_from_slice(&(s.seat_count() as u16).to_le_bytes());
        for seat in &s.seats {
            f32s(&mut out, seat);
        }
        out.extend_from_slice(&(s.desc_count() as u16).to_le_bytes());
        for d in 0..s.desc_count() {
            let toks = s.desc(d);
            out.extend_from_slice(&(toks.len() as u16).to_le_bytes());
            for t in toks {
                out.extend_from_slice(&t.to_le_bytes());
            }
        }
        for (_, _, objs) in s.groups() {
            out.extend_from_slice(&(objs.len() as u16).to_le_bytes());
            for o in objs {
                out.extend_from_slice(&o.card.to_le_bytes());
                out.extend_from_slice(&o.desc.to_le_bytes());
                f32s(&mut out, &o.feats);
            }
        }
        f32s(&mut out, &row.win);
        f32s(&mut out, &row.life);
        f32s(&mut out, &[row.game_len]);
        out.extend_from_slice(&row.traj.to_le_bytes());
        out.extend_from_slice(&row.ply.to_le_bytes());
    }
    out
}

/// `None` on any magic / version / length mismatch — a stale or torn shard
/// is dropped whole.
pub fn read_table_shard(bytes: &[u8]) -> Option<Vec<TableRow>> {
    let mut r = Reader { b: bytes, pos: 0 };
    if r.take(4)? != TABLE_SHARD_MAGIC || r.u32()? != TABLE_SHARD_VERSION {
        return None;
    }
    let n = r.u32()? as usize;
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let mut global = [0.0f32; TABLE_GLOBAL_FEATS];
        for g in global.iter_mut() {
            *g = r.f32()?;
        }
        let seats = r.u16()? as usize;
        let mut s = TableState::with_seats(seats);
        s.global = global;
        for slot in 0..seats {
            for f in s.seat_mut(slot).iter_mut() {
                *f = r.f32()?;
            }
        }
        let descs = r.u16()? as usize;
        let mut toks = Vec::new();
        for _ in 0..descs {
            let len = r.u16()? as usize;
            toks.clear();
            for _ in 0..len {
                toks.push(r.u16()?);
            }
            s.add_desc(&toks);
        }
        for g in 0..seats * TABLE_ZONES {
            let len = r.u16()? as usize;
            for _ in 0..len {
                let card = r.u16()?;
                let desc = r.u16()?;
                if desc != NO_DESC && desc as usize >= descs {
                    return None;
                }
                let mut feats = [0.0f32; TABLE_OBJ_FEATS];
                for f in feats.iter_mut() {
                    *f = r.f32()?;
                }
                s.push(g / TABLE_ZONES, g % TABLE_ZONES, TableObject { card, desc, feats });
            }
        }
        let mut win = vec![0.0f32; seats];
        for w in win.iter_mut() {
            *w = r.f32()?;
        }
        let mut life = vec![0.0f32; seats];
        for l in life.iter_mut() {
            *l = r.f32()?;
        }
        let game_len = r.f32()?;
        let traj = r.u32()?;
        let ply = r.u16()?;
        rows.push(TableRow { state: s, win, life, game_len, traj, ply });
    }
    (r.pos == bytes.len()).then_some(rows)
}

// ───────────────────────────── inference net ────────────────────────────

/// The optional interaction stack: zone and slot tags, then blocks.
#[derive(Debug, Clone)]
struct TableStack {
    tag_zone: Tensor2, // [TABLE_ZONES, h]
    tag_slot: Tensor2, // [SLOT_TAGS, h]
    blocks: Vec<TBlock>,
}

/// The table value net, loaded from a trainer-exported safetensors file.
#[derive(Debug, Clone)]
pub struct TableNet {
    name_emb: Tensor2, // [vocab, E]
    tok_emb: Tensor2,  // [DESC_BUCKETS, E]
    obj_w: Tensor2,    // [h, 2E + TABLE_OBJ_FEATS]
    obj_b: Vec<f32>,
    stack: Option<TableStack>,
    seat_w: Tensor2, // [SH, TABLE_ZONES * 2h + SEAT_FEATS]
    seat_b: Vec<f32>,
    seat_slot: Tensor2, // [SLOT_TAGS, SH]
    trunk1_w: Tensor2,  // [h1, 3 SH + TABLE_GLOBAL_FEATS]
    trunk1_b: Vec<f32>,
    trunk2_w: Tensor2, // [h2, h1]
    trunk2_b: Vec<f32>,
    value1_w: Tensor2, // [VH, h2 + SH]
    value1_b: Vec<f32>,
    value2_w: Tensor2, // [1, VH]
    value2_b: Vec<f32>,
}

impl TableNet {
    pub fn vocab_size(&self) -> usize {
        self.name_emb.rows
    }

    /// Grow the name table to `new_vocab` rows — see
    /// [`PlayNet::pad_vocab`](crate::PlayNet::pad_vocab).
    pub fn pad_vocab(&mut self, new_vocab: usize) -> Result<(), NnError> {
        if new_vocab < self.name_emb.rows {
            return Err(NnError::BadTensor(
                "name_emb.weight",
                format!(
                    "net vocab {} is larger than the encoder's {new_vocab}",
                    self.name_emb.rows
                ),
            ));
        }
        self.name_emb.pad_rows(new_vocab);
        Ok(())
    }

    /// `(emb_dim, obj_hidden, seat_hidden, h1, h2, value_hidden, blocks)`
    /// as the file carries them.
    pub fn arch(&self) -> (usize, usize, usize, usize, usize, usize, usize) {
        (
            self.name_emb.cols,
            self.obj_w.rows,
            self.seat_w.rows,
            self.trunk1_w.rows,
            self.trunk2_w.rows,
            self.value1_w.rows,
            self.stack.as_ref().map_or(0, |s| s.blocks.len()),
        )
    }

    /// Parse a safetensors byte buffer. Training-only heads (`head_life.*`,
    /// `head_len.*`) are ignored; anything the forward pass reads is checked
    /// against the rest of the net here, so [`Self::forward`] can trust it.
    pub fn load(bytes: &[u8]) -> Result<TableNet, NnError> {
        let st = safetensors::SafeTensors::deserialize(bytes)
            .map_err(|e| NnError::BadFile(e.to_string()))?;
        let get = |name: &'static str| get_tensor(&st, name);
        let bias = |name: &'static str| get(name).map(|t| t.data);

        // The stack is all-or-nothing: both tags and at least one block, or
        // none of them. A partial set would run an architecture the weights
        // were not trained for.
        let n_blocks = count_tblocks(&st);
        let tags =
            ["tag_zone.weight", "tag_slot.weight"].iter().filter(|n| st.tensor(n).is_ok()).count();
        let stack = match (tags, n_blocks) {
            (0, 0) => None,
            (2, n) if n > 0 => Some(TableStack {
                tag_zone: get("tag_zone.weight")?,
                tag_slot: get("tag_slot.weight")?,
                blocks: load_tblocks(&st, n)?,
            }),
            _ => {
                return Err(NnError::BadTensor(
                    "tag_*/tblocks.*",
                    format!(
                        "{tags} of 2 tags with {n_blocks} blocks — the stack is all or nothing"
                    ),
                ));
            }
        };

        let net = TableNet {
            name_emb: get("name_emb.weight")?,
            tok_emb: get("tok_emb.weight")?,
            obj_w: get("obj.weight")?,
            obj_b: bias("obj.bias")?,
            stack,
            seat_w: get("seat.weight")?,
            seat_b: bias("seat.bias")?,
            seat_slot: get("seat_slot.weight")?,
            trunk1_w: get("trunk1.weight")?,
            trunk1_b: bias("trunk1.bias")?,
            trunk2_w: get("trunk2.weight")?,
            trunk2_b: bias("trunk2.bias")?,
            value1_w: get("value1.weight")?,
            value1_b: bias("value1.bias")?,
            value2_w: get("value2.weight")?,
            value2_b: bias("value2.bias")?,
        };

        let e = net.name_emb.cols;
        let h = net.obj_w.rows;
        let sh = net.seat_w.rows;
        let h1 = net.trunk1_w.rows;
        let h2 = net.trunk2_w.rows;
        let vh = net.value1_w.rows;
        for (name, ok) in [
            ("tok_emb.weight", net.tok_emb.rows == DESC_BUCKETS && net.tok_emb.cols == e),
            ("obj.weight", net.obj_w.cols == 2 * e + TABLE_OBJ_FEATS),
            ("obj.bias", net.obj_b.len() == h),
            ("seat.weight", net.seat_w.cols == TABLE_ZONES * 2 * h + SEAT_FEATS),
            ("seat.bias", net.seat_b.len() == sh),
            ("seat_slot.weight", net.seat_slot.rows == SLOT_TAGS && net.seat_slot.cols == sh),
            ("trunk1.weight", net.trunk1_w.cols == 3 * sh + TABLE_GLOBAL_FEATS),
            ("trunk1.bias", net.trunk1_b.len() == h1),
            ("trunk2.weight", net.trunk2_w.cols == h1),
            ("trunk2.bias", net.trunk2_b.len() == h2),
            ("value1.weight", net.value1_w.cols == h2 + sh),
            ("value1.bias", net.value1_b.len() == vh),
            ("value2.weight", net.value2_w.rows == 1 && net.value2_w.cols == vh),
            ("value2.bias", net.value2_b.len() == 1),
        ] {
            if !ok {
                return Err(NnError::BadTensor(
                    name,
                    "shape inconsistent with the rest of the net".into(),
                ));
            }
        }
        if let Some(stk) = &net.stack {
            if !h.is_multiple_of(crate::ATTN_HEADS) {
                return Err(NnError::BadTensor(
                    "tblocks.*",
                    format!("obj_hidden {h} not divisible by {} heads", crate::ATTN_HEADS),
                ));
            }
            if stk.tag_zone.rows != TABLE_ZONES || stk.tag_zone.cols != h {
                return Err(NnError::BadTensor(
                    "tag_zone.weight",
                    format!("want {TABLE_ZONES}x{h}"),
                ));
            }
            if stk.tag_slot.rows != SLOT_TAGS || stk.tag_slot.cols != h {
                return Err(NnError::BadTensor("tag_slot.weight", format!("want {SLOT_TAGS}x{h}")));
            }
            for (i, blk) in stk.blocks.iter().enumerate() {
                if !blk.shapes_fit(h) {
                    return Err(NnError::BadTensor(
                        "tblocks.*",
                        format!("block {i} shapes inconsistent with obj_hidden {h}"),
                    ));
                }
            }
        }
        Ok(net)
    }

    /// Each slot's chance to win, in the state's slot order, summing to 1.
    /// Empty for a state with no seats.
    pub fn forward(&self, s: &TableState) -> Vec<f32> {
        let logits = self.logits(s);
        if logits.is_empty() {
            return logits;
        }
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = logits.iter().map(|l| (l - max).exp()).collect();
        let sum: f32 = exps.iter().sum();
        exps.into_iter().map(|e| e / sum).collect()
    }

    /// The encoded seat's (slot 0's) chance to win — the number a search
    /// leaf or a scored pilot reads.
    pub fn win_prob(&self, s: &TableState) -> f32 {
        self.forward(s).first().copied().unwrap_or(0.0)
    }

    /// The value head's raw per-slot logits.
    pub fn logits(&self, s: &TableState) -> Vec<f32> {
        let seats = s.seat_count();
        if seats == 0 {
            return Vec::new();
        }
        let e = self.name_emb.cols;
        let h = self.obj_w.rows;
        let sh = self.seat_w.rows;

        // Descriptor embeddings, once per distinct card in the state: the
        // mean of the token rows. An empty descriptor embeds as zeros.
        let nd = s.desc_count();
        let mut demb = vec![0.0f32; nd * e];
        for d in 0..nd {
            let toks = s.desc(d);
            if toks.is_empty() {
                continue;
            }
            let row = &mut demb[d * e..(d + 1) * e];
            for &t in toks {
                let t = if (t as usize) < self.tok_emb.rows { t as usize } else { 0 };
                for (acc, v) in row.iter_mut().zip(&self.tok_emb.data[t * e..(t + 1) * e]) {
                    *acc += v;
                }
            }
            let inv = 1.0 / toks.len() as f32;
            for v in row.iter_mut() {
                *v *= inv;
            }
        }

        // Every object's input row, then the object layer as one product.
        let n = s.len();
        let k = self.obj_w.cols;
        let mut x = vec![0.0f32; n * k];
        let mut group = Vec::with_capacity(n);
        let mut i = 0;
        for (slot, zone, objs) in s.groups() {
            for o in objs {
                let row = &mut x[i * k..(i + 1) * k];
                let c = if (o.card as usize) < self.name_emb.rows { o.card as usize } else { 0 };
                row[..e].copy_from_slice(&self.name_emb.data[c * e..(c + 1) * e]);
                if o.desc != NO_DESC && (o.desc as usize) < nd {
                    let d = o.desc as usize;
                    row[e..2 * e].copy_from_slice(&demb[d * e..(d + 1) * e]);
                }
                row[2 * e..].copy_from_slice(&o.feats);
                group.push((slot, zone));
                i += 1;
            }
        }
        let mut hs = vec![0.0f32; n * h];
        self.obj_w.matmul_t(&x, n, &mut hs);
        for row in hs.chunks_exact_mut(h) {
            for (v, b) in row.iter_mut().zip(&self.obj_b) {
                *v = (*v + b).max(0.0);
            }
        }

        if let Some(stk) = &self.stack
            && n > 0
        {
            for (i, &(slot, zone)) in group.iter().enumerate() {
                let tz = &stk.tag_zone.data[zone * h..(zone + 1) * h];
                let ts = &stk.tag_slot.data[slot_tag(slot) * h..(slot_tag(slot) + 1) * h];
                for j in 0..h {
                    hs[i * h + j] += tz[j] + ts[j];
                }
            }
            run_tblocks(&stk.blocks, &mut hs, n, h);
        }

        // Mean and max per (slot, zone), laid out as each slot's seat-layer
        // input: its zones' pools, then its seat features.
        let seat_in_w = self.seat_w.cols;
        let mut seat_in = vec![0.0f32; seats * seat_in_w];
        let mut counts = vec![0usize; seats * TABLE_ZONES];
        for slot in 0..seats {
            for zone in 0..TABLE_ZONES {
                let base = slot * seat_in_w + zone * 2 * h;
                seat_in[base + h..base + 2 * h].fill(f32::NEG_INFINITY);
            }
        }
        for (i, &(slot, zone)) in group.iter().enumerate() {
            counts[slot * TABLE_ZONES + zone] += 1;
            let base = slot * seat_in_w + zone * 2 * h;
            for j in 0..h {
                let v = hs[i * h + j];
                seat_in[base + j] += v;
                let mx = &mut seat_in[base + h + j];
                *mx = mx.max(v);
            }
        }
        for slot in 0..seats {
            for zone in 0..TABLE_ZONES {
                let base = slot * seat_in_w + zone * 2 * h;
                let c = counts[slot * TABLE_ZONES + zone];
                if c == 0 {
                    seat_in[base..base + 2 * h].fill(0.0);
                    continue;
                }
                let inv = 1.0 / c as f32;
                for m in seat_in[base..base + h].iter_mut() {
                    *m *= inv;
                }
            }
            let fbase = slot * seat_in_w + TABLE_ZONES * 2 * h;
            seat_in[fbase..fbase + SEAT_FEATS].copy_from_slice(s.seat(slot));
        }

        // The seat layer, shared by every slot, plus the slot's own row.
        let mut z = vec![0.0f32; seats * sh];
        self.seat_w.matmul_t(&seat_in, seats, &mut z);
        for (slot, row) in z.chunks_exact_mut(sh).enumerate() {
            let tag = &self.seat_slot.data[slot_tag(slot) * sh..(slot_tag(slot) + 1) * sh];
            for ((v, b), t) in row.iter_mut().zip(&self.seat_b).zip(tag) {
                *v = (*v + b + t).max(0.0);
            }
        }

        // Trunk: me, the opponents' mean and max, the globals.
        let mut trunk_in = vec![0.0f32; self.trunk1_w.cols];
        trunk_in[..sh].copy_from_slice(&z[..sh]);
        if seats > 1 {
            let (mean, rest) = trunk_in[sh..3 * sh].split_at_mut(sh);
            rest.fill(f32::NEG_INFINITY);
            for row in z[sh..].chunks_exact(sh) {
                for j in 0..sh {
                    mean[j] += row[j];
                    rest[j] = rest[j].max(row[j]);
                }
            }
            let inv = 1.0 / (seats - 1) as f32;
            for m in mean.iter_mut() {
                *m *= inv;
            }
        }
        trunk_in[3 * sh..].copy_from_slice(&s.global);
        let mut t1 = vec![0.0f32; self.trunk1_w.rows];
        self.trunk1_w.matvec_t(&trunk_in, &mut t1);
        for (v, b) in t1.iter_mut().zip(&self.trunk1_b) {
            *v = (*v + b).max(0.0);
        }
        let h2 = self.trunk2_w.rows;
        let mut t2 = vec![0.0f32; h2];
        self.trunk2_w.matvec_t(&t1, &mut t2);
        for (v, b) in t2.iter_mut().zip(&self.trunk2_b) {
            *v = (*v + b).max(0.0);
        }

        // One logit per slot off [trunk ⊕ that slot's seat vector].
        let vin_w = self.value1_w.cols;
        let mut vin = vec![0.0f32; seats * vin_w];
        for slot in 0..seats {
            let row = &mut vin[slot * vin_w..(slot + 1) * vin_w];
            row[..h2].copy_from_slice(&t2);
            row[h2..].copy_from_slice(&z[slot * sh..(slot + 1) * sh]);
        }
        let vh = self.value1_w.rows;
        let mut u = vec![0.0f32; seats * vh];
        self.value1_w.matmul_t(&vin, seats, &mut u);
        for row in u.chunks_exact_mut(vh) {
            for (v, b) in row.iter_mut().zip(&self.value1_b) {
                *v = (*v + b).max(0.0);
            }
        }
        let mut logits = vec![0.0f32; seats];
        self.value2_w.matmul_t(&u, seats, &mut logits);
        for l in logits.iter_mut() {
            *l += self.value2_b[0];
        }
        logits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_safetensors;

    fn row(seats: usize) -> TableRow {
        let mut s = TableState::with_seats(seats);
        s.global[0] = 0.5;
        s.global[TABLE_GLOBAL_FEATS - 1] = 1.0;
        for slot in 0..seats {
            s.seat_mut(slot)[0] = slot as f32 * 0.25;
        }
        let d0 = s.add_desc(&[3, 9, 16_000]);
        let d1 = s.add_desc(&[]);
        let mut feats = [0.0; TABLE_OBJ_FEATS];
        feats[0] = 0.375;
        feats[TABLE_OBJ_FEATS - 1] = 1.0;
        s.push(0, Z_BF, TableObject { card: 7, desc: d0, feats });
        s.push(0, Z_HAND, TableObject { card: 2, desc: d1, feats: [0.5; TABLE_OBJ_FEATS] });
        s.push(
            seats - 1,
            Z_COMMAND,
            TableObject { card: 0, desc: NO_DESC, feats: [0.25; TABLE_OBJ_FEATS] },
        );
        let mut win = vec![0.0; seats];
        win[1] = 1.0;
        TableRow { state: s, win, life: vec![0.75; seats], game_len: 0.4, traj: 9, ply: 2 }
    }

    #[test]
    fn table_shard_roundtrip_is_exact() {
        let rows = vec![row(2), row(4), row(6)];
        let bytes = write_table_shard(&rows);
        assert_eq!(read_table_shard(&bytes).expect("decodes"), rows);
        assert!(read_table_shard(&bytes[..bytes.len() - 1]).is_none(), "torn shard");
        let mut stale = bytes.clone();
        stale[4] ^= 0xFF;
        assert!(read_table_shard(&stale).is_none(), "version bump");
        // A play-net shard is not a table shard, whatever its length.
        assert!(read_table_shard(&crate::write_shard(&[])).is_none());
    }

    #[test]
    fn groups_walk_slot_major_and_descriptors_read_back() {
        let r = row(3);
        let s = &r.state;
        assert_eq!(s.seat_count(), 3);
        assert_eq!(s.desc(0), &[3, 9, 16_000]);
        assert!(s.desc(1).is_empty());
        let got: Vec<(usize, usize, usize)> = s
            .groups()
            .filter(|(_, _, o)| !o.is_empty())
            .map(|(sl, z, o)| (sl, z, o.len()))
            .collect();
        assert_eq!(got, vec![(0, Z_BF, 1), (0, Z_HAND, 1), (2, Z_COMMAND, 1)]);
        assert_eq!(s.group(2, Z_COMMAND)[0].feats[0], 0.25);
    }

    /// A net small enough to check by hand, as `(name, shape, data)`:
    /// E = 1, h = 1, SH = 1, h1 = h2 = VH = 1. Every weight that could hide
    /// a layout mistake is non-zero somewhere.
    fn tiny() -> Vec<(&'static str, Vec<usize>, Vec<f32>)> {
        let mut tok = vec![0.0f32; DESC_BUCKETS];
        tok[3] = 1.0;
        tok[5] = 3.0;
        // obj input [name, desc, feats...]: 1 on name, 1 on desc, 1 on feat 0.
        let mut obj = vec![0.0f32; 2 + TABLE_OBJ_FEATS];
        obj[0] = 1.0;
        obj[1] = 1.0;
        obj[2] = 1.0;
        // seat input [zone pools (mean, max) x 6, seat feats]: 1 on the
        // battlefield mean, 1 on seat feat 0.
        let seat_in = TABLE_ZONES * 2 + SEAT_FEATS;
        let mut seat = vec![0.0f32; seat_in];
        seat[0] = 1.0;
        seat[TABLE_ZONES * 2] = 1.0;
        // trunk [me, opp mean, opp max, globals]: me − opp mean.
        let mut trunk1 = vec![0.0f32; 3 + TABLE_GLOBAL_FEATS];
        trunk1[0] = 1.0;
        trunk1[1] = -1.0;
        vec![
            ("name_emb.weight", vec![3, 1], vec![0.0, 2.0, 0.5]),
            ("tok_emb.weight", vec![DESC_BUCKETS, 1], tok),
            ("obj.weight", vec![1, 2 + TABLE_OBJ_FEATS], obj),
            ("obj.bias", vec![1], vec![0.0]),
            ("seat.weight", vec![1, seat_in], seat),
            ("seat.bias", vec![1], vec![0.0]),
            ("seat_slot.weight", vec![SLOT_TAGS, 1], vec![0.0; SLOT_TAGS]),
            ("trunk1.weight", vec![1, 3 + TABLE_GLOBAL_FEATS], trunk1),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![1.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            // value input [t2, seat]: the trunk shifts every slot alike, the
            // seat vector separates them.
            ("value1.weight", vec![1, 2], vec![0.5, 1.0]),
            ("value1.bias", vec![1], vec![0.0]),
            ("value2.weight", vec![1, 1], vec![1.0]),
            ("value2.bias", vec![1], vec![0.0]),
        ]
    }

    /// The forward pass against arithmetic done on paper, including the
    /// descriptor mean and the softmax over slots.
    #[test]
    fn forward_matches_hand_computation() {
        let net = TableNet::load(&to_safetensors(&tiny())).expect("loads");
        let mut s = TableState::with_seats(3);
        let d = s.add_desc(&[3, 5]); // mean of 1.0 and 3.0 = 2.0
        let mut feats = [0.0; TABLE_OBJ_FEATS];
        feats[0] = 0.5;
        // Slot 0: card 1 (2.0) + desc 2.0 + 0.5 = 4.5 → seat 4.5 + feat.
        s.push(0, Z_BF, TableObject { card: 1, desc: d, feats });
        // Slot 2: card 2 (0.5), no desc, feat 0 → 0.5; a second object with
        // card 0 and feat 1.5 → 1.5; mean 1.0.
        s.push(2, Z_BF, TableObject { card: 2, desc: NO_DESC, feats: [0.0; TABLE_OBJ_FEATS] });
        let mut f2 = [0.0; TABLE_OBJ_FEATS];
        f2[0] = 1.5;
        s.push(2, Z_BF, TableObject { card: 0, desc: NO_DESC, feats: f2 });
        s.seat_mut(1)[0] = 0.25;
        // z = [4.5, 0.25, 1.0]; trunk = relu(me − mean(opp)) = 4.5 − 0.625 =
        // 3.875; each logit is 0.5 · 3.875 + z (relu of positives).
        let want_logits = [4.5f32 + 1.9375, 0.25 + 1.9375, 1.0 + 1.9375];
        let got = net.logits(&s);
        for (g, w) in got.iter().zip(want_logits) {
            assert!((g - w).abs() < 1e-5, "logits {got:?} want {want_logits:?}");
        }
        let probs = net.forward(&s);
        let sum: f32 = want_logits.iter().map(|l| l.exp()).sum();
        for (p, l) in probs.iter().zip(want_logits) {
            assert!((p - l.exp() / sum).abs() < 1e-6, "{probs:?}");
        }
        assert!((probs.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!((net.win_prob(&s) - probs[0]).abs() < 1e-9);
    }

    /// At two slots the per-seat softmax is the play net's scalar head:
    /// `p0 = sigmoid(l0 − l1)`.
    #[test]
    fn two_slots_are_a_sigmoid_of_the_logit_gap() {
        let net = TableNet::load(&to_safetensors(&tiny())).expect("loads");
        let mut s = TableState::with_seats(2);
        s.seat_mut(0)[0] = 1.25;
        s.seat_mut(1)[0] = 0.5;
        let l = net.logits(&s);
        let want = 1.0 / (1.0 + (-(l[0] - l[1])).exp());
        assert!((net.win_prob(&s) - want).abs() < 1e-6);
    }

    /// With the slot tags zeroed, opponents are a set: swapping two
    /// opponents' seats swaps their probabilities and leaves slot 0's alone.
    #[test]
    fn opponents_are_a_set_under_equal_slot_tags() {
        let net = TableNet::load(&to_safetensors(&tiny())).expect("loads");
        let mut a = TableState::with_seats(3);
        a.seat_mut(1)[0] = 0.75;
        a.seat_mut(2)[0] = 2.0;
        let mut b = TableState::with_seats(3);
        b.seat_mut(1)[0] = 2.0;
        b.seat_mut(2)[0] = 0.75;
        let (pa, pb) = (net.forward(&a), net.forward(&b));
        assert!((pa[0] - pb[0]).abs() < 1e-6);
        assert!((pa[1] - pb[2]).abs() < 1e-6 && (pa[2] - pb[1]).abs() < 1e-6);
    }

    #[test]
    fn load_rejects_partial_stacks_and_play_net_files() {
        let mut half = tiny();
        half.push(("tag_zone.weight", vec![TABLE_ZONES, 1], vec![0.0; TABLE_ZONES]));
        assert!(TableNet::load(&to_safetensors(&half)).is_err(), "a tag without blocks");
        let mut short = tiny();
        short.retain(|(n, _, _)| *n != "tok_emb.weight");
        short.push(("tok_emb.weight", vec![10, 1], vec![0.0; 10]));
        assert!(TableNet::load(&to_safetensors(&short)).is_err(), "a short token table");
        // A play-net file carries none of the table names.
        let play = to_safetensors(&[
            ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
            ("obj.weight", vec![1, 1 + crate::OBJ_FEATS], vec![0.0; 1 + crate::OBJ_FEATS]),
            ("obj.bias", vec![1], vec![0.0]),
        ]);
        assert!(TableNet::load(&play).is_err());
        // And the reverse: a table file is not a play net.
        assert!(crate::PlayNet::load(&to_safetensors(&tiny())).is_err());
    }

    #[test]
    fn padding_the_name_table_leaves_known_rows_alone() {
        let mut net = TableNet::load(&to_safetensors(&tiny())).expect("loads");
        let mut s = TableState::with_seats(2);
        s.push(0, Z_BF, TableObject { card: 1, desc: NO_DESC, feats: [0.0; TABLE_OBJ_FEATS] });
        let before = net.forward(&s);
        net.pad_vocab(10).expect("grows");
        assert_eq!(net.vocab_size(), 10);
        assert_eq!(net.forward(&s), before);
        assert!(net.pad_vocab(5).is_err());
    }
}
