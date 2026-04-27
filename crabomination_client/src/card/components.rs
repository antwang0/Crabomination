use bevy::prelude::*;
use crabomination::card::CardId;

pub const CARD_WIDTH: f32 = 3.0;
pub const CARD_HEIGHT: f32 = CARD_WIDTH * 88.0 / 63.0;
pub const CARD_THICKNESS: f32 = 0.02;

pub const DECK_CARD_Y_STEP: f32 = CARD_THICKNESS * 1.5;

pub const HOVER_LIFT_AMOUNT: f32 = 0.6;
pub const HOVER_LIFT_SPEED: f32 = 8.0;

/// Marker for any card entity.
#[derive(Component)]
pub struct Card;

/// Marker present on any entity that is currently running an animation.
/// Prevents a second animation from being inserted before the first completes.
#[derive(Component)]
pub struct Animating;

/// Marker: the pointer is currently hovering over this card (or one of its children).
#[derive(Component)]
pub struct CardHovered;

/// Links a card to its spawned border highlight entities.
#[derive(Component)]
pub struct CardBorderHighlight(pub Entity, pub Entity);

/// Stores the front texture path so the peek popup can load it.
#[derive(Component)]
pub struct CardFrontTexture(pub String);

/// Marker for the front-face child mesh of a card entity. Used by systems
/// that need to swap the front material at runtime — e.g. the MDFC flip
/// (`sync_flipped_hand_cards`) replaces this child's material with the
/// back-face texture when the player flips a hand card.
#[derive(Component)]
pub struct FrontFaceMesh;

/// Deferred marker: on the next frame, walk this entity's children,
/// find the `FrontFaceMesh` child, replace its `MeshMaterial3d` with
/// `new_front`, update the parent's `CardFrontTexture` to `new_path`,
/// then remove this component. Used by the hand→battlefield transition
/// for flipped MDFCs so the played (back) face shows on the bf via the
/// front-child mesh under standard orientation.
#[derive(Component)]
pub struct SwapFrontMaterial {
    pub new_front: Handle<StandardMaterial>,
    pub new_path: String,
}

/// Marker for the back-face child mesh of a card entity. For MDFC hand
/// cards the back-child is painted with the back-face's Scryfall image
/// at spawn time so flipping the card 180° actually reveals the
/// alternate face (instead of the cardback).
#[derive(Component)]
pub struct BackFaceMesh;

/// Marker tracking the persistent flipped state of an MDFC hand card.
/// Inserted when a flip animation starts toward the back face; removed
/// when it animates back to the front. `sync_flipped_hand_cards`
/// reconciles this against `FlippedHandCards.flipped` and attaches an
/// `MdfcFlipAnimation` whenever they disagree.
#[derive(Component)]
pub struct FlippedFace;

/// 180° flip animation for MDFC right-clicks. Rotates the parent around
/// its local Y axis from `start_rotation` to `start_rotation *
/// Quat::from_rotation_y(PI)` over `progress: 0.0..1.0`. Both card
/// faces are painted with proper Scryfall images, so the rotation by
/// itself reveals the alternate face — no mid-animation material swap
/// needed.
#[derive(Component)]
pub struct MdfcFlipAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_rotation: Quat,
}

/// Tracks an in-progress flip animation. `progress` goes from 0.0 to 1.0.
#[derive(Component)]
pub struct CardFlipAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_rotation: Quat,
    pub end_rotation: Quat,
    pub start_y: f32,
}

/// Marks a card as belonging to the player's hand.
#[derive(Component)]
pub struct HandCard {
    pub slot: usize,
}

/// Smoothly slides a settled hand card from its current position to a new slot position.
#[derive(Component)]
pub struct HandSlideAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_translation: Vec3,
    pub target_translation: Vec3,
    pub target_rotation: Quat,
}

/// In-progress draw-card animation: moves a card from the deck to a hand slot.
#[derive(Component)]
pub struct DrawCardAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_translation: Vec3,
    pub start_rotation: Quat,
    pub target_translation: Vec3,
    pub target_rotation: Quat,
}

/// Smooth hover-lift animation for cards.
#[derive(Component)]
pub struct CardHoverLift {
    pub current_lift: f32,
    pub target_lift: f32,
    /// Canonical resting position (no lift applied).
    pub base_translation: Vec3,
}

/// Marker for cards in the deck pile.
#[derive(Component)]
pub struct DeckCard {
    pub index: usize,
}

