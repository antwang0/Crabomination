use bevy::prelude::*;

use super::components::{BattlefieldCard, CardHoverLift, CardHovered, HandCard, BF_HOVER_LIFT, HOVER_LIFT_AMOUNT};

pub fn on_card_over(
    ev: On<Pointer<Over>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
    mut lifts: Query<(&Transform, &mut CardHoverLift, Option<&HandCard>, Option<&BattlefieldCard>)>,
) {
    let hit_entity = ev.entity;
    if let Ok(child_of) = parents.get(hit_entity) {
        let parent = child_of.parent();
        if let Ok(mut ec) = commands.get_entity(parent) {
            ec.insert(CardHovered);
        }
        if let Ok((transform, mut lift, hand_card, battlefield)) = lifts.get_mut(parent) {
            let target = if hand_card.is_some() {
                HOVER_LIFT_AMOUNT
            } else if battlefield.is_some() {
                BF_HOVER_LIFT
            } else {
                return;
            };
            lift.base_translation = transform.translation - Vec3::Y * lift.current_lift;
            lift.target_lift = target;
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
