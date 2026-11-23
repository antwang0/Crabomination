//! Card-name → `CardDefinition` resolver hook. `CardInstance` serializes by
//! name, and deserializing needs the full definition, but the name→factory
//! index lives in the top-level `crabomination` crate, which depends on this
//! one. So that crate installs a resolver via [`set_card_resolver`] at load
//! time (its `ctor` registration), and `CardInstance`'s `Deserialize` calls
//! [`resolve_card`]; with none installed, deserialization fails.

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
