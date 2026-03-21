//! Game HUD overlay and all game-logic Bevy systems:
//! bot AI, human auto-advance, player input, visual sync, and text refresh.

use std::collections::{HashMap, HashSet};
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use crabomination::card::{CardId, SpellEffect};
use crabomination::game::{GameAction, StackItem, Target, TurnStep};
use crabomination::mana::{Color as ManaColor, ManaCost, ManaSymbol};

use crate::bot::{bot_declare_blocks, bot_take_action, human_auto_target};
use crate::card::{
    bf_card_transform, bot_hand_card_transform, card_front_material, hand_card_transform,
    land_group_info, spawn_single_card, BattlefieldCard, BotDeckPile, BotHandCard, CardHoverLift,
    CardHovered, CardMeshAssets, CardOwner, DeckCard, DrawCardAnimation, GameCardId, GraveyardPile,
    HandCard, HandSlideAnimation, PlayCardAnimation, PlayerTargetZone, RevealPeekAnimation,
    SendToGraveyardAnimation, StackCard, TapAnimation, TapState, ValidTarget, BOT_DECK_POSITION,
    BOT_GRAVEYARD_POSITION, CARD_THICKNESS, CARD_WIDTH, DECK_CARD_Y_STEP, HUMAN_GRAVEYARD_POSITION,
    LAND_STACK_OFFSET_X, LAND_STACK_OFFSET_Z,
};
use crate::game::{
    format_mana_pool, BlockingState, BotTimer, GameLog, GameResource, TargetingState, BOT, HUMAN,
};
use super::ui::RevealPopupState;

/// Max number of PassPriority calls issued by the "End Turn" button/key.
/// Enough to cover all remaining steps in a turn without risking an infinite loop.
const MAX_END_TURN_PASSES: usize = 20;

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TurnInfoText;

#[derive(Component)]
pub struct PlayerStatusText;

#[derive(Component)]
pub struct BotStatusText;

#[derive(Component)]
pub struct GameLogText;

#[derive(Component)]
pub struct HintText;

#[derive(Component)]
pub struct PassPriorityButton;

#[derive(Component)]
pub struct AttackAllButton;

#[derive(Component)]
pub struct EndTurnButton;

#[derive(Component)]
pub struct PhaseStepLabel(pub TurnStep);

/// Custom gizmo config group for blocking indicators (thicker lines).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BlockingGizmos;

/// Custom gizmo config group for attack indicators (sword overlay).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AttackerGizmos;

// ── HUD setup ─────────────────────────────────────────────────────────────────

pub fn setup_game_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let tf = |size: f32| TextFont { font: font.clone(), font_size: size, ..default() };

    // Top-left: turn / step info
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
        ))
        .with_children(|p| {
            p.spawn((Text::new(""), tf(16.0), TextColor(Color::WHITE), TurnInfoText));
        });

    // Left side: phase chart
    let all_steps = [
        (TurnStep::Untap,           "Untap"),
        (TurnStep::Upkeep,          "Upkeep"),
        (TurnStep::Draw,            "Draw"),
        (TurnStep::PreCombatMain,   "Main 1"),
        (TurnStep::BeginCombat,     "Begin Combat"),
        (TurnStep::DeclareAttackers,"Attackers"),
        (TurnStep::DeclareBlockers, "Blockers"),
        (TurnStep::CombatDamage,    "Damage"),
        (TurnStep::EndCombat,       "End Combat"),
        (TurnStep::PostCombatMain,  "Main 2"),
        (TurnStep::End,             "End"),
        (TurnStep::Cleanup,         "Cleanup"),
    ];
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(6.0)),
                row_gap: Val::Px(2.0),
                min_width: Val::Px(110.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|p| {
            for (step, label) in &all_steps {
                p.spawn((
                    Text::new(*label),
                    tf(12.0),
                    TextColor(Color::srgba(0.55, 0.55, 0.55, 0.8)),
                    PhaseStepLabel(*step),
                ));
            }
        });

    // Top-right: bot status
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                min_width: Val::Px(270.0),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.0, 0.0, 0.82)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                tf(16.0),
                TextColor(Color::srgb(1.0, 0.65, 0.65)),
                BotStatusText,
            ));
        });

    // Right side: game log
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(120.0),
                right: Val::Px(10.0),
                width: Val::Px(280.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                tf(12.0),
                TextColor(Color::srgb(0.78, 0.78, 0.78)),
                GameLogText,
            ));
        });

    // Bottom: player panel
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                max_width: Val::Px(560.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.1, 0.22, 0.82)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                tf(14.0),
                TextColor(Color::srgb(0.65, 0.88, 1.0)),
                PlayerStatusText,
            ));
            p.spawn((
                Text::new(""),
                tf(13.0),
                TextColor(Color::srgb(1.0, 0.9, 0.35)),
                HintText,
            ));
            // Action buttons row
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    Node { padding: UiRect::all(Val::Px(8.0)), ..default() },
                    BackgroundColor(Color::srgb(0.45, 0.08, 0.08)),
                    Button,
                    AttackAllButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Attack All (A)"),
                        tf(13.0),
                        TextColor(Color::WHITE),
                    ));
                });

                p.spawn((
                    Node { padding: UiRect::all(Val::Px(8.0)), ..default() },
                    BackgroundColor(Color::srgb(0.08, 0.28, 0.48)),
                    Button,
                    PassPriorityButton,
                ))
                .with_children(|p| {
                    p.spawn((Text::new("Pass (P)"), tf(13.0), TextColor(Color::WHITE)));
                });

                p.spawn((
                    Node { padding: UiRect::all(Val::Px(8.0)), ..default() },
                    BackgroundColor(Color::srgb(0.08, 0.38, 0.18)),
                    Button,
                    EndTurnButton,
                ))
                .with_children(|p| {
                    p.spawn((Text::new("End Turn (E)"), tf(13.0), TextColor(Color::WHITE)));
                });
            });
        });
}

// ── HUD text update ───────────────────────────────────────────────────────────

