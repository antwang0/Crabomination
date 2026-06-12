use bevy::prelude::*;
use crabomination::card::CardId;
use std::collections::{HashMap, VecDeque};

use crate::theme;

/// One log entry with text + the color it should render in.
#[derive(Clone)]
pub struct LogEntry {
    pub text: String,
    pub color: Color,
    /// Turn-divider row — `update_log_text` renders these with extra
    /// spacing so each turn is visually separated in the scrollback.
    pub divider: bool,
    /// Asset path of the event's primary card, when one was resolvable —
    /// lets the log row preview the card on hover (`ui_card_hover`).
    pub card_art: Option<String>,
    /// The text *without* any `×N` repeat suffix, plus the current repeat
    /// count, used to coalesce a run of identical event lines (#7). Kept
    /// private — callers read `text`.
    raw: String,
    count: u32,
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

    /// Push a log entry tinted with the given color. Plain push (no
    /// coalescing) — used by non-event surfaces (menu, decision modal,
    /// export prompt, rematch banner).
    pub fn push_colored(&mut self, msg: impl Into<String>, color: Color) {
        let text = msg.into();
        self.entries.push_back(LogEntry {
            raw: text.clone(),
            text,
            color,
            divider: false,
            card_art: None,
            count: 1,
        });
        self.trim();
    }

    /// Push a per-event log line, coalescing a run of identical
    /// (text + colour) events into a single row with a `×N` multiplier
    /// instead of flooding the scrollback — token swarms, multi-hit
    /// combat, repeated pings (#7). A divider or any differing line
    /// breaks the run.
    /// Coalescing event push without a hover-preview card (tests and
    /// art-less call sites).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn push_event(&mut self, msg: impl Into<String>, color: Color) {
        self.push_event_with_art(msg, color, None);
    }

    /// Coalescing event push, plus the event's primary card-art path so
    /// the log row can preview the card on hover (`None` = plain row).
    pub fn push_event_with_art(
        &mut self,
        msg: impl Into<String>,
        color: Color,
        card_art: Option<String>,
    ) {
        let text = msg.into();
        if let Some(last) = self.entries.back_mut()
            && !last.divider
            && last.color == color
            && last.raw == text
        {
            last.count += 1;
            last.text = format!("{} ×{}", last.raw, last.count);
            return;
        }
        self.entries.push_back(LogEntry {
            raw: text.clone(),
            text,
            color,
            divider: false,
            card_art,
            count: 1,
        });
        self.trim();
    }

    /// Insert a turn-divider row (#5). Always a fresh entry — it breaks
    /// any in-progress coalescing run, so events from different turns
    /// never merge across the boundary.
    pub fn push_divider(&mut self, label: impl Into<String>) {
        let text = label.into();
        self.entries.push_back(LogEntry {
            raw: text.clone(),
            text,
            color: theme::TEXT_SECONDARY,
            divider: true,
            card_art: None,
            count: 1,
        });
        self.trim();
    }

    fn trim(&mut self) {
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
    /// Pre-chosen mode index for a modal spell (`Effect::ChooseMode`).
    /// Set by the mode-pick modal before targeting opens so the eventual
    /// `CastSpell` carries the right mode index. `None` for non-modal
    /// casts.
    pub pending_mode: Option<usize>,
    /// When `Some`, this targeting session is moving an Equipment (CR 702.6)
    /// onto a creature. The picked creature is submitted via
    /// `GameAction::Equip { equipment, target }`. Set by the `E` keybind on a
    /// controlled Equipment; cleared once the equip is submitted (or
    /// cancelled). Takes precedence over the spell/ability target paths.
    pub pending_equip_source: Option<CardId>,
    /// When `Some`, this targeting session is casting a prepared
    /// creature's prepare spell (SOS Prepare). The picked target is
    /// submitted via `GameAction::CastPrepareSpell { creature_id, .. }`.
    /// Set by the ability-menu "Cast <spell>" entry when the spell takes
    /// a target; cleared once the cast is submitted (or cancelled).
    pub pending_prepare_source: Option<CardId>,
    /// When `Some((times, mechanic))`, the pending cast pays its Squad /
    /// Replicate / Multikicker cost `times` times — the eventual submit
    /// routes through the matching `CastSpell*` action. Set by the
    /// pay-times stepper.
    pub pending_pay_times: Option<(u32, PayTimesMechanic)>,
}

/// Which pay-N-times cast mechanic the stepper is configuring.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PayTimesMechanic {
    Squad,
    Replicate,
    Multikicker,
}

