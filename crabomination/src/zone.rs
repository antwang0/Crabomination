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
const GY_LANE_TOKEN: u32 = 4;
/// A card here carries a trigger that can fire from the graveyard — the
/// question every combat-damage dispatch asks of the dealer's controller's
/// graveyard (PERF `(-210)`), every step's `fire_step_triggers` asks of the
/// active player's (`(-218)`), and every event dispatch asks of both
/// (`(-230)`), each with a definition deref per card without it.
const GY_LANE_TRIGGER: u32 = 6;

/// The [`GY_LANE_TRIGGER`] predicate: definition-only, as every lane
/// predicate must be. Both graveyard-firing families — the
/// `FromYourGraveyard` scope and the `SelfSource` kinds the dispatcher fires
/// from here (cycling, milling, discard, "put into a graveyard from
/// anywhere"; `is_graveyard_self_source_kind` is the one list). Wider than
/// the two combat/step walkers need, which is the sound direction for them.
fn card_has_graveyard_trigger(c: &CardInstance) -> bool {
    use crate::effect::EventScope;
    c.definition.triggered_abilities.iter().any(|t| match t.event.scope {
        EventScope::FromYourGraveyard => true,
        EventScope::SelfSource => crate::game::effects::is_graveyard_self_source_kind(&t.event.kind),
        _ => false,
    })
}

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
/// CR 704.5d's per-card question: is this a token sitting outside the
/// battlefield? The per-card half of [`Graveyard::has_token`] and
/// [`CardPile::has_token`].
fn card_is_token(c: &CardInstance) -> bool {
    c.is_token
}

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
    #[inline]
    pub fn has_anthem(&self) -> bool {
        self.lane(GY_LANE_ANTHEM, card_has_anthem, "anthem")
    }

    /// CR 611 — true when some card here grants an activated ability from the
    /// graveyard (Riftstone Portal), the leg `grant_scan` walks both
    /// graveyards for on every call. Memoized on the same word.
    pub fn has_activated_grant(&self) -> bool {
        self.lane(GY_LANE_ACT_GRANT, card_has_gy_activated_grant, "activated-grant")
    }

    /// CR 704.5d — does this graveyard hold a token that has to cease to
    /// exist? Asked of every graveyard on every state-based sweep; the answer
    /// is `false` on nearly all of them. Instance state, not definition
    /// state — sound for the same reason the other two lanes are, since a
    /// write is the only thing that can change it and every write clears the
    /// word.
    pub fn has_token(&self) -> bool {
        self.lane(GY_LANE_TOKEN, card_is_token, "token")
    }

    /// One lane's answer: a word load and two mask tests on a hit, the zone
    /// walk plus one store on a miss. `what` names the lane in the audit.
    #[inline]
    /// Does any card here carry a trigger that can fire from the graveyard
    /// (`card_has_graveyard_trigger`)? Memoized; a miss walks the zone once.
    /// Read by `fire_combat_damage_triggers`, `fire_step_triggers` and
    /// `dispatch_triggers_for_events`.
    pub fn has_graveyard_trigger(&self) -> bool {
        self.lane(GY_LANE_TRIGGER, card_has_graveyard_trigger, "graveyard-trigger")
    }

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

/// A plain card pile — library, hand, exile — with the one whole-zone
/// question the state-based sweep asks of it: **does it hold a token?**
///
/// CR 704.5d makes a token outside the battlefield cease to exist, and the
/// sweep that enforces it walked every library, hand and exile on every
/// sweep for a case that fires a handful of times a game. The library is the
/// expensive one: a whole deck per seat per sweep.
///
/// Same shape and the same soundness argument as [`Graveyard`]: [`DerefMut`],
/// the `&mut` [`IntoIterator`] and [`push`](Self::push) are the only routes to
/// the cards, and each clears the memo — so the enumeration of write sites
/// never has to be right. [`Deref`] leaves every read site untouched, which is
/// why swapping the field type touches no caller.
#[derive(Debug, Default)]
pub struct CardPile {
    cards: CowBox<Vec<CardInstance>>,
    /// One two-bit lane, packed like the other zones' so a single store on
    /// the write path clears it.
    lanes: AtomicU8,
}

