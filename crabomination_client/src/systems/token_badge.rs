//! "×N" count chip on each token pile.
//!
//! `creature_card_transform` piles identical tokens (same name, P/T,
//! tapped state) so wide boards stay legible, fanning only the first few
//! (`layout::TOKEN_PILE_STEPS`); this puts the count on a chip at the pile's
//! top card, cream with dark print like the counter chips, on its upper-right
//! corner as seen and swelling as the count changes. Mechanism mirrors
//! `pt_label`: a screen-space node reprojected from the card's world
//! position every frame, reconciled against the engine view.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, GameCardId};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::UiFonts;

/// Renders below default-z (0) UI so popups / tooltips / modals win —
/// same band as the P/T badge.
const BADGE_Z: i32 = crate::theme::layer::CARD_OVERLAY;
/// The count's ink and the chip's rim: the counter chips' dark print on
/// their cream face (`coin_mesh::INLAY`).
const CHIP_INK: Color = Color::srgb(0.12, 0.10, 0.08);

/// The card's corners, in card-local space: top-right, top-left,
/// bottom-left, bottom-right.
const CORNERS: [Vec3; 4] = [
    Vec3::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, 0.0),
    Vec3::new(-CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, 0.0),
    Vec3::new(-CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0, 0.0),
    Vec3::new(CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0, 0.0),
];

/// How far up and to the right a point on screen is (UI px, y down): the
/// corner that scores highest is the upper-right one as seen.
fn upper_right(at: Vec2) -> f32 {
    at.x - at.y
}

/// The count's size on a card `card_width` UI px across.
fn chip_font_size(card_width: f32) -> f32 {
    (card_width * 0.15).round().clamp(15.0, 30.0)
}

fn chip_label(count: usize) -> String {
    format!("×{count}")
}

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
    cards: Query<(Entity, &GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    cover_cards: crate::card::cover::CoverQuery,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_scale: Res<UiScale>,
    mut badges: Query<(Entity, &TokenPileBadge, &mut Node, &mut Text, &mut TextFont)>,
) {
    let Some(cv) = &view.0 else {
        for (e, ..) in &mut badges {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    // Hidden while another card lies over it (`card::cover`).
    let cover = crate::card::cover::CardCover::new(cam_xform, &cover_cards);

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

    // card_id → where the pile's chip goes on screen and its glyph size: the
    // top card's upper-right corner as seen (an opponent's cards are turned
    // to face them), unless another card lies over it.
    let mut placed: HashMap<CardId, (Vec2, f32)> = HashMap::new();
    for (e, gid, gtf) in &cards {
        if !desired.contains_key(&gid.0) {
            continue;
        }
        let project =
            |local: Vec3| crate::theme::project_to_ui(camera, cam_xform, &ui_scale, gtf.transform_point(local));
        let Some((corner, at)) = CORNERS
            .iter()
            .filter_map(|&c| project(c).map(|at| (c, at)))
            .max_by(|a, b| upper_right(a.1).total_cmp(&upper_right(b.1)))
        else {
            continue;
        };
        if cover.hides_local(e, gtf, corner * 0.85) {
            continue;
        }
        let (Some(left), Some(right)) = (project(CORNERS[3]), project(CORNERS[0])) else { continue };
        placed.insert(gid.0, (at, chip_font_size(left.distance(right))));
    }

    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, badge, mut node, mut text, mut font) in &mut badges {
        let Some(&count) = desired.get(&badge.0) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(badge.0);
        let label = chip_label(count);
        if text.0 != label {
            text.0 = label;
            // The count swells as it changes, as a counter chip's does.
            commands.entity(e).try_insert(crate::theme::OverlayPulse::default());
        }
        match placed.get(&badge.0) {
            Some(&(at, size)) => {
                node.display = Display::Flex;
                node.left = Val::Px(at.x);
                node.top = Val::Px(at.y);
                if font.font_size != FontSize::Px(size) {
                    font.font_size = FontSize::Px(size);
                }
            }
            None => node.display = Display::None,
        }
    }

    for (id, count) in desired {
        if seen.contains(&id) {
            continue;
        }
        let (at, size) = placed.get(&id).copied().unwrap_or((Vec2::splat(-1000.0), 16.0));
        commands.spawn((
            TokenPileBadge(id),
            Text::new(chip_label(count)),
            ui_fonts.tf(size),
            TextColor(CHIP_INK),
            // Faux bold: the UI face is a Light cut.
            TextShadow { offset: Vec2::new(0.7, 0.0), color: CHIP_INK },
            BackgroundColor(crate::systems::coin_mesh::INLAY),
            BorderColor::all(CHIP_INK),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(at.x),
                top: Val::Px(at.y),
                padding: UiRect::axes(Val::Px(7.0), Val::Px(1.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(999.0)),
                display: if placed.contains_key(&id) { Display::Flex } else { Display::None },
                ..default()
            },
            // Centred on the corner: left/top are the corner.
            UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
            Pickable::IGNORE,
            GlobalZIndex(BADGE_Z),
            InGameRoot,
            crate::theme::OverlayPulse::default(),
        ));
    }
}
