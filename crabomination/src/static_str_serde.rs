//! Serde adapter for `&'static str` fields.
//!
//! The catalog uses string-literal `&'static str` for card names, ability
//! descriptions, and the like. To round-trip those through JSON we need
//! to widen them to owned `String` on the wire, then narrow back to
//! `&'static str` on load. There is no safe way to extend a `String`'s
//! lifetime to `'static` without either:
//!
//! 1. interning into a global table (the principled approach), or
//! 2. leaking the heap allocation via `Box::leak` (cheap, finite cost
//!    for a debug-replay workflow that loads a handful of states per
//!    session).
//!
//! We pick option 2: every restored snapshot leaks a few hundred bytes
//! per non-catalog string, which is acceptable for the debug-export
//! lifecycle. Catalog-sourced strings (card names) are deduped via
//! `name_index` in `catalog::lookup_by_name`, so the leak is bounded by
//! the catalog size on first load and zero thereafter.
//!
//! Use as `#[serde(with = "crate::static_str_serde")]` on a
//! `&'static str` field.

use serde::de::Deserialize;
use serde::{Deserializer, Serializer};

pub fn serialize<S: Serializer>(s: &&'static str, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(s)
}

pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<&'static str, D::Error> {
    let s = String::deserialize(de)?;
    Ok(intern(s))
}

/// Re-export of the leaking interner so non-serde callers (e.g. snapshot
/// restore that builds `&'static str` fields from owned strings) can use
/// the same memory-management contract.
pub fn intern(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
