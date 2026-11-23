//! The per-hit memo audits, and whether a board-wide audit is already running
//! on this thread.
//!
//! Under `debug_assertions` every `CardData` memo re-derives its value on a
//! hit, and every battlefield lane in the engine's `zone.rs` re-walks the board
//! on a read. A lane's walk reads those per-card memos, so audits nested — on a
//! debug pod game that was ~30 % of all samples (2026-10-02, mostly the
//! mana-summary memo under the mana-static lane).
//!
//! Inside a board-wide audit the per-card audits trust their memo: the board
//! audit checks the board memo against the cards, and each card memo is checked
//! on its own top-level reads, which are still re-derived on every hit (the
//! property the sampled gather-memo audit gave up). Release builds read nothing
//! here: [`memo_audit_eq!`](crate::memo_audit_eq) tests
//! `cfg!(debug_assertions)` first, and the board audits sit inside
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
