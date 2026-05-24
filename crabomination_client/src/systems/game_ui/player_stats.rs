//! 2-D HUD chip strips that mirror each player's life / hand / deck /
//! graveyard counts plus the viewer's mana pool. Sibling to
//! [`crate::systems::game_ui::crest`] which owns the 3-D anchor; the
//! chips remain as a compact corner roster that stays readable even
//! when the table is busy.

use bevy::prelude::*;
use crabomination::game::TurnStep;

use crate::game::TargetingState;
use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};

use super::{
    ManaPipRow, OpponentStatsContainer, OpponentStatusPanel, PlayerHudPanel, PlayerStatsRow,
};

// ── Stat chips ───────────────────────────────────────────────────────────────

/// Per-stat visual style for a player stat chip: (background, text colour).
/// Picked to mirror the mana-pip palette — saturated enough to register
/// as distinct chips, dim enough not to compete with the action buttons.
fn stat_chip_style(kind: StatChipKind) -> (Color, Color) {
    match kind {
        StatChipKind::Name => (Color::srgba(0.18, 0.22, 0.34, 1.0), theme::TEXT_INFO),
        StatChipKind::Life => (Color::srgba(0.30, 0.18, 0.18, 1.0), theme::TEXT_PRIMARY),
        StatChipKind::Hand => (Color::srgba(0.18, 0.24, 0.20, 1.0), theme::TEXT_PRIMARY),
        StatChipKind::Deck => (Color::srgba(0.20, 0.20, 0.24, 1.0), theme::TEXT_BODY),
        StatChipKind::Grave => (Color::srgba(0.16, 0.16, 0.16, 1.0), theme::TEXT_SECONDARY),
    }
}

#[derive(Clone, Copy)]
pub(super) enum StatChipKind {
    Name,
    Life,
    Hand,
    Deck,
    Grave,
}

pub(super) fn spawn_stat_chip(
    parent: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    kind: StatChipKind,
    text: String,
) {
    let (bg, fg) = stat_chip_style(kind);
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(bg),
            // Chip itself is non-interactive — clicks fall through to
            // the enclosing `PlayerHudPanel` Button so the whole row
            // acts as a single targeting hit-region.
            Pickable::IGNORE,
        ))
        .with_children(|chip| {
            chip.spawn((
                Text::new(text),
                ui_fonts.tf(12.0),
                TextColor(fg),
                Pickable::IGNORE,
            ));
        });
}

// ── Seat-stamp + targeting outline ───────────────────────────────────────────

/// Patch the viewer's `PlayerHudPanel.seat` to match the current
/// `your_seat`. The viewer panel is spawned at setup time before any
/// `ClientView` exists, so we stamp the seat in lazily once the first
/// view arrives. Cheap (a single component write on view changes).
pub fn sync_player_hud_seat(
    view: Res<CurrentView>,
    mut q: Query<&mut PlayerHudPanel>,
) {
    let Some(cv) = &view.0 else { return };
    for mut panel in &mut q {
        if panel.seat == usize::MAX {
            panel.seat = cv.your_seat;
        }
    }
}

/// Border colours for the player-chip "I am a click target" affordance.
/// Pulses the same yellow used by [`crate::card::ValidTarget`] permanent
/// outlines and the blocker-selection diamond so the visual vocabulary
/// is consistent across the targeting flow.
const PLAYER_CHIP_BORDER_IDLE: Color = Color::NONE;
const PLAYER_CHIP_BORDER_TARGET: Color = Color::srgb(1.0, 0.88, 0.0);

