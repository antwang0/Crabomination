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

/// Does this card carry a `GraveyardAnthem` static? The per-card half of
/// [`Graveyard::has_anthem`].
fn card_has_anthem(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::GraveyardAnthem { .. }))
}

impl Graveyard {
    /// True when some card in this graveyard carries a `GraveyardAnthem`
    /// static. Memoized; a miss walks the zone once.
    ///
    /// The memo is sound by construction, and the `debug_assert!` is the
    /// audit anyway: it costs nothing in any release profile and turns the
    /// whole suite — and any `-C debug-assertions=yes` ladder run, which deals
    /// far more interesting graveyards than 18,793 tests do — into a check
    /// that no write path ever reached the cards without clearing it.
    pub fn has_anthem(&self) -> bool {
        debug_assert!(
            self.anthem.load(Ordering::Relaxed) == UNKNOWN
                || (self.anthem.load(Ordering::Relaxed) == PRESENT)
                    == self.cards.iter().any(card_has_anthem),
            "graveyard anthem memo is stale: a write reached the cards without clearing it",
        );
        match self.anthem.load(Ordering::Relaxed) {
            ABSENT => false,
            PRESENT => true,
            _ => {
                let found = self.cards.iter().any(card_has_anthem);
                self.anthem.store(if found { PRESENT } else { ABSENT }, Ordering::Relaxed);
                found
            }
        }
    }

