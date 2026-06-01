use bevy::prelude::*;

use crate::card::{
    Card, CardBorderHighlight, CardFrontTexture, CardHighlightAssets, CardHovered,
    CastableHighlight, DeckCard, DeckPile, GameCardId, GraveyardPile, HandCard, PileHovered,
    CARD_THICKNESS,
};
use crate::game::GraveyardBrowserState;
use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};

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

/// One card tile inside the graveyard browser. Stores the card's
/// display name so the hover-name-tooltip system can render it.
#[derive(Component)]
pub struct GraveyardCardItem {
    pub name: String,
}

/// Marker for the live name tooltip rendered above the graveyard
/// browser whenever the user hovers a card tile.
#[derive(Component)]
pub struct GraveyardCardNameTooltip;

#[derive(Component)]
pub struct RevealPopup;

#[derive(Component)]
pub struct PileTooltip {
    /// The message currently rendered. Tracked so `pile_tooltip` can
    /// detect when the hovered pile changes and refresh the text instead
    /// of leaving a stale tooltip pinned (e.g. dragging the cursor from
    /// one player's deck to the other's graveyard).
    pub msg: String,
}

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

/// Spawn / despawn a green "castable now" border around each viewer hand
/// card listed in the view's `castable_hand` set (computed server-side via
/// the engine's `would_accept` dry-run, so it already reflects timing,
/// mana, taxes, and target availability). Mirrors the hover-border /
/// put-on-library highlight pattern.
///
/// Suppressed while the targeting cursor is active — there the gold
/// valid-target borders own the hand visuals and a second border set would
/// only stack and z-fight. The set is also naturally empty when the viewer
/// lacks priority (you can't cast then anyway).
#[allow(clippy::type_complexity)]
pub fn update_castable_highlights(
    mut commands: Commands,
    view: Res<CurrentView>,
    targeting: Res<crate::game::TargetingState>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    hand_cards: Query<(Entity, &GameCardId, Option<&CastableHighlight>), With<HandCard>>,
    // A card played/discarded mid-highlight keeps its entity (HandCard is
    // *removed*, not despawned — see `sync_game_visuals`), so its green
    // border would otherwise linger on the battlefield/graveyard. Strip the
    // highlight from anything that left the hand still wearing it.
    left_hand: Query<(Entity, &CastableHighlight), Without<HandCard>>,
) {
    let Some(assets) = highlight_assets else { return };

    for (entity, h) in &left_hand {
        commands.entity(h.back).despawn();
        commands.entity(h.front).despawn();
        commands.entity(entity).remove::<CastableHighlight>();
    }

    let castable: std::collections::HashSet<crabomination::card::CardId> = if targeting.active {
        std::collections::HashSet::new()
    } else {
        view.0
            .as_ref()
            .map(|cv| cv.castable_hand.iter().copied().collect())
            .unwrap_or_default()
    };

    for (entity, gid, marker) in &hand_cards {
        let should = castable.contains(&gid.0);
        match (should, marker) {
            (true, None) => {
                // Sits slightly proud of the hover border (0.001) so a card
                // that is both castable and hovered shows both rings without
                // z-fighting.
                let offset = CARD_THICKNESS / 2.0 + 0.0018;
                let back = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.castable_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                let front = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.castable_material.clone()),
                        Transform::from_xyz(0.0, 0.0, offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert(CastableHighlight { back, front })
                    .add_children(&[back, front]);
            }
            (false, Some(h)) => {
                commands.entity(h.back).despawn();
                commands.entity(h.front).despawn();
                commands.entity(entity).remove::<CastableHighlight>();
            }
            _ => {}
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
                    BackgroundColor(theme::OVERLAY_BG_LIGHT),
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
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
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
        let owner_name: String = view
            .0
            .as_ref()
            .and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {owner}"));
        let owner_label = format!("{owner_name}'s");

        // (name, recast-badge): badge is "Flashback {N}" / "Retrace" for
        // cards castable from the graveyard, surfaced from the view's new
        // recast flags so players can see their options at a glance.
        let card_names: Vec<(String, Option<String>)> = view.0.as_ref()
            .map(|cv| cv.players[owner].graveyard.iter().map(|c| {
                let badge = if let Some(fb) = &c.flashback_cost {
                    Some(format!("Flashback {{{}}}", fb.cmc()))
                } else if c.retrace {
                    Some("Retrace".to_string())
                } else {
                    None
                };
                (c.name.clone(), badge)
            }).collect())
            .unwrap_or_default();
        let count = card_names.len();

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
                BackgroundColor(theme::OVERLAY_BG),
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
                BackgroundColor(theme::PANEL_BG),
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
                            ui_fonts.tf(18.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });

                // Card grid
                if card_names.is_empty() {
                    panel.spawn((
                        Text::new("Empty"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_SECONDARY),
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
                            for (name, badge) in &card_names {
                                let path = scryfall::card_asset_path(name);
                                let texture: Handle<Image> = asset_server.load(&path);
                                // Each tile is a Button so Bevy's
                                // `Interaction` component reports
                                // hover/press. The ImageNode inside
                                // is `Pickable::IGNORE` so events
                                // bubble up to the button rather than
                                // landing on the image directly. The
                                // outer button captures clicks too,
                                // which prevents accidental close-on-
                                // click since the dim overlay used to
                                // swallow clicks that passed through
                                // the cards.
                                grid.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(BROWSER_CARD_WIDTH),
                                        height: Val::Px(BROWSER_CARD_HEIGHT),
                                        ..default()
                                    },
                                    GraveyardCardItem { name: name.clone() },
                                ))
                                .with_children(|tile| {
                                    tile.spawn((
                                        ImageNode { image: texture, ..default() },
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ));
                                    // Recast badge pinned to the bottom of
                                    // the tile (Flashback / Retrace).
                                    if let Some(label) = badge {
                                        tile.spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                bottom: Val::Px(2.0),
                                                left: Val::Px(2.0),
                                                right: Val::Px(2.0),
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                            BackgroundColor(theme::OVERLAY_BG),
                                            Pickable::IGNORE,
                                        ))
                                        .with_children(|b| {
                                            b.spawn((
                                                Text::new(label.clone()),
                                                ui_fonts.tf(10.0),
                                                TextColor(theme::TEXT_PRIMARY),
                                                Pickable::IGNORE,
                                            ));
                                        });
                                    }
                                });
                            }
                        });
                }

                // Close hint
                panel.spawn((
                    Text::new("Click outside or press Esc to close"),
                    ui_fonts.tf(11.0),
                    TextColor(theme::TEXT_MUTED),
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
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    deck_hovered: Query<(), (With<DeckCard>, With<CardHovered>)>,
    pile_hovered: Query<&DeckPile, With<PileHovered>>,
    gy_hovered: Query<&GraveyardPile, With<PileHovered>>,
    existing: Query<(Entity, &PileTooltip)>,
) {
    let cv = view.0.as_ref();
    let lib_size = |owner: usize| -> usize {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.library.size)
            .unwrap_or(0)
    };
    let gy_count = |owner: usize| -> usize {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.graveyard.len())
            .unwrap_or(0)
    };
    let player_name = |owner: usize| -> String {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {owner}"))
    };
    let viewer = cv.map(|cv| cv.your_seat).unwrap_or(0);

    let text = if !deck_hovered.is_empty() {
        // Hovering a face-up viewer-deck card visual.
        let count = lib_size(viewer);
        Some(format!("{} library: {count} card{}", player_name(viewer), if count == 1 { "" } else { "s" }))
    } else if let Some(pile) = pile_hovered.iter().next() {
        let count = lib_size(pile.owner);
        Some(format!("{} library: {count} card{}", player_name(pile.owner), if count == 1 { "" } else { "s" }))
    } else if let Some(gy) = gy_hovered.iter().next() {
        let count = gy_count(gy.owner);
        Some(format!("{}'s graveyard: {count} card{} — click to browse", player_name(gy.owner), if count == 1 { "" } else { "s" }))
    } else {
        None
    };

    if let Some(msg) = text {
        // If a tooltip is already pinned with the *same* text, leave it.
        // Otherwise despawn the stale one so we can respawn with the
        // current message (fixes the cursor-moved-between-piles staleness).
        let up_to_date = existing.iter().any(|(_, t)| t.msg == msg);
        if !up_to_date {
            for (e, _) in existing.iter() {
                commands.entity(e).despawn();
            }
            // Pin the tooltip's *center* to the screen midline. Without
            // the negative margin Bevy positions the node's left edge at
            // 50% and the tooltip drifts right of center (worse the
            // longer the string). 140px is half the widest tooltip text
            // ("Player N's graveyard: NN cards — click to browse") at
            // the 14px font, which is close enough for visual centering
            // without measuring text width per frame.
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(130.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-140.0)),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::OVERLAY_BG_HEAVY),
                PileTooltip { msg: msg.clone() },
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(msg),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        }
    } else {
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
    }
}

