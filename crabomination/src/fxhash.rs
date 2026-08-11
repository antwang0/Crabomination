//! Deterministic-order hash containers for the engine.
//!
//! `std`'s default `RandomState` is seeded from OS entropy per process
//! *and* bumped per map instance, so two `HashMap`s built from the same
//! insertions iterate differently — in the next process and in the same
//! one. Any map whose iteration reaches game logic (a `find` over
//! `values()`, a `collect()` into an ordered structure, a tie broken by
//! walk order) therefore makes a seeded game unreproducible, which is what
//! the cube pool's determinism failure was.
//!
//! Swapping the hasher fixes the class rather than the sites: order stays
//! a function of the keys and the insertions, both of which the seed
//! already determines. It is not a licence to depend on the order — a map
//! whose walk order decides a rules outcome is still a rules bug — but it
//! cannot silently desynchronize two runs any more.
//!
//! FxHash is the rustc hasher: no seed, one multiply-and-rotate per word.
//! Vendored rather than pulled in as a dependency because it is 30 lines
//! and the engine's determinism should not hinge on a version bump.
//! Not collision-resistant against chosen keys — the engine's keys are
//! `CardId`s, seat indices and `&'static str` card names, none of them
//! attacker-chosen, and the maps are board-sized.

use std::hash::{BuildHasherDefault, Hasher};

/// `HashMap` with a fixed hasher. Prefer this over `std::collections::HashMap`
/// everywhere in the engine. Note `HashMap::new()` does not exist for a
/// custom hasher — use `HashMap::default()`.
pub type HashMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;

/// `HashSet` with a fixed hasher. See [`HashMap`].
pub type HashSet<T> = std::collections::HashSet<T, FxBuildHasher>;

/// The `BuildHasher` behind [`HashMap`] / [`HashSet`]. Stateless, so
/// `Default` is the whole constructor and `Clone` is free.
pub type FxBuildHasher = BuildHasherDefault<FxHasher>;

/// Multiplier from rustc's `FxHasher` — the odd 64-bit constant from the
/// Fibonacci-hashing family.
const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// rustc's `FxHasher`: `hash = (hash.rotate_left(5) ^ word) * SEED`, one
/// round per machine word. Deterministic by construction — no state beyond
/// the accumulator.
#[derive(Debug, Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while let Some((chunk, tail)) = rest.split_first_chunk::<8>() {
            self.add(u64::from_ne_bytes(*chunk));
            rest = tail;
        }
        if let Some((chunk, tail)) = rest.split_first_chunk::<4>() {
            self.add(u32::from_ne_bytes(*chunk) as u64);
            rest = tail;
        }
        if let Some((chunk, tail)) = rest.split_first_chunk::<2>() {
            self.add(u16::from_ne_bytes(*chunk) as u64);
            rest = tail;
        }
        if let Some(&b) = rest.first() {
            self.add(b as u64);
        }
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.add(n as u64);
    }

    #[inline]
    fn write_u16(&mut self, n: u16) {
        self.add(n as u64);
    }

    #[inline]
    fn write_u32(&mut self, n: u32) {
        self.add(n as u64);
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.add(n);
    }

    #[inline]
    fn write_u128(&mut self, n: u128) {
        self.add(n as u64);
        self.add((n >> 64) as u64);
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.add(n as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;

    /// The property the whole module exists for: same insertions, same walk
    /// order — which `RandomState` does not give even twice in one process.
    #[test]
    fn iteration_order_is_a_function_of_the_insertions() {
        let build = || -> Vec<u32> {
            let mut m: HashMap<u32, u32> = HashMap::default();
            for i in 0..64u32 {
                m.insert(i * 7 + 1, i);
            }
            m.keys().copied().collect()
        };
        assert_eq!(build(), build());
    }

    /// A hasher with a fixed accumulator would pass the test above and be
    /// useless; check it actually mixes.
    #[test]
    fn distinct_keys_hash_distinctly() {
        let h = |x: u64| {
            let mut s = FxHasher::default();
            x.hash(&mut s);
            s.finish()
        };
        let n = (0..256u64).map(h).collect::<std::collections::BTreeSet<_>>().len();
        assert_eq!(n, 256, "collisions on 256 consecutive integers");
    }

    /// `write` has to consume every byte, tail included, or two strings
    /// differing only in their last byte collide.
    #[test]
    fn write_consumes_the_tail() {
        let h = |s: &str| {
            let mut st = FxHasher::default();
            st.write(s.as_bytes());
            st.finish()
        };
        assert_ne!(h("goblin guide"), h("goblin guidf"));
        assert_ne!(h("a"), h("b"));
        assert_ne!(h("abcdefghij"), h("abcdefghik"));
    }
}
