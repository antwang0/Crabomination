//! Quality preset selector panel (bottom-right HUD overlay).

use bevy::prelude::*;

use crate::render_quality::{ChangeQuality, RenderQuality};

const QUALITY_BTN_ACTIVE: Color = Color::srgb(0.15, 0.45, 0.15);
const QUALITY_BTN_INACTIVE: Color = Color::srgb(0.12, 0.12, 0.18);

#[derive(Component)]
pub struct QualityButton(pub RenderQuality);

pub fn setup_quality_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    quality: Res<RenderQuality>,
) {
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let tf = |size: f32| TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(6.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Quality"),
                tf(11.0),
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|p| {
                for q in RenderQuality::ALL {
                    let bg = if q == *quality { QUALITY_BTN_ACTIVE } else { QUALITY_BTN_INACTIVE };
                    p.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        Button,
                        QualityButton(q),
                    ))
                    .with_children(|p| {
                        p.spawn((Text::new(q.label()), tf(12.0), TextColor(Color::WHITE)));
                    });
                }
            });
        });
}

pub fn handle_quality_buttons(
    mut buttons: Query<(&Interaction, &QualityButton, &mut BackgroundColor)>,
    quality: Res<RenderQuality>,
    mut messages: MessageWriter<ChangeQuality>,
) {
    let pressed = buttons
        .iter()
        .find(|(i, _, _)| **i == Interaction::Pressed)
        .map(|(_, btn, _)| btn.0);

    if let Some(q) = pressed {
        messages.write(ChangeQuality(q));
    }

    if pressed.is_some() || quality.is_changed() {
        let active = pressed.unwrap_or(*quality);
        for (_, btn, mut bg) in buttons.iter_mut() {
            *bg = if btn.0 == active {
                BackgroundColor(QUALITY_BTN_ACTIVE)
            } else {
                BackgroundColor(QUALITY_BTN_INACTIVE)
            };
        }
    }
}
