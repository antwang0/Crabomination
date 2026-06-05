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
use crate::mana::{Color, SpendRestriction};

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

/// Which players a player-targeted static effect affects. The static
/// is anchored on a permanent (the "source") and reads off that
/// source's controller seat at recompute time. Used by
/// `StaticEffect::PlayerCannotGainLife` and any future player-static
/// (lose-life redirection, hand-size caps, draw caps, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerStaticTarget {
    /// The source's controller — Sulfuric Vortex's "no player can gain
    /// life" applied to the controller-only side. Rare.
    Controller,
    /// Each opponent of the source's controller — the default for the
    /// printed "your opponents can't gain life" wording (Erebos,
    /// Rampaging Ferocidon, Tainted Remedy approximation).
    EachOpponent,
    /// Every player on the table — Sulfuric Vortex (each player can't
    /// gain life), Stigma Lasher's "permanents you control share the
    /// no-lifegain rider" template.
    EachPlayer,
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

    /// All tokens created by `Effect::CreateToken` in the current
    /// resolution (the multi-token variant of `LastCreatedToken`). Used
    /// by Fractal Spawning ("create two 0/0 Fractals, put a +1/+1
    /// counter on each of them") and any multi-mint-then-counter
    /// printed Oracle. Resets between resolution roots; within an
    /// `Effect::Seq`, every CreateToken from the current resolution
    /// is included. Push: modern_decks batch 28.
    LastCreatedTokens,

    /// All cards moved by `Effect::Move` (and Mill / Exile shortcuts)
    /// in the current resolution. Used by Practiced Scrollsmith,
    /// Suspend Aggression, Tablet of Discovery, Ark of Hunger, etc.
    /// to chain a `GrantMayPlay` immediately after the Move targets
    /// the same card(s). Cleared between resolution roots.
    LastMoved,

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
    /// The single creature `ctx.controller` controls with the least
    /// toughness (first in battlefield order on a tie). Resolves to an
    /// empty set when the controller has no creatures. Powers Bolster
    /// (CR 701.21 — "choose a creature with the least toughness among
    /// creatures you control").
    LeastToughnessYouControl,
    /// The permanent this one is attached to (for Auras/Equipment).
    AttachedTo(Box<Selector>),
    /// All permanents attached to `anchor`.
    AttachedToMe(Box<Selector>),

    /// Top `count` cards of `who`'s library.
    TopOfLibrary { who: PlayerRef, count: Value },
    /// Greedy walk of the top of `who`'s library, including each card
    /// (in order) until the running mana-value sum reaches `threshold`
    /// inclusive (i.e. the final card pushes the sum past the gate, and
    /// is included). Used by Improvisation Capstone's "exile cards from
    /// the top of your library until you exile cards with total mana
    /// value 4 or greater" rider — the engine previously hard-coded
    /// `Const(4)` cards, which under-counts when the top is land-heavy
    /// (top three MV-0 lands + one MV-4 spell = 4 cards, sum 4; the
    /// printed Oracle would walk past lands and stop when sum hits the
    /// threshold). Resolution: walk top→down summing each card's
    /// computed MV; stop after including the card that raises sum to
    /// ≥ threshold. Empty library returns nothing; library smaller than
    /// the running cap returns the whole library.
    TopOfLibraryUntilMvAtLeast { who: PlayerRef, threshold: Value },
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

    /// Walk `inner` in iteration order, accumulating `value_of_each`
    /// per entity, and take entities greedily while the running sum
    /// stays ≤ `cap`. Entities whose value would push the sum over
    /// `cap` are skipped; iteration continues so smaller items can
    /// still fit. Used by Spell Satchel's "Choose any number of
    /// target IS cards in your graveyard with total mana value 4 or
    /// less. Return them to your hand." The greedy walk gives the
    /// AutoDecider a deterministic pick; a real UI player would
    /// surface a per-card pick prompt with the same running cap.
    TakeWithSumCap {
        inner: Box<Selector>,
        cap: Box<Value>,
        value_of_each: Box<Value>,
    },

    /// All battlefield permanents (including the anchor itself) whose
    /// printed name matches the entity resolved by `inner`. Powers the
    /// printed "and each other permanent with the same name" / "all
    /// permanents with that name" riders — Maelstrom Pulse, Echoing Truth,
    /// Bile Blight-style sweepers. `inner` is typically `Target(0)`; if it
    /// resolves to nothing (or a non-permanent), this yields nothing.
    SharingNameWith(Box<Selector>),

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
    /// Maximum graveyard size across **every alive player** in the game.
    /// Reads `players[*].graveyard.len()` and returns the max. Backs
    /// "if a graveyard has 20 or more cards" payoffs (Visions of Beyond,
    /// future Tombstalker / Mercurial Chemister-style scaling). Distinct
    /// from `GraveyardSizeOf(p)` which only inspects a single player.
    MaxGraveyardSize,
    /// Number of cards in `who`'s library. Used by Body of Research's
    /// "for each card in your library" Fractal-token scaling.
    LibrarySizeOf(PlayerRef),
    /// The X value paid in the spell's cost.
    XFromCost,
    /// Number of spells cast this turn by controller (Storm).
    StormCount,
    /// CR 700.5 — controller's devotion to the given color(s): the number
    /// of mana symbols matching any listed color among the mana costs of
    /// permanents they control. Hybrid / Phyrexian pips count for each
    /// color half they contain. Drives Gray Merchant of Asphodel, the Nyx
    /// gods, Nykthos.
    DevotionTo(Vec<crate::mana::Color>),
    /// Counters of the given type on `what`.
    CountersOn { what: Box<Selector>, kind: CounterType },
    Sum(Vec<Value>),
    Diff(Box<Value>, Box<Value>),
    Times(Box<Value>, Box<Value>),
    Min(Box<Value>, Box<Value>),
    Max(Box<Value>, Box<Value>),
    /// Clamp the inner value to ≥0.
    NonNeg(Box<Value>),
    /// Conditional: if `value` ≥ `threshold`, evaluate `then`, else `else_`.
    /// Powers "if X is 4 or more, …" scaling (Mossborn Hydra's doubled
    /// counters at X≥4).
    IfAtLeast { value: Box<Value>, threshold: i32, then: Box<Value>, else_: Box<Value> },
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
    /// Maximum, across all players, of cards discarded so far within
    /// the current effect resolution. Reads from
    /// `state.cards_discarded_per_player_this_resolution`. Used by
    /// Windfall's printed "draws cards equal to the greatest number of
    /// cards a player discarded this way" — a `Seq([Discard(EachPlayer,
    /// 100), Draw(EachPlayer, MaxCardsDiscardedThisEffectByAnyPlayer)])`
    /// produces the correct dynamic yield instead of the prior flat 7.
    /// Reset to 0 between independent resolutions.
    MaxCardsDiscardedThisEffectByAnyPlayer,
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
    /// Number of distinct card types among the cards in `who`'s graveyard.
    /// Backs Broodspinner's "create that many Insects equal to the number of
    /// card types among cards in your graveyard" payoff.
    DistinctTypesInGraveyard { who: PlayerRef },
    /// Number of distinct card types among the cards in exile stamped
    /// `exiled_with = source` (the resolving source). Backs Keen-Eyed
    /// Curator's "four or more card types among cards exiled with this
    /// creature" threshold.
    DistinctCardTypesExiledWith,
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
    /// Number of creatures controlled by the resolved player. Sibling of
    /// `PermanentCountControlledBy` filtered to creatures. Powers
    /// Biorhythm's "each player's life total becomes the number of
    /// creatures they control" inside a `ForEach` over each player.
    CreatureCountControlledBy(PlayerRef),
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
    /// Number of creatures that died under `who`'s control so far this
    /// turn. Backed by `Player.creatures_died_this_turn` (bumped from the
    /// SBA death loop). Powers Witherbloom "harvest" payoffs that scale
    /// off the turn's carnage (e.g. "draw a card for each creature that
    /// died under your control this turn"). The companion predicate is
    /// `Predicate::CreaturesDiedThisTurnAtLeast`.
    CreaturesDiedThisTurn(PlayerRef),
    /// Number of creatures that died this turn across **every** player.
    /// Sums `Player.creatures_died_this_turn` over all seats. Powers
    /// table-wide aristocrat scaling, mirroring
    /// `Predicate::CreaturesDiedThisTurnTotalAtLeast`.
    CreaturesDiedThisTurnTotal,
    /// Number of permanents destroyed by `Effect::Destroy` earlier in this
    /// same resolution. Backed by `GameState.permanents_destroyed_this_resolution`.
    /// Powers Culling Ritual's "Add {B} or {G} for each permanent destroyed
    /// this way" — evaluate it in a later `Seq` step after the destruction.
    PermanentsDestroyedThisResolution,
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
    /// The value is odd. Powers "if it has an odd number of counters on it"
    /// riders (Sab-Sunen, Luxa Embodied). `ValueIsOdd(0)` is false (zero is
    /// even, CR-flavor).
    ValueIsOdd(Value),
    /// It's `who`'s turn.
    IsTurnOf(PlayerRef),
    /// The given entity's properties match the filter.
    EntityMatches { what: Selector, filter: SelectionRequirement },
    /// `who` has gained at least `at_least` total life this turn.
    /// Backed by `Player.life_gained_this_turn`. Used by Strixhaven's
    /// **Infusion** rider — "If you gained life this turn, …".
    LifeGainedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// CR 700.6 — `who` has the city's blessing. "As long as you have the
    /// city's blessing, …" (Ascend payoffs).
    HasCityBlessing { who: PlayerRef },
    /// CR 731 — it's currently day.
    IsDay,
    /// CR 731 — it's currently night.
    IsNight,
    /// CR 724 — `who` is the monarch ("as long as you're the monarch, …").
    IsMonarch { who: PlayerRef },
    /// True if any player matched by `who` has been dealt damage this turn.
    /// Backed by `Player.was_dealt_damage_this_turn`. Powers Bloodthirst
    /// (CR 702.54) — pair with `who: EachOpponent` for "if an opponent was
    /// dealt damage this turn."
    PlayerDamagedThisTurn { who: PlayerRef },
    /// True if any player matched by `who` has lost life this turn (damage or
    /// direct life loss). Backed by `Player.lost_life_this_turn`. Powers
    /// Spectacle (CR 702.111) — pair with `who: EachOpponent`.
    PlayerLostLifeThisTurn { who: PlayerRef },
    /// True if any player matched by `who` has an effective life total at most
    /// `life`. Powers "unless an opponent has N or less life" gates (Vampire
    /// Lacerator).
    PlayerLifeAtMost { who: PlayerRef, life: i32 },
    /// True if any player matched by `who` has the most life, or is tied for
    /// the most, among all (non-eliminated) players. Powers Dethrone (CR
    /// 702.105 — "attacks the player with the most life or tied for most
    /// life"); pair with `who: DefendingPlayer` on an `Attacks` trigger.
    PlayerHasMostLife { who: PlayerRef },
    /// True if any player matched by `who` has strictly less life than at least
    /// one of their opponents. Geyadrone Dihada's "if you have less life than
    /// an opponent" loyalty-reset rider.
    PlayerHasLessLifeThanOpponent { who: PlayerRef },
    /// True if the effect's source creature attacked this turn (CR 702.142
    /// Boast gate). Backed by `CardInstance.attacked_this_turn`.
    SourceAttackedThisTurn,
    /// True if the effect's source permanent is currently saddled (CR
    /// 702.171). Backed by `CardInstance.saddled`; gates "whenever this
    /// attacks while saddled" triggers on Mounts.
    SourceSaddled,
    /// True if any player `who` resolves to attacked with a creature this
    /// turn (Raid, CR 702.108 ability word). Backed by
    /// `Player.attacked_this_turn`.
    PlayerAttackedThisTurn { who: PlayerRef },
    /// True if any player `who` resolves to has cast a blue or black spell
    /// this turn (Veil of Summer's conditional cantrip).
    CastBlueOrBlackThisTurn { who: PlayerRef },
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
    /// `who` has cast *exactly* `count` spells so far this turn. Backed by
    /// `Player.spells_cast_this_turn` (already incremented for the current
    /// cast at trigger time). Used by "whenever a player casts their second
    /// spell each turn" triggers (Ledger Shredder) — pair with
    /// `PlayerRef::Triggerer` + `EventScope::AnyPlayer` so it reads the
    /// caster's own count and fires exactly on the Nth spell.
    SpellsCastThisTurnEquals { who: PlayerRef, count: Value },
    /// At least `at_least` creatures controlled by `who` died this turn.
    /// Backed by `Player.creatures_died_this_turn` (bumped from the SBA
    /// dies handler and `remove_to_graveyard_with_triggers`). Used by
    /// Witherbloom "if a creature died under your control this turn, …"
    /// end-step payoffs (Essenceknit Scholar).
    CreaturesDiedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// At least `at_least` creatures died this turn under **any** player's
    /// control — the global "Morbid" condition (CR 700.4 "a creature died
    /// this turn"). Sums `Player.creatures_died_this_turn` across all
    /// players, so a removal spell that killed an opponent's creature
    /// earlier this turn satisfies it. Cleaner than OR-ing
    /// `CreaturesDiedThisTurnAtLeast` over each seat. Used by Tragic Slip.
    CreaturesDiedThisTurnTotalAtLeast { at_least: Value },
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
    /// True if the just-cast spell (located via `ctx.trigger_source`) is
    /// itself a card matching `filter` — e.g. a noncreature spell (Sprite
    /// Dragon, Dragon's Rage Channeler) or an artifact spell. Evaluated
    /// against the topmost matching `StackItem::Spell`'s card definition.
    CastSpellMatches(SelectionRequirement),
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
    /// True if the just-cast spell's *owner* is not `ctx.controller`. A
    /// spell's owner is the player who owns the physical card (CR
    /// 108.3) — typically the same as its controller, but they diverge
    /// when one player casts another's card (Sen Triplets, Wandering
    /// Archaic, Possibility Storm, etc.). Used by Nita, Forum
    /// Conciliator's "whenever you cast a spell you don't own" trigger
    /// — fires only on the rare path where you cast a spell from an
    /// opponent's zone. Evaluated against `ctx.trigger_source`'s
    /// `StackItem::Spell.card.owner`.
    CastSpellNotOwnedByYou,
    /// True if `ctx.source` (the listening permanent's id) is currently
    /// in the engine's `permanents_gained_counter_this_turn` set — i.e.
    /// the listening permanent has had one or more counters put on it
    /// during the current turn. Used by Fractal Tender's end-step rider
    /// ("if you put a counter on this creature this turn, …"). Cleared
    /// at cleanup along with the other "this turn" tallies.
    SourceGainedCounterThisTurn,
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
    /// True when the resolving spell was cast from its caster's hand
    /// (the typical case). Inverse of `CastFromGraveyard`. Reserved for
    /// "if you cast this spell from your hand, …" rider patterns —
    /// Quandrix, the Proof's "instant and sorcery spells you cast from
    /// your hand have cascade" static gates against this predicate.
    /// Note: triggers / activated abilities default `cast_from_hand`
    /// to `true`, so this predicate evaluates as `True` outside of
    /// spell-resolution context too.
    CastFromHand,
    /// True when the resolving spell was kicked (CR 702.32) — its optional
    /// kicker cost was paid at cast time. Reads `EffectContext.kicked`,
    /// stamped from the resolving `CardInstance.kicked` flag. Used by
    /// "if this spell was kicked, …" riders (Tear Asunder). Non-spell
    /// contexts default `kicked` to `false`.
    SpellWasKicked,
    /// True if any opponent of `ctx.controller` controls more lands
    /// than `ctx.controller` does. Backed by walking the battlefield
    /// and counting `Land` permanents per seat. Used by catch-up ramp
    /// spells like Gift of Estates ("If an opponent controls more
    /// lands than you, …"), Tithe, Knight of the White Orchid's ETB
    /// trigger, and Land Tax.
    OpponentControlsMoreLandsThanYou,
    /// True when exactly one creature is attacking this combat — the
    /// CR 702.83a "attacks alone" condition that gates Exalted. Read
    /// from `GameState.attacking.len() == 1`. Outside a combat with
    /// declared attackers it evaluates `false`. Combined with an
    /// `Attacks / YourControl` trigger it implements the printed
    /// Exalted reminder ("Whenever a creature you control attacks
    /// alone, that creature gets +1/+1 until end of turn").
    AttackingAlone,
    /// CR 700.4-ish — **Delirium**: `who`'s graveyard holds cards of at
    /// least 4 distinct card types (the count of *types*, not cards). Backed
    /// by scanning the graveyard's `definition.card_types`. Used by Unholy
    /// Heat, Dragon's Rage Channeler, etc.
    DeliriumActive { who: PlayerRef },
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
    /// "On top or bottom — the card's owner chooses." Used by Run Behind
    /// ("Target creature's owner puts it on their choice of the top or
    /// bottom of their library."). At `place_card_in_dest` time, the
    /// owner is asked via `Decision::OptionalTrigger { description: "Put
    /// on top of library?" }`; yes = top, no = bottom. AutoDecider's
    /// default (`Bool(false)`) collapses to bottom — preserving the
    /// previous Run-Behind behavior. ScriptedDecider can flip to top
    /// for tests.
    OwnerChoice,
    /// "Put this card Nth from the top of the owner's library." Used by
    /// Approach of the Second Sun ({6}{W}{W}: "If this spell was cast
    /// from your hand and you've cast another spell named Approach of
    /// the Second Sun this game, you win the game. Otherwise, put this
    /// spell's owner gains 7 life and puts this spell into their
    /// library seventh from the top.") and similar cards. Per CR 401.7,
    /// if the library has fewer than N cards, the card goes on the
    /// bottom instead. `FromTop(0)` is equivalent to `Top`.
    FromTop(usize),
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
    /// Add `amount` mana, each pip chosen from the given color subset
    /// (player chooses per pip). The restricted-palette sibling of
    /// `AnyColors`. Used by Culling Ritual's "Add {B} or {G} for each
    /// permanent destroyed this way". Falls back to the first listed
    /// color when the decider can't choose.
    OfColors(Vec<Color>, Value),
    /// Add one mana of any color a controller's opponent's land could
    /// produce. The pool of legal colors is the union of basic-land
    /// types under any opponent's control (`Plains` → White, `Island`
    /// → Blue, `Swamp` → Black, `Mountain` → Red, `Forest` → Green).
    /// If no opponent controls a basic-typed land, falls back to
    /// colorless (so the activation never silently no-ops). Used by
    /// Fellwar Stone — `{T}: Add one mana of any color an opponent's
    /// land could produce.`
    AnyColorOpponentCouldProduce,
    /// Add one mana of any color a basic land *you* control could produce —
    /// the controller-side mirror of `AnyColorOpponentCouldProduce`. The
    /// legal-color set is the union of basic-land types under the
    /// controller's own permanents. Falls back to colorless if none.
    /// Star Compass.
    AnyColorYouCouldProduce,
    /// Player chooses a color, then adds mana of that color equal to their
    /// devotion to it (CR 700.5). Nykthos, Shrine to Nyx's second ability.
    DevotionOfChosenColor,
    /// Resolve the inner payload normally, but tag every colored pip it
    /// produces with `restriction` ("Spend this mana only to …"). Used by
    /// the Strixhaven school mana sources (Abstract Paintmage, Tablet of
    /// Discovery, Hydro-Channeler, Great Hall of the Biblioplex,
    /// Resonating Lute). Colorless pips in a wrapped payload are added
    /// unrestricted — no current card produces restricted colorless mana.
    Restricted(Box<ManaPayload>, SpendRestriction),
    /// Add one mana of the color stamped on the source's `chosen_color`
    /// (Coldsteel Heart, choose-a-color rocks). Falls back to colorless when
    /// no color was chosen.
    ChosenColorOfSource,
}

