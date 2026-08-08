//! Game-scoped randomness (`GameRng`).
//!
//! Everything the *rules* randomize — mulligan reshuffles, "at random"
//! discards, random target picks, a shuffled library — draws from the
//! `GameState`'s own stream rather than the thread RNG. Two reasons, both
//! of which had already cost us:
//!
//! * **Reproducibility.** A seeded game has to replay identically, in this
//!   process and the next. With `rand::rng()` in the mulligan path a fixed
//!   seed reproduced the deal and then diverged the moment either seat
//!   mulliganed, so a self-play trace could not be replayed and a crash at
//!   game 400 000 could not be re-run.
//! * **Paired ladder validity.** `simulate_match_pairs_piloted` plays one
//!   shuffle from both seats so the deal cancels. Any reshuffle drawn from
//!   the thread RNG un-cancels it silently — the pairing still *reports* a
//!   variance reduction, just a smaller one than it should, on exactly the
//!   games (mulligans) where the deal matters most.
//!
//! Interior mutability (an atomic, not a `Cell`, so `GameState` stays
//! `Sync`) so a draw needs only `&self`: the call sites are overwhelmingly
//! `self.players[x].library.shuffle(...)`, which cannot also borrow `self`
//! mutably. `Relaxed` is the right ordering — a `GameState` is owned by one
//! thread at a time, and the atomic is here for the trait bound, not for
//! cross-thread communication.

use std::sync::atomic::{AtomicU64, Ordering};

/// SplitMix64 — the seeding PRNG from the xoshiro family. Chosen over
/// `StdRng` because it is one `u64` of state (so cloning a `GameState` for
/// a probe is free) and has no `&mut` requirement to work around.
#[derive(Debug)]
pub struct GameRng(AtomicU64);

const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

impl GameRng {
    /// A stream pinned to `seed`. Same seed, same game.
    pub fn seeded(seed: u64) -> Self {
        Self(AtomicU64::new(seed))
    }

    /// A stream from OS entropy — what a real match gets, so hands are not
    /// predictable from the outside.
    pub fn from_entropy() -> Self {
        use rand::RngExt;
        Self::seeded(rand::rng().random())
    }

    /// Re-pin an existing state's stream. Used by the seeded simulators
    /// after the template clone.
    pub fn reseed(&self, seed: u64) {
        self.0.store(seed, Ordering::Relaxed);
    }

    /// A `rand`-compatible handle. Borrows `&self`, so it composes with a
    /// `&mut` borrow of any *other* field of the same struct.
    pub fn draw(&self) -> Draw<'_> {
        Draw(self)
    }

    /// A child stream for a subgame, derived from (and advancing) this one
    /// so the parent's determinism carries into it.
    pub fn fork(&self) -> Self {
        Self::seeded(self.next_u64())
    }

    fn next_u64(&self) -> u64 {
        let s = self.0.fetch_add(GAMMA, Ordering::Relaxed).wrapping_add(GAMMA);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Cloning a state clones the stream *position*, so a probe continues
/// where the real game is and the real game is unaffected by the probe.
impl Clone for GameRng {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::Relaxed)))
    }
}

/// A deserialized snapshot gets a fresh unpredictable stream: the position
/// isn't serialized (it isn't part of the visible game state, and pinning
/// it would let a client that can read a snapshot predict its own deck).
impl Default for GameRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

/// Borrowing `rand` handle — see [`GameRng::draw`].
pub struct Draw<'a>(&'a GameRng);

impl rand::TryRng for Draw<'_> {
    type Error = std::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.0.next_u64() as u32)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.next_u64())
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        for chunk in dst.chunks_mut(8) {
            let bytes = self.0.next_u64().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn same_seed_same_stream() {
        let (a, b) = (GameRng::seeded(9), GameRng::seeded(9));
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn a_clone_continues_the_stream_without_disturbing_it() {
        let a = GameRng::seeded(3);
        let _ = a.next_u64();
        let b = a.clone();
        // The clone's next draw is the one the original would have made,
        // and burning it on the clone leaves the original where it was.
        let from_clone = b.next_u64();
        assert_eq!(from_clone, a.next_u64());
    }

    #[test]
    fn shuffles_reproduce() {
        let shuffled = |seed| {
            let r = GameRng::seeded(seed);
            let mut v: Vec<u32> = (0..40).collect();
            v.shuffle(&mut r.draw());
            v
        };
        assert_eq!(shuffled(1), shuffled(1));
        assert_ne!(shuffled(1), shuffled(2));
    }

    /// `fill_bytes` has to fill a non-multiple-of-8 tail, not panic on it.
    #[test]
    fn fill_bytes_handles_a_short_tail() {
        let r = GameRng::seeded(5);
        let mut buf = [0u8; 13];
        let mut d = r.draw();
        rand::Rng::fill_bytes(&mut d, &mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }
}
