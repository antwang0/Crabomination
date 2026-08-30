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

/// Memo states, shared by both zones' lanes.
const UNKNOWN: u8 = 0;
const ABSENT: u8 = 1;
const PRESENT: u8 = 2;

/// Two-bit lanes on [`Graveyard`], packed exactly as [`Battlefield`]'s are so
/// one store on the write path clears every lane. A `u8` takes four.
const GY_LANE_ANTHEM: u32 = 0;
const GY_LANE_ACT_GRANT: u32 = 2;

/// A player's graveyard: the CoW card list, plus the whole-zone questions hot
/// walkers ask of it — "does any card here carry a
/// [`StaticEffect::GraveyardAnthem`](crate::effect::StaticEffect::GraveyardAnthem)?"
/// (the Incarnation cycle's "as long as this card is in your graveyard and you
/// control a [Land subtype] …") and the `GrantActivatedAbilityFromGraveyard`
/// twin `grant_scan` asks (Riftstone Portal). One lane per question; see the
/// `GY_LANE_*` constants.
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
    /// Two-bit lanes; see the `GY_LANE_*` constants.
    lanes: AtomicU8,
}

/// Does this card carry a `GraveyardAnthem` static? The per-card half of
/// [`Graveyard::has_anthem`].
fn card_has_anthem(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::GraveyardAnthem { .. }))
}

/// Does this card grant an activated ability from the graveyard (Riftstone
/// Portal)? The [`GY_LANE_ACT_GRANT`] predicate — the same bare match
/// `grant_scan`'s walk makes, so the lane is exact rather than an
/// over-approximation.
fn card_has_gy_activated_grant(c: &CardInstance) -> bool {
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            crate::effect::StaticEffect::GrantActivatedAbilityFromGraveyard { .. }
        )
    })
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
        self.lane(GY_LANE_ANTHEM, card_has_anthem, "anthem")
    }

    /// CR 611 — true when some card here grants an activated ability from the
    /// graveyard (Riftstone Portal), the leg `grant_scan` walks both
    /// graveyards for on every call. Memoized on the same word.
    pub fn has_activated_grant(&self) -> bool {
        self.lane(GY_LANE_ACT_GRANT, card_has_gy_activated_grant, "activated-grant")
    }

    /// One lane's answer: a word load and two mask tests on a hit, the zone
    /// walk plus one store on a miss. `what` names the lane in the audit.
    #[inline]
    fn lane(&self, shift: u32, walk: fn(&CardInstance) -> bool, what: &str) -> bool {
        let cur = (self.lanes.load(Ordering::Relaxed) >> shift) & 0b11;
        debug_assert!(
            cur == UNKNOWN || (cur == PRESENT) == self.cards.iter().any(walk),
            "graveyard {what} memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            ABSENT => false,
            PRESENT => true,
            _ => {
                let found = self.cards.iter().any(walk);
                let word = self.lanes.load(Ordering::Relaxed);
                let bits = if found { PRESENT } else { ABSENT };
                self.lanes.store((word & !(0b11 << shift)) | (bits << shift), Ordering::Relaxed);
                found
            }
        }
    }

    /// Append, through [`CowBox`]'s own `push` so an unshare materializes
    /// with room for the card. Inherent, so it shadows the `Deref`'d
    /// `Vec::push`; the memo is invalidated exactly as `DerefMut` would.
    pub fn push(&mut self, card: CardInstance) {
        self.lanes.store(0, Ordering::Relaxed);
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
    fn memo_lane(&self, shift: u32) -> Option<bool> {
        match (self.lanes.load(Ordering::Relaxed) >> shift) & 0b11 {
            ABSENT => Some(false),
            PRESENT => Some(true),
            _ => None,
        }
    }

    /// The anthem lane, for the tests that predate the second one.
    #[cfg(test)]
    fn memo(&self) -> Option<bool> {
        self.memo_lane(GY_LANE_ANTHEM)
    }
}

