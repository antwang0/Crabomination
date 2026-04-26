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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayedKind {
    /// At the beginning of `controller`'s next upkeep.
    YourNextUpkeep,
    /// At the beginning of the next end step (any player's).
    NextEndStep,
}

// ── Pending decisions (suspendable resolution) ───────────────────────────────

/// A decision the engine is waiting on before it can continue resolving the
/// current spell or ability.
#[derive(Debug)]
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
#[derive(Debug)]
pub(crate) enum ResumeContext {
    Spell {
        card: Box<CardInstance>,
        caster: usize,
        target: Option<Target>,
        mode: usize,
        x_value: u32,
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

#[derive(Debug, Clone)]
pub enum PendingEffectState {
    ScryPeeked { count: usize, player: usize },
    SurveilPeeked { count: usize, player: usize },
    SearchPending { player: usize, to: crate::effect::ZoneDest },
    PutOnLibraryPending { player: usize, count: usize },
    /// Suspended on a `ChooseColor` for an `AnyOneColor(count)` mana
    /// payload — Black Lotus, Birds of Paradise, Mox Diamond. The UI picks
    /// a color and the engine adds `count` mana of that color.
    AnyOneColorPending { player: usize, count: u32 },
}

// ── Events ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum GameEvent {
    StepChanged(TurnStep),
    TurnStarted { player: usize, turn: u32 },
    CardDrawn { player: usize, card_id: CardId },
    CardDiscarded { player: usize, card_id: CardId },
    LandPlayed { player: usize, card_id: CardId },
    SpellCast { player: usize, card_id: CardId },
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
#[derive(Debug, Clone)]
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
        /// True if this spell can't be countered by spells or abilities
        /// (Cavern of Souls–style protection). `Effect::CounterSpell` skips
        /// these stack items.
        uncounterable: bool,
    },
    /// A triggered/loyalty ability waiting to resolve.
    Trigger {
        source: CardId,
        controller: usize,
        effect: Box<Effect>,
        target: Option<Target>,
        mode: Option<usize>,
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
}
