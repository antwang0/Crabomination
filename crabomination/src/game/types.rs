use serde::{Deserialize, Serialize};

use crate::card::{CardId, CardInstance, CounterType};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::mana::{Color, ManaError};

// ── Turn step sequence ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnStep {
    Untap,
    Upkeep,
    Draw,
    PreCombatMain,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    CombatDamage,
    EndCombat,
    PostCombatMain,
    End,
    Cleanup,
}

impl TurnStep {
    pub fn next(self) -> Self {
        match self {
            TurnStep::Untap => TurnStep::Upkeep,
            TurnStep::Upkeep => TurnStep::Draw,
            TurnStep::Draw => TurnStep::PreCombatMain,
            TurnStep::PreCombatMain => TurnStep::BeginCombat,
            TurnStep::BeginCombat => TurnStep::DeclareAttackers,
            TurnStep::DeclareAttackers => TurnStep::DeclareBlockers,
            TurnStep::DeclareBlockers => TurnStep::FirstStrikeDamage,
            TurnStep::FirstStrikeDamage => TurnStep::CombatDamage,
            TurnStep::CombatDamage => TurnStep::EndCombat,
            TurnStep::EndCombat => TurnStep::PostCombatMain,
            TurnStep::PostCombatMain => TurnStep::End,
            TurnStep::End => TurnStep::Cleanup,
            TurnStep::Cleanup => TurnStep::Untap,
        }
    }

    pub fn is_main_phase(self) -> bool {
        matches!(self, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
    }

    pub fn is_combat_phase(self) -> bool {
        matches!(
            self,
            TurnStep::BeginCombat
                | TurnStep::DeclareAttackers
                | TurnStep::DeclareBlockers
                | TurnStep::FirstStrikeDamage
                | TurnStep::CombatDamage
                | TurnStep::EndCombat
        )
    }
}

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
}

/// One attacker's declared assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attack {
    pub attacker: CardId,
    pub target: AttackTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    PlayLand(CardId),
    /// Play a modal-double-faced-card land using its **back face**. The
    /// resulting `CardInstance.definition` is swapped to the back face's
    /// definition before entering the battlefield, so all subsequent abilities
    /// (mana abilities, ETB triggers, land types) come from the back face.
    PlayLandBack(CardId),
    CastSpell { card_id: CardId, target: Option<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Cast a modal-double-faced card via its **back face**. Mirrors
    /// `PlayLandBack` but for non-land back faces (creature/instant/
    /// sorcery). The card's `definition` is swapped to the back face's
    /// definition before payment + cast, so cost / type / effect all
    /// resolve against the back face. Used by SOS MDFCs whose two
    /// faces are a creature and a spell (Studious First-Year //
    /// Rampant Growth, Adventurous Eater // Have a Bite, Emeritus of
    /// Truce // Swords to Plowshares, etc.).
    CastSpellBack {
        card_id: CardId,
        target: Option<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a spell with `Keyword::Convoke`, tapping each creature in
    /// `convoke_creatures` to contribute {1} generic mana toward the cost
    /// (real Magic also allows tapping for one colored mana matching the
    /// creature's identity — we collapse to generic for now; converge
    /// tracking still counts the creature's colors). Each must be an
    /// untapped creature controlled by the caster.
    CastSpellConvoke {
        card_id: CardId,
        target: Option<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: Vec<CardId>,
    },
    /// Cast a spell paying its `alternative_cost` instead of its regular
    /// mana cost. `pitch_card` is the hand card (e.g., a blue card for Force
    /// of Will/Negation) being exiled to satisfy the alt cost — `None` when
    /// the alt cost has no exile requirement.
    CastSpellAlternative {
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    ActivateAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    /// Declare attackers: each attacker picks a defending player or a
    /// planeswalker controlled by a non-active player.
    DeclareAttackers(Vec<Attack>),
    DeclareBlockers(Vec<(CardId, CardId)>),
    ActivateLoyaltyAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    CastFlashback { card_id: CardId, target: Option<Target>, mode: Option<usize>, x_value: Option<u32> },
    PassPriority,
    SubmitDecision(DecisionAnswer),
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
    /// True for one-shot triggers; removed after firing.
    pub fires_once: bool,
}

/// What kind of future event a delayed trigger waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedKind {
    /// At the beginning of `controller`'s next upkeep.
    YourNextUpkeep,
    /// At the beginning of the next end step (any player's).
    NextEndStep,
    /// At the beginning of `controller`'s next pre-combat main phase.
    /// Used by Chancellor of the Tangle ("at the beginning of your first
    /// main phase, add {G}"). Fires once on the controller's PreCombatMain
    /// step so the mana lands in the pool with main-phase windows still
    /// open (mana pools empty on step transition, MTG rule 500.4).
    YourNextMainPhase,
}

// ── Pending decisions (suspendable resolution) ───────────────────────────────

/// A decision the engine is waiting on before it can continue resolving the
/// current spell or ability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecision {
    pub decision: Decision,
    pub(crate) resume: ResumeContext,
}

