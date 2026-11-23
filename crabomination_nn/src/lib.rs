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
//! It lives outside the engine so the workspace can build it at
//! `opt-level = 3` in debug (root `Cargo.toml`): the forward pass runs
//! inside the bot's simulation loops, where opt-level 0 would make it cost
//! more than the game logic around it.
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
//! * Optionally, one of two pre-pool interaction architectures, chosen by
//!   which tensors the file carries: a single tagged attention layer
//!   (`attn.*`, see [`ATTN_HEADS`]) or a stack of pre-LN transformer
//!   blocks (`tblocks.group.weight` + `tblocks.{i}.{ln1,ln2}.*`,
//!   `tblocks.{i}.attn.{q,k,v,o}.*`, `tblocks.{i}.{ffn1,ffn2}.*`). A file
//!   carrying both, or a partial set of either, is rejected.
//!
//! Hidden sizes are read from the tensor shapes. The constants below are
//! the trainer's standard configuration plus what the encoder bakes into
//! the data (feature and group counts), which is fixed per format version.

use std::collections::BTreeMap;

pub mod table;

/// Zone groups, in trunk-input order. Seat-relative: "self" is the seat
/// being evaluated.
pub const NUM_GROUPS: usize = 8;
/// Group indices into [`EncodedState::groups`].
pub const G_BF_SELF: usize = 0;
pub const G_BF_OPP: usize = 1;
pub const G_HAND_SELF: usize = 2;
pub const G_GY_SELF: usize = 3;
pub const G_GY_OPP: usize = 4;
/// The encoded seat's own library as an unordered multiset: one object per
/// distinct name, remaining count in feature 27. Information the seat
/// genuinely has (its own decklist); previously only `library.len() / 40`.
/// A library really is an unordered set, so the bag-of-cards prior fits.
///
/// Order never reaches the net: entries are emitted sorted by vocabulary
/// index and pooling/attention are permutation-invariant. The encoder must
/// not leak the shuffle.
pub const G_LIB_SELF: usize = 5;
/// The stack, split by controller. One object per stack item (the spell's
/// card, or a trigger's source card), depth from the top in feature 36.
/// Replaces a bare count (`global[18]`) that could not say *what* is on the
/// stack or what it aims at. Stacks are shallow, so pooling is near-lossless.
pub const G_STACK_SELF: usize = 6;
pub const G_STACK_OPP: usize = 7;

/// Per-object feature count. Baked into encoded rows — bump
/// [`SHARD_VERSION`] if it changes.
///
/// * 12..=19 (round 4) keyword flags: flying, reach, menace, deathtouch,
///   lifelink, trample, first-or-double strike, vigilance.
/// * 20..=26 castability: colour pips, castable now, castable next turn.
/// * 27 library multiplicity.
///
/// 28..=36 (round 12) relation block: pooling cannot represent an edge
/// between objects, but a flag on each endpoint survives it, and attention
/// can match flagged endpoints across groups.
///
/// * 28 `is_blocking` / 29 `is_blocked` — combat edges from `block_map`.
/// * 30 `is_attached` — this object is an aura/equipment on a host.
/// * 31 `has_own_attachment` / 32 `has_opp_attachment` — the host's side,
///   split by the attachment's controller (own aura = buff, opposing one =
///   Pacifism).
/// * 33 `targeted_by_stack` — something on the stack aims at this.
/// * 34 counters other than loyalty/prepared, count / 4 (P/T counters
///   already arrive through effective P/T).
/// * 35 `is_attachment_type` — aura or equipment by printed type.
/// * 36 stack depth from the top, / 4 (stack groups only).
///
/// 37..=44 (round 28, v6):
///
/// * 37/38 counterpart power / effective-toughness sums, / 8 — on a
///   blocked attacker its blockers, on a blocker the attackers it blocks
///   (so a chump and a trade differ). An object is never both in one combat.
/// * 39 `attacking_non_player` — the attack targets a planeswalker or
///   battle, not a life total.
/// * 40..=44 keyword classes not in 12..=19: haste, hard-to-target
///   (ward/hexproof/shroud/protection), indestructible, hard-to-block
///   (unblockable/shadow/fear/intimidate/skulk/horsemanship/landwalk),
///   defender. Load-bearing for tokens and grants.
///
/// 45..=52 (round 40, v7):
///
/// * 45..=47 expiry: 45 damage marked, / 8 (feature 5 is `toughness −
///   damage`, which can't tell a damaged 4/4 from a 4/1); 46/47 the P/T
///   delta that expires at cleanup (`CardInstance::power_bonus`), / 4,
///   signed.
/// * 48..=52 counter kinds split out of feature 34: 48 +1/+1 / 4,
///   49 −1/−1 / 4, 50 stun / 2, 51 Page / 3, 52 Growth / 3.
///
/// 53..=58 (v8, 2026-08-30, modern precondition 3), gated by the `v8`
/// ablation bit:
///
/// * 53 artifact / 54 enchantment type bits — printed off the battlefield,
///   layer-computed on it; tokens and off-vocab cards had no other way to
///   tell a Chalice from a Signet from a Saga.
/// * 55..=58 modern counter kinds: 55 Lore / 3, 56 Charge / 4,
///   57 Shield / 2, 58 Finality / 2 (previously folded into feature 34).
pub const OBJ_FEATS: usize = 59;
/// Global scalar feature count. Baked into encoded rows likewise.
///
/// * 36..=42 (round 28): combat step one-hots (36 declare-attackers,
///   37 declare-blockers, 38 damage/end-combat — slot 11 alone conflated
///   them); 39/40 unblocked attacker power aimed at self/opp, / 12;
///   41/42 exile sizes self/opp, / 10.
/// * 43..=54 (round 40, v7) history: turn-scoped counters, self then
///   opponent, reset each turn, so "this turn" card text can tell whether
///   it is live. 43/44 life gained / 5; 45/46 instants+sorceries cast / 3;
///   47/48 spells cast / 4; 49/50 creatures died / 3; 51/52 cards that
///   left the graveyard / 3; 53/54 cards exiled / 3.
/// * 55/56 (v8): land drops remaining this turn, self then opponent, / 2.
pub const GLOBAL_FEATS: usize = 57;

/// `(OBJ_FEATS, GLOBAL_FEATS)` of earlier encoder generations, oldest
/// first: v5 (pre-round-28), v6 (rounds 28-39), v7 (round 40 to v8).
/// [`PlayNet::load`] widens such checkpoints with zero columns, which
/// computes exactly what the old binary did, so every historical net stays
/// loadable and golden traces do not move.
///
/// A file must match one generation on *both* counts; matching only one is
/// corruption and falls through to the shape check to fail loudly.
///
/// Public because the trainer must apply the same rule (`--use-best` pads
/// an old checkpoint before candle's exact-shape `VarMap::load`). The two
/// loaders must never disagree.
pub const LEGACY_FEATS: [(usize, usize); 3] = [(37, 36), (45, 43), (53, 55)];

/// Standard trainer configuration (the file's shapes win at load time).
/// Quadrupled in round 4; capacity turned out not to be the limit (see
/// [`ATTN_HEADS`]). Parameter count depends on the vocabulary too, so it is
/// not quoted; the trunk dominates, input `NUM_GROUPS · 2 · OBJ_HIDDEN +
/// GLOBAL_FEATS`.
pub const EMB_DIM: usize = 32;
pub const OBJ_HIDDEN: usize = 64;
pub const TRUNK_H1: usize = 512;
pub const TRUNK_H2: usize = 256;

/// Attention heads in the optional interaction layer. Must divide
/// [`OBJ_HIDDEN`].
///
/// The layer exists because per-group pooling cannot relate objects in
/// different zones ("my flier gets through because they have no flier or
/// reach"). The pooled net plateaued at `eval_material`'s AUC (0.746 vs
/// 0.755), which is why widening the trunk in round 4 could not help.
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
///
/// The objects of all [`NUM_GROUPS`] groups live in one buffer, group by
/// group in index order, with each group's length in `counts` — one
/// allocation instead of eight `Vec`s' worth (PERF `(-107)`).
///
/// Objects must be appended in group order — [`EncodedState::push`] and
/// [`EncodedState::push_default`] debug-assert it.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedState {
    pub global: [f32; GLOBAL_FEATS],
    objs: Vec<EncodedObject>,
    counts: [u32; NUM_GROUPS],
}

// Hand-written rather than derived: `Default` for arrays stops at length
// 32, and both feature counts have outgrown that.
impl Default for EncodedState {
    fn default() -> Self {
        Self { global: [0.0; GLOBAL_FEATS], objs: Vec::new(), counts: [0; NUM_GROUPS] }
    }
}

impl EncodedState {
    /// Reserve room for `n` objects across all groups. One call before the
    /// first push is the whole point of the layout.
    #[inline]
    pub fn reserve(&mut self, n: usize) {
        self.objs.reserve(n);
    }

    /// Every object, in group order.
    #[inline]
    pub fn objects(&self) -> &[EncodedObject] {
        &self.objs
    }

    /// Every object, in group order, mutably. The group boundaries are
    /// unchanged by anything this can do.
    #[inline]
    pub fn objects_mut(&mut self) -> &mut [EncodedObject] {
        &mut self.objs
    }

    /// Total object count across all groups.
    #[inline]
    pub fn len(&self) -> usize {
        self.objs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.objs.is_empty()
    }

    #[inline]
    pub fn group_len(&self, g: usize) -> usize {
        self.counts[g] as usize
    }

    #[inline]
    fn group_start(&self, g: usize) -> usize {
        self.counts[..g].iter().map(|&c| c as usize).sum()
    }

    /// Group `g`'s objects.
    #[inline]
    pub fn group(&self, g: usize) -> &[EncodedObject] {
        let start = self.group_start(g);
        &self.objs[start..start + self.counts[g] as usize]
    }

    /// Each group's slice in index order, walking the buffer once.
    pub fn groups(&self) -> impl Iterator<Item = &[EncodedObject]> {
        let mut start = 0usize;
        self.counts.iter().map(move |&c| {
            let s = start;
            start += c as usize;
            &self.objs[s..start]
        })
    }

