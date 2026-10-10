//! CR 701.23 — "search your library for up to N cards" is ONE search. The
//! engine runs it as N single picks (`Effect::SearchUpToN`), and each pick
//! used to search anew: Ob Nixilis Unshackled's "whenever an opponent
//! searches their library" fired once per card, and Leonin Arbiter's {2}
//! was charged once per card — an unpaid tax on the first pick was simply
//! asked again on the next.
//!
//! The batch state makes the first pick the search: its prohibitions, tax
//! and `PlayerSearchedLibrary` event stand for the batch, and every later
//! pick either goes ahead silently or, when the first was refused, finds
//! nothing too.

/// Where the running `SearchUpToN` batch stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SearchBatch {
    /// No batch: each search is its own (the plain `Search` path).
    #[default]
    Idle,
    /// A batch whose first pick hasn't searched yet.
    First,
    /// The batch searched: later picks skip the gate.
    Searched,
    /// The first pick was refused (can't search, unpaid tax): so is the rest.
    Refused,
}

impl SearchBatch {
    /// The gate for one pick: `None` runs the usual checks (and then reports
    /// through [`SearchBatch::settle`]); `Some(go)` skips them, searching
    /// only when `go`.
    pub(crate) fn skip_gate(self) -> Option<bool> {
        match self {
            SearchBatch::Idle | SearchBatch::First => None,
            SearchBatch::Searched => Some(true),
            SearchBatch::Refused => Some(false),
        }
    }

    /// The state after a gated pick searched (`ok`) or was refused.
    pub(crate) fn settle(self, ok: bool) -> SearchBatch {
        match self {
            SearchBatch::First if ok => SearchBatch::Searched,
            SearchBatch::First => SearchBatch::Refused,
            other => other,
        }
    }
}
