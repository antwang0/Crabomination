//! The in-game HUD: its marker components, the panels `setup_game_hud`
//! spawns, and the systems that keep the turn line and the panels' layout
//! current.

use super::*;

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TurnInfoText;

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
/// rendering as an empty dark rectangle under the action buttons).
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

/// The viewer's own HUD panel, top-left. (`PlayerHudPanel` is on every
/// seat's chip row.)
#[derive(Component)]
pub struct ViewerHudPanel;

/// The left control column: phase chart, action buttons, prompt line.
#[derive(Component)]
pub struct LeftControlColumn;

/// The viewer's HUD panel's wrapping row of stat chips and mana pips, which
/// `fit_player_hud_width` caps.
#[derive(Component)]
pub struct ViewerChipRows;

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

/// The in-turn action column (Pass / End Turn / Next Turn / Auto-pass).
#[derive(Component)]
pub struct ActionColumn;

/// Hide the in-turn actions once the viewer is out of a pod that goes on:
/// there is no priority left to pass.
pub fn hide_actions_when_out(view: Res<CurrentView>, mut q: Query<&mut Node, With<ActionColumn>>) {
    if !view.is_changed() {
        return;
    }
    let out = view
        .0
        .as_ref()
        .is_some_and(|cv| cv.players.iter().any(|p| p.seat == cv.your_seat && p.eliminated));
    let want = if out { Display::None } else { Display::Flex };
    for mut node in &mut q {
        if node.display != want {
            node.display = want;
        }
    }
}

#[derive(Component)]
pub struct AutoPassButtonLabel;

#[derive(Component)]
pub struct NextTurnButton;

/// "Export State" — in the Esc menu (the X hotkey does the same).
#[derive(Component)]
pub struct ExportStateButton;

/// "Surrender" button in the Esc menu — concedes the match (CR 104.3a).
/// Guarded by a two-click confirm (see [`SurrenderConfirm`]) so a stray
/// click can't throw the game.
#[derive(Component)]
pub struct SurrenderButton;

/// Text label inside [`SurrenderButton`], swapped to a confirm prompt while
/// the surrender is armed.
#[derive(Component)]
pub struct SurrenderButtonLabel;

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
            ViewerHudPanel,
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
                ViewerChipRows,
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                // Wraps at the panel's width (`fit_player_hud_width`).
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: Val::Px(4.0),
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
    //
    // The chart heads the left control column; the action buttons stack
    // under it (below). Both sit high on the left edge, where the table is
    // at its narrowest in the view: in the near-left corner the buttons cost
    // a pod's cards about a sixth of their size (`card::framing::hud_rects`).
    let left_column = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(110.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            LeftControlColumn,
            InGameRoot,
        ))
        .id();
    commands.entity(left_column).with_children(|col| {
        col
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(6.0)),
                    row_gap: Val::Px(2.0),
                    min_width: Val::Px(118.0),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
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
                        // This step's segment of the progress rail
                        // (`update_phase_chart`).
                        row.spawn((
                            Node {
                                width: Val::Px(3.0),
                                align_self: AlignSelf::Stretch,
                                margin: UiRect::right(Val::Px(4.0)),
                                border_radius: BorderRadius::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(PHASE_RAIL_AHEAD),
                            PhaseRail,
                            Pickable::IGNORE,
                        ));
                        row.spawn((
                            Text::new(format!("   {}", step_short_label(*step))),
                            tf(12.0),
                            TextColor(theme::TEXT_MUTED),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });
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
            crate::systems::scroll::Scrollable::default(),
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

    // Action buttons, under the phase chart in the left control column.
    // Only the in-turn actions live here: Export State, Surrender and Leave
    // are in the Esc menu (`quality::setup_quality_panel`). The bottom
    // centre stays clear for the hand fan.
    commands
        .entity(left_column)
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
                ActionColumn,
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

                // Take back the latest action (`systems::takeback`); the
                // caption names it, and both grey out with nothing to.
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    Button,
                    crate::systems::takeback::UndoButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Undo (Z)"),
                        tf(13.0),
                        TextColor(theme::TEXT_MUTED),
                        crate::systems::takeback::UndoButtonLabel,
                    ));
                    p.spawn((
                        Text::new("nothing to take back"),
                        tf(10.0),
                        TextColor(theme::TEXT_MUTED),
                        crate::systems::takeback::UndoButtonCaption,
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
            });
            // The prompt line ("Your priority — respond, or P to pass",
            // "Choose a target", a trigger to answer) under the buttons it
            // names, as MTGO and XMage place theirs. It sat bottom-centre,
            // over the hand it was asking you to play from. Under the
            // buttons, so a long prompt never moves them.
            // `card::framing::hud_rects` keeps the table clear of it.
            wrap.spawn((
                Node {
                    width: Val::Px(170.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    display: Display::None,
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
                HintChip,
            ))
            .with_children(|chip| {
                chip.spawn((Text::new(""), tf(13.0), TextColor(theme::ACCENT_GOLD), HintText));
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

    // Stack panel (hidden when the stack is empty; rebuilt on change),
    // beside the 3-D stack lane (`place_stack_panel`); bottom-centre until
    // a camera fit has placed the lane. The outer node is transparent — it
    // just positions its child.
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
            StackPanelAnchor,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(3.0),
                    // The width the lane's fit keeps room for; a long
                    // line wraps.
                    width: Val::Px(crate::card::framing::STACK_PANEL_WIDTH),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
                StackPanel,
            ));
        });

    // Bottom-right: a small, always-present affordance pointing at the
    // keyboard-shortcut overlay (`toggle_shortcut_help`). Beneath the rest
    // of the HUD: a duel's stack panel can sit in this corner, and a deep
    // stack's reaches the hint.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(12.0),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(theme::layer::HUD - 1),
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