pub fn update_turn_text(
    game: Res<GameResource>,
    mut q: Query<&mut Text, With<TurnInfoText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let s = &game.state;
    t.0 = if let Some(winner) = s.game_over {
        match winner {
            Some(p) => format!("GAME OVER — {} wins!", if p == HUMAN { "You" } else { "Bot" }),
            None => "GAME OVER — Draw!".into(),
        }
    } else {
        format!(
            "Turn {} | {:?} | {}'s turn",
            s.turn_number,
            s.step,
            if s.active_player_idx == HUMAN { "Your" } else { "Bot's" }
        )
    };
}

pub fn update_player_text(
    game: Res<GameResource>,
    mut q: Query<&mut Text, With<PlayerStatusText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let s = &game.state;
    let p = &s.players[HUMAN];
    t.0 = format!(
        "You | Life: {} | Hand: {} | Deck: {} | GY: {} | Mana: {}",
        p.life,
        p.hand.len(),
        p.library.len(),
        p.graveyard.len(),
        format_mana_pool(s, HUMAN)
    );
}

pub fn update_bot_text(
    game: Res<GameResource>,
    mut q: Query<&mut Text, With<BotStatusText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let s = &game.state;
    let p = &s.players[BOT];
    t.0 = format!(
        "Bot | Life: {} | Hand: {} | Deck: {} | GY: {}",
        p.life, p.hand.len(), p.library.len(), p.graveyard.len()
    );
}

pub fn update_game_log(log: Res<GameLog>, mut q: Query<&mut Text, With<GameLogText>>) {
    let Ok(mut t) = q.single_mut() else { return };
    t.0 = log.entries.join("\n");
}

pub fn update_phase_chart(
    game: Res<GameResource>,
    mut labels: Query<(&PhaseStepLabel, &mut TextColor)>,
) {
    let current = game.state.step;
    for (label, mut color) in &mut labels {
        *color = if label.0 == current {
            TextColor(Color::srgb(1.0, 0.88, 0.0))
        } else {
            TextColor(Color::srgba(0.55, 0.55, 0.55, 0.8))
        };
    }
}

pub fn draw_blocking_gizmos(
    game: Res<GameResource>,
    blocking: Res<BlockingState>,
    bf_cards: Query<(&Transform, &GameCardId, &CardOwner), With<BattlefieldCard>>,
    mut gizmos: Gizmos<BlockingGizmos>,
) {
    if game.state.step != TurnStep::DeclareBlockers || game.state.active_player_idx != BOT {
        return;
    }

    // Build id → world position map
    let mut positions: HashMap<crabomination::card::CardId, Vec3> = HashMap::new();
    for (transform, gid, _) in &bf_cards {
        positions.insert(gid.0, transform.translation + Vec3::Y * 0.15);
    }

    let attacking = game.state.attacking();

    // Highlight unblocked attackers in red, blocked in green
    for &attacker_id in attacking {
        let is_blocked = blocking.assignments.iter().any(|(_, a)| *a == attacker_id);
        if let Some(&pos) = positions.get(&attacker_id) {
            let color = if is_blocked { Color::srgb(0.0, 0.9, 0.3) } else { Color::srgb(1.0, 0.2, 0.2) };
            draw_diamond(&mut gizmos, pos, 1.1, color);
        }
    }

    // Highlight selected blocker in yellow + guide arrows to each unassigned attacker
    if let Some(blocker_id) = blocking.selected_blocker {
        if let Some(&pos) = positions.get(&blocker_id) {
            draw_diamond(&mut gizmos, pos, 1.1, Color::srgb(1.0, 0.88, 0.0));
            for &attacker_id in attacking {
                let already_assigned = blocking.assignments.iter().any(|(_, a)| *a == attacker_id);
                if !already_assigned {
                    if let Some(&att_pos) = positions.get(&attacker_id) {
                        gizmos.arrow(pos, att_pos, Color::srgba(1.0, 0.88, 0.0, 0.7))
                            .with_tip_length(0.6);
                    }
                }
            }
        }
    }

    // Draw confirmed assignment arrows in green
    for &(blocker_id, attacker_id) in &blocking.assignments {
        if let Some(&b_pos) = positions.get(&blocker_id) {
            if let Some(&a_pos) = positions.get(&attacker_id) {
                gizmos.arrow(b_pos, a_pos, Color::srgb(0.0, 0.9, 0.3))
                    .with_tip_length(0.6);
                draw_diamond(&mut gizmos, b_pos, 1.1, Color::srgb(0.0, 0.9, 0.3));
            }
        }
    }
}

pub fn draw_attacker_overlays(
    game: Res<GameResource>,
    bf_cards: Query<(&Transform, &GameCardId), With<BattlefieldCard>>,
    mut gizmos: Gizmos<AttackerGizmos>,
) {
    let attacking = game.state.attacking();
    if attacking.is_empty() {
        return;
    }

    let mut positions: HashMap<crabomination::card::CardId, Vec3> = HashMap::new();
    for (transform, gid) in &bf_cards {
        positions.insert(gid.0, transform.translation);
    }

    for &attacker_id in attacking {
        if let Some(&pos) = positions.get(&attacker_id) {
            draw_crossed_swords(&mut gizmos, pos, Color::srgb(1.0, 0.35, 0.05));
        }
    }
}

