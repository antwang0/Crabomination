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
mod freerunning;
mod gift;
mod innistrad;
mod lands;
mod ltr;
mod mayhem;
mod modern;
mod mom;
mod omen;
mod recent;
mod recent2;
mod recent3;
mod recent4;
mod recent5;
mod recent6;
mod recent7;
mod recent8;
mod recent9;
mod recent10;
mod recent11;
mod recent12;
mod recent13;
mod recent14;
mod recent15;
mod recent16;
mod recent17;
mod recent18;
mod recent19;
mod recent20;
mod recent21;
mod recent22;
mod recent23;
mod recent24;
mod spells;
mod survival;
mod tarkir;
mod webslinging;

pub use creatures::*;
pub use freerunning::*;
pub use gift::*;
pub use innistrad::*;
pub use lands::*;
pub use ltr::*;
pub use mayhem::*;
pub use modern::*;
pub use mom::*;
pub use omen::*;
pub use recent::*;
pub use recent2::*;
pub use recent3::*;
pub use recent4::*;
pub use recent5::*;
pub use recent6::*;
pub use recent7::*;
pub use recent8::*;
pub use recent9::*;
pub use recent10::*;
pub use recent11::*;
pub use recent12::*;
pub use recent13::*;
pub use recent14::*;
pub use recent15::*;
pub use recent16::*;
pub use recent17::*;
pub use recent18::*;
pub use recent19::*;
pub use recent20::*;
pub use recent21::*;
pub use recent22::*;
pub use recent23::*;
pub use recent24::*;
pub use spells::*;
pub use survival::*;
pub use tarkir::*;
pub use webslinging::*;
