//! MTG continuous-effect layer system (CR 613).
//!
//! Continuous effects from static abilities and resolved spells/abilities are
//! collected here and applied in layer order to produce a `ComputedPermanent`
//! — the "true" state of each permanent after all effects are considered.
//!
//! Layer order:
//!   1. Copy effects
//!   2. Control-changing effects
//!   3. Text-changing effects
//!   4. Type-changing effects
//!   5. Color-changing effects
//!   6. Ability-adding/removing effects
//!      7a. P/T set by characteristic-defining abilities
//!      7b. P/T set to specific value
//!      7c. P/T modifications (+N/+M)
//!      7d. P/T switching
//!      Counters (+1/+1, -1/-1) are applied after 7c per CR 613.7f.

use crate::card::{
    CardId, CardType, CounterType, CreatureType, Keyword, LandType, SelectionRequirement, Subtypes,
    Supertype,
};
use crate::mana::{Color, ColorSet};
use serde::{Deserialize, Serialize};

/// Animate Artifact's "as long as enchanted artifact isn't a creature", read
/// inside the layer pass off the printed card: the computed type is what is
/// being built (the CR 613.8 dependency on other animations is not modeled).
pub(crate) fn printed_noncreature(card: &crate::card::CardInstance) -> bool {
    !card.definition.is_creature()
}

// ── Duration ──────────────────────────────────────────────────────────────────

/// How long a continuous effect persists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectDuration {
    /// Lasts while the source permanent is on the battlefield (static ability).
    WhileSourceOnBattlefield,
    /// Expires at the next end-of-turn Cleanup step.
    UntilEndOfTurn,
    /// Lasts until the beginning of the next turn (any player's).
    UntilNextTurn,
    /// CR 611.2b — "until your next turn": lasts through the intervening
    /// turns and expires as the recorded player's next turn begins. Carries
    /// the turn it was installed on so it never expires on its own turn.
    UntilYourNextTurn { player: usize, installed_turn: u32 },
    /// CR 611.2b — "until your next upkeep": the untap step of the recorded
    /// player's next turn still sees the effect; it expires as that upkeep
    /// begins (Xenic Poltergeist).
    UntilYourNextUpkeep { player: usize, installed_turn: u32 },
    /// CR 611.2b — "until the end of your next turn": expires in the cleanup
    /// step of the recorded player's first turn after `installed_turn`.
    UntilEndOfYourNextTurn { player: usize, installed_turn: u32 },
    /// "During its controller's next combat phase" (Sizzling Soloist): expires
    /// as the first combat phase of `player`'s that starts after it began
    /// ends. `installed_turn` is one back when that combat can still come
    /// this turn.
    ThroughPlayersNextCombat { player: usize, installed_turn: u32 },
    /// Expires when the current combat phase ends (CR 511.2 — "Effects
    /// that last 'until end of combat' expire at the end of the combat
    /// phase"). Cleared as the end-of-combat step ends. If the effect
    /// is registered outside a combat phase it falls through to the
    /// next combat phase's end-of-combat step.
    UntilEndOfCombat,
    /// CR 611.2c — lasts while the source permanent stays tapped on the
    /// battlefield (Hisoka's Guard). Swept alongside the `while_source_tapped`
    /// control steals.
    WhileSourceTapped,
    /// CR 611.2c — lasts while the source stays attached to a permanent
    /// (Assimilation Aegis). Swept alongside `WhileSourceTapped`.
    WhileSourceAttached,
    /// Indefinite (e.g. counters, "for as long as" effects).
    Indefinite,
    /// CR 611.2b — "for as long as you control [the source]": ends when the
    /// source leaves the battlefield or `player` stops controlling it.
    WhileSourceControlledBy { player: usize },
    /// CR 611.2c — lasts while each affected permanent has a `kind` counter;
    /// swept beside `WhileSourceTapped`.
    WhileHasCounter(crate::card::CounterType),
    /// The keyword-counter sibling (Minas Morgul's shadow counter).
    WhileHasKeywordCounter(crate::effect::KeywordCounterKind),
    /// Lasts while each affected permanent stays tapped (Braided Net).
    WhileTapped,
}

// ── Layer classification ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum Layer {
    L1Copy        = 1,
    L2Control     = 2,
    L3Text        = 3,
    L4Type        = 4,
    L5Color       = 5,
    L6Ability     = 6,
    L7PowerTough  = 7,
}

/// Sub-layer within Layer 7 for power/toughness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum PtSublayer {
    /// 7a: characteristic-defining abilities (e.g. Tarmogoyf).
    CharDefining,
    /// 7b: set to a specific value (e.g. "becomes a 1/1").
    SetValue,
    /// 7c: +N/+M modifications (Giant Growth, Glorious Anthem).
    Modify,
    /// 7d: switch power and toughness.
    Switch,
}

// ── Modification ──────────────────────────────────────────────────────────────

/// The actual change made by a continuous effect within its layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modification {
    // ── Layer 2 ──────────────────────────────────────────────────────────────
    ChangeController(usize),

    // ── Layer 3 ──────────────────────────────────────────────────────────────
    /// CR 612 — replace all instances of one color word with another
    /// (Protection-from-color keywords). Trait Doctoring, Mind Bend.
    ReplaceColorWord(Color, Color),
    /// CR 612 / 305.7 — replace all instances of one basic land type with
    /// another (type line + landwalk; the intrinsic mana ability follows
    /// the computed land type).
    ReplaceBasicLandType(crate::card::LandType, crate::card::LandType),

    // ── Layer 4 ──────────────────────────────────────────────────────────────
    AddCardType(CardType),
    RemoveCardType(CardType),
    SetCardTypes(Vec<CardType>),
    AddCreatureType(CreatureType),
    /// CR 205.1b — "isn't a [type]" (Werewolf Pack Leader's "isn't a Human").
    /// A no-op on an object that is every creature type.
    RemoveCreatureType(CreatureType),
    SetCreatureTypes(Vec<CreatureType>),
    /// CR 205.3 — "is every creature type" (Omo). A type effect, so a layer-6
    /// strip leaves it; the pass reports it as a computed changeling.
    EveryCreatureType,
    AddLandType(LandType),
    SetLandTypes(Vec<LandType>),
    /// Vraska, Betrayal's Sting's −2 — "becomes a Treasure artifact".
    SetArtifactSubtypes(Vec<crate::card::ArtifactSubtype>),
    /// Layer 4 — add one artifact subtype without clearing the printed set
    /// (Bludgeon Brawl's granted Equipment).
    AddArtifactSubtype(crate::card::ArtifactSubtype),
    /// CR 205.4 — grant a supertype (Legendary, etc.). Used by the Ring's
    /// level-1 emblem (CR 701.54c — "your Ring-bearer is legendary").
    AddSupertype(Supertype),
    /// CR 205.4 — remove a supertype (Thermal Flux's "isn't snow").
    RemoveSupertype(Supertype),

    // ── Layer 5 ──────────────────────────────────────────────────────────────
    AddColor(Color),
    SetColors(Vec<Color>),
    LoseAllColors,

    // ── Layer 6 ──────────────────────────────────────────────────────────────
    AddKeyword(Keyword),
    RemoveKeyword(Keyword),
    /// CR 113.11 — "loses [keyword] and can't have or gain [keyword]"
    /// (the Theros Archetypes). Unlike `RemoveKeyword`, wins over *any*
    /// grant regardless of timestamp: collected during the walk and
    /// stripped after every Add has applied.
    CantHaveKeyword(Keyword),
    RemoveAllAbilities,
    /// CR 113.10b — Blood Sun's "loses all abilities except mana abilities":
    /// a strip that leaves the printed mana abilities activatable
    /// ([`ComputedPermanent::keeps_mana_abilities`]).
    RemoveAllAbilitiesButMana,
    /// "If a triggered ability of that creature triggers, that ability
    /// triggers an additional time" (Lwaxana Troi). Changes no
    /// characteristic; `linked_trigger_extra_fires` counts the entries.
    TriggersAdditionalTime,

    // ── Layer 7 ──────────────────────────────────────────────────────────────
    SetPowerToughness(i32, i32),   // 7b
    /// 7b — set base power and toughness each equal to the affected
    /// permanent's own mana value (Opalescence / Starfield of Nyx turning
    /// non-Aura enchantments into `MV/MV` creatures). Read per-permanent from
    /// its printed mana cost.
    SetPowerToughnessToManaValue,  // 7b
    /// Set base power only (layer 7b), leaving base toughness intact — "this
    /// creature's base power becomes N" (Belligerent Yearling). Ordered with
    /// `SetPowerToughness` by timestamp; later wins.
    SetPower(i32),                 // 7b
    /// Set base toughness only (layer 7b), leaving base power intact — "creatures
    /// your opponents control have base toughness N" (Maha, Its Feathers Night).
    /// Mirror of `SetPower`; later wins by timestamp; `SetPowerToughness` (both
    /// halves) overrides it when its timestamp is later.
    SetToughness(i32),             // 7b
    /// 7b — base power / toughness each a `Value`, read with the affected
    /// permanent's controller as "you" on every gather (Svogthos). The gather
    /// rewrites it to `SetPowerToughness` before any layer applies it.
    SetPowerToughnessLive(Box<(crate::effect::Value, crate::effect::Value)>),
    ModifyPower(i32),              // 7c
    ModifyToughness(i32),          // 7c
    ModifyPowerToughness(i32, i32),// 7c
    /// 7c — +per_power/+per_toughness for each of the affected creature's own
    /// creature types, capped at `max` types (Diligent Zookeeper). Reads the
    /// layer-4 computed creature types of the same card.
    ModifyPtPerOwnCreatureType(i32, i32, u32),
    /// +N/+0 for each time the permanent has attacked this turn (Moraug).
    ModifyPowerPerTimesAttacked(i32),
    /// 7c — +per_power/+per_toughness for each of the affected creature's own
    /// colors, only while it has two or more (Knight of New Alara). Reads the
    /// layer-5 computed colors of the same card.
    ModifyPtPerOwnColorIfMulticolored(i32, i32),
    SwitchPowerToughness,          // 7d
}

// ── Continuous effect ─────────────────────────────────────────────────────────