/// Draw a crossed-swords (⚔) symbol in the XZ plane at `pos`, slightly raised.
fn draw_crossed_swords(gizmos: &mut Gizmos<AttackerGizmos>, pos: Vec3, color: Color) {
    let y = pos.y + 0.18;
    // Blade half-length and guard half-length
    let r: f32 = 0.7;
    let g: f32 = 0.22;
    // Guard sits 35% from the handle toward the tip on each sword
    let gf: f32 = 0.35;

    // Sword 1: handle at (-x,-z), tip at (+x,+z)
    let s1_handle = Vec3::new(pos.x - r, y, pos.z - r);
    let s1_tip    = Vec3::new(pos.x + r, y, pos.z + r);
    let s1_guard  = s1_handle.lerp(s1_tip, gf);
    // Guard perpendicular to blade 1 → along (+x,-z) direction
    let gd1 = Vec3::new(1.0, 0.0, -1.0).normalize() * g;
    gizmos.line(s1_handle, s1_tip, color);
    gizmos.line(s1_guard - gd1, s1_guard + gd1, color);

    // Sword 2: handle at (+x,-z), tip at (-x,+z)
    let s2_handle = Vec3::new(pos.x + r, y, pos.z - r);
    let s2_tip    = Vec3::new(pos.x - r, y, pos.z + r);
    let s2_guard  = s2_handle.lerp(s2_tip, gf);
    // Guard perpendicular to blade 2 → along (+x,+z) direction
    let gd2 = Vec3::new(1.0, 0.0, 1.0).normalize() * g;
    gizmos.line(s2_handle, s2_tip, color);
    gizmos.line(s2_guard - gd2, s2_guard + gd2, color);
}

fn draw_diamond(gizmos: &mut Gizmos<BlockingGizmos>, pos: Vec3, r: f32, color: Color) {
    let n = pos + Vec3::Z * r;
    let s = pos - Vec3::Z * r;
    let e = pos + Vec3::X * r;
    let w = pos - Vec3::X * r;
    gizmos.line(n, e, color);
    gizmos.line(e, s, color);
    gizmos.line(s, w, color);
    gizmos.line(w, n, color);
}

pub fn update_hint(
    game: Res<GameResource>,
    targeting: Res<TargetingState>,
    blocking: Res<BlockingState>,
    mut q: Query<&mut Text, With<HintText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    if game.state.is_game_over() {
        t.0 = String::new();
        return;
    }
    if targeting.active {
        t.0 = "Click a target (creature or opponent). Right-click/Esc to cancel.".into();
        return;
    }
    let stack_size = game.state.stack.len();
    if stack_size > 0 {
        t.0 = format!("{stack_size} item(s) on stack. P = Resolve.");
        return;
    }
    // Human defending against bot's attack
    if game.state.step == TurnStep::DeclareBlockers && game.state.active_player_idx == BOT {
        t.0 = if blocking.selected_blocker.is_some() {
            "Click an attacking creature to assign the block. Right-click to cancel.".into()
        } else {
            "Click one of your creatures to block with it. P = Skip blocking.".into()
        };
        return;
    }
    t.0 = match (game.state.active_player_idx, game.state.step) {
        (HUMAN, TurnStep::PreCombatMain) | (HUMAN, TurnStep::PostCombatMain) => {
            "Click a hand card to play it. P = Pass Priority.".into()
        }
        (HUMAN, TurnStep::DeclareAttackers) => {
            "A = Attack with all eligible creatures. P = Pass (no attack).".into()
        }
        (HUMAN, TurnStep::DeclareBlockers) => {
            "Bot is assigning blocks. P = Proceed to combat.".into()
        }
        (HUMAN, _) => String::new(),
        (BOT, _) => "Bot is thinking...".into(),
        _ => String::new(),
    };
}

// ── Visual sync: reconcile 3D card entities with game state ──────────────────

/// Helper: count cards per row for a given owner.
fn bf_row_counts(
    state: &crabomination::game::GameState,
    owner: usize,
) -> (usize, usize) {
    let lands = state.battlefield.iter().filter(|c| c.owner == owner && c.definition.is_land()).count();
    let creatures = state.battlefield.iter().filter(|c| c.owner == owner && !c.definition.is_land()).count();
    (lands, creatures)
}

/// Find the slot of a card within its row.
fn bf_row_slot(
    state: &crabomination::game::GameState,
    owner: usize,
    card_id: CardId,
    is_land: bool,
) -> Option<usize> {
    state
        .battlefield
        .iter()
        .filter(|c| c.owner == owner && c.definition.is_land() == is_land)
        .position(|c| c.id == card_id)
}

/// Returns the world-space transform for a land card accounting for identical-land stacking.
fn land_card_transform(
    state: &crabomination::game::GameState,
    owner: usize,
    card_id: crabomination::card::CardId,
    is_bot: bool,
) -> Option<Transform> {
    let (group_slot, index, total_groups) = land_group_info(state, owner, card_id)?;
    let base = bf_card_transform(group_slot, total_groups, true, is_bot, false);
    let sign = if is_bot { -1.0 } else { 1.0 };
    let stagger = Vec3::new(
        index as f32 * LAND_STACK_OFFSET_X * sign,
        index as f32 * CARD_THICKNESS * 1.5,
        index as f32 * LAND_STACK_OFFSET_Z * sign,
    );
    Some(Transform {
        translation: base.translation + stagger,
        rotation: base.rotation,
        scale: base.scale,
    })
}

