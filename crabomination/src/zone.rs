//! Zone wrappers that carry a maintained answer alongside their cards.
//!
//! A hot whole-zone walk has nowhere to keep a presence gate except a
//! wrapper. The wrapper holds the answer and its `&mut` entry points
//! invalidate it, so invalidation is a property of the type rather than an
//! enumeration of write sites.

use crabomination_base::memo_audit::board_audit;
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
const GY_LANE_TOKEN: u32 = 4;
/// A card here carries a trigger that can fire from the graveyard. Asked by
/// combat-damage dispatch (PERF `(-210)`), `fire_step_triggers` (`(-218)`)
/// and every event dispatch (`(-230)`).
const GY_LANE_TRIGGER: u32 = 6;

/// The [`GY_LANE_TRIGGER`] predicate (definition-only, as every lane
/// predicate must be): the `FromYourGraveyard` scope plus the `SelfSource`
/// kinds fired from here (`is_graveyard_self_source_kind` is the one list).
/// Wider than the combat/step walkers need, which is the sound direction.
fn card_has_graveyard_trigger(c: &CardInstance) -> bool {
    use crate::effect::EventScope;
    c.definition.triggered_abilities.iter().any(|t| {
        t.event.scope.from_graveyard()
            || (t.event.scope == EventScope::SelfSource
                && crate::game::effects::is_graveyard_self_source_kind(&t.event.kind))
    })
}

/// A player's graveyard: the CoW card list plus memoized whole-zone
/// questions hot walkers ask of it, one two-bit lane each (`GY_LANE_*`) —
/// e.g. "any
/// [`StaticEffect::GraveyardAnthem`](crate::effect::StaticEffect::GraveyardAnthem)?"
/// (Incarnation cycle) or "any graveyard activated grant?" (Riftstone Portal).
/// The memo replaced two ungated walks over a zone that grows all game
/// (eighty-fourth pass).
///
/// Sound by construction: [`DerefMut`] and the `&mut` [`IntoIterator`] are
/// the only mutable routes to the cards and both clear the memo. [`Deref`]
/// leaves read sites untouched.
#[derive(Debug, Default)]
pub struct Graveyard {
    cards: CowBox<Vec<CardInstance>>,
    /// Two-bit lanes; see the `GY_LANE_*` constants.
    lanes: AtomicU8,
}

/// Does this card carry a `GraveyardAnthem` static (or Dearly Departed's
/// `GraveyardMatchingEntersWithExtraCounters`)? The per-card half of
/// [`Graveyard::has_anthem`].
fn card_has_anthem(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| {
            matches!(
                sa.effect,
                crate::effect::StaticEffect::GraveyardAnthem { .. }
                    | crate::effect::StaticEffect::GraveyardMatchingEntersWithExtraCounters { .. }
            )
        })
}

/// CR 704.5d's per-card question: is this a token outside the battlefield?
/// The per-card half of [`Graveyard::has_token`] and [`CardPile::has_token`].
fn card_is_token(c: &CardInstance) -> bool {
    c.is_token
}

/// The [`GY_LANE_ACT_GRANT`] predicate (Riftstone Portal): the same bare
/// match `grant_scan` makes, so the lane is exact.
fn card_has_gy_activated_grant(c: &CardInstance) -> bool {
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            crate::effect::StaticEffect::GrantActivatedAbilityFromGraveyard { .. }
        )
    })
}

impl Graveyard {
    /// True when some card here carries a `GraveyardAnthem` static.
    /// Memoized; a miss walks the zone once. The `debug_assert!` in
    /// `lane` audits the memo in the suite and any
    /// debug-assertions ladder run, free in release.
    #[inline]
    pub fn has_anthem(&self) -> bool {
        self.lane(GY_LANE_ANTHEM, card_has_anthem, "anthem")
    }

    /// CR 611 — true when some card here grants an activated ability from the
    /// graveyard (Riftstone Portal); `grant_scan` asks on every call.
    pub fn has_activated_grant(&self) -> bool {
        self.lane(GY_LANE_ACT_GRANT, card_has_gy_activated_grant, "activated-grant")
    }

    /// CR 704.5d — does this graveyard hold a token that must cease to exist?
    /// Asked on every state-based sweep. Instance state, but still sound:
    /// only a write can change it and every write clears the word.
    pub fn has_token(&self) -> bool {
        self.lane(GY_LANE_TOKEN, card_is_token, "token")
    }

    #[inline]
    /// Does any card here carry a trigger that can fire from the graveyard
    /// (`card_has_graveyard_trigger`)? Memoized; a miss walks the zone once.
    pub fn has_graveyard_trigger(&self) -> bool {
        self.lane(GY_LANE_TRIGGER, card_has_graveyard_trigger, "graveyard-trigger")
    }

