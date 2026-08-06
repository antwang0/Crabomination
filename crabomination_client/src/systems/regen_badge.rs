//! Regeneration-shield chip over battlefield permanents (CR 701.19).
//!
//! A shield changes what "destroy that creature" means for the rest of the
//! turn, but it left no mark on the board — you had to hover the permanent to
//! find it in the counter tooltip. Mechanism mirrors `free_cast_badge`: a
//! screen-space node reprojected from the card's world position, reconciled
//! against the engine view every frame.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, GameCardId};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::{self, UiFonts};

/// Same band as the P/T and token badges — popups and modals still win.
const BADGE_Z: i32 = -1;
/// Tuck the chip just inside the card's projected bottom-left corner, clear
/// of the top-right token count and the top-left free-cast chip.
const BADGE_OFFSET_X: f32 = 6.0;
const BADGE_OFFSET_Y: f32 = 12.0;

/// Screen-space chip marking a permanent's live regeneration shields.
#[derive(Component)]
pub struct RegenBadge(pub CardId);

/// The chip's text: bare shield glyph for the usual one, `⛨×N` past that.
fn chip_text(shields: u32) -> String {
    if shields > 1 { format!("⛨×{shields}") } else { "⛨".into() }
}

/// Reconcile regeneration chips with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_regen_badges(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut badges: Query<(Entity, &RegenBadge, &mut Node, &mut Text)>,
) {
    let Some(cv) = &view.0 else {
        for (e, ..) in &mut badges {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    let desired: HashMap<CardId, u32> = cv
        .battlefield
        .iter()
        .filter(|c| c.regeneration_shields > 0)
        .map(|c| (c.id, c.regeneration_shields))
        .collect();
    let corner_local = Vec3::new(-CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0, 0.0);
    let mut card_corner: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        if desired.contains_key(&gid.0) {
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
    for (e, badge, mut node, mut text) in &mut badges {
        let Some(shields) = desired.get(&badge.0).copied() else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(badge.0);
        let want = chip_text(shields);
        if text.0 != want {
            text.0 = want;
        }
        match card_corner.get(&badge.0).copied().and_then(anchor) {
            Some((x, y)) => {
                node.display = Display::Flex;
                node.left = Val::Px(x);
                node.top = Val::Px(y);
            }
            None => node.display = Display::None,
        }
    }

    for (id, shields) in desired.iter().filter(|(id, _)| !seen.contains(id)) {
        let (left, top) = card_corner
            .get(id)
            .copied()
            .and_then(anchor)
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            RegenBadge(*id),
            Text::new(chip_text(*shields)),
            ui_fonts.tf(13.0),
            TextColor(theme::ACCENT_GREEN),
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

#[cfg(test)]
mod tests {
    use super::chip_text;

    #[test]
    fn the_chip_only_spells_out_a_count_past_one() {
        assert_eq!(chip_text(1), "⛨");
        assert_eq!(chip_text(3), "⛨×3");
    }
}