impl Clone for Graveyard {
    /// The cards clone as a `CowBox` (a refcount bump); the memo describes
    /// those same cards, so it comes along.
    fn clone(&self) -> Self {
        Self {
            cards: self.cards.clone(),
            lanes: AtomicU8::new(self.lanes.load(Ordering::Relaxed)),
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
        self.lanes.store(0, Ordering::Relaxed);
        &mut self.cards
    }
}

impl From<Vec<CardInstance>> for Graveyard {
    fn from(cards: Vec<CardInstance>) -> Self {
        Self { cards: cards.into(), lanes: AtomicU8::new(0) }
    }
}

impl From<CowBox<Vec<CardInstance>>> for Graveyard {
    fn from(cards: CowBox<Vec<CardInstance>>) -> Self {
        Self { cards, lanes: AtomicU8::new(0) }
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
/// `PRESENT` triple the graveyard's memo uses, and a `u32` word takes
/// sixteen of them.
///
/// **A lane has exactly one predicate**, however many callers ask through it
/// ([`LANE_SHIELD`] has nine). Nothing enforces that structurally; what
/// catches a second predicate on a lane is [`Battlefield::lane`]'s
/// `debug_assert!`, which recomputes with the predicate it was *handed* and
/// compares against the stored bit.
const LANE_LAND: u32 = 0;
const LANE_CREATURE: u32 = 2;
const LANE_DAMAGE_SCALE: u32 = 4;
const LANE_OUTGOING_PREVENT: u32 = 6;
const LANE_INCOMING_PREVENT: u32 = 8;
const LANE_DISPATCH: u32 = 10;
const LANE_SHIELD: u32 = 12;
const LANE_GRANT: u32 = 14;
const LANE_LISTENER: u32 = 16;
const LANE_ACT_GRANT: u32 = 18;
/// Not a presence lane: `PRESENT` here means "the member list beside it is
/// computed", and the list is what the caller reads. It shares the word so
/// that the one store every write path already makes clears it too.
const LANE_TRIGGERER: u32 = 20;
const LANE_MASK: u32 = 0b11;

/// Does this permanent contribute anything to
/// [`GameState::dispatch_board_scan`](crate::game::GameState::dispatch_board_scan)?
/// The [`LANE_DISPATCH`] predicate, and the one the lane's `debug_assert!`
/// audits against — so the memo and the walk it gates cannot drift.
fn card_has_dispatch_bits(c: &CardInstance) -> bool {
    c.dispatch_scan_bits() & crate::card::dispatch_bits::BOARD_SCAN != 0
}

/// Can this permanent grant a keyword to anything, whatever the predicate
/// asks? The [`LANE_GRANT`] predicate, and the one that lane's
/// `debug_assert!` audits against.
fn card_has_any_grant_bits(c: &CardInstance) -> bool {
    c.grant_scan_bits() & crate::card::grant_bits::ANY_GRANT != 0
}

/// Does this permanent carry a `YourControl` / `AnyPlayer` printed trigger —
/// the two scopes the combat-damage dispatch's listener walk looks for? The
/// [`LANE_LISTENER`] predicate.
fn card_has_listener_bits(c: &CardInstance) -> bool {
    c.dispatch_scan_bits() & crate::card::dispatch_bits::LISTENER != 0
}

/// Does this permanent carry a printed `GrantActivatedAbility` static, under
/// whatever CR 611.2 wrappers? The [`LANE_ACT_GRANT`] predicate — an
/// over-approximation of `grant_scan`'s walk, which also *evaluates* those
/// wrappers, and that is the sound direction.
fn card_has_activated_grant(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| crate::effect::static_effect_grants_activated(&sa.effect))
}

/// Can this permanent contribute a trigger of its **own** to
/// [`GameState::dispatch_triggers_for_events`](crate::game::GameState::dispatch_triggers_for_events)
/// — a printed triggered ability or a CR 721.2a Station band? The
/// [`LANE_TRIGGERER`] predicate, and the dispatcher's fast-path `continue`
/// calls it rather than spelling it a second time: a member list and the gate
/// it stands in for have to be the same question or the list is unsound.
pub(crate) fn card_is_triggerer(c: &CardInstance) -> bool {
    !c.definition.triggered_abilities.is_empty() || !c.definition.station.is_empty()
}

/// [`card_is_triggerer`] over a whole board as an index mask — the member
/// list's audit, recomputed by [`Battlefield::trigger_members`]'
/// `debug_assert!`. `0` past 64 cards, which is where the list stops being
/// representable and so is never stored.
fn triggerer_bits(cards: &[CardInstance]) -> u64 {
    if cards.len() > 64 {
        return 0;
    }
    let mut bits = 0u64;
    for (i, c) in cards.iter().enumerate() {
        if card_is_triggerer(c) {
            bits |= 1u64 << i;
        }
    }
    bits
}

/// `CRAB_TRIG_CENSUS` — the trigger dispatcher's member-list census.
///
/// PERF `(-115)`: the lane replaces the dispatch walk's per-card gate with a
/// list of the indices that can pass it, so what it is worth is
/// `hits x (board - members)` visits against the walks the misses still make.
/// Three counts answer that and nothing else does — the lane's read and its
/// fill both inline away, so no callgrind row carries them.
///
/// Off unless the variable is set; one `OnceLock`-backed load when it is.
#[cfg(feature = "trig-census")]
pub mod trig_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// Dispatches that asked the lane (i.e. reached it with no grant live).
    pub static ASKS: AtomicU64 = AtomicU64::new(0);
    /// …of which the lane answered.
    pub static HITS: AtomicU64 = AtomicU64::new(0);
    /// Permanents on the board over those asks, and members over the hits —
    /// the two sides of what a hit skips.
    pub static BOARD: AtomicU64 = AtomicU64::new(0);
    pub static MEMBERS: AtomicU64 = AtomicU64::new(0);
    /// Dispatches that walked the board with a grant live, where the lane is
    /// no use and the walk is the whole cost, and the permanents they walked.
    pub static GRANTED: AtomicU64 = AtomicU64::new(0);
    pub static GRANT_BOARD: AtomicU64 = AtomicU64::new(0);
    /// Loop bodies the walk actually runs — `popcount` on a hit, the whole
    /// board otherwise. The denominator every per-visit Ir figure needs, and
    /// it is free: both numbers are known before the walk starts.
    pub static VISITS: AtomicU64 = AtomicU64::new(0);

    /// `0` not yet read, `1` off, `2` on. **Not a `OnceLock`**: the gate is
    /// asked once per dispatch and `get_or_init` there is ~20 Ir apiece. One
    /// relaxed byte and a compare is ~2, and the store is idempotent, so the
    /// race two threads can run here has one outcome.
    static ON: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

