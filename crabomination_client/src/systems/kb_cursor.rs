//! Keyboard navigation cursor: lets the user select a card / player
//! zone without touching the mouse.
//!
//! Design:
//! * [`KeyboardCursor`] holds the current selection (a `GameCardId` in
//!   hand or on a battlefield, or an opponent seat for player-target).
//! * [`KeyboardSelected`] is a marker component placed on the entity
//!   matching the cursor. `sync_kb_hover_marker` mirrors it into the
//!   existing `CardHovered` marker every frame so the hover highlight
//!   and hand-card lift animation work without further changes.
//! * `handle_keyboard_cursor_input` consumes Tab / Arrow / Enter / Esc
//!   to update the cursor selection. Gameplay-level shortcut keys
//!   (Enter as "click", F flip MDFC, L alt-cost, M ability menu) are
//!   handled in `game_ui::handle_game_input` next to the existing
//!   mouse paths.
//!
//! The cursor traverses a canonical, view-derived order:
//! `Hand → Your Battlefield → Opponent Battlefield → Opponent Player Zone`.
//! Left / Right cycles within that flat list (wrapping); Tab /
//! Shift+Tab jumps to the next / previous *zone* boundary so the user
//! can move between regions quickly.

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::card::{BattlefieldCard, GameCardId, HandCard, PlayerTargetZone};
use crate::game::AltCastState;
use crate::net_plugin::CurrentView;

/// Current keyboard-cursor selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KbSelection {
    Hand(CardId),
    Battlefield(CardId),
    PlayerZone(usize),
}

#[derive(Resource, Default)]
pub struct KeyboardCursor {
    pub selection: Option<KbSelection>,
}

/// Inserted on the entity that matches `KeyboardCursor::selection`.
/// `sync_kb_hover_marker` mirrors this into `CardHovered` so the
/// existing hover-highlight + hand-lift visuals apply for free.
#[derive(Component)]
pub struct KeyboardSelected;

/// Screen-vertical row groupings. Ordered top-of-screen (index 0) →
/// bottom-of-screen so W/S can map directly to "previous / next row".
///
/// Spatial mapping (camera at +Z=14 looking at origin):
/// * `OppLands` (z=-8.5) — opponent's lands, far end of table
/// * `OppPlayer` (z=-6) — opponent target zone strip
/// * `OppCreatures` (z=-3.5)
/// * `MyCreatures` (z=+3.5)
/// * `MyLands` (z=+8.5)
/// * `MyHand` (z=+12) — closest to camera, bottom of screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KbRow {
    OppLands,
    OppPlayer,
    OppCreatures,
    MyCreatures,
    MyLands,
    MyHand,
}

