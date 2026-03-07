use std::f32::consts::PI;

use bevy::prelude::*;

use crate::card::{
    hand_card_transform, CardFlipAnimation, CardHoverLift, DeckCard, DeckShuffleAnimation,
    DrawCardAnimation, HandCard, HandSlideAnimation, ShufflePhase, CARD_WIDTH, HOVER_LIFT_SPEED,
};

const SPREAD_DURATION: f32 = 0.1;
const COLLAPSE_DURATION: f32 = 0.1;
const SHUFFLE_DURATION: f32 = 0.35;

pub fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Smoothly lerp each card's Y toward its target (base + hover lift).
/// Skipped for any card that has another animation currently driving its transform.
pub fn animate_hover_lift(
    time: Res<Time>,
    mut cards: Query<
        (&mut Transform, &mut CardHoverLift),
        (
            Without<DrawCardAnimation>,
            Without<DeckShuffleAnimation>,
            Without<CardFlipAnimation>,
            Without<HandSlideAnimation>,
        ),
    >,
) {
    let dt = time.delta_secs();
    for (mut transform, mut lift) in &mut cards {
        let speed = HOVER_LIFT_SPEED * dt;
        lift.current_lift += (lift.target_lift - lift.current_lift) * speed.min(1.0);
        transform.translation = lift.base_translation + Vec3::Y * lift.current_lift;
    }
}

pub fn animate_flip(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(Entity, &mut Transform, &mut CardFlipAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * anim.speed;

        if anim.progress >= 1.0 {
            transform.rotation = anim.end_rotation;
            transform.translation.y = anim.start_y;
            commands.entity(entity).remove::<CardFlipAnimation>();
        } else {
            let t = ease_in_out(anim.progress);
            transform.rotation = anim.start_rotation.slerp(anim.end_rotation, t);

            let flip_angle = t * PI;
            let y_offset = (CARD_WIDTH / 2.0) * flip_angle.sin().abs();
            transform.translation.y = anim.start_y + y_offset;
        }
    }
}

pub fn animate_deck_shuffle(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut DeckShuffleAnimation,
        &mut DeckCard,
        &mut CardHoverLift,
    )>,
) {
    let dt = time.delta_secs();
    let flat_rotation =
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(PI);

    for (entity, mut transform, mut anim, mut deck_card, mut lift) in &mut cards {
        anim.phase_timer += dt;

        match anim.phase {
            ShufflePhase::Spread => {
                let effective = (anim.phase_timer - anim.spread_delay).max(0.0);
                let t = ease_in_out((effective / SPREAD_DURATION).clamp(0.0, 1.0));
                transform.translation = anim.phase_start_translation.lerp(anim.spread_target, t);
                transform.rotation = anim.phase_start_rotation.slerp(flat_rotation, t);

                if anim.phase_timer >= anim.spread_wait {
                    transform.translation = anim.spread_target;
                    transform.rotation = flat_rotation;
                    anim.phase = ShufflePhase::Shuffle;
                    anim.phase_timer = 0.0;
                    anim.phase_start_translation = transform.translation;
                    anim.phase_start_rotation = transform.rotation;
                }
            }
            ShufflePhase::Shuffle => {
                let t = ease_in_out((anim.phase_timer / SHUFFLE_DURATION).clamp(0.0, 1.0));
                let base = anim.phase_start_translation.lerp(anim.shuffled_spread_target, t);
                let arc_offset = (t * PI).sin() * anim.shuffle_arc_x;
                transform.translation = Vec3::new(base.x + arc_offset, base.y, base.z);

                if anim.phase_timer >= SHUFFLE_DURATION {
                    transform.translation = anim.shuffled_spread_target;
                    anim.phase = ShufflePhase::Collapse;
                    anim.phase_timer = 0.0;
                    anim.phase_start_translation = transform.translation;
                    anim.phase_start_rotation = transform.rotation;
                }
            }
            ShufflePhase::Collapse => {
                let effective = (anim.phase_timer - anim.collapse_delay).max(0.0);
                let t = ease_in_out((effective / COLLAPSE_DURATION).clamp(0.0, 1.0));
                transform.translation =
                    anim.phase_start_translation.lerp(anim.restack_target, t);
                transform.rotation = anim.phase_start_rotation.slerp(flat_rotation, t);

                let total_time = anim.collapse_delay + COLLAPSE_DURATION;
                if anim.phase_timer >= total_time {
                    transform.translation = anim.restack_target;
                    transform.rotation = flat_rotation;
                    deck_card.index = anim.new_index;
                    lift.base_translation = anim.restack_target;
                    lift.current_lift = 0.0;
                    commands.entity(entity).remove::<DeckShuffleAnimation>();
                }
            }
        }
    }
}

/// Animate cards flying from the deck to the hand.
pub fn animate_draw_card(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut DrawCardAnimation,
        &mut CardHoverLift,
        &HandCard,
    )>,
    all_hand_cards: Query<&HandCard>,
) {
    let total = all_hand_cards.iter().count();

    for (entity, mut transform, mut anim, mut lift, hand_card) in &mut cards {
        anim.progress += time.delta_secs() * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 3.0;

        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            let final_t = hand_card_transform(hand_card.slot, total);
            transform.translation = final_t.translation;
            transform.rotation = final_t.rotation;
            lift.base_translation = final_t.translation;
            lift.current_lift = 0.0;
            lift.target_lift = 0.0;
            commands.entity(entity).remove::<DrawCardAnimation>();
        }
    }
}

pub fn animate_hand_slide(
    mut commands: Commands,
    time: Res<Time>,
    mut cards: Query<(Entity, &mut Transform, &mut HandSlideAnimation, &mut CardHoverLift)>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += time.delta_secs() * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let pos = anim.start_translation.lerp(anim.target_translation, t);
        transform.translation = pos;
        transform.rotation = anim.target_rotation;

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            lift.base_translation = anim.target_translation;
            commands.entity(entity).remove::<HandSlideAnimation>();
        }
    }
}
