//! HUD overlay, visual sync, input handling, and ability menu. Gizmos and the
//! quality panel live in their own sibling modules (`gizmos.rs`, `quality.rs`).

use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::{GameAction, Target, TurnStep};
use crabomination::net::StackItemView;

use super::ui::RevealPopupState;
use crate::card::{
    Animating, BattlefieldCard, CARD_THICKNESS, CARD_WIDTH, CardHoverLift, CardHovered,
    CardMeshAssets, CardOwner, DECK_CARD_Y_STEP, DeckCard, DeckPile, DrawCardAnimation,
    GameCardId, GraveyardPile, HandCard, HandSlideAnimation, OpponentHandCard,
    PlayCardAnimation, PlayerTargetZone, RevealPeekAnimation, SendToGraveyardAnimation,
    StackCard, TapAnimation, TapState, ValidTarget, back_face_rotation, bf_card_transform,
    card_back_face_material, card_front_material, deck_position, graveyard_position,
    hand_card_transform, land_card_transform, spawn_single_card,
};
use crate::game::{AbilityMenuState, BlockingState, GameLog, TargetingState, format_mana_pool_from_pool};
use crate::net_plugin::{CurrentView, LatestServerEvents, NetOutbox};

/// System set label for the ordered game-logic chain (mulligan → advance → input).
#[derive(bevy::ecs::schedule::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameLogicSet;

/// Bundled mutable resources for `handle_game_input` to stay within Bevy's 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct GameInputResources<'w> {
    pub log: ResMut<'w, GameLog>,
    pub targeting: ResMut<'w, TargetingState>,
    pub blocking: ResMut<'w, BlockingState>,
    pub reveal: ResMut<'w, RevealPopupState>,
    pub menu_state: ResMut<'w, AbilityMenuState>,
    pub ff: ResMut<'w, FastForward>,
    pub alt_cast: ResMut<'w, crate::game::AltCastState>,
    pub flipped_hand: ResMut<'w, crate::game::FlippedHandCards>,
    pub card_names: ResMut<'w, crate::game::CardNames>,
}
/// Process `SwapFrontMaterial` markers: walk each entity's children,
/// find the `FrontFaceMesh` child, swap its `MeshMaterial3d` to the
/// requested handle, update the parent's `CardFrontTexture` so peek /
/// hover popups stay in sync, and remove the marker. Used by the
/// hand→battlefield transition for flipped MDFCs (the played-back-face
/// image needs to land on the front-child mesh under standard bf
/// orientation).
#[allow(clippy::type_complexity)]
pub fn apply_swap_front_material(
    mut commands: Commands,
    swap_q: Query<
        (Entity, &Children, &crate::card::SwapFrontMaterial),
        With<crate::card::SwapFrontMaterial>,
    >,
    mut front_meshes: Query<
        &mut MeshMaterial3d<StandardMaterial>,
        With<crate::card::FrontFaceMesh>,
    >,
    mut tex_q: Query<&mut crate::card::CardFrontTexture>,
) {
    for (entity, children, swap) in &swap_q {
        for child in children.iter() {
            if let Ok(mut mat) = front_meshes.get_mut(child) {
                *mat = MeshMaterial3d(swap.new_front.clone());
                break;
            }
        }
        if let Ok(mut tex) = tex_q.get_mut(entity) {
            tex.0 = swap.new_path.clone();
        }
        commands
            .entity(entity)
            .remove::<crate::card::SwapFrontMaterial>();
    }
}

/// Pretty-print a `GameEventWire` for the in-game log, resolving any
/// CardId via the running `CardNames` map so the player sees real card
/// names instead of `CardId(N)` debug strings.
fn format_event(ev: &crabomination::net::GameEventWire, names: &crate::game::CardNames) -> String {
    use crabomination::net::GameEventWire as E;
    let n = |id: crabomination::card::CardId| names.get(id);
    match ev {
        E::StepChanged(s) => format!("Step → {s:?}"),
        E::TurnStarted { player, turn } => format!("Turn {turn} — P{player}"),
        E::CardDrawn { player, card_id } => format!("P{player} drew {}", n(*card_id)),
        E::CardDiscarded { player, card_id } => {
            format!("P{player} discarded {}", n(*card_id))
        }
        E::LandPlayed { player, card_id } => format!("P{player} played {}", n(*card_id)),
        E::SpellCast { player, card_id } => format!("P{player} cast {}", n(*card_id)),
        E::AbilityActivated { source } => format!("{} ability activated", n(*source)),
        E::ManaAdded { player, color } => format!("P{player} adds {color:?}"),
        E::ColorlessManaAdded { player } => format!("P{player} adds colorless"),
        E::PermanentEntered { card_id } => format!("{} entered the battlefield", n(*card_id)),
        E::PermanentExiled { card_id } => format!("{} was exiled", n(*card_id)),
        E::DamageDealt { amount, to_player, to_card } => match (to_player, to_card) {
            (Some(p), _) => format!("{amount} damage → P{p}"),
            (_, Some(cid)) => format!("{amount} damage → {}", n(*cid)),
            _ => format!("{amount} damage"),
        },
        E::LifeLost { player, amount } => format!("P{player} loses {amount} life"),
        E::LifeGained { player, amount } => format!("P{player} gains {amount} life"),
        E::CreatureDied { card_id } => format!("{} died", n(*card_id)),
        E::PumpApplied { card_id, power, toughness } => {
            format!("{} +{power}/+{toughness}", n(*card_id))
        }
        E::CounterAdded { card_id, counter_type, count } => {
            format!("+{count} {counter_type:?} on {}", n(*card_id))
        }
        E::CounterRemoved { card_id, counter_type, count } => {
            format!("−{count} {counter_type:?} on {}", n(*card_id))
        }
        E::PermanentTapped { card_id } => format!("{} tapped", n(*card_id)),
        E::PermanentUntapped { card_id } => format!("{} untapped", n(*card_id)),
        E::TokenCreated { card_id } => format!("token {} created", n(*card_id)),
        E::CardMilled { player, card_id } => format!("P{player} milled {}", n(*card_id)),
        E::ScryPerformed { player, looked_at, bottomed } => {
            format!("P{player} scry {looked_at} ({bottomed} to bottom)")
        }
        E::AttackerDeclared(cid) => format!("{} attacks", n(*cid)),
        E::BlockerDeclared { blocker, attacker } => {
            format!("{} blocks {}", n(*blocker), n(*attacker))
        }
        E::CombatResolved => "Combat resolved".into(),
        E::FirstStrikeDamageResolved => "First-strike damage resolved".into(),
        E::TopCardRevealed { player, card_name, .. } => {
            format!("P{player} revealed {card_name}")
        }
        E::AttachmentMoved { attachment, attached_to } => match attached_to {
            Some(target) => format!("{} attached to {}", n(*attachment), n(*target)),
            None => format!("{} unattached", n(*attachment)),
        },
        E::PoisonAdded { player, amount } => format!("P{player} +{amount} poison"),
        E::LoyaltyAbilityActivated { planeswalker, loyalty_change } => {
            format!("{} loyalty {loyalty_change:+}", n(*planeswalker))
        }
        E::LoyaltyChanged { card_id, new_loyalty } => {
            format!("{} loyalty = {new_loyalty}", n(*card_id))
        }
        E::PlaneswalkerDied { card_id } => format!("{} died (planeswalker)", n(*card_id)),
        E::SpellsCopied { original, count } => {
            format!("{} copied ×{count}", n(*original))
        }
        E::SurveilPerformed { player, looked_at, graveyarded } => {
            format!("P{player} surveil {looked_at} ({graveyarded} to graveyard)")
        }
        E::GameOver { winner } => match winner {
            Some(p) => format!("Game over — P{p} wins"),
            None => "Game over — draw".into(),
        },
    }
}

