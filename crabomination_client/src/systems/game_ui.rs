//! Game HUD overlay and all game-logic Bevy systems:
//! bot AI, human auto-advance, player input, visual sync, and text refresh.

use std::collections::{HashMap, HashSet};
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::effect::{Effect, ManaPayload};
use crabomination::game::{GameAction, StackItem, Target, TurnStep};
use crabomination::mana::{Color as ManaColor, ManaCost, ManaSymbol};
use rand::seq::SliceRandom;

use super::ui::RevealPopupState;
use crate::bot::p1_mulligan_decision;
use crate::bot::{p0_auto_target, p1_declare_blocks, p1_take_action};
use crate::card::{
    Animating, BattlefieldCard, CARD_THICKNESS, CARD_WIDTH, CardHoverLift, CardHovered,
    CardMeshAssets, CardOwner, DECK_CARD_Y_STEP, DECK_POSITION, DeckCard, DrawCardAnimation,
    GameCardId, GraveyardPile, HandCard, HandSlideAnimation, LAND_STACK_OFFSET_X,
    LAND_STACK_OFFSET_Z, P0_GRAVEYARD_POSITION, P1_DECK_POSITION, P1_GRAVEYARD_POSITION,
    P1DeckPile, P1HandCard, PlayCardAnimation, PlayerTargetZone, ReturnToDeckAnimation,
    RevealPeekAnimation, SendToGraveyardAnimation, StackCard, TapAnimation, TapState, ValidTarget,
    bf_card_transform, card_front_material, hand_card_transform, land_group_info,
    p1_hand_card_transform, spawn_single_card,
};
use crate::game::{
    BlockingState, GameLog, GameResource, MulliganState, P1Timer, PLAYER_0, PLAYER_1,
    TargetingState, format_mana_pool,
};
use crate::render_quality::{ChangeQuality, RenderQuality};

/// Max number of PassPriority calls issued by the "End Turn" button/key.
/// Enough to cover all remaining steps in a turn without risking an infinite loop.
const MAX_END_TURN_PASSES: usize = 20;

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TurnInfoText;

#[derive(Component)]
pub struct PlayerStatusText;

#[derive(Component)]
pub struct P1StatusText;

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
pub struct NextTurnButton;

#[derive(Component)]
pub struct PhaseStepLabel(pub TurnStep);

/// Marker for quality preset buttons in the quality selector panel.
#[derive(Component)]
pub struct QualityButton(pub RenderQuality);

/// Marker for the mulligan phase overlay node.
#[derive(Component)]
pub struct MulliganOverlay;

/// Status text within the mulligan overlay.
#[derive(Component)]
pub struct MulliganStatusText;

/// "Keep" button in the mulligan overlay.
#[derive(Component)]
pub struct MulliganKeepButton;

/// "Mulligan" button in the mulligan overlay.
#[derive(Component)]
pub struct MulliganMulliganButton;

/// Latched button presses collected by `poll_action_buttons` each frame,
/// consumed by `handle_game_input`.  Using a resource instead of inline
/// button queries keeps `handle_game_input` under Bevy's system-param limit.
#[derive(Resource, Default)]
pub struct ButtonState {
    pub pass: bool,
    pub attack: bool,
    pub end_turn: bool,
    pub next_turn: bool,
}

/// Reads all action-button `Interaction` components and writes results into
/// `ButtonState`.  Must run before `handle_game_input` in the same frame.
pub fn poll_action_buttons(
    mut state: ResMut<ButtonState>,
    pass_btn: Query<&Interaction, (Changed<Interaction>, With<PassPriorityButton>)>,
    attack_btn: Query<&Interaction, (Changed<Interaction>, With<AttackAllButton>)>,
    end_turn_btn: Query<&Interaction, (Changed<Interaction>, With<EndTurnButton>)>,
    next_turn_btn: Query<&Interaction, (Changed<Interaction>, With<NextTurnButton>)>,
) {
    state.pass = pass_btn.iter().any(|i| *i == Interaction::Pressed);
    state.attack = attack_btn.iter().any(|i| *i == Interaction::Pressed);
    state.end_turn = end_turn_btn.iter().any(|i| *i == Interaction::Pressed);
    state.next_turn = next_turn_btn.iter().any(|i| *i == Interaction::Pressed);
}

/// Custom gizmo config group for blocking indicators (thicker lines).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BlockingGizmos;

/// Custom gizmo config group for attack indicators (sword overlay).
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AttackerGizmos;

// ── HUD setup ─────────────────────────────────────────────────────────────────

pub fn setup_game_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let tf = |size: f32| TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    };

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
            p.spawn((
                Text::new(""),
                tf(16.0),
                TextColor(Color::WHITE),
                TurnInfoText,
            ));
        });

    // Left side: phase chart
    let all_steps = [
        (TurnStep::Untap, "Untap"),
        (TurnStep::Upkeep, "Upkeep"),
        (TurnStep::Draw, "Draw"),
        (TurnStep::PreCombatMain, "Main 1"),
        (TurnStep::BeginCombat, "Begin Combat"),
        (TurnStep::DeclareAttackers, "Attackers"),
        (TurnStep::DeclareBlockers, "Blockers"),
        (TurnStep::CombatDamage, "Damage"),
        (TurnStep::EndCombat, "End Combat"),
        (TurnStep::PostCombatMain, "Main 2"),
        (TurnStep::End, "End"),
        (TurnStep::Cleanup, "Cleanup"),
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
                P1StatusText,
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
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
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
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.28, 0.48)),
                    Button,
                    PassPriorityButton,
                ))
                .with_children(|p| {
                    p.spawn((Text::new("Pass (Space)"), tf(13.0), TextColor(Color::WHITE)));
                });

                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.38, 0.18)),
                    Button,
                    EndTurnButton,
                ))
                .with_children(|p| {
                    p.spawn((Text::new("End Turn (E)"), tf(13.0), TextColor(Color::WHITE)));
                });

                p.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.18, 0.48)),
                    Button,
                    NextTurnButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Next Turn (N)"),
                        tf(13.0),
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
}

// ── HUD text update ───────────────────────────────────────────────────────────

pub fn update_turn_text(game: Res<GameResource>, mut q: Query<&mut Text, With<TurnInfoText>>) {
    let Ok(mut t) = q.single_mut() else { return };
    let s = &game.state;
    t.0 = if let Some(winner) = s.game_over {
        match winner {
            Some(p) => format!(
                "GAME OVER — {} wins!",
                if p == PLAYER_0 {
                    "Player 0"
                } else {
                    "Player 1"
                }
            ),
            None => "GAME OVER — Draw!".into(),
        }
    } else {
        format!(
            "Turn {} | {:?} | {}'s turn",
            s.turn_number,
            s.step,
            if s.active_player_idx == PLAYER_0 {
                "Player 0"
            } else {
                "Player 1"
            }
        )
    };
}

pub fn update_player_text(
    game: Res<GameResource>,
    mut q: Query<&mut Text, With<PlayerStatusText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let s = &game.state;
    let p = &s.players[PLAYER_0];
    t.0 = format!(
        "Player 0 | Life: {} | Hand: {} | Deck: {} | GY: {} | Mana: {}",
        p.life,
        p.hand.len(),
        p.library.len(),
        p.graveyard.len(),
        format_mana_pool(s, PLAYER_0)
    );
}

