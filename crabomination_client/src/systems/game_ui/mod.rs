//! HUD overlay, visual sync, input handling, and ability menu. Gizmos and the
//! quality panel live in their own sibling modules (`gizmos.rs`, `quality.rs`).
//!
//! This module is being split into focused submodules; everything still
//! re-exports from here so external callers (`main.rs`, sibling systems)
//! see the same flat namespace.

mod buttons;
mod player_stats;
mod popups;

pub use buttons::{
    handle_audit_buttons, handle_auto_pass_toggle, handle_export_keypress,
    handle_surrender_leave_buttons,
    poll_action_buttons, poll_player_chip_clicks, pulse_urgent_pass_button, sync_audit_buttons,
    update_attack_all_visibility, update_attack_button_label, update_pass_button,
};
pub use player_stats::{
    animate_life_flash, sync_player_hud_seat, trigger_life_flash, update_mana_pips,
    update_opponent_panel_tint, update_opponent_stats_rows, update_player_chip_target_outline,
    update_player_stats_chips, LifeFlashTracker,
};
pub use popups::{
    handle_ability_menu, handle_alt_cast_buttons, handle_pay_times_buttons, spawn_ability_menu,
    spawn_alt_cast_modal, spawn_pay_times_modal, trigger_reveal_animation,
};

use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::{GameAction, Target, TurnStep};
use crabomination::net::StackItemView;

use super::ui::RevealPopupState;
use crate::card::{
    Animating, BattlefieldCard, CARD_THICKNESS, CardHoverLift, CardHovered,
    CardMeshAssets, CardOwner, DECK_CARD_Y_STEP, DeckCard, DeckPile, DrawCardAnimation,
    GameCardId, GraveyardPile, HandCard, HandSlideAnimation, OpponentHandCard,
    PlayCardAnimation, PlayerTargetZone, SendToGraveyardAnimation,
    StackCard, TapAnimation, TapState, ValidTarget, back_face_rotation, bf_card_transform,
    card_back_face_material, card_front_material, deck_position, graveyard_position,
    hand_card_transform, land_card_transform, spawn_single_card, stack_card_transform,
};
use crate::game::{AbilityMenuState, BlockingState, GameLog, TargetingState};
use crate::net_plugin::{CurrentView, LatestServerEvents, NetOutbox};
use crate::theme::{self, HoverTint, UiFonts};

/// System set label for the ordered game-logic chain (mulligan → advance → input).
#[derive(bevy::ecs::schedule::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameLogicSet;

/// In-flight bf→{hand, graveyard} animation queries bundled together so
/// `sync_game_visuals` stays under Bevy's 16-param tuple limit.
/// `hand_zoom` rides along here for the same reason — sync_game_visuals
/// needs the current zoom to compute hand-target transforms.
#[derive(bevy::ecs::system::SystemParam)]
pub struct InFlightAnims<'w, 's> {
    pub gy: Query<'w, 's, &'static SendToGraveyardAnimation>,
    pub to_hand: Query<'w, 's, (&'static GameCardId, &'static crate::card::ReturnToHandAnimation)>,
    pub hand_zoom: Res<'w, crate::card::HandZoom>,
    pub gameplay: Res<'w, crate::config::GameplayConfig>,
}

/// Bundled mutable resources for `handle_game_input` to stay within Bevy's 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct GameInputResources<'w> {
    pub log: ResMut<'w, GameLog>,
    pub targeting: ResMut<'w, TargetingState>,
    pub blocking: ResMut<'w, BlockingState>,
    pub attacking: ResMut<'w, crate::game::AttackingState>,
    pub reveal: ResMut<'w, RevealPopupState>,
    pub menu_state: ResMut<'w, AbilityMenuState>,
    pub ff: ResMut<'w, FastForward>,
    pub alt_cast: ResMut<'w, crate::game::AltCastState>,
    pub flipped_hand: ResMut<'w, crate::game::FlippedHandCards>,
    pub card_names: ResMut<'w, crate::game::CardNames>,
    pub export_prompt: ResMut<'w, crate::systems::export_prompt::ExportPromptState>,
    pub debug_console: ResMut<'w, crate::systems::debug_console::DebugConsoleState>,
    pub legal_targets: ResMut<'w, crate::game::LegalTargets>,
    pub modal_cast: ResMut<'w, crate::game::PendingModalCast>,
    pub pay_times: ResMut<'w, crate::game::PayTimesState>,
}
/// Process `SwapFrontMaterial` markers: walk each entity's children,
/// find the `FrontFaceMesh` child, swap its `MeshMaterial3d` to the
/// requested handle, update the parent's `CardFrontTexture` so peek /
/// hover popups stay in sync, and remove the marker. Used by the
/// hand→battlefield transition for flipped MDFCs (the played-back-face
/// image needs to land on the front-child mesh under standard bf
/// orientation).
#[allow(clippy::type_complexity)]
pub fn apply_swap_front_material(
    mut commands: Commands,
    swap_q: Query<
        (Entity, &Children, &crate::card::SwapFrontMaterial),
        With<crate::card::SwapFrontMaterial>,
    >,
    mut front_meshes: Query<
        &mut MeshMaterial3d<StandardMaterial>,
        With<crate::card::FrontFaceMesh>,
    >,
    mut tex_q: Query<&mut crate::card::CardFrontTexture>,
) {
    for (entity, children, swap) in &swap_q {
        for child in children.iter() {
            if let Ok(mut mat) = front_meshes.get_mut(child) {
                *mat = MeshMaterial3d(swap.new_front.clone());
                break;
            }
        }
        if let Ok(mut tex) = tex_q.get_mut(entity) {
            tex.0 = swap.new_path.clone();
        }
        commands
            .entity(entity)
            .remove::<crate::card::SwapFrontMaterial>();
    }
}

/// Per-variant log color. Damage / death is red, life-gain green,
/// mana / casts gold, step changes muted, combat orange, etc. Anything
/// not specifically classified falls back to body text.
fn event_color(ev: &crabomination::net::GameEventWire) -> Color {
    use crabomination::net::GameEventWire as E;
    match ev {
        E::DamageDealt { .. }
        | E::LifeLost { .. }
        | E::PoisonAdded { .. }
        | E::CreatureDied { .. }
        | E::PlaneswalkerDied { .. } => theme::TEXT_DANGER,

        // Prevented damage is a protective/beneficial outcome — colour it
        // like life-gain rather than the red damage events.
        E::LifeGained { .. } | E::DamagePrevented { .. } => theme::TEXT_GOOD,

        E::StepChanged(_) | E::TurnStarted { .. } => theme::TEXT_SECONDARY,

        // Phasing — a permanent slips out of (or back into) existence. Read
        // as a transient/secondary board event; the glyph carries the cue
        // since the permanent itself vanishes from the board view.
        E::PermanentPhasedOut { .. } => theme::TEXT_SECONDARY,

        E::CardDrawn { .. }
        | E::CardDiscarded { .. }
        | E::CardMilled { .. }
        | E::ScryPerformed { .. }
        | E::SurveilPerformed { .. }
        | E::TopCardRevealed { .. }
        | E::CardLeftGraveyard { .. } => theme::ACCENT_BLUE,

        E::ManaAdded { .. }
        | E::ColorlessManaAdded { .. }
        | E::SpellCast { .. }
        | E::AbilityActivated { .. }
        | E::LoyaltyAbilityActivated { .. }
        | E::SpellsCopied { .. } => theme::ACCENT_GOLD,

        E::CounterAdded { .. }
        | E::CounterRemoved { .. }
        | E::LoyaltyChanged { .. }
        | E::PumpApplied { .. } => theme::ACCENT_BLUE,

        E::AttackerDeclared(_)
        | E::BlockerDeclared { .. }
        | E::AttackerWentUnblocked { .. }
        | E::CombatResolved
        | E::FirstStrikeDamageResolved => theme::ACCENT_ORANGE,

        E::GameOver { .. } => theme::ACCENT_GOLD,

        // Randomization + resource gains read as gold "value" events.
        E::CoinFlipWon { .. }
        | E::CoinFlipLost { .. }
        | E::DiceRolled { .. }
        | E::EnergyGained { .. } => theme::ACCENT_GOLD,

        _ => theme::TEXT_BODY,
    }
}

/// Leading glyph for a log entry, so the red/green colour coding in
/// [`event_color`] isn't the *only* signal distinguishing harm from
/// benefit (colour-blind accessibility). `▼` = damage / life loss,
/// `✖` = a permanent died, `▲` = life gained / damage prevented. Empty
/// for everything else — the colour + text already carry those.
fn event_glyph(ev: &crabomination::net::GameEventWire) -> &'static str {
    use crabomination::net::GameEventWire as E;
    match ev {
        E::DamageDealt { .. } | E::LifeLost { .. } | E::PoisonAdded { .. } => "▼ ",
        E::CreatureDied { .. } | E::PlaneswalkerDied { .. } => "✖ ",
        E::LifeGained { .. } | E::DamagePrevented { .. } => "▲ ",
        // Coin flips and die rolls — a die glyph flags randomization outcomes.
        E::CoinFlipWon { .. } | E::CoinFlipLost { .. } | E::DiceRolled { .. } => "⚄ ",
        E::EnergyGained { .. } => "⚡ ",
        // Phasing — a hollow circle reads as "now you don't".
        E::PermanentPhasedOut { .. } => "◌ ",
        _ => "",
    }
}

/// Pretty-print a `GameEventWire` for the in-game log, resolving any
/// CardId via the running `CardNames` map (so the player sees real card
/// names instead of `CardId(N)` debug strings) and seats via the view's
/// player names (instead of `P0`/`P1`). Thin wrapper around the
/// engine-side `GameEventWire::fmt_for_log` so new event variants only
/// need a body added in one place (`crabomination/src/net.rs`).
fn format_event(
    ev: &crabomination::net::GameEventWire,
    names: &crate::game::CardNames,
    view: Option<&crabomination::net::ClientView>,
) -> String {
    ev.fmt_for_log(
        &|id| names.get(id),
        &|seat| {
            view.map(|cv| player_name(cv, seat)).unwrap_or_else(|| format!("P{seat}"))
        },
    )
}

/// Drives End Turn / Next Turn fast-forward: while these flags are set,
/// `auto_advance_p0` keeps submitting `PassPriority` each frame until the
/// target condition is reached.
#[derive(Resource, Default)]
pub struct FastForward {
    /// Pass until the active player is no longer us (hand off to bot's turn).
    pub end_turn: bool,
    /// Pass until we're back in our own PreCombatMain with priority.
    pub next_turn: bool,
    /// Pass until the game reaches this step (right-click a phase-chart
    /// row — roadmap "click-to-advance"). Cleared on arrival.
    pub pass_until: Option<TurnStep>,
    /// Auto-pass disabled (the "hold priority" toggle — H key / toolbar
    /// button): `auto_advance_p0` never passes for the player, every
    /// priority window is stepped through manually. Explicit fast-forwards
    /// (End Turn / Next Turn / click-to-advance) still override.
    pub manual_priority: bool,
}

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TurnInfoText;

/// Flex-row of per-step chips (the "phase bar"): one chip per turn step,
/// the current one highlighted. Rebuilt by `update_phase_bar` when the
/// step or active player changes.
#[derive(Component)]
pub struct PhaseBarRow;

/// Flex-row container that holds the viewer's stat chips (name, life,
/// hand, deck, graveyard). `update_player_stats_chips` rebuilds its
/// children whenever the view changes — one tinted chip per stat,
/// styled to match the mana-pip row visually.
#[derive(Component)]
pub struct PlayerStatsRow;

/// Flex-row container next to the player status text. `update_mana_pips`
/// rebuilds its children whenever the viewer's mana pool changes — one
/// small tinted chip per mana colour with the count inside.
#[derive(Component)]
pub struct ManaPipRow;

/// Flex-column container inside the top-right opponent panel that holds
/// one chip-row per opponent. `update_opponent_stats_rows` rebuilds the
/// children when the view changes.
#[derive(Component)]
pub struct OpponentStatsContainer;

/// Container for the opponent status strip. Marker so the
/// `update_opponent_panel_tint` system can flip the background between
/// the neutral HUD colour and the danger-red variant depending on the
/// viewer's threat state.
#[derive(Component)]
pub struct OpponentStatusPanel;

/// Container node (flex column) that holds one `Text` child per log
/// entry. `update_log_text` despawns and rebuilds these children when
/// `GameLog` changes.
#[derive(Component)]
pub struct GameLogPanel;

/// Outer chrome node of the game log. `position_log_below_opponents` keeps its
/// `top` just under the opponent status panel so a tall (multi-opponent,
/// wrapped) panel never overlaps the log.
#[derive(Component)]
pub struct GameLogOuterPanel;

#[derive(Component)]
pub struct HintText;

/// Wrapper around `HintText` that owns the dark-tinted padding. We
/// toggle this node's `Display` based on whether the hint string is
/// non-empty so the chip vanishes entirely between hints (instead of
/// rendering as an empty dark rectangle floating above the hand).
#[derive(Component)]
pub struct HintChip;

#[derive(Component)]
pub struct PassPriorityButton;

/// Inserted on the Pass button while an opponent's spell is on the
/// stack waiting for the viewer's response. `pulse_urgent_pass_button`
/// uses this marker to drive the amber pulse animation; removing it
/// lets the BG settle at whatever `update_pass_button` last wrote.
#[derive(Component)]
pub struct PassButtonUrgent;

#[derive(Component)]
pub struct AttackAllButton;

/// Container for the Attack All button — toggled visible only during
/// the viewer's `DeclareAttackers` step when at least one creature is
/// eligible to attack.
#[derive(Component)]
pub struct AttackAllPanel;

/// Marker on the Text inside [`AttackAllButton`] so the label can be
/// swapped between "Attack All (A)" (empty plan → fallback to "attack
/// all eligible at next opp") and "Confirm Attack (A)" (the viewer has
/// hand-picked one or more attackers via the per-creature click flow).
#[derive(Component)]
pub struct AttackButtonLabel;

/// Clickable "player avatar" — the per-seat HUD chip-row (viewer's
/// bottom-left panel, opponents' rows in the top-right strip). Holding
/// this component + a `Button` makes the whole row act as the player's
/// targeting hit-region.
#[derive(Component, Clone, Copy)]
pub struct PlayerHudPanel {
    pub seat: usize,
}

#[derive(Component)]
pub struct EndTurnButton;

/// Toolbar toggle for `FastForward::manual_priority` ("Auto-pass: On/Off").
#[derive(Component)]
pub struct AutoPassButton;

#[derive(Component)]
pub struct AutoPassButtonLabel;

#[derive(Component)]
pub struct NextTurnButton;

#[derive(Component)]
pub struct ExportStateButton;

/// "Surrender" button — concedes the match (CR 104.3a). Guarded by a
/// two-click confirm (see [`SurrenderConfirm`]) so a stray click can't throw
/// the game.
#[derive(Component)]
pub struct SurrenderButton;

/// Text label inside [`SurrenderButton`], swapped to a confirm prompt while
/// the surrender is armed.
#[derive(Component)]
pub struct SurrenderButtonLabel;

/// "Leave Match" button — abandons the match and returns to the main menu.
/// `OnExit(AppState::InGame)` (`teardown_net_session`) drops the connection.
#[derive(Component)]
pub struct LeaveButton;

/// Two-click arming state for the Surrender button. The first click arms it
/// (and the label changes to a confirm prompt) until `armed_until`; a second
/// click before then sends the concession, and the arm lapses silently
/// otherwise so a forgotten first click can't surrender later.
#[derive(Resource, Default)]
pub struct SurrenderConfirm {
    /// `Time::elapsed_secs` after which an armed confirm expires. `None` when
    /// not armed.
    pub armed_until: Option<f32>,
}

/// Audit-mode "Mark Verified" button — only `Display::Flex` when the
/// in-game session was launched from the audit picker. Adds the card
/// to `AuditedCards` and returns to the picker.
#[derive(Component)]
pub struct AuditMarkVerifiedButton;

/// Audit-mode "Skip" button — returns to the picker without changing
/// the verified set.
#[derive(Component)]
pub struct AuditSkipButton;

/// Marker for every top-level UI node spawned while in `AppState::InGame`.
/// `OnExit(AppState::InGame)` despawns all of them so a return to the
/// menu doesn't leak the HUD on top of the menu UI, and so a follow-up
/// `OnEnter(InGame)` re-runs `setup_game_hud` from scratch without
/// stacking duplicates.
#[derive(Component)]
pub struct InGameRoot;

/// Root of the dynamic stack display panel.  Children are rebuilt each time
/// the view changes.  `Node::display` is toggled between `None` / `Flex`
/// depending on whether the stack is empty.
#[derive(Component)]
pub struct StackPanel;

/// Top-center panel showing the projected per-player life swing for the
/// current attacker/blocker assignment. `update_combat_preview_panel`
/// toggles `Node::display` and rebuilds its rows; hidden outside combat.
#[derive(Component)]
pub struct CombatPreviewPanel;

/// Marker on the text label inside `PassPriorityButton` so `update_pass_button`
/// can find it without a nested child walk.
#[derive(Component)]
pub struct PassButtonLabel;

