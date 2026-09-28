//! Whether a point on one card is hidden under another card.
//!
//! The card overlays (counter labels, the P/T and loyalty badges, keyword
//! strips, the corner badges) are screen-space UI, drawn over every 3-D
//! card. Where cards overlap — a wrapped creature row, a tapped card turned
//! under its neighbour, the hand fanned over the land row — a covered
//! card's overlays printed on the card on top of it. [`CardCover`] asks
//! whether another card's face lies between the camera and the point an
//! overlay hangs from, and the overlay hides while it does.

use bevy::prelude::*;

use super::{CARD_HEIGHT, CARD_WIDTH, GameCardId};

/// Every drawn card, for [`CardCover::new`].
pub type CoverQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static GlobalTransform, &'static InheritedVisibility), With<GameCardId>>;

/// The drawn cards as seen from the camera.
pub struct CardCover {
    eye: Vec3,
    /// Each card's entity and its world → card-local transform.
    cards: Vec<(Entity, bevy::math::Affine3A)>,
}

impl CardCover {
    pub fn new(eye: &GlobalTransform, cards: &CoverQuery) -> Self {
        Self::from_cards(
            eye.translation(),
            cards.iter().filter(|(_, _, seen)| seen.get()).map(|(e, t, _)| (e, *t)),
        )
    }

    fn from_cards(eye: Vec3, cards: impl Iterator<Item = (Entity, GlobalTransform)>) -> Self {
        let cards = cards.map(|(e, t)| (e, t.affine().inverse())).collect();
        Self { eye, cards }
    }

    /// Whether `world`, a point on or above card `own`, is behind another
    /// card's face as the camera sees it.
    pub fn hides(&self, own: Entity, world: Vec3) -> bool {
        self.cards.iter().any(|&(card, to_local)| {
            if card == own {
                return false;
            }
            let eye = to_local.transform_point3(self.eye);
            let at = to_local.transform_point3(world);
            // The sight line crosses the card's face plane between the eye
            // and the point...
            if (eye.z > 0.0) == (at.z > 0.0) {
                return false;
            }
            let hit = eye + (at - eye) * (eye.z / (eye.z - at.z));
            // ...inside the card.
            hit.x.abs() <= CARD_WIDTH / 2.0 && hit.y.abs() <= CARD_HEIGHT / 2.0
        })
    }

    /// [`hides`](Self::hides) for a point in `own`'s local space. An overlay
    /// hanging from a corner or an edge asks about a point a little inside
    /// it (`local * 0.85`), so a neighbour that only touches the edge
    /// doesn't hide it.
    pub fn hides_local(&self, own: Entity, card: &GlobalTransform, local: Vec3) -> bool {
        self.hides(own, card.transform_point(local))
    }

    /// The middle of the part of card `own` that shows, in its local space:
    /// its centre when nothing lies over it, the middle of what shows when
    /// another card overlaps it, and its centre again when it is covered
    /// entirely. An arrow meeting a card aims here, so it lands on that card
    /// rather than on the one lying over it.
    pub fn visible_centre(&self, own: Entity, card: &GlobalTransform) -> Vec3 {
        let grid = (-1..=1).flat_map(|i| (-2..=2).map(move |j| (i, j)));
        let shown: Vec<Vec3> = grid
            .map(|(i, j)| Vec3::new(i as f32 * CARD_WIDTH * 0.3, j as f32 * CARD_HEIGHT * 0.2, 0.0))
            .filter(|&local| !self.hides_local(own, card, local))
            .collect();
        if shown.is_empty() { Vec3::ZERO } else { shown.iter().sum::<Vec3>() / shown.len() as f32 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A card lying face up on the table at `at`, turned like the
    /// battlefield's.
    fn flat(at: Vec3) -> GlobalTransform {
        GlobalTransform::from(
            Transform::from_translation(at).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        )
    }

    #[test]
    fn a_card_on_top_hides_what_it_covers_and_nothing_else() {
        let eye = Vec3::new(0.0, 30.0, 20.0);
        let under = flat(Vec3::ZERO);
        let over = flat(Vec3::new(0.0, 0.03, 2.0));
        let (a, b) = (Entity::from_raw_u32(1).unwrap(), Entity::from_raw_u32(2).unwrap());
        let cover = CardCover::from_cards(eye, [(a, under), (b, over)].into_iter());
        // The lower card's bottom edge lies under the upper card...
        let bottom = Vec3::new(0.0, -CARD_HEIGHT / 2.0, 0.0) * 0.85;
        assert!(cover.hides_local(a, &under, bottom));
        // ...its top edge is clear of it...
        let top = Vec3::new(0.0, CARD_HEIGHT / 2.0, 0.0) * 0.85;
        assert!(!cover.hides_local(a, &under, top));
        // ...and nothing covers the card on top.
        assert!(!cover.hides_local(b, &over, bottom));
        assert!(!cover.hides_local(b, &over, top));
    }

    #[test]
    fn a_covered_card_is_met_in_the_part_that_shows() {
        let eye = Vec3::new(0.0, 30.0, 20.0);
        let under = flat(Vec3::ZERO);
        let over = flat(Vec3::new(0.0, 0.03, 2.0));
        let (a, b) = (Entity::from_raw_u32(1).unwrap(), Entity::from_raw_u32(2).unwrap());
        let cover = CardCover::from_cards(eye, [(a, under), (b, over)].into_iter());
        // The lower card shows only its top: it is met there, square across it.
        let spot = cover.visible_centre(a, &under);
        assert!(spot.y > CARD_HEIGHT * 0.2 && spot.x.abs() < 1e-5, "{spot}");
        // The card on top is met in its middle.
        assert!(cover.visible_centre(b, &over).length() < 1e-5);
    }
}