    /// One lane's answer: a word load on a hit, the zone walk plus one store
    /// on a miss. `what` names the lane in the audit.
    fn lane(&self, shift: u32, walk: fn(&CardInstance) -> bool, what: &str) -> bool {
        let cur = (self.lanes.load(Ordering::Relaxed) >> shift) & 0b11;
        debug_assert!(
            cur == UNKNOWN
                || board_audit(|| (cur == PRESENT) == self.cards.iter().any(walk)),
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

    /// Append via [`CowBox`]'s `push` (an unshare materializes with room for
    /// the card). Shadows the `Deref`'d `Vec::push`; clears the memo as
    /// `DerefMut` does.
    pub fn push(&mut self, mut card: CardInstance) {
        card.in_library = false;
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

/// A plain card pile — library, hand, exile — with memoized whole-zone
/// questions (`PILE_LANE_*`), first of all CR 704.5d's "does it hold a
/// token?", which the state-based sweep used to answer by walking every
/// library per seat per sweep.
///
/// Same soundness argument as [`Graveyard`]: [`DerefMut`], the `&mut`
/// [`IntoIterator`] and [`push`](Self::push) are the only routes to the cards
/// and each clears the memo. [`Deref`] leaves read sites untouched.
#[derive(Debug, Default)]
pub struct CardPile {
    cards: CowBox<Vec<CardInstance>>,
    /// One two-bit lane, packed like the other zones' so a single store on
    /// the write path clears it.
    lanes: AtomicU8,
    /// Bumped wherever `lanes` is cleared — the twin of
    /// [`Battlefield::writes`]. `GameState::exile` feeds the gather, so the
    /// gather memo's key needs it (PERF `(-303)`). Only a *change* is read.
    writes: u32,
    /// A player's library (set by [`CardPile::library`]). Every card this pile
    /// admits is stamped `in_library`; any other pile clears the stamp.
    library: bool,
    /// Cards admitted to this library from another zone since the last
    /// [`take_arrivals`](Self::take_arrivals) — "one or more cards are put
    /// into a library from anywhere" (Wan Shi Tong, All-Knowing).
    arrivals: u16,
}

/// CR 704.5d — a token is here.
const PILE_LANE_TOKEN: u32 = 0;
/// CR 702.49 — a card here is encoded on a creature (Cipher). An *instance*
/// predicate, which a pile lane may hold: every `&mut` route into the cards
/// (`push`, `DerefMut`) clears the whole word, unlike the battlefield's.
const PILE_LANE_ENCODED: u32 = 2;
/// A card here carries a `TriggerZone::WhileExiled` trigger ("if this card is
/// exiled" — Senu, Keen-Eyed Protector) or a `WhileSuspended` one (CR
/// 702.62b — Curse of the Cabal). Definition-only.
const PILE_LANE_EXILE_TRIGGER: u32 = 4;

fn card_has_exile_trigger(c: &CardInstance) -> bool {
    c.definition.triggered_abilities.iter().any(|t| {
        matches!(t.event.zone, crate::effect::TriggerZone::WhileExiled | crate::effect::TriggerZone::WhileSuspended)
    })
}

impl CardPile {
    /// One lane's answer: a word load on a hit, the pile walk plus one store
    /// on a miss. The store writes only this lane; the others reset to
    /// unknown.
    #[inline]
    fn lane(&self, shift: u32, walk: fn(&CardInstance) -> bool, what: &str) -> bool {
        let cur = (self.lanes.load(Ordering::Relaxed) >> shift) & 0b11;
        debug_assert!(
            cur == UNKNOWN
                || board_audit(|| (cur == PRESENT) == self.cards.iter().any(walk)),
            "card-pile {what} memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            ABSENT => false,
            PRESENT => true,
            _ => {
                let found = self.cards.iter().any(walk);
                let bits = if found { PRESENT } else { ABSENT };
                self.lanes.store(bits << shift, Ordering::Relaxed);
                found
            }
        }
    }

    /// CR 704.5d — does this pile hold a token? Memoized; a miss walks the
    /// pile once.
    pub fn has_token(&self) -> bool {
        self.lane(PILE_LANE_TOKEN, card_is_token, "token")
    }

    /// CR 702.49 — is any card here encoded on a creature? Memoized; read by
    /// `fire_combat_damage_triggers` in front of its Cipher walk of exile
    /// (PERF `(-218)`).
    pub fn has_encoded(&self) -> bool {
        self.lane(PILE_LANE_ENCODED, |c| c.encoded_on.is_some(), "encoded")
    }

    /// Does any card here carry a `TriggerZone::WhileExiled` trigger?
    /// Memoized; read by every event dispatch in front of its exile walk.
    pub fn has_exile_trigger(&self) -> bool {
        self.lane(PILE_LANE_EXILE_TRIGGER, card_has_exile_trigger, "exile-trigger")
    }

    /// Append via [`CowBox::push`] (an unshare materializes with room for the
    /// card). Shadows the `Deref`'d `Vec::push`.
    pub fn push(&mut self, mut card: CardInstance) {
        self.admit(&mut card);
        self.note_write();
        self.cards.push(card);
    }

    /// `Vec::insert`, shadowed so the card is [`admit`](Self::admit)ted.
    pub fn insert(&mut self, index: usize, mut card: CardInstance) {
        self.admit(&mut card);
        self.note_write();
        self.cards.insert(index, card);
    }

    /// `Vec::extend`, shadowed so each card is [`admit`](Self::admit)ted.
    pub fn extend(&mut self, cards: impl IntoIterator<Item = CardInstance>) {
        self.note_write();
        for mut card in cards {
            self.admit(&mut card);
            self.cards.push(card);
        }
    }

    /// An empty library pile.
    pub fn library() -> Self {
        Self { library: true, ..Self::default() }
    }

    /// Make this pile a library: its cards count as already in it (a
    /// deserialized or rebuilt library).
    pub fn mark_library(&mut self) {
        self.library = true;
        if self.cards.iter().any(|c| !c.in_library) {
            for c in self.deref_mut().iter_mut() {
                c.in_library = true;
            }
        }
    }

    /// CR 400.7 — a card reaching a library from another zone is "put into a
    /// library"; one already in it (a scry, a reorder) is not.
    #[inline]
    fn admit(&mut self, card: &mut CardInstance) {
        if self.library {
            if !card.in_library {
                card.in_library = true;
                self.arrivals = self.arrivals.saturating_add(1);
            }
        } else {
            card.in_library = false;
        }
    }

    /// Cards put into this library from elsewhere since the last call.
    #[inline]
    pub fn arrivals(&self) -> u16 {
        self.arrivals
    }

    /// Read and reset [`arrivals`](Self::arrivals). Touches no card, so it
    /// never unshares the pile.
    pub fn take_arrivals(&mut self) -> u16 {
        std::mem::take(&mut self.arrivals)
    }

    /// One `&mut` reach at this pile: clear the lane word and bump
    /// [`writes`](Self::writes). The two call sites are this `push` and
    /// `DerefMut`, which the `&mut` `IntoIterator` arm routes through.
    #[inline]
    fn note_write(&mut self) {
        self.lanes.store(0, Ordering::Relaxed);
        self.writes = self.writes.wrapping_add(1);
    }

    /// This pile's write counter — see [`writes`](Self::writes).
    #[inline]
    pub fn writes(&self) -> u32 {
        self.writes
    }

    /// True when both handles still share one allocation — the [`CowBox`]
    /// contract, forwarded for the zone tests.
    pub fn shares_with(&self, other: &Self) -> bool {
        self.cards.shares_with(&other.cards)
    }

    /// The lane's current state, for the invalidation tests: `None` while
    /// unknown.
    #[cfg(test)]
    fn memo(&self) -> Option<bool> {
        match (self.lanes.load(Ordering::Relaxed) >> PILE_LANE_TOKEN) & 0b11 {
            ABSENT => Some(false),
            PRESENT => Some(true),
            _ => None,
        }
    }
}

impl Clone for CardPile {
    /// A refcount bump on the cards; the memo and write counter come along,
    /// so a clone diverges from its parent on its own first write.
    fn clone(&self) -> Self {
        Self {
            cards: self.cards.clone(),
            lanes: AtomicU8::new(self.lanes.load(Ordering::Relaxed)),
            writes: self.writes,
            library: self.library,
            arrivals: self.arrivals,
        }
    }
}

impl Deref for CardPile {
    type Target = Vec<CardInstance>;
    fn deref(&self) -> &Vec<CardInstance> {
        &self.cards
    }
}

impl DerefMut for CardPile {
    fn deref_mut(&mut self) -> &mut Vec<CardInstance> {
        self.note_write();
        &mut self.cards
    }
}

impl From<Vec<CardInstance>> for CardPile {
    fn from(cards: Vec<CardInstance>) -> Self {
        Self { cards: cards.into(), lanes: AtomicU8::new(0), writes: 0, library: false, arrivals: 0 }
    }
}

impl From<CowBox<Vec<CardInstance>>> for CardPile {
    fn from(cards: CowBox<Vec<CardInstance>>) -> Self {
        Self { cards, lanes: AtomicU8::new(0), writes: 0, library: false, arrivals: 0 }
    }
}

// `for c in &pile` / `for c in &mut pile` — deref coercion doesn't apply to
// `for` loops, so forward `IntoIterator` explicitly. The `&mut` arm is the
// second invalidation point.
impl<'a> IntoIterator for &'a CardPile {
    type Item = &'a CardInstance;
    type IntoIter = std::slice::Iter<'a, CardInstance>;
    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter()
    }
}

impl<'a> IntoIterator for &'a mut CardPile {
    type Item = &'a mut CardInstance;
    type IntoIter = std::slice::IterMut<'a, CardInstance>;
    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().iter_mut()
    }
}

impl serde::Serialize for CardPile {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.cards.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for CardPile {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        CowBox::<Vec<CardInstance>>::deserialize(deserializer).map(CardPile::from)
    }
}

/// A library's `Deserialize`: the pile, marked as a library.
pub fn deserialize_library<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<CardPile, D::Error> {
    let mut pile = <CardPile as serde::Deserialize>::deserialize(deserializer)?;
    pile.mark_library();
    Ok(pile)
}

/// Packed two-bit memo lanes for [`Battlefield`] (`UNKNOWN` / `ABSENT` /
/// `PRESENT`), so one store on the write path clears every lane. A `u64`
/// holds thirty-two (a `u32` until PERF `(-207)`).
///
/// **A lane has exactly one predicate**, however many callers ask through it.
/// Only [`Battlefield::lane`]'s `debug_assert!` (recomputing with the handed
/// predicate) catches a second one.
const LANE_LAND: u32 = 0;
const LANE_CREATURE: u32 = 2;
const LANE_DAMAGE_SCALE: u32 = 4;
const LANE_OUTGOING_PREVENT: u32 = 6;
const LANE_INCOMING_PREVENT: u32 = 8;
const LANE_DISPATCH: u32 = 10;
const LANE_SHIELD: u32 = 12;
const LANE_GRANT: u32 = 14;
const LANE_LISTENER: u32 = 16;
/// PERF `(-128)`: how many times anything took `&mut` at the battlefield
/// (its three chokepoints). `sba_census` uses it to count SBA sweeps that
/// follow no mutable reach — what a dirty-bit gate could serve. Global
/// because the census runs one thread.
#[cfg(feature = "trig-census")]
pub static BATTLEFIELD_REACHES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// One `&mut` reach at the battlefield.
#[cfg(feature = "trig-census")]
#[inline]
pub fn note_battlefield_reach() {
    if trig_census::on() {
        BATTLEFIELD_REACHES.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(not(feature = "trig-census"))]
#[inline(always)]
pub fn note_battlefield_reach() {}

const LANE_ACT_GRANT: u32 = 18;
/// Not a presence lane: `PRESENT` means "the member list beside it is
/// computed". It shares the word so every write path's store clears it.
const LANE_TRIGGERER: u32 = 20;
const LANE_PT_REDUCE: u32 = 22;
const LANE_GATE_KEYWORD: u32 = 24;
/// Any permanent carries a `dispatch_bits::MANA_STATIC` static (mana-path
/// walks, PERF `(-197)`).
const LANE_MANA_STATIC: u32 = 26;
/// Any permanent grants activated abilities to *countered* creatures
/// (Agatha's Soul Cauldron) — why counters matter to the grants-nothing gate
/// (PERF `(-199)`).
const LANE_COUNTER_GRANT: u32 = 28;
/// Any permanent redirects a card bound for a graveyard (per-death walks,
/// PERF `(-203)`).
const LANE_DEATH_REDIRECT: u32 = 30;
/// Any permanent's *definition* can change a card type (`type_bits::ALL`;
/// the attachment-gated half over-approximated, as a lane predicate reads no
/// instance field). `ABSENT` settles `card_type_change_unscoped`'s walk
/// (PERF `(-207)`).
const LANE_CARD_TYPE: u32 = 32;
/// Any permanent carries a draw-replacement static (`draw_one`, PERF
/// `(-233)`). See [`card_has_draw_static`].
const LANE_DRAW_STATIC: u32 = 34;
/// Any permanent changes how *another* permanent enters (ETB-trigger
/// suppressor / doubler, enters-tapped, lands-enter-untapped; PERF
/// `(-234)`). See [`card_has_etb_static`].
const LANE_ETB_STATIC: u32 = 36;
/// Any permanent replaces or redirects damage to a player
/// (`deal_damage_to_from`, `damage_redirect_target`; PERF `(-235)`). See
/// [`card_has_damage_replacement_static`].
const LANE_DAMAGE_STATIC: u32 = 38;
/// Any permanent locks or extends land plays (`can_player_play_land`; PERF
/// `(-236)`). See [`card_has_land_play_static`].
const LANE_LAND_PLAY_STATIC: u32 = 40;
/// Any permanent lets mana be spent as though it were any colour (payment
/// relaxation; PERF `(-237)`). See [`card_has_any_color_static`].
const LANE_ANY_COLOR_STATIC: u32 = 42;
/// Any permanent changes a maximum hand size (`effective_max_hand_size`;
/// PERF `(-238)`). See [`card_has_hand_size_static`].
const LANE_HAND_SIZE_STATIC: u32 = 44;
/// Any permanent puts counters on *another* permanent as it enters (the two
/// enters-with-counters walkers; PERF `(-239)`). See
/// [`card_has_etb_counter_static`].
const LANE_ETB_COUNTER_STATIC: u32 = 46;
/// Any permanent carries a damage-prevention static
/// (`prevent_static_scan`; PERF `(-240)`). See [`card_has_prevent_static`].
const LANE_PREVENT_STATIC: u32 = 48;
/// Any permanent charges a block tax (`BlockTaxToController`, read by
/// `block_tax_for`; PERF `(-241)`). See [`card_has_block_tax_static`].
const LANE_BLOCK_TAX_STATIC: u32 = 50;
/// Any permanent carries a static `do_untap` reads (Mist of Stagnation,
/// Seedborn Muse, prevention / cap families, off-turn untappers; PERF
/// `(-249)`). See [`card_has_untap_static`].
const LANE_UNTAP_STATIC: u32 = 52;
/// Any permanent changes life-total arithmetic (`adjust_life`'s helpers;
/// PERF `(-262)`). See [`card_has_life_static`].
const LANE_LIFE_STATIC: u32 = 54;
/// Any permanent's *definition* can grant activated abilities along an
/// attachment or Soulbond pairing (`grant_scan`; PERF `(-315)`). The instance
/// gates (`attached_to`, `soulbond_partner`) are over-approximated. See
/// [`card_has_attach_grant`].
const LANE_ATTACH_GRANT: u32 = 56;
/// Any permanent lets summoning-sick creatures activate as though they had
/// haste (Tyvar, CR 602.5a; `tap_ability_summoning_sick`, PERF `(-316)`).
/// See [`card_has_haste_static`].
const LANE_HASTE_STATIC: u32 = 58;
/// Any permanent caps attackers or blockers per combat (Silent Arbiter,
/// Dueling Grounds; `combat_participation_cap`, PERF `(-324)`). One lane for
/// both flavours. See [`card_has_combat_cap_static`].
const LANE_COMBAT_CAP: u32 = 60;
/// [`Battlefield::attach_fold`]'s stamp half (bits 0-30) and its one-bit
/// answer (bit 31). A word of 0 is "never folded"; `writes` starts at 1.
const ATTACH_STAMP: u32 = 0x7fff_ffff;
const ATTACH_PRESENT: u32 = 1 << 31;
const LANE_MASK: u64 = 0b11;
/// Bit 0 of every lane field — set exactly on the `ABSENT` lanes.
const LANE_ABSENT_BITS: u64 = 0x5555_5555_5555_5555;
/// The lane count the predicate table below covers (shift 0 ..= 60).
const LANE_COUNT: usize = 31;

/// ⚠ **One two-bit field is left in the word** (shift 62). Past that, a new
/// lane needs a second word on `Battlefield` (priced by
/// `cow::tests::game_state_stays_small`) or a shared lane; this assert makes
/// overflow a build failure. PERF `(-347)`: the last slot is not worth
/// spending on `card_color_change_unscoped`.
const _: () = assert!(
    (LANE_COUNT as u32) * 2 <= u64::BITS,
    "the lane word is full: a thirty-third lane needs a second word or a shared lane",
);

/// A lane's definition-only predicate.
type LanePredicate = fn(&CardInstance) -> bool;

/// Every presence lane's predicate, indexed by shift / 2, so a membership
/// write can update a lane off the **one card it moved** (PERF `(-213)`): a
/// push turns `ABSENT` to `PRESENT` iff the card passes; a removal of a
/// passing card drops `PRESENT` to `UNKNOWN`. `None` on member-list lanes.
/// **Each entry must be the very predicate that lane's callers hand to
/// [`Battlefield::lane`]** — the lane audit fails the suite on drift.
const LANE_PREDICATES: [Option<LanePredicate>; LANE_COUNT] = [
    Some(crate::game::card_can_change_land_types),       // LANE_LAND
    Some(crate::game::card_can_change_creature_types),   // LANE_CREATURE
    Some(crate::game::card_can_scale_damage),            // LANE_DAMAGE_SCALE
    Some(crate::game::card_can_prevent_outgoing_damage), // LANE_OUTGOING_PREVENT
    Some(crate::game::card_can_prevent_incoming_damage), // LANE_INCOMING_PREVENT
    None,                                                // LANE_DISPATCH (member list)
    Some(crate::game::card_can_shield_damage),           // LANE_SHIELD
    None,                                                // LANE_GRANT (member list)
    Some(card_has_listener_bits),                        // LANE_LISTENER
    Some(card_has_activated_grant),                      // LANE_ACT_GRANT
    None,                                                // LANE_TRIGGERER (member list)
    Some(crate::game::card_can_reduce_toughness),        // LANE_PT_REDUCE
    Some(card_has_gate_keyword),                         // LANE_GATE_KEYWORD
    Some(card_has_mana_static),                          // LANE_MANA_STATIC
    Some(crate::game::actions::card_grants_to_countered),// LANE_COUNTER_GRANT
    Some(crate::game::actions::card_redirects_deaths),   // LANE_DEATH_REDIRECT
    Some(crate::game::card_can_change_card_types_def),   // LANE_CARD_TYPE
    Some(card_has_draw_static),                          // LANE_DRAW_STATIC
    Some(card_has_etb_static),                           // LANE_ETB_STATIC
    Some(card_has_damage_replacement_static),         // LANE_DAMAGE_STATIC
    Some(card_has_land_play_static),                  // LANE_LAND_PLAY_STATIC
    Some(card_has_any_color_static),                  // LANE_ANY_COLOR_STATIC
    Some(card_has_hand_size_static),                  // LANE_HAND_SIZE_STATIC
    Some(card_has_etb_counter_static),                   // LANE_ETB_COUNTER_STATIC
    Some(card_has_prevent_static),                       // LANE_PREVENT_STATIC
    Some(card_has_block_tax_static),                     // LANE_BLOCK_TAX_STATIC
    Some(card_has_untap_static),                         // LANE_UNTAP_STATIC
    Some(card_has_life_static),                          // LANE_LIFE_STATIC
    Some(crate::game::actions::card_has_attach_grant),   // LANE_ATTACH_GRANT
    Some(card_has_haste_static),                         // LANE_HASTE_STATIC
    Some(card_has_combat_cap_static),                    // LANE_COMBAT_CAP
];

/// Does this permanent contribute anything to
/// [`GameState::dispatch_board_scan`](crate::game::GameState::dispatch_board_scan)?
/// The [`LANE_DISPATCH`] predicate, also the one its audit checks against.
fn card_has_dispatch_bits(c: &CardInstance) -> bool {
    c.dispatch_scan_bits() & crate::card::dispatch_bits::BOARD_SCAN != 0
}

/// The [`LANE_MANA_STATIC`] predicate: a static the mana-ability path reads
/// per activation (the engine's one definition of it).
fn card_has_mana_static(c: &CardInstance) -> bool {
    crate::game::actions::card_has_mana_static(c)
}


/// Can this permanent grant a keyword to anything? The [`LANE_GRANT`]
/// predicate, also the one its audit checks against.
fn card_has_any_grant_bits(c: &CardInstance) -> bool {
    c.grant_scan_bits() & crate::card::grant_bits::ANY_GRANT != 0
}

/// Does this permanent carry a `YourControl` / `AnyPlayer` printed trigger —
/// the two scopes the combat-damage dispatch's listener walk looks for? The
/// [`LANE_LISTENER`] predicate.
fn card_has_listener_bits(c: &CardInstance) -> bool {
    c.dispatch_scan_bits() & crate::card::dispatch_bits::LISTENER != 0
}

/// Does this permanent carry a printed `GrantActivatedAbility` static under
/// any CR 611.2 wrapper? The [`LANE_ACT_GRANT`] predicate; over-approximates
/// `grant_scan` (which evaluates the wrappers), the sound direction.
fn card_has_activated_grant(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| crate::effect::static_effect_grants_activated(&sa.effect))
}

/// Is any keyword
/// [`GameState::board_keyword_matching`](crate::game::GameState::board_keyword_matching)
/// is ever asked about *printed* on this permanent? The [`LANE_GATE_KEYWORD`]
/// predicate: the union of every caller's list. A keyword missing here gives
/// a wrong `false` on the printed leg, which the gate's `debug_assert!`
/// catches.
///
/// ⚠ `Soulbond` is here for `apply_soulbond_pairing`, which reads the lane
/// directly rather than via `board_keyword_matching` (PERF `(-327)`).
/// Widening the union costs other callers only on boards with the keyword.
pub(crate) fn card_has_gate_keyword(c: &CardInstance) -> bool {
    use crate::card::Keyword::*;
    c.definition.keywords.iter().any(|k| {
        matches!(
            k,
            Soulbond
                | MustAttack
                | MustAttackOrBlock
                | MustAttackIfAnotherAttacks
                | MustAttackChosenPlayer
                | MustBlock
                | MustBeBlocked
                | MustBeBlockedByAtLeast(_)
                | AllMustBlock
                | CantBeBlockedUnlessAllBlock
                | CantBeBlockedByMoreThanOne
                // The block planner's spare-capacity pass (PERF `(-336)`).
                | CanBlockAnyNumber
                | CanBlockAdditional(_)
                | CantBlock
                | Flanking
                | Bushido(_)
                | Rampage(_)
                | Phasing
                | CumulativeUpkeep(_)
                | DoesntUntapWhileCounter(_)
                | DoesntUntapIfAttackedLastTurn
        )
    })
}

/// Can this permanent contribute a trigger of its **own** to
/// [`GameState::dispatch_triggers_for_events`](crate::game::GameState::dispatch_triggers_for_events)
/// — a printed triggered ability or a CR 722.2a Station band? The
/// [`LANE_TRIGGERER`] predicate. The dispatcher's fast path calls it too: the
/// member list and the gate it replaces must ask the same question.
pub(crate) fn card_is_triggerer(c: &CardInstance) -> bool {
    !c.definition.triggered_abilities.is_empty() || !c.definition.station.is_empty()
}

/// Does this permanent's definition change how a permanent enters — the
/// ETB-trigger suppressors / doublers (`etb_trigger_multiplier`), the copier
/// (`entering_trigger_copiers`, Aboleth Spawn), cross-permanent
/// `EntersTapped` and `LandsEnterUntapped`? The [`LANE_ETB_STATIC`]
/// predicate; definition-only.
fn card_has_etb_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::SuppressCreatureEtbTriggers { .. }
                | S::EtbTriggerSpotlight
                | S::DoubleControllerEtbTriggers { .. }
                | S::CopyOpponentsEnteringCreatureTriggers
                | S::EntersTapped { .. }
                | S::LandsEnterUntapped
                | S::MatchingEnterUntapped { .. }
                | S::OthersEnterWithSourceTapState
        )
    })
}