impl PendingDecision {
    pub fn acting_player(&self) -> usize {
        match &self.resume {
            ResumeContext::Spell { caster, .. } => *caster,
            ResumeContext::Trigger { controller, .. } => *controller,
            ResumeContext::Ability { controller, .. } => *controller,
            ResumeContext::Mulligan { player, .. } => *player,
        }
    }
}

/// Recorded where resolution suspended so it can resume after the decision.
/// `remaining` is whatever effects in the original tree still need to run
/// after the answered decision is applied (e.g. the `Draw` half of `Opt`
/// suspended on its `Scry`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ResumeContext {
    Spell {
        card: Box<CardInstance>,
        caster: usize,
        target: Option<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        in_progress: PendingEffectState,
        remaining: Effect,
    },
    Trigger {
        source: CardId,
        controller: usize,
        target: Option<Target>,
        mode: usize,
        in_progress: PendingEffectState,
        remaining: Effect,
        /// X paid into the originating spell's cost. Threaded so a
        /// suspended ETB trigger that consults `Value::XFromCost` reads
        /// the right value when it's resumed after a pending decision.
        /// Defaults to 0 for snapshot backwards-compatibility.
        #[serde(default)]
        x_value: u32,
        /// Converge value (number of distinct colors of mana spent on
        /// the originating spell's cost). Same role as `x_value` for
        /// `Value::ConvergedValue`.
        #[serde(default)]
        converged_value: u32,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingEffectState {
    ScryPeeked { count: usize, player: usize },
    SurveilPeeked { count: usize, player: usize },
    SearchPending { player: usize, to: crate::effect::ZoneDest },
    PutOnLibraryPending { player: usize, count: usize },
    /// Suspended on a `ChooseColor` for an `AnyOneColor(count)` mana
    /// payload — Black Lotus, Birds of Paradise, Mox Diamond. The UI picks
    /// a color and the engine adds `count` mana of that color.
    AnyOneColorPending { player: usize, count: u32 },
    /// Suspended on a `DiscardChosen` decision (Inquisition of Kozilek,
    /// Thoughtseize). The caster picks cards from `target_player`'s hand;
    /// the apply step removes them and graveyards them.
    DiscardChosenPending { target_player: usize },
    /// Suspended on a `ChooseCreatureType` decision for `Effect::NameCreatureType`
    /// (Cavern of Souls). The chooser picks a creature type and the engine
    /// stamps it onto `target_id.chosen_creature_type`.
    ChooseCreatureTypePending { target_id: CardId },
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Which face / cast path a `SpellCast` event came from. Lets replays /
/// spectator UIs distinguish a back-face MDFC cast from the printed front
/// face, and a Flashback graveyard-replay from a normal hand cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CastFace {
    /// Default: cast from hand (or a generic alt-cost path).
    #[default]
    Front,
    /// Cast via `GameAction::CastSpellBack` against a non-land MDFC's
    /// `back_face`. The card's definition is swapped to the back face's
    /// for the duration of the cast and resolution.
    Back,
    /// Cast via `Keyword::Flashback` from the controller's graveyard.
    /// The card exiles after resolution rather than going to graveyard.
    Flashback,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    StepChanged(TurnStep),
    TurnStarted { player: usize, turn: u32 },
    CardDrawn { player: usize, card_id: CardId },
    CardDiscarded { player: usize, card_id: CardId },
    LandPlayed { player: usize, card_id: CardId },
    /// `face` distinguishes front-face / back-face / flashback casts.
    /// Defaults to `Front` for the typical hand cast; back-face MDFC
    /// casts and flashback graveyard replays carry the right tag so
    /// replays can render the correct cost.
    SpellCast { player: usize, card_id: CardId, face: CastFace },
    AbilityActivated { source: CardId },
    ManaAdded { player: usize, color: Color },
    ColorlessManaAdded { player: usize },
    PermanentEntered { card_id: CardId },
    PermanentExiled { card_id: CardId },
    DamageDealt { amount: u32, to_player: Option<usize>, to_card: Option<CardId> },
    LifeLost { player: usize, amount: u32 },
    LifeGained { player: usize, amount: u32 },
    CreatureDied { card_id: CardId },
    PumpApplied { card_id: CardId, power: i32, toughness: i32 },
    CounterAdded { card_id: CardId, counter_type: CounterType, count: u32 },
    CounterRemoved { card_id: CardId, counter_type: CounterType, count: u32 },
    PermanentTapped { card_id: CardId },
    PermanentUntapped { card_id: CardId },
    TokenCreated { card_id: CardId },
    CardMilled { player: usize, card_id: CardId },
    ScryPerformed { player: usize, looked_at: usize, bottomed: usize },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    CombatResolved,
    FirstStrikeDamageResolved,
    TopCardRevealed { player: usize, card_name: &'static str, is_land: bool },
    AttachmentMoved { attachment: CardId, attached_to: Option<CardId> },
    PoisonAdded { player: usize, amount: u32 },
    LoyaltyAbilityActivated { planeswalker: CardId, loyalty_change: i32 },
    LoyaltyChanged { card_id: CardId, new_loyalty: i32 },
    PlaneswalkerDied { card_id: CardId },
    SpellsCopied { original: CardId, count: u32 },
    SurveilPerformed { player: usize, looked_at: usize, graveyarded: usize },
    /// A card left `player`'s graveyard (returned to hand, battlefield, or
    /// exiled from there). Fires per card removed. Used by Strixhaven
    /// "cards leave your graveyard" payoffs.
    CardLeftGraveyard { player: usize, card_id: CardId },
    GameOver { winner: Option<usize> },
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
        /// Chosen mode index for `ChooseMode` effects (0 if `None`).
        mode: Option<usize>,
        /// X paid into the spell's cost. Threaded into `EffectContext.x_value`
        /// at resolution time so `Value::XFromCost` reads the actual paid X.
        x_value: u32,
        /// Number of distinct colors of mana spent paying this spell's
        /// cost. Read by `Value::ConvergedValue` (Prismatic Ending, Pest
        /// Control). Convoke pips contribute generic only, so they don't
        /// raise this value.
        converged_value: u32,
        /// True if this spell can't be countered by spells or abilities
        /// (Cavern of Souls–style protection). `Effect::CounterSpell` skips
        /// these stack items.
        uncounterable: bool,
        /// Cast-face / cast-path the spell came from (Front from hand,
        /// Back-face MDFC, or Flashback from graveyard). Threaded into
        /// `EffectContext.cast_face` at resolution time so
        /// `Predicate::CastFromGraveyard` can gate Antiquities-style
        /// "if cast from graveyard" riders. Defaults to `Front` for
        /// snapshot back-compat via `#[serde(default)]`.
        #[serde(default)]
        face: crate::game::types::CastFace,
        /// True if this stack item is a *copy* of a spell rather than the
        /// original cast (Casualty / Storm / Choreographed Sparks /
        /// Prismari Storm grant / Aziza's tap-3-to-copy / Mica's
        /// sac-artifact-to-copy). Copies don't pay costs, don't fire
        /// `SpellCast` triggers, and ceaseat resolution instead of going
        /// to the graveyard. Defaults to `false` for snapshot back-compat
        /// via `#[serde(default)]`.
        #[serde(default)]
        is_copy: bool,
    },
    /// A triggered/loyalty ability waiting to resolve.
    Trigger {
        source: CardId,
        controller: usize,
        effect: Box<Effect>,
        target: Option<Target>,
        mode: Option<usize>,
        /// X paid into the originating spell's cost, threaded so
        /// `Value::XFromCost` reads the right number when the trigger
        /// resolves. ETB triggers fired off a spell on resolution
        /// inherit the spell's X; loyalty/state triggers default to 0.
        /// Defaults to 0 via `#[serde(default)]` for snapshot
        /// backwards-compatibility.
        #[serde(default)]
        x_value: u32,
        /// Converge value (number of distinct colors of mana spent on
        /// the originating spell's cost). Threaded the same way as
        /// `x_value` so ETB triggers consulting `Value::ConvergedValue`
        /// (Rancorous Archaic, Snarl Song, Together as One) read the
        /// right number. Defaults to 0 for snapshot
        /// backwards-compatibility.
        #[serde(default)]
        converged_value: u32,
    },
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    #[error("It is not your priority to act")]
    NotYourPriority,
    #[error("Sorcery-speed only: stack must be empty and it must be your main phase")]
    SorcerySpeedOnly,
    #[error("Card {0:?} not found in hand")]
    CardNotInHand(CardId),
    #[error("Card {0:?} not found on battlefield")]
    CardNotOnBattlefield(CardId),
    #[error("Card {0:?} is not a land")]
    NotALand(CardId),
    #[error("Already played a land this turn")]
    AlreadyPlayedLand,
    #[error("Card {0:?} is tapped")]
    CardIsTapped(CardId),
    #[error("Creature {0:?} has summoning sickness")]
    SummoningSickness(CardId),
    #[error("Creature {0:?} cannot block (tapped, not a creature, or flying restriction)")]
    CannotBlock(CardId),
    #[error("Attacker {0:?} has Menace and must be blocked by two or more creatures")]
    MenaceRequiresTwoBlockers(CardId),
    #[error("Card {0:?} has Hexproof and cannot be targeted by opponents")]
    TargetHasHexproof(CardId),
    #[error("Card {0:?} has Shroud and cannot be targeted")]
    TargetHasShroud(CardId),
    #[error("Card {0:?} has protection from that color/quality")]
    TargetHasProtection(CardId),
    #[error("Mana: {0}")]
    Mana(#[from] ManaError),
    #[error("Wrong step for this action (currently {actual:?})")]
    WrongStep { actual: TurnStep },
    #[error("This action requires a target")]
    TargetRequired,
    #[error("Invalid target")]
    InvalidTarget,
    #[error("Ability index out of bounds")]
    AbilityIndexOutOfBounds,
    #[error("Activated ability already used this turn (once-per-turn)")]
    AbilityAlreadyUsedThisTurn,
    #[error("Activated ability's `activate only if` condition is not met")]
    AbilityConditionNotMet,
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
}
