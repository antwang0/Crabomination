//! Button polling and per-button visibility / label / pulse drivers.
//!
//! Two responsibilities:
//! * Latched-input polling — [`poll_action_buttons`] /
//!   [`poll_player_chip_clicks`] read `Changed<Interaction>` and write
//!   into [`ButtonState`], so the giant `handle_game_input` system can
//!   consume the results without holding a fan of button queries that
//!   would push it over Bevy's system-param limit.
//! * Per-button sync — visibility / label / colour / pulse for the
//!   attack-all, pass-priority, audit, and export buttons. These are
//!   small and frame-local; grouping them keeps the chrome-driver
//!   surface in one place.

use bevy::prelude::*;
use crabomination::game::{GameAction, TurnStep};

use crate::menu::AppState;
use crate::net_plugin::{CurrentView, NetOutbox};
use crate::theme::{self, HoverTint};

use super::{
    AttackAllButton, AttackAllPanel, AttackButtonLabel, AuditMarkVerifiedButton, AuditSkipButton,
    AutoPassButton, AutoPassButtonLabel, ButtonState, EndTurnButton, ExportStateButton,
    FastForward, LeaveButton, NextTurnButton, PassButtonLabel, PassButtonUrgent,
    PassPriorityButton, PlayerHudPanel, SurrenderButton, SurrenderButtonLabel, SurrenderConfirm,
};

/// How long (seconds) an armed Surrender stays armed waiting for the confirm
/// second click before lapsing back to its idle label.
const SURRENDER_CONFIRM_SECS: f32 = 3.0;

// ── Button polling ───────────────────────────────────────────────────────────

/// Reads all action-button `Interaction` components and writes results into
/// `ButtonState`.  Must run before `handle_game_input` in the same frame.
pub fn poll_action_buttons(
    mut state: ResMut<ButtonState>,
    pass_btn: Query<&Interaction, (Changed<Interaction>, With<PassPriorityButton>)>,
    attack_btn: Query<&Interaction, (Changed<Interaction>, With<AttackAllButton>)>,
    end_turn_btn: Query<&Interaction, (Changed<Interaction>, With<EndTurnButton>)>,
    next_turn_btn: Query<&Interaction, (Changed<Interaction>, With<NextTurnButton>)>,
    export_btn: Query<&Interaction, (Changed<Interaction>, With<ExportStateButton>)>,
) {
    state.pass = pass_btn.iter().any(|i| *i == Interaction::Pressed);
    state.attack = attack_btn.iter().any(|i| *i == Interaction::Pressed);
    state.end_turn = end_turn_btn.iter().any(|i| *i == Interaction::Pressed);
    state.next_turn = next_turn_btn.iter().any(|i| *i == Interaction::Pressed);
    state.export = export_btn.iter().any(|i| *i == Interaction::Pressed);
    // Player-chip slot is reset every frame and set only when a panel
    // actually fired a Pressed transition this frame.
    state.player_chip = None;
}

/// Sibling to `poll_action_buttons` for the per-seat player chips. Lives
/// in its own system so the click-target query can hold both
/// `Changed<Interaction>` and `PlayerHudPanel` without colliding with
/// the action-button query set.
pub fn poll_player_chip_clicks(
    mut state: ResMut<ButtonState>,
    chips: Query<(&Interaction, &PlayerHudPanel), Changed<Interaction>>,
) {
    for (interaction, panel) in &chips {
        if *interaction == Interaction::Pressed {
            state.player_chip = Some(panel.seat);
            // First press wins — subsequent panels are ignored this
            // frame so two overlapping chips can't both fire.
            return;
        }
    }
}

// ── Attack-all panel ─────────────────────────────────────────────────────────

