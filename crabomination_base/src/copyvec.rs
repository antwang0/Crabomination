//! `CopyVec` — a [`SmallVec`] of `Copy` items whose `Clone` is a `memcpy`.
//!
//! `smallvec`'s own `Clone` is `SmallVec::from(self.as_slice())`, which lands
//! in `extend_from_slice` → `Extend::extend`: a `reserve` plus an *external*
//! `next()` loop, even for a `Copy` item and even when the vector is empty.
//! `SmallVec::from_slice` is a `ptr::copy_nonoverlapping` and is on stable for
//! exactly this case; the crate reaches it only through an inherent method,
//! never through the `Clone` impl.
//!
//! That matters because these vectors live in the CoW'd state groups, so they
//! are cloned once per *deep copy* rather than once per game event:
//! `SmallVec as Extend::extend` is **337,280 calls / 22.7 M Ir / 0.81 % of
//! `cube`**, and 251,044 of those calls are under
//! `Arc::clone_from_ref_in` or `GameState::clone` (ninety-eighth pass).
//!
//! The wrapper is `Deref`/`DerefMut` to `SmallVec`, so every read and write
//! site keeps compiling unchanged; only the field's declared type and its
//! construction move.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::{Array, SmallVec};
use std::ops::{Deref, DerefMut};

/// A `SmallVec<A>` whose `Clone` copies rather than iterates. `A::Item` must
/// be `Copy` for the `Clone` impl to exist at all, which is what makes the
/// memcpy sound.
pub struct CopyVec<A: Array>(SmallVec<A>);

impl<A: Array> CopyVec<A> {
    #[inline]
    pub fn new() -> Self {
        Self(SmallVec::new())
    }
}

impl<A: Array> Default for CopyVec<A> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Array> Clone for CopyVec<A>
where
    A::Item: Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(SmallVec::from_slice(self.0.as_slice()))
    }
}

impl<A: Array> Deref for CopyVec<A> {
    type Target = SmallVec<A>;
    #[inline]
    fn deref(&self) -> &SmallVec<A> {
        &self.0
    }
}

impl<A: Array> DerefMut for CopyVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut SmallVec<A> {
        &mut self.0
    }
}

impl<A: Array> AsRef<[A::Item]> for CopyVec<A> {
    #[inline]
    fn as_ref(&self) -> &[A::Item] {
        self.0.as_slice()
    }
}

impl<A: Array> std::fmt::Debug for CopyVec<A>
where
    A::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<A: Array> PartialEq for CopyVec<A>
where
    A::Item: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A: Array> Eq for CopyVec<A> where A::Item: Eq {}

impl<A: Array> From<SmallVec<A>> for CopyVec<A> {
    fn from(v: SmallVec<A>) -> Self {
        Self(v)
    }
}

impl<A: Array> FromIterator<A::Item> for CopyVec<A> {
    fn from_iter<I: IntoIterator<Item = A::Item>>(iter: I) -> Self {
        Self(SmallVec::from_iter(iter))
    }
}

impl<A: Array> IntoIterator for CopyVec<A> {
    type Item = A::Item;
    type IntoIter = smallvec::IntoIter<A>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Deref coercion doesn't apply to `for` loops, so forward both borrowing arms.
impl<'a, A: Array> IntoIterator for &'a CopyVec<A> {
    type Item = &'a A::Item;
    type IntoIter = std::slice::Iter<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, A: Array> IntoIterator for &'a mut CopyVec<A> {
    type Item = &'a mut A::Item;
    type IntoIter = std::slice::IterMut<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<A: Array> Serialize for CopyVec<A>
where
    A::Item: Serialize,
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de, A: Array> Deserialize<'de> for CopyVec<A>
where
    A::Item: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        SmallVec::<A>::deserialize(d).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_round_trips_through_the_inline_and_spilled_forms() {
        let mut v: CopyVec<[u32; 2]> = CopyVec::new();
        assert_eq!(v.clone().as_slice(), &[] as &[u32]);
        v.push(1);
        v.push(2);
        assert!(!v.spilled());
        assert_eq!(v.clone().as_slice(), &[1, 2]);
        v.push(3);
        assert!(v.spilled(), "past the inline capacity");
        assert_eq!(v.clone().as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn writes_and_reads_go_through_deref() {
        let mut v: CopyVec<[u32; 4]> = CopyVec::default();
        v.extend_from_slice(&[5, 6, 7]);
        assert!(v.contains(&6));
        v.retain(|x| *x != 6);
        assert_eq!(v.as_slice(), &[5, 7]);
        assert_eq!((&v).into_iter().copied().sum::<u32>(), 12);
        for x in &mut v {
            *x += 1;
        }
        assert_eq!(v.as_slice(), &[6, 8]);
        v.clear();
        assert!(v.is_empty());
    }

    #[test]
    fn serde_round_trips_as_a_bare_list() {
        let v: CopyVec<[u32; 2]> = vec![1u32, 2, 3].into_iter().collect();
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "[1,2,3]");
        let back: CopyVec<[u32; 2]> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.as_slice(), &[1, 2, 3]);
    }
}