    /// Append, through [`CowBox`]'s own `push` so an unshare materializes
    /// with room for the card. Inherent, so it shadows the `Deref`'d
    /// `Vec::push`; the memo is invalidated exactly as `DerefMut` would.
    pub fn push(&mut self, card: CardInstance) {
        self.anthem.store(UNKNOWN, Ordering::Relaxed);
        self.cards.push(card);
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

/// Packed two-bit memo lanes for [`Battlefield`], so one store on the write
/// path clears every lane. Each lane holds the same `UNKNOWN` / `ABSENT` /
/// `PRESENT` triple the graveyard's memo uses.
const LANE_LAND: u32 = 0;
const LANE_CREATURE: u32 = 2;
const LANE_MASK: u8 = 0b11;

/// The battlefield: the CoW card list, plus the two whole-board questions the
/// layer-4 type gates ask of it — "can any permanent here contribute an
/// `AddLandType` / `SetLandTypes` / `ReplaceBasicLandType` to the gathered
/// set", and the same for the creature-type family.
///
/// `land_type_change_in_scope` and `creature_type_change_in_scope` are
/// 0.60 / 1.13 / 0.52 % of the three pools and each miss is a 262-642 Ir walk
/// over every permanent (PERF `(-87)`). The freeze-scope `PresenceGate`
/// memoizes them for the lifetime of a scope, but roughly half the misses
/// arrive holding `&mut self` — `activate_ability_inner`'s CR 602.5 tap gate,
/// the bot's `available_mana` — and a scope that borrows `&self` can never
/// serve those. `CRAB_GATE_CENSUS` (pass 97) measured the gates' inputs
/// unchanged on 93.95 % of `cube`'s 242,788 asks, so the answer wants to
/// outlive the scope, and the zone is where it can.
///
/// The memo is sound *by construction*, like [`Graveyard`]'s: [`DerefMut`]
/// and the `&mut` [`IntoIterator`] are the only ways to reach the cards
/// mutably, and both clear it. That is broader than the gates need — a tap,
/// a damage mark or a counter cannot move either answer, which only reads
/// `attached_to` and the definition — but an enumeration of the write sites
/// that *can* move it is exactly what this file's header refuses to depend
/// on. `(-87)` prices the gap: the exact invalidation would repeat on
/// 93.95 / 75.76 / 74.30 % of asks and this one on 84.32 / 35.95 / 29.42 %.
#[derive(Debug, Default)]
pub struct Battlefield {
    cards: CowBox<Vec<CardInstance>>,
    /// `LANE_LAND` / `LANE_CREATURE` two-bit lanes; see the constants.
    type_gates: AtomicU8,
}

impl Battlefield {
    /// True when some permanent here satisfies `walk` — the layer-4 land-type
    /// contributor test. Memoized; a miss walks the board once.
    #[inline]
    pub fn has_land_type_changer(
        &self,
        walk: impl Fn(&CardInstance) -> bool + Copy,
    ) -> bool {
        self.lane(LANE_LAND, walk)
    }

    /// The creature-type twin of
    /// [`has_land_type_changer`](Self::has_land_type_changer).
    #[inline]
    pub fn has_creature_type_changer(
        &self,
        walk: impl Fn(&CardInstance) -> bool + Copy,
    ) -> bool {
        self.lane(LANE_CREATURE, walk)
    }

    /// One lane's answer: a word load and two mask tests on a hit, the board
    /// walk plus one store on a miss.
    ///
    /// **`walk` is `Copy` and passed to `any` by value, never by reference.**
    /// `&F: FnMut` routes every element through
    /// `core::ops::function::impls::call_mut`, which does not inline: the
    /// first build of this memo took the gates' 33.7 M Ir down to 5.9 M and
    /// handed **18.8 M** of it straight back through that shim (`cube`,
    /// ninety-eighth pass). A fn item passed by value monomorphizes into the
    /// loop exactly as `battlefield.iter().any(card_can_change_land_types)`
    /// did before the memo existed.
    ///
    /// The `debug_assert!` is the audit that no write path reached the cards
    /// without clearing the word: it costs nothing in any release profile, and
    /// `scripts/robustness_grid.sh` is what fires it over boards the suite
    /// never deals.
    #[inline]
    fn lane(&self, shift: u32, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        let cur = (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK;
        debug_assert!(
            cur == UNKNOWN || (cur == PRESENT) == self.cards.iter().any(walk),
            "battlefield type-gate memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            ABSENT => false,
            PRESENT => true,
            _ => self.walk_and_store(shift, walk),
        }
    }

    /// The miss path, out of line so the hit path stays a word load at each of
    /// the two call sites.
    ///
    /// **Not `#[cold]`.** It is taken on roughly half the asks, and marking it
    /// cold makes LLVM optimize it for size — which stops the `walk` fn item
    /// inlining into the loop and turns every card visit into a call. That
    /// cost 9.2 M Ir over 920,622 visits on `cube` and ate two thirds of the
    /// memo's win (ninety-eighth pass). `#[inline(never)]` instead: one
    /// instance per predicate, with the predicate inlined into its loop, and
    /// the two call sites keep a word load for the hit.
    #[inline(never)]
    fn walk_and_store(&self, shift: u32, walk: impl Fn(&CardInstance) -> bool) -> bool {
        let found = self.cards.iter().any(walk);
        let lane = if found { PRESENT } else { ABSENT };
        let word = self.type_gates.load(Ordering::Relaxed);
        self.type_gates
            .store((word & !(LANE_MASK << shift)) | (lane << shift), Ordering::Relaxed);
        found
    }

    /// Append, through [`CowBox`]'s own `push` so an unshare materializes with
    /// room for the card (PERF `(-76)`). Inherent, so it shadows the `Deref`'d
    /// `Vec::push`; the memo is invalidated exactly as `DerefMut` would.
    pub fn push(&mut self, card: CardInstance) {
        self.type_gates.store(0, Ordering::Relaxed);
        self.cards.push(card);
    }

    /// True when both handles still share one allocation — the [`CowBox`]
    /// contract, forwarded for the zone tests.
    pub fn shares_with(&self, other: &Self) -> bool {
        self.cards.shares_with(&other.cards)
    }

    /// One lane's current state, for the invalidation tests: `None` while
    /// unknown.
    #[cfg(test)]
    fn memo(&self, shift: u32) -> Option<bool> {
        match (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK {
            ABSENT => Some(false),
            PRESENT => Some(true),
            _ => None,
        }
    }
}

impl Clone for Battlefield {
    /// The cards clone as a `CowBox` (a refcount bump); the memo describes
    /// those same cards, so it comes along.
    fn clone(&self) -> Self {
        Self {
            cards: self.cards.clone(),
            type_gates: AtomicU8::new(self.type_gates.load(Ordering::Relaxed)),
        }
    }
}

impl Deref for Battlefield {
    type Target = Vec<CardInstance>;
    fn deref(&self) -> &Vec<CardInstance> {
        &self.cards
    }
}

impl DerefMut for Battlefield {
    fn deref_mut(&mut self) -> &mut Vec<CardInstance> {
        self.type_gates.store(0, Ordering::Relaxed);
        &mut self.cards
    }
}

impl From<Vec<CardInstance>> for Battlefield {
    fn from(cards: Vec<CardInstance>) -> Self {
        Self { cards: cards.into(), type_gates: AtomicU8::new(0) }
    }
}

impl From<CowBox<Vec<CardInstance>>> for Battlefield {
    fn from(cards: CowBox<Vec<CardInstance>>) -> Self {
        Self { cards, type_gates: AtomicU8::new(0) }
    }
}

// `for c in &bf` / `for c in &mut bf` — as on `Graveyard`, deref coercion
// doesn't apply to `for` loops. The `&mut` arm is the second invalidation
// point.
impl<'a> IntoIterator for &'a Battlefield {
    type Item = &'a CardInstance;
    type IntoIter = std::slice::Iter<'a, CardInstance>;
    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter()
    }
}

impl<'a> IntoIterator for &'a mut Battlefield {
    type Item = &'a mut CardInstance;
    type IntoIter = std::slice::IterMut<'a, CardInstance>;
    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().iter_mut()
    }
}

impl serde::Serialize for Battlefield {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.cards.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Battlefield {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        CowBox::<Vec<CardInstance>>::deserialize(deserializer).map(Battlefield::from)
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

    fn bf(defs: Vec<crate::card::CardDefinition>) -> Battlefield {
        defs.into_iter()
            .enumerate()
            .map(|(i, d)| CardInstance::new(CardId(i as u32), d, 0))
            .collect::<Vec<_>>()
            .into()
    }

    /// Blood Moon sets every nonbasic land's type line; Grizzly Bears does
    /// nothing of the kind. Both lanes answer independently and memoize.
    #[test]
    fn type_gate_lanes_are_detected_and_memoized() {
        let land = |c: &CardInstance| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, crate::effect::StaticEffect::LandTypeChanger { .. })
            })
        };
        let never = |_: &CardInstance| false;