/// Does this permanent's definition replace or redirect damage dealt to a
/// player — The Mindskinner, Crumbling Sanctuary,
/// Delaying Shield, Nefarious Lich, Pariah's Shield, Palisade Giant? The
/// [`LANE_DAMAGE_STATIC`] predicate; definition-only.
fn card_has_damage_replacement_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::YourDamageToOpponentsBecomesMill
                | S::PlayerDamageBecomesExileFromLibrary
                | S::ReplaceDamageToYouWithCountersOnSource { .. }
                | S::ReplaceDamageToYouWithGraveyardExile
                | S::PreventDamageToYouMillTwice
                | S::PreventDamageToYouExileFromGraveyard
                | S::ReplaceDamageToYouRemovingStampedCounters { .. }
                | S::RedirectControllerDamageToEquippedCreature
                | S::RedirectDamageToSelf
        )
    })
}

/// Does this permanent's definition carry a static that locks or extends
/// land plays — Aggressive Mining, Damping Engine, an extra-land-per-turn
/// grant? The [`LANE_LAND_PLAY_STATIC`] predicate; definition-only.
fn card_has_land_play_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect.ungated(),
            S::NoPlayerCanPlayLands
                | S::ControllerCantPlayLands
                | S::MostPermanentsCantPlay
                | S::OpponentsWithMoreCantPlayThatType
                | S::ExtraLandPerTurn
                | S::ExtraLandPerTurnEachPlayer
        )
    })
}

/// Does this permanent's definition let mana be spent as any colour —
/// Mycosynth Lattice, Unexpected Potential, Emissary's Ploy? The
/// [`LANE_ANY_COLOR_STATIC`] predicate; definition-only.
fn card_has_any_color_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::PlayersMaySpendManaAsAnyColor
                | S::YouMaySpendManaAsAnyColor
                | S::MaySpendManaAsAnyColorForNamedSpells
                | S::MaySpendManaAsAnyColorForCreaturesWithChosenMv
                | S::MaySpendManaAsAnyColorForCreatureSpells
        )
    })
}

/// Does this permanent's definition change a maximum hand size — Reliquary
/// Tower, Jin-Gitaxias, Thought Nibbler, Necrodominance, Cursed Rack, Minamo
/// Scrollkeeper? The [`LANE_HAND_SIZE_STATIC`] predicate; definition-only.
fn card_has_hand_size_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::NoMaximumHandSize
                | S::AllPlayersNoMaximumHandSize
                | S::OpponentsMaxHandSizeReduced(_)
                | S::ControllerMaxHandSizeReduced(_)
                | S::ControllerMaxHandSize(_)
                | S::ControllerMaxHandSizeFromSourceCounters(_)
                | S::ChosenPlayerMaxHandSize(_)
                | S::ControllerMaxHandSizeIncreased(_)
        )
    })
}

/// Does this permanent's definition put counters on another permanent as
/// it enters — Metallic Mimic, Oath of Gideon, Arlinn, Master Biomancer,
/// Muzzio's Preparations, Giada, Hardened Scales-style cast riders? The
/// [`LANE_ETB_COUNTER_STATIC`] predicate; definition-only.
fn card_has_etb_counter_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect.ungated(),
            S::PlaneswalkersEnterWithExtraLoyalty { .. }
                | S::ChosenTypeEntersWithCounter { .. }
                | S::TypeEntersWithCounter { .. }
                | S::TypeEntersWithCountersPerControlled { .. }
                | S::TypedCreaturesEnterWithExtraCounter { .. }
                | S::MatchingEntersWithExtraCounters { .. }
                | S::MatchingEntersWithKeywordCounter { .. }
                | S::MatchingEntersWithCountersPer { .. }
                | S::OtherCreaturesEnterWithCountersEqualToSourcePower { .. }
                | S::OtherCreaturesEnterAsAdditionalType { .. }
                | S::MatchingEntersWith { .. }
                | S::OtherCreaturesEnterWithCountersEqualToSourceToughness { .. }
                | S::OtherCreaturesEnterWithCountersEqualToSourceCounters { .. }
                | S::OtherCreaturesEnterWithCountersPerTreasureMana { .. }
                | S::ExtraEtbCountersForCreatureCasts { .. }
        )
    })
}

/// Does this permanent's definition carry a damage-prevention static
/// `prevent_static_scan` folds? The [`LANE_PREVENT_STATIC`] predicate (the
/// same arm list, so a clear lane is exactly a zero mask); definition-only.
fn card_has_prevent_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::DamageCantBePrevented
                | S::CombatDamageCantBePrevented
                | S::ControllerCreaturesCombatDamageCantBePrevented
                | S::SourceDamageCantBePrevented
                | S::PreventTargetingDamageWhileYouControlAnotherCreature
                | S::PreventAllDamageToThisFromBlocked
                | S::PreventCombatDamageToThisFromMatching { .. }
                | S::ReduceDamageToControllerFromSource { .. }
                | S::ControllerProtectionFromMatching { .. }
                | S::PreventSmallDamageToThis { .. }
                | S::PreventDamageToAttachedPerPermanent { .. }
                | S::PreventDamageByRemovingCounters { .. }
                | S::PreventDamageToThisRedirect
                | S::PreventAllDamageToControllerFromOthersSources
                | S::AllDamageDealtAsThoughWither
                | S::PreventSpellDamageToYouAndYourOtherPermanents
                | S::PreventAllSpellDamage
                | S::PreventAllButOneDamageToYouAndYourHeroes
                | S::PreventDamageToYouAndYoursFromNamed
                | S::ReduceCreatureCombatDamageToYouAndYoursByCounters { .. }
        )
    })
}

/// Does this permanent's definition cap attackers or blockers per combat
/// (Silent Arbiter, Dueling Grounds)? The [`LANE_COMBAT_CAP`] predicate — the
/// same test `combat_participation_cap` makes; definition-only.
fn card_has_combat_cap_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as SE;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(sa.effect, SE::MaxAttackersPerCombat(_) | SE::MaxBlockersPerCombat(_))
    })
}

/// Does this permanent's definition charge a block tax (Archangel of
/// Tithes, Heat Wave)? The [`LANE_BLOCK_TAX_STATIC`] predicate;
/// definition-only.
fn card_has_block_tax_static(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| {
            matches!(
                sa.effect,
                crate::effect::StaticEffect::BlockTaxToController { .. }
                    | crate::effect::StaticEffect::BlockTaxForEnchantedAttacker { .. }
            )
        })
}

/// Does this permanent's definition carry a static `do_untap` reads, under
/// any `While*` wrapper (peeled unconditionally, the sound direction)? The
/// [`LANE_UNTAP_STATIC`] predicate; definition-only.
pub(crate) fn card_has_untap_static(c: &CardInstance) -> bool {
    c.definition.static_abilities.iter().any(|sa| static_effect_touches_untap(&sa.effect))
}

fn static_effect_touches_untap(e: &crate::effect::StaticEffect) -> bool {
    use crate::effect::StaticEffect as S;
    match e {
        S::WhileClassLevelAtLeast { inner, .. }
        | S::WhileYourTurn { inner }
        | S::WhileNotYourTurn { inner }
        | S::WhileCountersAtLeast { inner, .. }
        | S::WhileCondition { inner, .. } => static_effect_touches_untap(inner),
        S::PermanentsDontUntap
        | S::UntapAllYoursEachUntapStep
        | S::UntapYoursEachUntapStepFiltered(_)
        | S::UntapSelfEachOtherUntapStep
        | S::UntapOnlyChosenTypeWhileUntapped
        | S::PreventUntap { .. }
        | S::PreventUntapGlobal { .. }
        | S::CantBecomeUntapped { .. }
        | S::MaxOneUntapPerStep { .. }
        | S::MaxUntapsPerStep { .. }
        | S::UntapSelfEachUntapStep
        | S::UntapAttachedEachUntapStep
        | S::OpponentsStunCountersStay => true,
        _ => false,
    }
}

/// Does this permanent's definition change life-total arithmetic (what
/// `adjust_life`'s helpers walk for) or replace a life *payment* (Ashiok,
/// Wicked Manipulator), under any `While*` wrapper (peeled unconditionally,
/// the sound direction)? The [`LANE_LIFE_STATIC`] predicate; definition-only.
pub(crate) fn card_has_life_static(c: &CardInstance) -> bool {
    c.definition.static_abilities.iter().any(|sa| static_effect_touches_life(&sa.effect))
}

fn static_effect_touches_life(e: &crate::effect::StaticEffect) -> bool {
    use crate::effect::StaticEffect as S;
    match e {
        S::WhileClassLevelAtLeast { inner, .. }
        | S::WhileYourTurn { inner }
        | S::WhileNotYourTurn { inner }
        | S::WhileCountersAtLeast { inner, .. }
        | S::WhileCondition { inner, .. } => static_effect_touches_life(inner),
        S::LifeGainBecomesLoss { .. }
        | S::LifeGainBecomesDraw
        | S::LifeGainBonus { .. }
        | S::LifeGainMultiplier { .. }
        | S::LifeLossMultiplier { .. }
        | S::ControllerLifeGainDoubledAtOrBelow { .. }
        | S::PlayerCannotGainLife { .. }
        | S::PlayerCannotLoseLife { .. }
        | S::PlayersCantPayLifeOrSacrificeCreaturesForCosts
        | S::PlayersCantPayLifeOrSacrificeNonlandForCosts
        | S::PlayersCantPayLifeForNonManaCosts
        // Painter's Servant shares the lane: `painted_colors` is gated on it
        // (one rare card; the lane word has no field to spare, see below).
        | S::ChosenColorEverywhere
        | S::OpponentLifeLossDoubledDuringYourTurn
        | S::PayLifeExilesLibraryTopInstead
        | S::LifeToZeroTransformsSelf { .. } => true,
        _ => false,
    }
}

/// Does this permanent's definition grant the haste-as-though for
/// activations (Tyvar, CR 602.5a), under any `While*` wrapper (peeled, the
/// sound direction)? The [`LANE_HASTE_STATIC`] predicate; definition-only, so
/// the `controller == p` test stays the caller's.
fn card_has_haste_static(c: &CardInstance) -> bool {
    c.definition.static_abilities.iter().any(|sa| static_effect_is_haste_gate(&sa.effect))
}

fn static_effect_is_haste_gate(e: &crate::effect::StaticEffect) -> bool {
    use crate::effect::StaticEffect as S;
    match e {
        S::WhileClassLevelAtLeast { inner, .. }
        | S::WhileYourTurn { inner }
        | S::WhileNotYourTurn { inner }
        | S::WhileCountersAtLeast { inner, .. }
        | S::WhileCondition { inner, .. } => static_effect_is_haste_gate(inner),
        S::ControllerCreatureAbilitiesAsThoughHaste => true,
        _ => false,
    }
}

