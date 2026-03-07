use bevy::prelude::*;

use crate::card::{Card, CardBorderHighlight, CardFrontTexture, CardHighlightAssets, CardHovered, CARD_THICKNESS};

#[derive(Component)]
pub struct PeekPopup;

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
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        width: Val::Px(300.0),
                        height: Val::Auto,
                        ..default()
                    },
                    Transform::from_xyz(-150.0, 0.0, 0.0),
                    PeekPopup,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        ImageNode {
                            image: texture,
                            ..default()
                        },
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Auto,
                            ..default()
                        },
                    ));
                });
        }
    } else if !should_show {
        for entity in &existing_popup {
            commands.entity(entity).despawn();
        }
    }
}