/// The [`Modification`] families the engine's presence gates ask the stored
/// effect list about, one bit each — see [`ContinuousEffects::families`].
pub mod mod_families {
    /// `AddCardType` / `RemoveCardType` / `SetCardTypes` (layer 4).
    pub const CARD_TYPE: u32 = 1 << 0;
    /// `AddLandType` / `SetLandTypes` / `ReplaceBasicLandType`.
    pub const LAND_TYPE: u32 = 1 << 1;
    /// `AddCreatureType` / `SetCreatureTypes`.
    pub const CREATURE_TYPE: u32 = 1 << 2;
    /// `AddColor` / `SetColors` / `LoseAllColors` (layer 5).
    pub const COLOR: u32 = 1 << 3;
    /// `AddKeyword` (layer 6).
    pub const KEYWORD: u32 = 1 << 4;
    /// `RemoveAllAbilities` (layer 6).
    pub const STRIP: u32 = 1 << 5;
    /// A layer-7 modification that can lower toughness
    /// (`modification_reduces_toughness`).
    pub const TOUGHNESS_REDUCE: u32 = 1 << 6;
    /// `AddKeyword` of a CR 602.5 ability lock
    /// (`card::is_ability_lock_keyword`) — the [`KEYWORD`] subset the
    /// activation gates ask about, so they read one word instead of the list.
    pub const ABILITY_LOCK: u32 = 1 << 7;
    /// A modification that edits an existing keyword rather than granting
    /// one: `RemoveKeyword` / `CantHaveKeyword` (layer 6) and the layer-3
    /// rewrites `ReplaceColorWord` (`Protection`) / `ReplaceBasicLandType`
    /// (`Landwalk`). Not in [`KEYWORD`] (that bit means "a grant") nor
    /// [`STRIP`] (that means `RemoveAllAbilities` exactly).
    ///
    /// ⚠ These four had no family until PERF `(-346)`, which silently broke a
    /// gate; [`ContinuousEffects::any_in_family`] now catches that in tests.
    pub const KEYWORD_EDIT: u32 = 1 << 8;
    /// `SetPowerToughnessLive` — the gather's rewrite pass asks for it.
    pub const LIVE_PT: u32 = 1 << 9;
    /// `TriggersAdditionalTime` — the trigger-dispatch gate reads one word.
    pub const TRIGGER_DOUBLE: u32 = 1 << 10;
    /// The fold is computed; bit 31, so a zero word is "unknown".
    pub(super) const VALID: u32 = 1 << 31;
}

/// The [`mod_families`] `m` belongs to.
///
/// ⚠ Exhaustive on purpose (it was `_ => 0` until PERF `(-346)`). A
/// family-less modification is silently invisible to every
/// [`ContinuousEffects::has_family`] gate, so the rules question gets the
/// printed answer. A new variant must be named here; family-less ones say why.
pub fn modification_families(m: &Modification) -> u32 {
    use Modification as M;
    use mod_families as F;
    let kind = match m {
        M::AddCardType(_) | M::RemoveCardType(_) | M::SetCardTypes(_) => F::CARD_TYPE,
        M::AddLandType(_) | M::SetLandTypes(_) => F::LAND_TYPE,
        // Two families: a layer-4 land-type change *and* the layer-3 rewrite
        // of a `Landwalk` payload (CR 612 / 702.15).
        M::ReplaceBasicLandType(..) => F::LAND_TYPE | F::KEYWORD_EDIT,
        M::AddCreatureType(_) | M::RemoveCreatureType(_) | M::SetCreatureTypes(_) => F::CREATURE_TYPE,
        // Read through the computed changeling, so the keyword gates see it.
        M::EveryCreatureType => F::CREATURE_TYPE | F::KEYWORD,
        M::AddColor(_) | M::SetColors(_) | M::LoseAllColors => F::COLOR,
        M::AddKeyword(k) if crate::card::is_ability_lock_keyword(k) => F::KEYWORD | F::ABILITY_LOCK,
        M::AddKeyword(_) => F::KEYWORD,
        M::RemoveKeyword(_) | M::CantHaveKeyword(_) | M::ReplaceColorWord(..) => F::KEYWORD_EDIT,
        M::RemoveAllAbilities | M::RemoveAllAbilitiesButMana => F::STRIP,
        M::SetPowerToughnessLive(_) => F::LIVE_PT,
        M::TriggersAdditionalTime => F::TRIGGER_DOUBLE,
        // Family-less because nothing gates on them: the requirement walker's
        // `has_stype` / `has_atype` read the computed view unconditionally,
        // as do controller checks. Add a family when a gate wants one (cost
        // model: PERF `(-345)`).
        M::AddSupertype(_)
        | M::RemoveSupertype(_)
        | M::SetArtifactSubtypes(_)
        | M::AddArtifactSubtype(_)
        | M::ChangeController(_) => 0,
        // Layer 7: the only gated question (toughness) is answered by the
        // shared oracle `modification_reduces_toughness` below, not a second
        // arm list.
        M::SetPowerToughness(..)
        | M::SetPowerToughnessToManaValue
        | M::SetPower(_)
        | M::SetToughness(_)
        | M::ModifyPower(_)
        | M::ModifyToughness(_)
        | M::ModifyPowerToughness(..)
        | M::ModifyPtPerOwnCreatureType(..)
        | M::ModifyPowerPerTimesAttacked(_)
        | M::ModifyPtPerOwnColorIfMulticolored(..)
        | M::SwitchPowerToughness => 0,
    };
    let pt = if crate::game::modification_reduces_toughness(m) { F::TOUGHNESS_REDUCE } else { 0 };
    kind | pt
}

/// The resolved continuous effects (`GameState::continuous_effects`) in a
/// `CowBox`, with a cached fold of the [`mod_families`] present (PERF
/// `(-208)`) so presence gates read one word instead of walking the list.
///
/// Reads go through `Deref`; every `&mut` route (`DerefMut`,
/// [`push`](Self::push)) clears the fold and the next ask recomputes it.
/// Atomic rather than `Cell` so `GameState` stays `Sync` (same contract as
/// `Battlefield`'s lanes).
#[derive(Debug, Default)]
pub struct ContinuousEffects {
    list: crate::cow::CowBox<Vec<ContinuousEffect>>,
    /// `mod_families::VALID | families`, or 0 while unknown.
    fold: std::sync::atomic::AtomicU32,
    /// Count of `&mut` reaches at the list, bumped where `fold` is cleared —
    /// the twin of [`crate::zone::Battlefield`]'s `writes` (PERF `(-306)`),
    /// part of the gather memo's key. Only a change is read.
    writes: u32,
}

impl ContinuousEffects {
    /// The families present in the list — [`modification_families`] OR'd
    /// over every entry. A word load on a hit; audited against the walk on
    /// every hit under debug assertions.
    #[inline]
    pub fn families(&self) -> u32 {
        use std::sync::atomic::Ordering::Relaxed;
        let w = self.fold.load(Relaxed);
        if w & mod_families::VALID != 0 {
            debug_assert_eq!(
                w & !mod_families::VALID,
                self.compute_families(),
                "continuous-effects fold is stale: a write reached the list without clearing it",
            );
            return w & !mod_families::VALID;
        }
        self.fill()
    }

    /// Is any entry of `family` in the list?
    #[inline]
    pub fn has_family(&self, family: u32) -> bool {
        self.families() & family != 0
    }

    /// `has_family(mask) && iter().any(pred)`, with the premise audited.
    ///
    /// ⚠ The short-circuit is only sound if every entry `pred` accepts carries
    /// a bit of `mask` in [`modification_families`]. That once shipped wrong
    /// (PERF `(-345)`: a "loses flying" effect was invisible to its gate).
    /// Under `debug-assertions` a `false` answer re-runs `pred` over the list
    /// and asserts it finds nothing, so a mis-mapped modification fails tests.
    ///
    /// ⚠ `pred` is `Copy` and passed to `any` by value, never by reference:
    /// `&F: FnMut` goes through a non-inlined `call_mut` (see
    /// `Battlefield::lane`'s doc; the first cut paid +0.16 % on all pools).
    #[inline]
    pub fn any_in_family(
        &self,
        mask: u32,
        pred: impl Fn(&ContinuousEffect) -> bool + Copy,
    ) -> bool {
        let hit = self.has_family(mask) && self.list.iter().any(pred);
        debug_assert!(
            hit || !self.list.iter().any(pred),
            "a modification this predicate accepts carries none of the mask's families — \
             modification_families and the gate's mask have drifted apart",
        );
        hit
    }

    #[inline(never)]
    fn fill(&self) -> u32 {
        let f = self.compute_families();
        self.fold.store(f | mod_families::VALID, std::sync::atomic::Ordering::Relaxed);
        f
    }

    fn compute_families(&self) -> u32 {
        self.list.iter().fold(0, |m, e| m | modification_families(&e.modification))
    }

    /// `Vec::push` through the `CowBox`'s unshare-aware push, clearing the fold.
    #[inline]
    pub fn push(&mut self, effect: ContinuousEffect) {
        self.note_write();
        self.list.push(effect);
    }

    /// One `&mut` reach at the list: clear the family fold and bump
    /// [`writes`](Self::writes). The two call sites are this `push` and
    /// `DerefMut`.
    #[inline]
    fn note_write(&mut self) {
        self.fold.store(0, std::sync::atomic::Ordering::Relaxed);
        self.writes = self.writes.wrapping_add(1);
    }

    /// The list's write counter — see [`writes`](Self::writes).
    #[inline]
    pub fn writes(&self) -> u32 {
        self.writes
    }
}

impl std::ops::Deref for ContinuousEffects {
    type Target = Vec<ContinuousEffect>;
    #[inline]
    fn deref(&self) -> &Vec<ContinuousEffect> {
        &self.list
    }
}

impl std::ops::DerefMut for ContinuousEffects {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<ContinuousEffect> {
        self.note_write();
        &mut self.list
    }
}

impl Clone for ContinuousEffects {
    fn clone(&self) -> Self {
        Self {
            list: self.list.clone(),
            fold: std::sync::atomic::AtomicU32::new(
                self.fold.load(std::sync::atomic::Ordering::Relaxed),
            ),
            writes: self.writes,
        }
    }
}

impl From<Vec<ContinuousEffect>> for ContinuousEffects {
    fn from(list: Vec<ContinuousEffect>) -> Self {
        Self { list: list.into(), fold: std::sync::atomic::AtomicU32::new(0), writes: 0 }
    }
}

impl Serialize for ContinuousEffects {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.list.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ContinuousEffects {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::cow::CowBox::<Vec<ContinuousEffect>>::deserialize(deserializer)
            .map(|list| Self { list, fold: std::sync::atomic::AtomicU32::new(0), writes: 0 })
    }
}

/// A single continuous effect active in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousEffect {
    /// Unique ID (monotonically increasing timestamp for ordering within a layer).
    pub timestamp: u64,
    /// The permanent that generated this effect (used for `WhileSourceOnBattlefield`).
    pub source: CardId,
    /// Which permanents this effect applies to.
    /// `None` means it applies to the source itself.
    pub affected: AffectedPermanents,
    pub layer: Layer,
    pub sublayer: Option<PtSublayer>,
    pub duration: EffectDuration,
    pub modification: Modification,
}

/// The ids an [`AffectedPermanents::Specific`] names.
///
/// Inline rather than a `Vec`: `CopyVec<[CardId; 4]>` is the same 24 bytes,
/// so the slot costs nothing at clone (PERF `(-152)`'s rule) and removes an
/// allocation from almost every emitter, which names one permanent.
/// `CopyVec` rather than `SmallVec` so `ContinuousEffect::clone` is a
/// `memcpy`. PERF `(-161)`.
pub type AffectedIds = crate::copyvec::CopyVec<[CardId; 4]>;

