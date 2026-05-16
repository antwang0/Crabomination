//! Primitive, data-driven effect system.
//!
//! Replaces the earlier variant-per-effect `SpellEffect`/`TriggerCondition`/
//! `EffectCondition`/`StaticAbilityTemplate` quartet with a small set of
//! composable algebras:
//!
//! * [`Selector`] — lazy reference to game objects or players, resolved at
//!   effect-time.
//! * [`Value`]    — numeric expression (counts, life totals, X).
//! * [`Predicate`]— game-state boolean (for conditional effects).
//! * [`Effect`]   — the unified instruction tree executed by the resolver.
//! * [`EventSpec`]— structural trigger filter over the [`GameEvent`] stream.
//! * [`StaticEffect`] — description of a static ability's continuous effect.
//!
//! Everything that was previously a one-off enum variant lives as a tree of
//! these primitives; a new card rarely needs engine changes.

use serde::{Deserialize, Serialize};

use crate::card::{CounterType, Keyword, LandType, SelectionRequirement, TokenDefinition, Zone};
use crate::mana::Color;

// ── PlayerRef / ZoneRef ───────────────────────────────────────────────────────

/// Lightweight reference to one or more players.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerRef {
    /// The controller of the ability/spell.
    You,
    /// A specific chosen target slot (must resolve to a player).
    Target(u8),
    /// Each opponent of the controller.
    EachOpponent,
    /// Every player in turn order.
    EachPlayer,
    /// The active player (whose turn it is).
    ActivePlayer,
    /// The owner of a selected entity.
    OwnerOf(Box<Selector>),
    /// The controller of a selected entity.
    ControllerOf(Box<Selector>),
    /// The player who triggered the event (for triggered abilities).
    Triggerer,
    /// A specific seat index. Used internally to flatten selector-based
    /// player refs (e.g. `OwnerOf(Selector)`) into a concrete seat before
    /// passing them across context boundaries — the original-card lookup
    /// can become stale once the card has been moved out of its source zone.
    Seat(usize),
    /// The player or planeswalker controller being attacked by the source
    /// creature. Resolves to `None` when the source isn't currently
    /// attacking. Used for "defending player" triggers (Goblin Guide,
    /// Hypnotic Specter).
    DefendingPlayer,
}

/// A zone plus optional owner (for zones like Hand/Library/Graveyard that
/// are per-player). Battlefield, Stack, Command are global.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneRef {
    Library(PlayerRef),
    Hand(PlayerRef),
    Graveyard(PlayerRef),
    Exile,
    Battlefield,
    Stack,
    Command,
}

// ── Selector ─────────────────────────────────────────────────────────────────

/// A lazy reference to a (possibly empty, possibly multi-) set of game
/// objects — permanents, cards in other zones, or players.
///
/// Resolved by the effect engine at execution time against the current game
/// state; used as the operand of most [`Effect`] mutations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    /// The source permanent/spell/ability itself.
    This,
    /// The ability/spell's controller as a "player object" (for damage, etc).
    You,
    /// A chosen target slot from the cast-time target list.
    Target(u8),
    /// A chosen target slot with a restriction that must be validated at cast time.
    TargetFiltered { slot: u8, filter: SelectionRequirement },
    /// The object that caused this trigger (attacker, dying creature, etc).
    TriggerSource,
    /// The player/object that answered a pending decision.
    ChoiceResult(u8),

    /// The most-recently-created token from `Effect::CreateToken` in
    /// the current resolution. Used by Quandrix-style "create a token,
    /// then put X +1/+1 counters on it" cards (Fractal Anomaly,
    /// Applied Geometry). Resets between resolution roots — within a
    /// single `Effect::Seq`, the latest CreateToken's id is visible.
    LastCreatedToken,

    /// The chosen target slot (0-indexed) of the spell whose cast
    /// triggered this ability. Resolves against the topmost matching
    /// `StackItem::Spell` (the just-cast spell whose `SpellCast` event
    /// produced this trigger). Empty when the trigger source isn't a
    /// spell or the slot is unfilled. Used by Strixhaven Repartee
    /// payoff effects whose body operates on the spell's target rather
    /// than choosing a fresh one — e.g. Conciliator's Duelist's "exile
    /// up to one *target* creature".
    CastSpellTarget(u8),

    /// All game objects matching `filter` in `zone`.
    EachMatching { zone: ZoneRef, filter: SelectionRequirement },
    /// All permanents on the battlefield matching `filter`.
    EachPermanent(SelectionRequirement),
    /// The permanent this one is attached to (for Auras/Equipment).
    AttachedTo(Box<Selector>),
    /// All permanents attached to `anchor`.
    AttachedToMe(Box<Selector>),

    /// Top `count` cards of `who`'s library.
    TopOfLibrary { who: PlayerRef, count: Value },
    /// Bottom `count` cards of `who`'s library.
    BottomOfLibrary { who: PlayerRef, count: Value },
    /// Every card in `who`'s zone matching `filter`.
    CardsInZone { who: PlayerRef, zone: Zone, filter: SelectionRequirement },

    /// Cards discarded earlier in this same resolution (across all players)
    /// matching `filter`. Backed by
    /// `GameState.discarded_card_ids_this_resolution`. Used by Mind Roots's
    /// "Put up to one land card discarded this way onto the battlefield
    /// tapped under your control" rider — at resolution time the discarded
    /// cards have already moved into their owner's graveyard, and this
    /// selector walks `discarded_card_ids_this_resolution` then filters in
    /// the gy zone.
    DiscardedThisResolution { filter: SelectionRequirement },

    /// A single player, lifted to selector form.
    Player(PlayerRef),

    /// Take at most `count` entities from `inner` (in resolution order).
    /// Wraps another selector to clamp how many entities flow through —
    /// used by SOS Heated Argument's "you may exile *a card* from your
    /// graveyard", Practiced Scrollsmith's "exile *target* noncreature/
    /// nonland card from your graveyard", and Pull from the Grave's
    /// "up to two creature cards from your graveyard". The cap is
    /// evaluated against the controller's resolution context, so values
    /// like `Value::CountersOn(...)` work as expected.
    Take { inner: Box<Selector>, count: Box<Value> },

    /// No entities (placeholder/default).
    None,
}

impl Selector {
    pub fn attached_to(inner: Selector) -> Self {
        Selector::AttachedTo(Box::new(inner))
    }

    /// Wrap `inner` so it returns at most `count` entities in resolution
    /// order. Sugar for `Selector::Take { inner, count }`.
    pub fn take(inner: Selector, count: Value) -> Self {
        Selector::Take {
            inner: Box::new(inner),
            count: Box::new(count),
        }
    }

    /// Wrap `inner` so it returns at most one entity. Sugar for
    /// `Selector::Take { inner, count: 1 }`.
    pub fn one_of(inner: Selector) -> Self {
        Selector::take(inner, Value::Const(1))
    }
}

// ── Value ────────────────────────────────────────────────────────────────────

/// A numeric expression evaluated at effect-time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Const(i32),
    /// Number of entities the selector resolves to.
    CountOf(Box<Selector>),
    PowerOf(Box<Selector>),
    ToughnessOf(Box<Selector>),
    LifeOf(PlayerRef),
    HandSizeOf(PlayerRef),
    GraveyardSizeOf(PlayerRef),
    /// Number of cards in `who`'s library. Used by Body of Research's
    /// "for each card in your library" Fractal-token scaling.
    LibrarySizeOf(PlayerRef),
    /// The X value paid in the spell's cost.
    XFromCost,
    /// Number of spells cast this turn by controller (Storm).
    StormCount,
    /// Counters of the given type on `what`.
    CountersOn { what: Box<Selector>, kind: CounterType },
    Sum(Vec<Value>),
    Diff(Box<Value>, Box<Value>),
    Times(Box<Value>, Box<Value>),
    Min(Box<Value>, Box<Value>),
    Max(Box<Value>, Box<Value>),
    /// Clamp the inner value to ≥0.
    NonNeg(Box<Value>),
    /// Power of the most recently sacrificed creature this resolution
    /// (set by `Effect::SacrificeAndRemember`). Used by Thud / Greater
    /// Gargadon-style sacrifice + damage spells.
    SacrificedPower,
    /// Toughness of the most recently sacrificed creature this
    /// resolution (set by `Effect::SacrificeAndRemember`). Used by
    /// Tribute to Hunger (gain life equal to sacrificed creature's
    /// toughness) and similar sacrifice + lifegain spells.
    SacrificedToughness,
    /// Number of cards discarded so far within the current effect
    /// resolution. Bumped by every `GameEvent::CardDiscarded` emission
    /// in `Effect::Discard` / `Effect::DiscardChosen`. Used by Borrowed
    /// Knowledge mode 1 ("draw cards equal to the number of cards
    /// discarded this way"), Colossus of the Blood Age's die trigger,
    /// and similar "draw what you discarded" payoffs. Reset to 0
    /// between independent resolutions, so a `Seq([Discard, Draw])`
    /// reads exactly the discards from this resolution.
    CardsDiscardedThisEffect,
    /// Number of *creature* cards discarded so far within the current
    /// effect resolution. Bumped alongside `CardsDiscardedThisEffect`
    /// whenever the discarded card has `CardType::Creature`. Used by
    /// Plargg, Dean of Chaos's "if a creature card was discarded this
    /// way, this creature deals 2 damage to any target" conditional
    /// rider — gates an `Effect::If { ValueAtLeast(this, 1), ... }`.
    /// Reset to 0 between independent resolutions.
    CreatureCardsDiscardedThisEffect,
    /// Mana value (CMC) of the first card the selector resolves to.
    /// Looks the card up across the battlefield, graveyards, exile, and
    /// hands. Used by Wrath of the Skies (destroy each nonland with mana
    /// value X) and similar "filter by mana value" effects.
    ManaValueOf(Box<Selector>),
    /// Converge value: the number of distinct colors of mana spent on the
    /// spell's cost. Stashed on `StackItem::Spell` at cast time and read
    /// from `EffectContext.converged_value` here. Used by Prismatic
    /// Ending and Pest Control.
    ConvergedValue,
    /// Total mana spent paying the originating spell's cost. Stashed on
    /// `StackItem::Spell.mana_spent` at cast time, propagated onto
    /// spell-cast `StackItem::Trigger.mana_spent`, and read from
    /// `EffectContext.mana_spent` here. Powers SOS's Increment /
    /// Opus payoffs: Cuboid Colony / Berta / Fractal Tender's
    /// "Whenever you cast a spell, if the amount of mana you spent is
    /// greater than this creature's power or toughness, put a +1/+1
    /// counter on this creature", and Opus's "this creature gets +N/+N
    /// for the rest of the turn (and an extra +N/+N if five or more
    /// mana was spent)".
    CastSpellManaSpent,
    /// Number of distinct card types in the top `count` cards of `who`'s
    /// library. Used by Atraxa, Grand Unifier's reveal-and-sort ETB —
    /// "reveal the top 10, take up to one of each card type" is
    /// approximated as "draw N where N = distinct types in those 10".
    DistinctTypesInTopOfLibrary { who: PlayerRef, count: Box<Value> },
    /// Number of cards `who` has drawn on the current turn. Powers
    /// Strixhaven's Quandrix scaling — Fractal Anomaly's "X +1/+1
    /// counters where X is the number of cards you've drawn this turn"
    /// and similar payoffs. Backed by `Player.cards_drawn_this_turn`,
    /// reset on the player's untap.
    CardsDrawnThisTurn(PlayerRef),
    /// Two raised to the inner value, clamped to a sane upper bound (≤30).
    /// Used by SOS Mathemagics — "target player draws 2ˣ cards" — so the
    /// X-cost bombshell scales correctly at the small/medium values
    /// typical of casting it. The clamp avoids deck-out / overflow when
    /// X is ≥31.
    Pow2(Box<Value>),
    /// Half of the inner value, rounded down. Used by SOS Pox Plague's
    /// "loses half their life", "discards half", "sacrifices half"
    /// clauses.
    HalfDown(Box<Value>),
    /// Number of permanents controlled by the resolved player. Useful for
    /// per-player effects like Pox Plague's "sacrifices half the
    /// permanents they control" clause inside a `ForEach` over each
    /// player, where `Selector::EachPermanent(ControlledByYou)` would
    /// always read the spell's controller instead of the iterated
    /// player.
    PermanentCountControlledBy(PlayerRef),
    /// Number of loyalty counters on the first permanent the selector
    /// resolves to. Used by Strixhaven's **Confront the Past** mode 2
    /// ("Confront the Past deals damage to target planeswalker equal to
    /// the number of loyalty counters on it") and any future
    /// "loyalty-counter-X" payoff. Returns 0 for non-permanents and
    /// non-planeswalkers (the field is just the `CounterType::Loyalty`
    /// count, which is 0 for cards without loyalty).
    LoyaltyOf(Box<Selector>),
    /// The amount carried by the event that fired the current trigger
    /// (life gained, life lost, damage dealt, cards drawn, …). Read
    /// from `EffectContext.event_amount`, which is set by the
    /// `dispatch_triggers_for_events` dispatcher from the event's
    /// `amount` field. Used by Light of Promise's "Whenever you gain
    /// life, put that many +1/+1 counters on target creature you
    /// control." — the trigger body reads `Value::TriggerEventAmount`
    /// for the count of counters to drop. Returns 0 in non-trigger
    /// resolution contexts (spells, activated abilities, delayed
    /// triggers that have moved past the original event).
    TriggerEventAmount,
}