/// (Removed: `BadgeOverlay` 2-D screen-space badges — replaced by
/// 3-D coin counters on the battlefield + an Alt-key tooltip showing
/// modified P/T and counter detail. See `systems::counter_coins`.)

#[derive(Component)]
pub struct PhaseStepLabel(pub TurnStep);

/// Latched button presses collected by `poll_action_buttons` each frame,
/// consumed by `handle_game_input`.  Using a resource instead of inline
/// button queries keeps `handle_game_input` under Bevy's system-param limit.
#[derive(Resource, Default)]
pub struct ButtonState {
    pub pass: bool,
    pub attack: bool,
    pub end_turn: bool,
    pub next_turn: bool,
    pub export: bool,
    /// Seat of a `PlayerHudPanel` that was clicked this frame. `None`
    /// when no panel was clicked.
    pub player_chip: Option<usize>,
}

// ── HUD setup ─────────────────────────────────────────────────────────────────

pub fn setup_game_hud(mut commands: Commands, ui_fonts: Res<UiFonts>) {
    let tf = |size: f32| ui_fonts.tf(size);

    // Top-left: turn / step info + player status (stacked).
    // Player status used to live in a bottom-left panel that covered
    // the leftmost hand cards; consolidating it up here frees the
    // bottom of the screen for the hand and the action button strip.
    //
    // The outer panel itself is the viewer's clickable
    // [`PlayerHudPanel`] hit-region (Arena / MTGO convention — the
    // player avatar/portrait is the target). Seat is patched in by
    // `sync_player_hud_seat` once the first view arrives, since the
    // viewer seat isn't known at setup time. The border alternates
    // between transparent and yellow under `update_player_chip_target_outline`.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                min_width: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme::HUD_BG),
            BorderColor::all(Color::NONE),
            Button,
            PlayerHudPanel { seat: usize::MAX },
            InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                tf(16.0),
                TextColor(theme::TEXT_PRIMARY),
                TurnInfoText,
                Pickable::IGNORE,
            ));
            // Phase bar: one chip per step, current step highlighted.
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(2.0),
                    ..default()
                },
                PhaseBarRow,
                Pickable::IGNORE,
            ));
            // Stats row: small tinted chips for life / hand / deck / grave
            // (rebuilt by `update_player_stats_chips`) and a sibling row
            // of mana pips. Both use the same chip visual vocabulary so
            // the entire HUD strip reads as one consistent control bar.
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                    PlayerStatsRow,
                    Pickable::IGNORE,
                ));
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(3.0),
                        ..default()
                    },
                    ManaPipRow,
                    Pickable::IGNORE,
                ));
            });
        });

    // Left side: phase chart — pushed down below the (now taller)
    // turn + player panel. Each row is a Node so we can tint the
    // current step's background (not just its text). `update_phase_chart`
    // rewrites the text label with a leading "▶" / "  " marker so the
    // active step is recognisable in peripheral vision without colour.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(110.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(6.0)),
                row_gap: Val::Px(2.0),
                min_width: Val::Px(118.0),
                ..default()
            },
            BackgroundColor(theme::HUD_BG),
            InGameRoot,
        ))
        .with_children(|p| {
            for (step, _) in PHASE_CHART_STEPS {
                // `Button` so a click can cycle the step's priority stop
                // (see `phase_bar::handle_phase_chart_clicks`).
                p.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    PhaseStepLabel(*step),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("   {}", step_short_label(*step))),
                        tf(12.0),
                        TextColor(theme::TEXT_MUTED),
                        Pickable::IGNORE,
                    ));
                });
            }
        });

    // Top-right: opponent status panel. Background starts neutral and only
    // flips to the red HUD_BG_DANGER variant when the viewer is genuinely
    // threatened (see `update_opponent_panel_tint`).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                min_width: Val::Px(270.0),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(theme::HUD_BG),
            OpponentStatusPanel,
            InGameRoot,
        ))
        .with_children(|p| {
            // `update_opponent_stats_rows` rebuilds one chip-row per opponent
            // on view changes — one visible row per seat.
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                OpponentStatsContainer,
            ));
        });

    // Right side: game log, positioned just *below* the opponent panel by
    // `position_log_below_opponents` (its top tracks the panel's live height,
    // so the three wrapped Commander opponent rows can't overlap it).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(120.0),
                right: Val::Px(10.0),
                width: Val::Px(280.0),
                max_height: Val::Px(420.0),
                padding: UiRect::all(Val::Px(8.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::HUD_BG),
            GameLogOuterPanel,
            InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                GameLogPanel,
            ));
        });

    // Bottom-center: hint toast — a centered text strip just above
    // the action button row. Replaces the old gold hint line that
    // lived inside the bottom-left player panel.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(56.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            InGameRoot,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    display: Display::None,
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
                HintChip,
            ))
            .with_children(|chip| {
                chip.spawn((
                    Text::new(""),
                    tf(13.0),
                    TextColor(theme::ACCENT_GOLD),
                    HintText,
                ));
            });
        });

    // Bottom-left: action buttons in a vertical column. Stacking them
    // along the left edge (below the phase chart) keeps the entire
    // bottom-center clear for the hand fan, which is the highest-
    // traffic part of the screen during play.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            InGameRoot,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(6.0)),
                    align_items: AlignItems::Stretch,
                    min_width: Val::Px(150.0),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
            ))
            .with_children(|p| {
                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_INFO_BG),
                    Button,
                    PassPriorityButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Pass (Space)"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                        PassButtonLabel,
                    ));
                });

                // End Turn / Next Turn are both "advance the game" actions
                // — neither is an affirmative choice, so they share the
                // neutral-info blue rather than primary-green/accent-purple.
                // Reserves the strong colours for actual commitments
                // (Pass-while-spell-on-stack stays primary/urgent).
                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_INFO_BG),
                    HoverTint::new(theme::BUTTON_INFO_BG),
                    Button,
                    EndTurnButton,
                ))
                .with_children(|p| {
                    p.spawn((Text::new("End Turn (E)"), tf(13.0), TextColor(theme::TEXT_PRIMARY)));
                });

                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_INFO_BG),
                    HoverTint::new(theme::BUTTON_INFO_BG),
                    Button,
                    NextTurnButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Next Turn (N)"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });

                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    Button,
                    AutoPassButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Auto-pass: On (H)"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                        AutoPassButtonLabel,
                    ));
                });

                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    Button,
                    ExportStateButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Export State (X)"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });

                // Audit-mode buttons. Display::None by default; the
                // `sync_audit_buttons` system flips them visible
                // whenever `AuditTarget.0.is_some()`.
                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        display: Display::None,
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_PRIMARY_BG),
                    HoverTint::new(theme::BUTTON_PRIMARY_BG),
                    Button,
                    AuditMarkVerifiedButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Mark Verified"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });
                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        display: Display::None,
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    Button,
                    AuditSkipButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Skip"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });

                // Match-exit controls, kept at the bottom of the strip and
                // visually separated (danger red / neutral) so they're not
                // mistaken for in-turn actions.
                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_DANGER_BG),
                    HoverTint::new(theme::BUTTON_DANGER_BG),
                    Button,
                    SurrenderButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Surrender"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                        SurrenderButtonLabel,
                    ));
                });

                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    Button,
                    LeaveButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Leave Match"),
                        tf(13.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });
            });
        });

    // Bottom-center attack-prompt panel — only visible during the viewer's
    // own DeclareAttackers step when there's at least one creature able to
    // attack. `update_attack_all_visibility` flips Display::None / Flex.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(140.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            AttackAllPanel,
            InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_DANGER_BG),
                HoverTint::new(theme::BUTTON_DANGER_BG),
                Button,
                AttackAllButton,
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Attack All (A)"),
                    tf(15.0),
                    TextColor(theme::TEXT_PRIMARY),
                    AttackButtonLabel,
                ));
            });
        });

    // Bottom-center: stack panel (hidden when stack is empty; rebuilt each frame).
    // Outer node is full-width and transparent — it just centers its child.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(140.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(3.0),
                    min_width: Val::Px(420.0),
                    max_width: Val::Px(560.0),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
                StackPanel,
            ));
        });

    // Bottom-right: a small, always-present affordance pointing at the
    // keyboard-shortcut overlay (`toggle_shortcut_help`). The corner is
    // otherwise unused — the hand and stack sit centre, the log up-right.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(12.0),
                ..default()
            },
            Pickable::IGNORE,
            InGameRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("F1 / ?  Shortcuts"),
                tf(12.0),
                TextColor(theme::TEXT_MUTED),
                Pickable::IGNORE,
            ));
        });

    // Top-center: combat life-swing preview. Outer node is full-width and
    // just centers the inner box; `update_combat_preview_panel` toggles the
    // box's `display` and rebuilds its rows (mirrors the stack panel).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
            InGameRoot,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    row_gap: Val::Px(2.0),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
                CombatPreviewPanel,
                Pickable::IGNORE,
            ));
        });
}

// ── HUD text update ───────────────────────────────────────────────────────────

/// Rebuild the phase-bar chips when the step / active player changes.
/// Combat steps are tinted red on the active chip so the combat phase
/// reads at a glance; the viewer's own turn highlights gold.
pub fn update_phase_bar(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    row_q: Query<Entity, With<PhaseBarRow>>,
    mut last: Local<Option<(crabomination::TurnStep, usize)>>,
) {
    use crabomination::TurnStep as S;
    let Some(cv) = &view.0 else { return };
    let Ok(row) = row_q.single() else { return };
    if *last == Some((cv.step, cv.active_player)) {
        return;
    }
    *last = Some((cv.step, cv.active_player));
    const STEPS: [(S, &str); 13] = [
        (S::Untap, "UN"),
        (S::Upkeep, "UP"),
        (S::Draw, "DR"),
        (S::PreCombatMain, "M1"),
        (S::BeginCombat, "BC"),
        (S::DeclareAttackers, "AT"),
        (S::DeclareBlockers, "BL"),
        (S::FirstStrikeDamage, "FS"),
        (S::CombatDamage, "CD"),
        (S::EndCombat, "EC"),
        (S::PostCombatMain, "M2"),
        (S::End, "EN"),
        (S::Cleanup, "CL"),
    ];
    let is_combat = |s: S| {
        matches!(
            s,
            S::BeginCombat
                | S::DeclareAttackers
                | S::DeclareBlockers
                | S::FirstStrikeDamage
                | S::CombatDamage
                | S::EndCombat
        )
    };
    commands.entity(row).despawn_children();
    commands.entity(row).with_children(|p| {
        for (step, label) in STEPS {
            let current = step == cv.step;
            let (bg, fg) = if current && is_combat(step) {
                (Color::srgb(0.45, 0.15, 0.12), theme::TEXT_PRIMARY)
            } else if current {
                (Color::srgb(0.65, 0.52, 0.18), Color::srgb(0.08, 0.07, 0.03))
            } else {
                (Color::srgba(1.0, 1.0, 1.0, 0.06), Color::srgba(1.0, 1.0, 1.0, 0.45))
            };
            p.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(3.0), Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(bg),
                Pickable::IGNORE,
            ))
            .with_children(|chip| {
                chip.spawn((
                    Text::new(label),
                    ui_fonts.tf(9.0),
                    TextColor(fg),
                    Pickable::IGNORE,
                ));
            });
        }
    });
}

pub fn update_turn_text(
    view: Res<CurrentView>,
    mut q: Query<&mut Text, With<TurnInfoText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    // The dedicated centered Game Over modal owns the end-game UI; the
    // corner HUD just holds a placeholder so it isn't visually empty.
    t.0 = if cv.game_over.is_some() {
        String::new()
    } else {
        format!(
            "Turn {} | {:?} | {}'s turn",
            cv.turn,
            cv.step,
            player_name(cv, cv.active_player)
        )
    };
}

fn player_name(cv: &crabomination::net::ClientView, seat: usize) -> String {
    cv.players
        .iter()
        .find(|p| p.seat == seat)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {seat}"))
}

pub fn update_log_text(
    mut commands: Commands,
    log: Res<GameLog>,
    ui_fonts: Res<UiFonts>,
    panel_q: Query<Entity, With<GameLogPanel>>,
) {
    if !log.is_changed() {
        return;
    }
    let Ok(panel) = panel_q.single() else { return };
    commands.entity(panel).despawn_children();
    commands.entity(panel).with_children(|p| {
        for entry in log.entries.iter().rev() {
            // Turn dividers get a little breathing room above/below so
            // each turn reads as its own block in the scrollback.
            let node = if entry.divider {
                Node { margin: UiRect::vertical(Val::Px(3.0)), ..default() }
            } else {
                Node::default()
            };
            p.spawn((
                Text::new(entry.text.clone()),
                ui_fonts.tf(if entry.divider { 11.0 } else { 12.0 }),
                TextColor(entry.color),
                node,
                Pickable::IGNORE,
            ));
        }
    });
}

/// Keep the game log positioned just below the opponent status panel. The
/// panel grows with the opponent count (one chip-row each, wrapping to a
/// couple of lines in Commander), so a fixed log `top` would overlap it for
/// 3+ players. Reads the panel's live computed height (`ComputedNode::size`
/// is in logical px here) and parks the log 8px under it.
pub fn position_log_below_opponents(
    panel_q: Query<&bevy::ui::ComputedNode, With<OpponentStatusPanel>>,
    mut log_q: Query<&mut Node, With<GameLogOuterPanel>>,
) {
    let Ok(panel) = panel_q.single() else { return };
    let Ok(mut log) = log_q.single_mut() else { return };
    let panel_h = panel.size().y;
    if panel_h <= 0.0 {
        return;
    }
    // Panel sits at top:10; place the log 8px below its bottom edge.
    let target = Val::Px(10.0 + panel_h + 8.0);
    if log.top != target {
        log.top = target;
    }
}

/// Canonical phase ordering rendered by the left-edge phase chart.
/// Kept here (not inline in `setup_game_hud`) so `update_phase_chart`
/// can reuse the same ordering / label strings.
pub(crate) const PHASE_CHART_STEPS: &[(TurnStep, &str)] = &[
    (TurnStep::Untap, "Untap"),
    (TurnStep::Upkeep, "Upkeep"),
    (TurnStep::Draw, "Draw"),
    (TurnStep::PreCombatMain, "Main 1"),
    (TurnStep::BeginCombat, "Begin Combat"),
    (TurnStep::DeclareAttackers, "Attackers"),
    (TurnStep::DeclareBlockers, "Blockers"),
    (TurnStep::CombatDamage, "Damage"),
    (TurnStep::EndCombat, "End Combat"),
    (TurnStep::PostCombatMain, "Main 2"),
    (TurnStep::End, "End"),
    (TurnStep::Cleanup, "Cleanup"),
];

fn step_short_label(step: TurnStep) -> &'static str {
    PHASE_CHART_STEPS
        .iter()
        .find(|(s, _)| *s == step)
        .map(|(_, l)| *l)
        .unwrap_or("")
}

/// Subtle row background for the currently-active phase. Sits between
/// `HUD_BG` and `BUTTON_INFO_BG` so the active row reads as "lit up"
/// without competing with the action buttons.
const PHASE_ROW_ACTIVE_BG: Color = Color::srgba(0.18, 0.18, 0.28, 0.85);

pub fn update_phase_chart(
    view: Res<CurrentView>,
    stops: Option<Res<crate::systems::phase_bar::StopConfig>>,
    ff: Option<Res<FastForward>>,
    mut rows: Query<(&PhaseStepLabel, &Children, &mut BackgroundColor)>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
) {
    use crate::systems::phase_bar::StopMode;
    let Some(cv) = &view.0 else { return };
    let current = cv.step;
    let my_turn = cv.active_player == cv.your_seat;
    let pass_target = ff.as_ref().and_then(|f| f.pass_until);
    for (label, children, mut bg) in &mut rows {
        let active = label.0 == current;
        let mode = stops
            .as_ref()
            .map(|s| s.mode(my_turn, label.0))
            .unwrap_or_default();
        *bg = BackgroundColor(if active { PHASE_ROW_ACTIVE_BG } else { Color::NONE });
        // Each row has exactly one Text child — rewrite its content + colour.
        // A configured stop reads as a suffix tag scoped to the kind of turn
        // currently shown (clicking the row cycles it — see
        // `phase_bar::handle_phase_chart_clicks`).
        for child in children.iter() {
            if let Ok((mut text, mut color)) = texts.get_mut(child) {
                let marker = if active {
                    "▶ "
                } else if pass_target == Some(label.0) {
                    "⏩ "
                } else {
                    "   "
                };
                let stop_tag = match mode {
                    StopMode::Auto => "",
                    StopMode::Always => "  [stop]",
                    StopMode::Skip => "  [skip]",
                };
                text.0 = format!("{marker}{}{stop_tag}", step_short_label(label.0));
                *color = TextColor(match (active, mode) {
                    (true, _) => theme::ACCENT_YELLOW,
                    (false, StopMode::Always) => theme::ACCENT_ORANGE,
                    (false, StopMode::Skip) => theme::TEXT_MUTED.with_alpha(0.5),
                    (false, StopMode::Auto) => theme::TEXT_MUTED,
                });
            }
        }
    }
}

// ── Phase banner ──────────────────────────────────────────────────────────────