    #[inline]
    pub fn on() -> bool {
        match ON.load(Relaxed) {
            1 => false,
            2 => true,
            _ => init(),
        }
    }

    #[cold]
    #[inline(never)]
    fn init() -> bool {
        let v = match std::env::var("CRAB_TRIG_CENSUS") {
            Ok(v) => !v.is_empty() && v != "0",
            _ => false,
        };
        ON.store(if v { 2 } else { 1 }, Relaxed);
        v
    }

    /// One dispatch: `members` is `Some(count)` on a hit.
    #[cold]
    #[inline(never)]
    pub fn tick(no_grants: bool, board: usize, members: Option<u32>) {
        if !no_grants {
            GRANTED.fetch_add(1, Relaxed);
            GRANT_BOARD.fetch_add(board as u64, Relaxed);
            VISITS.fetch_add(board as u64, Relaxed);
            return;
        }
        ASKS.fetch_add(1, Relaxed);
        BOARD.fetch_add(board as u64, Relaxed);
        match members {
            Some(m) => {
                HITS.fetch_add(1, Relaxed);
                MEMBERS.fetch_add(m as u64, Relaxed);
                VISITS.fetch_add(m as u64, Relaxed);
            }
            None => {
                VISITS.fetch_add(board as u64, Relaxed);
            }
        }
    }

    /// `(asks, hits, board, members, granted, grant_board, visits)`.
    pub fn snapshot() -> [u64; 7] {
        [
            ASKS.load(Relaxed),
            HITS.load(Relaxed),
            BOARD.load(Relaxed),
            MEMBERS.load(Relaxed),
            GRANTED.load(Relaxed),
            GRANT_BOARD.load(Relaxed),
            VISITS.load(Relaxed),
        ]
    }
}

/// The battlefield: the CoW card list, plus the whole-board questions asked of
/// it often enough to want a memo — the two layer-4 type gates ("can any
/// permanent here contribute an `AddLandType` / `SetLandTypes` /
/// `ReplaceBasicLandType` to the gathered set", and the same for the
/// creature-type family), the three damage-scaling / prevention gates, the
/// whole-board damage-shield family, and the trigger dispatcher's board scan.
/// One lane per question; see the `LANE_*` constants.
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
/// **The invalidation is two disjoint questions, and each is answered where it
/// is cheap.** Every lane is a function of exactly one thing: the multiset of
/// *definitions* on the battlefield (`card_can_change_*_types` is written to
/// read no instance field, deliberately — see its doc comment;
/// `dispatch_scan_bits` is a pure function of the definition by construction).
///
/// * *Membership* — a permanent entering or leaving — can only happen through
///   [`DerefMut`] or the inherent [`push`](Self::push), and both clear the
///   lanes. There is no third route: `Vec`'s own mutators are reached through
///   `DerefMut`.
/// * *A definition rewrite on a permanent already here* is reached through an
///   element handle, two derefs below anything this type can observe. It is
///   caught by stamping each computed lane with
///   [`definition_epoch`](crate::card::definition_epoch), which the two
///   accessors that can rewrite a definition bump.
///
/// So [`iter_mut`](Self::iter_mut), [`get_mut`](Self::get_mut) and the `&mut`
/// [`IntoIterator`] — the tap, damage, counter and untap paths, 139,280
/// invalidations a `cube` run before this split — leave the lanes alone.
#[derive(Debug, Default)]
pub struct Battlefield {
    cards: CowBox<Vec<CardInstance>>,
    /// Two-bit lanes; see the `LANE_*` constants.
    type_gates: std::sync::atomic::AtomicU32,
    /// The `definition_epoch` the lanes were computed at. A lane is valid only
    /// while this still matches.
    def_epoch: std::sync::atomic::AtomicU64,
    /// The index [`find_by_id`](Self::find_by_id) last returned from. Needs no
    /// invalidation at all — see that function.
    find_hint: std::sync::atomic::AtomicU32,
    /// Which battlefield *indices* satisfy [`card_is_triggerer`], one bit
    /// each. Read only while [`LANE_TRIGGERER`] says the list is computed, so
    /// it inherits the lanes' invalidation whole and needs none of its own.
    trig_members: std::sync::atomic::AtomicU64,
}

impl Battlefield {
    /// The permanent with `id`, through a one-entry index memo.
    ///
    /// `GameState::battlefield_find` is 556 always-inlined call sites and
    /// **4.03 % of the program** when it was first read (PERF `(-38)`;
    /// 2.35 % at the last `cg_sites` read, after four of its six named sites
    /// were paid off with `_on` forms). The remaining ones are the callers
    /// that ask about the *same* permanent several times running — the
    /// combat resolver's per-pair freeze scope asks three or four times for
    /// the blocker alone, once per prevention gate — and a one-entry memo is
    /// what that shape wants.
    ///
    /// **The memo needs no invalidation, because the answer is verified
    /// rather than trusted.** `cards[hint].id == id` is the whole contract: a
    /// stale index, a shorter board, a different permanent in that slot all
    /// fail the same test and fall through to the scan. That is why this sits
    /// outside the lanes' epoch/clear protocol — nothing a write can do to
    /// this zone can make a *hit* wrong.
    ///
    /// The one premise a hit rests on that a scan does not is that **ids on
    /// the battlefield are unique** — a scan stops at the first match, a hint
    /// can name a later one. That is a global engine invariant (`CardId`s come
    /// off a monotonic counter), and the `debug_assert!` on the scan path is
    /// its audit: it costs the tail of a walk the miss was making anyway, and
    /// `scripts/robustness_grid.sh` is what fires it. Auditing it on the *hit*
    /// path instead would put a board walk in front of the memo it is there to
    /// avoid.
    ///
    /// Break-even is a repeat rate of about a tenth: a hit is a relaxed load,
    /// a bounds check and one `CardId` compare against the ~58 Ir a scan of
    /// ~23 `Arc`-boxed permanents costs (PERF's "price a `find` at its
    /// expected stopping point"), and a miss adds those same few instructions
    /// plus one store on top of the scan it was going to make anyway.
    #[inline]
    pub fn find_by_id(&self, id: crate::card::CardId) -> Option<&CardInstance> {
        let hint = self.find_hint.load(Ordering::Relaxed) as usize;
        if let Some(c) = self.cards.get(hint)
            && c.id == id
        {
            return Some(c);
        }
        let (i, c) = self.cards.iter().enumerate().find(|(_, c)| c.id == id)?;
        debug_assert!(
            !self.cards[i + 1..].iter().any(|o| o.id == id),
            "two battlefield permanents share a CardId: the find hint could name either",
        );
        self.find_hint.store(i as u32, Ordering::Relaxed);
        Some(c)
    }

