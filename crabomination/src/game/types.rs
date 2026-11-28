use serde::{Deserialize, Serialize};

use crate::card::{CardId, CardInstance, CounterType};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::mana::{Color, ManaError};

/// An insertion-ordered set of ids, backed by a `Vec`.
///
/// Not a `HashSet`, for the same two reasons as
/// [`CounterBag`](crate::card::CounterBag). Clone cost: `ColdState` carries
/// ~20 of these behind one `CowBox`, and an empty `hashbrown` clone still
/// walks its control bytes where an empty `Vec` clone allocates nothing
/// (this removed `ColdState`'s share of `RawTable::clone` under the CoW
/// unshare). Order: `RandomState` reseeds a `HashSet`'s iteration order per
/// process, which would leak into game state.
///
/// A handful of ids each, so linear scans beat hashing. `insert` keeps set
/// semantics and returns novelty like `HashSet::insert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdSet<T>(Vec<T>);

impl<T> Default for IdSet<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: PartialEq> IdSet<T> {
    pub fn insert(&mut self, v: T) -> bool {
        if self.0.contains(&v) {
            return false;
        }
        self.0.push(v);
        true
    }
    pub fn contains(&self, v: &T) -> bool {
        self.0.contains(v)
    }
    pub fn remove(&mut self, v: &T) -> bool {
        match self.0.iter().position(|x| x == v) {
            Some(i) => {
                self.0.remove(i);
                true
            }
            None => false,
        }
    }
    pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        self.0.retain(f);
    }
}

impl<T> IdSet<T> {
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T: PartialEq> PartialEq for IdSet<T> {
    /// Set equality: order-insensitive, so a value round-tripped through a
    /// snapshot still compares equal however it was rebuilt.
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().all(|v| other.0.contains(v))
    }
}

impl<T: Eq> Eq for IdSet<T> {}

impl<'a, T> IntoIterator for &'a IdSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: PartialEq> FromIterator<T> for IdSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self {
        let mut out = Self::default();
        for v in it {
            out.insert(v);
        }
        out
    }
}

impl<T: PartialEq> Extend<T> for IdSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, it: I) {
        for v in it {
            self.insert(v);
        }
    }
}

/// [`IdSet`] with an inline buffer, for a local that never leaves its frame.
///
/// Stored `IdSet`/`IdMap` fields keep the `Vec` backing on purpose: inside
/// `ColdState` and `PlayerData` the group's size class is the constraint
/// (PERF's 1,016-byte `PlayerData` ceiling). A local instead pays one heap
/// allocation on first insert and never re-grows, so an inline buffer
/// removes it where `with_capacity` would only move it. Four slots covered
/// every measured case; wider spills to the heap. PERF `(-158)`.
#[derive(Debug, Clone)]
pub struct SmallIdSet<T>(smallvec::SmallVec<[T; 4]>);

impl<T> Default for SmallIdSet<T> {
    fn default() -> Self {
        Self(smallvec::SmallVec::new())
    }
}

impl<T: PartialEq> SmallIdSet<T> {
    pub fn insert(&mut self, v: T) -> bool {
        if self.0.contains(&v) {
            return false;
        }
        self.0.push(v);
        true
    }
    pub fn contains(&self, v: &T) -> bool {
        self.0.contains(v)
    }
}

impl<T> SmallIdSet<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Clippy's `len_without_is_empty`: a `len` with no `is_empty` beside it
    /// is the shape that makes a caller write `x.len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T: PartialEq> FromIterator<T> for SmallIdSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self {
        let mut out = Self::default();
        for v in it {
            out.insert(v);
        }
        out
    }
}

impl<T: PartialEq> Extend<T> for SmallIdSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, it: I) {
        for v in it {
            self.insert(v);
        }
    }
}

impl<'a, T> IntoIterator for &'a SmallIdSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// [`IdMap`] with an inline buffer. Locals only, like [`SmallIdSet`]; the
/// bodies are [`IdMap`]'s verbatim, checked by
/// `small_id_containers_agree_with_the_heap_backed_ones`. The surface is only
/// what current callers and that test use — add a method beside its `IdMap`
/// twin when needed.
#[derive(Debug, Clone)]
pub struct SmallIdMap<K, V>(smallvec::SmallVec<[(K, V); 4]>);

impl<K, V> Default for SmallIdMap<K, V> {
    fn default() -> Self {
        Self(smallvec::SmallVec::new())
    }
}

impl<K: PartialEq, V> SmallIdMap<K, V> {
    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.iter().find(|(x, _)| x == k).map(|(_, v)| v)
    }
    pub fn contains_key(&self, k: &K) -> bool {
        self.0.iter().any(|(x, _)| x == k)
    }
}

impl<K, V> SmallIdMap<K, V> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }
}

impl<K: PartialEq, V: Default> SmallIdMap<K, V> {
    /// `entry(k).or_default()` without the `Entry` machinery.
    pub fn entry_or_default(&mut self, k: K) -> &mut V {
        if let Some(i) = self.0.iter().position(|(x, _)| *x == k) {
            return &mut self.0[i].1;
        }
        self.0.push((k, V::default()));
        &mut self.0.last_mut().expect("just pushed").1
    }
}

impl<K, V> IntoIterator for SmallIdMap<K, V> {
    type Item = (K, V);
    type IntoIter = smallvec::IntoIter<[(K, V); 4]>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An insertion-ordered map from ids to values, backed by a `Vec`.
///
/// The map sibling of [`IdSet`], here for determinism of order:
/// `dispatch_triggers_for_events` walks `died_card_snapshots` and each
/// candidate's position decides its stack slot, so a `RandomState`-ordered
/// `HashMap` could stack LKI triggers differently per process. Insertion
/// order is die order, as CR 603.3b wants. Holds a handful of entries
/// (cleared every dispatch batch), so linear scans beat hashing.
#[derive(Debug, Clone)]
pub struct IdMap<K, V>(Vec<(K, V)>);

impl<K, V> Default for IdMap<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K: PartialEq, V> IdMap<K, V> {
    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.iter().find(|(x, _)| x == k).map(|(_, v)| v)
    }
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.0.iter_mut().find(|(x, _)| x == k).map(|(_, v)| v)
    }
    pub fn contains_key(&self, k: &K) -> bool {
        self.0.iter().any(|(x, _)| x == k)
    }
    /// Set `k`'s value, returning the previous one — `HashMap::insert`'s
    /// contract. An existing key keeps its position.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self.0.iter_mut().find(|(x, _)| *x == k) {
            Some((_, have)) => Some(std::mem::replace(have, v)),
            None => {
                self.0.push((k, v));
                None
            }
        }
    }
    pub fn remove(&mut self, k: &K) -> Option<V> {
        let i = self.0.iter().position(|(x, _)| x == k)?;
        Some(self.0.remove(i).1)
    }
}

impl<K, V> IdMap<K, V> {
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// The backing `Vec`'s capacity — the "is this still holding a buffer?"
    /// guard the two per-seat discard maps use before dropping themselves.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }
    /// `HashMap::retain`'s contract, in insertion order.
    pub fn retain(&mut self, mut f: impl FnMut(&K, &mut V) -> bool) {
        self.0.retain_mut(|(k, v)| f(k, v));
    }
}

impl<K: PartialEq, V: Default> IdMap<K, V> {
    /// `entry(k).or_default()` without the `Entry` machinery.
    pub fn entry_or_default(&mut self, k: K) -> &mut V {
        if let Some(i) = self.0.iter().position(|(x, _)| *x == k) {
            return &mut self.0[i].1;
        }
        self.0.push((k, V::default()));
        &mut self.0.last_mut().expect("just pushed").1
    }
}

impl<K, V> IntoIterator for IdMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K: PartialEq, V> FromIterator<(K, V)> for IdMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(it: I) -> Self {
        let mut out = Self::default();
        for (k, v) in it {
            out.insert(k, v);
        }
        out
    }
}

/// Serialized as a **map**, so swapping a `HashMap` field for an `IdMap` one
/// leaves the wire shape alone (`IdSet` does the same for a sequence).
impl<K: serde::Serialize, V: serde::Serialize> serde::Serialize for IdMap<K, V> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = ser.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            m.serialize_entry(k, v)?;
        }
        m.end()
    }
}

impl<'de, K: serde::Deserialize<'de>, V: serde::Deserialize<'de>> serde::Deserialize<'de>
    for IdMap<K, V>
{
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct Vis<K, V>(std::marker::PhantomData<(K, V)>);
        impl<'de, K: serde::Deserialize<'de>, V: serde::Deserialize<'de>> serde::de::Visitor<'de>
            for Vis<K, V>
        {
            type Value = IdMap<K, V>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a map")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut a: A,
            ) -> Result<Self::Value, A::Error> {
                let mut out = Vec::with_capacity(a.size_hint().unwrap_or(0));
                while let Some((k, v)) = a.next_entry()? {
                    out.push((k, v));
                }
                Ok(IdMap(out))
            }
        }
        de.deserialize_map(Vis(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod id_set_tests {
    use super::{IdMap, IdSet, SmallIdMap, SmallIdSet};

    /// The inline types copy the heap types' bodies, so only a typo can split
    /// them: run one op sequence crossing the four-slot buffer through both.
    /// PERF `(-158)`.
    #[test]
    fn small_id_containers_agree_with_the_heap_backed_ones() {
        let ops = [5u32, 1, 5, 9, 2, 7, 7, 3, 1, 11, 0];
        let mut heap: IdSet<u32> = IdSet::default();
        let mut small: SmallIdSet<u32> = SmallIdSet::default();
        for v in ops {
            assert_eq!(heap.insert(v), small.insert(v), "novelty of {v}");
            assert_eq!(heap.len(), small.len());
            assert_eq!(heap.contains(&v), small.contains(&v));
        }
        assert!(heap.iter().eq(small.iter()), "insertion order");

        let mut hm: IdMap<u32, u32> = IdMap::default();
        let mut sm: SmallIdMap<u32, u32> = SmallIdMap::default();
        for v in ops {
            *hm.entry_or_default(v) += v;
            *sm.entry_or_default(v) += v;
            assert_eq!(hm.get(&v), sm.get(&v));
            assert_eq!(hm.contains_key(&v), sm.contains_key(&v));
            assert_eq!(hm.len(), sm.len());
        }
        assert!(hm.keys().eq(sm.keys()), "insertion order");
        assert!(hm.iter().eq(sm.iter()));
        assert_eq!(hm.is_empty(), sm.is_empty());

        let from_iter: SmallIdSet<u32> = ops.into_iter().collect();
        let heap_from_iter: IdSet<u32> = ops.into_iter().collect();
        assert!(heap_from_iter.iter().eq(from_iter.iter()));
        let mut ext = from_iter;
        ext.extend([4u32, 5, 12]);
        assert_eq!(ext.len(), 10, "extend keeps set semantics past the inline buffer");
    }

    /// The three properties the field swap relies on: `insert` keeps set
    /// semantics and reports novelty like `HashSet::insert`, iteration is
    /// insertion order (never `RandomState`'s), and equality ignores order
    /// so a rebuilt value still compares equal.
    #[test]
    fn set_semantics_insertion_order_and_unordered_eq() {
        let mut s: IdSet<u32> = IdSet::default();
        assert!(s.insert(7));
        assert!(!s.insert(7), "duplicate insert reports not-new");
        s.insert(3);
        assert_eq!(s.len(), 2);
        assert_eq!(s.iter().copied().collect::<Vec<_>>(), vec![7, 3], "insertion order");
        assert!(s.contains(&3) && !s.contains(&9));
        assert!(s.remove(&7) && !s.remove(&7));
        assert_eq!(s.iter().copied().collect::<Vec<_>>(), vec![3]);

        let a: IdSet<u32> = [1, 2].into_iter().collect();
        let b: IdSet<u32> = [2, 1].into_iter().collect();
        assert_eq!(a, b, "equality is order-insensitive");
    }

    /// The trigger dispatcher relies on this: iteration is die order and
    /// re-writing a key keeps its position (a `HashMap` stacked LKI triggers
    /// in per-process `RandomState` order).
    #[test]
    fn id_map_iterates_in_insertion_order() {
        let mut m: super::IdMap<u32, &str> = super::IdMap::default();
        m.insert(9, "a");
        m.insert(2, "b");
        assert_eq!(m.insert(9, "a2"), Some("a"), "re-insert returns the old value");
        assert_eq!(m.values().copied().collect::<Vec<_>>(), vec!["a2", "b"], "position kept");
        assert_eq!(m.get(&2), Some(&"b"));
        assert_eq!(m.remove(&9), Some("a2"));
        assert!(!m.contains_key(&9) && m.len() == 1);
    }

    /// `IdMap` keeps the `HashMap` wire shape — a map — so a field swapped
    /// from one to the other round-trips existing snapshots. `block_map` is
    /// the swap that needed it (PERF `(-146)`).
    #[test]
    fn id_map_serializes_as_a_map() {
        let mut m: super::IdMap<u32, Vec<u32>> = super::IdMap::default();
        m.insert(9, vec![1]);
        m.insert(2, vec![]);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, r#"{"9":[1],"2":[]}"#);
        let back: super::IdMap<u32, Vec<u32>> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.keys().copied().collect::<Vec<_>>(), vec![9, 2], "order survives");
        assert_eq!(back.get(&9), Some(&vec![1]));
    }

    /// `entry_or_default` and `retain`, the two `HashMap` methods the
    /// `block_map` swap needed.
    #[test]
    fn id_map_entry_or_default_and_retain() {
        let mut m: super::IdMap<u32, Vec<u32>> = super::IdMap::default();
        m.entry_or_default(5).push(1);
        m.entry_or_default(5).push(2);
        m.entry_or_default(6).push(3);
        assert_eq!(m.get(&5), Some(&vec![1, 2]), "second call reuses the slot");
        m.retain(|_, v| {
            v.retain(|x| *x != 1);
            !v.is_empty()
        });
        assert_eq!(m.keys().copied().collect::<Vec<_>>(), vec![5, 6]);
        m.retain(|k, _| *k != 5);
        assert_eq!(m.keys().copied().collect::<Vec<_>>(), vec![6]);
    }

    /// Same wire shape as the `HashSet` it replaces — a sequence — so
    /// existing snapshots round-trip.
    #[test]
    fn serializes_as_a_sequence() {
        let s: IdSet<u32> = [5, 6].into_iter().collect();
        assert_eq!(serde_json::to_string(&s).unwrap(), "[5,6]");
        let back: IdSet<u32> = serde_json::from_str("[5,6]").unwrap();
        assert_eq!(back, s);
    }
}

/// serde default for `ImpulsePending.take` (back-compat: a snapshot without
/// the field takes one card).
pub fn one() -> usize {
    1
}

/// serde default for `SearchPending.include_library` (back-compat: a snapshot
/// without the field was a plain library search).
pub fn default_true() -> bool {
    true
}

// ── Turn step sequence ────────────────────────────────────────────────────────

// `TurnStep` lives in `crabomination_base` (below the card catalog) so cards
// keying off steps don't pull in game logic; re-exported for existing paths.
pub use crabomination_base::TurnStep;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Player(usize),
    Permanent(CardId),
}

/// What an attacking creature is attacking. In multiplayer each attacker
/// chooses one of the defending players or a planeswalker controlled by one
/// of them; in 2-player games this is always `Player(opponent)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackTarget {
    Player(usize),
    Planeswalker(CardId),
    /// CR 508.4 — a Battle controlled by a player other than its protector.
    /// In 2-player games this is the active player's own Siege (defended by the
    /// opponent it chose as protector).
    Battle(CardId),
}

/// One attacker's declared assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attack {
    pub attacker: CardId,
    pub target: AttackTarget,
}