impl Value {
    pub const ZERO: Value = Value::Const(0);
    pub const ONE: Value = Value::Const(1);
    pub fn count(sel: Selector) -> Self { Value::CountOf(Box::new(sel)) }
}

// ── Predicate ────────────────────────────────────────────────────────────────

/// A boolean game-state condition (for `Effect::If` / cast-time checks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Predicate {
    True,
    False,
    Not(Box<Predicate>),
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    /// At least one entity matches the selector.
    SelectorExists(Selector),
    /// Selector resolves to at least `n` entities.
    SelectorCountAtLeast { sel: Selector, n: Value },
    /// lhs ≥ rhs.
    ValueAtLeast(Value, Value),
    /// lhs ≤ rhs.
    ValueAtMost(Value, Value),
    /// lhs = rhs. Compresses the previous `All([≥, ≤])` idiom used by
    /// MV-equals filters (Postmortem Lunge "creature card with mana
    /// value X", Fix What's Broken "each card with mana value X").
    ValueEquals(Value, Value),
    /// It's `who`'s turn.
    IsTurnOf(PlayerRef),
    /// The given entity's properties match the filter.
    EntityMatches { what: Selector, filter: SelectionRequirement },
    /// `who` has gained at least `at_least` total life this turn.
    /// Backed by `Player.life_gained_this_turn`. Used by Strixhaven's
    /// **Infusion** rider — "If you gained life this turn, …".
    LifeGainedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has had at least `at_least` cards leave their graveyard
    /// this turn. Backed by `Player.cards_left_graveyard_this_turn`.
    /// Used by Lorehold "if a card left your graveyard this turn"
    /// payoffs — Living History's combat trigger, Primary Research's
    /// end-step draw rider, Wilt in the Heat's cost reduction rider.
    CardsLeftGraveyardThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` spells on the current turn.
    /// Backed by `Player.spells_cast_this_turn`. Used by Burrog Barrage
    /// ("if you've cast another instant or sorcery spell this turn, …")
    /// and similar pumps that key off spell-count.
    SpellsCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// At least `at_least` creatures controlled by `who` died this turn.
    /// Backed by `Player.creatures_died_this_turn` (bumped from the SBA
    /// dies handler and `remove_to_graveyard_with_triggers`). Used by
    /// Witherbloom "if a creature died under your control this turn, …"
    /// end-step payoffs (Essenceknit Scholar).
    CreaturesDiedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has caused at least `at_least` cards to be exiled this turn.
    /// Backed by `Player.cards_exiled_this_turn`. Used by Strixhaven
    /// "if one or more cards were put into exile this turn" payoffs
    /// (Ennis the Debate Moderator).
    CardsExiledThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` instant **or** sorcery spells on
    /// the current turn. Refines `SpellsCastThisTurnAtLeast` (which
    /// counts every spell type) for cards that explicitly gate on the
    /// "instant or sorcery" subset — Potioner's Trove's "Activate only
    /// if you've cast an instant or sorcery spell this turn", future
    /// Magecraft-adjacent payoffs. Backed by
    /// `Player.instants_or_sorceries_cast_this_turn`.
    InstantsOrSorceriesCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` creature spells on the current
    /// turn. Backed by `Player.creatures_cast_this_turn`. Reserved for
    /// future "if you've cast a creature spell this turn, …" payoffs.
    CreaturesCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// True if the spell pointed to by `Selector::TriggerSource` (typically
    /// the just-cast spell during a `SpellCast` trigger evaluation) has at
    /// least one chosen target matching `filter`. Used by Strixhaven's
    /// **Repartee** trigger — "Whenever you cast an instant or sorcery
    /// spell that targets a creature, …" — by chaining
    /// `cast_is_instant_or_sorcery()` AND `CastSpellTargetsMatch(Creature)`.
    /// Evaluated against the topmost matching `StackItem::Spell`'s `target`
    /// slot.
    CastSpellTargetsMatch(SelectionRequirement),
    /// True if the spell pointed to by `ctx.trigger_source` (the just-cast
    /// spell driving a `SpellCast` trigger) has at least one `{X}` symbol
    /// in its mana cost. Used by Quandrix's "whenever you cast a spell
    /// with `{X}` in its mana cost" payoffs (Geometer's Arthropod,
    /// Matterbending Mage, Paradox Surveyor's reveal filter). Evaluated
    /// against the topmost matching `StackItem::Spell`'s `card.definition.
    /// mana_cost` via `ManaCost::has_x()`.
    CastSpellHasX,
    /// True if the just-cast spell's total mana spent (the value stashed
    /// on `StackItem::Spell.mana_spent` at cast time, threaded onto the
    /// `StackItem::Trigger.mana_spent`) is at least `at_least`. Powers
    /// Opus's "if five or more mana was spent to cast that spell"
    /// branches (Deluge Virtuoso, Expressive Firedancer-style bigger-
    /// payoff modal) and Increment's "mana spent > P or T" gate (read
    /// from `ctx.mana_spent` at trigger-resolution time).
    CastSpellManaSpentAtLeast(u32),
    /// True if the just-cast spell's total mana spent is **strictly
    /// greater than** the source permanent's power or toughness. Used
    /// by SOS's Increment keyword payoff: "Whenever you cast a spell,
    /// if the amount of mana you spent is greater than this creature's
    /// power or toughness, put a +1/+1 counter on this creature."
    /// Evaluated against `ctx.source` (the listening permanent) at
    /// trigger-evaluation time.
    IncrementSatisfied,
    /// True if `who`'s `zone` contains at least `at_least` cards whose
    /// `definition.name` matches the resolving spell's name. Used by
    /// Dragon's Approach's "if you have four or more cards named Dragon's
    /// Approach in your graveyard, search your library for a Dragon
    /// creature card" rider. The name is read from
    /// `EffectContext.source_name` (the resolving spell's name); when no
    /// source is available the predicate is `False`.
    SameNamedInZoneAtLeast { who: PlayerRef, zone: Zone, at_least: Value },
    /// True when the resolving spell was cast from its caster's
    /// graveyard (typically via Flashback / Aftermath / Jump-Start /
    /// Yawgmoth's Will-style "cast from graveyard" effects). Backed by
    /// `EffectContext.cast_from_hand == false`, which is stamped by
    /// `for_spell_with_source` from the resolving card's
    /// `CardInstance.cast_from_hand` flag. Used by Increasing Vengeance
    /// ("If this spell was cast from a graveyard, copy that spell twice
    /// instead") and Antiquities on the Loose's "cast from anywhere
    /// other than your hand" rider.
    CastFromGraveyard,
    /// True if any opponent of `ctx.controller` controls more lands
    /// than `ctx.controller` does. Backed by walking the battlefield
    /// and counting `Land` permanents per seat. Used by catch-up ramp
    /// spells like Gift of Estates ("If an opponent controls more
    /// lands than you, …"), Tithe, Knight of the White Orchid's ETB
    /// trigger, and Land Tax.
    OpponentControlsMoreLandsThanYou,
}

// ── Duration ─────────────────────────────────────────────────────────────────

/// How long a temporary effect persists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Duration {
    /// Expires at the next end-of-turn Cleanup step.
    EndOfTurn,
    /// Expires when combat ends this turn.
    EndOfCombat,
    /// Until controller's next untap step.
    UntilYourNextUntap,
    /// Until the start of the next turn.
    UntilNextTurn,
    /// Indefinite (for effects like "gain control" without a clause).
    Permanent,
}

// ── Library positions, scry modes, mana ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPosition {
    Top,
    Bottom,
    Shuffled,
}

/// Where the non-matching revealed cards go after a
/// `RevealUntilFind` resolves. The default (`Graveyard`) matches the
/// historical behavior baked into older catalogs; SOS Strixhaven cards
/// like Geometer's Arthropod and Paradox Surveyor print "put the rest
/// on the bottom of your library in a random order" and use
/// `BottomRandom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RevealMissDest {
    /// Misses go to the controller's graveyard (legacy / Spoils-style).
    #[default]
    Graveyard,
    /// Misses go on the bottom of the controller's library, randomized.
    /// The engine inserts each miss in the order it was revealed; with
    /// no RNG hook available the order is effectively "as-revealed",
    /// which is a reasonable approximation since gameplay doesn't read
    /// the bottom of the library in any deterministic way before the
    /// next shuffle.
    BottomRandom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneDest {
    Hand(PlayerRef),
    Library { who: PlayerRef, pos: LibraryPosition },
    Graveyard,
    Exile,
    /// Battlefield under `controller`, optionally tapped.
    Battlefield { controller: PlayerRef, tapped: bool },
}

/// Where a countered spell goes after being lifted off the stack. The
/// default (graveyard) matches CR 701.5g; Memory Lapse routes to the
/// owner's library top, Spell Crumple routes to exile, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounteredSpellZone {
    /// Top of the spell-owner's library (Memory Lapse).
    OwnerLibraryTop,
    /// Bottom of the spell-owner's library.
    OwnerLibraryBottom,
    /// Owner's hand (Remand).
    OwnerHand,
    /// Exile (Spell Crumple).
    Exile,
}

/// What mana to add to a pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManaPayload {
    /// Add each listed color once.
    Colors(Vec<Color>),
    /// Add `amount` colorless mana.
    Colorless(Value),
    /// Add `amount` mana of one specified color (no player choice).
    /// Used by power-scaled mana abilities like Topiary Lecturer's
    /// "{T}: Add an amount of {G} equal to this creature's power" or
    /// Cryptolith Rite-style "add 1 of color X for each Y you control".
    OfColor(Color, Value),
    /// Add `amount` mana of any one color (player chooses).
    AnyOneColor(Value),
    /// Add `amount` mana of any colors (player chooses each).
    AnyColors(Value),
}

// ── Event specification (triggers) ───────────────────────────────────────────