    /// The `&mut` twin of [`find_by_id`](Self::find_by_id), and the reason it
    /// is not spelled `iter_mut().find(…)`: that form takes `&mut` at the
    /// cards **before** it knows whether the id is here, so it pays a
    /// [`CowBox`](crate::cow::CowBox) unshare — a deep copy of the whole zone
    /// — even to answer `None`. Locating on the shared side and reaching for
    /// `&mut` only on a hit costs a miss nothing, and puts the one-entry memo
    /// on the write path that the read path has had since PERF `(-38)`.
    ///
    /// Same uniqueness premise as `find_by_id`, audited by the same
    /// `debug_assert!` on the scan path.
    #[inline]
    pub fn find_by_id_mut(&mut self, id: crate::card::CardId) -> Option<&mut CardInstance> {
        let hint = self.find_hint.load(Ordering::Relaxed) as usize;
        let i = if self.cards.get(hint).is_some_and(|c| c.id == id) {
            hint
        } else {
            let i = self.cards.iter().position(|c| c.id == id)?;
            debug_assert!(
                !self.cards[i + 1..].iter().any(|o| o.id == id),
                "two battlefield permanents share a CardId: the find hint could name either",
            );
            self.find_hint.store(i as u32, Ordering::Relaxed);
            i
        };
        self.cards_unchecked_mut().get_mut(i)
    }

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

    /// CR 614.2/614.5 — can any permanent here scale a damage event
    /// (`GameState::damage_scaling_in_scope`'s board half)?
    #[inline]
    pub fn has_damage_scaler(&self, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        self.lane(LANE_DAMAGE_SCALE, walk)
    }

    /// CR 615 — can any permanent here prevent damage a *source* would deal
    /// (`GameState::combat_damage_prevented_from`'s board half)?
    #[inline]
    pub fn has_outgoing_damage_prevention(
        &self,
        walk: impl Fn(&CardInstance) -> bool + Copy,
    ) -> bool {
        self.lane(LANE_OUTGOING_PREVENT, walk)
    }

    /// CR 615 — the incoming twin of
    /// [`has_outgoing_damage_prevention`](Self::has_outgoing_damage_prevention)
    /// (`GameState::combat_damage_prevented_to_self`'s board half).
    #[inline]
    pub fn has_incoming_damage_prevention(
        &self,
        walk: impl Fn(&CardInstance) -> bool + Copy,
    ) -> bool {
        self.lane(LANE_INCOMING_PREVENT, walk)
    }

    /// CR 615 — can any permanent here carry one of the whole-board damage
    /// shields? The board half of nine `GameState` walks — Iroas, Glacial
    /// Chasm, Mark of Asylum, The Wanderer, Emmara Tandris, Rune-Tail's
    /// Essence, Well-Laid Plans, Light of Sanction, Indentured Oaf — each of
    /// which walks every permanent's `static_abilities` for one variant and
    /// finds nothing on almost every board. The predicate is their *union*:
    /// each walk's own filter is an instance field applied after the static is
    /// found, so the definition question they share is the whole walk.
    #[inline]
    pub fn has_damage_shield(&self, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        self.lane(LANE_SHIELD, walk)
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
        let stale = self.def_epoch.load(Ordering::Relaxed)
            != crate::card::definition_epoch();
        let cur = if stale {
            UNKNOWN as u32
        } else {
            (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK
        };
        debug_assert!(
            cur == UNKNOWN as u32 || (cur == PRESENT as u32) == self.cards.iter().any(walk),
            "battlefield type-gate memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            c if c == ABSENT as u32 => false,
            c if c == PRESENT as u32 => true,
            _ => self.walk_and_store(shift, walk),
        }
    }

    /// `iter_mut` that does **not** invalidate — the tap / damage / counter /
    /// untap paths. Inherent, so it shadows the `Deref`'d `Vec::iter_mut` at
    /// every existing call site without touching one. Sound because the lanes
    /// read only definitions, and a definition rewrite through the handle this
    /// hands out bumps `definition_epoch` instead.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, CardInstance> {
        self.cards_unchecked_mut().iter_mut()
    }

