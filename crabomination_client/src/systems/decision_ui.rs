//! UI for pending player-choice decisions (scry, color choice, searches…).
//!
//! Reads the pending decision from `CurrentView` (server-projected) and
//! submits answers via `NetOutbox`. During the mulligan phase `CurrentView`
//! is empty and no modal is shown.

use bevy::prelude::*;

use crabomination::{
    card::CardId,
    decision::DecisionAnswer,
    game::{GameAction, Target},
    net::DecisionWire,
};

use crate::game::{GameLog, LegalTargets};
use crate::net_plugin::{CurrentView, NetOutbox};
use crate::scryfall;
use crate::theme::{self, HoverTint, UiFonts};

#[derive(Component)]
pub struct DecisionModal;

#[derive(Component)]
pub struct ScryToggleButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct ScryReorderButton {
    pub card_id: CardId,
    pub delta: i32,
}

#[derive(Component)]
pub struct DecisionConfirmButton;

/// ← / → reorder button for the OrderTriggers modal (CR 603.3b). Moves the
/// trigger `delta` slots in the stack-push ordering.
#[derive(Component)]
pub struct TriggerReorderButton {
    pub source: CardId,
    pub delta: i32,
}

#[derive(Component)]
pub struct MulliganKeepButton;

#[derive(Component)]
pub struct MulliganTakeButton;

#[derive(Component)]
pub struct SearchSelectButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct PutOnLibrarySelectButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct DiscardSelectButton {
    pub card_id: CardId,
}

/// While a hand-card visual entity is selected for bottoming during the
/// PutOnLibrary phase, this component stores the two border child entities
/// drawn around it. Stored separately from `CardBorderHighlight` so the
/// hover-highlight system (which manages its own `CardBorderHighlight`)
/// doesn't conflict.
#[derive(Component)]
pub struct PutOnLibraryHighlight {
    back: Entity,
    front: Entity,
}

/// Local UI state tracked during an in-flight decision. Cleared when the
/// server's `pending_decision` goes back to `None`.
#[derive(Resource, Default)]
pub struct DecisionUiState {
    /// For Scry: per-card "send to bottom" flags (false = keep on top).
    pub scry: Vec<(CardId, bool)>,
    /// For SearchLibrary: the card the player selected (None = failed search).
    pub search_selected: Option<CardId>,
    /// For PutOnLibrary / Mulligan bottoming: ordered list of selected card IDs.
    pub put_on_library: Vec<CardId>,
    /// For Discard (Inquisition / Thoughtseize picker): selected card IDs.
    pub discard_selected: Vec<CardId>,
    /// For OrderTriggers (CR 603.3b): the working stack-push order of the
    /// controller's simultaneous triggers (index 0 pushed first).
    pub trigger_order: Vec<CardId>,
    /// CardId the modal was last spawned for — avoids respawning each frame.
    pub spawned_for: Option<DecisionKey>,
}

/// Fingerprint of a pending decision. Used to detect when a new decision
/// arrived (so the modal respawns) vs. the same one still showing.
#[derive(Clone, PartialEq, Eq)]
pub enum DecisionKey {
    Scry(Vec<CardId>),
    Search(Vec<CardId>),
    PutOnLibrary(Vec<CardId>),
    Discard(Vec<CardId>, u32),
    Mulligan(Vec<CardId>, usize),
    ChooseColor(CardId),
    /// `Decision::ChooseTarget` — keyed by the legal-target list so a
    /// re-spawned decision with a different legal set is treated as
    /// new (the old highlight-set needs to clear).
    ChooseTarget(CardId, Vec<Target>),
    /// `Decision::Learn` — keyed by the offered Lesson ids.
    Learn(Vec<CardId>),
    /// `Decision::OrderTriggers` — keyed by the trigger source ids.
    OrderTriggers(Vec<CardId>),
}

fn decision_key(decision: &DecisionWire) -> Option<DecisionKey> {
    match decision {
        DecisionWire::Scry { cards, .. } => Some(DecisionKey::Scry(
            cards.iter().map(|(id, _)| *id).collect(),
        )),
        DecisionWire::SearchLibrary { candidates, .. } => Some(DecisionKey::Search(
            candidates.iter().map(|(id, _)| *id).collect(),
        )),
        DecisionWire::PutOnLibrary { hand, .. } => Some(DecisionKey::PutOnLibrary(
            hand.iter().map(|(id, _)| *id).collect(),
        )),
        DecisionWire::Discard { hand, count, .. } => Some(DecisionKey::Discard(
            hand.iter().map(|(id, _)| *id).collect(),
            *count,
        )),
        DecisionWire::Mulligan { hand, mulligans_taken, .. } => Some(DecisionKey::Mulligan(
            hand.iter().map(|(id, _)| *id).collect(),
            *mulligans_taken,
        )),
        DecisionWire::ChooseColor { source, .. } => Some(DecisionKey::ChooseColor(*source)),
        DecisionWire::Learn { lessons, .. } => {
            Some(DecisionKey::Learn(lessons.iter().map(|(id, _)| *id).collect()))
        }
        DecisionWire::ChooseTarget { source, legal, .. } => {
            Some(DecisionKey::ChooseTarget(*source, legal.clone()))
        }
        DecisionWire::OrderTriggers { triggers, .. } => {
            Some(DecisionKey::OrderTriggers(triggers.iter().map(|(id, _)| *id).collect()))
        }
        _ => None,
    }
}

