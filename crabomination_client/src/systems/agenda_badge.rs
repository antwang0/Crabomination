//! Chosen-name chip floated over a hidden-agenda conspiracy (CR 702.106).
//!
//! The server ships `KnownCard.agenda_names` to the conspiracy's controller
//! while it is face down and to everyone once it is face up; without a chip
//! the command-zone card shows only its own name, so the player has no way to
//! recall what they named. Mechanism mirrors `token_badge`: a screen-space
//! node reprojected from the card's world position, reconciled every frame.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::MainCamera;
use crate::card::{CARD_HEIGHT, CARD_WIDTH, CommandZoneCard, GameCardId};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::{self, UiFonts};

/// Same band as the P/T and token badges — popups and modals still win.
const BADGE_Z: i32 = -1;
/// Tuck the chip just below the card's projected bottom-left corner.
const BADGE_OFFSET_X: f32 = 8.0;
const BADGE_OFFSET_Y: f32 = -6.0;

/// Screen-space chip naming what a command-zone conspiracy chose.
#[derive(Component)]
pub struct AgendaBadge(pub CardId);

/// Reconcile agenda chips with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_agenda_badges(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<CommandZoneCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut badges: Query<(Entity, &AgendaBadge, &mut Node, &mut Text)>,
) {
    let Some(cv) = &view.0 else {
        for (e, _, _, _) in &mut badges {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    let mut desired: HashMap<CardId, String> = HashMap::new();
    for player in &cv.players {
        for entry in &player.command {
            if let crabomination::net::HandCardView::Known(k) = entry
                && !k.agenda_names.is_empty()
            {
                desired.insert(k.id, k.agenda_names.join(" / "));
            }
        }
    }

    let corner_local = Vec3::new(-CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0, 0.0);
    let mut card_corner: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        if desired.contains_key(&gid.0) {
            card_corner.insert(gid.0, gtf.transform_point(corner_local));
        }
    }

    fn anchor(camera: &Camera, cam_xform: &GlobalTransform, world: Vec3) -> Option<(f32, f32)> {
        camera
            .world_to_viewport(cam_xform, world)
            .ok()
            .map(|v| (v.x - BADGE_OFFSET_X, v.y - BADGE_OFFSET_Y))
    }

    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, badge, mut node, mut text) in &mut badges {
        match desired.get(&badge.0) {
            Some(names) => {
                seen.insert(badge.0);
                if let Some(world) = card_corner.get(&badge.0).copied()
                    && let Some((x, y)) = anchor(camera, cam_xform, world)
                {
                    node.display = Display::Flex;
                    node.left = Val::Px(x);
                    node.top = Val::Px(y);
                } else {
                    node.display = Display::None;
                }
                *text = Text::new(format!("named {names}"));
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    for (id, names) in desired {
        if seen.contains(&id) {
            continue;
        }
        let (left, top) = card_corner
            .get(&id)
            .copied()
            .and_then(|world| anchor(camera, cam_xform, world))
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            AgendaBadge(id),
            Text::new(format!("named {names}")),
            ui_fonts.tf(14.0),
            TextColor(theme::ACCENT_GOLD),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.10, 0.92)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(BADGE_Z),
            InGameRoot,
        ));
    }
}
