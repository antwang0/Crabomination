//! A bot token estimate's per-seat multipliers, read once. The estimates are
//! `&self`, so the board can't change under them, but each mint they price
//! walked the battlefield for doublers and triplers: a 3,000-Scute board
//! priced one landfall per Swarm and one stacked trigger per Swarm trigger,
//! and a 6-seat strict sweep block (seed 2942180) ran past 20 minutes.

use std::cell::RefCell;

use super::GameState;

thread_local! {
    /// Open scopes, the state they read, and each seat's (doublers, triplers).
    static MEMO: RefCell<(u32, usize, Vec<[Option<u32>; 2]>)> = const { RefCell::new((0, 0, Vec::new())) };
}

/// Held for one estimate: while any is alive, [`cached`] serves the state it
/// was opened on, which it borrows so the board can't change under it.
/// Nested scopes share the outermost one's memo.
pub(crate) struct Scope<'a>(std::marker::PhantomData<&'a GameState>);

impl<'a> Scope<'a> {
    pub(crate) fn open(state: &'a GameState) -> Scope<'a> {
        MEMO.with_borrow_mut(|(depth, addr, _)| {
            if *depth == 0 {
                *addr = state as *const GameState as usize;
            }
            *depth += 1;
        });
        Scope(std::marker::PhantomData)
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        MEMO.with_borrow_mut(|(depth, _, seats)| {
            *depth -= 1;
            if *depth == 0 {
                seats.clear();
            }
        });
    }
}

/// `compute()` for `seat`'s multiplier `kind` (0 doublers, 1 triplers), from
/// the open scope's memo when one is open on `state`.
pub(crate) fn cached(state: &GameState, seat: usize, kind: usize, compute: impl FnOnce() -> u32) -> u32 {
    let addr = state as *const GameState as usize;
    let hit = MEMO.with_borrow(|(depth, a, seats)| {
        (*depth > 0 && *a == addr).then(|| seats.get(seat).and_then(|s| s[kind]))
    });
    match hit {
        None => compute(),
        Some(Some(v)) => v,
        Some(None) => {
            let v = compute();
            MEMO.with_borrow_mut(|(_, _, seats)| {
                if seats.len() <= seat {
                    seats.resize(seat + 1, [None; 2]);
                }
                seats[seat][kind] = Some(v);
            });
            v
        }
    }
}