/// Drives End Turn / Next Turn fast-forward: while these flags are set,
/// `auto_advance_p0` keeps submitting `PassPriority` each frame until the
/// target condition is reached.
#[derive(Resource, Default)]
pub struct FastForward {
    /// Pass until the active player is no longer us (hand off to bot's turn).
    pub end_turn: bool,
    /// Pass until we're back in our own PreCombatMain with priority.
    pub next_turn: bool,
}

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

/// Root entity of the floating ability context menu.
#[derive(Component)]
pub struct AbilityMenu;

/// One item in the ability menu.
#[derive(Component)]
pub struct AbilityMenuItem {
    pub card_id: CardId,
    pub ability_index: usize,
}

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

pub fn update_turn_text(
    view: Res<CurrentView>,
    mut q: Query<&mut Text, With<TurnInfoText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    t.0 = if let Some(winner) = cv.game_over {
        match winner {
            Some(p) => format!("GAME OVER — {} wins!", player_name(cv, p)),
            None => "GAME OVER — Draw!".into(),
        }
    } else {
        format!(
            "Turn {} | {:?} | {}'s turn",
            cv.turn,
            cv.step,
            player_name(cv, cv.active_player)
        )
    };
}

fn player_name(cv: &crabomination::net::ClientView, seat: usize) -> String {
    cv.players
        .iter()
        .find(|p| p.seat == seat)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {seat}"))
}

pub fn update_player_text(
    view: Res<CurrentView>,
    mut q: Query<&mut Text, With<PlayerStatusText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.players.iter().find(|p| p.seat == cv.your_seat) else { return };
    t.0 = format!(
        "{} | Life: {} | Hand: {} | Deck: {} | GY: {} | Mana: {}",
        p.name,
        p.life,
        p.hand.len(),
        p.library.size,
        p.graveyard.len(),
        format_mana_pool_from_pool(&p.mana_pool)
    );
}

/// Render a status line per opponent. The HUD container only has one
/// `P1StatusText` entity today — it shows summed status if there are multiple
/// opponents.
pub fn update_p1_text(
    view: Res<CurrentView>,
    mut q: Query<&mut Text, With<P1StatusText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    let lines: Vec<String> = cv
        .players
        .iter()
        .filter(|p| p.seat != cv.your_seat)
        .map(|p| {
            format!(
                "{} | Life: {} | Hand: {} | Deck: {} | GY: {}",
                p.name,
                p.life,
                p.hand.len(),
                p.library.size,
                p.graveyard.len()
            )
        })
        .collect();
    t.0 = lines.join("\n");
}

/// Render the in-memory `GameLog.entries` into the `GameLogText` widget.
/// Without this system the log resource accumulated entries that never
/// appeared on screen.
pub fn update_log_text(
    log: Res<GameLog>,
    mut q: Query<&mut Text, With<GameLogText>>,
) {
    if !log.is_changed() {
        return;
    }
    let Ok(mut t) = q.single_mut() else { return };
    t.0 = log.entries.join("\n");
}

pub fn update_phase_chart(
    view: Res<CurrentView>,
    mut labels: Query<(&PhaseStepLabel, &mut TextColor)>,
) {
    let Some(cv) = &view.0 else { return };
    let current = cv.step;
    for (label, mut color) in &mut labels {
        *color = if label.0 == current {
            TextColor(Color::srgb(1.0, 0.88, 0.0))
        } else {
            TextColor(Color::srgba(0.55, 0.55, 0.55, 0.8))
        };
    }
}


pub fn update_hint(
    view: Res<CurrentView>,
    targeting: Res<TargetingState>,
    blocking: Res<BlockingState>,
    mut q: Query<&mut Text, With<HintText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() {
        t.0 = String::new();
        return;
    }
    if targeting.active {
        t.0 = "Click a target (creature or opponent). Right-click/Esc to cancel.".into();
        return;
    }
    let stack_size = cv.stack.len();
    if stack_size > 0 {
        t.0 = format!("{stack_size} item(s) on stack. P = Resolve.");
        return;
    }
    let your_seat = cv.your_seat;
    let viewer_is_defending =
        cv.step == TurnStep::DeclareBlockers && cv.active_player != your_seat && cv.priority == your_seat;
    if viewer_is_defending {
        t.0 = if blocking.selected_blocker.is_some() {
            "Click an attacking creature to assign the block. Right-click to cancel.".into()
        } else {
            "Click one of your creatures to block with it. P = Skip blocking.".into()
        };
        return;
    }
    t.0 = match (cv.active_player == your_seat, cv.step) {
        (true, TurnStep::PreCombatMain) | (true, TurnStep::PostCombatMain) => {
            "Click a hand card to play it. P = Pass Priority.".into()
        }
        (true, TurnStep::DeclareAttackers) => {
            "A = Attack with all eligible creatures. P = Pass (no attack).".into()
        }
        (true, TurnStep::DeclareBlockers) => {
            "Opponent is assigning blocks. P = Proceed to combat.".into()
        }
        (true, _) => String::new(),
        (false, _) => format!("{} is thinking...", player_name(cv, cv.active_player)),
    };
}

// ── Visual sync: reconcile 3D card entities with the server-projected view ───

/// Count `(lands, creatures)` for the given owner.
fn bf_row_counts(battlefield: &[crabomination::net::PermanentView], owner: usize) -> (usize, usize) {
    let lands = battlefield.iter().filter(|c| c.owner == owner && c.is_land()).count();
    let creatures = battlefield.iter().filter(|c| c.owner == owner && !c.is_land()).count();
    (lands, creatures)
}