// `PartialEq` is load-bearing: the bot's combat-instant enumerator dedupes
// candidates with it instead of by `format!("{a:?}")`. PERF `(-290)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameAction {
    PlayLand(CardId),
    /// Play an MDFC land by its back face: `definition` is swapped to the back
    /// face before it enters, so all its abilities come from that face.
    PlayLandBack(CardId),
    /// CR 305 — play a land from the controller's graveyard, legal only while
    /// a `StaticEffect::MayPlayLandsFromGraveyard` permanent (Crucible of
    /// Worlds, Ramunap Excavator) is in play.
    PlayLandFromGraveyard(CardId),
    /// CR 702.139 — Companion: pay {3} at sorcery speed to move the
    /// designated companion from the sideboard ("outside the game") into
    /// its owner's hand. Once per game (the card leaves the sideboard).
    CompanionToHand(CardId),
    CastSpell { card_id: CardId, target: Option<Target>, additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// CR 702.33b — cast a spell paying any subset of its `kicker_options`
    /// ("Kicker {A} and/or {B}" — the Volver cycle). Each paid index is
    /// stamped on the spell for `Predicate::SpellWasKickedWith`.
    CastSpellKickers {
        card_id: CardId,
        kickers: Vec<u8>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.32 — cast a spell paying its optional Kicker cost; the spell is
    /// stamped `kicked` for `Predicate::SpellWasKicked` (Tear Asunder).
    CastSpellKicked {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.103 — cast an enchantment-creature card for its Bestow cost
    /// as an Aura spell targeting a creature. It enters attached and is not
    /// a creature while bestowed. `target` must be a creature permanent.
    CastBestow {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.27 — cast a spell paying its optional Buyback cost in
    /// addition to the mana cost. The resolving spell returns to its
    /// owner's hand instead of the graveyard.
    CastSpellBuyback {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.41 — Cast a modal spell paying its optional Entwine cost;
    /// every mode runs in order at resolution.
    CastSpellEntwine {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 709.5 — Cast a Room card's chosen door (left or right) for that
    /// door's cost; the permanent enters with that door unlocked.
    CastRoomDoor {
        card_id: CardId,
        right: bool,
    },
    /// CR 709.5e / 116.2m — special action: pay a locked door's cost at
    /// sorcery speed to unlock it.
    UnlockRoomDoor {
        card_id: CardId,
        right: bool,
    },
    /// CR 702.62 — Suspend a card from hand: pay its suspend cost and exile
    /// it with N time counters. Removed one per owner's upkeep; the spell is
    /// cast for free when the last counter comes off.
    Suspend { card_id: CardId },
    /// CR 702.62a — Suspend X: suspend paying `x_value` for the cost's {X}
    /// and putting that many time counters on it ("X can't be 0").
    SuspendX { card_id: CardId, x_value: u32 },
    /// CR 702.143 — Foretell a card from hand: pay {2} and exile it
    /// face-down to cast later for its foretell cost.
    Foretell { card_id: CardId },
    /// CR 702.36b — cast a card with Morph face down as a 2/2 creature for {3}.
    CastFaceDown { card_id: CardId },
    /// CR 708.5 — turn a face-down permanent face up (special action, any time
    /// you have priority): pay its Morph cost, or — for a manifested creature
    /// card — its mana cost. Fires `EventKind::TurnedFaceUp`.
    TurnFaceUp { card_id: CardId },
    /// CR 702.36b — turn a face-down permanent up paying an `{X}` in its morph
    /// cost (Warbreak Trumpeter's Morph {X}{X}{R}). `TurnFaceUp` is this with
    /// X = 0; the turn-up trigger reads the paid X via `Value::XFromCost`.
    TurnFaceUpForX { card_id: CardId, x_value: u32 },
    /// CR 702.143c — cast a previously foretold card from exile for its
    /// foretell cost.
    CastForetold {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 716 — cast the instant/sorcery "adventure" half of an Adventurer
    /// card from hand. On resolution the card is exiled with permission to
    /// cast the creature half later.
    CastAdventure {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.183 — cast a card's Omen half from hand for its Omen cost. On
    /// resolution or counter the card is shuffled into its owner's library.
    CastOmen {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.165 — cast a Gift spell, promising the gift to an opponent: the
    /// spell resolves its enhanced `gifted_effect`.
    CastGift {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 716 — cast the creature half of a card that's in exile after going
    /// on an adventure (its `on_adventure` flag is set).
    CastAdventureCreature {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.160 — cast a Prototype artifact creature for its prototype
    /// cost. It enters with the prototype's mana cost, color, and size,
    /// keeping its abilities and types.
    CastPrototype {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.140 — cast a creature with Mutate for its mutate cost, merging
    /// it onto `target` (a non-Human creature you own). `on_top` controls
    /// whether the new card's characteristics sit on top of the merged pile.
    CastMutate {
        card_id: CardId,
        target: CardId,
        on_top: bool,
        x_value: Option<u32>,
    },
    /// CR 709 — cast the **right** half of a split card from hand, paying the
    /// right half's cost. Resolves the right half's effect and goes to the
    /// graveyard like any spell.
    CastSplitRight {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.102 — Fuse: cast both halves of a split card as one spell,
    /// paying both costs. `target` is the left half's target; the right
    /// half's target is `additional_targets` slot 0.
    CastSplitFused {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.127 — cast the **Aftermath** (right) half of a split card from
    /// the graveyard, paying the right half's cost. Exiled on resolution.
    CastAftermath {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.153 — cast a spell paying its optional Casualty cost,
    /// sacrificing `sacrifice` (a creature you control with power ≥ the
    /// casualty number) as an additional cost. On cast the spell is copied.
    CastSpellCasualty {
        card_id: CardId,
        sacrifice: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.176 — cast a spell paying its optional Bargain cost. If
    /// `sacrifice` is `Some`, that artifact/enchantment/token you control is
    /// sacrificed as an additional cost and the spell is stamped "bargained"
    /// (`Predicate::SpellWasBargained`).
    CastSpellBargain {
        card_id: CardId,
        sacrifice: Option<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.172 — cast a Spree spell choosing `spree_modes` (one or more
    /// distinct mode indices). Each chosen mode's mana cost is folded into the
    /// total; `target` (slot 0) and `additional_targets` supply per-mode
    /// targets in the order the chosen modes appear.
    CastSpellSpree {
        card_id: CardId,
        spree_modes: Vec<u8>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        x_value: Option<u32>,
    },
    /// CR 702.47 — cast a spell splicing the given hand cards onto it. Each
    /// splice card (revealed, stays in hand) must carry `Keyword::Splice`
    /// whose quality matches the cast spell's subtypes; its splice cost is
    /// paid additionally and its rules text resolves after the main effect.
    /// Spliced effect `i` reads its target from `additional_targets[i]`.
    CastSpellSpliced {
        card_id: CardId,
        splice_cards: Vec<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 601.2b — cast a spell paying its optional "sacrifice any number of
    /// creatures, {N} less each" additional cost. Each id in `sacrifices`
    /// (a creature you control) is sacrificed as an additional cost and the
    /// generic cost drops by `sacrifice_cost_reduction` per creature.
    CastSpellSacrificeReduce {
        card_id: CardId,
        sacrifices: Vec<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.157 — cast a creature spell paying its optional Squad cost
    /// `times` times. The squad cost is charged that many extra times and the
    /// resolving spell is stamped `squad_count = times`, minting that many
    /// token copies on ETB.
    CastSpellSquad {
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.33c — cast a spell paying its Multikicker cost `times` times.
    /// The kicker cost is charged that many extra times; the resolving spell
    /// is stamped `kicked` + `kick_count = times` (`Value::TimesKicked`).
    CastSpellMultikicked {
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.107 — cast an instant/sorcery paying its optional Replicate cost
    /// `times` times. The replicate cost is charged that many extra times and
    /// the spell is copied that many times (copies may choose new targets).
    CastSpellReplicate {
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.78 — cast a spell paying its optional Conspire cost: tap two
    /// untapped creatures you control (summoning sickness irrelevant) that
    /// each share a color with it, copying the spell once (new targets
    /// allowed).
    CastSpellConspire {
        card_id: CardId,
        conspire_creatures: [CardId; 2],
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.170 — Plot a card from hand: pay its plot cost and exile it
    /// face-up. Special action, main phase + empty stack only.
    Plot { card_id: CardId },
    /// CR 702.170d — cast a plotted card from exile without paying its mana
    /// cost, on a turn after it was plotted.
    CastPlotted {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast an MDFC via its non-land back face — `PlayLandBack` for spells.
    /// `definition` is swapped before payment, so cost, type and effect all
    /// come from the back face (STX Deans).
    CastSpellBack {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// SOS Prepare — cast a copy of a prepared creature's inset spell. Legal
    /// while `creature_id` is on the battlefield under the actor's control
    /// with a `CounterType::Prepared` counter, which casting removes. Paid
    /// and timed like a normal spell of its type; the copy ceases to exist
    /// when it leaves the stack.
    CastPrepareSpell {
        creature_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a spell with `Keyword::Convoke`, tapping each creature in
    /// `convoke_creatures` toward the cost (CR 702.51): each pays one mana of
    /// a color it is where the cost still wants that color, otherwise {1}.
    /// Each must be an untapped creature controlled by the caster. Improvise
    /// (CR 702.126) and waterbend helpers ride the same slot as artifacts.
    CastSpellConvoke {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: Vec<CardId>,
    },
    /// CR 701.67 — cast a spell paying its "waterbend {N}" additional cost:
    /// each artifact/creature in `helpers` taps for {1} of it (clamped to N,
    /// the rest from mana). For an optional rider this is the pay branch and
    /// plain `CastSpell` the decline. For `waterbend {X}`, `x_value` is X.
    CastSpellWaterbend {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        helpers: Vec<CardId>,
    },
    /// Cast a spell with `Keyword::Delve` (CR 702.66): each card in
    /// `delve_cards` (from the caster's graveyard) is exiled to pay {1} of the
    /// generic cost. Colored and {X} pips still come from mana.
    CastSpellDelve {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        delve_cards: Vec<CardId>,
    },
    /// Convoke and delve in one cast (CR 702.51 + 702.66 — neither is an
    /// alternative cost, so they combine): Hogaak, Arisen Necropolis, whose
    /// mana can't pay. `CastSpellConvoke`, `CastSpellDelve` and this route by
    /// the card's zone — hand, command zone (CR 903.8) or a graveyard cast
    /// permission.
    CastSpellConvokeDelve {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: Vec<CardId>,
        delve_cards: Vec<CardId>,
    },
    /// Cast a spell paying its `alternative_cost` instead of its regular
    /// mana cost. `pitch_card` is the hand card (e.g., a blue card for Force
    /// of Will/Negation) being exiled to satisfy the alt cost — `None` when
    /// the alt cost has no exile requirement.
    CastSpellAlternative {
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 118.9a — cast a spell from hand for the alternative cost a
    /// permanent grants it (Henzie's blitz, Kentaro's {X}) rather than its own
    /// printed one (evoke, dash): only one alternative cost applies.
    CastSpellGrantedAlternative {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    ActivateAbility {
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        /// Extra chosen targets (slots 1+) for two-target activated abilities
        /// (Autumn-Tail, Kitsune Sage). Empty for the single-target majority.
        #[serde(default)]
        additional_targets: Vec<Target>,
        /// X paid to an `{X}` in the activation cost, read by the body as
        /// `Value::XFromCost` (Pernicious Deed, Walking Ballista).
        #[serde(default)]
        x_value: Option<u32>,
        /// CR 601.2b — the mode for a modal activated ability
        /// (`Effect::ChooseMode` atop its body). `None` lets the engine ask
        /// (UI seat: resolution-time modal; bot: its decider). A submitted
        /// mode is authoritative and drives per-slot target validation — the
        /// only way a UI seat can pick a mode whose body targets.
        #[serde(default)]
        mode: Option<usize>,
    },
    /// Activate an ability flagged `ActivatedAbility.waterbend` (CR 701.67,
    /// artifacts or creatures) or `.convoke` (CR 702.51, creatures —
    /// Heirloom Epic): each helper taps for {1} of the generic cost.
    ActivateAbilityWaterbend {
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        #[serde(default)]
        x_value: Option<u32>,
        helpers: Vec<CardId>,
    },
    /// Declare attackers: each attacker picks a defending player or a
    /// planeswalker controlled by a non-active player.
    DeclareAttackers(Vec<Attack>),
    /// CR 702.22c — declare attackers with their bands. Each band has at
    /// most one member without `Keyword::Banding`, all attacking one
    /// defender (CR 702.22d).
    DeclareAttackersBanded { attacks: Vec<Attack>, bands: Vec<Vec<CardId>> },
    /// CR 508.1g — declare attackers together with the optional attack costs
    /// paid. `exert` lists the attackers that exert (CR 701.43d); any other
    /// doesn't, so gets neither the skipped untap nor the "when you do" bonus.
    /// Plain `DeclareAttackers` leaves the choice to the engine's policy.
    DeclareAttackersExerting { attacks: Vec<Attack>, exert: Vec<CardId> },
    DeclareBlockers(Vec<(CardId, CardId)>),
    ActivateLoyaltyAbility { card_id: CardId, ability_index: usize, target: Option<Target>, #[serde(default)] x_value: Option<u32> },
    CastFlashback { card_id: CardId, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// CR 702.187 — cast a graveyard card with `Keyword::Mayhem` for its
    /// mayhem cost, legal only if its owner discarded it this turn. Delegates
    /// to the flashback machinery (exile-after tail included).
    CastMayhem { card_id: CardId, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// CR 702.180 — cast a graveyard card with `Keyword::Harmonize` for its
    /// harmonize cost, optionally tapping one untapped creature you control to
    /// reduce the total cost by generic mana equal to that creature's power.
    /// Delegates to the flashback machinery (exile-after tail included).
    CastHarmonize {
        card_id: CardId,
        #[serde(default)]
        tap_creature: Option<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a graveyard card with `Keyword::Disturb` (CR 702.146) transformed
    /// — the back face goes on the stack — for its disturb cost. `target` is
    /// used when the back face is an Aura (it needs an enchant target).
    CastDisturb {
        card_id: CardId,
        #[serde(default)]
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
    },
    /// Cast a graveyard card with `Keyword::Retrace` (CR 702.81) for its
    /// mana cost plus discarding a land card from hand. Unlike Flashback,
    /// the spell returns to the graveyard after resolving (no exile).
    CastRetrace { card_id: CardId, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Cast a graveyard card with `Keyword::Escape` (CR 702.139) for its
    /// escape mana cost plus exiling `exile_cards` (exactly N other cards
    /// from the caster's graveyard). Instants/sorceries resolve to the
    /// graveyard (re-escapable); permanents enter the battlefield.
    CastEscape { card_id: CardId, exile_cards: Vec<CardId>, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Cast a graveyard card with `Keyword::FlashbackTap(N)` by tapping
    /// exactly N untapped creatures you control (Group Project); the tap is
    /// the whole cost. Exiled on resolution per CR 702.34a via
    /// `cast_via_flashback`.
    CastFlashbackTap {
        card_id: CardId,
        tap_creatures: Vec<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a card from graveyard or exile without paying its mana cost,
    /// consuming the `CardInstance.may_play_until` permission from
    /// `Effect::GrantMayPlay` (Practiced Scrollsmith, the Paradigm cards).
    /// Rejected if the permission is missing or expired or the card has
    /// moved to a zone where the cast no longer applies.
    CastFromZoneWithoutPaying {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a commander from the command zone, paying the printed cost plus
    /// {2} per prior cast from there this game (commander tax, CR 903.8).
    /// Targets / modes / X as `CastSpell`. Phase J's replacement effect
    /// returns it to the command zone when it would leave play.
    CastFromCommandZone {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        /// CR 601.2f — pay the commander's `alternative_cost` (dash,
        /// offering, Fist of Suns) instead of its mana cost. The tax is an
        /// *additional* cost, so it still applies on top.
        #[serde(default)]
        alternative: bool,
        /// The hand card an exile-filter alternative cost pitches; `None`
        /// for every other alt cost, and ignored unless `alternative`.
        #[serde(default)]
        pitch_card: Option<CardId>,
    },
    PassPriority,
    SubmitDecision(DecisionAnswer),
    /// CR 702.29 — cycle a hand card with `Keyword::Cycling(cost)`: pay,
    /// discard, draw. "When you cycle this card" triggers fire from wherever
    /// the card lands after the discard (CR 702.29c).
    Cycle {
        card_id: CardId,
        /// X paid to an `{X}` symbol in the cycling cost ({X}{1}{U} —
        /// Shark Typhoon). Ignored for X-less cycling costs.
        #[serde(default)]
        x_value: Option<u32>,
    },
    /// CR 702.77 — Activate a card's Reinforce ability from your hand. `card_id`
    /// must carry a `Keyword::Reinforce(n, cost)`. Pays the cost, discards the
    /// card, then puts N +1/+1 counters on `target`.
    Reinforce { card_id: CardId, target: Target },
    /// Activate a card's `discard_activated` ability from your hand: pay the
    /// cost, discard the card, then resolve its targetless effect (Magma Opus).
    ActivateDiscardAbility { card_id: CardId },
    /// CR 702.29e — Activate a card's Landcycling ability from your hand.
    /// `card_id` must carry a `Keyword::Landcycling(cost, type)`. Pays the
    /// cost, discards the card, then searches the library for a land of the
    /// named type and puts it into hand (shuffling after).
    Landcycle { card_id: CardId },
    /// CR 702.6 — equip onto `target` (a creature you control) at sorcery
    /// speed (CR 702.6e), paying `Keyword::Equip(cost)`; `equipped_bonus`
    /// applies via the layer system.
    Equip { equipment: CardId, target: CardId },
    /// CR 702.151 — reconfigure at sorcery speed (Lion Sash): `Some(c)`
    /// attaches to your creature (no longer a creature while attached),
    /// `None` unattaches.
    Reconfigure { equipment: CardId, target: Option<CardId> },
    /// CR 702.122 — crew a Vehicle at instant speed by tapping
    /// `crew_creatures` (untapped, yours, not the Vehicle) with total power
    /// ≥ `Keyword::Crew(N)`; it becomes an artifact creature until end of
    /// turn.
    Crew { vehicle: CardId, crew_creatures: Vec<CardId> },
    /// CR 702.171 — saddle a Mount at sorcery speed by tapping `creatures`
    /// (untapped, yours, not the Mount) with total power ≥
    /// `Keyword::Saddle(N)`; it is saddled until end of turn.
    Saddle { mount: CardId, creatures: Vec<CardId> },
    /// CR 702.49 — Ninjutsu. During the declare-blockers step, return an
    /// unblocked attacker (`returning`) to hand and put `ninja` (a card with
    /// Ninjutsu in your hand) onto the battlefield tapped and attacking the
    /// same defender, paying the ninjutsu cost.
    Ninjutsu { ninja: CardId, returning: CardId },
    /// CR 104.3a — the acting player concedes, at any time. Carries no seat:
    /// the server attributes it to the sending connection so no client can
    /// concede for another (`GameState::concede`, `handle_action`).
    Concede,
    /// CR 901.9 — roll the planar die. A special action: legal only while the
    /// active player has priority with an empty stack during their own main
    /// phase, and it costs {N} where N is the number of times they have
    /// already taken it this turn.
    RollPlanarDie,
    /// CR 116.2j / 702.106b — turn a face-down hidden-agenda conspiracy in
    /// your command zone face up. A special action: legal any time the
    /// player holding priority controls the card.
    RevealConspiracy { card_id: CardId },
}

/// CR 901.3a — the three distinct faces of the planar die.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanarFace {
    /// Four of the six faces: nothing happens (CR 901.9a).
    Blank,
    /// Chaos ensues (CR 901.9b).
    Chaos,
    /// The planeswalking ability triggers (CR 901.9c).
    Planeswalker,
}

// ── Delayed triggers ─────────────────────────────────────────────────────────

/// A trigger registered by a resolved spell or ability that fires at a
/// specified future moment ("at the beginning of your next upkeep, ...",
/// "at the beginning of the next end step, exile this", ...). Stored on
/// `GameState::delayed_triggers` and consumed by the step-event dispatcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayedTrigger {
    /// Whose ability this is — used both for `YourNextUpkeep`-style scope
    /// matching and for the resolution ctx when the trigger fires.
    pub controller: usize,
    /// CardId of the spell/permanent that registered this trigger. Used for
    /// the resulting `StackItem::Trigger`'s `source` slot — even if the
    /// source has since left play.
    pub source: CardId,
    /// What event activates this trigger.
    pub kind: DelayedKind,
    /// Effect tree to run when the trigger fires.
    pub effect: Effect,
    /// Optional target (e.g. Goryo's exiles the reanimated creature).
    pub target: Option<Target>,
    /// Token created in the scheduling resolution, so a delayed body's
    /// `Selector::LastCreatedToken` still finds it at fire time (Saheeli
    /// Rai's +1, Reflection of Kiki-Jiki's sac-at-end-step rider).
    #[serde(default)]
    pub bound_token: Option<CardId>,
    /// The scheduling resolution's trigger subject, so a delayed body's
    /// `Selector::TriggerSource` still resolves at fire time (Shirei's "return
    /// that card"). Takes precedence over `bound_token`.
    #[serde(default)]
    pub bound_subject: Option<crate::game::effects::EntityRef>,
    /// True for one-shot triggers; removed after firing.
    pub fires_once: bool,
    /// Turn this watcher survives through: `Some(n)` keeps it until turn
    /// `n`'s cleanup ("until the end of your next turn" — Season of the
    /// Bold); `None` uses the kind's own duration.
    #[serde(default)]
    pub expires_after_turn: Option<u32>,
}

/// What kind of future event a delayed trigger waits for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedKind {
    /// Fires at the beginning of *every* player's upkeep, with the active
    /// player as the acting seat (Dimensional Breach). Registered with
    /// `fires_once: false`; the body removes it once its work is done.
    EachPlayersUpkeep,
    /// "At the beginning of the next combat" — fires on the next Begin Combat
    /// step, whoever's turn it is (Legion's Initiative).
    NextCombat,
    /// CR 508.2 — `player`'s next declare attackers step on a turn after
    /// `after_turn`, fired as the declaration finishes
    /// (`declare_attackers_delayed.rs`).
    PlayersNextDeclareAttackers { player: usize, after_turn: u32 },
    /// At the beginning of `controller`'s next upkeep.
    YourNextUpkeep,
    /// "At the beginning of the next turn's upkeep" — whoever's turn it is,
    /// gated on a turn after `after_turn` so a registration made during an
    /// upkeep waits for the *next* one (the Homelands cantrip cycle).
    NextUpkeep { after_turn: u32 },
    /// At the beginning of the next end step (any player's).
    NextEndStep,
    /// "At the beginning of the end step of that player's next turn" — fires
    /// on seat `player`'s End step, but only on a turn after `after_turn`
    /// (Suppress).
    PlayersNextEndStep { player: usize, after_turn: u32 },
    /// "At the beginning of that player's next draw step" (Nafs Asp), gated on
    /// a turn after `after_turn` so a same-turn registration waits.
    PlayersNextDrawStep { player: usize, after_turn: u32 },
    /// "At the beginning of that player's next upkeep" (Plague Reaver), gated
    /// on a turn after `after_turn`.
    PlayersNextUpkeep { player: usize, after_turn: u32 },
    /// At the beginning of the next cleanup step (Waylay). Fires after the end
    /// step, so the affected objects survive it.
    NextCleanupStep,
    /// At the beginning of `controller`'s next precombat main phase
    /// (Chancellor of the Tangle), so the mana lands while main-phase
    /// windows are open (pools empty between steps, CR 500.4).
    YourNextMainPhase,
    /// "At the beginning of your next main phase" — first or second, the
    /// controller's next main phase to begin (CR 505.1). Mana Drain
    /// countering on your own turn banks for this turn's second main phase.
    YourNextMainPhaseAny,
    /// "When [card] dies this turn, …" (CR 603.4) — fires on the watched
    /// card's `CreatureDied`; registered by `Effect::WhenTargetDiesThisTurn`,
    /// swept at cleanup (Searing Blood).
    WhenCardDies(crate::card::CardId),
    /// "… when that token dies" with no turn window (Tatsumasa, the Dragon's
    /// Fang). Same shape as `WhenCardDies` but survives cleanup.
    WhenTokenDies(crate::card::CardId),
    /// "When that creature dies" with no turn window (Xira, the Golden
    /// Sting): watches the card at that battlefield timestamp, so the card
    /// returning is a new object it ignores (CR 400.7); dropped once that
    /// object is gone. With a counter kind, fires only if it died with one
    /// on it (CR 603.4).
    WhenObjectDies(crate::card::CardId, u64, Option<crate::card::CounterType>),
    /// "Whenever a creature you control enters this turn, …" (CR 603.4) —
    /// once per entering creature, which is the trigger source. Swept at
    /// cleanup (First Day of Class).
    CreatureYouControlEntersThisTurn,
    /// "Whenever a creature [matching filter] enters this turn, …" — any
    /// controller's (CR 603.4); the filter reads the entering creature with
    /// the trigger's controller as "you". Theoretical Duplication. Expires at
    /// cleanup.
    MatchingCreatureEntersThisTurn(crate::card::SelectionRequirement),
    /// The same watch "until your next turn" (Kardur, Doomscourge): expires
    /// as the controller's next turn begins rather than at cleanup.
    MatchingCreatureEntersUntilYourNextTurn(crate::card::SelectionRequirement),
    /// "Until end of turn, whenever a creature you control dies, …" (CR 603.4)
    /// — control read from death LKI; the dead creature is the trigger
    /// source. Swept at cleanup (Waltz of Rage).
    CreatureYouControlDiesThisTurn,
    /// "Until end of turn, whenever a creature [filter] dies, …" (CR 603.4) —
    /// any player's, matched on death LKI; the dead creature is the trigger
    /// source. Swept at cleanup (Massacre Girl).
    MatchingCreatureDiesThisTurn(crate::card::SelectionRequirement),
    /// "Until end of turn, whenever a creature you control deals combat damage
    /// to a player, …" (CR 603.4). Fires per qualifying combat-damage event; the
    /// dealing creature is the trigger source. Expires at cleanup. Mistway Spy.
    CreatureYouControlDealsCombatDamageThisTurn,
    /// "Until end of turn, whenever one or more [filter] creatures you control
    /// deal combat damage to a player, …" — CR 603.2c: once per damaged player
    /// per combat-damage step, the first matching dealer the trigger source,
    /// the damaged seat the trigger player. Aphelia, Viper Whisperer.
    MatchingCreaturesDealCombatDamageBatchThisTurn(crate::card::SelectionRequirement),
    /// "Whenever a creature with the chosen name deals combat damage to a
    /// player this turn, …" (The Clone Saga III) — any controller's creature,
    /// once per creature per combat-damage event. Swept at cleanup.
    NamedCreatureDealsCombatDamageToPlayerThisTurn(String),
    /// CR 603.4 — "When you next activate an exhaust ability that isn't a mana
    /// ability this turn, [body]" (Pit Automaton). Consumed by the first such
    /// activation; expires at cleanup.
    YourNextExhaustActivationThisTurn,
    /// "When you cast your next spell this turn, …" (CR 603.7e). Fires once
    /// on the controller's next spell cast, with the cast spell bound as the
    /// trigger source; expires at cleanup if no spell was cast. Codie.
    YourNextSpellCastThisTurn,
    /// "When you next attack this turn, [body]" (CR 603.7e). Consumed by the
    /// controller's next attack declaration, after the attackers are tapped;
    /// expires at cleanup. All-Out Assault.
    YourNextAttackThisTurn,
    /// "When you next cast an instant or sorcery spell this turn, copy that
    /// spell" (Mercurial Spelldancer). Non-matching casts leave it armed.
    YourNextInstantSorceryCastThisTurn,
    /// `Effect::OnYourNextSpellOfTypeThisTurn` — consumed only by the
    /// controller's next cast of this card type (Smoldering Stagecoach).
    YourNextSpellOfTypeThisTurn(crate::card::CardType),
    /// `Effect::OnYourNextSpellMatchingThisTurn` — consumed only by the
    /// controller's next cast matching the requirement (Brass Infiniscope).
    YourNextSpellMatchingThisTurn(crate::card::SelectionRequirement),
    /// "When you cast a spell with the chosen name for the first time this
    /// turn, …" (CR 603.7e, name-gated). Fires once on the controller's next
    /// cast whose name matches the source's `named_card`; non-matching casts
    /// don't consume it. Expires at cleanup. Medomai's Prophecy III.
    YourNextNamedSpellThisTurn,
    /// "Until your next turn, whenever a creature attacks you or a
    /// planeswalker you control, [body]" (Tamiyo, Seasoned Scholar's +2).
    /// Fires per qualifying attacker (the attacker is the trigger source);
    /// expires when the controller's next turn begins.
    CreatureAttacksYouUntilYourNextTurn,
    /// "Until end of turn, whenever a creature matching the filter attacks,
    /// [body]" (Summon: Leviathan II/III). Fires per qualifying attacker —
    /// any player's — with the attacker as the trigger source; the body runs
    /// for the registering controller. Expires at cleanup.
    MatchingCreatureAttacksThisTurn(crate::card::SelectionRequirement),
    /// "Whenever that creature attacks one of your opponents this turn"
    /// (Maeve, Insidious Singer): fires per declaration in which the watched
    /// creature attacks a player who is an opponent of the watcher. Swept at
    /// cleanup with the other "this turn" kinds.
    WhenCardAttacksOpponentThisTurn(crate::card::CardId),
    /// Delif's Cone — "when [it] attacks and isn't blocked" this turn.
    WhenCardAttacksUnblockedThisTurn(crate::card::CardId),
    /// `Effect::OnMatchingBlocksThisTurn` — fires per declared blocker that
    /// matches; the blocker is the trigger source. Expires at cleanup.
    MatchingCreatureBlocksThisTurn(crate::card::SelectionRequirement),
    /// "When [that permanent] leaves the battlefield, …" — fires when the
    /// watched card leaves the battlefield, any turn (not turn-scoped).
    /// Hofri Ghostforge's token rider.
    WhenCardLeavesBattlefield(crate::card::CardId),
    /// The watched card leaves the battlefield or becomes untapped, whichever
    /// is first (Merieke Ri Berit).
    WhenCardLeavesOrUntaps(crate::card::CardId),
    /// "When you lose control of [card]" (Ray of Command): fires once on the
    /// first `ControlChanged` moving `card` away from `player`; dropped
    /// unfired when `card` leaves the battlefield.
    WhenPlayerLosesControlOf { card: crate::card::CardId, player: usize },
    /// "When you cast [card] this turn" (Havengul Lich): fires on `card`'s
    /// `SpellCast` by `player`; dropped when the card leaves its zone uncast.
    WhenPlayerCastsCard { card: crate::card::CardId, player: usize },
    /// "When [that permanent] leaves the battlefield **this turn**, …"
    /// (Runesword). Same firing site as `WhenCardLeavesBattlefield` but
    /// expires at cleanup.
    WhenCardLeavesBattlefieldThisTurn(crate::card::CardId),
    /// CR 702.55 — Haunt. Fires when the haunted creature (the watched id)
    /// dies, any turn (not turn-scoped, never expires at cleanup). The source
    /// is the exiled haunting card. Registered by `Effect::HauntCreature`.
    WhenHauntedCreatureDies(crate::card::CardId),
    /// "At the beginning of each combat this turn, …" (CR 603.4). Fires at
    /// the start of every Begin-Combat step on the controller's turn; does
    /// not fire once, expires at cleanup. Registered by
    /// `Effect::AtEachCombatThisTurn` (Full Throttle).
    EachCombatThisTurn,
    /// "At the beginning of that combat" — the `n`th combat of the
    /// controller's turn, an added one (Moraug). Fires once; expires at cleanup.
    CombatNumberThisTurn(u32),
    /// "At end of combat, …" — fires once at the current turn's end-of-combat
    /// step (`Effect::DelayUntil { kind: EndOfCombat }`; Fortune, Loyal Steed).
    EndOfCombat,
    /// "Until end of turn, whenever [the watched creature] deals damage, [body]"
    /// (CR 603.4) — any damage to anything, amount via
    /// `Value::TriggerEventAmount`. Swept at cleanup (Paladin of Prahv).
    SourceDealsDamageThisTurn(crate::card::CardId),
    /// "Until end of turn, whenever [the watched creature] deals combat damage
    /// to a player, [body]" (CR 603.4 — Captain Howler's pumped creature). The
    /// combat-and-player-scoped sibling of `SourceDealsDamageThisTurn`.
    SourceDealsCombatDamageToPlayerThisTurn(crate::card::CardId),
    /// "Until your next turn, whenever [the watched creature] deals combat
    /// damage, [body]" (CR 603.7d — Tamiyo, Field Researcher's +1). Fires per
    /// combat-damage assignment; expires as its controller's turn begins.
    SourceDealsCombatDamageUntilYourNextTurn(crate::card::CardId),
    /// `SourceDealsCombatDamageUntilYourNextTurn` for combat damage to a
    /// player only (Kang Dynasty's goaded creatures).
    SourceDealsCombatDamageToPlayerUntilYourNextTurn(crate::card::CardId),
    /// "Until end of turn, whenever you gain life, [body]" (CR 603.4). Fires per
    /// `LifeGained` event whose recipient is `DelayedTrigger.controller`, with
    /// the amount bound via `Value::TriggerEventAmount`. Expires at cleanup.
    /// Vizkopa Guildmage's second ability.
    YouGainLifeThisTurn,
    /// "For each 1 damage prevented this way, [body]" (Inkshield, CR 603.4):
    /// fires per combat damage event to `DelayedTrigger.controller` that was
    /// prevented, the amount via `Value::TriggerEventAmount`. Expires at
    /// cleanup.
    CombatDamageToYouPreventedThisTurn,
    /// "Whenever a card is put into an opponent's graveyard from anywhere this
    /// turn, [body]" (CR 603.4). That owner is bound as the body's `Target(0)`,
    /// so "that player" is exact in multiplayer. Swept at cleanup
    /// (Duskmantle Guildmage).
    CardEntersOpponentGraveyardThisTurn,
    /// "Whenever a spell or ability an opponent controls causes you to
    /// discard cards this turn, [body]" (CR 603.4) — per card discarded while
    /// `GameState.resolution_causer` is an opponent; that card is the trigger
    /// source. Swept at cleanup (Pure Intentions).
    OpponentCausesYouToDiscardThisTurn,
    /// Until end of turn, each time an opponent's permanent damages the
    /// controller, with that permanent as trigger source (Hellish Rebuke).
    /// Swept at cleanup.
    OpponentPermanentDamagesYouThisTurn,
}

// ── Pending decisions (suspendable resolution) ───────────────────────────────

/// A decision the engine is waiting on before it can continue resolving the
/// current spell or ability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecision {
    pub decision: Decision,
    pub resume: ResumeContext,
}

impl PendingDecision {
    /// [`PendingEffectState::departed_filler`] for an effect resolution
    /// paused on a per-seat ask.
    pub(crate) fn departed_filler(&self) -> Option<crate::decision::DecisionAnswer> {
        match &self.resume {
            ResumeContext::Spell { in_progress, .. }
            | ResumeContext::Trigger { in_progress, .. }
            | ResumeContext::Ability { in_progress, .. } => in_progress.departed_filler(),
            _ => None,
        }
    }

    /// CR 800.4a — whether the resolving spell or ability belongs to seat `p`
    /// (cast, owned or controlled by it), so it ceases to exist if `p` leaves.
    pub fn resolves_for(&self, p: usize) -> bool {
        match &self.resume {
            ResumeContext::Spell { card, caster, .. } => *caster == p || card.owner == p,
            ResumeContext::Trigger { controller, .. } | ResumeContext::Ability { controller, .. } => *controller == p,
            _ => false,
        }
    }

    pub fn acting_player(&self) -> usize {
        // A library search is answered by the seat the decision names — the
        // searching player (CR 701.19a), which for an opponent-owned search
        // (Boseiju, Karametra under a wants_ui opponent) or a cross-library
        // pick (Seek) is not the resume's owner.
        if let crate::decision::Decision::SearchLibrary { player, .. } = &self.decision {
            return *player;
        }
        // CR 701.9a — a player discards cards *they* choose, so "target player
        // discards N" (Arcane Omens) asks `Decision::Discard.player`, not the
        // caster (who used to pick the opponent's discards).
        if let crate::decision::Decision::Discard { player, .. } = &self.decision {
            return *player;
        }
        // Usually the caster / controller answers; `answering_player` covers
        // exceptions such as a forced sacrifice (CR 701.16), where the Edict's
        // target chooses.
        match &self.resume {
            ResumeContext::Spell { in_progress, caster, .. } => {
                in_progress.answering_player().unwrap_or(*caster)
            }
            ResumeContext::Trigger { in_progress, controller, .. } => {
                in_progress.answering_player().unwrap_or(*controller)
            }
            ResumeContext::Ability { in_progress, controller, .. } => {
                in_progress.answering_player().unwrap_or(*controller)
            }
            ResumeContext::Mulligan { player, .. } => *player,
            ResumeContext::TriggerTargetPick { pending, chooser, .. } => chooser.unwrap_or(pending.controller),
            ResumeContext::TriggerModePick { pending, .. } => pending.controller,
            // The ordering run is a single controller's triggers.
            ResumeContext::TriggerOrder { run, .. } => {
                run.first().map(|c| c.controller).unwrap_or(0)
            }
            ResumeContext::LandEntry { player, .. } => *player,
            ResumeContext::CleanupDiscard { player } => *player,
            ResumeContext::CommanderReturn { owner, .. } => *owner,
            ResumeContext::LegendRule { player, .. } => *player,
            ResumeContext::CombatDamage { player, .. } => *player,
            ResumeContext::CastAdditionalCost { caster, .. } => *caster,
            ResumeContext::ActionFloatConfirm { actor, .. } => *actor,
            ResumeContext::ActionSearchPick { actor, .. } => *actor,
            ResumeContext::ActionCounterPick { actor, .. } => *actor,
            ResumeContext::ActivateAbilityChoice { activator, .. } => *activator,
            ResumeContext::CastExtraTargetPick { caster, .. } => *caster,
            ResumeContext::CastXPick { caster, .. } => *caster,
            ResumeContext::CastSlot0TargetPick { caster, .. } => *caster,
            ResumeContext::LoyaltyExtraTargetPick { caster, .. } => *caster,
        }
    }
}

impl PendingEffectState {
    /// `inner`, answered by `player` (see [`Self::Seated`]).
    pub(crate) fn seated(player: usize, inner: Self) -> Self {
        Self::Seated { player, inner: Box::new(inner) }
    }

    /// The player who must answer this suspended decision, when it isn't the
    /// owning spell's caster / ability's controller: a forced sacrifice
    /// (CR 701.16 — the Edict's target chooses) and seat-routed yes/no
    /// questions (rhystic taxes, Tempting Offer, Clash). Everything else
    /// returns `None` (answered by the resume's owner).
    pub(crate) fn answering_player(&self) -> Option<usize> {
        match self {
            PendingEffectState::SacrificePending { player } => Some(*player),
            PendingEffectState::SeatBoolAnswerPending { player } => Some(*player),
            PendingEffectState::MayDoSeatAnswerPending { player } => Some(*player),
            PendingEffectState::Seated { player, .. } => Some(*player),
            PendingEffectState::AnyOneColorPending { player, .. }
            | PendingEffectState::DevotionColorPending { player }
            | PendingEffectState::LearnPending { player } => Some(*player),
            PendingEffectState::SeatAmountAnswerPending { player, .. } => Some(*player),
            PendingEffectState::CardsAnswerPending { player } => Some(*player),
            PendingEffectState::SeatCardsAnswerPending { player } => Some(*player),
            PendingEffectState::SeatTargetAnswerPending { player } => Some(*player),
            PendingEffectState::MayCastExiledPending { player, .. } => Some(*player),
            PendingEffectState::StashNamePending { player } => Some(*player),
            _ => None,
        }
    }

    /// CR 800.4a — the placeholder a per-seat ask takes when its seat left
    /// while it waited: logged under that seat, which the re-run's
    /// `skip_departed_answers` passes over. `None` for every other shape.
    pub(crate) fn departed_filler(&self) -> Option<crate::decision::DecisionAnswer> {
        use crate::decision::DecisionAnswer as A;
        match self {
            PendingEffectState::SeatBoolAnswerPending { .. }
            | PendingEffectState::MayDoSeatAnswerPending { .. } => Some(A::Bool(false)),
            // CR 800.4a — a departed chooser's pick is the least it can be.
            PendingEffectState::Seated { inner, .. } => inner.departed_filler().or(match **inner {
                PendingEffectState::AmountAnswerPending { .. } => Some(A::Amount(0)),
                PendingEffectState::ModeAnswerPending { .. } => Some(A::Mode(0)),
                _ => None,
            }),
            PendingEffectState::LearnPending { .. } => Some(A::Learn(crate::decision::LearnChoice::Decline)),
            PendingEffectState::SeatAmountAnswerPending { .. } => Some(A::Amount(0)),
            PendingEffectState::SeatCardsAnswerPending { .. } => Some(A::Cards(Vec::new())),
            PendingEffectState::SeatTargetAnswerPending { .. } => Some(A::DeclineTarget),
            _ => None,
        }
    }
}

/// A triggered ability gathered during event dispatch, before APNAP
/// ordering (CR 603.3) and `EventSpec::filter` gating. Module-scope so the
/// same-controller ordering pass can own the collected vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCandidate {
    pub source: CardId,
    pub effect: Effect,
    pub controller: usize,
    pub filter: Option<crate::effect::Predicate>,
    pub subject: Option<crate::game::effects::EntityRef>,
    pub event_amount: u32,
    /// The originating event was an ETB — gates Strict Proctor's tax
    /// (CR 614) at push time.
    pub triggered_by_etb: bool,
    /// True if the originating event was a creature dying — Drivnod's
    /// "a creature dying causes … triggers an additional time" doubler.
    #[serde(default)]
    pub triggered_by_death: bool,
    /// The permanent whose leaving the battlefield caused this trigger —
    /// Gandalf the White's leave doubler reads its last known information.
    #[serde(default)]
    pub triggered_by_leave: Option<CardId>,
    /// The originating event was a creature attacking (Isshin / Windcrag
    /// Siege doubler).
    #[serde(default)]
    pub triggered_by_attack: bool,
    /// The originating event was a land entering (`LandPlayed`, reported by
    /// every land entry) — Ancient Greenwarden's doubler.
    #[serde(default)]
    pub triggered_by_land_entry: bool,
    /// The originating event was a permanent turning face up (Panoptic
    /// Projektor's doubler).
    #[serde(default)]
    pub triggered_by_face_up: bool,
    /// The originating event was a card draw (Krang, the All-Powerful's
    /// doubler).
    #[serde(default)]
    pub triggered_by_draw: bool,
    /// The player whose `LifeGained` fired this trigger (Dr. Beverly
    /// Crusher's doubler).
    #[serde(default)]
    pub lifegain_player: Option<usize>,
    /// Controller of the creature whose being dealt damage fired this trigger
    /// (Wayta, Trainer Prodigy's doubler).
    #[serde(default)]
    pub damaged_creature_controller: Option<usize>,
    /// CR 605.1b — fired from a mana ability (a permanent tapped for mana).
    #[serde(default)]
    pub from_mana_ability: bool,
    /// The seat that *caused* the firing event (`event_actor`) — the caster
    /// for a `BecameTarget`, the sacrificing player for a sacrifice, and so
    /// on. Read back at resolution through `PlayerRef::TriggerEventPlayer`
    /// ("that spell or ability's controller" — Lava Runner).
    #[serde(default)]
    pub actor: Option<usize>,
}

/// One triggered ability waiting to be pushed onto the stack, captured after
/// `EventSpec::filter` gating so the resume path needn't re-walk filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTriggerPush {
    pub source: CardId,
    pub controller: usize,
    pub effect: Effect,
    pub subject: Option<crate::game::effects::EntityRef>,
    pub event_amount: u32,
    /// Mode picked at push time; `None` for a non-modal trigger.
    #[serde(default)]
    pub mode: Option<usize>,
    /// CR 603.4 — intervening 'if' clause, re-checked at resolution (the
    /// trigger does nothing if it fails). Set by `fire_step_triggers` from
    /// `EventSpec.filter` when that is a resolve-time gate; `None` otherwise.
    #[serde(default)]
    pub intervening_if: Option<crate::card::Predicate>,
    /// The seat that caused the firing event — see `TriggerCandidate.actor`.
    #[serde(default)]
    pub actor: Option<usize>,
    /// CR 605.1b — fired from a mana ability. With a targetless mana-adding
    /// body that makes it a mana ability, resolved off-stack (CR 605.4a).
    #[serde(default)]
    pub from_mana_ability: bool,
    /// CR 601.2b — the originating cast's X, converge count and mana spent,
    /// for a trigger queued straight off a resolving spell (its own ETB),
    /// where no permanent yet carries them. Zero at every other push site,
    /// keeping the resolve-time fallbacks — see
    /// [`GameState::push_pending_trigger`].
    ///
    /// [`GameState::push_pending_trigger`]: crate::game::GameState::push_pending_trigger
    #[serde(default)]
    pub x_value: u32,
    #[serde(default)]
    pub converged_value: u32,
    #[serde(default)]
    pub mana_spent: u32,
}

/// Which "another permanent" activated-ability cost a suspended
/// [`ResumeContext::ActivateAbilityChoice`] is gathering a pick for, so the
/// resume stashes the answer in the matching transient field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityCostChoice {
    /// `sac_other_filter` — "Sacrifice another …".
    SacOther,
    /// `tap_other_filter` — "Tap an untapped … you control".
    TapOther,
    /// `exile_other_filter` — "Exile N cards from your graveyard" (Grim
    /// Lavamancer, Scrapheap Scrounger). Answered via a `ChooseCards` modal
    /// since graveyard cards aren't selectable with the in-scene cursor.
    ExileOther,
    /// `sac_any_number_filter` — "…and any number of [filter] you control"
    /// (Sword of the Ages). Answered via a `ChooseCards` modal with `min: 0`.
    SacAnyNumber,
    /// The ability's *effect* targets a graveyard card ("Return target
    /// creature card from your graveyard" — Cauldron of Essence). Answered
    /// via a `ChooseCards` modal; the resume binds the pick as the
    /// activation's slot-0 target.
    GraveyardTarget,
    /// The ability's mana cost has {X} and the activator sent no X
    /// ({X}, {T}: … — Berta, Imbraham). Answered via a `ChooseAmount`
    /// modal; the resume replays with the chosen X.
    XValue,
    /// `exile_permanent_cost` — "Exile a [filter] you control:" (Primordial
    /// Mist's face-down permanent). Answered as a battlefield `Target`.
    ExilePermanent,
    /// `exiled_with_self_to_graveyard_cost` — "Put a creature card exiled
    /// with this into its owner's graveyard:" (Shelob). Answered as `Cards`.
    ExiledWithToGraveyard,
    /// `unattach_equipment_cost` — "Unattach an Equipment from [this]:"
    /// (Captain America, First Avenger's Throw). Answered as a battlefield
    /// `Target`.
    UnattachEquipment,
    /// `discard_cost` — "Discard a [filter] card:" (Solitary Cell's legendary
    /// card, Fauna Shaman's creature card). Answered as `Cards`; a seat that
    /// isn't asked still discards the cheapest matches.
    Discard,
    /// `blight_cost` — CR 701.68a: the activator chooses the creature that
    /// takes the -1/-1 counters. Answered as a battlefield `Target`.
    Blight,
    /// `bounce_other_filter` with a count of one — "Return a [filter] you
    /// control to its owner's hand:" (Zareth San). A battlefield `Target`.
    BounceOther,
}

impl ResumeContext {
    /// Whether this suspend stashed an action to *replay*, rather than
    /// pausing something already committed.
    ///
    /// Affordance probes must not read a replay suspend's `Ok` as "accepted":
    /// nothing was paid yet, and treating it as castable let the bot commit
    /// unpayable casts whose failed replay wedged the game. The other
    /// contexts pause the resolution of something already done, so their
    /// `Ok` is honest.
    pub(crate) fn is_action_replay(&self) -> bool {
        matches!(
            self,
            ResumeContext::CastAdditionalCost { .. }
                | ResumeContext::ActionFloatConfirm { .. }
                | ResumeContext::ActionSearchPick { .. }
                | ResumeContext::ActionCounterPick { .. }
                | ResumeContext::ActivateAbilityChoice { .. }
                | ResumeContext::CastExtraTargetPick { .. }
                | ResumeContext::CastXPick { .. }
                | ResumeContext::CastSlot0TargetPick { .. }
                | ResumeContext::LoyaltyExtraTargetPick { .. }
        )
    }
}

/// Recorded where resolution suspended so it can resume after the decision.
/// `remaining` is whatever effects in the original tree still need to run
/// after the answered decision is applied (e.g. the `Draw` half of `Opt`
/// suspended on its `Scry`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResumeContext {
    /// CR 603.3b — a networked controller is ordering a same-controller
    /// run of simultaneous triggers. `ordered` holds the runs already
    /// sequenced, `run` the one being asked about, `rest` the candidates
    /// after it (which may suspend again on a later run).
    TriggerOrder {
        ordered: Vec<TriggerCandidate>,
        run: Vec<TriggerCandidate>,
        rest: Vec<TriggerCandidate>,
    },
    Spell {
        card: Box<CardInstance>,
        caster: usize,
        target: Option<Target>,
        /// Additional targets (slots 1+). Defaults to empty for snapshot
        /// back-compat with single-target spells.
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        /// Mana spent on the cast, for `Value::CastSpellManaSpent` on resume.
        #[serde(default)]
        mana_spent: u32,
        in_progress: PendingEffectState,
        remaining: Effect,
        /// Which part of the spell `remaining` belongs to: 0 the main effect,
        /// 1 a fused split's right half (CR 709 / 702.102), 2 + i spliced
        /// effect `i` (CR 702.47b). Without it, a suspend replayed the whole
        /// tail on every resume (Far // Away took two creatures).
        #[serde(default)]
        tail_stage: u8,
    },
    Trigger {
        source: CardId,
        controller: usize,
        target: Option<Target>,
        mode: usize,
        in_progress: PendingEffectState,
        remaining: Effect,
        /// The originating spell's X, for `Value::XFromCost` on resume.
        #[serde(default)]
        x_value: u32,
        /// Converge value, as `x_value` for `Value::ConvergedValue`.
        #[serde(default)]
        converged_value: u32,
        /// Mana spent, as `x_value` for `Value::CastSpellManaSpent`.
        #[serde(default)]
        mana_spent: u32,
        /// The trigger's subject (`ctx.trigger_source`), so
        /// `PlayerRef::Triggerer` / `Selector::TriggerSource` resolve on resume.
        #[serde(default)]
        trigger_source_ent: Option<crate::game::effects::EntityRef>,
        /// The firing event's amount (`Value::TriggerEventAmount`).
        #[serde(default)]
        event_amount: u32,
        /// The player the firing event named (`ControlledByTriggerPlayer`).
        #[serde(default)]
        trigger_player: Option<usize>,
        /// Extra targets (slots 1+) for two-target activated abilities,
        /// preserved across a mid-resolution suspend.
        #[serde(default)]
        additional_targets: Vec<Target>,
        /// `StackItem::Trigger::ts_in_gy`, kept across the suspend (CR 400.7).
        #[serde(default)]
        ts_in_gy: bool,
    },
    Ability {
        source: CardId,
        controller: usize,
        target: Option<Target>,
        in_progress: PendingEffectState,
        remaining: Effect,
    },
    /// Pre-game mulligan phase for `player`. After this player keeps,
    /// mulligan advances to `next_player` (None = all players done, start game).
    Mulligan {
        player: usize,
        mulligans_taken: usize,
        next_player: Option<usize>,
    },
    /// CR 700.2a / 603.3c — a hand-paying seat (`Player::manual_mana`) picks
    /// the mode of its modal trigger whose modes target, before the target:
    /// target slots are bound at push time, so the pick can't defer to
    /// resolution the way a targetless modal's does. On answer, `pending`
    /// takes the mode and the queue drains on from it (asking its target).
    TriggerModePick {
        pending: PendingTriggerPush,
        remaining: Vec<PendingTriggerPush>,
    },
    /// Suspended before pushing a trigger whose `wants_ui` controller must
    /// pick a target. On answer `pending` is pushed with it and `remaining`
    /// (the rest of the event batch) drains on, possibly suspending again.
    TriggerTargetPick {
        pending: PendingTriggerPush,
        remaining: Vec<PendingTriggerPush>,
        /// The seat aiming the trigger, when not its controller ("that
        /// player may … target …" — Curse of Inertia's attacker).
        #[serde(default)]
        chooser: Option<usize>,
        /// The slots named so far when the ask is for a further target of an
        /// "up to N target" fan-out (`trigger_slots.rs`); empty for slot 0.
        #[serde(default)]
        picked: Vec<Target>,
    },
    /// CR 601.2b — a `wants_ui` caster picks what to sacrifice (Crop
    /// Rotation) or discard (Big Score) for an additional cast cost. The cast
    /// suspended before anything was paid and `cast_spell` is re-run verbatim
    /// with the pick in `pending_cast_sacrifices` / `pending_cast_discards`
    /// (possibly suspending again for a further cost).
    CastAdditionalCost {
        caster: usize,
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        /// The suspended cast was kicked (CR 702.33 — the replay re-invokes
        /// `cast_spell_kicked`, so an action-kicker sacrifice choice
        /// round-trips without dropping the kick).
        #[serde(default)]
        kicked: bool,
    },
    /// CR 601.2g — a `wants_ui` payer answers "spend floating mana, or tap
    /// sources?" before a payment. The action suspended at its payment point
    /// (pre-payment mutations rolled back) and the stored `GameAction` is
    /// replayed verbatim with the choice in `pending_cast_spend_float`
    /// (`Bool(true)` = spend the float).
    ActionFloatConfirm {
        actor: usize,
        action: Box<GameAction>,
    },
    /// CR 601.2b — a `manual_mana` caster names the creature the next counter
    /// of a "remove N counters from among creatures you control" cost comes
    /// off (Dawnhand Dissident's exile cast). Answer `Cards([id])`, appended
    /// to `pending_cast_counter_plan`; `action` replays (and may ask again).
    ActionCounterPick {
        actor: usize,
        action: Box<GameAction>,
    },
    /// CR 614.12a — a land drop's as-enters replacement asks its `wants_ui`
    /// controller (Cavern of Souls). Not a replay: the land is already in
    /// play, so the resume finishes the entry (`finish_land_entry`).
    LandEntry {
        player: usize,
        card_id: CardId,
        in_progress: PendingEffectState,
        remaining: Effect,
    },
    /// CR 702.29e — a `wants_ui` cycler is picking which matching library
    /// card a landcycle / typecycle fetches. The action suspended before any
    /// cost was paid and is replayed verbatim with the pick stashed in
    /// `pending_landcycle_pick`.
    ActionSearchPick {
        actor: usize,
        action: Box<GameAction>,
    },
    /// CR 602.5 — a `wants_ui` activator picks for one of an activated
    /// ability's cost choices; `kind` names which transient field gets the
    /// pick. The activation suspended before any cost was paid and
    /// `activate_ability` is re-run verbatim on answer.
    ActivateAbilityChoice {
        activator: usize,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        x_value: Option<u32>,
        kind: AbilityCostChoice,
    },
    /// CR 514.1 — a `wants_ui` active player discards down to hand size at
    /// cleanup. Re-posed on an under-discard; otherwise cleanup continues.
    CleanupDiscard { player: usize },
    /// CR 903.9a — a prompting owner chooses whether `commander`, in their
    /// graveyard or in exile, goes to the command zone
    /// (`GameState::pose_commander_return`).
    CommanderReturn { owner: usize, commander: CardId },
    /// CR 704.5j — a prompting `player` chooses which of their same-name
    /// legends to keep (`GameState::pose_legend_choice`).
    LegendRule { player: usize, name: String, duplicates: Vec<(CardId, String)> },
    /// CR 510.1c-d — a `wants_ui` active player chooses `attacker`'s damage
    /// ordering / assignment. The answer is cached and the damage step
    /// re-entered.
    CombatDamage {
        /// The active player making the choice (always the attacking player).
        player: usize,
        attacker: CardId,
        kind: CombatDecisionKind,
    },
    /// A `wants_ui` caster's spell needs a target slot the client didn't send
    /// (Chelonian Tackle's fight defender). Suspended before payment; the
    /// pick is appended to `additional_targets` and the action replayed.
    CastExtraTargetPick {
        caster: usize,
        action: Box<GameAction>,
    },
    /// A `wants_ui` caster's {X} spell arrived with no X (the client never
    /// sends one). The cast suspended on a `ChooseAmount` before any cost
    /// was paid; on answer the amount is written into the action's
    /// `x_value` and the action is replayed.
    CastXPick {
        caster: usize,
        action: Box<GameAction>,
    },
    /// A `wants_ui` caster's spell targets an off-board card ("return
    /// target card from your graveyard") the cursor can't select. The cast
    /// suspended on a `ChooseCards` modal; on answer the pick becomes the
    /// action's slot-0 target and the action is replayed.
    CastSlot0TargetPick {
        caster: usize,
        action: Box<GameAction>,
    },
    /// A prompting seat's multi-target loyalty ability (Jeska's −X "each of
    /// up to three targets") suspended to name its next slot before any
    /// cost was paid; on answer the pick joins `picked` (a decline closes
    /// the list) and the activation re-runs.
    LoyaltyExtraTargetPick {
        caster: usize,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        picked: Vec<Target>,
        x_value: Option<u32>,
    },
}

impl GameEvent {
    /// True for cost-payment events that cannot make a state-based action
    /// apply: announcement, mana, tapping, targeting, hand and library
    /// traffic. Anything moving a permanent, counter or life total — or of
    /// unknown kind — is excluded, since the gate sweeps on doubt. So is a
    /// card leaving a graveyard: graveyard counts feed P/T (Lhurgoyf), and
    /// casting out of one once left a 0/0 Germ alive.
    pub(crate) fn inert_for_state_based_actions(&self) -> bool {
        use GameEvent as E;
        matches!(
            self,
            E::SpellCast { .. }
                | E::AbilityActivated { .. }
                | E::ManaAdded { .. }
                | E::ColorlessManaAdded { .. }
                | E::TappedForMana { .. }
                | E::PermanentTapped { .. }
                | E::PermanentUntapped { .. }
                | E::BecameTarget { .. }
                | E::ChoseTargets { .. }
                | E::SpellTargetChanged { .. }
                | E::CardDrawn { .. }
                | E::FirstCardDrawnThisTurn { .. }
                | E::NthCardDrawnThisTurn { .. }
                | E::CardDiscarded { .. }
                | E::DiscardedBatch { .. }
                | E::OpponentCausedYouToDiscard { .. }
                | E::LibraryShuffled { .. }
                | E::PlayerSearchedLibrary { .. }
                | E::TopCardRevealed { .. }
                | E::ScriedOrSurveiled { .. }
                | E::ScryPerformed { .. }
                | E::SurveilPerformed { .. }
                | E::CardCycled { .. }
                | E::StepChanged(_)
                | E::TurnStarted { .. }
        )
    }
}

impl GameAction {
    /// True for actions a Revel-in-Silence lock forbids: casting any spell
    /// (every `Cast*` variant) or activating a loyalty ability.
    pub(crate) fn is_cast_or_loyalty(&self) -> bool {
        self.is_cast() || matches!(self, GameAction::ActivateLoyaltyAbility { .. })
    }

    /// True for every `Cast*` action variant — the CR 601 "cast a spell"
    /// surface, gated by one-spell-per-turn locks (Rule of Law).
    pub(crate) fn is_cast(&self) -> bool {
        use GameAction as A;
        matches!(
            self,
            A::CastSpell { .. }
                | A::CastSpellKicked { .. }
                | A::CastSpellKickers { .. }
                | A::CastSpellMultikicked { .. }
                | A::CastBestow { .. }
                | A::CastSpellBuyback { .. }
                | A::CastSpellEntwine { .. }
                | A::CastRoomDoor { .. }
                | A::CastFaceDown { .. }
                | A::CastForetold { .. }
                | A::CastAdventure { .. }
                | A::CastOmen { .. }
                | A::CastGift { .. }
                | A::CastAdventureCreature { .. }
                | A::CastPrototype { .. }
                | A::CastMutate { .. }
                | A::CastSplitRight { .. }
                | A::CastSplitFused { .. }
                | A::CastAftermath { .. }
                | A::CastSpellCasualty { .. }
                | A::CastSpellBargain { .. }
                | A::CastSpellSpree { .. }
                | A::CastSpellSpliced { .. }
                | A::CastSpellSacrificeReduce { .. }
                | A::CastSpellSquad { .. }
                | A::CastSpellReplicate { .. }
                | A::CastSpellConspire { .. }
                | A::CastPlotted { .. }
                | A::CastSpellBack { .. }
                | A::CastPrepareSpell { .. }
                | A::CastSpellConvoke { .. }
                | A::CastSpellWaterbend { .. }
                | A::CastSpellDelve { .. }
                | A::CastSpellConvokeDelve { .. }
                | A::CastSpellAlternative { .. }
                | A::CastSpellGrantedAlternative { .. }
                | A::CastFlashback { .. }
                | A::CastMayhem { .. }
                | A::CastHarmonize { .. }
                | A::CastDisturb { .. }
                | A::CastRetrace { .. }
                | A::CastEscape { .. }
                | A::CastFlashbackTap { .. }
                | A::CastFromZoneWithoutPaying { .. }
                | A::CastFromCommandZone { .. }
        )
    }

    /// The card of a cast variant that runs the hand pipeline
    /// (`cast_spell_with_convoke`) and so can cast a commander from the
    /// command zone on a hop (CR 903.8; Adventure CR 716.3).
    /// `CastFromCommandZone` and mutate have their own paths.
    pub(crate) fn command_zone_hop_card(&self) -> Option<CardId> {
        use GameAction as A;
        match self {
            A::CastSpellKicked { card_id, .. }
            | A::CastSpellKickers { card_id, .. }
            | A::CastSpellMultikicked { card_id, .. }
            | A::CastBestow { card_id, .. }
            | A::CastSpellWaterbend { card_id, .. }
            | A::CastSpellConvoke { card_id, .. }
            | A::CastSpellDelve { card_id, .. }
            | A::CastSpellConvokeDelve { card_id, .. }
            | A::CastFaceDown { card_id }
            | A::CastAdventure { card_id, .. } => Some(*card_id),
            _ => None,
        }
    }

    /// The cast card of a `Cast*` action (all but `CastPrepareSpell`, which
    /// names a battlefield creature), for lock pieces that inspect the
    /// spell's types before payment.
    pub(crate) fn cast_card_id(&self) -> Option<CardId> {
        use GameAction as A;
        match self {
            A::CastSpell { card_id, .. }
            | A::CastSpellKicked { card_id, .. }
            | A::CastSpellMultikicked { card_id, .. }
            | A::CastBestow { card_id, .. }
            | A::CastSpellBuyback { card_id, .. }
            | A::CastSpellEntwine { card_id, .. }
            | A::CastRoomDoor { card_id, .. }
            | A::CastFaceDown { card_id, .. }
            | A::CastForetold { card_id, .. }
            | A::CastAdventure { card_id, .. }
            | A::CastOmen { card_id, .. }
            | A::CastGift { card_id, .. }
            | A::CastAdventureCreature { card_id, .. }
            | A::CastPrototype { card_id, .. }
            | A::CastMutate { card_id, .. }
            | A::CastSplitRight { card_id, .. }
            | A::CastSplitFused { card_id, .. }
            | A::CastAftermath { card_id, .. }
            | A::CastSpellCasualty { card_id, .. }
            | A::CastSpellBargain { card_id, .. }
            | A::CastSpellSpree { card_id, .. }
            | A::CastSpellSpliced { card_id, .. }
            | A::CastSpellSacrificeReduce { card_id, .. }
            | A::CastSpellSquad { card_id, .. }
            | A::CastSpellReplicate { card_id, .. }
            | A::CastSpellConspire { card_id, .. }
            | A::CastPlotted { card_id, .. }
            | A::CastSpellBack { card_id, .. }
            | A::CastSpellConvoke { card_id, .. }
            | A::CastSpellWaterbend { card_id, .. }
            | A::CastSpellDelve { card_id, .. }
            | A::CastSpellConvokeDelve { card_id, .. }
            | A::CastSpellAlternative { card_id, .. }
            | A::CastSpellGrantedAlternative { card_id, .. }
            | A::CastFlashback { card_id, .. }
            | A::CastMayhem { card_id, .. }
            | A::CastHarmonize { card_id, .. }
            | A::CastDisturb { card_id, .. }
            | A::CastRetrace { card_id, .. }
            | A::CastEscape { card_id, .. }
            | A::CastFlashbackTap { card_id, .. }
            | A::CastFromZoneWithoutPaying { card_id, .. }
            | A::CastFromCommandZone { card_id, .. } => Some(*card_id),
            _ => None,
        }
    }
}

/// Which of an attacker's two combat-damage choices a `ResumeContext::
/// CombatDamage` is waiting on: the order its multiple blockers are dealt
/// damage in (CR 510.1c), or how its power is divided among them (510.1d).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatDecisionKind {
    Order,
    Assign,
    /// Yes/no: assign this attacker's damage as though it weren't blocked.
    AsThoughUnblocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingEffectState {
    ScryPeeked { count: usize, player: usize },
    /// Look at the top `count` cards and put them back on top in any order
    /// (Index, Spire Owl). Unlike Scry, no card may be bottomed — every
    /// peeked card returns to the top in the chosen order.
    RearrangePeeked { count: usize, player: usize },
    /// The bottom `count` cards of the library, to be put back in the chosen
    /// order (`Effect::OrderLibraryBottom`).
    BottomOrderPeeked { count: usize, player: usize },
    SurveilPeeked { count: usize, player: usize },
    SearchPending {
        player: usize,
        to: crate::effect::ZoneDest,
        /// Filter-matching candidate ids computed at raise time; a pick
        /// outside the set is rejected (enforces e.g. "with different
        /// names" across sequential searches).
        #[serde(default)]
        eligible: Option<Vec<crate::card::CardId>>,
        /// Multi-zone search (library *and/or* graveyard/hand — Delivery
        /// Moogle, Dark Supplicant): a pick not found in the library is taken
        /// from whichever of these zones holds it.
        #[serde(default)]
        include_graveyard: bool,
        #[serde(default)]
        include_hand: bool,
        /// False for a zones-search that omits the library — no shuffle.
        #[serde(default = "crate::game::types::default_true")]
        include_library: bool,
        /// The searching effect's source, so a `ZoneDest::ExileWithSourceStamp`
        /// destination can stamp `exiled_with` after the placement (Distant
        /// Memories). `place_card_in_dest` has no effect context of its own.
        #[serde(default)]
        source: Option<crate::card::CardId>,
        /// Opposition Agent — the seat controlling this search (it answered
        /// the pick); a card found in the library is exiled with a may-play
        /// permission for this seat instead of going to `to`.
        #[serde(default)]
        hijacked_by: Option<usize>,
    },
    /// Dakkon −6: the picked hand/graveyard card enters the battlefield.
    PutFromZonesPending { player: usize },
    /// Cabal Therapy: the named-card answer makes `who` discard every
    /// matching hand card.
    NameDiscardMatchingPending { who: usize },
    /// Slaughter Games — exile every card named by the chooser from `who`'s
    /// hand, graveyard, and library, then shuffle.
    NameExileAllZonesPending { who: usize },
    /// Brain Pry: `who` discards one card with the named name; if they have
    /// none, `namer` draws a card.
    NameDiscardOneOrDrawPending { who: usize, namer: usize },
    /// Tamiyo +1: the named-card answer drives the reveal-top sort
    /// (matching name → hand, rest → graveyard).
    NameRevealTopPending { player: usize, count: usize },
    /// Conundrum Sphinx: stash `player`'s named card in
    /// `GameState.scratch.names_this_resolution` for the later reveal step.
    StashNamePending { player: usize },
    /// CR 701.48 — suspended on a `Decision::Learn`. The resume step reads
    /// `DecisionAnswer::Learn(choice)` and reveals a Lesson into `player`'s
    /// hand, rummages (discard-then-draw), or declines.
    LearnPending { player: usize },
    /// Suspended on a `LookPickToHand` decision (Impulse / Strategic
    /// Planning / Flow State). `revealed` is the peeked top-of-library set;
    /// the chosen card goes to hand and the rest to the bottom of the
    /// library (or graveyard if `rest_to_graveyard`).
    ImpulsePending {
        player: usize,
        revealed: Vec<CardId>,
        rest_to_graveyard: bool,
        /// Subset of `revealed` eligible to take (Satyr Wayfinder — lands
        /// only); `None` = all, `Some` may be empty.
        #[serde(default)]
        eligible: Option<Vec<CardId>>,
        /// Number of cards to put into hand (default 1). The decision picks
        /// the first; any remainder auto-fills from the eligible revealed set.
        #[serde(default = "crate::game::types::one")]
        take: usize,
        /// Picks enter the battlefield instead of going to hand (Collected
        /// Company).
        #[serde(default)]
        to_battlefield: bool,
        /// When `to_battlefield`, picks enter tapped (Ignis Scientia's
        /// dig-for-a-land).
        #[serde(default)]
        tapped: bool,
        /// Picks stay on top of the library (Sage of Days).
        #[serde(default)]
        keep_on_top: bool,
        /// "If you put a [filter] card into your hand this way, gain N life"
        /// (Chrome Courier).
        #[serde(default)]
        gain_life_if_pick: Option<(crate::card::SelectionRequirement, u32)>,
        /// Gain life equal to the greatest power among creature cards milled
        /// by the rest-to-graveyard sweep (Discerning Taste).
        #[serde(default)]
        gain_life_greatest_power_rest: bool,
        /// Printed "you may put …": an empty or partial UI pick is honored
        /// rather than auto-filled.
        #[serde(default)]
        optional: bool,
        /// Picked LAND cards route to the battlefield tapped; other picks
        /// go to hand (Zimone's Experiment).
        #[serde(default)]
        picked_lands_to_battlefield: bool,
        /// Bottom the rest in a RANDOM order instead of revealed order.
        #[serde(default)]
        rest_bottom_random: bool,
        /// Non-picked cards are exiled instead of bottomed/milled (Devourer
        /// of Destiny's opening-hand reveal). Overrides `rest_to_graveyard`.
        #[serde(default)]
        rest_to_exile: bool,
        /// The rest stay on top of the library (Diabolic Vision).
        #[serde(default)]
        rest_on_top: bool,
        /// "If you put a card into your hand this way, [effect]" (Sidequest:
        /// Catch a Fish). Runs once, after the picks land, with `source` as the
        /// effect's source so `Selector::This` reaches the card that asked.
        #[serde(default)]
        then_if_picked: Option<Box<crate::effect::Effect>>,
        /// The mirror rider, run when the pick set came back empty.
        #[serde(default)]
        then_if_not_picked: Option<Box<crate::effect::Effect>>,
        /// Picks matching this filter route to the battlefield (Break Out).
        #[serde(default)]
        picked_matching_to_battlefield: Option<crate::card::SelectionRequirement>,
        /// Battlefield-routed picks gain haste until end of turn (Break Out).
        #[serde(default)]
        battlefield_haste: bool,
        /// `LookPick::one_each` — each pick fills a distinct filter.
        #[serde(default)]
        one_each: Vec<crate::card::SelectionRequirement>,
        /// `LookPick::rest_to_hand_if` — read after the picks land.
        #[serde(default)]
        rest_to_hand_if: Option<crate::effect::Predicate>,
        /// `LookPick::total_mana_value_cap`, evaluated as the look resolved.
        #[serde(default)]
        mv_cap: Option<u32>,
        /// Source of the originating `LookPickToHand`, for `then_if_picked`.
        #[serde(default)]
        source: Option<CardId>,
    },
    /// Suspended on a `SearchLibrary` pick for `Effect::PayLifeLookTake`
    /// (Plunge into Darkness mode 1): the chosen card goes to hand and the
    /// rest of `revealed` are exiled.
    PayLifeLookPending {
        player: usize,
        revealed: Vec<CardId>,
    },
    /// Suspended on the exile pick of `Effect::PayLifeRevealExileFromHand`
    /// (Vizkopa Confessor): the controller chose one of `opp`'s revealed hand
    /// cards to exile.
    PayLifeExileFromHandPending {
        opp: usize,
        revealed: Vec<CardId>,
    },
    /// Suspended on a `ChooseCards` pick for `Effect::RevealTopTakeOnePerType`
    /// (Atraxa, Grand Unifier — "up to one card of each card type"). Picks
    /// are validated one-per-type at apply time; the rest go to the bottom.
    TakeOnePerTypePending {
        player: usize,
        revealed: Vec<CardId>,
    },
    PutOnLibraryPending { player: usize, count: usize },
    /// Suspended on a `ChooseColor` for an `AnyOneColor(count)` mana payload
    /// (Black Lotus); adds `count` of the color, carrying `restriction` if
    /// set (Great Hall of the Biblioplex).
    AnyOneColorPending {
        player: usize,
        count: u32,
        restriction: Option<crate::mana::SpendRestriction>,
    },
    /// Suspended on a `ChooseColor` for a `DevotionOfChosenColor` mana
    /// payload — Nykthos, Shrine to Nyx. The player picks a color, then the
    /// engine adds mana of that color equal to their devotion to it.
    DevotionColorPending { player: usize },
    /// Suspended on a `ChooseColor` for `Effect::PreventAllDamageFromChosen
    /// ColorThisTurn` (Avacyn, Guardian Angel): one shield per carried
    /// recipient, scoped to the chosen color.
    PreventFromChosenColorPending { targets: Vec<PreventionTarget> },
    /// Suspended on a `DiscardChosen` decision (Inquisition of Kozilek,
    /// Thoughtseize). The caster picks cards from `target_player`'s hand;
    /// the apply step removes them and graveyards them.
    DiscardChosenPending {
        target_player: usize,
        /// `DiscardAnyNumber`'s "up to N" cap, applied to the answer.
        #[serde(default)]
        max: Option<u32>,
    },
    /// Suspended on a `BottomChosenFromHandAndDraw` decision (Vendilion
    /// Clique). The caster picks cards from `target_player`'s hand; the apply
    /// step bottoms each chosen card and has `target_player` draw a
    /// replacement.
    BottomChosenFromHandAndDrawPending { target_player: usize },
    TopChosenFromHandPending { target_player: usize },
    /// CR 701.16 — suspended on a `ChooseTarget` for a forced single
    /// sacrifice (Edicts, Annihilator 1). `player` sacrifices and chooses.
    /// Multi-sacrifice keeps the auto-pick.
    SacrificePending { player: usize },
    /// Suspended on an `ExileChosenUntilSourceLeaves` decision (Brain
    /// Maggot, Tidehollow Sculler, Kitesail Freebooter). The caster picks
    /// cards from `target_player`'s hand; the apply step exiles them linked
    /// to `source`, returning them to `return_to` when `source` leaves.
    ExileChosenUntilSourceLeavesPending {
        target_player: usize,
        source: CardId,
        return_to: crate::card::ExileReturnZone,
    },
    /// Suspended on an `ExileChosenFromHand` decision (Thought-Knot Seer). The
    /// caster picks cards from `target_player`'s hand; the apply step exiles
    /// them permanently.
    ExileChosenFromHandPending {
        target_player: usize,
        #[serde(default)]
        link_source: Option<crate::card::CardId>,
        #[serde(default)]
        face_down: bool,
    },
    /// Suspended on a `HoneFromHand` decision (Uvilda, Dean of Perfection).
    /// The owner picks an instant/sorcery from their hand; the apply step
    /// exiles it with `count` hone counters.
    HoneFromHandPending { target_player: usize, count: u32 },
    /// Suspended on an `ExileFromHandTaxed` decision (Elite Spellbinder). The
    /// caster picks card(s) from `target_player`'s hand; the apply step exiles
    /// each and grants the owner a may-play permission taxed `extra_cost` more.
    ExileFromHandTaxedPending { target_player: usize, extra_cost: u32 },
    /// Suspended on a `ChooseCreatureType` for `Effect::NameCreatureType`
    /// (Cavern of Souls), stamped onto `target_id.chosen_creature_type`.
    /// Non-empty `options` is a closed list (A Killer Among Us); an answer
    /// outside it clamps to the first.
    ChooseCreatureTypePending {
        target_id: CardId,
        #[serde(default)]
        options: Vec<crate::card::CreatureType>,
    },
    /// CR 612.1 — suspended on a `ChooseCreatureTypePair` for
    /// `Effect::ReplaceCreatureTypeText`; the answer rewrites `target_id`'s
    /// definition in place (Artificial Evolution).
    ReplaceCreatureTypeTextPending {
        target_id: CardId,
        /// The fixed replacement type, when the card names one (New Blood).
        #[serde(default)]
        to: Option<crate::card::CreatureType>,
    },
    /// Suspended on a `NameCard` decision for `Effect::NameCard` (Pithing
    /// Needle, Phyrexian Revoker). The chooser names a card and the engine
    /// stamps it onto `target_id.named_card`.
    NameCardPending {
        target_id: CardId,
        /// CR 201.4a namespace restriction; an answer outside it is dropped.
        #[serde(default)]
        restrict_to: Option<crate::card::SelectionRequirement>,
    },
    /// Suspended on the seat ballot of `Effect::ChoosePlayerForSource`; the
    /// `Amount(i)` answer stamps `seats[i]` on `target_id.chosen_player`.
    ChosenPlayerPending { target_id: CardId, seats: Vec<usize> },
    /// As `ChosenPlayerPending`, for "choose two players": `pairs[i]` is
    /// recorded in `chosen_player_pairs` (Sower of Discord).
    ChosenPlayerPairPending { target_id: CardId, pairs: Vec<(usize, usize)> },
    /// Suspended on a `NameCard` decision for `Effect::NameOpponentCastLock`
    /// (Academic Probation). The chooser names a card and the engine records
    /// it in `players[caster].opponents_cant_cast_named`.
    OpponentNameLockPending { caster: usize },
    /// Suspended on the `NameCard` decision of
    /// `Effect::NameCardThenNamedCreatureCombatDamageThisTurn`; the answer arms
    /// a `NamedCreatureDealsCombatDamageToPlayerThisTurn` watcher running `body`.
    NamedCreatureCombatDamagePending { controller: usize, source: CardId, body: Box<Effect> },
    /// Suspended on a `ChooseModes` for a resolution-time "choose N" /
    /// Escalate effect. The validated answer goes to
    /// `GameState.scratch.stashed_resolution_answer`, which the re-queued
    /// effect consumes instead of re-asking.
    ModesAnswerPending { num_modes: usize },
    /// Suspended on a `ChooseMode` for a modal trigger whose pick was
    /// deferred to resolution (`wants_ui` controller). Same stash-and-rerun
    /// shape as `ModesAnswerPending`.
    ModeAnswerPending { num_modes: usize },
    /// Suspended on a `ChooseAmount` ("pay any amount of life", "sacrifice
    /// any number"). The answer is clamped to `max` then stashed.
    AmountAnswerPending { max: u32 },
    /// As `AmountAnswerPending`, but the number is chosen by `player` rather
    /// than the resolving controller (Choice of Damnations, Pain's Reward).
    SeatAmountAnswerPending { player: usize, max: u32 },
    /// Suspended on the `OptionalTrigger` raised by `Effect::MayDo`. The
    /// yes/no answer is stashed for the re-run. Kept for older saves; new
    /// asks name their seat (`MayDoSeatAnswerPending`).
    MayDoAnswerPending,
    /// As `MayDoAnswerPending`, answered by `player` — the seat whose "may"
    /// it is, which under a per-player fan-out ("each opponent may draw")
    /// is not the resolving controller.
    MayDoSeatAnswerPending { player: usize },
    /// `inner`, answered by `player` rather than the resume's owner: the seat
    /// the ask prompted (a per-player fan-out's opponent, a payer, a chooser).
    /// The answer is applied as `inner`'s.
    Seated { player: usize, inner: Box<PendingEffectState> },
    /// A yes/no routed to `player` (rhystic taxes, Tempting Offer, Clash).
    /// The answer is appended to `GameState.scratch.resolution_answer_log`,
    /// which the re-run replays in ask order.
    SeatBoolAnswerPending { player: usize },
    /// Suspended on a `DivideDamage` split. The raw division is stashed;
    /// the re-run renormalises a malformed answer (wrong length / sum).
    DivisionAnswerPending,
    /// Suspended on a `ChooseCreatureType` whose answer feeds a one-shot
    /// resolution effect (Crippling Fear's "except chosen type" sweep)
    /// rather than a permanent stamp (`ChooseCreatureTypePending`).
    CreatureTypeAnswerPending,
    /// Suspended on a `ChooseCards` pick inside a resolving effect
    /// (fateseal, mill-then-take). Stash-and-rerun like `ModesAnswerPending`;
    /// the re-queued arm validates the ids itself, so a bad answer degrades
    /// to the legal subset. `player` is the picker (an opponent choosing
    /// what to bounce).
    CardsAnswerPending { player: usize },
    /// As `CardsAnswerPending`, but the answer is *appended* to
    /// `GameState.scratch.resolution_answer_log` so the arm can ask more questions
    /// after it (`ask_seat_cards_logged` — the pile split's pick-then-choose).
    SeatCardsAnswerPending { player: usize },
    /// A `Target` pick routed to a seat that is NOT the resolving controller,
    /// appended to `GameState.scratch.resolution_answer_log` so a loop can ask
    /// one per seat (`ask_seat_target_logged` — the council ballots, Cuombajj
    /// Witches' opponent aiming the damage).
    SeatTargetAnswerPending { player: usize },
    /// "Cast the exiled card without paying?" after a dig already mutated
    /// state (Cascade, Discover, Ripple). No re-run: yes casts `card`, no
    /// applies `decline`.
    MayCastExiledPending {
        player: usize,
        card: crate::card::CardId,
        decline: MayCastDecline,
        /// The cast is the continuation's (`Effect::CastExiledFreeAsking`),
        /// which asks the spell's target; the answer is only logged here.
        #[serde(default)]
        then_cast: bool,
    },
}

/// Where a declined [`PendingEffectState::MayCastExiledPending`] card goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MayCastDecline {
    /// Stays in exile (Possibility Storm, Amped Raptor).
    LeaveInExile,
    /// Owner's hand (Discover).
    ToHand,
    /// Bottom of the owner's library (Cascade, Ripple).
    ToBottom,
}

/// Sentinel trigger mode: the pick was deferred to resolution
/// (`pick_trigger_mode`, for a `wants_ui` controller whose modal trigger has
/// no targeting modes). Only the `Effect::ChooseMode` arm checks for it.
pub(crate) const MODE_PICK_DEFERRED: usize = usize::MAX;

// ── Events ────────────────────────────────────────────────────────────────────

/// Which face / cast path a `SpellCast` event came from, for replays and
/// spectator UIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CastFace {
    /// Default: cast from hand (or a generic alt-cost path).
    #[default]
    Front,
    /// Cast via `GameAction::CastSpellBack` (non-land MDFC back face).
    Back,
    /// Cast via `Keyword::Flashback` from the graveyard.
    Flashback,
}

/// What a [`PreventionShield`] protects (CR 615). A shield watches for a
/// damage event aimed at this object and prevents some or all of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreventionTarget {
    Player(usize),
    Permanent(CardId),
    /// A team shield with one shared pool: "prevent the next N damage that a
    /// source of your choice would deal to you and/or permanents you control"
    /// (Refraction Trap). Matches the seat and everything it controls.
    PlayerAndPermanents(usize),
    /// Every creature `.0` controls, read per damage event so later gains
    /// are covered (Surge of Salvation).
    CreaturesOf(usize),
    /// A floating shield with no recipient restriction: "the next time a source
    /// of your choice would deal damage to any target this turn, prevent that
    /// damage" (Martyr's Cause). Pair with `source` + `one_event`.
    Anything,
}

/// A cast spell's colors, card types and mana value, kept per player for the
/// turn (`Player.spell_casts_this_turn`) for "if an opponent cast a [color]
/// [type] spell this turn" riders.
///
/// Four bytes and `Copy`, so the per-turn list is a `CopyVec` and neither the
/// cast nor the `PlayerData` unshare allocates. PERF `(-150)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastProfile {
    pub colors: crate::mana::ColorSet,
    pub card_types: crate::card::CardTypeSet,
    /// The cast half's printed mana value, saturated at 255 (Rhino,
    /// Barreling Brute); fits the padding byte.
    #[serde(default)]
    pub mana_value: u8,
}
const _: () = assert!(std::mem::size_of::<CastProfile>() == 4);

impl Default for PreventionTarget {
    fn default() -> Self { PreventionTarget::Player(0) }
}

/// A continuous prevention effect (CR 615.1) as a "shield" around a player
/// or permanent, from `Effect::PreventNextDamage` /
/// `PreventAllDamageThisTurn`; cleared at cleanup (CR 514.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreventionShield {
    pub target: PreventionTarget,
    /// `None` = prevent all damage to the target this turn; `Some(n)` =
    /// prevent the next `n` damage, then the shield expires (CR 615.7).
    pub remaining: Option<u32>,
    /// CR 615.1 — when set and `target` is a player, that player gains
    /// life equal to the damage this shield prevents (Reverse Damage).
    #[serde(default)]
    pub gain_life: bool,
    /// CR 615.7 — shield only soaks damage from this source (Circle of
    /// Protection's chosen source). `None` = any source.
    #[serde(default)]
    pub source: Option<crate::card::CardId>,
    /// "The next time [it] would deal damage" — the shield soaks one whole
    /// damage event, then expires (Circle of Protection).
    #[serde(default)]
    pub one_event: bool,
    /// Deflecting Palm — damage this shield prevents is dealt to the
    /// damage source's controller.
    #[serde(default)]
    pub reflect: bool,
    /// The chosen source's controller, stamped when the reflect shield is
    /// created — the fallback when the source has left every visible zone
    /// by damage time (a resolving spell).
    #[serde(default)]
    pub source_controller: Option<usize>,
    /// Ria Ivor — the seat that mints one Phyrexian Mite per point of damage
    /// this shield prevents.
    #[serde(default)]
    pub mint_mites_for: Option<usize>,
    /// Brace for Impact — "for each 1 damage prevented this way, put a +1/+1
    /// counter on that creature." Applies to the shielded permanent itself.
    #[serde(default)]
    pub counters_on_target: bool,
    /// The kind `counters_on_target` places; `None` is +1/+1 (Sacred Boon's
    /// +0/+1).
    #[serde(default)]
    pub counter_kind: Option<crate::card::CounterType>,
    /// Kill-Suit Cultist — "destroy that creature instead": the shielded
    /// permanent is destroyed after the shield pass. Pairs with `one_event`.
    #[serde(default)]
    pub destroy: bool,
    /// CR 614.9 — Carom / Razia: damage this shield prevents is dealt to
    /// this permanent instead (a chosen redirect target, distinct from
    /// `reflect`'s "deal to the source's controller").
    #[serde(default)]
    pub redirect_to: Option<crate::card::CardId>,
    /// CR 615.7 — shield only soaks damage from sources of this color
    /// (Avacyn, Guardian Angel's chosen color). `None` = any source color.
    #[serde(default)]
    pub source_color: Option<crate::mana::Color>,
    /// The player-target sibling of `redirect_to` — the soaked damage is
    /// dealt to this seat instead (Acolyte's Reward aimed at a player).
    #[serde(default)]
    pub redirect_to_player: Option<usize>,
    /// Bone Mask — "exile cards from the top of your library equal to the
    /// damage prevented this way". The seat whose library pays the toll.
    #[serde(default)]
    pub exile_top_for: Option<usize>,
    /// Shadowbane — `gain_life` only pays out when the damage source is this
    /// colour. `None` = no colour gate. (Single-colour: the multi-colour form
    /// lives on `PreventedSource`, which isn't `Copy`.)
    #[serde(default)]
    pub gain_life_color: Option<crate::mana::Color>,
    /// Seat that gains the life when `gain_life` fires on a shield whose
    /// `target` isn't a player (Shadowbane covers creatures too).
    #[serde(default)]
    pub gain_life_to: Option<usize>,
    /// Soaks only damage from sources this seat doesn't control ("by sources
    /// you don't control" — Comeuppance). `None` = any controller.
    #[serde(default)]
    pub not_from_controller: Option<usize>,
    /// Comeuppance — what this shield soaks is dealt back by this card: to the
    /// source itself when it's a creature on the battlefield, else to the
    /// source's controller.
    #[serde(default)]
    pub reflect_by: Option<crate::card::CardId>,
    /// Channel Harm — `reflect_by`'s damage goes to this creature instead.
    #[serde(default)]
    pub reflect_to: Option<crate::card::CardId>,
    /// "Prevent all but `all_but` of that damage" (Forcefield): an
    /// unbudgeted shield that lets this much through. 0 = prevent it all.
    #[serde(default)]
    pub all_but: u32,
    /// "The next time it would deal combat damage" (Forcefield): soaks only
    /// combat damage (CR 510.2).
    #[serde(default)]
    pub combat_only: bool,
}

/// CR 615.7 — a turn-long shield around one chosen damage *source*
/// (Burrenton Forge-Tender, Hallow, Awe Strike, Protective Sphere).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PreventedSource {
    pub source: CardId,
    /// Seat that gains life equal to the damage prevented (Hallow). `None`
    /// for a plain shield.
    pub gain_life_to: Option<usize>,
    /// CR 615.8 — the shield expires after soaking one damage event
    /// (Awe Strike) rather than lasting the turn.
    pub one_instance: bool,
    /// Restricts `gain_life_to` to damage from a source of one of these
    /// colours, rechecked at damage time (Samite Ministration's black-or-red
    /// rider). Empty = no colour gate.
    #[serde(default)]
    pub gain_life_colors: Vec<crate::mana::Color>,
    /// Restricts the shield to damage dealt to this seat ("would deal damage
    /// **to you**" — New Way Forward). `None` prevents all damage the source
    /// would deal (Hallow, Burrenton Forge-Tender).
    #[serde(default)]
    pub to_player: Option<usize>,
    /// "When damage is prevented this way, [effect]" (New Way Forward). Runs
    /// for `rider_controller` with the prevented amount bound to
    /// `Value::TriggerEventAmount` and the shielded source as the trigger
    /// source.
    #[serde(default)]
    pub rider: Option<crate::effect::Effect>,
    #[serde(default)]
    pub rider_controller: usize,
}

impl PreventedSource {
    pub fn new(source: CardId) -> Self {
        Self {
            source,
            gain_life_to: None,
            one_instance: false,
            gain_life_colors: Vec::new(),
            to_player: None,
            rider: None,
            rider_controller: 0,
        }
    }
}

/// CR 614 — "until end of turn, if a player taps a land for mana, it produces
/// `output` instead of any other type and amount" (Pale Moon, Harvest Mage).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LandManaReplacement {
    /// Only this seat's land taps are replaced ("if **you** tap a land");
    /// `None` replaces every player's.
    pub who: Option<usize>,
    /// Only nonbasic lands (Pale Moon).
    pub nonbasic_only: bool,
    /// Only this one land (Quarum Trench Gnomes). `None` = every land in
    /// scope. Set together with `indefinite`, since the printed per-land
    /// wordings say "this effect lasts indefinitely".
    #[serde(default)]
    pub land: Option<crate::card::CardId>,
    /// Survives the cleanup sweep that clears the turn-scoped entries.
    #[serde(default)]
    pub indefinite: bool,
    /// What the land produces instead.
    pub output: LandManaOutput,
    /// Only lands of this type (Chaos Moon's Mountains). `None` = any.
    #[serde(default)]
    pub land_type: Option<crate::card::LandType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LandManaOutput {
    /// One colorless mana (Pale Moon).
    Colorless,
    /// One mana of a color the tapping player picks (Harvest Mage).
    ColorOfChoice,
}

/// CR 732 — the game's day/night designation. The game starts as neither
/// (`GameState.day_night == None`); once it becomes day or night it has
/// exactly one of these from then on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DayNight {
    Day,
    Night,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    StepChanged(TurnStep),
    TurnStarted { player: usize, turn: u32 },
    CardDrawn { player: usize, card_id: CardId },
    /// CR 121 — `player`'s first draw of the turn, carrying the card
    /// (God-Eternal Kefnet).
    FirstCardDrawnThisTurn { player: usize, card_id: CardId },
    /// CR 121 — the turn's `n`th card drawn by `player` (fired once per
    /// turn, when `cards_drawn_this_turn` reaches `n`; emitted for 2 and 3).
    /// Gleaming Splendor (2), Astrologian's Planisphere (3).
    NthCardDrawnThisTurn { player: usize, card_id: CardId, n: u32 },
    CardDiscarded { player: usize, card_id: CardId },
    /// A spell or ability an opponent of `player` controls caused that
    /// discard (`GameState.resolution_causer`). Spiritual Focus.
    OpponentCausedYouToDiscard { player: usize, card_id: CardId },
    /// CR 701.9 — one or more discards in a single resolution, fired once
    /// beside the per-card `CardDiscarded`s with the count as event amount
    /// (Magmakin Artillerist).
    DiscardedBatch { player: usize, count: u32 },
    /// A land entered under `player`'s control; landfall hears every entry.
    /// `played` only for a land play (CR 305.1); `from_hand` only for a play
    /// from hand (The Lost and the Damned).
    LandPlayed { player: usize, card_id: CardId, played: bool, from_hand: bool },
    /// `face` distinguishes front-face / back-face / flashback casts.
    SpellCast { player: usize, card_id: CardId, face: CastFace },
    /// CR 115.7 — a spell on the stack had its single target redirected.
    SpellTargetChanged { card_id: CardId, new_target: Target },
    /// `exhaust`: a CR 702.177 Exhaust ability (Adrenaline Jockey). `adapt`:
    /// CR 702.108 (Gyre Engineer). `tap_cost`: {T}/{Q} was in the cost —
    /// Haunting Wind keys on its absence, since the tap event covers that
    /// half. `sacrificed`: permanents were sacrificed to pay (Ashnod the
    /// Uncaring).
    AbilityActivated {
        source: CardId,
        exhaust: bool,
        adapt: bool,
        tap_cost: bool,
        sacrificed: bool,
        /// The event's amount: life paid as part of the cost (Verrak, Warped
        /// Sengir's "if life was paid to activate it … pay that much life
        /// again"), or for a loyalty ability the loyalty counters it removed
        /// (Way of the Mind Sculptor).
        paid: u32,
        /// CR 702.49a — a ninjutsu ability (Satoru Umezawa's "whenever you
        /// activate a ninjutsu ability"); `source` is the ninja.
        ninjutsu: bool,
        /// CR 605.1a — a mana ability without `{T}` (a `{T}` one emits
        /// `TappedForMana`). Only `ActivationTrait::ManaWithoutTap` hears it.
        mana: bool,
        /// CR 606.1 — a loyalty ability (Way of the Paradox's "whenever you
        /// activate a loyalty ability"); `source` is the planeswalker.
        loyalty: bool,
        /// CR 702.193 — a power-up ability (Marvel Boy, Noh-Varr).
        power_up: bool,
        /// CR 702.128 / 702.129 — an embalm or eternalize ability (Vizier of
        /// the Anointed); `source` is the card exiled to pay for it.
        embalm: bool,
        /// Mana from a Treasure paid for it (Vazi, Keen Negotiator).
        treasure: bool,
        /// CR 702.142 — a boast ability (Frenzied Raider).
        boast: bool,
    },
    /// `source` is the producing permanent when there is one, for client
    /// mana-flow visuals; `None` for rituals and decision-resolved mana.
    ManaAdded { player: usize, color: Color, source: Option<CardId> },
    ColorlessManaAdded { player: usize, source: Option<CardId> },
    PermanentEntered { card_id: CardId },
    PermanentExiled { card_id: CardId },
    /// CR 701.52a — a player rolled the d6 to visit their Attractions.
    RolledToVisitAttractions { player: usize, result: u8 },
    /// CR 718.5 — one Attraction's lit-up numbers matched the roll, so its
    /// visit ability triggers.
    AttractionVisited { card_id: CardId },
    /// CR 701.51c — `player` opened an Attraction: `card_id` entered from
    /// their Attraction deck.
    AttractionOpened { card_id: CardId, player: usize },
    /// CR 702.159b — `player` claimed the prize of the Attraction `card_id`.
    PrizeClaimed { card_id: CardId, player: usize },
    /// `KeywordAct::DidAtRandom` — `player` did something at random (Chira).
    DidAtRandom { player: usize },
    /// `KeywordAct::FacedDilemma` — `player` chose one or more modes for a
    /// spell or ability (Elim Garak).
    FacedDilemma { player: usize },
    /// A card was put into exile from the battlefield, a graveyard or a library
    /// (CR 400.7 — the origin matters to Ketramose and Laelia). `from` is one
    /// [`crate::effect::exile_from`] bit; `player` is the zone's player (the
    /// permanent's last controller, else the card's owner).
    CardExiledFrom { card_id: CardId, player: usize, from: u8 },
    /// `combat` is true for combat damage (CR 510), false for damage from
    /// spells/abilities (burn, pingers, Fight). Read by "whenever … is dealt
    /// *noncombat* damage" triggers (Chandra's Spitfire).
    DamageDealt {
        amount: u32,
        to_player: Option<usize>,
        to_card: Option<CardId>,
        combat: bool,
        /// Controller of the damage's source, if attributable.
        from_controller: Option<usize>,
        /// The damage source, if attributable (Talon of Pain's "other than
        /// this" exclusion).
        from_card: Option<CardId>,
    },
    /// Some or all of a damage event was prevented by a shield (CR 615.13).
    /// `amount` is the prevented portion.
    DamagePrevented { amount: u32, to_player: Option<usize>, to_card: Option<CardId> },
    LifeLost { player: usize, amount: u32 },
    /// `player` lost `amount` life (not through damage) during the resolution
    /// of a spell or ability `player` controlled — synthesized at the end of the
    /// outermost resolution, beside that `LifeLost` (Sahir, Visitor in Darkness).
    LifeLostToOwnEffect { player: usize, amount: u32 },
    LifeGained { player: usize, amount: u32 },
    /// `player` gained `amount` life during the resolution of instant or
    /// sorcery spell `spell` (its own gain or its lifelink damage).
    SpellCausedLifeGain { player: usize, spell: CardId, amount: u32 },
    /// CR 702.30a — `controller` paid `card_id`'s echo cost.
    EchoPaid { card_id: CardId, controller: usize },
    /// CR 123.3 — `player` put a sticker on `card_id`.
    StickerPlaced { card_id: CardId, player: usize },
    /// `count` cards were put into libraries from other zones since the last
    /// dispatch; `player` owns the first library. Synthesized at dispatch from
    /// the library piles' arrival tallies.
    CardsPutIntoLibrary { player: usize, count: u32 },
    /// A creature became tapped to pay a teamwork cost (Agent Maria Hill);
    /// emitted beside its `PermanentTapped`.
    TappedForTeamwork { card_id: CardId, controller: usize },
    /// `count` cards left `player`'s hand for exile, or were exiled from the
    /// battlefield by a spell or ability `player` controlled, since the last
    /// trigger dispatch — synthesized there from `pending_exile_tally`
    /// (Ranar, Hero of Bretagard).
    CardsExiledFromHandOrBy { player: usize, count: u32 },
    /// CR 701.12 — `card_id` fought (each fighter gets one), controlled by
    /// `controller` as the fight began.
    CreatureFought { card_id: CardId, controller: usize },
    /// A player paid life as a cost (CR 118.8 — Font of Agonies). Distinct
    /// from `LifeLost`, which also fires for damage and drains.
    PaidLife { player: usize, amount: u32 },
    /// CR 701.22/701.42 — `player` scried or surveiled (a nonzero peek).
    /// `surveil` distinguishes the two for surveil-only / scry-only payoffs;
    /// `looked_at` is the trigger amount ("for each card looked at").
    ScriedOrSurveiled { player: usize, surveil: bool, looked_at: u32 },
    /// CR 715.2e — the final chapter ability of `player`'s Saga `saga`
    /// resolved; `mana_value` is the Saga's, read as the chapter resolved.
    SagaFinalChapterResolved { player: usize, saga: CardId, mana_value: u32 },
    /// CR 701.34 — `player` proliferated (once per proliferate instance;
    /// a doubled proliferate emits two events).
    Proliferated { player: usize },
    /// CR 701.61 — `player` foraged (exiled three graveyard cards or sacrificed
    /// a Food). Powers "whenever you forage" payoffs.
    Foraged { player: usize },
    /// `player` bent (CR 701.65b / 701.66b / 701.67c / 702.189b).
    Bent { player: usize, kind: crate::effect::BendKind },
    /// CR 701.16a — `player` investigated once (`Effect::Investigate`);
    /// `first_this_turn` marks their first investigation of the turn.
    Investigated { player: usize, first_this_turn: bool },
    /// CR 702.110b — `card_id` exploited a creature (its exploit trigger
    /// sacrificed one). `controller` is the exploiter's; the exploited
    /// creature's last-known power (the event amount), Human-ness and token-ness
    /// ride along for "exploits a non-Human / nontoken creature" (Henry Wu).
    Exploited { card_id: CardId, controller: usize, power: i32, human: bool, token: bool },
    /// CR 702.154c — attacking `card_id` enlisted a creature.
    Enlisted { card_id: CardId, controller: usize },
    /// CR 702.112b — `card_id` became renowned (it wasn't before).
    BecameRenowned { card_id: CardId, controller: usize },
    /// CR 706.2 — one die of a roll came up `result` (after modifiers); one
    /// per die, beside the roll's single `DiceRolled`.
    DieResult { player: usize, result: u8 },
    /// CR 701.59 — `player` collected evidence (as a cost or via
    /// `Effect::CollectEvidence`). Powers "whenever you collect evidence"
    /// payoffs (Surveillance Monitor, Evidence Examiner).
    EvidenceCollected { player: usize },
    /// CR 702.165 — `player` gave a promised gift as a spell/ability with Gift
    /// resolved. Powers "whenever you give a gift" (Jolly Gerbils).
    GiftGiven { player: usize },
    /// CR 717.2 — the Class `source` (controlled by `player`) gained a level,
    /// reaching `level`. Powers "when this Class becomes level N" triggers
    /// (`EventKind::ClassLevelReached`, `EventScope::This`, filtered on
    /// `Predicate::SourceClassLevelIs(N)`).
    ClassLevelReached { source: CardId, player: usize, level: u8 },
    /// CR 700.14 — `player` paid a spell cost, bringing their running
    /// total of mana spent to cast spells this turn to `total`. "Whenever
    /// you expend N" triggers (`EventKind::Expend` + `Predicate::
    /// ExpendReached(n)`) fire when `total` first reaches N.
    Expended { player: usize, total: u32 },
    /// CR 122 — a player got `amount` energy counters ({E}).
    EnergyGained { player: usize, amount: u32 },
    /// CR 700.13 — `player` committed a crime (cast a spell / activated an
    /// ability targeting an opponent, something they control or own, or a
    /// spell/ability they control). Fires once per qualifying spell or ability.
    CommittedCrime { player: usize },
    /// CR 701.23 — `player` searched their own library.
    PlayerSearchedLibrary { player: usize },
    /// CR 103.2c — a spell or ability caused `player` to shuffle their library
    /// (Psychogenic Probe). Game-setup shuffles don't emit it.
    /// `cause`: the controller of the spell or ability that made `player`
    /// shuffle (`resolution_causer`), `None` outside a resolution.
    LibraryShuffled { player: usize, cause: Option<usize> },
    /// CR 605 — `card_id` was tapped to pay a mana ability's `{T}` cost
    /// (Extraplanar Lens, War's Toll). The tapped permanent is the subject.
    TappedForMana { card_id: CardId, player: usize },
    /// CR 701.38 — a player cast one vote on a ballot; `choice` is the
    /// printed word they voted for.
    Voted { player: usize, choice: String },
    /// CR 701.38 — every player has finished voting on a ballot (Grudge
    /// Keeper). Fired once per `Effect::Vote`, after the tally is known.
    VotingFinished,
    /// CR 705.1 — `player` won a coin flip (Chance Encounter, Krark).
    CoinFlipWon { player: usize },
    /// CR 701.30d — `player` won a clash.
    ClashWon { player: usize },
    /// CR 701.49 — `player` ventured into `room` of `dungeon`.
    DungeonRoomEntered { player: usize, dungeon: String, room: String },
    /// CR 701.49d — `player` completed a dungeon.
    DungeonCompleted { player: usize },
    /// CR 705.1 — `player` lost a coin flip (Karplusan Minotaur,
    /// Goblin Bomb).
    CoinFlipLost { player: usize },
    /// CR 706.6 — `player` rolled `count` dice, once per roll instruction
    /// (Barbarian Class). `high` is the greatest modified result, for
    /// "roll a 5 or higher" filters.
    DiceRolled { player: usize, count: u32, high: u8 },
    /// CR 706.2 — one die `player` rolled showed its highest *natural* face.
    /// One event per such die (Netherese Puzzle-Ward).
    RolledNaturalMax { player: usize },
    CreatureDied { card_id: CardId },
    /// CR 700.4 — any permanent went from battlefield to graveyard;
    /// synthesized per dispatch batch with its last `controller` and type
    /// flags for `EventKind::CreatureOrArtifactDied` (G'raha Tia). Creatures
    /// also emit `CreatureDied`, so listeners pick exactly one rail.
    PermanentDied { card_id: CardId, controller: usize, is_creature: bool, is_artifact: bool },
    /// A creature was sacrificed by `who` (CR 701.16), fired before its
    /// `CreatureDied` (Mortician Beetle sees it first).
    CreatureSacrificed { card_id: CardId, who: usize },
    /// Any permanent sacrificed by `who` (Korvold, Mayhem Devil). For a
    /// creature it follows `CreatureSacrificed`, so creature-only triggers
    /// don't double-fire.
    PermanentSacrificed { card_id: CardId, who: usize },
    /// CR 603.6 — a creature left the battlefield without dying; `controller`
    /// is its last controller (Dour Port-Mage).
    CreatureLeftWithoutDying { card_id: CardId, controller: usize },
    /// CR 603.10 — any permanent left the battlefield for a non-graveyard
    /// zone: the half of `EventKind::PermanentLeavesBattlefield` that
    /// `CreatureDied` doesn't cover. `controller` is its last controller; the
    /// leaver is snapshotted into `died_card_snapshots` so its own leaves
    /// trigger reads LKI.
    PermanentLeftBattlefield { card_id: CardId, controller: usize },
    PumpApplied { card_id: CardId, power: i32, toughness: i32 },
    /// `placer` is the player who put the counters (CR 122.6): the controller
    /// of the resolving spell or ability, `None` outside a resolution — read
    /// as the permanent's controller then (CR 122.6a). `EventScope::YouPutCounters`.
    CounterAdded { card_id: CardId, counter_type: CounterType, count: u32, placer: Option<usize> },
    /// CR 122.1b — keyword counters are counters: `EventKind::AnyCounterAdded`
    /// hears them, with `placer` read as for `CounterAdded`.
    KeywordCounterAdded { card_id: CardId, keyword: crate::card::Keyword, count: u32, placer: Option<usize> },
    CounterRemoved { card_id: CardId, counter_type: CounterType, count: u32 },
    /// A permanent became tapped. `actor` is the player whose effect tapped
    /// it, `None` for game-driven taps (attack, crew, mana) — Solitary
    /// Sanctuary.
    PermanentTapped {
        card_id: CardId,
        actor: Option<usize>,
        /// True when the tap was caused by declaring the permanent as an
        /// attacker (CR 508.1f). "Becomes tapped, but not declared as an
        /// attacker" triggers (Verity Circle) exclude these.
        as_attacker: bool,
    },
    PermanentUntapped { card_id: CardId },
    /// CR 702.171 — a Mount was saddled; `riders` lists the tapped creatures.
    /// Powers `EventKind::CrewsOrSaddles` (the riders' triggers).
    MountSaddled { mount: CardId, riders: Vec<CardId> },
    /// CR 702.26 — a permanent phased out (moved to `GameState.phased_out`).
    PermanentPhasedOut { card_id: CardId },
    /// CR 702.26 — a permanent phased in (returned from `phased_out` during its
    /// controller's untap step). Fires "when this phases in" triggers.
    PermanentPhasedIn { card_id: CardId },
    /// CR 702.24 — `card_id`'s cumulative upkeep went unpaid.
    CumulativeUpkeepUnpaid { card_id: CardId, player: usize },
    /// CR 800.4 / 613.1b — a permanent's controller changed. Fires "when you
    /// gain control of this permanent" triggers (Risky Move); `from` is the
    /// previous controller.
    ControlChanged { card_id: CardId, from: usize, to: usize },
    /// CR 701.44 — `card_id` explored. `explored_land` (event amount 1/0)
    /// lets Nicanzil filter land vs nonland explores.
    Explored { card_id: CardId, controller: usize, explored_land: bool },
    /// CR 701.50 — `card_id` connived (its controller drew and discarded).
    Connived { card_id: CardId, controller: usize },
    /// CR 120.10 — `card_id` (controlled by `controller`) was dealt `amount`
    /// excess noncombat damage.
    ExcessNoncombatDamage { card_id: CardId, controller: usize, amount: u32 },
    /// CR 120.10 / 120.4a — the combat half of the excess-damage pair: a
    /// creature or planeswalker was dealt combat damage beyond lethal (beyond
    /// its loyalty). `EventKind::DealtExcessDamage { noncombat_only: true }` skips it.
    ExcessCombatDamage { card_id: CardId, controller: usize, amount: u32 },
    /// CR 701.57 — `player` performed a discover for `value` (Curator of
    /// Sun's Creation's "whenever you discover" payoff reads `value` via
    /// `Value::TriggerEventAmount`).
    Discovered { player: usize, value: u32 },
    /// CR 701.37 — a permanent became monstrous; `n` is the monstrosity
    /// value (event amount — Polukranos).
    BecameMonstrous { card_id: CardId, n: u32 },
    /// CR 712 — a permanent transformed to its other face.
    Transformed { card_id: CardId },
    /// CR 710 — a flip card flipped to its flipped (bottom) face.
    Flipped { card_id: CardId },
    /// CR 702.140 — a creature mutated (a mutate spell merged onto it).
    /// `card_id` is the resulting merged permanent (the host).
    Mutated { card_id: CardId },
    /// CR 708.8 — a face-down permanent was turned face up.
    TurnedFaceUp { card_id: CardId },
    TokenCreated { card_id: CardId },
    /// A permanent left the battlefield for a hand (Azorius Aethermage).
    /// `player` is whose hand it landed in.
    PermanentReturnedToHand { card_id: CardId, player: usize },
    /// CR 701.6 — `card_id` was countered on the stack. `player` is the
    /// counterer (the controller of the countering spell or ability), which is
    /// what `EventScope::YourControl`/`OpponentControl` reads.
    SpellCountered { card_id: CardId, player: usize },
    CardMilled { player: usize, card_id: CardId },
    /// A card went from `player`'s hand to their graveyard WITHOUT being
    /// discarded (an effect moved it there). With `CardDiscarded`, the
    /// "put into your graveyard from your hand" family
    /// (`EventKind::PutIntoGraveyardFromHand`: Desert Warfare).
    CardPutIntoGraveyardFromHand { player: usize, card_id: CardId },
    /// CR 701.25a — a surveiled card went library → graveyard. Not a mill
    /// (CR 701.13): matches `EventKind::CardMilled`, not `EventKind::Milled`.
    /// A `CardPutIntoGraveyard` is also emitted.
    CardSurveiledIntoGraveyard { player: usize, card_id: CardId },
    /// A card a reveal-until put from its library into a graveyard (Old
    /// Stickfingers' creature cards). The "put into a graveyard from a
    /// library" family (`EventKind::CardMilled`), not a mill.
    CardRevealedIntoGraveyard { player: usize, card_id: CardId },
    /// CR 701.8 — a permanent was destroyed by a spell or ability. `destroyer`
    /// is that effect's controller (Karmic Justice). Emitted only from the
    /// `Effect::Destroy` funnel, so combat/SBA deaths don't match.
    PermanentDestroyedByEffect {
        card_id: CardId,
        controller: usize,
        destroyer: usize,
        is_creature: bool,
    },
    /// DSK — `player` manifested dread, putting `milled` into their graveyard
    /// this way (the non-manifested of the top two). Drives
    /// `EventKind::ManifestedDread`; the subject is `milled`.
    ManifestedDread { player: usize, milled: CardId },
    ScryPerformed { player: usize, looked_at: usize, bottomed: usize },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    /// An attacker ended declare-blockers unblocked (CR 509.3g).
    AttackerWentUnblocked { attacker: CardId },
    CombatResolved,
    FirstStrikeDamageResolved,
    TopCardRevealed { player: usize, card_name: &'static str, is_land: bool },
    AttachmentMoved { attachment: CardId, attached_to: Option<CardId> },
    /// CR 303.4 — an Aura became attached to a permanent (on resolution or via
    /// a reattach). Drives "whenever an Aura you control becomes attached …"
    /// triggers (Siona, Captain of the Pyleas).
    AuraAttached { aura: CardId, attached_to: CardId },
    /// CR 702.122 — a Vehicle was crewed and became an artifact creature
    /// until end of turn. `crew` lists the tapped crew members (their
    /// `EventKind::CrewsOrSaddles` triggers fire).
    VehicleCrewed { vehicle: CardId, crew: Vec<CardId> },
    /// DSK Eerie — `room`'s second door was just unlocked (both doors now
    /// open). `controller` is the unlocking player. Drives "whenever you fully
    /// unlock a Room" triggers.
    RoomFullyUnlocked { room: CardId, controller: usize },
    /// CR 903.9 — `owner`'s commander `card_id` was just put into the command
    /// zone, by the CR 903.9a SBA or the 903.9b replacement. Drives "whenever
    /// your commander is put into the command zone from anywhere" (Myth
    /// Unbound).
    CommanderPutIntoCommandZone { card_id: CardId, owner: usize },
    /// MKM — `case` was just solved by `controller`. Drives "whenever you solve
    /// a Case" triggers (Case File Auditor).
    CaseSolved { case: CardId, controller: usize },
    PoisonAdded { player: usize, amount: u32 },
    /// CR 726 — `player` became the monarch.
    MonarchChanged { player: usize },
    /// CR 727 — `player` took the initiative.
    InitiativeTaken { player: usize },
    /// CR 700.6 — `player` got the city's blessing (Ascend).
    CityBlessingGained { player: usize },
    /// CR 701.54 — the Ring tempted `player`; `level` is their new
    /// temptation count (1–4), `bearer` the creature they chose (if any).
    RingTempted { player: usize, level: u32, bearer: Option<CardId> },
    /// CR 732 — the game became day or night. `was_transition` is false when
    /// established from neither, so "day becomes night" triggers (CR 502.2)
    /// skip it.
    DayNightChanged { day_night: DayNight, was_transition: bool },
    LoyaltyAbilityActivated { planeswalker: CardId, loyalty_change: i32 },
    LoyaltyChanged { card_id: CardId, new_loyalty: i32 },
    PlaneswalkerDied { card_id: CardId },
    /// `count` copies of a spell put on the stack by `controller`; `copy` is
    /// the last of them, so the trigger subject is a spell the copier controls
    /// (`PlayerRef::Triggerer` reads the copier, not the original's caster).
    SpellsCopied { copy: CardId, count: u32, controller: usize },
    SurveilPerformed { player: usize, looked_at: usize, graveyarded: usize },
    /// A card left `player`'s graveyard (returned to hand, battlefield, or
    /// exiled from there). Fires per card removed. Used by Strixhaven
    /// "cards leave your graveyard" payoffs.
    CardLeftGraveyard { player: usize, card_id: CardId },
    /// A card was put into `player`'s graveyard from anywhere. `is_land` is
    /// precomputed for `EventKind::LandPutIntoGraveyard` (The Gitrog
    /// Monster).
    CardPutIntoGraveyard { player: usize, card_id: CardId, is_land: bool },
    /// A card moved from `player`'s graveyard to their hand (dredge, or any
    /// graveyard→hand return). Drives `EventKind::PutIntoHandFromGraveyard`
    /// (Golgari Brownscale's "gain 2 life when returned to hand").
    CardPutIntoHandFromGraveyard { player: usize, card_id: CardId },
    /// A permanent became a target, per target at announce time (Tenured
    /// Concocter). `caster` is the spell's / ability's controller; `by` the
    /// declaring object, for `EventSpec::causer_filter`.
    BecameTarget { target: CardId, caster: usize, by: Option<CardId> },
    /// `BecameTarget`'s player half: `player` became a target of `caster`'s
    /// spell or ability. Shares its kind bit; only
    /// `EventScope::YouOrYourPermanentTargetedByOpponent` hears it.
    PlayerBecameTarget { player: usize, caster: usize, by: Option<CardId> },
    /// CR 601.2c / 602.2b — `chooser` finished choosing targets for a single
    /// spell or ability (`object`). Fires ONCE per targeting decision, unlike
    /// the per-object `BecameTarget` (Psychic Battle).
    ChoseTargets { chooser: usize, object: CardId },
    /// CR 701.15 — a regeneration shield was applied to this permanent.
    Regenerated {
        card_id: CardId,
    },
    /// CR 702.29c — `player` cycled `card_id`, fired beside the
    /// `CardDiscarded` so cycle triggers see a distinct event.
    CardCycled {
        player: usize,
        card_id: CardId,
        /// X paid to an `{X}` in the cycling cost (Shark Typhoon's
        /// "create an X/X" cycle trigger reads it as the event amount).
        x: u32,
    },
    /// CR 104.3a — `player` conceded and left the game. Emitted before the
    /// `GameOver` that the resulting state-based-action pass produces.
    PlayerConceded { player: usize },
    /// CR 104.3 / 800.4a — `player` lost and left, for any cause but a
    /// concession (`PlayerConceded`). Emitted by the SBA sweep before any
    /// `GameOver`; in a pod it is what says who went out.
    PlayerLost { player: usize, cause: crate::player::LossCause },
    GameOver { winner: Option<usize> },
    /// CR 608.2c — a resolution's instructions are followed in order: the
    /// step before this one killed something and a later step follows, so
    /// deaths on either side were sequential, not simultaneous (CR 603.10a).
    DeathStepEnded,
    /// CR 728.1 — the game was restarted; `starter` takes the first turn.
    GameRestarted { starter: usize },
}

// ── Priority ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityState {
    pub player_with_priority: usize,
    pub consecutive_passes: usize,
}

impl PriorityState {
    pub fn new(active_player: usize) -> Self {
        Self { player_with_priority: active_player, consecutive_passes: 0 }
    }
}

// ── Stack ─────────────────────────────────────────────────────────────────────

/// An item on the stack waiting to resolve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackItem {
    /// A non-land spell (instant, sorcery, or permanent) waiting to resolve.
    Spell {
        card: Box<CardInstance>,
        caster: usize,
        target: Option<Target>,
        /// Targets for slots 1+ (`Selector::Target(n)`, `n > 0`); `target`
        /// is slot 0.
        #[serde(default)]
        additional_targets: Vec<Target>,
        /// Chosen mode index for `ChooseMode` effects (0 if `None`).
        mode: Option<usize>,
        /// X paid, read at resolution as `Value::XFromCost`.
        x_value: u32,
        /// Distinct colors of mana spent (`Value::ConvergedValue` — Prismatic
        /// Ending). Convoke pays generic only, so it doesn't raise this.
        converged_value: u32,
        /// Total mana spent on the cost, for `Value::CastSpellManaSpent` /
        /// `Predicate::ManaSpentAtLeast` (SOS Increment / Opus).
        #[serde(default)]
        mana_spent: u32,
        /// Can't be countered (Cavern of Souls); `Effect::CounterSpell` skips it.
        uncounterable: bool,
    },
    /// A triggered/loyalty ability waiting to resolve.
    Trigger {
        source: CardId,
        controller: usize,
        effect: Box<Effect>,
        target: Option<Target>,
        mode: Option<usize>,
        /// The originating spell's X for `Value::XFromCost` (an ETB inherits
        /// it; other triggers are 0).
        #[serde(default)]
        x_value: u32,
        /// Converge value, as `x_value` (Rancorous Archaic).
        #[serde(default)]
        converged_value: u32,
        /// The trigger's subject (the cast spell for a `SpellCast` trigger,
        /// the entrant for an ETB) — distinct from `source`, the ability's
        /// owner. `None` falls back to `source`.
        #[serde(default)]
        trigger_source: Option<crate::game::effects::EntityRef>,
        /// The originating spell's mana spent, as `x_value` (Increment /
        /// Opus payoffs).
        #[serde(default)]
        mana_spent: u32,
        /// The fired event's amount (life gained, damage, …), set by
        /// `dispatch_triggers_for_events` for `Value::TriggerEventAmount`
        /// (Light of Promise).
        #[serde(default)]
        event_amount: u32,
        /// CR 603.4 — intervening 'if' clause, re-checked at resolution; if
        /// false the trigger leaves the stack doing nothing. Set by
        /// `fire_step_triggers` when `EventSpec.filter` is a resolve-time
        /// gate (Triskaidekaphile); `None` for trigger-only gates.
        #[serde(default)]
        intervening_if: Option<crate::card::Predicate>,
        /// Targets for slots 1+ beyond `target` (Autumn-Tail).
        #[serde(default)]
        additional_targets: Vec<Target>,
        /// An activated (CR 602), not triggered, ability — always a copyable
        /// nonmana activation, since mana abilities never reach the stack
        /// (Illusionist's Bracers).
        #[serde(default)]
        activated: bool,
        /// The player the firing event named (the damaged seat), seeding
        /// `trigger_event_player_scratch` so `ControlledByTriggerPlayer`
        /// filters re-check against it (CR 608.2b).
        #[serde(default)]
        trigger_player: Option<usize>,
        /// Per-colour breakdown of the mana spent paying this *activation's*
        /// cost (CR 106.6 — Protective Sphere's "shares a color with the mana
        /// spent on this activation cost"). Empty for triggered abilities.
        #[serde(default)]
        mana_spent_by_color: Vec<(crate::mana::Color, u32)>,
        /// CR 701.27g — set by `transform_permanent` on stack items whose
        /// source just transformed, so "transform it only if it hasn't
        /// transformed since" holds; read into `resolving_source_transformed`.
        #[serde(default)]
        source_transformed_since_push: bool,
        /// CR 113.7 / 115.1 — this ability's own identity as an object on the
        /// stack, so "target activated or triggered ability" names *it* rather
        /// than its source (two abilities of one source are two objects).
        /// Stamped by `GameState::push_stack` from a separate id space
        /// ([`is_stack_ability_id`]); a copy is stamped afresh. 0 = unstamped.
        #[serde(default)]
        ability_id: u32,
        /// CR 608.2b — bit `i` set when slot `i` was a battlefield permanent
        /// at push (`GameState::push_stack`), so resolution can tell "left its
        /// zone" from a graveyard target. Slots past 7 are unchecked.
        #[serde(default)]
        bf_slots: u8,
        /// CR 400.7 — `trigger_source` was in a graveyard at push. If it left
        /// by resolution it is a new object and "it" finds nothing (Tinybones).
        #[serde(default)]
        ts_in_gy: bool,
    },
}

/// First id of the stack-ability id space: above every card id the game
/// mints (`next_id` counts up from 1), below the emblem sources that count
/// down from `u32::MAX`.
pub const ABILITY_ID_BASE: u32 = 0x8000_0000;

/// True when `id` names an ability on the stack (`StackItem::Trigger::ability_id`)
/// rather than a card.
#[inline]
pub fn is_stack_ability_id(id: CardId) -> bool {
    (ABILITY_ID_BASE..0xFFFF_0000).contains(&id.0)
}

/// Builder for `StackItem::Trigger`: defaults every rider field, so push
/// sites chain only what they have (hand-written literals drifted — audit P3).
#[derive(Debug, Clone)]
pub struct TriggerPush {
    source: CardId,
    controller: usize,
    effect: Effect,
    target: Option<Target>,
    additional_targets: Vec<Target>,
    mode: Option<usize>,
    x_value: u32,
    converged_value: u32,
    trigger_source: Option<crate::game::effects::EntityRef>,
    mana_spent: u32,
    event_amount: u32,
    intervening_if: Option<crate::card::Predicate>,
    activated: bool,
    trigger_player: Option<usize>,
    mana_spent_by_color: Vec<(crate::mana::Color, u32)>,
}

impl TriggerPush {
    pub fn new(source: CardId, controller: usize, effect: Effect) -> Self {
        Self {
            source,
            controller,
            effect,
            target: None,
            additional_targets: Vec::new(),
            mode: None,
            x_value: 0,
            converged_value: 0,
            trigger_source: None,
            mana_spent: 0,
            event_amount: 0,
            intervening_if: None,
            activated: false,
            trigger_player: None,
            mana_spent_by_color: Vec::new(),
        }
    }
    /// The player the firing event named (the damaged seat for a combat-damage
    /// trigger), read back by `ControlledByTriggerPlayer` at resolution.
    pub fn trigger_player(mut self, p: Option<usize>) -> Self {
        self.trigger_player = p;
        self
    }
    /// Mark this item as an activated ability (CR 602) rather than a trigger.
    pub fn activated(mut self, v: bool) -> Self {
        self.activated = v;
        self
    }
    pub fn target(mut self, t: Option<Target>) -> Self {
        self.target = t;
        self
    }
    pub fn additional_targets(mut self, t: Vec<Target>) -> Self {
        self.additional_targets = t;
        self
    }
    pub fn mode(mut self, m: Option<usize>) -> Self {
        self.mode = m;
        self
    }
    pub fn x_value(mut self, x: u32) -> Self {
        self.x_value = x;
        self
    }
    pub fn converged_value(mut self, v: u32) -> Self {
        self.converged_value = v;
        self
    }
    pub fn trigger_source(mut self, ts: Option<crate::game::effects::EntityRef>) -> Self {
        self.trigger_source = ts;
        self
    }
    pub fn mana_spent(mut self, m: u32) -> Self {
        self.mana_spent = m;
        self
    }
    pub fn event_amount(mut self, a: u32) -> Self {
        self.event_amount = a;
        self
    }
    pub fn intervening_if(mut self, p: Option<crate::card::Predicate>) -> Self {
        self.intervening_if = p;
        self
    }
    /// Per-colour mana spent paying an activation's cost (Protective Sphere).
    pub fn mana_spent_by_color(mut self, m: Vec<(crate::mana::Color, u32)>) -> Self {
        self.mana_spent_by_color = m;
        self
    }
    pub fn build(mut self) -> StackItem {
        crate::game::target_hole::pad_empty_slot_zero(
            &self.effect,
            self.mode,
            &mut self.target,
            &self.additional_targets,
        );
        StackItem::Trigger {
            source: self.source,
            controller: self.controller,
            effect: Box::new(self.effect),
            target: self.target,
            mode: self.mode,
            x_value: self.x_value,
            converged_value: self.converged_value,
            trigger_source: self.trigger_source,
            mana_spent: self.mana_spent,
            event_amount: self.event_amount,
            intervening_if: self.intervening_if,
            additional_targets: self.additional_targets,
            activated: self.activated,
            trigger_player: self.trigger_player,
            mana_spent_by_color: self.mana_spent_by_color,
            // CR 701.27g — false at push by definition; `transform_permanent`
            // sets it on items already on the stack.
            source_transformed_since_push: false,
            ability_id: 0,
            bf_slots: 0,
            ts_in_gy: false,
        }
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    #[error("It is not your priority to act")]
    NotYourPriority,
    /// CR 300.2a — a land card (even one that's also an artifact) is played,
    /// not cast.
    #[error("A land can only be played, not cast")]
    CannotCastLand,
    #[error("Sorcery-speed only: stack must be empty and it must be your main phase")]
    SorcerySpeedOnly,
    #[error("You can't cast noncreature spells this turn")]
    CantCastNoncreature,
    /// CR 601.3e — the card has no mana cost; it can only be cast via an
    /// alternative cost or without paying (suspend, Living End cascade).
    #[error("card has no mana cost — it can't be cast this way")]
    NoManaCost,
    #[error("You can't cast permanent spells")]
    CantCastPermanentSpells,
    #[error("You can't cast spells or activate loyalty abilities this turn")]
    SilencedThisTurn,
    #[error("You can't cast spells — an epic spell you control has resolved")]
    EpicLocked,
    /// CR 702.61 — a spell with split second is on the stack.
    #[error("A spell with split second is on the stack — only mana abilities and special actions are allowed")]
    SplitSecondLock,
    #[error("A permanent names this spell — it can't be cast")]
    SpellNameLocked,
    /// A one-spell-per-turn lock (Rule of Law, CR 601) bars this cast.
    #[error("A one-spell-per-turn effect bars casting more spells this turn")]
    SpellLimitReached,
    /// CR 801 — the object or player is outside the actor's range of influence.
    #[error("Out of your range of influence")]
    OutOfRange,
    /// CR 733.3 — this free activation has been repeated without moving the
    /// game state; the loop must be broken by a different game choice.
    #[error("This would repeat a loop — make a different choice (CR 733.3)")]
    LoopMustBeBroken,
    #[error("Card {0:?} not found in hand")]
    CardNotInHand(CardId),
    #[error("Card {0:?} not found in graveyard")]
    CardNotInGraveyard(CardId),
    #[error("Card {0:?} not found on battlefield")]
    CardNotOnBattlefield(CardId),
    #[error("Card {0:?} is not a land")]
    NotALand(CardId),
    /// SOS Prepare — `CastPrepareSpell` on a creature that isn't a
    /// prepared preparation card under the caster's control.
    #[error("Creature {0:?} isn't a prepared creature you control")]
    NotPrepared(CardId),
    #[error("Already played a land this turn")]
    AlreadyPlayedLand,
    #[error("Card {0:?} is tapped")]
    CardIsTapped(CardId),
    #[error("Creature {0:?} has summoning sickness")]
    SummoningSickness(CardId),
    #[error("Creature {0:?} cannot block (tapped, not a creature, or flying restriction)")]
    CannotBlock(CardId),
    #[error("Creature {0:?} cannot attack (Defender / can't-attack restriction)")]
    CannotAttack(CardId),
    #[error("Attacker {0:?} has Menace and must be blocked by two or more creatures")]
    MenaceRequiresTwoBlockers(CardId),
    #[error("Attacker {0:?} can't be blocked by more than one creature")]
    CannotBeBlockedByMoreThanOne(CardId),
    #[error("Attacker {0:?} must be blocked if able and an idle blocker can block it")]
    MustBeBlockedIfAble(CardId),
    #[error("Card {0:?} has Hexproof and cannot be targeted by opponents")]
    TargetHasHexproof(CardId),
    #[error("Card {0:?} has Shroud and cannot be targeted")]
    TargetHasShroud(CardId),
    #[error("Card {0:?} has protection from that color/quality")]
    TargetHasProtection(CardId),
    #[error("Mana: {0}")]
    Mana(#[from] ManaError),
    /// A UI payer has more than one way to tap for this cost, so they tap
    /// sources themselves and re-submit. Bots and engine auto-pays still
    /// auto-tap.
    #[error("Tap mana to pay {cost} — you have more than one way to pay, so choose your sources")]
    ManualTapRequired { cost: String },
    #[error("Wrong step for this action (currently {actual:?})")]
    WrongStep { actual: TurnStep },
    #[error("This action requires a target")]
    TargetRequired,
    #[error("Invalid target")]
    InvalidTarget,
    #[error("The same object can't be chosen for two targets (CR 115.3)")]
    DuplicateTarget,
    #[error("Ability index out of bounds")]
    AbilityIndexOutOfBounds,
    #[error("Activated ability already used this turn (once-per-turn)")]
    AbilityAlreadyUsedThisTurn,
    #[error("Activated ability's `activate only if` condition is not met")]
    AbilityConditionNotMet,
    #[error("Activated abilities of this source are shut off (Pithing Needle / Phyrexian Revoker)")]
    AbilitySuppressedByNamedCard,
    #[error("Target does not meet the selection requirement for this effect")]
    SelectionRequirementViolated,
    #[error("The game is already over")]
    GameAlreadyOver,
    #[error("Cannot pass priority while blockers must be declared")]
    MustDeclareBlockers,
    #[error("Player {0}'s library is empty")]
    LibraryEmpty(usize),
    #[error("Stack is empty — no spell to counter")]
    StackEmpty,
    #[error("Mode index {0} out of bounds for ChooseOne effect")]
    ModeOutOfBounds(usize),
    #[error("No decision is currently pending")]
    NoDecisionPending,
    #[error("Cannot perform this action while a decision is pending")]
    DecisionPending,
    #[error("Submitted decision answer does not match the pending decision kind")]
    DecisionAnswerMismatch,
    #[error("Card has no alternative (pitch) cost")]
    NoAlternativeCost,
    #[error("Pitch card {0:?} is missing from hand or doesn't match the alternative cost's filter")]
    InvalidPitchCard(CardId),
    #[error("Cannot attack player {0} (active player, eliminated, or out of range)")]
    InvalidAttackTarget(usize),
    #[error("Planeswalker {0:?} is not a valid attack target")]
    InvalidPlaneswalkerAttackTarget(CardId),
    #[error("Blocker {blocker:?} cannot block an attacker targeting a different player")]
    BlockerWrongDefender { blocker: CardId },
    #[error("Planeswalker {0:?} has already used a loyalty ability this turn")]
    LoyaltyAbilityAlreadyUsed(CardId),
    #[error("Not enough loyalty on {0:?} to pay this ability's cost")]
    NotEnoughLoyalty(CardId),
    #[error("Cannot pay this ability's life cost (would lose at or below 0 life)")]
    InsufficientLife,
    #[error("Flashback-tap cast invalid (missing keyword, wrong tap count, or invalid creature)")]
    FlashbackTapInvalid,
    #[error("Card {0:?} has Ward — cannot pay the ward cost")]
    TargetHasWard(CardId),
    #[error("Card {0:?} is not an Equipment you can equip")]
    NotEquipment(CardId),
    #[error("Not enough energy to pay this ability's {{E}} cost")]
    InsufficientEnergy,
    #[error("This split card has no Fuse, so its halves can't be cast together")]
    NotFusable,
}
