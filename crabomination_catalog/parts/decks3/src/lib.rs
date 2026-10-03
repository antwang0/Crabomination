//! One part of the card catalog. See `crabomination_catalog/src/sets/mod.rs`.

#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

// `crate::catalog::..` inside the card files resolves to this crate.
extern crate self as catalog;

// `crate::card`, `crate::effect`, `crate::mana`, `crate::game::..`, and the
// core's root re-exports (`crate::catalog::fractal_token`).
#[allow(unused_imports)]
use crabomination_catalog_core::*;

pub mod sets;
