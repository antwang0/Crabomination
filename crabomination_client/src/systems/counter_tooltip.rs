//! Alt-key tooltip showing counter detail and modified P/T for the
//! battlefield card under the cursor.
//!
//! Hold either Alt (left/right) while hovering a card to surface a small
//! HUD panel with:
//!
//! - Card name + current power/toughness (if creature) and `(was X/Y)`
//!   when the values differ from the printed P/T.
//! - Loyalty count (for planeswalkers).
//! - One row per counter type and quantity (`+1/+1 ×3`, `Stun ×2`, …).
//!
//! The 3-D counter coins handle the at-a-glance "this card has stuff on
//! it" indicator; this tooltip is the click-through for the details
//! a player needs when the coin column gets dense.

use bevy::prelude::*;
use crabomination::card::{CardId, CardType, CounterType};

use crate::card::{BattlefieldCard, CardHovered, GameCardId};
use crate::net_plugin::CurrentView;

/// Root marker for the floating tooltip panel.
#[derive(Component)]
pub struct AltTooltip;

/// Marker on the tooltip's text node so the update system can rewrite it
/// without doing a child walk.
#[derive(Component)]
pub struct AltTooltipText;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn update_alt_tooltip(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    hovered: Query<(&GameCardId, &Transform), (With<BattlefieldCard>, With<CardHovered>)>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut tooltip_q: Query<(Entity, &mut Node), With<AltTooltip>>,
    mut text_q: Query<&mut Text, With<AltTooltipText>>,
) {
    let alt_held = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    // No alt or no hovered card → tear down any tooltip.
    let hovered_card: Option<(CardId, Vec3)> = if alt_held {
        hovered.iter().next().map(|(gid, t)| (gid.0, t.translation))
    } else {
        None
    };

    let Some((card_id, card_pos)) = hovered_card else {
        for (e, _) in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.battlefield.iter().find(|p| p.id == card_id) else {
        // Card left the battlefield — drop the tooltip.
        for (e, _) in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    let body = build_tooltip_body(p);

    // Project the card's world position to viewport pixels for placement.
    let Ok((camera, cam_xform)) = cameras.single() else { return };
    let Ok(screen) = camera.world_to_viewport(
        cam_xform,
        card_pos + Vec3::new(0.9, 0.6, 1.4),
    ) else {
        return;
    };

    if let Ok((_, mut node)) = tooltip_q.single_mut() {
        // Update existing tooltip's position and text.
        node.left = Val::Px(screen.x + 18.0);
        node.top = Val::Px(screen.y - 16.0);
        if let Ok(mut text) = text_q.single_mut()
            && text.0 != body
        {
            text.0 = body;
        }
        return;
    }

    // Spawn fresh tooltip.
    let font = asset_server.load("fonts/MiranoExtendedFreebie-Light.ttf");
    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen.x + 18.0),
                top: Val::Px(screen.y - 16.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.93)),
            AltTooltip,
            crate::systems::game_ui::InGameRoot,
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(body),
            TextFont { font, font_size: 13.0, ..default() },
            TextColor(Color::srgba(0.95, 0.95, 1.0, 1.0)),
            AltTooltipText,
            Pickable::IGNORE,
        ));
    });
}

fn build_tooltip_body(p: &crabomination::net::PermanentView) -> String {
    let mut lines = Vec::new();
    lines.push(p.name.clone());

    // P/T summary for creatures.
    if p.card_types.contains(&CardType::Creature) {
        if p.power != p.base_power || p.toughness != p.base_toughness {
            lines.push(format!(
                "{}/{}  (printed {}/{})",
                p.power, p.toughness, p.base_power, p.base_toughness
            ));
        } else {
            lines.push(format!("{}/{}", p.power, p.toughness));
        }
    }

    // Loyalty for planeswalkers (separate from counters list since it's
    // the headline number on every walker). Push XLII: prefer the new
    // top-level `PermanentView.loyalty: Option<i32>` (CR 306.5c), fall
    // back to scanning `counters` for back-compat with views serialized
    // before the field was added.
    if p.card_types.contains(&CardType::Planeswalker) {
        let loyalty = p.loyalty.unwrap_or_else(|| {
            p.counters
                .iter()
                .find_map(|(k, v)| matches!(k, CounterType::Loyalty).then_some(*v as i32))
                .unwrap_or(0)
        });
        lines.push(format!("Loyalty: {loyalty}"));
    }

    // Counters (excluding loyalty, which we already broke out).
    let mut counters: Vec<(CounterType, u32)> = p
        .counters
        .iter()
        .filter(|(k, n)| !matches!(k, CounterType::Loyalty) && *n > 0)
        .map(|(k, n)| (*k, *n))
        .collect();
    counters.sort_by_key(|(k, _)| sort_key(*k));
    if !counters.is_empty() {
        lines.push(String::from("─────────"));
        for (kind, n) in counters {
            lines.push(format!("{} ×{}", counter_label(kind), n));
        }
    }

    if p.tapped {
        lines.push(String::from("(tapped)"));
    }
    if p.prepared {
        lines.push(String::from("(prepared)"));
    }

    lines.join("\n")
}

fn sort_key(kind: CounterType) -> u8 {
    match kind {
        CounterType::PlusOnePlusOne => 0,
        CounterType::MinusOneMinusOne => 1,
        CounterType::Charge => 2,
        CounterType::Stun => 3,
        CounterType::Time => 4,
        CounterType::Poison => 5,
        CounterType::Energy => 6,
        _ => 7,
    }
}

fn counter_label(kind: CounterType) -> &'static str {
    match kind {
        CounterType::PlusOnePlusOne => "+1/+1",
        CounterType::MinusOneMinusOne => "-1/-1",
        CounterType::Loyalty => "Loyalty",
        CounterType::Charge => "Charge",
        CounterType::Stun => "Stun",
        CounterType::Time => "Time",
        CounterType::Poison => "Poison",
        CounterType::Lore => "Lore",
        CounterType::Fade => "Fade",
        CounterType::Age => "Age",
        CounterType::Level => "Level",
        CounterType::Energy => "Energy",
        CounterType::Experience => "Experience",
        CounterType::Verse => "Verse",
        CounterType::Shield => "Shield",
        CounterType::Wish => "Wish",
        CounterType::Page => "Page",
        CounterType::Growth => "Growth",
    }
}
