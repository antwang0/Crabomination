use bevy::prelude::*;

use super::components::{CardHoverLift, CardHovered, HandCard, HOVER_LIFT_AMOUNT};

/// Pointer-over observer for flat zone entities (PlayerTargetZone) that have no parent card.
pub fn on_zone_over(ev: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(ev.entity).insert(CardHovered);
}

pub fn on_zone_out(ev: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(ev.entity).remove::<CardHovered>();
}

pub fn on_card_over(
    ev: On<Pointer<Over>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
    mut lifts: Query<(&Transform, &mut CardHoverLift, Option<&HandCard>)>,
) {
    let hit_entity = ev.entity;
    if let Ok(child_of) = parents.get(hit_entity) {
        let parent = child_of.parent();
        if let Ok(mut ec) = commands.get_entity(parent) {
            ec.insert(CardHovered);
        }
        if let Ok((transform, mut lift, hand_card)) = lifts.get_mut(parent) {
            if hand_card.is_some() {
                lift.base_translation = transform.translation - Vec3::Y * lift.current_lift;
                lift.target_lift = HOVER_LIFT_AMOUNT;
            }
        }
    }
}

pub fn on_card_out(
    ev: On<Pointer<Out>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
    mut lifts: Query<&mut CardHoverLift>,
) {
    let hit_entity = ev.entity;
    if let Ok(child_of) = parents.get(hit_entity) {
        let parent = child_of.parent();
        if let Ok(mut ec) = commands.get_entity(parent) {
            ec.remove::<CardHovered>();
        }
        if let Ok(mut lift) = lifts.get_mut(parent) {
            lift.target_lift = 0.0;
        }
    }
}