const CARD_ASPECT_RATIO: f32 = 88.0 / 63.0;
const CARD_W: f32 = 180.0;
const CARD_H: f32 = CARD_W * CARD_ASPECT_RATIO;

// Aliases that keep the existing call sites short and locally
// self-documenting. The actual values live in `theme.rs` so the modal
// look stays in sync with the rest of the chrome.
use theme::PANEL_TILE_BG as MODAL_TILE_BG;
use theme::BUTTON_TERTIARY_BG as REORDER_BG;
use theme::BUTTON_TERTIARY_BG_DISABLED as REORDER_BG_DISABLED;

/// Spawn or despawn the decision modal based on the server view. Only shows
/// for decisions owned by P0 (your_seat).
pub fn spawn_decision_ui(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut targeting: ResMut<crate::game::TargetingState>,
    mut legal_targets: ResMut<LegalTargets>,
    existing: Query<Entity, With<DecisionModal>>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
) {
    let Some(cv) = &view.0 else {
        // Mulligan / no view yet — tear down any existing modal.
        for e in &existing {
            commands.entity(e).despawn();
        }
        state.scry.clear();
        state.search_selected = None;
        state.put_on_library.clear();
        state.discard_selected.clear();
        state.spawned_for = None;
        // Clear any decision-driven targeting flag so the cursor
        // doesn't stay armed across a state change.
        if targeting.pending_decision_target {
            targeting.active = false;
            targeting.pending_decision_target = false;
        }
        legal_targets.permanents.clear();
        legal_targets.players.clear();
        legal_targets.enumerated = false;
        legal_targets.source_name.clear();
        legal_targets.description.clear();
        return;
    };

    let pending = match &cv.pending_decision {
        Some(pd) if pd.acting_player == cv.your_seat => pd,
        _ => {
            for e in &existing {
                commands.entity(e).despawn();
            }
            if state.spawned_for.is_some() {
                state.scry.clear();
                state.search_selected = None;
                state.put_on_library.clear();
                state.discard_selected.clear();
                state.spawned_for = None;
            }
            if targeting.pending_decision_target {
                targeting.active = false;
                targeting.pending_decision_target = false;
            }
            legal_targets.permanents.clear();
            legal_targets.players.clear();
            legal_targets.source_name.clear();
            legal_targets.description.clear();
            return;
        }
    };

    let wire = match &pending.decision {
        Some(d) => d,
        None => return,
    };

    let key = match decision_key(wire) {
        Some(k) => k,
        None => return,
    };

    if state.spawned_for.as_ref() == Some(&key) {
        return;
    }

    for e in &existing {
        commands.entity(e).despawn();
    }

    match wire {
        DecisionWire::Scry { cards, .. } => {
            if state.scry.is_empty() {
                state.scry = cards.iter().map(|(id, _)| (*id, false)).collect();
            }
            state.spawned_for = Some(key);
            let name_map: std::collections::HashMap<CardId, &str> =
                cards.iter().map(|(id, n)| (*id, n.as_str())).collect();
            let ordered: Vec<(CardId, String, bool)> = state
                .scry
                .iter()
                .map(|(id, bottom)| (*id, name_map[id].to_string(), *bottom))
                .collect();
            spawn_scry_modal(&mut commands, &asset_server, &ui_fonts, &ordered);
        }
        DecisionWire::SearchLibrary { candidates, .. } => {
            state.search_selected = None;
            state.spawned_for = Some(key);
            spawn_search_modal(&mut commands, &asset_server, &ui_fonts, candidates);
        }
        DecisionWire::PutOnLibrary { count, hand, .. } => {
            state.put_on_library.clear();
            state.spawned_for = Some(key);
            spawn_put_on_library_modal(&mut commands, &asset_server, &ui_fonts, hand, *count);
        }
        DecisionWire::Mulligan { hand, mulligans_taken, serum_powders, .. } => {
            state.spawned_for = Some(key);
            spawn_mulligan_modal(
                &mut commands,
                &asset_server,
                &ui_fonts,
                hand,
                *mulligans_taken,
                serum_powders,
            );
        }
        DecisionWire::ChooseColor { legal, .. } => {
            state.spawned_for = Some(key);
            spawn_choose_color_modal(&mut commands, &ui_fonts, legal);
        }
        DecisionWire::Learn { lessons, hand, .. } => {
            state.spawned_for = Some(key);
            spawn_learn_modal(&mut commands, &ui_fonts, lessons, hand);
        }
        DecisionWire::Discard { count, hand, .. } => {
            state.discard_selected.clear();
            state.spawned_for = Some(key);
            spawn_discard_modal(&mut commands, &asset_server, &ui_fonts, hand, *count);
        }
        DecisionWire::OrderTriggers { triggers, .. } => {
            if state.trigger_order.is_empty() {
                state.trigger_order = triggers.iter().map(|(id, _)| *id).collect();
            }
            state.spawned_for = Some(key);
            let name_map: std::collections::HashMap<CardId, &str> =
                triggers.iter().map(|(id, n)| (*id, n.as_str())).collect();
            let ordered: Vec<(CardId, String)> = state
                .trigger_order
                .iter()
                .map(|id| (*id, name_map.get(id).copied().unwrap_or("Triggered ability").to_string()))
                .collect();
            spawn_order_triggers_modal(&mut commands, &asset_server, &ui_fonts, &ordered);
        }
        DecisionWire::ChooseTarget { legal, source_name, description, .. } => {
            // No modal — reuse the existing in-scene targeting cursor.
            // Flipping `pending_decision_target` flags `handle_game_input`
            // to submit picks as `DecisionAnswer::Target` instead of
            // wrapping them in `CastSpell` / `ActivateAbility`.
            state.spawned_for = Some(key);
            targeting.active = true;
            targeting.pending_card_id = None;
            targeting.pending_ability_source = None;
            targeting.pending_ability_index = None;
            targeting.back_face_pending = false;
            targeting.pending_decision_target = true;
            legal_targets.permanents.clear();
            legal_targets.players.clear();
            // The server handed us the authoritative legal list.
            legal_targets.enumerated = true;
            for t in legal {
                match t {
                    Target::Permanent(id) => {
                        legal_targets.permanents.insert(*id);
                    }
                    Target::Player(s) => {
                        legal_targets.players.insert(*s);
                    }
                }
            }
            legal_targets.source_name = source_name.clone();
            legal_targets.description = description.clone();
        }
        _ => {}
    }
}