/// Show the Attack All panel iff: it's the viewer's `DeclareAttackers`
/// step, and the viewer has at least one creature legally able to attack
/// (untapped, no summoning sickness without haste, no Defender).
pub fn update_attack_all_visibility(
    view: Res<CurrentView>,
    mut q: Query<&mut Node, With<AttackAllPanel>>,
) {
    let Ok(mut node) = q.single_mut() else { return };
    let Some(cv) = &view.0 else {
        node.display = Display::None;
        return;
    };
    if cv.game_over.is_some() {
        node.display = Display::None;
        return;
    }
    let your_seat = cv.your_seat;
    let attacking_step =
        cv.step == TurnStep::DeclareAttackers && cv.active_player == your_seat;
    if !attacking_step {
        node.display = Display::None;
        return;
    }
    use crabomination::card::Keyword;
    let has_attackers = cv.battlefield.iter().any(|c| {
        c.owner == your_seat
            && c.is_creature()
            && !c.tapped
            && (!c.summoning_sick || c.keywords.contains(&Keyword::Haste))
            && !c.keywords.contains(&Keyword::Defender)
    });
    node.display = if has_attackers { Display::Flex } else { Display::None };
}

/// Swap the attack-button label depending on whether the viewer has
/// hand-picked attackers (a non-empty [`crate::game::AttackingState::plan`]).
pub fn update_attack_button_label(
    attacking: Res<crate::game::AttackingState>,
    mut q: Query<&mut Text, With<AttackButtonLabel>>,
) {
    let Ok(mut text) = q.single_mut() else { return };
    let want = if attacking.plan.is_empty() {
        "Attack All (A)"
    } else {
        "Confirm Attack (A)"
    };
    if text.0 != want {
        text.0 = want.to_string();
    }
}

// ── Pass-priority button ─────────────────────────────────────────────────────

/// Update the Pass Priority button's background color and label to reflect
/// whether the viewer has priority and what the stack looks like.
pub fn update_pass_button(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut btn_q: Query<(Entity, &mut BackgroundColor), With<PassPriorityButton>>,
    mut label_q: Query<&mut Text, With<PassButtonLabel>>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok((btn_entity, mut bg)) = btn_q.single_mut() else { return };
    let Ok(mut label) = label_q.single_mut() else { return };

    let Some(cv) = &view.0 else { return };

    let your_priority = cv.priority == cv.your_seat;

    use crabomination::net::{StackItemKind, StackItemView};
    let top_is_opp_spell = cv.stack.last().is_some_and(|item| {
        matches!(item,
            StackItemView::Known(k)
            if k.controller != cv.your_seat && k.kind == StackItemKind::Spell
        )
    });

    // Pick the new background based on priority + stack state, then write
    // it once. Track whether the urgent-pulse marker should be present so
    // we add/remove it in a single place. Refresh `HoverTint` to match the
    // new idle colour — the button's BG is state-driven, so we maintain
    // hover affordance manually instead of attaching a static `HoverTint`
    // at spawn time (which would fight the swaps).
    let (new_bg, new_label, urgent): (Color, &str, bool) =
        if your_priority && top_is_opp_spell {
            (theme::BUTTON_URGENT_BG, "Pass / Respond (Space)", true)
        } else if your_priority && !cv.stack.is_empty() {
            (theme::BUTTON_PRIMARY_BG, "Pass (Space)", false)
        } else if your_priority {
            (theme::BUTTON_INFO_BG, "Pass (Space)", false)
        } else {
            (theme::BUTTON_DISABLED_BG, "Pass (Space)", false)
        };
    *bg = BackgroundColor(new_bg);
    label.0 = new_label.into();
    let mut entity = commands.entity(btn_entity);
    // Skip `HoverTint` for the urgent state — `pulse_urgent_pass_button`
    // is animating the BG alpha and HoverTint would clobber it on every
    // hover event.
    if urgent {
        entity.insert(PassButtonUrgent);
        entity.remove::<HoverTint>();
    } else {
        entity.remove::<PassButtonUrgent>();
        entity.insert(HoverTint::new(new_bg));
    }
}