/// Find the slot of a card within its row.
fn bf_row_slot(
    battlefield: &[crabomination::net::PermanentView],
    owner: usize,
    card_id: CardId,
    is_land: bool,
) -> Option<usize> {
    battlefield
        .iter()
        .filter(|c| c.owner == owner && c.is_land() == is_land)
        .position(|c| c.id == card_id)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn sync_game_visuals(
    mut commands: Commands,
    view: Res<CurrentView>,
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
            Option<&crate::card::FlippedFace>,
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
    mut deck_pile_q: Query<
        (Entity, &DeckPile, &mut Transform, &mut CardHoverLift),
        (Without<GameCardId>, Without<OpponentHandCard>),
    >,
    mut graveyard_q: Query<
        (
            &GraveyardPile,
            &mut Transform,
            &mut Visibility,
            &mut CardHoverLift,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<DeckPile>, Without<GameCardId>),
    >,
    opponent_hand_q: Query<
        (Entity, &OpponentHandCard, &Transform, Has<Animating>),
        (Without<DeckPile>, Without<GraveyardPile>),
    >,
    gy_anims: Query<&SendToGraveyardAnimation>,
    all_bf_entities: Query<&GameCardId, With<BattlefieldCard>>,
    all_hand_entity_ids: Query<&GameCardId, With<HandCard>>,
) {
    let Some(cv) = &view.0 else { return };
    let viewer = cv.your_seat;
    let n_seats = cv.players.len();
    let gy_sizes: Vec<usize> = cv.players.iter().map(|p| p.graveyard.len()).collect();
    let gy_size = |owner: usize| gy_sizes.get(owner).copied().unwrap_or(0);
    let deck_size = |owner: usize| {
        cv.players
            .iter()
            .find(|p| p.seat == owner)
            .map(|p| p.library.size)
            .unwrap_or(0)
    };

    // ── Always update: deck pile heights, spawn/despawn to match deck sizes ───

    // Existing pile cards: shrink to current deck sizes.
    let mut deck_pile_counts: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    for (entity, pile, mut transform, mut lift) in &mut deck_pile_q {
        let size = deck_size(pile.owner);
        if pile.index >= size {
            commands.entity(entity).despawn();
        } else {
            let base = deck_position(pile.owner, viewer, n_seats);
            let y = pile.index as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(base.x, y, base.z);
            transform.translation = pos;
            lift.base_translation = pos;
            *deck_pile_counts.entry(pile.owner).or_default() += 1;
        }
    }
    // Spawn missing deck pile cards per seat to match library size.
    if let Some(card_assets_ref) = &card_assets {
        for seat in 0..n_seats {
            let target_size = deck_size(seat);
            let current = deck_pile_counts.get(&seat).copied().unwrap_or(0);
            if current >= target_size {
                continue;
            }
            let base = deck_position(seat, viewer, n_seats);
            let rot = back_face_rotation(seat, viewer);
            for i in current..target_size {
                let y = i as f32 * DECK_CARD_Y_STEP + 0.01;
                let pos = Vec3::new(base.x, y, base.z);
                commands.spawn((
                    Mesh3d(card_assets_ref.card_mesh.clone()),
                    MeshMaterial3d(card_assets_ref.back_material.clone()),
                    Transform::from_translation(pos).with_rotation(rot),
                    Visibility::default(),
                    DeckPile { owner: seat, index: i },
                    CardHoverLift { current_lift: 0.0, target_lift: 0.0, base_translation: pos },
                ));
            }
        }
    }

    let mut gy_in_flight: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for anim in &gy_anims {
        *gy_in_flight.entry(anim.owner).or_default() += 1;
    }

    for (gy, mut transform, mut vis, mut lift, _mat) in &mut graveyard_q {
        let gy_count = gy_size(gy.owner);
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_count.saturating_sub(in_flight);
        if arrived == 0 {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Visible;
            let base_pos = graveyard_position(gy.owner, viewer, n_seats);
            let y = arrived as f32 * DECK_CARD_Y_STEP + 0.01;
            let pos = Vec3::new(base_pos.x, y, base_pos.z);
            transform.translation = pos;
            lift.base_translation = pos;
        }
    }

    if !view.is_changed() && removed_animating.is_empty() {
        return;
    }
    let Some(card_assets) = card_assets else { return };

    // Snapshot of every opponent hand visual entity (entity, owner, slot, pos,
    // rot, is_animating). We feed a per-owner hand pool so an opponent
    // "playing" a card consumes one of their face-down hand visuals as the
    // animation start position.
    let all_opp_hand: Vec<(Entity, usize, usize, Vec3, Quat, bool)> = opponent_hand_q
        .iter()
        .map(|(e, bh, t, is_animating)| {
            (e, bh.owner, bh.slot, t.translation, t.rotation, is_animating)
        })
        .collect();

    // Per-opponent hand pool of available (non-animating) cards to consume on
    // a hand→battlefield promotion. Sort by slot so we consume in stable
    // order, then drop the slot field.
    let mut pool_with_slot: std::collections::HashMap<usize, Vec<(Entity, usize, Vec3, Quat)>> =
        std::collections::HashMap::new();
    for (e, owner, slot, pos, rot, is_anim) in &all_opp_hand {
        if *is_anim {
            continue;
        }
        pool_with_slot
            .entry(*owner)
            .or_default()
            .push((*e, *slot, *pos, *rot));
    }
    for pool in pool_with_slot.values_mut() {
        pool.sort_by_key(|(_, slot, _, _)| *slot);
    }
    let mut hand_pool_by_owner: std::collections::HashMap<usize, Vec<(Entity, Vec3, Quat)>> =
        pool_with_slot
            .into_iter()
            .map(|(owner, pool)| {
                (
                    owner,
                    pool.into_iter().map(|(e, _, pos, rot)| (e, pos, rot)).collect(),
                )
            })
            .collect();
    let mut promoted: HashSet<Entity> = HashSet::new();

    // ── Update graveyard pile face-up material ────────────────────────────────
    for (gy, _transform, _vis, _lift, mut mat) in &mut graveyard_q {
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_size(gy.owner).saturating_sub(in_flight);
        if arrived > 0 {
            let top_name: Option<String> = cv.players[gy.owner].graveyard.get(arrived - 1).map(|c| c.name.clone());
            if let Some(name) = top_name {
                *mat = MeshMaterial3d(card_front_material(&name, &mut materials, &asset_server));
            }
        }
    }

    // ── Build zone ID sets ────────────────────────────────────────────────────
    let stack_ids: HashSet<CardId> = cv.stack.iter().filter_map(|item| {
        if let StackItemView::Known(k) = item { Some(k.source) } else { None }
    }).collect();
    let hand_ids: HashSet<CardId> = cv.players[viewer].hand.iter().map(|c| c.id()).collect();
    let bf_ids_by_owner: std::collections::HashMap<usize, HashSet<CardId>> = (0..n_seats)
        .map(|seat| {
            (
                seat,
                cv.battlefield.iter().filter(|c| c.owner == seat).map(|c| c.id).collect(),
            )
        })
        .collect();
    let creatures_by_owner: std::collections::HashMap<usize, usize> = (0..n_seats)
        .map(|seat| (seat, bf_row_counts(&cv.battlefield, seat).1))
        .collect();
    let creature_count = |seat: usize| creatures_by_owner.get(&seat).copied().unwrap_or(0);
    let bf_ids_for = |seat: usize| -> &HashSet<CardId> {
        bf_ids_by_owner.get(&seat).expect("seat in bf_ids map")
    };
    let hand_total = hand_ids.len();
    let all_bf_ids: HashSet<CardId> = cv.battlefield.iter().map(|c| c.id).collect();
    let visual_bf_ids: HashSet<CardId> = all_bf_entities.iter().map(|gid| gid.0).collect();
    // IDs visible in any player's graveyard. Used to disambiguate where a
    // disappeared hand card actually went: present in a graveyard → was
    // discarded/resolved-as-spell → fly to graveyard. Absent → it was
    // shuffled back to library (mulligan) → fly to deck pile.
    let in_any_graveyard: HashSet<CardId> = cv
        .players
        .iter()
        .flat_map(|p| p.graveyard.iter().map(|c| c.id))
        .collect();
    // Top-of-deck position for the viewer (used as the destination of a
    // mulligan put-back animation and the start of fetch/tutor animations).
    let viewer_deck_base = deck_position(viewer, viewer, n_seats);
    let viewer_deck_top_y = (cv.players[viewer].library.size as f32) * DECK_CARD_Y_STEP + 0.5;
    let viewer_deck_top = Vec3::new(viewer_deck_base.x, viewer_deck_top_y, viewer_deck_base.z);
    let viewer_deck_back_rot = back_face_rotation(viewer, viewer);

    // ── Deck → Hand transitions (viewer's deck-card visual → face-up hand) ──
    for (entity, game_id, _deck_card, transform) in &deck_cards {
        if hand_ids.contains(&game_id.0) {
            let slot = cv.players[viewer].hand.iter().position(|c| c.id() == game_id.0)
                .unwrap_or(hand_total.saturating_sub(1));
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total);
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

    // ── Spawn viewer hand cards that have no visual entity yet ───────────────
    {
        let has_entity: HashSet<CardId> = all_hand_entity_ids.iter().map(|gid| gid.0)
            .chain(deck_cards.iter().map(|(_, gid, _, _)| gid.0))
            .collect();
        let deck_base = deck_position(viewer, viewer, n_seats);
        let deck_y = cv.players[viewer].library.size as f32 * DECK_CARD_Y_STEP + 0.5;
        let deck_pos = Vec3::new(deck_base.x, deck_y, deck_base.z);
        for (slot, card_view) in cv.players[viewer].hand.iter().enumerate() {
            use crabomination::net::HandCardView;
            let HandCardView::Known(known) = card_view else { continue };
            let card_id = known.id;
            if has_entity.contains(&card_id) { continue; }
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total);
            let front_mat = card_front_material(&known.name, &mut materials, &asset_server);
            // For MDFC cards (Pathways), paint the back-child with the
            // back face's Scryfall image instead of the cardback so a
            // 180° flip animation reveals the alternate face. Uses the
            // `_back`-suffixed asset path that the prefetch downloaded
            // with `face=back`.
            let back_mat = if let Some(back_name) = known.back_face_name.as_deref() {
                card_back_face_material(back_name, &mut materials, &asset_server)
            } else {
                card_assets.back_material.clone()
            };
            let entity = spawn_single_card(
                &mut commands,
                &card_assets.card_mesh,
                front_mat,
                back_mat,
                Transform::from_translation(deck_pos),
                GameCardId(card_id),
                &known.name,
                target.translation,
            );
            commands.entity(entity).insert((
                HandCard { slot },
                Animating,
                DrawCardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: deck_pos,
                    start_rotation: Transform::from_translation(deck_pos).rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                },
            ));
        }
    }

    // ── Viewer hand → Battlefield / Stack / graveyard transitions ───────────
    for (entity, game_id, transform, stack_card, _lift, flipped_marker) in &hand_cards {
        if bf_ids_for(viewer).contains(&game_id.0) {
            let bf_card = cv.battlefield.iter().find(|c| c.id == game_id.0);
            let is_land = bf_card.is_some_and(|c| c.is_land());
            let target = if is_land {
                land_card_transform(&cv.battlefield, viewer, viewer, n_seats, game_id.0)
                    .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, true, false))
            } else {
                let slot = bf_row_slot(&cv.battlefield, viewer, game_id.0, false).unwrap_or(0);
                bf_card_transform(viewer, viewer, n_seats, slot, creature_count(viewer), false, false)
            };
            // Flipped MDFC played as its back face: the engine swapped
            // the card's definition to the back face, so the bf card's
            // name is the back-face name. We need the front-child mesh
            // to display the back-face image (via the `_back` asset
            // path) so the played face is up on the battlefield. Animate
            // back to the standard (un-flipped) bf rotation; without
            // this, the play animation would un-rotate the card and
            // expose the original front face (the wrong side).
            if flipped_marker.is_some()
                && let Some(bf) = bf_card
            {
                let new_front = card_back_face_material(&bf.name, &mut materials, &asset_server);
                commands.entity(entity).insert(crate::card::SwapFrontMaterial {
                    new_front,
                    new_path: crate::scryfall::card_back_face_asset_path(&bf.name),
                });
                commands.entity(entity).remove::<crate::card::FlippedFace>();
            }
            commands
                .entity(entity)
                .remove::<HandCard>()
                .remove::<StackCard>()
                .remove::<CardHovered>()
                .insert(BattlefieldCard { is_land })
                .insert(CardOwner(viewer))
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
                let idx = cv.stack.iter().position(|item| matches!(item, StackItemView::Known(k) if k.source == game_id.0)).unwrap_or(0);
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
            // The hand card is no longer anywhere visible. If a graveyard
            // now holds it the card was discarded or resolved as a spell —
            // fly it to the graveyard. Otherwise it was shuffled back into a
            // library (mulligan put-back) — fly it to the deck pile.
            if in_any_graveyard.contains(&game_id.0) {
                let gy_pos = graveyard_position(viewer, viewer, n_seats);
                let gy_rot = back_face_rotation(viewer, viewer);
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
                        target_translation: gy_pos,
                        target_rotation: gy_rot,
                        owner: viewer,
                    });
            } else {
                commands
                    .entity(entity)
                    .remove::<HandCard>()
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(crate::card::ReturnToDeckAnimation {
                        progress: 0.0,
                        speed: 1.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: viewer_deck_top,
                        target_rotation: viewer_deck_back_rot,
                    });
            }
        }
    }

    // ── Battlefield → Graveyard transitions ──────────────────────────────────
    for (entity, game_id, owner, _bf, transform, _) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) {
            let gy_pos = graveyard_position(owner.0, viewer, n_seats);
            let gy_rot = back_face_rotation(owner.0, viewer);
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

    // ── Spawn new opponent battlefield cards (one pass per opponent seat) ───
    for seat in 0..n_seats {
        if seat == viewer {
            continue;
        }
        let to_spawn: Vec<(CardId, String, bool, bool)> = cv
            .battlefield
            .iter()
            .filter(|c| c.owner == seat && !visual_bf_ids.contains(&c.id))
            .map(|c| (c.id, c.name.clone(), c.is_land(), c.tapped))
            .collect();

        for (card_id, card_name, is_land, tapped) in to_spawn {
            let target = if is_land {
                land_card_transform(&cv.battlefield, seat, viewer, n_seats, card_id)
                    .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, true, false))
            } else {
                let slot = bf_row_slot(&cv.battlefield, seat, card_id, false).unwrap_or(0);
                bf_card_transform(seat, viewer, n_seats, slot, creature_count(seat), false, tapped)
            };

            // Consume one of this opponent's face-down hand visuals as the
            // animation start (so the card appears to fly out of their hand).
            let pool = hand_pool_by_owner.entry(seat).or_default();
            let (start_pos, start_rot) = if let Some((hand_entity, pos, rot)) = pool.pop() {
                commands.entity(hand_entity).despawn();
                promoted.insert(hand_entity);
                (pos, rot)
            } else {
                // Opponent has no hand-card visual to consume — the card
                // came from their library or graveyard (fetchland, tutor,
                // reanimate, token). Animate from the top of their deck
                // pile rather than from where their hand fan would be.
                let opp_deck_base = deck_position(seat, viewer, n_seats);
                let opp_deck_size = cv
                    .players
                    .iter()
                    .find(|p| p.seat == seat)
                    .map(|p| p.library.size)
                    .unwrap_or(0);
                let y = (opp_deck_size as f32) * DECK_CARD_Y_STEP + 0.5;
                let pos = Vec3::new(opp_deck_base.x, y, opp_deck_base.z);
                (pos, back_face_rotation(seat, viewer))
            };

            let front_mat = card_front_material(&card_name, &mut materials, &asset_server);
            let entity = spawn_single_card(
                &mut commands,
                &card_assets.card_mesh,
                front_mat,
                card_assets.back_material.clone(),
                Transform::from_translation(start_pos).with_rotation(start_rot),
                GameCardId(card_id),
                &card_name,
                target.translation,
            );
            commands.entity(entity).insert((
                BattlefieldCard { is_land },
                CardOwner(seat),
                TapState { tapped },
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
    }

    // ── Opponent hand reconciliation (one pass per opponent seat) ───────────
    let mut removed_opp_entities: HashSet<Entity> = promoted.clone();
    for seat in 0..n_seats {
        if seat == viewer {
            continue;
        }
        let target_hand_size = cv
            .players
            .iter()
            .find(|p| p.seat == seat)
            .map(|p| p.hand.len())
            .unwrap_or(0);

        let opp_hand_for_seat: Vec<&(Entity, usize, usize, Vec3, Quat, bool)> = all_opp_hand
            .iter()
            .filter(|(_, owner, _, _, _, _)| *owner == seat)
            .collect();
        let visual_count = opp_hand_for_seat
            .iter()
            .filter(|(e, _, _, _, _, _)| !promoted.contains(e))
            .count();

        if visual_count > target_hand_size {
            let mut sorted: Vec<_> = opp_hand_for_seat
                .iter()
                .filter(|(e, _, _, _, _, _)| !promoted.contains(e))
                .collect();
            sorted.sort_by_key(|(_, _, slot, _, _, _)| std::cmp::Reverse(*slot));
            for entry in sorted.iter().take(visual_count - target_hand_size) {
                let entity = entry.0;
                commands.entity(entity).despawn();
                removed_opp_entities.insert(entity);
            }
        } else if visual_count < target_hand_size {
            let base = deck_position(seat, viewer, n_seats);
            let deck_y = target_hand_size as f32 * DECK_CARD_Y_STEP + 0.5;
            let deck_pos = Vec3::new(base.x, deck_y, base.z);
            for slot in visual_count..target_hand_size {
                let target = hand_card_transform(seat, viewer, n_seats, slot, target_hand_size);
                let back_mat = card_assets.back_material.clone();
                let card_mesh = card_assets.card_mesh.clone();
                let start_transform = Transform::from_translation(deck_pos);
                commands
                    .spawn((
                        start_transform,
                        Visibility::default(),
                        OpponentHandCard { owner: seat, slot },
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

        // Despawn any cards whose slot is now out of range (animating card
        // with high slot may have escaped the count pass).
        for entry in &opp_hand_for_seat {
            let (entity, _, slot, _, _, _) = **entry;
            if !removed_opp_entities.contains(&entity) && slot >= target_hand_size {
                commands.entity(entity).despawn();
                removed_opp_entities.insert(entity);
            }
        }

        // Slide remaining non-animating cards to their new positions.
        for entry in &opp_hand_for_seat {
            let (entity, _, slot, pos, _rot, is_animating) = **entry;
            if removed_opp_entities.contains(&entity) || is_animating {
                continue;
            }
            let target = hand_card_transform(seat, viewer, n_seats, slot, target_hand_size);
            if pos.distance(target.translation) > 0.001 {
                commands
                    .entity(entity)
                    .insert(Animating)
                    .insert(HandSlideAnimation {
                        progress: 0.0,
                        speed: 4.0,
                        start_translation: pos,
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

    // ── Spawn new viewer battlefield cards that don't have entities yet ──────
    let viewer_to_spawn: Vec<(CardId, String, bool, bool)> = cv
        .battlefield
        .iter()
        .filter(|c| {
            c.owner == viewer
                && !visual_bf_ids.contains(&c.id)
                && !hand_cards.iter().any(|(_, gid, _, _, _, _)| gid.0 == c.id)
        })
        .map(|c| (c.id, c.name.clone(), c.is_land(), c.tapped))
        .collect();

    // Battlefield cards that didn't come from the viewer's hand (fetchlands,
    // tutors that drop directly onto the battlefield, reanimate, tokens) get
    // a `library → battlefield` arc animation starting from the top of the
    // viewer's deck pile, instead of teleporting in.
    for (card_id, card_name, is_land, tapped) in viewer_to_spawn {
        let target = if is_land {
            land_card_transform(&cv.battlefield, viewer, viewer, n_seats, card_id)
                .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, true, false))
        } else {
            let slot = bf_row_slot(&cv.battlefield, viewer, card_id, false).unwrap_or(0);
            bf_card_transform(viewer, viewer, n_seats, slot, creature_count(viewer), false, tapped)
        };
        let front_mat = card_front_material(&card_name, &mut materials, &asset_server);
        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(viewer_deck_top).with_rotation(viewer_deck_back_rot),
            GameCardId(card_id),
            &card_name,
            target.translation,
        );
        commands.entity(entity).insert((
            BattlefieldCard { is_land },
            CardOwner(viewer),
            TapState { tapped },
            Animating,
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: viewer_deck_top,
                start_rotation: viewer_deck_back_rot,
                target_translation: target.translation,
                target_rotation: target.rotation,
            },
        ));
    }

    // ── Rebalance viewer hand slots ──────────────────────────────────────────
    for (entity, game_id, _transform, stack_card, lift, _flipped_marker) in &hand_cards {
        if stack_card.is_some() { continue; }
        if !hand_ids.contains(&game_id.0) { continue; }
        let Some(new_slot) = cv.players[viewer].hand.iter().position(|c| c.id() == game_id.0) else { continue };
        let target = hand_card_transform(viewer, viewer, n_seats, new_slot, hand_total);
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
        if !all_bf_ids.contains(&game_id.0) { continue; }
        let is_land = bf.is_land;

        let game_tapped = cv.battlefield.iter().find(|c| c.id == game_id.0).is_some_and(|c| c.tapped);
        let visual_tapped = tap_state.is_some_and(|ts| ts.tapped);

        let target = if is_land {
            let Some(t) = land_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0) else { continue };
            let base_rot = t.rotation;
            let tapped_rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * base_rot;
            let rot = if game_tapped { tapped_rot } else { base_rot };
            Transform { translation: t.translation, rotation: rot, scale: t.scale }
        } else {
            let row_total = creature_count(owner.0);
            let Some(slot) = bf_row_slot(&cv.battlefield, owner.0, game_id.0, false) else { continue };
            bf_card_transform(owner.0, viewer, n_seats, slot, row_total, false, game_tapped)
        };

        if game_tapped != visual_tapped {
            let (untapped_rot, tapped_rot) = if is_land {
                let land_base = land_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0)
                    .map(|t| t.rotation).unwrap_or(target.rotation);
                let land_tapped = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * land_base;
                (land_base, land_tapped)
            } else {
                let row_total = creature_count(owner.0);
                let slot = bf_row_slot(&cv.battlefield, owner.0, game_id.0, false).unwrap_or(0);
                (
                    bf_card_transform(owner.0, viewer, n_seats, slot, row_total, false, false).rotation,
                    bf_card_transform(owner.0, viewer, n_seats, slot, row_total, false, true).rotation,
                )
            };
            let (start, end) = if game_tapped { (untapped_rot, tapped_rot) } else { (tapped_rot, untapped_rot) };
            commands
                .entity(entity)
                .insert(Animating)
                .insert(TapAnimation { progress: 0.0, speed: 4.0, start_rotation: start, target_rotation: end })
                .insert(TapState { tapped: game_tapped });
        }

        commands.entity(entity).insert(CardHoverLift {
            current_lift: 0.0,
            target_lift: 0.0,
            base_translation: target.translation,
        });
    }

    // ── Despawn viewer deck cards no longer in hand ─────────────────────────
    for (entity, game_id, _deck_card, _transform) in &deck_cards {
        if !hand_ids.contains(&game_id.0) {
            commands.entity(entity).despawn();
        }
    }
}

/// Scan wire events for `TopCardRevealed` and arm the reveal popup if found.
fn check_reveal_wire(events: &[crabomination::net::GameEventWire], reveal: &mut RevealPopupState) {
    for ev in events {
        if let crabomination::net::GameEventWire::TopCardRevealed { player, card_name, .. } = ev {
            reveal.card_path = Some(crate::scryfall::card_asset_path(card_name));
            reveal.revealed_player = Some(*player);
        }
    }
}

// ── Auto-advance non-interactive steps ───────────────────────────────────────

pub fn auto_advance_p0(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut ff: ResMut<FastForward>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(outbox) = outbox else { return };
    if cv.game_over.is_some() { return; }
    // Any pending decision suspends normal step advancement. If it's our
    // decision, the dedicated decision UI submits the answer; if it's an
    // opponent's, we just wait. Spamming `PassPriority` every frame here
    // floods the server with `DecisionPending` rejections (most visibly
    // during mulligan).
    if cv.pending_decision.is_some() { return; }
    if cv.priority != cv.your_seat { return; }

    let your_seat = cv.your_seat;

    if ff.end_turn && cv.active_player != your_seat {
        ff.end_turn = false;
    }
    if ff.next_turn && cv.active_player == your_seat && cv.step == TurnStep::PreCombatMain {
        ff.next_turn = false;
        return;
    }

    // Don't auto-advance during interactive blocking when the viewer is
    // defending against any opponent's attack.
    if cv.step == TurnStep::DeclareBlockers && cv.active_player != your_seat {
        return;
    }

    let should_advance = ff.end_turn || ff.next_turn || matches!(
        cv.step,
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw
            | TurnStep::BeginCombat | TurnStep::CombatDamage
            | TurnStep::EndCombat | TurnStep::End | TurnStep::Cleanup
    ) || cv.active_player != your_seat;

    if should_advance {
        outbox.submit(GameAction::PassPriority);
    }
}

// ── Player 0 input ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_game_input(
    mut commands: Commands,
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut r: GameInputResources,
    server_events: Res<LatestServerEvents>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered_hand: Query<&GameCardId, (With<CardHovered>, With<HandCard>)>,
    hovered_bf: Query<(&GameCardId, &CardOwner), (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_target_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    valid_targets: Query<Entity, With<ValidTarget>>,
    btns: Res<ButtonState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let log = &mut *r.log;
    let targeting = &mut *r.targeting;
    let blocking = &mut *r.blocking;
    let reveal = &mut *r.reveal;
    let menu_state = &mut *r.menu_state;
    let ff = &mut *r.ff;
    let card_names = &mut *r.card_names;

    // Refresh the card-name lookup from the current view so the event
    // formatter can resolve any CardId it sees. Hand / battlefield /
    // graveyard / stack are all included; opponent face-down hand
    // entries stay anonymous.
    if let Some(cv) = view.0.as_ref() {
        for player in &cv.players {
            for h in &player.hand {
                if let crabomination::net::HandCardView::Known(k) = h {
                    card_names.by_id.insert(k.id, k.name.clone());
                }
            }
            for g in &player.graveyard {
                card_names.by_id.insert(g.id, g.name.clone());
            }
        }
        for c in &cv.battlefield {
            card_names.by_id.insert(c.id, c.name.clone());
        }
        for item in &cv.stack {
            if let crabomination::net::StackItemView::Known(k) = item {
                card_names.by_id.insert(k.source, k.name.clone());
            }
        }
    }

    // Update game log from server events.
    for ev in &server_events.0 {
        check_reveal_wire(std::slice::from_ref(ev), reveal);
        log.push(format_event(ev, card_names));
    }

    let Some(cv) = &view.0 else { return };
    let Some(outbox) = outbox else { return };
    let your_seat = cv.your_seat;

    // While a decision is pending for this viewer (mulligan, scry, search,
    // put-on-library, …), drop into decision-handling mode: the dedicated
    // decision UI systems own input — including 3D hand-card clicks for
    // PutOnLibrary — so we early-out here to avoid double-handling.
    if let Some(pd) = &cv.pending_decision
        && pd.acting_player == your_seat
    {
        return;
    }
    {

        // ── Blocking (defending against any opponent's attack) ──────────────
        if cv.step == TurnStep::DeclareBlockers && cv.active_player != your_seat && cv.priority == your_seat {
            let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
            if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
                blocking.selected_blocker = None;
                return;
            }
            if mouse.just_pressed(MouseButton::Left)
                && let Some((game_id, owner)) = hovered_bf.iter().next()
            {
                if owner.0 == your_seat {
                    let already_assigned = blocking.assignments.iter().any(|(b, _)| *b == game_id.0);
                    let is_creature = cv.battlefield.iter().any(|c| c.id == game_id.0 && !c.is_land());
                    if is_creature && !already_assigned {
                        blocking.selected_blocker = Some(game_id.0);
                    }
                } else if owner.0 != your_seat
                    && let Some(blocker_id) = blocking.selected_blocker
                {
                    let is_attacker = cv.battlefield.iter().any(|c| c.id == game_id.0 && c.attacking);
                    if is_attacker {
                        blocking.assignments.push((blocker_id, game_id.0));
                        blocking.selected_blocker = None;
                    }
                }
            }
            if pass {
                let assignments = std::mem::take(&mut blocking.assignments);
                blocking.selected_blocker = None;
                if !assignments.is_empty() {
                    outbox.submit(GameAction::DeclareBlockers(assignments));
                }
                outbox.submit(GameAction::PassPriority);
            }
            return;
        }

        if cv.game_over.is_some() || cv.priority != your_seat { return; }

        // ── Targeting mode ────────────────────────────────────────────────────
        if targeting.active {
            if mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
                cancel_targeting(&mut commands, targeting, &valid_targets);
                return;
            }
            if mouse.just_pressed(MouseButton::Left) {
                let is_ability_target = targeting.pending_ability_source.is_some();
                for (game_id, _owner) in &hovered_bf {
                    let target = Target::Permanent(game_id.0);
                    if is_ability_target {
                        if let (Some(src), Some(idx)) = (targeting.pending_ability_source, targeting.pending_ability_index) {
                            outbox.submit(GameAction::ActivateAbility { card_id: src, ability_index: idx, target: Some(target) });
                            cancel_targeting(&mut commands, targeting, &valid_targets);
                            return;
                        }
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        outbox.submit(GameAction::CastSpell { card_id: pending_id, target: Some(target), mode: None, x_value: None });
                        cancel_targeting(&mut commands, targeting, &valid_targets);
                        return;
                    }
                }
                for zone in &hovered_target_zone {
                    let target = Target::Player(zone.0);
                    if is_ability_target {
                        if let (Some(src), Some(idx)) = (targeting.pending_ability_source, targeting.pending_ability_index) {
                            outbox.submit(GameAction::ActivateAbility { card_id: src, ability_index: idx, target: Some(target) });
                            cancel_targeting(&mut commands, targeting, &valid_targets);
                            return;
                        }
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        outbox.submit(GameAction::CastSpell { card_id: pending_id, target: Some(target), mode: None, x_value: None });
                        cancel_targeting(&mut commands, targeting, &valid_targets);
                        return;
                    }
                }
            }
            return;
        }

        // ── Normal input ──────────────────────────────────────────────────────

        // Right-click P0 battlefield card → ability menu.
        // Right-click P0 hand card with an alt cost → alt-cast pitch modal.
        // Right-click P0 hand card with a back face (MDFC) → flip face.
        if mouse.just_pressed(MouseButton::Right) {
            if let Some(game_id) = hovered_hand.iter().next() {
                let card_id = game_id.0;
                let known: Option<crabomination::net::KnownCard> =
                    cv.players[your_seat].hand.iter().find_map(|h| match h {
                        crabomination::net::HandCardView::Known(k) if k.id == card_id => {
                            Some(k.clone())
                        }
                        _ => None,
                    });
                if let Some(k) = known {
                    if k.has_alternative_cost {
                        r.alt_cast.pending = Some(card_id);
                    } else if k.back_face_name.is_some() {
                        // Toggle MDFC flipped state. Visuals (texture swap)
                        // are reconciled in `sync_flipped_hand_cards`.
                        if !r.flipped_hand.flipped.insert(card_id) {
                            r.flipped_hand.flipped.remove(&card_id);
                        }
                    }
                }
            } else if let Some((game_id, owner)) = hovered_bf.iter().next() {
                if owner.0 == your_seat {
                    let card_id = game_id.0;
                    let has_non_mana = cv.battlefield.iter().find(|c| c.id == card_id)
                        .is_some_and(|c| c.abilities.iter().any(|a| !a.is_mana));
                    if has_non_mana {
                        menu_state.card_id = Some(card_id);
                        menu_state.spawn_pos = windows.single().ok()
                            .and_then(|w| w.cursor_position())
                            .unwrap_or(Vec2::new(400.0, 300.0));
                    } else {
                        menu_state.card_id = None;
                    }
                } else {
                    menu_state.card_id = None;
                }
            } else {
                menu_state.card_id = None;
            }
        }

        if mouse.just_pressed(MouseButton::Left) && menu_state.card_id.is_some() {
            menu_state.card_id = None;
        }

        let in_main = matches!(cv.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain);
        if in_main && mouse.just_pressed(MouseButton::Left)
            && let Some(game_id) = hovered_hand.iter().next()
        {
            // Find card details from the view.
            if let Some(card) = cv.players[your_seat].hand.iter().find_map(|h| {
                if let crabomination::net::HandCardView::Known(k) = h && k.id == game_id.0 { return Some(k.clone()); } None
            }) {
                use crabomination::card::CardType;
                if card.card_types.contains(&CardType::Land) {
                    if r.flipped_hand.flipped.contains(&card.id) {
                        outbox.submit(GameAction::PlayLandBack(card.id));
                    } else {
                        outbox.submit(GameAction::PlayLand(card.id));
                    }
                } else if card.needs_target {
                    targeting.active = true;
                    targeting.pending_card_id = Some(card.id);
                } else {
                    outbox.submit(GameAction::CastSpell { card_id: card.id, target: None, mode: None, x_value: None });
                }
            }
        }

        // Attack All (A) — default target is the next opponent (first alive
        // seat after `your_seat`). Multi-defender target picking has no UI yet.
        let attack = keyboard.just_pressed(KeyCode::KeyA) || btns.attack;
        if attack && cv.step == TurnStep::DeclareAttackers {
            use crabomination::game::{Attack, AttackTarget};
            let next_opp = cv
                .players
                .iter()
                .map(|p| p.seat)
                .find(|s| *s != your_seat)
                .unwrap_or(your_seat);
            let attacks: Vec<Attack> = cv
                .battlefield
                .iter()
                .filter(|c| c.owner == your_seat && !c.is_land() && !c.tapped && !c.summoning_sick)
                .map(|c| Attack {
                    attacker: c.id,
                    target: AttackTarget::Player(next_opp),
                })
                .collect();
            outbox.submit(GameAction::DeclareAttackers(attacks));
        }

        // Pass Priority (Space)
        let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
        if pass {
            outbox.submit(GameAction::PassPriority);
        }

        // End Turn (E) — set fast-forward to skip to opponent's turn
        let end_turn = keyboard.just_pressed(KeyCode::KeyE) || btns.end_turn;
        if end_turn {
            ff.end_turn = true;
            outbox.submit(GameAction::PassPriority);
        }

        // Next Turn (N) — fast-forward through opponent's turn to our next main
        let next_turn = keyboard.just_pressed(KeyCode::KeyN) || btns.next_turn;
        if next_turn {
            ff.next_turn = true;
            outbox.submit(GameAction::PassPriority);
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
    targeting.pending_ability_source = None;
    targeting.pending_ability_index = None;
    for entity in valid_targets.iter() {
        commands.entity(entity).remove::<ValidTarget>();
    }
}

/// Spawn or despawn the floating ability context menu based on `AbilityMenuState`.
pub fn spawn_ability_menu(
    mut commands: Commands,
    view: Res<CurrentView>,
    menu_state: Res<AbilityMenuState>,
    existing: Query<Entity, With<AbilityMenu>>,
) {
    if !menu_state.is_changed() { return; }

    for e in &existing {
        commands.entity(e).despawn();
    }

    let Some(card_id) = menu_state.card_id else { return };
    let Some(cv) = view.0.as_ref() else { return };
    let Some(pv) = cv.battlefield.iter().find(|p| p.id == card_id) else { return };

    let abilities: Vec<(usize, String)> = pv.abilities.iter()
        .filter(|a| !a.is_mana)
        .map(|a| (a.index, format!("{}: {}", a.cost_label, a.effect_label)))
        .collect();
    if abilities.is_empty() { return; }
    let card_name = pv.name.clone();

    let pos = menu_state.spawn_pos;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x),
                top: Val::Px(pos.y),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.97)),
            AbilityMenu,
        ))
        .with_children(|menu| {
            menu.spawn((
                Text::new(card_name),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgba(0.8, 0.8, 1.0, 1.0)),
            ));
            for (ability_index, label) in abilities {
                menu.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.20, 0.22, 0.32, 0.95)),
                    AbilityMenuItem { card_id, ability_index },
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}

