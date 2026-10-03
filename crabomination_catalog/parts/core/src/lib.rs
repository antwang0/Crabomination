//! The catalog's core part: the shared card-building helpers and the hub
//! sets other parts build on. See `crabomination_catalog/src/sets/mod.rs`.

#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

// `crate::catalog::..` inside the card files resolves to this crate.
extern crate self as catalog;

// `crate::card`, `crate::effect`, `crate::mana` and `crate::game::..` inside
// the card files. Public so every other part re-exposes the same names.
pub use crabomination_base::{card, effect, mana};

/// `crate::game::types::TurnStep`, `crate::game::TurnStep`, and the engine
/// token factories under `crate::game::effects::*` (all in the base crate).
pub mod game {
    pub use crabomination_base::TurnStep;
    pub mod types {
        pub use crabomination_base::TurnStep;
    }
    pub mod effects {
        pub use crabomination_base::tokens::{
            blood_token, detective_token, eldrazi_spawn_token, food_token,
            map_token, powerstone_token, treasure_token,
        };
    }
}

pub mod sets;

/// A zero-arg factory that produces a fresh `CardDefinition`. The name→factory
/// registry (in the top-level `crabomination` crate) is built from these.
/// Defined here rather than in the facade so `crate::CardFactory` resolves in
/// every part (the per-set `all_*_card_factories` lists spell it that way).
pub type CardFactory = fn() -> crabomination_base::card::CardDefinition;

// `crate::catalog::fractal_token` and friends, as the monolithic crate's root
// re-exports spelled them.
pub use sets::cmdr::*;
pub use sets::lea::*;
pub use sets::one::*;
pub use sets::rna::*;
pub use sets::sos::*;
pub use sets::thb::*;