pub fn update_p1_text(game: Res<GameResource>, mut q: Query<&mut Text, With<P1StatusText>>) {
    let Ok(mut t) = q.single_mut() else { return };
    let s = &game.state;
    let p = &s.players[PLAYER_1];
    t.0 = format!(
        "Player 1 | Life: {} | Hand: {} | Deck: {} | GY: {}",
        p.life,
        p.hand.len(),
        p.library.len(),
        p.graveyard.len()
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
    if game.state.step != TurnStep::DeclareBlockers || game.state.active_player_idx != PLAYER_1 {
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
            let color = if is_blocked {
                Color::srgb(0.0, 0.9, 0.3)
            } else {
                Color::srgb(1.0, 0.2, 0.2)
            };
            draw_diamond(&mut gizmos, pos, 1.1, color);
        }
    }

    // Highlight selected blocker in yellow + guide arrows to each unassigned attacker
    if let Some(blocker_id) = blocking.selected_blocker
        && let Some(&pos) = positions.get(&blocker_id)
    {
        draw_diamond(&mut gizmos, pos, 1.1, Color::srgb(1.0, 0.88, 0.0));
        for &attacker_id in attacking {
            let already_assigned = blocking.assignments.iter().any(|(_, a)| *a == attacker_id);
            if !already_assigned && let Some(&att_pos) = positions.get(&attacker_id) {
                gizmos
                    .arrow(pos, att_pos, Color::srgba(1.0, 0.88, 0.0, 0.7))
                    .with_tip_length(0.6);
            }
        }
    }

    // Draw confirmed assignment arrows in green
    for &(blocker_id, attacker_id) in &blocking.assignments {
        if let Some(&b_pos) = positions.get(&blocker_id)
            && let Some(&a_pos) = positions.get(&attacker_id)
        {
            gizmos
                .arrow(b_pos, a_pos, Color::srgb(0.0, 0.9, 0.3))
                .with_tip_length(0.6);
            draw_diamond(&mut gizmos, b_pos, 1.1, Color::srgb(0.0, 0.9, 0.3));
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
    let s1_tip = Vec3::new(pos.x + r, y, pos.z + r);
    let s1_guard = s1_handle.lerp(s1_tip, gf);
    // Guard perpendicular to blade 1 → along (+x,-z) direction
    let gd1 = Vec3::new(1.0, 0.0, -1.0).normalize() * g;
    gizmos.line(s1_handle, s1_tip, color);
    gizmos.line(s1_guard - gd1, s1_guard + gd1, color);

    // Sword 2: handle at (+x,-z), tip at (-x,+z)
    let s2_handle = Vec3::new(pos.x + r, y, pos.z - r);
    let s2_tip = Vec3::new(pos.x - r, y, pos.z + r);
    let s2_guard = s2_handle.lerp(s2_tip, gf);
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
    mulligan: Res<MulliganState>,
    mut q: Query<&mut Text, With<HintText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    if mulligan.active {
        t.0 = if !mulligan.initial_deal_done {
            "Dealing opening hands...".into()
        } else if mulligan.p0_cards_to_bottom > 0 {
            format!(
                "Click {} card{} in your hand to put to the bottom of your library.",
                mulligan.p0_cards_to_bottom,
                if mulligan.p0_cards_to_bottom == 1 {
                    ""
                } else {
                    "s"
                }
            )
        } else if !mulligan.p0_kept {
            "Keep your hand (K) or take a mulligan (M).".into()
        } else {
            "Waiting for opponent...".into()
        };
        return;
    }
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
    // Player 0 defending against player 1's attack
    if game.state.step == TurnStep::DeclareBlockers && game.state.active_player_idx == PLAYER_1 {
        t.0 = if blocking.selected_blocker.is_some() {
            "Click an attacking creature to assign the block. Right-click to cancel.".into()
        } else {
            "Click one of your creatures to block with it. P = Skip blocking.".into()
        };
        return;
    }
    t.0 = match (game.state.active_player_idx, game.state.step) {
        (PLAYER_0, TurnStep::PreCombatMain) | (PLAYER_0, TurnStep::PostCombatMain) => {
            "Click a hand card to play it. P = Pass Priority.".into()
        }
        (PLAYER_0, TurnStep::DeclareAttackers) => {
            "A = Attack with all eligible creatures. P = Pass (no attack).".into()
        }
        (PLAYER_0, TurnStep::DeclareBlockers) => {
            "Player 1 is assigning blocks. P = Proceed to combat.".into()
        }
        (PLAYER_0, _) => String::new(),
        (PLAYER_1, _) => "Player 1 is thinking...".into(),
        _ => String::new(),
    };
}

// ── Visual sync: reconcile 3D card entities with game state ──────────────────

/// Helper: count cards per row for a given owner.
fn bf_row_counts(state: &crabomination::game::GameState, owner: usize) -> (usize, usize) {
    let lands = state
        .battlefield
        .iter()
        .filter(|c| c.owner == owner && c.definition.is_land())
        .count();
    let creatures = state
        .battlefield
        .iter()
        .filter(|c| c.owner == owner && !c.definition.is_land())
        .count();
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn sync_game_visuals(
    mut commands: Commands,
    game: Res<GameResource>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_assets: Option<Res<CardMeshAssets>>,
    removed_animating: RemovedComponents<Animating>,
    hand_cards: Query<
        (
            Entity,
            &GameCardId,
            &Transform,
            Option<&StackCard>,
            &CardHoverLift,
        ),
        (With<HandCard>, Without<Animating>),
    >,
    deck_cards: Query<
        (Entity, &GameCardId, &DeckCard, &Transform),
        (Without<HandCard>, Without<Animating>),
    >,
    bf_cards: Query<
        (
            Entity,
            &GameCardId,
            &CardOwner,
            &BattlefieldCard,
            &Transform,
            Option<&TapState>,
        ),
        (Without<HandCard>, Without<Animating>),
    >,
    mut bot_deck_q: Query<
        (Entity, &P1DeckPile, &mut Transform, &mut CardHoverLift),
        Without<GameCardId>,
    >,
    mut graveyard_q: Query<
        (
            &GraveyardPile,
            &mut Transform,
            &mut Visibility,
            &mut CardHoverLift,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<P1DeckPile>, Without<GameCardId>),
    >,
    bot_hand_q: Query<
        (Entity, &P1HandCard, &Transform, Has<Animating>),
        (Without<P1DeckPile>, Without<GraveyardPile>),
    >,
    gy_anims: Query<&SendToGraveyardAnimation>,
    all_bf_entities: Query<&GameCardId, With<BattlefieldCard>>,
) {
    let state = &game.state;

    // ── Always update: deck/graveyard heights and bot hand count ─────────────
    // These must run every frame so they stay correct after spawning.

    let p1_deck_size = state.players[PLAYER_1].library.len();
    for (entity, pile, mut transform, mut lift) in &mut bot_deck_q {
        if pile.index >= p1_deck_size {
            commands.entity(entity).despawn();
        } else {
            let y = pile.index as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(P1_DECK_POSITION.x, y, P1_DECK_POSITION.z);
            transform.translation = pos;
            lift.base_translation = pos;
        }
    }

    // Count in-flight graveyard animations per owner to delay pile visibility.
    let mut gy_in_flight: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
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
            let base_pos = if gy.owner == PLAYER_0 {
                P0_GRAVEYARD_POSITION
            } else {
                P1_GRAVEYARD_POSITION
            };
            let y = arrived as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(base_pos.x, y, base_pos.z);
            transform.translation = pos;
            lift.base_translation = pos;
        }
    }

    if !game.is_changed() && removed_animating.is_empty() {
        return;
    }
    let Some(card_assets) = card_assets else {
        return;
    };

    // ── Player 1 hand + battlefield sync ──────────────────────────────────────
    // Collect all P1HandCards first (used for both hand reconciliation and BF
    // spawn start positions). Include animating cards in the count so we never
    // double-spawn.
    let p1_hand_size = state.players[PLAYER_1].hand.len();
    let all_p1_hand: Vec<(Entity, usize, Vec3, Quat, bool)> = bot_hand_q
        .iter()
        .map(|(e, bh, t, is_animating)| (e, bh.slot, t.translation, t.rotation, is_animating))
        .collect();

    // Pool of non-animating hand cards available to use as animation origins for
    // newly played P1 BF cards (highest-slot first so we pull from the "end" of the fan).
    // Sorted ascending by slot so pop() always yields the highest-slotted card,
    // keeping remaining slots contiguous and < p1_hand_size after promotion.
    let mut hand_pool: Vec<(Entity, Vec3, Quat)> = {
        let mut pool: Vec<_> = all_p1_hand
            .iter()
            .filter(|(_, _, _, _, is_anim)| !is_anim)
            .map(|(e, slot, pos, rot, _)| (*e, *slot, *pos, *rot))
            .collect();
        pool.sort_by_key(|(_, slot, _, _)| *slot);
        pool.into_iter()
            .map(|(e, _, pos, rot)| (e, pos, rot))
            .collect()
    };
    // Entities consumed from the hand pool for BF animations (excluded from hand count).
    let mut promoted: HashSet<Entity> = HashSet::new();

    // ── Update graveyard pile face-up material ────────────────────────────────
    for (gy, _transform, _vis, _lift, mut mat) in &mut graveyard_q {
        let graveyard = &state.players[gy.owner].graveyard;
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = graveyard.len().saturating_sub(in_flight);
        if arrived > 0
            && let Some(top) = graveyard.get(arrived - 1)
        {
            *mat = MeshMaterial3d(card_front_material(
                top.definition.name,
                &mut materials,
                &asset_server,
            ));
        }
    }

    // Build sets of CardIds in each game-engine zone
    let stack_ids: HashSet<CardId> = state
        .stack
        .iter()
        .filter_map(|item| {
            if let StackItem::Spell { card, .. } = item {
                Some(card.id)
            } else {
                None
            }
        })
        .collect();
    let hand_ids: HashSet<CardId> = state.players[PLAYER_0].hand.iter().map(|c| c.id).collect();
    let p0_bf_ids: HashSet<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == PLAYER_0)
        .map(|c| c.id)
        .collect();
    let p1_bf_ids: HashSet<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.owner == PLAYER_1)
        .map(|c| c.id)
        .collect();
    let all_bf_ids: HashSet<CardId> = p0_bf_ids.iter().chain(p1_bf_ids.iter()).copied().collect();

    let hand_total = hand_ids.len();
    let (_p0_lands, p0_creatures) = bf_row_counts(state, PLAYER_0);
    let (_p1_lands, p1_creatures) = bf_row_counts(state, PLAYER_1);

    // Include animating BF entities so in-flight cards aren't re-spawned.
    let visual_bf_ids: HashSet<CardId> = all_bf_entities.iter().map(|gid| gid.0).collect();

    // ── Deck → Hand transitions ──────────────────────────────────────────────
    for (entity, game_id, _deck_card, transform) in &deck_cards {
        if hand_ids.contains(&game_id.0) {
            let slot = state.players[PLAYER_0]
                .hand
                .iter()
                .position(|c| c.id == game_id.0)
                .unwrap_or(hand_total.saturating_sub(1));
            let target = hand_card_transform(slot, hand_total);
            commands
                .entity(entity)
                .remove::<DeckCard>()
                .insert(HandCard { slot })
                .insert(Animating)
                .insert(DrawCardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                });
        }
    }

    // ── Hand → Battlefield / Stack transitions ───────────────────────────────
    for (entity, game_id, transform, stack_card, _lift) in &hand_cards {
        if p0_bf_ids.contains(&game_id.0) {
            let card_inst = state.battlefield.iter().find(|c| c.id == game_id.0);
            let is_land = card_inst.is_some_and(|c| c.definition.is_land());
            let target = if is_land {
                land_card_transform(state, PLAYER_0, game_id.0, false)
                    .unwrap_or_else(|| bf_card_transform(0, 1, true, false, false))
            } else {
                let row_total = p0_creatures;
                let slot = bf_row_slot(state, PLAYER_0, game_id.0, false).unwrap_or(0);
                bf_card_transform(slot, row_total, false, false, false)
            };
            commands
                .entity(entity)
                .remove::<HandCard>()
                .remove::<StackCard>()
                .remove::<CardHovered>()
                .insert(BattlefieldCard { is_land })
                .insert(CardOwner(PLAYER_0))
                .insert(TapState { tapped: false })
                .insert(Animating)
                .insert(PlayCardAnimation {
                    progress: 0.0,
                    speed: 2.0,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                });
        } else if stack_ids.contains(&game_id.0) {
            if stack_card.is_none() {
                // First time we see this card on the stack — fly it to the center
                let idx = state.stack.iter().position(|item| {
                    matches!(item, StackItem::Spell { card, .. } if card.id == game_id.0)
                }).unwrap_or(0);
                let total = stack_ids.len();
                let target = stack_card_transform(idx, total);
                commands
                    .entity(entity)
                    .remove::<CardHovered>()
                    .insert(StackCard)
                    .insert(Animating)
                    .insert(PlayCardAnimation {
                        progress: 0.0,
                        speed: 2.0,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                    });
            }
        } else if !hand_ids.contains(&game_id.0) {
            // Determine destination: library (mulligan return) or graveyard.
            let returned_to_library = state.players[PLAYER_0]
                .library
                .iter()
                .any(|c| c.id == game_id.0);
            if returned_to_library {
                let deck_rot = Quat::from_rotation_x(FRAC_PI_2) * Quat::from_rotation_z(PI);
                commands
                    .entity(entity)
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(ReturnToDeckAnimation {
                        progress: 0.0,
                        speed: 2.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: DECK_POSITION,
                        target_rotation: deck_rot,
                    });
            } else {
                commands
                    .entity(entity)
                    .remove::<HandCard>()
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(SendToGraveyardAnimation {
                        progress: 0.0,
                        speed: 1.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: P0_GRAVEYARD_POSITION,
                        target_rotation: Quat::from_rotation_x(-FRAC_PI_2),
                        owner: PLAYER_0,
                    });
            }
        }
    }

    // ── Battlefield → Graveyard transitions ──────────────────────────────────
    for (entity, game_id, owner, _bf, transform, _) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) {
            let (gy_pos, gy_rot) = if owner.0 == PLAYER_0 {
                (P0_GRAVEYARD_POSITION, Quat::from_rotation_x(-FRAC_PI_2))
            } else {
                (
                    P1_GRAVEYARD_POSITION,
                    Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI),
                )
            };
            commands
                .entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardHovered>()
                .insert(Animating)
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

    // ── Spawn new player 1 battlefield cards ─────────────────────────────────
    // Where possible, pull a real P1HandCard entity's position as the animation
    // start so the card visually flies from the bot's hand to the board.
    for card in state.battlefield.iter().filter(|c| c.owner == PLAYER_1) {
        if visual_bf_ids.contains(&card.id) {
            continue;
        }
        let is_land = card.definition.is_land();
        let target = if is_land {
            land_card_transform(state, PLAYER_1, card.id, true)
                .unwrap_or_else(|| bf_card_transform(0, 1, true, true, false))
        } else {
            let row_total = p1_creatures;
            let slot = bf_row_slot(state, PLAYER_1, card.id, false).unwrap_or(0);
            bf_card_transform(slot, row_total, false, true, card.tapped)
        };

        // Reuse an actual hand card position; fall back to computed center if none available.
        let (start_pos, start_rot) = if let Some((hand_entity, pos, rot)) = hand_pool.pop() {
            commands.entity(hand_entity).despawn();
            promoted.insert(hand_entity);
            (pos, rot)
        } else {
            let c = p1_hand_card_transform(0, 1);
            (c.translation, c.rotation)
        };

        let front_mat = card_front_material(card.definition.name, &mut materials, &asset_server);
        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(start_pos).with_rotation(start_rot),
            GameCardId(card.id),
            card.definition.name,
            target.translation,
        );
        commands.entity(entity).insert((
            BattlefieldCard { is_land },
            CardOwner(PLAYER_1),
            TapState {
                tapped: card.tapped,
            },
            Animating,
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: start_pos,
                start_rotation: start_rot,
                target_translation: target.translation,
                target_rotation: target.rotation,
            },
        ));
    }

    // ── Player 1 hand reconciliation ──────────────────────────────────────────
    {
        // Cards promoted to BF are already despawned above; exclude them from count.
        let visual_p1_hand_size = all_p1_hand.len() - promoted.len();
        let mut removed: HashSet<Entity> = promoted.clone();

        if visual_p1_hand_size > p1_hand_size {
            let mut sorted: Vec<_> = all_p1_hand
                .iter()
                .filter(|(e, _, _, _, _)| !promoted.contains(e))
                .collect();
            sorted.sort_by_key(|(_, slot, _, _, _)| std::cmp::Reverse(*slot));
            for (entity, _, _, _, _) in sorted.iter().take(visual_p1_hand_size - p1_hand_size) {
                commands.entity(*entity).despawn();
                removed.insert(*entity);
            }
        } else if visual_p1_hand_size < p1_hand_size {
            let deck_y = p1_hand_size as f32 * DECK_CARD_Y_STEP + 0.5;
            let deck_pos = Vec3::new(P1_DECK_POSITION.x, deck_y, P1_DECK_POSITION.z);
            for slot in visual_p1_hand_size..p1_hand_size {
                let target = p1_hand_card_transform(slot, p1_hand_size);
                let back_mat = card_assets.back_material.clone();
                let card_mesh = card_assets.card_mesh.clone();
                let start_transform = Transform::from_translation(deck_pos);
                commands
                    .spawn((
                        start_transform,
                        Visibility::default(),
                        P1HandCard { slot },
                        CardHoverLift {
                            current_lift: 0.0,
                            target_lift: 0.0,
                            base_translation: target.translation,
                        },
                        Animating,
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

        // Despawn any cards whose slot is now out of range (can happen when an
        // animating card had a high slot that wasn't cleaned up by the count pass).
        for (entity, slot, _, _, _) in &all_p1_hand {
            if !removed.contains(entity) && *slot >= p1_hand_size {
                commands.entity(*entity).despawn();
                removed.insert(*entity);
            }
        }

        // Slide remaining non-animating cards to their new positions.
        for (entity, slot, pos, _rot, is_animating) in &all_p1_hand {
            if removed.contains(entity) || *is_animating {
                continue;
            }
            let target = p1_hand_card_transform(*slot, p1_hand_size);
            if pos.distance(target.translation) > 0.001 {
                commands
                    .entity(*entity)
                    .insert(Animating)
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

    // ── Also spawn new player 0 battlefield cards that don't have entities yet ─
    // (e.g. cards that entered via game actions without going through hand→bf)
    for card in state.battlefield.iter().filter(|c| c.owner == PLAYER_0) {
        if visual_bf_ids.contains(&card.id)
            || hand_cards.iter().any(|(_, gid, _, _, _)| gid.0 == card.id)
        {
            continue;
        }
        let is_land = card.definition.is_land();
        let target = if is_land {
            land_card_transform(state, PLAYER_0, card.id, false)
                .unwrap_or_else(|| bf_card_transform(0, 1, true, false, false))
        } else {
            let row_total = p0_creatures;
            let slot = bf_row_slot(state, PLAYER_0, card.id, false).unwrap_or(0);
            bf_card_transform(slot, row_total, false, false, card.tapped)
        };
        let front_mat = card_front_material(card.definition.name, &mut materials, &asset_server);

        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            target,
            GameCardId(card.id),
            card.definition.name,
            target.translation,
        );
        commands.entity(entity).insert((
            BattlefieldCard { is_land },
            CardOwner(PLAYER_0),
            TapState {
                tapped: card.tapped,
            },
        ));
    }

    // ── Rebalance hand slots ─────────────────────────────────────────────────
    for (entity, game_id, _transform, stack_card, lift) in &hand_cards {
        if stack_card.is_some() {
            continue;
        } // don't slide stack cards back to hand
        if !hand_ids.contains(&game_id.0) {
            continue;
        }
        let Some(new_slot) = state.players[PLAYER_0]
            .hand
            .iter()
            .position(|c| c.id == game_id.0)
        else {
            continue;
        };
        let target = hand_card_transform(new_slot, hand_total);
        // Compare base position (no hover-lift offset) to avoid spurious slides on hovered cards.
        let dist = (lift.base_translation - target.translation).length();
        if dist > 0.1 {
            commands
                .entity(entity)
                .insert(Animating)
                .insert(HandSlideAnimation {
                    progress: 0.0,
                    speed: 3.0,
                    start_translation: lift.base_translation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                });
        }
        commands.entity(entity).insert(HandCard { slot: new_slot });
    }

    // ── Rebalance battlefield positions + sync tapped state ──────────────────
    for (entity, game_id, owner, bf, _transform, tap_state) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) {
            continue;
        }
        let is_bot = owner.0 == PLAYER_1;
        let is_land = bf.is_land;

        // Check tapped state from game engine
        let game_tapped = state
            .battlefield
            .iter()
            .find(|c| c.id == game_id.0)
            .is_some_and(|c| c.tapped);
        let visual_tapped = tap_state.is_some_and(|ts| ts.tapped);

        // Use stacked layout for lands, standard slot layout for creatures
        let target = if is_land {
            match land_card_transform(state, owner.0, game_id.0, is_bot) {
                Some(t) => {
                    // Apply tap rotation on top of stagger translation
                    let base_rot = t.rotation;
                    let tapped_rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * base_rot;
                    let rot = if game_tapped { tapped_rot } else { base_rot };
                    Transform {
                        translation: t.translation,
                        rotation: rot,
                        scale: t.scale,
                    }
                }
                None => continue,
            }
        } else {
            let row_total = if is_bot { p1_creatures } else { p0_creatures };
            let Some(slot) = bf_row_slot(state, owner.0, game_id.0, false) else {
                continue;
            };
            bf_card_transform(slot, row_total, false, is_bot, game_tapped)
        };

        if game_tapped != visual_tapped {
            let (untapped_rot, tapped_rot) = if is_land {
                let _base = target.rotation;
                // For lands, tapped rot was already computed; get both variants
                let land_base = land_card_transform(state, owner.0, game_id.0, is_bot)
                    .map(|t| t.rotation)
                    .unwrap_or(target.rotation);
                let land_tapped = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * land_base;
                (land_base, land_tapped)
            } else {
                let row_total = if is_bot { p1_creatures } else { p0_creatures };
                let slot = bf_row_slot(state, owner.0, game_id.0, false).unwrap_or(0);
                (
                    bf_card_transform(slot, row_total, false, is_bot, false).rotation,
                    bf_card_transform(slot, row_total, false, is_bot, true).rotation,
                )
            };
            let (start, end) = if game_tapped {
                (untapped_rot, tapped_rot)
            } else {
                (tapped_rot, untapped_rot)
            };
            commands
                .entity(entity)
                .insert(Animating)
                .insert(TapAnimation {
                    progress: 0.0,
                    speed: 4.0,
                    start_rotation: start,
                    target_rotation: end,
                })
                .insert(TapState {
                    tapped: game_tapped,
                });
        }

        commands.entity(entity).insert(CardHoverLift {
            current_lift: 0.0,
            target_lift: 0.0,
            base_translation: target.translation,
        });
    }

    // ── Update player 0 deck card visibility (remove drawn cards) ─────────
    let p0_lib_ids: HashSet<CardId> = state.players[PLAYER_0]
        .library
        .iter()
        .map(|c| c.id)
        .collect();
    for (entity, game_id, _deck_card, _transform) in &deck_cards {
        if !p0_lib_ids.contains(&game_id.0) && !hand_ids.contains(&game_id.0) {
            commands.entity(entity).despawn();
        }
    }
}

// ── Player 1 system ───────────────────────────────────────────────────────────

/// Scan `events` for `TopCardRevealed` and arm the reveal popup if found.
fn check_reveal(events: &[crabomination::game::GameEvent], reveal: &mut RevealPopupState) {
    for ev in events {
        if let crabomination::game::GameEvent::TopCardRevealed {
            player, card_name, ..
        } = ev
        {
            reveal.card_path = Some(crate::scryfall::card_asset_path(card_name));
            reveal.revealed_player = Some(*player);
        }
    }
}

pub fn p1_system(
    time: Res<Time>,
    mut timer: ResMut<P1Timer>,
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut reveal: ResMut<RevealPopupState>,
    mut last_key: Local<Option<(usize, TurnStep)>>,
    mulligan: Res<MulliganState>,
) {
    if mulligan.active {
        return;
    }
    if game.state.is_game_over() {
        return;
    }

    let current_key = (game.state.active_player_idx, game.state.step);
    let step_just_changed = *last_key != Some(current_key);
    *last_key = Some(current_key);

    let is_p1_blocking =
        game.state.step == TurnStep::DeclareBlockers && game.state.active_player_idx == PLAYER_0;
    if is_p1_blocking && step_just_changed {
        let evs = p1_declare_blocks(&mut game.state, &mut rand::rng());
        log.apply_events(&evs);
        return;
    }

    if game.state.active_player_idx != PLAYER_1 {
        return;
    }

    // Wait for player 0 to finish declaring blocks before advancing
    if game.state.step == TurnStep::DeclareBlockers {
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let evs = p1_take_action(&mut game.state, &mut rand::rng());
    log.apply_events(&evs);
    check_reveal(&evs, &mut reveal);
}

// ── Auto-advance non-interactive steps for player 0 ──────────────────────────

pub fn auto_advance_p0(
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut reveal: ResMut<RevealPopupState>,
    mulligan: Res<MulliganState>,
) {
    if mulligan.active {
        return;
    }
    if game.state.is_game_over() || game.state.player_with_priority() != PLAYER_0 {
        return;
    }
    let should_advance = matches!(
        game.state.step,
        TurnStep::Untap
            | TurnStep::Upkeep
            | TurnStep::Draw
            | TurnStep::BeginCombat
            | TurnStep::CombatDamage
            | TurnStep::EndCombat
            | TurnStep::End
            | TurnStep::Cleanup
    ) || game.state.active_player_idx == PLAYER_1;
    if should_advance && let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
        log.apply_events(&evs);
        check_reveal(&evs, &mut reveal);
    }
}

// ── Player 0 input ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_game_input(
    mut commands: Commands,
    mut game: ResMut<GameResource>,
    mut log: ResMut<GameLog>,
    mut targeting: ResMut<TargetingState>,
    mut blocking: ResMut<BlockingState>,
    mut reveal: ResMut<RevealPopupState>,
    mut mulligan: ResMut<MulliganState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered_hand: Query<&GameCardId, (With<CardHovered>, With<HandCard>)>,
    hovered_bf: Query<(&GameCardId, &CardOwner), (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_target_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    valid_targets: Query<Entity, With<ValidTarget>>,
    btns: Res<ButtonState>,
) {
    // ── London Mulligan bottoming phase ──────────────────────────────────────
    if mulligan.active && mulligan.p0_cards_to_bottom > 0 {
        if mouse.just_pressed(MouseButton::Left)
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let player = &mut game.state.players[PLAYER_0];
            if let Some(pos) = player.hand.iter().position(|c| c.id == game_id.0) {
                let card = player.hand.remove(pos);
                player.library.push(card);
                mulligan.p0_cards_to_bottom -= 1;
                if mulligan.p0_cards_to_bottom == 0 {
                    mulligan.p0_kept = true;
                    log.push(format!(
                        "Player 0 keeps ({} cards)",
                        game.state.players[PLAYER_0].hand.len()
                    ));
                }
            }
        }
        return;
    }

    if mulligan.active {
        return;
    }

    // ── Blocking (player 0 defends on player 1's turn) ───────────────────────
    if game.state.step == TurnStep::DeclareBlockers && game.state.active_player_idx == PLAYER_1 {
        let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
        if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
            blocking.selected_blocker = None;
            return;
        }
        if mouse.just_pressed(MouseButton::Left)
            && let Some((game_id, owner)) = hovered_bf.iter().next()
        {
            if owner.0 == PLAYER_0 {
                // Only allow selecting creatures that can legally block something
                // and haven't already been assigned to an attacker.
                let already_assigned = blocking.assignments.iter().any(|(b, _)| *b == game_id.0);
                if game.state.can_block_any_attacker(game_id.0) && !already_assigned {
                    blocking.selected_blocker = Some(game_id.0);
                }
            } else if owner.0 == PLAYER_1
                && let Some(blocker_id) = blocking.selected_blocker
            {
                // Only allow assigning to an attacker the blocker can legally block.
                if game.state.attacking().contains(&game_id.0)
                    && game.state.blocker_can_block_attacker(blocker_id, game_id.0)
                {
                    blocking.assignments.push((blocker_id, game_id.0));
                    blocking.selected_blocker = None;
                }
            }
        }
        if pass {
            let assignments = std::mem::take(&mut blocking.assignments);
            blocking.selected_blocker = None;
            if !assignments.is_empty()
                && let Ok(evs) = game
                    .state
                    .perform_action(GameAction::DeclareBlockers(assignments))
            {
                log.apply_events(&evs);
            }
            if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
                log.apply_events(&evs);
            }
        }
        return;
    }

    if game.state.is_game_over() || game.state.active_player_idx != PLAYER_0 {
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
                            mode: None,
                            x_value: None,
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
                        mode: None,
                        x_value: None,
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
    if in_main
        && mouse.just_pressed(MouseButton::Left)
        && let Some(game_id) = hovered_hand.iter().next()
    {
        play_hand_card_by_id(&mut game, &mut log, &mut targeting, game_id.0);
    }

    // Attack All (A)
    let attack = keyboard.just_pressed(KeyCode::KeyA) || btns.attack;
    if attack && game.state.step == TurnStep::DeclareAttackers {
        let ids: Vec<_> = game
            .state
            .battlefield
            .iter()
            .filter(|c| c.owner == PLAYER_0 && c.can_attack())
            .map(|c| c.id)
            .collect();
        if let Ok(evs) = game.state.perform_action(GameAction::DeclareAttackers(ids)) {
            log.apply_events(&evs);
        }
    }

    // Pass Priority (Space)
    let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
    if pass && let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
        log.apply_events(&evs);
        check_reveal(&evs, &mut reveal);
    }

    // End Turn (E) — pass priority repeatedly until it's player 1's turn
    let end_turn = keyboard.just_pressed(KeyCode::KeyE) || btns.end_turn;
    if end_turn {
        for _ in 0..MAX_END_TURN_PASSES {
            if game.state.is_game_over() {
                break;
            }
            if game.state.active_player_idx != PLAYER_0 {
                break;
            }
            if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
                log.apply_events(&evs);
            } else {
                break;
            }
        }
    }

    // Next Turn (N) — run through P1's entire turn and stop at P0's next main phase
    let next_turn = keyboard.just_pressed(KeyCode::KeyN) || btns.next_turn;
    if next_turn {
        advance_to_p0_main(&mut game, &mut log, &mut reveal);
    }
}

