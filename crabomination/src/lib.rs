#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

// The foundational data model (card / mana / effect / static_str_serde /
// TurnStep / token factories) now lives in `crabomination_base`, and the card
// catalog in `crabomination_catalog`. Both are split out so that editing a
// card or a card type doesn't recompile the game engine and the test suite.
// Re-exported here under their original paths so the rest of the workspace
// (and external consumers) keep using `crabomination::card`, `::mana`, etc.
pub use crabomination_base::TurnStep;
pub use crabomination_base::{card, effect, mana, static_str_serde};

/// The card catalog, augmented with the name→factory registry. The registry
/// lives in this crate (rather than `crabomination_catalog`) because it
/// aggregates the cube, demo decks and SoS pools defined here.
pub mod catalog {
    pub use crabomination_catalog::*;

    pub use crate::card_registry::{all_known_factories, lookup_by_name};
}

mod card_registry;

pub mod cube;
pub mod decklist;
pub mod decision;
pub mod demo;
pub mod draft;
pub mod format;
pub mod game;
pub mod net;
pub mod player;
pub mod replacement;
pub mod server;
pub mod snapshot;
pub mod sos_mode;
pub mod team;

// `CardInstance` (in `crabomination_base`) round-trips through serde by card
// *name* and rebuilds the definition via a resolver hook. Install that resolver
// before `main` so every binary and test binary linking this crate can
// deserialize saved game state without an explicit init call.
#[ctor::ctor(unsafe)]
fn register_card_resolver() {
    crabomination_base::set_card_resolver(card_registry::lookup_by_name);
}
