use bevy::prelude::*;
use crabomination::card::CardId;

pub const CARD_WIDTH: f32 = 3.0;
pub const CARD_HEIGHT: f32 = CARD_WIDTH * 88.0 / 63.0;
pub const CARD_THICKNESS: f32 = 0.02;

pub const DECK_CARD_Y_STEP: f32 = CARD_THICKNESS * 1.5;

pub const HOVER_LIFT_AMOUNT: f32 = 0.6;
pub const HOVER_LIFT_SPEED: f32 = 8.0;

/// Resolution-driven scale factor for the viewer's hand cards. At 4K
/// (≥1440 logical pixels tall) hands render at their original world
/// size; on lower-resolution displays each card is enlarged to keep
/// the on-screen pixel size roughly constant. Only the viewer's own
/// hand is zoomed — opponent hands are already face-down, so their
/// size doesn't affect readability.
///
/// Updated by `update_hand_zoom_from_window` whenever the window
/// changes size.
#[derive(Resource, Clone, Copy, Debug)]
pub struct HandZoom(pub f32);

impl Default for HandZoom {
    fn default() -> Self {
        Self(1.0)
    }
}

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

/// Marker for a card sitting in a player's command zone (Commander
/// commanders, Conspiracies). Rendered as a fixed-position pile
/// near the seat's deck/graveyard, face-up; clicking the viewer's
/// own command zone routes through `GameAction::CastFromCommandZone`.
/// `owner` is the seat the zone belongs to; `slot` is the stack
/// index within that zone.
#[derive(Component)]
pub struct CommandZoneCard {
    pub owner: usize,
    #[allow(dead_code)]
    pub slot: usize,
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
    /// Token permanents despawn (instead of flying to a graveyard pile)
    /// when they leave the battlefield. The visual layer mirrors the
    /// MTG "tokens cease to exist" rule via state-based actions but
    /// needs to know after the engine has already removed the card
    /// from `cv.battlefield`, hence this flag is mirrored on the
    /// 3-D entity.
    pub is_token: bool,
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

/// Per-seat marker tagging a player as a targetable entity. Now a bare,
/// invisible entity — the visible player representation is the 2-D HUD
/// panel. It exists so keyboard targeting has something to stamp
/// `CardHovered` onto: `kb_cursor::apply_keyboard_selection` places the
/// selection here and `handle_game_input`'s `hovered_target_zone` query
/// reads it to submit the Player target. Mouse clicks are submitted from
/// the 2-D panel instead (`poll_player_chip_clicks`).
#[derive(Component)]
pub struct PlayerTargetZone(pub usize);

/// Combat-animation offset for a battlefield creature. `progress`
/// interpolates 0..1 along `target_offset` (the world-space vector from
/// the card's base position to whatever it's lunging at — defending
/// player's icon for attackers, the attacker's position for blockers).
/// `update_combat_lurch_targets` writes the desired `target_progress`
/// each frame from the view's combat step; `animate_combat_lurch`
/// lerps `progress` toward it and writes `target_offset * progress`
/// onto the card's transform. Removed once both are ~0.
#[derive(Component, Default, Clone, Copy)]
pub struct CombatLurch {
    pub progress: f32,
    pub target_progress: f32,
    pub target_offset: Vec3,
}

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

/// Animates a permanent flying back to its owner's hand (e.g. after
/// Unsummon, Boomerang). On completion the entity either restores its
/// `HandCard` slot (viewer's hand — keep it visible face-up) or
/// despawns (opponent's hand — the regular `OpponentHandCard` spawn
/// loop will replace it with a face-down placeholder).
#[derive(Component)]
pub struct ReturnToHandAnimation {
    pub progress: f32,
    pub speed: f32,
    pub start_translation: Vec3,
    pub start_rotation: Quat,
    pub target_translation: Vec3,
    pub target_rotation: Quat,
    /// True when bouncing into the viewer's own hand. Drives whether
    /// the entity is converted to a HandCard on completion (true) or
    /// despawned and re-spawned face-down by the opponent-hand sync (false).
    pub to_viewer: bool,
    /// Hand slot to restore on completion (only meaningful when `to_viewer`).
    pub target_slot: usize,
    /// Seat that owns the destination hand. Used by the opponent-hand
    /// reconciliation pass to count an in-flight bounce as one of that
    /// seat's hand visuals — otherwise the reconciler spawns a
    /// duplicate face-down placeholder while the bounce is mid-arc.
    pub target_owner: usize,
    /// Scale to snap to on completion. Battlefield cards are scale 1.0
    /// but the viewer's hand may be enlarged on low-res displays
    /// (resolution-driven hand zoom), so the bounce target needs to
    /// match the rest of the destination hand. Opponent targets stay
    /// at 1.0.
    pub target_scale: f32,
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
    /// Captured at animation start so the system can lerp scale back
    /// to 1.0 over the flight; a hand card may be zoomed >1 by the
    /// resolution-driven hand-zoom system.
    pub start_scale: f32,
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
    /// Green border for a hand card castable *for its normal cost* now.
    /// Distinct from the gold `border_material` (hover / valid-target /
    /// put-on-library) so a castable card reads differently from a
    /// targeting candidate.
    pub castable_material: Handle<StandardMaterial>,
    /// Cyan border for a hand card playable only via an *alternative*
    /// path (Dash, pitch/exile, etc.) — not affordable for its normal
    /// cost. Signals "you can play this, but it costs something other
    /// than just mana" vs. the plain-green hard-castable.
    pub alt_castable_material: Handle<StandardMaterial>,
    /// Red border for battlefield creatures the current combat preview
    /// projects to die. Distinct from gold/green so a doomed creature
    /// reads as a warning, not a candidate.
    pub dying_material: Handle<StandardMaterial>,
}

/// Links a viewer hand card to its spawned "playable now" border meshes
/// (back, front). Mirrors [`CardBorderHighlight`] but driven by
/// `update_castable_highlights` off the view's castable/alt-cost sets
/// rather than by pointer hover. `alt` records which colour is currently
/// applied (green hard-castable vs. cyan alt-cost) so the system can
/// re-tint a card whose category changes without leaking borders.
#[derive(Component)]
pub struct CastableHighlight {
    pub back: Entity,
    pub front: Entity,
    pub alt: bool,
}

/// Links a battlefield card to its spawned red "will die in combat" border
/// meshes (back, front). Driven by `update_dying_highlights` off the
/// view's `combat_preview.dying_creatures` set.
#[derive(Component)]
pub struct DyingHighlight {
    pub back: Entity,
    pub front: Entity,
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
