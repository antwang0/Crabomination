//! The in-game HUD and the table's 3-D cards: the panels and what keeps
//! them current, the sync from the server's view to the card entities, and
//! the viewer's input. Gizmos and the quality panel live in their own
//! sibling modules (`gizmos.rs`, `quality.rs`).
//!
//! - `hud` — the HUD's marker components and `setup_game_hud`
//! - `phase`, `hint`, `log`, `stack_panel`, `combat_preview` — its panels
//! - `player_stats`, `life_ticker`, `life_graph`, `table_awareness` — the
//!   seats' readouts
//! - `buttons`, `popups`, `hand_menu` — its buttons, pickers and menus
//! - `visual_sync` — card entities reconciled with the view
//! - `input`, `auto_advance` — the viewer's clicks and keys, and passing
//!   for them through steps that need nothing
//!
//! Everything re-exports from here, so callers (`main.rs`, sibling systems)
//! see one flat namespace, and each submodule sees this one's imports and
//! its siblings' items through `use super::*`.

mod auto_advance;
mod buttons;
mod combat_preview;
pub mod hand_menu;
mod hint;
mod hud;
mod input;
mod life_graph;
pub mod life_ticker;
mod log;
mod phase;
mod player_stats;
mod popups;
mod stack_panel;
pub mod table_awareness;
mod visual_sync;

pub use auto_advance::*;
pub use combat_preview::*;
pub use hint::*;
pub use hud::*;
pub use input::*;
pub use log::*;
pub use phase::*;
pub use stack_panel::*;
pub use visual_sync::*;
pub use buttons::{
    handle_audit_buttons, handle_auto_pass_toggle, handle_export_keypress,
    handle_planar_die_keypress, handle_reveal_conspiracy_keypress,
    handle_surrender_button,
    poll_action_buttons, poll_player_chip_clicks, pulse_urgent_pass_button, sync_audit_buttons,
    update_attack_all_visibility, update_attack_button_label, update_pass_button,
};
pub use hand_menu::{handle_hand_menu, spawn_hand_menu, HandMenuState};
pub use life_graph::{record_life_history, sync_life_graph, toggle_life_graph, LifeHistory};
pub use player_stats::{
    sync_player_hud_seat, update_mana_pips, update_opponent_panel_tint, update_opponent_stats_rows,
    update_player_chip_target_outline, update_player_stats_chips,
};
pub use table_awareness::TableAwarenessPlugin;
pub(crate) use popups::close_pickers;
pub use popups::{
    cancel_pickers_on_escape,
    handle_ability_menu, handle_alt_cast_buttons, handle_helper_tap_buttons,
    handle_pay_times_buttons, handle_split_cast_buttons, handle_spree_cast_buttons,
    spawn_ability_menu, spawn_alt_cast_modal, spawn_helper_tap_modal, spawn_pay_times_modal,
    spawn_spree_cast_modal, spawn_split_cast_modal, trigger_reveal_animation,
};

use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

use bevy::prelude::*;
use crabomination::card::{CardId, CardType};
use crabomination::game::{GameAction, Target, TurnStep};
use crabomination::net::StackItemView;

use super::ui::RevealPopupState;
use crate::card::{
    Animating, BattlefieldCard, CARD_THICKNESS, CardHoverLift, CardHovered,
    CardMeshAssets, CardOwner, DECK_CARD_Y_STEP, DeckPile, DrawCardAnimation,
    GameCardId, GraveyardPile, HandCard, HandSlideAnimation, OpponentHandCard,
    PlayCardAnimation, PlayerTargetZone, SendToGraveyardAnimation,
    StackCard, TapAnimation, TapState, back_face_rotation, bf_card_transform,
    card_back_face_material, card_front_material, deck_position, graveyard_position,
    creature_card_transform, hand_card_transform, back_row_card_transform, in_back_row, spawn_single_card,
};
use crate::game::{AbilityMenuState, BlockingState, GameLog, TargetingState};
use crate::net_plugin::{CurrentView, LatestServerEvents, NetOutbox};
use crate::theme::{self, HoverTint, UiFonts};

/// System set label for the ordered game-logic chain (mulligan → advance → input).
#[derive(bevy::ecs::schedule::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameLogicSet;

/// `seat`'s name as the HUD shows it.
fn player_name(cv: &crabomination::net::ClientView, seat: usize) -> String {
    cv.players
        .iter()
        .find(|p| p.seat == seat)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {seat}"))
}
