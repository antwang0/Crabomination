//! Card-name → `CardDefinition` resolver hook.
//!
//! `CardInstance` round-trips through serde by *name*: serialization writes
//! the card's name, and deserialization must rebuild the full
//! `CardDefinition`. The authoritative name→factory index lives in the
//! top-level `crabomination` crate (it aggregates the cube, demo decks, SoS
//! and STX pools), which depends on this crate — so this crate cannot call it
//! directly without a dependency cycle.
//!
//! Instead the top-level crate installs a resolver via [`set_card_resolver`]
//! at load time (see its `ctor` registration), and `CardInstance`'s
//! `Deserialize` impl calls [`resolve_card`]. If no resolver has been
//! installed, resolution fails and deserialization returns an error.

use std::sync::OnceLock;

use crate::card::CardDefinition;

/// A function that resolves a card name to its full definition, or `None` if
/// the name is unknown to the registry.
pub type CardResolver = fn(&str) -> Option<CardDefinition>;

static RESOLVER: OnceLock<CardResolver> = OnceLock::new();

/// Install the card-name resolver. Idempotent: the first installation wins
/// (subsequent calls are ignored), which is what we want for the top crate's
/// one-shot `ctor` registration.
pub fn set_card_resolver(resolver: CardResolver) {
    let _ = RESOLVER.set(resolver);
}

/// Resolve a card name to its definition using the installed resolver.
/// Returns `None` if no resolver is installed or the name is unknown.
pub fn resolve_card(name: &str) -> Option<CardDefinition> {
    RESOLVER.get().and_then(|resolve| resolve(name))
}