    /// Append to group `g`. Groups must be filled in index order.
    #[inline]
    pub fn push(&mut self, g: usize, o: EncodedObject) {
        debug_assert!(
            self.counts[g + 1..].iter().all(|&c| c == 0),
            "EncodedState groups must be filled in index order (pushing {g})"
        );
        self.objs.push(o);
        self.counts[g] += 1;
    }

    /// Append a zeroed object to group `g` and hand back a handle to fill
    /// in place — see `encode_card_object_into` for what the by-value form
    /// cost. Groups must be filled in index order.
    #[inline]
    pub fn push_default(&mut self, g: usize) -> &mut EncodedObject {
        self.push(g, EncodedObject::default());
        self.objs.last_mut().expect("just pushed")
    }
}

impl Default for EncodedObject {
    fn default() -> Self {
        Self { card: 0, feats: [0.0; OBJ_FEATS] }
    }
}

/// Auxiliary short-horizon targets: `[Δlife-diff, Δpower-diff,
/// Δcreature-diff, opp hand next]`, each measured to the seat's next
/// snapshot and scaled — dense where `win` is sparse. Terminal rows carry
/// zero deltas.
pub const AUX_FEATS: usize = 4;

/// Rows of the policy head's decision-kind table (`head_policy_kind.*`):
/// one per kind of decision the engine's move list distinguishes (priority,
/// attack, block, each prompt kind — `server::decision_menu::Kind::index`),
/// with headroom. A policy logit is the shared head's plus the kind's row,
/// so the same successor state can score differently as the answer to a
/// discard than as a main-phase play.
pub const POLICY_KINDS: usize = 32;

/// The action head's object roles (`head_action.role.*` rows): what an
/// object is to a move — see [`ActionRef`].
pub const ACTION_ROLES: usize = 12;
/// Most object references one move carries into the action head; a longer
/// declaration keeps its first ones.
pub const ACTION_MAX_REFS: usize = 16;
/// The action head's hidden width the trainer builds (the engine reads it
/// off the tensors).
pub const ACTION_WIDTH: usize = 32;
/// [`ActionRef`]'s object for a reference with no encoded object (a
/// player): the role's own vector stands alone.
pub const NO_OBJECT: u16 = u16::MAX;
/// One object a move involves: its role, and its index in the decision
/// state's flat object list ([`EncodedState::objects`], group order), or
/// [`NO_OBJECT`].
pub type ActionRef = (u8, u16);

/// `TrainRow` labels are stamped after the game ends: `win` 1.0 / 0.0 /
/// 0.5 (draw) for the encoded seat, `life_diff` final (self − opp) life
/// clamped to ±20 and scaled by 1/20, `game_len` turns remaining from this
/// snapshot / 15. The two auxiliary targets aid credit assignment, not play.
#[derive(Debug, Clone, PartialEq)]
pub struct TrainRow {
    pub state: EncodedState,
    /// The game's actual result from this seat: 1 win, 0 loss. This stays
    /// the ground truth even when the trainer fits a bootstrapped target
    /// derived from it (see `SampleWindow::relabel_lambda`).
    pub win: f32,
    pub life_diff: f32,
    pub game_len: f32,
    /// Trajectory id, one per (game, seat). Rows bootstrap only against
    /// their own successors, so the trainer reassembles trajectories from a
    /// shuffled window by this.
    pub traj: u32,
    /// Position within the trajectory, ascending. Recorded rather than
    /// inferred from row order because the window samples and evicts.
    pub ply: u16,
    /// See [`AUX_FEATS`]. All-zero when the recorder predates them.
    pub aux: [f32; AUX_FEATS],
    /// The opponent's held cards at this snapshot as vocab indices
    /// (round 39): belief-head ground truth the encoder never sees.
    /// Index-0 (unknown) cards are skipped; duplicates appear once per copy.
    pub opp_hand: Vec<u16>,
}

// ───────────────────────────── shard format ─────────────────────────────

