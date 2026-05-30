//! `crabomination_base` — the foundational, slow-changing data model shared by
//! the card catalog and the game engine.
//!
//! This crate holds the pure card/mana/effect type definitions plus the
//! engine-baked token factories and the `TurnStep` enum. It deliberately
//! contains **no** game logic and **no** dependency on the (huge) card
//! catalog, so that editing a card or a piece of game logic does not force a
//! recompile of these types. See the workspace-level compile-time notes.
//!
//! The single place the model needs to reach "up" into the catalog is
//! `CardInstance`'s `Deserialize` impl, which must turn a serialized card
//! *name* back into a full `CardDefinition`. That dependency is inverted via
//! [`registry`]: the top-level crate installs a resolver at startup.

// These were crate-level allows on the original monolithic `crabomination`
// crate; they travel here with the card/effect type definitions.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

pub mod card;
pub mod effect;
pub mod mana;
pub mod registry;
pub mod static_str_serde;
pub mod tokens;
pub mod turn_step;

pub use registry::set_card_resolver;
pub use turn_step::TurnStep;