/// `AffectedIds` must not be wider than a `Vec` (the inline slot must stay
/// free to clone). 64-bit only: on wasm32 a `Vec` is 12 bytes.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(
    std::mem::size_of::<AffectedIds>() <= std::mem::size_of::<Vec<CardId>>(),
);

/// Describes which permanents a continuous effect affects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffectedPermanents {
    /// Only the source permanent itself.
    Source,
    /// All permanents matching the given predicate.
    /// `controller: Some(p)` — only that player's permanents; `None` — all.
    /// `exclude_source` — the printed "other [type]" wording (Hofri
    /// Ghostforge). `card_types` is a conjunction (`[Artifact, Creature]` =
    /// "artifact creatures"); empty = any type.
    All {
        controller: Option<usize>,
        card_types: Vec<CardType>,
        #[serde(default)]
        exclude_source: bool,
        /// Color filter, matched against cost colors (Honor of the Pure).
        /// `None` = any color.
        #[serde(default)]
        color: Option<crate::mana::Color>,
        /// Token filter: `Some(true)` tokens only (Intangible Virtue),
        /// `Some(false)` nontokens only (Always Watching), `None` either.
        #[serde(default)]
        token: Option<bool>,
        /// Optional ownership filter relative to the permanent's controller
        /// ("creatures you control but don't own" — Laughing Jasper Flint).
        /// `None` = either; `Some(false)` = borrowed permanents only.
        #[serde(default)]
        owned_by_controller: Option<bool>,
        /// "Colorless only" filter (Ruination Guide). Devoid-aware (CR 702.114).
        #[serde(default)]
        colorless: bool,
    },
    /// All permanents controlled by players outside the source's team (2HG
    /// teammates are spared). `friendly_seats` lists the team; empty falls
    /// back to the single-seat `source_controller` check (old snapshots).
    AllOpponents {
        source_controller: usize,
        card_types: Vec<CardType>,
        #[serde(default)]
        friendly_seats: Vec<usize>,
        /// Narrows the opponent scope by colour / creature type / counter, so
        /// "blue creatures your opponents control" isn't read as all of them
        /// (Heat Wave). `None` on each keeps the unnarrowed scope.
        #[serde(default)]
        color: Option<crate::mana::Color>,
        #[serde(default)]
        creature_type: Option<crate::card::CreatureType>,
        #[serde(default)]
        counter: Option<crate::card::CounterType>,
    },
    /// A specific set of permanents.
    Specific(AffectedIds),
    /// All creatures with the given creature type (lord effect).
    /// `controller: Some(p)` restricts to that player's creatures; `None` = all.
    /// `exclude_source` — printed "other [creature type]s" (Goblin King).
    AllWithCreatureType {
        controller: Option<usize>,
        creature_type: crate::card::CreatureType,
        #[serde(default)]
        exclude_source: bool,
    },
    /// All permanents with at least `at_least` counters of `counter` (SOS
    /// Emil's "creatures with +1/+1 counters have trample"). `controller:
    /// None` = all players; empty `card_types` = any type.
    AllWithCounter {
        controller: Option<usize>,
        card_types: Vec<CardType>,
        counter: crate::card::CounterType,
        at_least: u32,
    },
    /// A requirement the simpler variants can't express (disjunctions,
    /// `IsNonbasicLand`, type unions), matched card-locally via
    /// [`requirement_matches_card`] relative to `source_controller`. Only
    /// emitted when every leaf is readable from printed characteristics, so
    /// no `GameState` is needed (Thalia, Heretic Cathar).
    CardMatch {
        source_controller: usize,
        requirement: Box<SelectionRequirement>,
    },
    /// `CardMatch` also gated on computed power (Temur Ascendancy), evaluated
    /// in a second pass once gate-free power is known; acyclic because a
    /// layer-6 grant doesn't feed layer 7 (CR 613.8).
    CardMatchPowerGated {
        source_controller: usize,
        requirement: Box<SelectionRequirement>,
        power_at_least: i32,
    },
}

impl AffectedPermanents {
    /// One named permanent — the shape almost every emitter builds, and the
    /// one [`AffectedIds`]'s inline slot exists for.
    pub fn just(id: CardId) -> Self {
        Self::Specific(std::iter::once(id).collect())
    }
}

// ── Computed permanent ────────────────────────────────────────────────────────

/// Which `CardDefinition` field an [`Overlay`] projects into, as a type
/// parameter: the printed-side read is a field offset rather than a stored
/// `fn` pointer (8 bytes per overlay and an opaque indirect call).
pub trait DefProj: 'static {
    type Val: Clone + 'static;
    fn printed(def: &crate::card::CardDefinition) -> &Self::Val;
}

/// [`DefProj`] for a characteristic stored as a `Vec` and read as a slice —
/// the override side is a `Box<[T]>`, so the item type has to be nameable.
pub trait DefProjList: 'static {
    type Item: Clone + 'static;
    fn printed(def: &crate::card::CardDefinition) -> &Vec<Self::Item>;
}

macro_rules! def_proj {
    ($name:ident, $val:ty, $field:ident) => {
        pub struct $name;
        impl DefProj for $name {
            type Val = $val;
            #[inline]
            fn printed(def: &crate::card::CardDefinition) -> &$val {
                &def.$field
            }
        }
    };
}

def_proj!(CardTypesOf, Vec<CardType>, card_types);
def_proj!(SupertypesOf, Vec<Supertype>, supertypes);
def_proj!(SubtypesOf, Subtypes, subtypes);

pub struct KeywordsOf;
impl DefProjList for KeywordsOf {
    type Item = Keyword;
    #[inline]
    fn printed(def: &crate::card::CardDefinition) -> &Vec<Keyword> {
        &def.keywords
    }
}

/// A layer-computed characteristic that is usually exactly the printed one:
/// the layer pass's working form, borrowing the definition.
///
/// The printed value is cloned only when a layer writes through `DerefMut`
/// (the pass used to clone every collection it never modified). `Deref` /
/// `DerefMut` to `T` keep the pass body reading like the plain type; the
/// override is boxed so the struct stays 24 bytes (`Subtypes` is seven
/// `Vec`s).
///
/// Not the stored form because of the borrow: [`into_overlay`](Self::into_overlay)
/// drops it so [`ComputedPermanent`] holds one definition handle, not four
/// (PERF `(-70)`).
pub(crate) struct Printed<'a, P: DefProj> {
    src: &'a crate::card::CardDefinition,
    over: Option<Box<P::Val>>,
}

impl<'a, P: DefProj> Printed<'a, P> {
    #[inline]
    pub(crate) fn new(src: &'a crate::card::CardDefinition) -> Self {
        Self { src, over: None }
    }

    /// Hand any override to the stored form, dropping the borrow. The
    /// definition comes back from [`ComputedPermanent::def`].
    #[inline]
    pub(crate) fn into_overlay(self) -> Overlay<P> {
        Overlay { over: self.over }
    }
}

/// [`Printed`] without the definition handle: the stored form. Reads take
/// the definition from [`ComputedPermanent`]'s single `Arc<CardDefinition>`.
pub struct Overlay<P: DefProj> {
    over: Option<Box<P::Val>>,
}

impl<P: DefProj> Overlay<P> {
    /// The override once a layer wrote one, else the printed value.
    #[inline]
    fn get<'a>(&'a self, def: &'a crate::card::CardDefinition) -> &'a P::Val {
        match &self.over {
            Some(v) => v,
            None => P::printed(def),
        }
    }
}

impl<P: DefProj> Clone for Overlay<P> {
    fn clone(&self) -> Self {
        Self { over: self.over.clone() }
    }
}

/// [`Overlay`] for a list — see [`PrintedList`] for why the override is a
/// `Box<[T]>`.
pub struct OverlayList<P: DefProjList> {
    over: Option<Box<[P::Item]>>,
}

impl<P: DefProjList> OverlayList<P> {
    #[inline]
    fn get<'a>(&'a self, def: &'a crate::card::CardDefinition) -> &'a [P::Item] {
        match &self.over {
            Some(v) => v,
            None => P::printed(def),
        }
    }
}

impl<P: DefProjList> Clone for OverlayList<P> {
    fn clone(&self) -> Self {
        Self { over: self.over.clone() }
    }
}

/// [`Printed`] for a list, with the override stored as a `Box<[T]>`.
///
/// One allocation instead of `Box<Vec<T>>`'s two; the list is materialized
/// and appended once, so `Vec` headroom is never used. The fat pointer's +8
/// bytes on `ComputedPermanent` was measured cheaper than the allocation
/// (eighty-fourth pass). Unboxing the override (`Option<T>`, +208 bytes) lost
/// 1.755 % and is in TODO's "do not rebuild".
///
/// `Deref<Target = [T]>`: every reader uses slice API; the writers are the
/// inherent methods below.
pub(crate) struct PrintedList<'a, P: DefProjList> {
    src: &'a crate::card::CardDefinition,
    over: Option<Box<[P::Item]>>,
}

impl<'a, P: DefProjList> PrintedList<'a, P> {
    #[inline]
    pub(crate) fn new(src: &'a crate::card::CardDefinition) -> Self {
        Self { src, over: None }
    }

    /// [`Printed::into_overlay`] for a list.
    #[inline]
    pub(crate) fn into_overlay(self) -> OverlayList<P> {
        OverlayList { over: self.over }
    }

    /// The printed list, or the override once one exists.
    #[inline]
    fn current(&self) -> &[P::Item] {
        match &self.over {
            Some(v) => v,
            None => P::printed(self.src),
        }
    }

    /// Append, materializing with room for the element in one allocation.
    pub(crate) fn push(&mut self, value: P::Item) {
        let mut v: Vec<P::Item> = Vec::with_capacity(self.current().len() + 1);
        v.extend_from_slice(self.current());
        v.push(value);
        self.over = Some(v.into_boxed_slice());
    }

    /// `Vec::retain`, materializing the survivors. One allocation whether or
    /// not an override already exists — the printed side is far the common
    /// one, and every caller here is behind a presence gate.
    pub(crate) fn retain(&mut self, mut f: impl FnMut(&P::Item) -> bool) {
        let kept: Vec<P::Item> = self.current().iter().filter(|x| f(x)).cloned().collect();
        self.over = Some(kept.into_boxed_slice());
    }

    /// Empty the list (CR 113.10b's "loses all abilities"). No allocation —
    /// an empty `Box<[T]>` is a dangling pointer.
    pub(crate) fn clear(&mut self) {
        self.over = Some(Vec::new().into_boxed_slice());
    }

    /// Mutable access to the elements, materializing first. Cannot change the
    /// length — the two callers rewrite keywords in place (Trait Doctoring's
    /// protection-colour swap, the landwalk retype).
    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, P::Item> {
        if self.over.is_none() {
            self.over = Some(self.current().to_vec().into_boxed_slice());
        }
        self.over.as_mut().unwrap().iter_mut()
    }
}

