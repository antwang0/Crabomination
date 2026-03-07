use bevy::prelude::*;

pub const CARD_WIDTH: f32 = 3.0;
pub const CARD_HEIGHT: f32 = CARD_WIDTH * 88.0 / 63.0;
pub const CARD_THICKNESS: f32 = 0.02;

pub const DECK_SIZE: usize = 30;
pub const DECK_POSITION: Vec3 = Vec3::new(-6.0, 0.0, 0.0);
pub const DECK_CARD_Y_STEP: f32 = CARD_THICKNESS * 1.5;

pub const HOVER_LIFT_AMOUNT: f32 = 0.6;
pub const HOVER_LIFT_SPEED: f32 = 8.0;

/// Marker for any card entity.
#[derive(Component)]
pub struct Card;

/// Marker: the pointer is currently hovering over this card (or one of its children).
#[derive(Component)]
pub struct CardHovered;

/// Links a card to its spawned border highlight entities.
#[derive(Component)]
pub struct CardBorderHighlight(pub Entity, pub Entity);

/// Stores the front texture path so the peek popup can load it.
#[derive(Component)]
pub struct CardFrontTexture(pub String);

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

#[derive(Clone, Copy, PartialEq, Eq)]
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

/// Resource holding shared mesh/material handles for card highlighting.
#[derive(Resource)]
pub struct CardHighlightAssets {
    pub border_mesh: Handle<Mesh>,
    pub border_material: Handle<StandardMaterial>,
}