/// Remembers the last turn/step the banner system reacted to, so it only
/// flashes a banner on a genuine transition (not every view refresh).
#[derive(Resource, Default)]
pub struct PhaseBannerTracker {
    last_turn: u32,
    last_step: Option<TurnStep>,
    /// `false` until the first view is observed — priming the tracker on
    /// that first view avoids flashing a banner for whatever state the
    /// client happens to connect into (e.g. mid-mulligan).
    primed: bool,
}

/// A transient, fading center-screen banner ("Your Turn", "Combat", …).
/// `animate_phase_banner` counts `remaining` down and fades the text,
/// despawning at zero.
#[derive(Component)]
pub struct PhaseBanner {
    remaining: f32,
    total: f32,
}

const PHASE_BANNER_SECS: f32 = 1.5;
/// Peak opacity of the dark chip behind a phase banner. Faded in step with the
/// text by `animate_phase_banner`, so the chip never lingers after the text.
const PHASE_BANNER_BG_ALPHA: f32 = 0.66;

/// Flash a banner on milestone transitions: the start of a turn ("Your
/// Turn" / "<name>'s Turn") and entry into combat ("Combat"). Deliberately
/// not every step — a banner per step would be noise.
pub fn trigger_phase_banner(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut tracker: ResMut<PhaseBannerTracker>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<PhaseBanner>>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() {
        return;
    }

    let prev_step = tracker.last_step;
    let prev_turn = tracker.last_turn;
    let was_primed = tracker.primed;
    tracker.last_turn = cv.turn;
    tracker.last_step = Some(cv.step);
    tracker.primed = true;
    if !was_primed {
        return;
    }

    let banner: Option<(String, Color)> = if cv.turn != prev_turn {
        Some(if cv.active_player == cv.your_seat {
            ("Your Turn".to_string(), theme::ACCENT_GOLD)
        } else {
            (format!("{}'s Turn", player_name(cv, cv.active_player)), theme::TEXT_DANGER)
        })
    } else if prev_step != Some(cv.step) && cv.step == TurnStep::DeclareAttackers {
        Some(("Combat".to_string(), theme::ACCENT_ORANGE))
    } else {
        None
    };

    let Some((text, color)) = banner else { return };
    // Replace any in-flight banner so rapid transitions don't stack.
    for e in &existing {
        commands.entity(e).despawn();
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
            InGameRoot,
            PhaseBanner { remaining: PHASE_BANNER_SECS, total: PHASE_BANNER_SECS },
        ))
        .with_children(|p| {
            p.spawn((
                // Padded, semi-transparent chip behind the text so the big
                // coloured banner reads against the busy 3-D board. Its alpha
                // is faded in step with the text by `animate_phase_banner`.
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.04, 0.07, PHASE_BANNER_BG_ALPHA)),
                Text::new(text),
                ui_fonts.tf(46.0),
                TextColor(color),
                Pickable::IGNORE,
            ));
        });
}

/// Count the banner down and fade its text (and chip background) out over the
/// last ~40% of its life; despawn when elapsed.
pub fn animate_phase_banner(
    mut commands: Commands,
    time: Res<Time>,
    mut banners: Query<(Entity, &mut PhaseBanner, &Children)>,
    mut texts: Query<&mut TextColor>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    for (entity, mut banner, children) in &mut banners {
        banner.remaining -= time.delta_secs();
        if banner.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = (banner.remaining / banner.total).clamp(0.0, 1.0);
        // Hold full opacity, then ease out over the final 40%.
        let alpha = (frac / 0.4).min(1.0);
        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                let c = tc.0.to_srgba();
                tc.0 = Color::srgba(c.red, c.green, c.blue, alpha);
            }
            // The chip background and text live on the same child entity;
            // scale its alpha off the same curve (from the fixed peak so the
            // per-frame rewrite stays idempotent).
            if let Ok(mut bg) = backgrounds.get_mut(child) {
                let c = bg.0.to_srgba();
                bg.0 = Color::srgba(c.red, c.green, c.blue, PHASE_BANNER_BG_ALPHA * alpha);
            }
        }
    }
}

pub fn update_hint(
    view: Res<CurrentView>,
    targeting: Res<TargetingState>,
    legal_targets: Res<crate::game::LegalTargets>,
    blocking: Res<BlockingState>,
    time: Res<Time>,
    mut q: Query<(&mut Text, &mut TextColor, &mut TextFont), With<HintText>>,
) {
    let Ok((mut t, mut color, mut font)) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() {
        apply_hint(&mut t, &mut color, &mut font, String::new(), theme::ACCENT_GOLD, 13.0);
        return;
    }
    if targeting.active {
        let (msg, hint_color, hint_size) = if targeting.pending_decision_target {
            let src = if legal_targets.source_name.is_empty() {
                "A triggered ability".to_string()
            } else {
                legal_targets.source_name.clone()
            };
            let body = if legal_targets.description.is_empty() {
                "needs a target".to_string()
            } else {
                legal_targets.description.clone()
            };
            (
                format!("⚡ {src}: {body}. Click / Enter on a highlighted target."),
                theme::ACCENT_BLUE,
                15.0_f32,
            )
        } else {
            (
                "Click / Enter on a target. Tab,← → select. Esc cancels.".to_string(),
                theme::ACCENT_GOLD,
                13.0_f32,
            )
        };
        apply_hint(&mut t, &mut color, &mut font, msg, hint_color, hint_size);
        return;
    }
    if !cv.stack.is_empty() {
        let your_priority = cv.priority == cv.your_seat;
        use crabomination::net::StackItemKind;
        // Style the hint based on the top-of-stack kind so triggered
        // abilities visually stand out from spells (and from idle
        // status messages). Triggers are the most "surprising" event —
        // they can fire without the player initiating anything — so
        // they get the boldest treatment.
        let (style_text, style_color, style_size) = if your_priority {
            match cv.stack.last() {
                Some(StackItemView::Known(k)) => {
                    let ctrl = if k.controller == cv.your_seat {
                        "Your".to_string()
                    } else {
                        format!("{}'s", player_name(cv, k.controller))
                    };
                    let tgt = match &k.target {
                        Some(Target::Player(s)) => format!(
                            " targeting {}",
                            if *s == cv.your_seat { "you".into() } else { player_name(cv, *s) }
                        ),
                        Some(Target::Permanent(id)) => cv
                            .battlefield
                            .iter()
                            .find(|p| p.id == *id)
                            .map(|p| format!(" targeting {}", p.name))
                            .unwrap_or_default(),
                        None => String::new(),
                    };
                    match k.kind {
                        StackItemKind::Trigger => (
                            format!(
                                "⚡ TRIGGER: {} {}{}. Respond from hand, or Space to let it resolve.",
                                ctrl, k.name, tgt
                            ),
                            theme::ACCENT_BLUE,
                            16.0_f32,
                        ),
                        StackItemKind::Spell => (
                            format!(
                                "↻ {} {} on stack{}. Respond from hand, or Space to let it resolve.",
                                ctrl, k.name, tgt
                            ),
                            theme::ACCENT_ORANGE,
                            14.0_f32,
                        ),
                    }
                }
                _ => (
                    format!("{} item(s) on stack. Space = let it resolve.", cv.stack.len()),
                    theme::ACCENT_GOLD,
                    13.0_f32,
                ),
            }
        } else {
            (
                format!("Waiting for {} to act on the stack.", player_name(cv, cv.priority)),
                theme::TEXT_SECONDARY,
                13.0_f32,
            )
        };
        apply_hint(&mut t, &mut color, &mut font, style_text, style_color, style_size);
        return;
    }
    let your_seat = cv.your_seat;
    let viewer_is_defending =
        cv.step == TurnStep::DeclareBlockers && cv.active_player != your_seat && cv.priority == your_seat;
    if viewer_is_defending {
        let msg = if blocking.selected_blocker.is_some() {
            "Click / Enter on an attacker to assign the block. Esc cancels.".to_string()
        } else {
            "Click / Enter on a creature to block with it. Tab,← → select. P skip.".to_string()
        };
        apply_hint(&mut t, &mut color, &mut font, msg, theme::ACCENT_GOLD, 13.0);
        return;
    }
    let body = match (cv.active_player == your_seat, cv.step) {
        (true, TurnStep::PreCombatMain) | (true, TurnStep::PostCombatMain) => {
            "Click / Enter to play. Tab,← →: select. F flip · L alt · M ability. P pass.".to_string()
        }
        (true, TurnStep::DeclareAttackers) => {
            "A = Attack with all eligible creatures. P = Pass (no attack).".to_string()
        }
        (true, TurnStep::DeclareBlockers) => {
            "Opponent is assigning blocks. P = Proceed to combat.".to_string()
        }
        (true, _) => String::new(),
        (false, _) => {
            // Cycle 1→2→3 dots every ~0.4s so the player can tell the
            // bot is still working and the game hasn't hung. Stick on a
            // visible dot count (never zero) so the line is the same
            // width every frame.
            let phase = (time.elapsed_secs() / 0.4) as u64 % 3;
            let dots = match phase {
                0 => ".  ",
                1 => ".. ",
                _ => "...",
            };
            format!("{} is thinking{}", player_name(cv, cv.active_player), dots)
        }
    };
    apply_hint(&mut t, &mut color, &mut font, body, theme::ACCENT_GOLD, 13.0);
}

/// Apply `(text, colour, size)` to the hint chip, skipping the write
/// when the field already matches. Avoids spurious change-detection
/// fanout from per-frame writes (this system runs every frame).
fn apply_hint(
    text: &mut Text,
    color: &mut TextColor,
    font: &mut TextFont,
    new_text: String,
    new_color: Color,
    new_size: f32,
) {
    if text.0 != new_text {
        text.0 = new_text;
    }
    if color.0 != new_color {
        *color = TextColor(new_color);
    }
    if (font.font_size - new_size).abs() > f32::EPSILON {
        font.font_size = new_size;
    }
}

/// Sync the dark-tinted hint chip's visibility to the current
/// `HintText` content. Lives in its own system so every early-return
/// path in `update_hint` (game-over, targeting, stack, blocking, etc.)
/// is covered without per-branch ceremony.
pub fn sync_hint_chip_visibility(
    text_q: Query<&Text, With<HintText>>,
    mut chip_q: Query<&mut Node, With<HintChip>>,
) {
    let Ok(text) = text_q.single() else { return };
    let Ok(mut chip) = chip_q.single_mut() else { return };
    let target = if text.0.is_empty() { Display::None } else { Display::Flex };
    if chip.display != target {
        chip.display = target;
    }
}

// ── Stack panel ───────────────────────────────────────────────────────────────

/// Resolve a `Target` to a display string given the current view.
fn target_display(cv: &crabomination::net::ClientView, tgt: &Target) -> String {
    match tgt {
        Target::Player(s) => {
            if *s == cv.your_seat { "you".into() } else { player_name(cv, *s) }
        }
        Target::Permanent(id) => cv
            .battlefield
            .iter()
            .find(|p| p.id == *id)
            .map(|p| p.name.clone())
            .or_else(|| {
                // Check graveyards (e.g. Goryo's Vengeance).
                cv.players.iter().flat_map(|p| &p.graveyard)
                    .find(|g| g.id == *id)
                    .map(|g| g.name.clone())
            })
            .unwrap_or_else(|| "target".into()),
    }
}

/// Rebuild the `StackPanel` children whenever the view changes.
/// Stack is LIFO: the last element resolves next.  We show top-of-stack first.
#[allow(clippy::too_many_arguments)]
pub fn update_stack_panel(
    view: Res<CurrentView>,
    mut commands: Commands,
    mut panel_q: Query<(Entity, &mut Node), With<StackPanel>>,
    ui_fonts: Res<UiFonts>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok((panel_entity, mut node)) = panel_q.single_mut() else { return };

    // Always wipe and rebuild — stack changes are infrequent.
    commands.entity(panel_entity).despawn_children();

    let Some(cv) = &view.0 else {
        node.display = Display::None;
        return;
    };

    if cv.stack.is_empty() {
        node.display = Display::None;
        return;
    }

    node.display = Display::Flex;

    let tf = |size: f32| ui_fonts.tf(size);

    let your_priority = cv.priority == cv.your_seat;
    let priority_name = if your_priority {
        "You".to_string()
    } else {
        player_name(cv, cv.priority)
    };
    let header = format!(
        "Stack  {}  |  {} has priority",
        if cv.stack.len() == 1 { "1 item".to_string() } else { format!("{} items", cv.stack.len()) },
        priority_name,
    );

    commands.entity(panel_entity).with_children(|p| {
        // Header row
        p.spawn((
            Text::new(header),
            tf(12.0),
            TextColor(theme::TEXT_BODY),
        ));
        // Divider
        p.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        });

        // Items: top of stack (last index) shown first — that's what resolves next.
        // Triggers get a stronger badge (higher alpha + larger font) so a
        // surprise trigger doesn't blend into a row of spells.
        use crabomination::net::{StackItemKind, StackItemView};
        for (offset, item) in cv.stack.iter().rev().enumerate() {
            // Resolution-order index: top of stack is "1" (resolves next).
            let resolves_at = offset + 1;
            let (kind_str, kind_color, name, ctrl_str, tgt_str, is_trigger) = match item {
                StackItemView::Known(k) => {
                    let (kstr, kcol, is_trig) = match k.kind {
                        StackItemKind::Spell   => ("SPELL",   theme::ACCENT_ORANGE, false),
                        StackItemKind::Trigger => ("⚡ TRIGGER", theme::ACCENT_BLUE, true),
                    };
                    let ctrl = if k.controller == cv.your_seat {
                        "You".to_string()
                    } else {
                        player_name(cv, k.controller)
                    };
                    // Every chosen target — primary slot plus the
                    // additional-target slots of multi-target spells
                    // (Divide-damage, Support, fused splits).
                    let all_targets: Vec<String> = k
                        .target
                        .iter()
                        .chain(k.additional_targets.iter())
                        .map(|t| target_display(cv, t))
                        .collect();
                    let tgt = if all_targets.is_empty() {
                        String::new()
                    } else {
                        format!("  →  {}", all_targets.join(", "))
                    };
                    (kstr, kcol, k.name.clone(), ctrl, tgt, is_trig)
                }
                StackItemView::Hidden { controller, .. } => {
                    let ctrl = if *controller == cv.your_seat {
                        "You".to_string()
                    } else {
                        player_name(cv, *controller)
                    };
                    ("?", theme::TEXT_MUTED, "Hidden card".to_string(), ctrl, String::new(), false)
                }
            };
            let badge_alpha = if is_trigger { 0.45 } else { 0.22 };
            let badge_font = if is_trigger { 12.0 } else { 10.0 };
            let name_color = if is_trigger { theme::ACCENT_BLUE } else { theme::TEXT_PRIMARY };

            // Row — triggers get a faint tinted background so they pop
            // even at a glance.
            let row_bg = if is_trigger {
                Color::srgba(
                    kind_color.to_srgba().red,
                    kind_color.to_srgba().green,
                    kind_color.to_srgba().blue,
                    0.10,
                )
            } else {
                Color::NONE
            };
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(row_bg),
            ))
            .with_children(|row| {
                // Resolution-order index — "1" resolves next.
                row.spawn((
                    Node {
                        min_width: Val::Px(16.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|idx| {
                    idx.spawn((
                        Text::new(format!("{resolves_at}.")),
                        tf(11.0),
                        TextColor(theme::TEXT_MUTED),
                    ));
                });

                // Kind badge
                row.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(
                        kind_color.to_srgba().red,
                        kind_color.to_srgba().green,
                        kind_color.to_srgba().blue,
                        badge_alpha,
                    )),
                ))
                .with_children(|badge| {
                    badge.spawn((Text::new(kind_str), tf(badge_font), TextColor(kind_color)));
                });

                // Name  ·  controller  →  target
                let label = format!("{}  ·  {}{}", name, ctrl_str, tgt_str);
                let label_size = if is_trigger { 13.0 } else { 12.0 };
                row.spawn((Text::new(label), tf(label_size), TextColor(name_color)));
            });
        }
    });
}

// ── Combat preview ────────────────────────────────────────────────────────────