/// Handle clicks on ability menu items.
pub fn handle_ability_menu(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut targeting: ResMut<TargetingState>,
    mut menu_state: ResMut<AbilityMenuState>,
    query: Query<(&Interaction, &AbilityMenuItem), Changed<Interaction>>,
) {
    for (interaction, item) in &query {
        if *interaction != Interaction::Pressed { continue; }

        let Some(cv) = &view.0 else { menu_state.card_id = None; continue };
        let Some(pv) = cv.battlefield.iter().find(|p| p.id == item.card_id) else { menu_state.card_id = None; continue };
        let Some(av) = pv.abilities.iter().find(|a| a.index == item.ability_index) else { menu_state.card_id = None; continue };

        if av.needs_target {
            targeting.active = true;
            targeting.pending_card_id = None;
            targeting.pending_ability_source = Some(item.card_id);
            targeting.pending_ability_index = Some(item.ability_index);
        } else if let Some(ob) = &outbox {
            ob.submit(GameAction::ActivateAbility {
                card_id: item.card_id,
                ability_index: item.ability_index,
                target: None,
            });
        }

        menu_state.card_id = None;
    }
}


// ── Alt-cast (pitch / evoke) modal ───────────────────────────────────────────

#[derive(Component)]
pub struct AltCastModal;

