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
pub struct DecisionConfirmButton;

/// Local UI state tracked during an in-flight decision. Cleared when the
/// engine's `pending_decision` goes back to `None`.
#[derive(Resource, Default)]
pub struct DecisionUiState {
    /// For Scry: per-card "send to bottom" flags (false = keep on top).
    /// Order matches the peeked order from the engine (top of library first).
    pub scry: Vec<(CardId, bool)>,
    /// CardId the modal was last spawned for — avoids respawning each frame.
    pub spawned_for: Option<DecisionKey>,
}

/// Fingerprint of a pending decision. Used to detect when a new decision
/// arrived (so the modal respawns) vs. the same one still showing.
#[derive(Clone, PartialEq, Eq)]
pub enum DecisionKey {
    Scry(Vec<CardId>),
}

fn decision_key(decision: &Decision) -> Option<DecisionKey> {
    match decision {
        Decision::Scry { cards, .. } => Some(DecisionKey::Scry(
            cards.iter().map(|(id, _)| *id).collect(),
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

    // Fresh decision — despawn old modal, initialize state, spawn new modal.
    for e in &existing {
        commands.entity(e).despawn();
    }

    match &pending.decision {
        Decision::Scry { cards, .. } => {
            state.scry = cards.iter().map(|(id, _)| (*id, false)).collect();
            state.spawned_for = Some(key);
            spawn_scry_modal(&mut commands, &asset_server, cards);
        }
        _ => {}
    }
}

fn spawn_scry_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    cards: &[(CardId, &'static str)],
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

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!("Scry {}: click a card to toggle Bottom ↓", cards.len())),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
        ));

        // Row of card buttons.
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (card_id, name) in cards {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    row.spawn((
                        Button,
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(CARD_W),
                            padding: UiRect::all(Val::Px(6.0)),
                            row_gap: Val::Px(4.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BTN_BG_OFF),
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
                            Text::new("Top"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });

        // Confirm button.
        panel.spawn((
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

/// Handle clicks on the scry toggle buttons: flip the card's Top/Bottom state
/// and update its label + background color.
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

/// Handle the Confirm button: build a `ScryOrder` answer from the current
/// state and submit it to the engine.
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
        if game.state.pending_decision.is_none() {
            continue;
        }
        let mut kept_top = Vec::new();
        let mut bottom = Vec::new();
        for (id, going_bottom) in &state.scry {
            if *going_bottom {
                bottom.push(*id);
            } else {
                kept_top.push(*id);
            }
        }
        let answer = DecisionAnswer::ScryOrder { kept_top, bottom };
        if let Ok(evs) = game.state.perform_action(GameAction::SubmitDecision(answer)) {
            log.apply_events(&evs);
        }
        state.scry.clear();
        state.spawned_for = None;
    }
}