pub fn sync_game_visuals(
    mut commands: Commands,
    game: Res<GameResource>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_assets: Option<Res<CardMeshAssets>>,
    hand_cards: Query<(Entity, &GameCardId, &Transform, Option<&StackCard>), (With<HandCard>, Without<SendToGraveyardAnimation>, Without<DrawCardAnimation>)>,
    deck_cards: Query<(Entity, &GameCardId, &DeckCard, &Transform), Without<HandCard>>,
    bf_cards: Query<(Entity, &GameCardId, &CardOwner, &BattlefieldCard, &Transform, Option<&TapState>), (Without<HandCard>, Without<SendToGraveyardAnimation>)>,
    mut bot_deck_q: Query<(Entity, &BotDeckPile, &mut Transform, &mut CardHoverLift), Without<GameCardId>>,
    mut graveyard_q: Query<(&GraveyardPile, &mut Transform, &mut Visibility, &mut CardHoverLift, &mut MeshMaterial3d<StandardMaterial>), (Without<BotDeckPile>, Without<GameCardId>)>,
    bot_hand_q: Query<(Entity, &BotHandCard, &Transform), (Without<BotDeckPile>, Without<GraveyardPile>, Without<SendToGraveyardAnimation>)>,
    gy_anims: Query<&SendToGraveyardAnimation>,
) {
    let state = &game.state;

    // ── Always update: deck/graveyard heights and bot hand count ─────────────
    // These must run every frame so they stay correct after spawning.

    let bot_deck_size = state.players[BOT].library.len();
    for (entity, pile, mut transform, mut lift) in &mut bot_deck_q {
        if pile.index >= bot_deck_size {
            commands.entity(entity).despawn();
        } else {
            let y = pile.index as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(BOT_DECK_POSITION.x, y, BOT_DECK_POSITION.z);
            transform.translation = pos;
            lift.base_translation = pos;
        }
    }

    // Count in-flight graveyard animations per owner to delay pile visibility.
    let mut gy_in_flight: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for anim in &gy_anims {
        *gy_in_flight.entry(anim.owner).or_default() += 1;
    }

    for (gy, mut transform, mut vis, mut lift, _mat) in &mut graveyard_q {
        let gy_count = state.players[gy.owner].graveyard.len();
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_count.saturating_sub(in_flight);
        if arrived == 0 {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Visible;
            let base_pos = if gy.owner == HUMAN { HUMAN_GRAVEYARD_POSITION } else { BOT_GRAVEYARD_POSITION };
            let y = arrived as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(base_pos.x, y, base_pos.z);
            transform.translation = pos;
            lift.base_translation = pos;
        }
    }

    if !game.is_changed() {
        return;
    }
    let Some(card_assets) = card_assets else { return };

    // ── Bot hand sync (spawn/despawn/slide on game state change) ─────────────
    {
        let bot_hand_size = state.players[BOT].hand.len();
        let all_bot_hand: Vec<(Entity, usize, Vec3, Quat)> = bot_hand_q.iter()
            .map(|(e, bh, t)| (e, bh.slot, t.translation, t.rotation))
            .collect();
        let visual_bot_hand_size = all_bot_hand.len();

        let mut removed: HashSet<Entity> = HashSet::new();

        if visual_bot_hand_size > bot_hand_size {
            // Cards left hand — despawn immediately. The BF entity's PlayCardAnimation
            // already shows played cards flying to the battlefield; we don't want a
            // second graveyard animation that conflicts with it.
            let mut sorted = all_bot_hand.clone();
            sorted.sort_by_key(|(_, slot, _, _)| std::cmp::Reverse(*slot));
            for (entity, _, _, _) in sorted.iter().take(visual_bot_hand_size - bot_hand_size) {
                commands.entity(*entity).despawn();
                removed.insert(*entity);
            }
        } else if visual_bot_hand_size < bot_hand_size {
            // Cards added to hand — spawn new draw animations.
            let deck_y = bot_hand_size as f32 * DECK_CARD_Y_STEP + 0.5;
            let deck_pos = Vec3::new(BOT_DECK_POSITION.x, deck_y, BOT_DECK_POSITION.z);
            for slot in visual_bot_hand_size..bot_hand_size {
                let target = bot_hand_card_transform(slot, bot_hand_size);
                let back_mat = card_assets.back_material.clone();
                let card_mesh = card_assets.card_mesh.clone();
                let start_transform = Transform::from_translation(deck_pos);
                commands.spawn((
                    start_transform,
                    Visibility::default(),
                    BotHandCard { slot },
                    CardHoverLift {
                        current_lift: 0.0,
                        target_lift: 0.0,
                        base_translation: target.translation,
                    },
                    PlayCardAnimation {
                        progress: 0.0,
                        speed: 2.0,
                        start_translation: deck_pos,
                        start_rotation: start_transform.rotation,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(card_mesh.clone()),
                        MeshMaterial3d(back_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                    ));
                    parent.spawn((
                        Mesh3d(card_mesh.clone()),
                        MeshMaterial3d(back_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                            .with_rotation(Quat::from_rotation_y(PI)),
                    ));
                });
            }
        }

        // Slide remaining cards to their new positions.
        for (entity, slot, pos, _rot) in &all_bot_hand {
            if removed.contains(entity) { continue; }
            let target = bot_hand_card_transform(*slot, bot_hand_size);
            if pos.distance(target.translation) > 0.001 {
                commands.entity(*entity)
                    .insert(HandSlideAnimation {
                        progress: 0.0,
                        speed: 4.0,
                        start_translation: *pos,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                    })
                    .insert(CardHoverLift {
                        current_lift: 0.0,
                        target_lift: 0.0,
                        base_translation: target.translation,
                    });
            }
        }
    }

    // ── Update graveyard pile face-up material ────────────────────────────────
    for (gy, _transform, _vis, _lift, mut mat) in &mut graveyard_q {
        let graveyard = &state.players[gy.owner].graveyard;
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = graveyard.len().saturating_sub(in_flight);
        if arrived > 0 {
            if let Some(top) = graveyard.get(arrived - 1) {
                *mat = MeshMaterial3d(card_front_material(top.definition.name, &mut materials, &asset_server));
            }
        }
    }

    // Build sets of CardIds in each game-engine zone
    let stack_ids: HashSet<CardId> = state.stack.iter().filter_map(|item| {
        if let StackItem::Spell { card, .. } = item { Some(card.id) } else { None }
    }).collect();
    let hand_ids: HashSet<CardId> = state.players[HUMAN].hand.iter().map(|c| c.id).collect();
    let human_bf_ids: HashSet<CardId> = state.battlefield.iter().filter(|c| c.owner == HUMAN).map(|c| c.id).collect();
    let bot_bf_ids: HashSet<CardId> = state.battlefield.iter().filter(|c| c.owner == BOT).map(|c| c.id).collect();
    let all_bf_ids: HashSet<CardId> = human_bf_ids.iter().chain(bot_bf_ids.iter()).copied().collect();

    let hand_total = hand_ids.len();
    let (human_lands, human_creatures) = bf_row_counts(state, HUMAN);
    let (bot_lands, bot_creatures) = bf_row_counts(state, BOT);

    let visual_bf_ids: HashSet<CardId> = bf_cards.iter().map(|(_, gid, _, _, _, _)| gid.0).collect();

    // ── Deck → Hand transitions ──────────────────────────────────────────────
    for (entity, game_id, _deck_card, transform) in &deck_cards {
        if hand_ids.contains(&game_id.0) {
            let slot = state.players[HUMAN].hand.iter().position(|c| c.id == game_id.0)
                .unwrap_or(hand_total.saturating_sub(1));
            let target = hand_card_transform(slot, hand_total);
            commands.entity(entity)
                .remove::<DeckCard>()
                .insert(HandCard { slot })
                .insert(DrawCardAnimation {
                    progress: 0.0, speed: 1.5,
                    start_translation: transform.translation, start_rotation: transform.rotation,
                    target_translation: target.translation, target_rotation: target.rotation,
                });
        }
    }

    // ── Hand → Battlefield / Stack transitions ───────────────────────────────
    for (entity, game_id, transform, stack_card) in &hand_cards {
        if human_bf_ids.contains(&game_id.0) {
            let card_inst = state.battlefield.iter().find(|c| c.id == game_id.0);
            let is_land = card_inst.map_or(false, |c| c.definition.is_land());
            let target = if is_land {
                land_card_transform(state, HUMAN, game_id.0, false).unwrap_or_else(|| {
                    bf_card_transform(0, 1, true, false, false)
                })
            } else {
                let row_total = human_creatures;
                let slot = bf_row_slot(state, HUMAN, game_id.0, false).unwrap_or(0);
                bf_card_transform(slot, row_total, false, false, false)
            };
            commands.entity(entity)
                .remove::<HandCard>()
                .remove::<StackCard>()
                .remove::<CardHovered>()
                .insert(BattlefieldCard { is_land })
                .insert(CardOwner(HUMAN))
                .insert(TapState { tapped: false })
                .insert(PlayCardAnimation {
                    progress: 0.0, speed: 2.0,
                    start_translation: transform.translation, start_rotation: transform.rotation,
                    target_translation: target.translation, target_rotation: target.rotation,
                });
        } else if stack_ids.contains(&game_id.0) {
            if stack_card.is_none() {
                // First time we see this card on the stack — fly it to the center
                let idx = state.stack.iter().position(|item| {
                    matches!(item, StackItem::Spell { card, .. } if card.id == game_id.0)
                }).unwrap_or(0);
                let total = stack_ids.len();
                let target = stack_card_transform(idx, total);
                commands.entity(entity)
                    .remove::<CardHovered>()
                    .insert(StackCard)
                    .insert(PlayCardAnimation {
                        progress: 0.0, speed: 2.0,
                        start_translation: transform.translation, start_rotation: transform.rotation,
                        target_translation: target.translation, target_rotation: target.rotation,
                    });
            }
        } else if !hand_ids.contains(&game_id.0) {
            commands.entity(entity)
                .remove::<HandCard>()
                .remove::<CardHovered>()
                .insert(SendToGraveyardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: HUMAN_GRAVEYARD_POSITION,
                    target_rotation: Quat::from_rotation_x(-FRAC_PI_2),
                    owner: HUMAN,
                });
        }
    }

    // ── Battlefield → Graveyard transitions ──────────────────────────────────
    for (entity, game_id, owner, _bf, transform, _) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) {
            let (gy_pos, gy_rot) = if owner.0 == HUMAN {
                (HUMAN_GRAVEYARD_POSITION, Quat::from_rotation_x(-FRAC_PI_2))
            } else {
                (BOT_GRAVEYARD_POSITION, Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI))
            };
            commands.entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardHovered>()
                .insert(SendToGraveyardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: gy_pos,
                    target_rotation: gy_rot,
                    owner: owner.0,
                });
        }
    }

    // ── Spawn new bot battlefield cards ──────────────────────────────────────
    // Use the bot hand center position as the animation start point.
    let bot_hand_center = bot_hand_card_transform(0, 1);
    for card in state.battlefield.iter().filter(|c| c.owner == BOT) {
        if visual_bf_ids.contains(&card.id) {
            continue;
        }
        let is_land = card.definition.is_land();
        let target = if is_land {
            land_card_transform(state, BOT, card.id, true).unwrap_or_else(|| {
                bf_card_transform(0, 1, true, true, false)
            })
        } else {
            let row_total = bot_creatures;
            let slot = bf_row_slot(state, BOT, card.id, false).unwrap_or(0);
            bf_card_transform(slot, row_total, false, true, card.tapped)
        };
        let front_mat = card_front_material(card.definition.name, &mut materials, &asset_server);

        let entity = spawn_single_card(
            &mut commands, &card_assets.card_mesh, front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(bot_hand_center.translation).with_rotation(bot_hand_center.rotation),
            GameCardId(card.id), card.definition.name, target.translation,
        );
        commands.entity(entity).insert((
            BattlefieldCard { is_land },
            CardOwner(BOT),
            TapState { tapped: card.tapped },
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: bot_hand_center.translation,
                start_rotation: bot_hand_center.rotation,
                target_translation: target.translation,
                target_rotation: target.rotation,
            },
        ));
    }

    // ── Also spawn new human battlefield cards that don't have entities yet ──
    // (e.g. cards that entered via game actions without going through hand→bf)
    for card in state.battlefield.iter().filter(|c| c.owner == HUMAN) {
        if visual_bf_ids.contains(&card.id) || hand_cards.iter().any(|(_, gid, _, _)| gid.0 == card.id) {
            continue;
        }
        let is_land = card.definition.is_land();
        let target = if is_land {
            land_card_transform(state, HUMAN, card.id, false).unwrap_or_else(|| {
                bf_card_transform(0, 1, true, false, false)
            })
        } else {
            let row_total = human_creatures;
            let slot = bf_row_slot(state, HUMAN, card.id, false).unwrap_or(0);
            bf_card_transform(slot, row_total, false, false, card.tapped)
        };
        let front_mat = card_front_material(card.definition.name, &mut materials, &asset_server);

        let entity = spawn_single_card(
            &mut commands, &card_assets.card_mesh, front_mat,
            card_assets.back_material.clone(), target,
            GameCardId(card.id), card.definition.name, target.translation,
        );
        commands.entity(entity).insert((
            BattlefieldCard { is_land },
            CardOwner(HUMAN),
            TapState { tapped: card.tapped },
        ));
    }

    // ── Rebalance hand slots ─────────────────────────────────────────────────
    for (entity, game_id, transform, stack_card) in &hand_cards {
        if stack_card.is_some() { continue; } // don't slide stack cards back to hand
        if !hand_ids.contains(&game_id.0) { continue; }
        let Some(new_slot) = state.players[HUMAN].hand.iter().position(|c| c.id == game_id.0) else { continue; };
        let target = hand_card_transform(new_slot, hand_total);
        let dist = (transform.translation - target.translation).length();
        if dist > 0.1 {
            commands.entity(entity).insert(HandSlideAnimation {
                progress: 0.0, speed: 3.0,
                start_translation: transform.translation,
                target_translation: target.translation,
                target_rotation: target.rotation,
            });
        }
        commands.entity(entity).insert(HandCard { slot: new_slot });
    }

    // ── Rebalance battlefield positions + sync tapped state ──────────────────
    for (entity, game_id, owner, bf, _transform, tap_state) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) { continue; }
        let is_bot = owner.0 == BOT;
        let is_land = bf.is_land;

        // Check tapped state from game engine
        let game_tapped = state.battlefield.iter()
            .find(|c| c.id == game_id.0)
            .map_or(false, |c| c.tapped);
        let visual_tapped = tap_state.map_or(false, |ts| ts.tapped);

        // Use stacked layout for lands, standard slot layout for creatures
        let target = if is_land {
            match land_card_transform(state, owner.0, game_id.0, is_bot) {
                Some(t) => {
                    // Apply tap rotation on top of stagger translation
                    let base_rot = t.rotation;
                    let tapped_rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * base_rot;
                    let rot = if game_tapped { tapped_rot } else { base_rot };
                    Transform { translation: t.translation, rotation: rot, scale: t.scale }
                }
                None => continue,
            }
        } else {
            let row_total = if is_bot { bot_creatures } else { human_creatures };
            let Some(slot) = bf_row_slot(state, owner.0, game_id.0, false) else { continue; };
            bf_card_transform(slot, row_total, false, is_bot, game_tapped)
        };

        if game_tapped != visual_tapped {
            let (untapped_rot, tapped_rot) = if is_land {
                let base = target.rotation;
                // For lands, tapped rot was already computed; get both variants
                let land_base = land_card_transform(state, owner.0, game_id.0, is_bot)
                    .map(|t| t.rotation)
                    .unwrap_or(target.rotation);
                let land_tapped = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * land_base;
                (land_base, land_tapped)
            } else {
                let row_total = if is_bot { bot_creatures } else { human_creatures };
                let slot = bf_row_slot(state, owner.0, game_id.0, false).unwrap_or(0);
                (
                    bf_card_transform(slot, row_total, false, is_bot, false).rotation,
                    bf_card_transform(slot, row_total, false, is_bot, true).rotation,
                )
            };
            let (start, end) = if game_tapped { (untapped_rot, tapped_rot) } else { (tapped_rot, untapped_rot) };
            commands.entity(entity)
                .insert(TapAnimation { progress: 0.0, speed: 4.0, start_rotation: start, target_rotation: end })
                .insert(TapState { tapped: game_tapped });
        }

        commands.entity(entity).insert(CardHoverLift {
            current_lift: 0.0,
            target_lift: 0.0,
            base_translation: target.translation,
        });
    }

    // ── Update human deck card visibility (remove drawn cards) ────────────
    let human_lib_ids: HashSet<CardId> = state.players[HUMAN].library.iter().map(|c| c.id).collect();
    for (entity, game_id, _deck_card, _transform) in &deck_cards {
        if !human_lib_ids.contains(&game_id.0) && !hand_ids.contains(&game_id.0) {
            commands.entity(entity).despawn();
        }
    }
}

