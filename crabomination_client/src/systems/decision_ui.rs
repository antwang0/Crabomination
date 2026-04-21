//! UI for pending player-choice decisions (scry, color choice, searches…).
//!
//! The engine surfaces a `PendingDecision` on `GameState` when resolution
//! needs input. This module renders a modal for P0's decisions and submits
//! the answer via `GameAction::SubmitDecision`. The pattern is dispatched on
//! the `Decision` variant so new decision kinds slot in without touching the
//! surrounding plumbing.

use bevy::prelude::*;

use crabomination::{
    card::CardId,
    decision::{Decision, DecisionAnswer},
    game::GameAction,
};

use crate::game::{GameLog, GameResource, PLAYER_0};
use crate::scryfall;

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

#[derive(Component)]
pub struct SearchSelectButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct PutOnLibrarySelectButton {
    pub card_id: CardId,
}

/// Local UI state tracked during an in-flight decision. Cleared when the
/// engine's `pending_decision` goes back to `None`.
#[derive(Resource, Default)]
pub struct DecisionUiState {
    /// For Scry: per-card "send to bottom" flags (false = keep on top).
    /// Order of this vec is the player's chosen ordering (top-left = top of library).
    pub scry: Vec<(CardId, bool)>,
    /// For SearchLibrary: the card the player selected (None = failed search).
    pub search_selected: Option<CardId>,
    /// For PutOnLibrary: ordered list of selected card IDs (index 0 = topmost).
    pub put_on_library: Vec<CardId>,
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
}

fn decision_key(decision: &Decision) -> Option<DecisionKey> {
    match decision {
        Decision::Scry { cards, .. } => Some(DecisionKey::Scry(
            cards.iter().map(|(id, _)| *id).collect(),
        )),
        Decision::SearchLibrary { candidates, .. } => Some(DecisionKey::Search(
            candidates.iter().map(|(id, _)| *id).collect(),
        )),
        Decision::PutOnLibrary { hand, .. } => Some(DecisionKey::PutOnLibrary(
            hand.iter().map(|(id, _)| *id).collect(),
        )),
        _ => None,
    }
}

const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.97);
const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
const CARD_ASPECT_RATIO: f32 = 88.0 / 63.0;
const CARD_W: f32 = 180.0;
const CARD_H: f32 = CARD_W * CARD_ASPECT_RATIO;
const BTN_BG_OFF: Color = Color::srgba(0.20, 0.20, 0.24, 0.95);
const BTN_BG_ON: Color = Color::srgba(0.60, 0.25, 0.25, 0.95);
const CONFIRM_BG: Color = Color::srgba(0.20, 0.45, 0.25, 0.98);
const REORDER_BG: Color = Color::srgba(0.25, 0.30, 0.40, 0.95);
const REORDER_BG_DISABLED: Color = Color::srgba(0.15, 0.15, 0.18, 0.6);

/// Spawn or despawn the decision modal based on the engine state. Only shows
/// for decisions owned by P0; P1 (bot) answers are handled in the bot module.
pub fn spawn_decision_ui(
    mut commands: Commands,
    game: Res<GameResource>,
    mut state: ResMut<DecisionUiState>,
    existing: Query<Entity, With<DecisionModal>>,
    asset_server: Res<AssetServer>,
) {
    let pending = match &game.state.pending_decision {
        Some(pd) if pd.acting_player() == PLAYER_0 => pd,
        _ => {
            // No P0 decision — tear down any existing modal.
            for e in &existing {
                commands.entity(e).despawn();
            }
            if state.spawned_for.is_some() {
                state.scry.clear();
                state.search_selected = None;
                state.put_on_library.clear();
                state.spawned_for = None;
            }
            return;
        }
    };

    let key = match decision_key(&pending.decision) {
        Some(k) => k,
        None => return, // unsupported decision type; bot/auto will handle
    };

    if state.spawned_for.as_ref() == Some(&key) {
        return; // already up for this exact decision
    }

    // Fresh decision or respawn after reorder — despawn old modal, spawn new one.
    for e in &existing {
        commands.entity(e).despawn();
    }

    match &pending.decision {
        Decision::Scry { cards, .. } => {
            if state.scry.is_empty() {
                state.scry = cards.iter().map(|(id, _)| (*id, false)).collect();
            }
            state.spawned_for = Some(key);
            let name_map: std::collections::HashMap<CardId, &'static str> =
                cards.iter().cloned().collect();
            let ordered: Vec<(CardId, &'static str, bool)> = state
                .scry
                .iter()
                .map(|(id, bottom)| (*id, name_map[id], *bottom))
                .collect();
            spawn_scry_modal(&mut commands, &asset_server, &ordered);
        }
        Decision::SearchLibrary { candidates, .. } => {
            state.search_selected = None;
            state.spawned_for = Some(key);
            spawn_search_modal(&mut commands, &asset_server, candidates);
        }
        Decision::PutOnLibrary { count, hand, .. } => {
            state.put_on_library.clear();
            state.spawned_for = Some(key);
            spawn_put_on_library_modal(&mut commands, &asset_server, hand, *count);
        }
        _ => {}
    }
}

fn spawn_scry_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ordered: &[(CardId, &'static str, bool)],
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
            BackgroundColor(OVERLAY_BG),
            // Overlay absorbs clicks so nothing behind it is accidentally clicked.
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
                ..default()
            },
            BackgroundColor(PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let n = ordered.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "Scry {n}: click card to toggle Bottom  ·  ← → to reorder  ·  left = top of library"
            )),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));

        // Row of card columns.
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

                    // Card column — not a button; children handle interactions.
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        // Clickable card face — toggles Top/Bottom.
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
                            BackgroundColor(if *is_bottom { BTN_BG_ON } else { BTN_BG_OFF }),
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
                                TextFont { font_size: 14.0, ..default() },
                                TextColor(Color::WHITE),
                                Pickable::IGNORE,
                            ));
                        });

                        // Reorder row — ← and → are siblings of the card, not children.
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
                                        TextFont { font_size: 16.0, ..default() },
                                        TextColor(if disabled {
                                            Color::srgba(0.5, 0.5, 0.5, 0.6)
                                        } else {
                                            Color::WHITE
                                        }),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        // Confirm button.
        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(CONFIRM_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
    });
}