/// Pulse the border of any [`PlayerHudPanel`] that is currently a legal
/// click target — i.e. either:
///
/// * Spell / ability / decision targeting is active and a player would
///   be a valid pick (the engine's per-spell filter isn't consulted
///   here — we highlight all seats and let the click try; an invalid
///   choice surfaces as a `GameError` rather than silently failing).
/// * The viewer is mid-declare-attackers and has a pending attacker
///   (`AttackingState::last_added`) waiting for a defender pick. Only
///   *opponent* seats highlight in that case.
pub fn update_player_chip_target_outline(
    targeting: Res<TargetingState>,
    legal_targets: Res<crate::game::LegalTargets>,
    attacking: Res<crate::game::AttackingState>,
    view: Res<CurrentView>,
    time: Res<Time>,
    mut q: Query<(&PlayerHudPanel, &mut BorderColor)>,
) {
    let Some(cv) = &view.0 else {
        for (_, mut border) in &mut q {
            *border = BorderColor::all(PLAYER_CHIP_BORDER_IDLE);
        }
        return;
    };
    let your_seat = cv.your_seat;

    // Pulsing yellow — same period as the existing target-highlight
    // material on permanents.
    let pulse = 0.55 + 0.45 * (time.elapsed_secs() * 4.0).sin().abs();
    let pulsed = Color::srgb(pulse, pulse * 0.88, 0.0);

    let targeting_active = targeting.active;
    // Any targeting flow with an enumerated legal set (server-driven
    // `Decision::ChooseTarget` OR cast-time targeting via the catalog
    // evaluator) restricts the pulse to seats actually listed. Spell /
    // ability targets we couldn't enumerate fall through to the "any
    // seat is clickable" behaviour.
    let have_legal_set = targeting_active
        && (!legal_targets.players.is_empty() || !legal_targets.permanents.is_empty());
    let legal_player_set = if have_legal_set {
        Some(&legal_targets.players)
    } else {
        None
    };
    let attack_pick = cv.step == TurnStep::DeclareAttackers
        && cv.active_player == your_seat
        && cv.priority == your_seat
        && attacking.last_added.is_some();

    for (panel, mut border) in &mut q {
        let highlight = if let Some(set) = legal_player_set {
            set.contains(&panel.seat)
        } else if targeting_active {
            true
        } else if attack_pick {
            panel.seat != your_seat
        } else {
            false
        };
        *border = BorderColor::all(if highlight { pulsed } else { PLAYER_CHIP_BORDER_IDLE });
        let _ = PLAYER_CHIP_BORDER_TARGET;
    }
}

// ── Viewer chip row ──────────────────────────────────────────────────────────

/// Rebuild the viewer's stat chip row whenever the view changes.
/// Sibling system to `update_mana_pips` — same visual vocabulary, so
/// the entire HUD-strip reads as a single chip-based status bar.
pub fn update_player_stats_chips(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    row_q: Query<Entity, With<PlayerStatsRow>>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok(row) = row_q.single() else { return };
    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.players.iter().find(|p| p.seat == cv.your_seat) else { return };
    commands.entity(row).despawn_children();
    commands.entity(row).with_children(|row| {
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Name, p.name.clone());
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Life, format!("♥ {}", p.life));
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Hand, format!("✋ {}", p.hand.len()));
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Deck, format!("▤ {}", p.library.size));
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Grave, format!("✟ {}", p.graveyard.len()));
    });
}

// ── Mana pips ────────────────────────────────────────────────────────────────

/// Per-color visual style for a mana pip: background tint + readable
/// text colour for the count rendered on top.
fn mana_pip_colors(color: Option<crabomination::mana::Color>) -> (Color, Color) {
    use crabomination::mana::Color as MC;
    match color {
        Some(MC::White) => (Color::srgb(0.95, 0.93, 0.85), Color::srgb(0.20, 0.18, 0.15)),
        Some(MC::Blue) => (Color::srgb(0.30, 0.55, 0.90), theme::TEXT_PRIMARY),
        Some(MC::Black) => (Color::srgb(0.18, 0.18, 0.22), theme::TEXT_PRIMARY),
        Some(MC::Red) => (Color::srgb(0.85, 0.30, 0.25), theme::TEXT_PRIMARY),
        Some(MC::Green) => (Color::srgb(0.30, 0.65, 0.35), theme::TEXT_PRIMARY),
        // Colorless
        None => (Color::srgb(0.70, 0.70, 0.72), Color::srgb(0.20, 0.20, 0.22)),
    }
}

