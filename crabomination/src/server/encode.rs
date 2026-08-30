//! GameState → net-input encoding for the SOS sealed value net.
//!
//! The other half of the contract lives in `crabomination_nn` (the tensor
//! types, the shard format, and the forward pass); this module owns the one
//! thing only the engine can do — reading a [`GameState`] into those types.
//! Everything encoded here is information the encoded seat could see across
//! the table: its own hand, both boards and graveyards, and *counts* of the
//! hidden zones. The opponent's hand and either library's contents never
//! enter the feature vector, so a net trained on these rows can't learn to
//! peek.
//!
//! Scope was deliberately SOS sealed through 2026-08-30 — one set's worth
//! of names kept the embedding table small and well-fed. The vocabulary
//! now also carries the modern cube pool ([`crate::cube::cube_pool_all`]),
//! appended after the frozen SOS seed so no SOS index moved and every
//! post-freeze net pads cleanly; a net *trained* before the append simply
//! has zero rows for the cube names and reads them as unknown, exactly as
//! it always did. Tokens and anything still off-vocabulary encode as
//! index 0 (unknown) and are represented by their object features alone —
//! a Pest token is "an unknown 1/1" to the net, which is most of what it
//! needs to know.

use crate::fxhash::HashMap;

use crabomination_nn::{
    EncodedObject, EncodedState, G_BF_OPP, G_BF_SELF, G_GY_OPP, G_GY_SELF, G_HAND_SELF,
    G_LIB_SELF, G_STACK_OPP, G_STACK_SELF, GLOBAL_FEATS,
};

use crate::card::{CardInstance, CounterType};
use crate::game::actions::color_index;
use crate::game::{GameState, TurnStep};
use crate::mana::ManaCost;

/// Card-name → embedding-index table. Index 0 is reserved for unknown
/// names.
///
/// **Indices come from [`crate::server::vocab_snapshot::VOCAB_SNAPSHOT`],
/// not from the pool.** They used to be the pool's sorted-name order, which
/// made every trained net a hostage to the card list: adding one SOS card
/// shifted the index of every card sorting after it, and the seven
/// committed deck nets are dead of exactly that. A snapshot name owns its
/// index whether or not it is still in the pool; a pool name outside the
/// snapshot is appended after it in sorted order. So the table only ever
/// grows at the end, and a net trained against a shorter one can be
/// zero-padded rather than retired.
pub struct Vocab {
    map: HashMap<&'static str, u16>,
    /// The same table keyed on the name's **address**, holding its length
    /// beside the index.
    ///
    /// A card's name is a `&'static str` literal owned by its catalog
    /// factory, so the same card always presents the same pointer and a
    /// pointer hash replaces a string hash *plus* the `memcmp` that confirms
    /// it. `index_of` is ~137,000 calls over twenty `selfplay_train` games —
    /// one per encoded object plus one per library card.
    ///
    /// **The length is not redundant.** Two `&str` at the same address with
    /// the same length are the same bytes, so that pair identifies a string;
    /// the address *alone* does not, because the linker is free to lay a
    /// short literal at the front of a longer one and hand both the same
    /// pointer. One `usize` compare closes that, and a miss is free anyway.
    ///
    /// A **cache, not a second source of truth**: it is filled from `map`, so
    /// a hit returns exactly what `map` would, and anything whose name did
    /// not come from a pool factory (a token, an off-set card) simply misses
    /// and falls through.
    by_ptr: HashMap<usize, (usize, u16)>,
}

impl Vocab {
    /// The frozen-snapshot universe: every card in the SOS draftable pool,
    /// the five basic lands the sealed builder adds, and — since
    /// 2026-08-30 — the modern cube pool, indexed against the frozen
    /// snapshot. The name predates the cube append and is kept for
    /// call-site stability; there is one growing vocabulary, not one per
    /// format, which is what lets a single net play every pool.
    pub fn sos_sealed() -> Vocab {
        let mut map: HashMap<&'static str, u16> = HashMap::default();
        for (i, name) in crate::server::vocab_snapshot::VOCAB_SNAPSHOT.iter().enumerate() {
            // The snapshot is generated, so a duplicate in it would be a
            // silent aliasing of two cards onto one embedding row.
            debug_assert!(!map.contains_key(name), "duplicate snapshot name {name}");
            map.insert(name, (i + 1) as u16);
        }
        // The cube pool joined the vocabulary 2026-08-30 (the modern-pool
        // track's first precondition): its names are frozen in the
        // snapshot after the SOS seed, and this union is what the
        // coverage tests below hold to the snapshot.
        let cube = crate::cube::cube_pool_all();
        let mut fresh: std::collections::BTreeSet<&'static str> = crate::draft::sos_draft_pool()
            .iter()
            .map(|f| f().name)
            .chain(cube.iter().map(|f| f().name))
            .filter(|n| !map.contains_key(n))
            .collect();
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            if !map.contains_key(basic) {
                fresh.insert(basic);
            }
        }
        for (next, name) in (map.len() as u16 + 1..).zip(fresh) {
            map.insert(name, next);
        }
        // Key the cache on the *pools'* name pointers, not the snapshot's:
        // the snapshot is its own array of literals and a card's definition
        // carries a different one, so a cache built from the snapshot would
        // miss every lookup.
        let mut by_ptr: HashMap<usize, (usize, u16)> = HashMap::default();
        for name in crate::draft::sos_draft_pool()
            .iter()
            .map(|f| f().name)
            .chain(cube.iter().map(|f| f().name))
            .chain(["Plains", "Island", "Swamp", "Mountain", "Forest"])
        {
            if let Some(&i) = map.get(name) {
                by_ptr.insert(name.as_ptr() as usize, (name.len(), i));
            }
        }
        Vocab { map, by_ptr }
    }

    /// Total index count including the reserved unknown slot — the
    /// embedding table's row count.
    pub fn size(&self) -> usize {
        self.map.len() + 1
    }

    /// 0 for anything unrecognized (tokens, off-set cards).
    pub fn index_of(&self, name: &str) -> u16 {
        if let Some(&(len, i)) = self.by_ptr.get(&(name.as_ptr() as usize))
            && len == name.len()
        {
            debug_assert_eq!(Some(i), self.map.get(name).copied(), "vocab pointer cache drifted");
            return i;
        }
        self.map.get(name).copied().unwrap_or(0)
    }
}

/// Which round-11 feature blocks the encoder emits.
///
/// A measurement control, in the house style of keeping the replaced
/// behaviour available: the library group and the castability block landed
/// together with a vocabulary change, so "the new encoder scores worse" has
/// three candidate causes and no way to separate them without being able to
/// switch each block off while everything else stays fixed.
///
/// Ablated blocks are *zeroed*, not removed — feature counts and
/// [`crabomination_nn::SHARD_VERSION`] are unchanged, so an ablated run and
/// a full run produce interchangeable shards and identically-shaped nets.
/// Process-global for the same reason the net slot and the bot's jitter seed
/// are: the encoder is called from deep inside the search and threading a
/// config through every call site would be a worse trade than a flag set
/// once at startup.
static ABLATE: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);
const ABLATE_LIBRARY: u16 = 1;
const ABLATE_CASTABILITY: u16 = 2;
const ABLATE_RELATIONS: u16 = 4;
const ABLATE_COMBAT: u16 = 8;
const ABLATE_KW: u16 = 16;
const ABLATE_HISTORY: u16 = 32;
const ABLATE_EXPIRY: u16 = 64;
const ABLATE_COUNTERS: u16 = 128;
const ABLATE_V8: u16 = 256;

/// Every ablatable block by the name the trainer and the ladder accept.
/// One table so the two binaries can't drift on which names are legal —
/// a mistyped block that silently ablates nothing is the failure mode
/// this exists to prevent.
///
/// * `lib` / `cast` — the round-11 library group and castability block.
/// * `rel` — the round-12 block: the relation flags (28..=33, 35),
///   the stack groups, and stack depth. Feature 34 (the counter sum)
///   shipped with this block but is battlefield state, not a relation,
///   and no longer rides its bit — see `encode_battlefield_object`.
/// * `combat` / `kw` — round 28 (v6): combat structure (object feats
///   37..=39, globals 36..=40) and keyword classes plus exile counts
///   (object feats 40..=44, globals 41..=42).
/// * `hist` / `exp` / `ctr` — round 40 (v7): turn-scoped history
///   (globals 43..=54), expiry (object feats 45..=47), and the counter
///   split (object feats 48..=52).
/// * `v8` — 2026-08-30 (modern precondition 3): artifact/enchantment
///   type bits (object feats 53/54), the modern counter kinds
///   (55..=58: Lore, Charge, Shield, Finality), and land drops
///   remaining (globals 55/56).
pub const ABLATION_BLOCKS: [(&str, u16); 9] = [
    ("lib", ABLATE_LIBRARY),
    ("cast", ABLATE_CASTABILITY),
    ("rel", ABLATE_RELATIONS),
    ("combat", ABLATE_COMBAT),
    ("kw", ABLATE_KW),
    ("hist", ABLATE_HISTORY),
    ("exp", ABLATE_EXPIRY),
    ("ctr", ABLATE_COUNTERS),
    ("v8", ABLATE_V8),
];