impl<P: DefProjList> std::ops::Deref for PrintedList<'_, P> {
    type Target = [P::Item];
    #[inline]
    fn deref(&self) -> &[P::Item] {
        self.current()
    }
}

/// Appending, the one write that bypasses `DerefMut`: `Vec::clone` returns
/// `capacity == len`, so clone-then-push always reallocated (eighty-first
/// tip). Materializing with room for the element saves that allocation.
/// Inherent, so it shadows the `Deref`'d `Vec::push` at existing call sites.
impl<T: Clone + 'static, P: DefProj<Val = Vec<T>>> Printed<'_, P> {
    #[inline]
    pub(crate) fn push(&mut self, value: T) {
        match &mut self.over {
            Some(v) => v.push(value),
            None => {
                let printed = P::printed(self.src);
                let mut v = Vec::with_capacity(printed.len() + 1);
                v.extend_from_slice(printed);
                v.push(value);
                self.over = Some(Box::new(v));
            }
        }
    }
}

impl<P: DefProj> std::ops::Deref for Printed<'_, P> {
    type Target = P::Val;
    #[inline]
    fn deref(&self) -> &P::Val {
        match &self.over {
            Some(v) => v,
            None => P::printed(self.src),
        }
    }
}

/// The clone point: the first layer write materializes the printed value.
impl<P: DefProj> std::ops::DerefMut for Printed<'_, P> {
    #[inline]
    fn deref_mut(&mut self) -> &mut P::Val {
        let src = self.src;
        self.over.get_or_insert_with(|| Box::new(P::printed(src).clone()))
    }
}

/// The fully resolved state of a permanent after all layers are applied.
/// Use this instead of reading `CardInstance` fields directly when layers matter.
///
/// The four characteristics are private [`Overlay`]s read through same-named
/// accessors ([`card_types`](Self::card_types) …), each projecting into the
/// single `def` handle (PERF `(-70)`).
#[derive(Clone)]
pub struct ComputedPermanent {
    pub id: CardId,
    pub controller: usize,
    /// The printed definition every overlay above projects into.
    pub def: std::sync::Arc<crate::card::CardDefinition>,
    card_types: Overlay<CardTypesOf>,
    supertypes: Overlay<SupertypesOf>,
    subtypes: Overlay<SubtypesOf>,
    pub colors: ColorSet,
    keywords: OverlayList<KeywordsOf>,
    pub power: i32,
    pub toughness: i32,
    /// CR 613.4 — base P/T (after 7a/7b, before 7c, counters, 7d), a nibble
    /// each: 0..=13 exact, 14 = negative, 15 = 14+. Readers only compare with
    /// a small printed N (Bess, Soul Nourisher); the byte rides padding.
    /// Read through [`Self::base_pt`].
    base_pt_nibbles: u8,
    /// A `RemoveAllAbilities` effect is in scope: trigger dispatch and
    /// ability resolution skip the printed abilities too, not just keywords
    /// (CR 113.10 — Turn to Frog, Lignify).
    pub lost_all_abilities: bool,
    /// [`Self::creature_types_set`] and [`Self::keeps_mana_abilities`] as bits:
    /// one byte, so the struct stays 72 (PERF's padding probe).
    flags: u8,
}

const CREATURE_TYPES_SET: u8 = 1;
const KEEPS_MANA_ABILITIES: u8 = 2;

impl ComputedPermanent {
    /// CR 702.73a / 613.1d — a layer-4 effect set this permanent's creature
    /// types (Curse of Conformity's "lose all creature types"), which
    /// overrides changeling's characteristic-defining "every type".
    pub fn creature_types_set(&self) -> bool {
        self.flags & CREATURE_TYPES_SET != 0
    }

    /// CR 113.10b — the strip behind `lost_all_abilities` is only Blood Sun's
    /// "except mana abilities": the printed mana abilities still activate.
    pub fn keeps_mana_abilities(&self) -> bool {
        self.flags & KEEPS_MANA_ABILITIES != 0
    }

    /// CR 613.4 — base power and toughness (see `base_pt_nibbles`): exact in
    /// 0..=13, `-1` standing for any negative value, `i32::MAX` for 14 or more
    /// — so `==`/`<=` against a small nonnegative N answer exactly.
    pub fn base_pt(&self) -> (i32, i32) {
        let get = |n: u8| match n {
            14 => -1,
            15 => i32::MAX,
            n => i32::from(n),
        };
        (get(self.base_pt_nibbles & 15), get(self.base_pt_nibbles >> 4))
    }

    /// CR 613 layer 4 — the computed card types.
    #[inline]
    pub fn card_types(&self) -> &Vec<CardType> {
        self.card_types.get(&self.def)
    }

    /// CR 613 layer 4 — the computed supertypes.
    #[inline]
    pub fn supertypes(&self) -> &Vec<Supertype> {
        self.supertypes.get(&self.def)
    }

    /// CR 613 layer 4 — the computed subtypes.
    #[inline]
    pub fn subtypes(&self) -> &Subtypes {
        self.subtypes.get(&self.def)
    }

    /// CR 613 layer 6 — the computed keywords.
    #[inline]
    pub fn keywords(&self) -> &[Keyword] {
        self.keywords.get(&self.def)
    }
}

/// Prints the *effective* characteristics, which is what the four `Printed`
/// fields' own `Debug` used to do before they became projections.
impl std::fmt::Debug for ComputedPermanent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputedPermanent")
            .field("id", &self.id)
            .field("controller", &self.controller)
            .field("card_types", self.card_types())
            .field("supertypes", self.supertypes())
            .field("subtypes", self.subtypes())
            .field("colors", &self.colors)
            .field("keywords", &self.keywords())
            .field("power", &self.power)
            .field("toughness", &self.toughness)
            .field("lost_all_abilities", &self.lost_all_abilities)
            .field("keeps_mana_abilities", &self.keeps_mana_abilities())
            .field("creature_types_set", &self.creature_types_set())
            .finish()
    }
}

// ── Layer application ─────────────────────────────────────────────────────────

/// Apply all active continuous effects to every permanent in `battlefield`,
/// returning one `ComputedPermanent` per card instance.
///
/// `effects` must already be filtered to only active (non-expired) effects.
pub fn apply_layers(
    battlefield: &[crate::card::CardInstance],
    effects: &[ContinuousEffect],
) -> Vec<ComputedPermanent> {
    let gates = SecondPass::of(effects);
    battlefield
        .iter()
        .map(|card| compute_permanent_gated(card, effects, gates))
        .collect()
}

/// Whether the CR 613.8 gated re-runs can fire at all. A property of the
/// effect list, not the permanent, so a battlefield pass computes it once, in
/// one walk (separate `any()`s each run to the end on the common board).
/// Packed into a `u16` ([`Flags`] unpacked) because it rides inline in
/// `GameState`'s gather memos, capped by `cow::tests::game_state_stays_small`.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SecondPass(u16);

/// [`SecondPass`] unpacked.
#[derive(Clone, Copy, Default)]
struct Flags {
    power: bool,
    type_changer: bool,
    type_lord: bool,
    /// CR 613.8 — a layer-4 card-type change and an effect whose set is
    /// filtered by card type: the set must read the computed types.
    card_type_changer: bool,
    card_type_filtered: bool,
    /// CR 613.8 — a layer-5 color change and an effect whose set is
    /// filtered by color (Honor of the Pure): the set reads computed colors.
    color_changer: bool,
    color_filtered: bool,
    /// CR 613.8 — a layer-6 keyword change and an effect whose set is
    /// filtered by a keyword (Favorable Winds): the set reads computed ones.
    keyword_changer: bool,
    keyword_filtered: bool,
}

impl SecondPass {
    fn pack(f: Flags) -> Self {
        let bits = [
            f.power,
            f.type_changer,
            f.type_lord,
            f.card_type_changer,
            f.card_type_filtered,
            f.color_changer,
            f.color_filtered,
            f.keyword_changer,
            f.keyword_filtered,
        ];
        SecondPass(bits.iter().enumerate().fold(0, |m, (i, b)| m | (u16::from(*b) << i)))
    }

    fn unpack(self) -> Flags {
        let b = |i: u16| self.0 & (1 << i) != 0;
        Flags {
            power: b(0),
            type_changer: b(1),
            type_lord: b(2),
            card_type_changer: b(3),
            card_type_filtered: b(4),
            color_changer: b(5),
            color_filtered: b(6),
            keyword_changer: b(7),
            keyword_filtered: b(8),
        }
    }

    pub(crate) fn of(effects: &[ContinuousEffect]) -> Self {
        let mut g = Flags::default();
        for e in effects {
            match &e.affected {
                AffectedPermanents::CardMatchPowerGated { .. } => g.power = true,
                AffectedPermanents::AllWithCreatureType { .. } => g.type_lord = true,
                AffectedPermanents::All { card_types, .. }
                | AffectedPermanents::AllOpponents { card_types, .. }
                | AffectedPermanents::AllWithCounter { card_types, .. }
                    if !card_types.is_empty() =>
                {
                    g.card_type_filtered = true
                }
                AffectedPermanents::CardMatch { requirement, .. } => {
                    // One walk for the four leaf families a pass 2 can feed.
                    let bits = requirement_leaf_bits(requirement);
                    g.card_type_filtered |= bits & leaf::CARD_TYPE != 0;
                    g.type_lord |= bits & leaf::CREATURE_TYPE != 0;
                    g.color_filtered |= bits & leaf::COLOR != 0;
                    g.keyword_filtered |= bits & leaf::KEYWORD != 0;
                }
                _ => {}
            }
            g.color_filtered |= match &e.affected {
                AffectedPermanents::All { color, colorless, .. } => color.is_some() || *colorless,
                AffectedPermanents::AllOpponents { color, .. } => color.is_some(),
                _ => false,
            };
            match e.modification {
                Modification::SetCreatureTypes(_)
                | Modification::AddCreatureType(_)
                | Modification::RemoveCreatureType(_) => g.type_changer = true,
                // CR 702.73a — a granted changeling makes the bearer every type.
                Modification::AddKeyword(Keyword::Changeling) | Modification::EveryCreatureType => {
                    g.type_changer = true;
                    g.keyword_changer = true;
                }
                Modification::AddCardType(_) | Modification::RemoveCardType(_) | Modification::SetCardTypes(_) => {
                    g.card_type_changer = true
                }
                Modification::AddColor(_) | Modification::SetColors(_) | Modification::LoseAllColors => {
                    g.color_changer = true
                }
                Modification::AddKeyword(_)
                | Modification::RemoveKeyword(_)
                | Modification::CantHaveKeyword(_)
                | Modification::RemoveAllAbilities
                | Modification::RemoveAllAbilitiesButMana => g.keyword_changer = true,
                _ => {}
            }
        }
        Self::pack(g)
    }
}

