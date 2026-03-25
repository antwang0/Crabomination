use bevy::prelude::*;

use crate::card::{
    P1DeckPile, Card, CardBorderHighlight, CardFrontTexture, CardHighlightAssets, CardHovered,
    DeckCard, GraveyardPile, PileHovered, CARD_THICKNESS,
};
use crate::game::{GraveyardBrowserState, GameResource, PLAYER_1, PLAYER_0};

/// Tracks a pending top-card reveal popup.
#[derive(Resource, Default)]
pub struct RevealPopupState {
    /// Asset path of the card to show, or None when no popup is active.
    pub card_path: Option<String>,
    /// Which player's top card was revealed (sets the animation target).
    pub revealed_player: Option<usize>,
}
use crate::scryfall;

#[derive(Component)]
pub struct PeekPopup;

#[derive(Component)]
pub struct GraveyardBrowser;

#[derive(Component)]
pub struct RevealPopup;

#[derive(Component)]
pub struct PileTooltip;

pub fn highlight_hovered_cards(
    mut commands: Commands,
    added: Query<Entity, (With<Card>, Added<CardHovered>)>,
    mut removed: RemovedComponents<CardHovered>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    borders: Query<&CardBorderHighlight>,
) {
    let Some(assets) = highlight_assets else {
        return;
    };

    for card_entity in &added {
        let offset = CARD_THICKNESS / 2.0 + 0.001;
        let back_border = commands
            .spawn((
                Mesh3d(assets.border_mesh.clone()),
                MeshMaterial3d(assets.border_material.clone()),
                Transform::from_xyz(0.0, 0.0, -offset),
                Pickable::IGNORE,
            ))
            .id();
        let front_border = commands
            .spawn((
                Mesh3d(assets.border_mesh.clone()),
                MeshMaterial3d(assets.border_material.clone()),
                Transform::from_xyz(0.0, 0.0, offset),
                Pickable::IGNORE,
            ))
            .id();
        commands
            .entity(card_entity)
            .insert(CardBorderHighlight(back_border, front_border))
            .add_children(&[back_border, front_border]);
    }

    for card_entity in removed.read() {
        if let Ok(border) = borders.get(card_entity) {
            commands.entity(border.0).despawn();
            commands.entity(border.1).despawn();
            commands.entity(card_entity).remove::<CardBorderHighlight>();
        }
    }
}

/// Standard Magic card aspect ratio (height / width).
const CARD_ASPECT_RATIO: f32 = 88.0 / 63.0;
const POPUP_WIDTH: f32 = 340.0;
const POPUP_HEIGHT: f32 = POPUP_WIDTH * CARD_ASPECT_RATIO;