/// Fast-forward through P1's automated turn, stopping at P0's next PreCombatMain
/// or when P0 must manually declare blockers.
fn advance_to_p0_main(game: &mut GameResource, log: &mut GameLog, reveal: &mut RevealPopupState) {
    const MAX: usize = 400;
    let mut rng = rand::rng();
    for _ in 0..MAX {
        if game.state.is_game_over() {
            break;
        }
        // Stop when P0 is back in their main phase with priority.
        if game.state.active_player_idx == PLAYER_0
            && game.state.step == TurnStep::PreCombatMain
            && game.state.player_with_priority() == PLAYER_0
        {
            break;
        }
        // Stop so P0 can manually declare blockers.
        if game.state.active_player_idx == PLAYER_1 && game.state.step == TurnStep::DeclareBlockers
        {
            break;
        }

        let priority = game.state.player_with_priority();
        if game.state.active_player_idx == PLAYER_1 && priority == PLAYER_1 {
            // Run P1's action inline (bypasses the 0.4s timer).
            let evs = p1_take_action(&mut game.state, &mut rng);
            log.apply_events(&evs);
            check_reveal(&evs, reveal);
            if evs.is_empty() {
                // P1 returned nothing (e.g. waiting on blockers) — break to avoid spin.
                break;
            }
        } else if game.state.active_player_idx == PLAYER_0
            && game.state.step == TurnStep::DeclareBlockers
        {
            // On P0's turn DeclareBlockers: auto-run P1's block declarations then pass.
            let evs = p1_declare_blocks(&mut game.state, &mut rng);
            log.apply_events(&evs);
            if let Ok(evs) = game.state.perform_action(GameAction::PassPriority) {
                log.apply_events(&evs);
            }
        } else {
            match game.state.perform_action(GameAction::PassPriority) {
                Ok(evs) => {
                    log.apply_events(&evs);
                    check_reveal(&evs, reveal);
                }
                Err(_) => break,
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
fn spell_needs_target(effect: &Effect) -> bool {
    use crabomination::effect::Selector;
    fn has_target_selector(e: &Effect) -> bool {
        match e {
            Effect::DealDamage { to, .. } => matches!(to, Selector::Target(_)),
            Effect::Destroy { what }
            | Effect::Exile { what }
            | Effect::CounterSpell { what }
            | Effect::Move { what, .. } => matches!(what, Selector::Target(_)),
            Effect::PumpPT { what, .. } => matches!(what, Selector::Target(_)),
            Effect::Seq(steps) => steps.iter().any(has_target_selector),
            Effect::If { then, else_, .. } => has_target_selector(then) || has_target_selector(else_),
            _ => false,
        }
    }
    has_target_selector(effect)
}

fn play_hand_card_by_id(
    game: &mut GameResource,
    log: &mut GameLog,
    targeting: &mut TargetingState,
    card_id: CardId,
) {
    let hand = &game.state.players[PLAYER_0].hand;
    let Some(card) = hand.iter().find(|c| c.id == card_id) else {
        return;
    };
    let card = card.clone();

    if card.definition.is_land() {
        if let Ok(evs) = game.state.perform_action(GameAction::PlayLand(card.id)) {
            log.apply_events(&evs);
        }
        return;
    }

    // Check if spell needs a target
    if spell_needs_target(&card.definition.effect) {
        // Enter targeting mode
        targeting.active = true;
        targeting.pending_card_id = Some(card.id);
        targeting.pending_cost = card.definition.cost.clone();
        return;
    }

    // No target needed — cast immediately
    auto_tap_lands(game, log, &card.definition.cost);
    let target = p0_auto_target(&game.state, &card.definition);
    if let Ok(evs) = game.state.perform_action(GameAction::CastSpell {
        card_id: card.id,
        target,
        mode: None,
        x_value: None,
    }) {
        log.apply_events(&evs);
    }
}

fn effect_produces_color(effect: &Effect, color: ManaColor) -> bool {
    match effect {
        Effect::AddMana { pool, .. } => match pool {
            ManaPayload::Colors(cs) => cs.contains(&color),
            ManaPayload::AnyOneColor(_) | ManaPayload::AnyColors(_) => true,
            ManaPayload::Colorless(_) => false,
        },
        Effect::Seq(steps) => steps.iter().any(|s| effect_produces_color(s, color)),
        _ => false,
    }
}

fn effect_is_pure_mana(effect: &Effect) -> bool {
    match effect {
        Effect::AddMana { .. } => true,
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(effect_is_pure_mana),
        _ => false,
    }
}

fn source_produces_color(card: &crabomination::card::CardInstance, color: ManaColor) -> bool {
    card.definition
        .activated_abilities
        .iter()
        .any(|a| effect_produces_color(&a.effect, color))
}

fn is_mana_source(card: &crabomination::card::CardInstance) -> bool {
    card.definition
        .activated_abilities
        .iter()
        .any(|a| effect_is_pure_mana(&a.effect))
}

fn auto_tap_lands(game: &mut GameResource, log: &mut GameLog, cost: &ManaCost) {
    // Collect required colored pips and generic count from the cost
    let mut needed_colors: Vec<ManaColor> = Vec::new();
    let mut generic: u32 = 0;
    for sym in &cost.symbols {
        match sym {
            ManaSymbol::Colored(c) => needed_colors.push(*c),
            ManaSymbol::Generic(n) => generic += n,
            ManaSymbol::Hybrid(a, b) => {
                needed_colors.push(*a);
                needed_colors.push(*b);
            }
            ManaSymbol::Phyrexian(c) => needed_colors.push(*c),
            ManaSymbol::Colorless(n) => generic += n,
            ManaSymbol::Snow | ManaSymbol::X => {}
        }
    }

    // Tap a color-matched mana source for each colored pip
    for color in needed_colors {
        let source_id = game
            .state
            .battlefield
            .iter()
            .find(|c| c.owner == PLAYER_0 && !c.tapped && source_produces_color(c, color))
            .map(|c| c.id);
        if let Some(id) = source_id
            && let Ok(evs) = game.state.perform_action(GameAction::ActivateAbility {
                card_id: id,
                ability_index: 0,
                target: None,
            })
        {
            log.apply_events(&evs);
        }
    }

    // Tap any remaining untapped mana source for generic pips
    for _ in 0..generic {
        let source_id = game
            .state
            .battlefield
            .iter()
            .find(|c| c.owner == PLAYER_0 && !c.tapped && is_mana_source(c))
            .map(|c| c.id);
        let Some(id) = source_id else { break };
        if let Ok(evs) = game.state.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
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
#[allow(clippy::type_complexity)]
pub fn trigger_reveal_animation(
    mut reveal: ResMut<RevealPopupState>,
    deck_cards: Query<
        (Entity, &DeckCard, &Transform),
        (
            Without<RevealPeekAnimation>,
            Without<Animating>,
            Without<P1DeckPile>,
        ),
    >,
    bot_deck_cards: Query<
        (Entity, &P1DeckPile, &Transform),
        (
            Without<RevealPeekAnimation>,
            Without<Animating>,
            Without<DeckCard>,
        ),
    >,
    mut commands: Commands,
) {
    let Some(player) = reveal.revealed_player.take() else {
        return;
    };

    // face_up = 180° around the card's own local Y axis (long/height edge), so the flip
    // rotates along the long axis like turning a page.
    if player == PLAYER_0 {
        // Player 1's Goblin Guide revealed player 0's top card.
        if let Some((entity, _, transform)) = deck_cards.iter().max_by_key(|(_, dc, _)| dc.index) {
            let face_up = transform.rotation * Quat::from_rotation_y(std::f32::consts::PI);
            commands
                .entity(entity)
                .insert(Animating)
                .insert(RevealPeekAnimation {
                    progress: 0.0,
                    speed: 0.6,
                    start_rotation: transform.rotation,
                    face_up_rotation: face_up,
                    start_y: transform.translation.y,
                });
        }
    } else {
        // Player 0's Goblin Guide revealed player 1's top card.
        if let Some((entity, _, transform)) =
            bot_deck_cards.iter().max_by_key(|(_, bp, _)| bp.index)
        {
            let face_up = transform.rotation * Quat::from_rotation_y(std::f32::consts::PI);
            commands
                .entity(entity)
                .insert(Animating)
                .insert(RevealPeekAnimation {
                    progress: 0.0,
                    speed: 0.6,
                    start_rotation: transform.rotation,
                    face_up_rotation: face_up,
                    start_y: transform.translation.y,
                });
        }
    }
}

// ── Quality selector panel ─────────────────────────────────────────────────────

const QUALITY_BTN_ACTIVE: Color = Color::srgb(0.15, 0.45, 0.15);
// ── Mulligan UI setup ─────────────────────────────────────────────────────────

pub fn setup_mulligan_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let tf = |size: f32| TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    };

    // Full-screen transparent wrapper centers the panel.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            MulliganOverlay,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(14.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.15, 0.88)),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Opening Hand"),
                    tf(18.0),
                    TextColor(Color::srgb(1.0, 0.88, 0.0)),
                ));
                p.spawn((
                    Text::new(""),
                    tf(14.0),
                    TextColor(Color::WHITE),
                    MulliganStatusText,
                ));
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        Node {
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.08, 0.45, 0.08)),
                        Button,
                        MulliganKeepButton,
                    ))
                    .with_children(|p| {
                        p.spawn((Text::new("Keep (K)"), tf(14.0), TextColor(Color::WHITE)));
                    });

                    p.spawn((
                        Node {
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.45, 0.12, 0.08)),
                        Button,
                        MulliganMulliganButton,
                    ))
                    .with_children(|p| {
                        p.spawn((Text::new("Mulligan (M)"), tf(14.0), TextColor(Color::WHITE)));
                    });
                });
            });
        });
}