// ── Event specification (triggers) ───────────────────────────────────────────

/// Kinds of game events a trigger can watch for. Mirrors the `GameEvent`
/// stream in [`GameEvent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// A permanent entered the battlefield.
    EntersBattlefield,
    /// A creature died (hit a graveyard from the battlefield).
    CreatureDied,
    /// A creature was sacrificed. Per CR 701.16, "sacrifice" is a distinct
    /// game event from "die" — Mortician Beetle / Yahenni / Bone Picker
    /// ("Whenever a player sacrifices a creature") want this specific
    /// event, not a death-of-any-cause trigger. The `Effect::Sacrifice`
    /// resolver emits both events in order (CreatureSacrificed first,
    /// then CreatureDied) so existing death-triggers still fire.
    CreatureSacrificed,
    /// Any permanent was sacrificed (creature, artifact, enchantment,
    /// land, planeswalker). Per CR 701.16, this is the broader-scope
    /// sibling of `CreatureSacrificed` — "Whenever you sacrifice a
    /// permanent" payoffs (Korvold, Fae-Cursed King; Mayhem Devil;
    /// Cruel Celebrant for permanents) want this event so they catch
    /// Treasure-sac, Clue-sac, Food-sac, and land-sacrifice resolutions
    /// alongside creature sacrifices. The `Effect::Sacrifice` resolver
    /// emits this event for every sacrificed permanent, regardless of
    /// type; for creatures it additionally emits `CreatureSacrificed`
    /// (creatures fire BOTH events, matching CR 701.16's "every
    /// sacrifice of a creature is a sacrifice of a permanent" wording).
    PermanentSacrificed,
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
    /// An attacking creature finished the declare-blockers step
    /// without any blockers assigned to it (CR 509.3g — "Whenever
    /// [creature] attacks and isn't blocked"). Fires once per
    /// unblocked attacker after `declare_blockers` completes. The
    /// `BecomesBlocked` event fires for attackers WITH blockers; this
    /// is the parallel rail for unblocked attackers.
    AttacksAndIsntBlocked,
    /// Combat damage was dealt to a player by a creature.
    DealsCombatDamageToPlayer,
    /// Combat damage was dealt to a creature by a creature.
    DealsCombatDamageToCreature,
    /// CR 702.130 — **Enrage**: a permanent was dealt damage (combat or
    /// non-combat). Fires the source's enrage trigger. Unlike
    /// `DealsCombatDamageToCreature` (which is keyed on the *dealer* and
    /// only on combat damage), this is keyed on the *recipient* and fires
    /// on any damage — combat, burn spells, Fight, pingers — matching the
    /// printed "Whenever this creature is dealt damage" wording. The
    /// damage amount is exposed to the trigger body via
    /// `Value::TriggerEventAmount`. Used with `EventScope::SelfSource` for
    /// enrage creatures; `AnyPlayer`/`YourControl` scopes also work for
    /// "whenever a creature you control is dealt damage" payoffs.
    DealtDamage,
    /// A player gained life.
    LifeGained,
    /// A player lost life.
    LifeLost,
    /// The game entered a particular step.
    StepBegins(crate::turn_step::TurnStep),
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
    /// A permanent became the target of a spell or activated ability.
    /// Fires once per Permanent target at announce-time (when the spell
    /// hits the stack or the activated ability is pushed). Multi-target
    /// spells emit one event per target. For BecameTarget triggers the
    /// trigger source must be the targeted permanent (an implicit
    /// "target == source" check applied by `event_matches_spec`); the
    /// EventScope refines on the caster (`OpponentControl` → caster is
    /// an opponent, `YourControl` → caster is you). Used by SOS Tenured
    /// Concocter's "Whenever this creature becomes the target of a
    /// spell or ability an opponent controls, you may draw a card".
    BecameTarget,
    /// CR 702.29c — A card was cycled (the controller paid a cycling
    /// cost to discard it from hand and draw). This event is emitted
    /// from `GameState::cycle_card` *in addition* to `CardDiscarded`,
    /// so cycle-specific triggers ("When you cycle this card",
    /// "Whenever a player cycles a card") can fire without also
    /// triggering on regular hand-discards. The triggered ability's
    /// source typically lives on the cycled card itself (CR 702.29c —
    /// "These abilities trigger from whatever zone the card winds up
    /// in after it's cycled" — the engine reads the card from its new
    /// home in the graveyard). `EventScope::SelfSource` fires when the
    /// source card was the one cycled; `EventScope::YourControl` fires
    /// when any of the controller's cards were cycled.
    CardCycled,
    /// CR 702.108 — a permanent became untapped (Inspired). Fired once per
    /// permanent that flips tapped→untapped during the untap step. The
    /// triggering permanent is the event subject.
    BecomesUntapped,
    /// A permanent became tapped (Magda, Brazen Outlaw). The tapped
    /// permanent is the event subject; matched to `GameEvent::PermanentTapped`.
    Tapped,
    /// CR 701.40 — a permanent explored (Wildgrowth Walker, Tishana's
    /// Wayfinder payoffs). The exploring permanent is the event subject;
    /// matched to `GameEvent::Explored`.
    Explored,
    /// CR 701.31 — a permanent became monstrous (Fleecemane Lion, Nessian
    /// Wilds Ravager "when this becomes monstrous" triggers). The permanent
    /// is the event subject; matched to `GameEvent::BecameMonstrous`.
    BecameMonstrous,
    /// CR 107.16 — the controller got one or more {E} (energy counters).
    /// Fires once per `AddEnergy` resolution ("Whenever you get one or more
    /// {E}"); the amount is exposed via `Value::TriggerEventAmount`. The
    /// event subject is the player; matched to `GameEvent::EnergyGained`.
    EnergyGained,
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
    /// A permanent **you control** (any, including the source) becomes the
    /// target of a spell or ability an **opponent** controls. Used with
    /// `EventKind::BecameTarget` — Battle Mammoth. Unlike SelfSource, the
    /// targeted permanent need not be the trigger source; the dispatcher
    /// checks the targeted permanent's controller == trigger controller and
    /// the caster is an opponent.
    YourPermanentTargetedByOpponent,
    /// A creature an **opponent** controls attacks the source's controller
    /// (or a planeswalker they control). Used with `EventKind::Attacks`; the
    /// dispatcher binds the attacking creature's controller into the
    /// trigger's target slot (so "that creature's controller gains control"
    /// resolves via `PlayerRef::Target(0)` — Coveted Jewel).
    ControllerAttackedByOpponent,
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
/// What happens to tokens minted by [`Effect::CreateTokenAttacking`] when
/// the combat phase ends (CR 511.3 / the Mobilize end-of-combat sacrifice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttackingTokenCleanup {
    /// Tokens persist (a plain "tapped and attacking" mint).
    #[default]
    None,
    /// Sacrifice the tokens at end of combat (Mobilize).
    SacrificeAtEndOfCombat,
    /// Exile the tokens at end of combat (Myriad-style temporary copies).
    ExileAtEndOfCombat,
}

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
    /// CR 705 — flip a coin `count` times. For each flip, asks the
    /// controller's decider for `Decision::CoinFlip` (heads = true,
    /// tails = false), then runs `on_heads` or `on_tails`. Used by
    /// Karplusan Minotaur, Mana Clash, Krark's Thumb, and Ral Zarek's
    /// -7 ultimate.
    FlipCoin {
        count: Value,
        on_heads: Box<Effect>,
        on_tails: Box<Effect>,
    },
    /// CR 706 — roll `count` N-sided dice. For each die, ask the
    /// controller's decider for `Decision::DieRoll { sides }` (which
    /// returns `DecisionAnswer::DieRoll(n)` with `1 <= n <= sides`),
    /// then walk `results` and run the **first** matching arm whose
    /// `[low, high]` range covers `n`. If no arm matches, no effect
    /// runs for that die. Mirrors `FlipCoin`'s shape for the die
    /// equivalent. Used by Goblin Goliath, Wand of the Elements, and
    /// future Krark / Aether Sphere Harvester-style "roll N dice with
    /// a results table" cards. CR 706.1 + 706.3 covered; CR 706.2
    /// result modifiers are now applied via `modifier` (reroll /
    /// 706.2b is still engine-wide ⏳).
    RollDie {
        /// Number of sides on each die (e.g. 6 for d6, 20 for d20).
        /// Must be at least 2.
        sides: u8,
        /// Number of dice to roll. Each die rolls independently and
        /// runs its own results-table dispatch.
        count: Value,
        /// CR 706.2 — a flat modifier added to each natural die result
        /// before the results table is consulted (e.g. "roll a d20 and
        /// add 2"). The modified result is floored at 1 (a die's result
        /// is never reduced below 1) but may exceed `sides`, which lets
        /// an "N+" top arm catch boosted rolls. Defaults to 0 (no
        /// modifier) for snapshot back-compat. The natural roll is still
        /// what the decider returns; the modifier is applied on top.
        #[serde(default = "crate::effect::zero_value")]
        modifier: Value,
        /// CR 706.2b — reroll threshold. When greater than 0, any natural
        /// result `<= reroll_at_most` is rerolled exactly once and the new
        /// natural face is kept (even if it's also low — a single reroll
        /// per die, per the "reroll … once" pattern). The modifier is
        /// applied after the (re)roll. Models cards like "if you roll a 1,
        /// reroll" / "you may reroll any die that rolled a 1-N". Defaults
        /// to 0 (never reroll) for snapshot back-compat.
        #[serde(default)]
        reroll_at_most: u8,
        /// CR 706.3a — the results table. Each arm is `(low, high,
        /// effect)`; the first arm with `low <= rolled <= high` fires
        /// for that die. Use `(low, sides, effect)` for an "N+" arm,
        /// or `(n, n, effect)` for a single-number arm.
        results: Vec<(u8, u8, Effect)>,
        /// CR 706.5 — "if any of the dice rolled the same number, [effect]".
        /// When `count >= 2` and two or more dice share a *natural* face
        /// (doubles), this fires once after the per-die results dispatch.
        /// Defaults to `None` (no doubles check) for snapshot back-compat.
        #[serde(default)]
        on_doubles: Option<Box<Effect>>,
    },
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
    /// Each target-bearing mode owns its own target slot, assigned by the
    /// mode's position among the target-bearing modes in `picks`: the first
    /// such mode reads `ctx.targets[0]`, the second `ctx.targets[1]`, and so
    /// on. This lets "choose one or both" spells whose modes target
    /// different things (Steal the Show — target player for one mode, target
    /// creature for the other) supply and resolve a target per chosen mode.
    /// Non-targeting modes run with the full context. Cast-time validation
    /// keys off the same default-`picks` ordering
    /// (`target_filter_for_slot_in_mode`), so targets line up at resolution
    /// even when a decider runs only a subset.
    ///
    /// Limitation: because `picks` is the card's default, cast-time target
    /// validation assumes the default mode set; a decider that picks a
    /// different subset still supplies targets in the default-picks slot
    /// order. Full cast-time mode selection is tracked in TODO.md.
    ChooseN { picks: Vec<u8>, modes: Vec<Effect> },
    /// CR 702.119 — Escalate. "Choose one or more. You pay the escalate cost
    /// for each mode chosen beyond the first." The cast-time `mode` is the
    /// base (always-chosen) mode; a `Decision::ChooseModes` answer escalates
    /// to additional distinct modes, running `cost` (Collective Brutality's
    /// "discard a card", capped by hand size) once per extra mode. Each
    /// chosen target-bearing mode owns a target slot in run order. AutoDecider
    /// keeps just the base mode → no escalate cost, so a plain modal cast is
    /// unaffected. Modeled at resolution (escalate cards are sorceries with
    /// no cost/effect response window).
    Escalate {
        modes: Vec<Effect>,
        cost: Box<Effect>,
    },
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
    /// "Deal N damage divided as you choose among one or more / any number
    /// of targets." Targets are chosen at cast time across slots
    /// `0..max_targets` (each filtered by `filter`); the per-target split
    /// is decided at resolution via `Decision::DivideDamage` (AutoDecider
    /// spreads as evenly as possible). Used by Forked Bolt, Pyrokinesis,
    /// Fiery Cannonade-adjacent "divide" spells, Crackle with Power.
    DealDamageDivided {
        total: Value,
        filter: SelectionRequirement,
        max_targets: u8,
    },
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
    /// Each player the selector resolves to loses life equal to half their
    /// *own* current life total (rounded up when `rounded_up`, else down).
    /// Per-player evaluation — `LoseLife`'s single global amount can't scale
    /// to each target's own total. Stingerback Terror ("each opponent loses
    /// half their life, rounded up").
    LoseHalfLife { who: Selector, rounded_up: bool },
    /// Set a player's life total to a specific value (CR 119.5).
    /// "If an effect sets a player's life total to a specific number, the
    /// player gains or loses the necessary amount of life to end up with
    /// the new total." Used by Biorhythm-style "set life to creature
    /// count", Tree of Redemption-style "exchange life with toughness",
    /// and any future effect that pins life to a specific number.
    ///
    /// Implementation note: the resolver computes `delta = new_total -
    /// current_life` and emits either a `LifeGained` event (delta > 0)
    /// or `LifeLost` event (delta < 0). Delta of 0 emits no event
    /// (matches CR 119.9 / 119.10 zero-life-change semantics).
    SetLifeTotal { who: Selector, amount: Value },
    /// CR 701.12c — "exchange life totals" between the players the two
    /// selectors resolve to (Soul Conduit, Magus of the Mirror, Mirror
    /// Universe-style swaps). Each side's previous total is captured before
    /// either changes, then each player gains/loses to reach the other's
    /// previous total; `LifeGained`/`LifeLost` events fire so lifegain-
    /// matters payoffs see the swing. A no-op when both selectors land on
    /// the same player.
    ExchangeLifeTotals { a: Selector, b: Selector },
    /// One-turn "[selected players] can't gain life this turn" lock.
    /// Sets `Player.cannot_gain_life_this_turn = true` for each player
    /// the selector resolves to. Cleared by `do_untap` at the start
    /// of any new turn. Distinct from `StaticEffect::PlayerCannotGainLife`
    /// — this is a one-shot effect with no source permanent to anchor
    /// to (Skullcrack, Sulfurous Blast's flashback rider, future
    /// one-turn lifegain locks).
    LifeGainLockThisTurn { who: Selector },
    /// Set `Player.spells_uncounterable_this_turn` on each resolved player —
    /// "spells you control can't be countered for the rest of this turn"
    /// (Veil of Summer). Cleared at the next untap.
    GrantSpellsUncounterableThisTurn { who: Selector },
    /// Set `Player.cant_cast_noncreature_this_turn` on each resolved player —
    /// "those players can't cast noncreature spells this turn"
    /// (Ranger-Captain of Eos). Cleared at the next untap.
    CantCastNoncreatureThisTurn { who: Selector },
    /// Controller loses `amount` life, a different selector gains it.
    Drain { from: Selector, to: Selector, amount: Value },

    /// CR 122 / 107.16 — the controller gets `amount` energy counters
    /// ({E}). Energy is a per-player resource pool (`Player.energy`), not
    /// tied to any object. "You get {E}{E}" → `AddEnergy(Const(2))`.
    AddEnergy(Value),
    /// Pay `amount` energy counters as a cost at resolution; if the
    /// controller has at least that much, deduct it and resolve `then`,
    /// otherwise do nothing. Models energy-only activated/triggered
    /// payoffs ("Pay {E}{E}{E}: …") without a dedicated cost field on
    /// `ActivatedAbility` — the player commits by activating, and the
    /// energy is consumed when the ability resolves.
    PayEnergy { amount: u32, then: Box<Effect> },
    /// "Sacrifice/return this unless you pay {E}…" (CR 107.16). Pays `amount`
    /// energy if the controller can afford it; otherwise resolves `otherwise`
    /// (typically `SacrificeSource` / return-to-hand). AutoDecider pays when
    /// able. Lathnu Hellion, Greenbelt Rampager.
    PayEnergyOrElse { amount: u32, otherwise: Box<Effect> },

    // ── Cards / draw / discard / mill ────────────────────────────────────────
    Draw    { who: Selector, amount: Value },
    /// CR 701.45 — Learn. `who` may reveal a Lesson card they own from their
    /// sideboard ("outside the game") and put it into their hand, or discard
    /// a card to draw a card. Resolved via `Decision::Learn`. When `who`'s
    /// sideboard holds no Lesson, falls back to the legacy `Draw 1`
    /// approximation so no-sideboard games behave as before.
    Learn   { who: PlayerRef },
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
    /// Each player the selector resolves to mills half the cards in their
    /// *own* library (rounded up when `rounded_up`, else down). Per-player —
    /// `Mill`'s global amount can't scale to each target's own library size.
    /// Lord Xander, the Collector ("target opponent mills half their library,
    /// rounded down").
    MillHalf { who: Selector, rounded_up: bool },
    /// Each player the selector resolves to discards half the cards in their
    /// *own* hand (rounded up when `rounded_up`, else down), chosen the same
    /// way as `Discard` (random pick-first for the bot harness). Lord Xander
    /// ("target opponent discards half the cards in their hand, rounded down").
    DiscardHalf { who: Selector, rounded_up: bool },
    /// Each player the selector resolves to sacrifices half the permanents
    /// they control matching `filter` (rounded up when `rounded_up`, else
    /// down). Per-player. Lord Xander ("target opponent sacrifices half the
    /// permanents they control, rounded down" — `filter` = `Permanent`).
    SacrificeHalf { who: Selector, filter: SelectionRequirement, rounded_up: bool },
    Scry    { who: PlayerRef, amount: Value },
    Surveil { who: PlayerRef, amount: Value },
    LookAtTop { who: PlayerRef, amount: Value },
    /// CR 701.31 — *monstrosity N*. If the source isn't already monstrous,
    /// put N +1/+1 counters on it and it becomes monstrous (emitting
    /// `GameEvent::BecameMonstrous`). Once monstrous, this is a no-op.
    Monstrosity { n: Value },
    /// CR 701.38 — *goad* each creature `what` resolves to: the resolving
    /// effect's controller is added to the creature's `goaded_by` list.
    /// Goaded creatures attack each combat if able and attack a player other
    /// than a goader if able, until that goader's next turn. Disrupt Decorum
    /// (mass goad), Bloodthirsty Blade.
    Goad { what: Selector },
    /// CR 702.39 — *provoke*: untap the creature `what` resolves to and force
    /// it to block this combat's source attacker if able. Sets the target's
    /// `must_block` to the effect source. Used by `shortcut::provoke`.
    Provoke { what: Selector },
    /// CR 701.40 — each permanent `who` resolves to *explores*: its
    /// controller reveals the top card of their library. If it's a land,
    /// it goes to hand; otherwise the exploring permanent gets a +1/+1
    /// counter and the revealed card stays on top (the optional
    /// "put into graveyard" choice is collapsed to keep-on-top). An empty
    /// library still counts as a (cardless) explore and grants the counter.
    /// Each explore emits `GameEvent::Explored` so payoff triggers
    /// ("whenever a creature you control explores") can fire.
    Explore { who: Selector },
    /// "Look at the top `count` cards of your library, put one of them into
    /// your hand, and the rest on the bottom of your library (or into your
    /// graveyard if `rest_to_graveyard`)." Impulse / Strategic Planning /
    /// Flow State. The controller picks via the `SearchLibrary` decision
    /// (auto-decider keeps the top card).
    /// `pick_filter` restricts which revealed cards are eligible to take
    /// (Satyr Wayfinder — "you may put a *land* card into your hand"); the
    /// rest (including non-eligible cards) follow `rest_to_graveyard`.
    /// `None` means any revealed card is eligible.
    LookPickToHand {
        who: PlayerRef,
        count: Value,
        #[serde(default)]
        rest_to_graveyard: bool,
        #[serde(default)]
        pick_filter: Option<SelectionRequirement>,
        /// How many cards to put into hand (default 1). When >1 the controller
        /// picks the first via the decision and the rest auto-fill from the
        /// remaining eligible revealed cards. Consult the Star Charts kicked.
        #[serde(default)]
        take: Option<Value>,
    },
    /// "Reveal the top `count` cards of your library. For each card type, you
    /// may put a card of that type from among them into your hand. Put the
    /// rest on the bottom of your library in a random order." Atraxa, Grand
    /// Unifier. Resolution takes one revealed card per card type present
    /// (a card satisfying multiple types is taken once); the leftovers are
    /// bottomed. Card types considered: artifact, battle, creature,
    /// enchantment, instant, land, planeswalker, sorcery.
    RevealTopTakeOnePerType { who: PlayerRef, count: Value },
    /// Reveal the top `count` cards of `who`'s library, put every card
    /// matching `filter` into their hand, and bottom the rest in a random
    /// order (CR 401.4). "Put any number" is resolved as take-all-matching
    /// (the value-maximizing default). Torsten, Founder of Benalia's
    /// "reveal seven, take creatures and/or lands."
    RevealTopTakeMatchingToHand { who: PlayerRef, count: Value, filter: SelectionRequirement },
    /// Each player in `who` exiles all but the bottom `keep` cards of their
    /// library (face down — face-down exile isn't modeled, so the cards are
    /// exiled plainly). Doomsday Excruciator's "each player exiles all but the
    /// bottom six cards of their library."
    ExileLibraryExceptBottom { who: PlayerRef, keep: Value },

    // ── Zone moves ───────────────────────────────────────────────────────────
    /// Move every entity the selector resolves to into `to`.
    Move { what: Selector, to: ZoneDest },
    /// Search `who`'s library for a card matching `filter` and move to `to`.
    Search { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// Shuffle `who`'s graveyard into their library.
    ShuffleGraveyardIntoLibrary { who: PlayerRef },
    /// Shuffle `who`'s library (CR 103.2c). Mind's Desire's pre-exile shuffle.
    ShuffleLibrary { who: PlayerRef },

    // ── Mana ─────────────────────────────────────────────────────────────────
    AddMana { who: PlayerRef, pool: ManaPayload },

    // ── Permanent mutations ──────────────────────────────────────────────────
    Destroy { what: Selector },
    /// CR 701.15g — "Destroy ... It can't be regenerated." Behaves like
    /// `Destroy` but bypasses regeneration shields (Terminate, Putrefy,
    /// Day of Judgment, Vindicate, ...). Indestructible and Shield-counter
    /// replacements still apply — only regeneration is denied.
    DestroyNoRegen { what: Selector },
    /// CR 701.15 — add a regeneration shield to each resolved permanent.
    /// The shield is a one-shot replacement that fires the next time the
    /// permanent would be destroyed this turn (tap + remove from combat +
    /// heal damage instead of dying). Powers "{cost}: Regenerate this
    /// creature" activated abilities (Drudge Skeletons, River Boa, Korlash).
    Regenerate { what: Selector },
    /// "If [each resolved permanent] would die this turn, exile it instead."
    /// Installs an until-end-of-turn death replacement (CR 614, same shape
    /// as a finality counter) on every permanent the selector resolves to.
    /// Because the redirect lasts the whole turn, it catches deaths from
    /// later combat / removal too, not just the spell's own damage. Used by
    /// Wilt in the Heat (paired with a `DealDamage`).
    ExileIfWouldDieThisTurn { what: Selector },
    /// "Target instant/sorcery card in your graveyard gains flashback until
    /// end of turn; its flashback cost equals its mana cost." Installs an
    /// until-end-of-turn `granted_flashback_eot` (= the card's own mana
    /// cost) on each resolved graveyard card, making it castable via the
    /// normal flashback path (pay the cost, exile on resolve). Used by the
    /// SOS "Flashback" instant.
    GrantFlashbackThisTurn { what: Selector },
    /// "[Each resolved card] gains miracle `cost` until end of turn." Stamps
    /// an until-end-of-turn `may_play_until` permission **plus** a
    /// `granted_alt_cast_cost_eot` of `cost`, so the controller may cast the
    /// card this turn by paying `cost` (rather than its full mana cost or
    /// for free). Used by Lorehold, the Historian's "instant and sorcery
    /// cards in your hand have miracle {2}" grant.
    GrantMiracle { what: Selector, cost: crate::mana::ManaCost },
    Exile   { what: Selector },
    /// The "Enduring" cycle (Bloomburrow): "When this dies, if it was a
    /// creature, return it to the battlefield. It's an enchantment." Returns
    /// the source from its owner's graveyard to the battlefield under their
    /// control, then strips the Creature card type from the returned object
    /// so it comes back as a noncreature enchantment (and the gate self-
    /// limits — a noncreature can't satisfy "if it was a creature", so it
    /// won't loop). No-op if the source isn't a creature card in a graveyard.
    ReturnSelfAsEnchantment,
    /// "Exile target [permanent], then search its owner's graveyard, hand,
    /// and library for any number of cards with the same name as that
    /// [permanent] and exile them. Then that player shuffles." Crumble to
    /// Dust, Spreading Plague-style name sweeps. `what` resolves the
    /// anchor permanent (slot 0); its printed name keys the sweep.
    ExileSameNameAsTarget { what: Selector },
    /// Exile target card(s) and stamp each with `exiled_with = source`, the
    /// permanent association read by counting effects like
    /// `Value::DistinctCardTypesExiledWith`. Keen-Eyed Curator's
    /// "{1}: Exile target card from a graveyard."
    ExileTaggedWithSource { what: Selector },
    /// "Exile any number of target cards from graveyards." The controller
    /// picks a subset (via `Decision::ChooseCards`) of every graveyard card
    /// matching `filter`; chosen cards move to exile. AutoDecider exiles
    /// nothing (the conservative "up to" default); the bot exiles opponents'
    /// cards. Devious Cover-Up's graveyard-strip rider.
    ExileAnyNumberFromGraveyards { filter: crate::card::SelectionRequirement },
    /// CR 603.6e — "Exile [what] until [this] leaves the battlefield."
    /// Moves the resolved card(s) to exile, linking each to the source
    /// permanent (the ability's source). When that source leaves play the
    /// engine returns the exiled card(s) to `return_to`. Powers Banisher
    /// Priest / Fiend Hunter / Oblivion Ring (return to battlefield) and
    /// Brain Maggot / Tidehollow Sculler (return to hand).
    ExileUntilSourceLeaves {
        what: Selector,
        return_to: crate::card::ExileReturnZone,
    },
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
    /// Animate each permanent picked by `what` into a creature for
    /// `duration` (the canonical "manland" effect — Celestial Colonnade,
    /// Creeping Tar Pit, Mutavault, …). Installs a stack of continuous
    /// effects: layer-4 `AddCardType(Creature)` + each creature subtype,
    /// layer-7b `SetPowerToughness`, and layer-6 keyword grants. The
    /// permanent keeps its other types (a land stays a land — it becomes a
    /// "land creature"). Typically targets `Selector::This` from an
    /// activated ability, but works on any resolved permanent.
    BecomeCreature {
        what: Selector,
        power: Value,
        toughness: Value,
        creature_types: Vec<crate::card::CreatureType>,
        keywords: Vec<Keyword>,
        duration: Duration,
    },
    GrantKeyword { what: Selector, keyword: Keyword, duration: Duration },
    /// Each permanent picked by `what` becomes a single color of the
    /// controller's choice for `duration` (CR 105 / layer 5 SetColors).
    /// Wild Mongrel ("becomes the color of your choice until end of turn").
    BecomeChosenColor { what: Selector, duration: Duration },
    /// The controller chooses a color as the source enters; stamp it onto the
    /// source's `chosen_color` (CR 614 — Coldsteel Heart, choose-a-color mana
    /// rocks). Read later by `ManaPayload::ChosenColorOfSource`.
    ChooseColorForSelf,
    /// Each permanent picked by `what` gains protection from a color of the
    /// controller's choice for `duration` (`Decision::ChooseColor` →
    /// `Keyword::Protection(color)`). Mother of Runes, Giver of Runes, Gods
    /// Willing, Apostle's Blessing.
    GrantProtectionFromChosenColor { what: Selector, duration: Duration },
    /// Grant a transient triggered ability to each permanent picked by
    /// `what`, for `duration`. Stashed in `GameState.
    /// granted_triggers_eot` (only EOT duration is wired today;
    /// Permanent grants would need a separate map). The dispatcher
    /// walks both printed `triggered_abilities` and granted ones,
    /// firing matching events from either source. Used by Root
    /// Manipulation ("creatures you control gain 'whenever this
    /// creature attacks, you gain 1 life'") and Rabid Attack
    /// ("creatures gain 'when this creature dies, draw a card'" — die
    /// half requires LTB-trigger-snapshot follow-up).
    GrantTriggeredAbility {
        what: Selector,
        trigger: Box<crate::card::TriggeredAbility>,
        duration: Duration,
    },
    /// "Target creature loses all abilities until end of turn." Installs a
    /// `Modification::RemoveAllAbilities` continuous effect against each
    /// resolved permanent at layer 6. While in scope, the layer system
    /// clears keywords on the computed permanent AND flips its
    /// `lost_all_abilities` flag — the trigger dispatcher and activated-
    /// ability resolver consult that flag to skip the printed
    /// triggered/activated abilities (CR 113.10b). Used by Turn to Frog,
    /// Mercurial Transformation, Lignify (the "loses all abilities" half
    /// of these "creature becomes X" effects).
    LoseAllAbilities { what: Selector, duration: Duration },
    AddCounter    { what: Selector, kind: CounterType, amount: Value },
    RemoveCounter { what: Selector, kind: CounterType, amount: Value },
    /// Remove every counter of every kind from `what` (CR 122.6 — Vampire
    /// Hexmage's "remove all counters from target permanent").
    RemoveAllCounters { what: Selector },
    /// Set the loyalty (CR 606) of `what` to `value` — a loyalty-set effect
    /// ("its loyalty becomes …" / "reset to its starting loyalty"). Overwrites
    /// the `Loyalty` counter count outright rather than adding/removing.
    /// Geyadrone Dihada's +1 loyalty-reset rider.
    SetLoyalty { what: Selector, value: Value },
    /// CR 122.1b — Add a keyword counter to `what`. The host gains the
    /// named keyword while at least one counter of this kind is present
    /// (applied as a layer-6 grant in `compute_battlefield`). Removed
    /// independently of the keyword; the host loses the keyword
    /// (assuming no other source) when the last keyword counter is
    /// removed. Push (modern_decks batch 183): added per CR 122.1b.
    AddKeywordCounter { what: Selector, keyword: crate::card::Keyword, amount: Value },
    /// CR 122.1b — Remove up to `amount` keyword counters of `keyword`
    /// from `what`. Clamped at the source's actual count; the host loses
    /// the keyword (assuming no other source) when the last counter of
    /// this kind is removed. Counterpart to `AddKeywordCounter`. Push
    /// (claude/modern_decks, batches 192-193): added — closes the loop
    /// for "strip flight" / "remove a vigilance counter" style effects.
    RemoveKeywordCounter { what: Selector, keyword: crate::card::Keyword, amount: Value },
    /// CR 122.5 — move `amount` counters of `kind` from `from` to `to`.
    /// Clamped at the source's actual counter count; emits a single
    /// `CounterRemoved` for the source and a single `CounterAdded` for
    /// each target. The doubling-counter replacement (CR 614.16) does
    /// NOT apply to the destination — moves are explicitly NOT counter
    /// creation under CR 122.5 (the counters already exist; they're
    /// being relocated). Powers Tester of the Tangential's "pay {X}.
    /// When you do, move X +1/+1 counters from this creature onto
    /// another target creature" combat trigger.
    MoveCounter   { from: Selector, to: Selector, kind: CounterType, amount: Value },
    /// CR 701.34a — Proliferate. "Choose any number of permanents and/or
    /// players that have a counter, then give each another counter of a
    /// kind already there." The auto-decider implements a strategic
    /// baseline: grow good counters (+1/+1, Loyalty, Charge, Page) on the
    /// controller's permanents, grow bad counters (-1/-1, Stun) on enemy
    /// permanents, any other kind by default, and add one poison to each
    /// opponent already poisoned. (No multi-select UI yet.)
    Proliferate,
    /// Gain control of `what`. `to` names the new controller (`None` = the
    /// effect's controller, the common case — Threaten, Act of Treason). A
    /// `Some(pref)` hands control to another player — Wishclaw Talisman's
    /// "target opponent gains control" downside. `#[serde(default)]` keeps
    /// pre-field snapshots deserializing as `None`.
    GainControl {
        what: Selector,
        #[serde(default)]
        to: Option<PlayerRef>,
        duration: Duration,
    },
    /// Create `count` copies of the given token under `who`'s control.
    CreateToken { who: PlayerRef, count: Value, definition: TokenDefinition },
    /// Amass N (CR 701.43): put `count` +1/+1 counters on an Army `who`
    /// controls, creating a 0/0 black Army creature token first if they
    /// control none. `extra_type` is added to the Army (Amass Zombies /
    /// Amass Orcs mint a token that's also that subtype).
    Amass { who: PlayerRef, count: Value, extra_type: Option<crate::card::CreatureType> },
    /// Create `count` tokens already tapped and attacking (CR 508.3a). The
    /// new tokens join the current combat attacking the same defender the
    /// effect's source is attacking (falling back to the controller's first
    /// opponent when the source isn't itself an attacker). Powers "create N
    /// tokens tapped and attacking" riders and Mobilize (CR 702.169).
    /// `cleanup` registers the tokens to leave at end of combat. No-op
    /// outside the combat phase.
    CreateTokenAttacking {
        who: PlayerRef,
        count: Value,
        definition: TokenDefinition,
        #[serde(default)]
        cleanup: AttackingTokenCleanup,
    },
    /// Myriad (CR 702.115): for each opponent of the source's controller
    /// other than the player the source is attacking, create a token that's
    /// a copy of the source, tapped and attacking that opponent. The copies
    /// are exiled at end of combat. No-op outside combat / when the source
    /// isn't attacking a player.
    Myriad,
    /// "The next instant or sorcery spell you cast this turn costs {amount}
    /// less to cast" (Thundertrap Trainer). Pushes a one-shot discount onto
    /// `Player.pending_is_discounts` that lapses after the next such spell.
    GrantNextInstantOrSorceryDiscountThisTurn { amount: u32 },
    /// Support N (CR 701.32): "Put a +1/+1 counter on each of up to N target
    /// creatures." Each of slots `0..max_targets` is an optional creature
    /// target (filtered by `filter`); every supplied target gains one +1/+1
    /// counter at resolution.
    SupportCounters { max_targets: u8, filter: SelectionRequirement },
    /// Enlist (CR 702.151): "As this attacks, you may tap a nonattacking
    /// creature you control without summoning sickness. When you do, add its
    /// power to this creature's power until end of turn." The "you may" /
    /// "which creature" collapses to auto-tapping the highest-power eligible
    /// creature (only when its power is positive, so it's never a downgrade).
    Enlist,
    /// Create `count` token copies of the permanent resolved by `source`,
    /// controlled by `who`. The copy inherits the source's printed
    /// CardDefinition (name, P/T, types, keywords, activated/triggered
    /// abilities, static abilities). `extra_creature_types` are added on
    /// top of the source's printed creature subtypes (Applied Geometry,
    /// Echocasting Symposium: "create a token that's a copy of target X,
    /// except it's a 0/0 Fractal creature in addition to its other
    /// types" — pass `vec![CreatureType::Fractal]` to honor the printed
    /// "in addition to" rider). The token is stamped with `is_token =
    /// true` so token-cleanup SBA removes it when it leaves the
    /// battlefield. Power/toughness override is honored when both
    /// `override_pt: Some((p, t))` is set (Applied Geometry overrides
    /// the source's printed P/T to 0/0). The override applies *before*
    /// any +1/+1 counter pile.
    CreateTokenCopyOf {
        who: PlayerRef,
        count: Value,
        source: Selector,
        #[serde(default)]
        extra_creature_types: Vec<crate::card::CreatureType>,
        #[serde(default)]
        override_pt: Option<(i32, i32)>,
        /// CR 707.2e rider — the token copy isn't legendary (Helm of the
        /// Host). Strips supertypes from the copy so the legend rule doesn't
        /// destroy it alongside a legendary host.
        #[serde(default)]
        non_legendary: bool,
    },
    /// CR 701.32 — Populate: `who` creates a token that's a copy of a creature
    /// token they control (their choice; AutoDecider keeps the highest-power
    /// one). No-op if they control no creature token.
    Populate { who: PlayerRef },
    /// CR 707.2 — `what` becomes a copy of the permanent resolved by
    /// `source`: its copiable characteristics (name, mana cost, card
    /// types, subtypes, abilities, P/T, loyalty) are overwritten with a
    /// clone of the source's current definition. The copy is locked in at
    /// resolution time — later changes to the source don't propagate (a
    /// one-shot definition rewrite, the same mechanism the engine uses for
    /// MDFC face-swap, rather than a layer-1 continuous effect). Instance
    /// state (id, owner, controller, counters, tapped, summoning sickness,
    /// token-ness) is preserved. `extra_creature_types` are added on top
    /// of the copied subtypes (Phantasmal Image's "it's an Illusion in
    /// addition to its other types"). Powers Clone, Phantasmal Image,
    /// Mockingbird (enter-as-a-copy). If `source` resolves to nothing the
    /// effect is a no-op (the copier stays itself — usually a 0/0 that
    /// dies to SBA, matching the printed "you may" decline).
    BecomeCopyOf {
        what: Selector,
        source: Selector,
        #[serde(default)]
        extra_creature_types: Vec<crate::card::CreatureType>,
    },
    /// Target becomes a basic land of `land_type` (losing other types/abilities).
    BecomeBasicLand { what: Selector, land_type: LandType, duration: Duration },
    /// Target becomes a creature with the given P/T and creature types,
    /// losing all other card types, abilities, and creature subtypes
    /// (CR 613 layers 4/6/7). Oko's "becomes a 3/3 Elk", Turn to Frog's
    /// "0/1 blue Frog with no abilities", etc. Defaults to a vanilla 1/1.
    ResetCreature {
        what: Selector,
        #[serde(default = "value_one")]
        power: Value,
        #[serde(default = "value_one")]
        toughness: Value,
        #[serde(default)]
        creature_types: Vec<crate::card::CreatureType>,
        duration: Duration,
    },
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
    /// Copy target spell `count` times, then "you may choose new targets
    /// for the copy/copies" (CR 707.12 + 115.7). Same resolver as
    /// `CopySpell`, but after each copy is pushed the controller's decider
    /// is consulted (`Decision::ChooseTarget`, original target offered
    /// first) to re-point the copy's primary target at any legal object.
    /// Reverberate / Fork / Twincast. AutoDecider keeps the original
    /// target (first legal); a scripted/UI decider can repoint it.
    CopySpellMayChooseTargets { what: Selector, count: Value },
    /// CR 115.7 — "You may choose new targets for target spell." Repoints
    /// the targeted spell's primary target in place (Redirect). The
    /// spell's controller's opponent (the redirector) is consulted via
    /// `Decision::ChooseTarget`, original offered first so AutoDecider
    /// keeps it. Unlike `CopySpellMayChooseTargets` this mutates the
    /// original spell rather than a copy.
    ChooseNewTargetsForSpell { what: Selector },

    // ── Cast-without-paying / may-play ───────────────────────────────────────
    /// "Until [duration], you may cast/play that card [from where it is]."
    /// Stamps `CardInstance.may_play_until` on every card matched by `what`
    /// (typically a single target in graveyard or exile). The granted
    /// player then invokes `GameAction::CastFromZoneWithoutPaying` during
    /// a sorcery-speed (or instant-speed if the card is an instant) window
    /// to actually cast it.
    ///
    /// `to_owner` flips the recipient from "this effect's controller" to
    /// "the matched card's owner" — used by Suspend Aggression's "its
    /// owner may play it until the end of their next turn."
    ///
    /// `exile_after` propagates to the permission so casts pay this off
    /// land the resolved instant/sorcery in exile (Nita, The Dawning
    /// Archaic). For permanent spells the flag is ignored — they enter
    /// the battlefield normally.
    GrantMayPlay {
        what: Selector,
        duration: crate::card::MayPlayDuration,
        #[serde(default)]
        to_owner: bool,
        #[serde(default)]
        exile_after: bool,
    },
    /// Resolve-now equivalent of `GrantMayPlay`: at effect resolution
    /// time, ask the controller "cast `what` without paying its mana
    /// cost?" via `Decision::OptionalTrigger`. On yes, the card is
    /// pushed through the free-cast helper from `source_zone` with
    /// auto-targets / auto-decisions; on no (or no match), nothing
    /// happens.
    ///
    /// Used by Improvisation Capstone (each exiled non-land card),
    /// The Dawning Archaic (attack trigger), Nita Forum Conciliator
    /// (could use either model; we use this for the trigger half).
    /// `source_zone` is `Graveyard` for "cast it from your graveyard"
    /// and `Exile` for "cast it from exile."
    CastWithoutPayingImmediate {
        what: Selector,
        source_zone: crate::card::Zone,
        #[serde(default)]
        exile_after: bool,
    },
    /// Paradigm (SOS supplemental keyword) — registers a non-one-shot
    /// `DelayedKind::YourNextMainPhase` trigger that on each of the
    /// controller's pre-combat main phases offers them "cast a copy of
    /// this from exile without paying its mana cost?" via
    /// `Effect::CastFreeParadigmCopy`. Used as the trailing effect of
    /// the Paradigm Lesson cycle (Restoration Seminar, Decorum
    /// Dissertation, Germination Practicum, Echocasting Symposium,
    /// Improvisation Capstone). Pairs with `exile_on_resolve = true` so
    /// the card lands in exile and stays reachable for the recurring
    /// copy trigger.
    RegisterParadigm,
    /// Paradigm body. At trigger-fire time, locates the trigger's
    /// `source` (the Paradigm-exiled card) in exile, asks the controller
    /// "cast a copy?" via OptionalTrigger, and on yes mints a tokenized
    /// copy of the card's definition + free-casts it with auto-targets.
    /// The original exiled card is left untouched so the recurrence
    /// continues each main phase.
    CastFreeParadigmCopy,

    /// Cascade (CR 702.85). Triggered "when you cast this spell": exile
    /// cards from the top of the controller's library until a nonland
    /// card with mana value strictly less than `max_mv` is exiled. The
    /// controller may cast that card without paying its mana cost. The
    /// remaining exiled cards go to the bottom of the library (random
    /// order — approximated as bottom, the same indistinguishable model
    /// `RevealMissDest::BottomRandom` uses, since the bottom ordering is
    /// hidden until the next shuffle/reveal).
    ///
    /// `max_mv` is the cascading spell's mana value. Card factories pass
    /// `Value::Const(printed_mv)` (cascade's MV gate is the printed cost,
    /// unaffected by cost reduction per CR 702.85b). The shortcut
    /// [`cascade`] wires the standard SpellCast/SelfSource trigger.
    Cascade { max_mv: Value },

    /// Exile the top card of `who`'s library and stamp a may-play
    /// permission on it for `duration`. Used by Conspiracy Theorist,
    /// Elemental Mascot, Ark of Hunger, Archaic's Agony and similar
    /// "exile top of library; until [end of next turn / end of turn] you
    /// may play that card" effects. A single-shot composite that
    /// combines `Move(TopOfLibrary → Exile)` + `MayPlayPermission`
    /// stamp atomically so the may-play targets the just-moved card.
    /// `count` exiles that many cards off the top (Fallen Shinobi peels
    /// two), stamping each with the may-play permission.
    ExileTopAndGrantMayPlay {
        who: PlayerRef,
        count: Value,
        duration: crate::card::MayPlayDuration,
    },

    // ── Sacrifice ────────────────────────────────────────────────────────────
    Sacrifice { who: Selector, count: Value, filter: SelectionRequirement },
    /// Sacrifice this effect's source permanent (CR 701.16), firing proper
    /// death triggers. Used by end-of-turn self-sacrifice (Blitz, Ball
    /// Lightning) where `Effect::Move { This → Graveyard }` would skip the
    /// `CreatureDied` event.
    SacrificeSource,
    /// "Sacrifice any number of [filter]. [payoff] for each one." The
    /// controller chooses how many to sacrifice via `Decision::ChooseAmount`
    /// (AutoDecider sacrifices none). For each sacrifice, `per_each` runs
    /// once — so a `GainLife 3` body pays 3 × count. Plunge into Darkness.
    SacrificeAnyNumber {
        who: PlayerRef,
        filter: SelectionRequirement,
        per_each: Box<Effect>,
    },
    /// "Pay any amount of life. Look at that many cards from the top of your
    /// library, put one into your hand, and exile the rest." The controller
    /// chooses the amount via `Decision::ChooseAmount` (capped at current
    /// life; AutoDecider pays 0). Plunge into Darkness mode 1.
    PayLifeLookTake { who: PlayerRef },
    /// "Sacrifice a [filter] with the greatest mana value" picker.
    /// Mirrors `Sacrifice` but the candidate sort prefers maximum CMC.
    /// Used by Soul Shatter ("Each opponent sacrifices a creature or
    /// planeswalker with the greatest mana value among permanents
    /// that player controls"). Auto-decider picks the highest-CMC
    /// matching permanent per player.
    SacrificeGreatestMV { who: Selector, count: Value, filter: SelectionRequirement },

    /// "Punisher" choice (CR 601-style "unless"). Each player `chooser`
    /// resolves to may avoid `otherwise` by performing one of `options`.
    /// The engine resolves heuristically: that player performs the first
    /// option they can afford (LoseLife within their life total, Sacrifice
    /// with a legal permanent); if none is affordable, `otherwise` runs
    /// for the ability's controller. Options run with the chooser as the
    /// effect controller, so they use `Selector::Player(PlayerRef::You)`.
    /// Indulgent Tormentor: each opponent pays 3 life or sacrifices a
    /// creature, otherwise the controller draws a card.
    Punisher {
        chooser: Selector,
        options: Vec<Effect>,
        otherwise: Box<Effect>,
    },

    // ── Counters on players ──────────────────────────────────────────────────
    AddPoison { who: Selector, amount: Value },
    /// CR 122.1i / 728 — give each resolved player `amount` rad counters.
    AddRadCounters { who: Selector, amount: Value },

    // ── Misc atomic operations needed by existing cards ──────────────────────
    /// Reveal the top card of `who`'s library; if `reveal_filter` matches, draw it.
    RevealTopAndDrawIf { who: PlayerRef, reveal_filter: SelectionRequirement },

    /// Reveal the top card of `who`'s library (fires `TopCardRevealed` event for
    /// the animation) without moving it. Used by Chaos Warp's "reveal top card"
    /// step where the put-onto-battlefield clause is handled separately.
    RevealTopCard { who: PlayerRef },

    /// Reveal the top card of `who`'s library; if it's a permanent card, put it
    /// onto the battlefield under its owner's control (firing ETB). Otherwise
    /// it stays on top. Chaos Warp.
    RevealTopPutPermanentOntoBattlefield { who: PlayerRef },

    /// Reveal the top `count` cards of the controller's library; an opponent
    /// chooses one of them, which goes to the controller's hand. Each
    /// remaining revealed card is exiled, gaining `counter` if `Some`.
    /// Karn, Scion of Urza's +1 (reveal two, opponent chooses, exile the
    /// other with a silver counter). The opponent's pick is a heuristic
    /// (give the controller the lowest-value card), mirroring `Punisher`.
    RevealTopOpponentChoosesToHand { count: Value, counter: Option<crate::card::CounterType> },

    /// Return one card the controller owns with a `counter` counter on it
    /// from exile to their hand (removing the counter). Karn's −1. When more
    /// than one qualifies the controller takes the highest-value one.
    ReturnFromExileWithCounter { counter: crate::card::CounterType },

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
    /// CR 603.6e — "Target player reveals their hand; you choose [count]
    /// card(s) matching [filter]. Exile [them] until [this] leaves the
    /// battlefield." The caster picks from `from`'s hand; the chosen
    /// card(s) are exiled and linked to the ability's source. Powers Brain
    /// Maggot / Tidehollow Sculler / Kitesail Freebooter (`return_to`
    /// = Hand).
    ExileChosenUntilSourceLeaves {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        return_to: crate::card::ExileReturnZone,
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

    /// "When [target creature] dies this turn, [body]." Registers an
    /// event-keyed delayed trigger watching `ctx.targets[0]`'s death. The
    /// targeted creature's controller is captured as `Target::Player` so the
    /// body can reference it via `Selector::Target(0)` even after the
    /// creature has left the battlefield. Expires at cleanup. Used by
    /// Searing Blood ("deals 3 damage to its controller").
    WhenTargetDiesThisTurn { body: Box<Effect> },

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

    /// Grant the controller one additional land play this turn. Used by
    /// Explore, Dryad of the Ilysian Grove, Oracle of Mul Daya, and similar
    /// "you may play an additional land" effects. Bumps
    /// `Player.extra_land_plays` by `count`.
    GrantExtraLandPlay { who: PlayerRef, count: Value },

    /// "As [this] enters, choose a creature type." Used by Cavern of Souls.
    /// Asks the controller via the `ChooseCreatureType` decision and stores
    /// the chosen type on the source permanent's `chosen_creature_type`
    /// field. Subsequent cast paths consult that field via
    /// `caster_grants_uncounterable` to gate which creature spells the
    /// Cavern protects (only those that share the named type).
    NameCreatureType { what: Selector },

    /// CR 201.3 — "As [this] enters, choose a card name." Pithing Needle,
    /// Phyrexian Revoker. Asks the controller via the `NameCard` decision and
    /// stores the chosen name on the source permanent's `named_card` field;
    /// `activate_ability` then suppresses non-mana activated abilities of
    /// sources with that name. `what` selects the permanent to stamp
    /// (typically `Selector::This`).
    NameCard { what: Selector },

    /// "[Player] skips their next `count` turns." Bumps the affected
    /// player's `skip_turns` counter; the turn-advance logic in
    /// `do_cleanup` decrements and bypasses each scheduled-skip turn.
    /// Used by Ral Zarek, Guest Lecturer's -7 ult ("Flip five coins.
    /// Target opponent skips their next X turns, where X is the number
    /// of coins that came up heads.") via a `FlipCoin` + `SkipTurns`
    /// chain.
    SkipTurns { who: PlayerRef, count: Value },
    /// CR 724 — `who` becomes the monarch. "You become the monarch."
    BecomeMonarch { who: PlayerRef },
    /// CR 702.131 — Ascend. If `who` controls ten or more permanents, they
    /// get the city's blessing (a permanent player designation). A no-op
    /// otherwise. "Ascend" on a sorcery/instant resolves once; the
    /// permanent-static variant re-checks each time it's seen.
    Ascend { who: PlayerRef },
    /// CR 731 — "it becomes day." Sets the game's day designation.
    BecomeDay,
    /// CR 731 — "it becomes night." Sets the game's night designation.
    BecomeNight,
    /// CR 500.7 — "[Player] takes [count] extra turn(s) after this one."
    /// Banks `count` onto each resolved player's `extra_turns`; consumed
    /// by `advance_turn`. Time Walk, Temporal Manipulation, Ral Zarek's
    /// -7 coin-flip emblem.
    TakeExtraTurn { who: PlayerRef, count: Value },
    /// CR 505.1b — "there is an additional combat phase after this one."
    /// Banks `count` onto `GameState.additional_combat_phases`; when the
    /// active player leaves the End of Combat step with the counter set, the
    /// turn loops back to Begin Combat (a fresh combat phase) instead of
    /// advancing to the postcombat main. Built for combat-phase activated
    /// extra-combat effects (Hellkite Charger, Aggravated Assault while in
    /// combat), usually paired with an `Untap` so creatures can attack again.
    /// Main-phase-cast "after this main phase, an additional combat + main"
    /// sorceries (Relentless Assault) aren't supported yet — see TODO.md.
    AdditionalCombatPhase { count: Value },
    /// CR 114 — "[Player] gets an emblem with '[triggered abilities]'."
    /// Appends an `Emblem` (named after its source) to the player's
    /// emblem zone. Emblems never leave; their triggered abilities fire
    /// from the command zone alongside battlefield permanents (the
    /// dispatcher walks each player's emblems). Used by planeswalker
    /// ultimates — Professor Dellian Fel's -6, the upkeep-draw / end-step
    /// emblems, etc.
    CreateEmblem { who: PlayerRef, name: String, triggered: Vec<TriggeredAbility> },

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

    /// CR 614.9 — "Prevent all combat damage that would be dealt to and dealt
    /// by `target` this turn." Adds the target creature to
    /// `GameState.combat_damage_prevented_creatures`; the combat resolver
    /// then skips that creature in both directions. Maze of Ith.
    PreventAllCombatDamageInvolving { target: Selector },

    /// "Prevent the next N damage that would be dealt to `target` this
    /// turn." (CR 615.7) Pushes a per-target prevention shield consumed
    /// by the non-combat damage path; the shield expires at cleanup.
    /// Samite Healer, Healing Salve, Awe Strike-style effects.
    PreventNextDamage { target: Selector, amount: Value },

    /// "Prevent all damage that would be dealt to `target` this turn."
    /// (CR 615) A fog scoped to one player/permanent — Pradesh Gypsies,
    /// "you don't lose / prevent all damage to you". Non-combat path.
    PreventAllDamageThisTurn { target: Selector },

    /// "Damage can't be prevented this turn." (CR 615.12) Sets a global
    /// flag that suppresses every prevention shield for the rest of the
    /// turn. Skullcrack, Heated Debate, Impractical Joke's rider.
    DamageCantBePreventedThisTurn,

    /// "Choose a creature type. Creatures other than creatures of the
    /// chosen type get -P/-T until end of turn." Crippling Fear-style
    /// choose-and-sweep primitive. Synchronously surfaces a
    /// `ChooseCreatureType` decision (caster's seat) and then applies
    /// `PumpPT(power, toughness, EOT)` to every battlefield creature
    /// whose `definition.subtypes.creature_types` does NOT contain the
    /// answered type. The decision is resolved synchronously off
    /// `self.decider`, so AutoDecider (which picks `Demon`) and
    /// ScriptedDecider both work; UI players don't get a separate
    /// prompt today (degraded to the auto-decider choice — same as
    /// other implicit-choice cards).
    DiminishCreaturesExceptChosenType { power: Value, toughness: Value },
}

/// Lightweight mirror of `DelayedKind` for use inside
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

mod query;

/// Serde default for `Effect::RollDie.modifier` — a zero (no-op) die-roll
/// modifier, so snapshots written before CR 706.2 modifiers existed still
/// deserialize cleanly.
pub fn zero_value() -> Value {
    Value::Const(0)
}

/// Serde default for `ResetCreature` P/T (vanilla 1/1).
pub fn value_one() -> Value {
    Value::Const(1)
}

fn zonedest_has_target(z: &ZoneDest) -> bool {
    match z {
        ZoneDest::Hand(p) | ZoneDest::Library { who: p, .. } => matches!(p, PlayerRef::Target(_)),
        ZoneDest::Battlefield { controller, .. } => matches!(controller, PlayerRef::Target(_)),
        ZoneDest::Graveyard | ZoneDest::Exile => false,
    }
}

// ── Static abilities / ability shells (see effect/abilities.rs) ─────────────
mod abilities;
pub use abilities::*;

// ── Helpers / shortcut constructors ──────────────────────────────────────────

pub mod shortcut;
