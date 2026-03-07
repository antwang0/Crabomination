use std::f32::consts::PI;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::card::{
    hand_card_transform, Card, CardFlipAnimation, CardHoverLift, DeckCard, DeckShuffleAnimation,
    DrawCardAnimation, HandCard, HandSlideAnimation, ShufflePhase, CARD_HEIGHT, CARD_THICKNESS,
    CARD_WIDTH, DECK_CARD_Y_STEP, DECK_POSITION,
};

const FLIP_SPEED: f32 = 1.7;
const DRAW_SPEED: f32 = 2.5;
const SPREAD_DURATION: f32 = 0.1;
const SPREAD_STAGGER: f32 = 0.01;
const COLLAPSE_STAGGER: f32 = 0.01;
const SPREAD_Y_STEP: f32 = CARD_HEIGHT * 0.03;

pub const HAND_SLIDE_SPEED: f32 = 6.0;

#[derive(Component)]
pub struct RotateButton;

#[derive(Component)]
pub struct ShuffleButton;

#[derive(Component)]
pub struct DrawButton;

pub fn trigger_rotate(
    mut commands: Commands,
    cards: Query<
        (Entity, &Transform),
        (
            With<Card>,
            Without<DeckCard>,
            Without<HandCard>,
            Without<CardFlipAnimation>,
        ),
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<RotateButton>)>,
) {
    let mut should_rotate = keyboard.just_pressed(KeyCode::KeyR);

    for interaction in &button_query {
        if *interaction == Interaction::Pressed {
            should_rotate = true;
        }
    }

    if should_rotate {
        for (entity, transform) in &cards {
            let start = transform.rotation;
            let end = Quat::from_rotation_z(PI) * start;
            commands.entity(entity).insert(CardFlipAnimation {
                progress: 0.0,
                speed: FLIP_SPEED,
                start_rotation: start,
                end_rotation: end,
                start_y: transform.translation.y,
            });
        }
    }
}

pub fn trigger_shuffle(
    mut commands: Commands,
    deck_cards: Query<(Entity, &Transform, &DeckCard), Without<DeckShuffleAnimation>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<ShuffleButton>)>,
) {
    let mut should_shuffle = keyboard.just_pressed(KeyCode::KeyS);
    for interaction in &button_query {
        if *interaction == Interaction::Pressed {
            should_shuffle = true;
        }
    }
    if !should_shuffle {
        return;
    }

    let mut cards_sorted: Vec<(Entity, Vec3, Quat, usize)> = deck_cards
        .iter()
        .map(|(e, t, d)| (e, t.translation, t.rotation, d.index))
        .collect();
    cards_sorted.sort_by_key(|&(_, _, _, idx)| idx);

    let count = cards_sorted.len();
    if count == 0 {
        return;
    }

    let mut new_order: Vec<usize> = (0..count).collect();
    new_order.shuffle(&mut rand::thread_rng());

    let spread_base_y = CARD_HEIGHT * 0.5;
    let spread_wait = (count - 1) as f32 * SPREAD_STAGGER + SPREAD_DURATION;

    for (slot, (entity, translation, rotation, _)) in cards_sorted.iter().enumerate() {
        let i = slot;

        let spread_target = Vec3::new(
            DECK_POSITION.x,
            spread_base_y + i as f32 * SPREAD_Y_STEP,
            DECK_POSITION.z + i as f32 * CARD_THICKNESS * 2.0,
        );

        let new_index = new_order[i];

        let shuffled_spread_target = Vec3::new(
            DECK_POSITION.x,
            spread_base_y + new_index as f32 * SPREAD_Y_STEP,
            DECK_POSITION.z + new_index as f32 * CARD_THICKNESS * 2.0,
        );

        let index_delta = new_index as f32 - i as f32;
        let shuffle_arc_x = index_delta.signum() * (index_delta.abs().sqrt() * CARD_WIDTH * 0.4);

        let restack_target = Vec3::new(
            DECK_POSITION.x,
            new_index as f32 * DECK_CARD_Y_STEP + 0.01,
            DECK_POSITION.z,
        );

        commands.entity(*entity).insert(DeckShuffleAnimation {
            phase: ShufflePhase::Spread,
            phase_timer: 0.0,
            spread_target,
            shuffled_spread_target,
            shuffle_arc_x,
            restack_target,
            new_index,
            spread_delay: i as f32 * SPREAD_STAGGER,
            spread_wait,
            collapse_delay: new_index as f32 * COLLAPSE_STAGGER,
            phase_start_translation: *translation,
            phase_start_rotation: *rotation,
        });
    }
}

pub fn trigger_draw(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<DrawButton>)>,
    deck_cards: Query<
        (Entity, &Transform, &CardHoverLift, &DeckCard),
        (Without<DrawCardAnimation>, Without<DeckShuffleAnimation>),
    >,
    hand_cards: Query<&HandCard>,
) {
    let mut should_draw = keyboard.just_pressed(KeyCode::KeyD);
    for interaction in &button_query {
        if *interaction == Interaction::Pressed {
            should_draw = true;
        }
    }
    if !should_draw {
        return;
    }

    let Some((top_entity, top_transform, top_lift, _)) =
        deck_cards.iter().max_by_key(|(_, _, _, d)| d.index)
    else {
        return;
    };

    let start_translation = top_lift.base_translation;
    let new_slot = hand_cards.iter().count();
    let target = hand_card_transform(new_slot, new_slot + 1);

    commands.entity(top_entity).insert((
        HandCard { slot: new_slot },
        DrawCardAnimation {
            progress: 0.0,
            speed: DRAW_SPEED,
            start_translation,
            start_rotation: top_transform.rotation,
            target_translation: target.translation,
            target_rotation: target.rotation,
        },
    ));
    commands.entity(top_entity).remove::<DeckCard>();
}

/// When the hand layout changes, start a slide animation for settled cards not already sliding.
pub fn rebalance_hand(
    mut commands: Commands,
    settled: Query<
        (Entity, &HandCard, &Transform, &CardHoverLift),
        (Without<DrawCardAnimation>, Without<HandSlideAnimation>),
    >,
    all_hand: Query<&HandCard>,
) {
    let total = all_hand.iter().count();
    for (entity, hand_card, transform, lift) in &settled {
        let target = hand_card_transform(hand_card.slot, total);
        if (lift.base_translation - target.translation).length_squared() > 0.0001 {
            commands.entity(entity).insert(HandSlideAnimation {
                progress: 0.0,
                speed: HAND_SLIDE_SPEED,
                start_translation: transform.translation,
                target_translation: target.translation,
                target_rotation: target.rotation,
            });
        }
    }
}
