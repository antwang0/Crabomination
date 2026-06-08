//! At-a-glance keyword flags floated over battlefield creatures.
//!
//! A creature's evergreen combat keywords (flying, deathtouch, lifelink, …)
//! only live in the card's text box, which is illegible once the card is
//! minified at the table's oblique angle — especially across the table on an
//! opponent's board. This floats a small abbreviated strip ("Fly DT LL") over
//! the top of each creature so the board reads at a glance.
//!
//! Mechanism mirrors `pt_label`: a screen-space UI strip reprojected from the
//! card's world position every frame, reconciled against the engine view
//! (spawned for newly-keyworded creatures, despawned when a creature loses all
//! displayable keywords or leaves the battlefield). It sits at the card's top
//! edge so it never collides with the bottom-right P/T badge, and renders
//! below default-z UI so popups / tooltips win.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::{CardId, Keyword};

use crate::card::{BattlefieldCard, GameCardId, CARD_HEIGHT};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::UiFonts;
use crate::MainCamera;

/// Renders below default-z (0) UI so popups / tooltips / modals win — same
/// band as the P/T badge.
const KW_Z: i32 = -1;
/// Lift the strip a few px above the card's projected top edge so it reads as
/// a banner sitting on the card rather than overlapping the title.
const KW_LIFT: f32 = 14.0;
/// Rough px width per character at the strip's font size, used only to centre
/// the strip horizontally over the card (the node itself auto-sizes).
const KW_CHAR_PX: f32 = 6.5;

/// Screen-space keyword strip tied to a battlefield card's `CardId`.
#[derive(Component)]
pub struct KeywordLabel(pub CardId);

/// Short tag for the combat/board-relevant keywords worth surfacing on a
/// permanent. Casting-only keywords (Flashback, Kicker, Buyback, …) return
/// `None` — they never matter for a creature already on the battlefield.
fn keyword_tag(kw: &Keyword) -> Option<&'static str> {
    use Keyword::*;
    Some(match kw {
        Flying => "Fly",
        Reach => "Rch",
        Menace => "Men",
        Trample => "Tmp",
        Vigilance => "Vig",
        FirstStrike => "FS",
        DoubleStrike => "DS",
        Deathtouch => "DT",
        Lifelink => "LL",
        Haste => "Hst",
        Defender => "Def",
        Indestructible => "Ind",
        Hexproof => "Hex",
        Shroud => "Shr",
        Unblockable => "Unb",
        Intimidate => "Int",
        Fear => "Fear",
        Infect => "Inf",
        Wither => "Wth",
        Skulk => "Skk",
        Shadow => "Shd",
        Horsemanship => "Hrs",
        Landwalk(_) => "Wlk",
        Protection(_) => "Pro",
        Ward(_) => "Ward",
        Toxic(_) => "Tox",
        _ => return None,
    })
}

/// Build the displayed strip for a permanent's keyword list: each displayable
/// keyword's tag, first-occurrence order, de-duplicated, space-joined. Empty
/// string when nothing is worth showing.
fn keyword_strip(keywords: &[Keyword]) -> String {
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut tags: Vec<&'static str> = Vec::new();
    for kw in keywords {
        if let Some(tag) = keyword_tag(kw)
            && seen.insert(tag)
        {
            tags.push(tag);
        }
    }
    tags.join(" ")
}

/// Reconcile keyword strips with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_keyword_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut labels: Query<(Entity, &KeywordLabel, &mut Node, &mut Text)>,
) {
    // No view (between matches): clear every strip and bail.
    let Some(cv) = &view.0 else {
        for (e, _, _, _) in &mut labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // card_id → world position of the card's top-centre (the title edge),
    // transformed through the flat battlefield rotation.
    let top_center_local = Vec3::new(0.0, CARD_HEIGHT / 2.0, 0.0);
    let mut card_top: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        card_top.insert(gid.0, gtf.transform_point(top_center_local));
    }

    // Desired strips: creatures with at least one displayable keyword and a
    // live battlefield entity to anchor against.
    let mut desired: HashMap<CardId, String> = HashMap::new();
    for p in &cv.battlefield {
        if !p.is_creature() || !card_top.contains_key(&p.id) {
            continue;
        }
        let strip = keyword_strip(&p.keywords);
        if !strip.is_empty() {
            desired.insert(p.id, strip);
        }
    }

    // Project a card-top world point to a viewport pixel, centring a strip of
    // `chars` glyphs over the card and lifting it above the top edge.
    let anchor = |world: Vec3, chars: usize| -> Option<(f32, f32)> {
        camera.world_to_viewport(cam_xform, world).ok().map(|v| {
            (v.x - chars as f32 * KW_CHAR_PX * 0.5, v.y - KW_LIFT)
        })
    };

    // Update existing strips; despawn any whose creature lost all keywords or
    // left the battlefield.
    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, label, mut node, mut text) in &mut labels {
        match desired.get(&label.0) {
            Some(strip) => {
                seen.insert(label.0);
                if let Some(world) = card_top.get(&label.0).copied()
                    && let Some((x, y)) = anchor(world, strip.chars().count())
                {
                    node.display = Display::Flex;
                    node.left = Val::Px(x);
                    node.top = Val::Px(y);
                } else {
                    node.display = Display::None;
                }
                if text.0 != *strip {
                    *text = Text::new(strip.clone());
                }
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    // Spawn strips for newly-keyworded creatures.
    for (id, strip) in desired {
        if seen.contains(&id) {
            continue;
        }
        let (left, top) = card_top
            .get(&id)
            .copied()
            .and_then(|world| anchor(world, strip.chars().count()))
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            KeywordLabel(id),
            Text::new(strip),
            ui_fonts.tf(12.0),
            TextColor(Color::srgb(0.96, 0.94, 0.80)),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.62)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(KW_Z),
            InGameRoot,
        ));
    }
}
