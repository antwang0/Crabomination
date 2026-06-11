//! Secrets of Strixhaven (SOS) — the Strixhaven setting's revisit.
//!
//! Cards listed in `STRIXHAVEN2.md`. Each module groups cards by primary
//! type to mirror the rest of the catalog. The set introduces the five
//! two-color "schools" (Lorehold, Prismari, Quandrix, Silverquill,
//! Witherbloom); cards from a single school are kept together within their
//! type-bucket to make the school-level implementation status easy to
//! eyeball.

pub use super::no_abilities;

mod artifacts;
mod creatures;
mod enchantments;
mod instants;
mod lands;
mod mdfcs;
mod sorceries;

pub use artifacts::*;
pub use creatures::*;
pub use enchantments::*;
pub use instants::*;
pub use lands::*;
pub use mdfcs::*;
pub use sorceries::*;