/// The CR 613.8 second pass's computed reads, one per characteristic a set
/// can be filtered by. `None` (or `ChangelingRead::Instance`) reads the
/// printed / instance value — the first pass, and every non-layer caller.
#[derive(Clone, Copy)]
pub(crate) struct Gates<'a> {
    /// The card's gate-free power (power-gated sets).
    power: Option<i32>,
    /// The computed creature types.
    types: Option<&'a [CreatureType]>,
    /// The computed card types.
    card_types: Option<&'a [CardType]>,
    changeling: ChangelingRead,
    /// The computed colors (layer 5).
    colors: Option<ColorSet>,
    /// The computed keywords (layer 6).
    keywords: Option<&'a [Keyword]>,
}

impl Gates<'_> {
    const PRINTED: Gates<'static> = Gates {
        power: None,
        types: None,
        card_types: None,
        changeling: ChangelingRead::Instance,
        colors: None,
        keywords: None,
    };
}

/// How a type-gated match reads changeling (CR 702.73a / 613.1d).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ChangelingRead {
    /// A layer-4 effect set the creature types: changeling counts for nothing.
    Off,
    /// The instance's own changeling (printed or a resolved grant) counts.
    Instance,
    /// Pass 1 found changeling granted by a static: every creature type.
    Every,
}

impl ChangelingRead {
    fn every_type(self, card: &crate::card::CardInstance) -> bool {
        match self {
            ChangelingRead::Off => false,
            ChangelingRead::Instance => card.has_keyword(&Keyword::Changeling),
            ChangelingRead::Every => true,
        }
    }
}

/// Apply all active continuous effects to a single permanent, returning
/// its `ComputedPermanent` — one iteration of [`apply_layers`].
pub fn apply_layers_one(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
) -> ComputedPermanent {
    compute_permanent(card, effects)
}

/// [`apply_layers_one`] with the list's [`SecondPass`] gates precomputed, for
/// a caller applying one list to several permanents. The gates must come
/// from the same `effects`.
pub(crate) fn apply_layers_one_gated(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
    gates: SecondPass,
) -> ComputedPermanent {
    compute_permanent_gated(card, effects, gates)
}

fn compute_permanent(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
) -> ComputedPermanent {
    compute_permanent_gated(card, effects, SecondPass::of(effects))
}

fn compute_permanent_gated(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
    gates: SecondPass,
) -> ComputedPermanent {
    // Power-gated effects (CR 613.8) need the card's gate-free power first:
    // pass 1 excludes them, pass 2 re-runs the full walk with each gate
    // resolved against that power. Keyword grants don't feed back into
    // layer 7, so the result is a fixpoint after one re-run.
    //
    // CR 613.8 — a creature-type lord (AllWithCreatureType) must see the
    // *computed* types, so a creature animated/retyped into the lord's tribe
    // (Turn to Frog into a Frog lord, Arcane Adaptation, etc.) gets the buff.
    // Only re-run when both a type-changer and a type lord are present.
    let gates = gates.unpack();
    let has_type_gated = gates.type_changer && gates.type_lord;
    let has_card_type_gated = gates.card_type_changer && gates.card_type_filtered;
    let has_color_gated = gates.color_changer && gates.color_filtered;
    let has_keyword_gated = gates.keyword_changer && gates.keyword_filtered;
    if gates.power || has_type_gated || has_card_type_gated || has_color_gated || has_keyword_gated {
        return compute_permanent_second_pass(
            card,
            effects,
            has_type_gated,
            has_card_type_gated,
            has_color_gated,
            has_keyword_gated,
        );
    }
    compute_permanent_pass(card, effects, Gates::PRINTED)
}

/// The CR 613.8 re-run, cold and out of line: almost no call takes it, and
/// inlining it put `pass1`'s frame in every caller's entry block. PERF
/// `(-131)` (rule from `(-129)`).
#[cold]
#[inline(never)]
fn compute_permanent_second_pass(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
    has_type_gated: bool,
    has_card_type_gated: bool,
    has_color_gated: bool,
    has_keyword_gated: bool,
) -> ComputedPermanent {
    let pass1 = compute_permanent_pass(card, effects, Gates::PRINTED);
    let gate_types = has_type_gated.then(|| pass1.subtypes().creature_types.clone());
    let gate_card_types = has_card_type_gated.then(|| pass1.card_types().clone());
    // CR 613.1d — a layer-4 "set creature types" overrides changeling's CDA,
    // so the type-gated re-run stops reading the keyword as every type; a
    // changeling a static granted (Maskwood Nexus) is every type to it too.
    let changeling = if pass1.creature_types_set() {
        ChangelingRead::Off
    } else if pass1.keywords().contains(&Keyword::Changeling) {
        ChangelingRead::Every
    } else {
        ChangelingRead::Instance
    };
    let gate_colors = has_color_gated.then_some(pass1.colors);
    let gate_keywords = has_keyword_gated.then(|| pass1.keywords().to_vec());
    compute_permanent_pass(
        card,
        effects,
        Gates {
            power: Some(pass1.power),
            types: gate_types.as_deref(),
            card_types: gate_card_types.as_deref(),
            changeling,
            colors: gate_colors,
            keywords: gate_keywords.as_deref(),
        },
    )
}