/// Rebuild the top-center combat-preview panel from `combat_preview`,
/// showing each player's projected life swing for the *currently declared*
/// attackers/blocks: `<who>: <life> → <new>` (red when dropping, green when
/// gaining), with a `(+N lifelink)` note where lifelink applies. Hidden
/// outside combat or when no player's life actually changes. Surfaces the
/// damage AND lifelink figures the engine already projects — the "who
/// dies" half is the red dying-creature borders (`update_dying_highlights`).
pub fn update_combat_preview_panel(
    view: Res<CurrentView>,
    mut commands: Commands,
    mut panel_q: Query<(Entity, &mut Node), With<CombatPreviewPanel>>,
    ui_fonts: Res<UiFonts>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok((panel, mut node)) = panel_q.single_mut() else { return };
    commands.entity(panel).despawn_children();

    let hide = |node: &mut Node| node.display = Display::None;
    let Some(cv) = &view.0 else { hide(&mut node); return };
    let Some(cp) = &cv.combat_preview else { hide(&mut node); return };

    let lookup = |table: &[(usize, i32)], seat: usize| {
        table.iter().find(|(s, _)| *s == seat).map(|(_, v)| *v).unwrap_or(0)
    };

    // One row per player whose life actually changes, ordered by seat.
    let mut seats: Vec<usize> = cv.players.iter().map(|p| p.seat).collect();
    seats.sort_unstable();
    let mut rows: Vec<(String, Color)> = Vec::new();
    for seat in seats {
        let Some(p) = cv.players.iter().find(|p| p.seat == seat) else { continue };
        let dmg = lookup(&cp.damage_to_players, seat);
        let gain = lookup(&cp.lifegain_to_players, seat);
        if dmg == 0 && gain == 0 {
            continue;
        }
        let new_life = p.life - dmg + gain;
        let who = if seat == cv.your_seat { "You".to_string() } else { player_name(cv, seat) };
        let note = if gain > 0 { format!("  (+{gain} lifelink)") } else { String::new() };
        let color = if new_life < p.life {
            theme::TEXT_DANGER
        } else if new_life > p.life {
            theme::TEXT_GOOD
        } else {
            theme::TEXT_BODY
        };
        rows.push((format!("{who}:  {} → {}{}", p.life, new_life, note), color));
    }

    // One row per attacked planeswalker projected to lose loyalty.
    for (pw, dmg) in &cp.damage_to_planeswalkers {
        let Some(p) = cv.battlefield.iter().find(|c| c.id == *pw) else { continue };
        let loyalty = p
            .counters
            .iter()
            .find(|(k, _)| matches!(k, crabomination::card::CounterType::Loyalty))
            .map(|(_, n)| *n as i32)
            .unwrap_or(0);
        let new_loyalty = (loyalty - dmg).max(0);
        let color = if p.controller == cv.your_seat { theme::TEXT_DANGER } else { theme::TEXT_GOOD };
        rows.push((format!("{}:  {} → {}", p.name, loyalty, new_loyalty), color));
    }

    // Numeric "who dies" summary to complement the red dying-creature borders:
    // split the projected casualties by controller so the player sees the
    // trade at a glance (yours in danger-red, opponents' in good-green).
    let (mut yours, mut theirs) = (0u32, 0u32);
    for id in &cp.dying_creatures {
        match cv.battlefield.iter().find(|c| c.id == *id) {
            Some(c) if c.controller == cv.your_seat => yours += 1,
            Some(_) => theirs += 1,
            None => {}
        }
    }
    if yours > 0 || theirs > 0 {
        let mut parts = Vec::new();
        if theirs > 0 { parts.push(format!("{theirs} theirs")); }
        if yours > 0 { parts.push(format!("{yours} yours")); }
        let color = if yours > theirs { theme::TEXT_DANGER } else { theme::TEXT_GOOD };
        rows.push((format!("Dying:  {}", parts.join(", ")), color));
    }

    if rows.is_empty() {
        hide(&mut node);
        return;
    }
    node.display = Display::Flex;
    let tf = |s: f32| ui_fonts.tf(s);
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new("Combat"), tf(11.0), TextColor(theme::ACCENT_ORANGE), Pickable::IGNORE));
        for (text, color) in rows {
            p.spawn((Text::new(text), tf(14.0), TextColor(color), Pickable::IGNORE));
        }
    });
}

// ── Visual sync: reconcile 3D card entities with the server-projected view ───

/// Count `(lands, creatures)` for the given owner.
fn bf_row_counts(battlefield: &[crabomination::net::PermanentView], owner: usize) -> (usize, usize) {
    let lands = battlefield.iter().filter(|c| c.owner == owner && c.is_land()).count();
    let creatures = battlefield.iter().filter(|c| c.owner == owner && !c.is_land()).count();
    (lands, creatures)
}

