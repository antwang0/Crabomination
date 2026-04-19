use crate::card::{CardId, CardInstance, CounterType, SpellEffect};
use crate::decision::{Decision, DecisionAnswer};
use crate::mana::{Color, ManaError};

// ── Turn step sequence ────────────────────────────────────────────────────────

/// Flat enumeration of every step and phase in an MTG turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnStep {
    Untap,
    Upkeep,
    Draw,
    PreCombatMain,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    /// First-strike and double-strike creatures deal damage here (CR 510.1).
    /// Only present when at least one attacker or blocker has first/double strike;
    /// skipped automatically if none are present.
    FirstStrikeDamage,
    /// Normal (non-first-strike) and double-strike creatures deal damage here.
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
    /// Cast a spell.
    ///
    /// - `target`: the chosen target for targeted spells (required when the
    ///   spell has a targeted effect).
    /// - `mode`: for `ChooseOne` modal spells, the 0-based index of the chosen
    ///   option. Defaults to `0` if `None`.
    CastSpell { card_id: CardId, target: Option<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Activate ability at `ability_index` on a permanent.
    ActivateAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    /// Declare one or more attackers at once (must be in DeclareAttackers step).
    DeclareAttackers(Vec<CardId>),
    /// Defending player assigns blockers: `(blocker_id, attacker_id)` pairs.
    DeclareBlockers(Vec<(CardId, CardId)>),
    /// Activate a loyalty ability on a planeswalker (sorcery speed, once per turn).
    ActivateLoyaltyAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    /// Cast a spell from the graveyard using its Flashback cost.
    CastFlashback { card_id: CardId, target: Option<Target>, mode: Option<usize>, x_value: Option<u32> },
    PassPriority,
    /// Submit an answer for a pending player-choice decision.
    SubmitDecision(DecisionAnswer),
}

// ── Pending decisions (suspendable resolution) ───────────────────────────────

/// A decision the engine is waiting on before it can continue resolving the
/// current spell or ability. The UI renders the `decision`, the player picks
/// an answer, and the engine resumes via `GameState::submit_decision`.
#[derive(Debug)]
pub struct PendingDecision {
    pub decision: Decision,
    pub(crate) resume: ResumeContext,
}

impl PendingDecision {
    /// The player who must answer this decision (the spell/ability controller).
    pub fn acting_player(&self) -> usize {
        match &self.resume {
            ResumeContext::Spell { caster, .. } => *caster,
            ResumeContext::Trigger { controller, .. } => *controller,
            ResumeContext::Ability { controller, .. } => *controller,
        }
    }
}

/// Internal state recording where resolution suspended so it can be picked up
/// after the decision answer arrives. Each variant mirrors the resolution
/// context (spell on the stack, triggered ability, activated ability).
#[derive(Debug)]
pub(crate) enum ResumeContext {
    Spell {
        card: Box<CardInstance>,
        caster: usize,
        target: Option<Target>,
        mode: usize,
        effects_done: usize,
        in_progress: PendingEffectState,
    },
    Trigger {
        source: CardId,
        controller: usize,
        effects: Vec<SpellEffect>,
        target: Option<Target>,
        mode: usize,
        effects_done: usize,
        in_progress: PendingEffectState,
    },
    Ability {
        source: CardId,
        controller: usize,
        effects: Vec<SpellEffect>,
        target: Option<Target>,
        effects_done: usize,
        in_progress: PendingEffectState,
    },
}

/// State snapshot for the partially-resolved effect when the engine suspends.
/// Carries the minimum info needed to finish that effect once the answer is in.
#[derive(Debug, Clone)]
pub enum PendingEffectState {
    /// Scry: top `count` cards of `player`'s library were peeked; they are still
    /// on top of the library in their original order, waiting for a reorder.
    ScryPeeked { count: usize, player: usize },
    /// Surveil: top `count` cards were peeked; player chooses which go to graveyard.
    SurveilPeeked { count: usize, player: usize },
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
    /// A scry resolved: `looked_at` top cards were examined; `bottomed` were sent
    /// to the bottom, the rest were kept on top in the decider's chosen order.
    ScryPerformed { player: usize, looked_at: usize, bottomed: usize },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    CombatResolved,
    FirstStrikeDamageResolved,
    /// Top card of `player`'s library was revealed; if `is_land` the card was drawn.
    TopCardRevealed { player: usize, card_name: &'static str, is_land: bool },
    /// A permanent was attached to another (Equipment equip, Aura ETB, etc.).
    AttachmentMoved { attachment: CardId, attached_to: Option<CardId> },
    /// A player gained poison counters.
    PoisonAdded { player: usize, amount: u32 },
    /// A loyalty ability was activated on a planeswalker.
    LoyaltyAbilityActivated { planeswalker: CardId, loyalty_change: i32 },
    /// A permanent's loyalty changed (from loyalty ability or damage).
    LoyaltyChanged { card_id: CardId, new_loyalty: i32 },
    /// A planeswalker died from 0 loyalty.
    PlaneswalkerDied { card_id: CardId },
    /// Spells were copied (Storm or copy effects).
    SpellsCopied { original: CardId, count: u32 },
    /// A surveil was performed.
    SurveilPerformed { player: usize, looked_at: usize, graveyarded: usize },
    GameOver { winner: Option<usize> },
}

// ── Priority ──────────────────────────────────────────────────────────────────

/// Tracks who currently has priority and how many consecutive passes have occurred.
#[derive(Debug, Clone)]
pub struct PriorityState {
    /// Index of the player who currently holds priority.
    pub player_with_priority: usize,
    /// Number of consecutive passes since the last stack action.
    /// When this equals the number of players, the top of the stack resolves
    /// (or the step advances if the stack is empty).
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
        /// Chosen mode index for `ChooseOne` effects (0 if `None`).
        mode: Option<usize>,
    },
    /// A triggered ability waiting to resolve (ETB, attack trigger, etc.).
    Trigger {
        source: CardId,
        controller: usize,
        effects: Vec<SpellEffect>,
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
