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
//! Scope is deliberately SOS sealed ([`Vocab::sos_sealed`]): one set's worth
//! of names keeps the embedding table small and well-fed. Tokens and
//! anything off-vocabulary encode as index 0 (unknown) and are represented
//! by their object features alone — a Pest token is "an unknown 1/1" to the
//! net, which is most of what it needs to know.

use std::collections::HashMap;

use crabomination_nn::{
    EncodedObject, EncodedState, G_BF_OPP, G_BF_SELF, G_GY_OPP, G_GY_SELF, G_HAND_SELF,
    G_LIB_SELF, G_STACK_OPP, G_STACK_SELF, GLOBAL_FEATS, OBJ_FEATS,
};

use crate::card::{CardInstance, CounterType};
use crate::game::actions::color_index;
use crate::game::{GameState, TurnStep};
use crate::mana::ManaCost;

/// Card-name → embedding-index table. Index 0 is reserved for unknown
/// names; real cards are indexed 1.. in sorted-name order, so the mapping
/// is stable as long as the set list is (and the trainer's embedding table
/// is sized off [`Vocab::size`], so the two stay in lockstep).
pub struct Vocab {
    map: HashMap<&'static str, u16>,
}

impl Vocab {
    /// The SOS sealed universe: every card in the draftable pool plus the
    /// five basic lands the sealed builder adds.
    pub fn sos_sealed() -> Vocab {
        let mut names: std::collections::BTreeSet<&'static str> =
            crate::draft::sos_draft_pool().iter().map(|f| f().name).collect();
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            names.insert(basic);
        }
        let map = names.into_iter().zip(1u16..).collect();
        Vocab { map }
    }

    /// Total index count including the reserved unknown slot — the
    /// embedding table's row count.
    pub fn size(&self) -> usize {
        self.map.len() + 1
    }

    /// 0 for anything unrecognized (tokens, off-set cards).
    pub fn index_of(&self, name: &str) -> u16 {
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
static ABLATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
const ABLATE_LIBRARY: u8 = 1;
const ABLATE_CASTABILITY: u8 = 2;
const ABLATE_RELATIONS: u8 = 4;
const ABLATE_COMBAT: u8 = 8;
const ABLATE_KW: u8 = 16;

/// Turn feature blocks off for an ablation run. All default on.
/// `relations` covers the round-12 block whole: the relation flags
/// (28..=35), the stack groups, and stack depth. `combat` is the
/// round-28 combat-structure block (object feats 37..=39, globals
/// 36..=40); `kw` is the round-28 keyword classes and exile counts
/// (object feats 40..=44, globals 41..=42).
pub fn set_encode_ablation(library: bool, castability: bool, relations: bool, combat: bool, kw: bool) {
    let mask = if library { 0 } else { ABLATE_LIBRARY }
        | if castability { 0 } else { ABLATE_CASTABILITY }
        | if relations { 0 } else { ABLATE_RELATIONS }
        | if combat { 0 } else { ABLATE_COMBAT }
        | if kw { 0 } else { ABLATE_KW };
    ABLATE.store(mask, std::sync::atomic::Ordering::Relaxed);
}

fn ablated(bit: u8) -> bool {
    ABLATE.load(std::sync::atomic::Ordering::Relaxed) & bit != 0
}

/// Encode the position from `seat`'s perspective. Two-player only, like
/// the rest of the bot stack.
pub fn encode_state(g: &GameState, seat: usize, vocab: &Vocab) -> EncodedState {
    let opp = 1 - seat;
    let mut s = EncodedState::default();

    // Relation context (round 12): unary summaries of edges the pooled
    // representation cannot carry — see the OBJ_FEATS doc, 28..=36.
    let no_rel = ablated(ABLATE_RELATIONS);
    let mut targeted: std::collections::HashSet<crate::card::CardId> = Default::default();
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
        g.battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.power().max(0), (c.toughness() - c.damage as i32).max(0)))
    };
    // Attacker → summed P/T of its blockers (`block_map` is blocker →
    // attackers, so this is the map inverted).
    let mut blocker_sums: HashMap<crate::card::CardId, (i32, i32)> = HashMap::new();
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

    for c in g.battlefield.iter() {
        let group = if c.controller == seat { G_BF_SELF } else { G_BF_OPP };
        let mut o = encode_battlefield_object(g, c, vocab);
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
            if g.block_map.values().any(|attackers| attackers.contains(&c.id)) {
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
        s.groups[group].push(o);
    }
    // Castability is per-seat state, so the hand's live/dead split is
    // computed against this seat's own untapped sources.
    let no_cast = ablated(ABLATE_CASTABILITY);
    let sources = if no_cast { Vec::new() } else { g.untapped_mana_colors(seat) };
    for c in g.players[seat].hand.iter() {
        let mut o = encode_card_object(c, vocab);
        if !no_cast && !c.definition.is_land() {
            o.feats[25] = if affordable(&c.definition.cost, &sources) { 1.0 } else { 0.0 };
            // Next turn is this turn plus one more source of any colour —
            // the land drop the seat has not made yet. Deliberately
            // optimistic about colour: "which of my cards come online if I
            // hit my drop" is the question, and a wrong-colour land is the
            // rarer case in a two-colour sealed deck.
            o.feats[26] = if affordable_with_extra(&c.definition.cost, &sources) { 1.0 } else { 0.0 };
        }
        s.groups[G_HAND_SELF].push(o);
    }
    for (group, p) in [(G_GY_SELF, seat), (G_GY_OPP, opp)] {
        for c in g.players[p].graveyard.iter() {
            s.groups[group].push(encode_card_object(c, vocab));
        }
    }
    encode_library(&mut s, g, seat, vocab);
    if !no_rel {
        encode_stack(&mut s, g, seat, vocab);
    }

    let (mut lands, mut untapped, mut creatures, mut power) = ([0i32; 2], [0i32; 2], [0i32; 2], [0i32; 2]);
    for c in g.battlefield.iter() {
        let side = if c.controller == seat { 0 } else { 1 };
        if c.definition.is_land() {
            lands[side] += 1;
            if !c.tapped {
                untapped[side] += 1;
            }
        }
        if c.definition.is_creature() {
            creatures[side] += 1;
            power[side] += c.power().max(0);
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
    let opp_sources = if no_cast { Vec::new() } else { g.untapped_mana_colors(opp) };
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
        // Power aimed at each life total through creatures nothing
        // blocks. Before blocks it is the whole attack; after, what got
        // through — the phase one-hots above disambiguate which.
        for a in g.attacking.iter() {
            if let crate::game::types::AttackTarget::Player(p) = a.target {
                let is_blocked = blocker_sums.contains_key(&a.attacker)
                    || g.block_map.values().any(|att| att.contains(&a.attacker));
                if !is_blocked {
                    if let Some((pw, _)) = eff_pt(a.attacker) {
                        gl[if p == seat { 39 } else { 40 }] += pw as f32 / 12.0;
                    }
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
    const _: () = assert!(GLOBAL_FEATS == 43, "extend the fill above when adding globals");

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
    let mut counts: std::collections::BTreeMap<u16, (&CardInstance, u32)> =
        std::collections::BTreeMap::new();
    for c in g.players[seat].library.iter() {
        let idx = vocab.index_of(c.definition.name);
        counts.entry(idx).and_modify(|e| e.1 += 1).or_insert((c, 1));
    }
    for (_, (c, n)) in counts {
        let mut o = encode_card_object(c, vocab);
        o.feats[27] = n as f32 / 4.0;
        s.groups[G_LIB_SELF].push(o);
    }
}

/// The stack, one object per item, split by controller like the
/// battlefield. Spells encode their own card; a trigger encodes its
/// source if that is still on the battlefield, and otherwise an unknown
/// object — rare, and its group and depth still carry signal. Depth from
/// the top of the stack (the item that resolves first) lands in feature
/// 36, because pooling would otherwise erase resolution order.
fn encode_stack(s: &mut EncodedState, g: &GameState, seat: usize, vocab: &Vocab) {
    use crate::game::types::StackItem;
    let n = g.stack.len();
    for (i, item) in g.stack.iter().enumerate() {
        let (mut o, controller) = match item {
            StackItem::Spell { card, caster, .. } => (encode_card_object(card, vocab), *caster),
            StackItem::Trigger { source, controller, .. } => (
                g.battlefield
                    .iter()
                    .find(|c| c.id == *source)
                    .map(|c| encode_card_object(c, vocab))
                    .unwrap_or_default(),
                *controller,
            ),
        };
        // The stack is a Vec used LIFO: the last element is the top.
        o.feats[36] = (n - 1 - i) as f32 / 4.0;
        let group = if controller == seat { G_STACK_SELF } else { G_STACK_OPP };
        s.groups[group].push(o);
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
fn affordable(cost: &ManaCost, sources: &[[bool; 5]]) -> bool {
    let mut pips = [0u32; 5];
    for c in cost.colored_symbols() {
        pips[color_index(c)] += 1;
    }
    let colored: u32 = pips.iter().sum();
    // `cmc` charges mono-hybrid its generic half while `colored_symbols`
    // also counts it as a pip; take whichever is larger so the total can
    // never come in under the coloured requirement.
    if (sources.len() as u32) < cost.cmc().max(colored) {
        return false;
    }
    (1u32..32).all(|mask| {
        let need: u32 = (0..5).filter(|i| mask >> i & 1 == 1).map(|i| pips[i]).sum();
        let have = sources
            .iter()
            .filter(|s| (0..5).any(|i| mask >> i & 1 == 1 && s[i]))
            .count() as u32;
        need <= have
    })
}

/// [`affordable`] with one more source that makes any colour — the land
/// drop the seat has not taken yet.
fn affordable_with_extra(cost: &ManaCost, sources: &[[bool; 5]]) -> bool {
    let mut plus = sources.to_vec();
    plus.push([true; 5]);
    affordable(cost, &plus)
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
fn encode_card_object(c: &CardInstance, vocab: &Vocab) -> EncodedObject {
    use crate::card::Keyword;
    let def = &c.definition;
    let mut feats = [0.0f32; OBJ_FEATS];
    feats[0] = def.cost.cmc() as f32 / 8.0;
    feats[1] = if def.is_creature() { 1.0 } else { 0.0 };
    feats[2] = if def.is_land() { 1.0 } else { 0.0 };
    feats[3] = if def.is_planeswalker() { 1.0 } else { 0.0 };
    feats[4] = def.power.max(0) as f32 / 8.0;
    feats[5] = def.toughness.max(0) as f32 / 8.0;
    // Evasion/combat keywords, granted ones included (`has_keyword`
    // reads printed + granted lists; the granted lists are simply empty
    // off the battlefield). First and double strike share a flag — for a
    // value function they mean the same thing at this resolution.
    for (i, kw) in [
        Keyword::Flying,
        Keyword::Reach,
        Keyword::Menace,
        Keyword::Deathtouch,
        Keyword::Lifelink,
        Keyword::Trample,
        Keyword::FirstStrike,
        Keyword::Vigilance,
    ]
    .iter()
    .enumerate()
    {
        if c.has_keyword(kw) {
            feats[12 + i] = 1.0;
        }
    }
    if c.has_keyword(&Keyword::DoubleStrike) {
        feats[18] = 1.0;
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
        for (i, kw) in
            [Keyword::Haste, Keyword::Indestructible, Keyword::Defender].iter().enumerate()
        {
            // 40 haste, 42 indestructible, 44 defender.
            if c.has_keyword(kw) {
                feats[40 + 2 * i] = 1.0;
            }
        }
        if c.ward().is_some() || any_keyword(c, is_hard_to_target) {
            feats[41] = 1.0;
        }
        if any_keyword(c, is_hard_to_block) {
            feats[43] = 1.0;
        }
    }
    EncodedObject { card: vocab.index_of(def.name), feats }
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
/// effective P/T (counters and pumps included), damage marked, tapped,
/// summoning sickness, loyalty, SOS prepared status, attacking.
fn encode_battlefield_object(g: &GameState, c: &CardInstance, vocab: &Vocab) -> EncodedObject {
    let mut o = encode_card_object(c, vocab);
    let f = &mut o.feats;
    f[4] = c.power().max(0) as f32 / 8.0;
    f[5] = (c.toughness() - c.damage as i32).max(0) as f32 / 8.0;
    f[6] = if c.tapped { 1.0 } else { 0.0 };
    f[7] = if c.summoning_sick { 1.0 } else { 0.0 };
    f[8] = c.counter_count(CounterType::Loyalty) as f32 / 8.0;
    f[9] = if c.counter_count(CounterType::Prepared) > 0 { 1.0 } else { 0.0 };
    f[10] = if g.attacking.iter().any(|a| a.attacker == c.id) { 1.0 } else { 0.0 };
    f[11] = if c.is_token { 1.0 } else { 0.0 };
    // Counters beyond loyalty/prepared, by count. P/T counters reach the
    // net twice — through effective P/T and here — which is deliberate: a
    // 3/3 that is a 2/2 plus a counter dies differently to bounce and
    // counter-hate than a printed 3/3 does.
    if !ablated(ABLATE_RELATIONS) {
        let special: u32 = c
            .counters
            .iter()
            .filter(|(k, _)| !matches!(k, CounterType::Loyalty | CounterType::Prepared))
            .map(|(_, v)| *v)
            .sum();
        f[34] = special as f32 / 4.0;
    }
    o
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

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    #[test]
    fn sos_vocab_is_substantial_and_stable() {
        let v = Vocab::sos_sealed();
        // Sanity range, not an exact count: the set list may grow a card
        // or two, but a collapse to near-zero means the pool wiring broke.
        assert!(v.size() > 150 && v.size() < 500, "vocab size {}", v.size());
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
        assert_eq!(s0.groups[G_BF_SELF].len(), 0);
        assert_eq!(s0.groups[G_BF_OPP].len(), 1);
        assert_eq!(s0.groups[G_HAND_SELF].len(), 1);
        assert_eq!(s0.groups[G_GY_SELF].len(), 1);
        assert_eq!(s0.groups[G_GY_OPP].len(), 0);
        assert!((s0.global[0] - 12.0 / 20.0).abs() < 1e-6);
        assert!((s0.global[1] - 1.0).abs() < 1e-6);
        assert_eq!(s0.global[9], 0.0, "seat 0 is not the active player");
        let opp_bear = &s0.groups[G_BF_OPP][0];
        assert_eq!(opp_bear.feats[6], 1.0, "tapped flag");
        assert!((opp_bear.feats[4] - 2.0 / 8.0).abs() < 1e-6, "2 power");
        // Grizzly Bears is off the SOS vocab — unknown index, features carry it.
        assert_eq!(opp_bear.card, vocab.index_of("Grizzly Bears"));

        // The same position from the other seat mirrors every self/opp pair.
        let s1 = encode_state(&g, 1, &vocab);
        assert_eq!(s1.groups[G_BF_SELF].len(), 1);
        assert_eq!(s1.groups[G_BF_OPP].len(), 0);
        assert_eq!(s1.groups[G_HAND_SELF].len(), 0, "opponent hand is hidden");
        assert!((s1.global[0] - 1.0).abs() < 1e-6);
        assert!((s1.global[1] - 12.0 / 20.0).abs() < 1e-6);
        assert_eq!(s1.global[9], 1.0);
        assert_eq!(s1.groups[G_GY_SELF].len(), 0);
        assert_eq!(s1.groups[G_GY_OPP].len(), 1);

        // Empty groups exist but are empty, never dropped.
        assert_eq!(s0.groups.len(), NUM_GROUPS);
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
        let lib = &s.groups[G_LIB_SELF];
        assert_eq!(lib.len(), 2, "four cards, two distinct names");
        // Sorted by vocabulary index, and the counts land in feat 27.
        let by_name: Vec<(u16, f32)> = lib.iter().map(|o| (o.card, o.feats[27])).collect();
        assert!(by_name[0].0 < by_name[1].0, "emitted in vocab-index order");
        let forest = by_name.iter().find(|e| e.0 == vocab.index_of("Forest")).unwrap();
        let island = by_name.iter().find(|e| e.0 == vocab.index_of("Island")).unwrap();
        assert!((forest.1 - 3.0 / 4.0).abs() < 1e-6, "three Forests");
        assert!((island.1 - 1.0 / 4.0).abs() < 1e-6, "one Island");
        // The opponent's library is never encoded — only its size.
        assert_eq!(encode_state(&g, 1, &vocab).groups[G_LIB_SELF].len(), 0);
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
        assert_eq!(one_forest.groups[G_HAND_SELF][0].feats[25], 0.0, "one land, two-mana spell");
        assert_eq!(one_forest.groups[G_HAND_SELF][0].feats[26], 1.0, "castable after a land drop");

        add_lands(&mut g, 0, catalog::island, 1, 200);
        let forest_island = encode_state(&g, 0, &vocab);
        assert_eq!(forest_island.groups[G_HAND_SELF][0].feats[25], 1.0, "{{1}}{{G}} off Forest+Island");

        let mut wrong = two_player_game();
        wrong.players[0].hand.push(bear());
        add_lands(&mut wrong, 0, catalog::island, 2, 300);
        let two_islands = encode_state(&wrong, 0, &vocab);
        assert_eq!(
            two_islands.groups[G_HAND_SELF][0].feats[25], 0.0,
            "two mana but no green source"
        );
        // Next turn it comes online: the assumed land drop is optimistic
        // about colour, so it covers the {G} and an Island pays the {1}.
        // That optimism is the documented approximation — a seat with no
        // green land left in its library will read as castable here.
        assert_eq!(two_islands.groups[G_HAND_SELF][0].feats[26], 1.0);

        // Printed colour pips ride along on every object regardless of zone.
        let g_pip = two_islands.groups[G_HAND_SELF][0].feats[20 + 4];
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
        assert_eq!(full.groups[G_LIB_SELF].len(), 1);
        assert_eq!(full.groups[G_HAND_SELF][0].feats[25], 1.0);
        assert!(full.global[24 + 4] > 0.0);

        set_encode_ablation(false, true, true, true, true);
        let no_lib = encode_state(&g, 0, &vocab);
        assert_eq!(no_lib.groups[G_LIB_SELF].len(), 0, "library group is empty");
        assert_eq!(no_lib.groups[G_HAND_SELF][0].feats[25], 1.0, "castability survives");
        assert!(no_lib.global[24 + 4] > 0.0);

        set_encode_ablation(true, false, true, true, true);
        let no_cast = encode_state(&g, 0, &vocab);
        assert_eq!(no_cast.groups[G_LIB_SELF].len(), 1, "library survives");
        assert_eq!(no_cast.groups[G_HAND_SELF][0].feats[25], 0.0, "castable-now zeroed");
        assert_eq!(no_cast.groups[G_HAND_SELF][0].feats[26], 0.0, "castable-next zeroed");
        assert_eq!(no_cast.groups[G_HAND_SELF][0].feats[20 + 4], 0.0, "pips zeroed");
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
        g.block_map.insert(crate::card::CardId(101), vec![crate::card::CardId(100)]);
        set_encode_ablation(true, true, true, true, true);
        let with_rel = encode_state(&g, 0, &vocab);
        assert!(with_rel.groups[G_BF_SELF].iter().any(|o| o.feats[29] == 1.0), "blocked flag on");
        set_encode_ablation(true, true, false, true, true);
        let no_rel = encode_state(&g, 0, &vocab);
        assert!(no_rel.groups[G_BF_SELF].iter().all(|o| o.feats[29] == 0.0), "blocked flag off");
        assert_eq!(no_rel.groups[G_LIB_SELF].len(), 1, "library survives rel ablation");
        assert_eq!(no_rel.groups[G_HAND_SELF][0].feats[25], 1.0, "castability survives");

        // The combat bit blanks the round-28 combat structure and only
        // it. IDs 100/101 are the Forests above — power 0, so the
        // endpoint sums stay 0 and the phase one-hot is the readable
        // difference.
        g.step = TurnStep::DeclareBlockers;
        let with_combat = encode_state(&g, 0, &vocab);
        assert_eq!(with_combat.global[37], 1.0, "declare-blockers one-hot on");
        set_encode_ablation(true, true, true, false, true);
        let no_combat = encode_state(&g, 0, &vocab);
        assert_eq!(no_combat.global[37], 0.0, "declare-blockers one-hot off");
        assert_eq!(no_combat.groups[G_LIB_SELF].len(), 1, "library survives combat ablation");
        assert!(
            no_combat.groups[G_BF_SELF].iter().any(|o| o.feats[29] == 1.0),
            "relation flags survive combat ablation"
        );

        // The kw bit blanks the keyword classes and exile counts.
        g.exile.push(CardInstance::new(crate::card::CardId(950), catalog::grizzly_bears(), 0));
        set_encode_ablation(true, true, true, true, true);
        let with_kw = encode_state(&g, 0, &vocab);
        assert!((with_kw.global[41] - 1.0 / 10.0).abs() < 1e-6, "exile count on");
        set_encode_ablation(true, true, true, true, false);
        let no_kw = encode_state(&g, 0, &vocab);
        assert_eq!(no_kw.global[41], 0.0, "exile count off");
        assert_eq!(no_kw.global[37], 1.0, "combat block survives kw ablation");

        g.step = TurnStep::PreCombatMain;
        g.exile.clear();
        set_encode_ablation(true, true, true, true, true);
        assert_eq!(encode_state(&g, 0, &vocab), with_rel, "all on restores the full encoding");
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
        g.block_map.insert(crate::card::CardId(3), vec![crate::card::CardId(1)]);
        g.block_map.insert(crate::card::CardId(4), vec![crate::card::CardId(1)]);

        let s = encode_state(&g, 0, &vocab);
        // Both creatures are off the SOS vocab (index 0), so objects are
        // told apart by their power feature, not the card index.
        let giant = s.groups[G_BF_OPP]
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
            s.groups[G_BF_SELF].iter().filter(|o| o.feats[37] > 0.0).collect();
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
        let attacker = s.groups[G_BF_OPP]
            .iter()
            .find(|o| (o.feats[4] - 2.0 / 8.0).abs() < 1e-6)
            .expect("the attacking 2/2 encoded");
        assert_eq!(attacker.feats[39], 1.0, "attacking a non-player target");
        assert_eq!(s.global[39], 0.0, "walker attack leaves the life total alone");
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

        let o = &encode_state(&g, 0, &vocab).groups[G_BF_SELF][0];
        assert_eq!(o.feats[40], 1.0, "haste");
        assert_eq!(o.feats[41], 1.0, "hexproof → hard to target");
        assert_eq!(o.feats[42], 0.0, "not indestructible");
        assert_eq!(o.feats[43], 1.0, "shadow → hard to block");
        assert_eq!(o.feats[44], 0.0, "not a defender");

        // A removed keyword no longer counts, exact-variant or class.
        g.battlefield[0].removed_keywords.push(Keyword::Hexproof);
        let o = &encode_state(&g, 0, &vocab).groups[G_BF_SELF][0];
        assert_eq!(o.feats[41], 0.0, "removed hexproof does not flag");
        assert_eq!(o.feats[43], 1.0, "shadow unaffected");
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
        let me = &s.groups[G_BF_SELF][0];
        assert_eq!(me.feats[33], 1.0, "my bear is targeted by the stack");
        assert_eq!(me.feats[30], 0.0);
        let aura_obj =
            s.groups[G_BF_SELF].iter().find(|o| o.feats[30] == 1.0).expect("aura is_attached");
        assert_eq!(aura_obj.feats[35], 1.0, "printed aura type flag");
        let host = &s.groups[G_BF_OPP][0];
        assert_eq!(host.feats[32], 1.0, "their bear wears an opposing attachment");
        assert_eq!(host.feats[31], 0.0, "not an own attachment");
        // The trigger encodes its battlefield source into the opp stack
        // group, top of stack at depth 0.
        assert_eq!(s.groups[G_STACK_OPP].len(), 1);
        assert_eq!(s.groups[G_STACK_SELF].len(), 0);
        assert_eq!(s.groups[G_STACK_OPP][0].card, vocab.index_of("Grizzly Bears"));
        assert_eq!(s.groups[G_STACK_OPP][0].feats[36], 0.0);

        // Seat-relative like every other group: the same stack is "mine"
        // from seat 1.
        let s1 = encode_state(&g, 1, &vocab);
        assert_eq!(s1.groups[G_STACK_SELF].len(), 1);
        assert_eq!(s1.groups[G_STACK_OPP].len(), 0);
        assert_eq!(s1.groups[G_BF_SELF][0].feats[31], 0.0);
        assert_eq!(s1.groups[G_BF_SELF][0].feats[32], 1.0, "opposing aura from either view");
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
}