    /// `get_mut` on the same contract as [`iter_mut`](Self::iter_mut).
    #[inline]
    pub fn get_mut(&mut self, i: usize) -> Option<&mut CardInstance> {
        self.cards_unchecked_mut().get_mut(i)
    }

    /// `&mut` at the cards **without** clearing the lanes. Private: every
    /// caller is one of the element-write accessors above, whose contract is
    /// that membership does not move. Public mutation goes through
    /// [`DerefMut`], which clears.
    #[inline]
    fn cards_unchecked_mut(&mut self) -> &mut Vec<CardInstance> {
        &mut self.cards
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
        // A rewrite since the epoch was read would be missed by the stamp, so
        // read it *before* the walk: the stamp is then conservative — a lane
        // computed across a rewrite is thrown away on the next ask.
        let epoch = crate::card::definition_epoch();
        let found = self.cards.iter().any(walk);
        self.store_lane(shift, epoch, found);
        found
    }

    /// Write one lane with what a walk found, stamped at the `epoch` that walk
    /// started from.
    fn store_lane(&self, shift: u32, epoch: u64, found: bool) {
        let lane = if found { PRESENT as u32 } else { ABSENT as u32 };
        let word = if self.def_epoch.swap(epoch, Ordering::Relaxed) == epoch {
            self.type_gates.load(Ordering::Relaxed)
        } else {
            // A different epoch was stamped: the other lanes describe an older
            // definition set and must not survive.
            0
        };
        self.type_gates
            .store((word & !(LANE_MASK << shift)) | (lane << shift), Ordering::Relaxed);
    }

