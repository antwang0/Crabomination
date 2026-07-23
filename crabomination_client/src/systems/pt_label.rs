//! Floating power/toughness overlay for modified battlefield creatures,
//! a loyalty badge for planeswalkers, and a defense badge for Battles.
//!
//! Whenever a creature's *computed* power/toughness (after counters,
//! auras, and other layer effects) differs from its *printed* base, we
//! float a small `P/T` text badge at the card's bottom-right corner —
//! the same spot the printed P/T box sits — so the player can read the
//! real fighting stats at a glance. The badge is coloured green when the
//! creature is bigger than its printed base and red when it's smaller, so a
//! pump reads apart from a debuff without doing the subtraction. Unmodified
//! creatures show nothing
//! (their printed P/T is already on the card art). Planeswalkers always
//! carry a `◆N` badge in the same corner: their live loyalty total is
//! otherwise only readable by counting 3-D counter coins.
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
    mut labels: Query<(Entity, &PtLabel, &mut Node, &mut Text, &mut TextColor)>,
) {
    // No view (e.g. between matches): clear every badge and bail.
    let Some(cv) = &view.0 else {
        for (e, ..) in &mut labels {
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

    // Badge colours: a buffed creature reads green, a shrunk one red, so the
    // player can tell a pump from a debuff at a glance without doing the
    // subtraction. Net-neutral swaps (e.g. +1/-1) and planeswalker loyalty
    // stay neutral black.
    const BUFF: Color = Color::srgb(0.10, 0.55, 0.12);
    const DEBUFF: Color = Color::srgb(0.72, 0.10, 0.10);
    const NEUTRAL: Color = Color::BLACK;

    // Desired badges: creatures whose computed P/T differs from base
    // (showing "P/T"), plus every planeswalker (showing "◆loyalty"). A
    // creature-planeswalker (Grist) prefers the combat-relevant P/T.
    let mut desired: HashMap<CardId, (String, Color)> = HashMap::new();
    for p in &cv.battlefield {
        if !card_corner.contains_key(&p.id) {
            continue;
        }
        if p.is_creature() {
            if p.power != p.base_power || p.toughness != p.base_toughness {
                let delta = (p.power + p.toughness) - (p.base_power + p.base_toughness);
                let color = match delta.cmp(&0) {
                    std::cmp::Ordering::Greater => BUFF,
                    std::cmp::Ordering::Less => DEBUFF,
                    std::cmp::Ordering::Equal => NEUTRAL,
                };
                desired.insert(p.id, (format!("{}/{}", p.power, p.toughness), color));
            }
            continue;
        }
        if p.card_types.contains(&crabomination::card::CardType::Planeswalker) {
            let loyalty = p
                .counters
                .iter()
                .find(|(k, _)| *k == crabomination::card::CounterType::Loyalty)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            desired.insert(p.id, (format!("\u{25c6}{loyalty}"), NEUTRAL));
            continue;
        }
        // Battle cards (Sieges) carry a defense count that otherwise only
        // reads off the 3-D counter coins — surface it as a `\u{25c7}N` badge
        // in the same corner, the white-diamond sibling of the loyalty badge.
        if p.card_types.contains(&crabomination::card::CardType::Battle) {
            let defense = p
                .counters
                .iter()
                .find(|(k, _)| *k == crabomination::card::CounterType::Defense)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            desired.insert(p.id, (format!("\u{25c7}{defense}"), NEUTRAL));
        }
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
    for (e, label, mut node, mut text, mut color) in &mut labels {
        match desired.get(&label.0) {
            Some((body, badge_color)) => {
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
                *text = Text::new(body.clone());
                color.0 = *badge_color;
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    // Spawn badges for newly-modified creatures / new planeswalkers.
    for (id, (body, badge_color)) in desired {
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
            Text::new(body),
            ui_fonts.tf(18.0),
            // Green when buffed, red when shrunk, black when net-neutral /
            // loyalty — on a white background that mirrors the printed P/T box.
            TextColor(badge_color),
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
