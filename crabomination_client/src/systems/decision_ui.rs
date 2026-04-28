//! UI for pending player-choice decisions (scry, color choice, searches…).
//!
//! Reads the pending decision from `CurrentView` (server-projected) and
//! submits answers via `NetOutbox`. During the mulligan phase `CurrentView`
//! is empty and no modal is shown.

use bevy::prelude::*;

use crabomination::{
    card::CardId,
    decision::DecisionAnswer,
    game::GameAction,
    net::DecisionWire,
};

use crate::game::{GameLog};
use crate::net_plugin::{CurrentView, NetOutbox};
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

/// Spawn or despawn the decision modal based on the server view. Only shows
/// for decisions owned by P0 (your_seat).
pub fn spawn_decision_ui(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    existing: Query<Entity, With<DecisionModal>>,
    asset_server: Res<AssetServer>,
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
            spawn_scry_modal(&mut commands, &asset_server, &ordered);
        }
        DecisionWire::SearchLibrary { candidates, .. } => {
            state.search_selected = None;
            state.spawned_for = Some(key);
            spawn_search_modal(&mut commands, &asset_server, candidates);
        }
        DecisionWire::PutOnLibrary { count, hand, .. } => {
            state.put_on_library.clear();
            state.spawned_for = Some(key);
            spawn_put_on_library_modal(&mut commands, &asset_server, hand, *count);
        }
        DecisionWire::Mulligan { hand, mulligans_taken, serum_powders, .. } => {
            state.spawned_for = Some(key);
            spawn_mulligan_modal(
                &mut commands,
                &asset_server,
                hand,
                *mulligans_taken,
                serum_powders,
            );
        }
        DecisionWire::ChooseColor { legal, .. } => {
            state.spawned_for = Some(key);
            spawn_choose_color_modal(&mut commands, legal);
        }
        _ => {}
    }
}

fn spawn_scry_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
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
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(85.0),
                max_height: Val::Percent(90.0),
                ..default()
            },
            BackgroundColor(PANEL_BG),
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
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
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
                        BackgroundColor(BTN_BG_OFF),
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
                            TextFont { font_size: 10.0, ..default() },
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

/// Inquisition / Thoughtseize discard picker. Caster sees the target's
/// hand and clicks `count` cards to send to the graveyard.
fn spawn_discard_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
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
            Text::new(format!(
                "Choose {count} card(s) to discard from your opponent's hand"
            )),
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

/// Compact PutOnLibrary banner. Card selection happens by clicking the 3D
/// hand cards directly; this banner shows the prompt, the running count of
/// selected cards, and a Confirm button.
fn spawn_put_on_library_modal(
    commands: &mut Commands,
    _asset_server: &AssetServer,
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
                ..default()
            },
            BackgroundColor(PANEL_BG),
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
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        panel.spawn((
            Text::new(format!("0 / {count} selected")),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(1.0, 0.85, 0.4)),
            PutOnLibraryCountText,
        ));

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
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Marker for the live "X / N selected" text inside the PutOnLibrary banner.
#[derive(Component)]
pub struct PutOnLibraryCountText;

const KEEP_BG: Color = Color::srgba(0.15, 0.45, 0.18, 0.97);
const MULL_BG: Color = Color::srgba(0.48, 0.12, 0.10, 0.97);

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
        BackgroundColor(PANEL_BG),
    )).id();
    commands.entity(root).add_child(panel);

    let title = if mulligans_taken == 0 {
        "Keep this opening hand?".to_string()
    } else {
        format!("Mulligan {mulligans_taken} — keep this hand?")
    };

    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() })
        .with_children(|btns| {
            btns.spawn((
                Button,
                Node { padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)), ..default() },
                BackgroundColor(KEEP_BG),
                MulliganKeepButton,
            ))
            .with_children(|b| { b.spawn((Text::new("Keep (K)"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });

            btns.spawn((
                Button,
                Node { padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)), ..default() },
                BackgroundColor(MULL_BG),
                MulliganTakeButton,
            ))
            .with_children(|b| { b.spawn((Text::new("Mulligan (M)"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });

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
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
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
            *bg = BackgroundColor(BTN_BG_OFF);
        } else if state.put_on_library.len() < required_count {
            state.put_on_library.push(id);
            *bg = BackgroundColor(BTN_BG_ON);
        }
    }
}

/// Handle clicks on Inquisition/Thoughtseize discard candidate cards.
/// Toggles inclusion in the selection up to `count` cards (taken from
/// the live `DecisionWire::Discard` count). Selected cards highlight in
/// `BTN_BG_ON`; clicking a selected card unselects it.
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
            *bg = BackgroundColor(BTN_BG_OFF);
        } else if state.discard_selected.len() < required_count {
            state.discard_selected.push(id);
            *bg = BackgroundColor(BTN_BG_ON);
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
            BTN_BG_ON
        } else {
            BTN_BG_OFF
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
        *bg = BackgroundColor(if going_bottom { BTN_BG_ON } else { BTN_BG_OFF });
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
                ..default()
            },
            BackgroundColor(PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Choose a color"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
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
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(text_color),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        });
    });
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