/// Does this permanent's definition carry a static that can replace, skip,
/// redirect or tax a draw? The [`LANE_DRAW_STATIC`] predicate: the union of
/// what `draw_one` and its helpers match, so a clear lane is authoritative.
/// Definition-only; their `active_static` gates only narrow it.
fn card_has_draw_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    // `ungated`: Archmage Ascension's tutor is `WhileCountersAtLeast`.
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect.ungated(),
            S::PlayersSkipDraws
                | S::PlayersCantDraw
                | S::SharedFate
                | S::PlayersDrawExiledPlayable
                | S::ControllerMaySkipDraws
                | S::MayReplaceDrawWithCounter { .. }
                | S::MayReplaceDrawWithTutor
                | S::MayDrawFromSourceExilePile
                | S::MayReplaceDrawWithRevealUntilKind
                | S::ReplaceDrawWithLookN { .. }
                | S::FirstDrawEachTurnMillsOneOfTwo
                | S::ReplaceDrawWithImpulse { .. }
                | S::ReplaceDrawWithExileFaceDownWithSource
                | S::ReplaceDrawCreaturesToGraveyard
                | S::ReplaceDrawWithGraveyardReturn
                | S::ChainsOfMephistopheles
                | S::EmptyHandDrawBonus { .. }
                | S::DrawsRevealedTaxed { .. }
                | S::DrawsRevealedOthersMayPayLifeToBin { .. }
                | S::OpponentExtraDrawsRedirected
                | S::ControllerDrawsFromBottom
        )
    })
}

/// [`card_has_any_grant_bits`] over a whole board as an index mask — the
/// grant member list's audit, recomputed by
/// [`Battlefield::grant_members`]' `debug_assert!`. `0` past 64 cards.
fn grant_bits(cards: &[CardInstance]) -> u64 {
    if cards.len() > 64 {
        return 0;
    }
    cards
        .iter()
        .enumerate()
        .filter(|(_, c)| card_has_any_grant_bits(c))
        .fold(0u64, |bits, (i, _)| bits | (1 << i))
}

/// [`card_is_triggerer`] over a whole board as an index mask — the member
/// list's audit, recomputed by [`Battlefield::trigger_members`]'
/// `debug_assert!`. `0` past 64 cards (the list is never stored there).
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

/// [`card_has_dispatch_bits`] over a whole board as an index mask — the
/// dispatch member list's audit. `0` past 64 cards.
fn dispatch_bits(cards: &[CardInstance]) -> u64 {
    if cards.len() > 64 {
        return 0;
    }
    let mut bits = 0u64;
    for (i, c) in cards.iter().enumerate() {
        if card_has_dispatch_bits(c) {
            bits |= 1u64 << i;
        }
    }
    bits
}

/// `CRAB_TRIG_CENSUS` — the trigger dispatcher's member-list census.
///
/// PERF `(-115)`: the member list saves `hits x (board - members)` visits;
/// these counts measure that, since the lane inlines away and no callgrind
/// row carries it. Off unless the variable is set.
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
    /// Dispatches that walked the board with a grant live (the lane is no
    /// use there), and the permanents they walked.
    pub static GRANTED: AtomicU64 = AtomicU64::new(0);
    pub static GRANT_BOARD: AtomicU64 = AtomicU64::new(0);
    /// Loop bodies the walk runs — `popcount` on a hit, the whole board
    /// otherwise. The denominator for per-visit Ir figures.
    pub static VISITS: AtomicU64 = AtomicU64::new(0);

    /// PERF `(-120)`: **which** of the four gates made a dispatch grant-live
    /// (the sixteen `GRANT_*` mask buckets), and each bucket's visits.
    pub static REASON: [AtomicU64; 16] = [const { AtomicU64::new(0) }; 16];
    pub static REASON_VISITS: [AtomicU64; 16] = [const { AtomicU64::new(0) }; 16];
    /// The same masks under `(-120)`'s event-kind filter. `[0]` is "would
    /// have become no-grant".
    pub static FILTERED: [AtomicU64; 16] = [const { AtomicU64::new(0) }; 16];
    pub static FILTERED_VISITS: [AtomicU64; 16] = [const { AtomicU64::new(0) }; 16];

    /// Reason-mask bits: board `GrantTriggeredAbility` statics (already
    /// event-kind filtered), CR 611.2 turn-scoped watchers, per-card EOT
    /// grants, and `equipped_bonus` triggers. Mask `0` is `no_grants`.
    pub const GRANT_STATIC: u8 = 1;
    pub const GRANT_TURN: u8 = 2;
    pub const GRANT_OWN: u8 = 4;
    pub const GRANT_EQUIP: u8 = 8;
    /// Bucket labels, indexed by the mask.
    pub const REASON_NAMES: [&str; 16] = [
        "none", "static", "turn", "static+turn", "own", "static+own", "turn+own",
        "static+turn+own", "equip", "static+equip", "turn+equip", "static+turn+equip", "own+equip",
        "static+own+equip", "turn+own+equip", "all",
    ];

    /// `0` not yet read, `1` off, `2` on. **Not a `OnceLock`**: asked once
    /// per dispatch, and a relaxed byte is ~2 Ir against `get_or_init`'s ~20.
    /// The store is idempotent, so the init race has one outcome.
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

    /// One dispatch: `members` is `Some(count)` on a hit, `reason` is the
    /// gate mask above (`0` = `no_grants`) and `filtered` the same mask with
    /// `(-120)`'s event-kind filter applied to the three gates that lack one.
    #[cold]
    #[inline(never)]
    pub fn tick(reason: u8, filtered: u8, board: usize, members: Option<u32>) {
        let no_grants = reason == 0;
        let visits = if no_grants { members.map(u64::from) } else { None }.unwrap_or(board as u64);
        REASON[reason as usize].fetch_add(1, Relaxed);
        REASON_VISITS[reason as usize].fetch_add(visits, Relaxed);
        FILTERED[filtered as usize].fetch_add(1, Relaxed);
        FILTERED_VISITS[filtered as usize].fetch_add(visits, Relaxed);
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

    /// `(dispatches, visits)` per reason bucket, and the same under
    /// `(-120)`'s filter: `(reason, reason_visits, filtered, filtered_visits)`.
    pub fn reason_snapshot() -> [[u64; 16]; 4] {
        fn read(a: &[AtomicU64; 16]) -> [u64; 16] {
            std::array::from_fn(|i| a[i].load(Relaxed))
        }
        [read(&REASON), read(&REASON_VISITS), read(&FILTERED), read(&FILTERED_VISITS)]
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

/// `CRAB_TRIG_CENSUS` — the **activated**-grant walk's census, PERF `(-122)`.
///
/// Compares a board-wide requirement mask built at `grant_scan` time
/// (`grants x board` evaluations) against the per-permanent walk in
/// `granted_abilities_of_inner` (one per asking permanent x grant). Same
/// variable as the trigger census.
#[cfg(feature = "trig-census")]
pub mod grant_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// `grant_scan` calls.
    pub static SCANS: AtomicU64 = AtomicU64::new(0);
    /// …of which carried at least one `EachPermanent` activated grant.
    pub static GRANT_SCANS: AtomicU64 = AtomicU64::new(0);
    /// What a board-wide mask would cost: `grants x battlefield.len()`
    /// requirement evaluations, summed over those scans.
    pub static MASK_EVALS: AtomicU64 = AtomicU64::new(0);
    /// What the walk actually costs: one evaluation per (asking permanent x
    /// grant), counted at the two `evaluate_requirement_static_on` sites in
    /// `granted_abilities_of_inner`.
    pub static WALK_EVALS: AtomicU64 = AtomicU64::new(0);

    /// One `grant_scan`: how many `EachPermanent` grants it found and how big
    /// the board was.
    #[cold]
    #[inline(never)]
    pub fn scan(grants: usize, board: usize) {
        SCANS.fetch_add(1, Relaxed);
        if grants > 0 {
            GRANT_SCANS.fetch_add(1, Relaxed);
            MASK_EVALS.fetch_add((grants * board) as u64, Relaxed);
        }
    }

    #[cold]
    #[inline(never)]
    pub fn walk(evals: usize) {
        WALK_EVALS.fetch_add(evals as u64, Relaxed);
    }

    /// `(scans, grant_scans, mask_evals, walk_evals)`.
    pub fn snapshot() -> [u64; 4] {
        [
            SCANS.load(Relaxed),
            GRANT_SCANS.load(Relaxed),
            MASK_EVALS.load(Relaxed),
            WALK_EVALS.load(Relaxed),
        ]
    }
}

/// `CRAB_TRIG_CENSUS` — the requirement walker's recursion census, PERF
/// `(-126)`.
///
/// Counts the *shape* of `evaluate_requirement_static_hinted`'s
/// `And` / `Or` / `Not` recursion (leaf vs nested children) — what flattening
/// or an inline leaf fast path could serve; callgrind can't split a recursive
/// edge by callee discriminant. Same variable as the censuses above.
#[cfg(feature = "trig-census")]
pub mod req_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// Entries to the walker, all depths.
    pub static CALLS: AtomicU64 = AtomicU64::new(0);
    /// …of which took an `And` / `Or` / `Not` arm.
    pub static COMBINATORS: AtomicU64 = AtomicU64::new(0);
    /// Recursive calls those arms actually made (short-circuits excluded).
    pub static CHILD_CALLS: AtomicU64 = AtomicU64::new(0);
    /// …of which had a leaf child — the population an inline leaf fast path
    /// in the combinator arms could serve.
    pub static LEAF_CHILD_CALLS: AtomicU64 = AtomicU64::new(0);
    /// Combinator arms whose child is *itself* a combinator — the population a
    /// flattened `All`/`Any` slice variant could collapse.
    pub static NESTED_CHILD_CALLS: AtomicU64 = AtomicU64::new(0);

    #[cold]
    #[inline(never)]
    pub fn call() {
        CALLS.fetch_add(1, Relaxed);
    }

    #[cold]
    #[inline(never)]
    pub fn combinator() {
        COMBINATORS.fetch_add(1, Relaxed);
    }

    /// One recursive call about to be made, tagged by whether the child is
    /// itself a combinator.
    #[cold]
    #[inline(never)]
    pub fn child(nested: bool) {
        CHILD_CALLS.fetch_add(1, Relaxed);
        if nested {
            NESTED_CHILD_CALLS.fetch_add(1, Relaxed);
        } else {
            LEAF_CHILD_CALLS.fetch_add(1, Relaxed);
        }
    }

    /// `(calls, combinators, child_calls, leaf_children, nested_children)`.
    pub fn snapshot() -> [u64; 5] {
        [
            CALLS.load(Relaxed),
            COMBINATORS.load(Relaxed),
            CHILD_CALLS.load(Relaxed),
            LEAF_CHILD_CALLS.load(Relaxed),
            NESTED_CHILD_CALLS.load(Relaxed),
        ]
    }
}

/// PERF `(-129)`: which factor of (pairs x batch) drives
/// `event_matches_spec`'s call count, and how many calls come from
/// (permanent, trigger) pairs no event in the batch could match — what a
/// per-pair kind gate would remove. Counters sit outside the
/// `for ev in events` loop so the census doesn't measure itself.
#[cfg(feature = "trig-census")]
pub mod ems_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// Dispatches that reached the battlefield walk, and the events they
    /// carried — the mean batch size is the ratio.
    pub static DISPATCHES: AtomicU64 = AtomicU64::new(0);
    pub static BATCH: AtomicU64 = AtomicU64::new(0);
    /// Distinct `EventKind` discriminants per dispatch's triggers, summed —
    /// the cost of a per-dispatch kind memo, against `PAIRS` it would serve.
    pub static KINDS: AtomicU64 = AtomicU64::new(0);
    /// (permanent, trigger) pairs that reached the `for ev in events` loop.
    pub static PAIRS: AtomicU64 = AtomicU64::new(0);
    /// …of which no event in the batch could match, by the same `source:
    /// None` over-approximation the grant pre-filters use.
    pub static DEAD_PAIRS: AtomicU64 = AtomicU64::new(0);
    /// `event_matches_spec` calls the loop made, and the share of them a dead
    /// pair made — the ceiling on the gate.
    pub static CALLS: AtomicU64 = AtomicU64::new(0);
    pub static DEAD_CALLS: AtomicU64 = AtomicU64::new(0);

    /// One dispatch: its batch size and the distinct kinds its walk presented.
    pub fn dispatch(batch: u64, kinds: u64) {
        DISPATCHES.fetch_add(1, Relaxed);
        BATCH.fetch_add(batch, Relaxed);
        KINDS.fetch_add(kinds, Relaxed);
    }

    /// One (permanent, trigger) pair about to run the event loop.
    pub fn pair(batch: u64, dead: bool) {
        PAIRS.fetch_add(1, Relaxed);
        CALLS.fetch_add(batch, Relaxed);
        if dead {
            DEAD_PAIRS.fetch_add(1, Relaxed);
            DEAD_CALLS.fetch_add(batch, Relaxed);
        }
    }

    /// `(dispatches, batch, kinds, pairs, dead_pairs, calls, dead_calls)`.
    pub fn snapshot() -> [u64; 7] {
        [
            DISPATCHES.load(Relaxed),
            BATCH.load(Relaxed),
            KINDS.load(Relaxed),
            PAIRS.load(Relaxed),
            DEAD_PAIRS.load(Relaxed),
            CALLS.load(Relaxed),
            DEAD_CALLS.load(Relaxed),
        ]
    }
}

/// Slots in [`Battlefield`]'s find memo, direct-mapped by `CardId`. **One
/// slot thrashes**: callers alternate between two or three permanents
/// (attacker and blocker), so each ask evicted the next one's answer. 16 slots
/// cost 60 bytes against a ~15-25 permanent board.
const FIND_HINTS: usize = 16;

/// The memo slot a `CardId` maps to. Ids come off a monotonic counter, so the
/// low bits are the spread-out end of them.
#[inline]
const fn hint_slot(id: crate::card::CardId) -> usize {
    (id.0 as usize) & (FIND_HINTS - 1)
}