// ── Bot system ────────────────────────────────────────────────────────────────

/// Scan `events` for `TopCardRevealed` and arm the reveal popup if found.
fn check_reveal(events: &[crabomination::game::GameEvent], reveal: &mut RevealPopupState) {
    for ev in events {
        if let crabomination::game::GameEvent::TopCardRevealed { player, card_name, .. } = ev {
            reveal.card_path = Some(crate::scryfall::card_asset_path(card_name));
            reveal.revealed_player = Some(*player);
        }
    }
}

pub fn bot_system(
    time: Res<Time>,
    mut timer: ResMut<BotTimer>,
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut reveal: ResMut<RevealPopupState>,
    mut last_key: Local<Option<(usize, TurnStep)>>,
) {
    if game.state.is_game_over() { return; }

    let current_key = (game.state.active_player_idx, game.state.step);
    let step_just_changed = *last_key != Some(current_key);
    *last_key = Some(current_key);

    let is_bot_blocking = game.state.step == TurnStep::DeclareBlockers && game.state.active_player_idx == HUMAN;
    if is_bot_blocking && step_just_changed {
        let evs = bot_declare_blocks(&mut game.state, &mut rand::rng());
        log.apply_events(&evs);
        return;
    }

    if game.state.active_player_idx != BOT { return; }

    // Wait for human to finish declaring blocks before advancing
    if game.state.step == TurnStep::DeclareBlockers { return; }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }

    let evs = bot_take_action(&mut game.state, &mut rand::rng());
    log.apply_events(&evs);
    check_reveal(&evs, &mut reveal);
}