fn compute_permanent_pass(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
    gates: Gates,
) -> ComputedPermanent {
    // Start from the base card definition — borrowed, not cloned: each of
    // these materializes only if a layer below actually writes to it.
    let mut controller = card.controller;
    let def = &card.definition;
    let mut card_types = Printed::<CardTypesOf>::new(def);
    let mut supertypes = Printed::<SupertypesOf>::new(def);
    let mut subtypes = Printed::<SubtypesOf>::new(def);
    // CR 702.103d — while bestowed, the permanent is an Aura enchantment,
    // not a creature.
    if card.bestowed {
        card_types.retain(|t| !matches!(t, CardType::Creature));
        if !card_types.contains(&CardType::Enchantment) {
            card_types.push(CardType::Enchantment);
        }
        if !subtypes
            .enchantment_subtypes
            .contains(&crate::card::EnchantmentSubtype::Aura)
        {
            subtypes
                .enchantment_subtypes
                .push(crate::card::EnchantmentSubtype::Aura);
        }
    }
    let mut colors = colors_from_card(card);
    let mut keywords = PrintedList::<KeywordsOf>::new(def);
    // The EOT keyword grants, keyword counters and removal lists are
    // `CardCold` members, so one byte settles all three guards below on a
    // card that never wrote the group (PERF `(-351)`).
    let cold_pristine = card.cold_pristine();
    debug_assert!(
        !cold_pristine
            || (card.granted_keywords_eot.is_empty()
                && card.keyword_counters.is_empty()
                && card.removed_keywords_eot.is_empty()
                && card.removed_keywords.is_empty()),
        "cold_written said pristine and a CardCold layer input is set",
    );
    // EOT-granted keywords join the walk as synthetic L6 effects at their
    // grant timestamps (CR 613.7), so a grant after a strip survives it and
    // an earlier one doesn't. Untracked grants default to timestamp 0.
    // Cleared at Cleanup (`clear_end_of_turn_effects`). The emptiness check
    // skips a `collect` call on nearly every permanent.
    let eot_grants: Vec<ContinuousEffect> = if cold_pristine || card.granted_keywords_eot.is_empty()
    {
        Vec::new()
    } else {
        card.granted_keywords_eot
            .iter()
            .enumerate()
            .map(|(i, kw)| ContinuousEffect {
                timestamp: card.granted_keywords_eot_ts.get(i).copied().unwrap_or(0),
                source: card.id,
                affected: AffectedPermanents::Source,
                layer: Layer::L6Ability,
                sublayer: None,
                duration: EffectDuration::UntilEndOfTurn,
                modification: Modification::AddKeyword(kw.clone()),
            })
            .collect()
    };
    // CR 122.1b — keyword counters: each keyword counter type on the
    // permanent grants the named keyword while at least one counter of
    // that type is present. Applied as a layer-6 keyword addition.
    if !cold_pristine {
        for (kw, count) in &card.keyword_counters {
            if *count > 0 && !keywords.contains(kw) {
                keywords.push(kw.clone());
            }
        }
    }

    // Base P/T from the definition. Direct pump bonuses (`power_bonus`,
    // from `Effect::PumpPT`) are layer-7c modifications and so apply
    // *after* a 7b set (CR 613.7c — an animated manland keeps its pumps).
    let base_power = card.definition.base_power();
    let base_toughness = card.definition.base_toughness();
    // CR 613.7f — +1/+1, -1/-1, and the rarer one-sided counters
    // (-0/-1, -0/-2, -1/-0, +1/+0, +0/+1, +2/+0, +0/+2), which affect only one
    // of power/toughness, so track per-stat deltas.
    // One pass over the counters rather than a keyed lookup per kind (a
    // permanent carries one or two kinds at most). The sum is
    // order-independent, so `HashMap` order cannot leak into game state.
    let (counter_power_delta, counter_toughness_delta) = {
        let (mut dp, mut dt) = (0i32, 0i32);
        for (kind, n) in &card.counters {
            let n = *n as i32;
            let (p, t) = match kind {
                CounterType::PlusOnePlusOne => (1, 1),
                CounterType::MinusOneMinusOne => (-1, -1),
                CounterType::MinusZeroMinusOne => (0, -1),
                CounterType::MinusZeroMinusTwo => (0, -2),
                CounterType::MinusOneMinusZero => (-1, 0),
                CounterType::PlusOnePlusZero => (1, 0),
                CounterType::PlusZeroPlusOne => (0, 1),
                CounterType::PlusTwoPlusZero => (2, 0),
                CounterType::PlusZeroPlusTwo => (0, 2),
                CounterType::PlusTwoPlusTwo => (2, 2),
                CounterType::PlusOnePlusTwo => (1, 2),
                CounterType::MinusTwoMinusOne => (-2, -1),
                _ => continue,
            };
            dp += p * n;
            dt += t * n;
        }
        (dp, dt)
    };

    let mut set_pt: Option<(i32, i32)> = None;
    let mut set_power_only: Option<i32> = None;
    let mut set_toughness_only: Option<i32> = None;
    let mut mod_power: i32 = 0;
    let mut mod_toughness: i32 = 0;
    let mut switched = false;
    let mut lost_all_abilities = false;
    let mut full_strip = false;
    let mut strip_all_ts = 0u64;
    let mut creature_types_set = false;
    let mut every_creature_type = false;
    let mut cant_have_keywords: Vec<Keyword> = Vec::new();

    // Sort effects by layer, sublayer, timestamp — building the list only
    // when there is something to put in it (it is usually empty).
    //
    // Skipping the build (gather leaves the list layer-sorted, walk a
    // `filter`) was tried and lost (sixty-eighth pass; PERF's Log).
    //
    // Inline storage, not the heap (PERF `(-106)`): the `Vec` form was the
    // largest allocation context. Eight slots cover a list ~2 long
    // (`(-56b)`, which also refuted four ways of not building it).
    let mut sorted: smallvec::SmallVec<[&ContinuousEffect; 8]> = smallvec::SmallVec::new();
    if !effects.is_empty() || !eot_grants.is_empty() {
        // Two loops rather than one over a `chain` (`Chain::next` branches on
        // every element of the first side). Plain `push` loops, not
        // `extend` (PERF `(-229)`): `SmallVec::extend` over a `Filter` is
        // inlined or not per build, and the out-of-line outcome is costly.
        for e in effects.iter() {
            if affects(e, effects, card, &gates) {
                sorted.push(e);
            }
        }
        for e in eot_grants.iter() {
            sorted.push(e);
        }
        sorted.sort_by(|a, b| {
            a.layer.cmp(&b.layer)
                .then(a.sublayer.cmp(&b.sublayer))
                .then(a.timestamp.cmp(&b.timestamp))
        });
    }

    // By reference: `SmallVec`'s by-value `IntoIter` moves the whole inline
    // buffer.
    for effect in &sorted {
        match &effect.modification {
            // Layer 2
            Modification::ChangeController(c) => controller = *c,

            // Layer 3 — text-changing (CR 612). Runs before type/color/
            // ability layers, so the swapped words feed everything below.
            Modification::ReplaceColorWord(from, to) => {
                for kw in keywords.iter_mut() {
                    if matches!(kw, crate::card::Keyword::Protection(c) if c == from) {
                        *kw = crate::card::Keyword::Protection(*to);
                    }
                }
            }
            Modification::ReplaceBasicLandType(from, to) => {
                for lt in subtypes.land_types.iter_mut() {
                    if lt == from {
                        *lt = *to;
                    }
                }
                for kw in keywords.iter_mut() {
                    if matches!(kw, crate::card::Keyword::Landwalk(l) if l == from) {
                        *kw = crate::card::Keyword::Landwalk(*to);
                    }
                }
            }

            // Layer 4
            Modification::AddCardType(t) => {
                if !card_types.contains(t) { card_types.push(t.clone()); }
            }
            Modification::RemoveCardType(t) => card_types.retain(|x| x != t),
            Modification::SetCardTypes(ts) => *card_types = ts.clone(),
            Modification::AddSupertype(s) => {
                if !supertypes.contains(s) { supertypes.push(*s); }
            }
            Modification::RemoveSupertype(s) => supertypes.retain(|x| x != s),
            Modification::AddCreatureType(ct) => {
                if !subtypes.creature_types.contains(ct) {
                    subtypes.creature_types.push(*ct);
                }
            }
            Modification::RemoveCreatureType(ct) => {
                if !every_creature_type {
                    subtypes.creature_types.retain(|x| x != ct);
                }
            }
            Modification::SetCreatureTypes(cts) => {
                subtypes.creature_types = cts.clone();
                creature_types_set = true;
                every_creature_type = false;
            }
            Modification::EveryCreatureType => {
                every_creature_type = true;
                creature_types_set = false;
            }
            Modification::AddLandType(lt) => {
                if !subtypes.land_types.contains(lt) {
                    subtypes.land_types.push(*lt);
                }
            }
            Modification::SetLandTypes(lts) => subtypes.land_types = lts.clone(),
            Modification::SetArtifactSubtypes(ats) => subtypes.artifact_subtypes = ats.clone(),
            Modification::AddArtifactSubtype(at) => {
                if !subtypes.artifact_subtypes.contains(at) {
                    subtypes.artifact_subtypes.push(*at);
                }
            }

            // Layer 5
            Modification::AddColor(c) => colors.insert(*c),
            Modification::SetColors(cs) => colors = cs.iter().collect(),
            Modification::LoseAllColors => colors.clear(),

            // Layer 6. Duplicate grants of a binary keyword collapse to one,
            // but Toxic/Poisonous instances are cumulative (CR 702.180b /
            // 702.70b — "total toxic value" sums every instance; Plague
            // Nurse's granted toxic 1 stacks with a printed toxic 1).
            Modification::AddKeyword(kw) => {
                // Ward too: each instance is its own trigger (CR 113.2c), so
                // a granted ward {1} beside a printed one charges twice; and
                // riot, whose instances each work separately (CR 702.136b).
                let cumulative =
                    matches!(kw, Keyword::Toxic(_) | Keyword::Poisonous(_) | Keyword::Ward(_) | Keyword::Riot);
                if cumulative || !keywords.contains(kw) {
                    keywords.push(kw.clone());
                }
            }
            Modification::RemoveKeyword(kw) => keywords.retain(|k| k != kw),
            Modification::CantHaveKeyword(kw) => {
                keywords.retain(|k| k != kw);
                cant_have_keywords.push(kw.clone());
            }
            Modification::RemoveAllAbilities => {
                keywords.clear();
                lost_all_abilities = true;
                full_strip = true;
                strip_all_ts = effect.timestamp;
            }
            // Changes no characteristic (trigger dispatch counts it).
            Modification::TriggersAdditionalTime => {}
            Modification::RemoveAllAbilitiesButMana => {
                keywords.clear();
                lost_all_abilities = true;
                strip_all_ts = effect.timestamp;
            }

            // Layer 7 (modifications arrive in timestamp order; later wins)
            Modification::SetPowerToughness(p, t) => {
                set_pt = Some((*p, *t));
                set_power_only = None;
                set_toughness_only = None;
            }
            Modification::SetPowerToughnessToManaValue => {
                let mv = card.definition.cost.cmc() as i32;
                set_pt = Some((mv, mv));
                set_power_only = None;
            }
            // Rewritten by the gather; one that reaches here sets nothing.
            Modification::SetPowerToughnessLive(_) => {}
            Modification::SetPower(p) => set_power_only = Some(*p),
            Modification::SetToughness(t) => set_toughness_only = Some(*t),
            // Saturating: a pump sized by a power can reach i32::MAX, and two
            // until-end-of-combat ones overflowed the sum.
            Modification::ModifyPower(n) => mod_power = mod_power.saturating_add(*n),
            Modification::ModifyToughness(n) => mod_toughness = mod_toughness.saturating_add(*n),
            Modification::ModifyPowerToughness(p, t) => {
                mod_power = mod_power.saturating_add(*p);
                mod_toughness = mod_toughness.saturating_add(*t);
            }
            Modification::ModifyPtPerOwnCreatureType(pp, pt, max) => {
                // Creature types are finalized at layer 4, processed before
                // this layer-7 modification (effects arrive layer-sorted).
                let n = (subtypes.creature_types.len() as u32).min(*max) as i32;
                mod_power += pp * n;
                mod_toughness += pt * n;
            }
            Modification::ModifyPowerPerTimesAttacked(pp) => {
                mod_power += pp * i32::from(card.times_attacked_this_turn);
            }
            Modification::ModifyPtPerOwnColorIfMulticolored(pp, pt) => {
                // Colors are final at layer 5, before this layer-7 effect.
                let n = colors.len() as i32;
                if n >= 2 {
                    mod_power += pp * n;
                    mod_toughness += pt * n;
                }
            }
            Modification::SwitchPowerToughness => switched = !switched,
        }
    }
    // CR 613.7 — keyword counters put after the latest "loses all abilities"
    // keep their keywords (Abigale, Eloquent First-Year).
    if lost_all_abilities && !cold_pristine && card.keyword_counters_ts > strip_all_ts {
        for (kw, count) in &card.keyword_counters {
            if *count > 0 && !keywords.contains(kw) {
                keywords.push(kw.clone());
            }
        }
    }

    // Until-EOT keyword removals (Shadowspear) apply last — the removal's
    // timestamp is by construction later than any static grant this turn.
    // CR 702.22 — "loses all 'bands with other' abilities" (Shelkin Brownie,
    // Tolaria) can't name a payload, so a removal of `BandsWithOther` strips
    // every instance whatever its quality.
    // Gated on non-empty input: `retain` materializes the printed list even
    // when it removes nothing.
    if !cold_pristine
        && (!card.removed_keywords_eot.is_empty() || !card.removed_keywords.is_empty())
    {
        let removes_all_bands = card
            .removed_keywords_eot
            .iter()
            .chain(&card.removed_keywords)
            .any(|k| matches!(k, Keyword::BandsWithOther(_)));
        keywords.retain(|k| {
            !(card.removed_keywords_eot.contains(k)
                || card.removed_keywords.contains(k)
                || (removes_all_bands && matches!(k, Keyword::BandsWithOther(_))))
        });
    }
    // CR 113.11 — "can't have or gain" beats any grant, whatever its
    // timestamp (Archetype of Courage vs. a later first-strike anthem).
    if !cant_have_keywords.is_empty() {
        keywords.retain(|k| !cant_have_keywords.contains(k));
    }
    // CR 613.1d / 613.3 — a layer-4 "every creature type", or a printed
    // changeling's CDA (applied first in layer 4), survives a layer-6 strip;
    // it reads as changeling to each type check.
    let every_creature_type = every_creature_type
        || (lost_all_abilities && !creature_types_set && def.keywords.contains(&Keyword::Changeling));
    if every_creature_type && !keywords.contains(&Keyword::Changeling) {
        keywords.push(Keyword::Changeling);
    } else if !every_creature_type && creature_types_set && keywords.contains(&Keyword::Changeling) {
        // A later layer-4 "set" overwrote changeling's every type (the
        // changeling ruling); the computed view stops answering every type.
        keywords.retain(|k| *k != Keyword::Changeling);
    }

    // Compute final P/T.
    let (mut power, mut toughness) = if let Some((p, t)) = set_pt {
        (p, t)
    } else {
        (base_power, base_toughness)
    };
    // Layer-7b "base power becomes N" (power only) overrides the power half.
    if let Some(p) = set_power_only {
        power = p;
    }
    // Layer-7b "base toughness becomes N" (toughness only) overrides the
    // toughness half (Maha's opponent debuff).
    if let Some(t) = set_toughness_only {
        toughness = t;
    }
    let nib = |n: i32| match n {
        ..0 => 14,
        0..=13 => n as u8,
        _ => 15,
    };
    let base_pt_nibbles = nib(power) | (nib(toughness) << 4);
    // Saturating, as in `CardInstance::pump` (Exponential Growth can reach
    // `i32::MAX`). Saturation is per step and the order is load-bearing, so
    // the adds cannot be folded into one `i64` sum (`MAX + 1 - 1` differs).
    // When all eight terms are zero — most permanents — the adds are
    // skipped behind one OR (PERF `(-308)`).
    if (mod_power
        | mod_toughness
        | card.power_bonus
        | card.toughness_bonus
        | card.perm_power_bonus
        | card.perm_toughness_bonus
        | counter_power_delta
        | counter_toughness_delta)
        != 0
    {
        power = power
            .saturating_add(mod_power)
            .saturating_add(card.power_bonus)
            .saturating_add(card.perm_power_bonus);
        toughness = toughness
            .saturating_add(mod_toughness)
            .saturating_add(card.toughness_bonus)
            .saturating_add(card.perm_toughness_bonus);
        // Counters applied after 7c (CR 613.7f).
        power = power.saturating_add(counter_power_delta);
        toughness = toughness.saturating_add(counter_toughness_delta);
    }
    if switched {
        std::mem::swap(&mut power, &mut toughness);
    }

    ComputedPermanent {
        id: card.id,
        controller,
        // The one handle: the four overlays above dropped theirs.
        def: card.definition.arc(),
        card_types: card_types.into_overlay(),
        supertypes: supertypes.into_overlay(),
        subtypes: subtypes.into_overlay(),
        colors,
        keywords: keywords.into_overlay(),
        power,
        toughness,
        base_pt_nibbles,
        lost_all_abilities,
        flags: if creature_types_set { CREATURE_TYPES_SET } else { 0 }
            | if lost_all_abilities && !full_strip { KEEPS_MANA_ABILITIES } else { 0 },
    }
}

