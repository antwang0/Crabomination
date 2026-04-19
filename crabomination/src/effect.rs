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

use crate::card::{CounterType, Keyword, LandType, SelectionRequirement, TokenDefinition, Zone};
use crate::mana::Color;

// ── PlayerRef / ZoneRef ───────────────────────────────────────────────────────

/// Lightweight reference to one or more players.
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// A zone plus optional owner (for zones like Hand/Library/Graveyard that
/// are per-player). Battlefield, Stack, Command are global.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// A single player, lifted to selector form.
    Player(PlayerRef),

    /// No entities (placeholder/default).
    None,
}

impl Selector {
    pub fn attached_to(inner: Selector) -> Self {
        Selector::AttachedTo(Box::new(inner))
    }
}

// ── Value ────────────────────────────────────────────────────────────────────

/// A numeric expression evaluated at effect-time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Const(i32),
    /// Number of entities the selector resolves to.
    CountOf(Box<Selector>),
    PowerOf(Box<Selector>),
    ToughnessOf(Box<Selector>),
    LifeOf(PlayerRef),
    HandSizeOf(PlayerRef),
    GraveyardSizeOf(PlayerRef),
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
}

impl Value {
    pub const ZERO: Value = Value::Const(0);
    pub const ONE: Value = Value::Const(1);
    pub fn count(sel: Selector) -> Self { Value::CountOf(Box::new(sel)) }
}

// ── Predicate ────────────────────────────────────────────────────────────────

/// A boolean game-state condition (for `Effect::If` / cast-time checks).
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// It's `who`'s turn.
    IsTurnOf(PlayerRef),
    /// The given entity's properties match the filter.
    EntityMatches { what: Selector, filter: SelectionRequirement },
}

// ── Duration ─────────────────────────────────────────────────────────────────

/// How long a temporary effect persists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryPosition {
    Top,
    Bottom,
    Shuffled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneDest {
    Hand(PlayerRef),
    Library { who: PlayerRef, pos: LibraryPosition },
    Graveyard,
    Exile,
    /// Battlefield under `controller`, optionally tapped.
    Battlefield { controller: PlayerRef, tapped: bool },
}

/// What mana to add to a pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaPayload {
    /// Add each listed color once.
    Colors(Vec<Color>),
    /// Add `amount` colorless mana.
    Colorless(Value),
    /// Add `amount` mana of any one color (player chooses).
    AnyOneColor(Value),
    /// Add `amount` mana of any colors (player chooses each).
    AnyColors(Value),
}

// ── Event specification (triggers) ───────────────────────────────────────────