/// The battlefield: the CoW card list plus memoized whole-board questions,
/// one two-bit lane each (see the `LANE_*` constants) — layer-4 type gates,
/// damage scaling / prevention / shield gates, the dispatcher's board scan.
///
/// The freeze-scope `PresenceGate` can't serve the half of the asks that
/// arrive holding `&mut self` (CR 602.5 tap gate, the bot's
/// `available_mana`), and `CRAB_GATE_CENSUS` (pass 97) found the inputs
/// unchanged on ~94 % of asks (PERF `(-87)`), so the answer lives on the zone
/// and outlives the scope.
///
/// **Invalidation is two disjoint questions.** Every lane is a function of
/// the multiset of *definitions* on the battlefield
/// (`card_can_change_*_types` deliberately reads no instance field;
/// `dispatch_scan_bits` is definition-pure).
///
/// * *Membership* changes only through [`DerefMut`] or the inherent
///   [`push`](Self::push), and both clear the lanes (`Vec`'s mutators go
///   through `DerefMut`).
/// * *A definition rewrite in place* is caught by stamping each lane with
///   [`definition_epoch`](crate::card::definition_epoch), which the two
///   definition-rewriting accessors bump.
///
/// So [`iter_mut`](Self::iter_mut), [`get_mut`](Self::get_mut) and the `&mut`
/// [`IntoIterator`] — the tap, damage, counter and untap paths — leave the
/// lanes alone.
#[derive(Debug)]
pub struct Battlefield {
    cards: CowBox<Vec<CardInstance>>,
    /// Two-bit lanes; see the `LANE_*` constants.
    type_gates: std::sync::atomic::AtomicU64,
    /// The `definition_epoch` the lanes were computed at. A lane is valid only
    /// while this still matches.
    def_epoch: std::sync::atomic::AtomicU64,
    /// The indices [`find_by_id`](Self::find_by_id) last returned, one slot
    /// per `CardId` low-bit class. Needs no invalidation (see that function).
    find_hints: [std::sync::atomic::AtomicU32; FIND_HINTS],
    /// Which battlefield *indices* satisfy [`card_is_triggerer`], one bit
    /// each. Read only while [`LANE_TRIGGERER`] says it is computed, so it
    /// inherits the lanes' invalidation.
    trig_members: std::sync::atomic::AtomicU64,
    /// The dispatch scan's member list (PERF `(-215)`): the indices whose
    /// definition carries a `BOARD_SCAN` bit, valid while [`LANE_DISPATCH`]
    /// reads `PRESENT`. Same contract as `trig_members`.
    dispatch_members: std::sync::atomic::AtomicU64,
    /// Which indices satisfy [`card_has_any_grant_bits`], on the same
    /// contract under [`LANE_GRANT`].
    grant_members: std::sync::atomic::AtomicU64,
    /// How many times anything took `&mut` at this zone — the per-object twin
    /// of [`BATTLEFIELD_REACHES`], bumped at the same six chokepoints.
    ///
    /// The stamp for memos that read *instance* fields (tapped, damage,
    /// counters, `attached_to`), which the definition-keyed lanes can't
    /// serve: an equal stamp means no `&mut` reached the cards since. Starts
    /// at 1 so 0 is "never stamped".
    ///
    /// **Per-object, not global, by design**: a bot probe mutates a clone and
    /// leaves the parent untouched; a global counter would charge the
    /// clone's writes to the parent.
    ///
    /// Plain `u32`: writers hold `&mut self`, readers `&self` on the same
    /// thread. 32 bits for `GameState`'s size guard; a false hit needs
    /// exactly 2^32 reaches between a store and a read (a game made ~10^6).
    writes: u32,
    /// The `writes` value `sba_fold` was computed at; 0 means "not computed"
    /// (the counter starts at 1).
    sba_stamp: std::sync::atomic::AtomicU32,
    /// [`sba_fold`](Self::sba_fold)'s packed answer.
    sba_fold: std::sync::atomic::AtomicU64,
    /// [`attach_fold`](Self::attach_fold): its one-bit answer in bit 31, the
    /// `writes` it was folded at in bits 0-30, and **0 means "not computed"**
    /// (the counter starts at 1).
    ///
    /// One word rather than stamp plus byte for the size guard
    /// (`cow::tests::game_state_stays_small`). Its own stamp, not
    /// `sba_stamp`: the folds are taken at different moments, so a shared
    /// stamp would let either validate the other. 31 bits keeps `writes`'
    /// wraparound argument.
    attach_fold: std::sync::atomic::AtomicU32,
}

impl Battlefield {
    /// The permanent with `id`, through a [`FIND_HINTS`]-slot index memo
    /// direct-mapped by id (PERF `(-38)`). Callers such as the combat
    /// resolver ask about the same few permanents repeatedly.
    ///
    /// **No invalidation: the answer is verified, not trusted.**
    /// `cards[hint].id == id` is the whole contract — a stale index, shorter
    /// board or different permanent all fall through to the scan — so this
    /// sits outside the lanes' epoch/clear protocol.
    ///
    /// A hit does rest on **battlefield ids being unique** (a hint may name a
    /// later match than a scan would). That's a global invariant, audited by
    /// the scan path's `debug_assert!` (fired by
    /// `scripts/robustness_grid.sh`); auditing on the hit path would put a
    /// board walk in front of the memo.
    ///
    /// Break-even is a repeat rate of about a tenth against a ~58 Ir scan.
    #[inline]
    pub fn find_by_id(&self, id: crate::card::CardId) -> Option<&CardInstance> {
        let slot = hint_slot(id);
        let hint = self.find_hints[slot].load(Ordering::Relaxed) as usize;
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
        self.find_hints[slot].store(i as u32, Ordering::Relaxed);
        Some(c)
    }

    /// The `&mut` twin of [`find_by_id`](Self::find_by_id). Not
    /// `iter_mut().find(…)`: that takes `&mut` **before** knowing the id is
    /// here, paying a [`CowBox`](crate::cow::CowBox) unshare (a deep copy of
    /// the zone) even for `None`. Locates on the shared side instead; same
    /// uniqueness premise and audit as `find_by_id`.
    #[inline]
    pub fn find_by_id_mut(&mut self, id: crate::card::CardId) -> Option<&mut CardInstance> {
        let slot = hint_slot(id);
        let hint = self.find_hints[slot].load(Ordering::Relaxed) as usize;
        let i = if self.cards.get(hint).is_some_and(|c| c.id == id) {
            hint
        } else {
            let i = self.cards.iter().position(|c| c.id == id)?;
            debug_assert!(
                !self.cards[i + 1..].iter().any(|o| o.id == id),
                "two battlefield permanents share a CardId: the find hint could name either",
            );
            self.find_hints[slot].store(i as u32, Ordering::Relaxed);
            i
        };
        self.cards_unchecked_mut().get_mut(i)
    }