fn spawn_scry_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    ordered: &[(CardId, String, bool)],
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(16.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let n = ordered.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "Scry {n}: click card to toggle Bottom  ·  ← → to reorder  ·  left = top of library"
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (i, (card_id, name, is_bottom)) in ordered.iter().enumerate() {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    let at_left = i == 0;
                    let at_right = i == n - 1;

                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Button,
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(CARD_W),
                                padding: UiRect::all(Val::Px(6.0)),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(if *is_bottom { theme::BUTTON_SELECTED_BG } else { MODAL_TILE_BG }),
                            ScryToggleButton { card_id: *card_id },
                        ))
                        .with_children(|cb| {
                            cb.spawn((
                                ImageNode { image: texture, ..default() },
                                Node {
                                    width: Val::Px(CARD_W - 12.0),
                                    height: Val::Px(CARD_H - 12.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cb.spawn((
                                Text::new(if *is_bottom { "Bottom" } else { "Top" }),
                                ui_fonts.tf(14.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });

                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for (label, delta, disabled) in
                                [("←", -1i32, at_left), ("→", 1, at_right)]
                            {
                                r.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if disabled {
                                        REORDER_BG_DISABLED
                                    } else {
                                        REORDER_BG
                                    }),
                                    ScryReorderButton {
                                        card_id: *card_id,
                                        delta,
                                    },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(if disabled {
                                            theme::TEXT_MUTED
                                        } else {
                                            theme::TEXT_PRIMARY
                                        }),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// CR 603.3b — modal letting the viewer order their own simultaneous
/// triggers. Renders each trigger as a card tile with ← / → reorder
/// buttons; index 0 (leftmost) is pushed onto the stack first, so the
/// rightmost trigger resolves first (LIFO). Confirm submits the order.
fn spawn_order_triggers_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    ordered: &[(CardId, String)],
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(20.0)),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let n = ordered.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(
                "Order your triggers  ·  ← → to reorder  ·  rightmost resolves first",
            ),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (i, (source, name)) in ordered.iter().enumerate() {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    let at_left = i == 0;
                    let at_right = i == n - 1;

                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(CARD_W),
                                padding: UiRect::all(Val::Px(6.0)),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(MODAL_TILE_BG),
                        ))
                        .with_children(|cb| {
                            cb.spawn((
                                ImageNode { image: texture, ..default() },
                                Node {
                                    width: Val::Px(CARD_W - 12.0),
                                    height: Val::Px(CARD_H - 12.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cb.spawn((
                                Text::new(format!("{}.", i + 1)),
                                ui_fonts.tf(14.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });

                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for (label, delta, disabled) in
                                [("←", -1i32, at_left), ("→", 1, at_right)]
                            {
                                r.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if disabled {
                                        REORDER_BG_DISABLED
                                    } else {
                                        REORDER_BG
                                    }),
                                    TriggerReorderButton { source: *source, delta },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(if disabled {
                                            theme::TEXT_MUTED
                                        } else {
                                            theme::TEXT_PRIMARY
                                        }),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

fn spawn_search_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    candidates: &[(CardId, String)],
) {
    // Search candidates are typically the entire library (60 cards). The
    // generic CARD_W of 180px would overflow the viewport vertically; use
    // a compact size for this dialog and scroll the grid when it spills.
    const SEARCH_CARD_W: f32 = 110.0;
    let search_card_h = SEARCH_CARD_W * CARD_ASPECT_RATIO;

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(85.0),
                max_height: Val::Percent(90.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let count = candidates.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "Search your library — click a card to select it ({count} card{s})",
                s = if count == 1 { "" } else { "s" }
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        // Scrollable grid: bounded height + Overflow::scroll_y so the
        // mouse wheel pages through long candidate lists. Without the
        // bound the panel sizes itself to the children and overflows the
        // window.
        panel
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::FlexStart,
                    max_height: Val::Vh(70.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                Pickable::default(),
            ))
            .with_children(|row| {
                for (card_id, name) in candidates {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    row.spawn((
                        Button,
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(SEARCH_CARD_W),
                            padding: UiRect::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(MODAL_TILE_BG),
                        SearchSelectButton { card_id: *card_id },
                    ))
                    .with_children(|cb| {
                        cb.spawn((
                            ImageNode { image: texture, ..default() },
                            Node {
                                width: Val::Px(SEARCH_CARD_W - 8.0),
                                height: Val::Px(search_card_h - 8.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        cb.spawn((
                            Text::new(name.clone()),
                            ui_fonts.tf(10.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Discard picker. Used in two cases that share the same wire format:
///
/// - **Self-discard** (Charging Strifeknight, Faithless Looting, Frantic
///   Search, etc.) — `Effect::Discard { random: false }`. The modal shows
///   the player's own hand.
/// - **Inquisition / Thoughtseize** — `Effect::DiscardChosen`. The caster
///   sees the target opponent's hand and picks for them.
fn spawn_discard_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    candidates: &[(CardId, String)],
    count: u32,
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(16.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(90.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!("Choose {count} card(s) to discard")),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|row| {
                for (card_id, name) in candidates {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    row.spawn((
                        Button,
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(CARD_W),
                            padding: UiRect::all(Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(MODAL_TILE_BG),
                        DiscardSelectButton { card_id: *card_id },
                    ))
                    .with_children(|cb| {
                        cb.spawn((
                            ImageNode { image: texture, ..default() },
                            Node {
                                width: Val::Px(CARD_W - 12.0),
                                height: Val::Px(CARD_H - 12.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        cb.spawn((
                            Text::new(name.clone()),
                            ui_fonts.tf(12.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });
        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Compact PutOnLibrary banner. Card selection happens by clicking the 3D
/// hand cards directly; this banner shows the prompt, the running count of
/// selected cards, and a Confirm button.
fn spawn_put_on_library_modal(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    _hand: &[(CardId, String)],
    count: usize,
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Pass-through transparent overlay so the 3D hand cards remain
            // clickable through the unfilled regions of the root.
            bevy::picking::Pickable::IGNORE,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "Click {count} card{} from your hand to put on the bottom of your library."
                ,
                if count == 1 { "" } else { "s" }
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        panel.spawn((
            Text::new(format!("0 / {count} selected")),
            ui_fonts.tf(14.0),
            TextColor(theme::ACCENT_GOLD),
            PutOnLibraryCountText,
        ));

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(16.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Marker for the live "X / N selected" text inside the PutOnLibrary banner.
#[derive(Component)]
pub struct PutOnLibraryCountText;

/// One Serum-Powder-style helper button. Carries the powder card's ID so
/// the click handler can submit `DecisionAnswer::SerumPowder(id)`.
#[derive(Component, Debug, Clone, Copy)]
pub struct MulliganSerumPowderButton(pub CardId);

/// Compact mulligan banner. The hand itself is rendered in 3D on the table —
/// this banner just shows the prompt and the Keep / Mulligan / Serum Powder
/// buttons. `serum_powders` is one CardId per Serum-Powder-style helper
/// currently in hand; renders one button each.
fn spawn_mulligan_modal(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    _hand: &[(CardId, String)],
    mulligans_taken: usize,
    serum_powders: &[CardId],
) {
    let root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // Transparent overlay — the 3D hand stays visible AND clickable
        // through the empty regions of the root. Pickable::IGNORE makes the
        // root pass-through; the panel below has BackgroundColor and so
        // re-acquires picking just where its rect is.
        bevy::picking::Pickable::IGNORE,
        DecisionModal,
    )).id();

    let panel = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(theme::PANEL_BG),
    )).id();
    commands.entity(root).add_child(panel);

    let title = if mulligans_taken == 0 {
        "Keep this opening hand?".to_string()
    } else {
        format!("Mulligan {mulligans_taken} — keep this hand?")
    };

    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title), ui_fonts.tf(18.0), TextColor(theme::TEXT_PRIMARY)));
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() })
        .with_children(|btns| {
            btns.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                MulliganKeepButton,
            ))
            .with_children(|b| { b.spawn((Text::new("Keep (K)"), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY))); });

            btns.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_DANGER_BG),
                HoverTint::new(theme::BUTTON_DANGER_BG),
                MulliganTakeButton,
            ))
            .with_children(|b| { b.spawn((Text::new("Mulligan (M)"), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY))); });

            // One button per Serum-Powder-style helper currently in hand.
            // Clicking submits DecisionAnswer::SerumPowder(id) — exiles the
            // hand and deals a fresh seven without bumping the mulligan
            // ladder. Multiple powders in hand stack as separate buttons.
            for (idx, powder_id) in serum_powders.iter().enumerate() {
                let label = if serum_powders.len() == 1 {
                    "Serum Powder".to_string()
                } else {
                    format!("Serum Powder #{}", idx + 1)
                };
                btns.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(REORDER_BG),
                    MulliganSerumPowderButton(*powder_id),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });
            }
        });
    });
}

/// Handle clicks on put-on-library candidate cards: add/remove from ordered selection.
#[allow(clippy::type_complexity)]
pub fn handle_put_on_library_select(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<
        (&Interaction, &PutOnLibrarySelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let Some(cv) = &view.0 else { return };
    let required_count = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
        Some(DecisionWire::PutOnLibrary { count, .. }) => *count,
        _ => return,
    };

    for (interaction, btn, mut bg) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        let id = btn.card_id;
        if let Some(pos) = state.put_on_library.iter().position(|&x| x == id) {
            state.put_on_library.remove(pos);
            *bg = BackgroundColor(MODAL_TILE_BG);
        } else if state.put_on_library.len() < required_count {
            state.put_on_library.push(id);
            *bg = BackgroundColor(theme::BUTTON_SELECTED_BG);
        }
    }
}

/// Handle clicks on Inquisition/Thoughtseize discard candidate cards.
/// Toggles inclusion in the selection up to `count` cards (taken from
/// the live `DecisionWire::Discard` count). Selected cards highlight in
/// `theme::BUTTON_SELECTED_BG`; clicking a selected card unselects it.
#[allow(clippy::type_complexity)]
pub fn handle_discard_select(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<
        (&Interaction, &DiscardSelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let required_count = match view
        .0
        .as_ref()
        .and_then(|v| v.pending_decision.as_ref())
        .and_then(|p| p.decision.as_ref())
    {
        Some(DecisionWire::Discard { count, .. }) => *count as usize,
        _ => return,
    };
    for (interaction, btn, mut bg) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let id = btn.card_id;
        if let Some(pos) = state.discard_selected.iter().position(|&x| x == id) {
            state.discard_selected.remove(pos);
            *bg = BackgroundColor(MODAL_TILE_BG);
        } else if state.discard_selected.len() < required_count {
            state.discard_selected.push(id);
            *bg = BackgroundColor(theme::BUTTON_SELECTED_BG);
        }
    }
}

/// Handle clicks on search candidate cards: highlight the selected card and
/// clear the highlight on whichever card was previously selected. The model
/// only carries `Option<CardId>`, so without resetting the prior button the
/// UI would show every clicked card as still selected even though only the
/// latest click counts.
#[allow(clippy::type_complexity)]
pub fn handle_search_select(
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<(&Interaction, &SearchSelectButton, &mut BackgroundColor), With<Button>>,
) {
    let pressed = buttons
        .iter()
        .find_map(|(i, btn, _)| (*i == Interaction::Pressed).then_some(btn.card_id));
    let Some(picked) = pressed else { return };
    state.search_selected = Some(picked);
    for (_, btn, mut bg) in buttons.iter_mut() {
        *bg = BackgroundColor(if btn.card_id == picked {
            theme::BUTTON_SELECTED_BG
        } else {
            MODAL_TILE_BG
        });
    }
}

/// Handle clicks on the scry toggle buttons: flip the card's Top/Bottom state
/// and update its label + background color.
#[allow(clippy::type_complexity)]
pub fn handle_scry_toggles(
    mut state: ResMut<DecisionUiState>,
    mut toggles: Query<
        (&Interaction, &ScryToggleButton, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut texts: Query<&mut Text>,
) {
    for (interaction, button, mut bg, children) in toggles.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(entry) = state.scry.iter_mut().find(|(id, _)| *id == button.card_id) else {
            continue;
        };
        entry.1 = !entry.1;
        let going_bottom = entry.1;
        *bg = BackgroundColor(if going_bottom { theme::BUTTON_SELECTED_BG } else { MODAL_TILE_BG });
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = if going_bottom { "Bottom".into() } else { "Top".into() };
            }
        }
    }
}

/// Handle clicks on ← / → reorder buttons: swap the card in the ordering and
/// respawn the modal to reflect the new positions.
pub fn handle_scry_reorder(
    mut state: ResMut<DecisionUiState>,
    query: Query<(&Interaction, &ScryReorderButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(pos) = state.scry.iter().position(|(id, _)| *id == btn.card_id) else {
            continue;
        };
        let new_pos =
            (pos as i32 + btn.delta).clamp(0, state.scry.len() as i32 - 1) as usize;
        if new_pos != pos {
            state.scry.swap(pos, new_pos);
        }
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle clicks on the OrderTriggers ← / → buttons: swap the trigger in
/// the push ordering and respawn the modal to reflect the new positions.
pub fn handle_trigger_reorder(
    mut state: ResMut<DecisionUiState>,
    query: Query<(&Interaction, &TriggerReorderButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(pos) = state.trigger_order.iter().position(|id| *id == btn.source) else {
            continue;
        };
        let new_pos =
            (pos as i32 + btn.delta).clamp(0, state.trigger_order.len() as i32 - 1) as usize;
        if new_pos != pos {
            state.trigger_order.swap(pos, new_pos);
        }
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle the Confirm button: build the appropriate answer based on which
/// decision is pending and submit it to the server via NetOutbox.
pub fn handle_confirm(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut log: ResMut<GameLog>,
    mut state: ResMut<DecisionUiState>,
    confirm: Query<&Interaction, (Changed<Interaction>, With<DecisionConfirmButton>)>,
) {
    for interaction in &confirm {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(cv) = &view.0 else { continue };
        let wire = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
            Some(d) => d,
            None => continue,
        };

        let answer = match wire {
            DecisionWire::Scry { .. } => {
                let mut kept_top = Vec::new();
                let mut bottom = Vec::new();
                for (id, going_bottom) in &state.scry {
                    if *going_bottom { bottom.push(*id); } else { kept_top.push(*id); }
                }
                DecisionAnswer::ScryOrder { kept_top, bottom }
            }
            DecisionWire::SearchLibrary { .. } => {
                DecisionAnswer::Search(state.search_selected)
            }
            DecisionWire::PutOnLibrary { count, .. } => {
                if state.put_on_library.len() < *count { continue; }
                DecisionAnswer::PutOnLibrary(state.put_on_library.clone())
            }
            DecisionWire::Discard { count, .. } => {
                if state.discard_selected.len() < *count as usize { continue; }
                DecisionAnswer::Discard(state.discard_selected.clone())
            }
            DecisionWire::OrderTriggers { .. } => {
                DecisionAnswer::TriggerOrder(state.trigger_order.clone())
            }
            _ => continue,
        };

        if let Some(outbox) = &outbox {
            outbox.submit(GameAction::SubmitDecision(answer));
        }
        log.push("Decision submitted.");
        state.scry.clear();
        state.search_selected = None;
        state.put_on_library.clear();
        state.discard_selected.clear();
        state.trigger_order.clear();
        state.spawned_for = None;
    }
}

/// Update the live "X / N selected" text in the PutOnLibrary banner.
pub fn update_put_on_library_count_text(
    view: Res<CurrentView>,
    state: Res<DecisionUiState>,
    mut q: Query<&mut Text, With<PutOnLibraryCountText>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(count) = cv.pending_decision.as_ref().and_then(|p| match p.decision.as_ref() {
        Some(DecisionWire::PutOnLibrary { count, .. }) => Some(*count),
        _ => None,
    }) else {
        return;
    };
    let selected = state.put_on_library.len();
    for mut t in &mut q {
        t.0 = format!("{selected} / {count} selected");
    }
}

/// Sync the `PutOnLibraryHighlight` marker component on the viewer's hand
/// cards to mirror `state.put_on_library`. Spawns/despawns gold border child
/// meshes so the player can see which cards are currently selected to bottom.
#[allow(clippy::type_complexity)]
pub fn update_put_on_library_visuals(
    mut commands: Commands,
    state: Res<DecisionUiState>,
    view: Res<CurrentView>,
    highlight_assets: Option<Res<crate::card::CardHighlightAssets>>,
    hand_cards: Query<
        (Entity, &crate::card::GameCardId, Option<&PutOnLibraryHighlight>),
        With<crate::card::HandCard>,
    >,
) {
    // Active only while a PutOnLibrary decision is pending for the viewer.
    let active = view
        .0
        .as_ref()
        .and_then(|cv| {
            let pd = cv.pending_decision.as_ref()?;
            if pd.acting_player != cv.your_seat {
                return None;
            }
            match pd.decision.as_ref()? {
                DecisionWire::PutOnLibrary { .. } => Some(()),
                _ => None,
            }
        })
        .is_some();

    let Some(assets) = highlight_assets else { return };
    let selected: std::collections::HashSet<CardId> = if active {
        state.put_on_library.iter().copied().collect()
    } else {
        std::collections::HashSet::new()
    };

    for (entity, gid, marker) in &hand_cards {
        let should_be_selected = selected.contains(&gid.0);
        match (should_be_selected, marker) {
            (true, None) => {
                let offset = crate::card::CARD_THICKNESS / 2.0 + 0.0015;
                let back = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.border_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -offset),
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                let front = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.border_material.clone()),
                        Transform::from_xyz(0.0, 0.0, offset),
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert(PutOnLibraryHighlight { back, front })
                    .add_children(&[back, front]);
            }
            (false, Some(highlight)) => {
                commands.entity(highlight.back).despawn();
                commands.entity(highlight.front).despawn();
                commands.entity(entity).remove::<PutOnLibraryHighlight>();
            }
            _ => {}
        }
    }
}

/// Click on a 3D hand card while a PutOnLibrary decision is pending: toggle
/// the card's membership in `state.put_on_library`.
pub fn handle_put_on_library_hand_click(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mouse: Res<bevy::input::ButtonInput<MouseButton>>,
    hovered_hand: Query<&crate::card::GameCardId, (With<crate::card::CardHovered>, With<crate::card::HandCard>)>,
) {
    let Some(cv) = &view.0 else { return };
    let pending = match cv.pending_decision.as_ref() {
        Some(p) if p.acting_player == cv.your_seat => p,
        _ => return,
    };
    let Some(DecisionWire::PutOnLibrary { count, .. }) = pending.decision.as_ref() else { return };
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(gid) = hovered_hand.iter().next() else { return };
    let id = gid.0;
    if let Some(pos) = state.put_on_library.iter().position(|x| *x == id) {
        state.put_on_library.remove(pos);
    } else if state.put_on_library.len() < *count {
        state.put_on_library.push(id);
    }
}

/// Handle Keep / Mulligan / Serum Powder button presses (and keyboard
/// shortcuts K / M / P for the first-listed powder).
pub fn handle_mulligan_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    keep_q: Query<&Interaction, (Changed<Interaction>, With<MulliganKeepButton>)>,
    mull_q: Query<&Interaction, (Changed<Interaction>, With<MulliganTakeButton>)>,
    powder_q: Query<
        (&Interaction, &MulliganSerumPowderButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let Some(cv) = &view.0 else { return };
    let serum_powders = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
        Some(DecisionWire::Mulligan { serum_powders, .. }) => serum_powders.clone(),
        _ => return,
    };
    let Some(outbox) = outbox else { return };

    let keep = keep_q.iter().any(|i| *i == Interaction::Pressed) || keyboard.just_pressed(KeyCode::KeyK);
    let mull = mull_q.iter().any(|i| *i == Interaction::Pressed) || keyboard.just_pressed(KeyCode::KeyM);
    let pressed_powder = powder_q
        .iter()
        .find_map(|(int, btn)| (*int == Interaction::Pressed).then_some(btn.0))
        .or_else(|| {
            // P shortcut → consume the first listed powder.
            (keyboard.just_pressed(KeyCode::KeyP)).then(|| serum_powders.first().copied()).flatten()
        });

    if keep {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Keep));
        state.spawned_for = None;
    } else if mull {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::TakeMulligan));
        state.spawned_for = None;
    } else if let Some(id) = pressed_powder {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::SerumPowder(id)));
        state.spawned_for = None;
    }
}

// ── ChooseColor modal (Black Lotus / Birds of Paradise) ─────────────────────

#[derive(Component)]
pub struct ChooseColorButton(pub crabomination::mana::Color);

fn spawn_choose_color_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    legal: &[crabomination::mana::Color],
) {
    use crabomination::mana::Color as ManaColor;
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(14.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Choose a color"),
            ui_fonts.tf(18.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            for c in legal {
                let (bg, label) = match c {
                    ManaColor::White => (Color::srgb(0.92, 0.92, 0.78), "White"),
                    ManaColor::Blue  => (Color::srgb(0.30, 0.55, 0.90), "Blue"),
                    ManaColor::Black => (Color::srgb(0.20, 0.20, 0.24), "Black"),
                    ManaColor::Red   => (Color::srgb(0.85, 0.28, 0.20), "Red"),
                    ManaColor::Green => (Color::srgb(0.25, 0.60, 0.30), "Green"),
                };
                let text_color = if matches!(c, ManaColor::White) {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    ChooseColorButton(*c),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        ui_fonts.tf(14.0),
                        TextColor(text_color),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        });
    });
}

// ── Learn modal (Lessons sideboard) ─────────────────────────────────────────

#[derive(Component)]
pub struct LearnFetchButton(pub CardId);
#[derive(Component)]
pub struct LearnRummageButton(pub CardId);
#[derive(Component)]
pub struct LearnDeclineButton;

fn spawn_learn_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    lessons: &[(CardId, String)],
    hand: &[(CardId, String)],
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Stretch,
                min_width: Val::Px(280.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    // A labelled, full-width pick button. Returns it so the caller tags it
    // with the right marker component.
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Learn — reveal a Lesson, or discard a card to draw"),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        p.spawn((
            Text::new("Reveal a Lesson into your hand:"),
            ui_fonts.tf(13.0),
            TextColor(theme::TEXT_SECONDARY),
        ));
        for (id, name) in lessons {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.45, 0.30)),
                LearnFetchButton(*id),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(name.clone()),
                    ui_fonts.tf(14.0),
                    TextColor(Color::WHITE),
                    bevy::picking::Pickable::IGNORE,
                ));
            });
        }

        if !hand.is_empty() {
            p.spawn((
                Text::new("Or discard a card to draw:"),
                ui_fonts.tf(13.0),
                TextColor(theme::TEXT_SECONDARY),
            ));
            for (id, name) in hand {
                p.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.40, 0.30, 0.20)),
                    LearnRummageButton(*id),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(format!("Discard {name}")),
                        ui_fonts.tf(13.0),
                        TextColor(Color::WHITE),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        }

        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(Color::srgb(0.30, 0.30, 0.34)),
            LearnDeclineButton,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Decline"),
                ui_fonts.tf(14.0),
                TextColor(Color::WHITE),
                bevy::picking::Pickable::IGNORE,
            ));
        });
    });
}