pub fn update_turn_text(
    view: Res<CurrentView>,
    mut q: Query<&mut Text, With<TurnInfoText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    // Writing Text unconditionally would dirty it (re-shape/re-layout)
    // every frame; the inputs only change with the view. `is_added` covers
    // the HUD entity spawning after the last view change.
    if !view.is_changed() && !t.is_added() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    // The dedicated centered Game Over modal owns the end-game UI; the
    // corner HUD just holds a placeholder so it isn't visually empty.
    t.0 = if cv.game_over.is_some() {
        String::new()
    } else {
        let whose = if cv.active_player == cv.your_seat {
            "Your turn".to_string()
        } else {
            format!("{}'s turn", player_name(cv, cv.active_player))
        };
        format!("Turn {} · {} · {whose}", cv.turn, step_name(cv.step))
    };
}

/// Keep the left control column just below the viewer's HUD panel, whose
/// chips wrap to a second or third line in a narrow window or a large UI:
/// at a fixed `top` the panel covered the phase chart's first rows.
pub fn position_left_column_below_hud(
    hud_q: Query<&bevy::ui::ComputedNode, With<ViewerHudPanel>>,
    mut column_q: Query<&mut Node, With<LeftControlColumn>>,
) {
    let Ok(hud) = hud_q.single() else { return };
    let Ok(mut column) = column_q.single_mut() else { return };
    let hud_h = hud.size().y * hud.inverse_scale_factor();
    if hud_h <= 0.0 {
        return;
    }
    // The panel sits at top:10; leave 10 px under it (110 for a one-row panel).
    let target = Val::Px((10.0 + hud_h + 10.0).max(110.0).round());
    if column.top != target {
        column.top = target;
    }
}

/// Wrap the viewer's HUD chips at the width the camera fit reserves for the
/// panel (`framing::player_panel_width`), or sooner when a pod's opponent
/// panel is wider than the fit assumes, so they never run under the
/// opponent panel — at 1280x720 they covered its life and hand counts.
///
/// The cap goes on the chip row, not the panel: an absolute panel with a
/// `max_width` wrapped its row but sized its own height for one line, so
/// the second hung below it.
#[allow(clippy::type_complexity)]
pub fn fit_player_hud_width(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    opponents_q: Query<&bevy::ui::ComputedNode, With<OpponentStatusPanel>>,
    mut rows_q: Query<&mut Node, (With<ViewerChipRows>, Without<PlayerStatsRow>)>,
    mut stats_q: Query<(&mut Node, Option<&Children>), (With<PlayerStatsRow>, Without<ViewerChipRows>)>,
    pips_q: Query<&bevy::ui::ComputedNode, With<ManaPipRow>>,
    sizes: Query<&bevy::ui::ComputedNode>,
    ui_scale: Res<UiScale>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok(mut rows) = rows_q.single_mut() else { return };
    let Ok((mut stats, chips)) = stats_q.single_mut() else { return };
    // In UI px, which `UiScale` multiplies, as the panels are laid out.
    let w = window.width() / ui_scale.0;
    let ui_px = |n: &bevy::ui::ComputedNode| n.size().x * n.inverse_scale_factor();
    // Both panels sit 10 px in from their edges; keep 10 px between them.
    let beside_opponents = opponents_q.single().map_or(f32::INFINITY, |p| w - 30.0 - ui_px(p));
    let panel = (crate::card::framing::player_panel_width(w) - 10.0).min(beside_opponents);
    // Less the panel's padding (8 px) and border (2 px) either side.
    let max = (panel.max(260.0) - 20.0).round();
    if rows.max_width != Val::Px(max) {
        rows.max_width = Val::Px(max);
    }
    // Wrap only when the chips don't fit. A row that fits to within a
    // rounding error could still wrap its last chip — at 80 % "→ your
    // priority" dropped to a second line, and hung below the panel, which
    // had sized itself for one.
    let chips: Vec<f32> =
        chips.into_iter().flat_map(|c| c.iter()).filter_map(|c| sizes.get(c).ok().map(ui_px)).collect();
    let pips = pips_q.single().map_or(0.0, ui_px);
    // The rows' gaps: 4 px between chips, 6 px before the mana pips.
    let content = chips.iter().sum::<f32>() + 4.0 * chips.len().saturating_sub(1) as f32 + 6.0 + pips;
    let wrap = if content > max { FlexWrap::Wrap } else { FlexWrap::NoWrap };
    if stats.flex_wrap != wrap {
        stats.flex_wrap = wrap;
    }
    if rows.flex_wrap != wrap {
        rows.flex_wrap = wrap;
    }
}