#[derive(Component)]
pub struct AltCastPitchButton {
    pub spell: CardId,
    pub pitch: CardId,
}

#[derive(Component)]
pub struct AltCastCancelButton;

/// Spawn or despawn the alt-cast pitch picker based on `AltCastState`.
pub fn spawn_alt_cast_modal(
    mut commands: Commands,
    view: Res<CurrentView>,
    state: Res<crate::game::AltCastState>,
    existing: Query<Entity, With<AltCastModal>>,
) {
    let want_open = state.pending.is_some();
    let is_open = !existing.is_empty();
    if !state.is_changed() && want_open == is_open {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    let Some(spell_id) = state.pending else { return };
    let Some(cv) = &view.0 else { return };

    // Collect candidate pitch cards: every other Known card in the viewer's
    // hand. The engine validates the filter and returns InvalidPitchCard if
    // wrong color; we render every card so the player sees the full hand.
    let candidates: Vec<(CardId, String)> = cv
        .players
        .get(cv.your_seat)
        .map(|p| {
            p.hand
                .iter()
                .filter_map(|h| match h {
                    crabomination::net::HandCardView::Known(k) if k.id != spell_id => {
                        Some((k.id, k.name.clone()))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
            AltCastModal,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    min_width: Val::Px(360.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.97)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Cast for alternative cost — pick a card to exile:"),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                if candidates.is_empty() {
                    panel.spawn((
                        Text::new("(no other cards in hand to pitch)"),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));
                }
                for (pid, name) in candidates {
                    panel
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.20, 0.30, 0.50, 1.0)),
                            AltCastPitchButton { spell: spell_id, pitch: pid },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(name),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(Color::WHITE),
                                bevy::picking::Pickable::IGNORE,
                            ));
                        });
                }
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.30, 0.10, 0.10, 1.0)),
                        AltCastCancelButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Cancel"),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(Color::WHITE),
                            bevy::picking::Pickable::IGNORE,
                        ));
                    });
            });
        });
}