    /// One split lane's read, shared by the three caller-filled lanes below:
    /// `Ok(present)` is the memo's answer, `Err(epoch)` means "unknown — walk,
    /// then hand that same epoch back to the matching `store_*`".
    ///
    /// `audit` is the lane's predicate, recomputed by the `debug_assert!` so a
    /// stale word — a write that reached the cards without clearing it — fails
    /// loudly under `-C debug-assertions=yes`; `what` names the lane in that
    /// message. Costs nothing in any release profile.
    ///
    /// `what` is a *format argument*, so the three lanes share one literal in
    /// the binary: `robustness_grid.sh`'s "is the assertion actually compiled
    /// in" check counts `memo is stale` and still sees it, but a per-lane
    /// `strings` grep will not. The name is there at runtime, where it matters.
    #[inline]
    fn split_lane(
        &self,
        shift: u32,
        audit: fn(&CardInstance) -> bool,
        what: &str,
    ) -> Result<bool, u64> {
        let epoch = crate::card::definition_epoch();
        let cur = if self.def_epoch.load(Ordering::Relaxed) != epoch {
            UNKNOWN as u32
        } else {
            (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK
        };
        debug_assert!(
            cur == UNKNOWN as u32 || (cur == PRESENT as u32) == self.cards.iter().any(audit),
            "battlefield {what} memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            c if c == ABSENT as u32 => Ok(false),
            c if c == PRESENT as u32 => Ok(true),
            _ => Err(epoch),
        }
    }

    /// The dispatch lane, for a caller that fills it from a walk it was going
    /// to make anyway: `Ok(present)` is the memo's answer, and `Err(epoch)`
    /// means "unknown — walk, then hand that same epoch back to
    /// [`store_dispatch`](Self::store_dispatch)".
    ///
    /// The two type gates ask through the closure form above because their
    /// walk *is* a predicate. The dispatcher's is not: the same pass also
    /// builds two grant lists, so a closure form would walk the board twice on
    /// every miss — and the `DerefMut` invalidation serves only 48.1 / 38.3 /
    /// 32.2 % of the type gates' asks (PERF `(-87)`), which is about where a
    /// doubled miss breaks even. Splitting the read from the store makes the
    /// miss cost the walk the caller was making anyway and nothing else.
    pub fn dispatch_lane(&self) -> Result<bool, u64> {
        self.split_lane(LANE_DISPATCH, card_has_dispatch_bits, "dispatch")
    }

    /// Record what the caller's own walk found in the dispatch lane. `epoch`
    /// is the one [`dispatch_lane`](Self::dispatch_lane) handed out, i.e. read
    /// before the walk.
    pub fn store_dispatch(&self, epoch: u64, found: bool) {
        self.store_lane(LANE_DISPATCH, epoch, found);
    }

    /// The keyword-grant lane, same caller-filled contract as
    /// [`dispatch_lane`](Self::dispatch_lane): `Ok(present)` is the memo,
    /// `Err(epoch)` means "walk, then hand that epoch to
    /// [`store_grant`](Self::store_grant)".
    ///
    /// Split rather than closure form because `keyword_grant_in_scope`'s walk
    /// is *not* the lane's predicate — it evaluates a caller-supplied
    /// `Fn(&Keyword)` per card on top of the bit — so a closure lane would
    /// walk twice on every miss. The walk already loads `grant_scan_bits` per
    /// card, so filling this from it is free.
    pub fn grant_lane(&self) -> Result<bool, u64> {
        self.split_lane(LANE_GRANT, card_has_any_grant_bits, "grant")
    }

    /// Record what the caller's own walk found in the keyword-grant lane.
    pub fn store_grant(&self, epoch: u64, found: bool) {
        self.store_lane(LANE_GRANT, epoch, found);
    }

    /// The combat-damage listener lane, same caller-filled contract as
    /// [`dispatch_lane`](Self::dispatch_lane). Split form because the walk it
    /// gates builds two trigger lists rather than answering a predicate — and
    /// that walk cannot short-circuit, so the fill it hands back is exact.
    pub fn listener_lane(&self) -> Result<bool, u64> {
        self.split_lane(LANE_LISTENER, card_has_listener_bits, "listener")
    }

    /// Record what the caller's own walk found in the listener lane.
    pub fn store_listener(&self, epoch: u64, found: bool) {
        self.store_lane(LANE_LISTENER, epoch, found);
    }

    /// The activated-grant lane, same caller-filled contract as
    /// [`dispatch_lane`](Self::dispatch_lane). Split form because
    /// `grant_scan`'s walk builds a list and evaluates each grant's CR 611.2
    /// gate, so it is not the lane's predicate — but it is already iterating
    /// each permanent's `static_abilities`, which is empty on most of a board,
    /// so filling the lane from it costs one wrapper-peel per printed static.
    pub fn act_grant_lane(&self) -> Result<bool, u64> {
        self.split_lane(LANE_ACT_GRANT, card_has_activated_grant, "activated-grant")
    }

    /// Record what the caller's own walk found in the activated-grant lane.
    pub fn store_act_grant(&self, epoch: u64, found: bool) {
        self.store_lane(LANE_ACT_GRANT, epoch, found);
    }

    /// The trigger dispatcher's **member list**: `Ok(bits)` names the
    /// battlefield indices carrying a printed trigger or a Station band,
    /// `Err(epoch)` means "unknown — walk, then hand that same epoch to
    /// [`store_trigger_members`](Self::store_trigger_members)".
    ///
    /// Every other lane answers *is there one*, which still leaves the caller
    /// walking the board; this one answers *which*, so on a hit the walk does
    /// not happen. That is what separates it from the forty-eighth pass's
    /// refuted trigger-carrier mask (+0.58 %, PERF): those bits were rebuilt
    /// inside `dispatch_board_scan` on every dispatch, so the loads added to
    /// one walk paid for the loads removed from another. These outlive the
    /// dispatch, and the fill rides the walk a miss was making anyway.
    pub fn trigger_members(&self) -> Result<u64, u64> {
        let epoch = crate::card::definition_epoch();
        let known = self.def_epoch.load(Ordering::Relaxed) == epoch
            && (self.type_gates.load(Ordering::Relaxed) >> LANE_TRIGGERER) & LANE_MASK
                == PRESENT as u32;
        if !known {
            return Err(epoch);
        }
        let bits = self.trig_members.load(Ordering::Relaxed);
        debug_assert!(
            bits == triggerer_bits(&self.cards),
            "battlefield triggerer memo is stale: a write reached the cards without clearing it",
        );
        Ok(bits)
    }

    /// Record the member list a caller's own full walk built, stamped at the
    /// `epoch` [`trigger_members`](Self::trigger_members) handed out. A board
    /// wider than 64 has no list, so the lane stays unknown there and the next
    /// dispatch walks — the caller must not have built bits for one either.
    pub fn store_trigger_members(&self, epoch: u64, bits: u64) {
        if self.cards.len() > 64 {
            return;
        }
        self.trig_members.store(bits, Ordering::Relaxed);
        self.store_lane(LANE_TRIGGERER, epoch, true);
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
            c if c == ABSENT as u32 => Some(false),
            c if c == PRESENT as u32 => Some(true),
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
            type_gates: std::sync::atomic::AtomicU32::new(
                self.type_gates.load(Ordering::Relaxed),
            ),
            def_epoch: std::sync::atomic::AtomicU64::new(
                self.def_epoch.load(Ordering::Relaxed),
            ),
            find_hint: std::sync::atomic::AtomicU32::new(
                self.find_hint.load(Ordering::Relaxed),
            ),
            trig_members: std::sync::atomic::AtomicU64::new(
                self.trig_members.load(Ordering::Relaxed),
            ),
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
        Self::from(CowBox::from(cards))
    }
}

impl From<CowBox<Vec<CardInstance>>> for Battlefield {
    fn from(cards: CowBox<Vec<CardInstance>>) -> Self {
        Self {
            cards,
            type_gates: std::sync::atomic::AtomicU32::new(0),
            def_epoch: std::sync::atomic::AtomicU64::new(0),
            find_hint: std::sync::atomic::AtomicU32::new(0),
            trig_members: std::sync::atomic::AtomicU64::new(0),
        }
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

// `for c in &mut bf` is an element-write loop, not a membership change: it
// forwards to the non-clearing `iter_mut` above, not to `DerefMut`.
impl<'a> IntoIterator for &'a mut Battlefield {
    type Item = &'a mut CardInstance;
    type IntoIter = std::slice::IterMut<'a, CardInstance>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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

    /// The second lane. Riftstone Portal is the only
    /// `GrantActivatedAbilityFromGraveyard` shape in the catalog, so the lane
    /// it gates is `ABSENT` on every board the bench deals — which is what
    /// makes it a win and also what makes it invisible to the suite unless a
    /// test deals the card. Both lanes answer independently off one packed
    /// word, and neither store disturbs the other.
    #[test]
    fn the_activated_grant_lane_is_independent_of_the_anthem_lane() {
        let g = gy(vec![crate::catalog::riftstone_portal(), crate::catalog::wonder()]);
        assert_eq!(g.memo_lane(GY_LANE_ACT_GRANT), None, "lazy until asked");
        assert!(g.has_activated_grant());
        assert_eq!(g.memo_lane(GY_LANE_ACT_GRANT), Some(true));
        assert_eq!(g.memo_lane(GY_LANE_ANTHEM), None, "the other lane is untouched");
        assert!(g.has_anthem(), "and Wonder still answers it");
        assert_eq!(
            g.memo_lane(GY_LANE_ACT_GRANT),
            Some(true),
            "packing did not disturb the first lane",
        );

        let plain = gy(vec![crate::catalog::shivan_dragon()]);
        assert!(!plain.has_activated_grant());
        assert_eq!(plain.memo_lane(GY_LANE_ACT_GRANT), Some(false));
    }

    /// A write clears **both** lanes, which is the packing's whole contract:
    /// a card entering the graveyard can carry either static.
    #[test]
    fn a_graveyard_write_clears_every_lane() {
        let mut g = gy(vec![crate::catalog::shivan_dragon()]);
        assert!(!g.has_anthem());
        assert!(!g.has_activated_grant());
        assert_eq!(g.memo_lane(GY_LANE_ANTHEM), Some(false));
        assert_eq!(g.memo_lane(GY_LANE_ACT_GRANT), Some(false));

        g.push(CardInstance::new(CardId(99), crate::catalog::riftstone_portal(), 0));
        assert_eq!(g.memo_lane(GY_LANE_ANTHEM), None, "push cleared the anthem lane");
        assert_eq!(g.memo_lane(GY_LANE_ACT_GRANT), None, "and the grant lane, one store");
        assert!(g.has_activated_grant(), "and the new card is seen");
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

    /// `find_by_id`'s index memo is verified, not invalidated: every way it
    /// can go stale — a removal that shifts the board under it, a removal
    /// that shortens the board past it, the card it names leaving — has to
    /// fall through to the scan and answer correctly anyway.
    #[test]
    fn find_by_id_hint_survives_every_membership_change() {
        let mut b = bf(vec![
            crate::catalog::grizzly_bears(),
            crate::catalog::blood_moon(),
            crate::catalog::grizzly_bears(),
        ]);
        // Warm the hint on the last slot, then shift the board under it.
        assert_eq!(b.find_by_id(CardId(2)).map(|c| c.id), Some(CardId(2)));
        b.remove(0);
        assert_eq!(b.find_by_id(CardId(2)).map(|c| c.id), Some(CardId(2)), "shifted");
        assert_eq!(b.find_by_id(CardId(1)).map(|c| c.id), Some(CardId(1)));
        assert!(b.find_by_id(CardId(0)).is_none(), "the removed card is gone");
        // A hint past the end of a shortened board is a miss, not a panic.
        assert_eq!(b.find_by_id(CardId(2)).map(|c| c.id), Some(CardId(2)));
        b.pop();
        assert!(b.find_by_id(CardId(2)).is_none());
        assert_eq!(b.find_by_id(CardId(1)).map(|c| c.id), Some(CardId(1)));
        // An empty board answers `None` through the same path.
        b.pop();
        assert!(b.find_by_id(CardId(1)).is_none());
    }

    /// The `&mut` twin locates on the shared side: a miss answers `None`
    /// without unsharing, where `iter_mut().find(…)` deep-copied the whole
    /// zone first. A hit writes, so it unshares — and the snapshot doesn't
    /// see the write.
    #[test]
    fn find_by_id_mut_misses_without_unsharing() {
        let mut a = bf(vec![crate::catalog::grizzly_bears(), crate::catalog::blood_moon()]);
        let b = a.clone();
        assert!(a.shares_with(&b), "clone is a refcount bump");
        assert!(a.find_by_id_mut(CardId(404)).is_none(), "not on this board");
        assert!(a.shares_with(&b), "a miss must not pay the unshare");

        a.find_by_id_mut(CardId(1)).expect("on this board").tapped = true;
        assert!(!a.shares_with(&b), "a hit writes, so it unshares");
        assert!(a[1].tapped);
        assert!(!b[1].tapped, "the snapshot is untouched");

        // Same hint contract as `find_by_id`: it is verified, not
        // invalidated, so a board that shrinks under it still answers.
        a.pop();
        assert!(a.find_by_id_mut(CardId(1)).is_none(), "the card left");
        assert_eq!(a.find_by_id_mut(CardId(0)).map(|c| c.id), Some(CardId(0)));
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

        // Element writes do NOT invalidate — that is the whole split.
        assert!(!b.has_land_type_changer(never));
        for c in &mut b {
            c.tapped = true;
        }
        assert_eq!(b.memo(LANE_LAND), Some(false), "&mut iteration is an element write");
        b.iter_mut().for_each(|c| c.tapped = false);
        assert_eq!(b.memo(LANE_LAND), Some(false), "so is iter_mut");
        assert!(b.get_mut(0).is_some());
        assert_eq!(b.memo(LANE_LAND), Some(false), "so is get_mut");

        // Membership does, through `DerefMut`.
        b.pop();
        assert_eq!(b.memo(LANE_LAND), None, "DerefMut invalidated");
    }

    /// A definition rewrite reaches the cards two derefs below anything the
    /// zone can observe, so the lanes are stamped with `definition_epoch`
    /// instead. Rewriting through the element handle must still be seen.
    #[test]
    fn battlefield_definition_rewrite_invalidates_through_the_epoch() {
        let land = |c: &CardInstance| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, crate::effect::StaticEffect::LandTypeChanger { .. })
            })
        };
        let mut b = bf(vec![crate::catalog::grizzly_bears()]);
        assert!(!b.has_land_type_changer(land));
        assert_eq!(b.memo(LANE_LAND), Some(false));