/// Kinds of game events a trigger can watch for. Mirrors the `GameEvent`
/// stream in [`crate::game::types::GameEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// A permanent entered the battlefield.
    EntersBattlefield,
    /// A creature died (hit a graveyard from the battlefield).
    CreatureDied,
    /// Any permanent left the battlefield.
    PermanentLeavesBattlefield,
    /// A card was drawn.
    CardDrawn,
    /// A card was discarded.
    CardDiscarded,
    /// A land was played.
    LandPlayed,
    /// A spell was cast.
    SpellCast,
    /// A creature was declared as an attacker.
    Attacks,
    /// A creature was declared as a blocker. Fired once per blocker
    /// from `declare_blockers` (CR 509.1i). Dispatched in addition to
    /// the existing `BecomesBlocked` event on the attacker side.
    Blocks,
    /// A creature became blocked.
    BecomesBlocked,
    /// Combat damage was dealt to a player by a creature.
    DealsCombatDamageToPlayer,
    /// Combat damage was dealt to a creature by a creature.
    DealsCombatDamageToCreature,
    /// A player gained life.
    LifeGained,
    /// A player lost life.
    LifeLost,
    /// The game entered a particular step.
    StepBegins(crate::game::types::TurnStep),
    /// The active player's turn just began.
    TurnBegins,
    /// A counter was added to a permanent/player.
    CounterAdded(CounterType),
    /// An ability was activated.
    AbilityActivated,
    /// One or more cards left a player's graveyard (returned to hand /
    /// battlefield, exiled from graveyard, etc.). Used by Strixhaven
    /// "cards leave your graveyard" payoffs (Garrison Excavator, Living
    /// History, Spirit Mascot, Hardened Academic). The event fires once
    /// per card removed; the trigger handler is expected to be idempotent
    /// across batches (Strixhaven cards say "one or more cards" but the
    /// engine fires per-card and lets the trigger fire as many times).
    CardLeftGraveyard,
}

/// Whose events does this trigger listen for?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventScope {
    /// Only events whose subject is the source permanent itself
    /// ("When this ... enters", "When this attacks").
    SelfSource,
    /// Events caused/controlled by the source's controller ("whenever you
    /// cast a spell", "whenever you gain life").
    YourControl,
    /// Events caused/controlled by an opponent.
    OpponentControl,
    /// Any player.
    AnyPlayer,
    /// Another creature/permanent under your control (excludes `This`).
    AnotherOfYours,
    /// The active player (for step-based triggers).
    ActivePlayer,
    /// Trigger fires while the source card sits in **its owner's
    /// graveyard** (not the battlefield). Used by recursion creatures —
    /// Bloodghast's landfall, Ichorid's upkeep return, Silversmote
    /// Ghoul's lifegain return — where the ability fires off the
    /// graveyard copy and typically references it via `Selector::This`.
    /// The dispatcher walks graveyards in addition to the battlefield
    /// for triggers with this scope; the trigger's effective controller
    /// is the graveyard owner.
    FromYourGraveyard,
}

/// A structural filter over the unified `GameEvent` stream. The trigger fires
/// when an event of `kind` arrives, scoped per `scope`, and the optional
/// `filter` predicate holds in the post-event game state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSpec {
    pub kind: EventKind,
    pub scope: EventScope,
    /// Optional cast-time predicate (e.g. "whenever you cast a noncreature
    /// spell" is SpellCast + filter=NotCreatureSpell).
    pub filter: Option<Predicate>,
}

impl EventSpec {
    pub fn new(kind: EventKind, scope: EventScope) -> Self {
        Self { kind, scope, filter: None }
    }
    pub fn with_filter(mut self, p: Predicate) -> Self {
        self.filter = Some(p);
        self
    }
}

// ── Effect ───────────────────────────────────────────────────────────────────

