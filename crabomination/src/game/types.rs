use crate::card::{CardId, CardInstance, CounterType};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::mana::{Color, ManaError};

// ── Turn step sequence ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub enum Target {
    Player(usize),
    Permanent(CardId),
}

#[derive(Debug, Clone)]
pub enum GameAction {
    PlayLand(CardId),
    CastSpell { card_id: CardId, target: Option<Target>, mode: Option<usize>, x_value: Option<u32> },
    ActivateAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    DeclareAttackers(Vec<CardId>),
    DeclareBlockers(Vec<(CardId, CardId)>),
    ActivateLoyaltyAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    CastFlashback { card_id: CardId, target: Option<Target>, mode: Option<usize>, x_value: Option<u32> },
    PassPriority,
    SubmitDecision(DecisionAnswer),
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
}

#[derive(Debug, Clone)]
pub enum PendingEffectState {
    ScryPeeked { count: usize, player: usize },
    SurveilPeeked { count: usize, player: usize },
    SearchPending { player: usize, to: crate::effect::ZoneDest },
    PutOnLibraryPending { player: usize, count: usize },
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

#[derive(Debug, Clone)]
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
}
