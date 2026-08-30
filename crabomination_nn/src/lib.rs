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
//! * Optionally, one of two pre-pool interaction architectures, chosen by
//!   which tensors the file carries: a single tagged attention layer
//!   (`attn.*`, see [`ATTN_HEADS`]) or a stack of pre-LN transformer
//!   blocks (`tblocks.group.weight` + `tblocks.{i}.{ln1,ln2}.*`,
//!   `tblocks.{i}.attn.{q,k,v,o}.*`, `tblocks.{i}.{ffn1,ffn2}.*`). A file
//!   carrying both, or a partial set of either, is rejected.
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
///
/// Feats 37..=44 are the round-28 (v6) block pair. 37..=39 is combat
/// structure: the round-12 flags said a creature *is* blocked, not by
/// what, so a 2/2 chump and a 5/5 trade encoded identically:
///
/// * 37/38 counterpart power / effective-toughness sums, / 8 — on a
///   blocked attacker, its blockers summed; on a blocker, the attackers
///   it blocks summed. An object is never both in one combat.
/// * 39 `attacking_non_player` — the attack targets a planeswalker or
///   battle, not a life total.
/// * 40..=44 keyword classes invisible to feats 12..=19: haste,
///   hard-to-target (ward/hexproof/shroud/protection), indestructible,
///   hard-to-block (unblockable/shadow/fear/intimidate/skulk/
///   horsemanship/landwalk), defender. Mostly redundant with the card
///   embedding for in-vocab cards; load-bearing for tokens and grants.
///
/// Feats 45..=52 are the round-40 (v7) pair of blocks. Both carry
/// information the earlier features could not express at all, as
/// opposed to re-describing the visible board:
///
/// * 45..=47 the *expiry* block. 45 damage marked, / 8 — feature 5 is
///   `toughness − damage`, so a 4/4 with three damage and a 4/1 encoded
///   identically, though the first is whole again at cleanup. 46/47 the
///   power / toughness delta that expires at cleanup
///   (`CardInstance::power_bonus`, from until-end-of-turn pumps), / 4
///   and signed: a 5/5 off a combat trick encoded as a printed 5/5.
/// * 48..=52 the *counter* block. Feature 34 sums every counter that
///   isn't loyalty or prepared into one scalar, so three +1/+1, three
///   stun and three Page counters all read 0.75. These split out the
///   kinds a value function trades on differently: 48 +1/+1 / 4,
///   49 −1/−1 / 4, 50 stun / 2 (the bearer's next untap is skipped),
///   51 Page / 3, 52 Growth / 3. +1/+1 and −1/−1 reach the net through
///   effective P/T as well; the rest had no path to it at all.
///
/// Feats 53..=58 are the v8 block (2026-08-30, modern precondition 3),
/// gated by the `v8` ablation bit:
///
/// * 53 artifact / 54 enchantment type bits — printed off the
///   battlefield, layer-computed on it. The embedding carries the type
///   for in-vocab cards, but tokens and every off-vocab card fell back
///   to three type flags that could not tell a Chalice from a Signet
///   from a Saga: "not a creature, not a land, not a planeswalker" was
///   one class holding most of a modern board's non-creature permanents.
/// * 55..=58 the modern counter kinds the v7 split has no slot for:
///   55 Lore / 3 (a saga's chapter IS its state), 56 Charge / 4
///   (Chalice on 1 vs 3 are different cards), 57 Shield / 2,
///   58 Finality / 2. All previously folded into feature 34's
///   undifferentiated sum.
pub const OBJ_FEATS: usize = 59;
/// Global scalar feature count. Baked into encoded rows likewise.
///
/// Globals 36..=42 are round 28: fine combat phase one-hots (36
/// declare-attackers, 37 declare-blockers, 38 damage/end-combat — the
/// original coarse slot 11 collapsed "blocks pending" and "damage
/// dealt", which are the states the combat sims evaluate most), 39/40
/// unblocked attacker power aimed at self/opp, / 12, and 41/42 exile
/// sizes self/opp, / 10.
///
/// Globals 43..=54 are the round-40 (v7) *history* block: six
/// turn-scoped counters, self then opponent, reset at each turn
/// boundary. Every other global is a static snapshot, which made the
/// encoding a pure function of the current position — and a card whose
/// text reads what happened this turn was not merely under-described,
/// the net could not tell whether its ability was live. The six are the
/// ones SoS actually reads: 43/44 life gained (infusion, Foolish Fate),
/// / 5; 45/46 instants and sorceries cast (Burrog Barrage), / 3; 47/48
/// spells cast (storm), / 4; 49/50 creatures died, / 3; 51/52 cards
/// that left the graveyard, / 3; 53/54 cards exiled, / 3.
///
/// Globals 55/56 are the v8 block: land drops remaining this turn, self
/// then opponent, / 2 (Azusa-class effects are the headroom). "Can I
/// still land-and-spell this turn" is the most common main-phase
/// sequencing read, and until now feature 26's next-turn castability
/// simply assumed the drop was available.
pub const GLOBAL_FEATS: usize = 57;