/// The root instruction tree evaluated by the effect resolver.
///
/// All effects and abilities — spell effects, triggered-ability effects,
/// activated-ability effects — are `Effect` trees. Combinators let a single
/// card express modal choices, iteration, and conditionals without needing
/// engine changes per card.
//
// `large_enum_variant`: `CreateToken { definition: TokenDefinition, .. }`
// is the outlier (~368 bytes) — Boxing `TokenDefinition` is a structural
// change that touches every card factory and serde path. Tracked in
// TODO.md ("Box `TokenDefinition` in `Effect::CreateToken`") as a future
// cleanup; the stack footprint of `Effect` is fine in practice (most
// effects are deep behind `Box<Effect>` already via `Seq` / `ForEach`).
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    // ── Combinators ──────────────────────────────────────────────────────────
    /// Do nothing.
    Noop,
    /// Execute each inner effect in order.
    Seq(Vec<Effect>),
    /// If `cond` holds, execute `then`, else `else_`.
    If { cond: Predicate, then: Box<Effect>, else_: Box<Effect> },
    /// Execute `body` once per entity the `selector` resolves to.
    /// Inside `body`, `Selector::TriggerSource` refers to the current entity.
    ForEach { selector: Selector, body: Box<Effect> },
    /// Execute `body` `count` times.
    Repeat { count: Value, body: Box<Effect> },
    /// Modal — controller picks one of `modes` at cast time; the chosen index
    /// is stored in the stack item's `mode` field.
    ChooseMode(Vec<Effect>),
    /// "Choose `picks.len()` —" multi-mode pick. At resolution, runs each
    /// mode whose index appears in `picks` (in that order). Used by the
    /// Strixhaven Command cycle (Witherbloom / Lorehold / Quandrix /
    /// Silverquill / Prismari Commands), Charms, and any other "choose
    /// two of four" spell.
    ///
    /// CR 700.2d covers this: "If a player is allowed to choose more
    /// than one mode for a modal spell or ability, that player normally
    /// can't choose the same mode more than once." The `picks` field
    /// stores the controller's chosen indices; the auto-decider feeds
    /// them in deterministically (a sensible default for each card),
    /// and a later mode-pick UI can override the picks per-cast.
    ///
    /// Modes share the spell's single target slot (`ctx.targets[0]`).
    /// The picked modes are run in `picks` order; if multiple modes
    /// each need a target, only the *first* picked mode's target
    /// filter is enforced (engine has no multi-target slots yet).
    /// Mode-pick UI plumbing is tracked in TODO.md as future work.
    ChooseN { picks: Vec<u8>, modes: Vec<Effect> },
    /// "You may [body]" — emit a yes/no decision via
    /// `Decision::OptionalTrigger`. Run `body` only on `Bool(true)`. The
    /// `description` string is shown to the player (and serialized into
    /// the decision wire payload). The decision is asked of the
    /// effect's *controller* (`ctx.controller`).
    ///
    /// Used by SOS / STX cards that bake a "you may" into the middle of
    /// a sequence: Stadium Tidalmage's loot trigger, Pursue the Past's
    /// optional discard, Witherbloom Charm mode 0's optional sacrifice,
    /// Tenured Concocter's may-draw on becomes-target, and any future
    /// "you may pay X to do Y" rider where the cost itself is `Effect::
    /// Noop` (free) or already paid up-front. For paid optional costs
    /// (Bayou Groff's "may pay {1} to return on death") see the related
    /// `Effect::MayPay` primitive in TODO.md — `MayDo` is the no-cost
    /// variant.
    ///
    /// The `AutoDecider` answers `false` (skip) by default; tests can
    /// override via `ScriptedDecider::new([DecisionAnswer::Bool(true)])`.
    /// This matches MTG rules: any "you may" defaults to "no" unless the
    /// controller actively chooses to do it.
    ///
    /// `description` is a `String` (rather than `&'static str`) because
    /// `Effect` derives `Deserialize` and serde requires owned data when
    /// the parent enum is bound to a non-static lifetime via the rest of
    /// `GameState`. Card factories pass `"…".into()` which is a no-cost
    /// `&str → String` move at construction time.
    MayDo { description: String, body: Box<Effect> },

    /// Optional **paid** branch: the controller is asked yes/no, and if
    /// they accept *and* can afford `mana_cost`, the engine deducts the
    /// mana from their pool and runs `body`. If the controller declines
    /// or can't afford the cost, the body is skipped.
    ///
    /// Sibling to `Effect::MayDo` (the no-cost variant). Powers cards
    /// like Bayou Groff ("when this dies, you may pay {1}; if you do,
    /// return it to its owner's hand"), Killian's Confidence's "may pay
    /// {W/B} to return from gy", and any other "may pay X to do Y"
    /// rider where the cost is pure mana.
    ///
    /// Cost evaluation walks the controller's *pool* (already-floated
    /// mana) — the engine doesn't tap lands automatically inside an
    /// `Effect::MayPay`, matching MTG's "you can't activate mana
    /// abilities mid-resolution unless the rules let you." Tests that
    /// want to exercise the paid path should pre-float the mana
    /// (`game.players[c].mana_pool.add_colored(...)`) and feed
    /// `DecisionAnswer::Bool(true)` to the scripted decider.
    ///
    /// X-cost variants (where the optional cost has its own X prompt)
    /// are out of scope here — those should land as a sibling
    /// `MayPayX { mana_cost, x_value, body }` if/when needed.
    MayPay {
        description: String,
        mana_cost: crate::mana::ManaCost,
        body: Box<Effect>,
    },

    /// Reveal-from-hand gate: "you may reveal a [filter] card from your
    /// hand. If you do, run `then`; otherwise run `else_`." Used by the
    /// STX Snarl dual-land cycle (Frostboil, Furycalm, Necroblossom,
    /// Shineshadow, Vineglimmer) — the printed Oracle reads "As ~~~
    /// enters, you may reveal a [C1] or [C2] card from your hand. If you
    /// don't, ~~~ enters tapped."
    ///
    /// Asked of the effect's *controller* (`ctx.controller`). Filter is
    /// evaluated against each hand card via `evaluate_requirement_on_card`.
    /// AutoDecider auto-reveals whenever a matching card exists — the
    /// bot always wants to keep the land untapped if it can. A future
    /// UI wire could surface a `Decision::Reveal` shape so a human
    /// player can decline to reveal (a strategic bluff); not modeled
    /// here since no test exercises the decline-with-match path.
    ///
    /// If no card matches the filter, `else_` runs unconditionally
    /// (matches printed "if you don't reveal, …" — including the case
    /// where you can't).
    IfRevealFromHand {
        filter: SelectionRequirement,
        then: Box<Effect>,
        else_: Box<Effect>,
    },

    // ── Damage / life ────────────────────────────────────────────────────────
    DealDamage { to: Selector, amount: Value },
    /// Two creatures fight: each deals damage equal to its current
    /// power to the other simultaneously. Both creatures take damage
    /// and die simultaneously to SBA. `attacker` is typically
    /// `Selector::Target(0)` or `Selector::This` (a friendly fighter);
    /// `defender` is typically `Selector::Target(1)` or an
    /// auto-selected opp creature. If either selector resolves to no
    /// permanent the effect no-ops cleanly (matches MTG's "if either
    /// is no longer a creature, no damage is dealt"). Used by SOS
    /// Chelonian Tackle, STX Decisive Denial mode 1, and similar
    /// fight-style green/quandrix removal.
    Fight { attacker: Selector, defender: Selector },
    GainLife  { who: Selector, amount: Value },
    LoseLife  { who: Selector, amount: Value },
    /// Controller loses `amount` life, a different selector gains it.
    Drain { from: Selector, to: Selector, amount: Value },

    // ── Cards / draw / discard / mill ────────────────────────────────────────
    Draw    { who: Selector, amount: Value },
    /// Discard `amount` cards. If `random`, chosen randomly; else by `who`.
    Discard { who: Selector, amount: Value, random: bool },
    /// Discard any number of cards (0 to hand-size, player's choice). Used by
    /// "discard any number of cards, then draw that many cards plus one"
    /// effects (Colossus of the Blood Age, Mind Roots-style "any number"
    /// discards). The discarded count is added to
    /// `state.cards_discarded_this_resolution`, so a follow-up `Draw` step
    /// in the same `Seq` can reference `Value::CardsDiscardedThisEffect`
    /// for the "draw equal to discarded" rider. AutoDecider picks 0 (the
    /// conservative default); ScriptedDecider supplies the exact discard
    /// list via `DecisionAnswer::Discard(_)`.
    DiscardAnyNumber { who: Selector },
    /// Set `Player.no_maximum_hand_size = true` on each resolved player,
    /// for the rest of the game. Used by Wisdom of Ages ("You have no
    /// maximum hand size for the rest of the game"), Reliquary Tower's
    /// static (which actually wires through a layer, but the simpler
    /// "for the rest of the game" cards can flip the flag directly).
    /// Skips the cleanup-step CR 514.1 discard-down-to-7 in
    /// `do_cleanup`.
    SetNoMaxHandSize { who: Selector },
    Mill    { who: Selector, amount: Value },
    Scry    { who: PlayerRef, amount: Value },
    Surveil { who: PlayerRef, amount: Value },
    LookAtTop { who: PlayerRef, amount: Value },

    // ── Zone moves ───────────────────────────────────────────────────────────
    /// Move every entity the selector resolves to into `to`.
    Move { what: Selector, to: ZoneDest },
    /// Search `who`'s library for a card matching `filter` and move to `to`.
    Search { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// Shuffle `who`'s graveyard into their library.
    ShuffleGraveyardIntoLibrary { who: PlayerRef },

    // ── Mana ─────────────────────────────────────────────────────────────────
    AddMana { who: PlayerRef, pool: ManaPayload },

    // ── Permanent mutations ──────────────────────────────────────────────────
    Destroy { what: Selector },
    Exile   { what: Selector },
    Tap     { what: Selector },
    /// Untap every permanent the selector resolves to. The optional
    /// `up_to` cap limits the count to "up to N" — used by Frantic
    /// Search ("untap up to three lands"), Cryptolith Rite-style
    /// abilities, etc. `None` means "untap all matching" (the
    /// pre-cap default behavior). When the selector resolves to more
    /// than `up_to` matches, the picker takes the first `up_to`
    /// in resolution order; auto-resolution favors highest-CMC lands
    /// for max mana refund.
    Untap   { what: Selector, #[serde(default)] up_to: Option<Value> },
    /// Give a temporary +P/+T bonus.
    PumpPT  { what: Selector, power: Value, toughness: Value, duration: Duration },
    /// Override the resolved permanent's base power and toughness via a
    /// layer-7b continuous effect. Unlike `PumpPT` (which adds to the
    /// existing P/T via direct bonus fields), `SetBasePT` installs a
    /// proper `Modification::SetPowerToughness(p, t)` continuous effect
    /// that participates in the layer system. Used by Strixhaven's
    /// **Square Up** ({U}{R}: "Until end of turn, target creature has
    /// base power and toughness 0/4") and any future "base P/T
    /// becomes" effect. Counters and +P/+T modifications still stack
    /// on top per CR 613.7f / 613.7c — so a +1/+1 counter on a Square-
    /// Upped creature makes it 1/5, not 1/1.
    SetBasePT { what: Selector, power: Value, toughness: Value, duration: Duration },
    GrantKeyword { what: Selector, keyword: Keyword, duration: Duration },
    AddCounter    { what: Selector, kind: CounterType, amount: Value },
    RemoveCounter { what: Selector, kind: CounterType, amount: Value },
    Proliferate,
    GainControl { what: Selector, duration: Duration },
    /// Create `count` copies of the given token under `who`'s control.
    CreateToken { who: PlayerRef, count: Value, definition: TokenDefinition },
    /// Target becomes a basic land of `land_type` (losing other types/abilities).
    BecomeBasicLand { what: Selector, land_type: LandType, duration: Duration },
    /// Target creature becomes a vanilla 1/1, loses all abilities.
    ResetCreature  { what: Selector, duration: Duration },
    /// Attach `what` (Aura/Equipment) to `to`.
    Attach { what: Selector, to: Selector },

    // ── Stack interaction ────────────────────────────────────────────────────
    /// Counter target spell (removes from stack; sends to owner's graveyard).
    CounterSpell { what: Selector },
    /// Counter target spell and route it to a specific zone instead of the
    /// owner's graveyard. CR 701.6a's default is "a countered spell is put
    /// into its owner's graveyard"; cards like Memory Lapse / Remand /
    /// Spell Crumple print an "instead" clause that overrides this. The
    /// on-stack card is removed from the stack and placed into `zone`
    /// (top of library for Memory Lapse; exile for Spell Crumple; owner's
    /// hand for Remand).
    CounterSpellToZone {
        what: Selector,
        zone: CounteredSpellZone,
    },
    /// Counter target activated/triggered ability. The selector resolves
    /// to a permanent (the ability's source), and the engine removes the
    /// topmost `StackItem::Trigger` whose `source` matches. Used by
    /// Consign to Memory.
    CounterAbility { what: Selector },
    /// Counter target spell **unless** its controller pays `mana_cost`.
    /// At resolution, the engine attempts to auto-pay on behalf of the
    /// targeted spell's controller — if affordable, the spell stays;
    /// otherwise it's countered. Used by Mystical Dispute (counter unless
    /// controller pays {3}). Spells flagged `uncounterable` are skipped.
    CounterUnlessPaid {
        what: Selector,
        mana_cost: crate::mana::ManaCost,
    },
    /// CR 702.21 — Ward's "counter that spell or ability unless its
    /// controller pays [cost]" trigger body. Walks the stack for the
    /// topmost `Spell` with `card.id == target` or `Trigger` with
    /// `source == target`, then tries to auto-pay the `cost` on behalf
    /// of that item's controller. If unpaid, the item is removed
    /// (spells go to graveyard; abilities just vanish off the stack).
    ///
    /// Distinct from `CounterUnlessPaid` because (a) it also counters
    /// activated/triggered abilities (for the "or ability" half of CR
    /// 702.21a), and (b) the cost menu is the broader
    /// `WardCost` (mana / life / discard / sacrifice creature).
    CounterUnless {
        what: Selector,
        cost: crate::card::WardCost,
    },
    /// Copy target spell/ability `count` times.
    CopySpell    { what: Selector, count: Value },
    /// Copy target spell **unless** its caster pays `mana_cost`. Used by
    /// Wandering Archaic ("Whenever an opponent casts an instant or sorcery
    /// spell, that player may pay {2}. If they don't, you may copy the
    /// spell."). The resolver: (a) walks the stack for the spell whose
    /// `card.id` matches `what`; (b) asks the spell's caster yes/no via
    /// `Decision::OptionalTrigger`; (c) if they accept *and* can afford
    /// `mana_cost` from their pool, deducts it and skips the copy; (d) if
    /// they decline or can't afford, copies the spell `count` times above
    /// it on the stack.
    ///
    /// AutoDecider's default answer is `false` (decline to pay) — the
    /// printed Oracle implies most casters won't have an extra {2}
    /// floating, so the conservative default is "let the copy happen."
    /// ScriptedDecider can override via `DecisionAnswer::Bool(true)` for
    /// tests that want to exercise the pay path.
    CopySpellUnlessPaid {
        what: Selector,
        mana_cost: crate::mana::ManaCost,
        count: Value,
    },

    // ── Sacrifice ────────────────────────────────────────────────────────────
    Sacrifice { who: Selector, count: Value, filter: SelectionRequirement },
    /// "Sacrifice a [filter] with the greatest mana value" picker.
    /// Mirrors `Sacrifice` but the candidate sort prefers maximum CMC.
    /// Used by Soul Shatter ("Each opponent sacrifices a creature or
    /// planeswalker with the greatest mana value among permanents
    /// that player controls"). Auto-decider picks the highest-CMC
    /// matching permanent per player.
    SacrificeGreatestMV { who: Selector, count: Value, filter: SelectionRequirement },

    // ── Counters on players ──────────────────────────────────────────────────
    AddPoison { who: Selector, amount: Value },

    // ── Misc atomic operations needed by existing cards ──────────────────────
    /// Reveal the top card of `who`'s library; if `reveal_filter` matches, draw it.
    RevealTopAndDrawIf { who: PlayerRef, reveal_filter: SelectionRequirement },

    /// Reveal the top card of `who`'s library (fires `TopCardRevealed` event for
    /// the animation) without moving it. Used by Chaos Warp's "reveal top card"
    /// step where the put-onto-battlefield clause is handled separately.
    RevealTopCard { who: PlayerRef },

    /// Controller chooses `count` cards from their hand and puts them on top of
    /// their library in a chosen order (first chosen = topmost).
    PutOnLibraryFromHand { who: PlayerRef, count: Value },

    /// Sacrifice one creature `who` controls matching `filter` and store its
    /// power in the resolution context for later `Value::SacrificedPower`
    /// references. Used by Thud (sacrifice creature, deal damage equal to
    /// its power) and similar spells.
    SacrificeAndRemember { who: PlayerRef, filter: SelectionRequirement },

    /// "Target opponent reveals their hand. You choose a card from it
    /// matching `filter`. They discard it." Inquisition of Kozilek,
    /// Thoughtseize, etc. Currently the **caster** auto-picks the first
    /// matching card via `AutoDecider`; an interactive picker UI is a
    /// future improvement.
    DiscardChosen {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
    },

    // ── Delayed triggers and pact costs ──────────────────────────────────────
    /// Register a delayed triggered ability that fires later. `kind` selects
    /// the future event (your next upkeep, next end step, …); `body` is the
    /// effect that resolves when the trigger fires. Captures the current
    /// `ctx.targets[0]` so the body can reference it via `Selector::Target(0)`.
    DelayUntil {
        kind: DelayedTriggerKind,
        body: Box<Effect>,
    },

    /// "Pay {cost} or you lose the game." Used for pact upkeep payments
    /// (Pact of Negation, Summoner's Pact). Auto-pays when the controller
    /// can afford; eliminates the controller otherwise. (No interactive
    /// "do I want to pay?" prompt yet — pact costs are virtually always
    /// paid, and skipping the prompt avoids another suspend path.)
    PayOrLoseGame {
        mana_cost: crate::mana::ManaCost,
        life_cost: u32,
    },

    /// Add `count` "first-spell tax" charges against each player resolved
    /// by the selector. Each charge taxes that player's next spell {1}
    /// more (consumed at cast time via `consume_first_spell_tax`). Used by
    /// Chancellor of the Annex's opening-hand reveal — `who: EachOpponent`.
    AddFirstSpellTax {
        who: PlayerRef,
        count: Value,
    },

    /// Set `Player.sorceries_as_flash` on each resolved player so they may
    /// cast sorcery spells at instant speed until their next turn.
    /// Cleared in `do_untap`. Used by Teferi, Time Raveler's +1.
    GrantSorceriesAsFlash { who: PlayerRef },

    /// "Reveal cards from the top of `who`'s library until you reveal a
    /// card matching `find`, or `cap` cards have been revealed. Put the
    /// found card (if any) into `to`; mill the rest, lose 1 life per
    /// card revealed." Used by Spoils of the Vault.
    ///
    /// The auto-decider picks the **first** matching card (so the search
    /// resolves deterministically in tests). Real Oracle has the player
    /// name a card up-front; we bypass that, instead matching anything
    /// passing `find`. The "lose 1 per revealed" rider is wired to
    /// `life_per_revealed` so callers can disable it (Spoils → 1; future
    /// "search until type, no life cost" cards → 0).
    ///
    /// `miss_dest` controls where the non-matching revealed cards end
    /// up. Defaults to `RevealMissDest::Graveyard` for snapshot
    /// back-compat — the previous behavior. Several Strixhaven cards
    /// (Geometer's Arthropod, Paradox Surveyor, Follow the Lumarets)
    /// printed-want misses placed on the bottom of the library in
    /// random order; pass `RevealMissDest::BottomRandom` to honor that.
    RevealUntilFind {
        who: PlayerRef,
        find: SelectionRequirement,
        to: ZoneDest,
        cap: Value,
        life_per_revealed: u32,
        #[serde(default)]
        miss_dest: RevealMissDest,
    },

    /// "As [this] enters, choose a creature type." Used by Cavern of Souls.
    /// Asks the controller via the `ChooseCreatureType` decision and stores
    /// the chosen type on the source permanent's `chosen_creature_type`
    /// field. Subsequent cast paths consult that field via
    /// `caster_grants_uncounterable` to gate which creature spells the
    /// Cavern protects (only those that share the named type).
    NameCreatureType { what: Selector },

    /// "[Player] wins the game." Used by Approach of the Second Sun's
    /// second-cast win condition, Coalition Victory, Test of Endurance,
    /// Felidar Sovereign, and similar alt-win effects. The engine
    /// eliminates every other player so the standard
    /// `check_state_based_actions` win-detection path (≤ 1 alive player
    /// → `game_over = Some(winner)`) promotes the named player to the
    /// winner on the next SBA pass. No CR violation: the state-based
    /// action approach matches CR 104.2a's "you win the game" wording.
    WinGame { who: PlayerRef },

    /// "Prevent all combat damage that would be dealt this turn." Sets
    /// `GameState.prevent_combat_damage_this_turn = true`; combat
    /// damage resolution (`resolve_combat_damage_with_filter`) reads
    /// the flag and zeroes every assigned damage value (CR 615.1
    /// replacement-effect emulation — see the note on the field). The
    /// flag clears in `do_cleanup` alongside other until-end-of-turn
    /// state. Used by Owlin Shieldmage's ETB and the Holy Day / fog
    /// family of effects.
    PreventAllCombatDamageThisTurn,
}

/// Lightweight mirror of `crate::game::types::DelayedKind` for use inside
/// `Effect`. Kept separate so `effect.rs` doesn't need to import from
/// `game::`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedTriggerKind {
    YourNextUpkeep,
    NextEndStep,
    /// "At the beginning of your next pre-combat main phase, …" Used by
    /// Chancellor of the Tangle's opening-hand reveal — the mana ritual
    /// fires on main rather than upkeep so the {G} doesn't empty out of
    /// the pool before the player can spend it (mana pools clear on
    /// step transition, MTG rule 500.4).
    YourNextMainPhase,
}