// ── Auto-advance non-interactive steps for the human player ──────────────────

pub fn auto_advance_human(
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut reveal: ResMut<RevealPopupState>,
) {
    if game.state.is_game_over() || game.state.active_player_idx != HUMAN { return; }
    let should_advance = matches!(
        game.state.step,
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw | TurnStep::BeginCombat
            | TurnStep::CombatDamage | TurnStep::EndCombat | TurnStep::End | TurnStep::Cleanup
    );
    if should_advance {
        if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
            log.apply_events(&evs);
            check_reveal(&evs, &mut reveal);
        }
    }
}

// ── Human player input ────────────────────────────────────────────────────────

pub fn handle_game_input(
    mut commands: Commands,
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut targeting: ResMut<TargetingState>,
    mut blocking: ResMut<BlockingState>,
    mut reveal: ResMut<RevealPopupState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered_hand: Query<&GameCardId, (With<CardHovered>, With<HandCard>)>,
    hovered_bf: Query<(&GameCardId, &CardOwner), (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_target_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    valid_targets: Query<Entity, With<ValidTarget>>,
    pass_btn: Query<&Interaction, (Changed<Interaction>, With<PassPriorityButton>)>,
    attack_btn: Query<&Interaction, (Changed<Interaction>, With<AttackAllButton>)>,
    end_turn_btn: Query<&Interaction, (Changed<Interaction>, With<EndTurnButton>)>,
) {
    // ── Blocking (human defends on bot's turn) ───────────────────────────────
    if game.state.step == TurnStep::DeclareBlockers && game.state.active_player_idx == BOT {
        let mut pass = keyboard.just_pressed(KeyCode::KeyP) || keyboard.just_pressed(KeyCode::Space);
        for i in &pass_btn {
            if *i == Interaction::Pressed { pass = true; }
        }
        if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
            blocking.selected_blocker = None;
            return;
        }
        if mouse.just_pressed(MouseButton::Left) {
            if let Some((game_id, owner)) = hovered_bf.iter().next() {
                if owner.0 == HUMAN {
                    // Only allow selecting creatures that can legally block something
                    // and haven't already been assigned to an attacker.
                    let already_assigned = blocking.assignments.iter().any(|(b, _)| *b == game_id.0);
                    if game.state.can_block_any_attacker(game_id.0) && !already_assigned {
                        blocking.selected_blocker = Some(game_id.0);
                    }
                } else if owner.0 == BOT {
                    if let Some(blocker_id) = blocking.selected_blocker {
                        // Only allow assigning to an attacker the blocker can legally block.
                        if game.state.attacking().contains(&game_id.0)
                            && game.state.blocker_can_block_attacker(blocker_id, game_id.0)
                        {
                            blocking.assignments.push((blocker_id, game_id.0));
                            blocking.selected_blocker = None;
                        }
                    }
                }
            }
        }
        if pass {
            let assignments = std::mem::take(&mut blocking.assignments);
            blocking.selected_blocker = None;
            if !assignments.is_empty() {
                if let Ok(evs) = game.state.perform_action(GameAction::DeclareBlockers(assignments)) {
                    log.apply_events(&evs);
                }
            }
            if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
                log.apply_events(&evs);
            }
        }
        return;
    }

    if game.state.is_game_over() || game.state.active_player_idx != HUMAN {
        return;
    }

    // ── Targeting mode ───────────────────────────────────────────────────────
    if targeting.active {
        // Cancel targeting
        if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
            cancel_targeting(&mut commands, &mut targeting, &valid_targets);
            return;
        }

        if mouse.just_pressed(MouseButton::Left) {
            // Check if hovering a valid battlefield target
            for (game_id, _owner) in &hovered_bf {
                {
                    if let Some(pending_id) = targeting.pending_card_id {
                        let target = Target::Permanent(game_id.0);
                        let cost = targeting.pending_cost.clone();
                        auto_tap_lands(&mut game, &mut log, &cost);
                        if let Ok(evs) = game.state.perform_action(GameAction::CastSpell {
                            card_id: pending_id,
                            target: Some(target),
                        }) {
                            log.apply_events(&evs);
                        }
                        cancel_targeting(&mut commands, &mut targeting, &valid_targets);
                        return;
                    }
                }
            }
            // Check if hovering a player target zone
            for zone in &hovered_target_zone {
                if let Some(pending_id) = targeting.pending_card_id {
                    let cost = targeting.pending_cost.clone();
                    auto_tap_lands(&mut game, &mut log, &cost);
                    if let Ok(evs) = game.state.perform_action(GameAction::CastSpell {
                        card_id: pending_id,
                        target: Some(Target::Player(zone.0)),
                    }) {
                        log.apply_events(&evs);
                    }
                    cancel_targeting(&mut commands, &mut targeting, &valid_targets);
                    return;
                }
            }
        }
        return; // Don't process other input while targeting
    }

    // ── Normal input ─────────────────────────────────────────────────────────
    let in_main = matches!(
        game.state.step,
        TurnStep::PreCombatMain | TurnStep::PostCombatMain
    );
    if in_main && mouse.just_pressed(MouseButton::Left) {
        if let Some(game_id) = hovered_hand.iter().next() {
            play_hand_card_by_id(&mut game, &mut log, &mut targeting, game_id.0);
        }
    }

    // Attack All (A)
    let mut attack = keyboard.just_pressed(KeyCode::KeyA);
    for i in &attack_btn {
        if *i == Interaction::Pressed { attack = true; }
    }
    if attack && game.state.step == TurnStep::DeclareAttackers {
        let ids: Vec<_> = game.state.battlefield.iter()
            .filter(|c| c.owner == HUMAN && c.can_attack())
            .map(|c| c.id).collect();
        if let Ok(evs) = game.state.perform_action(GameAction::DeclareAttackers(ids)) {
            log.apply_events(&evs);
        }
    }

    // Pass Priority (P / Space)
    let mut pass = keyboard.just_pressed(KeyCode::KeyP) || keyboard.just_pressed(KeyCode::Space);
    for i in &pass_btn {
        if *i == Interaction::Pressed { pass = true; }
    }
    if pass {
        if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
            log.apply_events(&evs);
            check_reveal(&evs, &mut reveal);
        }
    }

    // End Turn (E) — pass priority repeatedly until it's the bot's turn
    let mut end_turn = keyboard.just_pressed(KeyCode::KeyE);
    for i in &end_turn_btn {
        if *i == Interaction::Pressed { end_turn = true; }
    }
    if end_turn {
        for _ in 0..MAX_END_TURN_PASSES {
            if game.state.is_game_over() { break; }
            if game.state.active_player_idx != HUMAN { break; }
            if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
                log.apply_events(&evs);
            } else {
                break;
            }
        }
    }
}

