//! `OftenEmpty` — a `Vec` whose `Clone` tests `is_empty()` first.
//!
//! `Vec::clone` is an out-of-line call that builds a `RawVec` and copies
//! `len` elements even when `len` is 0 — ~45 Ir for a `Vec::new()`. The
//! per-object lists on `CardData` are empty on nearly every permanent and
//! are cloned on every CoW unshare of the card, so the empties were most of
//! `Vec::clone`'s 471 k calls on a six-game `cube` run (PERF `(-200)`).
//! The guard inlines to a length test; a non-empty list clones exactly as
//! before. Same size as the `Vec` it wraps, so `(-165)`'s owner-growth rule
//! does not apply.
//!
//! `Deref`/`DerefMut` make it read and write like the `Vec`; the iterator
//! and `From` impls cover the sites `Deref` cannot reach (`for x in &list`,
//! struct literals, `assert_eq!` against a `Vec`).

use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct OftenEmpty<T>(pub Vec<T>);

/// Manual so `T` needs no `Default` of its own.
impl<T> Default for OftenEmpty<T> {
    #[inline]
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: Clone> Clone for OftenEmpty<T> {
    #[inline]
    fn clone(&self) -> Self {
        if self.0.is_empty() { Self(Vec::new()) } else { Self(self.0.clone()) }
    }
}

impl<T> Deref for OftenEmpty<T> {
    type Target = Vec<T>;
    #[inline]
    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> DerefMut for OftenEmpty<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for OftenEmpty<T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> From<OftenEmpty<T>> for Vec<T> {
    #[inline]
    fn from(v: OftenEmpty<T>) -> Self {
        v.0
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for OftenEmpty<T> {
    #[inline]
    fn eq(&self, other: &Vec<T>) -> bool {
        self.0 == *other
    }
}

impl<T> FromIterator<T> for OftenEmpty<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl<T> IntoIterator for OftenEmpty<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a OftenEmpty<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OftenEmpty<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::OftenEmpty;

    #[test]
    fn clone_keeps_contents_and_empties_stay_unallocated() {
        let full: OftenEmpty<u32> = vec![1, 2, 3].into();
        assert_eq!(full.clone(), vec![1, 2, 3]);
        let empty: OftenEmpty<u32> = OftenEmpty::default();
        let c = empty.clone();
        assert!(c.is_empty());
        assert_eq!(c.capacity(), 0);
    }
}