/// Feature counts of earlier encoder generations, oldest first: v5
/// (pre-round-28), v6 (round 28 through 39), and v7 (round 40 through
/// the 2026-08-30 v8 block). Checkpoints trained
/// against one of these have weight matrices sized to it, and
/// [`PlayNet::load`] widens them with zero columns rather than
/// rejecting them, which computes exactly what the old binary computed
/// — the new features multiply into zeros. The champion (and every
/// historical net) stays loadable, and golden traces do not move.
///
/// A file must match one generation on *both* counts. Matching one and
/// not the other is a corrupt file, not a version, and falls through to
/// the shape check below to fail loudly.
///
/// Public because the trainer has to apply the same rule: `--use-best`
/// hands a checkpoint from an earlier generation to the candle model,
/// whose `VarMap::load` is exact-shape, so it pads by this table before
/// setting. The two loaders disagreeing is the failure mode — a pilot
/// that loads in the engine and not in the trainer, mid-run.
pub const LEGACY_FEATS: [(usize, usize); 3] = [(37, 36), (45, 43), (53, 55)];

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
///
/// The objects of all [`NUM_GROUPS`] groups live in **one** buffer, laid
/// out group by group in index order, with each group's length in
/// `counts`. Eight separate `Vec`s cost 5.25 allocations per encoded
/// state (66,485 `reserve` growths over 12,660 states, the actor's
/// second-largest growth row — PERF `(-107)`); one buffer with one
/// up-front reserve costs one. The emitted *values* are unchanged, so
/// shards and trained nets are unaffected.
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
    /// The *opponent's* held card names at this snapshot, as vocab
    /// indices (round 39). Ground truth the encoder deliberately never
    /// sees — the recorder can look, the input cannot — so a belief
    /// head can be trained to predict it from observable state alone.
    /// Unknown-name cards (index 0) are skipped: a name the vocabulary
    /// cannot express is not a learnable prediction target. Duplicates
    /// appear once per copy held.
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

    /// Grow the embedding table to `new_vocab` rows, zero-filling the ones
    /// this net never saw.
    ///
    /// Sound because card names own a frozen embedding index
    /// (`server::vocab_snapshot`): a vocabulary only ever grows at the end,
    /// so every row this net has still means the card it was trained on.
    /// Before that guarantee existed, indices were the sorted pool's order
    /// and one added card shifted them all — which is why the seven
    /// committed deck nets are unloadable rather than merely short.
    /// Narrowing is not possible and is rejected.
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

/// Row-major dense matrix (`data[r * cols + c]`).
#[derive(Debug, Clone)]
pub struct Tensor2 {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f32>,
}