/// Click handler for the Learn modal's three action types. Submits the
/// chosen `LearnChoice` as the pending decision's answer.
pub fn handle_learn_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    fetch: Query<(&Interaction, &LearnFetchButton), Changed<Interaction>>,
    rummage: Query<(&Interaction, &LearnRummageButton), Changed<Interaction>>,
    decline: Query<&Interaction, (Changed<Interaction>, With<LearnDeclineButton>)>,
) {
    use crabomination::decision::LearnChoice;
    let Some(cv) = &view.0 else { return };
    if !matches!(
        cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()),
        Some(DecisionWire::Learn { .. })
    ) {
        return;
    }
    let Some(outbox) = outbox else { return };
    let submit = |choice: LearnChoice, state: &mut DecisionUiState| {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Learn(choice)));
        state.spawned_for = None;
    };
    for (interaction, btn) in &fetch {
        if *interaction == Interaction::Pressed {
            submit(LearnChoice::FetchLesson(btn.0), &mut state);
            return;
        }
    }
    for (interaction, btn) in &rummage {
        if *interaction == Interaction::Pressed {
            submit(LearnChoice::Rummage { discard: btn.0 }, &mut state);
            return;
        }
    }
    for interaction in &decline {
        if *interaction == Interaction::Pressed {
            submit(LearnChoice::Decline, &mut state);
            return;
        }
    }
}

