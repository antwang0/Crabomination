//! Zone wrappers that carry a maintained answer alongside their cards.
//!
//! A zone whose whole-collection walk is hot enough to want a presence gate
//! has nowhere to put one: the walk *is* the gate. The wrapper is where the
//! answer lives, and its `&mut` entry points are where it is invalidated —
//! so the invalidation is a property of the type rather than an enumeration
//! of write sites that has to stay right.

use crate::card::CardInstance;
use crate::cow::CowBox;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU8, Ordering};

/// Memo states for [`Graveyard::has_anthem`].
const UNKNOWN: u8 = 0;
const ABSENT: u8 = 1;
const PRESENT: u8 = 2;

/// A player's graveyard: the CoW card list, plus the one question two hot
/// walkers ask of it — "does any card here carry a
/// [`StaticEffect::GraveyardAnthem`](crate::effect::StaticEffect::GraveyardAnthem)?"
/// (the Incarnation cycle's "as long as this card is in your graveyard and you
/// control a [Land subtype] …").
///
/// `gather_continuous_effects_inner`'s anthem pass and
/// `keyword_grant_in_scope`'s off-battlefield tail are both ungated
/// whole-zone walks over a zone that grows all game; deleting the pair is
/// `fixed` -0.717 % / `cube` -0.423 % (eighty-fourth pass).
///
/// The memo is sound *by construction*: [`DerefMut`] and the `&mut`
/// [`IntoIterator`] are the only ways to reach the cards mutably, and both
/// clear it — the enumeration of graveyard write sites never has to be right.
/// [`Deref`] leaves every read site untouched.
#[derive(Debug, Default)]
pub struct Graveyard {
    cards: CowBox<Vec<CardInstance>>,
    anthem: AtomicU8,
}

impl Graveyard {
    /// True when some card in this graveyard carries a `GraveyardAnthem`
    /// static. Memoized; a miss walks the zone once.
    pub fn has_anthem(&self) -> bool {
        match self.anthem.load(Ordering::Relaxed) {
            ABSENT => false,
            PRESENT => true,
            _ => {
                let found = self.cards.iter().any(|c| {
                    c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, crate::effect::StaticEffect::GraveyardAnthem { .. })
                    })
                });
                self.anthem.store(if found { PRESENT } else { ABSENT }, Ordering::Relaxed);
                found
            }
        }
    }

    /// True when both handles still share one allocation — the [`CowBox`]
    /// contract, forwarded for the zone tests.
    pub fn shares_with(&self, other: &Self) -> bool {
        self.cards.shares_with(&other.cards)
    }

    /// The memo's current state, for the invalidation tests: `None` while
    /// unknown.
    #[cfg(test)]
    fn memo(&self) -> Option<bool> {
        match self.anthem.load(Ordering::Relaxed) {
            ABSENT => Some(false),
            PRESENT => Some(true),
            _ => None,
        }
    }
}

impl Clone for Graveyard {
    /// The cards clone as a `CowBox` (a refcount bump); the memo describes
    /// those same cards, so it comes along.
    fn clone(&self) -> Self {
        Self {
            cards: self.cards.clone(),
            anthem: AtomicU8::new(self.anthem.load(Ordering::Relaxed)),
        }
    }
}

impl Deref for Graveyard {
    type Target = Vec<CardInstance>;
    fn deref(&self) -> &Vec<CardInstance> {
        &self.cards
    }
}

impl DerefMut for Graveyard {
    fn deref_mut(&mut self) -> &mut Vec<CardInstance> {
        self.anthem.store(UNKNOWN, Ordering::Relaxed);
        &mut self.cards
    }
}

impl From<Vec<CardInstance>> for Graveyard {
    fn from(cards: Vec<CardInstance>) -> Self {
        Self { cards: cards.into(), anthem: AtomicU8::new(UNKNOWN) }
    }
}

impl From<CowBox<Vec<CardInstance>>> for Graveyard {
    fn from(cards: CowBox<Vec<CardInstance>>) -> Self {
        Self { cards, anthem: AtomicU8::new(UNKNOWN) }
    }
}

// `for c in &gy` / `for c in &mut gy` — deref coercion doesn't apply to `for`
// loops, so forward `IntoIterator` explicitly. The `&mut` arm is the second
// invalidation point.
impl<'a> IntoIterator for &'a Graveyard {
    type Item = &'a CardInstance;
    type IntoIter = std::slice::Iter<'a, CardInstance>;
    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter()
    }
}

impl<'a> IntoIterator for &'a mut Graveyard {
    type Item = &'a mut CardInstance;
    type IntoIter = std::slice::IterMut<'a, CardInstance>;
    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().iter_mut()
    }
}

impl serde::Serialize for Graveyard {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.cards.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Graveyard {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        CowBox::<Vec<CardInstance>>::deserialize(deserializer).map(Graveyard::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardId, CardInstance};

    fn gy(defs: Vec<crate::card::CardDefinition>) -> Graveyard {
        defs.into_iter()
            .enumerate()
            .map(|(i, d)| CardInstance::new(CardId(i as u32), d, 0))
            .collect::<Vec<_>>()
            .into()
    }

    /// The Incarnation cycle is the only `GraveyardAnthem` shape, and the memo
    /// has to see it.
    #[test]
    fn anthem_presence_is_detected_and_memoized() {
        let g = gy(vec![crate::catalog::wonder(), crate::catalog::shivan_dragon()]);
        assert_eq!(g.memo(), None, "lazy until asked");
        assert!(g.has_anthem());
        assert_eq!(g.memo(), Some(true));

        let plain = gy(vec![crate::catalog::shivan_dragon()]);
        assert!(!plain.has_anthem());
        assert_eq!(plain.memo(), Some(false));
    }

    /// The whole point: a write invalidates, and a read does not.
    #[test]
    fn writes_invalidate_and_reads_do_not() {
        let mut g = gy(vec![crate::catalog::shivan_dragon()]);
        assert!(!g.has_anthem());
        assert_eq!(g.memo(), Some(false));
        // Reads keep the memo.
        assert_eq!(g.iter().count(), 1);
        assert_eq!((&g).into_iter().count(), 1);
        assert_eq!(g.memo(), Some(false), "a read does not invalidate");
        // A push through `DerefMut` clears it, and the answer moves with it.
        g.push(CardInstance::new(CardId(99), crate::catalog::wonder(), 0));
        assert_eq!(g.memo(), None, "the write invalidated");
        assert!(g.has_anthem());
        // So does `&mut` iteration, even used read-only.
        for _ in &mut g {}
        assert_eq!(g.memo(), None, "&mut iteration invalidated");
    }

    /// The CoW contract survives the wrapper, and the memo travels with the
    /// snapshot it describes.
    #[test]
    fn clone_shares_and_carries_the_memo() {
        let mut a = gy(vec![crate::catalog::wonder()]);
        assert!(a.has_anthem());
        let b = a.clone();
        assert!(a.shares_with(&b), "clone is a refcount bump");
        assert_eq!(b.memo(), Some(true), "the memo describes the same cards");
        a.pop();
        assert!(!a.shares_with(&b), "the write unshared");
        assert_eq!(b.memo(), Some(true), "the snapshot's answer is still right");
        assert!(!a.has_anthem());
    }

    #[test]
    fn serde_round_trips_as_a_bare_list() {
        let g = gy(vec![crate::catalog::shivan_dragon()]);
        let json = serde_json::to_string(&g).unwrap();
        assert!(json.starts_with('['), "serializes as the card list: {json}");
        let back: Graveyard = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back.memo(), None, "a fresh deserialize starts unknown");
    }
}
