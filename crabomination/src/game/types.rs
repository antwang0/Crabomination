use serde::{Deserialize, Serialize};

use crate::card::{CardId, CardInstance, CounterType};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::mana::{Color, ManaError};

/// serde default for `ImpulsePending.take` (back-compat: a snapshot without
/// the field takes one card).
pub(crate) fn one() -> usize {
    1
}

// ── Turn step sequence ────────────────────────────────────────────────────────

// `TurnStep` now lives in `crabomination_base` (below the card catalog in the
// crate graph) so that cards keying off specific steps don't pull in game
// logic. Re-exported here so existing `game::types::TurnStep` /
// `game::TurnStep` paths keep working unchanged.
pub use crabomination_base::TurnStep;

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
    /// CR 305 — play a land from the controller's graveyard, legal only while
    /// a `StaticEffect::MayPlayLandsFromGraveyard` permanent (Crucible of
    /// Worlds, Ramunap Excavator) is in play.
    PlayLandFromGraveyard(CardId),
    CastSpell { card_id: CardId, target: Option<Target>, additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// CR 702.32 — cast a spell paying its optional Kicker cost in addition
    /// to the mana cost. The resolving spell is stamped `kicked` so
    /// `Predicate::SpellWasKicked` riders fire (Tear Asunder).
    CastSpellKicked {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.103 — cast an enchantment-creature card for its Bestow cost
    /// as an Aura spell targeting a creature. It enters attached and is not
    /// a creature while bestowed. `target` must be a creature permanent.
    CastBestow {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.27 — cast a spell paying its optional Buyback cost in
    /// addition to the mana cost. The resolving spell returns to its
    /// owner's hand instead of the graveyard.
    CastSpellBuyback {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.62 — Suspend a card from hand: pay its suspend cost and exile
    /// it with N time counters. Removed one per owner's upkeep; the spell is
    /// cast for free when the last counter comes off.
    Suspend { card_id: CardId },
    /// CR 702.143 — Foretell a card from hand: pay {2} and exile it
    /// face-down to cast later for its foretell cost.
    Foretell { card_id: CardId },
    /// CR 702.143c — cast a previously foretold card from exile for its
    /// foretell cost.
    CastForetold {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 715 — cast the instant/sorcery "adventure" half of an Adventurer
    /// card from hand. On resolution the card is exiled with permission to
    /// cast the creature half later.
    CastAdventure {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 715 — cast the creature half of a card that's in exile after going
    /// on an adventure (its `on_adventure` flag is set).
    CastAdventureCreature {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 709 — cast the **right** half of a split card from hand, paying the
    /// right half's cost. Resolves the right half's effect and goes to the
    /// graveyard like any spell.
    CastSplitRight {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.102 — Fuse: cast both halves of a split card as one spell,
    /// paying both costs. `target` is the left half's target; the right
    /// half's target is `additional_targets` slot 0.
    CastSplitFused {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.127 — cast the **Aftermath** (right) half of a split card from
    /// the graveyard, paying the right half's cost. Exiled on resolution.
    CastAftermath {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.153 — cast a spell paying its optional Casualty cost,
    /// sacrificing `sacrifice` (a creature you control with power ≥ the
    /// casualty number) as an additional cost. On cast the spell is copied.
    CastSpellCasualty {
        card_id: CardId,
        sacrifice: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// CR 702.170 — Plot a card from hand: pay its plot cost and exile it
    /// face-up. Special action, main phase + empty stack only.
    Plot { card_id: CardId },
    /// CR 702.170d — cast a plotted card from exile without paying its mana
    /// cost, on a turn after it was plotted.
    CastPlotted {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
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
        #[serde(default)]
        additional_targets: Vec<Target>,
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
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: Vec<CardId>,
    },
    /// Cast a spell with `Keyword::Delve` (CR 702.66), exiling each card in
    /// `delve_cards` from the caster's graveyard to pay {1} of the spell's
    /// generic cost. Each must currently be in the caster's graveyard. The
    /// generic-only clamp matches convoke — colored and {X} pips still come
    /// from real mana.
    CastSpellDelve {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        delve_cards: Vec<CardId>,
    },
    /// Cast a spell paying its `alternative_cost` instead of its regular
    /// mana cost. `pitch_card` is the hand card (e.g., a blue card for Force
    /// of Will/Negation) being exiled to satisfy the alt cost — `None` when
    /// the alt cost has no exile requirement.
    CastSpellAlternative {
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    ActivateAbility {
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        /// X value paid to an `{X}` symbol in the activation's mana cost.
        /// Threaded through to `EffectContext.x_value` so the body can
        /// read `Value::XFromCost`. Used by Pernicious Deed's
        /// `{X}, Sacrifice this: destroy each permanent with MV ≤ X`,
        /// Walking Ballista's `{X}: this creature deals X damage`, and
        /// other X-cost activated abilities.
        #[serde(default)]
        x_value: Option<u32>,
    },
    /// Declare attackers: each attacker picks a defending player or a
    /// planeswalker controlled by a non-active player.
    DeclareAttackers(Vec<Attack>),
    DeclareBlockers(Vec<(CardId, CardId)>),
    ActivateLoyaltyAbility { card_id: CardId, ability_index: usize, target: Option<Target> },
    CastFlashback { card_id: CardId, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Cast a graveyard card with `Keyword::Retrace` (CR 702.81) for its
    /// mana cost plus discarding a land card from hand. Unlike Flashback,
    /// the spell returns to the graveyard after resolving (no exile).
    CastRetrace { card_id: CardId, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Cast a graveyard card with `Keyword::Escape` (CR 702.139) for its
    /// escape mana cost plus exiling `exile_cards` (exactly N other cards
    /// from the caster's graveyard). Instants/sorceries resolve to the
    /// graveyard (re-escapable); permanents enter the battlefield.
    CastEscape { card_id: CardId, exile_cards: Vec<CardId>, target: Option<Target>, #[serde(default)] additional_targets: Vec<Target>, mode: Option<usize>, x_value: Option<u32> },
    /// Cast a graveyard card with `Keyword::FlashbackTap(N)` by tapping
    /// `tap_creatures` (must list exactly N untapped creatures the
    /// caster controls). Used by Group Project ("Flashback—Tap three
    /// untapped creatures you control"). Mana cost is zero (the tap is
    /// the *entire* flashback cost); the resolved spell routes to exile
    /// per CR 702.34d via the standard `cast_via_flashback` plumbing.
    CastFlashbackTap {
        card_id: CardId,
        tap_creatures: Vec<CardId>,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a card from its current zone (graveyard or exile) without
    /// paying its mana cost — consumes the `CardInstance.may_play_until`
    /// permission stamped by `Effect::GrantMayPlay`. Used by Practiced
    /// Scrollsmith, Suspend Aggression, Elemental Mascot, Tablet of
    /// Discovery, Ark of Hunger, Archaic's Agony, and the Paradigm cards'
    /// upkeep recurrence. Source zone is read off the permission target's
    /// current zone (single-pass `find_card_zone`); the action rejects
    /// when no permission exists, has expired, or the card has moved to a
    /// zone where the cast no longer makes sense.
    CastFromZoneWithoutPaying {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    /// Cast a Commander from your command zone (Phase L). The mana
    /// cost is the printed cost plus `{2}` for every prior time this
    /// commander has been cast from the command zone this game
    /// (the "commander tax", CR 903.8). Targets / modes / X follow
    /// the same convention as `CastSpell`. The card is removed from
    /// the command zone, pushed onto the stack as a spell, and ends
    /// up on the battlefield (or in the appropriate zone) on
    /// resolution — and Phase J's registered replacement effect
    /// snags it back to the command zone when it would leave play.
    CastFromCommandZone {
        card_id: CardId,
        target: Option<Target>,
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    },
    PassPriority,
    SubmitDecision(DecisionAnswer),
    /// CR 702.29 — Activate a card's Cycling ability from your hand.
    /// `card_id` must be in the caster's hand and carry a
    /// `Keyword::Cycling(cost)`. Pays the cycling cost, discards the
    /// card to the controller's graveyard, draws a card. Per CR
    /// 702.29c, "When you cycle this card" triggers fire from
    /// whatever zone the card winds up in after the discard.
    Cycle { card_id: CardId },
    /// CR 702.6 — Activate an Equipment's equip ability, moving it onto
    /// `target` (a creature you control). Sorcery-speed only (CR 702.6e):
    /// legal only during the controller's main phase with an empty stack.
    /// Pays the `Keyword::Equip(cost)` mana cost, then attaches the
    /// Equipment, conferring its `equipped_bonus` via the layer system.
    Equip { equipment: CardId, target: CardId },
    /// CR 702.151 — Reconfigure an Equipment-creature. Sorcery-speed only.
    /// With `target: Some(c)`, pay the reconfigure cost and attach to a
    /// creature you control (it stops being a creature while attached).
    /// With `target: None`, pay the cost to unattach it (it becomes a
    /// creature again). Lion Sash.
    Reconfigure { equipment: CardId, target: Option<CardId> },
    /// CR 702.122 — Crew a Vehicle. Taps each creature in `crew_creatures`
    /// (each must be an untapped creature the activator controls, other than
    /// the Vehicle); their total power must meet or exceed the Vehicle's
    /// `Keyword::Crew(N)`. On success the Vehicle becomes an artifact
    /// creature until end of turn. Crew is an activated ability usable any
    /// time the controller has priority (instant speed, CR 702.122c).
    Crew { vehicle: CardId, crew_creatures: Vec<CardId> },
    /// CR 702.171 — Saddle a Mount. Taps each creature in `creatures` (each
    /// an untapped creature the activator controls, other than the Mount);
    /// their total power must meet or exceed the Mount's `Keyword::Saddle(N)`.
    /// On success the Mount becomes saddled until end of turn. Sorcery speed.
    Saddle { mount: CardId, creatures: Vec<CardId> },
    /// CR 702.49 — Ninjutsu. During the declare-blockers step, return an
    /// unblocked attacker (`returning`) to hand and put `ninja` (a card with
    /// Ninjutsu in your hand) onto the battlefield tapped and attacking the
    /// same defender, paying the ninjutsu cost.
    Ninjutsu { ninja: CardId, returning: CardId },
    /// CR 104.3a — the acting player concedes and leaves the game
    /// immediately. Carries no seat: the *server* attributes it to the
    /// connection that sent it (so a client can't concede on another seat's
    /// behalf), and a player may concede at any time regardless of priority.
    /// See `GameState::concede` and the server's `handle_action` intercept.
    Concede,
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
    /// "When [card] dies this turn, …" — fires on a `CreatureDied` event for
    /// the watched card id (CR 603.4 event-keyed delayed trigger). Registered
    /// by `Effect::WhenTargetDiesThisTurn` capturing the targeted creature.
    /// Expires at cleanup if the watched card hasn't died (the "this turn"
    /// window). Powers Searing Blood's "deals 3 to its controller".
    WhenCardDies(crate::card::CardId),
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
            ResumeContext::TriggerTargetPick { pending, .. } => pending.controller,
            ResumeContext::CleanupDiscard { player } => *player,
            ResumeContext::CombatDamage { player, .. } => *player,
        }
    }
}

/// One triggered ability waiting to be pushed onto the stack.
/// Captures the resolved per-trigger data after filter-gating (the
/// `EventSpec::filter` predicate is already evaluated when this is
/// built) so the resume path doesn't need to re-walk filters.
///
/// Holds Effect / Option / Vec — boxed everywhere it'd otherwise
/// bloat `ResumeContext` past the threshold the Clippy
/// `large_enum_variant` lint trips on, but kept inline here because
/// the existing variants in `ResumeContext` are already this size.
/// A triggered ability gathered during event dispatch, before APNAP
/// ordering (CR 603.3) and `EventSpec::filter` gating. Lives at module
/// scope so the same-controller ordering pass can take ownership of the
/// collected vector.
#[derive(Clone)]
pub(crate) struct TriggerCandidate {
    pub source: CardId,
    pub effect: Effect,
    pub controller: usize,
    pub filter: Option<crate::effect::Predicate>,
    pub subject: Option<crate::game::effects::EntityRef>,
    pub event_amount: u32,
    /// True if the originating event was an ETB (PermanentEntered). Strict
    /// Proctor's CR 614 tax only applies to ETB-triggered abilities — this
    /// flag is read at push-time to gate the tax.
    pub triggered_by_etb: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTriggerPush {
    pub source: CardId,
    pub controller: usize,
    pub effect: Effect,
    pub subject: Option<crate::game::effects::EntityRef>,
    pub event_amount: u32,
    /// Modal trigger mode (already picked at push-time). `None`
    /// when the trigger isn't modal.
    #[serde(default)]
    pub mode: Option<usize>,
    /// CR 603.4 — intervening 'if' clause. When `Some(pred)`, the
    /// resolver re-checks the predicate against the current game state
    /// before running the trigger's body; failure means the trigger
    /// fizzles. Populated by `fire_step_triggers` from the trigger's
    /// `EventSpec.filter` when that filter is intended as a resolve-
    /// time gate. Defaults to `None` for snapshot back-compat and for
    /// triggers whose filter is intended only as a trigger-time gate.
    #[serde(default)]
    pub intervening_if: Option<crate::card::Predicate>,
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
        /// Additional targets (slots 1+). Defaults to empty for snapshot
        /// back-compat with single-target spells.
        #[serde(default)]
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        /// Total mana spent paying the originating spell's cost. Threaded
        /// so a suspended spell resolution that consults
        /// `Value::CastSpellManaSpent` reads the right amount on resume.
        /// Defaults to 0 for snapshot backwards-compatibility.
        #[serde(default)]
        mana_spent: u32,
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
        /// Total mana spent paying the originating spell's cost. Same
        /// role as `x_value` / `converged_value` for
        /// `Value::CastSpellManaSpent`.
        #[serde(default)]
        mana_spent: u32,
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
    /// Suspended *before* pushing a triggered ability onto the stack
    /// because its controller has `wants_ui` and the effect requires
    /// a target. `pending` is the one trigger awaiting its target;
    /// `remaining` is the queue of triggers from the same event
    /// batch that haven't been processed yet. On answer, push
    /// `pending` with the chosen target, then continue draining
    /// `remaining` (which may suspend again on the next targeted
    /// trigger).
    TriggerTargetPick {
        pending: PendingTriggerPush,
        remaining: Vec<PendingTriggerPush>,
    },
    /// CR 514.1 — the active player's hand is over the maximum hand size
    /// at cleanup and they have `wants_ui`, so the discard-down is surfaced
    /// as an interactive `Decision::Discard`. On answer, the chosen cards
    /// are discarded; if the hand is still over the maximum (an under-
    /// discard) the decision is re-posed, otherwise the rest of cleanup and
    /// the step advance run.
    CleanupDiscard { player: usize },
    /// CR 510.1c-d — the active player (with `wants_ui`) is choosing combat
    /// damage ordering / assignment for `attacker` during the current damage
    /// step. On answer the choice is cached and the damage step is re-entered
    /// (gathering the next choice or applying damage once all are made).
    CombatDamage {
        /// The active player making the choice (always the attacking player).
        player: usize,
        attacker: CardId,
        kind: CombatDecisionKind,
    },
}

/// Which of an attacker's two combat-damage choices a `ResumeContext::
/// CombatDamage` is waiting on: the order its multiple blockers are dealt
/// damage in (CR 510.1c), or how its power is divided among them (510.1d).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CombatDecisionKind {
    Order,
    Assign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingEffectState {
    ScryPeeked { count: usize, player: usize },
    /// Look at the top `count` cards and put them back on top in any order
    /// (Index, Spire Owl). Unlike Scry, no card may be bottomed — every
    /// peeked card returns to the top in the chosen order.
    RearrangePeeked { count: usize, player: usize },
    SurveilPeeked { count: usize, player: usize },
    SearchPending { player: usize, to: crate::effect::ZoneDest },
    /// CR 701.45 — suspended on a `Decision::Learn`. The resume step reads
    /// `DecisionAnswer::Learn(choice)` and reveals a Lesson into `player`'s
    /// hand, rummages (discard-then-draw), or declines.
    LearnPending { player: usize },
    /// Suspended on a `LookPickToHand` decision (Impulse / Strategic
    /// Planning / Flow State). `revealed` is the peeked top-of-library set;
    /// the chosen card goes to hand and the rest to the bottom of the
    /// library (or graveyard if `rest_to_graveyard`).
    ImpulsePending {
        player: usize,
        revealed: Vec<CardId>,
        rest_to_graveyard: bool,
        /// Subset of `revealed` eligible to take (Satyr Wayfinder — lands
        /// only). `None` means "all revealed are eligible" (no filter /
        /// back-compat); `Some(vec)` lists the only takeable cards (which
        /// may be empty when nothing matched the filter).
        #[serde(default)]
        eligible: Option<Vec<CardId>>,
        /// Number of cards to put into hand (default 1). The decision picks
        /// the first; any remainder auto-fills from the eligible revealed set.
        #[serde(default = "crate::game::types::one")]
        take: usize,
    },
    /// Suspended on a `SearchLibrary` pick for `Effect::PayLifeLookTake`
    /// (Plunge into Darkness mode 1): the chosen card goes to hand and the
    /// rest of `revealed` are exiled.
    PayLifeLookPending {
        player: usize,
        revealed: Vec<CardId>,
    },
    PutOnLibraryPending { player: usize, count: usize },
    /// Suspended on a `ChooseColor` for an `AnyOneColor(count)` mana
    /// payload — Black Lotus, Birds of Paradise, Mox Diamond. The UI picks
    /// a color and the engine adds `count` mana of that color. When
    /// `restriction` is set (Great Hall of the Biblioplex, Resonating
    /// Lute), the added mana carries that spend restriction.
    AnyOneColorPending {
        player: usize,
        count: u32,
        restriction: Option<crate::mana::SpendRestriction>,
    },
    /// Suspended on a `ChooseColor` for a `DevotionOfChosenColor` mana
    /// payload — Nykthos, Shrine to Nyx. The player picks a color, then the
    /// engine adds mana of that color equal to their devotion to it.
    DevotionColorPending { player: usize },
    /// Suspended on a `DiscardChosen` decision (Inquisition of Kozilek,
    /// Thoughtseize). The caster picks cards from `target_player`'s hand;
    /// the apply step removes them and graveyards them.
    DiscardChosenPending { target_player: usize },
    /// Suspended on an `ExileChosenUntilSourceLeaves` decision (Brain
    /// Maggot, Tidehollow Sculler, Kitesail Freebooter). The caster picks
    /// cards from `target_player`'s hand; the apply step exiles them linked
    /// to `source`, returning them to `return_to` when `source` leaves.
    ExileChosenUntilSourceLeavesPending {
        target_player: usize,
        source: CardId,
        return_to: crate::card::ExileReturnZone,
    },
    /// Suspended on a `ChooseCreatureType` decision for `Effect::NameCreatureType`
    /// (Cavern of Souls). The chooser picks a creature type and the engine
    /// stamps it onto `target_id.chosen_creature_type`.
    ChooseCreatureTypePending { target_id: CardId },
    /// Suspended on a `NameCard` decision for `Effect::NameCard` (Pithing
    /// Needle, Phyrexian Revoker). The chooser names a card and the engine
    /// stamps it onto `target_id.named_card`.
    NameCardPending { target_id: CardId },
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

/// What a [`PreventionShield`] protects (CR 615). A shield watches for a
/// damage event aimed at this object and prevents some or all of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreventionTarget {
    Player(usize),
    Permanent(CardId),
}

/// A continuous prevention effect (CR 615.1) modelled as a "shield" around
/// a player or permanent. Created by `Effect::PreventNextDamage` /
/// `PreventAllDamageThisTurn`; consumed by the non-combat damage path and
/// cleared at cleanup (the "this turn" window, CR 514.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreventionShield {
    pub target: PreventionTarget,
    /// `None` = prevent all damage to the target this turn; `Some(n)` =
    /// prevent the next `n` damage, then the shield expires (CR 615.7).
    pub remaining: Option<u32>,
}

/// CR 731 — the game's day/night designation. The game starts as neither
/// (`GameState.day_night == None`); once it becomes day or night it has
/// exactly one of these from then on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DayNight {
    Day,
    Night,
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
    /// Some or all of a damage event was prevented by a shield (CR 615.13).
    /// `amount` is the prevented portion.
    DamagePrevented { amount: u32, to_player: Option<usize>, to_card: Option<CardId> },
    LifeLost { player: usize, amount: u32 },
    LifeGained { player: usize, amount: u32 },
    /// CR 122 — a player got `amount` energy counters ({E}).
    EnergyGained { player: usize, amount: u32 },
    /// CR 705.1 — `player` won a coin flip (Chance Encounter, Krark).
    CoinFlipWon { player: usize },
    CreatureDied { card_id: CardId },
    /// A creature was sacrificed by `who` (CR 701.16). Fires before the
    /// corresponding `CreatureDied` event so order-sensitive sacrifice
    /// triggers (Mortician Beetle, Yahenni-class) see the
    /// sacrifice-specific event first.
    CreatureSacrificed { card_id: CardId, who: usize },
    /// A permanent (any card type) was sacrificed by `who`. Broader-
    /// scope sibling of `CreatureSacrificed` — fires for every
    /// sacrifice resolution; for creature sacrifices the engine emits
    /// `CreatureSacrificed` first (preserving Mortician-style sub-
    /// trigger order) and `PermanentSacrificed` second. The two
    /// events let "whenever you sacrifice a permanent" payoffs
    /// (Korvold, Mayhem Devil, Yarok) catch Treasure / Clue / Food /
    /// land sacrifices alongside creature sacrifices, without
    /// double-firing creature-specific sub-triggers.
    PermanentSacrificed { card_id: CardId, who: usize },
    PumpApplied { card_id: CardId, power: i32, toughness: i32 },
    CounterAdded { card_id: CardId, counter_type: CounterType, count: u32 },
    CounterRemoved { card_id: CardId, counter_type: CounterType, count: u32 },
    PermanentTapped { card_id: CardId },
    PermanentUntapped { card_id: CardId },
    /// CR 701.40 — a permanent explored. `card_id` is the exploring
    /// permanent; `controller` is its controller (whose library was
    /// revealed).
    Explored { card_id: CardId, controller: usize },
    /// CR 701.31 — a permanent became monstrous.
    BecameMonstrous { card_id: CardId },
    TokenCreated { card_id: CardId },
    CardMilled { player: usize, card_id: CardId },
    ScryPerformed { player: usize, looked_at: usize, bottomed: usize },
    AttackerDeclared(CardId),
    BlockerDeclared { blocker: CardId, attacker: CardId },
    /// An attacker finished declare-blockers without any blockers
    /// assigned (CR 509.3g). Emitted at the end of `declare_blockers`
    /// once for each attacker with zero entries in `block_map`.
    AttackerWentUnblocked { attacker: CardId },
    CombatResolved,
    FirstStrikeDamageResolved,
    TopCardRevealed { player: usize, card_name: &'static str, is_land: bool },
    AttachmentMoved { attachment: CardId, attached_to: Option<CardId> },
    /// CR 702.122 — a Vehicle was crewed and became an artifact creature
    /// until end of turn.
    VehicleCrewed { vehicle: CardId },
    PoisonAdded { player: usize, amount: u32 },
    /// CR 724 — `player` became the monarch.
    MonarchChanged { player: usize },
    /// CR 700.6 — `player` got the city's blessing (Ascend).
    CityBlessingGained { player: usize },
    /// CR 731 — the game became day or night.
    DayNightChanged { day_night: DayNight },
    LoyaltyAbilityActivated { planeswalker: CardId, loyalty_change: i32 },
    LoyaltyChanged { card_id: CardId, new_loyalty: i32 },
    PlaneswalkerDied { card_id: CardId },
    SpellsCopied { original: CardId, count: u32 },
    SurveilPerformed { player: usize, looked_at: usize, graveyarded: usize },
    /// A card left `player`'s graveyard (returned to hand, battlefield, or
    /// exiled from there). Fires per card removed. Used by Strixhaven
    /// "cards leave your graveyard" payoffs.
    CardLeftGraveyard { player: usize, card_id: CardId },
    /// A permanent became the target of a spell or activated ability.
    /// `target` is the permanent being targeted; `caster` is the player
    /// who cast the spell / activated the ability. Fires per target at
    /// announce-time (immediately after the spell hits the stack or the
    /// ability is pushed). Used by SOS Tenured Concocter and any future
    /// "whenever this becomes the target of …" trigger.
    BecameTarget { target: CardId, caster: usize },
    /// CR 702.29c — `player` cycled `card_id` (paid the cycling cost
    /// and discarded the card to draw). Fires *in addition* to the
    /// `CardDiscarded` emission from the same activation, so cycle-
    /// specific triggers ("When you cycle this card", "Whenever a
    /// player cycles a card") see a distinct event from regular hand
    /// discards.
    CardCycled { player: usize, card_id: CardId },
    /// CR 104.3a — `player` conceded and left the game. Emitted before the
    /// `GameOver` that the resulting state-based-action pass produces.
    PlayerConceded { player: usize },
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
        /// Additional targets for slots 1+ (`Selector::Target(n)` with
        /// `n > 0`). `target` carries slot 0 for back-compat; multi-target
        /// spells like Crackle with Power / Snow Day populate this Vec
        /// with the rest. Defaults to empty for snapshot round-trip.
        #[serde(default)]
        additional_targets: Vec<Target>,
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
        /// Total mana spent paying this spell's cost (sum of all
        /// `ManaPool` decreases during payment). Read by
        /// `Value::CastSpellManaSpent` and `Predicate::ManaSpentAtLeast`
        /// for SOS's Increment / Opus payoffs that scale on "amount of
        /// mana spent to cast that spell". Defaults to 0 via
        /// `#[serde(default)]` for snapshot back-compat.
        #[serde(default)]
        mana_spent: u32,
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
        /// The trigger's "source" entity (the just-cast spell, the
        /// dying creature, etc.) — distinct from `source`, which
        /// always points to the ability-owner. For
        /// `EventKind::SpellCast/YourControl` triggers, this carries
        /// the cast spell's CardId so the resolving trigger can locate
        /// it (Effect::CopySpell, Selector::CastSpellTarget). For ETB
        /// triggers, it's the ETBing permanent. Defaults to `None`
        /// for snapshot back-compat; trigger resolutions then fall
        /// back to using `source` as `trigger_source`.
        #[serde(default)]
        trigger_source: Option<crate::game::effects::EntityRef>,
        /// Total mana spent paying the originating spell's cost.
        /// Threaded so spell-cast triggers consulting
        /// `Value::CastSpellManaSpent` (Increment / Opus payoffs:
        /// Cuboid Colony, Berta, Fractal Tender, Tackle Artist, …)
        /// read the right amount. Defaults to 0 for snapshot
        /// backwards-compatibility.
        #[serde(default)]
        mana_spent: u32,
        /// Per-event amount of the fired event — life gained, life lost,
        /// damage dealt, cards drawn, etc. Set by
        /// `dispatch_triggers_for_events` from the event payload (typically
        /// the event's `amount` field) so trigger bodies can read it via
        /// `Value::TriggerEventAmount`. Used by Light of Promise's
        /// "Whenever you gain life, put that many +1/+1 counters on
        /// target creature you control." Defaults to 0 for snapshot
        /// backwards-compatibility.
        #[serde(default)]
        event_amount: u32,
        /// CR 603.4 — intervening 'if' clause that must be re-checked
        /// as the trigger resolves. "When/Whenever/At [trigger event], if
        /// [condition], [effect]." When `Some(pred)`, the trigger
        /// resolver re-evaluates `pred` against the *current* game state
        /// before running the body. If the predicate now evaluates to
        /// false, the trigger does nothing (it's still removed from the
        /// stack but its `effect` is not run). Populated by
        /// `fire_step_triggers` from the trigger's `EventSpec.filter`
        /// when that filter is intended as a resolve-time gate
        /// (Triskaidekaphile-style "if you have exactly 13 cards in your
        /// hand, …" upkeep gates and similar). Defaults to `None` for
        /// snapshot backwards-compatibility and for triggers whose
        /// `event.filter` is intended as a trigger-only gate (the
        /// majority).
        #[serde(default)]
        intervening_if: Option<crate::card::Predicate>,
    },
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    #[error("It is not your priority to act")]
    NotYourPriority,
    #[error("Sorcery-speed only: stack must be empty and it must be your main phase")]
    SorcerySpeedOnly,
    #[error("You can't cast noncreature spells this turn")]
    CantCastNoncreature,
    #[error("Card {0:?} not found in hand")]
    CardNotInHand(CardId),
    #[error("Card {0:?} not found in graveyard")]
    CardNotInGraveyard(CardId),
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
    #[error("Creature {0:?} cannot attack (Defender / can't-attack restriction)")]
    CannotAttack(CardId),
    #[error("Attacker {0:?} has Menace and must be blocked by two or more creatures")]
    MenaceRequiresTwoBlockers(CardId),
    #[error("Attacker {0:?} can't be blocked by more than one creature")]
    CannotBeBlockedByMoreThanOne(CardId),
    #[error("Attacker {0:?} must be blocked if able and an idle blocker can block it")]
    MustBeBlockedIfAble(CardId),
    #[error("Card {0:?} has Hexproof and cannot be targeted by opponents")]
    TargetHasHexproof(CardId),
    #[error("Card {0:?} has Shroud and cannot be targeted")]
    TargetHasShroud(CardId),
    #[error("Card {0:?} has protection from that color/quality")]
    TargetHasProtection(CardId),
    #[error("Mana: {0}")]
    Mana(#[from] ManaError),
    /// The payer (a UI player) has more than one way to tap mana for this
    /// cost, so the engine declines to choose for them — they tap the
    /// sources they want, then re-submit. Only raised for human-initiated
    /// casts/activations; bots and engine-driven auto-pays still auto-tap.
    #[error("Tap mana to pay {cost} — you have more than one way to pay, so choose your sources")]
    ManualTapRequired { cost: String },
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
    #[error("Activated abilities of this source are shut off (Pithing Needle / Phyrexian Revoker)")]
    AbilitySuppressedByNamedCard,
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
    #[error("Flashback-tap cast invalid (missing keyword, wrong tap count, or invalid creature)")]
    FlashbackTapInvalid,
    #[error("Card {0:?} has Ward — cannot pay the ward cost")]
    TargetHasWard(CardId),
    #[error("Card {0:?} is not an Equipment you can equip")]
    NotEquipment(CardId),
    #[error("Not enough energy to pay this ability's {{E}} cost")]
    InsufficientEnergy,
    #[error("This split card has no Fuse, so its halves can't be cast together")]
    NotFusable,
}