/// Opening-hand ("if this is in your opening hand, you may ...") effect.
/// Resolved by `GameState::apply_opening_hand_effects` after all players
/// finish mulligans and before the first turn begins.
///
/// Each variant covers one of the canonical Magic shapes:
/// * **Leyline / Gemstone Caverns** — the card begins the game on the
///   battlefield instead of in hand.
/// * **Chancellor of the Tangle / of the Annex** — the card stays in hand,
///   but reveals at game start to register a one-shot trigger that fires
///   later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpeningHandEffect {
    /// "If [card] is in your opening hand, you may begin the game with it
    /// on the battlefield." After moving to play, run `extra` so cards like
    /// Gemstone Caverns can stamp themselves with a luck counter (or any
    /// other one-shot ETB-style follow-up).
    StartInPlay {
        tapped: bool,
        extra: Effect,
    },
    /// "You may reveal [card] from your opening hand. If you do, [body]."
    /// The card stays in hand; we register a `DelayedTrigger` of `kind`
    /// whose effect is `body`. Used by the Chancellors.
    RevealForDelayedTrigger {
        kind: DelayedTriggerKind,
        body: Effect,
    },
    /// "Any time you could mulligan and [card] is in your hand, you may
    /// exile all the cards from your hand, then draw that many cards."
    /// Surfaces as an additional answer in the mulligan decision; not run
    /// post-mulligan. The variant exists so the catalog can declaratively
    /// flag the card and `apply_opening_hand_effects` skips it.
    MulliganHelper,
}

impl Default for Effect {
    /// Default `Effect` is `Noop` — a permanent with no spell effect
    /// (creature, enchantment, etc.) leaves this slot at its default.
    /// Lets `CardDefinition` derive `Default`, so card constructors can
    /// use `..Default::default()` and skip boilerplate.
    fn default() -> Self { Effect::Noop }
}

impl Effect {
    pub const NOOP: Effect = Effect::Noop;

    pub fn seq(effects: Vec<Effect>) -> Self {
        if effects.is_empty() { Effect::Noop }
        else if effects.len() == 1 { effects.into_iter().next().unwrap() }
        else { Effect::Seq(effects) }
    }

