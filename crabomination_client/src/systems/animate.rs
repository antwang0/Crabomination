use std::f32::consts::PI;

use bevy::prelude::*;

use crate::card::{
    hand_card_transform, Animating, CardFlipAnimation, CardHoverLift, DeckCard,
    DeckShuffleAnimation, DrawCardAnimation, HandCard, HandSlideAnimation, MdfcFlipAnimation,
    PlayCardAnimation, ReturnToDeckAnimation, ReturnToHandAnimation, RevealPeekAnimation,
    SendToGraveyardAnimation, ShufflePhase, TapAnimation, CARD_WIDTH, HOVER_LIFT_SPEED,
};
use crate::net_plugin::CurrentView;

/// Global animation playback speed multiplier (1.0 = normal, 2.0 = double speed, etc.).
#[derive(Resource)]
pub struct AnimationSpeed(pub f32);

impl Default for AnimationSpeed {
    fn default() -> Self { AnimationSpeed(1.0) }
}

const SPREAD_DURATION: f32 = 0.1;
const COLLAPSE_DURATION: f32 = 0.1;
const SHUFFLE_DURATION: f32 = 0.35;

pub fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Smoothly lerp each card's Y toward its target (base + hover lift).
/// Skipped for any card that has another animation currently driving its transform.
#[allow(clippy::type_complexity)]
pub fn animate_hover_lift(
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<
        (&mut Transform, &mut CardHoverLift),
        (
            Without<DrawCardAnimation>,
            Without<DeckShuffleAnimation>,
            Without<CardFlipAnimation>,
            Without<HandSlideAnimation>,
            Without<PlayCardAnimation>,
            Without<TapAnimation>,
            Without<SendToGraveyardAnimation>,
            Without<ReturnToDeckAnimation>,
            Without<ReturnToHandAnimation>,
            Without<RevealPeekAnimation>,
        ),
    >,
) {
    let dt = time.delta_secs() * speed.0;
    for (mut transform, mut lift) in &mut cards {
        let spd = HOVER_LIFT_SPEED * dt;
        lift.current_lift += (lift.target_lift - lift.current_lift) * spd.min(1.0);
        transform.translation = lift.base_translation + Vec3::Y * lift.current_lift;
    }
}

pub fn animate_flip(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut CardFlipAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;

        if anim.progress >= 1.0 {
            transform.rotation = anim.end_rotation;
            transform.translation.y = anim.start_y;
            commands.entity(entity).remove::<CardFlipAnimation>().remove::<Animating>();
        } else {
            let t = ease_in_out(anim.progress);
            transform.rotation = anim.start_rotation.slerp(anim.end_rotation, t);

            let flip_angle = t * PI;
            let y_offset = (CARD_WIDTH / 2.0) * flip_angle.sin().abs();
            transform.translation.y = anim.start_y + y_offset;
        }
    }
}

/// Drive the MDFC 180° flip: rotate the parent around its local Y axis
/// from `start_rotation` to `start_rotation * Quat::from_rotation_y(PI)`
/// over `progress: 0.0..1.0`. Both faces are pre-painted with their
/// proper Scryfall images, so this rotation alone reveals the alternate
/// face. After two flips (each +180°) the parent has rotated 360° and
/// is back at its original orientation.
#[allow(clippy::type_complexity)]
pub fn animate_mdfc_flip(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut MdfcFlipAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;
        let t = anim.progress.min(1.0);
        let eased = ease_in_out(t);
        let angle = eased * PI;
        transform.rotation = anim.start_rotation * Quat::from_rotation_y(angle);

        if anim.progress >= 1.0 {
            transform.rotation = anim.start_rotation * Quat::from_rotation_y(PI);
            commands.entity(entity).remove::<MdfcFlipAnimation>();
        }
    }
}

pub fn animate_deck_shuffle(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut DeckShuffleAnimation,
        &mut DeckCard,
        &mut CardHoverLift,
    )>,
) {
    let dt = time.delta_secs() * speed.0;
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
                    commands.entity(entity).remove::<DeckShuffleAnimation>().remove::<Animating>();
                }
            }
        }
    }
}

/// Animate cards flying from the deck to the hand. `HandCard` only ever
/// marks the viewer's own hand, so on completion we snap the card into the
/// viewer's hand fan (seat=viewer).
pub fn animate_draw_card(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    view: Res<CurrentView>,
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
    let (viewer, n_seats) = view
        .0
        .as_ref()
        .map(|cv| (cv.your_seat, cv.players.len()))
        .unwrap_or((0, 2));

    for (entity, mut transform, mut anim, mut lift, hand_card) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 3.0;

        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            let final_t = hand_card_transform(viewer, viewer, n_seats, hand_card.slot, total);
            transform.translation = final_t.translation;
            transform.rotation = final_t.rotation;
            lift.base_translation = final_t.translation;
            lift.current_lift = 0.0;
            lift.target_lift = 0.0;
            commands.entity(entity).remove::<DrawCardAnimation>().remove::<Animating>();
        }
    }
}

