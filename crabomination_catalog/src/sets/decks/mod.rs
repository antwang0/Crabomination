//! Demo-deck card factories.
//!
//! Cards used by the BRG-combo and Goryo's Vengeance demo decks (see
//! `crabomination::demo::build_demo_state` and `DECK_FEATURES.md` at the repo
//! root). Many cards here ship as **stubs** — correct cost, type line, P/T,
//! and keywords, but `Effect::Noop` (or a simplified placeholder) for
//! abilities that need engine features the engine doesn't yet have. Each stub
//! carries a doc-comment marking what's omitted; promote them as engine
//! features land.
//!
//! Assembled from the four decks parts (`crabomination_catalog/parts/decks*`),
//! which compile these files; see `sets/mod.rs` for the split.

pub use crabomination_catalog_decks1::sets::decks::*;
pub use crabomination_catalog_decks2::sets::decks::*;
pub use crabomination_catalog_decks3::sets::decks::*;
pub use crabomination_catalog_decks4::sets::decks::*;