// ── Mulligan system ───────────────────────────────────────────────────────────

pub fn mulligan_system(
    mut game: ResMut<GameResource>,
    mut mulligan: ResMut<MulliganState>,
    mut log: ResMut<GameLog>,
    keyboard: Res<ButtonInput<KeyCode>>,
    keep_buttons: Query<&Interaction, (With<MulliganKeepButton>, Changed<Interaction>)>,
    mull_buttons: Query<&Interaction, (With<MulliganMulliganButton>, Changed<Interaction>)>,
    time: Res<Time>,
) {
    if !mulligan.active {
        return;
    }

    // Deal initial hands once.
    if !mulligan.initial_deal_done {
        for _ in 0..7 {
            game.state.players[PLAYER_0].draw_top();
        }
        for _ in 0..7 {
            game.state.players[PLAYER_1].draw_top();
        }
        mulligan.initial_deal_done = true;
        mulligan.p1_timer.reset();
        return;
    }

    // ── Player 0 input ────────────────────────────────────────────────────────
    // Skip button input while in bottoming phase (card clicks are handled in handle_game_input).
    if !mulligan.p0_kept && mulligan.p0_cards_to_bottom == 0 {
        let keep_clicked = keep_buttons.iter().any(|i| *i == Interaction::Pressed);
        let mull_clicked = mull_buttons.iter().any(|i| *i == Interaction::Pressed);

        if keyboard.just_pressed(KeyCode::KeyK) || keep_clicked {
            // London Mulligan: put (mulligan_count) cards to the bottom before keeping.
            if mulligan.p0_mulligans > 0 {
                mulligan.p0_cards_to_bottom = mulligan.p0_mulligans;
                log.push(format!(
                    "Player 0 keeps — click {} card{} to put to bottom",
                    mulligan.p0_mulligans,
                    if mulligan.p0_mulligans == 1 { "" } else { "s" }
                ));
            } else {
                mulligan.p0_kept = true;
                log.push(format!(
                    "Player 0 keeps ({} cards)",
                    game.state.players[PLAYER_0].hand.len()
                ));
            }
        } else if keyboard.just_pressed(KeyCode::KeyM) || mull_clicked {
            // London Mulligan: always draw 7.
            game.state.players[PLAYER_0].return_hand_to_library();
            game.state.players[PLAYER_0]
                .library
                .shuffle(&mut rand::rng());
            for _ in 0..7 {
                game.state.players[PLAYER_0].draw_top();
            }
            mulligan.p0_mulligans += 1;
            log.push(format!(
                "Player 0 mulligans (drawing 7, {} so far)",
                mulligan.p0_mulligans
            ));
        }
    }

    // ── Player 1 bot decision ─────────────────────────────────────────────────
    if !mulligan.p1_kept {
        mulligan.p1_timer.tick(time.delta());
        if mulligan.p1_timer.just_finished() {
            let keep = p1_mulligan_decision(&game.state, mulligan.p1_mulligans);
            if keep || mulligan.p1_mulligans >= 3 {
                // London Mulligan: randomly put (p1_mulligans) cards to the bottom.
                let n = mulligan.p1_mulligans;
                let mut ids: Vec<crabomination::card::CardId> = game.state.players[PLAYER_1]
                    .hand
                    .iter()
                    .map(|c| c.id)
                    .collect();
                ids.shuffle(&mut rand::rng());
                let to_bottom: Vec<crabomination::card::CardId> = ids.into_iter().take(n).collect();
                for id in to_bottom {
                    if let Some(pos) = game.state.players[PLAYER_1]
                        .hand
                        .iter()
                        .position(|c| c.id == id)
                    {
                        let card = game.state.players[PLAYER_1].hand.remove(pos);
                        game.state.players[PLAYER_1].library.push(card);
                    }
                }
                mulligan.p1_kept = true;
                log.push(format!(
                    "Player 1 keeps ({} cards)",
                    game.state.players[PLAYER_1].hand.len()
                ));
            } else {
                // London Mulligan: always draw 7.
                game.state.players[PLAYER_1].return_hand_to_library();
                game.state.players[PLAYER_1]
                    .library
                    .shuffle(&mut rand::rng());
                for _ in 0..7 {
                    game.state.players[PLAYER_1].draw_top();
                }
                mulligan.p1_mulligans += 1;
                log.push(format!(
                    "Player 1 mulligans (drawing 7, {} so far)",
                    mulligan.p1_mulligans
                ));
                mulligan.p1_timer =
                    bevy::time::Timer::from_seconds(1.5, bevy::time::TimerMode::Once);
            }
        }
    }

    // ── End mulligan phase ────────────────────────────────────────────────────
    if mulligan.p0_kept && mulligan.p1_kept {
        mulligan.active = false;
        log.push("Mulligan complete. Game begins!");
    }
}

