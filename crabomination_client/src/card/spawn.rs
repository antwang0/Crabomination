use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::components::{
    P1DeckPile, Card, CardFrontTexture, CardHighlightAssets, CardHoverLift, CardMeshAssets,
    CardOwner, DeckCard, GameCardId, GraveyardPile, HandCard, PileHovered, PlayerTargetZone,
    CARD_THICKNESS, P1_DECK_POSITION, P1_GRAVEYARD_POSITION, DECK_CARD_Y_STEP, DECK_POSITION,
    P0_GRAVEYARD_POSITION,
};
use super::hand::hand_card_transform;
use super::mesh::{card_border_mesh, card_mesh};
use super::observers::{on_card_out, on_card_over, on_zone_out, on_zone_over};

use crate::game::{GraveyardBrowserState, GameResource, PLAYER_1, PLAYER_0};
use crate::scryfall;

/// Spawn 3D card entities for both players' libraries (deck) and opening hands.
pub fn spawn_game_cards(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    game: &GameResource,
    segments: usize,
) {
    let card_mesh_handle = card_mesh(meshes, segments);
    let border_mesh_handle = card_border_mesh(meshes, segments);

    let back_texture: Handle<Image> = asset_server.load("cards/cardback.png");
    let back_material = materials.add(StandardMaterial {
        base_color_texture: Some(back_texture),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    // Highlight assets (shared by hover system)
    let border_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.0),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(CardHighlightAssets {
        border_mesh: border_mesh_handle,
        border_material,
    });
    commands.insert_resource(CardMeshAssets {
        card_mesh: card_mesh_handle.clone(),
        back_material: back_material.clone(),
    });

    let state = &game.state;

    // ── Player 0 cards ───────────────────────────────────────────────────────

    // Library → DeckCard entities
    for (i, card) in state.players[PLAYER_0].library.iter().enumerate() {
        let y = i as f32 * DECK_CARD_Y_STEP + 0.01;
        let front_mat = card_front_material(card.definition.name, materials, asset_server);
        let pos = Vec3::new(DECK_POSITION.x, y, DECK_POSITION.z);

        let entity = spawn_single_card(
            commands,
            &card_mesh_handle,
            front_mat,
            back_material.clone(),
            Transform::from_translation(pos)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2) * Quat::from_rotation_z(PI)),
            GameCardId(card.id),
            card.definition.name,
            pos,
        );
        commands
            .entity(entity)
            .insert((DeckCard { index: i }, CardOwner(PLAYER_0)));
    }

    // Hand → HandCard entities
    let hand = &state.players[PLAYER_0].hand;
    let total = hand.len();
    for (i, card) in hand.iter().enumerate() {
        let target = hand_card_transform(i, total);
        let front_mat = card_front_material(card.definition.name, materials, asset_server);

        let entity = spawn_single_card(
            commands,
            &card_mesh_handle,
            front_mat,
            back_material.clone(),
            target,
            GameCardId(card.id),
            card.definition.name,
            target.translation,
        );
        commands
            .entity(entity)
            .insert((HandCard { slot: i }, CardOwner(PLAYER_0)));
    }

    // ── Player 1 deck pile — one face-down entity per library card ────────────
    {
        let p1_lib = state.players[PLAYER_1].library.len();
        let rot = Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI);
        for i in 0..p1_lib {
            let y = i as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(P1_DECK_POSITION.x, y, P1_DECK_POSITION.z);
            commands.spawn((
                Mesh3d(card_mesh_handle.clone()),
                MeshMaterial3d(back_material.clone()),
                Transform::from_translation(pos).with_rotation(rot),
                Visibility::default(),
                P1DeckPile { index: i },
                CardHoverLift { current_lift: 0.0, target_lift: 0.0, base_translation: pos },
            ))
            .observe(on_pile_over)
            .observe(on_pile_out);
        }
    }

    // ── Graveyard piles (initially empty, visual entities for both players) ──
    // Player 0 GY: same orientation as player 0 BF cards (rotation_x(-PI/2)).
    // Player 1 GY: same orientation as player 1 BF cards (rotation_x(-PI/2) * rotation_z(PI)).
    commands.spawn((
        Mesh3d(card_mesh_handle.clone()),
        MeshMaterial3d(back_material.clone()),
        Transform::from_translation(P0_GRAVEYARD_POSITION)
            .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        Visibility::Hidden,
        GraveyardPile { owner: PLAYER_0 },
        CardHoverLift { current_lift: 0.0, target_lift: 0.0, base_translation: P0_GRAVEYARD_POSITION },
    ))
    .observe(on_pile_over)
    .observe(on_pile_out)
    .observe(on_graveyard_click::<PLAYER_0>);

    commands.spawn((
        Mesh3d(card_mesh_handle.clone()),
        MeshMaterial3d(back_material.clone()),
        Transform::from_translation(P1_GRAVEYARD_POSITION)
            .with_rotation(Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI)),
        Visibility::Hidden,
        GraveyardPile { owner: PLAYER_1 },
        CardHoverLift { current_lift: 0.0, target_lift: 0.0, base_translation: P1_GRAVEYARD_POSITION },
    ))
    .observe(on_pile_over)
    .observe(on_pile_out)
    .observe(on_graveyard_click::<PLAYER_1>);

    // ── Player target zones (invisible clickable areas representing players) ─
    // Player 1 target zone: placed near player 1's battlefield side
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(8.0, 3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.0, 0.0, 0.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.01, -6.0),
        Visibility::default(),
        PlayerTargetZone(PLAYER_1),
    ))
    .observe(on_zone_over)
    .observe(on_zone_out);
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_single_card(
    commands: &mut Commands,
    card_mesh: &Handle<Mesh>,
    front_material: Handle<StandardMaterial>,
    back_material: Handle<StandardMaterial>,
    transform: Transform,
    game_card_id: GameCardId,
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

fn on_graveyard_click<const OWNER: usize>(
    _ev: On<Pointer<Click>>,
    mut browser: ResMut<GraveyardBrowserState>,
) {
    if browser.open && browser.owner == OWNER {
        browser.open = false;
    } else {
        browser.open = true;
        browser.owner = OWNER;
    }
}

pub fn card_front_material(
    name: &str,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
    let asset_path = scryfall::card_asset_path(name);
    let texture: Handle<Image> = asset_server.load(&asset_path);
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    })
}