    /// True when some permanent's definition satisfies `walk` — the layer-4
    /// card-type contributor test, definition half. Memoized; a miss walks
    /// the board once.
    #[inline]
    pub fn has_card_type_changer(
        &self,
        walk: impl Fn(&CardInstance) -> bool + Copy,
    ) -> bool {
        self.lane(LANE_CARD_TYPE, walk)
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

    /// CR 613.4 layer 7 — can any permanent here contribute a
    /// *toughness-lowering* modification (`GameState::pt_reduction_in_scope`'s
    /// board half)? Asked once per state-based sweep, over the whole board.
    #[inline]
    pub fn has_toughness_reducer(&self, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        self.lane(LANE_PT_REDUCE, walk)
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

    /// CR 615 — can any permanent here carry a whole-board damage shield?
    /// The board half of nine `GameState` walks (Iroas, Glacial Chasm, Mark
    /// of Asylum, The Wanderer, Emmara Tandris, Rune-Tail's Essence,
    /// Well-Laid Plans, Light of Sanction, Indentured Oaf). The predicate is
    /// their union; each walk's own filter is an instance field applied after.
    #[inline]
    pub fn has_damage_shield(&self, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        self.lane(LANE_SHIELD, walk)
    }

    /// Does any permanent here carry a printed gate keyword
    /// ([`card_has_gate_keyword`])? Definition-only, so the lane holds it
    /// across the tap / damage / counter writes that a scope memo cannot.
    #[inline]
    pub fn has_gate_keyword(&self) -> bool {
        self.lane(LANE_GATE_KEYWORD, card_has_gate_keyword)
    }

    /// Does any permanent here carry a draw-replacement static
    /// ([`card_has_draw_static`])? Read once per draw by `draw_one`, in
    /// front of its eleven board walks (PERF `(-233)`).
    #[inline]
    pub fn has_draw_static(&self) -> bool {
        self.lane(LANE_DRAW_STATIC, card_has_draw_static)
    }

    /// Does any permanent here carry a static that changes how another
    /// permanent enters ([`card_has_etb_static`])? Read once per entering
    /// permanent by `etb_trigger_multiplier` and
    /// `apply_enters_tapped_replacement` (PERF `(-234)`).
    #[inline]
    pub fn has_etb_static(&self) -> bool {
        self.lane(LANE_ETB_STATIC, card_has_etb_static)
    }

    /// Does any permanent here carry a damage-replacing or -redirecting static
    /// ([`card_has_damage_replacement_static`])? Read once per damage event by
    /// `deal_damage_to_from` and `damage_redirect_target` (PERF `(-235)`).
    #[inline]
    pub fn has_damage_replacement_static(&self) -> bool {
        self.lane(LANE_DAMAGE_STATIC, card_has_damage_replacement_static)
    }

    /// Does any permanent here carry a land-play lock or extension
    /// ([`card_has_land_play_static`])? Read by `can_player_play_land`,
    /// `extra_land_plays_per_turn` and `damping_engine_locks` (PERF `(-236)`).
    #[inline]
    pub fn has_land_play_static(&self) -> bool {
        self.lane(LANE_LAND_PLAY_STATIC, card_has_land_play_static)
    }

    /// Does any permanent here let mana be spent as any colour
    /// ([`card_has_any_color_static`])? Read once per payment by the cost
    /// relaxation (PERF `(-237)`).
    #[inline]
    pub fn has_any_color_static(&self) -> bool {
        self.lane(LANE_ANY_COLOR_STATIC, card_has_any_color_static)
    }

    /// Does any permanent here change a maximum hand size
    /// ([`card_has_hand_size_static`])? Read once per `effective_max_hand_size`
    /// call, in front of its four walks (PERF `(-238)`).
    #[inline]
    pub fn has_hand_size_static(&self) -> bool {
        self.lane(LANE_HAND_SIZE_STATIC, card_has_hand_size_static)
    }

    /// Does any permanent here put counters on another permanent as it
    /// enters ([`card_has_etb_counter_static`])? Read per resolving permanent
    /// spell and per entering creature (PERF `(-239)`); the command-zone term
    /// beside it is the caller's.
    #[inline]
    pub fn has_etb_counter_static(&self) -> bool {
        self.lane(LANE_ETB_COUNTER_STATIC, card_has_etb_counter_static)
    }

    /// Does any permanent here carry a damage-prevention static
    /// ([`card_has_prevent_static`])? Read once per damage event by
    /// `prevent_static_scan` (PERF `(-240)`).
    #[inline]
    pub fn has_prevent_static(&self) -> bool {
        self.lane(LANE_PREVENT_STATIC, card_has_prevent_static)
    }

    /// Does any permanent here charge a block tax
    /// ([`card_has_block_tax_static`])? Read per declared blocker by
    /// `block_tax_for` and by the bot's `block_tax_present` (PERF `(-241)`).
    #[inline]
    pub fn has_block_tax_static(&self) -> bool {
        self.lane(LANE_BLOCK_TAX_STATIC, card_has_block_tax_static)
    }

    /// Does any permanent here cap combat participation
    /// ([`card_has_combat_cap_static`])? Read by `combat_participation_cap`,
    /// whose callers are all outside a freeze scope — hence a lane, not a
    /// presence slot (PERF `(-324)`).
    #[inline]
    pub fn has_combat_cap_static(&self) -> bool {
        self.lane(LANE_COMBAT_CAP, card_has_combat_cap_static)
    }

    /// Does any permanent here carry a static the untap step reads
    /// ([`card_has_untap_static`])? Read once per `do_untap`, which is once
    /// per turn on every simulation clone too (PERF `(-249)`).
    #[inline]
    pub fn has_untap_static(&self) -> bool {
        self.lane(LANE_UNTAP_STATIC, card_has_untap_static)
    }

    /// Does any permanent here carry a life-arithmetic static
    /// ([`card_has_life_static`])? Read by `adjust_life`'s seven helpers in
    /// front of their board walks, on every life change (PERF `(-262)`).
    #[inline]
    pub fn has_life_static(&self) -> bool {
        self.lane(LANE_LIFE_STATIC, card_has_life_static)
    }

    /// Does any permanent here carry an attachment- or pairing-scoped grant of
    /// activated abilities ([`card_has_attach_grant`])? Read once per
    /// `grant_scan`, in front of the walk that collects the Equipment and
    /// Soulbond halves (PERF `(-315)`).
    #[inline]
    pub fn has_attach_grant(&self) -> bool {
        self.lane(LANE_ATTACH_GRANT, crate::game::actions::card_has_attach_grant)
    }

    /// Does any permanent here let its controller activate a summoning-sick
    /// creature's abilities as though it had haste
    /// ([`card_has_haste_static`])? Read by `tap_ability_summoning_sick`
    /// (PERF `(-316)`).
    #[inline]
    pub fn has_haste_static(&self) -> bool {
        self.lane(LANE_HASTE_STATIC, card_has_haste_static)
    }

    /// One lane's answer: a word load on a hit, the board walk plus one store
    /// on a miss.
    ///
    /// **`walk` is `Copy` and passed to `any` by value, never by reference.**
    /// `&F: FnMut` goes through a non-inlining `call_mut` shim, which gave
    /// back over half the memo's win when first built (ninety-eighth pass).
    ///
    /// The `debug_assert!` audits that no write reached the cards without
    /// clearing the word; `scripts/robustness_grid.sh` fires it on boards the
    /// suite never deals.
    #[inline]
    fn lane(&self, shift: u32, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        let stale = self.def_epoch.load(Ordering::Relaxed)
            != crate::card::definition_epoch();
        let cur = if stale {
            UNKNOWN as u64
        } else {
            (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK
        };
        debug_assert!(
            cur == UNKNOWN as u64
                || board_audit(|| (cur == PRESENT as u64) == self.cards.iter().any(walk)),
            "battlefield type-gate memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            c if c == ABSENT as u64 => false,
            c if c == PRESENT as u64 => true,
            _ => self.walk_and_store(shift, walk),
        }
    }

    /// `iter_mut` that does **not** invalidate — the tap / damage / counter /
    /// untap paths. Shadows the `Deref`'d `Vec::iter_mut`. Sound because the
    /// lanes read only definitions, and a definition rewrite through these
    /// handles bumps `definition_epoch`.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, CardInstance> {
        self.cards_unchecked_mut().iter_mut()
    }

    /// `get_mut` on the same contract as [`iter_mut`](Self::iter_mut).
    #[inline]
    pub fn get_mut(&mut self, i: usize) -> Option<&mut CardInstance> {
        self.cards_unchecked_mut().get_mut(i)
    }

    /// `&mut` at the cards **without** clearing the lanes. Private: only the
    /// element-write accessors, which never move membership, call it. Public
    /// mutation goes through [`DerefMut`], which clears.
    #[inline]
    fn cards_unchecked_mut(&mut self) -> &mut Vec<CardInstance> {
        self.note_write();
        &mut self.cards
    }

    /// One `&mut` reach at this zone: the census tick and the
    /// [`writes`](Self::writes) bump, at all six places that hand out `&mut`
    /// at the cards.
    #[inline]
    fn note_write(&mut self) {
        note_battlefield_reach();
        self.writes = self.writes.wrapping_add(1);
    }

    /// This zone's write counter — see [`writes`](Self::writes).
    #[inline]
    pub fn writes(&self) -> u32 {
        self.writes
    }

    /// A whole-board fold over *instance* fields, memoized on the zone.
    ///
    /// `Some(packed)` is the last stored fold, valid because no `&mut` has
    /// reached the cards since (see [`writes`](Self::writes)). `None` means
    /// "fold, then call [`store_sba_fold`](Self::store_sba_fold)". One
    /// consumer (`GameState::sba_board_scan`), which owns the packing.
    #[inline]
    pub fn sba_fold(&self) -> Option<u64> {
        (self.sba_stamp.load(Ordering::Relaxed) == self.writes)
            .then(|| self.sba_fold.load(Ordering::Relaxed))
    }

    /// Record a fold, valid until the next `&mut` at the cards. Value before
    /// stamp, so a reader that sees the stamp sees a value at least as new.
    #[inline]
    pub fn store_sba_fold(&self, packed: u64) {
        self.sba_fold.store(packed, Ordering::Relaxed);
        self.sba_stamp.store(self.writes, Ordering::Relaxed);
    }

    /// Is any permanent here attached to something, or Soulbond-paired?
    /// `Some(present)` when the answer is this board's — no `&mut` has reached
    /// the cards since it was folded — and `None` means "walk, then hand the
    /// answer to [`store_attach_fold`](Self::store_attach_fold)".
    ///
    /// ⚠ **Both facts read INSTANCE fields, so this is a `writes`-keyed fold
    /// like [`sba_fold`](Self::sba_fold), NOT a `definition_epoch`-keyed
    /// lane**: lanes survive element writes, and an `attached_to` assignment
    /// is one. PERF `(-318)`.
    ///
    /// **One bit, not two**: both halves gate the same walk, so one
    /// short-circuiting `any` keeps the miss path cheap (the pairing half was
    /// ~never present).
    #[inline]
    pub fn attach_fold(&self) -> Option<bool> {
        let w = self.attach_fold.load(Ordering::Relaxed);
        (w != 0 && w & ATTACH_STAMP == self.writes & ATTACH_STAMP)
            .then_some(w & ATTACH_PRESENT != 0)
    }

    /// Record what an attachment walk found, valid until the next `&mut` at
    /// the cards. One store, answer and stamp together.
    #[inline]
    pub fn store_attach_fold(&self, present: bool) {
        let w = (self.writes & ATTACH_STAMP) | if present { ATTACH_PRESENT } else { 0 };
        self.attach_fold.store(w, Ordering::Relaxed);
    }

    /// The miss path, out of line so the hit path stays a word load.
    ///
    /// **Not `#[cold]`**: taken on about half the asks, and cold makes LLVM
    /// optimize for size, which stops `walk` inlining into the loop — that ate
    /// two thirds of the memo's win (ninety-eighth pass). `#[inline(never)]`
    /// keeps one instance per predicate with the predicate inlined.
    #[inline(never)]
    fn walk_and_store(&self, shift: u32, walk: impl Fn(&CardInstance) -> bool) -> bool {
        // Read the epoch *before* the walk so the stamp is conservative: a
        // lane computed across a rewrite is discarded on the next ask.
        let epoch = crate::card::definition_epoch();
        let found = self.cards.iter().any(walk);
        self.store_lane(shift, epoch, found);
        found
    }

    /// Write one lane with what a walk found, stamped at the `epoch` that walk
    /// started from.
    fn store_lane(&self, shift: u32, epoch: u64, found: bool) {
        let lane = if found { PRESENT as u64 } else { ABSENT as u64 };
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
    /// stale word fails loudly under debug assertions; `what` names the lane.
    ///
    /// `what` is a *format argument*, so the lanes share one literal:
    /// `robustness_grid.sh`'s compiled-in check (grepping `memo is stale`)
    /// still sees it, but a per-lane `strings` grep will not.
    #[inline]
    fn split_lane(
        &self,
        shift: u32,
        audit: fn(&CardInstance) -> bool,
        what: &str,
    ) -> Result<bool, u64> {
        let epoch = crate::card::definition_epoch();
        let cur = if self.def_epoch.load(Ordering::Relaxed) != epoch {
            UNKNOWN as u64
        } else {
            (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK
        };
        debug_assert!(
            cur == UNKNOWN as u64
                || board_audit(|| (cur == PRESENT as u64) == self.cards.iter().any(audit)),
            "battlefield {what} memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            c if c == ABSENT as u64 => Ok(false),
            c if c == PRESENT as u64 => Ok(true),
            _ => Err(epoch),
        }
    }

    /// The dispatch lane, for a caller that fills it from a walk it was going
    /// to make anyway: `Ok(present)` is the memo's answer, and `Err(epoch)`
    /// means "unknown — walk, then hand that same epoch back to
    /// [`store_dispatch`](Self::store_dispatch)".
    ///
    /// Split read/store rather than the closure form: the dispatcher's pass
    /// also builds two grant lists, so a closure would walk the board twice on
    /// every miss, about break-even at the hit rates PERF `(-87)` measured.
    pub fn dispatch_lane(&self) -> Result<bool, u64> {
        self.dispatch_members().map(|bits| bits != 0)
    }

    /// The dispatch scan's **member list** (PERF `(-215)`): `Ok(bits)`
    /// names the battlefield indices whose definition carries a
    /// `BOARD_SCAN` bit ([`card_has_dispatch_bits`]), so a scan visits only
    /// those; `Err(epoch)` means "unknown — walk, then hand that same epoch
    /// to [`store_dispatch_members`](Self::store_dispatch_members)". Kept
    /// exact through membership writes like the other two lists.
    pub fn dispatch_members(&self) -> Result<u64, u64> {
        let epoch = crate::card::definition_epoch();
        let known = self.def_epoch.load(Ordering::Relaxed) == epoch
            && (self.type_gates.load(Ordering::Relaxed) >> LANE_DISPATCH) & LANE_MASK
                == PRESENT as u64;
        if !known {
            return Err(epoch);
        }
        let bits = self.dispatch_members.load(Ordering::Relaxed);
        debug_assert!(
            board_audit(|| bits == dispatch_bits(&self.cards)),
            "battlefield dispatch memo is stale: a write reached the cards without clearing it",
        );
        Ok(bits)
    }

    /// Record the member list a caller's own full walk built, stamped at the
    /// `epoch` [`dispatch_members`](Self::dispatch_members) handed out. A
    /// board wider than 64 has no list, so the lane stays unknown there.
    pub fn store_dispatch_members(&self, epoch: u64, bits: u64) {
        if self.cards.len() > 64 {
            return;
        }
        self.dispatch_members.store(bits, Ordering::Relaxed);
        self.store_lane(LANE_DISPATCH, epoch, true);
    }

    /// The keyword-grant lane's **member list**: `Ok(bits)` names the
    /// battlefield indices whose definition can grant a keyword at all
    /// ([`card_has_any_grant_bits`]), `Err(epoch)` means "unknown — walk,
    /// then hand that same epoch to
    /// [`store_grant_members`](Self::store_grant_members)".
    ///
    /// A list, not a presence bit (PERF `(-189)`): `keyword_grant_in_scope`
    /// asks about *one* keyword, and any lord, aura or equipment made a
    /// presence bit read `PRESENT`, so the caller still walked the board. Same
    /// shape and invalidation as [`trigger_members`](Self::trigger_members);
    /// boards wider than 64 have no list.
    pub fn grant_members(&self) -> Result<u64, u64> {
        let epoch = crate::card::definition_epoch();
        let known = self.def_epoch.load(Ordering::Relaxed) == epoch
            && (self.type_gates.load(Ordering::Relaxed) >> LANE_GRANT) & LANE_MASK
                == PRESENT as u64;
        if !known {
            return Err(epoch);
        }
        let bits = self.grant_members.load(Ordering::Relaxed);
        debug_assert!(
            board_audit(|| bits == grant_bits(&self.cards)),
            "battlefield grant memo is stale: a write reached the cards without clearing it",
        );
        Ok(bits)
    }

    /// Record the member list a caller's own full walk built, stamped at the
    /// `epoch` [`grant_members`](Self::grant_members) handed out. A board
    /// wider than 64 has no list, so the lane stays unknown there.
    pub fn store_grant_members(&self, epoch: u64, bits: u64) {
        if self.cards.len() > 64 {
            return;
        }
        self.grant_members.store(bits, Ordering::Relaxed);
        self.store_lane(LANE_GRANT, epoch, true);
    }

    /// The combat-damage listener lane, same caller-filled contract as
    /// [`dispatch_lane`](Self::dispatch_lane). Split because the gated walk
    /// builds two trigger lists; it can't short-circuit, so its fill is exact.
    pub fn listener_lane(&self) -> Result<bool, u64> {
        self.split_lane(LANE_LISTENER, card_has_listener_bits, "listener")
    }

    /// Record what the caller's own walk found in the listener lane.
    pub fn store_listener(&self, epoch: u64, found: bool) {
        self.store_lane(LANE_LISTENER, epoch, found);
    }

    /// The mana-static lane, same caller-filled contract as
    /// [`dispatch_lane`](Self::dispatch_lane): does any permanent's definition
    /// carry a `MANA_STATIC` static? Read once per mana-ability activation
    /// by `GameState::board_has_mana_static`.
    pub fn mana_static_lane(&self) -> Result<bool, u64> {
        self.split_lane(LANE_MANA_STATIC, card_has_mana_static, "mana-static")
    }

    /// Record what the caller's own walk found in the mana-static lane.
    pub fn store_mana_static(&self, epoch: u64, found: bool) {
        self.store_lane(LANE_MANA_STATIC, epoch, found);
    }

    /// Does any permanent's definition grant activated abilities to countered
    /// creatures (Agatha's Soul Cauldron)? `walk` is
    /// `actions::card_grants_to_countered`; asked lazily by the grants-nothing
    /// gate so a board without one never reads counter bags (PERF `(-199)`).
    #[inline]
    pub fn has_counter_granter(&self, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        self.lane(LANE_COUNTER_GRANT, walk)
    }

    /// Does any permanent's definition carry a graveyard-redirecting static?
    /// `walk` is the engine's memo-word read (`actions::card_redirects_deaths`);
    /// asked once per death and per graveyard placement (PERF `(-203)`).
    #[inline]
    pub fn has_death_redirect(&self, walk: impl Fn(&CardInstance) -> bool + Copy) -> bool {
        self.lane(LANE_DEATH_REDIRECT, walk)
    }

    /// The activated-grant lane, same caller-filled contract as
    /// [`dispatch_lane`](Self::dispatch_lane). Split because `grant_scan`'s
    /// walk builds a list and evaluates CR 611.2 gates rather than the lane's
    /// predicate; it already iterates `static_abilities`, so the fill is cheap.
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
    /// Answers *which*, not *is there one*, so a hit skips the walk. Unlike
    /// the forty-eighth pass's refuted trigger-carrier mask (rebuilt on every
    /// dispatch), these bits outlive the dispatch and are filled by the walk a
    /// miss makes anyway.
    pub fn trigger_members(&self) -> Result<u64, u64> {
        let epoch = crate::card::definition_epoch();
        let known = self.def_epoch.load(Ordering::Relaxed) == epoch
            && (self.type_gates.load(Ordering::Relaxed) >> LANE_TRIGGERER) & LANE_MASK
                == PRESENT as u64;
        if !known {
            return Err(epoch);
        }
        let bits = self.trig_members.load(Ordering::Relaxed);
        debug_assert!(
            board_audit(|| bits == triggerer_bits(&self.cards)),
            "battlefield triggerer memo is stale: a write reached the cards without clearing it",
        );
        Ok(bits)
    }

    /// Record the member list a caller's own full walk built, stamped at the
    /// `epoch` [`trigger_members`](Self::trigger_members) handed out. A board
    /// wider than 64 has no list (the lane stays unknown); the caller must not
    /// build bits for one either.
    pub fn store_trigger_members(&self, epoch: u64, bits: u64) {
        if self.cards.len() > 64 {
            return;
        }
        self.trig_members.store(bits, Ordering::Relaxed);
        self.store_lane(LANE_TRIGGERER, epoch, true);
    }

    /// Visit every permanent that can carry a printed trigger or a Station
    /// band — the [`trigger_members`](Self::trigger_members) list on a hit,
    /// the whole board on a miss (this never fills the list) — or the whole
    /// board when `all` (a grant may give a trigger to anything). One call
    /// site for `f`, not an `if` over two walks: two generic walks left the
    /// closure out of line (+0.24 % sealed at `4db20c51`).
    #[inline]
    pub fn for_each_triggerer_or_all(&self, all: bool, mut f: impl FnMut(&CardInstance)) {
        let n = self.cards.len();
        // One bit loop, one call site: "all" is a full mask per 64-card
        // chunk. A second `f` call site is a second inlining decision LLVM
        // declined.
        let (whole, mut bits) = match self.trigger_members() {
            Ok(bits) if !all => (false, bits),
            _ => (true, 0),
        };
        let mut base = 0usize;
        loop {
            if whole {
                let left = n - base;
                if left == 0 {
                    break;
                }
                bits = u64::MAX >> (64 - left.min(64));
            }
            while bits != 0 {
                let i = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if let Some(c) = self.cards.get(base + i) {
                    f(c);
                }
            }
            base += 64;
            if !whole || base >= n {
                break;
            }
        }
    }

    /// Append via [`CowBox`]'s `push` so an unshare materializes with room
    /// (PERF `(-76)`); shadows the `Deref`'d `Vec::push`.
    ///
    /// **An addition can only turn a lane `PRESENT`**, decided off the new
    /// card alone: `PRESENT` lanes stay, `ABSENT` lanes are re-answered by
    /// [`LANE_PREDICATES`], member lists are extended (PERF `(-212)`..`(-214)`).
    pub fn push(&mut self, mut card: CardInstance) {
        card.entered_from_library = card.in_library;
        card.in_library = false;
        self.note_write();
        self.lanes_after_push(&card);
        self.cards.push(card);
    }

    /// `Vec::remove`, shadowed: **a removal can only turn a lane `ABSENT`**,
    /// and only via a witness — a `PRESENT` lane drops to `UNKNOWN` iff the
    /// removed card passes its predicate (PERF `(-212)`, `(-213)`). Every
    /// death-path removal goes through here or [`take_by_id`](Self::take_by_id).
    pub fn remove(&mut self, index: usize) -> CardInstance {
        self.note_write();
        let card = self.cards.remove(index);
        self.lanes_after_removal(Some((index, &card)));
        card
    }

    /// The card with `id`, taken off the battlefield — `remove` by id, with
    /// the same lane contract.
    pub fn take_by_id(&mut self, id: crate::card::CardId) -> Option<CardInstance> {
        let pos = self.cards.iter().position(|c| c.id == id)?;
        Some(self.remove(pos))
    }

    /// `Vec::retain`, shadowed: removal of cards the zone doesn't see, so
    /// `PRESENT` lanes and member lists drop and `ABSENT` lanes stay.
    pub fn retain(&mut self, f: impl FnMut(&CardInstance) -> bool) {
        self.note_write();
        self.lanes_after_removal(None);
        self.cards.retain(f);
    }

    /// `Vec::pop`, shadowed, on the removal contract.
    pub fn pop(&mut self) -> Option<CardInstance> {
        self.note_write();
        let card = self.cards.pop();
        self.lanes_after_removal(card.as_ref().map(|c| (self.cards.len(), c)));
        card
    }

    /// The member-list lanes as `(lane, predicate, list)`, so membership
    /// writes keep each list exact (PERF `(-214)`): a push sets bit `len` if
    /// the card qualifies, a removal at `i` shifts higher bits down. The
    /// predicate is the list audit's.
    fn member_lanes(&self) -> [(u32, LanePredicate, &std::sync::atomic::AtomicU64); 3] {
        [
            (LANE_GRANT, card_has_any_grant_bits, &self.grant_members),
            (LANE_TRIGGERER, card_is_triggerer, &self.trig_members),
            (LANE_DISPATCH, card_has_dispatch_bits, &self.dispatch_members),
        ]
    }

    /// The lane word after `card` joined: `ABSENT` lanes re-answered off
    /// the card, `PRESENT` lanes kept, member lists extended by the card's
    /// bit (or dropped past 64 cards). A word stamped at another definition
    /// epoch is dead already and is cleared whole.
    #[inline(never)]
    fn lanes_after_push(&self, card: &CardInstance) {
        let w = self.type_gates.load(Ordering::Relaxed);
        if w == 0 {
            return;
        }
        if self.def_epoch.load(Ordering::Relaxed) != crate::card::definition_epoch() {
            self.type_gates.store(0, Ordering::Relaxed);
            return;
        }
        let mut out = w;
        let index = self.cards.len();
        for (shift, pred, list) in self.member_lanes() {
            if (w >> shift) & LANE_MASK != PRESENT as u64 {
                continue;
            }
            if index >= 64 {
                out &= !(LANE_MASK << shift);
            } else if pred(card) {
                list.store(list.load(Ordering::Relaxed) | (1 << index), Ordering::Relaxed);
            }
        }
        for (i, pred) in LANE_PREDICATES.iter().enumerate() {
            let shift = 2 * i as u32;
            if (w >> shift) & LANE_MASK != ABSENT as u64 {
                continue;
            }
            if let Some(p) = pred
                && p(card)
            {
                out = (out & !(LANE_MASK << shift)) | ((PRESENT as u64) << shift);
            }
        }
        self.type_gates.store(out, Ordering::Relaxed);
    }

    /// The lane word after a removal: `ABSENT` lanes kept, `PRESENT` lanes
    /// kept iff the leaver fails their predicate, member lists shifted down
    /// past `index` — all dropped when the leaver is unknown (`retain`).
    #[inline(never)]
    fn lanes_after_removal(&self, removed: Option<(usize, &CardInstance)>) {
        let w = self.type_gates.load(Ordering::Relaxed);
        if w == 0 {
            return;
        }
        if self.def_epoch.load(Ordering::Relaxed) != crate::card::definition_epoch() {
            self.type_gates.store(0, Ordering::Relaxed);
            return;
        }
        // Bit 0 of a field is exactly `ABSENT`; member lanes never hold it.
        let mut out = w & LANE_ABSENT_BITS;
        if let Some((index, card)) = removed {
            for (shift, _, list) in self.member_lanes() {
                if (w >> shift) & LANE_MASK == PRESENT as u64 {
                    // `index` <= 63 (lists exist only up to 64 cards);
                    // `checked_shr` avoids the shift-by-64 overflow the
                    // closing grid found.
                    let bits = list.load(Ordering::Relaxed);
                    let low = bits & ((1u64 << index) - 1);
                    let high = bits.checked_shr(index as u32 + 1).unwrap_or(0) << index;
                    list.store(low | high, Ordering::Relaxed);
                    out |= (PRESENT as u64) << shift;
                }
            }
            for (i, pred) in LANE_PREDICATES.iter().enumerate() {
                let shift = 2 * i as u32;
                if (w >> shift) & LANE_MASK == PRESENT as u64 && pred.is_some_and(|p| !p(card)) {
                    out |= (PRESENT as u64) << shift;
                }
            }
        }
        self.type_gates.store(out, Ordering::Relaxed);
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
            c if c == ABSENT as u64 => Some(false),
            c if c == PRESENT as u64 => Some(true),
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
            type_gates: std::sync::atomic::AtomicU64::new(
                self.type_gates.load(Ordering::Relaxed),
            ),
            def_epoch: std::sync::atomic::AtomicU64::new(
                self.def_epoch.load(Ordering::Relaxed),
            ),
            find_hints: std::array::from_fn(|i| {
                std::sync::atomic::AtomicU32::new(self.find_hints[i].load(Ordering::Relaxed))
            }),
            trig_members: std::sync::atomic::AtomicU64::new(
                self.trig_members.load(Ordering::Relaxed),
            ),
            dispatch_members: std::sync::atomic::AtomicU64::new(
                self.dispatch_members.load(Ordering::Relaxed),
            ),
            grant_members: std::sync::atomic::AtomicU64::new(
                self.grant_members.load(Ordering::Relaxed),
            ),
            // The clone shares these cards until a write bumps the *clone's*
            // counter, so memos stamped before the clone stay valid for it
            // and a probe starts warm.
            writes: self.writes,
            sba_stamp: std::sync::atomic::AtomicU32::new(self.sba_stamp.load(Ordering::Relaxed)),
            sba_fold: std::sync::atomic::AtomicU64::new(self.sba_fold.load(Ordering::Relaxed)),
            attach_fold: std::sync::atomic::AtomicU32::new(
                self.attach_fold.load(Ordering::Relaxed),
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
        self.note_write();
        self.type_gates.store(0, Ordering::Relaxed);
        &mut self.cards
    }
}

impl Default for Battlefield {
    fn default() -> Self {
        Self::from(Vec::new())
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
            type_gates: std::sync::atomic::AtomicU64::new(0),
            def_epoch: std::sync::atomic::AtomicU64::new(0),
            find_hints: std::array::from_fn(|_| std::sync::atomic::AtomicU32::new(0)),
            trig_members: std::sync::atomic::AtomicU64::new(0),
            dispatch_members: std::sync::atomic::AtomicU64::new(0),
            grant_members: std::sync::atomic::AtomicU64::new(0),
            writes: 1,
            sba_stamp: std::sync::atomic::AtomicU32::new(0),
            sba_fold: std::sync::atomic::AtomicU64::new(0),
            attach_fold: std::sync::atomic::AtomicU32::new(0),
        }
    }
}

// `for c in &bf` / `for c in &mut bf` — as on `Graveyard`, deref coercion
// doesn't apply to `for` loops.
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

    /// Riftstone Portal is the only `GrantActivatedAbilityFromGraveyard`
    /// shape, so the lane is invisible to the suite unless a test deals it.
    /// Both lanes share one packed word and neither store disturbs the other.
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

    /// CR 704.5d's graveyard lane: instance state, sound because only a
    /// write can change the answer.
    #[test]
    fn the_graveyard_token_lane_reads_instance_state() {
        let mut g = gy(vec![crate::catalog::shivan_dragon()]);
        assert!(!g.has_token());
        assert_eq!(g.memo_lane(GY_LANE_TOKEN), Some(false));
        assert_eq!(g.memo_lane(GY_LANE_ANTHEM), None, "the other lanes are untouched");
        g.push(CardInstance::new_token(CardId(99), crate::catalog::shivan_dragon(), 0));
        assert_eq!(g.memo_lane(GY_LANE_TOKEN), None, "the push invalidated");
        assert!(g.has_token());
        g.retain(|c| !c.is_token);
        assert!(!g.has_token(), "the retain invalidated through DerefMut");
    }

    fn pile(tokens: usize, plain: usize) -> CardPile {
        let mut v: Vec<CardInstance> = (0..plain)
            .map(|i| CardInstance::new(CardId(i as u32), crate::catalog::shivan_dragon(), 0))
            .collect();
        v.extend((0..tokens).map(|i| {
            CardInstance::new_token(CardId(1000 + i as u32), crate::catalog::shivan_dragon(), 0)
        }));
        v.into()
    }

    /// The pile's one lane: lazy, then remembered, and `false` on the pile
    /// every library actually is.
    #[test]
    fn pile_token_presence_is_detected_and_memoized() {
        let empty_of_tokens = pile(0, 3);
        assert_eq!(empty_of_tokens.memo(), None, "lazy until asked");
        assert!(!empty_of_tokens.has_token());
        assert_eq!(empty_of_tokens.memo(), Some(false));

        let with = pile(1, 2);
        assert!(with.has_token());
        assert_eq!(with.memo(), Some(true));
    }

    /// Every route to the cards invalidates; no read does.
    #[test]
    fn pile_writes_invalidate_and_reads_do_not() {
        let mut p = pile(0, 2);
        assert!(!p.has_token());
        assert_eq!(p.iter().count(), 2);
        assert_eq!((&p).into_iter().count(), 2);
        assert_eq!(p.memo(), Some(false), "a read does not invalidate");

        p.push(CardInstance::new_token(CardId(7), crate::catalog::shivan_dragon(), 0));
        assert_eq!(p.memo(), None, "push invalidated");
        assert!(p.has_token());

        for _ in &mut p {}
        assert_eq!(p.memo(), None, "&mut iteration invalidated");

        p.retain(|c| !c.is_token);
        assert!(!p.has_token(), "DerefMut invalidated and the answer moved");
    }

    /// The encoded lane holds an instance fact, so every `&mut` route must
    /// clear it — including an in-place edit of the flag through `DerefMut`.
    #[test]
    fn pile_encoded_lane_follows_the_instance_flag() {
        let mut p = pile(0, 2);
        assert!(!p.has_encoded());
        assert!(!p.has_token(), "the two lanes share one word");
        p[0].encoded_on = Some(CardId(9));
        assert!(p.has_encoded(), "DerefMut cleared the lane and the walk sees the flag");
        assert!(!p.has_token(), "the other lane refilled to the same answer");
        p[0].encoded_on = None;
        assert!(!p.has_encoded());
        let mut q = p.clone();
        q.push(CardInstance::new_token(CardId(7), crate::catalog::shivan_dragon(), 0));
        assert!(!q.has_encoded(), "push cleared and the new card is not encoded");
        assert!(q.has_token());
    }

    /// The CoW contract survives the pile wrapper too.
    #[test]
    fn pile_clone_shares_and_carries_the_memo() {
        let mut a = pile(1, 1);
        assert!(a.has_token());
        let b = a.clone();
        assert!(a.shares_with(&b), "clone is a refcount bump");
        assert_eq!(b.memo(), Some(true), "the memo describes the same cards");
        a.retain(|c| !c.is_token);
        assert!(!a.shares_with(&b), "the write unshared");
        assert_eq!(b.memo(), Some(true), "the snapshot's answer is still right");
        assert!(!a.has_token());
    }

    /// A pile round-trips through serde as a bare list, so no snapshot
    /// schema moves with the wrapper.
    #[test]
    fn pile_serde_round_trips_as_a_bare_list() {
        let p = pile(1, 1);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.starts_with('['), "a pile serializes as its cards: {json:.40}");
        let back: CardPile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back.memo(), None, "a fresh pile starts unknown");
        assert!(back.has_token());
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

    /// `find_by_id`'s memo is verified, not invalidated: every way it goes
    /// stale (board shifted, shortened, card gone) must fall through to the
    /// scan and still answer correctly.
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
    /// without unsharing; a hit unshares, and the snapshot doesn't see the
    /// write.
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
        let land = crate::game::card_can_change_land_types;
        let creature = crate::game::card_can_change_creature_types;
        let mut b = bf(vec![crate::catalog::grizzly_bears()]);
        assert!(!b.has_land_type_changer(land));
        assert!(!b.has_creature_type_changer(creature));
        assert_eq!(b.memo(LANE_LAND), Some(false));
        assert_eq!(b.memo(LANE_CREATURE), Some(false));

        assert_eq!(b.iter().count(), 1);
        assert_eq!((&b).into_iter().count(), 1);
        assert_eq!(b.memo(LANE_LAND), Some(false), "a read does not invalidate");

        // A push answers each ABSENT lane off the new card (PERF `(-213)`).
        b.push(CardInstance::new(CardId(99), crate::catalog::blood_moon(), 0));
        assert_eq!(b.memo(LANE_LAND), Some(true), "Blood Moon rewrites land types");
        assert_eq!(b.memo(LANE_CREATURE), Some(false), "and not creature types");
        assert!(b.has_land_type_changer(land), "the kept answer audits clean");

        // Element writes do NOT invalidate — that is the whole split.
        for c in &mut b {
            c.tapped = true;
        }
        assert_eq!(b.memo(LANE_LAND), Some(true), "&mut iteration is an element write");
        b.iter_mut().for_each(|c| c.tapped = false);
        assert_eq!(b.memo(LANE_LAND), Some(true), "so is iter_mut");
        assert!(b.get_mut(0).is_some());
        assert_eq!(b.memo(LANE_LAND), Some(true), "so is get_mut");

        // Membership does: a removal drops a PRESENT lane whose witness left
        // (PERF `(-212)`), and any other route through `DerefMut` clears whole.
        b.pop();
        assert_eq!(b.memo(LANE_LAND), None, "the Moon was the witness");
        assert_eq!(b.memo(LANE_CREATURE), Some(false), "an ABSENT lane survives a removal");
        b.insert(0, CardInstance::new(CardId(7), crate::catalog::grizzly_bears(), 0));
        assert_eq!(b.memo(LANE_CREATURE), None, "DerefMut invalidated");
    }

    /// A definition rewrite through an element handle is invisible to the
    /// zone, so the `definition_epoch` stamp must catch it.
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
        let land = crate::game::card_can_change_land_types;
        let mut a = bf(vec![crate::catalog::grizzly_bears()]);
        assert!(!a.has_land_type_changer(land));
        let b = a.clone();
        assert!(a.shares_with(&b), "clone is a refcount bump");
        assert_eq!(b.memo(LANE_LAND), Some(false), "the memo describes the same cards");
        a.push(CardInstance::new(CardId(7), crate::catalog::blood_moon(), 0));
        assert!(!a.shares_with(&b), "the write unshared");
        assert_eq!(b.memo(LANE_LAND), Some(false), "the snapshot's answer is still right");
        assert_eq!(a.memo(LANE_LAND), Some(true), "and the writer's moved with its card");
    }