fn spawn_search_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    candidates: &[(CardId, &'static str)],
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
            BackgroundColor(OVERLAY_BG),
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
                ..default()
            },
            BackgroundColor(PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new("Search your library — click a card to select it"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
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
                        BackgroundColor(BTN_BG_OFF),
                        SearchSelectButton { card_id: *card_id },
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
                            Text::new(*name),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });

        // Confirm button (disabled look until a card is selected).
        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(CONFIRM_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
    });
}

fn spawn_put_on_library_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    hand: &[(CardId, &'static str)],
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
            BackgroundColor(OVERLAY_BG),
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
                ..default()
            },
            BackgroundColor(PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!("Choose {count} card(s) to put on top of your library (first chosen = top)")),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
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
                for (card_id, name) in hand {
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
                        BackgroundColor(BTN_BG_OFF),
                        PutOnLibrarySelectButton { card_id: *card_id },
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
                            Text::new(*name),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::WHITE),
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
                    ..default()
                },
                BackgroundColor(CONFIRM_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Handle clicks on put-on-library candidate cards: add/remove from ordered selection.
#[allow(clippy::type_complexity)]
pub fn handle_put_on_library_select(
    game: Res<GameResource>,
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<
        (&Interaction, &PutOnLibrarySelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let required_count = match game.state.pending_decision.as_ref().map(|p| &p.decision) {
        Some(Decision::PutOnLibrary { count, .. }) => *count,
        _ => return,
    };

    for (interaction, btn, mut bg) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        let id = btn.card_id;
        if let Some(pos) = state.put_on_library.iter().position(|&x| x == id) {
            // Deselect: remove from list.
            state.put_on_library.remove(pos);
            *bg = BackgroundColor(BTN_BG_OFF);
        } else if state.put_on_library.len() < required_count {
            // Select: add to end of ordered list.
            state.put_on_library.push(id);
            *bg = BackgroundColor(BTN_BG_ON);
        }
    }
}

/// Handle clicks on search candidate cards: highlight the selected card.
#[allow(clippy::type_complexity)]
pub fn handle_search_select(
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<
        (&Interaction, &SearchSelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, btn, mut bg) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        state.search_selected = Some(btn.card_id);
        *bg = BackgroundColor(BTN_BG_ON);
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
        *bg = BackgroundColor(if going_bottom { BTN_BG_ON } else { BTN_BG_OFF });
        // Update the child Text ("Top" / "Bottom").
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
        // Despawn modal and clear spawned_for so spawn_decision_ui respawns it next frame.
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle the Confirm button: build the appropriate answer based on which
/// decision is pending and submit it to the engine.
pub fn handle_confirm(
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut state: ResMut<DecisionUiState>,
    confirm: Query<&Interaction, (Changed<Interaction>, With<DecisionConfirmButton>)>,
) {
    for interaction in &confirm {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(pending) = &game.state.pending_decision else { continue };

        let answer = match &pending.decision {
            Decision::Scry { .. } => {
                let mut kept_top = Vec::new();
                let mut bottom = Vec::new();
                for (id, going_bottom) in &state.scry {
                    if *going_bottom { bottom.push(*id); } else { kept_top.push(*id); }
                }
                DecisionAnswer::ScryOrder { kept_top, bottom }
            }
            Decision::SearchLibrary { .. } => {
                DecisionAnswer::Search(state.search_selected)
            }
            Decision::PutOnLibrary { count, .. } => {
                if state.put_on_library.len() < *count { continue; }
                DecisionAnswer::PutOnLibrary(state.put_on_library.clone())
            }
            _ => continue,
        };

        if let Ok(evs) = game.state.perform_action(GameAction::SubmitDecision(answer)) {
            log.apply_events(&evs);
        }
        state.scry.clear();
        state.search_selected = None;
        state.put_on_library.clear();
        state.spawned_for = None;
    }
}