// ── Mulligan UI update ────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
pub fn update_mulligan_ui(
    mulligan: Res<MulliganState>,
    game: Res<GameResource>,
    mut overlay_q: Query<&mut Visibility, With<MulliganOverlay>>,
    mut status_q: Query<&mut Text, With<MulliganStatusText>>,
    mut keep_btn_q: Query<
        &mut Visibility,
        (
            With<MulliganKeepButton>,
            Without<MulliganOverlay>,
            Without<MulliganMulliganButton>,
        ),
    >,
    mut mull_btn_q: Query<
        &mut Visibility,
        (
            With<MulliganMulliganButton>,
            Without<MulliganOverlay>,
            Without<MulliganKeepButton>,
        ),
    >,
) {
    let Ok(mut vis) = overlay_q.single_mut() else {
        return;
    };
    if !mulligan.active || !mulligan.initial_deal_done {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Visible;

    // Update status text
    if let Ok(mut text) = status_q.single_mut() {
        let p0_status = if mulligan.p0_cards_to_bottom > 0 {
            format!(
                "You: put {} card{} to bottom",
                mulligan.p0_cards_to_bottom,
                if mulligan.p0_cards_to_bottom == 1 {
                    ""
                } else {
                    "s"
                }
            )
        } else if mulligan.p0_kept {
            format!(
                "You: kept {} cards",
                game.state.players[PLAYER_0].hand.len()
            )
        } else if mulligan.p0_mulligans == 0 {
            "You: 7 cards".into()
        } else {
            format!(
                "You: 7 cards — keep & put {} back, or mull again",
                mulligan.p0_mulligans
            )
        };
        let p1_status = if mulligan.p1_kept {
            format!(
                "Opp: kept {} cards",
                game.state.players[PLAYER_1].hand.len()
            )
        } else {
            "Opp: deciding...".into()
        };
        text.0 = format!("{p0_status}  |  {p1_status}");
    }

    // Hide Keep/Mulligan buttons once player 0 has decided or is in bottoming phase.
    // Use Inherited (not Visible) so parent Visibility::Hidden propagates correctly.
    let show_btns = !mulligan.p0_kept && mulligan.p0_cards_to_bottom == 0;
    for mut v in &mut keep_btn_q {
        *v = if show_btns {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut v in &mut mull_btn_q {
        *v = if show_btns {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

const QUALITY_BTN_INACTIVE: Color = Color::srgb(0.12, 0.12, 0.18);

pub fn setup_quality_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    quality: Res<RenderQuality>,
) {
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let tf = |size: f32| TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(6.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Quality"),
                tf(11.0),
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|p| {
                for q in RenderQuality::ALL {
                    let bg = if q == *quality {
                        QUALITY_BTN_ACTIVE
                    } else {
                        QUALITY_BTN_INACTIVE
                    };
                    p.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        Button,
                        QualityButton(q),
                    ))
                    .with_children(|p| {
                        p.spawn((Text::new(q.label()), tf(12.0), TextColor(Color::WHITE)));
                    });
                }
            });
        });
}

pub fn handle_quality_buttons(
    mut buttons: Query<(&Interaction, &QualityButton, &mut BackgroundColor)>,
    quality: Res<RenderQuality>,
    mut messages: MessageWriter<ChangeQuality>,
) {
    // First pass: find if any button was just pressed (immutable borrow)
    let pressed = buttons
        .iter()
        .find(|(i, _, _)| **i == Interaction::Pressed)
        .map(|(_, btn, _)| btn.0);

    if let Some(q) = pressed {
        messages.write(ChangeQuality(q));
    }

    // Second pass: sync button colors (mutable borrow)
    // Update immediately on press; otherwise track the quality resource.
    if pressed.is_some() || quality.is_changed() {
        let active = pressed.unwrap_or(*quality);
        for (_, btn, mut bg) in buttons.iter_mut() {
            *bg = if btn.0 == active {
                BackgroundColor(QUALITY_BTN_ACTIVE)
            } else {
                BackgroundColor(QUALITY_BTN_INACTIVE)
            };
        }
    }
}
