//! Enlarged card preview for hovered *UI* nodes — stack-panel tiles and
//! game-log lines. The 3-D battlefield/hand counterpart lives in
//! `ui::hover_card_preview`; this reuses its anchor math so the preview
//! behaves identically (sits beside the cursor on whichever side has
//! room, never covers the hovered element).
//!
//! Usage: attach `UiCardHover(asset_path)` plus Bevy's `Button` (for
//! `Interaction` tracking) to any UI node. One preview shows at a time —
//! the first hovered source wins.

use bevy::prelude::*;

use crate::systems::ui::{
    preview_anchor, HOVER_PREVIEW_HEIGHT, HOVER_PREVIEW_MARGIN, HOVER_PREVIEW_WIDTH,
};
use crate::theme;

/// Hovering this UI node previews the card image at `.0` (an asset path
/// from `scryfall::card_asset_path`).
#[derive(Component)]
pub struct UiCardHover(pub String);

/// The floating preview spawned while a `UiCardHover` node is hovered.
/// Stores the shown path so a hover moving between sources rebuilds.
#[derive(Component)]
pub struct UiCardHoverPreview {
    path: String,
}

#[allow(clippy::type_complexity)]
pub fn ui_card_hover_preview(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sources: Query<(&Interaction, &UiCardHover)>,
    asset_server: Res<AssetServer>,
    mut existing: Query<(Entity, &mut Node, &UiCardHoverPreview)>,
) {
    let despawn_all =
        |commands: &mut Commands, existing: &Query<(Entity, &mut Node, &UiCardHoverPreview)>| {
            for (e, _, _) in existing.iter() {
                commands.entity(e).despawn();
            }
        };

    let Ok(window) = windows.single() else {
        despawn_all(&mut commands, &existing);
        return;
    };
    let desired: Option<(String, Vec2)> = match window.cursor_position() {
        Some(cursor) => sources
            .iter()
            .find(|(i, _)| !matches!(i, Interaction::None))
            .map(|(_, h)| (h.0.clone(), cursor)),
        None => None,
    };
    let Some((path, cursor)) = desired else {
        despawn_all(&mut commands, &existing);
        return;
    };

    let win = Vec2::new(window.width(), window.height());
    let (x, y) = preview_anchor(
        cursor,
        win,
        HOVER_PREVIEW_WIDTH,
        HOVER_PREVIEW_HEIGHT,
        HOVER_PREVIEW_MARGIN,
    );

    if let Ok((entity, mut node, marker)) = existing.single_mut() {
        node.left = Val::Px(x);
        node.top = Val::Px(y);
        if marker.path == path {
            return;
        }
        commands.entity(entity).despawn();
    }

    let texture: Handle<Image> = asset_server.load(&path);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            width: Val::Px(HOVER_PREVIEW_WIDTH),
            height: Val::Px(HOVER_PREVIEW_HEIGHT),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
            ..default()
        },
        BorderColor::all(theme::ACCENT_GOLD),
        ImageNode { image: texture, ..default() },
        Pickable::IGNORE,
        UiCardHoverPreview { path },
        crate::systems::game_ui::InGameRoot,
        GlobalZIndex(30),
    ));
}