/// Determine which colors a card has from its mana cost symbols
/// (CR 105.2 + the Devoid CDA).
fn colors_from_card(card: &crate::card::CardInstance) -> ColorSet {
    // CR 709.5 — a Room permanent has only its unlocked doors' mana cost, so
    // its colors are theirs: a locked Room is colorless, not both doors'
    // colors (the requirement walker already read it this way).
    if card.definition.room.is_some()
        && let Some(v) = card.room_battlefield_view()
    {
        return v.definition.printed_color_set();
    }
    // Memoized (`card::CardMemo`): the costliest of the pass's definition
    // reads, and it cannot change without a write to the card.
    card.printed_color_set()
}

/// Returns true if `effect` affects `card`. `gate_power` is the card's
/// gate-free computed power for `CardMatchPowerGated` effects; `None` (pass 1)
/// excludes them.
#[allow(clippy::too_many_arguments)]
fn affects(
    effect: &ContinuousEffect,
    all: &[ContinuousEffect],
    card: &crate::card::CardInstance,
    gates: &Gates,
) -> bool {
    // CR 613.6 — an effect that starts applying in layer 4 keeps the set it
    // had there, fixed before layer 4 changed any type. So in the second
    // pass a `CardMatch` that is part of a card-type changer (same source,
    // same requirement) reads printed types; otherwise March of the
    // Machines' "each noncreature artifact" would read its own output.
    let gate_card_types = match (&effect.affected, gates.card_types) {
        (AffectedPermanents::CardMatch { requirement, .. }, Some(_))
            if all.iter().any(|o| {
                o.source == effect.source
                    && matches!(
                        o.modification,
                        Modification::AddCardType(_) | Modification::RemoveCardType(_) | Modification::SetCardTypes(_)
                    )
                    && matches!(&o.affected, AffectedPermanents::CardMatch { requirement: r, .. } if r == requirement)
            }) =>
        {
            None
        }
        _ => gates.card_types,
    };
    affected_includes_gated(&effect.affected, effect.source, card, &Gates { card_types: gate_card_types, ..*gates })
}

/// Whether `card` is one of the permanents described by `affected`, given the
/// describing effect's `source` permanent — the enters-tapped replacement
/// (CR 614.13) reuses it without building a `ContinuousEffect`. Power-gated
/// matches read printed power + counters (callers with a computed view use
/// [`affected_includes_gated`]).
pub(crate) fn affected_includes(
    affected: &AffectedPermanents,
    source: CardId,
    card: &crate::card::CardInstance,
) -> bool {
    let printed_power = card.definition.base_power()
        + card.counter_count(CounterType::PlusOnePlusOne) as i32
        - card.counter_count(CounterType::MinusOneMinusOne) as i32;
    affected_includes_gated(affected, source, card, &Gates { power: Some(printed_power), ..Gates::PRINTED })
}

#[allow(clippy::too_many_arguments)]
fn affected_includes_gated(
    affected: &AffectedPermanents,
    source: CardId,
    card: &crate::card::CardInstance,
    gates: &Gates,
) -> bool {
    let Gates {
        power: gate_power,
        types: gate_types,
        card_types: gate_card_types,
        changeling: changeling_types,
        colors: gate_colors,
        keywords: gate_keywords,
    } = *gates;
    let computed_types = gate_card_types.unwrap_or(&card.definition.card_types);
    let colors = || gate_colors.unwrap_or_else(|| colors_from_card(card));
    match affected {
        AffectedPermanents::Source => source == card.id,
        AffectedPermanents::Specific(ids) => ids.contains(&card.id),
        AffectedPermanents::All {
            controller,
            card_types,
            exclude_source,
            color,
            token,
            colorless,
            owned_by_controller,
        } => {
            if *exclude_source && source == card.id {
                return false;
            }
            // Cheapest-first `&&` chain: the seat compare rejects about half
            // the board, and LLVM won't sink the colour walks past an AND.
            controller.is_none_or(|c| c == card.controller)
                && token.is_none_or(|want| card.is_token == want)
                && owned_by_controller
                    .is_none_or(|want| (card.owner == card.controller) == want)
                && (card_types.is_empty()
                    || card_types.iter().all(|t| computed_types.contains(t)))
                // CR 105.2 — coloured pips (hybrid, Phyrexian too) UNION the
                // colour indicator, empty under Devoid; not `Colored(_)` pips
                // alone, which missed {G/W} creatures and indicator tokens.
                && color.is_none_or(|want| colors().contains(want))
                // CR 702.114 — Devoid is folded into `printed_color_set`.
                && (!*colorless || colors().is_empty())
        }
        AffectedPermanents::AllOpponents {
            source_controller,
            card_types,
            friendly_seats,
            color,
            creature_type,
            counter,
        } => {
            // Empty `friendly_seats` → legacy 1v1 check (snapshots from
            // before push-XLIII didn't populate it).
            let ctrl_ok = if friendly_seats.is_empty() {
                card.controller != *source_controller
            } else {
                !friendly_seats.contains(&card.controller)
            };
            // Short-circuit, cheapest first — see the `All` arm.
            ctrl_ok
                && (card_types.is_empty()
                    || card_types.iter().all(|t| computed_types.contains(t)))
                && counter.is_none_or(|k| card.counter_count(k) > 0)
                && color.is_none_or(|want| colors().contains(want))
                && creature_type.as_ref().is_none_or(|ct| {
                    let typed = match gate_types {
                        Some(types) => types.contains(ct),
                        None => card.definition.subtypes.creature_types.contains(ct),
                    };
                    typed || changeling_types.every_type(card)
                })
        }
        AffectedPermanents::AllWithCreatureType { controller, creature_type, exclude_source } => {
            if *exclude_source && source == card.id {
                return false;
            }
            controller.is_none_or(|c| c == card.controller)
                && card.definition.card_types.contains(&CardType::Creature)
                // CR 613.8 — match against computed types when a type-changer
                // is in play (pass 2), else the printed type line.
                && (match gate_types {
                    Some(types) => types.contains(creature_type),
                    None => {
                        card.definition.subtypes.creature_types.contains(creature_type)
                    }
                } || changeling_types.every_type(card))
        }
        AffectedPermanents::AllWithCounter { controller, card_types, counter, at_least } => {
            controller.is_none_or(|c| c == card.controller)
                && (card_types.is_empty()
                    || card_types.iter().all(|t| computed_types.contains(t)))
                && card.counter_count(*counter) >= *at_least
        }
        AffectedPermanents::CardMatch { source_controller, requirement } => {
            // "This creature is the chosen type in addition to its other
            // types" (Roaming Throne) — a bare `IsSource` requirement names
            // the source itself, which only this arm can see.
            if matches!(**requirement, crate::card::SelectionRequirement::IsSource) {
                return source == card.id;
            }
            // CR "other ... you control": `OtherThanSource` is matched here
            // (where the source id is known) rather than in the source-blind
            // `requirement_matches_card`, which treats it as always-true.
            if requirement_mentions_other_than_source(requirement) && source == card.id {
                return false;
            }
            // CR 709.5 — a Room permanent has only its unlocked doors' cost
            // (mana value, uncomputed colors) and names.
            let room;
            let card = match card.room_battlefield_view() {
                Some(v) => {
                    room = v;
                    &room
                }
                None => card,
            };
            requirement_matches_card_typed(
                requirement,
                card,
                *source_controller,
                computed_types,
                gate_types.map(|t| (t, changeling_types)),
                gate_colors,
                gate_keywords,
            )
        }
        AffectedPermanents::CardMatchPowerGated { source_controller, requirement, power_at_least } => {
            let Some(power) = gate_power else { return false };
            if power < *power_at_least {
                return false;
            }
            if requirement_mentions_other_than_source(requirement) && source == card.id {
                return false;
            }
            requirement_matches_card(requirement, card, *source_controller)
        }
    }
}

/// True when every leaf of `req` is readable from the card instance alone
/// (no combat or zone state), so [`requirement_matches_card`] can match it
/// without a `GameState` and `affected_from_requirement` may route it
/// through [`AffectedPermanents::CardMatch`].
pub(crate) fn requirement_is_card_only(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::Any | R::Permanent | R::PermanentCard | R::Creature | R::Artifact | R::Enchantment
        | R::Planeswalker | R::Land | R::Nonland | R::Noncreature | R::IsBasicLand
        | R::IsNonbasicLand | R::IsToken | R::NotToken | R::ControlledByYou
        | R::ControlledByOpponent | R::Colorless => true,
        // Color counts read the printed colors, like `HasColor` (Rienne's
        // "other multicolored creatures you control").
        R::Multicolored | R::Monocolored => true,
        // An instance field like `is_token`. A leaf missing from this list
        // drops the whole static (`card_match_fallback` returns `None`).
        R::OwnedByYou => true,
        // Live instance state, re-read each layer recompute (Augusta, Dean
        // of Order).
        R::Tapped | R::Untapped | R::Unattached | R::AttachedToYou => true,
        // Face-down is likewise a live `CardInstance` flag (Ixidor, Reality
        // Sculptor's "face-down creatures get +1/+1").
        R::FaceDown => true,
        // The printed activated abilities (Raggadragga's "each creature you
        // control with a mana ability").
        R::HasManaAbility => true,
        // Live counter state likewise (Innkeeper's Talent).
        R::WithAnyCounter | R::WithCounter(_) => true,
        R::IsOutlaw | R::HasColor(_) | R::HasCreatureType(_) | R::HasLandType(_) | R::HasSupertype(_)
        | R::HasArtifactSubtype(_) | R::HasEnchantmentSubtype(_) | R::HasCardType(_)
        | R::HasMutate => true,
        // Keyword presence is live instance state too; see
        // `requirement_matches_card`'s doc.
        R::HasKeyword(_) | R::HasToxic | R::HasCumulativeUpkeep | R::HasModular | R::HasProtection | R::HasPartner => true,
        // OtherThanSource is matched in `affects()` (which knows the source id),
        // so it's safe to route a filter containing it through CardMatch.
        R::OtherThanSource => true,
        // Read against printed P/T + counters on each layer recompute (the
        // same approximation `CardMatchPowerGated` uses) — Tapestry Warden.
        R::ToughnessGreaterThanPower | R::PowerToughnessUnequal => true,
        // Mana value is a printed characteristic, read like the type leaves.
        R::ManaValueAtMost(_) | R::ManaValueAtLeast(_) | R::ManaValueExactly(_) => true,
        // So is a name (Leitmotif Composer's name-scoped grant was dropped
        // without this).
        R::HasName(_) => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_is_card_only(a) && requirement_is_card_only(b)
        }
        R::Not(inner) => requirement_is_card_only(inner),
        _ => false,
    }
}

