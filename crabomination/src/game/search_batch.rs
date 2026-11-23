//! CR 701.23 — "search your library for up to N cards" is one search, run as
//! N picks (`Effect::SearchUpToN`). The first pick is the search: its
//! prohibitions, tax (Leonin Arbiter) and `PlayerSearchedLibrary` event (Ob
//! Nixilis Unshackled) stand for the batch; later picks proceed silently, or
//! find nothing if the first was refused.

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
