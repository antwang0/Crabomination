//! Alt-key tooltip showing counter detail and modified P/T for the
//! battlefield card under the cursor.
//!
//! Hold either Alt (left/right) while hovering a card to surface a small
//! HUD panel with:
//!
//! - Current power/toughness (if creature) and `(printed X/Y)`
//!   when the values differ from the printed P/T.
//! - Loyalty count (for planeswalkers).
//! - One row per counter type and quantity (`+1/+1 ×3`, `Stun ×2`, …).
//!
//! The 3-D counter coins handle the at-a-glance "this card has stuff on
//! it" indicator; this tooltip is the click-through for the details
//! a player needs when the coin column gets dense.
//!
//! Anchored to the bottom-right corner of the viewport rather than
//! floating next to the 3-D card, because the peek popup
//! (`systems::ui::peek_popup`) also lights up on Alt-hold and centers a
//! large card-art image — a card-adjacent tooltip would overlap it.

use bevy::prelude::*;
use crabomination::card::{CardId, CardType, CounterType};

use crate::card::{BattlefieldCard, CardHovered, GameCardId};
use crate::net_plugin::CurrentView;
use crate::theme::UiFonts;

/// Root marker for the floating tooltip panel.
#[derive(Component)]
pub struct AltTooltip;

/// Marker on the tooltip's text node so the update system can rewrite it
/// without doing a child walk.
#[derive(Component)]
pub struct AltTooltipText;

#[allow(clippy::too_many_arguments)]
pub fn update_alt_tooltip(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    hovered: Query<&GameCardId, (With<BattlefieldCard>, With<CardHovered>)>,
    mut tooltip_q: Query<Entity, With<AltTooltip>>,
    mut text_q: Query<&mut Text, With<AltTooltipText>>,
) {
    let alt_held = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    // No alt or no hovered card → tear down any tooltip.
    let hovered_card_id: Option<CardId> = if alt_held {
        hovered.iter().next().map(|gid| gid.0)
    } else {
        None
    };

    let Some(card_id) = hovered_card_id else {
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.battlefield.iter().find(|p| p.id == card_id) else {
        // Card left the battlefield — drop the tooltip.
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    // Build the body without the card name (the peek popup already
    // shows the card art with its name). If there's nothing
    // interesting (no P/T mod, no loyalty, no counters, not tapped),
    // skip the tooltip entirely so we don't render an empty panel.
    let Some(body) = build_tooltip_body(p) else {
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    if let Ok(_) = tooltip_q.single_mut() {
        // Existing tooltip — just refresh its text.
        if let Ok(mut text) = text_q.single_mut()
            && text.0 != body
        {
            text.0 = body;
        }
        return;
    }

    // Spawn fresh tooltip pinned to the bottom-right corner so it
    // never overlaps the centered peek-popup card art.
    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(10.0),
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
            ui_fonts.tf(13.0),
            TextColor(Color::srgba(0.95, 0.95, 1.0, 1.0)),
            AltTooltipText,
            Pickable::IGNORE,
        ));
    });
}

/// Build the tooltip body. Returns `None` when the card has nothing
/// the peek-popup art doesn't already show — we don't want a tiny
/// dark panel popping up just to repeat "this is a creature with 2/2"
/// while the user is looking at the full card art.
fn build_tooltip_body(p: &crabomination::net::PermanentView) -> Option<String> {
    let mut lines = Vec::new();

    // P/T summary — only if modified, since the peek popup shows the
    // printed P/T as part of the card art.
    if p.card_types.contains(&CardType::Creature)
        && (p.power != p.base_power || p.toughness != p.base_toughness)
    {
        lines.push(format!(
            "{}/{}  (printed {}/{})",
            p.power, p.toughness, p.base_power, p.base_toughness
        ));
    }

    // Loyalty for planeswalkers (separate from counters list since it's
    // the headline number on every walker).
    if p.card_types.contains(&CardType::Planeswalker) {
        let loyalty = p
            .counters
            .iter()
            .find_map(|(k, v)| matches!(k, CounterType::Loyalty).then_some(*v))
            .unwrap_or(0);
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
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for (kind, n) in counters {
            lines.push(format!("{} ×{}", counter_label(kind), n));
        }
    }

    // Effective keywords (after layer effects). Show these so anthem
    // effects ("All your creatures have Lifelink", Inkling Verselord)
    // are visible even when the printed text doesn't include the
    // keyword. Keep the list compact — one line, comma-separated.
    if !p.keywords.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        let mut kw_strs: Vec<String> = p.keywords.iter().map(keyword_label).collect();
        kw_strs.sort();
        kw_strs.dedup();
        lines.push(kw_strs.join(", "));
    }

    // Activated abilities — show the cost + effect label so players
    // can see "this Witherbloom Pledgemage has {1}{B}, Pay 1 life:
    // Draw a card" without clicking through to the activator UI.
    let ability_lines: Vec<String> = p
        .abilities
        .iter()
        .filter(|a| !a.is_mana)
        .map(|a| {
            let cost = if a.cost_label.is_empty() { "—".to_string() } else { a.cost_label.clone() };
            format!("{}: {}", cost, a.effect_label)
        })
        .collect();
    if !ability_lines.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in ability_lines {
            lines.push(l);
        }
    }

    // Triggered abilities — short event-prefixed labels exposed by the
    // server via `PermanentView.triggered_ability_labels`. Lets players
    // see "ETB: Draw a card", "Magecraft: Drain 1", "Dies: Mill 2"
    // without round-tripping through the card-text panel.
    if !p.triggered_ability_labels.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in &p.triggered_ability_labels {
            lines.push(l.clone());
        }
    }

    if p.tapped {
        lines.push(String::from("(tapped)"));
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Render a `Keyword` as a short human string for the tooltip. Keeps
/// the labels short ("Lifelink", "First Strike") so a card with several
/// granted keywords doesn't blow out the tooltip line.
fn keyword_label(kw: &crabomination::card::Keyword) -> String {
    use crabomination::card::Keyword as K;
    match kw {
        K::Flying => "Flying".into(),
        K::FirstStrike => "First Strike".into(),
        K::DoubleStrike => "Double Strike".into(),
        K::Lifelink => "Lifelink".into(),
        K::Vigilance => "Vigilance".into(),
        K::Trample => "Trample".into(),
        K::Deathtouch => "Deathtouch".into(),
        K::Haste => "Haste".into(),
        K::Menace => "Menace".into(),
        K::Reach => "Reach".into(),
        K::Defender => "Defender".into(),
        K::Indestructible => "Indestructible".into(),
        K::Hexproof => "Hexproof".into(),
        K::Flash => "Flash".into(),
        K::Shroud => "Shroud".into(),
        K::Ward(n) => format!("Ward {n:?}"),
        K::Protection(c) => format!("Protection from {c:?}"),
        K::Cycling(_) => "Cycling".into(),
        K::Flashback(_) => "Flashback".into(),
        K::Convoke => "Convoke".into(),
        _ => format!("{kw:?}"),
    }
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
        CounterType::Prepared => "Prepared",
    }
}