impl PayTimesMechanic {
    pub fn label(self) -> &'static str {
        match self {
            Self::Squad => "Squad",
            Self::Replicate => "Replicate",
            Self::Multikicker => "Multikicker",
        }
    }
}

/// Legal targets surfaced by the engine's `Decision::ChooseTarget`.
/// Populated when the targeting cursor is satisfying a server prompt;
/// drives the legal-target highlight rings and the player-chip outline so
/// only legal seats / permanents pulse during the pick. Empty otherwise
/// (spell / ability target picks fall through to the older
/// "highlight everything clickable" path since the client doesn't know
/// the filter for those yet).
#[derive(Resource, Default)]
pub struct LegalTargets {
    pub permanents: std::collections::HashSet<CardId>,
    pub players: std::collections::HashSet<usize>,
    /// True once a legal set has actually been computed for the current
    /// pick (server `Decision::ChooseTarget` or cast-time enumeration). This
    /// distinguishes an *enumerated-but-empty* set (e.g. Beaming Defiance
    /// with no creatures you control → highlight nothing) from "not
    /// enumerated" (unknown filter → fall back to highlight-everything). Set
    /// when populated; reset when the targeting flow clears.
    pub enumerated: bool,
    /// Printed source-card name (e.g. "Ascendant Dustspeaker") shown in
    /// the hint banner.
    pub source_name: String,
    /// Short effect description (e.g. "exile target card from a
    /// graveyard") shown after the source name.
    pub description: String,
}

/// State for the activated-ability context menu (right-click on P0 battlefield card).
#[derive(Resource, Default)]
pub struct AbilityMenuState {
    pub card_id: Option<CardId>,
    pub spawn_pos: Vec2,
}

/// Hand card the viewer just clicked to cast that needs a "Choose one —"
/// mode pick before the cast can be submitted (Artistic Process, Charms,
/// the Command cycle). Cleared once a mode is picked or the user cancels.
/// When `Some`, [`spawn_mode_pick_ui`] draws the picker modal; clicking a
/// mode either casts immediately (`needs_target=false`) or arms the
/// targeting cursor with `pending_mode` set (`needs_target=true`).
#[derive(Resource, Default)]
pub struct PendingModalCast {
    pub card_id: Option<CardId>,
    pub card_name: String,
    pub modes: Vec<(String, bool)>,
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

/// Squad / Replicate / Multikicker "pay N times" stepper (CR 702.157 /
/// 702.107 / 702.33c). Set when the user right-clicks a hand card with one
/// of those mechanics; the modal steps the count and submits the matching
/// `CastSpell*` action.
#[derive(Resource, Default)]
pub struct PayTimesState {
    /// The spell being configured + which mechanic pays N times.
    pub pending: Option<(CardId, PayTimesMechanic)>,
    /// How many extra times to pay the cost (≥ 1).
    pub times: u32,
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

#[cfg(test)]
mod log_tests {
    use super::*;

    #[test]
    fn push_event_coalesces_identical_consecutive_lines() {
        let mut log = GameLog::default();
        log.push_event("Bear deals 1", Color::WHITE);
        log.push_event("Bear deals 1", Color::WHITE);
        log.push_event("Bear deals 1", Color::WHITE);
        assert_eq!(log.entries.len(), 1, "three identical events collapse to one row");
        assert_eq!(log.entries.back().unwrap().text, "Bear deals 1 ×3");
    }

    #[test]
    fn push_event_does_not_coalesce_when_text_or_colour_differs() {
        let mut log = GameLog::default();
        log.push_event("Bear deals 1", Color::WHITE);
        log.push_event("Wolf deals 1", Color::WHITE); // different text
        log.push_event("Bear deals 1", Color::BLACK); // different colour
        assert_eq!(log.entries.len(), 3);
        assert!(log.entries.iter().all(|e| !e.text.contains('×')));
    }

    #[test]
    fn divider_breaks_a_coalescing_run() {
        let mut log = GameLog::default();
        log.push_event("Bear deals 1", Color::WHITE);
        log.push_divider("── Turn 2 ──");
        log.push_event("Bear deals 1", Color::WHITE);
        // The divider sits between two separate single-count rows; the
        // post-divider event must not merge into the pre-divider one.
        assert_eq!(log.entries.len(), 3);
        assert!(log.entries[1].divider);
        assert_eq!(log.entries.back().unwrap().text, "Bear deals 1");
    }
}
