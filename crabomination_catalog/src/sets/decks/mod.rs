//! Demo-deck card factories.
//!
//! Cards used by the BRG-combo and Goryo's Vengeance demo decks (see
//! `crabomination::demo::build_demo_state` and `DECK_FEATURES.md` at the repo
//! root). Many cards here ship as **stubs** — correct cost, type line, P/T,
//! and keywords, but `Effect::Noop` (or a simplified placeholder) for
//! abilities that need engine features the engine doesn't yet have. Each stub
//! carries a doc-comment marking what's omitted; promote them as engine
//! features land.

mod creatures;
mod gift;
mod lands;
mod mayhem;
mod modern;
mod omen;
mod recent;
mod spells;
mod survival;
mod tarkir;
mod webslinging;

pub use creatures::*;
pub use gift::*;
pub use lands::*;
pub use mayhem::*;
pub use modern::*;
pub use omen::*;
pub use recent::*;
pub use spells::*;
pub use survival::*;
pub use tarkir::*;
pub use webslinging::*;