/// True if `req` contains an `OtherThanSource` leaf anywhere — the source
/// exclusion is then applied in `affects()` against the live source id.
fn requirement_mentions_other_than_source(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::OtherThanSource => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_other_than_source(a) || requirement_mentions_other_than_source(b)
        }
        R::Not(inner) => requirement_mentions_other_than_source(inner),
        _ => false,
    }
}

/// Evaluate a card-only requirement (see [`requirement_is_card_only`]) against
/// a single permanent. `source_controller` resolves `ControlledByYou` /
/// `ControlledByOpponent`. Unsupported leaves resolve to `false`.
///
/// ⚠ The keyword leaves read the instance (`CardInstance::has_keyword` /
/// `has_keyword_tag`), not `definition.keywords`: EOT grants, keyword
/// counters (CR 122.1b) and the removal lists are layer-pass inputs like
/// `Tapped`, so this is not circular, and skipping them mis-scoped keyword
/// anthems. Type leaves stay printed: creature types are layer-4 output,
/// handled by `AllWithCreatureType` plus `gate_types`.
pub(crate) fn requirement_matches_card(
    req: &SelectionRequirement,
    card: &crate::card::CardInstance,
    source_controller: usize,
) -> bool {
    requirement_matches_card_typed(req, card, source_controller, &card.definition.card_types, None, None, None)
}

/// The leaf families of a requirement a CR 613.8 second pass reads from the
/// computed view (`requirement_leaf_bits`).
mod leaf {
    pub const CARD_TYPE: u8 = 1;
    pub const CREATURE_TYPE: u8 = 2;
    pub const COLOR: u8 = 4;
    pub const KEYWORD: u8 = 8;
}

/// Which of [`requirement_matches_card_typed`]'s computed-view leaf families
/// `req` reads: card type, creature type, color, keyword.
fn requirement_leaf_bits(req: &SelectionRequirement) -> u8 {
    use SelectionRequirement as R;
    match req {
        R::Creature | R::Artifact | R::Enchantment | R::Planeswalker | R::Land | R::Nonland | R::Noncreature => {
            leaf::CARD_TYPE
        }
        R::HasCardType(_) => leaf::CARD_TYPE,
        // An outlaw is a creature (CR 613.8 — an animated Assassin Treasure).
        R::IsOutlaw => leaf::CARD_TYPE | leaf::CREATURE_TYPE,
        R::HasCreatureType(_) => leaf::CREATURE_TYPE,
        R::HasColor(_) | R::Colorless | R::Multicolored | R::Monocolored => leaf::COLOR,
        R::HasKeyword(_) | R::HasProtection | R::HasPartner => leaf::KEYWORD,
        R::And(a, b) | R::Or(a, b) => requirement_leaf_bits(a) | requirement_leaf_bits(b),
        R::Not(inner) => requirement_leaf_bits(inner),
        _ => 0,
    }
}

/// [`requirement_matches_card`] with the card-type leaves read from `types`
/// — the CR 613.8 second pass hands in the layer-4 computed types, so "each
/// nonartifact creature" skips a creature a static made an artifact (The
/// Flesh Is Weak). `ctypes` is the second pass's computed creature types and
/// whether changeling still counts (CR 613.1d), else the printed line.
pub(crate) fn requirement_matches_card_typed(
    req: &SelectionRequirement,
    card: &crate::card::CardInstance,
    source_controller: usize,
    types: &[CardType],
    ctypes: Option<(&[CreatureType], ChangelingRead)>,
    // CR 613.8 — the computed colors, else the printed ones.
    colors: Option<ColorSet>,
    // CR 613.8 — the computed keywords, else the instance's own.
    keywords: Option<&[Keyword]>,
) -> bool {
    use SelectionRequirement as R;
    let def = &card.definition;
    // Printed, not `colors_from_card`: this matcher also reads cards off the
    // battlefield, where a Room is both doors (CR 709.4), not its unlocked
    // ones. Battlefield callers pass the computed colors in.
    let color_set = || colors.unwrap_or_else(|| def.printed_color_set());
    let has_ctype = |ct: &CreatureType| match ctypes {
        Some((t, changeling)) => t.contains(ct) || changeling.every_type(card),
        None => def.subtypes.creature_types.contains(ct) || card.has_keyword(&Keyword::Changeling),
    };
    match req {
        R::Any | R::Permanent => true,
        R::PermanentCard => def.is_permanent(),
        R::Creature => types.contains(&CardType::Creature),
        R::Artifact => types.contains(&CardType::Artifact),
        R::Enchantment => types.contains(&CardType::Enchantment),
        R::Planeswalker => types.contains(&CardType::Planeswalker),
        R::Land => types.contains(&CardType::Land),
        R::Nonland => !types.contains(&CardType::Land),
        R::Noncreature => !types.contains(&CardType::Creature),
        R::IsBasicLand => def.is_land() && def.supertypes.contains(&Supertype::Basic),
        R::IsNonbasicLand => def.is_land() && !def.supertypes.contains(&Supertype::Basic),
        R::ProducesColorless => def.produces_colorless(),
        R::IsSnow => def.is_snow(),
        R::IsToken => card.is_token,
        R::NotToken => !card.is_token,
        R::HasManaAbility => def.activated_abilities.iter().any(|a| crate::game::actions::is_mana_ability(&a.effect)),
        // A milled or revealed card's printed abilities (Tazri, Stalwart
        // Survivor's "creature cards with activated abilities that aren't mana
        // abilities"): this walker answered false for both leaves.
        R::HasActivatedAbility => def.has_activated_ability(),
        R::HasTapAbility => def.activated_abilities.iter().any(|a| a.tap_cost),
        R::HasEmbalmOrEternalize => def.has_embalm_or_eternalize(),
        // CR 715.1 — "a card that has an Adventure" among milled/revealed
        // cards (Seek Thrills): this walker answered false.
        R::HasAdventure => def.adventure.is_some(),
        R::HasNonManaActivatedAbility => {
            def.activated_abilities.iter().any(|a| !crate::game::actions::is_mana_ability(&a.effect))
        }
        R::Tapped => card.tapped,
        R::FaceDown => card.face_down,
        R::Untapped => !card.tapped,
        R::Unattached => card.attached_to.is_none() && card.attached_to_player.is_none(),
        R::AttachedToYou => card.attached_to_player == Some(source_controller),
        R::WithCounter(k) => card.counter_count(*k) > 0,
        R::WithAnyCounter => card.has_any_counter(),
        R::ControlledByYou => card.controller == source_controller,
        R::ControlledByOpponent => card.controller != source_controller,
        R::OwnedByYou => card.owner == source_controller,
        R::HasCardType(t) => types.contains(t),
        R::HasSupertype(s) => def.supertypes.contains(s),
        R::HasCreatureType(ct) => has_ctype(ct),
        // OTJ — an outlaw creature (CR 613.8: the second pass's types, so
        // Vihaan's animated Assassin Treasures count).
        R::IsOutlaw => {
            types.contains(&CardType::Creature)
                && crate::game::effects::OUTLAW_TYPES.iter().any(has_ctype)
        }
        R::HasLandType(lt) => def.subtypes.land_types.contains(lt),
        R::HasArtifactSubtype(a) => def.subtypes.artifact_subtypes.contains(a),
        R::HasEnchantmentSubtype(e) => def.subtypes.enchantment_subtypes.contains(e),
        R::HasKeyword(k) => keywords.map_or_else(|| card.has_keyword(k), |kws| kws.contains(k)),
        R::HasToxic => card.has_toxic(),
        R::HasCumulativeUpkeep => card.has_cumulative_upkeep(),
        R::HasModular => card.has_modular(),
        R::HasProtection => keywords.map_or_else(|| card.has_protection(), |kws| kws.iter().any(Keyword::is_protection)),
        R::HasPartner => keywords.map_or_else(|| card.has_partner(), |kws| kws.iter().any(Keyword::is_partner)),
        R::HasMutate => def.mutate.is_some(),
        // CR 105.2 — coloured pips (hybrid, Phyrexian included) UNION the
        // colour indicator, empty under Devoid (Kitchen Finks' {G/W}, an
        // indicator-only token).
        R::HasColor(c) => color_set().contains(c),
        // CR 702.114 — Devoid is a CDA: the object is colorless regardless of
        // its (possibly colored) cost pips.
        R::Colorless => color_set().is_empty(),
        R::Multicolored => color_set().len() >= 2,
        R::Monocolored => color_set().len() == 1,
        R::And(a, b) => {
            requirement_matches_card_typed(a, card, source_controller, types, ctypes, colors, keywords)
                && requirement_matches_card_typed(b, card, source_controller, types, ctypes, colors, keywords)
        }
        R::Or(a, b) => {
            requirement_matches_card_typed(a, card, source_controller, types, ctypes, colors, keywords)
                || requirement_matches_card_typed(b, card, source_controller, types, ctypes, colors, keywords)
        }
        R::Not(inner) => !requirement_matches_card_typed(inner, card, source_controller, types, ctypes, colors, keywords),
        // Source exclusion is enforced in `affects()` (source id known there);
        // treat as always-matching for the printed-characteristics walk.
        R::OtherThanSource => true,
        // Read against printed P/T + counters, the `CardMatchPowerGated`
        // approximation (Tapestry Warden).
        R::ToughnessGreaterThanPower => {
            let plus = card.counter_count(CounterType::PlusOnePlusOne) as i32;
            let minus = card.counter_count(CounterType::MinusOneMinusOne) as i32;
            (def.base_toughness() + plus - minus) > (def.base_power() + plus - minus)
        }
        R::PowerToughnessUnequal => def.base_toughness() != def.base_power(),
        // A card that names itself in its own printed filter (Gravebane
        // Zombie's dies-replacement).
        R::HasName(n) => def.has_name(n),
        // Bello, Bard of the Brambles' "with mana value 4 or greater" is
        // `Not(ManaValueAtMost(3))`: with no arm here the leaf fell to
        // `false` and the `Not` animated every artifact — Mind Stone too.
        R::ManaValueAtMost(n) => def.cost.cmc() <= *n,
        R::ManaValueAtLeast(n) => def.cost.cmc() >= *n,
        R::ManaValueExactly(n) => def.cost.cmc() == *n,
        _ => false,
    }
}