    /// The dispatch lane is filled by its caller, so its contract is the pair:
    /// an unknown lane hands out an epoch, the store makes the next ask a hit,
    /// and a membership write puts it back to unknown.
    #[test]
    fn dispatch_lane_round_trips_through_its_caller() {
        let mut b = bf(vec![crate::catalog::grizzly_bears()]);
        let Err(epoch) = b.dispatch_lane() else { panic!("a fresh zone is unknown") };
        assert_eq!(dispatch_bits(&b), 0, "Grizzly Bears contributes nothing to the dispatch scan");
        b.store_dispatch_members(epoch, dispatch_bits(&b));
        assert_eq!(b.dispatch_lane(), Ok(false));
        assert_eq!(b.memo(LANE_LAND), None, "the other lanes are untouched");

        // Element writes keep it; membership does not.
        b.iter_mut().for_each(|c| c.tapped = true);
        assert_eq!(b.dispatch_lane(), Ok(false), "an element write is not a membership change");
        b.push(CardInstance::new(CardId(98), crate::catalog::grave_titan(), 0));
        assert_eq!(b.dispatch_lane(), Ok(false), "a push answers off the new card: a printed trigger is no board-scan bit");
        b.push(CardInstance::new(CardId(99), crate::catalog::blood_moon(), 0));
        assert_eq!(b.dispatch_lane(), Ok(true), "Blood Moon strips (CR 305.7): PRESENT off the new card");
        b.remove(1);
        assert_eq!(b.dispatch_lane(), Ok(true), "the Titan was not the witness");
        b.retain(|c| c.id != CardId(99));
        assert!(b.dispatch_lane().is_err(), "retain drops a PRESENT lane");
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
        b.store_dispatch_members(epoch, dispatch_bits(&b));
        assert_eq!(b.dispatch_lane(), Ok(true));
        assert_ne!(b.dispatch_members(), Ok(0), "and the list names the contributor");
        assert_eq!(b.memo(LANE_LAND), Some(false), "the land lane survived the store");
    }