fn cancel_targeting(
    commands: &mut Commands,
    targeting: &mut TargetingState,
    valid_targets: &Query<Entity, With<ValidTarget>>,
) {
    targeting.active = false;
    targeting.pending_card_id = None;
    for entity in valid_targets.iter() {
        commands.entity(entity).remove::<ValidTarget>();
    }
}

/// Check if a spell needs a player-chosen target.
fn spell_needs_target(effects: &[SpellEffect]) -> bool {
    effects.iter().any(|e| {
        matches!(
            e,
            SpellEffect::DealDamage { .. }
                | SpellEffect::DestroyCreature { .. }
                | SpellEffect::PumpCreature { .. }
        )
    })
}

fn play_hand_card_by_id(
    game: &mut GameResource,
    log: &mut GameLog,
    targeting: &mut TargetingState,
    card_id: CardId,
) {
    let hand = &game.state.players[HUMAN].hand;
    let Some(card) = hand.iter().find(|c| c.id == card_id) else { return; };
    let card = card.clone();

    if card.definition.is_land() {
        if let Ok(evs) = game.state.perform_action(GameAction::PlayLand(card.id)) {
            log.apply_events(&evs);
        }
        return;
    }

    // Check if spell needs a target
    if spell_needs_target(&card.definition.spell_effects) {
        // Enter targeting mode
        targeting.active = true;
        targeting.pending_card_id = Some(card.id);
        targeting.pending_cost = card.definition.cost.clone();
        return;
    }

    // No target needed — cast immediately
    auto_tap_lands(game, log, &card.definition.cost);
    let target = human_auto_target(&game.state, &card.definition);
    if let Ok(evs) = game.state.perform_action(GameAction::CastSpell { card_id: card.id, target }) {
        log.apply_events(&evs);
    }
}