/// Alpha-pulse the Pass button while it's in its urgent (amber) state
/// so an opponent's pending spell catches the eye. Sine wave with a
/// 1.2 s period; alpha never drops below 0.55 so the button stays
/// readable at the dim end of the cycle.
pub fn pulse_urgent_pass_button(
    time: Res<Time>,
    mut q: Query<&mut BackgroundColor, With<PassButtonUrgent>>,
) {
    let Ok(mut bg) = q.single_mut() else { return };
    let phase = (time.elapsed_secs() * std::f32::consts::TAU / 1.2).sin();
    let alpha = 0.775 + 0.225 * phase;
    let mut c = theme::BUTTON_URGENT_BG.to_srgba();
    c.alpha = alpha;
    *bg = BackgroundColor(Color::Srgba(c));
}

// ── Audit buttons ────────────────────────────────────────────────────────────

/// Show / hide the audit-mode HUD buttons based on whether the active
/// session was launched from the audit picker. `AuditTarget.0` carries
/// the card name when in audit mode.
pub fn sync_audit_buttons(
    target: Res<crate::audit::AuditTarget>,
    mut verified_q: Query<
        &mut Node,
        (
            With<AuditMarkVerifiedButton>,
            Without<AuditSkipButton>,
        ),
    >,
    mut skip_q: Query<
        &mut Node,
        (
            With<AuditSkipButton>,
            Without<AuditMarkVerifiedButton>,
        ),
    >,
) {
    if !target.is_changed() {
        return;
    }
    let visible = if target.0.is_some() { Display::Flex } else { Display::None };
    if let Ok(mut n) = verified_q.single_mut() {
        n.display = visible;
    }
    if let Ok(mut n) = skip_q.single_mut() {
        n.display = visible;
    }
}

/// Audit-mode click handlers. "Mark Verified" persists the current
/// audit target into `AuditedCards` and exits to the picker; "Skip"
/// exits without changing the set.
pub fn handle_audit_buttons(
    mut next_state: ResMut<NextState<crate::menu::AppState>>,
    mut target: ResMut<crate::audit::AuditTarget>,
    mut verified: ResMut<crate::audit::AuditedCards>,
    verified_btn: Query<&Interaction, (Changed<Interaction>, With<AuditMarkVerifiedButton>)>,
    skip_btn: Query<&Interaction, (Changed<Interaction>, With<AuditSkipButton>)>,
) {
    let pressed_verified = verified_btn.iter().any(|i| *i == Interaction::Pressed);
    let pressed_skip = skip_btn.iter().any(|i| *i == Interaction::Pressed);
    if !(pressed_verified || pressed_skip) {
        return;
    }
    if pressed_verified
        && let Some(name) = target.0.clone()
    {
        crate::audit::mark_verified(&mut verified, name);
    }
    // Either action returns to the picker; clear the target so the
    // HUD buttons hide on the next sync.
    target.0 = None;
    next_state.set(crate::menu::AppState::Audit);
}

// ── Auto-pass toggle ─────────────────────────────────────────────────────────

/// Toggle `FastForward::manual_priority` from the toolbar button or the `H`
/// key ("hold priority"). While on, `auto_advance_p0` never passes for the
/// player and every priority window is manual.
pub fn handle_auto_pass_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_console: Res<crate::systems::debug_console::DebugConsoleState>,
    chat: Res<crate::systems::chat::ChatInputState>,
    mut ff: ResMut<FastForward>,
    btn: Query<&Interaction, (Changed<Interaction>, With<AutoPassButton>)>,
    mut label_q: Query<&mut Text, With<AutoPassButtonLabel>>,
) {
    let key = !debug_console.card_input_focused
        && !chat.open
        && keyboard.just_pressed(KeyCode::KeyH);
    let pressed = btn.iter().any(|i| *i == Interaction::Pressed);
    if key || pressed {
        ff.manual_priority = !ff.manual_priority;
    }
    let want = if ff.manual_priority { "Auto-pass: Off (H)" } else { "Auto-pass: On (H)" };
    if let Ok(mut t) = label_q.single_mut()
        && t.0 != want
    {
        t.0 = want.to_string();
    }
}