/// Magic + version guard the on-disk row format. Bump the version whenever
/// `OBJ_FEATS`, `GLOBAL_FEATS`, group order, or the row layout changes —
/// stale shards must fail loudly, not decode as garbage.
pub const SHARD_MAGIC: [u8; 4] = *b"CRML";
pub const SHARD_VERSION: u32 = 9;

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
        for group in row.state.groups() {
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
        out.extend_from_slice(&(row.opp_hand.len() as u16).to_le_bytes());
        for c in &row.opp_hand {
            out.extend_from_slice(&c.to_le_bytes());
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
        for g in 0..NUM_GROUPS {
            let len = r.u16()? as usize;
            state.reserve(len);
            for _ in 0..len {
                let card = r.u16()?;
                let mut feats = [0.0; OBJ_FEATS];
                for f in feats.iter_mut() {
                    *f = r.f32()?;
                }
                state.push(g, EncodedObject { card, feats });
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
        let oh_len = r.u16()? as usize;
        let mut opp_hand = Vec::with_capacity(oh_len);
        for _ in 0..oh_len {
            opp_hand.push(r.u16()?);
        }
        rows.push(TrainRow { state, win, life_diff, game_len, traj, ply, aux, opp_hand });
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

/// Deck-label shard: decklists labelled with a gauntlet win rate rather
/// than one game's outcome (distillation set). Accumulates across
/// generation passes by read, extend, rewrite — it is tiny.
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
/// over the card multiset: embedding rows summed, meaned and maxed, then a
/// two-layer trunk over `[pools ⊕ deck features]`. Its own safetensors
/// file, reusing the play net's trunk tensor names.
///
/// Passed `selfplay_train --gate-builder`: net-judged best-of-32 sealed
/// builds beat `static_build_score`'s ~61 % over 1 200 games, two seeds.
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

    /// Grow the embedding table to `new_vocab` rows, zero-filling the ones
    /// this net never saw. Sound because card names own a frozen index
    /// (`server::vocab_snapshot`) and a vocabulary only grows at the end.
    /// (Nets from before that freeze are unloadable, not merely short.)
    /// Narrowing is rejected.
    pub fn pad_vocab(&mut self, new_vocab: usize) -> Result<(), NnError> {
        if new_vocab < self.emb.rows {
            return Err(NnError::BadTensor(
                "emb.weight",
                format!("net vocab {} is larger than the encoder's {new_vocab}", self.emb.rows),
            ));
        }
        self.emb.pad_rows(new_vocab);
        Ok(())
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
            // Out of range means "a card this net was not trained on", and
            // index 0 is the reserved unknown slot. Clamping to the *last*
            // row instead would hand it some unrelated card's embedding.
            let c = if (card as usize) < self.emb.rows { card as usize } else { 0 };
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

/// Outputs a [`matvec_raw`] block keeps in accumulators: 64 f32 are eight
/// 256-bit registers, enough independent add chains to cover the add's
/// latency.
const MV_BLOCK: usize = 64;
/// [`matmul_raw`]'s register block: 4 input rows × 16 outputs, eight
/// 256-bit accumulators fed by two weight loads per input column.
const MM_ROWS: usize = 4;
const MM_COLS: usize = 16;

/// `e^x` for `x ≤ 0` (max-subtracted softmax scores) as a branch-free,
/// vectorisable polynomial instead of a libm call per element. Cephes'
/// single-precision reduction: relative error ~1e-7, f32 rounding order.
#[inline(always)]
fn exp_neg(x: f32) -> f32 {
    // Below −87 the result underflows f32's normal range; clamp so the
    // exponent bits below stay valid (the weight is ~1e-38 either way).
    let x = x.max(-87.0);
    let n = (x * std::f32::consts::LOG2_E + 0.5).floor();
    let r = x - n * 0.693_359_4 - n * -2.121_944_4e-4;
    let z = r * r;
    let p = ((((1.987_569_1e-4 * r + 1.398_199_9e-3) * r + 8.333_452e-3) * r + 4.166_579_6e-2) * r
        + 1.666_666_5e-1)
        * r
        + 0.5; // Cephes' 5.0000001201e-1, which rounds to exactly 0.5 in f32
    let e = p * z + r + 1.0;
    // 2^n by building the exponent field directly. `n as i32` saturates,
    // which blocks vectorisation; adding 1.5·2^23 leaves integer n in the
    // low mantissa bits (exact for |n| < 2^22; here −126 ≤ n ≤ 0).
    const MAGIC: f32 = 12_582_912.0; // 1.5 · 2^23
    let n_bits = (n + MAGIC).to_bits().wrapping_sub(MAGIC.to_bits());
    e * f32::from_bits(n_bits.wrapping_add(127) << 23)
}

/// Largest element of `xs` (−∞ when empty), in lane-wise partial maxima
/// so the reduction vectorises; `max` is associative, so the answer is
/// the one a sequential scan gives.
#[inline(always)]
fn max_lanes(xs: &[f32]) -> f32 {
    let mut acc = [f32::NEG_INFINITY; 8];
    let (chunks, rest) = xs.as_chunks::<8>();
    for c in chunks {
        for l in 0..8 {
            acc[l] = acc[l].max(c[l]);
        }
    }
    let mut m = f32::NEG_INFINITY;
    for &v in rest.iter().chain(&acc) {
        m = m.max(v);
    }
    m
}

/// Sum of `xs` in eight lane-wise partial sums (vectorised; the order
/// differs from a sequential sum by f32 rounding only).
#[inline(always)]
fn sum_lanes(xs: &[f32]) -> f32 {
    let mut acc = [0.0f32; 8];
    let (chunks, rest) = xs.as_chunks::<8>();
    for c in chunks {
        for l in 0..8 {
            acc[l] += c[l];
        }
    }
    let mut s = 0.0f32;
    for &v in rest {
        s += v;
    }
    for v in acc {
        s += v;
    }
    s
}

/// Multi-head scaled dot-product attention over `n` objects: `q`, `k`,
/// `v` packed `n × h`, the context written into `ctx` (`n × h`). Shared by
/// the single attention layer and the transformer blocks.
///
/// Each head is two blocked matmuls: `S = Q·Kᵀ` (keys transposed and
/// padded to the block width) and `C = P·V`. Softmax runs over the real
/// keys only and padded weights are zero, so padding never shows.
#[inline(always)]
fn attention_core(q: &[f32], k: &[f32], v: &[f32], n: usize, h: usize, ctx: &mut [f32]) {
    let hd = h / ATTN_HEADS;
    let scale = 1.0 / (hd as f32).sqrt();
    let np = n.div_ceil(MM_COLS) * MM_COLS;
    let mut qh = vec![0.0f32; n * hd];
    let mut kt = vec![0.0f32; hd * np];
    let mut vh = vec![0.0f32; np * hd];
    let mut scores = vec![0.0f32; n * np];
    let mut ch = vec![0.0f32; n * hd];
    for head in 0..ATTN_HEADS {
        let off = head * hd;
        for i in 0..n {
            qh[i * hd..(i + 1) * hd].copy_from_slice(&q[i * h + off..i * h + off + hd]);
            vh[i * hd..(i + 1) * hd].copy_from_slice(&v[i * h + off..i * h + off + hd]);
            for d in 0..hd {
                kt[d * np + i] = k[i * h + off + d];
            }
        }
        kernel::matmul(&qh, n, hd, &kt, np, &mut scores);
        for row in scores.chunks_exact_mut(np) {
            let real = &mut row[..n];
            for sc in real.iter_mut() {
                *sc *= scale;
            }
            // Max-subtracted softmax. Max, exp and sum are separate passes
            // because a loop carrying a sequential float sum won't vectorise.
            let max = max_lanes(real);
            for sc in real.iter_mut() {
                *sc = exp_neg(*sc - max);
            }
            let inv = 1.0 / sum_lanes(real);
            for sc in real.iter_mut() {
                *sc *= inv;
            }
            row[n..].fill(0.0);
        }
        kernel::matmul(&scores, n, np, &vh, hd, &mut ch);
        for i in 0..n {
            ctx[i * h + off..i * h + off + hd].copy_from_slice(&ch[i * hd..(i + 1) * hd]);
        }
    }
}

/// `out = W · x` given `wt` = Wᵀ (`k × m`, `k = x.len()`, `m = out.len()`):
/// each `x[c]` is broadcast against a block of [`MV_BLOCK`] accumulated
/// outputs, so each weight loads once and no output needs a horizontal
/// reduction (unlike row-wise [`dot`]s). Every output sums inputs in order
/// `c = 0..k`, as [`matmul_raw`] does, so both kernels give the same bits.
#[inline(always)]
fn matvec_raw(x: &[f32], wt: &[f32], m: usize, out: &mut [f32]) {
    debug_assert_eq!(wt.len(), x.len() * m);
    debug_assert_eq!(out.len(), m);
    let mut j0 = 0;
    while j0 + MV_BLOCK <= m {
        let mut acc = [0.0f32; MV_BLOCK];
        for (c, &xc) in x.iter().enumerate() {
            // Skipping a zero input is bit-exact and skips its weight row.
            // Trunk inputs are zero-heavy: empty zones pool to zero (stack
            // groups almost always) and relu zeroes ~half the hidden layer.
            if xc == 0.0 {
                continue;
            }
            let w: &[f32; MV_BLOCK] = wt[c * m + j0..c * m + j0 + MV_BLOCK].try_into().unwrap();
            for l in 0..MV_BLOCK {
                acc[l] += xc * w[l];
            }
        }
        out[j0..j0 + MV_BLOCK].copy_from_slice(&acc);
        j0 += MV_BLOCK;
    }
    if j0 < m {
        let rest = &mut out[j0..];
        rest.fill(0.0);
        for (c, &xc) in x.iter().enumerate() {
            for (o, w) in rest.iter_mut().zip(&wt[c * m + j0..(c + 1) * m]) {
                *o += xc * w;
            }
        }
    }
}

/// `out[i] = W · x[i]` for `n` input rows packed in `x` (`n × k`), given
/// `wt` = Wᵀ (`k × m`); `out` is `n × m`. A register block of [`MM_ROWS`]
/// rows × [`MM_COLS`] outputs shares every weight load across the rows.
/// A width the block doesn't divide, and the last `n % MM_ROWS` rows, go
/// through [`matvec_raw`]; the summation order is the same either way.
#[inline(always)]
fn matmul_raw(x: &[f32], n: usize, k: usize, wt: &[f32], m: usize, out: &mut [f32]) {
    debug_assert_eq!(x.len(), n * k);
    debug_assert_eq!(wt.len(), k * m);
    debug_assert_eq!(out.len(), n * m);
    let mut i = 0;
    if m.is_multiple_of(MM_COLS) {
        while i + MM_ROWS <= n {
            let xs: [&[f32]; MM_ROWS] = std::array::from_fn(|r| &x[(i + r) * k..(i + r + 1) * k]);
            for j0 in (0..m).step_by(MM_COLS) {
                let mut acc = [[0.0f32; MM_COLS]; MM_ROWS];
                for c in 0..k {
                    let w: &[f32; MM_COLS] = wt[c * m + j0..c * m + j0 + MM_COLS].try_into().unwrap();
                    for r in 0..MM_ROWS {
                        let xv = xs[r][c];
                        for l in 0..MM_COLS {
                            acc[r][l] += xv * w[l];
                        }
                    }
                }
                for (r, a) in acc.iter().enumerate() {
                    out[(i + r) * m + j0..(i + r) * m + j0 + MM_COLS].copy_from_slice(a);
                }
            }
            i += MM_ROWS;
        }
    }
    for i in i..n {
        matvec_raw(&x[i * k..(i + 1) * k], wt, m, &mut out[i * m..(i + 1) * m]);
    }
}

/// The two kernels as separately compiled functions, picked per call.
///
/// Inlined into the forward pass's one big AVX2 function, a kernel ran at
/// about a third of its isolated speed (register allocation); out of line
/// it keeps its tight loop. A handful of calls per forward, so the dispatch
/// is free.
mod kernel {
    #[inline]
    pub(super) fn matmul(x: &[f32], n: usize, k: usize, wt: &[f32], m: usize, out: &mut [f32]) {
        #[cfg(target_arch = "x86_64")]
        if super::avx2_fma() {
            // SAFETY: the detected features gate the call.
            return unsafe { matmul_avx2(x, n, k, wt, m, out) };
        }
        matmul_base(x, n, k, wt, m, out)
    }

    #[inline]
    pub(super) fn matvec(x: &[f32], wt: &[f32], m: usize, out: &mut [f32]) {
        #[cfg(target_arch = "x86_64")]
        if super::avx2_fma() {
            // SAFETY: the detected features gate the call.
            return unsafe { matvec_avx2(x, wt, m, out) };
        }
        matvec_base(x, wt, m, out)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2", enable = "fma")]
    #[inline(never)]
    unsafe fn matmul_avx2(x: &[f32], n: usize, k: usize, wt: &[f32], m: usize, out: &mut [f32]) {
        super::matmul_raw(x, n, k, wt, m, out)
    }

    #[inline(never)]
    fn matmul_base(x: &[f32], n: usize, k: usize, wt: &[f32], m: usize, out: &mut [f32]) {
        super::matmul_raw(x, n, k, wt, m, out)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2", enable = "fma")]
    #[inline(never)]
    unsafe fn matvec_avx2(x: &[f32], wt: &[f32], m: usize, out: &mut [f32]) {
        super::matvec_raw(x, wt, m, out)
    }

    #[inline(never)]
    fn matvec_base(x: &[f32], wt: &[f32], m: usize, out: &mut [f32]) {
        super::matvec_raw(x, wt, m, out)
    }
}

/// Row-major dense matrix (`data[r * cols + c]`).
#[derive(Debug, Clone)]
pub struct Tensor2 {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f32>,
    /// `data` transposed (`t[c * rows + r]`), which the forward pass's
    /// blocked kernels read: an input broadcast against one contiguous
    /// run of outputs. Kept in step by [`Tensor2::new`] and both pads.
    t: Vec<f32>,
}

/// Eight-accumulator dot product. Strict f32 semantics stop LLVM from
/// reassociating a single-accumulator loop into SIMD lanes; splitting the
/// accumulation makes the reassociation explicit so it vectorises.
///
/// `inline(always)` is load-bearing: the AVX2 wrapper relies on this body
/// inlining into its `#[target_feature]` scope. (Rust never contracts
/// `a * b + c` into an FMA, so every kernel here is a multiply and an add.)
#[inline(always)]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    const LANES: usize = 8;
    let n = a.len().min(b.len());
    let chunks = n / LANES;
    let mut acc = [0.0f32; LANES];
    for i in 0..chunks {
        let ao = &a[i * LANES..(i + 1) * LANES];
        let bo = &b[i * LANES..(i + 1) * LANES];
        for l in 0..LANES {
            acc[l] += ao[l] * bo[l];
        }
    }
    let mut tail = 0.0f32;
    for i in chunks * LANES..n {
        tail += a[i] * b[i];
    }
    let mut sum = tail;
    for v in acc {
        sum += v;
    }
    sum
}

/// [`dot`] compiled with AVX2 (8-wide lanes vs the SSE2 baseline's 4),
/// picked at runtime. Safety: the caller checks the CPU features.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
unsafe fn dot_avx2(a: &[f32], b: &[f32]) -> f32 {
    dot(a, b)
}

/// Whether this CPU has AVX2 and FMA, detected once.
#[cfg(target_arch = "x86_64")]
#[inline]
fn avx2_fma() -> bool {
    use std::sync::OnceLock;
    static AVX2: OnceLock<bool> = OnceLock::new();
    *AVX2.get_or_init(|| {
        std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma")
    })
}

/// Runtime CPU-feature dispatch for [`dot`], the check cached to a bool.
#[inline]
fn dot_dispatch(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    if avx2_fma() {
        // SAFETY: the detected features gate the call.
        return unsafe { dot_avx2(a, b) };
    }
    dot(a, b)
}

impl Tensor2 {
    fn new(rows: usize, cols: usize, data: Vec<f32>) -> Self {
        let mut t = Tensor2 { rows, cols, data, t: Vec::new() };
        t.retranspose();
        t
    }

    fn retranspose(&mut self) {
        let (rows, cols) = (self.rows, self.cols);
        self.t = vec![0.0; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                self.t[c * rows + r] = self.data[r * cols + c];
            }
        }
    }

    /// `out = self · x` — [`matvec_raw`] on this matrix's transposed copy.
    #[inline(always)]
    fn matvec_t(&self, x: &[f32], out: &mut [f32]) {
        debug_assert_eq!(x.len(), self.cols);
        kernel::matvec(x, &self.t, self.rows, out);
    }

    /// `out[i] = self · x[i]` for `n` packed rows — [`matmul_raw`] on this
    /// matrix's transposed copy. The per-object layers run as one call.
    #[inline(always)]
    fn matmul_t(&self, x: &[f32], n: usize, out: &mut [f32]) {
        kernel::matmul(x, n, self.cols, &self.t, self.rows, out);
    }

    /// `out = self · x` (no accumulation into prior contents).
    fn matvec(&self, x: &[f32], out: &mut [f32]) {
        debug_assert_eq!(x.len(), self.cols);
        debug_assert_eq!(out.len(), self.rows);
        for (r, o) in out.iter_mut().enumerate() {
            *o = dot_dispatch(&self.data[r * self.cols..(r + 1) * self.cols], x);
        }
    }

    /// Widen to `new_cols` by appending zero columns on the right. Input
    /// layouts put the newest features last (objects: `emb ++ feats`;
    /// trunk: pooled groups then globals), so a padded legacy matrix
    /// computes exactly what it did before the feature bump.
    fn pad_cols(&mut self, new_cols: usize) {
        debug_assert!(new_cols >= self.cols);
        let mut data = vec![0.0f32; self.rows * new_cols];
        for r in 0..self.rows {
            data[r * new_cols..r * new_cols + self.cols]
                .copy_from_slice(&self.data[r * self.cols..(r + 1) * self.cols]);
        }
        self.cols = new_cols;
        self.data = data;
        self.retranspose();
    }

    /// Extend to `new_rows` by appending zero rows at the bottom. Used for
    /// vocabulary growth: frozen card indices (`server::vocab_snapshot`)
    /// only grow at the end, and an unseen card embedding as zeros matches
    /// what an off-set card gets anyway.
    pub fn pad_rows(&mut self, new_rows: usize) {
        debug_assert!(new_rows >= self.rows);
        self.data.resize(new_rows * self.cols, 0.0);
        self.rows = new_rows;
        self.retranspose();
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
        raw.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect();
    Ok(Tensor2::new(rows, cols, data))
}

/// [`PlayNet`]'s action head. For a move with references `R` in a
/// decision of kind `k`, over the decision state's object embeddings `h`
/// and trunk output `t`:
/// `a = Σ_{(r, i) ∈ R} relu(obj · h_i + obj_b + role_r)` (`h_i` = 0 for
/// [`NO_OBJECT`]), `z = relu(ctx · t + ctx_b + kind_k + a)`,
/// `term = out · z + out_b`.
#[derive(Debug, Clone)]
struct ActionHead {
    obj_w: Tensor2,  // [A, h_obj]
    obj_b: Vec<f32>, // [A]
    role: Tensor2,   // [ACTION_ROLES, A]
    ctx_w: Tensor2,  // [A, h2]
    ctx_b: Vec<f32>, // [A]
    kind: Tensor2,   // [POLICY_KINDS, A]
    out_w: Tensor2,  // [1, A]
    out_b: f32,
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
    /// The policy head (`head_policy.*`, [1, h2] + [1]), only in weights
    /// trained with `--policy-head`. Consumed at inference (its logit over
    /// a successor state is the search prior), so it loads all-or-nothing.
    /// Absent (every checkpoint before round 37): rank by the win head.
    policy_w: Option<Tensor2>, // [1, h2]
    policy_b: f32,
    /// The decision-kind table (`head_policy_kind.*`, [POLICY_KINDS, h2] +
    /// [POLICY_KINDS]): kind `k`'s logit adds row `k`'s dot with the trunk
    /// and bias `k` to the shared head's. Present only with the policy
    /// head, all-or-nothing; absent means every kind reads the shared head.
    policy_kind_w: Option<Tensor2>,
    policy_kind_b: Vec<f32>,
    /// The action head (`head_action.*`): a policy-logit term scoring a
    /// move by the objects it involves, read from the decision state's
    /// object embeddings — see [`PlayNet::forward_action_terms`]. Only
    /// beside the policy head, all-or-nothing.
    action: Option<ActionHead>,
    /// The opponent-hand belief head (`head_opp.*`, [vocab, h2] + [vocab]),
    /// round 39. Consumed by `determinize_hidden_belief`, so all-or-nothing.
    /// Absent (pre-round-39): uniform redeals.
    opp_w: Option<Tensor2>, // [vocab, h2]
    opp_b: Vec<f32>,        // [vocab] (empty when absent)
    /// Present only in weights trained with the interaction layer; absent
    /// means pure deep-sets. The file's tensors pick the architecture, not
    /// a caller flag.
    attn: Option<Attn>,
    /// Present only in weights trained with the transformer stack
    /// (`tblocks.*`). Mutually exclusive with [`Self::attn`] — a file
    /// carrying both is a trainer bug and is rejected at load.
    tstack: Option<TStack>,
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

/// Pre-pool transformer stack: the "proper blocks" successor to [`Attn`].
///
/// The group tag is added into the residual stream once at stack entry
/// (like a positional embedding): normed residual blocks have no clean
/// "scores only" place for it, and pooling is per-group anyway.
#[derive(Debug, Clone)]
struct TStack {
    /// Learned per-group tag, `[NUM_GROUPS, obj_hidden]`.
    group: Tensor2,
    blocks: Vec<TBlock>,
}

/// One pre-LN transformer block over the object set:
/// `x += attn(ln1(x)); x += ffn2(relu(ffn1(ln2(x))))`. No relu on the
/// residual stream, layer-norm eps 1e-5 (candle's default — parity).
#[derive(Debug, Clone)]
struct TBlock {
    ln1_w: Vec<f32>,
    ln1_b: Vec<f32>,
    q_w: Tensor2,
    q_b: Vec<f32>,
    k_w: Tensor2,
    k_b: Vec<f32>,
    v_w: Tensor2,
    v_b: Vec<f32>,
    o_w: Tensor2,
    o_b: Vec<f32>,
    ln2_w: Vec<f32>,
    ln2_b: Vec<f32>,
    ffn1_w: Tensor2,
    ffn1_b: Vec<f32>,
    ffn2_w: Tensor2,
    ffn2_b: Vec<f32>,
}

impl TBlock {
    /// Every tensor sized for a residual stream of width `h`.
    fn shapes_fit(&self, h: usize) -> bool {
        let square = [&self.q_w, &self.k_w, &self.v_w, &self.o_w].iter().all(|t| t.rows == h && t.cols == h);
        let vecs = [&self.ln1_w, &self.ln1_b, &self.ln2_w, &self.ln2_b, &self.q_b, &self.k_b, &self.v_b, &self.o_b]
            .iter()
            .all(|v| v.len() == h);
        let ffn = self.ffn1_w.cols == h
            && self.ffn1_b.len() == self.ffn1_w.rows
            && self.ffn2_w.cols == self.ffn1_w.rows
            && self.ffn2_w.rows == h
            && self.ffn2_b.len() == h;
        square && vecs && ffn
    }
}

/// [`get_tensor`] for a name built at runtime (a block index), reporting a
/// failure against the `family` name the caller's error carries.
fn get_tensor_dyn(
    st: &safetensors::SafeTensors<'_>,
    family: &'static str,
    name: &str,
) -> Result<Tensor2, NnError> {
    let Ok(view) = st.tensor(name) else {
        return Err(NnError::BadTensor(family, format!("missing {name}")));
    };
    if view.dtype() != safetensors::Dtype::F32 {
        return Err(NnError::BadTensor(family, format!("{name}: dtype {:?}, want F32", view.dtype())));
    }
    let (rows, cols) = match *view.shape() {
        [r, c] => (r, c),
        [n] => (1, n),
        _ => {
            return Err(NnError::BadTensor(family, format!("{name}: rank {} shape", view.shape().len())));
        }
    };
    let data = view.data().as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect();
    Ok(Tensor2::new(rows, cols, data))
}

/// How many `tblocks.{i}` blocks a file carries: counted while
/// `tblocks.{i}.ln1.weight` exists.
fn count_tblocks(st: &safetensors::SafeTensors<'_>) -> usize {
    let mut n = 0;
    while st.tensor(&format!("tblocks.{n}.ln1.weight")).is_ok() {
        n += 1;
    }
    n
}

/// The first `n` transformer blocks, each with its full tensor set — a gap
/// is a trainer/inference mismatch and fails the load.
fn load_tblocks(st: &safetensors::SafeTensors<'_>, n: usize) -> Result<Vec<TBlock>, NnError> {
    let getd = |name: &str| get_tensor_dyn(st, "tblocks.*", name);
    let mut blocks = Vec::with_capacity(n);
    for i in 0..n {
        let p = |suffix: &str| format!("tblocks.{i}.{suffix}");
        blocks.push(TBlock {
            ln1_w: getd(&p("ln1.weight"))?.data,
            ln1_b: getd(&p("ln1.bias"))?.data,
            q_w: getd(&p("attn.q.weight"))?,
            q_b: getd(&p("attn.q.bias"))?.data,
            k_w: getd(&p("attn.k.weight"))?,
            k_b: getd(&p("attn.k.bias"))?.data,
            v_w: getd(&p("attn.v.weight"))?,
            v_b: getd(&p("attn.v.bias"))?.data,
            o_w: getd(&p("attn.o.weight"))?,
            o_b: getd(&p("attn.o.bias"))?.data,
            ln2_w: getd(&p("ln2.weight"))?.data,
            ln2_b: getd(&p("ln2.bias"))?.data,
            ffn1_w: getd(&p("ffn1.weight"))?,
            ffn1_b: getd(&p("ffn1.bias"))?.data,
            ffn2_w: getd(&p("ffn2.weight"))?,
            ffn2_b: getd(&p("ffn2.bias"))?.data,
        });
    }
    Ok(blocks)
}

/// Layer norm over one row, eps matching candle's `LayerNormConfig`
/// default.
#[inline(always)]
fn layer_norm_row(x: &[f32], w: &[f32], b: &[f32], out: &mut [f32]) {
    let d = x.len();
    let mean = x.iter().sum::<f32>() / d as f32;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / d as f32;
    let inv = 1.0 / (var + 1e-5).sqrt();
    for i in 0..d {
        out[i] = (x[i] - mean) * inv * w[i] + b[i];
    }
}

/// The pre-LN transformer blocks over `n` objects' hidden rows (`hs`,
/// `n × h`), in place. Shared by [`PlayNet`]'s `tblocks.*` stack and the
/// table net ([`table::TableNet`]): the group / zone tags are the callers'
/// business and are already in `hs` by the time this runs.
#[inline(always)]
fn run_tblocks(blocks: &[TBlock], hs: &mut [f32], n: usize, h_obj: usize) {
    let mut xn = vec![0.0f32; n * h_obj];
    let mut proj = vec![0.0f32; n * h_obj];
    let mut ctx = vec![0.0f32; n * h_obj];
    for blk in blocks {
        // x += attn(ln1(x))
        for i in 0..n {
            layer_norm_row(
                &hs[i * h_obj..(i + 1) * h_obj],
                &blk.ln1_w,
                &blk.ln1_b,
                &mut xn[i * h_obj..(i + 1) * h_obj],
            );
        }
        let project = |w: &Tensor2, b: &[f32]| {
            let mut out = vec![0.0f32; n * h_obj];
            w.matmul_t(&xn, n, &mut out);
            for row in out.chunks_exact_mut(h_obj) {
                for (v, bb) in row.iter_mut().zip(b) {
                    *v += bb;
                }
            }
            out
        };
        let q = project(&blk.q_w, &blk.q_b);
        let k = project(&blk.k_w, &blk.k_b);
        let v = project(&blk.v_w, &blk.v_b);
        attention_core(&q, &k, &v, n, h_obj, &mut ctx);
        blk.o_w.matmul_t(&ctx, n, &mut proj);
        for i in 0..n {
            for j in 0..h_obj {
                hs[i * h_obj + j] += proj[i * h_obj + j] + blk.o_b[j];
            }
        }

        // x += ffn2(relu(ffn1(ln2(x)))) — rows are independent, so the
        // two layers run over all of them at once.
        for i in 0..n {
            layer_norm_row(
                &hs[i * h_obj..(i + 1) * h_obj],
                &blk.ln2_w,
                &blk.ln2_b,
                &mut xn[i * h_obj..(i + 1) * h_obj],
            );
        }
        let f = blk.ffn1_w.rows;
        let mut f1 = vec![0.0f32; n * f];
        blk.ffn1_w.matmul_t(&xn, n, &mut f1);
        for row in f1.chunks_exact_mut(f) {
            for (v, b) in row.iter_mut().zip(&blk.ffn1_b) {
                *v = (*v + b).max(0.0);
            }
        }
        blk.ffn2_w.matmul_t(&f1, n, &mut proj);
        for i in 0..n {
            for j in 0..h_obj {
                hs[i * h_obj + j] += proj[i * h_obj + j] + blk.ffn2_b[j];
            }
        }
    }
}

impl PlayNet {
    /// Vocabulary size the embedding table was trained with. The encoder's
    /// vocab must be this or larger — see [`PlayNet::pad_vocab`].
    pub fn vocab_size(&self) -> usize {
        self.emb.rows
    }

    /// Grow the embedding table (and the vocabulary-sized opponent-card
    /// head that shadows it) to `new_vocab` rows. See
    /// [`DeckNet::pad_vocab`] for why zero-filling is sound.
    pub fn pad_vocab(&mut self, new_vocab: usize) -> Result<(), NnError> {
        if new_vocab < self.emb.rows {
            return Err(NnError::BadTensor(
                "emb.weight",
                format!("net vocab {} is larger than the encoder's {new_vocab}", self.emb.rows),
            ));
        }
        self.emb.pad_rows(new_vocab);
        if let Some(w) = &mut self.opp_w {
            w.pad_rows(new_vocab);
            self.opp_b.resize(new_vocab, 0.0);
        }
        Ok(())
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
            policy_w: None,
            policy_b: 0.0,
            policy_kind_w: None,
            policy_kind_b: Vec::new(),
            action: None,
            opp_w: None,
            opp_b: Vec::new(),
            attn: None,
            tstack: None,
        };

        // The policy head is consumed at inference (search priors), so half
        // of it is a trainer/inference mismatch: all-or-nothing.
        let policy_present = ["head_policy.weight", "head_policy.bias"]
            .iter()
            .filter(|n| st.tensor(n).is_ok())
            .count();
        let net = match policy_present {
            2 => {
                let b = get("head_policy.bias")?;
                if b.data.len() != 1 {
                    return Err(NnError::BadTensor(
                        "head_policy.bias",
                        format!("{} elements, want 1", b.data.len()),
                    ));
                }
                PlayNet {
                    policy_w: Some(get("head_policy.weight")?),
                    policy_b: b.data[0],
                    ..net
                }
            }
            0 => net,
            _ => {
                return Err(NnError::BadTensor(
                    "head_policy.*",
                    "one of weight/bias present without the other".into(),
                ));
            }
        };

        // The kind table, same all-or-nothing rule — and only beside the
        // head it adds to.
        let kind_present = ["head_policy_kind.weight", "head_policy_kind.bias"]
            .iter()
            .filter(|n| st.tensor(n).is_ok())
            .count();
        let net = match (kind_present, net.policy_w.is_some()) {
            (2, true) => PlayNet {
                policy_kind_w: Some(get("head_policy_kind.weight")?),
                policy_kind_b: get("head_policy_kind.bias")?.data,
                ..net
            },
            (0, _) => net,
            (2, false) => {
                return Err(NnError::BadTensor("head_policy_kind.*", "present without head_policy.*".into()));
            }
            _ => {
                return Err(NnError::BadTensor(
                    "head_policy_kind.*",
                    "one of weight/bias present without the other".into(),
                ));
            }
        };

        // The action head, same all-or-nothing rule, beside the policy head.
        const ACTION_TENSORS: [&str; 8] = [
            "head_action.obj.weight",
            "head_action.obj.bias",
            "head_action.role.weight",
            "head_action.ctx.weight",
            "head_action.ctx.bias",
            "head_action.kind.weight",
            "head_action.out.weight",
            "head_action.out.bias",
        ];
        let action_present = ACTION_TENSORS.iter().filter(|n| st.tensor(n).is_ok()).count();
        let net = match (action_present, net.policy_w.is_some()) {
            (8, true) => PlayNet {
                action: Some(ActionHead {
                    obj_w: get("head_action.obj.weight")?,
                    obj_b: get("head_action.obj.bias")?.data,
                    role: get("head_action.role.weight")?,
                    ctx_w: get("head_action.ctx.weight")?,
                    ctx_b: get("head_action.ctx.bias")?.data,
                    kind: get("head_action.kind.weight")?,
                    out_w: get("head_action.out.weight")?,
                    out_b: get("head_action.out.bias")?.data.first().copied().unwrap_or(0.0),
                }),
                ..net
            },
            (0, _) => net,
            (8, false) => {
                return Err(NnError::BadTensor("head_action.*", "present without head_policy.*".into()));
            }
            _ => return Err(NnError::BadTensor("head_action.*", "some of its tensors present without the rest".into())),
        };

        // The opponent-hand belief head, same all-or-nothing rule.
        let opp_present = ["head_opp.weight", "head_opp.bias"]
            .iter()
            .filter(|n| st.tensor(n).is_ok())
            .count();
        let net = match opp_present {
            2 => PlayNet {
                opp_w: Some(get("head_opp.weight")?),
                opp_b: get("head_opp.bias")?.data,
                ..net
            },
            0 => net,
            _ => {
                return Err(NnError::BadTensor(
                    "head_opp.*",
                    "one of weight/bias present without the other".into(),
                ));
            }
        };

        // The interaction layer is all-or-nothing: ignoring a partial set
        // would run the wrong architecture on the weights.
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

        // Transformer stack, all-or-nothing per block and across the stack:
        // blocks are counted while `tblocks.{i}.ln1.weight` exists and each
        // must carry its full tensor set.
        let getd = |name: &str| get_tensor_dyn(&st, "tblocks.*", name);
        let n_blocks = count_tblocks(&st);
        let has_group = st.tensor("tblocks.group.weight").is_ok();
        let net = if n_blocks > 0 || has_group {
            if net.attn.is_some() {
                return Err(NnError::BadTensor(
                    "tblocks.*",
                    "both attn.* and tblocks.* present — pick one architecture".into(),
                ));
            }
            if n_blocks == 0 || !has_group {
                return Err(NnError::BadTensor(
                    "tblocks.*",
                    "group tag and blocks must both be present".into(),
                ));
            }
            let blocks = load_tblocks(&st, n_blocks)?;
            PlayNet {
                tstack: Some(TStack { group: getd("tblocks.group.weight")?, blocks }),
                ..net
            }
        } else {
            net
        };

        // Widen older-generation checkpoints (see [`LEGACY_FEATS`]). Both
        // pads or neither: matching only one count fails the shape check.
        let mut net = net;
        for (obj, global) in LEGACY_FEATS {
            if net.obj_w.cols == net.emb.cols + obj
                && net.trunk1_w.cols == NUM_GROUPS * 2 * net.obj_w.rows + global
            {
                net.obj_w.pad_cols(net.emb.cols + OBJ_FEATS);
                net.trunk1_w.pad_cols(NUM_GROUPS * 2 * net.obj_w.rows + GLOBAL_FEATS);
                break;
            }
        }

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
            (
                "head_policy.weight",
                net.policy_w
                    .as_ref()
                    .is_none_or(|w| w.cols == net.trunk2_w.rows && w.rows == 1),
            ),
            (
                "head_policy_kind.weight",
                net.policy_kind_w.as_ref().is_none_or(|w| {
                    w.cols == net.trunk2_w.rows && w.rows == POLICY_KINDS && net.policy_kind_b.len() == POLICY_KINDS
                }),
            ),
            (
                "head_action.*",
                net.action.as_ref().is_none_or(|a| {
                    let w = a.obj_w.rows;
                    a.obj_w.cols == net.obj_w.rows
                        && a.obj_b.len() == w
                        && a.role.rows == ACTION_ROLES
                        && a.role.cols == w
                        && a.ctx_w.rows == w
                        && a.ctx_w.cols == net.trunk2_w.rows
                        && a.ctx_b.len() == w
                        && a.kind.rows == POLICY_KINDS
                        && a.kind.cols == w
                        && a.out_w.rows == 1
                        && a.out_w.cols == w
                }),
            ),
            (
                "head_opp.weight",
                net.opp_w.as_ref().is_none_or(|w| {
                    w.cols == net.trunk2_w.rows && w.rows == net.emb.rows && net.opp_b.len() == w.rows
                }),
            ),
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
        if let Some(ts) = &net.tstack {
            if h_obj % ATTN_HEADS != 0 {
                return Err(NnError::BadTensor(
                    "tblocks.*",
                    format!("obj_hidden {h_obj} not divisible by {ATTN_HEADS} heads"),
                ));
            }
            if ts.group.rows != NUM_GROUPS || ts.group.cols != h_obj {
                return Err(NnError::BadTensor(
                    "tblocks.*",
                    format!("group tag {}x{} != {NUM_GROUPS}x{h_obj}", ts.group.rows, ts.group.cols),
                ));
            }
            for (i, blk) in ts.blocks.iter().enumerate() {
                if !blk.shapes_fit(h_obj) {
                    return Err(NnError::BadTensor(
                        "tblocks.*",
                        format!("block {i} shapes inconsistent with obj_hidden {h_obj}"),
                    ));
                }
            }
        }
        Ok(net)
    }

    /// Per-object hidden vectors for every object on the board, flattened
    /// across groups: `(hs [n, h_obj], group index per object)`.
    #[inline(always)]
    fn encode_objects(&self, s: &EncodedState, h_obj: usize) -> (Vec<f32>, Vec<usize>) {
        let emb_dim = self.emb.cols;
        let k = self.obj_w.cols;
        let n: usize = s.len();
        // Pack every object's input row, then run the object layer over all
        // of them as one matrix product.
        let mut x = vec![0.0f32; n * k];
        let mut owner = Vec::with_capacity(n);
        let mut i = 0;
        for (gi, group) in s.groups().enumerate() {
            for o in group {
                // See `DeckNet::forward`: out of range is the unknown slot,
                // not the last row.
                let card =
                    if (o.card as usize) < self.emb.rows { o.card as usize } else { 0 };
                let row = &mut x[i * k..(i + 1) * k];
                row[..emb_dim].copy_from_slice(&self.emb.data[card * emb_dim..(card + 1) * emb_dim]);
                row[emb_dim..].copy_from_slice(&o.feats);
                owner.push(gi);
                i += 1;
            }
        }
        let mut hs = vec![0.0f32; n * h_obj];
        self.obj_w.matmul_t(&x, n, &mut hs);
        for row in hs.chunks_exact_mut(h_obj) {
            for (v, b) in row.iter_mut().zip(&self.obj_b) {
                *v = (*v + b).max(0.0);
            }
        }
        (hs, owner)
    }

    /// Multi-head self-attention over all objects, in place, with a
    /// residual and a relu.
    ///
    /// Every object attends to every other regardless of zone; a learned
    /// per-group tag on the input tells a query whose object it sees.
    ///
    /// Deliberately one layer, no layer norm. The full pre-norm stack is
    /// [`Self::transform`] (`tblocks.*`); old `attn.*` checkpoints keep
    /// loading through this path unchanged.
    #[inline(always)]
    fn attend(&self, hs: &mut [f32], owner: &[usize], h_obj: usize) {
        let Some(a) = &self.attn else { return };
        let n = owner.len();
        if n == 0 {
            return;
        }

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
            w.matmul_t(&xin, n, &mut out);
            for row in out.chunks_exact_mut(h_obj) {
                for (v, bb) in row.iter_mut().zip(b) {
                    *v += bb;
                }
            }
            out
        };
        let q = project(&a.q_w, &a.q_b);
        let k = project(&a.k_w, &a.k_b);
        let v = project(&a.v_w, &a.v_b);
        let mut ctx = vec![0.0f32; n * h_obj];
        attention_core(&q, &k, &v, n, h_obj, &mut ctx);

        // Output projection, residual against the pre-attention hidden
        // state, relu.
        let mut proj = vec![0.0f32; n * h_obj];
        a.o_w.matmul_t(&ctx, n, &mut proj);
        for i in 0..n {
            for j in 0..h_obj {
                hs[i * h_obj + j] = (hs[i * h_obj + j] + proj[i * h_obj + j] + a.o_b[j]).max(0.0);
            }
        }
    }

    /// The transformer stack: group tag once into the stream, then
    /// pre-LN blocks in place. Attention math matches [`Self::attend`];
    /// the differences are the normed input, no relu on the stream, and
    /// the FFN half-block.
    #[inline(always)]
    fn transform(&self, hs: &mut [f32], owner: &[usize], h_obj: usize) {
        let Some(ts) = &self.tstack else { return };
        let n = owner.len();
        if n == 0 {
            return;
        }

        for i in 0..n {
            let tag = &ts.group.data[owner[i] * h_obj..(owner[i] + 1) * h_obj];
            for j in 0..h_obj {
                hs[i * h_obj + j] += tag[j];
            }
        }

        run_tblocks(&ts.blocks, hs, n, h_obj);
    }

    /// Win probability for the seat the state was encoded for, in [0, 1].
    pub fn forward(&self, s: &EncodedState) -> f32 {
        let t2 = self.trunk_out(s);
        let mut logit = [0.0f32];
        self.head_win_w.matvec(&t2, &mut logit);
        let z = logit[0] + self.head_win_b[0];
        1.0 / (1.0 + (-z).exp())
    }

    /// The policy head's raw logit for the seat the state was encoded for
    /// — the search-prior score over a candidate's *successor* state.
    /// `None` when the net carries no policy head (every pre-round-37
    /// checkpoint), so callers fall back rather than reading garbage.
    ///
    /// A logit, not a probability: the softmax over a decision's
    /// candidates belongs to the caller who has the set.
    pub fn forward_policy(&self, s: &EncodedState) -> Option<f32> {
        let w = self.policy_w.as_ref()?;
        let t2 = self.trunk_out(s);
        let mut out = [0.0f32];
        w.matvec(&t2, &mut out);
        Some(out[0] + self.policy_b)
    }

    /// [`forward_policy`](Self::forward_policy) for a decision of kind
    /// `kind` (`< POLICY_KINDS`): the shared logit plus the kind table's
    /// row, when the net carries the table; the shared logit alone when it
    /// doesn't, or for an out-of-range kind.
    pub fn forward_policy_kind(&self, s: &EncodedState, kind: usize) -> Option<f32> {
        let w = self.policy_w.as_ref()?;
        let t2 = self.trunk_out(s);
        let mut out = [0.0f32];
        w.matvec(&t2, &mut out);
        let mut z = out[0] + self.policy_b;
        if let Some(kw) = &self.policy_kind_w
            && kind < kw.rows
        {
            let row = &kw.data[kind * kw.cols..(kind + 1) * kw.cols];
            z += row.iter().zip(&t2).map(|(a, b)| a * b).sum::<f32>() + self.policy_kind_b[kind];
        }
        Some(z)
    }

    pub fn has_policy_head(&self) -> bool {
        self.policy_w.is_some()
    }

    pub fn has_policy_kinds(&self) -> bool {
        self.policy_kind_w.is_some()
    }

    pub fn has_action_head(&self) -> bool {
        self.action.is_some()
    }

    /// The action head's term for each of a decision's moves (`cands[c]`
    /// the references of move `c`, first [`ACTION_MAX_REFS`] read), from
    /// one trunk pass over the decision state `root`. Added to each move's
    /// [`forward_policy_kind`](Self::forward_policy_kind) logit over its
    /// successor. `None` without the head. A reference past the state's
    /// objects reads as [`NO_OBJECT`]; a kind past the table, row 0.
    pub fn forward_action_terms(&self, root: &EncodedState, kind: usize, cands: &[Vec<ActionRef>]) -> Option<Vec<f32>> {
        let a = self.action.as_ref()?;
        let (t2, hs) = self.trunk_core(root);
        let h_obj = self.obj_w.rows;
        let n_obj = hs.len() / h_obj.max(1);
        let w = a.obj_w.rows;
        let k = if kind < POLICY_KINDS { kind } else { 0 };
        let mut base = vec![0.0f32; w];
        a.ctx_w.matvec(&t2, &mut base);
        for ((b, cb), kb) in base.iter_mut().zip(&a.ctx_b).zip(&a.kind.data[k * w..(k + 1) * w]) {
            *b += cb + kb;
        }
        let zero = vec![0.0f32; h_obj];
        let mut proj = vec![0.0f32; w];
        let mut out = Vec::with_capacity(cands.len());
        for refs in cands {
            let mut z = base.clone();
            for &(role, obj) in refs.iter().take(ACTION_MAX_REFS) {
                let h = if (obj as usize) < n_obj { &hs[obj as usize * h_obj..(obj as usize + 1) * h_obj] } else { &zero[..] };
                a.obj_w.matvec(h, &mut proj);
                let r = (role as usize).min(ACTION_ROLES - 1);
                let role_row = &a.role.data[r * w..(r + 1) * w];
                for (((zj, p), ob), rb) in z.iter_mut().zip(&proj).zip(&a.obj_b).zip(role_row) {
                    *zj += (p + ob + rb).max(0.0);
                }
            }
            out.push(a.out_b + a.out_w.data.iter().zip(&z).map(|(o, zj)| o * zj.max(0.0)).sum::<f32>());
        }
        Some(out)
    }

    /// Per-name probabilities that the *opponent* currently holds each
    /// vocabulary card, `None` when the net carries no belief head.
    /// Index 0 (unknown) is emitted but meaningless — the sampler treats
    /// out-of-vocab cards as neutral.
    pub fn forward_opp_hand(&self, s: &EncodedState) -> Option<Vec<f32>> {
        let w = self.opp_w.as_ref()?;
        let t2 = self.trunk_out(s);
        let mut out = vec![0.0f32; w.rows];
        w.matvec(&t2, &mut out);
        for (v, b) in out.iter_mut().zip(&self.opp_b) {
            *v = 1.0 / (1.0 + (-(*v + b)).exp());
        }
        Some(out)
    }

    /// The architecture the file carries — `(emb_dim, obj_hidden, h1, h2,
    /// attn, transformer blocks)` — for callers rebuilding this net in
    /// another framework (the batched eval server). Follow the file, never
    /// the run's flags: a wide-learner run piloted by a standard-width
    /// champion is normal (round 45's first abort).
    pub fn arch(&self) -> (usize, usize, usize, usize, bool, usize) {
        (
            self.emb.cols,
            self.obj_w.rows,
            self.trunk1_w.rows,
            self.trunk2_w.rows,
            self.attn.is_some(),
            self.tstack.as_ref().map_or(0, |t| t.blocks.len()),
        )
    }

    pub fn has_opp_head(&self) -> bool {
        self.opp_w.is_some()
    }

    /// The shared trunk: encode, interact, pool, two relu layers. Both
    /// heads read the returned activations.
    fn trunk_out(&self, s: &EncodedState) -> Vec<f32> {
        #[cfg(target_arch = "x86_64")]
        if avx2_fma() {
            // SAFETY: the detected features gate the call.
            return unsafe { self.trunk_out_avx2(s) };
        }
        self.trunk_out_body(s)
    }

    /// [`Self::trunk_out_body`] compiled with AVX2: the loops it inlines —
    /// the softmax, the pooling, the bias and relu passes — get 256-bit
    /// lanes. The matrix products themselves are the out-of-line
    /// [`kernel`] functions, which pick their own AVX2 build.
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn trunk_out_avx2(&self, s: &EncodedState) -> Vec<f32> {
        self.trunk_out_body(s)
    }

    #[inline(always)]
    fn trunk_out_body(&self, s: &EncodedState) -> Vec<f32> {
        self.trunk_core(s).0
    }

    /// The trunk's output and, beside it, every object's embedding after
    /// the interaction layers (`[n, h_obj]`, flat, the pooling's input) —
    /// what the action head reads its moves' objects from.
    #[inline(always)]
    fn trunk_core(&self, s: &EncodedState) -> (Vec<f32>, Vec<f32>) {
        let h_obj = self.obj_w.rows;
        let mut trunk_in = vec![0.0f32; self.trunk1_w.cols];

        // Encode every object, let them interact (no-op without the
        // attention tensors), then pool per group into the fixed-width
        // trunk input.
        let (mut hs, owner) = self.encode_objects(s, h_obj);
        self.attend(&mut hs, &owner, h_obj);
        self.transform(&mut hs, &owner, h_obj);

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
        self.trunk1_w.matvec_t(&trunk_in, &mut t1);
        for (v, b) in t1.iter_mut().zip(&self.trunk1_b) {
            *v = (*v + b).max(0.0);
        }
        let mut t2 = vec![0.0f32; self.trunk2_w.rows];
        self.trunk2_w.matvec_t(&t1, &mut t2);
        for (v, b) in t2.iter_mut().zip(&self.trunk2_b) {
            *v = (*v + b).max(0.0);
        }
        (t2, hs)
    }
}

/// Anything that can turn an encoded state into a win probability.
///
/// The engine's registry (`net_eval`) stores these rather than [`PlayNet`]
/// so the training harness can substitute a batched GPU evaluator without
/// the engine growing an ML dependency. Takes the state by value because a
/// remote implementation moves it into a queue.
pub trait NetEvaluator: Send + Sync {
    /// Win probability for the seat the state was encoded for, in [0, 1].
    fn eval(&self, s: EncodedState) -> f32;

    /// The policy head's logit for the state, or `None` without a policy
    /// head (the default; the search falls back to heuristic priors).
    fn eval_policy(&self, s: EncodedState) -> Option<f32> {
        let _ = s;
        None
    }

    /// [`eval_policy`](Self::eval_policy) for a decision of kind `kind`
    /// (see [`POLICY_KINDS`]). Default: the kind-free logit, which is
    /// exact for a net without the kind table.
    fn eval_policy_kind(&self, s: EncodedState, kind: u8) -> Option<f32> {
        let _ = kind;
        self.eval_policy(s)
    }

    /// The action head's term per move — see
    /// [`PlayNet::forward_action_terms`]. Default `None`: no head, and the
    /// policy is the successor logit alone.
    fn eval_action_terms(&self, root: EncodedState, kind: u8, cands: &[Vec<ActionRef>]) -> Option<Vec<f32>> {
        let _ = (root, kind, cands);
        None
    }

    /// Whether [`eval_action_terms`](Self::eval_action_terms) can answer.
    fn has_action(&self) -> bool {
        false
    }

    /// Whether [`eval_policy`](Self::eval_policy) can answer, so a profile
    /// can report its prior up front.
    fn has_policy(&self) -> bool {
        false
    }

    /// Per-name opponent-hand probabilities from the belief head, `None`
    /// when the evaluator's net has no such head. Default `None`; the
    /// determinizer falls back to the uniform redeal.
    fn eval_opp_hand(&self, s: EncodedState) -> Option<Vec<f32>> {
        let _ = s;
        None
    }

    /// Whether [`eval_opp_hand`](Self::eval_opp_hand) can answer at all.
    fn has_opp(&self) -> bool {
        false
    }
}

impl NetEvaluator for PlayNet {
    fn eval(&self, s: EncodedState) -> f32 {
        self.forward(&s)
    }

    fn eval_policy(&self, s: EncodedState) -> Option<f32> {
        self.forward_policy(&s)
    }

    fn eval_policy_kind(&self, s: EncodedState, kind: u8) -> Option<f32> {
        self.forward_policy_kind(&s, kind as usize)
    }

    fn eval_action_terms(&self, root: EncodedState, kind: u8, cands: &[Vec<ActionRef>]) -> Option<Vec<f32>> {
        self.forward_action_terms(&root, kind as usize, cands)
    }

    fn has_action(&self) -> bool {
        self.has_action_head()
    }

    fn has_policy(&self) -> bool {
        self.has_policy_head()
    }

    fn eval_opp_hand(&self, s: EncodedState) -> Option<Vec<f32>> {
        self.forward_opp_hand(&s)
    }

    fn has_opp(&self) -> bool {
        self.has_opp_head()
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
/// correct on little-endian targets (all of ours); a big-endian build fails
/// to compile rather than writing byte-swapped weights.
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
        state.push(G_BF_SELF, EncodedObject { card: 7, feats });
        state.push(G_GY_OPP, EncodedObject { card: 0, feats: [0.25; OBJ_FEATS] });
        TrainRow {
            state,
            win: 1.0,
            life_diff: 0.55,
            game_len: 0.2,
            traj: 42,
            ply: 3,
            aux: [0.1, -0.2, 0.3, 0.6],
            opp_hand: vec![2, 5, 5],
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
        s.push(G_BF_SELF, EncodedObject { card: 1, feats });
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
        s.push(G_BF_SELF, EncodedObject { card: 0, feats: feats2 });
        let want2 = 1.0 / (1.0 + (-4.75f32).exp());
        let got2 = net.forward(&s);
        assert!((got2 - want2).abs() < 1e-6, "got {got2}, want {want2}");
    }

    /// Padding the vocabulary preserves every prediction the net could
    /// already make, and a card the net never saw embeds as the unknown
    /// slot does — zeros.
    ///
    /// Loadability half of the defect that killed the old deck nets; the
    /// soundness half is `server::vocab_snapshot`'s frozen indices.
    #[test]
    fn padding_the_vocabulary_leaves_the_old_rows_alone() {
        let h_obj = 1usize;
        let trunk_in = NUM_GROUPS * 2 * h_obj + GLOBAL_FEATS;
        let mut obj_w = vec![0.0f32; 1 + OBJ_FEATS];
        obj_w[0] = 1.0;
        obj_w[1] = 1.0;
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
        let mut net = PlayNet::load(&bytes).expect("loads");

        let mut feats = [0.0; OBJ_FEATS];
        feats[0] = 0.5;
        let mut s = EncodedState::default();
        s.push(G_BF_SELF, EncodedObject { card: 1, feats });
        s.global[3] = 0.25;
        let before = net.forward(&s);

        net.pad_vocab(5).expect("widening is allowed");
        assert_eq!(net.vocab_size(), 5);
        assert!((net.forward(&s) - before).abs() < 1e-9, "a known card moved");

        // Card 4 is one of the new rows: zero embedding, so it scores
        // exactly as card 0 (the reserved unknown slot) does.
        let mut unknown = EncodedState::default();
        unknown.global[3] = 0.25;
        unknown.push(G_BF_SELF, EncodedObject { card: 4, feats });
        let mut slot_zero = EncodedState::default();
        slot_zero.global[3] = 0.25;
        slot_zero.push(G_BF_SELF, EncodedObject { card: 0, feats });
        assert!((net.forward(&unknown) - net.forward(&slot_zero)).abs() < 1e-9);

        // An index past the table is the unknown slot too, not the last
        // row — the clamp used to hand it some unrelated card's embedding.
        let mut past = EncodedState::default();
        past.global[3] = 0.25;
        past.push(G_BF_SELF, EncodedObject { card: 99, feats });
        assert!((net.forward(&past) - net.forward(&slot_zero)).abs() < 1e-9);

        // Narrowing is refused: the encoder would have no index for the
        // rows the net carries.
        assert!(net.pad_vocab(3).is_err());
    }

    /// The kind table adds its row to the shared logit, loads only beside
    /// the head, and leaves every kind-free answer as it was.
    #[test]
    fn policy_kind_table_adds_its_row_and_loads_only_with_the_head() {
        let h_obj = 1usize;
        let trunk_in = NUM_GROUPS * 2 * h_obj + GLOBAL_FEATS;
        let mut obj_w = vec![0.0f32; 1 + OBJ_FEATS];
        obj_w[0] = 1.0;
        obj_w[1] = 1.0;
        let base: Vec<(&str, Vec<usize>, Vec<f32>)> = vec![
            ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
            ("obj.weight", vec![1, 1 + OBJ_FEATS], obj_w),
            ("obj.bias", vec![1], vec![0.0]),
            ("trunk1.weight", vec![1, trunk_in], vec![1.0; trunk_in]),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![1.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            ("head_win.weight", vec![1, 1], vec![1.0]),
            ("head_win.bias", vec![1], vec![0.0]),
        ];
        let kw: Vec<f32> = (0..POLICY_KINDS).map(|k| k as f32).collect();
        let kb: Vec<f32> = (0..POLICY_KINDS).map(|k| -(k as f32) * 0.5).collect();
        let mut full = base.clone();
        full.push(("head_policy.weight", vec![1, 1], vec![3.0]));
        full.push(("head_policy.bias", vec![1], vec![-1.0]));
        full.push(("head_policy_kind.weight", vec![POLICY_KINDS, 1], kw.clone()));
        full.push(("head_policy_kind.bias", vec![POLICY_KINDS], kb.clone()));
        let net = PlayNet::load(&to_safetensors(&full)).expect("kind net loads");
        assert!(net.has_policy_kinds());
        // t2 = 5.25 (as in `forward_matches_hand_computation`); shared 14.75.
        let mut feats = [0.0; OBJ_FEATS];
        feats[0] = 0.5;
        let mut s = EncodedState::default();
        s.push(G_BF_SELF, EncodedObject { card: 1, feats });
        s.global[3] = 0.25;
        assert!((net.forward_policy(&s).unwrap() - 14.75).abs() < 1e-5);
        for k in [0usize, 3, 31] {
            let want = 14.75 + k as f32 * 5.25 - k as f32 * 0.5;
            let got = net.forward_policy_kind(&s, k).unwrap();
            assert!((got - want).abs() < 1e-4, "kind {k}: {got} vs {want}");
        }
        assert!((net.forward_policy_kind(&s, POLICY_KINDS).unwrap() - 14.75).abs() < 1e-5, "out of range reads the shared head");

        // Without the table every kind is the shared head.
        let mut head_only = base.clone();
        head_only.push(("head_policy.weight", vec![1, 1], vec![3.0]));
        head_only.push(("head_policy.bias", vec![1], vec![-1.0]));
        let plain = PlayNet::load(&to_safetensors(&head_only)).unwrap();
        assert!(!plain.has_policy_kinds());
        assert!((plain.forward_policy_kind(&s, 5).unwrap() - 14.75).abs() < 1e-5);

        // The table without the head, or half a table, is a mismatch.
        let mut orphan = base.clone();
        orphan.push(("head_policy_kind.weight", vec![POLICY_KINDS, 1], kw.clone()));
        orphan.push(("head_policy_kind.bias", vec![POLICY_KINDS], kb));
        assert!(PlayNet::load(&to_safetensors(&orphan)).is_err());
        let mut half = head_only.clone();
        half.push(("head_policy_kind.weight", vec![POLICY_KINDS, 1], kw));
        assert!(PlayNet::load(&to_safetensors(&half)).is_err());
    }

    /// The policy head reads the same trunk as the win head and answers
    /// only when its tensors are present; half a head is rejected (it is
    /// consumed at inference, unlike the aux heads).
    #[test]
    fn policy_head_loads_all_or_nothing_and_shares_the_trunk() {
        let h_obj = 1usize;
        let trunk_in = NUM_GROUPS * 2 * h_obj + GLOBAL_FEATS;
        let mut obj_w = vec![0.0f32; 1 + OBJ_FEATS];
        obj_w[0] = 1.0;
        obj_w[1] = 1.0;
        let base: Vec<(&str, Vec<usize>, Vec<f32>)> = vec![
            ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
            ("obj.weight", vec![1, 1 + OBJ_FEATS], obj_w),
            ("obj.bias", vec![1], vec![0.0]),
            ("trunk1.weight", vec![1, trunk_in], vec![1.0; trunk_in]),
            ("trunk1.bias", vec![1], vec![0.0]),
            ("trunk2.weight", vec![1, 1], vec![1.0]),
            ("trunk2.bias", vec![1], vec![0.0]),
            ("head_win.weight", vec![1, 1], vec![1.0]),
            ("head_win.bias", vec![1], vec![0.0]),
        ];

        // Without the head: no policy answer, and that is a valid net.
        let plain = PlayNet::load(&to_safetensors(&base)).expect("headless net loads");
        assert!(!plain.has_policy_head());

        let mut with_head = base.clone();
        with_head.push(("head_policy.weight", vec![1, 1], vec![3.0]));
        with_head.push(("head_policy.bias", vec![1], vec![-1.0]));
        let net = PlayNet::load(&to_safetensors(&with_head)).expect("policy net loads");
        assert!(net.has_policy_head());

        // Same state as `forward_matches_hand_computation`: t2 = 5.25.
        let mut feats = [0.0; OBJ_FEATS];
        feats[0] = 0.5;
        let mut s = EncodedState::default();
        s.push(G_BF_SELF, EncodedObject { card: 1, feats });
        s.global[3] = 0.25;
        // Win head is untouched by the extra tensors...
        let want_win = 1.0 / (1.0 + (-5.25f32).exp());
        assert!((net.forward(&s) - want_win).abs() < 1e-6);
        assert!((plain.forward(&s) - want_win).abs() < 1e-6);
        // ...and the policy logit is w·t2 + b = 3·5.25 − 1 = 14.75.
        let got = net.forward_policy(&s).expect("head answers");
        assert!((got - 14.75).abs() < 1e-5, "got {got}");
        assert_eq!(plain.forward_policy(&s), None);

        // Half a head is a mismatch, not a version.
        let mut partial = base.clone();
        partial.push(("head_policy.weight", vec![1, 1], vec![3.0]));
        assert!(PlayNet::load(&to_safetensors(&partial)).is_err(), "partial head must not load");
        // A head sized to the wrong trunk is rejected too.
        let mut wrong = base;
        wrong.push(("head_policy.weight", vec![1, 2], vec![3.0, 3.0]));
        wrong.push(("head_policy.bias", vec![1], vec![0.0]));
        assert!(PlayNet::load(&to_safetensors(&wrong)).is_err(), "mis-sized head must not load");
    }

    /// A checkpoint from every earlier encoder generation loads by
    /// zero-padding and cannot see newer features: a state with every
    /// newer slot lit scores exactly the legacy arithmetic. Keeps the
    /// champion and golden traces fixed across encoder bumps.
    #[test]
    fn legacy_checkpoints_load_zero_padded() {
        for (legacy_obj, legacy_global) in LEGACY_FEATS {
            let legacy_trunk_in = NUM_GROUPS * 2 + legacy_global;
            let mut obj_w = vec![0.0f32; 1 + legacy_obj];
            obj_w[0] = 1.0;
            obj_w[1] = 1.0;
            let bytes = to_safetensors(&[
                ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
                ("obj.weight", vec![1, 1 + legacy_obj], obj_w),
                ("obj.bias", vec![1], vec![0.0]),
                ("trunk1.weight", vec![1, legacy_trunk_in], vec![1.0; legacy_trunk_in]),
                ("trunk1.bias", vec![1], vec![0.0]),
                ("trunk2.weight", vec![1, 1], vec![1.0]),
                ("trunk2.bias", vec![1], vec![0.0]),
                ("head_win.weight", vec![1, 1], vec![1.0]),
                ("head_win.bias", vec![1], vec![0.0]),
            ]);
            let net = PlayNet::load(&bytes).expect("legacy shapes load");

            // Same state as `forward_matches_hand_computation`, plus every
            // feature slot this generation never saw set to something loud.
            let mut feats = [0.0; OBJ_FEATS];
            feats[0] = 0.5;
            for f in feats.iter_mut().skip(legacy_obj) {
                *f = 100.0;
            }
            let mut s = EncodedState::default();
            s.push(G_BF_SELF, EncodedObject { card: 1, feats });
            s.global[3] = 0.25;
            for gl in s.global.iter_mut().skip(legacy_global) {
                *gl = 100.0;
            }
            let want = 1.0 / (1.0 + (-5.25f32).exp());
            let got = net.forward(&s);
            assert!((got - want).abs() < 1e-6, "gen {legacy_obj}: got {got}, want {want}");

            // Half-legacy shapes are a corrupt file, not a version: rejected.
            let mut obj_w = vec![0.0f32; 1 + legacy_obj];
            obj_w[0] = 1.0;
            let trunk_in = NUM_GROUPS * 2 + GLOBAL_FEATS;
            let mixed = to_safetensors(&[
                ("emb.weight", vec![2, 1], vec![0.0, 2.0]),
                ("obj.weight", vec![1, 1 + legacy_obj], obj_w),
                ("obj.bias", vec![1], vec![0.0]),
                ("trunk1.weight", vec![1, trunk_in], vec![1.0; trunk_in]),
                ("trunk1.bias", vec![1], vec![0.0]),
                ("trunk2.weight", vec![1, 1], vec![1.0]),
                ("trunk2.bias", vec![1], vec![0.0]),
                ("head_win.weight", vec![1, 1], vec![1.0]),
                ("head_win.bias", vec![1], vec![0.0]),
            ]);
            assert!(PlayNet::load(&mixed).is_err(), "gen {legacy_obj}: mixed shapes must not load");
        }
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

