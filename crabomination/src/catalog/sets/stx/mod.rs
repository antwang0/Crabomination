//! Strixhaven: School of Mages (STX) — 2021.
//!
//! See `STRIXHAVEN2.md` at the repo root for the per-card status table and the
//! engine features that gate full implementations. The cards here are
//! grouped by college (Silverquill / Witherbloom / Lorehold / Prismari /
//! Quandrix) and most use the engine's existing primitives (Magecraft via
//! the new spell-cast filter, Learn approximated as Draw 1 until a Lesson
//! sideboard model lands).

pub use super::no_abilities;

mod all_factories;
mod extras;
mod iconic;
mod legends;
mod lessons;
mod lorehold;
mod mono;
mod prismari;
mod quandrix;
mod shared;
mod silverquill;
mod witherbloom;

pub use all_factories::all_stx_card_factories;
#[allow(ambiguous_glob_reexports)]
pub use extras::*;
pub use iconic::*;
pub use legends::*;
pub use lessons::*;
pub use lorehold::*;
pub use mono::*;
pub use prismari::*;
pub use quandrix::*;
pub use shared::*;
pub use silverquill::*;
pub use witherbloom::*;