/// Rebuild the `ManaPipRow` children to reflect the viewer's current
/// mana pool. One small tinted chip per non-zero colour, plus a
/// colorless chip if any colorless mana is in the pool. Width is
/// auto-sized so the chip grows with the digit count.
pub fn update_mana_pips(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    row_q: Query<Entity, With<ManaPipRow>>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok(row) = row_q.single() else { return };
    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.players.iter().find(|p| p.seat == cv.your_seat) else { return };

    commands.entity(row).despawn_children();

    use crabomination::mana::Color as MC;
    let colors = [
        (Some(MC::White), "W"),
        (Some(MC::Blue), "U"),
        (Some(MC::Black), "B"),
        (Some(MC::Red), "R"),
        (Some(MC::Green), "G"),
    ];
    let mut entries: Vec<(Option<MC>, &'static str, u32)> = colors
        .iter()
        .filter_map(|(c, sym)| {
            let n = p.mana_pool.amount(c.unwrap());
            (n > 0).then_some((*c, *sym, n))
        })
        .collect();
    let colorless = p.mana_pool.colorless_amount();
    if colorless > 0 {
        entries.push((None, "C", colorless));
    }

    commands.entity(row).with_children(|row| {
        if entries.is_empty() {
            row.spawn((
                Text::new("(no mana)"),
                ui_fonts.tf(11.0),
                TextColor(theme::TEXT_MUTED),
            ));
            return;
        }
        for (color, sym, count) in entries {
            let (bg, fg) = mana_pip_colors(color);
            row.spawn((
                Node {
                    min_width: Val::Px(22.0),
                    height: Val::Px(18.0),
                    padding: UiRect::axes(Val::Px(5.0), Val::Px(0.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(2.0),
                    // Pill shape — pip identity is "a round mana token",
                    // not panel chrome, so it keeps its rounding even on
                    // the otherwise-square HUD.
                    border_radius: BorderRadius::all(Val::Px(9.0)),
                    ..default()
                },
                BackgroundColor(bg),
            ))
            .with_children(|chip| {
                let label = if count > 1 {
                    format!("{sym}{count}")
                } else {
                    sym.to_string()
                };
                chip.spawn((
                    Text::new(label),
                    ui_fonts.tf(11.0),
                    TextColor(fg),
                ));
            });
        }
    });
}

// ── Opponent strip ───────────────────────────────────────────────────────────

/// Flip the opponent strip's background between the neutral HUD colour
/// and the red `HUD_BG_DANGER` variant. Red signals "you are actually
/// threatened" rather than "an opponent exists":
///
/// - Viewer life ≤ 5 → danger (low-life warning).
/// - Sum of opponent attackers-that-could-swing-now ≥ viewer life → danger
///   (lethal on board next combat).
///
/// "Attackers that could swing now" = controlled by an opponent, not a
/// land, untapped, and either not summoning-sick or has Haste, and not
/// a Defender.
pub fn update_opponent_panel_tint(
    view: Res<CurrentView>,
    mut q: Query<&mut BackgroundColor, With<OpponentStatusPanel>>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok(mut bg) = q.single_mut() else { return };
    let Some(cv) = &view.0 else { return };
    let Some(viewer) = cv.players.iter().find(|p| p.seat == cv.your_seat) else { return };

    use crabomination::card::Keyword;
    let lethal_on_board: i32 = cv
        .battlefield
        .iter()
        .filter(|c| {
            c.owner != cv.your_seat
                && c.is_creature()
                && !c.tapped
                && (!c.summoning_sick || c.keywords.contains(&Keyword::Haste))
                && !c.keywords.contains(&Keyword::Defender)
        })
        .map(|c| c.power.max(0))
        .sum();

    let threatened = viewer.life <= 5 || lethal_on_board >= viewer.life;
    *bg = BackgroundColor(if threatened { theme::HUD_BG_DANGER } else { theme::HUD_BG });
}

/// Rebuild one chip-row per opponent inside the top-right panel.
/// Replaces the old single-`Text`-with-newlines approach so each
/// opponent is visually distinct (especially in Commander where there
/// are 3 of them) and uses the same chip vocabulary as the viewer.
pub fn update_opponent_stats_rows(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    container_q: Query<Entity, With<OpponentStatsContainer>>,
) {
    if !view.is_changed() {
        return;
    }
    let Ok(container) = container_q.single() else { return };
    let Some(cv) = &view.0 else { return };
    commands.entity(container).despawn_children();
    let opponents: Vec<_> = cv.players.iter().filter(|p| p.seat != cv.your_seat).collect();
    commands.entity(container).with_children(|col| {
        for p in opponents {
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(2.0),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
                Button,
                // Opponent row is its own clickable player target — the
                // viewer can click it to satisfy spell targeting or to
                // reassign the in-progress attack-plan's last_added
                // attacker.
                PlayerHudPanel { seat: p.seat },
            ))
            .with_children(|row| {
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Name, p.name.clone());
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Life, format!("♥ {}", p.life));
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Hand, format!("✋ {}", p.hand.len()));
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Deck, format!("▤ {}", p.library.size));
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Grave, format!("✟ {}", p.graveyard.len()));
            });
        }
    });
}