/// Turn the named feature blocks *off* for an ablation run; everything
/// not named stays on, and an empty list is the full encoder. Replaces
/// the whole mask, so it is idempotent and a later call can restore the
/// default. Unknown names are an error rather than a no-op: silently
/// ignoring a typo produces a "control" run that is really a second
/// copy of the arm.
pub fn set_encode_ablation_off(off: &[&str]) -> Result<(), String> {
    let mut mask = 0u16;
    for name in off {
        match ABLATION_BLOCKS.iter().find(|(n, _)| n == name) {
            Some((_, bit)) => mask |= bit,
            None => {
                let known: Vec<&str> = ABLATION_BLOCKS.iter().map(|(n, _)| *n).collect();
                return Err(format!("unknown encoder block {name:?} (known: {})", known.join(", ")));
            }
        }
    }
    ABLATE.store(mask, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

fn ablated(bit: u16) -> bool {
    ABLATE.load(std::sync::atomic::Ordering::Relaxed) & bit != 0
}

/// Encode the position from `seat`'s perspective. Two-player only, like
/// the rest of the bot stack.
pub fn encode_state(g: &GameState, seat: usize, vocab: &Vocab) -> EncodedState {
    // One frozen-layer scope for the whole encode (modern precondition 2):
    // the battlefield reads below take layer-resolved truth per permanent
    // (`computed_permanent_on`, memoized per card inside the scope), and
    // the castability block already opened a scope of its own — nested
    // scopes reuse the outer gather, so this is one continuous-effect
    // gather per encoded state, not one per read.
    g.with_frozen_layers(|g| encode_state_inner(g, seat, vocab))
}

fn encode_state_inner(g: &GameState, seat: usize, vocab: &Vocab) -> EncodedState {
    let opp = 1 - seat;
    let mut s = EncodedState::default();

    // Relation context (round 12): unary summaries of edges the pooled
    // representation cannot carry — see the OBJ_FEATS doc, 28..=36.
    let no_rel = ablated(ABLATE_RELATIONS);
    let mut targeted: crate::fxhash::HashSet<crate::card::CardId> = Default::default();
    // (host id, attachment's controller) — resolved against the host's
    // own controller at encode time, because "who controls the aura on
    // this creature" is what separates a buff from a Pacifism.
    let mut attachments: Vec<(crate::card::CardId, usize)> = Vec::new();
    if !no_rel {
        use crate::game::types::{StackItem, Target};
        for item in g.stack.iter() {
            let (target, extra): (&Option<Target>, &[Target]) = match item {
                StackItem::Spell { target, additional_targets, .. } => {
                    (target, additional_targets)
                }
                StackItem::Trigger { target, .. } => (target, &[]),
            };
            for t in target.iter().chain(extra) {
                if let Target::Permanent(id) = t {
                    targeted.insert(*id);
                }
            }
        }
        for c in g.battlefield.iter() {
            if let Some(host) = c.attached_to {
                attachments.push((host, c.controller));
            }
        }
    }

    // Combat structure (round 28): the round-12 flags said a creature is
    // blocked; these say by what. Effective P/T of one combat's
    // counterparties, summed — a pooling-safe unary summary of the edge,
    // like the relation flags, but carrying the numbers the block sims
    // actually trade on.
    let no_combat = ablated(ABLATE_COMBAT);
    let eff_pt = |id: crate::card::CardId| {
        g.battlefield.iter().find(|c| c.id == id).and_then(|c| {
            let cp = g.computed_permanent_on(c)?;
            Some((cp.power.max(0), (cp.toughness - c.damage as i32).max(0)))
        })
    };
    // Attacker → summed P/T of its blockers (`block_map` is blocker →
    // attackers, so this is the map inverted).
    let mut blocker_sums: HashMap<crate::card::CardId, (i32, i32)> = HashMap::default();
    if !no_combat {
        for (blocker, attackers) in g.block_map.iter() {
            if let Some((p, t)) = eff_pt(*blocker) {
                for a in attackers {
                    let e = blocker_sums.entry(*a).or_insert((0, 0));
                    e.0 += p;
                    e.1 += t;
                }
            }
        }
    }

    // All eight groups share one buffer (PERF `(-107)`), so the whole state
    // is one reserve and one allocation. Every group's size is known before
    // its loop except the library's, which is deduplicated by name and so is
    // *bounded* by the library's length; that surplus is the one allocation's
    // slack, not a second allocation. Eight separately reserved `Vec`s were
    // 66,485 `do_reserve_and_handle` growths over 12,660 encoded states —
    // 5.25 apiece, 28.4 M Ir, 0.48 % of an actor.
    s.reserve(
        g.battlefield.len()
            + g.players[seat].hand.len()
            + g.players[seat].graveyard.len()
            + g.players[opp].graveyard.len()
            + if ablated(ABLATE_LIBRARY) { 0 } else { g.players[seat].library.len() }
            + if no_rel { 0 } else { g.stack.len() },
    );
    // The buffer is laid out group by group, so the battlefield's controller
    // split is two filtered passes rather than one interleaved one: the same
    // cards are encoded, in the same within-group order, by the same code.
    for (group, mine) in [(G_BF_SELF, true), (G_BF_OPP, false)] {
        for c in g.battlefield.iter().filter(|c| (c.controller == seat) == mine) {
            // Push an empty object and fill it in place: see
            // `encode_card_object_into` for what the by-value form cost.
            let o = s.push_default(group);
            encode_battlefield_object_into(g, c, vocab, o);
            if !no_combat {
                // An object is never both an attacker and a blocker in one
                // combat, so one feature pair serves both endpoints.
                let counterpart = blocker_sums.get(&c.id).copied().or_else(|| {
                    g.block_map.get(&c.id).map(|attackers| {
                        attackers.iter().filter_map(|a| eff_pt(*a)).fold((0, 0), |acc, (p, t)| {
                            (acc.0 + p, acc.1 + t)
                        })
                    })
                });
                if let Some((p, t)) = counterpart {
                    o.feats[37] = p as f32 / 8.0;
                    o.feats[38] = t as f32 / 8.0;
                }
                if g.attacking.iter().any(|a| {
                    a.attacker == c.id
                        && !matches!(a.target, crate::game::types::AttackTarget::Player(_))
                }) {
                    o.feats[39] = 1.0;
                }
            }
            if !no_rel {
                if g.block_map.contains_key(&c.id) {
                    o.feats[28] = 1.0;
                }
                // CR 509.1b: an attacker that became blocked stays blocked
                // when its blockers leave combat (first-strike deaths,
                // post-block removal) — `block_map` forgets that the moment
                // `remove_permanent_from_combat` runs, `blocked_attackers`
                // doesn't, and the damage step reads the latter. The union
                // also keeps synthetically built states (tests, hand-rolled
                // sims) that fill only the map reading as blocked.
                if g.blocked_attackers().contains(&c.id)
                    || g.block_map.values().any(|attackers| attackers.contains(&c.id))
                {
                    o.feats[29] = 1.0;
                }
                if c.attached_to.is_some() {
                    o.feats[30] = 1.0;
                }
                for (host, attach_ctl) in &attachments {
                    if *host == c.id {
                        if *attach_ctl == c.controller {
                            o.feats[31] = 1.0;
                        } else {
                            o.feats[32] = 1.0;
                        }
                    }
                }
                if targeted.contains(&c.id) {
                    o.feats[33] = 1.0;
                }
            }
        }
    }
    // Castability is per-seat state, so the hand's live/dead split is
    // computed against this seat's own untapped sources.
    let no_cast = ablated(ABLATE_CASTABILITY);
    // Both seats' source tables under ONE freeze scope. `mana_source_table`
    // opens its own, so two calls gathered the whole continuous-effect set
    // twice per encode and built two `perms` memos; nested scopes reuse the
    // outer one. `g` is `&GameState` for the whole function, so the
    // opponent's half is the same answer wherever it is computed — it is
    // read ~90 lines down, next to the globals it fills.
    let (sources, opp_sources) = if no_cast {
        (Vec::new(), Vec::new())
    } else {
        g.with_frozen_layers(|g| (g.untapped_mana_colors(seat), g.untapped_mana_colors(opp)))
    };
    // One mask cover for the whole hand — see `source_cover`.
    let n_sources = sources.len() as u32;
    let cover = source_cover(&sources);
    let cover_extra = cover_with_extra(&cover);
    for c in g.players[seat].hand.iter() {
        let o = s.push_default(G_HAND_SELF);
        encode_card_object_into(c, vocab, o);
        if !no_cast && !c.definition.is_land() {
            o.feats[25] =
                if affordable_covered(&c.definition.cost, n_sources, &cover) { 1.0 } else { 0.0 };
            // Next turn is this turn plus one more source of any colour —
            // the land drop the seat has not made yet. Deliberately
            // optimistic about colour: "which of my cards come online if I
            // hit my drop" is the question, and a wrong-colour land is the
            // rarer case in a two-colour sealed deck.
            o.feats[26] = if affordable_covered(&c.definition.cost, n_sources + 1, &cover_extra) {
                1.0
            } else {
                0.0
            };
        }
    }
    for (group, p) in [(G_GY_SELF, seat), (G_GY_OPP, opp)] {
        for c in g.players[p].graveyard.iter() {
            encode_card_object_into(c, vocab, s.push_default(group));
        }
    }
    encode_library(&mut s, g, seat, vocab);
    if !no_rel {
        encode_stack(&mut s, g, seat, vocab);
    }

    let (mut lands, mut untapped, mut creatures, mut power) = ([0i32; 2], [0i32; 2], [0i32; 2], [0i32; 2]);
    for c in g.battlefield.iter() {
        let side = if c.controller == seat { 0 } else { 1 };
        // Computed types and power, so an animated manland counts as the
        // creature it currently is and an anthem's pump reaches the power
        // totals — the same memoized layer pass the group loop paid.
        let (is_land, is_creature, pw) = match g.computed_permanent_on(c) {
            Some(cp) => {
                use crate::card::CardType;
                (
                    cp.card_types().contains(&CardType::Land),
                    cp.card_types().contains(&CardType::Creature),
                    cp.power.max(0),
                )
            }
            None => (c.definition.is_land(), c.definition.is_creature(), c.power().max(0)),
        };
        if is_land {
            lands[side] += 1;
            if !c.tapped {
                untapped[side] += 1;
            }
        }
        if is_creature {
            creatures[side] += 1;
            power[side] += pw;
        }
    }

    let gl = &mut s.global;
    gl[0] = g.players[seat].life as f32 / 20.0;
    gl[1] = g.players[opp].life as f32 / 20.0;
    gl[2] = g.players[seat].hand.len() as f32 / 7.0;
    gl[3] = g.players[opp].hand.len() as f32 / 7.0;
    gl[4] = g.players[seat].library.len() as f32 / 40.0;
    gl[5] = g.players[opp].library.len() as f32 / 40.0;
    gl[6] = g.players[seat].graveyard.len() as f32 / 15.0;
    gl[7] = g.players[opp].graveyard.len() as f32 / 15.0;
    gl[8] = g.turn_number as f32 / 15.0;
    gl[9] = if g.active_player_idx == seat { 1.0 } else { 0.0 };
    let step_slot = match g.step {
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw | TurnStep::PreCombatMain => 10,
        TurnStep::BeginCombat
        | TurnStep::DeclareAttackers
        | TurnStep::DeclareBlockers
        | TurnStep::FirstStrikeDamage
        | TurnStep::CombatDamage
        | TurnStep::EndCombat => 11,
        TurnStep::PostCombatMain => 12,
        TurnStep::End | TurnStep::Cleanup => 13,
    };
    gl[step_slot] = 1.0;
    gl[14] = untapped[0] as f32 / 6.0;
    gl[15] = untapped[1] as f32 / 6.0;
    gl[16] = lands[0] as f32 / 8.0;
    gl[17] = lands[1] as f32 / 8.0;
    gl[18] = g.stack.len() as f32 / 3.0;
    gl[19] = g.attacking.len() as f32 / 4.0;
    gl[20] = creatures[0] as f32 / 6.0;
    gl[21] = creatures[1] as f32 / 6.0;
    gl[22] = power[0] as f32 / 12.0;
    gl[23] = power[1] as f32 / 12.0;
    // Mana actually available, by colour, for both seats. gl[14..=15]
    // already counted untapped *lands*; these count untapped *sources*
    // (mana creatures and rocks included) and say what colours they make.
    // The opponent's half is public and is what makes "they have two
    // untapped blue" — the shape of every instant-speed decision — even
    // representable.
    if !no_cast {
        for (base, src) in [(24, &sources), (30, &opp_sources)] {
            for ci in 0..5 {
                gl[base + ci] = src.iter().filter(|m| m[ci]).count() as f32 / 6.0;
            }
            gl[base + 5] = src.len() as f32 / 6.0;
        }
    }
    if !no_combat {
        // Fine combat phase. The coarse slot 11 collapses "attacks
        // declared, blocks pending" and "damage dealt" — opposite worlds
        // to a value function, and the states the combat sims evaluate
        // most. DeclareAttackers still reads as pre-blocks even with
        // `g.attacking` filled, which is exactly the attack sim's leaf.
        let step_slot = match g.step {
            TurnStep::DeclareAttackers => Some(36),
            TurnStep::DeclareBlockers => Some(37),
            TurnStep::FirstStrikeDamage | TurnStep::CombatDamage | TurnStep::EndCombat => Some(38),
            _ => None,
        };
        if let Some(slot) = step_slot {
            gl[slot] = 1.0;
        }
        // Power aimed at each life total: unblocked attackers in full,
        // blocked tramplers by the excess over their live blockers'
        // effective toughness (the default lethal-to-each assignment,
        // CR 702.19g — everything when every blocker is already gone).
        // Before blocks it is the whole attack; after, what gets
        // through — the phase one-hots above disambiguate which.
        for a in g.attacking.iter() {
            if let crate::game::types::AttackTarget::Player(p) = a.target {
                // CR 509.1b again (see feat 29): `blocked_attackers`, not
                // `block_map`, is what the damage step reads. Without it an
                // attacker whose blocker died to first strike read its full
                // power into this global while the engine deals zero — a
                // wrong signal at exactly the settled combat states the
                // search evaluates most.
                let blocked = g.blocked_attackers().contains(&a.attacker)
                    || blocker_sums.contains_key(&a.attacker)
                    || g.block_map.values().any(|att| att.contains(&a.attacker));
                let Some(c) = g.battlefield.iter().find(|c| c.id == a.attacker) else {
                    continue;
                };
                // Computed power and trample: a granted trample (sword,
                // anthem rider) is exactly the case the raw keyword walk
                // missed.
                let (pw, trample) = match g.computed_permanent_on(c) {
                    Some(cp) => (
                        cp.power.max(0),
                        cp.keywords().contains(&crate::card::Keyword::Trample),
                    ),
                    None => (c.power().max(0), c.has_keyword(&crate::card::Keyword::Trample)),
                };
                let through = if !blocked {
                    pw
                } else if trample {
                    let (_, t) = blocker_sums.get(&a.attacker).copied().unwrap_or((0, 0));
                    (pw - t).max(0)
                } else {
                    0
                };
                if through > 0 {
                    gl[if p == seat { 39 } else { 40 }] += through as f32 / 12.0;
                }
            }
        }
    }
    if !ablated(ABLATE_KW) {
        // Exile sizes — the one public zone the encoding had no trace
        // of. Counts only: contents wait on zone groups (and face-down
        // exile is hidden information anyway).
        gl[41] = g.exile.iter().filter(|c| c.owner == seat).count() as f32 / 10.0;
        gl[42] = g.exile.iter().filter(|c| c.owner == opp).count() as f32 / 10.0;
    }
    if !ablated(ABLATE_HISTORY) {
        // Turn-scoped history (round 40). Everything above is a static
        // snapshot, which made the whole encoding a pure function of the
        // current position — so for a card that reads what has already
        // happened this turn, the net could not tell whether its ability
        // was live. `Player` tracks ~90 such counters; these six are the
        // ones the SoS pool actually reads, self then opponent.
        //
        // Scales are "how many before this stops mattering", not
        // normalizations of the observed range: the predicates are
        // thresholds (gain 3 life, cast 2 instants) and what a value
        // function wants is where the position sits against the
        // threshold, so saturating a little past it is the right shape.
        for (side, p) in [(0usize, seat), (1, opp)] {
            let pl = &g.players[p];
            gl[43 + side] = pl.life_gained_this_turn as f32 / 5.0;
            gl[45 + side] = pl.instants_or_sorceries_cast_this_turn as f32 / 3.0;
            gl[47 + side] = pl.spells_cast_this_turn as f32 / 4.0;
            gl[49 + side] = pl.creatures_died_this_turn as f32 / 3.0;
            gl[51 + side] = pl.cards_left_graveyard_this_turn as f32 / 3.0;
            gl[53 + side] = pl.cards_exiled_this_turn as f32 / 3.0;
        }
    }
    if !ablated(ABLATE_V8) {
        // Land drops remaining this turn, self then opponent, / 2
        // (Azusa-class grants are the headroom). "Can I still
        // land-and-spell this turn" is the most common main-phase
        // sequencing read; feature 26's next-turn castability assumed
        // the drop was there, and now the net can see whether it is.
        // `can_player_play_land` checks locks and the count, not the
        // turn, so the off-turn opponent reads their standing drop —
        // public, and the active-player flag (gl 9) says whose turn
        // makes it spendable.
        for (side, p) in [(0usize, seat), (1, opp)] {
            let remaining = if g.can_player_play_land(p) {
                g.max_lands_per_turn(p)
                    .saturating_sub(g.players[p].lands_played_this_turn)
            } else {
                0
            };
            gl[55 + side] = remaining as f32 / 2.0;
        }
    }
    const _: () = assert!(GLOBAL_FEATS == 57, "extend the fill above when adding globals");

    s
}

/// The seat's own library, deduplicated by card name.
///
/// One object per distinct name with its remaining count in feature 27,
/// rather than one per physical card: a sealed deck's eight Plains are one
/// fact, not eight, and collapsing them keeps the object count (and so the
/// quadratic attention cost) down while *adding* information the
/// enumerated form only carries implicitly — "two copies of that removal
/// spell left" is directly readable.
///
/// Emitted in vocabulary-index order so the library's actual shuffle can
/// never reach the net, whatever the architecture does downstream.
fn encode_library(s: &mut EncodedState, g: &GameState, seat: usize, vocab: &Vocab) {
    if ablated(ABLATE_LIBRARY) {
        return;
    }
    // A `BTreeMap<u16, _>` here was 179,404 `Entry::or_insert` calls and
    // 119,224 `IntoIter::dying_next` a sixty-game actor run — 0.63 % of it —
    // to group ~28 library cards into ~19 distinct names. A linear scan over
    // an inline buffer plus one sort answers the same question: the keys are
    // unique, so the sort is a total order and the emitted sequence is
    // byte-identical to the map's. Only the actor pays this at all; `--bench`
    // never encodes a state.
    let lib = &g.players[seat].library;
    let mut counts: smallvec::SmallVec<[(u16, &CardInstance, u32); 32]> =
        smallvec::SmallVec::new();
    for c in lib.iter() {
        let idx = c.vocab_index(|d| vocab.index_of(d.name));
        match counts.iter_mut().find(|e| e.0 == idx) {
            // First card of a name wins, exactly as `or_insert` did.
            Some(e) => e.2 += 1,
            None => counts.push((idx, c, 1)),
        }
    }
    counts.sort_unstable_by_key(|e| e.0);
    // By reference: this buffer is 32 x 24 bytes inline, so a by-value
    // `IntoIter` would move 768 bytes out of the frame on every encoded
    // state — PERF's hundred-and-third pass (3).
    for &(_, c, n) in &counts {
        let o = s.push_default(G_LIB_SELF);
        encode_card_object_into(c, vocab, o);
        o.feats[27] = n as f32 / 4.0;
    }
}

/// The stack, one object per item, split by controller like the
/// battlefield. Spells encode their own card; a trigger encodes its
/// source — from the battlefield, or for a source that has left it (a
/// dies / leaves-the-battlefield trigger, the most common trigger class
/// on the stack) from the engine's LKI snapshot or the graveyard. Only a
/// source in no readable zone encodes as an unknown object, and even that
/// keeps the multiplicity baseline every real object carries. Depth from
/// the top of the stack (the item that resolves first) lands in feature
/// 36, because pooling would otherwise erase resolution order.
fn encode_stack(s: &mut EncodedState, g: &GameState, seat: usize, vocab: &Vocab) {
    use crate::game::types::StackItem;
    let n = g.stack.len();
    // Two filtered passes, like the battlefield: the shared buffer is laid
    // out group by group, and `i` stays the item's index in the real stack
    // so the depth feature is unchanged.
    for (group, mine) in [(G_STACK_SELF, true), (G_STACK_OPP, false)] {
        for (i, item) in g.stack.iter().enumerate() {
            let controller = match item {
                StackItem::Spell { caster, .. } => *caster,
                StackItem::Trigger { controller, .. } => *controller,
            };
            if (controller == seat) != mine {
                continue;
            }
            let mut o = match item {
                StackItem::Spell { card, .. } => encode_card_object(card, vocab),
                StackItem::Trigger { source, .. } => {
                    // Battlefield first (the common, previously-only
                    // case); then CR 603.10 LKI, which carries the source
                    // as it last existed on the battlefield; then the
                    // graveyard, where a dies-trigger's source actually
                    // is. Exile is deliberately skipped: it mixes
                    // face-down (hidden) cards in, and a trigger source
                    // there is rare enough to take the unknown object
                    // instead of auditing the leak.
                    let inst = g
                        .battlefield
                        .iter()
                        .find(|c| c.id == *source)
                        .or_else(|| g.died_card_snapshots.get(source))
                        .or_else(|| g.leaves_bf_lki.get(source))
                        .or_else(|| {
                            g.players
                                .iter()
                                .flat_map(|p| p.graveyard.iter())
                                .find(|c| c.id == *source)
                        });
                    inst.map(|c| encode_card_object(c, vocab)).unwrap_or_else(|| {
                        let mut o = EncodedObject::default();
                        // The multiplicity baseline every other object
                        // gets from `encode_card_object`; without it the
                        // unknown trigger was the one object class
                        // off-distribution on feature 27.
                        o.feats[27] = 1.0 / 4.0;
                        o
                    })
                }
            };
            // The stack is a Vec used LIFO: the last element is the top.
            o.feats[36] = (n - 1 - i) as f32 / 4.0;
            s.push(group, o);
        }
    }
}

/// Can `cost` be paid right now off `sources`, one mana per source?
///
/// Exact for the model it assumes, by Hall's condition over the 32 colour
/// subsets: a multiset of coloured pips has a saturating assignment iff
/// for every subset of colours, the pips wanting those colours are no more
/// numerous than the sources able to make one of them. A saturating
/// assignment uses exactly one source per coloured pip, so the generic
/// remainder is satisfied iff the total source count covers the whole
/// mana value.
///
/// It is an approximation of the rules, deliberately so: sources that tap
/// for two mana, cost reduction, alternative costs, {X}, and hybrid pips
/// (counted as their first half by `colored_symbols`) all fall outside it.
/// This is a *feature* — "is this card roughly live" — not a legality
/// check, and the real payment path is [`GameState::auto_tap_for_cost`].
#[cfg(test)]
fn affordable(cost: &ManaCost, sources: &[[bool; 5]]) -> bool {
    affordable_covered(cost, sources.len() as u32, &source_cover(sources))
}

/// `[mask] -> how many of `sources` make at least one colour in `mask``.
///
/// The half of [`affordable`] that depends only on the *sources*, lifted out
/// so a hand sweep pays it once instead of once per card — and twice per
/// card, because the next-turn sibling used to clone the whole slice and walk
/// it again. `encode_state` called `affordable` 11,630 times over
/// twenty `selfplay_train` games for 11.3 M Ir (0.86 % of an actor), 5.8 per
/// state against one hand's worth of sources.
fn source_cover(sources: &[[bool; 5]]) -> [u32; 32] {
    let mut have = [0u32; 32];
    for s in sources {
        let m: usize = (0..5).filter(|i| s[*i]).fold(0, |a, i| a | 1 << i);
        for (mask, h) in have.iter_mut().enumerate().skip(1) {
            if mask & m != 0 {
                *h += 1;
            }
        }
    }
    have
}

/// [`affordable`] against a prebuilt [`source_cover`].
fn affordable_covered(cost: &ManaCost, n_sources: u32, have: &[u32; 32]) -> bool {
    let mut pips = [0u32; 5];
    for c in cost.colored_symbols() {
        pips[color_index(c)] += 1;
    }
    let colored: u32 = pips.iter().sum();
    // `cmc` charges mono-hybrid its generic half while `colored_symbols`
    // also counts it as a pip; take whichever is larger so the total can
    // never come in under the coloured requirement.
    if n_sources < cost.cmc().max(colored) {
        return false;
    }
    // `need[mask]` by the subset recurrence — 32 adds rather than 31 inner
    // loops over five colours.
    let mut need = [0u32; 32];
    for mask in 1usize..32 {
        let low = mask.trailing_zeros() as usize;
        need[mask] = need[mask & (mask - 1)] + pips[low];
    }
    (1usize..32).all(|mask| need[mask] <= have[mask])
}

/// A [`source_cover`] plus one source that makes every colour — the land drop
/// the seat has not taken yet. It intersects every non-empty mask, so each
/// entry gains exactly one; the old form pushed `[true; 5]` onto a **clone**
/// of the whole source slice, per hand card.
fn cover_with_extra(have: &[u32; 32]) -> [u32; 32] {
    let mut out = *have;
    for h in out.iter_mut().skip(1) {
        *h += 1;
    }
    out
}

/// Encode a decklist for the build net: vocab indices plus deck-level
/// features (spell curve, land/creature counts, color pips). The factory
/// list is the same shape the sealed builder and `recommend_pool` deal
/// in, so both can score builds without touching a `GameState`.
pub fn encode_deck(
    deck: &[crate::cube::CardFactory],
    vocab: &Vocab,
) -> (Vec<u16>, [f32; crabomination_nn::DECK_FEATS]) {
    use crate::mana::Color;
    let mut cards = Vec::with_capacity(deck.len());
    let mut feats = [0.0f32; crabomination_nn::DECK_FEATS];
    let (mut lands, mut creatures, mut mv_sum, mut spells) = (0u32, 0u32, 0u32, 0u32);
    let mut pips = [0u32; 5];
    for f in deck {
        let def = f();
        cards.push(vocab.index_of(def.name));
        if def.is_land() {
            lands += 1;
            continue;
        }
        spells += 1;
        let mv = def.cost.cmc();
        mv_sum += mv;
        // Curve buckets 0..=6: mv ≤1, 2, 3, 4, 5, 6, 7+.
        let bucket = (mv.clamp(1, 7) - 1) as usize;
        feats[bucket] += 1.0 / 8.0;
        if def.is_creature() {
            creatures += 1;
        }
        for c in def.cost.colored_symbols() {
            let i = match c {
                Color::White => 0,
                Color::Blue => 1,
                Color::Black => 2,
                Color::Red => 3,
                Color::Green => 4,
            };
            pips[i] += 1;
        }
    }
    feats[7] = lands as f32 / 17.0;
    feats[8] = creatures as f32 / 23.0;
    for (i, p) in pips.iter().enumerate() {
        feats[9 + i] = *p as f32 / 20.0;
    }
    feats[14] = pips.iter().filter(|&&p| p > 0).count() as f32 / 3.0;
    feats[15] = if spells > 0 { mv_sum as f32 / spells as f32 / 4.0 } else { 0.0 };
    (cards, feats)
}

/// Printed-card features shared by every zone (hand, graveyard, and the
/// base of a battlefield object).
/// The twelve keyword questions [`encode_card_object`] asks, as bits.
mod okw {
    pub const FLYING: u16 = 1 << 0;
    pub const REACH: u16 = 1 << 1;
    pub const MENACE: u16 = 1 << 2;
    pub const DEATHTOUCH: u16 = 1 << 3;
    pub const LIFELINK: u16 = 1 << 4;
    pub const TRAMPLE: u16 = 1 << 5;
    /// First strike and double strike share a flag — for a value function
    /// they mean the same thing at this resolution.
    pub const STRIKE: u16 = 1 << 6;
    pub const VIGILANCE: u16 = 1 << 7;
    pub const HASTE: u16 = 1 << 8;
    pub const INDESTRUCTIBLE: u16 = 1 << 9;
    pub const DEFENDER: u16 = 1 << 10;
    pub const HARD_TO_TARGET: u16 = 1 << 11;
    pub const HARD_TO_BLOCK: u16 = 1 << 12;
}

/// The exact-variant bit a keyword contributes, or 0.
fn okw_exact_bit(k: &crate::card::Keyword) -> u16 {
    use crate::card::Keyword as K;
    match k {
        K::Flying => okw::FLYING,
        K::Reach => okw::REACH,
        K::Menace => okw::MENACE,
        K::Deathtouch => okw::DEATHTOUCH,
        K::Lifelink => okw::LIFELINK,
        K::Trample => okw::TRAMPLE,
        K::FirstStrike | K::DoubleStrike => okw::STRIKE,
        K::Vigilance => okw::VIGILANCE,
        K::Haste => okw::HASTE,
        K::Indestructible => okw::INDESTRUCTIBLE,
        K::Defender => okw::DEFENDER,
        _ => 0,
    }
}

/// Every keyword bit [`encode_card_object`] needs, in **one** pass over this
/// card's own lists.
///
/// [`CardInstance::has_keyword`] re-walks `removed_keywords`,
/// `removed_keywords_eot`, the printed list, `granted_keywords_eot` and the
/// counter `Vec` on every keyword asked — and the encoder asks twelve, plus
/// two `any_keyword` class walks. On a `selfplay_train` actor that was
/// **831,492 `has_keyword` calls / 37.6 M Ir / 2.85 %**, 422 per encoded
/// state, and `encode_state` is 11.7 % of the actor to begin with. Inverting
/// it makes the cost the card's keyword *count* instead of the encoder's
/// question count.
///
/// Equivalent by construction and `debug_assert!`ed at the call site: a
/// keyword contributes its bit exactly when it appears printed, EOT-granted
/// or on a CR 122.1b counter and is not in either removal list — which is
/// `has_keyword`'s own definition. The two class bits read printed and
/// granted only, matching [`any_keyword`], which deliberately skips counters.
fn object_keyword_bits(c: &CardInstance) -> u16 {
    use crate::card::KeywordSlice;
    let removed = |k: &crate::card::Keyword| {
        c.removed_keywords.has_kw(k) || c.removed_keywords_eot.has_kw(k)
    };
    let mut m = 0u16;
    for k in c.definition.keywords.iter().chain(c.granted_keywords_eot.iter()) {
        if removed(k) {
            continue;
        }
        m |= okw_exact_bit(k);
        if is_hard_to_target(k) {
            m |= okw::HARD_TO_TARGET;
        }
        if is_hard_to_block(k) {
            m |= okw::HARD_TO_BLOCK;
        }
    }
    for (k, n) in c.keyword_counters.iter() {
        if *n > 0 && !removed(k) {
            m |= okw_exact_bit(k);
        }
    }
    m
}

fn encode_card_object(c: &CardInstance, vocab: &Vocab) -> EncodedObject {
    let mut o = EncodedObject::default();
    encode_card_object_into(c, vocab, &mut o);
    o
}

/// [`encode_card_object`] writing through a handle the caller already owns.
///
/// An `EncodedObject` is `{ u16, [f32; 53] }` = 216 bytes. Returned by value
/// and then pushed, each one is copied **twice**: zeroed on the stack, filled,
/// and memcpy'd into its group `Vec`. `encode_card_object` was 228,515 calls a
/// sixty-game actor run with a 6.9 M-Ir `__memcpy` edge of its own, and
/// `encode_state` another 7.8 M for the pushes (ninety-eighth pass's actor
/// profile). A caller that pushes an empty object and fills it in place pays
/// one of the two.
///
/// `out` is assumed **zeroed**: this writes only the features it sets, exactly
/// as the by-value form did over a fresh `[0.0; OBJ_FEATS]`.
fn encode_card_object_into(c: &CardInstance, vocab: &Vocab, out: &mut EncodedObject) {
    use crate::card::Keyword;
    let def = &c.definition;
    let feats = &mut out.feats;
    feats[0] = def.cost.cmc() as f32 / 8.0;
    feats[1] = if def.is_creature() { 1.0 } else { 0.0 };
    feats[2] = if def.is_land() { 1.0 } else { 0.0 };
    feats[3] = if def.is_planeswalker() { 1.0 } else { 0.0 };
    feats[4] = def.power.max(0) as f32 / 8.0;
    feats[5] = def.toughness.max(0) as f32 / 8.0;
    // v8: the two permanent classes the round-4 flags left in one
    // undifferentiated "none of the above" bucket. The embedding carries
    // the type for in-vocab cards; tokens and off-vocab cards live on
    // these bits alone, and a modern board is mostly made of them.
    if !ablated(ABLATE_V8) {
        feats[53] = if def.is_artifact() { 1.0 } else { 0.0 };
        feats[54] = if def.is_enchantment() { 1.0 } else { 0.0 };
    }
    // Evasion/combat keywords, granted ones included (`has_keyword`
    // reads printed + granted lists; the granted lists are simply empty
    // off the battlefield). First and double strike share a flag — for a
    // value function they mean the same thing at this resolution.
    let kb = object_keyword_bits(c);
    debug_assert_eq!(
        kb,
        {
            let mut want = 0u16;
            for (bit, kw) in [
                (okw::FLYING, Keyword::Flying),
                (okw::REACH, Keyword::Reach),
                (okw::MENACE, Keyword::Menace),
                (okw::DEATHTOUCH, Keyword::Deathtouch),
                (okw::LIFELINK, Keyword::Lifelink),
                (okw::TRAMPLE, Keyword::Trample),
                (okw::STRIKE, Keyword::FirstStrike),
                (okw::STRIKE, Keyword::DoubleStrike),
                (okw::VIGILANCE, Keyword::Vigilance),
                (okw::HASTE, Keyword::Haste),
                (okw::INDESTRUCTIBLE, Keyword::Indestructible),
                (okw::DEFENDER, Keyword::Defender),
            ] {
                if c.has_keyword(&kw) {
                    want |= bit;
                }
            }
            if any_keyword(c, is_hard_to_target) {
                want |= okw::HARD_TO_TARGET;
            }
            if any_keyword(c, is_hard_to_block) {
                want |= okw::HARD_TO_BLOCK;
            }
            want
        },
        "object_keyword_bits drifted from has_keyword / any_keyword",
    );
    for (i, bit) in [
        okw::FLYING,
        okw::REACH,
        okw::MENACE,
        okw::DEATHTOUCH,
        okw::LIFELINK,
        okw::TRAMPLE,
        okw::STRIKE,
        okw::VIGILANCE,
    ]
    .iter()
    .enumerate()
    {
        if kb & bit != 0 {
            feats[12 + i] = 1.0;
        }
    }
    // Colour requirement, printed. `cmc` alone said a card costs four; it
    // could not say the four was {2}{G}{G} in a deck with three Forests.
    if !ablated(ABLATE_CASTABILITY) {
        for col in def.cost.colored_symbols() {
            feats[20 + color_index(col)] += 1.0 / 2.0;
        }
    }
    // 25/26 (castable now / next turn) are hand-only and filled by the
    // caller, which is the only place that knows the seat's mana.
    // Multiplicity: one copy unless the library encoder says otherwise.
    feats[27] = 1.0 / 4.0;
    // An aura or equipment is a card whose whole value is an edge; the
    // printed-type flag lets the net treat "attachment in hand" as a
    // different kind of spell before any edge exists.
    if !ablated(ABLATE_RELATIONS) && (def.is_aura() || def.is_equipment()) {
        feats[35] = 1.0;
    }
    // Keyword classes (round 28) the round-4 evasion flags don't carry.
    // Mostly redundant with the card embedding for in-vocab cards; this
    // is for tokens (index 0) and granted keywords, which the embedding
    // can never see. Coarse by design: every flavour of hexproof,
    // protection and ward is one "hard to target" bit, every
    // can't-be-blocked variant one "hard to block" bit — a value
    // function trades on the class, not the fine print.
    if !ablated(ABLATE_KW) {
        for (i, bit) in [okw::HASTE, okw::INDESTRUCTIBLE, okw::DEFENDER].iter().enumerate() {
            // 40 haste, 42 indestructible, 44 defender.
            if kb & bit != 0 {
                feats[40 + 2 * i] = 1.0;
            }
        }
        // The Indestructible *counter* (CR 122.1) is handled in
        // `encode_battlefield_object_into`'s single counter walk, not
        // here — a second `counter_count` scan per object in every zone
        // is what that walk exists to avoid, and counters only sit on
        // battlefield objects anyway.
        if c.ward().is_some() || kb & okw::HARD_TO_TARGET != 0 {
            feats[41] = 1.0;
        }
        if kb & okw::HARD_TO_BLOCK != 0 {
            feats[43] = 1.0;
        }
    }
    // Memoized on the card object — `index_of` is a hash lookup and the
    // actor asks it once per encoded object plus once per library card,
    // 438,318 times a sixty-game run at ~49 Ir. See `CardData::vocab_index`.
    out.card = c.vocab_index(|d| vocab.index_of(d.name));
}

/// Any printed or EOT-granted keyword matching `pred`, minus removals.
/// Keyword *counters* are skipped — [`CardInstance::has_keyword`] covers
/// them for exact variants, and a counter granting a parametrized
/// keyword class is beyond this resolution.
fn any_keyword(c: &CardInstance, pred: fn(&crate::card::Keyword) -> bool) -> bool {
    c.definition
        .keywords
        .iter()
        .chain(c.granted_keywords_eot.iter())
        .filter(|k| !c.removed_keywords.contains(k) && !c.removed_keywords_eot.contains(k))
        .any(pred)
}

/// Hexproof, shroud, and protection in all their flavours. Ward is
/// checked separately through [`CardInstance::ward`], which already
/// reads grants.
fn is_hard_to_target(k: &crate::card::Keyword) -> bool {
    use crate::card::Keyword::*;
    matches!(
        k,
        Hexproof
            | HexproofFromColor(_)
            | HexproofFromMonocolored
            | HexproofFromMulticolored
            | HexproofExceptColors(_)
            | HexproofFromAbilities
            | Shroud
            | Protection(_)
            | ProtectionFromColoredSpells
            | ProtectionFromSpells
            | ProtectionFromCreatures
            | ProtectionFromMatching(_)
            | ProtectionFromCreatureType(_)
            | ProtectionFromSpellSubtype(_)
            | ProtectionFromManaValueExcept(_)
            | ProtectionFromMulticolored
            | ProtectionFromMonocolored
            | ProtectionFromCardType(_)
            | ProtectionFromInstants
            | ProtectionFromEverything
            | ProtectionFromOwnColors
    )
}

/// The can't-be-blocked family beyond the round-4 evasion flags (menace
/// and flying carry their own bits already).
fn is_hard_to_block(k: &crate::card::Keyword) -> bool {
    use crate::card::Keyword::*;
    matches!(
        k,
        Unblockable
            | Shadow
            | Horsemanship
            | Fear
            | Intimidate
            | Skulk
            | Landwalk(_)
            | LandwalkFiltered(_)
            | DomainLandwalk
    )
}

/// Battlefield objects add live state on top of the printed features:
/// effective P/T net of damage, the damage itself and the part of the
/// P/T that expires at cleanup, tapped, summoning sickness, loyalty,
/// SOS prepared status, attacking, and counters.
fn encode_battlefield_object_into(
    g: &GameState,
    c: &CardInstance,
    vocab: &Vocab,
    o: &mut EncodedObject,
) {
    encode_card_object_into(c, vocab, o);
    let f = &mut o.feats;
    f[6] = if c.tapped { 1.0 } else { 0.0 };
    f[7] = if c.summoning_sick { 1.0 } else { 0.0 };
    f[10] = if g.attacking.iter().any(|a| a.attacker == c.id) { 1.0 } else { 0.0 };
    f[11] = if c.is_token { 1.0 } else { 0.0 };
    // Layer-resolved truth (modern precondition 2). The base pass above
    // wrote the printed/instance view; a battlefield object overwrites it
    // with the computed one, so anthems and equipment pumps reach P/T,
    // type changes reach the type flags, and static keyword grants —
    // which never touch the instance fields the base walk reads — reach
    // every keyword-derived feature. `ComputedPermanent.keywords()` is
    // the final word (printed + EOT + CR 122.1b counters, all folded by
    // the gather, minus removals and lose-all), so overwriting rather
    // than OR-ing is what makes removals stick. One memoized layer pass
    // per permanent inside `encode_state`'s frozen scope; the raw
    // fallback is unreachable for a real battlefield walk and exists so
    // a malformed synthetic state degrades instead of panicking.
    match g.computed_permanent_on(c) {
        Some(cp) => {
            use crate::card::CardType;
            f[1] = if cp.card_types().contains(&CardType::Creature) { 1.0 } else { 0.0 };
            f[2] = if cp.card_types().contains(&CardType::Land) { 1.0 } else { 0.0 };
            f[3] = if cp.card_types().contains(&CardType::Planeswalker) { 1.0 } else { 0.0 };
            if !ablated(ABLATE_V8) {
                f[53] = if cp.card_types().contains(&CardType::Artifact) { 1.0 } else { 0.0 };
                f[54] =
                    if cp.card_types().contains(&CardType::Enchantment) { 1.0 } else { 0.0 };
            }
            f[4] = cp.power.max(0) as f32 / 8.0;
            f[5] = (cp.toughness - c.damage as i32).max(0) as f32 / 8.0;
            let mut kb = 0u16;
            let mut warded = false;
            for k in cp.keywords() {
                kb |= okw_exact_bit(k);
                if is_hard_to_target(k) {
                    kb |= okw::HARD_TO_TARGET;
                }
                if is_hard_to_block(k) {
                    kb |= okw::HARD_TO_BLOCK;
                }
                if matches!(k, crate::card::Keyword::Ward(..)) {
                    warded = true;
                }
            }
            for (i, bit) in [
                okw::FLYING,
                okw::REACH,
                okw::MENACE,
                okw::DEATHTOUCH,
                okw::LIFELINK,
                okw::TRAMPLE,
                okw::STRIKE,
                okw::VIGILANCE,
            ]
            .iter()
            .enumerate()
            {
                f[12 + i] = if kb & bit != 0 { 1.0 } else { 0.0 };
            }
            if !ablated(ABLATE_KW) {
                for (i, bit) in
                    [okw::HASTE, okw::INDESTRUCTIBLE, okw::DEFENDER].iter().enumerate()
                {
                    f[40 + 2 * i] = if kb & bit != 0 { 1.0 } else { 0.0 };
                }
                f[41] = if warded || kb & okw::HARD_TO_TARGET != 0 { 1.0 } else { 0.0 };
                f[43] = if kb & okw::HARD_TO_BLOCK != 0 { 1.0 } else { 0.0 };
                // The Indestructible *counter* lives in `counters`, not the
                // layer pass; the walk below re-ORs it into f[42].
            }
        }
        None => {
            f[4] = c.power().max(0) as f32 / 8.0;
            f[5] = (c.toughness() - c.damage as i32).max(0) as f32 / 8.0;
        }
    }
    // ONE walk of the counter bag for all eight counter slots. `CounterBag`
    // is a `Vec<(CounterType, u32)>` and `counter_count` is a linear scan of
    // it, so the seven calls this replaces (loyalty, prepared, and the five
    // kinds below) each re-walked the same list — 530,194 of the actor's
    // 864,830 `counter_count` calls, 5.57 M Ir, and most permanents carry no
    // counters at all so nearly all of it was call overhead on an empty
    // `Vec`. The slots are written to zero first rather than relying on a
    // freshly-defaulted object, so the function's output does not depend on
    // what the caller handed it.
    //
    // Feature 34 sums every kind except loyalty/prepared — unconditionally:
    // it is battlefield state like damage or tapped, not a relation. It
    // shipped inside the round-12 block and rode `rel`'s ablation bit, so
    // every `--ablate rel` arm ever run was measuring "relations plus
    // counters", attributable to neither. P/T counters reach the net twice
    // — through effective P/T and through 48/49 — which is deliberate: a
    // 3/3 that is a 2/2 plus a counter dies differently to bounce and
    // counter-hate than a printed 3/3 does.
    //
    // 48..53 are the kinds the SoS pool puts on permanents (+1/+1 by a wide
    // margin, then stun, Page and Growth) plus -1/-1, which the pool never
    // uses but every other format does; stun, Page and Growth had no path to
    // the net at all before them.
    let want_kinds = !ablated(ABLATE_COUNTERS);
    let want_kw = !ablated(ABLATE_KW);
    let want_v8 = !ablated(ABLATE_V8);
    f[8] = 0.0;
    f[9] = 0.0;
    if want_kinds {
        f[48..53].fill(0.0);
    }
    if want_v8 {
        f[55..59].fill(0.0);
    }
    let mut special = 0u32;
    for (kind, n) in c.counters.iter() {
        match kind {
            CounterType::Loyalty => f[8] = *n as f32 / 8.0,
            CounterType::Prepared => {
                if *n > 0 {
                    f[9] = 1.0;
                }
            }
            _ => special += *n,
        }
        // CR 122.1 / 702.12: an Indestructible *counter* grants the
        // ability (`is_indestructible` reads it), and the keyword walk in
        // `encode_card_object_into` can't see it — it lives here, not in
        // `keyword_counters`. Same ablation bit as the keyword flag it
        // completes; rides this walk so it costs no second scan.
        if want_kw && *n > 0 && matches!(kind, CounterType::Indestructible) {
            f[42] = 1.0;
        }
        if want_kinds {
            match kind {
                CounterType::PlusOnePlusOne => f[48] = *n as f32 / 4.0,
                CounterType::MinusOneMinusOne => f[49] = *n as f32 / 4.0,
                CounterType::Stun => f[50] = *n as f32 / 2.0,
                CounterType::Page => f[51] = *n as f32 / 3.0,
                CounterType::Growth => f[52] = *n as f32 / 3.0,
                _ => {}
            }
        }
        // v8: the kinds a modern pool trades on that fell into feature
        // 34's undifferentiated sum. A saga's chapter IS its state (a
        // stun counter and chapter III encoded identically); Chalice on
        // 1 and on 3 are different cards. Scales are "how many before
        // this stops mattering", per the house rule.
        if want_v8 {
            match kind {
                CounterType::Lore => f[55] = *n as f32 / 3.0,
                CounterType::Charge => f[56] = *n as f32 / 4.0,
                CounterType::Shield => f[57] = *n as f32 / 2.0,
                CounterType::Finality => f[58] = *n as f32 / 2.0,
                _ => {}
            }
        }
    }
    f[34] = special as f32 / 4.0;
    // Expiry (round 40). Features 4/5 encode the board as if nothing
    // reverts: 5 is toughness *net of* damage, so a 4/4 with three
    // damage read as a 4/1 though it is whole again at cleanup, and a
    // 5/5 off a combat trick read as a printed 5/5. Both matter to
    // exactly the decision a value function is asked for — whether a
    // block or a burn spell kills. `power_bonus`/`toughness_bonus` are
    // the until-end-of-turn deltas specifically; permanent pumps live
    // in `perm_*` and are correctly invisible here.
    if !ablated(ABLATE_EXPIRY) {
        f[45] = c.damage as f32 / 8.0;
        f[46] = c.power_bonus as f32 / 4.0;
        f[47] = c.toughness_bonus as f32 / 4.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::player::Player;
    use crabomination_nn::NUM_GROUPS;

    /// The ablation flag has to be process-global — actor threads must see
    /// what `main` set — and cargo runs the tests in this module as
    /// parallel threads of one process. So every test that encodes takes
    /// this lock: without it, `ablation_zeroes_exactly_the_block_it_names`
    /// would blank the library group underneath whichever test happened to
    /// be running beside it.
    static ENCODE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn encode_guard() -> std::sync::MutexGuard<'static, ()> {
        // A panicking test poisons the lock; that failure is already
        // reported, and propagating it would mask every other test here.
        ENCODE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Switch the named blocks off, panicking on a name the encoder
    /// doesn't know — inside a test an unknown block is a typo, and
    /// swallowing it would make the assertions below pass vacuously.
    fn off(blocks: &[&str]) {
        set_encode_ablation_off(blocks).expect("known block names");
    }

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    /// The frozen-index contract, which is what keeps a trained net loadable
    /// across a card addition: a snapshot name owns `position + 1` and a
    /// pool name outside the snapshot lands strictly after all of them.
    ///
    /// This is a regression test for the defect that killed all seven
    /// committed deck nets — indices derived from the sorted pool, so one
    /// inserted card shifted every later row.
    #[test]
    fn vocab_indices_come_from_the_frozen_snapshot() {
        use crate::server::vocab_snapshot::VOCAB_SNAPSHOT;
        let v = Vocab::sos_sealed();
        for (i, name) in VOCAB_SNAPSHOT.iter().enumerate() {
            assert_eq!(
                v.index_of(name),
                (i + 1) as u16,
                "{name} moved off its frozen index — every net trained against \
                 it now means a different card"
            );
        }
        assert!(v.size() > VOCAB_SNAPSHOT.len());
        // Anything the pool has that the snapshot does not sits after it,
        // and nothing collides.
        let mut seen: std::collections::BTreeSet<u16> = std::collections::BTreeSet::new();
        for f in crate::draft::sos_draft_pool() {
            let idx = v.index_of(f().name);
            assert_ne!(idx, 0, "pool card missing from vocab");
            assert!(seen.insert(idx) || idx <= VOCAB_SNAPSHOT.len() as u16);
        }
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            assert_ne!(v.index_of(basic), 0, "{basic} missing from vocab");
        }
    }

    /// A card added to the SOS set has to be written into the snapshot.
    ///
    /// The self-assigning fallback keeps the build working, but the index it
    /// hands out is sorted order over the *unsnapshotted* names — so a
    /// second addition reshuffles it and a net trained in between means the
    /// wrong card. That is the original defect, one generation later, and
    /// two card additions is the whole distance to it. Fail here instead,
    /// where the fix is one line.
    #[test]
    fn vocab_covers_the_sos_pool() {
        use crate::server::vocab_snapshot::VOCAB_SNAPSHOT;
        let frozen: std::collections::BTreeSet<&str> = VOCAB_SNAPSHOT.iter().copied().collect();
        let v = Vocab::sos_sealed();
        let mut missing: Vec<(u16, &str)> = crate::draft::sos_draft_pool()
            .iter()
            .map(|f| f().name)
            .chain(["Plains", "Island", "Swamp", "Mountain", "Forest"])
            .filter(|n| !frozen.contains(n))
            .map(|n| (v.index_of(n), n))
            .collect();
        missing.sort_unstable();
        missing.dedup();
        assert!(
            missing.is_empty(),
            "these SOS names have no frozen index: {:?}\n\
             Append them to `server/vocab_snapshot.rs` in that order, at the END of the array \
             — index k is the embedding row every trained net learned for that card.",
            missing.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
        );
    }

    /// The cube pool's names have frozen indices too (appended after the
    /// SOS seed, 2026-08-30). Same defect class as the SOS sibling above:
    /// an unsnapshotted name self-assigns from sorted order over the
    /// *unsnapshotted* set, so the next append reshuffles it and a net
    /// trained in between means the wrong card.
    #[test]
    fn vocab_covers_the_cube_pool() {
        use crate::server::vocab_snapshot::VOCAB_SNAPSHOT;
        let frozen: std::collections::BTreeSet<&str> = VOCAB_SNAPSHOT.iter().copied().collect();
        let v = Vocab::sos_sealed();
        let mut missing: Vec<(u16, &str)> = crate::cube::cube_pool_all()
            .iter()
            .map(|f| f().name)
            .filter(|n| !frozen.contains(n))
            .map(|n| (v.index_of(n), n))
            .collect();
        missing.sort_unstable();
        missing.dedup();
        assert!(
            missing.is_empty(),
            "these cube names have no frozen index: {:?}\n\
             Append them to `server/vocab_snapshot.rs` in that order, at the END of the array \
             — index k is the embedding row every trained net learned for that card.",
            missing.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
        );
    }

    /// The freeze boundary itself: position 162 is the last pre-freeze
    /// name, and `FROZEN_VOCAB_SIZE` is pinned to it as a literal. An
    /// insertion anywhere in the seed segment moves this name and every
    /// committed net with it — this is the loud version of that failure.
    #[test]
    fn the_freeze_boundary_is_pinned() {
        use crate::server::vocab_snapshot::{FROZEN_VOCAB_SIZE, VOCAB_SNAPSHOT};
        assert_eq!(FROZEN_VOCAB_SIZE, 164);
        assert_eq!(
            VOCAB_SNAPSHOT[FROZEN_VOCAB_SIZE - 2],
            "Zimone's Experiment",
            "the last pre-freeze name moved — an entry was inserted or removed \
             in the seed segment, and every committed net's rows now mean \
             different cards"
        );
    }

    /// The snapshot itself: no duplicate name, because two entries with the
    /// same name would alias two cards onto one embedding row and the later
    /// one would win silently.
    #[test]
    fn vocab_snapshot_has_no_duplicates() {
        use crate::server::vocab_snapshot::VOCAB_SNAPSHOT;
        let uniq: std::collections::BTreeSet<&str> = VOCAB_SNAPSHOT.iter().copied().collect();
        assert_eq!(uniq.len(), VOCAB_SNAPSHOT.len(), "duplicate name in VOCAB_SNAPSHOT");
    }

    /// The pointer cache has to actually *hit* on the pointer a real card
    /// definition carries.
    ///
    /// It is a pure optimization, so nothing else in the suite can tell the
    /// difference between a cache that answers every lookup and one that
    /// misses every lookup and quietly falls through to the string map. The
    /// day a pool factory builds its name at run time — a `format!`, a
    /// `String` field, a name assembled from a set code — the hit rate goes
    /// to zero and the encoder gets slower with no other symptom. Assert the
    /// hit here, where the message says what happened.
    #[test]
    fn the_vocab_pointer_cache_hits_on_the_pool_and_misses_off_it() {
        let v = Vocab::sos_sealed();
        let hit = |name: &str| {
            v.by_ptr.get(&(name.as_ptr() as usize)).is_some_and(|&(len, _)| len == name.len())
        };
        for f in crate::draft::sos_draft_pool() {
            let def = f();
            assert!(
                hit(def.name),
                "{} misses the vocab pointer cache — its name is no longer the \
                 same `&'static str` the pool factory hands out, so `index_of` \
                 is back to hashing a string per encoded object",
                def.name
            );
            assert_eq!(v.index_of(def.name), v.map.get(def.name).copied().unwrap_or(0));
        }
        // A name that never came from a factory falls through to the map,
        // and the map is still the only answer.
        let token = String::from("Zombie");
        assert!(!hit(&token));
        assert_eq!(v.index_of(&token), v.map.get("Zombie").copied().unwrap_or(0));
        assert_eq!(v.index_of("Definitely Not A Card"), 0);
    }

    #[test]
    fn sos_vocab_is_substantial_and_stable() {
        let v = Vocab::sos_sealed();
        // Sanity range, not an exact count: the pools may grow a card or
        // two, but a collapse means the pool wiring broke. The floor says
        // both pools arrived (the SOS seed alone is 164, the cube append
        // took it past 2,000); the ceiling catches a runaway enumeration.
        assert!(v.size() > 2000 && v.size() < 5000, "vocab size {}", v.size());
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            assert_ne!(v.index_of(basic), 0, "{basic} missing from vocab");
        }
        assert_eq!(v.index_of("Definitely Not A Card"), 0);
        // Indices are a function of the sorted name list — two builds agree.
        let v2 = Vocab::sos_sealed();
        assert_eq!(v.index_of("Plains"), v2.index_of("Plains"));
    }

    #[test]
    fn encode_reads_the_position_seat_relative() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        g.players[0].life = 12;
        g.players[1].life = 20;
        g.turn_number = 3;
        g.active_player_idx = 1;

        // A bear for seat 1, tapped; a card in seat 0's hand and graveyard.
        let bear = catalog::grizzly_bears();
        let mut inst = CardInstance::new(crate::card::CardId(901), bear, 1);
        inst.controller = 1;
        inst.tapped = true;
        g.battlefield.push(inst);
        let hand_card = CardInstance::new(crate::card::CardId(902), catalog::grizzly_bears(), 0);
        g.players[0].hand.push(hand_card);
        let dead = CardInstance::new(crate::card::CardId(903), catalog::grizzly_bears(), 0);
        g.players[0].graveyard.push(dead);

        let s0 = encode_state(&g, 0, &vocab);
        assert_eq!(s0.group_len(G_BF_SELF), 0);
        assert_eq!(s0.group_len(G_BF_OPP), 1);
        assert_eq!(s0.group_len(G_HAND_SELF), 1);
        assert_eq!(s0.group_len(G_GY_SELF), 1);
        assert_eq!(s0.group_len(G_GY_OPP), 0);
        assert!((s0.global[0] - 12.0 / 20.0).abs() < 1e-6);
        assert!((s0.global[1] - 1.0).abs() < 1e-6);
        assert_eq!(s0.global[9], 0.0, "seat 0 is not the active player");
        let opp_bear = &s0.group(G_BF_OPP)[0];
        assert_eq!(opp_bear.feats[6], 1.0, "tapped flag");
        assert!((opp_bear.feats[4] - 2.0 / 8.0).abs() < 1e-6, "2 power");
        // Grizzly Bears is off the SOS vocab — unknown index, features carry it.
        assert_eq!(opp_bear.card, vocab.index_of("Grizzly Bears"));

        // The same position from the other seat mirrors every self/opp pair.
        let s1 = encode_state(&g, 1, &vocab);
        assert_eq!(s1.group_len(G_BF_SELF), 1);
        assert_eq!(s1.group_len(G_BF_OPP), 0);
        assert_eq!(s1.group_len(G_HAND_SELF), 0, "opponent hand is hidden");
        assert!((s1.global[0] - 1.0).abs() < 1e-6);
        assert!((s1.global[1] - 12.0 / 20.0).abs() < 1e-6);
        assert_eq!(s1.global[9], 1.0);
        assert_eq!(s1.group_len(G_GY_SELF), 0);
        assert_eq!(s1.group_len(G_GY_OPP), 1);

        // Empty groups exist but are empty, never dropped.
        assert_eq!(s0.groups().count(), NUM_GROUPS);
        assert_eq!(s0.groups().map(|g| g.len()).sum::<usize>(), s0.len());
    }

    /// A land on the battlefield, `n` of them, controlled by `seat`.
    fn add_lands(g: &mut GameState, seat: usize, land: fn() -> crate::card::CardDefinition, n: u32, id0: u32) {
        for k in 0..n {
            let mut inst = CardInstance::new(crate::card::CardId(id0 + k), land(), seat);
            inst.controller = seat;
            g.battlefield.push(inst);
        }
    }

    #[test]
    fn the_library_encodes_as_a_deduplicated_multiset() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        // Three Forests and one Island, pushed interleaved.
        for (k, f) in [catalog::forest, catalog::island, catalog::forest, catalog::forest]
            .into_iter()
            .enumerate()
        {
            g.players[0].library.push(CardInstance::new(crate::card::CardId(700 + k as u32), f(), 0));
        }
        let s = encode_state(&g, 0, &vocab);
        let lib = &s.group(G_LIB_SELF);
        assert_eq!(lib.len(), 2, "four cards, two distinct names");
        // Sorted by vocabulary index, and the counts land in feat 27.
        let by_name: Vec<(u16, f32)> = lib.iter().map(|o| (o.card, o.feats[27])).collect();
        assert!(by_name[0].0 < by_name[1].0, "emitted in vocab-index order");
        let forest = by_name.iter().find(|e| e.0 == vocab.index_of("Forest")).unwrap();
        let island = by_name.iter().find(|e| e.0 == vocab.index_of("Island")).unwrap();
        assert!((forest.1 - 3.0 / 4.0).abs() < 1e-6, "three Forests");
        assert!((island.1 - 1.0 / 4.0).abs() < 1e-6, "one Island");
        // The opponent's library is never encoded — only its size.
        assert_eq!(encode_state(&g, 1, &vocab).group_len(G_LIB_SELF), 0);
    }

    /// The library is a *set* to the net: the shuffle is hidden
    /// information and must not survive encoding, whatever pooling or
    /// attention does with the group downstream.
    #[test]
    fn library_order_does_not_reach_the_encoding() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut a = two_player_game();
        let mut b = two_player_game();
        let deck = [catalog::forest, catalog::island, catalog::forest, catalog::plains];
        for (k, f) in deck.iter().enumerate() {
            a.players[0].library.push(CardInstance::new(crate::card::CardId(700 + k as u32), f(), 0));
        }
        for (k, f) in deck.iter().rev().enumerate() {
            b.players[0].library.push(CardInstance::new(crate::card::CardId(800 + k as u32), f(), 0));
        }
        assert_eq!(encode_state(&a, 0, &vocab), encode_state(&b, 0, &vocab));
    }

    #[test]
    fn castability_flags_read_the_seat_mana() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        // Grizzly Bears is {1}{G}. One Forest is not enough mana; one
        // Forest and one Island is; two Islands is enough *mana* but the
        // wrong colour, which is the case `cmc` alone could never see.
        let bear = || CardInstance::new(crate::card::CardId(902), catalog::grizzly_bears(), 0);
        g.players[0].hand.push(bear());

        add_lands(&mut g, 0, catalog::forest, 1, 100);
        let one_forest = encode_state(&g, 0, &vocab);
        assert_eq!(one_forest.group(G_HAND_SELF)[0].feats[25], 0.0, "one land, two-mana spell");
        assert_eq!(one_forest.group(G_HAND_SELF)[0].feats[26], 1.0, "castable after a land drop");

        add_lands(&mut g, 0, catalog::island, 1, 200);
        let forest_island = encode_state(&g, 0, &vocab);
        assert_eq!(forest_island.group(G_HAND_SELF)[0].feats[25], 1.0, "{{1}}{{G}} off Forest+Island");

        let mut wrong = two_player_game();
        wrong.players[0].hand.push(bear());
        add_lands(&mut wrong, 0, catalog::island, 2, 300);
        let two_islands = encode_state(&wrong, 0, &vocab);
        assert_eq!(
            two_islands.group(G_HAND_SELF)[0].feats[25], 0.0,
            "two mana but no green source"
        );
        // Next turn it comes online: the assumed land drop is optimistic
        // about colour, so it covers the {G} and an Island pays the {1}.
        // That optimism is the documented approximation — a seat with no
        // green land left in its library will read as castable here.
        assert_eq!(two_islands.group(G_HAND_SELF)[0].feats[26], 1.0);

        // Printed colour pips ride along on every object regardless of zone.
        let g_pip = two_islands.group(G_HAND_SELF)[0].feats[20 + 4];
        assert!((g_pip - 0.5).abs() < 1e-6, "one green pip");
    }

    #[test]
    fn available_mana_globals_cover_both_seats() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        add_lands(&mut g, 0, catalog::forest, 2, 100);
        add_lands(&mut g, 1, catalog::island, 3, 200);
        // One of the opponent's Islands is tapped: available mana is not
        // the same question as permanents controlled.
        g.battlefield.last_mut().unwrap().tapped = true;

        let s = encode_state(&g, 0, &vocab);
        assert!((s.global[24 + 4] - 2.0 / 6.0).abs() < 1e-6, "two green sources");
        assert_eq!(s.global[24 + 1], 0.0, "no blue sources");
        assert!((s.global[29] - 2.0 / 6.0).abs() < 1e-6, "two untapped sources total");
        assert!((s.global[30 + 1] - 2.0 / 6.0).abs() < 1e-6, "opponent has two untapped Islands");
        assert!((s.global[35] - 2.0 / 6.0).abs() < 1e-6);
    }

    /// The ablation control blanks each block and leaves the other
    /// standing, so a run with `--ablate lib` differs from the full
    /// encoder in the library group and nothing else.
    ///
    /// Serialized against the other tests by construction: it is the only
    /// one that touches the process-global, and it restores it.
    #[test]
    fn ablation_zeroes_exactly_the_block_it_names() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        g.players[0].hand.push(CardInstance::new(
            crate::card::CardId(902),
            catalog::grizzly_bears(),
            0,
        ));
        g.players[0]
            .library
            .push(CardInstance::new(crate::card::CardId(700), catalog::forest(), 0));
        add_lands(&mut g, 0, catalog::forest, 2, 100);

        let full = encode_state(&g, 0, &vocab);
        assert_eq!(full.group_len(G_LIB_SELF), 1);
        assert_eq!(full.group(G_HAND_SELF)[0].feats[25], 1.0);
        assert!(full.global[24 + 4] > 0.0);

        off(&["lib"]);
        let no_lib = encode_state(&g, 0, &vocab);
        assert_eq!(no_lib.group_len(G_LIB_SELF), 0, "library group is empty");
        assert_eq!(no_lib.group(G_HAND_SELF)[0].feats[25], 1.0, "castability survives");
        assert!(no_lib.global[24 + 4] > 0.0);

        off(&["cast"]);
        let no_cast = encode_state(&g, 0, &vocab);
        assert_eq!(no_cast.group_len(G_LIB_SELF), 1, "library survives");
        assert_eq!(no_cast.group(G_HAND_SELF)[0].feats[25], 0.0, "castable-now zeroed");
        assert_eq!(no_cast.group(G_HAND_SELF)[0].feats[26], 0.0, "castable-next zeroed");
        assert_eq!(no_cast.group(G_HAND_SELF)[0].feats[20 + 4], 0.0, "pips zeroed");
        for i in 24..36 {
            assert_eq!(no_cast.global[i], 0.0, "available-mana global {i} zeroed");
        }
        // Everything outside the two blocks is untouched.
        assert_eq!(no_cast.global[..24], full.global[..24]);

        // The relations bit blanks the round-12 block and only it: a
        // blocked attacker loses its flag, the stack groups empty, the
        // other blocks stand.
        g.attacking.push(crate::game::types::Attack {
            attacker: crate::card::CardId(100),
            target: crate::game::types::AttackTarget::Player(1),
        });
        g.block_map.insert(crate::card::CardId(101), smallvec::smallvec![crate::card::CardId(100)]);
        off(&[]);
        let with_rel = encode_state(&g, 0, &vocab);
        assert!(with_rel.group(G_BF_SELF).iter().any(|o| o.feats[29] == 1.0), "blocked flag on");
        off(&["rel"]);
        let no_rel = encode_state(&g, 0, &vocab);
        assert!(no_rel.group(G_BF_SELF).iter().all(|o| o.feats[29] == 0.0), "blocked flag off");
        assert_eq!(no_rel.group_len(G_LIB_SELF), 1, "library survives rel ablation");
        assert_eq!(no_rel.group(G_HAND_SELF)[0].feats[25], 1.0, "castability survives");

        // The combat bit blanks the round-28 combat structure and only
        // it. IDs 100/101 are the Forests above — power 0, so the
        // endpoint sums stay 0 and the phase one-hot is the readable
        // difference.
        g.step = TurnStep::DeclareBlockers;
        let with_combat = encode_state(&g, 0, &vocab);
        assert_eq!(with_combat.global[37], 1.0, "declare-blockers one-hot on");
        off(&["combat"]);
        let no_combat = encode_state(&g, 0, &vocab);
        assert_eq!(no_combat.global[37], 0.0, "declare-blockers one-hot off");
        assert_eq!(no_combat.group_len(G_LIB_SELF), 1, "library survives combat ablation");
        assert!(
            no_combat.group(G_BF_SELF).iter().any(|o| o.feats[29] == 1.0),
            "relation flags survive combat ablation"
        );

        // The kw bit blanks the keyword classes and exile counts.
        g.exile.push(CardInstance::new(crate::card::CardId(950), catalog::grizzly_bears(), 0));
        off(&[]);
        let with_kw = encode_state(&g, 0, &vocab);
        assert!((with_kw.global[41] - 1.0 / 10.0).abs() < 1e-6, "exile count on");
        off(&["kw"]);
        let no_kw = encode_state(&g, 0, &vocab);
        assert_eq!(no_kw.global[41], 0.0, "exile count off");
        assert_eq!(no_kw.global[37], 1.0, "combat block survives kw ablation");

        // Round 40. History is globals-only, expiry and counters are
        // object-only, so each blanks a disjoint slice — and the
        // combined `hist,exp,ctr` ablation is v6 parity, which is the
        // control arm the gate actually runs.
        g.players[0].life_gained_this_turn = 5;
        // The fixture's battlefield is Forests, so the one creature
        // below is unambiguous to `find` by the is-creature flag.
        assert!(!g.battlefield.iter().any(|c| c.definition.is_creature()));
        let mut pumped = CardInstance::new(crate::card::CardId(960), catalog::grizzly_bears(), 0);
        pumped.controller = 0;
        pumped.damage = 1;
        pumped.power_bonus = 2;
        pumped.toughness_bonus = 2;
        pumped.add_counters(CounterType::PlusOnePlusOne, 1);
        pumped.add_counters(CounterType::Stun, 1);
        g.battlefield.push(pumped);
        let find = |s: &EncodedState| {
            s.group(G_BF_SELF).iter().find(|o| o.feats[1] == 1.0).cloned().expect("the creature")
        };

        off(&[]);
        let v7 = encode_state(&g, 0, &vocab);
        assert_eq!(v7.global[43], 1.0, "5 life gained / 5");
        let c7 = find(&v7);
        assert!((c7.feats[45] - 1.0 / 8.0).abs() < 1e-6, "one damage marked");
        assert!((c7.feats[46] - 2.0 / 4.0).abs() < 1e-6, "temporary +2 power");
        assert!((c7.feats[48] - 1.0 / 4.0).abs() < 1e-6, "one +1/+1 counter");
        assert!((c7.feats[50] - 1.0 / 2.0).abs() < 1e-6, "one stun counter");

        off(&["hist"]);
        let no_hist = encode_state(&g, 0, &vocab);
        assert_eq!(no_hist.global[43], 0.0, "life-gained global off");
        assert_eq!(no_hist.global[..43], v7.global[..43], "older globals untouched");
        assert_eq!(find(&no_hist).feats, c7.feats, "object features untouched");

        off(&["exp"]);
        let no_exp = encode_state(&g, 0, &vocab);
        let c = find(&no_exp);
        assert_eq!([c.feats[45], c.feats[46], c.feats[47]], [0.0; 3], "expiry block off");
        assert_eq!(c.feats[48], c7.feats[48], "counter block survives");
        assert_eq!(no_exp.global, v7.global, "globals untouched");

        off(&["ctr"]);
        let c = find(&encode_state(&g, 0, &vocab));
        assert_eq!(c.feats[48..53], [0.0; 5], "counter block off");
        assert_eq!(c.feats[45], c7.feats[45], "expiry block survives");
        assert_eq!(c.feats[34], c7.feats[34], "the round-12 counter sum survives");

        // Feature 34 is battlefield state, not a relation: it survives
        // `rel` too. It used to ride that block's bit, so every `rel`
        // ablation arm was really measuring "relations plus counters".
        off(&["rel"]);
        let c = find(&encode_state(&g, 0, &vocab));
        assert!((c.feats[34] - 2.0 / 4.0).abs() < 1e-6, "counter sum survives rel ablation");

        // v6 parity: exactly the round-40-and-later slots are blank and
        // nothing else moved. `v8` joins the off-list since its block
        // (2026-08-30) — the v7 gate's historical control arm was
        // `hist,exp,ctr` alone, before v8 existed to blank.
        off(&["hist", "exp", "ctr", "v8"]);
        let v6 = encode_state(&g, 0, &vocab);
        assert_eq!(v6.global[..43], v7.global[..43]);
        assert_eq!(v6.global[43..], [0.0; 14]);
        let c6 = find(&v6);
        assert_eq!(c6.feats[..45], c7.feats[..45]);
        assert_eq!(c6.feats[45..], [0.0; 14]);

        g.battlefield.pop();
        g.players[0].life_gained_this_turn = 0;
        g.step = TurnStep::PreCombatMain;
        g.exile.clear();
        off(&[]);
        assert_eq!(encode_state(&g, 0, &vocab), with_rel, "all on restores the full encoding");
    }

    /// An unknown block name is an error, not a no-op: a typo'd
    /// `--ablate` would otherwise produce a "control" that is really a
    /// second copy of the arm, and the two runs would agree for the
    /// wrong reason.
    #[test]
    fn unknown_ablation_names_are_rejected() {
        let _guard = encode_guard();
        assert!(set_encode_ablation_off(&["combat", "nonsense"]).is_err());
        // The rejected call must not have left a partial mask behind.
        assert_eq!(ABLATE.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert!(set_encode_ablation_off(&["hist", "exp", "ctr"]).is_ok());
        off(&[]);
    }

    /// The round-28 combat-structure block: counterpart P/T sums across
    /// the block edges, attack-target kind, fine phase one-hots, and
    /// unblocked incoming power.
    #[test]
    fn combat_structure_reaches_the_encoding() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 1;
        // Their 3/3 and 2/2 attack me; my two bears gang-block the 3/3;
        // the 2/2 gets through.
        let mut big = CardInstance::new(crate::card::CardId(1), catalog::hill_giant(), 1);
        big.controller = 1;
        let mut small = CardInstance::new(crate::card::CardId(2), catalog::grizzly_bears(), 1);
        small.controller = 1;
        let mut b1 = CardInstance::new(crate::card::CardId(3), catalog::grizzly_bears(), 0);
        b1.controller = 0;
        let mut b2 = CardInstance::new(crate::card::CardId(4), catalog::grizzly_bears(), 0);
        b2.controller = 0;
        b2.damage = 1;
        for c in [big, small, b1, b2] {
            g.battlefield.push(c);
        }
        for id in [1, 2] {
            g.attacking.push(crate::game::types::Attack {
                attacker: crate::card::CardId(id),
                target: crate::game::types::AttackTarget::Player(0),
            });
        }
        g.block_map.insert(crate::card::CardId(3), smallvec::smallvec![crate::card::CardId(1)]);
        g.block_map.insert(crate::card::CardId(4), smallvec::smallvec![crate::card::CardId(1)]);

        let s = encode_state(&g, 0, &vocab);
        // Both creatures are off the SOS vocab (index 0), so objects are
        // told apart by their power feature, not the card index.
        let giant = s.group(G_BF_OPP)
            .iter()
            .find(|o| (o.feats[4] - 3.0 / 8.0).abs() < 1e-6)
            .expect("the 3/3 attacker encoded");
        // The blocked 3/3 sees 2+2 power and 2+1 effective toughness
        // (one bear carries a damage) across the table.
        assert!((giant.feats[37] - 4.0 / 8.0).abs() < 1e-6, "blockers' power on the attacker");
        assert!((giant.feats[38] - 3.0 / 8.0).abs() < 1e-6, "blockers' toughness on the attacker");
        // Each bear sees the 3/3 it blocks; the damage on one of them
        // changes its own row, not the counterpart sums.
        let bears: Vec<_> =
            s.group(G_BF_SELF).iter().filter(|o| o.feats[37] > 0.0).collect();
        assert_eq!(bears.len(), 2, "both blockers carry counterpart sums");
        for b in &bears {
            assert!((b.feats[37] - 3.0 / 8.0).abs() < 1e-6);
            assert!((b.feats[38] - 3.0 / 8.0).abs() < 1e-6);
        }
        // The unblocked 2/2 aims 2 power at my life total; nothing gets
        // through at theirs. Blocks-pending one-hot set, no other.
        assert!((s.global[39] - 2.0 / 12.0).abs() < 1e-6, "incoming unblocked power");
        assert_eq!(s.global[40], 0.0);
        assert_eq!(s.global[36], 0.0);
        assert_eq!(s.global[37], 1.0);
        assert_eq!(s.global[38], 0.0);
        // Seat-relative: the same combat from the attacker's chair.
        let s1 = encode_state(&g, 1, &vocab);
        assert_eq!(s1.global[39], 0.0);
        assert!((s1.global[40] - 2.0 / 12.0).abs() < 1e-6);

        // An attack at a planeswalker flags target kind and stays out of
        // the life-total sums.
        g.attacking[1].target =
            crate::game::types::AttackTarget::Planeswalker(crate::card::CardId(99));
        let s = encode_state(&g, 0, &vocab);
        let attacker = s.group(G_BF_OPP)
            .iter()
            .find(|o| (o.feats[4] - 2.0 / 8.0).abs() < 1e-6)
            .expect("the attacking 2/2 encoded");
        assert_eq!(attacker.feats[39], 1.0, "attacking a non-player target");
        assert_eq!(s.global[39], 0.0, "walker attack leaves the life total alone");
    }

    /// CR 509.1b / 510.1c — an attacker that became blocked stays blocked
    /// after every blocker leaves combat (first-strike deaths, post-block
    /// removal): the damage step deals nothing to the player, so the
    /// encoding must not read its power as incoming. Regression for the
    /// `block_map`-only read, which fed the full power into global 39/40
    /// at exactly the settled combat states the search evaluates most.
    #[test]
    fn a_blocked_attacker_stays_blocked_when_its_blockers_leave_combat() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        g.step = TurnStep::CombatDamage;
        g.active_player_idx = 1;
        let mut atk = CardInstance::new(crate::card::CardId(1), catalog::hill_giant(), 1);
        atk.controller = 1;
        g.battlefield.push(atk);
        g.attacking.push(crate::game::types::Attack {
            attacker: crate::card::CardId(1),
            target: crate::game::types::AttackTarget::Player(0),
        });
        // Declared blocked; the blocker has since left combat — gone from
        // `block_map`, exactly what `remove_permanent_from_combat` leaves.
        g.blocked_attackers.push(crate::card::CardId(1));

        let s = encode_state(&g, 0, &vocab);
        assert_eq!(s.global[39], 0.0, "a blocked attacker's power is not incoming");
        assert_eq!(s.group(G_BF_OPP)[0].feats[29], 1.0, "and it still reads as blocked");

        // The same attacker with trample tramples everything through:
        // CR 702.19g, no live blocker left to assign lethal to.
        g.battlefield[0].granted_keywords_eot.push(crate::card::Keyword::Trample);
        let s = encode_state(&g, 0, &vocab);
        assert!((s.global[39] - 3.0 / 12.0).abs() < 1e-6, "trample: everything through");

        // With a live 2/2 blocker back in the map, only the excess over
        // its effective toughness tramples through.
        let mut blocker = CardInstance::new(crate::card::CardId(3), catalog::grizzly_bears(), 0);
        blocker.controller = 0;
        g.battlefield.push(blocker);
        g.block_map.insert(crate::card::CardId(3), smallvec::smallvec![crate::card::CardId(1)]);
        let s = encode_state(&g, 0, &vocab);
        assert!((s.global[39] - 1.0 / 12.0).abs() < 1e-6, "trample over a live blocker: excess only");
    }

    /// A trigger whose source has left the battlefield — a dies trigger,
    /// the most common trigger class on the stack — used to encode as an
    /// all-zero object: unknown card, every feature blank, even the
    /// feature-27 multiplicity baseline every other object carries. It now
    /// reads the card from the LKI snapshots or the graveyard, and the
    /// truly-unknown fallback stays on distribution.
    #[test]
    fn a_dead_trigger_source_still_encodes_its_card() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        // The dead source sits in its owner's graveyard, where a resolved
        // dies-trigger's subject actually is while the trigger waits.
        let dead = CardInstance::new(crate::card::CardId(2), catalog::hill_giant(), 1);
        g.players[1].graveyard.push(dead);
        g.stack.push(
            crate::game::types::TriggerPush::new(
                crate::card::CardId(2),
                1,
                crate::effect::Effect::VentureInto { dungeon: "Undercity".into() },
            )
            .build(),
        );

        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_STACK_OPP)[0];
        assert_eq!(o.feats[1], 1.0, "the dead source still reads as a creature");
        assert!((o.feats[4] - 3.0 / 8.0).abs() < 1e-6, "with its printed power");
        assert!((o.feats[27] - 1.0 / 4.0).abs() < 1e-6, "and the multiplicity baseline");

        // A source in no readable zone encodes unknown — but keeps the
        // baseline, not the off-distribution zero it used to carry.
        g.stack.clear();
        g.stack.push(
            crate::game::types::TriggerPush::new(
                crate::card::CardId(999),
                1,
                crate::effect::Effect::VentureInto { dungeon: "Undercity".into() },
            )
            .build(),
        );
        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_STACK_OPP)[0];
        assert_eq!(o.card, 0, "unknown card");
        assert_eq!(o.feats[4], 0.0, "no invented features");
        assert!((o.feats[27] - 1.0 / 4.0).abs() < 1e-6, "baseline survives the fallback");
    }

    /// The round-28 keyword classes: granted keywords reach the flags
    /// (the card embedding can never see a grant), removals win, and the
    /// coarse classes cover their parametrized variants.
    #[test]
    fn keyword_classes_reach_the_encoding() {
        use crate::card::Keyword;
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        let mut c = CardInstance::new(crate::card::CardId(1), catalog::grizzly_bears(), 0);
        c.controller = 0;
        c.granted_keywords_eot.push(Keyword::Haste);
        c.granted_keywords_eot.push(Keyword::Hexproof);
        c.granted_keywords_eot.push(Keyword::Shadow);
        g.battlefield.push(c);

        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_BF_SELF)[0];
        assert_eq!(o.feats[40], 1.0, "haste");
        assert_eq!(o.feats[41], 1.0, "hexproof → hard to target");
        assert_eq!(o.feats[42], 0.0, "not indestructible");
        assert_eq!(o.feats[43], 1.0, "shadow → hard to block");
        assert_eq!(o.feats[44], 0.0, "not a defender");

        // A removed keyword no longer counts, exact-variant or class.
        g.battlefield[0].removed_keywords.push(Keyword::Hexproof);
        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_BF_SELF)[0];
        assert_eq!(o.feats[41], 0.0, "removed hexproof does not flag");
        assert_eq!(o.feats[43], 1.0, "shadow unaffected");

        // CR 122.1 / 702.12 — an Indestructible *counter* grants the
        // ability (`is_indestructible` reads it) without any keyword; the
        // flag has to see it too.
        g.battlefield[0].add_counters(CounterType::Indestructible, 1);
        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_BF_SELF)[0];
        assert_eq!(o.feats[42], 1.0, "an Indestructible counter sets the flag");
    }

    /// CR 122.1b — a keyword *counter* grants the ability, and on the
    /// battlefield every keyword-derived feature reads it: the computed
    /// list (layer-aware encoding, modern precondition 2) folds counters
    /// into the layer pass, so a Hexproof counter reaches the
    /// hard-to-target class flag that `any_keyword`'s printed+granted
    /// walk could never see. That old exclusion was a resolution limit
    /// ("a counter granting a parametrized keyword class is beyond this
    /// resolution"), not a ruling — the class flags exist to say what the
    /// permanent IS, and a hexproof-countered creature is hard to target.
    /// Off-battlefield objects keep the old walk (no layers off the
    /// battlefield).
    #[test]
    fn keyword_counters_reach_the_exact_flags_and_the_class_flags() {
        use crate::card::Keyword;
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        let mut c = CardInstance::new(crate::card::CardId(1), catalog::grizzly_bears(), 0);
        c.controller = 0;
        c.keyword_counters.add(Keyword::Flying, 1);
        c.keyword_counters.add(Keyword::Hexproof, 1);
        g.battlefield.push(c);

        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_BF_SELF)[0];
        assert_eq!(o.feats[12], 1.0, "a Flying counter sets the exact flag");
        assert_eq!(o.feats[41], 1.0, "a Hexproof counter sets the class flag (CR 122.1b)");

        // A removed keyword beats a counter, same as the layer order.
        g.battlefield[0].removed_keywords_eot.push(Keyword::Flying);
        let s = encode_state(&g, 0, &vocab);
        let o = &s.group(G_BF_SELF)[0];
        assert_eq!(o.feats[12], 0.0, "removal beats the counter");
    }

    /// The static-anthem hole, closed (modern precondition 2): a
    /// continuous +1/+1 resolved through the layer system reaches
    /// effective P/T and the power totals. This was the documented #1
    /// encoder gap for a modern pool — 258 `PumpPT` statics in the
    /// catalog against the 5 the SOS deferral was priced on — and until
    /// this change the creature encoded at its unbuffed stats while the
    /// engine fought with the buffed ones.
    #[test]
    fn a_static_anthem_reaches_the_encoding() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        let mut anthem =
            CardInstance::new(crate::card::CardId(1), catalog::glorious_anthem(), 0);
        anthem.controller = 0;
        g.battlefield.push(anthem);
        let mut mine = CardInstance::new(crate::card::CardId(2), catalog::grizzly_bears(), 0);
        mine.controller = 0;
        g.battlefield.push(mine);
        let mut theirs = CardInstance::new(crate::card::CardId(3), catalog::grizzly_bears(), 1);
        theirs.controller = 1;
        g.battlefield.push(theirs);

        let s = encode_state(&g, 0, &vocab);
        let buffed = s
            .group(G_BF_SELF)
            .iter()
            .find(|o| o.feats[1] == 1.0)
            .expect("the creature encoded");
        assert!((buffed.feats[4] - 3.0 / 8.0).abs() < 1e-6, "anthem power reaches feat 4");
        assert!((buffed.feats[5] - 3.0 / 8.0).abs() < 1e-6, "anthem toughness reaches feat 5");
        let unbuffed = &s.group(G_BF_OPP)[0];
        assert!((unbuffed.feats[4] - 2.0 / 8.0).abs() < 1e-6, "their bear is not ours to buff");
        assert!((s.global[22] - 3.0 / 12.0).abs() < 1e-6, "the power total sees the anthem");
        assert!((s.global[23] - 2.0 / 12.0).abs() < 1e-6);
    }

    /// The v8 block (modern precondition 3): artifact/enchantment type
    /// bits in every zone, the modern counter kinds, land drops
    /// remaining — and the `v8` ablation bit blanks exactly this block.
    #[test]
    fn the_v8_block_reaches_the_encoding() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        g.active_player_idx = 0;
        // An artifact in hand (printed path), an enchantment with two
        // lore counters on the battlefield (computed path + counters).
        g.players[0]
            .hand
            .push(CardInstance::new(crate::card::CardId(1), catalog::cauldron_of_essence(), 0));
        let mut saga = CardInstance::new(crate::card::CardId(2), catalog::glorious_anthem(), 0);
        saga.controller = 0;
        saga.add_counters(CounterType::Lore, 2);
        saga.add_counters(CounterType::Charge, 3);
        g.battlefield.push(saga);

        let s = encode_state(&g, 0, &vocab);
        let hand = &s.group(G_HAND_SELF)[0];
        assert_eq!(hand.feats[53], 1.0, "artifact bit in hand");
        assert_eq!(hand.feats[54], 0.0);
        let bf = &s.group(G_BF_SELF)[0];
        assert_eq!(bf.feats[54], 1.0, "enchantment bit on the battlefield");
        assert_eq!(bf.feats[53], 0.0);
        assert!((bf.feats[55] - 2.0 / 3.0).abs() < 1e-6, "lore counters split out");
        assert!((bf.feats[56] - 3.0 / 4.0).abs() < 1e-6, "charge counters split out");
        assert!((bf.feats[34] - 5.0 / 4.0).abs() < 1e-6, "feature 34 still sums them");
        // Land drops: neither seat has played one; both read a standing
        // drop (`can_player_play_land` checks locks, not the turn).
        assert!((s.global[55] - 1.0 / 2.0).abs() < 1e-6, "our drop remaining");
        assert!((s.global[56] - 1.0 / 2.0).abs() < 1e-6, "their standing drop");
        g.players[0].lands_played_this_turn = 1;
        let s2 = encode_state(&g, 0, &vocab);
        assert_eq!(s2.global[55], 0.0, "spent drop reads zero");

        // The ablation bit blanks exactly this block.
        off(&["v8"]);
        let s3 = encode_state(&g, 0, &vocab);
        let bf3 = &s3.group(G_BF_SELF)[0];
        assert_eq!(bf3.feats[53..59], [0.0; 6], "v8 object feats off");
        assert_eq!([s3.global[55], s3.global[56]], [0.0; 2], "v8 globals off");
        assert_eq!(bf3.feats[34], bf.feats[34], "the round-12 counter sum survives");
        assert_eq!(s3.group(G_HAND_SELF)[0].feats[53], 0.0, "hand bit off too");
        off(&[]);
    }

    /// A static keyword grant — which never touches the instance fields
    /// the base walk reads — reaches the keyword flags. Two Knight
    /// Exemplars buff each other (+1/+1 and indestructible to OTHER
    /// Knights), so each encodes as a 3/3 with the indestructible class
    /// flag on, off nothing but the layer pass.
    #[test]
    fn a_static_keyword_grant_reaches_the_flags() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        for id in [1, 2] {
            let mut k =
                CardInstance::new(crate::card::CardId(id), catalog::knight_exemplar(), 0);
            k.controller = 0;
            g.battlefield.push(k);
        }
        let s = encode_state(&g, 0, &vocab);
        for o in s.group(G_BF_SELF) {
            assert!((o.feats[4] - 3.0 / 8.0).abs() < 1e-6, "the other's +1/+1 lands");
            assert_eq!(o.feats[42], 1.0, "granted indestructible reaches the class flag");
            assert_eq!(o.feats[18], 1.0, "printed first strike still flags");
        }
    }

    /// The round-12 relation block: attachment edges split by controller,
    /// stack targeting, and the stack groups themselves.
    #[test]
    fn relations_and_the_stack_reach_the_encoding() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        // My bear; their bear; my Pacifism on *their* bear. The encoder
        // reads the attachment edge and controllers, not the card text.
        let mut mine = CardInstance::new(crate::card::CardId(1), catalog::grizzly_bears(), 0);
        mine.controller = 0;
        let mut theirs = CardInstance::new(crate::card::CardId(2), catalog::grizzly_bears(), 1);
        theirs.controller = 1;
        let mut aura = CardInstance::new(crate::card::CardId(3), catalog::pacifism(), 0);
        aura.controller = 0;
        aura.attached_to = Some(crate::card::CardId(2));
        g.battlefield.push(mine);
        g.battlefield.push(theirs);
        g.battlefield.push(aura);
        // A trigger on the stack, controlled by seat 1 off their bear,
        // aimed at my bear.
        g.stack.push(
            crate::game::types::TriggerPush::new(
                crate::card::CardId(2),
                1,
                crate::effect::Effect::VentureInto { dungeon: "Undercity".into() },
            )
            .target(Some(crate::game::types::Target::Permanent(crate::card::CardId(1))))
            .build(),
        );

        let s = encode_state(&g, 0, &vocab);
        let me = &s.group(G_BF_SELF)[0];
        assert_eq!(me.feats[33], 1.0, "my bear is targeted by the stack");
        assert_eq!(me.feats[30], 0.0);
        let aura_obj =
            s.group(G_BF_SELF).iter().find(|o| o.feats[30] == 1.0).expect("aura is_attached");
        assert_eq!(aura_obj.feats[35], 1.0, "printed aura type flag");
        let host = &s.group(G_BF_OPP)[0];
        assert_eq!(host.feats[32], 1.0, "their bear wears an opposing attachment");
        assert_eq!(host.feats[31], 0.0, "not an own attachment");
        // The trigger encodes its battlefield source into the opp stack
        // group, top of stack at depth 0.
        assert_eq!(s.group_len(G_STACK_OPP), 1);
        assert_eq!(s.group_len(G_STACK_SELF), 0);
        assert_eq!(s.group(G_STACK_OPP)[0].card, vocab.index_of("Grizzly Bears"));
        assert_eq!(s.group(G_STACK_OPP)[0].feats[36], 0.0);

        // Seat-relative like every other group: the same stack is "mine"
        // from seat 1.
        let s1 = encode_state(&g, 1, &vocab);
        assert_eq!(s1.group_len(G_STACK_SELF), 1);
        assert_eq!(s1.group_len(G_STACK_OPP), 0);
        assert_eq!(s1.group(G_BF_SELF)[0].feats[31], 0.0);
        assert_eq!(s1.group(G_BF_SELF)[0].feats[32], 1.0, "opposing aura from either view");
    }

    #[test]
    fn affordable_respects_colour_requirements_not_just_mana_value() {
        use crate::mana::{ManaSymbol, Color};
        let cost = ManaCost::new(vec![
            ManaSymbol::Generic(1),
            ManaSymbol::Colored(Color::White),
            ManaSymbol::Colored(Color::Blue),
        ]);
        let w = [true, false, false, false, false];
        let u = [false, true, false, false, false];
        let any = [true; 5];
        // Three sources, but two of them can only make white: {W}{U} needs
        // a saturating assignment and there is only one blue source, so
        // Hall's condition fails on the {W,U} subset.
        assert!(!affordable(&cost, &[w, w, w]));
        assert!(affordable(&cost, &[w, u, any]));
        // Enough colours, not enough mana.
        assert!(!affordable(&cost, &[w, u]));
        // Colourless sources cover the generic pip only.
        let colourless = [false; 5];
        assert!(affordable(&cost, &[w, u, colourless]));
        assert!(!affordable(&cost, &[w, colourless, colourless]));
    }

    /// `cover_with_extra` has to answer exactly what pushing a `[true; 5]`
    /// source onto the slice answered — that equivalence is the whole reason
    /// the next-turn flag no longer clones the source list per hand card.
    #[test]
    fn the_extra_source_cover_matches_pushing_an_any_colour_source() {
        use crate::mana::{Color, cost, generic, colored};
        let w = [true, false, false, false, false];
        let u = [false, true, false, false, false];
        let colourless = [false; 5];
        let costs = [
            cost(&[colored(Color::White), colored(Color::Blue)]),
            cost(&[generic(2), colored(Color::Blue), colored(Color::Blue)]),
            cost(&[generic(5)]),
            cost(&[colored(Color::Green)]),
        ];
        for pool in [
            &[][..],
            &[w][..],
            &[w, u][..],
            &[w, w, colourless][..],
            &[w, u, colourless, colourless][..],
        ] {
            let cover = source_cover(pool);
            let extra = cover_with_extra(&cover);
            let mut plus = pool.to_vec();
            plus.push([true; 5]);
            for c in &costs {
                assert_eq!(
                    affordable_covered(c, pool.len() as u32 + 1, &extra),
                    affordable(c, &plus),
                    "cost {} over {} sources",
                    c.summary(),
                    pool.len(),
                );
            }
        }
    }
}


