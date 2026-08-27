//! Copy-on-write box for snapshot-heavy state.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Copy-on-write box: `Clone` is an `Arc` bump; the first `&mut` access
/// after a clone copies the inner value (`Arc::make_mut`). `Deref`/
/// `DerefMut` make it read like the plain type at call sites.
///
/// Wraps the card-zone collections in `GameState` and `Player`, and the
/// rarely-written field groups (`PlayerCold`, `ColdState`), so a state clone
/// — dry-run probes, the `perform_action` transaction checkpoint, undo
/// snapshots — costs reference bumps instead of deep copies. A probe then
/// pays only for the zones its action actually mutates.
///
/// Sharp edge: any `&mut` access (including `iter_mut` used read-only)
/// copies the whole inner value while a snapshot shares it. That is never
/// *worse* than the eager clone this replaces, but prefer `&self` access
/// on hot read paths. For the card zones the edge is blunt:
/// [`CardInstance`](crate::card::CardInstance) is itself a CoW handle, so
/// unsharing a zone copies a vector of pointers and only the cards actually
/// written pay a deep clone.
#[derive(Debug)]
pub struct CowBox<T: Clone>(Arc<T>);

impl<T: Clone> CowBox<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }

    /// Consume the box, returning the inner value (cloning only when a
    /// snapshot still shares it).
    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.0).unwrap_or_else(|arc| (*arc).clone())
    }

    /// True when both handles still share one allocation — i.e. neither has
    /// been written since the clone. Test-only observability for the CoW
    /// contract; the engine never branches on it.
    pub fn shares_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// The one write shape that is not a general `DerefMut`: appending.
///
/// `Arc::make_mut` materializes with `Vec::clone`, which hands back
/// `capacity == len` — so the push that follows the first write after a state
/// clone **always** reallocates, two allocations for one appended element.
/// Materializing with room for it removes the second. Inherent, so it shadows
/// the `Deref`'d `Vec::push` at every existing call site without touching one.
///
/// Same device as `layers::Printed<Vec<_>>::push` one level out: there the
/// copy is the layer override, here it is the CoW unshare.
impl<T: Clone> CowBox<Vec<T>> {
    #[inline]
    pub fn push(&mut self, value: T) {
        if let Some(v) = Arc::get_mut(&mut self.0) {
            v.push(value);
            return;
        }
        let mut v = Vec::with_capacity(self.0.len() + 1);
        v.extend_from_slice(&self.0);
        v.push(value);
        self.0 = Arc::new(v);
    }
}

impl<T: Clone> Clone for CowBox<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: Clone + Default> Default for CowBox<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Deref for CowBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> DerefMut for CowBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
}

impl<T: Clone> From<T> for CowBox<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Clone + PartialEq> PartialEq for CowBox<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<T: Clone + Eq> Eq for CowBox<T> {}

// `for x in &zone` / `for x in &mut zone` — deref coercion doesn't apply
// to `for` loops, so forward IntoIterator explicitly.
impl<'a, T: Clone> IntoIterator for &'a CowBox<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&*self.0).into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut CowBox<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).into_iter()
    }
}

impl<T: Clone + serde::Serialize> serde::Serialize for CowBox<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Clone + serde::Deserialize<'de>> serde::Deserialize<'de> for CowBox<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_shares_until_written() {
        let mut a: CowBox<Vec<u32>> = vec![1, 2, 3].into();
        let b = a.clone();
        assert!(a.shares_with(&b), "clone shares the allocation");
        a.push(4);
        assert!(!a.shares_with(&b), "first write unshares");
        assert_eq!(*a, vec![1, 2, 3, 4]);
        assert_eq!(*b, vec![1, 2, 3], "the snapshot kept the old value");
    }
}
