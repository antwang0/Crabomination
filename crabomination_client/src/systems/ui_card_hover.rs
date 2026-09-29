//! Enlarged card preview for hovered *UI* nodes — stack-panel tiles and
//! game-log lines. The 3-D battlefield/hand counterpart lives in
//! `ui::hover_card_preview`; this reuses its anchor math so the preview
//! behaves identically (sits beside the hovered element on whichever side
//! has more room, clear of it where that side fits).
//!
//! Usage: attach `UiCardHover(asset_path)` plus Bevy's `Button` (for
//! `Interaction` tracking) to any UI node. One preview shows at a time —
//! the first hovered source wins.

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use crate::systems::ui::{
    preview_beside, HOVER_PREVIEW_HEIGHT, HOVER_PREVIEW_MARGIN, HOVER_PREVIEW_WIDTH,
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
    sources: Query<(&Interaction, &UiCardHover, &UiGlobalTransform, &ComputedNode)>,
    asset_server: Res<AssetServer>,
    mut existing: Query<(Entity, &mut Node, &UiCardHoverPreview)>,
    ui_scale: Res<UiScale>,
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
    let hovered = sources.iter().find(|(i, ..)| !matches!(i, Interaction::None));
    let Some((_, source, at, size)) = hovered.filter(|_| window.cursor_position().is_some()) else {
        despawn_all(&mut commands, &existing);
        return;
    };
    let path = source.0.clone();

    // The hovered node's rect and the window, in the UI px the preview is
    // laid out in.
    let scale = size.inverse_scale_factor();
    let target = Rect::from_center_size(at.translation * scale, size.size() * scale);
    let win = Vec2::new(window.width(), window.height()) / ui_scale.0;
    let (x, y) = preview_beside(target, win, HOVER_PREVIEW_WIDTH, HOVER_PREVIEW_HEIGHT, HOVER_PREVIEW_MARGIN);

    if let Ok((entity, mut node, marker)) = existing.single_mut() {
        if node.left != Val::Px(x) || node.top != Val::Px(y) {
            node.left = Val::Px(x);
            node.top = Val::Px(y);
        }
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
        GlobalZIndex(theme::layer::HOVER_PREVIEW),
    ));
}