// ── Mode pick (modal "Choose one —" spells) ───────────────────────────────

/// Marker on a modal-spell `Modes:` panel; despawned together when the
/// pick is submitted or cancelled.
#[derive(Component)]
pub struct ModalCastModal;

#[derive(Component)]
pub struct ModalCastButton(pub usize);

#[derive(Component)]
pub struct ModalCastCancel;

/// Spawn / despawn the "Choose one —" picker for modal spells. Driven by
/// the [`crate::game::PendingModalCast`] resource: when its `card_id`
/// becomes `Some`, a modal lists every mode's short description; clicking
/// a button either casts immediately (mode has no target) or arms the
/// targeting cursor with `pending_mode` set.
pub fn spawn_mode_pick_ui(
    mut commands: Commands,
    pending: Res<crate::game::PendingModalCast>,
    existing: Query<Entity, With<ModalCastModal>>,
    ui_fonts: Res<UiFonts>,
) {
    if pending.card_id.is_none() {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }
    if existing.iter().next().is_some() {
        return;
    }
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            ModalCastModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Stretch,
                min_width: Val::Px(360.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);
    let name = pending.card_name.clone();
    let modes = pending.modes.clone();
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(format!("{name} — choose one")),
            ui_fonts.tf(18.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        for (idx, (desc, needs_target)) in modes.iter().enumerate() {
            let label = if desc.is_empty() {
                format!("Mode {}", idx + 1)
            } else if *needs_target {
                format!("{}. {} (pick target)", idx + 1, desc)
            } else {
                format!("{}. {}", idx + 1, desc)
            };
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                ModalCastButton(idx),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(label),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        }
        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_TERTIARY_BG),
            HoverTint::new(theme::BUTTON_TERTIARY_BG),
            ModalCastCancel,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Cancel"),
                ui_fonts.tf(12.0),
                TextColor(theme::TEXT_SECONDARY),
                Pickable::IGNORE,
            ));
        });
    });
}

