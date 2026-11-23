//! Replacement effects framework (Commander prerequisite).
//!
//! A [`ReplacementEffect`] watches an event and returns a *modified* event in
//! its place. Only zone-change replacements are modelled (Commander's "→
//! command zone instead", CR 903.9b).
//!
//! Registered on [`crate::game::GameState`] via `register_replacement` and
//! consulted at placement by the zone-change paths (`place_card_in_dest`,
//! `remove_from_battlefield_to_graveyard_raw`,
//! `remove_from_battlefield_to_exile`): a match swaps the destination and the
//! registry is re-consulted, capped by [`MAX_REPLACEMENT_ITERATIONS`].

use serde::{Deserialize, Serialize};

use crate::card::{CardId, Zone};

/// Opaque identifier handed out by `register_replacement` so callers
/// can later unregister or query a specific effect. Monotonic per
/// `GameState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReplacementId(pub u32);

/// What card a replacement listens for. Narrow on purpose: Commander only
/// matches one card by `CardId`; broader predicates would extend this enum
/// without touching the resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementSource {
    /// Matches the specific card with this id.
    Card(CardId),
}

/// A registered zone-change replacement: when a card matching `source` would
/// move from `from` (any zone if `None`) to one of `to_zones`, the
/// destination becomes `redirect_to`.
///
/// `optional` is reserved for Commander's "may redirect" (the owner's yes/no
/// through the `Decider`); for now every replacement is mandatory.
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

/// Max registry walks per zone change (each fires at most one replacement):
/// headroom for legitimate chains, a stop for pathological loops.
pub const MAX_REPLACEMENT_ITERATIONS: usize = 16;
