//! Floating power/toughness overlay for modified or damaged battlefield
//! creatures, a loyalty badge for planeswalkers, and a defense badge for
//! Battles.
//!
//! Whenever a creature's *computed* power/toughness (after counters,
//! auras, and other layer effects) differs from its *printed* base, or it
//! has damage marked, we float a small `P/T` text badge at the card's
//! bottom-right corner — the same spot the printed P/T box sits — so the
//! player can read the real fighting stats at a glance. Each stat is
//! coloured on its own: green above the printed value, red below, so a pump
//! reads apart from a debuff, and a +2/−1 says which half went which way.
//! Marked damage comes off the toughness shown, in red, as MTG Arena shows
//! it: a damaged 4/4 reads "4/2", the damage it can still take. Unmodified,
//! undamaged creatures show nothing (their printed P/T is already on the
//! card art). Planeswalkers always carry a `◆N` badge in the same corner:
//! their loyalty has no other on-card readout.
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
const PT_Z: i32 = crate::theme::layer::CARD_OVERLAY;
/// Approximate badge footprint, used to tuck it just inside the card's
/// projected bottom-right corner rather than spilling off the edge.
const PT_OFFSET_X: f32 = 38.0;
const PT_OFFSET_Y: f32 = 22.0;

/// Screen-space P/T badge tied to a battlefield card's `CardId`.
#[derive(Component)]
pub struct PtLabel(pub CardId);

const BUFF: Color = Color::srgb(0.10, 0.55, 0.12);
const DEBUFF: Color = Color::srgb(0.72, 0.10, 0.10);
const NEUTRAL: Color = Color::BLACK;

/// A stat's colour against its printed value: green above, red below, black
/// as printed.
fn stat_tone(base: i32, now: i32) -> Color {
    match now.cmp(&base) {
        std::cmp::Ordering::Greater => BUFF,
        std::cmp::Ordering::Less => DEBUFF,
        std::cmp::Ordering::Equal => NEUTRAL,
    }
}

/// A badge's text in three runs, each with its colour: power, the slash,
/// toughness. A loyalty or defense badge is all lead.
type BadgeRuns = [(String, Color); 3];

/// The P/T badge for a creature, or `None` when its printed box already
/// says it all. Damage comes off the toughness shown, in red.
fn pt_runs(base_power: i32, base_toughness: i32, power: i32, toughness: i32, damage: u32) -> Option<BadgeRuns> {
    if power == base_power && toughness == base_toughness && damage == 0 {
        return None;
    }
    let left = toughness - damage as i32;
    let toughness_tone = if damage > 0 { DEBUFF } else { stat_tone(base_toughness, toughness) };
    Some([
        (power.to_string(), stat_tone(base_power, power)),
        ("/".to_string(), NEUTRAL),
        (left.to_string(), toughness_tone),
    ])
}

fn lead_only(text: String) -> BadgeRuns {
    [(text, NEUTRAL), (String::new(), NEUTRAL), (String::new(), NEUTRAL)]
}