/// Show the name of the graveyard card currently being hovered as
/// a tooltip pinned to the top of the browser panel. Despawns when
/// the cursor leaves all card tiles, or when the browser closes.
pub fn graveyard_card_hover_name(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    items: Query<(&Interaction, &GraveyardCardItem)>,
    existing: Query<Entity, With<GraveyardCardNameTooltip>>,
    browser: Query<(), With<GraveyardBrowser>>,
    mut current_text: Query<&mut Text, With<GraveyardCardNameTooltip>>,
) {
    // No browser open → drop the tooltip.
    if browser.is_empty() {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }

    let hovered_name = items
        .iter()
        .find(|(i, _)| matches!(**i, Interaction::Hovered | Interaction::Pressed))
        .map(|(_, item)| item.name.clone());

    let Some(name) = hovered_name else {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    };

    // Tooltip already exists — just refresh its text.
    if let Ok(mut t) = current_text.single_mut() {
        if t.0 != name {
            t.0 = name;
        }
        return;
    }

    // Spawn the tooltip pinned near the top of the screen so it's
    // always visible even when scrolling deep into a long graveyard.
    // Marker lives on the Text entity (parent + child) — keeping it
    // single-entity makes both the despawn and the text-refresh
    // queries straightforward.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-110.0)),
                width: Val::Px(220.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            Pickable::IGNORE,
            Text::new(name),
            ui_fonts.tf(16.0),
            TextColor(theme::ACCENT_GOLD),
            GraveyardCardNameTooltip,
        ));
}

/// Drives the top-card reveal popup — stays visible until the user clicks anywhere.
pub fn reveal_popup(
    mut commands: Commands,
    mut state: ResMut<RevealPopupState>,
    existing: Query<Entity, With<RevealPopup>>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
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
                    BackgroundColor(theme::OVERLAY_BG_HEAVY),
                    Pickable::IGNORE,
                    RevealPopup,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Goblin Guide reveals: (click to dismiss)"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::ACCENT_ORANGE),
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
