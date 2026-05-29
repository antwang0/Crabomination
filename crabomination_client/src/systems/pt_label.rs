//! Floating power/toughness overlay for modified battlefield creatures.
//!
//! Whenever a creature's *computed* power/toughness (after counters,
//! auras, and other layer effects) differs from its *printed* base, we
//! float a small `P/T` text badge at the card's bottom-right corner —
//! the same spot the printed P/T box sits — so the player can read the
//! real fighting stats at a glance. Unmodified creatures show nothing
//! (their printed P/T is already on the card art).
//!
//! Mechanism mirrors `game_ui::crest`'s floating life numeral: a
//! screen-space UI text node is reprojected from the card's world
//! position every frame. The badge is rendered *beneath* other UI (a
//! negative `GlobalZIndex`) so peek popups, tooltips, and modals always
//! draw on top of it. Labels are reconciled against the engine view —
//! spawned for newly-modified creatures, despawned when a creature
//! returns to base stats or leaves the battlefield.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, GameCardId};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::UiFonts;

/// Renders below default-z (0) UI so popups / tooltips / modals win.
const PT_Z: i32 = -1;
/// Approximate badge footprint, used to tuck it just inside the card's
/// projected bottom-right corner rather than spilling off the edge.
const PT_OFFSET_X: f32 = 38.0;
const PT_OFFSET_Y: f32 = 22.0;

/// Screen-space P/T badge tied to a battlefield card's `CardId`.
#[derive(Component)]
pub struct PtLabel(pub CardId);

/// Reconcile P/T badges with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_pt_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut labels: Query<(Entity, &PtLabel, &mut Node, &mut Text)>,
) {
    // No view (e.g. between matches): clear every badge and bail.
    let Some(cv) = &view.0 else {
        for (e, _, _, _) in &mut labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // card_id → world position of the card's bottom-right corner (the
    // printed P/T box). Transforming a card-local corner through the
    // card's `GlobalTransform` keeps the anchor correct under the flat
    // battlefield rotation and any perspective.
    let bottom_right_local = Vec3::new(CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0, 0.0);
    let mut card_corner: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        card_corner.insert(gid.0, gtf.transform_point(bottom_right_local));
    }

    // Desired badges: creatures whose computed P/T differs from base and
    // whose card has a live battlefield entity to anchor against.
    let mut desired: HashMap<CardId, (i32, i32)> = HashMap::new();
    for p in &cv.battlefield {
        if !p.is_creature() {
            continue;
        }
        if p.power == p.base_power && p.toughness == p.base_toughness {
            continue;
        }
        if !card_corner.contains_key(&p.id) {
            continue;
        }
        desired.insert(p.id, (p.power, p.toughness));
    }

    /// Project a card-corner world point to a viewport pixel anchor,
    /// tucking the badge just inside the corner so it overlaps the
    /// card's bottom-right rather than floating off it.
    fn anchor(camera: &Camera, cam_xform: &GlobalTransform, world: Vec3) -> Option<(f32, f32)> {
        camera
            .world_to_viewport(cam_xform, world)
            .ok()
            .map(|v| (v.x - PT_OFFSET_X, v.y - PT_OFFSET_Y))
    }

    // Update existing badges; despawn any whose creature is no longer
    // modified (or has left the battlefield).
    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, label, mut node, mut text) in &mut labels {
        match desired.get(&label.0) {
            Some(&(pw, tf)) => {
                seen.insert(label.0);
                if let Some(world) = card_corner.get(&label.0).copied()
                    && let Some((x, y)) = anchor(camera, cam_xform, world)
                {
                    node.display = Display::Flex;
                    node.left = Val::Px(x);
                    node.top = Val::Px(y);
                } else {
                    node.display = Display::None;
                }
                *text = Text::new(format!("{pw}/{tf}"));
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    // Spawn badges for newly-modified creatures.
    for (id, (pw, tf)) in desired {
        if seen.contains(&id) {
            continue;
        }
        let (left, top) = card_corner
            .get(&id)
            .copied()
            .and_then(|world| anchor(camera, cam_xform, world))
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            PtLabel(id),
            Text::new(format!("{pw}/{tf}")),
            ui_fonts.tf(18.0),
            // Black text on a white background — mirrors the printed
            // P/T box.
            TextColor(Color::BLACK),
            BackgroundColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                // Tight, symmetric padding so the white box hugs the
                // glyphs; centre the text within the box.
                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(PT_Z),
            InGameRoot,
        ));
    }
}