        // The rewrite goes through the element handle, not through the zone.
        b.iter_mut()
            .next()
            .unwrap()
            .set_definition(std::sync::Arc::new(crate::catalog::blood_moon()));
        assert!(
            b.has_land_type_changer(land),
            "the definition epoch moved, so the lane was recomputed",
        );
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

    /// The dispatch lane is filled by its caller, so its contract is the pair:
    /// an unknown lane hands out an epoch, the store makes the next ask a hit,
    /// and a membership write puts it back to unknown.
    #[test]
    fn dispatch_lane_round_trips_through_its_caller() {
        let mut b = bf(vec![crate::catalog::grizzly_bears()]);
        let Err(epoch) = b.dispatch_lane() else { panic!("a fresh zone is unknown") };
        let found = b.iter().any(card_has_dispatch_bits);
        assert!(!found, "Grizzly Bears contributes nothing to the dispatch scan");
        b.store_dispatch(epoch, found);
        assert_eq!(b.dispatch_lane(), Ok(false));
        assert_eq!(b.memo(LANE_LAND), None, "the other lanes are untouched");

        // Element writes keep it; membership does not.
        b.iter_mut().for_each(|c| c.tapped = true);
        assert_eq!(b.dispatch_lane(), Ok(false), "an element write is not a membership change");
        b.push(CardInstance::new(CardId(99), crate::catalog::blood_moon(), 0));
        assert!(b.dispatch_lane().is_err(), "push invalidated");
    }