/// Click on a pitch button → submit `CastSpellAlternative`. Click cancel →
/// clear the pending alt-cast.
pub fn handle_alt_cast_buttons(
    mut state: ResMut<crate::game::AltCastState>,
    outbox: Option<Res<NetOutbox>>,
    pitch_q: Query<(&Interaction, &AltCastPitchButton), Changed<Interaction>>,
    cancel_q: Query<&Interaction, (Changed<Interaction>, With<AltCastCancelButton>)>,
) {
    if cancel_q.iter().any(|i| *i == Interaction::Pressed) {
        state.pending = None;
        return;
    }
    for (interaction, btn) in &pitch_q {
        if *interaction == Interaction::Pressed
            && let Some(outbox) = &outbox
        {
            outbox.submit(GameAction::CastSpellAlternative {
                card_id: btn.spell,
                pitch_card: Some(btn.pitch),
                target: None,
                mode: None,
                x_value: None,
            });
            state.pending = None;
            return;
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

/// When RevealPopupState activates, start a RevealPeekAnimation on the top
/// deck card of the revealed player's pile.
#[allow(clippy::type_complexity)]
pub fn trigger_reveal_animation(
    mut reveal: ResMut<RevealPopupState>,
    deck_cards: Query<
        (Entity, &DeckPile, &Transform),
        (Without<RevealPeekAnimation>, Without<Animating>),
    >,
    mut commands: Commands,
) {
    let Some(player) = reveal.revealed_player.take() else {
        return;
    };

    // Top of the pile = highest index for that owner. face_up = 180° around
    // the card's local Y axis so the flip rotates along the long edge.
    let top = deck_cards
        .iter()
        .filter(|(_, dp, _)| dp.owner == player)
        .max_by_key(|(_, dp, _)| dp.index);
    if let Some((entity, _, transform)) = top {
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

// ── MDFC flip sync ────────────────────────────────────────────────────────────

/// Reconcile each viewer hand card's persistent flip state
/// (`FlippedFace` marker on the entity) against the user's intent
/// (`FlippedHandCards.flipped`). When they disagree, attach a 180°
/// `MdfcFlipAnimation` and toggle the marker. Both card faces are
/// already painted with their proper Scryfall images at spawn time, so
/// the rotation alone reveals the alternate face — no material swap.
/// Also drops stale flip entries when cards leave the hand.
#[allow(clippy::type_complexity)]
pub fn sync_flipped_hand_cards(
    mut commands: Commands,
    cv: Res<CurrentView>,
    mut flipped: ResMut<crate::game::FlippedHandCards>,
    hand_cards: Query<
        (
            Entity,
            &GameCardId,
            &Transform,
            Option<&crate::card::FlippedFace>,
        ),
        (With<HandCard>, Without<crate::card::MdfcFlipAnimation>),
    >,
) {
    let Some(view) = cv.0.as_ref() else { return };
    let viewer = view.your_seat;
    if viewer >= view.players.len() { return; }

    // Drop flips for cards that are no longer in the viewer's hand.
    let in_hand: HashSet<CardId> = view.players[viewer]
        .hand
        .iter()
        .map(|h| h.id())
        .collect();
    flipped.flipped.retain(|id| in_hand.contains(id));

    for (entity, game_id, transform, marker) in &hand_cards {
        let card_id = game_id.0;
        let should_be_flipped = flipped.flipped.contains(&card_id);
        let is_flipped = marker.is_some();
        if should_be_flipped == is_flipped {
            continue;
        }
        commands.entity(entity).insert(crate::card::MdfcFlipAnimation {
            progress: 0.0,
            speed: 2.5,
            start_rotation: transform.rotation,
        });
        if should_be_flipped {
            commands.entity(entity).insert(crate::card::FlippedFace);
        } else {
            commands.entity(entity).remove::<crate::card::FlippedFace>();
        }
    }
}