/// CR 704.5d — a token is here.
const PILE_LANE_TOKEN: u32 = 0;
/// CR 702.49 — a card here is encoded on a creature (Cipher). An *instance*
/// predicate, which a pile lane may hold: every `&mut` route into the cards
/// (`push`, `DerefMut`) clears the whole word, unlike the battlefield's.
const PILE_LANE_ENCODED: u32 = 2;

impl CardPile {
    /// One lane's answer: a word load and two mask tests on a hit, the pile
    /// walk plus one store on a miss. Lanes clear together, so a miss stores
    /// only its own lane and the others go back to unknown.
    #[inline]
    fn lane(&self, shift: u32, walk: fn(&CardInstance) -> bool, what: &str) -> bool {
        let cur = (self.lanes.load(Ordering::Relaxed) >> shift) & 0b11;
        debug_assert!(
            cur == UNKNOWN || (cur == PRESENT) == self.cards.iter().any(walk),
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

    /// Append through [`CowBox::push`] so the unshare materializes with room
    /// for the card. Inherent, so it shadows the `Deref`'d `Vec::push` at
    /// every existing call site.
    pub fn push(&mut self, card: CardInstance) {
        self.lanes.store(0, Ordering::Relaxed);
        self.cards.push(card);
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
    /// The cards clone as a `CowBox` (a refcount bump); the memo describes
    /// those same cards, so it comes along.
    fn clone(&self) -> Self {
        Self {
            cards: self.cards.clone(),
            lanes: AtomicU8::new(self.lanes.load(Ordering::Relaxed)),
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
        self.lanes.store(0, Ordering::Relaxed);
        &mut self.cards
    }
}

impl From<Vec<CardInstance>> for CardPile {
    fn from(cards: Vec<CardInstance>) -> Self {
        Self { cards: cards.into(), lanes: AtomicU8::new(0) }
    }
}

impl From<CowBox<Vec<CardInstance>>> for CardPile {
    fn from(cards: CowBox<Vec<CardInstance>>) -> Self {
        Self { cards, lanes: AtomicU8::new(0) }
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

/// Packed two-bit memo lanes for [`Battlefield`], so one store on the write
/// path clears every lane. Each lane holds the same `UNKNOWN` / `ABSENT` /
/// `PRESENT` triple the graveyard's memo uses, and a `u64` word takes
/// thirty-two of them (a `u32` until PERF `(-207)` filled its sixteen).
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
/// PERF `(-128)`: how many times anything took `&mut` at the battlefield —
/// the three chokepoints `Battlefield`'s own doc names as where every tap,
/// damage, counter and untap write goes. Read by `sba_census` to ask how many
/// state-based-action sweeps follow *no* mutable reach at all, which is the
/// population a dirty-bit gate could actually serve (the 18 % of sweeps whose
/// board fingerprint is unchanged is the ceiling; this is the reachable part
/// of it). Global rather than per-zone because the census runs one thread.
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
/// Not a presence lane: `PRESENT` here means "the member list beside it is
/// computed", and the list is what the caller reads. It shares the word so
/// that the one store every write path already makes clears it too.
const LANE_TRIGGERER: u32 = 20;
const LANE_PT_REDUCE: u32 = 22;
const LANE_GATE_KEYWORD: u32 = 24;
/// Any permanent's definition carries a `dispatch_bits::MANA_STATIC` static
/// — the presence question three per-activation walks on the mana path ask
/// (PERF `(-197)`).
const LANE_MANA_STATIC: u32 = 26;
/// Any permanent's definition grants activated abilities to *countered*
/// creatures (Agatha's Soul Cauldron) — the one reason a permanent's counter
/// bag matters to the grants-nothing gate (PERF `(-199)`).
const LANE_COUNTER_GRANT: u32 = 28;
/// Any permanent's definition carries a static that redirects a card bound
/// for a graveyard — the question four per-death walks ask (PERF `(-203)`).
const LANE_DEATH_REDIRECT: u32 = 30;
/// Any permanent's *definition* can change a card type — `type_bits::ALL`,
/// the attachment-gated half over-approximated (a lane predicate reads no
/// instance field). `ABSENT` settles `card_type_change_unscoped`'s
/// battlefield walk, which every land tap and every SBA death sweep asks
/// (PERF `(-207)`).
const LANE_CARD_TYPE: u32 = 32;
/// Any permanent's definition carries a draw-replacement static — the twelve
/// `StaticEffect`s `draw_one` walks the board for, up to eleven walks a draw
/// (PERF `(-233)`). See [`card_has_draw_static`].
const LANE_DRAW_STATIC: u32 = 34;
/// Any permanent's definition carries a static that changes how *another*
/// permanent enters — an ETB-trigger suppressor / doubler, an enters-tapped
/// or lands-enter-untapped static — the walks every entering permanent
/// makes (PERF `(-234)`). See [`card_has_etb_static`].
const LANE_ETB_STATIC: u32 = 36;
/// Any permanent's definition carries a static that replaces or redirects
/// damage to a player — the six `StaticEffect`s `deal_damage_to_from` and
/// `damage_redirect_target` walk the board for on every damage event (PERF
/// `(-235)`). See [`card_has_damage_replacement_static`].
const LANE_DAMAGE_STATIC: u32 = 38;
/// Any permanent's definition carries a static that locks or extends land
/// plays — the four `StaticEffect`s `can_player_play_land` and its helpers
/// walk the board for per land-play question (PERF `(-236)`). See
/// [`card_has_land_play_static`].
const LANE_LAND_PLAY_STATIC: u32 = 40;
/// Any permanent's definition lets mana be spent as though it were any
/// colour — the three `StaticEffect`s the payment relaxation walks the board
/// for on every payment (PERF `(-237)`). See [`card_has_any_color_static`].
const LANE_ANY_COLOR_STATIC: u32 = 42;
/// Any permanent's definition changes a maximum hand size — the six
/// `StaticEffect`s `effective_max_hand_size` walks the board for, four walks
/// a call (PERF `(-238)`). See [`card_has_hand_size_static`].
const LANE_HAND_SIZE_STATIC: u32 = 44;
/// Any permanent's definition puts counters on *another* permanent as it
/// enters — the eight `StaticEffect`s the two enters-with-counters walkers
/// match, per resolving permanent spell and per entering creature (PERF
/// `(-239)`). See [`card_has_etb_counter_static`].
const LANE_ETB_COUNTER_STATIC: u32 = 46;
/// Any permanent's definition carries a damage-prevention static —
/// `prevent_static_scan`'s twelve, folded into a mask by a board walk on every
/// damage event (PERF `(-240)`). See [`card_has_prevent_static`].
const LANE_PREVENT_STATIC: u32 = 48;
/// Any permanent's definition charges a block tax — `BlockTaxToController`,
/// the static `block_tax_for` walks the board for per declared blocker (PERF
/// `(-241)`). See [`card_has_block_tax_static`].
const LANE_BLOCK_TAX_STATIC: u32 = 50;
const LANE_MASK: u64 = 0b11;
/// Bit 0 of every lane field — set exactly on the `ABSENT` lanes.
const LANE_ABSENT_BITS: u64 = 0x5555_5555_5555_5555;
/// The lane count the predicate table below covers (shift 0 ..= 32).
const LANE_COUNT: usize = 26;

/// Every presence lane's predicate, indexed by lane shift / 2, so a
/// membership write can answer a lane off the **one card it moved**
/// (PERF `(-213)`): a push leaves an `ABSENT` lane `ABSENT` when the new
/// card fails its predicate and makes it `PRESENT` when it passes; a
/// removal leaves a `PRESENT` lane `PRESENT` when the removed card fails
/// it (the witness is elsewhere) and drops it to `UNKNOWN` when it passes.
/// `None` on the two member-list lanes. **Each entry must be the very
/// predicate that lane's callers hand to [`Battlefield::lane`]** — the
/// lane audits recompute against the handed one on every read, so an
/// entry that drifts fails the suite.
/// A lane's definition-only predicate.
type LanePredicate = fn(&CardInstance) -> bool;

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
];

/// Does this permanent contribute anything to
/// [`GameState::dispatch_board_scan`](crate::game::GameState::dispatch_board_scan)?
/// The [`LANE_DISPATCH`] predicate, and the one the lane's `debug_assert!`
/// audits against — so the memo and the walk it gates cannot drift.
fn card_has_dispatch_bits(c: &CardInstance) -> bool {
    c.dispatch_scan_bits() & crate::card::dispatch_bits::BOARD_SCAN != 0
}

/// Does this permanent's definition carry a static the mana-ability path
/// reads per activation? The [`LANE_MANA_STATIC`] predicate — the engine's
/// one definition of it, shared with the lane's fill.
fn card_has_mana_static(c: &CardInstance) -> bool {
    crate::game::actions::card_has_mana_static(c)
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

/// Is one of the keywords the whole-board presence gate
/// [`GameState::board_keyword_matching`](crate::game::GameState::board_keyword_matching)
/// is ever asked about *printed* on this permanent? The [`LANE_GATE_KEYWORD`]
/// predicate: the union of every caller's list, so one lane answers all of
/// them and the gate's per-permanent printed scan runs only on a board that
/// carries one. A caller asking about a keyword missing from this list gets a
/// wrong `false` on the printed leg — which is what the gate's own
/// `debug_assert!` (recomputing the board's keywords) fires on.
pub(crate) fn card_has_gate_keyword(c: &CardInstance) -> bool {
    use crate::card::Keyword::*;
    c.definition.keywords.iter().any(|k| {
        matches!(
            k,
            MustAttack
                | MustAttackOrBlock
                | MustAttackIfAnotherAttacks
                | MustBlock
                | MustBeBlocked
                | AllMustBlock
                | CantBeBlockedUnlessAllBlock
                | CantBeBlockedByMoreThanOne
                | Phasing
                | CumulativeUpkeep(_)
                | DoesntUntapWhileCounter(_)
                | DoesntUntapIfAttackedLastTurn
        )
    })
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

/// Does this permanent's definition carry a static that can replace, skip,
/// redirect or tax a draw? The [`LANE_DRAW_STATIC`] predicate — the union of
/// every `StaticEffect` `draw_one` and its helpers match on, so a clear lane
/// is authoritative for all of them. Definition-only; the per-instance
/// `active_static` gates inside those helpers only narrow it.
/// Does this permanent's definition carry a static that changes how a
/// permanent enters the battlefield — the ETB-trigger suppressors and
/// doublers `etb_trigger_multiplier` counts, the cross-permanent
/// `EntersTapped` statics and the `LandsEnterUntapped` override
/// `apply_enters_tapped_replacement` walks for? The [`LANE_ETB_STATIC`]
/// predicate; definition-only.
fn card_has_etb_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::SuppressCreatureEtbTriggers { .. }
                | S::EtbTriggerSpotlight
                | S::DoubleControllerEtbTriggers
                | S::EntersTapped { .. }
                | S::LandsEnterUntapped
        )
    })
}

/// Does this permanent's definition carry a static that replaces or
/// redirects damage dealt to a player — The Mindskinner, Crumbling Sanctuary,
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
            sa.effect,
            S::NoPlayerCanPlayLands
                | S::ControllerCantPlayLands
                | S::MostPermanentsCantPlay
                | S::ExtraLandPerTurn
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
                | S::MaySpendManaAsAnyColorForNamedSpells
                | S::MaySpendManaAsAnyColorForCreaturesWithChosenMv
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
                | S::OpponentsMaxHandSizeReduced(_)
                | S::ControllerMaxHandSizeReduced(_)
                | S::ControllerMaxHandSize(_)
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
            sa.effect,
            S::PlaneswalkersEnterWithExtraLoyalty { .. }
                | S::ChosenTypeEntersWithCounter { .. }
                | S::TypeEntersWithCounter { .. }
                | S::TypeEntersWithCountersPerControlled { .. }
                | S::TypedCreaturesEnterWithExtraCounter { .. }
                | S::MatchingEntersWithExtraCounters { .. }
                | S::OtherCreaturesEnterWithCountersEqualToSourcePower { .. }
                | S::ExtraEtbCountersForCreatureCasts { .. }
        )
    })
}

