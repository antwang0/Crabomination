use std::f32::consts::PI;

use bevy::prelude::*;

use super::components::{
    BackFaceMesh, Card, CardFrontTexture, CardHighlightAssets, CardHoverLift, CardMeshAssets,
    FrontFaceMesh,
    GraveyardPile, PileHovered, PlayerTargetZone,
    CARD_THICKNESS,
};
use super::layout::{back_face_rotation, graveyard_position};
use super::mesh::{card_border_mesh, card_mesh};
use super::observers::{on_card_out, on_card_over};

use crate::game::GraveyardBrowserState;
use crate::scryfall;

/// Initialize shared mesh/material assets and spawn always-present scaffolding
/// (per-seat graveyard piles and per-opponent target zones). Per-card entities
/// are spawned on-demand by `sync_game_visuals` once a `ClientView` arrives.
///
/// `n_seats` and `viewer_seat` determine the per-seat layout: the viewer sits
/// at the front of the table; each opponent gets a graveyard pile and a
/// click-to-target zone at the back of the table.
pub fn init_shared_assets(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    segments: usize,
    n_seats: usize,
    viewer_seat: usize,
) {
    let card_mesh_handle = card_mesh(meshes, segments);
    let border_mesh_handle = card_border_mesh(meshes, segments);

    // Lives at the asset-dir root rather than under `cards/` because
    // `cards/` is gitignored (downloaded card art).
    let back_texture: Handle<Image> = asset_server.load("cardback.png");
    // Unlit so the card back shows its art at full fidelity, matching the
    // unlit faces (see `card_front_material`). A lit PBR face is dimmed by
    // the angled key light and has its blacks lifted by ambient, which reads
    // as washed-out / faint — exactly what we don't want on text-heavy art.
    let back_material = materials.add(StandardMaterial {
        base_color_texture: Some(back_texture),
        unlit: true,
        ..default()
    });

    let border_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    // Green "castable now" border — visually distinct from the gold
    // hover / targeting border so the two highlights don't read as the
    // same affordance.
    let castable_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.90, 0.35),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    // Cyan "playable only via an alternative cost" border (Dash / pitch).
    let alt_castable_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.80, 0.95),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    // Red "will die in combat" border.
    let dying_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.12, 0.12),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    // Violet "can activate an ability now" border for battlefield permanents.
    let activatable_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.70, 0.45, 1.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(CardHighlightAssets {
        border_mesh: border_mesh_handle,
        border_material,
        castable_material,
        alt_castable_material,
        dying_material,
        activatable_material,
    });
    commands.insert_resource(CardMeshAssets {
        card_mesh: card_mesh_handle.clone(),
        back_material: back_material.clone(),
    });

    // One graveyard pile per seat. Hidden until non-empty.
    for seat in 0..n_seats {
        let pos = graveyard_position(seat, viewer_seat, n_seats);
        let rot = back_face_rotation(seat, viewer_seat, n_seats);
        commands
            .spawn((
                Mesh3d(card_mesh_handle.clone()),
                MeshMaterial3d(back_material.clone()),
                Transform::from_translation(pos).with_rotation(rot),
                Visibility::Hidden,
                GraveyardPile { owner: seat },
                CardHoverLift {
                    current_lift: 0.0,
                    target_lift: 0.0,
                    base_translation: pos,
                },
            ))
            .observe(on_pile_over)
            .observe(on_pile_out)
            .observe(on_graveyard_click);
    }

    // Per-seat **player target marker** — a bare, invisible entity
    // carrying the [`PlayerTargetZone`] tag for each seat. The player's
    // visible representation is now the 2-D HUD panel (avatar / life /
    // state), and mouse clicks are submitted from there
    // (`poll_player_chip_clicks`); combat-lurch and targeting arrows
    // point at the hand fan (`player_hand_anchor`).
    //
    // This marker exists only so keyboard targeting has an entity to
    // stamp `CardHovered` onto: `kb_cursor::apply_keyboard_selection`
    // places the selection here and `handle_game_input`'s
    // `hovered_target_zone` query reads it to submit the Player target.
    // The visual cue for the keyboard-selected seat lives on its 2-D
    // panel (see `update_player_chip_target_outline`).
    for seat in 0..n_seats {
        commands.spawn(PlayerTargetZone(seat));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_single_card(
    commands: &mut Commands,
    card_mesh: &Handle<Mesh>,
    front_material: Handle<StandardMaterial>,
    back_material: Handle<StandardMaterial>,
    transform: Transform,
    game_card_id: super::components::GameCardId,
    card_name: &str,
    base_pos: Vec3,
) -> Entity {
    commands
        .spawn((
            transform,
            Visibility::default(),
            Card,
            game_card_id,
            CardFrontTexture(scryfall::card_asset_path(card_name)),
            CardHoverLift {
                current_lift: 0.0,
                target_lift: 0.0,
                base_translation: base_pos,
            },
        ))
        .with_children(|parent| {
            // Front face (+Z)
            parent
                .spawn((
                    Mesh3d(card_mesh.clone()),
                    MeshMaterial3d(front_material),
                    Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                    FrontFaceMesh,
                ))
                .observe(on_card_over)
                .observe(on_card_out);
            // Back face (-Z)
            parent
                .spawn((
                    Mesh3d(card_mesh.clone()),
                    MeshMaterial3d(back_material),
                    Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                        .with_rotation(Quat::from_rotation_y(PI)),
                    BackFaceMesh,
                ))
                .observe(on_card_over)
                .observe(on_card_out);
        })
        .id()
}

fn on_pile_over(ev: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(ev.entity).insert(PileHovered);
}

fn on_pile_out(ev: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(ev.entity).remove::<PileHovered>();
}

/// Click on any graveyard pile toggles the browser to that pile's owner.
fn on_graveyard_click(
    ev: On<Pointer<Click>>,
    piles: Query<&GraveyardPile>,
    mut browser: ResMut<GraveyardBrowserState>,
) {
    let Ok(pile) = piles.get(ev.entity) else { return };
    if browser.open && browser.owner == pile.owner {
        browser.open = false;
    } else {
        browser.open = true;
        browser.owner = pile.owner;
    }
}

pub fn card_front_material(
    name: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
    let asset_path = scryfall::card_asset_path(name);
    let texture: Handle<Image> = asset_server.load(&asset_path);
    // Unlit: render the card art exactly as authored, like the Alt-zoom
    // popup (a screen-space `ImageNode`). A lit PBR material dims the flat,
    // angled face below full white and lets ambient light lift the dark text
    // toward gray — both of which make the text read as faint. The card is
    // pre-rendered art, not a surface we want physically shaded; showing it
    // unlit keeps full contrast on every quality tier.
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        unlit: true,
        ..default()
    })
}

/// Build a textured material for an MDFC back-face image. Loads from the
/// `_back`-suffixed asset path so stale front-face downloads with the
/// same name don't collide.
pub fn card_back_face_material(
    name: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
    let asset_path = scryfall::card_back_face_asset_path(name);
    let texture: Handle<Image> = asset_server.load(&asset_path);
    // Unlit for full-fidelity art, same rationale as `card_front_material`.
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        unlit: true,
        ..default()
    })
}
