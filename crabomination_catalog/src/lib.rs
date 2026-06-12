//! Card catalog — factory functions for every implemented Magic: The Gathering card.
//! Cards are grouped by the set in which they first appeared.
//!
//! This crate was split out of the monolithic `crabomination` crate so that
//! editing a card recompiles only the catalog, not the game engine or the test
//! suite. It depends solely on `crabomination_base` (the card/mana/effect data
//! model). The name→factory registry used by snapshot deserialization lives in
//! the top-level `crabomination` crate, since it also aggregates the cube/demo
//! pools.

// Crate-level allows inherited from the original monolithic `crabomination`
// crate, where the card factories lived.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

// ── Crate-private compatibility shims ────────────────────────────────────────
// To avoid rewriting the thousands of `crate::card` / `crate::effect` /
// `crate::game` / `crate::catalog` paths in the factory modules, re-expose the
// foundational items under their original crate-relative names.

// Lets `crate::catalog::foo()` paths inside the factories keep resolving.
extern crate self as catalog;

// `crate::card`, `crate::effect`, `crate::mana`.
use crabomination_base::{card, effect, mana};

// `crate::game::types::TurnStep`, `crate::game::TurnStep`, and the engine token
// factories under `crate::game::effects::*` (all now live in the base crate).
mod game {
    pub use crabomination_base::TurnStep;
    pub mod types {
        pub use crabomination_base::TurnStep;
    }
    pub mod effects {
        pub use crabomination_base::tokens::{
            blood_token, clue_token, detective_token, eldrazi_spawn_token, food_token,
            goldspan_treasure_token, map_token, treasure_token,
        };
    }
}

pub mod sets;

// Re-export everything so callers use `catalog::some_card()`.
pub use sets::akh::*;
pub use sets::all::*;
pub use sets::ap::*;
pub use sets::arn::*;
pub use sets::dis::*;
pub use sets::fem::*;
pub use sets::gpt::*;
pub use sets::ice::*;
pub use sets::inv::*;
pub use sets::khm::*;
pub use sets::kld::*;
pub use sets::ktk::*;
pub use sets::lci::*;
pub use sets::lea::*;
pub use sets::m11::*;
pub use sets::mkm::*;
pub use sets::ogw::*;
pub use sets::pc2::*;
pub use sets::por::*;
pub use sets::rav::*;
pub use sets::rtr::*;
pub use sets::tmp::*;
pub use sets::zen::*;
pub use sets::thb::*;
pub use sets::ths::*;
pub use sets::decks::*;
pub use sets::mod_set::*;
pub use sets::sos::*;
pub use sets::stx::*;
pub use sets::xtra::*;

use crabomination_base::card::CardDefinition;

/// A zero-arg factory that produces a fresh `CardDefinition`. The name→factory
/// registry (in the top-level `crabomination` crate) is built from these.
pub type CardFactory = fn() -> CardDefinition;
