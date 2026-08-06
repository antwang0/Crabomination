//! Ability-lock chip over battlefield permanents.
//!
//! Interdict and Hand to Hand both switch a permanent's activated abilities
//! off without changing anything visible about it — the only tell was a
//! rejected click. Mechanism mirrors `regen_badge`: a screen-space node
//! reprojected from the card's world position, reconciled against the engine
//! view every frame.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, GameCardId};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::{self, UiFonts};

/// Same band as the other permanent chips — popups and modals still win.
const BADGE_Z: i32 = -1;
/// Bottom-right corner, clear of the bottom-left regen chip and the top-right
/// token count.
const BADGE_OFFSET_X: f32 = -6.0;
const BADGE_OFFSET_Y: f32 = 12.0;

/// Screen-space chip marking a permanent whose activated abilities are locked.
#[derive(Component)]
pub struct LockBadge(pub CardId);

/// Reconcile ability-lock chips with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_lock_badges(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut badges: Query<(Entity, &LockBadge, &mut Node)>,
) {
    let Some(cv) = &view.0 else {
        for (e, ..) in &mut badges {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    let desired: HashSet<CardId> =
        cv.battlefield.iter().filter(|c| c.abilities_locked).map(|c| c.id).collect();
    let corner_local = Vec3::new(CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0, 0.0);
    let mut card_corner: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        if desired.contains(&gid.0) {
            card_corner.insert(gid.0, gtf.transform_point(corner_local));
        }
    }

    let anchor = |world: Vec3| -> Option<(f32, f32)> {
        camera
            .world_to_viewport(cam_xform, world)
            .ok()
            .map(|v| (v.x + BADGE_OFFSET_X, v.y - BADGE_OFFSET_Y))
    };

    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, badge, mut node) in &mut badges {
        if !desired.contains(&badge.0) {
            commands.entity(e).despawn();
            continue;
        }
        seen.insert(badge.0);
        match card_corner.get(&badge.0).copied().and_then(anchor) {
            Some((x, y)) => {
                node.display = Display::Flex;
                node.left = Val::Px(x);
                node.top = Val::Px(y);
            }
            None => node.display = Display::None,
        }
    }

    for id in desired.iter().filter(|id| !seen.contains(id)) {
        let (left, top) = card_corner
            .get(id)
            .copied()
            .and_then(anchor)
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            LockBadge(*id),
            Text::new("⊘"),
            ui_fonts.tf(13.0),
            TextColor(theme::TEXT_DANGER),
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