    /// A card that does carry dispatch bits stores `PRESENT`, and the lane
    /// packs beside the type gates without disturbing them.
    #[test]
    fn dispatch_lane_sees_a_contributor_and_packs_beside_the_type_gates() {
        let never = |_: &CardInstance| false;
        let b = bf(vec![crate::catalog::humility()]);
        assert!(b.iter().any(card_has_dispatch_bits), "Humility strips abilities");
        assert!(!b.has_land_type_changer(never));
        let Err(epoch) = b.dispatch_lane() else { panic!("unknown until filled") };
        b.store_dispatch(epoch, true);
        assert_eq!(b.dispatch_lane(), Ok(true));
        assert_eq!(b.memo(LANE_LAND), Some(false), "the land lane survived the store");
    }

    /// `(-115)`'s lane is a *positional* answer where the others are a
    /// presence bit, and the dispatcher reads it **instead of** the per-card
    /// gate — so a stale one silently drops a trigger rather than costing a
    /// walk. It has to fall over exactly where the presence lanes do.
    #[test]
    fn trigger_member_list_is_filled_and_invalidated() {
        let mut b = bf(vec![
            crate::catalog::grizzly_bears(),
            crate::catalog::grave_titan(),
            crate::catalog::grizzly_bears(),
        ]);
        assert_eq!(triggerer_bits(&b), 0b010, "only the Titan carries a printed trigger");
        let Err(epoch) = b.trigger_members() else { panic!("unknown until filled") };
        b.store_trigger_members(epoch, triggerer_bits(&b));
        assert_eq!(b.trigger_members(), Ok(0b010));

        b.iter_mut().for_each(|c| c.tapped = true);
        assert_eq!(b.trigger_members(), Ok(0b010), "an element write is not a membership change");

        b.push(CardInstance::new(CardId(9), crate::catalog::grizzly_bears(), 0));
        assert!(b.trigger_members().is_err(), "push invalidated");
        let Err(epoch) = b.trigger_members() else { unreachable!() };
        b.store_trigger_members(epoch, triggerer_bits(&b));
        assert_eq!(b.trigger_members(), Ok(0b010), "and refills at the new membership");

        b.pop();
        assert!(b.trigger_members().is_err(), "DerefMut invalidated");
    }

    /// Sixty-five permanents have no member list, so the lane stays unknown
    /// and every dispatch keeps walking. The dispatcher must not build bits
    /// for one either — an index of 64 does not fit the word.
    #[test]
    fn trigger_member_list_is_not_stored_past_sixty_four_permanents() {
        let b = bf((0..65).map(|_| crate::catalog::grizzly_bears()).collect());
        let Err(epoch) = b.trigger_members() else { panic!("unknown until filled") };
        b.store_trigger_members(epoch, 0);
        assert!(b.trigger_members().is_err(), "a 65-card board has no member list");
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
