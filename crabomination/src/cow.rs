//! Copy-on-write box for snapshot-heavy state.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Copy-on-write box: `Clone` is an `Arc` bump; the first `&mut` access
/// after a clone copies the inner value (`Arc::make_mut`). `Deref`/
/// `DerefMut` make it read like the plain type at call sites.
///
/// Wraps the card-zone collections in [`GameState`](crate::game::GameState)
/// and [`Player`](crate::player::Player) so a state clone — dry-run
/// probes, the `perform_action` transaction checkpoint, undo snapshots —
/// costs reference bumps instead of deep copies. A probe then pays only
/// for the zones its action actually mutates.
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
        assert!(Arc::ptr_eq(&a.0, &b.0), "clone shares the allocation");
        a.push(4);
        assert!(!Arc::ptr_eq(&a.0, &b.0), "first write unshares");
        assert_eq!(*a, vec![1, 2, 3, 4]);
        assert_eq!(*b, vec![1, 2, 3], "the snapshot kept the old value");
    }

    /// The `perform_action` transaction contract: a rejected action
    /// restores the exact pre-action state (TODO "Rollback / Undo"
    /// Phase 1). Serialized-state equality is the strongest observable
    /// check without a `PartialEq` on `GameState`.
    #[test]
    fn rejected_action_restores_state_exactly() {
        use crate::game::{GameAction, GameState};
        use crate::player::Player;
        let mut g = GameState::new(vec![Player::new(0, "A"), Player::new(1, "B")]);
        let id = g.add_card_to_hand(0, crate::catalog::shivan_dragon());
        g.priority.player_with_priority = 0;
        let before = serde_json::to_string(&g).unwrap();
        // No mana in pool and no lands: the cast is rejected during payment.
        let r = g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert!(r.is_err(), "unpayable cast is rejected");
        assert_eq!(serde_json::to_string(&g).unwrap(), before, "rejected action left no trace");
    }

    #[test]
    fn iteration_and_serde_round_trip() {
        let zone: CowBox<Vec<u32>> = vec![5, 6].into();
        let doubled: Vec<u32> = (&zone).into_iter().map(|x| x * 2).collect();
        assert_eq!(doubled, vec![10, 12]);
        let json = serde_json::to_string(&zone).unwrap();
        assert_eq!(json, "[5,6]");
        let back: CowBox<Vec<u32>> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, zone);
    }
}
