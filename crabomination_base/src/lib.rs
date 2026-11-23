//! `crabomination_base` — the slow-changing data model shared by the card
//! catalog and the engine: card / mana / effect types, the engine-baked token
//! factories and `TurnStep`. It has no game logic and no dependency on the
//! (huge) catalog, so editing a card or the engine doesn't recompile these
//! types.
//!
//! The one reach "up" into the catalog — `CardInstance`'s `Deserialize` turning
//! a card name back into a `CardDefinition` — is inverted via [`registry`]: the
//! top-level crate installs a resolver at startup.

// These were crate-level allows on the original monolithic `crabomination`
// crate; they travel here with the card/effect type definitions.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

pub mod card;
/// Debug-build memo audits that nest: see the module.
pub mod memo_audit;
pub mod copyvec;
pub mod cow;
pub mod dungeons;
pub mod effect;
pub mod mana;
pub mod oftenempty;
pub mod registry;
pub mod static_str_serde;
pub mod sticker;
pub mod textrewrite;
pub mod tokens;
pub mod turn_step;

pub use registry::set_card_resolver;
pub use turn_step::TurnStep;
