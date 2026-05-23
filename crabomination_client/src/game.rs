use bevy::prelude::*;
use crabomination::card::CardId;
use std::collections::{HashMap, VecDeque};

use crate::theme;

/// One log entry with text + the color it should render in.
#[derive(Clone)]
pub struct LogEntry {
    pub text: String,
    pub color: Color,
}

/// Maximum number of log entries kept in memory. Older entries are
/// evicted from the front (oldest-first) when this is exceeded.
pub const GAME_LOG_CAP: usize = 200;

/// Rolling log of human-readable events shown in the right-side overlay.
#[derive(Resource)]
pub struct GameLog {
    pub entries: VecDeque<LogEntry>,
}

impl Default for GameLog {
    fn default() -> Self {
        Self { entries: VecDeque::with_capacity(GAME_LOG_CAP) }
    }
}

impl GameLog {
    /// Push a plain log entry (default body color). Used by non-event
    /// surfaces — menu, decision modal, export prompt, rematch banner.
    pub fn push(&mut self, msg: impl Into<String>) {
        self.push_colored(msg, theme::TEXT_BODY);
    }

    /// Push a log entry tinted with the given color. Used by the per-event
    /// formatter so damage / mana / step / etc. entries are visually
    /// distinct in the scrollback.
    pub fn push_colored(&mut self, msg: impl Into<String>, color: Color) {
        self.entries.push_back(LogEntry { text: msg.into(), color });
        while self.entries.len() > GAME_LOG_CAP {
            self.entries.pop_front();
        }
    }
}

/// Targeting-mode UI state (when a spell/ability is waiting for the player to pick a target).
#[derive(Resource, Default)]
pub struct TargetingState {
    pub active: bool,
    /// The spell card the player is trying to cast (None when targeting for an ability).
    pub pending_card_id: Option<CardId>,
    /// When targeting for an activated ability rather than a spell.
    pub pending_ability_source: Option<CardId>,
    pub pending_ability_index: Option<usize>,
    /// When `true`, the pending target picks resolve through
    /// `GameAction::CastSpellBack` instead of `CastSpell` — used for
    /// non-land MDFCs being played via their back face. The flag is
    /// cleared once the cast is submitted (or cancelled).
    pub back_face_pending: bool,
    /// When `true`, this targeting session is satisfying a server-
    /// raised `Decision::ChooseTarget` (typically a triggered
    /// ability picking its target on the way to the stack). The
    /// picked target is submitted via
    /// `GameAction::SubmitDecision(DecisionAnswer::Target(t))` rather
    /// than `CastSpell` / `ActivateAbility`.
    pub pending_decision_target: bool,
}

/// State for the activated-ability context menu (right-click on P0 battlefield card).
#[derive(Resource, Default)]
pub struct AbilityMenuState {
    pub card_id: Option<CardId>,
    pub spawn_pos: Vec2,
}

/// State for the graveyard card browser popup.
#[derive(Resource, Default)]
pub struct GraveyardBrowserState {
    pub open: bool,
    pub owner: usize,
}

/// Active alt-cast (pitch) flow. Set when the user right-clicks a hand card
/// with an `alternative_cost`; the modal then prompts for a pitch card.
#[derive(Resource, Default)]
pub struct AltCastState {
    /// The spell being cast via alt cost (the player's hand card).
    pub pending: Option<CardId>,
}

/// Hand cards the viewer has flipped to their MDFC back face. Right-click on
/// an MDFC hand card toggles membership; left-clicks on flipped cards send
/// `PlayLandBack` instead of `PlayLand`. Cleared when a card leaves the
/// viewer's hand (handled in `sync_flipped_hand_cards`).
#[derive(Resource, Default)]
pub struct FlippedHandCards {
    pub flipped: std::collections::HashSet<CardId>,
}

/// Running card-id → name map used by the log formatter so events
/// surface human-readable names instead of opaque `CardId(N)` debug
/// strings. Populated each frame from the current `ClientView`.
#[derive(Resource, Default)]
pub struct CardNames {
    pub by_id: HashMap<CardId, String>,
}

impl CardNames {
    /// Look up a card's name; falls back to the bare ID if unknown.
    pub fn get(&self, id: CardId) -> String {
        self.by_id
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("#{}", id.0))
    }
}

/// Tracks player 0's blocker assignments during the DeclareBlockers step.
#[derive(Resource, Default)]
pub struct BlockingState {
    /// The creature the player clicked to block with.
    pub selected_blocker: Option<CardId>,
    /// Confirmed (blocker_id, attacker_id) assignments to submit on Pass.
    pub assignments: Vec<(CardId, CardId)>,
}

/// Tracks the viewer's in-progress attack plan during the DeclareAttackers
/// step. Each entry is a chosen attacker + the defender (player or
/// planeswalker) it's been pointed at. Cleared on submit, on Esc/right-
/// click, and whenever the step or active player changes.
///
/// The plan is *optional*: an empty plan + `A`/button falls back to
/// "attack all eligible at the next opponent" so the existing one-key
/// flow still works for single-opponent games.
#[derive(Resource, Default)]
pub struct AttackingState {
    pub plan: Vec<(CardId, crabomination::game::AttackTarget)>,
    /// The most-recently toggled-in attacker. Defender-zone clicks
    /// reassign this attacker's target. `None` after every confirm or
    /// reassign so two consecutive defender clicks don't both bind to
    /// the same attacker by accident.
    pub last_added: Option<CardId>,
}

impl AttackingState {
    pub fn contains(&self, id: CardId) -> bool {
        self.plan.iter().any(|(a, _)| *a == id)
    }

    pub fn remove(&mut self, id: CardId) {
        self.plan.retain(|(a, _)| *a != id);
        if self.last_added == Some(id) {
            self.last_added = None;
        }
    }

    pub fn clear(&mut self) {
        self.plan.clear();
        self.last_added = None;
    }

    /// Reassign the *most-recently toggled-in* attacker to a new target.
    /// Returns true if an attacker was reassigned. Clears `last_added`
    /// so the next defender click doesn't piggyback on the same
    /// attacker.
    pub fn set_target_for_last_added(
        &mut self,
        target: crabomination::game::AttackTarget,
    ) -> bool {
        let Some(id) = self.last_added.take() else {
            return false;
        };
        if let Some(entry) = self.plan.iter_mut().find(|(a, _)| *a == id) {
            entry.1 = target;
            return true;
        }
        false
    }
}
