//! "×N" count badge floated over each token pile.
//!
//! `creature_card_transform` cascades identical tokens (same name, P/T,
//! tapped state) into one pile so wide boards stay legible; this floats a
//! small count chip at the pile's top card so the player doesn't have to
//! count card edges. Mechanism mirrors `pt_label`: a screen-space node
//! reprojected from the card's world position every frame, reconciled
//! against the engine view.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, GameCardId};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::{self, UiFonts};

/// Renders below default-z (0) UI so popups / tooltips / modals win —
/// same band as the P/T badge.
const BADGE_Z: i32 = -1;
/// Tuck the chip just inside the card's projected top-right corner.
const BADGE_OFFSET_X: f32 = 30.0;
const BADGE_OFFSET_Y: f32 = -4.0;

/// Screen-space pile-count chip anchored to the pile's top card.
#[derive(Component)]
pub struct TokenPileBadge(pub CardId);

/// Reconcile pile-count badges with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_token_pile_badges(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut badges: Query<(Entity, &TokenPileBadge, &mut Node, &mut Text)>,
) {
    let Some(cv) = &view.0 else {
        for (e, _, _, _) in &mut badges {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // Pile membership per owner, keyed the same way the layout groups
    // (`creature_group_info_from_view`): tokens by visual identity. The
    // badge anchors to each pile's LAST member — the cascade's topmost
    // card (largest stagger offset).
    let mut desired: HashMap<CardId, usize> = HashMap::new();
    {
        // (owner, name, power, toughness, tapped) → (last id, count)
        let mut piles: HashMap<(usize, &str, i32, i32, bool), (CardId, usize)> = HashMap::new();
        for c in &cv.battlefield {
            if !c.is_token || c.is_land() {
                continue;
            }
            let entry = piles
                .entry((c.owner, c.name.as_str(), c.power, c.toughness, c.tapped))
                .or_insert((c.id, 0));
            entry.0 = c.id;
            entry.1 += 1;
        }
        for (top_id, count) in piles.into_values() {
            if count >= 2 {
                desired.insert(top_id, count);
            }
        }
    }

    // card_id → world position of the card's top-right corner.
    let top_right_local = Vec3::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, 0.0);
    let mut card_corner: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        if desired.contains_key(&gid.0) {
            card_corner.insert(gid.0, gtf.transform_point(top_right_local));
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
            Some(&count) => {
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
                *text = Text::new(format!("×{count}"));
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    for (id, count) in desired {
        if seen.contains(&id) {
            continue;
        }
        let (left, top) = card_corner
            .get(&id)
            .copied()
            .and_then(|world| anchor(camera, cam_xform, world))
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            TokenPileBadge(id),
            Text::new(format!("×{count}")),
            ui_fonts.tf(16.0),
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