/// Eight-accumulator dot product. The single-accumulator loop chains
/// every add through one register, and strict f32 semantics stop LLVM
/// from reassociating that into SIMD lanes — so the naive form runs
/// scalar however wide the machine is. Splitting the accumulation states
/// the reassociation explicitly, which is what lets the autovectorizer
/// use it. `matvec` is essentially the whole forward pass, so this
/// function is the net's throughput.
///
/// `inline(always)` is load-bearing: the AVX2 wrapper below relies on
/// this body inlining into its `#[target_feature]` scope, where LLVM
/// recompiles it with 256-bit lanes and FMA.
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

/// The same body compiled with AVX2 + FMA, picked at runtime. The
/// baseline x86-64 target this workspace builds for is SSE2-only; the
/// boxes it runs on are not, and the difference is 4-wide mul+add
/// against 8-wide FMA. Safety: the caller checks the CPU features.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
unsafe fn dot_avx2(a: &[f32], b: &[f32]) -> f32 {
    dot(a, b)
}

/// Runtime CPU-feature dispatch for [`dot`], the check cached to a bool.
#[inline]
fn dot_dispatch(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    {
        use std::sync::OnceLock;
        static AVX2: OnceLock<bool> = OnceLock::new();
        if *AVX2.get_or_init(|| {
            std::arch::is_x86_feature_detected!("avx2")
                && std::arch::is_x86_feature_detected!("fma")
        }) {
            // SAFETY: the detected features gate the call.
            return unsafe { dot_avx2(a, b) };
        }
    }
    dot(a, b)
}

impl Tensor2 {
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
    /// trunk: pooled groups then globals), so right-padding a legacy
    /// weight matrix makes the new inputs multiply into zeros — the
    /// padded net computes exactly what it did before the feature bump.
    fn pad_cols(&mut self, new_cols: usize) {
        debug_assert!(new_cols >= self.cols);
        let mut data = vec![0.0f32; self.rows * new_cols];
        for r in 0..self.rows {
            data[r * new_cols..r * new_cols + self.cols]
                .copy_from_slice(&self.data[r * self.cols..(r + 1) * self.cols]);
        }
        self.cols = new_cols;
        self.data = data;
    }