/// Find the slot of a card within its row.
fn bf_row_slot(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    card_id: CardId,
    is_land: bool,
) -> Option<usize> {
    battlefield
        .iter()
        .filter(|c| c.owner == owner && c.is_land() == is_land)
        .position(|c| c.id == card_id)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn sync_game_visuals(
    mut commands: Commands,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_assets: Option<Res<CardMeshAssets>>,
    removed_animating: RemovedComponents<Animating>,
    hand_cards: Query<
        (
            Entity,
            &GameCardId,
            &Transform,
            Option<&StackCard>,
            &CardHoverLift,
            Option<&crate::card::FlippedFace>,
        ),
        (With<HandCard>, Without<Animating>),
    >,
    deck_cards: Query<
        (Entity, &GameCardId, &DeckCard, &Transform),
        (Without<HandCard>, Without<Animating>),
    >,
    bf_cards: Query<
        (
            Entity,
            &GameCardId,
            &CardOwner,
            &BattlefieldCard,
            &Transform,
            Option<&TapState>,
        ),
        (Without<HandCard>, Without<Animating>),
    >,
    mut deck_pile_q: Query<
        (Entity, &DeckPile, &mut Transform, &mut CardHoverLift),
        (Without<GameCardId>, Without<OpponentHandCard>),
    >,
    mut graveyard_q: Query<
        (
            &GraveyardPile,
            &mut Transform,
            &mut Visibility,
            &mut CardHoverLift,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<DeckPile>, Without<GameCardId>),
    >,
    opponent_hand_q: Query<
        (Entity, &OpponentHandCard, &Transform, Has<Animating>),
        (Without<DeckPile>, Without<GraveyardPile>),
    >,
    inflight: InFlightAnims,
    all_bf_entities: Query<&GameCardId, With<BattlefieldCard>>,
    all_hand_entity_ids: Query<&GameCardId, With<HandCard>>,
    // Entities currently on the stack (StackCard but not HandCard = opponent cards).
    all_stack_entities: Query<(Entity, &GameCardId, &Transform), (With<StackCard>, Without<HandCard>)>,
) {
    let Some(cv) = &view.0 else { return };
    let viewer = cv.your_seat;
    let n_seats = cv.players.len();
    // A spectator's `viewer` is the sentinel `SPECTATOR_SEAT`, which indexes no
    // real player — they have no hand or library of their own. Resolve the
    // viewer's hand/library through these panic-safe accessors so the board
    // still renders for spectators (empty viewer hand, zero-height deck).
    let empty_hand: Vec<crabomination::net::HandCardView> = Vec::new();
    let viewer_hand = cv.players.get(viewer).map(|p| &p.hand).unwrap_or(&empty_hand);
    // Client-side hand sort (config `gameplay.sort_hand`): lands first,
    // then by mana value, then name — Arena-style layout regardless of
    // draw order. Hidden cards keep server order at the end. Purely a
    // display ordering; every interaction below keys off card ids.
    let sorted_hand: Vec<crabomination::net::HandCardView>;
    let viewer_hand: &Vec<crabomination::net::HandCardView> = if inflight.gameplay.sort_hand {
        let mut h = viewer_hand.clone();
        h.sort_by_key(|c| match c {
            crabomination::net::HandCardView::Known(k) => (
                if k.card_types.contains(&crabomination::card::CardType::Land) { 0 } else { 1 },
                k.cost.cmc(),
                k.name.clone(),
            ),
            crabomination::net::HandCardView::Hidden { .. } => (2, 0, String::new()),
        });
        sorted_hand = h;
        &sorted_hand
    } else {
        viewer_hand
    };
    let viewer_lib_size = cv.players.get(viewer).map(|p| p.library.size).unwrap_or(0);
    let gy_sizes: Vec<usize> = cv.players.iter().map(|p| p.graveyard.len()).collect();
    let gy_size = |owner: usize| gy_sizes.get(owner).copied().unwrap_or(0);
    let deck_size = |owner: usize| {
        cv.players
            .iter()
            .find(|p| p.seat == owner)
            .map(|p| p.library.size)
            .unwrap_or(0)
    };

    // ── Always update: deck pile heights, spawn/despawn to match deck sizes ───

    // Existing pile cards: shrink to current deck sizes.
    let mut deck_pile_counts: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    for (entity, pile, mut transform, mut lift) in &mut deck_pile_q {
        let size = deck_size(pile.owner);
        if pile.index >= size {
            commands.entity(entity).despawn();
        } else {
            let base = deck_position(pile.owner, viewer, n_seats);
            let y = pile.index as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(base.x, y, base.z);
            transform.translation = pos;
            lift.base_translation = pos;
            *deck_pile_counts.entry(pile.owner).or_default() += 1;
        }
    }
    // Spawn missing deck pile cards per seat to match library size.
    if let Some(card_assets_ref) = &card_assets {
        for seat in 0..n_seats {
            let target_size = deck_size(seat);
            let current = deck_pile_counts.get(&seat).copied().unwrap_or(0);
            if current >= target_size {
                continue;
            }
            let base = deck_position(seat, viewer, n_seats);
            let rot = back_face_rotation(seat, viewer, n_seats);
            for i in current..target_size {
                let y = i as f32 * DECK_CARD_Y_STEP + 0.01;
                let pos = Vec3::new(base.x, y, base.z);
                commands.spawn((
                    Mesh3d(card_assets_ref.card_mesh.clone()),
                    MeshMaterial3d(card_assets_ref.back_material.clone()),
                    Transform::from_translation(pos).with_rotation(rot),
                    Visibility::default(),
                    DeckPile { owner: seat, index: i },
                    CardHoverLift { current_lift: 0.0, target_lift: 0.0, base_translation: pos },
                ));
            }
        }
    }

    let mut gy_in_flight: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for anim in &inflight.gy {
        *gy_in_flight.entry(anim.owner).or_default() += 1;
    }
    // A card that has just left the battlefield/hand for a graveyard still has
    // its on-board entity *this* frame — the `SendToGraveyardAnimation` insert
    // and `BattlefieldCard`/`HandCard` removal are deferred commands, so the
    // entity is still a `BattlefieldCard`/`HandCard` and `inflight.gy` can't
    // see the new animation yet. Without counting it, the pile pops the card in
    // at full count while the soon-to-fly board copy is still drawn — the "two
    // copies" flicker. So treat any live battlefield/hand entity whose id is
    // already in a graveyard view as in-flight too. (Next frame it carries
    // `SendToGraveyardAnimation` and is counted via `inflight.gy` instead, so
    // there's no double-count.)
    let gy_id_sets: Vec<HashSet<CardId>> = cv
        .players
        .iter()
        .map(|p| p.graveyard.iter().map(|c| c.id).collect())
        .collect();
    for (_, gid, owner, _, _, _) in &bf_cards {
        if gy_id_sets.get(owner.0).is_some_and(|s| s.contains(&gid.0)) {
            *gy_in_flight.entry(owner.0).or_default() += 1;
        }
    }
    for (_, gid, _, _, _, _) in &hand_cards {
        if gy_id_sets.get(viewer).is_some_and(|s| s.contains(&gid.0)) {
            *gy_in_flight.entry(viewer).or_default() += 1;
        }
    }

    for (gy, mut transform, mut vis, mut lift, _mat) in &mut graveyard_q {
        let gy_count = gy_size(gy.owner);
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_count.saturating_sub(in_flight);
        if arrived == 0 {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Visible;
            let base_pos = graveyard_position(gy.owner, viewer, n_seats);
            let y = arrived as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(base_pos.x, y, base_pos.z);
            transform.translation = pos;
            lift.base_translation = pos;
        }
    }

    if !view.is_changed() && removed_animating.is_empty() {
        return;
    }
    let Some(card_assets) = card_assets else { return };

    // Snapshot of every opponent hand visual entity (entity, owner, slot, pos,
    // rot, is_animating). We feed a per-owner hand pool so an opponent
    // "playing" a card consumes one of their face-down hand visuals as the
    // animation start position.
    let all_opp_hand: Vec<(Entity, usize, usize, Vec3, Quat, bool)> = opponent_hand_q
        .iter()
        .map(|(e, bh, t, is_animating)| {
            (e, bh.owner, bh.slot, t.translation, t.rotation, is_animating)
        })
        .collect();

    // Per-opponent hand pool of available (non-animating) cards to consume on
    // a hand→battlefield promotion. Sort by slot so we consume in stable
    // order, then drop the slot field.
    let mut pool_with_slot: std::collections::HashMap<usize, Vec<(Entity, usize, Vec3, Quat)>> =
        std::collections::HashMap::new();
    for (e, owner, slot, pos, rot, is_anim) in &all_opp_hand {
        if *is_anim {
            continue;
        }
        pool_with_slot
            .entry(*owner)
            .or_default()
            .push((*e, *slot, *pos, *rot));
    }
    for pool in pool_with_slot.values_mut() {
        pool.sort_by_key(|(_, slot, _, _)| *slot);
    }
    let mut hand_pool_by_owner: std::collections::HashMap<usize, Vec<(Entity, Vec3, Quat)>> =
        pool_with_slot
            .into_iter()
            .map(|(owner, pool)| {
                (
                    owner,
                    pool.into_iter().map(|(e, _, pos, rot)| (e, pos, rot)).collect(),
                )
            })
            .collect();
    let mut promoted: HashSet<Entity> = HashSet::new();

    // ── Update graveyard pile face-up material ────────────────────────────────
    for (gy, _transform, _vis, _lift, mut mat) in &mut graveyard_q {
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_size(gy.owner).saturating_sub(in_flight);
        if arrived > 0 {
            let top_name: Option<String> = cv.players[gy.owner].graveyard.get(arrived - 1).map(|c| c.name.clone());
            if let Some(name) = top_name {
                *mat = MeshMaterial3d(card_front_material(&name, &mut materials, &asset_server));
            }
        }
    }

    // ── Build zone ID sets ────────────────────────────────────────────────────
    let stack_ids: HashSet<CardId> = cv.stack.iter().filter_map(|item| {
        if let StackItemView::Known(k) = item { Some(k.source) } else { None }
    }).collect();
    // Sources of *spell* stack items only. A viewer hand card is pulled out to
    // the stack only when the card itself is a spell on the stack (cast from
    // hand). A trigger or ability whose source merely happens to be a card in
    // hand — e.g. Chancellor of the Tangle's opening-hand mana trigger — must
    // NOT yank the source card out of the hand (it never left), or it strands
    // mid-table once the trigger resolves.
    let stack_spell_ids: HashSet<CardId> = cv.stack.iter().filter_map(|item| {
        if let StackItemView::Known(k) = item {
            (k.kind == crabomination::net::StackItemKind::Spell).then_some(k.source)
        } else { None }
    }).collect();
    let hand_ids: HashSet<CardId> = viewer_hand.iter().map(|c| c.id()).collect();
    let bf_ids_by_owner: std::collections::HashMap<usize, HashSet<CardId>> = (0..n_seats)
        .map(|seat| {
            (
                seat,
                cv.battlefield.iter().filter(|c| c.owner == seat).map(|c| c.id).collect(),
            )
        })
        .collect();
    let creatures_by_owner: std::collections::HashMap<usize, usize> = (0..n_seats)
        .map(|seat| (seat, bf_row_counts(&cv.battlefield, seat).1))
        .collect();
    let creature_count = |seat: usize| creatures_by_owner.get(&seat).copied().unwrap_or(0);
    let bf_ids_for = |seat: usize| -> &HashSet<CardId> {
        bf_ids_by_owner.get(&seat).expect("seat in bf_ids map")
    };
    let hand_total = hand_ids.len();
    let all_bf_ids: HashSet<CardId> = cv.battlefield.iter().map(|c| c.id).collect();
    let visual_bf_ids: HashSet<CardId> = all_bf_entities.iter().map(|gid| gid.0).collect();
    // Opponent stack-card entities that already have a 3-D visual.
    let visual_opp_stack_ids: HashSet<CardId> =
        all_stack_entities.iter().map(|(_, gid, _)| gid.0).collect();
    // IDs visible in any player's graveyard. Used to disambiguate where a
    // disappeared hand card actually went: present in a graveyard → was
    // discarded/resolved-as-spell → fly to graveyard. Absent → it was
    // shuffled back to library (mulligan) → fly to deck pile.
    let in_any_graveyard: HashSet<CardId> = cv
        .players
        .iter()
        .flat_map(|p| p.graveyard.iter().map(|c| c.id))
        .collect();
    // Top-of-deck position for the viewer (used as the destination of a
    // mulligan put-back animation and the start of fetch/tutor animations).
    let viewer_deck_base = deck_position(viewer, viewer, n_seats);
    let viewer_deck_top_y = (viewer_lib_size as f32) * DECK_CARD_Y_STEP + 0.5;
    let viewer_deck_top = Vec3::new(viewer_deck_base.x, viewer_deck_top_y, viewer_deck_base.z);
    let viewer_deck_back_rot = back_face_rotation(viewer, viewer, n_seats);

    // ── Deck → Hand transitions (viewer's deck-card visual → face-up hand) ──
    let hand_zoom = inflight.hand_zoom.0;
    for (entity, game_id, _deck_card, transform) in &deck_cards {
        if hand_ids.contains(&game_id.0) {
            let slot = viewer_hand.iter().position(|c| c.id() == game_id.0)
                .unwrap_or(hand_total.saturating_sub(1));
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total, hand_zoom);
            commands
                .entity(entity)
                .remove::<DeckCard>()
                .insert(HandCard { slot })
                .insert(Animating)
                .insert(DrawCardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                });
        }
    }

    // ── Spawn viewer hand cards that have no visual entity yet ───────────────
    {
        let has_entity: HashSet<CardId> = all_hand_entity_ids.iter().map(|gid| gid.0)
            .chain(deck_cards.iter().map(|(_, gid, _, _)| gid.0))
            // A bf permanent currently animating back to hand will become
            // a HandCard entity once the animation completes — don't spawn
            // a duplicate hand card in the meantime.
            .chain(inflight.to_hand.iter().filter_map(|(gid, anim)| {
                anim.to_viewer.then_some(gid.0)
            }))
            .collect();
        let deck_base = deck_position(viewer, viewer, n_seats);
        let deck_y = viewer_lib_size as f32 * DECK_CARD_Y_STEP + 0.5;
        let deck_pos = Vec3::new(deck_base.x, deck_y, deck_base.z);
        for (slot, card_view) in viewer_hand.iter().enumerate() {
            use crabomination::net::HandCardView;
            let HandCardView::Known(known) = card_view else { continue };
            let card_id = known.id;
            if has_entity.contains(&card_id) { continue; }
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total, hand_zoom);
            let front_mat = card_front_material(&known.name, &mut materials, &asset_server);
            // For MDFC cards (Pathways), paint the back-child with the
            // back face's Scryfall image instead of the cardback so a
            // 180° flip animation reveals the alternate face. Uses the
            // `_back`-suffixed asset path that the prefetch downloaded
            // with `face=back`.
            let back_mat = if let Some(back_name) = known.back_face_name.as_deref() {
                card_back_face_material(back_name, &mut materials, &asset_server)
            } else {
                card_assets.back_material.clone()
            };
            let entity = spawn_single_card(
                &mut commands,
                &card_assets.card_mesh,
                front_mat,
                back_mat,
                Transform::from_translation(deck_pos),
                GameCardId(card_id),
                &known.name,
                target.translation,
            );
            commands.entity(entity).insert((
                HandCard { slot },
                Animating,
                DrawCardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: deck_pos,
                    start_rotation: Transform::from_translation(deck_pos).rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                },
            ));
        }
    }

    // ── Viewer hand → Battlefield / Stack / graveyard transitions ───────────
    for (entity, game_id, transform, stack_card, _lift, flipped_marker) in &hand_cards {
        if bf_ids_for(viewer).contains(&game_id.0) {
            let bf_card = cv.battlefield.iter().find(|c| c.id == game_id.0);
            let is_land = bf_card.is_some_and(|c| c.is_land());
            let target = if is_land {
                land_card_transform(&cv.battlefield, viewer, viewer, n_seats, game_id.0)
                    .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, true, false))
            } else {
                let slot = bf_row_slot(&cv.battlefield, viewer, game_id.0, false).unwrap_or(0);
                bf_card_transform(viewer, viewer, n_seats, slot, creature_count(viewer), false, false)
            };
            // Flipped MDFC played as its back face: the engine swapped
            // the card's definition to the back face, so the bf card's
            // name is the back-face name. We need the front-child mesh
            // to display the back-face image (via the `_back` asset
            // path) so the played face is up on the battlefield. Animate
            // back to the standard (un-flipped) bf rotation; without
            // this, the play animation would un-rotate the card and
            // expose the original front face (the wrong side).
            // For a flipped MDFC, snap the play animation's start
            // rotation to the bf target so it never interpolates the
            // 180° back-flip on screen. The front-child material gets
            // swapped to the back-face image in the same frame
            // (`apply_swap_front_material`), so the visual stays on the
            // back-face image throughout: the animation overwrites
            // transform.rotation = start_rotation (target) on its first
            // tick, before render. Both children hold the back-face
            // image at that moment, so the snap is invisible.
            let anim_start_rot = if flipped_marker.is_some() {
                target.rotation
            } else {
                transform.rotation
            };
            if flipped_marker.is_some()
                && let Some(bf) = bf_card
            {
                let new_front = card_back_face_material(&bf.name, &mut materials, &asset_server);
                commands.entity(entity).insert(crate::card::SwapFrontMaterial {
                    new_front,
                    new_path: crate::scryfall::card_back_face_asset_path(&bf.name),
                });
                commands.entity(entity).remove::<crate::card::FlippedFace>();
            }
            let is_token = bf_card.is_some_and(|c| c.is_token);
            commands
                .entity(entity)
                .remove::<HandCard>()
                .remove::<StackCard>()
                .remove::<CardHovered>()
                .insert(BattlefieldCard { is_land, is_token })
                .insert(CardOwner(viewer))
                .insert(TapState { tapped: false })
                .insert(Animating)
                .insert(PlayCardAnimation {
                    progress: 0.0,
                    speed: 2.0,
                    start_translation: transform.translation,
                    start_rotation: anim_start_rot,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    start_scale: transform.scale.x,
                });
        } else if stack_spell_ids.contains(&game_id.0) {
            if stack_card.is_none() {
                let idx = cv.stack.iter().position(|item| matches!(item, StackItemView::Known(k) if k.source == game_id.0)).unwrap_or(0);
                let total = stack_ids.len();
                let target = stack_card_transform(idx, total);
                commands
                    .entity(entity)
                    .remove::<CardHovered>()
                    .insert(StackCard)
                    .insert(Animating)
                    .insert(PlayCardAnimation {
                        progress: 0.0,
                        speed: 2.0,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                        start_scale: transform.scale.x,
                    });
            }
        } else if !hand_ids.contains(&game_id.0) {
            // The hand card is no longer anywhere visible. If a graveyard
            // now holds it the card was discarded or resolved as a spell —
            // fly it to the graveyard. Otherwise it was shuffled back into a
            // library (mulligan put-back) — fly it to the deck pile.
            if in_any_graveyard.contains(&game_id.0) {
                let gy_pos = graveyard_position(viewer, viewer, n_seats);
                let gy_rot = back_face_rotation(viewer, viewer, n_seats);
                commands
                    .entity(entity)
                    .remove::<HandCard>()
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(SendToGraveyardAnimation {
                        progress: 0.0,
                        speed: 1.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: gy_pos,
                        target_rotation: gy_rot,
                        owner: viewer,
                    });
            } else {
                commands
                    .entity(entity)
                    .remove::<HandCard>()
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(crate::card::ReturnToDeckAnimation {
                        progress: 0.0,
                        speed: 1.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: viewer_deck_top,
                        target_rotation: viewer_deck_back_rot,
                    });
            }
        }
    }

    // ── Battlefield → {Hand, Graveyard, despawn} transitions ─────────────────
    // A permanent that has left the battlefield could have:
    //   • been bounced to its owner's hand (Unsummon, Boomerang) — fly back
    //     to the hand fan;
    //   • been a token whose state-based action removed it — despawn outright
    //     (no graveyard pile entry; CR 704.5d);
    //   • died / been destroyed / sacrificed — fly to the graveyard pile.
    let viewer_hand_ids: HashSet<CardId> = viewer_hand
        .iter()
        .map(|c| c.id())
        .collect();
    let opp_hand_card_ids: HashSet<(usize, CardId)> = cv
        .players
        .iter()
        .filter(|p| p.seat != viewer)
        .flat_map(|p| p.hand.iter().map(|c| (p.seat, c.id())))
        .collect();
    for (entity, game_id, owner, bf, transform, _) in &bf_cards {
        if all_bf_ids.contains(&game_id.0) { continue; }
        // Tokens leaving the battlefield: just despawn (no graveyard arc).
        if bf.is_token {
            commands.entity(entity).despawn();
            continue;
        }
        // Bounced to viewer's hand — animate to a hand slot and convert.
        if viewer_hand_ids.contains(&game_id.0) {
            let slot = viewer_hand
                .iter()
                .position(|c| c.id() == game_id.0)
                .unwrap_or(hand_total.saturating_sub(1));
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total, hand_zoom);
            commands
                .entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardOwner>()
                .remove::<CardHovered>()
                .insert(Animating)
                .insert(crate::card::ReturnToHandAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    to_viewer: true,
                    target_slot: slot,
                    target_owner: viewer,
                    target_scale: hand_zoom,
                });
            continue;
        }
        // Bounced to an opponent's hand — fly toward their hand area, then
        // despawn so the next sync frame replaces it with a face-down
        // OpponentHandCard at the correct slot.
        if opp_hand_card_ids.iter().any(|(_, id)| *id == game_id.0) {
            let opp_seat = opp_hand_card_ids
                .iter()
                .find(|(_, id)| *id == game_id.0)
                .map(|(s, _)| *s)
                .unwrap_or(owner.0);
            let opp_hand_size = cv
                .players
                .iter()
                .find(|p| p.seat == opp_seat)
                .map(|p| p.hand.len())
                .unwrap_or(1);
            // Opponent hand zoom stays at 1.0 — only the viewer's own
            // fan is enlarged for readability.
            let target = hand_card_transform(
                opp_seat,
                viewer,
                n_seats,
                opp_hand_size.saturating_sub(1),
                opp_hand_size.max(1),
                1.0,
            );
            commands
                .entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardOwner>()
                .remove::<CardHovered>()
                .insert(Animating)
                .insert(crate::card::ReturnToHandAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    to_viewer: false,
                    target_slot: 0,
                    target_owner: opp_seat,
                    target_scale: 1.0,
                });
            continue;
        }
        // Default: fly to the owner's graveyard pile.
        let gy_pos = graveyard_position(owner.0, viewer, n_seats);
        let gy_rot = back_face_rotation(owner.0, viewer, n_seats);
        commands
            .entity(entity)
            .remove::<BattlefieldCard>()
            .remove::<TapState>()
            .remove::<CardHovered>()
            .insert(Animating)
            .insert(SendToGraveyardAnimation {
                progress: 0.0,
                speed: 1.5,
                start_translation: transform.translation,
                start_rotation: transform.rotation,
                target_translation: gy_pos,
                target_rotation: gy_rot,
                owner: owner.0,
            });
    }

    // ── Spawn new opponent battlefield cards (one pass per opponent seat) ───
    for seat in 0..n_seats {
        if seat == viewer {
            continue;
        }
        let to_spawn: Vec<(CardId, String, bool, bool, bool)> = cv
            .battlefield
            .iter()
            .filter(|c| {
                c.owner == seat
                    && !visual_bf_ids.contains(&c.id)
                    // Already on-screen as a stack-card entity (transition step above handles it).
                    && !visual_opp_stack_ids.contains(&c.id)
            })
            .map(|c| (c.id, c.name.clone(), c.is_land(), c.tapped, c.is_token))
            .collect();

        for (card_id, card_name, is_land, tapped, is_token) in to_spawn {
            // Always animate to the *untapped* battlefield pose first. If the
            // engine state already has the card tapped (typical for a land
            // played-and-auto-tapped to pay a spell cost in the same tick),
            // the tap-state-sync pass below detects the resulting mismatch
            // on the next frame and queues the tap animation as a chained
            // follow-up. This used to spawn directly into the tapped pose,
            // making the tap invisible.
            let target = if is_land {
                land_card_transform(&cv.battlefield, seat, viewer, n_seats, card_id)
                    .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, true, false))
            } else {
                let slot = bf_row_slot(&cv.battlefield, seat, card_id, false).unwrap_or(0);
                bf_card_transform(seat, viewer, n_seats, slot, creature_count(seat), false, false)
            };
            let _ = tapped; // tap state applied on the next sync pass.

            // Consume one of this opponent's face-down hand visuals as the
            // animation start (so the card appears to fly out of their hand).
            let pool = hand_pool_by_owner.entry(seat).or_default();
            let (start_pos, start_rot) = if let Some((hand_entity, pos, rot)) = pool.pop() {
                commands.entity(hand_entity).despawn();
                promoted.insert(hand_entity);
                (pos, rot)
            } else {
                // Opponent has no hand-card visual to consume — the card
                // came from their library or graveyard (fetchland, tutor,
                // reanimate, token). Animate from the top of their deck
                // pile rather than from where their hand fan would be.
                let opp_deck_base = deck_position(seat, viewer, n_seats);
                let opp_deck_size = cv
                    .players
                    .iter()
                    .find(|p| p.seat == seat)
                    .map(|p| p.library.size)
                    .unwrap_or(0);
                let y = (opp_deck_size as f32) * DECK_CARD_Y_STEP + 0.5;
                let pos = Vec3::new(opp_deck_base.x, y, opp_deck_base.z);
                (pos, back_face_rotation(seat, viewer, n_seats))
            };

            let front_mat = card_front_material(&card_name, &mut materials, &asset_server);
            let entity = spawn_single_card(
                &mut commands,
                &card_assets.card_mesh,
                front_mat,
                card_assets.back_material.clone(),
                Transform::from_translation(start_pos).with_rotation(start_rot),
                GameCardId(card_id),
                &card_name,
                target.translation,
            );
            commands.entity(entity).insert((
                BattlefieldCard { is_land, is_token },
                CardOwner(seat),
                // Spawn untapped so the tap-state-sync pass below detects
                // the engine vs visual mismatch and animates the tap.
                TapState { tapped: false },
                Animating,
                PlayCardAnimation {
                    progress: 0.0,
                    speed: 2.0,
                    start_translation: start_pos,
                    start_rotation: start_rot,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    // Opponent hand visual → battlefield: never zoomed.
                    start_scale: 1.0,
                },
            ));
        }
    }

    // ── Opponent stack-card visuals ──────────────────────────────────────────
    // For each opponent spell on the stack that doesn't yet have a 3-D entity,
    // consume one face-down hand visual and spawn a face-up card that animates
    // to the stack hover position (matching how the viewer's own spells behave).
    use crabomination::net::StackItemView;
    for (idx, item) in cv.stack.iter().enumerate() {
        let StackItemView::Known(k) = item else { continue };
        if k.controller == viewer { continue; }
        if visual_opp_stack_ids.contains(&k.source) { continue; }
        if visual_bf_ids.contains(&k.source) { continue; }

        let seat = k.controller;
        let total = stack_ids.len();
        let target = stack_card_transform(idx, total);

        let pool = hand_pool_by_owner.entry(seat).or_default();
        let (start_pos, start_rot) = if let Some((hand_entity, pos, rot)) = pool.pop() {
            commands.entity(hand_entity).despawn();
            promoted.insert(hand_entity);
            (pos, rot)
        } else {
            let base = deck_position(seat, viewer, n_seats);
            let lib_size = cv.players.iter().find(|p| p.seat == seat)
                .map(|p| p.library.size).unwrap_or(0);
            let y = lib_size as f32 * DECK_CARD_Y_STEP + 0.5;
            (Vec3::new(base.x, y, base.z), back_face_rotation(seat, viewer, n_seats))
        };

        let front_mat = card_front_material(&k.name, &mut materials, &asset_server);
        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(start_pos).with_rotation(start_rot),
            GameCardId(k.source),
            &k.name,
            target.translation,
        );
        commands.entity(entity).insert((
            StackCard,
            CardOwner(seat),
            Animating,
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: start_pos,
                start_rotation: start_rot,
                target_translation: target.translation,
                target_rotation: target.rotation,
                start_scale: 1.0,
            },
        ));
    }

    // Transition opponent stack entities whose spell resolved to the battlefield.
    for (entity, game_id, transform) in all_stack_entities.iter() {
        let Some(bf_card) = cv.battlefield.iter().find(|c| c.id == game_id.0) else {
            // Not on battlefield — still on stack or resolved to graveyard.
            if !stack_ids.contains(&game_id.0) {
                // Spell has left the stack (resolved to graveyard / exile).
                commands.entity(entity).despawn();
            }
            continue;
        };
        let seat = bf_card.owner;
        let is_land = bf_card.is_land();
        let target = if is_land {
            land_card_transform(&cv.battlefield, seat, viewer, n_seats, game_id.0)
                .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, true, false))
        } else {
            let slot = bf_row_slot(&cv.battlefield, seat, game_id.0, false).unwrap_or(0);
            bf_card_transform(seat, viewer, n_seats, slot, creature_count(seat), false, bf_card.tapped)
        };
        commands.entity(entity)
            .remove::<StackCard>()
            .insert(BattlefieldCard { is_land, is_token: bf_card.is_token })
            .insert(CardOwner(seat))
            .insert(TapState { tapped: bf_card.tapped })
            .insert(Animating)
            .insert(PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: transform.translation,
                start_rotation: transform.rotation,
                target_translation: target.translation,
                target_rotation: target.rotation,
                // Stack cards are unscaled.
                start_scale: 1.0,
            });
    }

    // ── Opponent hand reconciliation (one pass per opponent seat) ───────────
    let mut removed_opp_entities: HashSet<Entity> = promoted.clone();
    for seat in 0..n_seats {
        if seat == viewer {
            continue;
        }
        let target_hand_size = cv
            .players
            .iter()
            .find(|p| p.seat == seat)
            .map(|p| p.hand.len())
            .unwrap_or(0);

        let opp_hand_for_seat: Vec<&(Entity, usize, usize, Vec3, Quat, bool)> = all_opp_hand
            .iter()
            .filter(|(_, owner, _, _, _, _)| *owner == seat)
            .collect();
        // In-flight bounces toward this opponent count as already-spawned
        // hand visuals — they despawn on completion and the next sync
        // frame sees a normal OpponentHandCard for the bounced card. Without
        // this, the reconciler races the bounce by spawning a duplicate
        // face-down placeholder.
        let inflight_to_seat = inflight
            .to_hand
            .iter()
            .filter(|(_, anim)| !anim.to_viewer && anim.target_owner == seat)
            .count();
        let visual_count = opp_hand_for_seat
            .iter()
            .filter(|(e, _, _, _, _, _)| !promoted.contains(e))
            .count()
            + inflight_to_seat;

        if visual_count > target_hand_size {
            let mut sorted: Vec<_> = opp_hand_for_seat
                .iter()
                .filter(|(e, _, _, _, _, _)| !promoted.contains(e))
                .collect();
            sorted.sort_by_key(|(_, _, slot, _, _, _)| std::cmp::Reverse(*slot));
            for entry in sorted.iter().take(visual_count - target_hand_size) {
                let entity = entry.0;
                commands.entity(entity).despawn();
                removed_opp_entities.insert(entity);
            }
        } else if visual_count < target_hand_size {
            let base = deck_position(seat, viewer, n_seats);
            let deck_y = target_hand_size as f32 * DECK_CARD_Y_STEP + 0.5;
            let deck_pos = Vec3::new(base.x, deck_y, base.z);
            for slot in visual_count..target_hand_size {
                // Opponent hand — zoom stays at 1.0.
                let target = hand_card_transform(seat, viewer, n_seats, slot, target_hand_size, 1.0);
                let back_mat = card_assets.back_material.clone();
                let card_mesh = card_assets.card_mesh.clone();
                let start_transform = Transform::from_translation(deck_pos);
                commands
                    .spawn((
                        start_transform,
                        Visibility::default(),
                        OpponentHandCard { owner: seat, slot },
                        CardHoverLift {
                            current_lift: 0.0,
                            target_lift: 0.0,
                            base_translation: target.translation,
                        },
                        Animating,
                        PlayCardAnimation {
                            progress: 0.0,
                            speed: 2.0,
                            start_translation: deck_pos,
                            start_rotation: start_transform.rotation,
                            target_translation: target.translation,
                            target_rotation: target.rotation,
                            // Opponent hand visual — unscaled.
                            start_scale: 1.0,
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Mesh3d(card_mesh.clone()),
                            MeshMaterial3d(back_mat.clone()),
                            Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                        ));
                        parent.spawn((
                            Mesh3d(card_mesh.clone()),
                            MeshMaterial3d(back_mat.clone()),
                            Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                                .with_rotation(Quat::from_rotation_y(PI)),
                        ));
                    });
            }
        }

        // Despawn any cards whose slot is now out of range (animating card
        // with high slot may have escaped the count pass).
        for entry in &opp_hand_for_seat {
            let (entity, _, slot, _, _, _) = **entry;
            if !removed_opp_entities.contains(&entity) && slot >= target_hand_size {
                commands.entity(entity).despawn();
                removed_opp_entities.insert(entity);
            }
        }

        // Slide remaining non-animating cards to their new positions.
        for entry in &opp_hand_for_seat {
            let (entity, _, slot, pos, _rot, is_animating) = **entry;
            if removed_opp_entities.contains(&entity) || is_animating {
                continue;
            }
            // Opponent hand — zoom stays at 1.0.
            let target = hand_card_transform(seat, viewer, n_seats, slot, target_hand_size, 1.0);
            if pos.distance(target.translation) > 0.001 {
                commands
                    .entity(entity)
                    .insert(Animating)
                    .insert(HandSlideAnimation {
                        progress: 0.0,
                        speed: 4.0,
                        start_translation: pos,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                    })
                    .insert(CardHoverLift {
                        current_lift: 0.0,
                        target_lift: 0.0,
                        base_translation: target.translation,
                    });
            }
        }
    }

    // ── Spawn new viewer battlefield cards that don't have entities yet ──────
    let viewer_to_spawn: Vec<(CardId, String, bool, bool, bool)> = cv
        .battlefield
        .iter()
        .filter(|c| {
            c.owner == viewer
                && !visual_bf_ids.contains(&c.id)
                && !hand_cards.iter().any(|(_, gid, _, _, _, _)| gid.0 == c.id)
        })
        .map(|c| (c.id, c.name.clone(), c.is_land(), c.tapped, c.is_token))
        .collect();

    // Battlefield cards that didn't come from the viewer's hand (fetchlands,
    // tutors that drop directly onto the battlefield, reanimate, tokens) get
    // a `library → battlefield` arc animation starting from the top of the
    // viewer's deck pile, instead of teleporting in.
    for (card_id, card_name, is_land, tapped, is_token) in viewer_to_spawn {
        // Same untapped-spawn pattern as the opponent path above: land at
        // the untapped pose, let tap-state-sync animate the tap on the
        // next frame if the engine state has the card already tapped.
        let target = if is_land {
            land_card_transform(&cv.battlefield, viewer, viewer, n_seats, card_id)
                .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, true, false))
        } else {
            let slot = bf_row_slot(&cv.battlefield, viewer, card_id, false).unwrap_or(0);
            bf_card_transform(viewer, viewer, n_seats, slot, creature_count(viewer), false, false)
        };
        let _ = tapped;
        let front_mat = card_front_material(&card_name, &mut materials, &asset_server);
        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(viewer_deck_top).with_rotation(viewer_deck_back_rot),
            GameCardId(card_id),
            &card_name,
            target.translation,
        );
        commands.entity(entity).insert((
            BattlefieldCard { is_land, is_token },
            CardOwner(viewer),
            TapState { tapped: false },
            Animating,
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: viewer_deck_top,
                start_rotation: viewer_deck_back_rot,
                target_translation: target.translation,
                target_rotation: target.rotation,
                // From viewer's deck (unscaled) — straight to battlefield (unscaled).
                start_scale: 1.0,
            },
        ));
    }

    // ── Rebalance viewer hand slots ──────────────────────────────────────────
    for (entity, game_id, _transform, stack_card, lift, _flipped_marker) in &hand_cards {
        if stack_card.is_some() { continue; }
        if !hand_ids.contains(&game_id.0) { continue; }
        let Some(new_slot) = viewer_hand.iter().position(|c| c.id() == game_id.0) else { continue };
        let target = hand_card_transform(viewer, viewer, n_seats, new_slot, hand_total, hand_zoom);
        let dist = (lift.base_translation - target.translation).length();
        if dist > 0.1 {
            commands
                .entity(entity)
                .insert(Animating)
                .insert(HandSlideAnimation {
                    progress: 0.0,
                    speed: 3.0,
                    start_translation: lift.base_translation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                });
        }
        commands.entity(entity).insert(HandCard { slot: new_slot });
    }

    // ── Rebalance battlefield positions + sync tapped state ──────────────────
    for (entity, game_id, owner, bf, _transform, tap_state) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) { continue; }
        let is_land = bf.is_land;

        let game_tapped = cv.battlefield.iter().find(|c| c.id == game_id.0).is_some_and(|c| c.tapped);
        let visual_tapped = tap_state.is_some_and(|ts| ts.tapped);

        let target = if is_land {
            let Some(t) = land_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0) else { continue };
            let base_rot = t.rotation;
            let tapped_rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * base_rot;
            let rot = if game_tapped { tapped_rot } else { base_rot };
            Transform { translation: t.translation, rotation: rot, scale: t.scale }
        } else {
            let row_total = creature_count(owner.0);
            let Some(slot) = bf_row_slot(&cv.battlefield, owner.0, game_id.0, false) else { continue };
            bf_card_transform(owner.0, viewer, n_seats, slot, row_total, false, game_tapped)
        };

        if game_tapped != visual_tapped {
            let (untapped_rot, tapped_rot) = if is_land {
                let land_base = land_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0)
                    .map(|t| t.rotation).unwrap_or(target.rotation);
                let land_tapped = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * land_base;
                (land_base, land_tapped)
            } else {
                let row_total = creature_count(owner.0);
                let slot = bf_row_slot(&cv.battlefield, owner.0, game_id.0, false).unwrap_or(0);
                (
                    bf_card_transform(owner.0, viewer, n_seats, slot, row_total, false, false).rotation,
                    bf_card_transform(owner.0, viewer, n_seats, slot, row_total, false, true).rotation,
                )
            };
            let (start, end) = if game_tapped { (untapped_rot, tapped_rot) } else { (tapped_rot, untapped_rot) };
            commands
                .entity(entity)
                .insert(Animating)
                .insert(TapAnimation { progress: 0.0, speed: 4.0, start_rotation: start, target_rotation: end })
                .insert(TapState { tapped: game_tapped });
        }

        commands.entity(entity).insert(CardHoverLift {
            current_lift: 0.0,
            target_lift: 0.0,
            base_translation: target.translation,
        });
    }

    // ── Despawn viewer deck cards no longer in hand ─────────────────────────
    for (entity, game_id, _deck_card, _transform) in &deck_cards {
        if !hand_ids.contains(&game_id.0) {
            commands.entity(entity).despawn();
        }
    }
}