    /// PERF `(-212)` — a membership write changes only the lanes it can: a
    /// push can only turn a lane `PRESENT`, a removal only `ABSENT`; member
    /// lists are kept exact (`(-214)`). The lane audits recompute on every
    /// read, so a wrongly kept state fails here.
    #[test]
    fn membership_writes_demote_only_the_lanes_they_can_change() {
        let land = crate::game::card_can_change_land_types;
        let creature = crate::game::card_can_change_creature_types;
        let mut b = bf(vec![crate::catalog::grizzly_bears(), crate::catalog::blood_moon()]);
        assert!(b.has_land_type_changer(land));
        assert!(!b.has_creature_type_changer(creature));
        assert_eq!(b.memo(LANE_LAND), Some(true));
        assert_eq!(b.memo(LANE_CREATURE), Some(false));

        // A push keeps PRESENT lanes and answers ABSENT ones off the card.
        b.push(CardInstance::new(CardId(9), crate::catalog::grizzly_bears(), 0));
        assert_eq!(b.memo(LANE_LAND), Some(true), "kept");
        assert_eq!(b.memo(LANE_CREATURE), Some(false), "the bear changes no creature type");

        // A removal keeps ABSENT lanes, and PRESENT ones whose witness stayed.
        assert!(b.take_by_id(CardId(9)).is_some());
        assert_eq!(b.memo(LANE_LAND), Some(true), "the bear was not the witness");
        assert_eq!(b.memo(LANE_CREATURE), Some(false), "ABSENT survives a removal");
        assert!(b.take_by_id(CardId(1)).is_some(), "the Moon leaves");
        assert_eq!(b.memo(LANE_LAND), None, "the witness left");
        assert_eq!(b.memo(LANE_CREATURE), Some(false));
        assert!(!b.has_land_type_changer(land), "refilled: no changer left");

        // `retain` cannot name what it dropped: PRESENT lanes fall, ABSENT stay.
        b.push(CardInstance::new(CardId(5), crate::catalog::blood_moon(), 0));
        assert_eq!(b.memo(LANE_LAND), Some(true));
        b.retain(|c| c.id != CardId(5));
        assert_eq!(b.memo(LANE_LAND), None, "retain drops PRESENT");
        assert_eq!(b.memo(LANE_CREATURE), Some(false), "and keeps ABSENT");
        assert!(b.pop().is_some());
        assert_eq!(b.memo(LANE_CREATURE), Some(false), "pop is a removal too");

        // The member lists are kept exact through both directions; a board
        // that grows past 64 cards loses its list.
        let mut b = bf(vec![crate::catalog::grizzly_bears(), crate::catalog::grave_titan()]);
        let Err(epoch) = b.trigger_members() else { panic!("unknown until filled") };
        b.store_trigger_members(epoch, triggerer_bits(&b));
        // The epoch is process-wide, so another test thread's definition
        // rewrite legitimately makes the memo stale (seen 2026-10-31). If the
        // epoch moved, re-stamp and re-read: this checks list exactness, not
        // thread isolation.
        let mut stamped = epoch;
        let mut members = |b: &Battlefield| match b.trigger_members() {
            Err(e) if e != stamped => {
                stamped = e;
                b.store_trigger_members(e, triggerer_bits(b));
                b.trigger_members()
            }
            r => r,
        };
        assert_eq!(members(&b), Ok(0b10));
        b.push(CardInstance::new(CardId(9), crate::catalog::grizzly_bears(), 0));
        assert_eq!(members(&b), Ok(0b10), "push kept the list");
        b.remove(0);
        assert_eq!(members(&b), Ok(0b01), "a removal shifted it");
        for i in 0..61 {
            b.push(CardInstance::new(CardId(100 + i), crate::catalog::grizzly_bears(), 0));
        }
        b.push(CardInstance::new(CardId(200), crate::catalog::grave_titan(), 0));
        assert_eq!(b.len(), 64);
        assert_eq!(members(&b), Ok(1 | (1 << 63)), "the 64th card is bit 63");
        // Removing the last index of a full list shifts nothing above it —
        // the shift-by-64 the closing grid found (PERF `(-214)`'s fix).
        assert!(b.remove(63).definition.name == "Grave Titan");
        assert_eq!(members(&b), Ok(1), "index 63 left, nothing above it to shift");
        b.push(CardInstance::new(CardId(201), crate::catalog::grave_titan(), 0));
        assert_eq!(members(&b), Ok(1 | (1 << 63)));
        b.push(CardInstance::new(CardId(202), crate::catalog::grizzly_bears(), 0));
        assert!(b.trigger_members().is_err(), "the 65th card drops the list");
    }

    /// `(-115)`'s lane is *positional* and read **instead of** the per-card
    /// gate, so a stale one silently drops a trigger. It must invalidate
    /// exactly where the presence lanes do.
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

        // Membership writes keep the list exact (PERF `(-214)`): a push
        // appends the new card's bit, a removal shifts the bits above it down.
        b.push(CardInstance::new(CardId(9), crate::catalog::grizzly_bears(), 0));
        assert_eq!(b.trigger_members(), Ok(0b0010), "a bear adds no bit");
        b.push(CardInstance::new(CardId(8), crate::catalog::grave_titan(), 0));
        assert_eq!(b.trigger_members(), Ok(0b10010), "a second Titan adds bit 4");
        b.remove(0);
        assert_eq!(b.trigger_members(), Ok(0b1001), "the bits above index 0 shifted down");
        b.remove(0);
        assert_eq!(b.trigger_members(), Ok(0b100), "the first Titan left: its bit went with it");
        assert_eq!(b.trigger_members(), Ok(triggerer_bits(&b)), "and the audit agrees");
        b.pop();
        assert_eq!(b.trigger_members(), Ok(0b000), "pop dropped the last Titan's bit");
        b.retain(|c| c.id != CardId(9));
        assert!(b.trigger_members().is_err(), "retain cannot name what it dropped");
    }

    /// The grant member list (PERF `(-189)`) is read *instead of* the
    /// per-card bit test, so a stale one silently misses a lord; it has to
    /// fall over exactly where the trigger list does.
    #[test]
    fn grant_member_list_is_filled_and_invalidated() {
        let mut b = bf(vec![
            crate::catalog::grizzly_bears(),
            crate::catalog::goblin_king(),
            crate::catalog::grizzly_bears(),
        ]);
        assert_eq!(grant_bits(&b), 0b010, "only the lord can grant a keyword");
        let Err(epoch) = b.grant_members() else { panic!("unknown until filled") };
        b.store_grant_members(epoch, grant_bits(&b));
        assert_eq!(b.grant_members(), Ok(0b010));

        b.iter_mut().for_each(|c| c.tapped = true);
        assert_eq!(b.grant_members(), Ok(0b010), "an element write is not a membership change");

        b.push(CardInstance::new(CardId(9), crate::catalog::goblin_king(), 0));
        assert_eq!(b.grant_members(), Ok(0b1010), "a second lord adds bit 3");
        b.remove(1);
        assert_eq!(b.grant_members(), Ok(0b100), "the first lord left, the second shifted down");
        assert_eq!(b.grant_members(), Ok(grant_bits(&b)), "and the audit agrees");
        b.pop();
        assert_eq!(b.grant_members(), Ok(0b000));
        b.retain(|c| c.id != CardId(0));
        assert!(b.grant_members().is_err(), "retain drops the list");

        let wide = bf((0..65).map(|_| crate::catalog::grizzly_bears()).collect());
        let Err(epoch) = wide.grant_members() else { panic!("unknown until filled") };
        wide.store_grant_members(epoch, 0);
        assert!(wide.grant_members().is_err(), "a 65-card board has no member list");
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

