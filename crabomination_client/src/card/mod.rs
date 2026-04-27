mod components;
pub mod layout;
mod mesh;
mod observers;
pub mod spawn;

pub use components::{
    Animating, BattlefieldCard, Card, CardBorderHighlight, CardFlipAnimation, CardFrontTexture,
    CardHighlightAssets, CardHoverLift, CardHovered, CardMeshAssets, CardOwner, DeckCard,
    DeckPile, DeckShuffleAnimation, DrawCardAnimation, FlippedFace, GameCardId, GraveyardPile,
    HandCard, HandSlideAnimation, MdfcFlipAnimation, OpponentHandCard, PileHovered,
    PlayCardAnimation, PlayerTargetZone, RevealPeekAnimation, ReturnToDeckAnimation,
    SendToGraveyardAnimation, ShufflePhase, StackCard, TapAnimation, TapState, ValidTarget,
    CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, DECK_CARD_Y_STEP, HOVER_LIFT_SPEED,
};
// `FrontFaceMesh` / `BackFaceMesh` are spawn-internal markers not used
// outside `card::spawn`, so they're not re-exported.
pub use layout::{
    back_face_rotation, bf_card_transform, deck_position, graveyard_position,
    hand_card_transform, land_card_transform,
};
pub use mesh::{create_border_mesh, create_rounded_rect_mesh, BORDER_WIDTH, CORNER_RADIUS};
pub use spawn::{card_back_face_material, card_front_material, init_shared_assets, spawn_single_card};