// ── Export hotkey ────────────────────────────────────────────────────────────

/// Pressing `X` while in-game (and outside an active debug-console card
/// input) opens the export prompt. Also fires on the export button via
/// `ButtonState::export`.
///
/// Without this hoist, pressing `X` inside `handle_game_input` is
/// gated by a stack of early-returns (no view, no outbox, decision
/// pending, etc.) — exactly the conditions where the user is most
/// likely to want to file a bug report.
pub fn handle_export_keypress(
    keyboard: Res<ButtonInput<KeyCode>>,
    btns: Res<ButtonState>,
    mut state: ResMut<crate::systems::export_prompt::ExportPromptState>,
    debug_console: Res<crate::systems::debug_console::DebugConsoleState>,
    chat: Res<crate::systems::chat::ChatInputState>,
) {
    if state.active {
        return;
    }
    if debug_console.card_input_focused || chat.open {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyX) || btns.export {
        crate::systems::export_prompt::open_export_prompt(&mut state);
    }
}

// ── Surrender / Leave ──────────────────────────────────────────────────────────

/// Drive the two match-exit buttons:
/// * **Surrender** — concede the game (CR 104.3a). Guarded by a two-click
///   confirm: the first press arms it (and the label flips to a prompt); a
///   second press within [`SURRENDER_CONFIRM_SECS`] submits
///   [`GameAction::Concede`]. A lone first press lapses silently so it can't
///   surrender a later game.
/// * **Leave Match** — return to the main menu; the `OnExit(InGame)`
///   `teardown_net_session` drops the connection.
#[allow(clippy::too_many_arguments)]
pub fn handle_surrender_leave_buttons(
    time: Res<Time>,
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut confirm: ResMut<SurrenderConfirm>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<crate::menu::PendingNetMode>,
    surrender_btn: Query<&Interaction, (Changed<Interaction>, With<SurrenderButton>)>,
    leave_btn: Query<&Interaction, (Changed<Interaction>, With<LeaveButton>)>,
    mut label_q: Query<&mut Text, With<SurrenderButtonLabel>>,
) {
    let now = time.elapsed_secs();

    // Lapse a stale arm so a forgotten first click can't surrender later.
    if let Some(until) = confirm.armed_until
        && now > until
    {
        confirm.armed_until = None;
        set_surrender_label(&mut label_q, "Surrender");
    }

    if leave_btn.iter().any(|i| *i == Interaction::Pressed) {
        confirm.armed_until = None;
        set_surrender_label(&mut label_q, "Surrender");
        // Mirror the settings-modal "Leave Game": drop any queued mode so the
        // next menu visit starts clean. `OnExit(InGame)` disconnects.
        pending.0 = None;
        next_state.set(AppState::Menu);
        return;
    }

    if surrender_btn.iter().any(|i| *i == Interaction::Pressed) {
        // Nothing to concede once the game is already decided.
        if view.0.as_ref().is_some_and(|cv| cv.game_over.is_some()) {
            return;
        }
        match confirm.armed_until {
            None => {
                confirm.armed_until = Some(now + SURRENDER_CONFIRM_SECS);
                set_surrender_label(&mut label_q, "Confirm Surrender?");
            }
            Some(_) => {
                if let Some(outbox) = &outbox {
                    outbox.submit(GameAction::Concede);
                }
                confirm.armed_until = None;
                set_surrender_label(&mut label_q, "Surrender");
            }
        }
    }
}

/// Set the Surrender button's label text (idempotent — skips redundant
/// writes so it doesn't dirty change-detection every frame).
fn set_surrender_label(q: &mut Query<&mut Text, With<SurrenderButtonLabel>>, s: &str) {
    if let Ok(mut t) = q.single_mut()
        && t.0 != s
    {
        t.0 = s.to_string();
    }
}