/// Scan wire events for `TopCardRevealed` and arm the reveal popup if found.
fn check_reveal_wire(events: &[crabomination::net::GameEventWire], reveal: &mut RevealPopupState) {
    for ev in events {
        if let crabomination::net::GameEventWire::TopCardRevealed { player, card_name, .. } = ev {
            reveal.card_path = Some(crate::scryfall::card_asset_path(card_name));
            reveal.revealed_player = Some(*player);
        }
    }
}

// ── Auto-advance non-interactive steps ───────────────────────────────────────

pub fn auto_advance_p0(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut ff: ResMut<FastForward>,
    stops: Option<Res<crate::systems::phase_bar::StopConfig>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(outbox) = outbox else { return };
    if cv.game_over.is_some() { return; }
    // Any pending decision suspends normal step advancement. If it's our
    // decision, the dedicated decision UI submits the answer; if it's an
    // opponent's, we just wait. Spamming `PassPriority` every frame here
    // floods the server with `DecisionPending` rejections (most visibly
    // during mulligan).
    if cv.pending_decision.is_some() { return; }
    if cv.priority != cv.your_seat { return; }
    // "Hold priority" toggle: step through every window manually. Explicit
    // fast-forwards still win (the player asked for them after toggling).
    if ff.manual_priority && !ff.end_turn && !ff.next_turn && ff.pass_until.is_none() {
        return;
    }

    let your_seat = cv.your_seat;

    // Phase-bar per-step stop override for this kind of turn (yours vs.
    // an opponent's). `Always` holds priority outright (an explicit
    // fast-forward still wins); `Skip` is folded into `should_advance`
    // below and also bypasses the combat view-holds.
    use crate::systems::phase_bar::StopMode;
    let stop_mode = stops
        .as_ref()
        .map(|s| s.mode(cv.active_player == your_seat, cv.step))
        .unwrap_or_default();
    if stop_mode == StopMode::Always && !ff.end_turn && !ff.next_turn && ff.pass_until.is_none()
    {
        return;
    }

    // Click-to-advance: arrived at the requested step — disarm and hold.
    if ff.pass_until == Some(cv.step) {
        ff.pass_until = None;
        return;
    }
    let passing_until = ff.pass_until.is_some();
    if ff.end_turn && cv.active_player != your_seat {
        ff.end_turn = false;
    }
    if ff.next_turn && cv.active_player == your_seat && cv.step == TurnStep::PreCombatMain {
        ff.next_turn = false;
        return;
    }

    // Hold priority on opp's DeclareAttackers when they declared
    // attackers — without this, the auto-pass fires before the viewer
    // sees who's swinging in, which makes the attacker lurch look like
    // it appears for the first time during DeclareBlockers. Skipped
    // when the opponent declined to attack (no `attacking`) so we
    // don't add dead-air to attack-less turns. End Turn / Next Turn
    // override the hold so the player can still skip the response
    // window when they don't care.
    if cv.step == TurnStep::DeclareAttackers
        && cv.active_player != your_seat
        && cv.battlefield.iter().any(|c| c.attacking)
        && !ff.end_turn
        && !ff.next_turn
        && !passing_until
        && stop_mode != StopMode::Skip
    {
        return;
    }

    // Don't auto-advance during interactive blocking when the viewer is
    // defending against any opponent's attack — *unless* there's nothing
    // to block: with Next Turn pressed, also skip blocker declaration when
    // either no attacker is targeting the viewer or the viewer has no
    // creature able to block (untapped, no Defender restriction —
    // summoning-sick creatures CAN block).
    if cv.step == TurnStep::DeclareBlockers && cv.active_player != your_seat {
        use crabomination::card::Keyword;
        // We only get to DeclareBlockers if at least one attacker was
        // declared — but we still gate the *skip* on having a viable
        // blocker too. An untapped, non-Defender creature you control
        // is a candidate; summoning-sick creatures CAN block, only
        // attacking is restricted.
        let any_attacker = cv.battlefield.iter().any(|c| c.attacking);
        let any_blocker = cv.battlefield.iter().any(|c| {
            c.owner == your_seat
                && c.is_creature()
                && !c.tapped
                && !c.keywords.contains(&Keyword::Defender)
        });
        let nothing_to_block = !any_attacker || !any_blocker;
        // An explicit per-step Skip means "never stop here" — the player
        // opted out of blocking; everything else holds the window open.
        if !((ff.next_turn || passing_until) && nothing_to_block) && stop_mode != StopMode::Skip {
            return;
        }
    }

    // Auto-pass on bookkeeping windows: non-main steps, the opponent's
    // turn (when no response is needed), and fast-forward frames.
    // On your own main phase, also auto-pass when the only things on
    // the stack are your own triggered/activated abilities (ETBs,
    // investigate, attack triggers, etc.) — those are pure bookkeeping.
    //
    // Crucially: if an opponent has a SPELL on the stack (e.g. they just
    // cast Birds of Paradise), we must NOT auto-pass — the viewer may
    // want to cast a counterspell or other response before it resolves.
    use crabomination::net::{StackItemKind, StackItemView};

    // Opponent stack items the viewer might want to respond to. Spells
    // are obvious; triggers are also responsive windows (CR 116.5 — the
    // active player passes priority first, the non-active player can
    // cast instants in response to a trigger before it resolves). Without
    // including triggers, an ETB / attack trigger on the bot's side
    // resolves before the viewer ever sees a stack pop, which the user
    // experiences as "the engine isn't pausing for priority at each
    // phase."
    let stack_has_opp_spell = cv.stack.iter().any(|item| matches!(
        item,
        StackItemView::Known(k)
            if k.controller != your_seat
                && matches!(k.kind, StackItemKind::Spell | StackItemKind::Trigger)
    ));

    let stack_is_own_triggers_only = !cv.stack.is_empty()
        && cv.stack.iter().all(|item| matches!(
            item,
            StackItemView::Known(k)
                if k.controller == your_seat && k.kind == StackItemKind::Trigger
        ));

    // End Turn (E) stops at opponent spells so the player can respond.
    // Next Turn (N) means "skip everything until my next main phase" —
    // it intentionally passes through opponent spells too.
    if stack_has_opp_spell && !ff.next_turn {
        return;
    }

    // Bookkeeping windows the viewer normally has no reason to act in.
    // (Untap has no priority window at all; the rest are pass-through steps
    // outside the main phases.)
    let bookkeeping_step = matches!(
        cv.step,
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw
            | TurnStep::BeginCombat | TurnStep::CombatDamage
            | TurnStep::EndCombat | TurnStep::End | TurnStep::Cleanup
    );

    // Whether the viewer actually has a legal instant-speed play in *this*
    // priority window. Every one of these lists is produced by the engine's
    // `would_accept` dry-run, so it's already priority- and timing-gated:
    // empty off-priority, and empty when the action isn't legal at the
    // current step (a sorcery in hand won't appear on the opponent's turn).
    // A non-empty list therefore means "you could legally act right now" —
    // e.g. crack a fetch land (an activatable ability) or hold up a
    // counterspell on the opponent's end step. When that's the case we stop
    // auto-passing and surface the window so the player gets priority.
    let has_instant_play = !cv.castable_hand.is_empty()
        || !cv.activatable_permanents.is_empty()
        || !cv.kickable_hand.is_empty()
        || !cv.buyback_hand.is_empty();

    // Auto-pass a window only when the viewer has nothing to do there —
    // unless they've explicitly asked to fast-forward. End Turn (E) skips
    // the rest of the viewer's own turn; Next Turn (N) skips the whole turn
    // cycle up to the viewer's next main phase. The stack-of-own-triggers
    // case is pure bookkeeping (ETBs, investigate, attack triggers) and
    // always advances so those don't strand the player.
    let should_advance = ff.end_turn
        || ff.next_turn
        || passing_until
        || cv.step == TurnStep::Untap
        || stack_is_own_triggers_only
        || stop_mode == StopMode::Skip
        || ((bookkeeping_step || cv.active_player != your_seat) && !has_instant_play);

    if should_advance {
        outbox.submit(GameAction::PassPriority);
    }
}