/// Kinds of game events a trigger can watch for. Mirrors the `GameEvent`
/// stream in [`crate::game::types::GameEvent`].
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// Whose events does this trigger listen for?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// A structural filter over the unified `GameEvent` stream. The trigger fires
/// when an event of `kind` arrives, scoped per `scope`, and the optional
/// `filter` predicate holds in the post-event game state.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

    // ── Damage / life ────────────────────────────────────────────────────────
    DealDamage { to: Selector, amount: Value },
    GainLife  { who: Selector, amount: Value },
    LoseLife  { who: Selector, amount: Value },
    /// Controller loses `amount` life, a different selector gains it.
    Drain { from: Selector, to: Selector, amount: Value },

    // ── Cards / draw / discard / mill ────────────────────────────────────────
    Draw    { who: Selector, amount: Value },
    /// Discard `amount` cards. If `random`, chosen randomly; else by `who`.
    Discard { who: Selector, amount: Value, random: bool },
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
    Untap   { what: Selector },
    /// Give a temporary +P/+T bonus.
    PumpPT  { what: Selector, power: Value, toughness: Value, duration: Duration },
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
    /// Counter target spell (removes from stack).
    CounterSpell { what: Selector },
    /// Copy target spell/ability `count` times.
    CopySpell    { what: Selector, count: Value },

    // ── Sacrifice ────────────────────────────────────────────────────────────
    Sacrifice { who: Selector, count: Value, filter: SelectionRequirement },

    // ── Counters on players ──────────────────────────────────────────────────
    AddPoison { who: Selector, amount: Value },

    // ── Misc atomic operations needed by existing cards ──────────────────────
    /// Reveal the top card of `who`'s library; if `reveal_filter` matches, draw it.
    RevealTopAndDrawIf { who: PlayerRef, reveal_filter: SelectionRequirement },
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
                Value::LifeOf(p) | Value::HandSizeOf(p) | Value::GraveyardSizeOf(p) => {
                    player_has_target(p)
                }
                Value::Sum(vs) => vs.iter().any(value_has_target),
                Value::Diff(a, b) | Value::Times(a, b) | Value::Min(a, b) | Value::Max(a, b) => {
                    value_has_target(a) || value_has_target(b)
                }
                Value::NonNeg(v) => value_has_target(v),
                _ => false,
            }
        }
        fn pred_has_target(p: &Predicate) -> bool {
            match p {
                Predicate::Not(q) => pred_has_target(q),
                Predicate::All(v) | Predicate::Any(v) => v.iter().any(pred_has_target),
                Predicate::SelectorExists(s) => sel_has_target(s),
                Predicate::SelectorCountAtLeast { sel, n } => sel_has_target(sel) || value_has_target(n),
                Predicate::ValueAtLeast(a, b) | Predicate::ValueAtMost(a, b) => {
                    value_has_target(a) || value_has_target(b)
                }
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
            Effect::DealDamage { to, amount } => sel_has_target(to) || value_has_target(amount),
            Effect::GainLife { who, amount } | Effect::LoseLife { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::Drain { from, to, amount } => {
                sel_has_target(from) || sel_has_target(to) || value_has_target(amount)
            }
            Effect::Draw { who, amount }
            | Effect::Mill { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::Discard { who, amount, .. } => sel_has_target(who) || value_has_target(amount),
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
                    ManaPayload::Colors(_) => false,
                }
            }
            Effect::Destroy { what }
            | Effect::Exile { what }
            | Effect::Tap { what }
            | Effect::Untap { what }
            | Effect::CounterSpell { what } => sel_has_target(what),
            Effect::PumpPT { what, power, toughness, .. } => {
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
            Effect::Sacrifice { who, count, .. } => sel_has_target(who) || value_has_target(count),
            Effect::AddPoison { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::RevealTopAndDrawIf { who, .. } => player_has_target(who),
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
                _ => None,
            }
        }
        match self {
            Effect::DealDamage { to, .. } => sel_filter(to),
            Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_filter(who),
            Effect::Destroy { what }
            | Effect::Exile { what }
            | Effect::Tap { what }
            | Effect::Untap { what }
            | Effect::CounterSpell { what }
            | Effect::GainControl { what, .. } => sel_filter(what),
            Effect::AddCounter { what, .. } | Effect::RemoveCounter { what, .. } => sel_filter(what),
            Effect::PumpPT { what, .. } => sel_filter(what),
            Effect::GrantKeyword { what, .. } => sel_filter(what),
            Effect::Move { what, .. } => sel_filter(what),
            _ => None,
        }
    }

    /// Walk the effect tree and return the first `SelectionRequirement` bound
    /// to the target slot `slot`, if any. Used for cast-time target validation.
    pub fn target_filter_for_slot(&self, slot: u8) -> Option<&SelectionRequirement> {
        fn sel_find(s: &Selector, slot: u8) -> Option<&SelectionRequirement> {
            match s {
                Selector::TargetFiltered { slot: s2, filter } if *s2 == slot => Some(filter),
                Selector::AttachedTo(i) | Selector::AttachedToMe(i) => sel_find(i, slot),
                _ => None,
            }
        }
        fn eff_find(e: &Effect, slot: u8) -> Option<&SelectionRequirement> {
            match e {
                Effect::Seq(v) => v.iter().find_map(|x| eff_find(x, slot)),
                Effect::If { then, else_, .. } => {
                    eff_find(then, slot).or_else(|| eff_find(else_, slot))
                }
                Effect::ForEach { selector, body } => {
                    sel_find(selector, slot).or_else(|| eff_find(body, slot))
                }
                Effect::Repeat { body, .. } => eff_find(body, slot),
                Effect::ChooseMode(modes) => modes.iter().find_map(|m| eff_find(m, slot)),
                Effect::DealDamage { to, .. } => sel_find(to, slot),
                Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_find(who, slot),
                Effect::Drain { from, to, .. } => sel_find(from, slot).or_else(|| sel_find(to, slot)),
                Effect::Draw { who, .. } | Effect::Mill { who, .. } => sel_find(who, slot),
                Effect::Discard { who, .. } => sel_find(who, slot),
                Effect::Move { what, .. } => sel_find(what, slot),
                Effect::Destroy { what }
                | Effect::Exile { what }
                | Effect::Tap { what }
                | Effect::Untap { what }
                | Effect::CounterSpell { what }
                | Effect::GainControl { what, .. } => sel_find(what, slot),
                Effect::PumpPT { what, .. } => sel_find(what, slot),
                Effect::GrantKeyword { what, .. } => sel_find(what, slot),
                Effect::AddCounter { what, .. } | Effect::RemoveCounter { what, .. } => {
                    sel_find(what, slot)
                }
                Effect::BecomeBasicLand { what, .. }
                | Effect::ResetCreature { what, .. } => sel_find(what, slot),
                Effect::Attach { what, to } => sel_find(what, slot).or_else(|| sel_find(to, slot)),
                Effect::CopySpell { what, .. } => sel_find(what, slot),
                Effect::Sacrifice { who, .. } => sel_find(who, slot),
                Effect::AddPoison { who, .. } => sel_find(who, slot),
                _ => None,
            }
        }
        eff_find(self, slot)
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
#[derive(Debug, Clone)]
pub struct StaticAbility {
    pub description: &'static str,
    pub effect: StaticEffect,
}

/// A continuous effect produced by a static ability. Subsumes the old
/// `StaticAbilityTemplate` enum; maps 1-to-1 to one or more
/// `layers::Modification`s.
#[derive(Debug, Clone)]
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
}

// ── Triggered / activated / loyalty ability shells ───────────────────────────

#[derive(Debug, Clone)]
pub struct TriggeredAbility {
    pub event: EventSpec,
    pub effect: Effect,
}

#[derive(Debug, Clone)]
pub struct ActivatedAbility {
    pub tap_cost: bool,
    pub mana_cost: crate::mana::ManaCost,
    pub effect: Effect,
    pub once_per_turn: bool,
    pub sorcery_speed: bool,
}

#[derive(Debug, Clone)]
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
    pub fn counter_target_spell() -> Effect { Effect::CounterSpell { what: target() } }
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
}