        let b = bf(vec![crate::catalog::blood_moon(), crate::catalog::grizzly_bears()]);
        assert_eq!(b.memo(LANE_LAND), None, "lazy until asked");
        assert!(b.has_land_type_changer(land));
        assert_eq!(b.memo(LANE_LAND), Some(true));
        assert_eq!(b.memo(LANE_CREATURE), None, "the other lane is untouched");
        assert!(!b.has_creature_type_changer(never));
        assert_eq!(b.memo(LANE_CREATURE), Some(false));
        assert_eq!(b.memo(LANE_LAND), Some(true), "packing did not disturb the first lane");
    }

    /// The whole point: a write invalidates both lanes, a read invalidates
    /// neither.
    #[test]
    fn battlefield_writes_invalidate_and_reads_do_not() {
        let never = |_: &CardInstance| false;
        let mut b = bf(vec![crate::catalog::grizzly_bears()]);
        assert!(!b.has_land_type_changer(never));
        assert!(!b.has_creature_type_changer(never));
        assert_eq!(b.memo(LANE_LAND), Some(false));
        assert_eq!(b.memo(LANE_CREATURE), Some(false));

        assert_eq!(b.iter().count(), 1);
        assert_eq!((&b).into_iter().count(), 1);
        assert_eq!(b.memo(LANE_LAND), Some(false), "a read does not invalidate");

        b.push(CardInstance::new(CardId(99), crate::catalog::blood_moon(), 0));
        assert_eq!(b.memo(LANE_LAND), None, "push invalidated");
        assert_eq!(b.memo(LANE_CREATURE), None, "both lanes, one store");

        assert!(!b.has_land_type_changer(never));
        for _ in &mut b {}
        assert_eq!(b.memo(LANE_LAND), None, "&mut iteration invalidated");

        assert!(!b.has_land_type_changer(never));
        b.pop();
        assert_eq!(b.memo(LANE_LAND), None, "DerefMut invalidated");
    }

    /// The CoW contract survives the wrapper, and the memo travels with the
    /// snapshot it describes.
    #[test]
    fn battlefield_clone_shares_and_carries_the_memo() {
        let never = |_: &CardInstance| false;
        let mut a = bf(vec![crate::catalog::grizzly_bears()]);
        assert!(!a.has_land_type_changer(never));
        let b = a.clone();
        assert!(a.shares_with(&b), "clone is a refcount bump");
        assert_eq!(b.memo(LANE_LAND), Some(false), "the memo describes the same cards");
        a.pop();
        assert!(!a.shares_with(&b), "the write unshared");
        assert_eq!(b.memo(LANE_LAND), Some(false), "the snapshot's answer is still right");
        assert_eq!(a.memo(LANE_LAND), None);
    }

    #[test]
    fn battlefield_serde_round_trips_as_a_bare_list() {
        let b = bf(vec![crate::catalog::grizzly_bears()]);
        let json = serde_json::to_string(&b).unwrap();
        assert!(json.starts_with('['), "serializes as the card list: {json}");
        let back: Battlefield = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back.memo(LANE_LAND), None, "a fresh deserialize starts unknown");
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
