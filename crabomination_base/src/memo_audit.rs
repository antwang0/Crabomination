//! The per-hit memo audits, and the one thing they need to know: whether a
//! board-wide audit is already running on this thread.
//!
//! Every `CardData` memo re-derives its value on a hit under
//! `debug_assertions` ("memo is stale"), and every battlefield lane in the
//! engine's `zone.rs` re-walks the board on a read the same way. A lane's walk
//! goes through those same per-card memo reads, so each audited lane read also
//! re-derived every permanent's memos — nested audits, one full re-derivation
//! per permanent per lane read. On a debug pod game that was ~30 % of all
//! samples, nearly all of it the mana-summary memo re-derived under the
//! mana-static lane's walk (100 `gdb` samples of
//! `cr_903_a_pod_run_is_independent_of_how_it_is_chunked`, 2026-10-02).
//!
//! Inside a board-wide audit the per-card audits trust their memo: the board
//! audit is checking the *board* memo against the cards, and each card memo is
//! checked on its own top-level reads. Outside one nothing changes — every
//! top-level read is still re-derived on every hit, which is the property the
//! sampled gather-memo audit (`GameState::gather_memo_agrees`) gave up and these
//! keep. Release builds read nothing here: [`memo_audit_eq!`](crate::memo_audit_eq)
//! tests `cfg!(debug_assertions)` first, and the board audits sit inside
//! `debug_assert!`s.

thread_local! {
    static AUDITING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// True while a [`board_audit`] runs on this thread: a memo hit skips its
/// own re-derivation.
#[inline]
pub fn nested() -> bool {
    AUDITING.with(std::cell::Cell::get)
}

/// Run a board-wide audit's walk with the per-card memo audits off, and hand
/// back what it computed.
pub fn board_audit<R>(walk: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            AUDITING.with(|a| a.set(self.0));
        }
    }
    let _restore = Restore(AUDITING.with(|a| a.replace(true)));
    walk()
}

/// `debug_assert_eq!` for a memo hit: skipped inside a [`board_audit`], and
/// in a release build without reading the flag at all.
#[macro_export]
macro_rules! memo_audit_eq {
    ($($t:tt)*) => {
        if cfg!(debug_assertions) && !$crate::memo_audit::nested() {
            debug_assert_eq!($($t)*);
        }
    };
}