// ── Player 0 input ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_game_input(
    mut commands: Commands,
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut r: GameInputResources,
    server_events: Res<LatestServerEvents>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered_hand: Query<&GameCardId, (With<CardHovered>, With<HandCard>)>,
    hovered_bf: Query<(&GameCardId, &CardOwner), (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_target_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    hovered_command_zone: Query<
        (&GameCardId, &crate::card::CommandZoneCard),
        With<CardHovered>,
    >,
    valid_targets: Query<Entity, With<ValidTarget>>,
    btns: Res<ButtonState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let log = &mut *r.log;
    let targeting = &mut *r.targeting;
    let blocking = &mut *r.blocking;
    let attacking = &mut *r.attacking;
    let reveal = &mut *r.reveal;
    let menu_state = &mut *r.menu_state;
    let ff = &mut *r.ff;
    let card_names = &mut *r.card_names;
    let legal_targets = &mut *r.legal_targets;
    let modal_cast = &mut *r.modal_cast;

    // Refresh the card-name lookup from the current view so the event
    // formatter can resolve any CardId it sees. Hand / battlefield /
    // graveyard / stack are all included; opponent face-down hand
    // entries stay anonymous.
    if let Some(cv) = view.0.as_ref() {
        for player in &cv.players {
            for h in &player.hand {
                if let crabomination::net::HandCardView::Known(k) = h {
                    card_names.by_id.insert(k.id, k.name.clone());
                }
            }
            for g in &player.graveyard {
                card_names.by_id.insert(g.id, g.name.clone());
            }
        }
        for c in &cv.battlefield {
            card_names.by_id.insert(c.id, c.name.clone());
        }
        for item in &cv.stack {
            if let crabomination::net::StackItemView::Known(k) = item {
                card_names.by_id.insert(k.source, k.name.clone());
            }
        }
    }

    // Update game log from server events. A `TurnStarted` becomes a
    // turn-divider row (#5); every other event coalesces consecutive
    // duplicates via `push_event` (#7).
    let view_ref = view.0.as_ref();
    for ev in &server_events.0 {
        check_reveal_wire(std::slice::from_ref(ev), reveal);
        if let crabomination::net::GameEventWire::TurnStarted { player, turn } = ev {
            let who = view_ref
                .map(|cv| player_name(cv, *player))
                .unwrap_or_else(|| format!("P{player}"));
            log.push_divider(format!("──  Turn {turn} · {who}  ──"));
            continue;
        }
        log.push_event(
            format!("{}{}", event_glyph(ev), format_event(ev, card_names, view_ref)),
            event_color(ev),
        );
    }

    let Some(cv) = &view.0 else { return };
    let Some(outbox) = outbox else { return };
    let your_seat = cv.your_seat;
    // Spectators occupy no seat (`your_seat == SPECTATOR_SEAT`) — they have no
    // gameplay shortcuts and indexing `players[your_seat]` below would panic.
    if your_seat >= cv.players.len() {
        return;
    }

    // While the export-state prompt is open the dedicated prompt system
    // owns the keyboard — bail out so typing the bug-description doesn't
    // also trigger gameplay shortcuts (Space → pass priority, etc.).
    if r.export_prompt.active {
        return;
    }

    // Same deal while the debug console's card-name field is focused —
    // typing into the buffer shouldn't fire `KeyCode::KeyA` (Attack All)
    // or other gameplay shortcuts.
    if r.debug_console.card_input_focused {
        return;
    }

    // While a decision is pending for this viewer (mulligan, scry, search,
    // put-on-library, …), drop into decision-handling mode: the dedicated
    // decision UI systems own input — including 3D hand-card clicks for
    // PutOnLibrary — so we early-out here to avoid double-handling.
    //
    // Exception: `Decision::ChooseTarget` is satisfied via the in-scene
    // targeting cursor (highlights legal battlefield permanents, click
    // to pick), so we let the targeting branch below run and submit the
    // answer via `GameAction::SubmitDecision` instead of `CastSpell`.
    if let Some(pd) = &cv.pending_decision
        && pd.acting_player == your_seat
    {
        let is_choose_target = matches!(
            pd.decision.as_ref(),
            Some(crabomination::net::DecisionWire::ChooseTarget { .. })
        );
        if !is_choose_target {
            return;
        }
    }
    {
        // Click-or-Enter: a single boolean that captures either a
        // mouse-left click or the keyboard "activate" key. Used at
        // every `mouse.just_pressed(MouseButton::Left)` site so the
        // keyboard cursor (see `systems::kb_cursor`) drives the same
        // gameplay paths as the mouse.
        let activate = mouse.just_pressed(MouseButton::Left)
            || keyboard.just_pressed(KeyCode::Enter)
            || keyboard.just_pressed(KeyCode::NumpadEnter);

        // ── Blocking (defending against any opponent's attack) ──────────────
        if cv.step == TurnStep::DeclareBlockers && cv.active_player != your_seat && cv.priority == your_seat {
            let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
            if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
                blocking.selected_blocker = None;
                return;
            }
            if activate
                && let Some((game_id, owner)) = hovered_bf.iter().next()
            {
                if owner.0 == your_seat {
                    let already_assigned = blocking.assignments.iter().any(|(b, _)| *b == game_id.0);
                    let is_creature = cv.battlefield.iter().any(|c| c.id == game_id.0 && c.is_creature());
                    if is_creature && !already_assigned {
                        blocking.selected_blocker = Some(game_id.0);
                    }
                } else if owner.0 != your_seat
                    && let Some(blocker_id) = blocking.selected_blocker
                {
                    let is_attacker = cv.battlefield.iter().any(|c| c.id == game_id.0 && c.attacking);
                    if is_attacker {
                        blocking.assignments.push((blocker_id, game_id.0));
                        blocking.selected_blocker = None;
                    }
                }
            }
            if pass {
                let assignments = std::mem::take(&mut blocking.assignments);
                blocking.selected_blocker = None;
                if !assignments.is_empty() {
                    outbox.submit(GameAction::DeclareBlockers(assignments));
                }
                outbox.submit(GameAction::PassPriority);
            }
            return;
        }

        // ── Attacker selection (our own DeclareAttackers, our priority) ─────
        //
        // Flow:
        //   • click an own creature → toggle in/out of the plan with a
        //     default target (next opponent). The toggled-in creature is
        //     stored as `last_added` so the next defender-click reassigns
        //     *its* target.
        //   • click an opponent's planeswalker → reassign `last_added` to
        //     that PW.
        //   • click an opponent's player disc (`PlayerTargetZone`) or 2-D
        //     HUD chip (`PlayerHudPanel`) → reassign `last_added` to that
        //     player.
        //   • Esc / right-click → clear plan.
        //   • `A` / Attack button (handled below) submits the plan, falling
        //     back to "attack all eligible at next opp" when empty.
        if cv.step == TurnStep::DeclareAttackers
            && cv.active_player == your_seat
            && cv.priority == your_seat
        {
            if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
                attacking.clear();
                // Fall through so other handlers can also react (e.g. close
                // an open ability menu via Escape).
            } else if activate {
                use crabomination::card::{CardType, Keyword};
                use crabomination::game::AttackTarget;
                let next_opp = cv
                    .players
                    .iter()
                    .map(|p| p.seat)
                    .find(|s| *s != your_seat)
                    .unwrap_or(your_seat);

                let mut consumed = false;
                if let Some((game_id, owner)) = hovered_bf.iter().next() {
                    let bf = cv.battlefield.iter().find(|c| c.id == game_id.0);
                    if owner.0 == your_seat {
                        let eligible = bf
                            .map(|c| {
                                c.is_creature()
                                    && !c.tapped
                                    && (!c.summoning_sick
                                        || c.keywords.contains(&Keyword::Haste))
                                    && !c.keywords.contains(&Keyword::Defender)
                            })
                            .unwrap_or(false);
                        if eligible {
                            if attacking.contains(game_id.0) {
                                attacking.remove(game_id.0);
                            } else {
                                attacking
                                    .plan
                                    .push((game_id.0, AttackTarget::Player(next_opp)));
                                attacking.last_added = Some(game_id.0);
                            }
                            consumed = true;
                        }
                    } else {
                        let is_pw = bf
                            .map(|c| c.card_types.contains(&CardType::Planeswalker))
                            .unwrap_or(false);
                        if is_pw
                            && attacking.set_target_for_last_added(
                                AttackTarget::Planeswalker(game_id.0),
                            )
                        {
                            consumed = true;
                        }
                    }
                }
                if !consumed
                    && let Some(zone) = hovered_target_zone.iter().next()
                    && zone.0 != your_seat
                    && attacking.set_target_for_last_added(AttackTarget::Player(zone.0))
                {
                    consumed = true;
                }
                if consumed {
                    return;
                }
            }
            // 2-D chip click during DeclareAttackers reassigns the
            // last-added attacker's defender. Mirrors the 3-D disc path
            // but always visible and easy to hit.
            if let Some(seat) = btns.player_chip
                && seat != your_seat
                && attacking.set_target_for_last_added(
                    crabomination::game::AttackTarget::Player(seat),
                )
            {
                return;
            }
        }

        if cv.game_over.is_some() || cv.priority != your_seat { return; }

        // ── Targeting mode ────────────────────────────────────────────────────
        if targeting.active {
            if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
                // Esc / right-click during a decision-driven target
                // pick can't "cancel" — the engine is blocked waiting
                // for an answer. Swallow the press silently so the
                // user can still keep clicking targets. Spell / ability
                // targeting can still cancel back to normal mode.
                if !targeting.pending_decision_target {
                    cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                }
                return;
            }
            if activate {
                let is_ability_target = targeting.pending_ability_source.is_some();
                let cast_back = targeting.back_face_pending;
                let is_decision = targeting.pending_decision_target;
                for (game_id, _owner) in &hovered_bf {
                    let target = Target::Permanent(game_id.0);
                    // Equip (CR 702.6) takes precedence: the pending session
                    // is moving an Equipment onto the clicked creature.
                    if let Some(equipment) = targeting.pending_equip_source {
                        outbox.submit(GameAction::Equip { equipment, target: game_id.0 });
                        cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                        return;
                    }
                    if is_decision {
                        // Engine-driven `ChooseTarget`: only submit when
                        // the click lands on an enumerated legal target,
                        // so the user can't accidentally fire a no-op
                        // decision that bounces back with a GameError.
                        if !legal_targets.permanents.contains(&game_id.0) {
                            continue;
                        }
                        outbox.submit(GameAction::SubmitDecision(
                            crabomination::decision::DecisionAnswer::Target(target),
                        ));
                        cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                        return;
                    } else if is_ability_target {
                        if let (Some(src), Some(idx)) = (targeting.pending_ability_source, targeting.pending_ability_index) {
                            outbox.submit(GameAction::ActivateAbility { card_id: src, ability_index: idx, target: Some(target), x_value: None });
                            cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                            return;
                        }
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        // Gate on the catalog-enumerated legal set when
                        // it was populated. Empty set falls back to
                        // "permissive" — same as ability targeting that
                        // we can't pre-enumerate.
                        let legal_set_populated = !legal_targets.permanents.is_empty()
                            || !legal_targets.players.is_empty();
                        let target_is_legal = match target {
                            Target::Permanent(id) => legal_targets.permanents.contains(&id),
                            Target::Player(s) => legal_targets.players.contains(&s),
                        };
                        if legal_set_populated && !target_is_legal {
                            continue;
                        }
                        let mode = targeting.pending_mode;
                        let action = build_pending_cast(
                            pending_id, Some(target), mode, cast_back, targeting.pending_pay_times,
                        );
                        outbox.submit(action);
                        cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                        return;
                    }
                }
                for zone in &hovered_target_zone {
                    let target = Target::Player(zone.0);
                    if is_decision {
                        if !legal_targets.players.contains(&zone.0) {
                            continue;
                        }
                        outbox.submit(GameAction::SubmitDecision(
                            crabomination::decision::DecisionAnswer::Target(target),
                        ));
                        cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                        return;
                    } else if is_ability_target {
                        if let (Some(src), Some(idx)) = (targeting.pending_ability_source, targeting.pending_ability_index) {
                            outbox.submit(GameAction::ActivateAbility { card_id: src, ability_index: idx, target: Some(target), x_value: None });
                            cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                            return;
                        }
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        // Gate on the catalog-enumerated legal set when
                        // it was populated. Empty set falls back to
                        // "permissive" — same as ability targeting that
                        // we can't pre-enumerate.
                        let legal_set_populated = !legal_targets.permanents.is_empty()
                            || !legal_targets.players.is_empty();
                        let target_is_legal = match target {
                            Target::Permanent(id) => legal_targets.permanents.contains(&id),
                            Target::Player(s) => legal_targets.players.contains(&s),
                        };
                        if legal_set_populated && !target_is_legal {
                            continue;
                        }
                        let mode = targeting.pending_mode;
                        let action = build_pending_cast(
                            pending_id, Some(target), mode, cast_back, targeting.pending_pay_times,
                        );
                        outbox.submit(action);
                        cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                        return;
                    }
                }
            }
            // 2-D HUD chip click — same Player target submission as the
            // 3-D disc above, but driven by `poll_player_chip_clicks` so
            // it fires independent of `activate` (the Button's Pressed
            // transition is the click signal).
            if let Some(seat) = btns.player_chip {
                let target = Target::Player(seat);
                let is_ability_target = targeting.pending_ability_source.is_some();
                let cast_back = targeting.back_face_pending;
                let is_decision = targeting.pending_decision_target;
                if is_decision {
                    if !legal_targets.players.contains(&seat) {
                        return;
                    }
                    outbox.submit(GameAction::SubmitDecision(
                        crabomination::decision::DecisionAnswer::Target(target),
                    ));
                    cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                    return;
                } else if is_ability_target {
                    if let (Some(src), Some(idx)) = (
                        targeting.pending_ability_source,
                        targeting.pending_ability_index,
                    ) {
                        outbox.submit(GameAction::ActivateAbility {
                            card_id: src,
                            ability_index: idx,
                            target: Some(target),
                            x_value: None,
                        });
                        cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                        return;
                    }
                } else if let Some(pending_id) = targeting.pending_card_id {
                    let legal_set_populated = !legal_targets.permanents.is_empty()
                        || !legal_targets.players.is_empty();
                    if legal_set_populated && !legal_targets.players.contains(&seat) {
                        return;
                    }
                    let mode = targeting.pending_mode;
                    let action = build_pending_cast(
                        pending_id, Some(target), mode, cast_back, targeting.pending_pay_times,
                    );
                    outbox.submit(action);
                    cancel_targeting(&mut commands, targeting, legal_targets, &valid_targets);
                    return;
                }
            }
            return;
        }

        // ── Normal input ──────────────────────────────────────────────────────

        // Right-click P0 battlefield card → ability menu.
        // Right-click P0 hand card with an alt cost → alt-cast pitch modal.
        // Right-click P0 hand card with a back face (MDFC) → flip face.
        // Keyboard-only equivalents (act on the keyboard cursor's
        // currently-selected card via the same hovered_* queries —
        // `sync_kb_hover_marker` mirrors KeyboardSelected into
        // CardHovered):
        //   F = flip MDFC face         (was: right-click hand)
        //   L = open alt-cost modal    (was: right-click hand)
        //   M = open ability menu      (was: right-click bf)
        let kb_flip = keyboard.just_pressed(KeyCode::KeyF);
        let kb_alt = keyboard.just_pressed(KeyCode::KeyL);
        let kb_menu = keyboard.just_pressed(KeyCode::KeyM);
        let any_right = mouse.just_pressed(MouseButton::Right);
        // Right-click is "do whatever's most useful for this card":
        // alt-cost if available, else flip if MDFC, else ability menu
        // on the battlefield. With keyboard we expose the three actions
        // as distinct keys (F / L / M) so the user can target whichever
        // they actually want — the right-click branch keeps its current
        // priority-cascade behaviour.
        if any_right {
            if let Some(game_id) = hovered_hand.iter().next() {
                let card_id = game_id.0;
                let known: Option<crabomination::net::KnownCard> =
                    cv.players[your_seat].hand.iter().find_map(|h| match h {
                        crabomination::net::HandCardView::Known(k) if k.id == card_id => {
                            Some(k.clone())
                        }
                        _ => None,
                    });
                if let Some(k) = known {
                    if k.has_alternative_cost && k.alt_cost_available {
                        r.alt_cast.pending = Some(card_id);
                    } else if cv.squadable_hand.contains(&card_id) {
                        r.pay_times.pending =
                            Some((card_id, crate::game::PayTimesMechanic::Squad));
                        r.pay_times.times = 1;
                    } else if cv.replicatable_hand.contains(&card_id) {
                        r.pay_times.pending =
                            Some((card_id, crate::game::PayTimesMechanic::Replicate));
                        r.pay_times.times = 1;
                    } else if cv.multikickable_hand.contains(&card_id) {
                        r.pay_times.pending =
                            Some((card_id, crate::game::PayTimesMechanic::Multikicker));
                        r.pay_times.times = 1;
                    } else if k.back_face_name.is_some()
                        && !r.flipped_hand.flipped.insert(card_id) {
                            r.flipped_hand.flipped.remove(&card_id);
                    }
                }
            } else if let Some((game_id, owner)) = hovered_bf.iter().next() {
                if owner.0 == your_seat {
                    let card_id = game_id.0;
                    let has_non_mana = cv.battlefield.iter().find(|c| c.id == card_id)
                        .is_some_and(|c| c.abilities.iter().any(|a| !a.is_mana));
                    if has_non_mana {
                        menu_state.card_id = Some(card_id);
                        menu_state.spawn_pos = windows.single().ok()
                            .and_then(|w| w.cursor_position())
                            .unwrap_or(Vec2::new(400.0, 300.0));
                    } else {
                        menu_state.card_id = None;
                    }
                } else {
                    menu_state.card_id = None;
                }
            } else {
                menu_state.card_id = None;
            }
        }

        // Keyboard-specific: F flips the selected hand MDFC face.
        if kb_flip
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let has_back = cv.players[your_seat].hand.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == card_id && k.back_face_name.is_some())
            });
            if has_back && !r.flipped_hand.flipped.insert(card_id) {
                r.flipped_hand.flipped.remove(&card_id);
            }
        }

        // Keyboard-specific: L opens the alt-cost modal for the selected
        // hand card if it has one.
        if kb_alt
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let has_alt = cv.players[your_seat].hand.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == card_id && k.has_alternative_cost && k.alt_cost_available)
            });
            if has_alt {
                r.alt_cast.pending = Some(card_id);
            }
        }

        // Keyboard-specific: C activates Cycling on the selected hand
        // card (CR 702.29). Submits `GameAction::Cycle` directly — the
        // server pays the cycling cost from the floated mana pool,
        // discards the card to graveyard, and draws.
        if keyboard.just_pressed(KeyCode::KeyC)
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let has_cycling = cv.players[your_seat].hand.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == card_id && k.has_cycling)
            });
            if has_cycling {
                outbox.submit(GameAction::Cycle { card_id });
            } else {
                // No plain Cycling — fall back to Landcycling (CR 702.29e) if
                // the card has it (fetch a land of the named type to hand).
                let has_landcycling = cv.players[your_seat].hand.iter().any(|h| {
                    matches!(h,
                        crabomination::net::HandCardView::Known(k)
                        if k.id == card_id && k.has_landcycling)
                });
                if has_landcycling {
                    outbox.submit(GameAction::Landcycle { card_id });
                }
            }
        }

        // Keyboard-specific: M opens the ability menu for the selected
        // viewer-controlled battlefield card.
        if kb_menu
            && let Some((game_id, owner)) = hovered_bf.iter().next()
            && owner.0 == your_seat
        {
            let card_id = game_id.0;
            let has_non_mana = cv.battlefield.iter().find(|c| c.id == card_id)
                .is_some_and(|c| c.abilities.iter().any(|a| !a.is_mana));
            if has_non_mana {
                menu_state.card_id = Some(card_id);
                // Centre on the window if there's no cursor position.
                menu_state.spawn_pos = windows.single().ok()
                    .map(|w| Vec2::new(w.width() * 0.5, w.height() * 0.5))
                    .unwrap_or(Vec2::new(400.0, 300.0));
            }
        }

        // Keyboard-specific: E begins equip targeting on the hovered
        // viewer-controlled Equipment (CR 702.6). Sets
        // `pending_equip_source`; the next battlefield-creature click
        // submits `GameAction::Equip { equipment, target }`.
        if keyboard.just_pressed(KeyCode::KeyE)
            && let Some((game_id, owner)) = hovered_bf.iter().next()
            && owner.0 == your_seat
        {
            let card_id = game_id.0;
            let is_equippable = cv
                .battlefield
                .iter()
                .find(|c| c.id == card_id)
                .is_some_and(|c| c.equippable);
            if is_equippable {
                targeting.active = true;
                targeting.pending_equip_source = Some(card_id);
            }
        }

        // Close the ability menu when the user clicks anywhere else.
        // Not bound to Enter — the menu itself is mouse-driven today
        // and pressing Enter to close it would conflict with future
        // keyboard-menu support. Esc clears the cursor (handled in
        // `kb_cursor::handle_keyboard_cursor_input`).
        if mouse.just_pressed(MouseButton::Left) && menu_state.card_id.is_some() {
            menu_state.card_id = None;
        }

        let in_main = matches!(cv.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain);
        if in_main && activate
            && let Some(game_id) = hovered_hand.iter().next()
        {
            // Find card details from the view.
            if let Some(card) = cv.players[your_seat].hand.iter().find_map(|h| {
                if let crabomination::net::HandCardView::Known(k) = h && k.id == game_id.0 { return Some(k.clone()); } None
            }) {
                use crabomination::card::CardType;
                let is_flipped = r.flipped_hand.flipped.contains(&card.id);
                let has_back = card.back_face_name.is_some();
                if card.card_types.contains(&CardType::Land) {
                    if is_flipped {
                        outbox.submit(GameAction::PlayLandBack(card.id));
                    } else {
                        outbox.submit(GameAction::PlayLand(card.id));
                    }
                } else if has_back && is_flipped {
                    // Non-land MDFC played via its back face (a creature/
                    // instant/sorcery on the front means the back is one of
                    // those — the engine's `cast_spell_back_face` swaps the
                    // definition before validating cost / type / effect).
                    // We don't know whether the back face takes a target, so
                    // play it through the targeting-prompt branch when the
                    // front does — close-enough for the SOS MDFC cycles
                    // whose backs are creature-targeting spells.
                    if card.needs_target {
                        targeting.active = true;
                        targeting.pending_card_id = Some(card.id);
                        targeting.back_face_pending = true;
                    } else {
                        outbox.submit(GameAction::CastSpellBack {
                            card_id: card.id, target: None, additional_targets: vec![], mode: None, x_value: None,
                        });
                    }
                } else if !card.modal_descriptions.is_empty() {
                    // Modal "Choose one —" spell: pop the mode-pick modal
                    // and defer the cast until a mode is selected.
                    modal_cast.card_id = Some(card.id);
                    modal_cast.card_name = card.name.clone();
                    modal_cast.modes = card
                        .modal_descriptions
                        .iter()
                        .zip(card.modal_needs_target.iter().chain(std::iter::repeat(&false)))
                        .map(|(d, nt)| (d.clone(), *nt))
                        .collect();
                } else if card.needs_target {
                    let legal = crate::systems::legal_target_filter::enumerate_for_cast(
                        cv,
                        &card.name,
                        None,
                    );
                    // If we enumerated the filter and nothing is legal (e.g.
                    // Beaming Defiance with no creatures you control), don't
                    // arm the targeting cursor — just tell the player.
                    let no_targets = legal
                        .as_ref()
                        .is_some_and(|l| l.permanents.is_empty() && l.players.is_empty());
                    if no_targets {
                        log.push(format!("No legal targets for {}.", card.name));
                    } else {
                        targeting.active = true;
                        targeting.pending_card_id = Some(card.id);
                        targeting.back_face_pending = false;
                        if let Some(l) = legal {
                            *legal_targets = l;
                        }
                    }
                } else {
                    outbox.submit(GameAction::CastSpell { card_id: card.id, target: None, additional_targets: vec![], mode: None, x_value: None });
                }
            }
        }

        // ── Left-click own permanent → activate mana ability ──────────────
        // Lets the user manually select mana sources before casting (basic
        // lands, dual lands, mana-rock creatures). When the permanent has
        // exactly one mana ability that doesn't need a target, fire it
        // immediately. Multiple mana abilities open the ability menu so
        // the user can pick which colour. Permanents without any mana
        // ability fall through (right-click + M still open the non-mana
        // menu for those). Skips tapped permanents — there's nothing to
        // activate.
        if activate
            && menu_state.card_id.is_none()
            && !targeting.active
            && let Some((game_id, owner)) = hovered_bf.iter().next()
            && owner.0 == your_seat
            && let Some(perm) = cv.battlefield.iter().find(|c| c.id == game_id.0)
            && !perm.tapped
        {
            let mana_abilities: Vec<&crabomination::net::AbilityView> = perm
                .abilities
                .iter()
                .filter(|a| a.is_mana && !a.needs_target)
                .collect();
            match mana_abilities.len() {
                1 => {
                    outbox.submit(GameAction::ActivateAbility {
                        card_id: perm.id,
                        ability_index: mana_abilities[0].index,
                        target: None,
                        x_value: None,
                    });
                }
                n if n > 1 => {
                    menu_state.card_id = Some(perm.id);
                    menu_state.spawn_pos = windows.single().ok()
                        .and_then(|w| w.cursor_position())
                        .unwrap_or(Vec2::new(400.0, 300.0));
                }
                _ => {}
            }
        }

        // ── Command zone click → cast from CZ ───────────────────────────
        // Phase L: clicking a card in the viewer's own command zone
        // routes through `CastFromCommandZone` so the {2}×N commander
        // tax is added on top of the printed cost. Targeted commanders
        // (rare — most commanders are vanilla legendaries) fall back to
        // the targeting prompt the same way `CastSpell` does. Foreign
        // command zones aren't clickable.
        if in_main && activate
            && let Some((game_id, cz_card)) = hovered_command_zone.iter().next()
            && cz_card.owner == your_seat
            && let Some(card) = cv.players[your_seat].command.iter().find_map(|h| {
                if let crabomination::net::HandCardView::Known(k) = h && k.id == game_id.0 {
                    return Some(k.clone());
                }
                None
            })
        {
            {
                if card.needs_target {
                    // Reuse the targeting modal — when the user picks a
                    // target it submits CastSpell today. We mark the
                    // pending cast as command-zone-sourced via a new
                    // resource if/when we wire the prompt for it.
                    // For now: skip targeted commanders from the
                    // command zone (the Rofellos demo commander is
                    // non-targeted, so this branch is dormant).
                    targeting.active = true;
                    targeting.pending_card_id = Some(card.id);
                    targeting.back_face_pending = false;
                } else {
                    outbox.submit(GameAction::CastFromCommandZone {
                        card_id: card.id,
                        target: None,
                        additional_targets: vec![],
                        mode: None,
                        x_value: None,
                    });
                }
            }
        }

        // Attack All / Confirm Attack (A). If the viewer has hand-picked
        // attackers via the per-creature click flow, submit *that* plan;
        // otherwise fall back to "attack all eligible at the next opponent"
        // so the one-key shortcut still works for single-opponent games.
        let attack = keyboard.just_pressed(KeyCode::KeyA) || btns.attack;
        if attack && cv.step == TurnStep::DeclareAttackers {
            use crabomination::game::{Attack, AttackTarget};
            let attacks: Vec<Attack> = if !attacking.plan.is_empty() {
                attacking
                    .plan
                    .iter()
                    .map(|(attacker, target)| Attack { attacker: *attacker, target: *target })
                    .collect()
            } else {
                let next_opp = cv
                    .players
                    .iter()
                    .map(|p| p.seat)
                    .find(|s| *s != your_seat)
                    .unwrap_or(your_seat);
                use crabomination::card::Keyword;
                cv.battlefield
                    .iter()
                    .filter(|c| {
                        c.owner == your_seat
                            && c.is_creature()
                            && !c.tapped
                            && (!c.summoning_sick || c.keywords.contains(&Keyword::Haste))
                            && !c.keywords.contains(&Keyword::Defender)
                    })
                    .map(|c| Attack {
                        attacker: c.id,
                        target: AttackTarget::Player(next_opp),
                    })
                    .collect()
            };
            attacking.clear();
            outbox.submit(GameAction::DeclareAttackers(attacks));
        }

        // Pass Priority (Space)
        let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
        if pass {
            outbox.submit(GameAction::PassPriority);
        }

        // End Turn (E) — set fast-forward to skip to opponent's turn
        let end_turn = keyboard.just_pressed(KeyCode::KeyE) || btns.end_turn;
        if end_turn {
            ff.end_turn = true;
            outbox.submit(GameAction::PassPriority);
        }

        // Next Turn (N) — fast-forward through opponent's turn to our next main
        let next_turn = keyboard.just_pressed(KeyCode::KeyN) || btns.next_turn;
        if next_turn {
            ff.next_turn = true;
            outbox.submit(GameAction::PassPriority);
        }

    }
}

