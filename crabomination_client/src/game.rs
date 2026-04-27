use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::mana::Color as ManaColor;
use std::collections::HashMap;

/// Per-frame log of human-readable events shown in the right-side overlay.
#[derive(Resource, Default)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn push(&mut self, msg: impl Into<String>) {
        self.entries.push(msg.into());
        if self.entries.len() > 16 {
            self.entries.remove(0);
        }
    }
}

/// Format a `ManaPool` as a compact string, e.g. "R:3".
pub fn format_mana_pool_from_pool(pool: &crabomination::mana::ManaPool) -> String {
    let colors = [
        (ManaColor::White, 'W'),
        (ManaColor::Blue, 'U'),
        (ManaColor::Black, 'B'),
        (ManaColor::Red, 'R'),
        (ManaColor::Green, 'G'),
    ];
    let mut parts: Vec<String> = colors
        .iter()
        .filter_map(|(c, sym)| {
            let n = pool.amount(*c);
            if n > 0 { Some(format!("{sym}:{n}")) } else { None }
        })
        .collect();
    let cl = pool.colorless_amount();
    if cl > 0 { parts.push(format!("C:{cl}")); }
    if parts.is_empty() { "0".into() } else { parts.join(" ") }
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