/// Does this permanent's definition carry one of the twelve damage-
/// prevention statics `prevent_static_scan` folds? The
/// [`LANE_PREVENT_STATIC`] predicate — the same arm list, so a clear lane is
/// exactly a zero mask. Definition-only.
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
                | S::PreventSmallDamageToThis { .. }
                | S::PreventDamageToAttachedPerPermanent { .. }
                | S::PreventDamageByRemovingCounters { .. }
                | S::PreventDamageToThisRedirect
                | S::PreventAllDamageToControllerFromOthersSources
        )
    })
}

/// Does this permanent's definition charge a block tax (Archangel of
/// Tithes, Heat Wave)? The [`LANE_BLOCK_TAX_STATIC`] predicate;
/// definition-only.
fn card_has_block_tax_static(c: &CardInstance) -> bool {
    c.definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::BlockTaxToController { .. }))
}

fn card_has_draw_static(c: &CardInstance) -> bool {
    use crate::effect::StaticEffect as S;
    c.definition.static_abilities.iter().any(|sa| {
        matches!(
            sa.effect,
            S::PlayersSkipDraws
                | S::SharedFate
                | S::PlayersDrawExiledPlayable
                | S::ControllerMaySkipDraws
                | S::MayReplaceDrawWithTutor
                | S::MayDrawFromSourceExilePile
                | S::MayReplaceDrawWithRevealUntilKind
                | S::ReplaceDrawWithLookN { .. }
                | S::ChainsOfMephistopheles
                | S::EmptyHandDrawBonus { .. }
                | S::DrawsRevealedTaxed { .. }
                | S::OpponentExtraDrawsRedirected
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

    /// PERF `(-120)`: **which** of the four gates made a dispatch grant-live,
    /// as the sixteen buckets of the `GRANT_*` reason mask, and the visits
    /// each bucket cost. `GRANTED` counts them as one bit and so cannot say
    /// whether the grant-live walks are reachable by an event-kind filter.
    pub static REASON: [AtomicU64; 16] = [const { AtomicU64::new(0) }; 16];
    pub static REASON_VISITS: [AtomicU64; 16] = [const { AtomicU64::new(0) }; 16];
    /// The same masks recomputed with the three unfiltered gates dropped when
    /// no event in the batch could match any ability under them — i.e. what
    /// `(-120)`'s filter would leave. `[0]` is "would have become no-grant".
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
/// The candidate there is to evaluate each `Selector::EachPermanent` grant's
/// requirement once over the board into an index mask at `grant_scan` time,
/// so `granted_abilities_of_inner`'s per-permanent ask becomes a bit test.
/// Whether that wins is one comparison and no callgrind row carries either
/// side: the mask would cost `grants x board` evaluations per scan, and the
/// walk costs one per (permanent that reaches `_inner`) x grant. Gated on the
/// same variable as the trigger census above.
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
/// 65 % of `evaluate_requirement_static_hinted`'s calls are its own recursion
/// through `And` / `Or` / `Not`, and the only devices that could remove them
/// need to know *what shape* the children are: flattening the combinators into
/// a slice variant only helps chains of three or more, and evaluating a leaf
/// child without a call only helps children that are leaves. Neither number is
/// on any callgrind row — a recursive edge is one row whatever the callee's
/// discriminant. Gated on the same variable as the censuses above.
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

/// PERF `(-129)`: what the trigger dispatcher's innermost loop actually
/// multiplies. `event_matches_spec` is 1,028,014 calls on `cube` against
/// 103,082 on `fixed` for roughly twice the board, so the question is which
/// factor of (pairs x batch) carries the ratio — and how many of those calls
/// are made by a (permanent, trigger) pair no event in the batch could ever
/// match, which is the population a per-pair kind gate would remove.
///
/// On `trig-census` beside the other three; the counters live outside the
/// `for ev in events` loop so the census itself cannot be what is measured.
#[cfg(feature = "trig-census")]
pub mod ems_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// Dispatches that reached the battlefield walk, and the events they
    /// carried — the mean batch size is the ratio.
    pub static DISPATCHES: AtomicU64 = AtomicU64::new(0);
    pub static BATCH: AtomicU64 = AtomicU64::new(0);
    /// Distinct `EventKind` discriminants a dispatch's triggers presented,
    /// summed — what a per-dispatch memo on the kind would have to evaluate
    /// against the batch, against `PAIRS` which is what it would serve.
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
/// Slots in [`Battlefield`]'s find memo. **One slot thrashes**: the callers
/// the memo exists for ask about two or three permanents in turn (the combat
/// resolver's attacker and blocker, the block picker's pair), so a single
/// entry is overwritten by the very next ask and hits only on an exact
/// repeat. Direct-mapping by `CardId` gives each of them its own slot; 16 is
/// 60 bytes more on the zone against a board of ~15-25 permanents.
const FIND_HINTS: usize = 16;

/// The memo slot a `CardId` maps to. Ids come off a monotonic counter, so the
/// low bits are the spread-out end of them.
#[inline]
const fn hint_slot(id: crate::card::CardId) -> usize {
    (id.0 as usize) & (FIND_HINTS - 1)
}

#[derive(Debug, Default)]
pub struct Battlefield {
    cards: CowBox<Vec<CardInstance>>,
    /// Two-bit lanes; see the `LANE_*` constants.
    type_gates: std::sync::atomic::AtomicU64,
    /// The `definition_epoch` the lanes were computed at. A lane is valid only
    /// while this still matches.
    def_epoch: std::sync::atomic::AtomicU64,
    /// The indices [`find_by_id`](Self::find_by_id) last returned, one slot
    /// per `CardId` low-bit class. Needs no invalidation at all — see that
    /// function.
    find_hints: [std::sync::atomic::AtomicU32; FIND_HINTS],
    /// Which battlefield *indices* satisfy [`card_is_triggerer`], one bit
    /// each. Read only while [`LANE_TRIGGERER`] says the list is computed, so
    /// it inherits the lanes' invalidation whole and needs none of its own.
    trig_members: std::sync::atomic::AtomicU64,
    /// The dispatch scan's member list (PERF `(-215)`): the indices whose
    /// definition carries a `BOARD_SCAN` bit, valid while [`LANE_DISPATCH`]
    /// reads `PRESENT`. Same contract as `trig_members`.
    dispatch_members: std::sync::atomic::AtomicU64,
    /// Which indices satisfy [`card_has_any_grant_bits`], on the same
    /// contract under [`LANE_GRANT`].
    grant_members: std::sync::atomic::AtomicU64,
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
    ///
    /// **It is [`FIND_HINTS`] entries, direct-mapped by id, and that is what
    /// makes the repeat rate real.** One entry hits only on an *exact*
    /// repeat, and the callers this memo exists for alternate — attacker,
    /// blocker, attacker — so each ask evicted the answer the next one
    /// wanted.
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
            UNKNOWN as u64
        } else {
            (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK
        };
        debug_assert!(
            cur == UNKNOWN as u64 || (cur == PRESENT as u64) == self.cards.iter().any(walk),
            "battlefield type-gate memo is stale: a write reached the cards without clearing it",
        );
        match cur {
            c if c == ABSENT as u64 => false,
            c if c == PRESENT as u64 => true,
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
        note_battlefield_reach();
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
            UNKNOWN as u64
        } else {
            (self.type_gates.load(Ordering::Relaxed) >> shift) & LANE_MASK
        };
        debug_assert!(
            cur == UNKNOWN as u64 || (cur == PRESENT as u64) == self.cards.iter().any(audit),
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
    /// The two type gates ask through the closure form above because their
    /// walk *is* a predicate. The dispatcher's is not: the same pass also
    /// builds two grant lists, so a closure form would walk the board twice on
    /// every miss — and the `DerefMut` invalidation serves only 48.1 / 38.3 /
    /// 32.2 % of the type gates' asks (PERF `(-87)`), which is about where a
    /// doubled miss breaks even. Splitting the read from the store makes the
    /// miss cost the walk the caller was making anyway and nothing else.
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
            bits == dispatch_bits(&self.cards),
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
    /// A presence bit was what this lane held until PERF `(-189)`, and it
    /// answered the wrong question: `keyword_grant_in_scope` asks about *one*
    /// keyword, and a board with any lord, aura or equipment on it reads
    /// `PRESENT` — 47 % of the asks on `fixed` — so the caller walked all ~23
    /// permanents to find the one or two granters and test them. The list
    /// names them, so a hit visits only those. Same shape as
    /// [`trigger_members`](Self::trigger_members), same invalidation, and a
    /// board wider than 64 has no list and keeps walking.
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
            bits == grant_bits(&self.cards),
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
    /// creatures (Agatha's Soul Cauldron)? `walk` is the engine's memo-word
    /// read (`actions::card_grants_to_countered`), asked lazily by the
    /// grants-nothing gate for a permanent that carries counters, so a board
    /// without one never reads its counter bags (PERF `(-199)`).
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
                == PRESENT as u64;
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

    /// Visit every permanent that can carry a printed trigger or a Station
    /// band: the [`trigger_members`](Self::trigger_members) list on a hit,
    /// the whole board on a miss (the dispatcher fills the list; this never
    /// does). For a walker whose per-card body reads only
    /// `definition.triggered_abilities` / `station` — the combat
    /// declaration's two whole-board trigger walks (PERF `(-222)`) and the
    /// combat-damage-to-player listener walk (`(-224)`) and the step-trigger
    /// walk when no static grant is live (`(-228)`), and the cast-trigger
    /// walk on the same condition (`(-231)`).
    #[inline]
    pub fn for_each_triggerer(&self, mut f: impl FnMut(&CardInstance)) {
        match self.trigger_members() {
            Ok(mut bits) => {
                while bits != 0 {
                    let i = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    if let Some(c) = self.cards.get(i) {
                        f(c);
                    }
                }
            }
            Err(_) => self.cards.iter().for_each(f),
        }
    }

    /// Append, through [`CowBox`]'s own `push` so an unshare materializes with
    /// room for the card (PERF `(-76)`). Inherent, so it shadows the `Deref`'d
    /// `Vec::push`.
    ///
    /// **An addition can only turn a lane `PRESENT`**, and whether it does
    /// is a question about the new card alone: every `PRESENT` lane stays,
    /// every `ABSENT` lane is re-answered off the card by its registered
    /// predicate ([`LANE_PREDICATES`]), and the two member-list lanes clear,
    /// their indices being what they hold (PERF `(-212)`, `(-213)`).
    pub fn push(&mut self, card: CardInstance) {
        note_battlefield_reach();
        self.lanes_after_push(&card);
        self.cards.push(card);
    }

    /// `Vec::remove`, shadowed: **a removal can only turn a lane `ABSENT`**,
    /// and only if the removed card was a witness — every `ABSENT` lane
    /// stays, a `PRESENT` lane stays when the card fails its predicate and
    /// drops to `UNKNOWN` when it passes (PERF `(-212)`, `(-213)`). Every
    /// removal route on the death path goes through here or
    /// [`take_by_id`](Self::take_by_id).
    pub fn remove(&mut self, index: usize) -> CardInstance {
        note_battlefield_reach();
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

    /// `Vec::retain`, shadowed: removal-only, but of cards the zone does not
    /// see go, so every `PRESENT` lane and member list drops and every
    /// `ABSENT` lane stays.
    pub fn retain(&mut self, f: impl FnMut(&CardInstance) -> bool) {
        note_battlefield_reach();
        self.lanes_after_removal(None);
        self.cards.retain(f);
    }

    /// `Vec::pop`, shadowed, on the removal contract.
    pub fn pop(&mut self) -> Option<CardInstance> {
        note_battlefield_reach();
        let card = self.cards.pop();
        self.lanes_after_removal(card.as_ref().map(|c| (self.cards.len(), c)));
        card
    }

    /// The member-list lanes as `(lane, predicate, list)`, for the membership
    /// writes to keep each list exact (PERF `(-214)`): a push appends bit
    /// `len` when the new card qualifies, a removal at `i` shifts the bits
    /// above `i` down one. The predicate is the one the list's audit
    /// recomputes with.
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
    /// kept when `card` (the one that left) fails their predicate and dropped
    /// otherwise, member lists shifted down past `index` — all three dropped
    /// whole when the leaver is unknown (`retain`).
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
                    // `index` is at most 63 (a list exists only up to 64
                    // cards); the bits above index 63 are none, and a shift
                    // by 64 is the overflow the closing grid found.
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
        note_battlefield_reach();
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
            type_gates: std::sync::atomic::AtomicU64::new(0),
            def_epoch: std::sync::atomic::AtomicU64::new(0),
            find_hints: std::array::from_fn(|_| std::sync::atomic::AtomicU32::new(0)),
            trig_members: std::sync::atomic::AtomicU64::new(0),
            dispatch_members: std::sync::atomic::AtomicU64::new(0),
            grant_members: std::sync::atomic::AtomicU64::new(0),
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

    /// CR 704.5d's lane on the graveyard. Instance state, not definition
    /// state — the only lane of the three that is, and it is sound for the
    /// same reason: a write is the only thing that can change the answer.
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

    /// PERF `(-212)` — a membership write demotes only the lanes it can
    /// change: a push can only turn a lane `PRESENT` (so `ABSENT` drops and
    /// `PRESENT` stays), a removal only `ABSENT` (the reverse); the
    /// member-list lanes clear either way. The lane audits recompute on
    /// every read, so a kept state that was wrong would fail here.
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
        assert_eq!(b.trigger_members(), Ok(0b10));
        b.push(CardInstance::new(CardId(9), crate::catalog::grizzly_bears(), 0));
        assert_eq!(b.trigger_members(), Ok(0b10), "push kept the list");
        b.remove(0);
        assert_eq!(b.trigger_members(), Ok(0b01), "a removal shifted it");
        for i in 0..61 {
            b.push(CardInstance::new(CardId(100 + i), crate::catalog::grizzly_bears(), 0));
        }
        b.push(CardInstance::new(CardId(200), crate::catalog::grave_titan(), 0));
        assert_eq!(b.len(), 64);
        assert_eq!(b.trigger_members(), Ok(1 | (1 << 63)), "the 64th card is bit 63");
        // Removing the last index of a full list shifts nothing above it —
        // the shift-by-64 the closing grid found (PERF `(-214)`'s fix).
        assert!(b.remove(63).definition.name == "Grave Titan");
        assert_eq!(b.trigger_members(), Ok(1), "index 63 left, nothing above it to shift");
        b.push(CardInstance::new(CardId(201), crate::catalog::grave_titan(), 0));
        assert_eq!(b.trigger_members(), Ok(1 | (1 << 63)));
        b.push(CardInstance::new(CardId(202), crate::catalog::grizzly_bears(), 0));
        assert!(b.trigger_members().is_err(), "the 65th card drops the list");
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

