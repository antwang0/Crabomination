//! Replacement effects framework (Phase H — Commander prerequisite).
//!
//! A [`ReplacementEffect`] watches a specific game event and, when it
//! fires, returns a *modified* event in place of the original. Today
//! the engine only models zone-change replacements (the Commander
//! "would go to graveyard/exile/hand/library → command zone instead"
//! rule, CR 903.9b); the data shape leaves room to grow.
//!
//! Replacements are registered on [`crate::game::GameState`] via
//! `register_replacement` and consulted by zone-change paths
//! (`place_card_in_dest`, `remove_from_battlefield_to_graveyard_raw`,
//! `remove_from_battlefield_to_exile`). The engine walks the registry
//! at the moment of placement, swaps the destination if a match
//! fires, and re-consults — capped by [`MAX_REPLACEMENT_ITERATIONS`]
//! so chained replacements can't infinite-loop.

use serde::{Deserialize, Serialize};

use crate::card::{CardId, Zone};

/// Opaque identifier handed out by `register_replacement` so callers
/// can later unregister or query a specific effect. Monotonic per
/// `GameState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReplacementId(pub u32);

/// What card a replacement effect listens for. Kept narrow on
/// purpose — the only consumer in scope (Commander) needs to match
/// a single known card by `CardId`. Broader predicates ("any creature
/// you control", "any card with X type") would extend this enum
/// without touching the resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementSource {
    /// Matches the specific card with this id.
    Card(CardId),
}

/// A registered zone-change replacement. Fires when a card matching
/// `source` would move *from* `from` (or any zone if `None`) *to*
/// any one of `to_zones`; the destination is rewritten to
/// `redirect_to`.
///
/// Commander's "may redirect to command zone" semantics live on top
/// of this: callers pass `optional: true` and the resolver delegates
/// the yes/no choice to the card owner via the `Decider` when
/// Phase L wires it in. For Phase H all replacements are mandatory —
/// `optional` is reserved for the next phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementEffect {
    pub id: ReplacementId,
    pub source: ReplacementSource,
    /// Origin zone filter. `None` matches any origin (e.g. "wherever
    /// this card is", useful for Commander's
    /// "would go to graveyard/exile/hand/library" rule which doesn't
    /// constrain origin).
    pub from: Option<Zone>,
    /// Destination zone filter. Must contain the would-be destination
    /// for the replacement to fire. Empty `to_zones` matches no
    /// destination (a no-op replacement).
    pub to_zones: Vec<Zone>,
    /// Where the card is rewritten to land instead.
    pub redirect_to: Zone,
    /// True if the affected card's owner may choose to apply or
    /// decline this replacement (CR 614.6 / CR 903.9b). Phase H
    /// always treats this as mandatory; Phase L wires the decision.
    #[serde(default)]
    pub optional: bool,
}

/// Hard cap on how many times the resolver will walk the registry
/// for a single zone change. Each pass can fire at most one
/// replacement, so this also caps the total replacements applied
/// per move. Set high enough that legitimate chained replacements
/// (commander-redirect → ETB-redirect, etc.) have plenty of headroom
/// while still stopping a pathological loop.
pub const MAX_REPLACEMENT_ITERATIONS: usize = 16;
