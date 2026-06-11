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
//! We pick option 1: a global dedup table. Each unique string leaks once
//! (`Box::leak`) and every later request for the same text returns the
//! cached `&'static str`, so the total leak is bounded by the number of
//! *unique* names — token mints in bot dry-run simulations no longer grow
//! memory per mint.
//!
//! Use as `#[serde(with = "crate::static_str_serde")]` on a
//! `&'static str` field.

use serde::de::Deserialize;
use serde::{Deserializer, Serializer};
use std::collections::HashSet;
use std::sync::Mutex;

/// Alias for `&'static str` serde fields. Serde's derive scans field types
/// *syntactically* for lifetimes and pins the whole struct to
/// `Deserialize<'static>` when it sees `&'static str` — even under a
/// `#[serde(with)]` adapter. The alias hides the lifetime from that scan,
/// keeping the containing struct deserializable for any `'de`.
pub type StaticStr = &'static str;

pub fn serialize<S: Serializer>(s: &&'static str, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(s)
}

pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<&'static str, D::Error> {
    let s = String::deserialize(de)?;
    Ok(intern(s))
}

static INTERN_TABLE: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);

/// Deduplicating interner: leaks each unique string once, returns the
/// cached `&'static str` thereafter. Shared by serde restores and every
/// non-serde caller that widens an owned string to `'static` (token
/// minting, snapshot restore).
pub fn intern(s: String) -> &'static str {
    let mut guard = INTERN_TABLE.lock().unwrap_or_else(|e| e.into_inner());
    let table = guard.get_or_insert_with(HashSet::new);
    if let Some(existing) = table.get(s.as_str()) {
        return existing;
    }
    let leaked: &'static str = Box::leak(s.into_boxed_str());
    table.insert(leaked);
    leaked
}