/// Rows-of-selectables for the current view. Each row's `Vec` is in
/// the spatial left-to-right order matching the world-X of the
/// corresponding entities, so A/D stepping through the inner vector
/// mirrors moving the cursor left/right on screen.
fn build_rows(cv: &crabomination::net::ClientView) -> Vec<(KbRow, Vec<KbSelection>)> {
    let viewer = cv.your_seat;

    let my_hand: Vec<KbSelection> = cv
        .players
        .iter()
        .find(|p| p.seat == viewer)
        .map(|p| {
            p.hand
                .iter()
                .filter_map(|h| match h {
                    crabomination::net::HandCardView::Known(k) => {
                        Some(KbSelection::Hand(k.id))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    // Battlefield iteration order mirrors the spatial slot order used
    // by `bf_card_transform` (slot N is at offset N - center along X),
    // so iterating `cv.battlefield.iter().filter(...)` matches what
    // the player sees on screen.
    let collect_bf = |owner_match: bool, want_land: bool| -> Vec<KbSelection> {
        cv.battlefield
            .iter()
            .filter(|c| (c.owner == viewer) == owner_match && c.is_land() == want_land)
            .map(|c| KbSelection::Battlefield(c.id))
            .collect()
    };

    let my_creatures = collect_bf(true, false);
    let my_lands = collect_bf(true, true);
    let opp_creatures = collect_bf(false, false);
    let opp_lands = collect_bf(false, true);

    let opp_players: Vec<KbSelection> = cv
        .players
        .iter()
        .filter(|p| p.seat != viewer)
        .map(|p| KbSelection::PlayerZone(p.seat))
        .collect();

    let rows = [
        (KbRow::OppLands, opp_lands),
        (KbRow::OppPlayer, opp_players),
        (KbRow::OppCreatures, opp_creatures),
        (KbRow::MyCreatures, my_creatures),
        (KbRow::MyLands, my_lands),
        (KbRow::MyHand, my_hand),
    ];
    // Preserve all rows (even empty ones) so the row-index used by W/S
    // matches `ROW_ORDER`. Empty rows are skipped during navigation.
    rows.into_iter().collect()
}

/// Locate the current selection within the rows. Returns
/// `(row_idx, col_idx)`.
fn current_position(
    rows: &[(KbRow, Vec<KbSelection>)],
    cur: Option<KbSelection>,
) -> Option<(usize, usize)> {
    let cur = cur?;
    for (r, (_, list)) in rows.iter().enumerate() {
        if let Some(c) = list.iter().position(|s| *s == cur) {
            return Some((r, c));
        }
    }
    None
}

/// First non-empty row searching in `direction` (1 = forward, -1 = back)
/// starting from `from`. Returns the row's index, or `None` if no
/// row in that direction has any selectables.
fn step_row(rows: &[(KbRow, Vec<KbSelection>)], from: usize, direction: i32) -> Option<usize> {
    let n = rows.len() as i32;
    let mut i = from as i32 + direction;
    while i >= 0 && i < n {
        if !rows[i as usize].1.is_empty() {
            return Some(i as usize);
        }
        i += direction;
    }
    None
}

/// First non-empty row anywhere in the layout, used to land the
/// cursor on something sensible on the first key press.
fn first_nonempty_row(rows: &[(KbRow, Vec<KbSelection>)]) -> Option<usize> {
    rows.iter().position(|(_, list)| !list.is_empty())
}

/// Tab / Arrow / Enter / Esc handling. Other gameplay shortcuts (Enter
/// as click, F flip, L alt, M menu) live in `handle_game_input`.
#[allow(clippy::too_many_arguments)]
pub fn handle_keyboard_cursor_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    view: Res<CurrentView>,
    mut cursor: ResMut<KeyboardCursor>,
    export_prompt: Res<crate::systems::export_prompt::ExportPromptState>,
    debug_console: Res<crate::systems::debug_console::DebugConsoleState>,
    alt_cast: Res<AltCastState>,
    auto_rematch: Res<crate::systems::game_over::AutoRematchState>,
    settings: Res<crate::systems::quality::SettingsOpen>,
    esc_consumed: Res<crate::systems::quality::EscConsumed>,
) {
    // Yield input to focused text fields / modals so typing doesn't
    // move the gameplay cursor. Settings modal also captures all
    // cursor input — keyboard cursor is meaningless while the
    // pause-style settings panel is open.
    if export_prompt.active
        || debug_console.card_input_focused
        || auto_rematch.focused
        || settings.0
    {
        return;
    }
    let Some(cv) = view.0.as_ref() else { return };

    let rows = build_rows(cv);
    if rows.iter().all(|(_, l)| l.is_empty()) {
        cursor.selection = None;
        return;
    }

    // If the previous selection is no longer present, drop it.
    if let Some(sel) = cursor.selection
        && !rows.iter().any(|(_, l)| l.iter().any(|s| *s == sel))
    {
        cursor.selection = None;
    }

    // Esc clears the cursor (matches existing "Esc = cancel" feel).
    // Skipped when the settings-toggle handler already consumed the
    // same Esc press this frame, so closing the settings menu doesn't
    // also wipe the cursor selection.
    if keyboard.just_pressed(KeyCode::Escape)
        && alt_cast.pending.is_none()
        && !esc_consumed.0
    {
        cursor.selection = None;
        return;
    }

    // WASD now navigates spatially:
    //   A / ← : left within row
    //   D / → : right within row
    //   W / ↑ : previous (higher-on-screen) row
    //   S / ↓ : next (lower-on-screen) row
    //   Tab   : same as S (Shift+Tab = W)
    //
    // `A` is gated to non-DeclareAttackers steps because the gameplay
    // handler binds it to "Attack with all" during that step.
    let in_attackers = cv.step == crabomination::game::TurnStep::DeclareAttackers
        && cv.active_player == cv.your_seat;
    let left = keyboard.just_pressed(KeyCode::ArrowLeft)
        || (!in_attackers && keyboard.just_pressed(KeyCode::KeyA));
    let right = keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD);
    let down_arrow = keyboard.just_pressed(KeyCode::ArrowDown);
    let up_arrow = keyboard.just_pressed(KeyCode::ArrowUp);
    let s_key = keyboard.just_pressed(KeyCode::KeyS);
    let w_key = keyboard.just_pressed(KeyCode::KeyW);
    let tab = keyboard.just_pressed(KeyCode::Tab);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let row_down = down_arrow || s_key || (tab && !shift);
    let row_up = up_arrow || w_key || (tab && shift);

    if !(left || right || row_up || row_down) {
        return;
    }

    // If nothing selected yet, first input lands on the first item of
    // a sensible row. `MyHand` is the most common starting point;
    // fall back to the first non-empty row if the hand is empty.
    let position = current_position(&rows, cursor.selection);
    let (row_idx, col_idx) = match position {
        Some(p) => p,
        None => {
            let row = rows
                .iter()
                .position(|(r, l)| *r == KbRow::MyHand && !l.is_empty())
                .or_else(|| first_nonempty_row(&rows));
            match row {
                Some(r) => {
                    cursor.selection = Some(rows[r].1[0]);
                    return;
                }
                None => return,
            }
        }
    };

    if left || right {
        let len = rows[row_idx].1.len();
        if len == 0 {
            return;
        }
        let next = if right {
            (col_idx + 1) % len
        } else {
            (col_idx + len - 1) % len
        };
        cursor.selection = Some(rows[row_idx].1[next]);
        return;
    }

    // Row change: move up or down. Preserve the column index by
    // clamping into the new row's length so the cursor lands roughly
    // beneath/above where it was.
    let dir = if row_up { -1 } else { 1 };
    let Some(new_row) = step_row(&rows, row_idx, dir) else {
        return;
    };
    let new_len = rows[new_row].1.len();
    let new_col = col_idx.min(new_len.saturating_sub(1));
    cursor.selection = Some(rows[new_row].1[new_col]);
}

/// Place / move the `KeyboardSelected` marker each frame so it always
/// matches `KeyboardCursor::selection`. Cheap: each query is filtered
/// to a handful of entities and the write set is tiny.
pub fn apply_keyboard_selection(
    mut commands: Commands,
    cursor: Res<KeyboardCursor>,
    view: Res<CurrentView>,
    selected_q: Query<Entity, With<KeyboardSelected>>,
    hand_q: Query<(Entity, &GameCardId), With<HandCard>>,
    bf_q: Query<(Entity, &GameCardId), With<BattlefieldCard>>,
    target_q: Query<(Entity, &PlayerTargetZone)>,
) {
    // Look up the target entity first, then only touch markers that
    // actually need to change. Previously we removed `KeyboardSelected`
    // from every selected entity and re-inserted on the target every
    // frame, which fired `RemovedComponents<KeyboardSelected>` for the
    // still-active selection — `sync_kb_hover_marker` then stripped
    // `CardHovered` from the same entity each frame and the highlight
    // flashed.
    let target_entity: Option<Entity> = match cursor.selection {
        _ if view.0.is_none() => None,
        Some(KbSelection::Hand(id)) => hand_q.iter().find(|(_, g)| g.0 == id).map(|(e, _)| e),
        Some(KbSelection::Battlefield(id)) => bf_q.iter().find(|(_, g)| g.0 == id).map(|(e, _)| e),
        Some(KbSelection::PlayerZone(seat)) => target_q.iter().find(|(_, z)| z.0 == seat).map(|(e, _)| e),
        None => None,
    };

    // Drop the marker from entities that shouldn't have it any more.
    for e in &selected_q {
        if Some(e) != target_entity {
            commands.entity(e).remove::<KeyboardSelected>();
        }
    }
    // Insert on the target only when it isn't already marked, so the
    // common "cursor hasn't moved" case is a no-op.
    if let Some(e) = target_entity
        && !selected_q.contains(e)
    {
        commands.entity(e).insert(KeyboardSelected);
    }
}

/// Mirror `KeyboardSelected` → `CardHovered` so the existing hover
/// highlight + hand-card lift react to keyboard selection without
/// modification.
///
/// Two ordering subtleties:
/// * Removes are processed *before* inserts. Within a single system,
///   `Commands` apply in queueing order, so doing removes first means
///   a same-frame insert wins and the highlight survives a cursor
///   re-assertion.
/// * A `RemovedComponents<KeyboardSelected>` event is only honoured
///   when the entity is no longer in `selected_q`. Otherwise a
///   "remove + insert in same frame" cycle elsewhere (e.g. mouse-out
///   while keyboard-selected) would strip `CardHovered` even though
///   the keyboard cursor still wants it shown.
pub fn sync_kb_hover_marker(
    mut commands: Commands,
    selected_q: Query<Entity, With<KeyboardSelected>>,
    mut removed: RemovedComponents<KeyboardSelected>,
    has_hovered: Query<(), With<crate::card::CardHovered>>,
) {
    for e in removed.read() {
        if !selected_q.contains(e)
            && let Ok(mut ec) = commands.get_entity(e)
        {
            ec.try_remove::<crate::card::CardHovered>();
        }
    }
    for e in &selected_q {
        if has_hovered.get(e).is_err() {
            commands.entity(e).insert(crate::card::CardHovered);
        }
    }
}
