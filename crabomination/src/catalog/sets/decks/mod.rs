//! Demo-deck card factories.
//!
//! Cards used by the BRG-combo and Goryo's Vengeance demo decks (see
//! `crabomination::demo::build_demo_state` and `DECK_FEATURES.md` at the repo
//! root). Many cards here ship as **stubs** — correct cost, type line, P/T,
//! and keywords, but `Effect::Noop` (or a simplified placeholder) for
//! abilities that need engine features the engine doesn't yet have. Each stub
//! carries a doc-comment marking what's omitted; promote them as engine
//! features land.

pub mod creatures;
pub mod lands;
pub mod modern;
pub mod spells;

pub use creatures::*;
pub use lands::*;
pub use modern::*;
pub use spells::*;
