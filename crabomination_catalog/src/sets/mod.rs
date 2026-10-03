//! The card catalog's `sets` module, assembled from its parts.
//!
//! One submodule per Magic set, named by the set's three-letter code; demo,
//! Commander and recent-set cards under `decks`. The card files all live
//! under this directory, but they are compiled by nine crates under
//! `crabomination_catalog/parts/`, each naming its files with `#[path]`, and
//! this module glob-re-exports them back into the one tree every caller uses
//! (`catalog::sets::stx::..`, `catalog::sets::decks::..`, `catalog::foo()`).
//!
//! **Why.** As one 745 k-line crate the catalog cost every card edit ~25 s
//! before anything downstream started, and nearly all of it was proportional
//! to the crate rather than to the edit: 9.3 s rewriting a 343 MB rlib and
//! 8.5 s re-serializing a 2 GB incremental dep graph (`-Ztime-passes`,
//! 2026-10-02). A part is a ninth of that, and the parts compile in parallel
//! on a cold build. PERF.md's build-time section has the A/B.
//!
//! **Where a new file goes.**
//! * A file under `decks/`: `parts/decks4/src/decks.rs` (two lines, `mod` and
//!   `pub use`), unless it uses another decks file's helpers
//!   (`super::woe_roles::..`), in which case it joins that file's part.
//! * A new set module: `parts/sets3/src/sets.rs`.
//! * A helper several sets share: `helpers.rs` (the core part).
//! A file declared in this crate instead (`pub mod foo;`) still builds — the
//! facade sees the core's helpers and every part — but the core_rules test
//! `every_card_file_is_compiled_by_exactly_one_catalog_part` names it, and it
//! recompiles with the facade on every card edit.
//!
//! **What a part may reference.** Its own files, and the core's `sets::*`
//! (helpers plus `cmdr`, `lea`, `one`, `rna`, `sos`, `thb`). Nothing in
//! another part: parts depend on the core only, so they build in parallel,
//! and a path into another part is a compile error, not a silent coupling.
//! A helper that crosses parts is `pub`, not `pub(crate)`.

pub use crabomination_catalog_core::sets::*;
pub use crabomination_catalog_sets1::sets::*;
pub use crabomination_catalog_sets2::sets::*;
pub use crabomination_catalog_sets3::sets::*;
pub use crabomination_catalog_stx::sets::*;
// The decks parts arrive through `decks/mod.rs`.

pub mod all_factories;
pub mod decks;