fn land_produces_color(card: &crabomination::card::CardInstance, color: ManaColor) -> bool {
    card.definition.activated_abilities.iter().any(|a| {
        a.effects.iter().any(|e| {
            if let SpellEffect::AddMana { colors } = e {
                colors.contains(&color)
            } else {
                false
            }
        })
    })
}

fn auto_tap_lands(game: &mut GameResource, log: &mut GameLog, cost: &ManaCost) {
    // Collect required colored pips and generic count from the cost
    let mut needed_colors: Vec<ManaColor> = Vec::new();
    let mut generic: u32 = 0;
    for sym in &cost.symbols {
        match sym {
            ManaSymbol::Colored(c) => needed_colors.push(*c),
            ManaSymbol::Generic(n) => generic += n,
        }
    }

    // Tap a color-matched land for each colored pip
    for color in needed_colors {
        let land_id = game.state.battlefield.iter()
            .find(|c| c.owner == HUMAN && c.definition.is_land() && !c.tapped && land_produces_color(c, color))
            .map(|c| c.id);
        if let Some(id) = land_id {
            if let Ok(evs) = game.state.perform_action(GameAction::ActivateAbility {
                card_id: id, ability_index: 0, target: None,
            }) {
                log.apply_events(&evs);
            }
        }
    }

    // Tap any remaining untapped land for generic pips
    for _ in 0..generic {
        let land_id = game.state.battlefield.iter()
            .find(|c| c.owner == HUMAN && c.definition.is_land() && !c.tapped)
            .map(|c| c.id);
        let Some(id) = land_id else { break };
        if let Ok(evs) = game.state.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None,
        }) {
            log.apply_events(&evs);
        } else {
            break;
        }
    }
}

/// Returns the world transform for a card sitting on the stack at the given index.
fn stack_card_transform(idx: usize, total: usize) -> Transform {
    let spacing = CARD_WIDTH + 0.5;
    let total_width = (total.saturating_sub(1) as f32) * spacing;
    let x = (idx as f32) * spacing - total_width / 2.0;
    Transform::from_translation(Vec3::new(x, 0.8, 0.0))
        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
}

/// When RevealPopupState activates, start a RevealPeekAnimation on the top deck card.
pub fn trigger_reveal_animation(
    mut reveal: ResMut<RevealPopupState>,
    deck_cards: Query<(Entity, &DeckCard, &Transform), (Without<RevealPeekAnimation>, Without<BotDeckPile>)>,
    bot_deck_cards: Query<(Entity, &BotDeckPile, &Transform), (Without<RevealPeekAnimation>, Without<DeckCard>)>,
    mut commands: Commands,
) {
    let Some(player) = reveal.revealed_player.take() else { return };

    // face_up = 180° around the card's own local X axis (width edge), so the flip
    // looks like opening a book rather than spinning around an arbitrary world axis.
    if player == HUMAN {
        // Bot's Goblin Guide revealed the human player's top card.
        if let Some((entity, _, transform)) = deck_cards
            .iter()
            .max_by_key(|(_, dc, _)| dc.index)
        {
            let face_up = transform.rotation * Quat::from_rotation_x(std::f32::consts::PI);
            commands.entity(entity).insert(RevealPeekAnimation {
                progress: 0.0,
                speed: 0.6,
                start_rotation: transform.rotation,
                face_up_rotation: face_up,
                start_y: transform.translation.y,
            });
        }
    } else {
        // Human's Goblin Guide revealed the bot's top card.
        if let Some((entity, _, transform)) = bot_deck_cards
            .iter()
            .max_by_key(|(_, bp, _)| bp.index)
        {
            let face_up = transform.rotation * Quat::from_rotation_x(std::f32::consts::PI);
            commands.entity(entity).insert(RevealPeekAnimation {
                progress: 0.0,
                speed: 0.6,
                start_rotation: transform.rotation,
                face_up_rotation: face_up,
                start_y: transform.translation.y,
            });
        }
    }
}