/// Reconcile P/T badges with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_pt_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(Entity, &GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    cover_cards: crate::card::cover::CoverQuery,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_scale: Res<UiScale>,
    mut labels: Query<(Entity, &PtLabel, &mut Node, &mut Text, &mut TextColor, &Children)>,
    mut spans: Query<(&mut TextSpan, &mut TextColor), Without<PtLabel>>,
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
    // A badge hides while another card lies over its corner (`card::cover`).
    let cover = crate::card::cover::CardCover::new(cam_xform, &cover_cards);
    let mut card_corner: HashMap<CardId, Vec3> = HashMap::new();
    let mut covered: HashSet<CardId> = HashSet::new();
    for (e, gid, gtf) in &cards {
        card_corner.insert(gid.0, gtf.transform_point(bottom_right_local));
        if cover.hides_local(e, gtf, bottom_right_local * 0.85) {
            covered.insert(gid.0);
        }
    }

    // Desired badges: creatures whose computed P/T differs from base or
    // that have damage marked (showing "P/T"), plus every planeswalker
    // (showing "◆loyalty"). A creature-planeswalker (Grist) prefers the
    // combat-relevant P/T.
    let mut desired: HashMap<CardId, BadgeRuns> = HashMap::new();
    for p in &cv.battlefield {
        if !card_corner.contains_key(&p.id) {
            continue;
        }
        if p.is_creature() {
            if let Some(runs) = pt_runs(p.base_power, p.base_toughness, p.power, p.toughness, p.damage) {
                desired.insert(p.id, runs);
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
            desired.insert(p.id, lead_only(format!("\u{25c6}{loyalty}")));
            continue;
        }
        // Battle cards (Sieges) carry a defense count with no other on-card
        // readout — surface it as a `\u{25c7}N` badge in the same corner,
        // the white-diamond sibling of the loyalty badge.
        if p.card_types.contains(&crabomination::card::CardType::Battle) {
            let defense = p
                .counters
                .iter()
                .find(|(k, _)| *k == crabomination::card::CounterType::Defense)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            desired.insert(p.id, lead_only(format!("\u{25c7}{defense}")));
        }
    }

    /// Project a card-corner world point to a viewport pixel anchor,
    /// tucking the badge just inside the corner so it overlaps the
    /// card's bottom-right rather than floating off it.
    fn anchor(
        camera: &Camera,
        cam_xform: &GlobalTransform,
        ui_scale: &UiScale,
        world: Vec3,
    ) -> Option<(f32, f32)> {
        crate::theme::project_to_ui(camera, cam_xform, ui_scale, world)
            .map(|v| (v.x - PT_OFFSET_X, v.y - PT_OFFSET_Y))
    }

    // Update existing badges; despawn any whose creature is no longer
    // modified (or has left the battlefield). Text is only written when it
    // changed — a write re-shapes it.
    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, label, mut node, mut text, mut color, children) in &mut labels {
        let Some(runs) = desired.get(&label.0) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(label.0);
        if !covered.contains(&label.0)
            && let Some(world) = card_corner.get(&label.0).copied()
            && let Some((x, y)) = anchor(camera, cam_xform, &ui_scale, world)
        {
            node.display = Display::Flex;
            node.left = Val::Px(x);
            node.top = Val::Px(y);
        } else {
            node.display = Display::None;
        }
        // A badge swells when its numbers change (`theme::OverlayPulse`).
        let mut changed = false;
        if text.0 != runs[0].0 {
            text.0 = runs[0].0.clone();
            changed = true;
        }
        if color.0 != runs[0].1 {
            color.0 = runs[0].1;
        }
        for (child, (body, tone)) in children.iter().zip(&runs[1..]) {
            if let Ok((mut span, mut color)) = spans.get_mut(child) {
                if span.0 != *body {
                    span.0 = body.clone();
                    changed = true;
                }
                if color.0 != *tone {
                    color.0 = *tone;
                }
            }
        }
        if changed {
            commands.entity(e).insert(crate::theme::OverlayPulse::default());
        }
    }

    // Spawn badges for newly-modified creatures / new planeswalkers.
    for (id, [lead, slash, tail]) in desired {
        if seen.contains(&id) {
            continue;
        }
        let (left, top) = card_corner
            .get(&id)
            .copied()
            .and_then(|world| anchor(camera, cam_xform, &ui_scale, world))
            .unwrap_or((-1000.0, -1000.0));
        commands
            .spawn((
                PtLabel(id),
                Text::new(lead.0),
                ui_fonts.tf(18.0),
                // Each run in its stat's tone, on a white background that
                // mirrors the printed P/T box.
                TextColor(lead.1),
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
                crate::theme::OverlayPulse::default(),
            ))
            .with_children(|badge| {
                for (body, tone) in [slash, tail] {
                    badge.spawn((TextSpan::new(body), ui_fonts.tf(18.0), TextColor(tone)));
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tones(runs: &BadgeRuns) -> (Color, Color) {
        (runs[0].1, runs[2].1)
    }

    #[test]
    fn each_stat_is_toned_against_its_printed_value() {
        let text = |r: &BadgeRuns| r.iter().map(|(s, _)| s.as_str()).collect::<String>();
        let pumped = pt_runs(2, 2, 3, 3, 0).unwrap();
        assert_eq!((text(&pumped).as_str(), tones(&pumped)), ("3/3", (BUFF, BUFF)));
        let shrunk = pt_runs(2, 2, 1, 1, 0).unwrap();
        assert_eq!(tones(&shrunk), (DEBUFF, DEBUFF));
        // A swap says which half went which way.
        assert_eq!(tones(&pt_runs(2, 2, 3, 1, 0).unwrap()), (BUFF, DEBUFF));
        assert_eq!(tones(&pt_runs(2, 2, 4, 2, 0).unwrap()), (BUFF, NEUTRAL));
        // As printed and unhurt: the card's own box says it.
        assert!(pt_runs(2, 2, 2, 2, 0).is_none());
    }

    #[test]
    fn damage_comes_off_the_toughness_shown_in_red() {
        let hurt = pt_runs(4, 4, 4, 4, 2).unwrap();
        assert_eq!(format!("{}{}{}", hurt[0].0, hurt[1].0, hurt[2].0), "4/2");
        assert_eq!(tones(&hurt), (NEUTRAL, DEBUFF));
        // Pumped and hurt: the power still reads the pump.
        let both = pt_runs(3, 3, 5, 5, 1).unwrap();
        assert_eq!(both[2].0, "4");
        assert_eq!(tones(&both), (BUFF, DEBUFF));
    }
}