pub fn peek_popup(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    hovered_cards: Query<&CardFrontTexture, (With<Card>, With<CardHovered>)>,
    existing_popup: Query<Entity, With<PeekPopup>>,
    asset_server: Res<AssetServer>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let should_show = alt_held && !hovered_cards.is_empty();

    if should_show && existing_popup.is_empty() {
        if let Some(front_texture) = hovered_cards.iter().next() {
            let texture: Handle<Image> = asset_server.load(&front_texture.0);
            // Full-screen overlay, flex-centered, with dim background.
            commands
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
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    Pickable::IGNORE,
                    PeekPopup,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        ImageNode {
                            image: texture,
                            ..default()
                        },
                        Node {
                            width: Val::Px(POPUP_WIDTH),
                            height: Val::Px(POPUP_HEIGHT),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
    } else if !should_show {
        for entity in &existing_popup {
            commands.entity(entity).despawn();
        }
    }
}

const BROWSER_CARD_WIDTH: f32 = 220.0;
const BROWSER_CARD_HEIGHT: f32 = BROWSER_CARD_WIDTH * CARD_ASPECT_RATIO;
const BROWSER_COLS: u32 = 4;

pub fn graveyard_browser(
    mut commands: Commands,
    mut state: ResMut<GraveyardBrowserState>,
    game: Res<GameResource>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<GraveyardBrowser>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    overlay_interaction: Query<&Interaction, (With<GraveyardBrowser>, With<Button>)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }

    // Close when the dim overlay (outside the panel) is clicked.
    for interaction in &overlay_interaction {
        if *interaction == Interaction::Pressed {
            state.open = false;
        }
    }

    let should_show = state.open;

    if should_show && existing.is_empty() {
        let owner = state.owner;
        let graveyard = &game.state.players[owner].graveyard;
        let owner_label = if owner == PLAYER_0 { "Player 0's" } else { "Player 1's" };
        let count = graveyard.len();

        let panel_width = BROWSER_CARD_WIDTH * BROWSER_COLS as f32 + 80.0;

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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                Button,
                GraveyardBrowser,
            ))
            .id();

        let panel = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(12.0),
                    width: Val::Px(panel_width),
                    max_height: Val::Percent(85.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.97)),
            ))
            .id();

        commands.entity(root).add_child(panel);

        commands.entity(panel)
            .with_children(|panel| {
                // Header row
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("{owner_label} Graveyard ({count} cards)")),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });

                // Card grid
                if graveyard.is_empty() {
                    panel.spawn((
                        Text::new("Empty"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Pickable::IGNORE,
                    ));
                } else {
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(8.0),
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|grid| {
                            for card in graveyard {
                                let path = scryfall::card_asset_path(card.definition.name);
                                let texture: Handle<Image> = asset_server.load(&path);
                                grid.spawn((
                                    ImageNode { image: texture, ..default() },
                                    Node {
                                        width: Val::Px(BROWSER_CARD_WIDTH),
                                        height: Val::Px(BROWSER_CARD_HEIGHT),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                            }
                        });
                }

                // Close hint
                panel.spawn((
                    Text::new("Click outside or press Esc to close"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                    Pickable::IGNORE,
                ));
            });
    } else if !should_show {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

pub fn pile_tooltip(
    mut commands: Commands,
    game: Res<GameResource>,
    deck_hovered: Query<(), (With<DeckCard>, With<CardHovered>)>,
    bot_deck_hovered: Query<(), (With<P1DeckPile>, With<PileHovered>)>,
    gy_hovered: Query<&GraveyardPile, With<PileHovered>>,
    existing: Query<Entity, With<PileTooltip>>,
) {
    let text = if !deck_hovered.is_empty() {
        let count = game.state.players[PLAYER_0].library.len();
        Some(format!("Player 0 library: {count} card{}", if count == 1 { "" } else { "s" }))
    } else if !bot_deck_hovered.is_empty() {
        let count = game.state.players[PLAYER_1].library.len();
        Some(format!("Player 1 library: {count} card{}", if count == 1 { "" } else { "s" }))
    } else if let Some(gy) = gy_hovered.iter().next() {
        let owner_label = if gy.owner == PLAYER_0 { "Player 0's" } else { "Player 1's" };
        let count = game.state.players[gy.owner].graveyard.len();
        Some(format!("{owner_label} graveyard: {count} card{} — click to browse", if count == 1 { "" } else { "s" }))
    } else {
        None
    };

    if let Some(msg) = text {
        if existing.is_empty() {
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(130.0),
                    left: Val::Percent(50.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.82)),
                PileTooltip,
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(msg),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
        } else {
            // Tooltip exists but we can't easily update the child text here,
            // so just leave it; it'll despawn and respawn on next frame if needed.
        }
    } else {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

/// Drives the top-card reveal popup — stays visible until the user clicks anywhere.
pub fn reveal_popup(
    mut commands: Commands,
    mut state: ResMut<RevealPopupState>,
    existing: Query<Entity, With<RevealPopup>>,
    asset_server: Res<AssetServer>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if state.card_path.is_some() {
        // Dismiss on any mouse click.
        if mouse.just_pressed(MouseButton::Left) {
            state.card_path = None;
            for e in &existing { commands.entity(e).despawn(); }
            return;
        }

        if existing.is_empty() {
            let path = state.card_path.clone().unwrap();
            let texture: Handle<Image> = asset_server.load(path);
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(18.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                    Pickable::IGNORE,
                    RevealPopup,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Goblin Guide reveals: (click to dismiss)"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.65, 0.2)),
                        Pickable::IGNORE,
                    ));
                    p.spawn((
                        ImageNode { image: texture, ..default() },
                        Node {
                            width: Val::Px(POPUP_WIDTH * 0.6),
                            height: Val::Px(POPUP_HEIGHT * 0.6),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
    } else {
        for e in &existing { commands.entity(e).despawn(); }
    }
}