    /// True if this effect (transitively) requires a chosen target (i.e.
    /// references `Selector::Target(_)` anywhere). Used for cast-time
    /// validation.
    pub fn requires_target(&self) -> bool {
        fn sel_has_target(s: &Selector) -> bool {
            match s {
                Selector::Target(_) | Selector::TargetFiltered { .. } => true,
                Selector::AttachedTo(i) | Selector::AttachedToMe(i) => sel_has_target(i),
                Selector::Take { inner, count } => {
                    sel_has_target(inner) || value_has_target(count)
                }
                Selector::TopOfLibrary { who, .. }
                | Selector::BottomOfLibrary { who, .. }
                | Selector::CardsInZone { who, .. }
                | Selector::Player(who) => player_has_target(who),
                _ => false,
            }
        }
        fn player_has_target(p: &PlayerRef) -> bool {
            match p {
                PlayerRef::Target(_) => true,
                PlayerRef::OwnerOf(s) | PlayerRef::ControllerOf(s) => sel_has_target(s),
                _ => false,
            }
        }
        fn value_has_target(v: &Value) -> bool {
            match v {
                Value::CountOf(s) | Value::PowerOf(s) | Value::ToughnessOf(s) => sel_has_target(s),
                Value::CountersOn { what, .. } => sel_has_target(what),
                Value::LifeOf(p) | Value::HandSizeOf(p) | Value::GraveyardSizeOf(p)
                | Value::LibrarySizeOf(p) => {
                    player_has_target(p)
                }
                Value::Sum(vs) => vs.iter().any(value_has_target),
                Value::Diff(a, b) | Value::Times(a, b) | Value::Min(a, b) | Value::Max(a, b) => {
                    value_has_target(a) || value_has_target(b)
                }
                Value::NonNeg(v) => value_has_target(v),
                Value::ManaValueOf(s) => sel_has_target(s),
                Value::LoyaltyOf(s) => sel_has_target(s),
                _ => false,
            }
        }
        fn pred_has_target(p: &Predicate) -> bool {
            match p {
                Predicate::Not(q) => pred_has_target(q),
                Predicate::All(v) | Predicate::Any(v) => v.iter().any(pred_has_target),
                Predicate::SelectorExists(s) => sel_has_target(s),
                Predicate::SelectorCountAtLeast { sel, n } => sel_has_target(sel) || value_has_target(n),
                Predicate::ValueAtLeast(a, b)
                | Predicate::ValueAtMost(a, b)
                | Predicate::ValueEquals(a, b) => value_has_target(a) || value_has_target(b),
                Predicate::IsTurnOf(p) => player_has_target(p),
                Predicate::EntityMatches { what, .. } => sel_has_target(what),
                _ => false,
            }
        }
        match self {
            Effect::Noop => false,
            Effect::Seq(v) => v.iter().any(|e| e.requires_target()),
            Effect::If { cond, then, else_ } => {
                pred_has_target(cond) || then.requires_target() || else_.requires_target()
            }
            Effect::ForEach { selector, body } => {
                sel_has_target(selector) || body.requires_target()
            }
            Effect::Repeat { count, body } => value_has_target(count) || body.requires_target(),
            Effect::ChooseMode(modes) => modes.iter().any(|e| e.requires_target()),
            Effect::ChooseN { modes, .. } => modes.iter().any(|e| e.requires_target()),
            Effect::MayDo { body, .. } => body.requires_target(),
            Effect::MayPay { body, .. } => body.requires_target(),
            Effect::IfRevealFromHand { then, else_, .. } => {
                then.requires_target() || else_.requires_target()
            }
            Effect::DealDamage { to, amount } => sel_has_target(to) || value_has_target(amount),
            Effect::Fight { attacker, defender } => {
                sel_has_target(attacker) || sel_has_target(defender)
            }
            Effect::GainLife { who, amount } | Effect::LoseLife { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::Drain { from, to, amount } => {
                sel_has_target(from) || sel_has_target(to) || value_has_target(amount)
            }
            Effect::Draw { who, amount }
            | Effect::Mill { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::Discard { who, amount, .. } => sel_has_target(who) || value_has_target(amount),
            Effect::DiscardAnyNumber { who } => sel_has_target(who),
            Effect::SetNoMaxHandSize { who } => sel_has_target(who),
            Effect::Scry { who, amount }
            | Effect::Surveil { who, amount }
            | Effect::LookAtTop { who, amount } => {
                player_has_target(who) || value_has_target(amount)
            }
            Effect::Move { what, to } => sel_has_target(what) || zonedest_has_target(to),
            Effect::Search { who, to, .. } => player_has_target(who) || zonedest_has_target(to),
            Effect::ShuffleGraveyardIntoLibrary { who } => player_has_target(who),
            Effect::AddMana { who, pool } => {
                player_has_target(who) || match pool {
                    ManaPayload::Colorless(v)
                    | ManaPayload::AnyOneColor(v)
                    | ManaPayload::AnyColors(v) => value_has_target(v),
                    ManaPayload::OfColor(_, v) => value_has_target(v),
                    ManaPayload::Colors(_) => false,
                }
            }
            Effect::Destroy { what }
            | Effect::Exile { what }
            | Effect::Tap { what }
            | Effect::Untap { what, .. }
            | Effect::CounterSpell { what }
            | Effect::CounterSpellToZone { what, .. }
            | Effect::CounterAbility { what }
            | Effect::CounterUnlessPaid { what, .. }
            | Effect::CounterUnless { what, .. } => sel_has_target(what),
            Effect::PumpPT { what, power, toughness, .. } => {
                sel_has_target(what) || value_has_target(power) || value_has_target(toughness)
            }
            Effect::SetBasePT { what, power, toughness, .. } => {
                sel_has_target(what) || value_has_target(power) || value_has_target(toughness)
            }
            Effect::GrantKeyword { what, .. } => sel_has_target(what),
            Effect::AddCounter { what, amount, .. }
            | Effect::RemoveCounter { what, amount, .. } => {
                sel_has_target(what) || value_has_target(amount)
            }
            Effect::Proliferate => false,
            Effect::GainControl { what, .. } => sel_has_target(what),
            Effect::CreateToken { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::BecomeBasicLand { what, .. }
            | Effect::ResetCreature { what, .. } => sel_has_target(what),
            Effect::Attach { what, to } => sel_has_target(what) || sel_has_target(to),
            Effect::CopySpell { what, count } => sel_has_target(what) || value_has_target(count),
            Effect::CopySpellUnlessPaid { what, count, .. } => {
                sel_has_target(what) || value_has_target(count)
            }
            Effect::Sacrifice { who, count, .. } => sel_has_target(who) || value_has_target(count),
            Effect::SacrificeGreatestMV { who, count, .. } => {
                sel_has_target(who) || value_has_target(count)
            }
            Effect::AddPoison { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::RevealTopAndDrawIf { who, .. } | Effect::RevealTopCard { who } => {
                player_has_target(who)
            }
            Effect::PutOnLibraryFromHand { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::DelayUntil { body, .. } => body.requires_target(),
            Effect::PayOrLoseGame { .. } => false,
            Effect::SacrificeAndRemember { .. } => false,
            Effect::AddFirstSpellTax { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::GrantSorceriesAsFlash { who } => player_has_target(who),
            Effect::RevealUntilFind { who, to, cap, .. } => {
                player_has_target(who)
                    || zonedest_has_target(to)
                    || value_has_target(cap)
            }
            Effect::DiscardChosen { from, count, .. } => {
                sel_has_target(from) || value_has_target(count)
            }
            Effect::NameCreatureType { what } => sel_has_target(what),
            Effect::WinGame { who } => player_has_target(who),
            Effect::PreventAllCombatDamageThisTurn => false,
        }
    }

    /// Extract the target's filter if this effect's top-level "what"/"to" is
    /// `Selector::Target(0)`. Used by UI/bot for target selection.
    pub fn primary_target_filter(&self) -> Option<&SelectionRequirement> {
        fn sel_filter(s: &Selector) -> Option<&SelectionRequirement> {
            match s {
                Selector::EachMatching { filter, .. } => Some(filter),
                Selector::EachPermanent(f) => Some(f),
                Selector::CardsInZone { filter, .. } => Some(filter),
                Selector::TargetFiltered { filter, .. } => Some(filter),
                Selector::Take { inner, .. } => sel_filter(inner),
                _ => None,
            }
        }
        match self {
            Effect::DealDamage { to, .. } => sel_filter(to),
            // Fight surfaces the *defender's* filter (the opp creature
            // we want to fight). The attacker is usually the friendly
            // already-on-bf source/target.
            Effect::Fight { defender, .. } => sel_filter(defender),
            Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_filter(who),
            Effect::Destroy { what }
            | Effect::Exile { what }
            | Effect::Tap { what }
            | Effect::Untap { what, .. }
            | Effect::CounterSpell { what }
            | Effect::CounterSpellToZone { what, .. }
            | Effect::CounterAbility { what }
            | Effect::CounterUnlessPaid { what, .. }
            | Effect::CounterUnless { what, .. }
            | Effect::GainControl { what, .. } => sel_filter(what),
            Effect::AddCounter { what, .. } | Effect::RemoveCounter { what, .. } => sel_filter(what),
            Effect::PumpPT { what, .. } => sel_filter(what),
            Effect::SetBasePT { what, .. } => sel_filter(what),
            Effect::GrantKeyword { what, .. } => sel_filter(what),
            Effect::Move { what, .. } => sel_filter(what),
            // Player-targeting effects: surface the filter so the bot's
            // auto-target heuristic can find the opp / caster without a
            // manual Target. The filter is typically `Player` (Mind Rot,
            // Sign in Blood) but can be narrower (Howling Mine-style "you").
            Effect::Discard { who, .. }
            | Effect::DiscardAnyNumber { who }
            | Effect::SetNoMaxHandSize { who }
            | Effect::Draw { who, .. }
            | Effect::Mill { who, .. } => sel_filter(who),
            Effect::Drain { to, .. } => sel_filter(to),
            Effect::AddPoison { who, .. } => sel_filter(who),
            // Edict-class effects: "target player sacrifices a permanent."
            // The `who` selector usually carries a `target_filtered(Player)`
            // filter (Sudden Edict, Cruel Edict-style spells); bare
            // `Selector::Target(0)` falls through unchanged so existing
            // edicts that pre-date the filter primitive (Diabolic Edict,
            // Geth's Verdict) keep their explicit-target casting contract.
            Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => {
                sel_filter(who)
            }
            // Compound effects: walk into the children. Spells like Goryo's
            // Vengeance wrap a `Move` (target legendary creature) in a
            // `Seq` alongside a delayed exile trigger; the primary target
            // is still the Move's target.
            Effect::Seq(v) => v.iter().find_map(|e| e.primary_target_filter()),
            Effect::If { then, else_, .. } => then
                .primary_target_filter()
                .or_else(|| else_.primary_target_filter()),
            Effect::DelayUntil { body, .. } => body.primary_target_filter(),
            // Modal cards: surface the first mode's filter as the
            // representative one (UI/bot still need *some* filter to
            // narrow target candidates). Mode-specific validation lives
            // in `target_filter_for_slot_in_mode`, which the cast paths
            // consult once the user/bot has picked a mode.
            Effect::ChooseMode(modes) => modes
                .iter()
                .find_map(|e| e.primary_target_filter()),
            Effect::ChooseN { modes, .. } => modes
                .iter()
                .find_map(|e| e.primary_target_filter()),
            // MayDo wraps an inner effect — surface its filter so the
            // cast prompt narrows correctly when the inner effect needs
            // a target (e.g. "you may sacrifice [target permanent]").
            Effect::MayDo { body, .. } => body.primary_target_filter(),
            Effect::MayPay { body, .. } => body.primary_target_filter(),
            Effect::IfRevealFromHand { then, else_, .. } => then
                .primary_target_filter()
                .or_else(|| else_.primary_target_filter()),
            _ => None,
        }
    }

    /// Heuristic: does this effect's primary target want to be a *friendly*
    /// permanent (one the caster controls) rather than an opponent's? Drives
    /// `auto_target_for_effect` so the random bot doesn't waste Vines of
    /// Vastwood on the opp's bear or Reckless Charge on its own.
    ///
    /// Returns true for unconditional self-buffs (positive `PumpPT`,
    /// `GrantKeyword`, `+1/+1` `AddCounter`). Returns false for hostile
    /// effects (Destroy, Exile, DealDamage, …) and ambiguous ones.
    pub fn prefers_friendly_target(&self) -> bool {
        match self {
            Effect::PumpPT { power, toughness, .. } => {
                // Pump is friendly when the bonus is non-negative; debuffs
                // (Tragic Slip, Last Gasp) want opponent targets.
                Self::value_is_non_negative(power) && Self::value_is_non_negative(toughness)
            }
            // SetBasePT to 0/N (Square Up) is hostile when the base
            // power drops below the printed body — used as a removal-
            // adjacent effect to neutralize attackers. The bot prefers
            // an opp creature unless the toughness bump is the bigger
            // tell.
            Effect::SetBasePT { .. } => false,
            Effect::GrantKeyword { keyword, .. } => Self::keyword_is_friendly(keyword),
            Effect::AddCounter { kind, .. } => matches!(kind, CounterType::PlusOnePlusOne),
            Effect::Seq(v) => v.iter().any(|e| e.prefers_friendly_target()),
            Effect::If { then, else_, .. } => {
                then.prefers_friendly_target() || else_.prefers_friendly_target()
            }
            Effect::DelayUntil { body, .. } | Effect::Repeat { body, .. } => {
                body.prefers_friendly_target()
            }
            Effect::ForEach { body, .. } => body.prefers_friendly_target(),
            // Reanimate-style spells move target → caster's hand or battlefield.
            // Without this, `auto_target_for_effect` picks an opp's battlefield
            // creature first, and Disentomb / Raise Dead happily steal it.
            Effect::Move { to, .. } => matches!(
                to,
                ZoneDest::Hand(PlayerRef::You)
                    | ZoneDest::Battlefield { controller: PlayerRef::You, .. }
            ),
            _ => false,
        }
    }

    /// This effect's primary target is meant to be a card in *some*
    /// graveyard. Covers two cases:
    /// - Reanimate-class (Disentomb, Raise Dead, Reanimate, Goryo's
    ///   Vengeance) — `Move target → Hand(You)` / `Battlefield(You)`.
    /// - Graveyard hate (Ghost Vacuum's "exile target card from a
    ///   graveyard") — `Move target → Exile`.
    ///
    /// The auto-target heuristic walks graveyards (in friendly/hostile
    /// order) before the battlefield when this is set, so an `Any`-filtered
    /// Move-to-Exile picks a graveyard resident rather than a battlefield
    /// permanent that happens to be at the top of the scan.
    ///
    /// Battlefield Move-to-Exile is rare in the catalog (the canonical
    /// permanent-exile effect is `Effect::Exile`), so collapsing both
    /// graveyard-walk cases under one classifier is safe.
    pub fn prefers_graveyard_target(&self) -> bool {
        match self {
            Effect::Move { to, .. } => matches!(
                to,
                ZoneDest::Hand(PlayerRef::You)
                    | ZoneDest::Battlefield { controller: PlayerRef::You, .. }
                    | ZoneDest::Exile
            ),
            Effect::Seq(v) => v.iter().any(|e| e.prefers_graveyard_target()),
            Effect::If { then, else_, .. } => {
                then.prefers_graveyard_target() || else_.prefers_graveyard_target()
            }
            Effect::DelayUntil { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::ForEach { body, .. } => body.prefers_graveyard_target(),
            _ => false,
        }
    }

    /// True if a `Target::Player(_)` is a meaningful primary target for this
    /// effect. The auto-target heuristic uses this to skip player candidates
    /// when the effect actually operates on permanents — without it, an
    /// `Any`-filtered Move (Regrowth) auto-targets the caster as a player and
    /// silently fizzles, since `Effect::Move` only consumes
    /// `EntityRef::{Permanent,Card}` and ignores Player entries.
    ///
    /// Returns true for effects that legitimately point at a player face:
    /// damage, life-gain/loss, drain, mill/draw/discard against a player ref,
    /// surveil/scry/look (no-op for non-player anyway). False for effects that
    /// move/tap/destroy/exile cards.
    pub fn accepts_player_target(&self) -> bool {
        match self {
            Effect::DealDamage { .. }
            | Effect::GainLife { .. }
            | Effect::LoseLife { .. }
            | Effect::Drain { .. }
            | Effect::Discard { .. }
            | Effect::DiscardAnyNumber { .. }
            | Effect::SetNoMaxHandSize { .. }
            | Effect::Draw { .. }
            | Effect::Mill { .. }
            | Effect::AddPoison { .. } => true,
            // Stack-targeted counter spells take a permanent slot but the
            // target is a stack item, not a player. Reject player target.
            Effect::CounterSpell { .. }
            | Effect::CounterSpellToZone { .. }
            | Effect::CounterAbility { .. }
            | Effect::CounterUnlessPaid { .. }
            | Effect::CounterUnless { .. } => false,
            // Permanent-targeting effects: skip Player.
            Effect::Destroy { .. }
            | Effect::Exile { .. }
            | Effect::Tap { .. }
            | Effect::Untap { .. }
            | Effect::Move { .. }
            | Effect::AddCounter { .. }
            | Effect::RemoveCounter { .. }
            | Effect::PumpPT { .. }
            | Effect::SetBasePT { .. }
            | Effect::GrantKeyword { .. }
            | Effect::GainControl { .. }
            | Effect::ResetCreature { .. }
            | Effect::BecomeBasicLand { .. }
            | Effect::Attach { .. }
            | Effect::Fight { .. } => false,
            // Compound effects: defer to whichever child first surfaces a
            // primary-target filter — the auto-target heuristic's slot 0
            // is shared across the Seq, so a leading `Move(target → exile)`
            // dictates the target type for the whole spell, even if a
            // trailing `If(... GainLife)` would also accept Player. The
            // real-card example is Cling to Dust:
            //   `Seq([Move(target → Exile), If(EntityMatches Creature, GainLife)])`
            // Without this rule the bot picked Player(opp) first, which
            // matched the `Any` filter but silently fizzled at Move
            // resolution (Move only consumes Permanent/Card refs).
            Effect::Seq(v) => v
                .iter()
                .find(|e| e.primary_target_filter().is_some())
                .map(|e| e.accepts_player_target())
                .unwrap_or_else(|| v.iter().any(|e| e.accepts_player_target())),
            Effect::If { then, else_, .. } => {
                // Prefer the `then` branch (the active outcome) — same
                // logic as `ability_effect_label`. Fall back to else_'s
                // classification if `then` doesn't have a primary target.
                if then.primary_target_filter().is_some() {
                    then.accepts_player_target()
                } else if else_.primary_target_filter().is_some() {
                    else_.accepts_player_target()
                } else {
                    then.accepts_player_target() || else_.accepts_player_target()
                }
            }
            Effect::DelayUntil { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::ForEach { body, .. } => body.accepts_player_target(),
            Effect::ChooseMode(modes) => modes.iter().any(|e| e.accepts_player_target()),
            Effect::ChooseN { modes, .. } => modes.iter().any(|e| e.accepts_player_target()),
            // Conservative default: anything we don't classify is permitted.
            // The legality gate (filter + check_target_legality) still rejects
            // mismatched types, this just changes the heuristic's preference
            // order.
            _ => true,
        }
    }

    fn value_is_non_negative(v: &Value) -> bool {
        match v {
            Value::Const(n) => *n >= 0,
            // Dynamic values (`SacrificedPower`, `XFromCost`, etc.) are always
            // ≥ 0 in practice.
            _ => true,
        }
    }

    fn keyword_is_friendly(kw: &Keyword) -> bool {
        // Defensive / offensive keywords benefit the controller. We exclude
        // negative-value keywords like Defender / "can't attack" if they ever
        // get added; for now every Keyword variant is a buff.
        !matches!(
            kw,
            Keyword::Defender // arguably a debuff in isolation
        )
    }

    /// Walk the effect tree and return the first `SelectionRequirement` bound
    /// to the target slot `slot`, if any. Used for cast-time target validation.
    ///
    /// `mode` lets modal cards (`ChooseMode`) constrain the search to the
    /// chosen branch rather than picking up the first matching filter from
    /// any mode. Pass `None` for non-modal effects or to fall through to
    /// the legacy behaviour (first match across all modes).
    pub fn target_filter_for_slot_in_mode(
        &self,
        slot: u8,
        mode: Option<usize>,
    ) -> Option<&SelectionRequirement> {
        fn sel_find(s: &Selector, slot: u8) -> Option<&SelectionRequirement> {
            match s {
                Selector::TargetFiltered { slot: s2, filter } if *s2 == slot => Some(filter),
                Selector::AttachedTo(i) | Selector::AttachedToMe(i) => sel_find(i, slot),
                Selector::Take { inner, .. } => sel_find(inner, slot),
                _ => None,
            }
        }
        fn eff_find(
            e: &Effect,
            slot: u8,
            mode: Option<usize>,
        ) -> Option<&SelectionRequirement> {
            match e {
                Effect::Seq(v) => v.iter().find_map(|x| eff_find(x, slot, mode)),
                Effect::If { then, else_, .. } => eff_find(then, slot, mode)
                    .or_else(|| eff_find(else_, slot, mode)),
                Effect::ForEach { selector, body } => {
                    sel_find(selector, slot).or_else(|| eff_find(body, slot, mode))
                }
                Effect::Repeat { body, .. } => eff_find(body, slot, mode),
                Effect::ChooseMode(modes) => match mode {
                    // Mode-aware path: only look in the chosen branch.
                    Some(m) if m < modes.len() => eff_find(&modes[m], slot, None),
                    // Legacy path: first hit across all modes.
                    _ => modes.iter().find_map(|m| eff_find(m, slot, None)),
                },
                // ChooseN: the auto-decider picks specific mode indices;
                // the slot-0 target should match whichever picked mode
                // is first to require one. Scan only the picked modes.
                Effect::ChooseN { picks, modes } => picks
                    .iter()
                    .filter_map(|&i| modes.get(i as usize))
                    .find_map(|m| eff_find(m, slot, None)),
                Effect::MayDo { body, .. } | Effect::MayPay { body, .. } => {
                    eff_find(body, slot, mode)
                }
                Effect::IfRevealFromHand { then, else_, .. } => {
                    eff_find(then, slot, mode).or_else(|| eff_find(else_, slot, mode))
                }
                Effect::DealDamage { to, .. } => sel_find(to, slot),
                Effect::Fight { attacker, defender } => {
                    sel_find(attacker, slot).or_else(|| sel_find(defender, slot))
                }
                Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_find(who, slot),
                Effect::Drain { from, to, .. } => sel_find(from, slot).or_else(|| sel_find(to, slot)),
                Effect::Draw { who, .. } | Effect::Mill { who, .. } => sel_find(who, slot),
                Effect::Discard { who, .. } => sel_find(who, slot),
                Effect::DiscardAnyNumber { who } => sel_find(who, slot),
                Effect::SetNoMaxHandSize { who } => sel_find(who, slot),
                Effect::Move { what, .. } => sel_find(what, slot),
                Effect::Destroy { what }
                | Effect::Exile { what }
                | Effect::Tap { what }
                | Effect::Untap { what, .. }
                | Effect::CounterSpell { what }
                | Effect::CounterSpellToZone { what, .. }
                | Effect::CounterAbility { what }
                | Effect::CounterUnlessPaid { what, .. }
                | Effect::CounterUnless { what, .. }
                | Effect::GainControl { what, .. } => sel_find(what, slot),
                Effect::PumpPT { what, .. } => sel_find(what, slot),
                Effect::SetBasePT { what, .. } => sel_find(what, slot),
                Effect::GrantKeyword { what, .. } => sel_find(what, slot),
                Effect::AddCounter { what, .. } | Effect::RemoveCounter { what, .. } => {
                    sel_find(what, slot)
                }
                Effect::BecomeBasicLand { what, .. }
                | Effect::ResetCreature { what, .. } => sel_find(what, slot),
                Effect::Attach { what, to } => sel_find(what, slot).or_else(|| sel_find(to, slot)),
                Effect::CopySpell { what, .. }
                | Effect::CopySpellUnlessPaid { what, .. } => sel_find(what, slot),
                Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => {
                    sel_find(who, slot)
                }
                Effect::AddPoison { who, .. } => sel_find(who, slot),
                _ => None,
            }
        }
        eff_find(self, slot, mode)
    }

    /// Mode-agnostic shorthand for `target_filter_for_slot_in_mode(slot, None)`.
    /// For modal effects, returns the first filter from any mode (legacy
    /// behaviour preserved for callers that don't yet thread mode info).
    pub fn target_filter_for_slot(&self, slot: u8) -> Option<&SelectionRequirement> {
        self.target_filter_for_slot_in_mode(slot, None)
    }
}

fn zonedest_has_target(z: &ZoneDest) -> bool {
    match z {
        ZoneDest::Hand(p) | ZoneDest::Library { who: p, .. } => matches!(p, PlayerRef::Target(_)),
        ZoneDest::Battlefield { controller, .. } => matches!(controller, PlayerRef::Target(_)),
        ZoneDest::Graveyard | ZoneDest::Exile => false,
    }
}

// ── Static abilities ─────────────────────────────────────────────────────────

/// A static ability description — what continuous effect(s) it emits while
/// its source is on the battlefield. Translated at layer-computation time
/// into concrete [`ContinuousEffect`] values by the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAbility {
    pub description: &'static str,
    pub effect: StaticEffect,
}

/// A continuous effect produced by a static ability. Subsumes the old
/// `StaticAbilityTemplate` enum; maps 1-to-1 to one or more
/// `layers::Modification`s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaticEffect {
    /// Grant +p/+t to everything the selector picks.
    PumpPT { applies_to: Selector, power: i32, toughness: i32 },
    /// Grant a keyword to everything the selector picks.
    GrantKeyword { applies_to: Selector, keyword: Keyword },
    /// Replace ETB for matching permanents ("enters tapped").
    EntersTapped { applies_to: Selector },
    /// Controller may play one additional land per turn.
    ExtraLandPerTurn,
    /// Generic cost reduction for spells matching filter.
    CostReduction { filter: SelectionRequirement, amount: u32 },
    /// Target-aware generic cost reduction for spells whose chosen target
    /// matches `target_filter`. Powers Killian, Ink Duelist's "spells you
    /// cast that target a creature cost {2} less to cast."
    ///
    /// Applied during `cast_spell_with_convoke` (and the back-face / alt-
    /// cost siblings) *after* the cast's target is validated. The reduction
    /// is clamped at the spell's current generic-pip total (it cannot
    /// reduce a colored pip), matching CR 601.2f / CR 117.7c.
    CostReductionTargetingFilter {
        spell_filter: SelectionRequirement,
        target_filter: SelectionRequirement,
        amount: u32,
    },
    /// Damping-Sphere-style "spells cost {amount} more after the first
    /// spell that player casts each turn." `filter` narrows which spells
    /// are taxed; the cost increase is applied at cast time when the
    /// caster's `Player.spells_cast_this_turn >= 1`.
    AdditionalCostAfterFirstSpell { filter: SelectionRequirement, amount: u32 },
    /// Leyline-of-Sanctity-style "you have hexproof": opponents can't
    /// target the source's controller with spells or abilities they
    /// control. Checked by `check_target_legality` for `Target::Player(_)`.
    ControllerHasHexproof,
    /// Damping-Sphere-style "lands that tap for more than one mana enter
    /// producing only {C}". Detected at `play_land` time: if any active
    /// `LandsTapColorlessOnly` static is in play, the entering land's
    /// mana abilities are replaced with a single `{T}: Add {C}` ability
    /// when the original would produce > 1 mana per tap. Skipped on the
    /// front-face of MDFCs (which have only one ability) and on basic
    /// lands (single-color, single-mana already).
    LandsTapColorlessOnly,
    /// Teferi, Time Raveler-style: each opponent can cast spells only any
    /// time they could cast a sorcery. Checked at cast time on the
    /// opponent's side.
    OpponentsSorceryTimingOnly,
    /// Teferi, Time Raveler +1: until your next turn, you may cast sorcery
    /// spells as though they had flash. Tracked via `Player.sorceries_as_flash`
    /// (set/cleared by the loyalty ability + `do_untap`).
    ControllerSorceriesAsFlash,
    /// "If one or more tokens would be created under your control, twice
    /// that many tokens are created instead." Used by Adrix and Nev,
    /// Twincasters (Quandrix uncommon legendary). Doubling Season uses a
    /// stronger variant that also doubles counter accrual; this variant
    /// covers the token half only. The static is read at
    /// `Effect::CreateToken` resolution time: each active `DoubleTokens`
    /// permanent the controller has on the battlefield doubles the
    /// token count (2 doublers → 4×, 3 → 8×, …). CR 614.13 framing —
    /// the effect is a replacement that scales the create-token event.
    DoubleTokens,
}

// ── Triggered / activated / loyalty ability shells ───────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub event: EventSpec,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivatedAbility {
    pub tap_cost: bool,
    pub mana_cost: crate::mana::ManaCost,
    pub effect: Effect,
    pub once_per_turn: bool,
    pub sorcery_speed: bool,
    /// True if activating this ability requires sacrificing the source
    /// permanent as part of its cost. The sacrifice is applied **after**
    /// tap and mana payment succeed but **before** the effect is queued
    /// for resolution — so by the time the effect runs (or is pushed onto
    /// the stack), the source is already in the graveyard. Used by cards
    /// like Mind Stone (`{1}, {T}, Sacrifice this: Draw a card`),
    /// Cathar Commando, Greater Good, Zuran Orb, etc.
    pub sac_cost: bool,
    /// Optional gating predicate. When set, the activation is rejected
    /// before any cost is paid unless the predicate evaluates to true
    /// against the source/controller context. Used by activated abilities
    /// that include a printed "activate only if …" clause:
    /// - Resonating Lute's `{T}: Draw a card. Activate only if you have
    ///   seven or more cards in your hand.`
    /// - Potioner's Trove's `{T}: You gain 2 life. Activate only if
    ///   you've cast an instant or sorcery spell this turn.`
    /// - Stone Docent's `{W}, Exile this card from your graveyard:
    ///   You gain 2 life. Surveil 1. Activate only as a sorcery.` (the
    ///   sorcery-speed half is already covered by `sorcery_speed`; the
    ///   gate here is for arbitrary predicates).
    #[serde(default)]
    pub condition: Option<Predicate>,
    /// Additional life-payment cost (in addition to mana, tap, and sac).
    /// Paid up front during activation. Activation is rejected with
    /// `GameError::InsufficientLife` when the controller's current life
    /// is below `life_cost` (mirrors the mana-cost pre-pay check). Used
    /// by activated abilities that bake "Pay N life:" into the cost
    /// line — Great Hall of the Biblioplex's `{T}, Pay 1 life: Add one
    /// mana of any color`, future Phyrexian-mana flavoured activations,
    /// City of Brass-style "tap for damage" hybrids, etc.
    ///
    /// Defaults to 0 via `#[serde(default)]` so existing literal
    /// initialisations pick up the new field automatically.
    #[serde(default)]
    pub life_cost: u32,
    /// True if this ability is activated from the controller's graveyard
    /// rather than the battlefield. The activation walker searches the
    /// graveyard for the source instead of the battlefield. Used by
    /// SOS cards with `{cost}: do X` activated abilities that read like
    /// "Activate only from your graveyard." — Summoned Dromedary's
    /// `{1}{W}: return this from gy to hand. sorcery.`, Teacher's Pest's
    /// `{B}{G}: return this from gy to bf tapped.`, Stone Docent (with
    /// `exile_self_cost`), Eternal Student (with `exile_self_cost`),
    /// and Postmortem Professor (with `exile_self_cost` toggled
    /// separately for the "exile an IS from gy" portion not handled
    /// here — the source itself is in gy).
    ///
    /// Defaults to false via `#[serde(default)]` so all existing
    /// literal initializations pick up the new field automatically.
    #[serde(default)]
    pub from_graveyard: bool,
    /// True if activating this ability exiles the source as part of
    /// its cost. Used together with `from_graveyard: true` for cards
    /// whose printed cost line reads "Exile this card from your
    /// graveyard: …" (Stone Docent, Eternal Student). The exile
    /// happens after tap (n/a from gy) + mana + life payments succeed
    /// but **before** the effect resolves, mirroring `sac_cost`'s
    /// timing.
    ///
    /// Defaults to false via `#[serde(default)]`.
    #[serde(default)]
    pub exile_self_cost: bool,
    /// Optional cost: exile a *different* card from the controller's
    /// graveyard matching this filter. Used by activated abilities
    /// whose printed cost line reads "Exile a [filter] card from your
    /// graveyard:" where the exiled card is **not** the source — for
    /// example Postmortem Professor's `{1}{B}, Exile an instant or
    /// sorcery card from your graveyard: Return this card from your
    /// graveyard to the battlefield.` and Lorehold Pledgemage's
    /// `{2}{R}{W}, Exile a card from your graveyard: This creature
    /// gets +1/+1 until end of turn.`
    ///
    /// The exile is applied after tap / mana / life payments succeed
    /// but before the effect resolves, mirroring `sac_cost` /
    /// `exile_self_cost`. If no graveyard card matches, activation is
    /// rejected with `GameError::SelectionRequirementViolated`. The
    /// auto-picker takes the lowest-CMC matching card so the activator
    /// keeps higher-value cards in their graveyard.
    ///
    /// Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub exile_other_filter: Option<SelectionRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyAbility {
    pub loyalty_cost: i32,
    pub effect: Effect,
}

// ── Helpers / shortcut constructors ──────────────────────────────────────────

pub mod shortcut {
    //! Common one-liner constructors for building card definitions tersely.
    use super::*;

    pub fn you() -> Selector { Selector::You }
    pub fn this() -> Selector { Selector::This }
    pub fn target() -> Selector { Selector::Target(0) }
    pub fn target_n(n: u8) -> Selector { Selector::Target(n) }
    pub fn target_filtered(filter: SelectionRequirement) -> Selector {
        Selector::TargetFiltered { slot: 0, filter }
    }
    pub fn trigger_source() -> Selector { Selector::TriggerSource }

    pub fn each_creature() -> Selector {
        Selector::EachPermanent(SelectionRequirement::Creature)
    }
    pub fn each_your_creature() -> Selector {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    }
    pub fn each_opponent_creature() -> Selector {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
        )
    }
    pub fn each_opponent() -> Selector { Selector::Player(PlayerRef::EachOpponent) }

    pub fn n(x: i32) -> Value { Value::Const(x) }
    pub fn count(s: Selector) -> Value { Value::count(s) }

    pub fn deal(amount: i32, to: Selector) -> Effect {
        Effect::DealDamage { to, amount: Value::Const(amount) }
    }
    pub fn gain_life(amount: i32) -> Effect {
        Effect::GainLife { who: you(), amount: Value::Const(amount) }
    }
    pub fn lose_life(amount: i32, who: Selector) -> Effect {
        Effect::LoseLife { who, amount: Value::Const(amount) }
    }
    pub fn draw(n: i32) -> Effect {
        Effect::Draw { who: you(), amount: Value::Const(n) }
    }
    pub fn discard(who: Selector, n: i32, random: bool) -> Effect {
        Effect::Discard { who, amount: Value::Const(n), random }
    }
    pub fn destroy_target() -> Effect { Effect::Destroy { what: target() } }
    pub fn exile_target() -> Effect { Effect::Exile { what: target() } }
    pub fn return_target_to_hand() -> Effect {
        Effect::Move { what: target(), to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(target()))) }
    }
    pub fn pump_target(power: i32, toughness: i32) -> Effect {
        Effect::PumpPT {
            what: target(),
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        }
    }
    pub fn counter_target_spell() -> Effect {
        Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        }
    }
    pub fn add_mana(colors: Vec<Color>) -> Effect {
        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(colors) }
    }
    pub fn add_colorless(n: i32) -> Effect {
        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(n)) }
    }
    pub fn add_any_one_color(n: i32) -> Effect {
        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(n)) }
    }

    pub fn etb(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect,
        }
    }
    pub fn on_attack(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect,
        }
    }
    pub fn on_dies(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect,
        }
    }

    /// Predicate matching "the just-cast spell is an instant or a sorcery".
    /// Built around `Selector::TriggerSource` — at the spell-cast site,
    /// `fire_spell_cast_triggers` binds the just-cast `CardId` to
    /// TriggerSource for the duration of filter evaluation, so a
    /// `Predicate::EntityMatches { what: TriggerSource, filter: … }` reads
    /// the cast spell.
    pub fn cast_is_instant_or_sorcery() -> Predicate {
        Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::HasCardType(crate::card::CardType::Instant)
                .or(SelectionRequirement::HasCardType(crate::card::CardType::Sorcery)),
        }
    }

    /// Strixhaven Magecraft trigger: "Whenever you cast or copy an instant
    /// or sorcery spell, `effect`." Bundles the spell-cast trigger with
    /// the [`cast_is_instant_or_sorcery`] predicate. Used by Eager
    /// First-Year, Witherbloom Apprentice, etc.
    pub fn magecraft(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect,
        }
    }

    /// Strixhaven Repartee trigger: "Whenever you cast an instant or sorcery
    /// spell that targets a creature, `effect`." Bundles the magecraft
    /// filter (instant or sorcery) with `Predicate::CastSpellTargetsMatch`
    /// (target is a creature). The spell's chosen target is read from the
    /// cast-time `StackItem::Spell.target` slot — Repartee fires only when
    /// the target is currently a creature.
    pub fn repartee(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    cast_is_instant_or_sorcery(),
                    Predicate::CastSpellTargetsMatch(SelectionRequirement::Creature),
                ]),
            ),
            effect,
        }
    }

    /// Convenience: a Magecraft trigger that pumps the source itself.
    /// Wraps [`magecraft`] with a `PumpPT` body whose `what:` is the
    /// triggering permanent (`Selector::This`). Used by self-pump
    /// magecraft creatures (Symmetry Sage's +1/+0; future Witherbloom /
    /// Lorehold apprentices) so the call site stays one line. Duration
    /// defaults to end-of-turn since every printed magecraft self-pump
    /// uses that duration.
    pub fn magecraft_self_pump(power: i32, toughness: i32) -> TriggeredAbility {
        magecraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        })
    }

    /// Convenience: a Repartee trigger that pumps the source itself.
    /// Same shape as [`magecraft_self_pump`] but gated on the additional
    /// "spell targets a creature" Repartee predicate. Used by Rehearsed
    /// Debater (current SOS catalog), and any future Repartee creature
    /// that scales with cast events targeting a creature.
    pub fn repartee_self_pump(power: i32, toughness: i32) -> TriggeredAbility {
        repartee(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        })
    }

    /// Convenience: a Magecraft trigger that untaps the source itself.
    /// Wraps [`magecraft`] with an `Effect::Untap` body whose `what:` is
    /// the triggering permanent (`Selector::This`). Used by STX Hall
    /// Monitor; future "magecraft → untap this" cards (Pop Quiz-style
    /// Wizard chains, Galazeth-style mana ramps) will reuse it.
    pub fn magecraft_self_untap() -> TriggeredAbility {
        magecraft(Effect::Untap {
            what: Selector::This,
            up_to: None,
        })
    }

    /// Convenience: a Magecraft trigger that drains `amount` life from
    /// each opponent into the controller. Wraps [`magecraft`] with an
    /// `Effect::Drain { from: EachOpponent, to: You, amount }` body.
    /// The drain template is the canonical Witherbloom magecraft payoff
    /// (Witherbloom Apprentice, Sedgemoor Witch's death-trigger
    /// payoff, etc.); this shortcut keeps the call site one line.
    pub fn magecraft_drain_each_opp(amount: i32) -> TriggeredAbility {
        magecraft(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        })
    }

    /// Strixhaven Quandrix "spell with `{X}` in its mana cost" trigger:
    /// fires on any spell cast by the controller whose printed cost
    /// contains an `{X}` symbol. Powered by `Predicate::CastSpellHasX`.
    /// Used by Geometer's Arthropod, Matterbending Mage, and any future
    /// Quandrix card that pays off X-cost spells.
    pub fn cast_has_x_trigger(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellHasX),
            effect,
        }
    }

    /// Prowess trigger: "Whenever you cast a noncreature spell, this
    /// creature gets +1/+1 until end of turn." Fires on every cast you
    /// control whose card type is **not** Creature (the printed Prowess
    /// keyword's reminder text). The pumped target is the source itself
    /// via `Selector::This`, so a single Prowess creature can drop the
    /// helper in one line and the trigger source is correctly threaded.
    ///
    /// Wired into card factories declaring `Keyword::Prowess` —
    /// Spectacle Mage, Eccentric Apprentice, etc. — to convert the
    /// keyword tag into a functional trigger. (The keyword itself
    /// remains in `card.keywords` for display + future "Prowess matters"
    /// payoffs to filter on.)
    pub fn prowess() -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(crate::card::CardType::Creature)
                        .negate(),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }
    }

    /// SOS Increment trigger: "Whenever you cast a spell, if the amount
    /// of mana you spent is greater than this creature's power or
    /// toughness, [body]." Powered by `Predicate::IncrementSatisfied`,
    /// which compares the just-cast spell's stashed `mana_spent` to the
    /// listening permanent's effective P/T. The canonical Increment
    /// payoff drops a +1/+1 counter on `Selector::This`, but the helper
    /// is body-agnostic so cards like Pensive Professor (gain a +1/+1
    /// counter and scry 1) can plug arbitrary effects in.
    ///
    /// Implements MTG comp rules 603.4 ("intervening 'if' clause"): the
    /// `IncrementSatisfied` predicate is checked both at trigger-event
    /// time (the `EventSpec.filter` gate, controlling whether the
    /// trigger goes on the stack) AND at resolution time (the wrapping
    /// `Effect::If`, controlling whether the body actually runs). If
    /// the source gains counters after this trigger goes on the stack
    /// but before it resolves, the resolution-time check can suppress
    /// the body even though the trigger fired.
    pub fn increment_trigger(effect: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::IncrementSatisfied),
            effect: Effect::If {
                cond: Predicate::IncrementSatisfied,
                then: Box::new(effect),
                else_: Box::new(Effect::Noop),
            },
        }
    }

    /// SOS Increment payoff that drops one +1/+1 counter on the source.
    /// Wraps [`increment_trigger`] with the standard `AddCounter` body
    /// targeting `Selector::This`. Used by Cuboid Colony / Fractal
    /// Tender / Berta and every other vanilla-Increment creature.
    pub fn increment_self_plus_one() -> TriggeredAbility {
        use crate::card::CounterType;
        increment_trigger(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })
    }

    /// Strixhaven Opus payoff trigger: "Whenever you cast an instant or
    /// sorcery spell, [body]. If five or more mana was spent to cast
    /// that spell, [bigger body] instead." Emits an `If`-gated effect
    /// whose `Predicate::CastSpellManaSpentAtLeast(5)` arm fires the
    /// bigger payoff. Used by Deluge Virtuoso, Expressive Firedancer,
    /// Magmablood Archaic and other Opus creatures.
    pub fn opus_trigger(small_body: Effect, big_body: Effect) -> TriggeredAbility {
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::If {
                cond: Predicate::CastSpellManaSpentAtLeast(5),
                then: Box::new(big_body),
                else_: Box::new(small_body),
            },
        }
    }
}
