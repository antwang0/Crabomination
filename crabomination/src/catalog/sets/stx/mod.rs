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
// `extras` was a single 40k-line file; it's split into `extras_NN` sibling
// modules (~60 factories each) purely to cut incremental compile time.
// They live at this same module depth so every `super::`/`super::super::`
// path in the card bodies resolves unchanged; the glob re-exports below
// keep the original flat `stx::*` API.
mod extras_00;
mod extras_01;
mod extras_02;
mod extras_03;
mod extras_04;
mod extras_05;
mod extras_06;
mod extras_07;
mod extras_08;
mod extras_09;
mod extras_10;
mod extras_11;
mod extras_12;
mod extras_13;
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
pub use extras_00::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_01::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_02::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_03::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_04::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_05::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_06::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_07::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_08::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_09::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_10::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_11::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_12::*;
#[allow(ambiguous_glob_reexports)]
pub use extras_13::*;
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