pub fn handle_mode_pick_buttons(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<crate::game::PendingModalCast>,
    mut targeting: ResMut<crate::game::TargetingState>,
    mut legal_targets: ResMut<crate::game::LegalTargets>,
    mut esc_consumed: ResMut<crate::systems::quality::EscConsumed>,
    btns: Query<(&Interaction, &ModalCastButton), Changed<Interaction>>,
    cancels: Query<&Interaction, (Changed<Interaction>, With<ModalCastCancel>)>,
) {
    if pending.card_id.is_none() {
        return;
    }
    // Esc dismisses the modal pick — sibling to the Cancel button.
    // Eat the keypress via `EscConsumed` so the same Esc doesn't also
    // close the settings panel / trigger any other Esc-bound action.
    if keyboard.just_pressed(KeyCode::Escape) {
        pending.card_id = None;
        pending.card_name.clear();
        pending.modes.clear();
        esc_consumed.0 = true;
        return;
    }
    for i in &cancels {
        if *i == Interaction::Pressed {
            pending.card_id = None;
            pending.card_name.clear();
            pending.modes.clear();
            return;
        }
    }
    let Some(outbox) = outbox else { return };
    for (i, btn) in &btns {
        if *i != Interaction::Pressed {
            continue;
        }
        let idx = btn.0;
        let needs_target = pending.modes.get(idx).map(|(_, n)| *n).unwrap_or(false);
        let card_id = pending.card_id.unwrap();
        if needs_target {
            targeting.active = true;
            targeting.pending_card_id = Some(card_id);
            targeting.pending_mode = Some(idx);
            targeting.back_face_pending = false;
            // Populate the highlight set from the chosen mode's slot-0
            // filter so the user sees rings on legal creatures (the
            // earlier path left it empty, which was the source of the
            // "highlights players but not creatures" bug).
            if let (Some(cv), name) = (&view.0, pending.card_name.clone())
                && let Some(legal) = crate::systems::legal_target_filter::enumerate_for_cast(
                    cv,
                    &name,
                    Some(idx),
                )
            {
                *legal_targets = legal;
            }
        } else {
            outbox.submit(GameAction::CastSpell {
                card_id,
                target: None,
                additional_targets: vec![],
                mode: Some(idx),
                x_value: None,
            });
        }
        pending.card_id = None;
        pending.card_name.clear();
        pending.modes.clear();
        return;
    }
}

/// Click handler for color buttons in the `ChooseColor` modal. Submits the
/// chosen color as the pending decision's answer.
pub fn handle_choose_color_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &ChooseColorButton), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    let is_color = matches!(
        cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()),
        Some(DecisionWire::ChooseColor { .. })
    );
    if !is_color {
        return;
    }
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Color(btn.0)));
            state.spawned_for = None;
            return;
        }
    }
}