pub fn animate_hand_slide(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut HandSlideAnimation, &mut CardHoverLift)>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let pos = anim.start_translation.lerp(anim.target_translation, t);
        transform.translation = pos;
        transform.rotation = anim.target_rotation;

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            lift.base_translation = anim.target_translation;
            commands.entity(entity).remove::<HandSlideAnimation>().remove::<Animating>();
        }
    }
}

/// Animate a card moving from hand to battlefield.
pub fn animate_play_card(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut PlayCardAnimation,
        &mut CardHoverLift,
    )>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.0;

        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            transform.rotation = anim.target_rotation;
            lift.base_translation = anim.target_translation;
            lift.current_lift = 0.0;
            lift.target_lift = 0.0;
            commands.entity(entity).remove::<PlayCardAnimation>().remove::<Animating>();
        }
    }
}

/// Animate a card flying to the graveyard with a tumbling spin, then despawn it.
pub fn animate_send_to_graveyard(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut SendToGraveyardAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        // Arc: rise then fall as the card travels to the graveyard
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.0;
        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Animate a hand card flying back to the deck during a mulligan, then
/// despawn it. The face-down deck pile (`DeckPile`) is sized off
/// `library.size` and is updated by `sync_game_visuals`, so the visual deck
/// height already reflects the returned card; keeping a separate per-card
/// entity around would just clutter the deck pile with duplicates.
pub fn animate_return_to_deck(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut ReturnToDeckAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;

        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.0;
        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Animate a permanent flying back to its owner's hand. On completion
/// the entity either becomes a `HandCard` (viewer's bounce target —
/// keep it on screen face-up) or despawns (opponent's bounce target —
/// the next sync frame re-spawns it as a face-down `OpponentHandCard`).
pub fn animate_return_to_hand(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut ReturnToHandAnimation, Option<&mut CardHoverLift>)>,
) {
    for (entity, mut transform, mut anim, mut lift) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        let arc_y = (anim.progress.clamp(0.0, 1.0) * PI).sin() * 2.5;
        let mut pos = anim.start_translation.lerp(anim.target_translation, t);
        pos.y += arc_y;
        transform.translation = pos;
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            transform.translation = anim.target_translation;
            transform.rotation = anim.target_rotation;
            if anim.to_viewer {
                if let Some(ref mut l) = lift {
                    l.base_translation = anim.target_translation;
                    l.current_lift = 0.0;
                    l.target_lift = 0.0;
                }
                commands
                    .entity(entity)
                    .insert(HandCard { slot: anim.target_slot })
                    .remove::<ReturnToHandAnimation>()
                    .remove::<Animating>();
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Animate a card tapping (90°) or untapping.
pub fn animate_tap(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut TapAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;
        let t = ease_in_out(anim.progress.clamp(0.0, 1.0));
        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.progress >= 1.0 {
            transform.rotation = anim.target_rotation;
            commands.entity(entity).remove::<TapAnimation>().remove::<Animating>();
        }
    }
}

/// Three-phase animation: flip to face-up (0-0.35), hold (0.35-0.65), flip back (0.65-1.0).
pub fn animate_reveal_peek(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(Entity, &mut Transform, &mut RevealPeekAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut cards {
        anim.progress += time.delta_secs() * speed.0 * anim.speed;
        let p = anim.progress.clamp(0.0, 1.0);

        let (rot_t, y_t) = if p < 0.25 {
            // Phase 1: flip to face-up
            let t = ease_in_out(p / 0.25);
            (t, t)
        } else if p < 0.75 {
            // Phase 2: hold face-up
            (1.0_f32, 1.0_f32)
        } else {
            // Phase 3: flip back
            let t = ease_in_out((p - 0.75) / 0.25);
            (1.0 - t, 1.0 - t)
        };

        transform.rotation = anim.start_rotation.slerp(anim.face_up_rotation, rot_t);
        let lift = (y_t * PI).sin().abs() * (CARD_WIDTH / 2.0);
        transform.translation.y = anim.start_y + lift;

        if anim.progress >= 1.0 {
            transform.rotation = anim.start_rotation;
            transform.translation.y = anim.start_y;
            commands.entity(entity).remove::<RevealPeekAnimation>().remove::<Animating>();
        }
    }
}

/// Keyboard shortcuts to adjust animation playback speed.
/// [ = half speed, ] = double speed, Backslash = reset to 1×.
pub fn adjust_animation_speed(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut speed: ResMut<AnimationSpeed>,
) {
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        speed.0 = (speed.0 * 0.5).max(ANIM_SPEED_MIN);
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        speed.0 = (speed.0 * 2.0).min(ANIM_SPEED_MAX);
    }
    if keyboard.just_pressed(KeyCode::Backslash) {
        speed.0 = 1.0;
    }
}

/// User-facing range for the animation-speed slider in the HUD. The
/// keyboard shortcuts ([, ], \\) clamp to the same bounds.
pub const ANIM_SPEED_MIN: f32 = 0.1;
pub const ANIM_SPEED_MAX: f32 = 10.0;

// ── Animation queueing ───────────────────────────────────────────────────────

/// Queued follow-up animation for an entity that's currently animating.
/// `dispatch_animation_queue` peels the next item off the queue once the
/// `Animating` marker is removed, so animations chain end-to-end instead
/// of overlapping on the same card.
///
/// Only a small subset of animation types are queueable today (the ones
/// that actually conflict on the same entity in practice — chiefly tap,
/// play, hand-slide, and graveyard-flight). Adding a new variant is a
/// one-line append to this enum plus a one-line arm in the dispatcher.
pub enum QueuedAnim {
    /// Run a tap/untap rotation animation. The bool is the `TapState`
    /// the card should land in once the animation finishes.
    Tap {
        anim: TapAnimation,
        target_tapped: bool,
    },
    /// Run a card-flight (hand → battlefield, deck → exile, etc.).
    Play(PlayCardAnimation),
    /// Slide a hand card to a new fan slot.
    HandSlide(HandSlideAnimation),
    /// Fly the card to the graveyard (it self-despawns on completion).
    SendToGraveyard(SendToGraveyardAnimation),
}

#[derive(Component, Default)]
pub struct AnimQueue {
    pub queue: std::collections::VecDeque<QueuedAnim>,
    /// Seconds of forced idle time before popping the next animation.
    /// Set when the previous animation finishes; counts down (scaled by
    /// `AnimationSpeed`) inside `dispatch_animation_queue`. Zero on a
    /// freshly-built queue so the first animation starts immediately.
    pub delay_remaining: f32,
}

impl AnimQueue {
    pub fn from_one(anim: QueuedAnim) -> Self {
        let mut q = AnimQueue::default();
        q.queue.push_back(anim);
        q
    }
}

/// Inter-animation breather. Short enough to feel snappy at 1× and naturally
/// shrinks at higher speeds (the same `AnimationSpeed` multiplier scales the
/// countdown).
pub const INTER_ANIM_DELAY: f32 = 0.18;

/// Pop the next queued animation onto an entity once it stops animating.
/// Runs every frame; entities without an `AnimQueue` are simply skipped.
/// Filtering with `Without<Animating>` ensures the previous animation has
/// finished (since each `animate_*` system removes `Animating` on
/// completion). When an animation just ended (detected via
/// `RemovedComponents<Animating>`), arms a brief delay so consecutive
/// queued animations don't visually merge.
pub fn dispatch_animation_queue(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<AnimationSpeed>,
    mut removed: RemovedComponents<Animating>,
    mut queues: ParamSet<(
        Query<&mut AnimQueue>,
        Query<(Entity, &mut AnimQueue), Without<Animating>>,
    )>,
) {
    // Arm the inter-anim delay on entities that just had `Animating`
    // removed (i.e. their previous animation finished this frame). Drain
    // before the dispatch loop so the same frame's pop sees the delay.
    {
        let mut all_queues = queues.p0();
        for entity in removed.read() {
            if let Ok(mut queue) = all_queues.get_mut(entity)
                && !queue.queue.is_empty()
            {
                queue.delay_remaining = INTER_ANIM_DELAY;
            }
        }
    }

    let dt = time.delta_secs() * speed.0;
    let mut q = queues.p1();
    for (entity, mut queue) in &mut q {
        if queue.delay_remaining > 0.0 {
            queue.delay_remaining = (queue.delay_remaining - dt).max(0.0);
            continue;
        }
        let Some(next) = queue.queue.pop_front() else {
            // Empty queue and not animating — drop the queue component
            // entirely so the entity stops matching this query.
            commands.entity(entity).remove::<AnimQueue>();
            continue;
        };
        let mut e = commands.entity(entity);
        e.insert(Animating);
        match next {
            QueuedAnim::Tap { anim, target_tapped } => {
                e.insert(anim);
                e.insert(crate::card::TapState { tapped: target_tapped });
            }
            QueuedAnim::Play(anim) => { e.insert(anim); }
            QueuedAnim::HandSlide(anim) => { e.insert(anim); }
            QueuedAnim::SendToGraveyard(anim) => { e.insert(anim); }
        }
    }
}