    /// Extend to `new_rows` by appending zero rows at the bottom.
    ///
    /// The one caller is the vocabulary: card names own a frozen index
    /// (`server::vocab_snapshot`), so a vocabulary only ever grows at the
    /// end and a net trained against a shorter one is correct on every row
    /// it has. A card it never saw embeds as zeros, which is what index 0 —
    /// the reserved unknown slot — does for an off-set card anyway.
    pub fn pad_rows(&mut self, new_rows: usize) {
        debug_assert!(new_rows >= self.rows);
        self.data.resize(new_rows * self.cols, 0.0);
        self.rows = new_rows;
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
    /// The policy head (`head_policy.*`, [1, h2] + [1]), present only in
    /// weights trained with `--policy-head`. Unlike the life/length heads
    /// this one is *consumed at inference*: its logit over a candidate's
    /// successor state is the search's prior, so it loads all-or-nothing
    /// like the attention tensors rather than being ignored as
    /// training-only. Absent means the net ranks with the win head alone,
    /// which is every checkpoint before round 37.
    policy_w: Option<Tensor2>, // [1, h2]
    policy_b: f32,
    /// The opponent-hand belief head (`head_opp.*`, [vocab, h2] + [vocab]),
    /// round 39. Consumed at inference by belief-weighted determinization
    /// (`determinize_hidden_belief`), so it loads all-or-nothing like the
    /// policy head. Absent means uniform redeals — every checkpoint
    /// before round 39.
    opp_w: Option<Tensor2>, // [vocab, h2]
    opp_b: Vec<f32>,        // [vocab] (empty when absent)
    /// Present only in weights trained with the interaction layer. Absent
    /// means the pure deep-sets net, which stays loadable unchanged — the
    /// two architectures are distinguished by which tensors the file
    /// carries, not by a flag the caller has to remember.
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
/// The group tag is added *into* the residual stream once at stack entry
/// (like a positional embedding) rather than per-layer outside the
/// residual — with normed residual blocks there is no clean "scores only"
/// place for it, and zone identity in the pooled representation is
/// harmless because pooling is per-group anyway.
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

/// Layer norm over one row, eps matching candle's `LayerNormConfig`
/// default.
fn layer_norm_row(x: &[f32], w: &[f32], b: &[f32], out: &mut [f32]) {
    let d = x.len();
    let mean = x.iter().sum::<f32>() / d as f32;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / d as f32;
    let inv = 1.0 / (var + 1e-5).sqrt();
    for i in 0..d {
        out[i] = (x[i] - mean) * inv * w[i] + b[i];
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
            opp_w: None,
            opp_b: Vec::new(),
            attn: None,
            tstack: None,
        };

        // The policy head is all-or-nothing for the same reason the
        // attention tensors are: it is consumed at inference (search
        // priors), so a file carrying half of it is a trainer/inference
        // mismatch, not an ignorable extra.
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

        // Transformer stack, likewise all-or-nothing per block and across
        // the stack: blocks are counted while `tblocks.{i}.ln1.weight`
        // exists, and each counted block must then carry its full tensor
        // set — a gap means a trainer/inference mismatch.
        let getd = |name: &str| -> Result<Tensor2, NnError> {
            match st.tensor(name) {
                Ok(_) => {}
                Err(_) => return Err(NnError::BadTensor("tblocks.*", format!("missing {name}"))),
            }
            let view = st.tensor(name).unwrap();
            if view.dtype() != safetensors::Dtype::F32 {
                return Err(NnError::BadTensor(
                    "tblocks.*",
                    format!("{name}: dtype {:?}, want F32", view.dtype()),
                ));
            }
            let (rows, cols) = match *view.shape() {
                [r, c] => (r, c),
                [n] => (1, n),
                _ => {
                    return Err(NnError::BadTensor(
                        "tblocks.*",
                        format!("{name}: rank {} shape", view.shape().len()),
                    ));
                }
            };
            let data = view
                .data()
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                .collect();
            Ok(Tensor2 { rows, cols, data })
        };
        let mut n_blocks = 0;
        while st.tensor(&format!("tblocks.{n_blocks}.ln1.weight")).is_ok() {
            n_blocks += 1;
        }
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
            let mut blocks = Vec::with_capacity(n_blocks);
            for i in 0..n_blocks {
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
            PlayNet {
                tstack: Some(TStack { group: getd("tblocks.group.weight")?, blocks }),
                ..net
            }
        } else {
            net
        };

        // Older-generation checkpoints are widened with zero columns
        // rather than rejected — see [`LEGACY_FEATS`]. Both pads or
        // neither: a file matching one legacy count but not the other
        // falls through to the shape check and fails loudly.
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
                let square =
                    [&blk.q_w, &blk.k_w, &blk.v_w, &blk.o_w].iter().all(|t| t.rows == h_obj && t.cols == h_obj);
                let vecs = [&blk.ln1_w, &blk.ln1_b, &blk.ln2_w, &blk.ln2_b, &blk.q_b, &blk.k_b, &blk.v_b, &blk.o_b]
                    .iter()
                    .all(|v| v.len() == h_obj);
                let ffn = blk.ffn1_w.cols == h_obj
                    && blk.ffn1_b.len() == blk.ffn1_w.rows
                    && blk.ffn2_w.cols == blk.ffn1_w.rows
                    && blk.ffn2_w.rows == h_obj
                    && blk.ffn2_b.len() == h_obj;
                if !(square && vecs && ffn) {
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
    fn encode_objects(&self, s: &EncodedState, h_obj: usize) -> (Vec<f32>, Vec<usize>) {
        let emb_dim = self.emb.cols;
        let n: usize = s.len();
        let mut hs = vec![0.0f32; n * h_obj];
        let mut owner = Vec::with_capacity(n);
        let mut x = vec![0.0f32; self.obj_w.cols];
        let mut i = 0;
        for (gi, group) in s.groups().enumerate() {
            for o in group {
                // See `DeckNet::forward`: out of range is the unknown slot,
                // not the last row.
                let card =
                    if (o.card as usize) < self.emb.rows { o.card as usize } else { 0 };
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
    /// all; the full pre-norm stack that pays that parity cost is
    /// [`Self::transform`] (`tblocks.*`), and old `attn.*` checkpoints
    /// keep loading through this path unchanged.
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
                    let dot: f32 = dot_dispatch(qi, kj);
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

    /// The transformer stack: group tag once into the stream, then
    /// pre-LN blocks in place. Attention math matches [`Self::attend`];
    /// the differences are the normed input, no relu on the stream, and
    /// the FFN half-block.
    fn transform(&self, hs: &mut [f32], owner: &[usize], h_obj: usize) {
        let Some(ts) = &self.tstack else { return };
        let n = owner.len();
        if n == 0 {
            return;
        }
        let hd = h_obj / ATTN_HEADS;
        let scale = 1.0 / (hd as f32).sqrt();

        for i in 0..n {
            let tag = &ts.group.data[owner[i] * h_obj..(owner[i] + 1) * h_obj];
            for j in 0..h_obj {
                hs[i * h_obj + j] += tag[j];
            }
        }

        let mut xn = vec![0.0f32; n * h_obj];
        for blk in &ts.blocks {
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
                for i in 0..n {
                    w.matvec(&xn[i * h_obj..(i + 1) * h_obj], &mut out[i * h_obj..(i + 1) * h_obj]);
                    for (v, bb) in out[i * h_obj..(i + 1) * h_obj].iter_mut().zip(b) {
                        *v += bb;
                    }
                }
                out
            };
            let q = project(&blk.q_w, &blk.q_b);
            let k = project(&blk.k_w, &blk.k_b);
            let v = project(&blk.v_w, &blk.v_b);

            let mut ctx = vec![0.0f32; n * h_obj];
            let mut scores = vec![0.0f32; n];
            for head in 0..ATTN_HEADS {
                let off = head * hd;
                for i in 0..n {
                    let qi = &q[i * h_obj + off..i * h_obj + off + hd];
                    let mut max = f32::NEG_INFINITY;
                    for (j, sc) in scores.iter_mut().enumerate() {
                        let kj = &k[j * h_obj + off..j * h_obj + off + hd];
                        let dot: f32 = dot_dispatch(qi, kj);
                        *sc = dot * scale;
                        max = max.max(*sc);
                    }
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
            let mut proj = vec![0.0f32; h_obj];
            for i in 0..n {
                blk.o_w.matvec(&ctx[i * h_obj..(i + 1) * h_obj], &mut proj);
                for j in 0..h_obj {
                    hs[i * h_obj + j] += proj[j] + blk.o_b[j];
                }
            }

            // x += ffn2(relu(ffn1(ln2(x))))
            let f = blk.ffn1_w.rows;
            let mut f1 = vec![0.0f32; f];
            for i in 0..n {
                layer_norm_row(
                    &hs[i * h_obj..(i + 1) * h_obj],
                    &blk.ln2_w,
                    &blk.ln2_b,
                    &mut xn[i * h_obj..(i + 1) * h_obj],
                );
                blk.ffn1_w.matvec(&xn[i * h_obj..(i + 1) * h_obj], &mut f1);
                for (v, b) in f1.iter_mut().zip(&blk.ffn1_b) {
                    *v = (*v + b).max(0.0);
                }
                blk.ffn2_w.matvec(&f1, &mut proj);
                for j in 0..h_obj {
                    hs[i * h_obj + j] += proj[j] + blk.ffn2_b[j];
                }
            }
        }
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
    /// A logit, not a probability: one state's policy score means nothing
    /// alone — the policy is the softmax over a decision's candidate set,
    /// and that normalisation belongs to the caller who has the set.
    pub fn forward_policy(&self, s: &EncodedState) -> Option<f32> {
        let w = self.policy_w.as_ref()?;
        let t2 = self.trunk_out(s);
        let mut out = [0.0f32];
        w.matvec(&t2, &mut out);
        Some(out[0] + self.policy_b)
    }

    pub fn has_policy_head(&self) -> bool {
        self.policy_w.is_some()
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

    /// The architecture the file actually carries — `(emb_dim,
    /// obj_hidden, h1, h2, attn, transformer blocks)` — for callers
    /// that must rebuild this net in another framework (the batched
    /// eval server). The shape follows the *file*, never the caller's
    /// run flags: a wide-learner run piloted by a standard-width
    /// champion is the normal case, and building the server at the
    /// run's width fails the load (round 45's first abort).
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
        let h_obj = self.obj_w.rows;
        let mut trunk_in = vec![0.0f32; self.trunk1_w.cols];

        // Encode every object, let them interact (no-op without the
        // attention tensors), then pool per group exactly as before —
        // pooling still supplies the fixed-width trunk input, it just
        // pools representations that have already seen the whole board.
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
        self.trunk1_w.matvec(&trunk_in, &mut t1);
        for (v, b) in t1.iter_mut().zip(&self.trunk1_b) {
            *v = (*v + b).max(0.0);
        }
        let mut t2 = vec![0.0f32; self.trunk2_w.rows];
        self.trunk2_w.matvec(&t1, &mut t2);
        for (v, b) in t2.iter_mut().zip(&self.trunk2_b) {
            *v = (*v + b).max(0.0);
        }
        t2
    }
}

/// Anything that can turn an encoded state into a win probability.
///
/// The engine's evaluation registry (`net_eval`) stores these rather than
/// [`PlayNet`] directly so the *training harness* can substitute a batched
/// GPU evaluator (encode on the game thread, ship the state to a collator,
/// block on the reply) without the engine growing an ML dependency. Takes
/// the state by value because a remote implementation has to move it into
/// a queue; the local one just forwards.
pub trait NetEvaluator: Send + Sync {
    /// Win probability for the seat the state was encoded for, in [0, 1].
    fn eval(&self, s: EncodedState) -> f32;

    /// The policy head's logit for the state, or `None` when the
    /// evaluator's net carries no policy head. Default `None` so
    /// evaluators that only serve win probabilities stay valid
    /// implementations — the search falls back to its heuristic priors.
    fn eval_policy(&self, s: EncodedState) -> Option<f32> {
        let _ = s;
        None
    }

    /// Whether [`eval_policy`](Self::eval_policy) can answer at all, so a
    /// profile can report which prior it will actually run with instead
    /// of discovering mid-game.
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
    /// This is the loadability half of the defect that killed the committed
    /// deck nets. The soundness half lives in `server::vocab_snapshot`: a
    /// row only keeps its meaning because a card name owns a frozen index.
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

    /// The policy head reads the same trunk as the win head and answers
    /// only when its tensors are present. A file carrying half the head
    /// is rejected: unlike the training-only aux heads, this one is
    /// consumed at inference, so a partial set is a mismatch and not an
    /// ignorable extra.
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

    /// A checkpoint from *any* earlier encoder generation — weight
    /// matrices sized to that generation's feature counts — loads by
    /// zero-padding, and the padded net cannot see the features added
    /// since: a state with every newer slot lit scores exactly what the
    /// legacy arithmetic says. This is what keeps the champion and the
    /// golden traces fixed across an encoder bump, and it has to hold
    /// for every generation in the table, not just the newest one.
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