/// Build the cast action for a pending spell-targeting session, honoring the
/// MDFC back-face flag and any Squad / Replicate / Multikicker pay-times rider.
fn build_pending_cast(
    card_id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
    cast_back: bool,
    pay_times: Option<(u32, crate::game::PayTimesMechanic)>,
) -> GameAction {
    match pay_times {
        Some((times, mechanic)) => {
            popups::pay_times_cast_action(mechanic, card_id, times, target, mode)
        }
        None if cast_back => GameAction::CastSpellBack {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        },
        None => GameAction::CastSpell {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        },
    }
}

fn cancel_targeting(
    commands: &mut Commands,
    targeting: &mut TargetingState,
    legal: &mut crate::game::LegalTargets,
    valid_targets: &Query<Entity, With<ValidTarget>>,
) {
    targeting.active = false;
    targeting.pending_card_id = None;
    targeting.pending_ability_source = None;
    targeting.pending_ability_index = None;
    targeting.back_face_pending = false;
    targeting.pending_decision_target = false;
    targeting.pending_mode = None;
    targeting.pending_equip_source = None;
    targeting.pending_pay_times = None;
    legal.permanents.clear();
    legal.players.clear();
    legal.source_name.clear();
    legal.description.clear();
    for entity in valid_targets.iter() {
        commands.entity(entity).remove::<ValidTarget>();
    }
}

// ── MDFC flip sync ────────────────────────────────────────────────────────────

/// Reconcile each viewer hand card's persistent flip state
/// (`FlippedFace` marker on the entity) against the user's intent
/// (`FlippedHandCards.flipped`). When they disagree, attach a 180°
/// `MdfcFlipAnimation` and toggle the marker. Both card faces are
/// already painted with their proper Scryfall images at spawn time, so
/// the rotation alone reveals the alternate face — no material swap.
/// Also drops stale flip entries when cards leave the hand.
#[allow(clippy::type_complexity)]
pub fn sync_flipped_hand_cards(
    mut commands: Commands,
    cv: Res<CurrentView>,
    mut flipped: ResMut<crate::game::FlippedHandCards>,
    hand_cards: Query<
        (
            Entity,
            &GameCardId,
            &Transform,
            Option<&crate::card::FlippedFace>,
        ),
        (With<HandCard>, Without<crate::card::MdfcFlipAnimation>),
    >,
) {
    let Some(view) = cv.0.as_ref() else { return };
    let viewer = view.your_seat;
    if viewer >= view.players.len() { return; }

    // Drop flips for cards that are no longer in the viewer's hand.
    let in_hand: HashSet<CardId> = view.players[viewer]
        .hand
        .iter()
        .map(|h| h.id())
        .collect();
    flipped.flipped.retain(|id| in_hand.contains(id));

    for (entity, game_id, transform, marker) in &hand_cards {
        let card_id = game_id.0;
        let should_be_flipped = flipped.flipped.contains(&card_id);
        let is_flipped = marker.is_some();
        if should_be_flipped == is_flipped {
            continue;
        }
        commands.entity(entity).insert(crate::card::MdfcFlipAnimation {
            progress: 0.0,
            speed: 2.5,
            start_rotation: transform.rotation,
        });
        if should_be_flipped {
            commands.entity(entity).insert(crate::card::FlippedFace);
        } else {
            commands.entity(entity).remove::<crate::card::FlippedFace>();
        }
    }
}

// ── Command zone sync ─────────────────────────────────────────────────────────

/// Spawn/despawn visual entities for command-zone cards (Commander
/// commanders, Conspiracies). Each card is rendered as a face-up
/// fixed-position card near the seat's edge of the table. Click
/// handling routes through `CastFromCommandZone` (see
/// `try_cast_from_command_zone`).
///
/// Simple model: each (owner, slot) pair gets one visual entity.
/// On view sync, despawn entities for slot pairs that are no
/// longer present and spawn new ones for arrivals. No animations
/// — cards just appear/disappear with the view update.
#[allow(clippy::type_complexity)]
pub fn sync_command_zone(
    mut commands: Commands,
    cv: Res<CurrentView>,
    card_assets: Option<Res<crate::card::CardMeshAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    existing: Query<(Entity, &GameCardId), With<crate::card::CommandZoneCard>>,
) {
    let Some(view) = cv.0.as_ref() else { return };
    let Some(card_assets) = card_assets else { return };
    let viewer = view.your_seat;
    let n_seats = view.players.len();

    // Collect every (CardId, owner, slot) currently in any command
    // zone, so the spawn loop can reuse the layout helper.
    let mut want: HashMap<CardId, (usize, usize)> = HashMap::new();
    let mut want_name: HashMap<CardId, String> = HashMap::new();
    for player in &view.players {
        for (slot, entry) in player.command.iter().enumerate() {
            if let crabomination::net::HandCardView::Known(k) = entry {
                want.insert(k.id, (player.seat, slot));
                want_name.insert(k.id, k.name.clone());
            }
        }
    }

    // Despawn visuals for cards no longer in any command zone.
    let mut have: HashSet<CardId> = HashSet::new();
    for (entity, game_id) in &existing {
        if !want.contains_key(&game_id.0) {
            commands.entity(entity).despawn();
        } else {
            have.insert(game_id.0);
        }
    }

    // Spawn fresh visuals for newly-arrived command-zone cards.
    for (card_id, (owner, slot)) in &want {
        if have.contains(card_id) {
            continue;
        }
        let name = want_name.get(card_id).cloned().unwrap_or_default();
        let target = crate::card::command_zone_card_transform(*owner, viewer, n_seats, *slot);
        let front_mat = card_front_material(&name, &mut materials, &asset_server);
        let back_mat = card_assets.back_material.clone();
        let entity = crate::card::spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            back_mat,
            target,
            GameCardId(*card_id),
            &name,
            target.translation,
        );
        commands.entity(entity).insert(crate::card::CommandZoneCard {
            owner: *owner,
            slot: *slot,
        });
    }
}
