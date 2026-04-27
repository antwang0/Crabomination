//! Modern-staple removal / counterspell / pump cards.
//!
//! Complements `decks::modern` (the broader Modern supplement that lives
//! alongside the BRG / Goryo's demo decks). Cards here are common interaction
//! pieces — single-target removal, narrow counterspells, combat tricks — that
//! don't belong in the demo decks but make the catalog more useful for
//! tests and future builds.

pub use super::no_abilities;

mod creatures;
mod instants;
mod lands;
mod sorceries;
mod spells;

pub use creatures::*;
pub use instants::*;
pub use lands::*;
pub use sorceries::*;
pub use spells::*;