/// Marker for an opponent's hand card visual (face-down, count-synced).
/// `owner` is the seat that owns this card; `slot` is its position in the
/// owner's hand fan.
#[derive(Component)]
pub struct OpponentHandCard {
    pub owner: usize,
    pub slot: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ShufflePhase {
    Spread,   // Cards cascade out into a vertical column
    Shuffle,  // Cards slide to their new positions within the spread
    Collapse, // Cards fly back to a stack in the new order
}

/// Three-phase shuffle animation state for each deck card.
#[derive(Component)]
pub struct DeckShuffleAnimation {
    pub phase: ShufflePhase,
    pub phase_timer: f32,
    /// Where this card spreads to in the waterfall column.
    pub spread_target: Vec3,
    /// Where this card ends up in the spread after shuffling.
    pub shuffled_spread_target: Vec3,
    /// X bulge for the arc during the shuffle phase (positive = right, negative = left).
    pub shuffle_arc_x: f32,
    /// Final stack position after collapse.
    pub restack_target: Vec3,
    pub new_index: usize,
    /// Stagger delay for this card's spread movement.
    pub spread_delay: f32,
    /// Total time to wait in the Spread phase before transitioning to Shuffle.
    /// All cards share this value so they enter Shuffle simultaneously.
    pub spread_wait: f32,
    /// Stagger delay for the collapse phase.
    pub collapse_delay: f32,
    pub phase_start_translation: Vec3,
    pub phase_start_rotation: Quat,
}

/// Links a visual card entity to a game-engine CardId.
#[derive(Component, Clone, Copy)]
pub struct GameCardId(pub CardId);

/// Tracks which player owns this visual card entity.
#[derive(Component, Clone, Copy)]
pub struct CardOwner(pub usize);

/// Marks a card as on the battlefield, with row information.
#[derive(Component)]
pub struct BattlefieldCard {
    pub is_land: bool,
}

/// Tracks a card's visual tapped state for animation.
#[derive(Component)]
pub struct TapState {
    pub tapped: bool,
}

/// Animates a card tapping/untapping (90° rotation).
#[derive(Component)]
pub struct TapAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_rotation: Quat,
    pub target_rotation: Quat,
}

/// Marker for a graveyard pile visual entity.
#[derive(Component)]
pub struct GraveyardPile {
    pub owner: usize,
}

/// Marker added to pile entities (deck/graveyard) while the pointer is over them.
#[derive(Component)]
pub struct PileHovered;

/// Visual entity for one card in a player's face-down deck pile.
/// `owner` identifies the seat; `index` is the card's height in the stack.
#[derive(Component)]
pub struct DeckPile {
    pub owner: usize,
    pub index: usize,
}

/// Marker for valid target entities during targeting mode.
#[derive(Component)]
pub struct ValidTarget;

/// Clickable zone representing a player as a target.
#[derive(Component)]
pub struct PlayerTargetZone(pub usize);

/// Animates a card flying to the graveyard pile; despawns the entity on completion.
#[derive(Component)]
pub struct SendToGraveyardAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_translation: Vec3,
    pub start_rotation: Quat,
    pub target_translation: Vec3,
    pub target_rotation: Quat,
    /// Which player's graveyard this card is headed to.
    pub owner: usize,
}

/// Animates a hand card back to the deck position during a mulligan.
/// On completion the entity is converted from HandCard to DeckCard.
#[derive(Component)]
pub struct ReturnToDeckAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_translation: Vec3,
    pub start_rotation: Quat,
    pub target_translation: Vec3,
    pub target_rotation: Quat,
}

/// Animates a card from hand to the battlefield.
#[derive(Component)]
pub struct PlayCardAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_translation: Vec3,
    pub start_rotation: Quat,
    pub target_translation: Vec3,
    pub target_rotation: Quat,
}

/// Three-phase peek animation for deck cards: flip face-up, hold briefly, flip back.
#[derive(Component)]
pub struct RevealPeekAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_rotation: Quat,
    pub face_up_rotation: Quat,
    pub start_y: f32,
}

/// Resource holding shared mesh/material handles for card highlighting.
#[derive(Resource)]
pub struct CardHighlightAssets {
    pub border_mesh: Handle<Mesh>,
    pub border_material: Handle<StandardMaterial>,
}

/// Resource holding shared mesh/material handles for dynamically spawning cards.
#[derive(Resource)]
pub struct CardMeshAssets {
    pub card_mesh: Handle<Mesh>,
    pub back_material: Handle<StandardMaterial>,
}

/// Marker for a card entity that is currently on the stack (cast but not yet resolved).
#[derive(Component)]
pub struct StackCard;
