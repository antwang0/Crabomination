//! 2-D HUD chip strips that mirror each player's life / hand / deck /
//! graveyard counts plus the viewer's mana pool. Sibling to
//! [`crate::systems::game_ui::crest`] which owns the 3-D anchor; the
//! chips remain as a compact corner roster that stays readable even
//! when the table is busy.

use bevy::prelude::*;
use crabomination::game::TurnStep;

use crate::game::TargetingState;
use crate::net_plugin::CurrentView;
use crate::systems::kb_cursor::{KbSelection, KeyboardCursor};
use crate::theme::{self, UiFonts};

use super::{
    InGameRoot, ManaPipRow, OpponentStatsContainer, OpponentStatusPanel, PlayerHudPanel,
    PlayerStatsRow,
};

// ── Stat chips ───────────────────────────────────────────────────────────────

/// Per-stat visual style for a player stat chip: (background, text colour).
/// Picked to mirror the mana-pip palette — saturated enough to register
/// as distinct chips, dim enough not to compete with the action buttons.
fn stat_chip_style(kind: StatChipKind) -> (Color, Color) {
    match kind {
        StatChipKind::Name => (Color::srgba(0.18, 0.22, 0.34, 1.0), theme::TEXT_INFO),
        StatChipKind::Hand => (Color::srgba(0.18, 0.24, 0.20, 1.0), theme::TEXT_PRIMARY),
        StatChipKind::Deck => (Color::srgba(0.20, 0.20, 0.24, 1.0), theme::TEXT_BODY),
        // Deck-out warning: a dangerously low library (≤ `LOW_LIBRARY_WARN`
        // cards) glows amber so the viewer sees the CR 104.3a / 704.5c
        // draw-from-empty loss closing in — especially relevant in the new
        // dredge/mill shells, where self-mill can race the clock.
        StatChipKind::DeckLow => (Color::srgba(0.40, 0.26, 0.10, 1.0), theme::TEXT_PRIMARY),
        StatChipKind::Grave => (Color::srgba(0.16, 0.16, 0.16, 1.0), theme::TEXT_SECONDARY),
        // Poison shades from a sickly green toward a warning red as it
        // approaches the CR 104.3c / 704.5c lethal threshold of 10.
        StatChipKind::Poison => (Color::srgba(0.20, 0.32, 0.14, 1.0), theme::TEXT_PRIMARY),
        // Emblems (CR 114) are permanent command-zone effects — a regal
        // gold tint distinguishes them from the other counters.
        StatChipKind::Emblem => (Color::srgba(0.34, 0.28, 0.10, 1.0), theme::TEXT_PRIMARY),
        // Devotion (CR 700.5) — a Theros-nyx indigo to set it apart.
        StatChipKind::Devotion => (Color::srgba(0.22, 0.16, 0.34, 1.0), theme::TEXT_PRIMARY),
        // Energy (CR 122) — a Kaladesh aether teal.
        StatChipKind::Energy => (Color::srgba(0.10, 0.30, 0.32, 1.0), theme::TEXT_PRIMARY),
        // Draw cap (CR 121.2b) — a muted warning amber: drawing is locked.
        StatChipKind::DrawCap => (Color::srgba(0.36, 0.24, 0.10, 1.0), theme::TEXT_PRIMARY),
        // Storm/magecraft count (spells cast this turn) — an Izzet ember
        // orange so prowess / Storm / magecraft players can read the count
        // building before they cast the payoff.
        StatChipKind::Storm => (Color::srgba(0.36, 0.18, 0.10, 1.0), theme::TEXT_PRIMARY),
    }
}

#[derive(Clone, Copy)]
pub(super) enum StatChipKind {
    Name,
    Hand,
    Deck,
    DeckLow,
    Grave,
    Poison,
    Emblem,
    Devotion,
    Energy,
    DrawCap,
    Storm,
}

/// Whether to surface the "spells cast this turn" chip. Shown once a
/// second spell has been cast this turn (the point where Storm / magecraft
/// / prowess payoffs start to matter), so single-spell turns stay
/// uncluttered.
pub(super) fn storm_chip_visible(spells_cast_this_turn: u32) -> bool {
    spells_cast_this_turn >= 2
}

/// Library size at or below which the Deck chip switches to its amber
/// deck-out warning style (CR 104.3a — a player with an empty library
/// loses the next time they'd draw).
const LOW_LIBRARY_WARN: usize = 3;

/// Pick the Deck chip style based on remaining library size.
fn deck_chip_kind(library_size: usize) -> StatChipKind {
    if library_size <= LOW_LIBRARY_WARN {
        StatChipKind::DeckLow
    } else {
        StatChipKind::Deck
    }
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

// ── Avatar + prominent life ───────────────────────────────────────────────────

/// Stable per-seat identity colour. Indexed by seat so each player keeps
/// the same colour all game (Arena / MTGO-style player colours), wrapping
/// for more than `PALETTE.len()` seats. This is the player's visual
/// identity in the HUD now that the 3-D coloured disc is gone.
pub(super) fn seat_color(seat: usize) -> Color {
    const PALETTE: [Color; 6] = [
        Color::srgb(0.30, 0.55, 0.90), // blue
        Color::srgb(0.85, 0.32, 0.28), // red
        Color::srgb(0.35, 0.70, 0.40), // green
        Color::srgb(0.80, 0.58, 0.22), // amber
        Color::srgb(0.62, 0.42, 0.82), // purple
        Color::srgb(0.28, 0.70, 0.72), // teal
    ];
    PALETTE[seat % PALETTE.len()]
}

/// Spawn a small square avatar badge — the seat's identity colour with
/// the player's first initial. Leads each player row so the player is
/// recognisable at a glance without reading the name.
fn spawn_avatar(parent: &mut ChildSpawnerCommands, ui_fonts: &UiFonts, seat: usize, name: &str) {
    let initial = name
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?');
    parent
        .spawn((
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(seat_color(seat)),
            Pickable::IGNORE,
        ))
        .with_children(|a| {
            a.spawn((
                Text::new(initial.to_string()),
                ui_fonts.tf(15.0),
                TextColor(Color::srgb(0.08, 0.08, 0.10)),
                Pickable::IGNORE,
            ));
        });
}

/// Threshold-graded `(background, text)` colours for the life badge,
/// mirroring the tiers the old crest life numeral used.
fn life_badge_style(life: i32) -> (Color, Color) {
    match life {
        l if l <= 0 => (Color::srgb(0.55, 0.10, 0.10), theme::TEXT_PRIMARY),
        l if l <= 5 => (Color::srgb(0.45, 0.14, 0.14), theme::TEXT_DANGER),
        l if l <= 10 => (Color::srgb(0.36, 0.26, 0.10), Color::srgb(1.0, 0.80, 0.42)),
        _ => (Color::srgba(0.30, 0.18, 0.18, 1.0), theme::ACCENT_GOLD),
    }
}

/// Spawn the prominent life readout — a larger badge than the other
/// chips, since life is the number the eye goes to. Replaces the
/// floating 3-D life numeral the crest used to project over each disc.
fn spawn_life_badge(parent: &mut ChildSpawnerCommands, ui_fonts: &UiFonts, life: i32) {
    let (bg, fg) = life_badge_style(life);
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(bg),
            Pickable::IGNORE,
        ))
        .with_children(|chip| {
            chip.spawn((
                Text::new(format!("\u{2665} {life}")),
                ui_fonts.tf(18.0),
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
/// Active player's panel — soft gold, mirrors the crest ring's
/// active-player ambient.
const PLAYER_PANEL_ACTIVE: Color = Color::srgb(0.85, 0.68, 0.32);
/// Viewer's own panel when threatened (low life or lethal-on-board) —
/// the red warning the crest ring used to glow.
const PLAYER_PANEL_THREAT: Color = Color::srgb(1.0, 0.30, 0.30);
/// Solid (non-pulsing) bright yellow for the seat currently under the
/// keyboard cursor — replaces the disc's hover-brighten cue, so the user
/// can see which player the keyboard has selected.
const PLAYER_PANEL_KB_SELECTED: Color = Color::srgb(1.0, 0.95, 0.40);

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
    cursor: Res<KeyboardCursor>,
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
    // A computed legal set (even an empty one) is authoritative: restrict
    // the pulse to the listed seats. Only a *non-enumerated* pick (unknown
    // filter) falls through to the "any seat is clickable" behaviour, so an
    // enumerated-but-empty set (Beaming Defiance with no creatures you
    // control) correctly highlights no players.
    let have_legal_set = targeting_active && legal_targets.enumerated;
    let legal_player_set = if have_legal_set {
        Some(&legal_targets.players)
    } else {
        None
    };
    let attack_pick = cv.step == TurnStep::DeclareAttackers
        && cv.active_player == your_seat
        && cv.priority == your_seat
        && attacking.last_added.is_some();

    // Viewer threat — low life or lethal-on-board next combat. Same
    // computation the crest ring used to drive its red glow; it now
    // borders the viewer's own panel instead.
    use crabomination::card::Keyword;
    let viewer_life = cv
        .players
        .iter()
        .find(|p| p.seat == your_seat)
        .map(|p| p.life)
        .unwrap_or(20);
    let lethal_on_board: i32 = cv
        .battlefield
        .iter()
        .filter(|c| {
            c.owner != your_seat
                && c.is_creature()
                && !c.tapped
                && (!c.summoning_sick || c.keywords.contains(&Keyword::Haste))
                && !c.keywords.contains(&Keyword::Defender)
        })
        .map(|c| c.power.max(0))
        .sum();
    let viewer_threatened = viewer_life <= 5 || lethal_on_board >= viewer_life;

    for (panel, mut border) in &mut q {
        // Priority: a live targeting/attack pick (pulsing yellow) wins,
        // then the viewer's own threat warning (red), then the active
        // player's turn ambient (gold), else idle.
        let targetable = if let Some(set) = legal_player_set {
            set.contains(&panel.seat)
        } else if targeting_active {
            true
        } else if attack_pick {
            panel.seat != your_seat
        } else {
            false
        };
        let kb_selected = cursor.selection == Some(KbSelection::PlayerZone(panel.seat));
        let color = if kb_selected {
            PLAYER_PANEL_KB_SELECTED
        } else if targetable {
            pulsed
        } else if panel.seat == your_seat && viewer_threatened {
            PLAYER_PANEL_THREAT
        } else if panel.seat == cv.active_player {
            PLAYER_PANEL_ACTIVE
        } else {
            PLAYER_CHIP_BORDER_IDLE
        };
        *border = BorderColor::all(color);
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
        spawn_avatar(row, &ui_fonts, p.seat, &p.name);
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Name, p.name.clone());
        spawn_life_badge(row, &ui_fonts, p.life);
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Hand, format!("✋ {}", p.hand.len()));
        spawn_stat_chip(row, &ui_fonts, deck_chip_kind(p.library.size), format!("▤ {}", p.library.size));
        spawn_stat_chip(row, &ui_fonts, StatChipKind::Grave, format!("✟ {}", p.graveyard.len()));
        // Poison is a hidden lose condition (lethal at 10) — only surface
        // the chip once the player has actually been poisoned to avoid
        // cluttering the HUD in non-infect games.
        if p.poison_counters > 0 {
            spawn_stat_chip(
                row,
                &ui_fonts,
                StatChipKind::Poison,
                format!("☠ {}/10", p.poison_counters),
            );
        }
        // CR 122 energy — only surface once the player has banked any {E}.
        if p.energy > 0 {
            spawn_stat_chip(row, &ui_fonts, StatChipKind::Energy, format!("⚡ {}", p.energy));
        }
        // Storm / magecraft count — surface once a second spell lands this
        // turn so Storm / prowess / magecraft payoffs can be read at a glance.
        if storm_chip_visible(p.spells_cast_this_turn) {
            spawn_stat_chip(
                row,
                &ui_fonts,
                StatChipKind::Storm,
                format!("✷ {}", p.spells_cast_this_turn),
            );
        }
        // CR 121.2b draw cap — only surface when a draw lock is active.
        if let Some(cap) = p.draw_cap {
            spawn_stat_chip(
                row,
                &ui_fonts,
                StatChipKind::DrawCap,
                format!("✎ {}/{}", p.cards_drawn_this_turn, cap),
            );
        }
        // CR 114 emblems — only surface when the player actually owns one.
        if !p.emblems.is_empty() {
            spawn_stat_chip(
                row,
                &ui_fonts,
                StatChipKind::Emblem,
                format!("✦ {}", p.emblems.len()),
            );
        }
        // CR 700.5 devotion — only surface in Theros-flavored games (any
        // nonzero color). Compact per-color readout, e.g. "◆ B3 G1".
        if p.devotion.iter().any(|&d| d > 0) {
            const SYM: [&str; 5] = ["W", "U", "B", "R", "G"];
            let body: String = p
                .devotion
                .iter()
                .enumerate()
                .filter(|&(_, &d)| d > 0)
                .map(|(i, &d)| format!("{}{}", SYM[i], d))
                .collect::<Vec<_>>()
                .join(" ");
            spawn_stat_chip(row, &ui_fonts, StatChipKind::Devotion, format!("◆ {body}"));
        }
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

    // During an actual combat the engine's `combat_preview` projects the
    // *declared* damage to each player — use that exact figure when present
    // (more accurate than the all-untapped-power pre-combat estimate).
    let projected = cv
        .combat_preview
        .as_ref()
        .and_then(|cp| {
            cp.damage_to_players
                .iter()
                .find(|(seat, _)| *seat == cv.your_seat)
                .map(|(_, d)| *d)
        })
        .unwrap_or(lethal_on_board);

    let threatened = viewer.life <= 5 || projected >= viewer.life;
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
                spawn_avatar(row, &ui_fonts, p.seat, &p.name);
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Name, p.name.clone());
                spawn_life_badge(row, &ui_fonts, p.life);
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Hand, format!("✋ {}", p.hand.len()));
                spawn_stat_chip(row, &ui_fonts, deck_chip_kind(p.library.size), format!("▤ {}", p.library.size));
                spawn_stat_chip(row, &ui_fonts, StatChipKind::Grave, format!("✟ {}", p.graveyard.len()));
                if p.poison_counters > 0 {
                    spawn_stat_chip(
                        row,
                        &ui_fonts,
                        StatChipKind::Poison,
                        format!("☠ {}/10", p.poison_counters),
                    );
                }
                if p.energy > 0 {
                    spawn_stat_chip(row, &ui_fonts, StatChipKind::Energy, format!("⚡ {}", p.energy));
                }
                if let Some(cap) = p.draw_cap {
                    spawn_stat_chip(
                        row,
                        &ui_fonts,
                        StatChipKind::DrawCap,
                        format!("✎ {}/{}", p.cards_drawn_this_turn, cap),
                    );
                }
                if !p.emblems.is_empty() {
                    spawn_stat_chip(
                        row,
                        &ui_fonts,
                        StatChipKind::Emblem,
                        format!("✦ {}", p.emblems.len()),
                    );
                }
            });
        }
    });
}

// ── Life-change flash ─────────────────────────────────────────────────────────

/// Tracks each seat's last-seen life so [`trigger_life_flash`] fires a
/// floating delta only on an actual change. Primes silently on the first
/// view so connecting mid-game doesn't flash the starting totals.
#[derive(Resource, Default)]
pub struct LifeFlashTracker {
    last: std::collections::HashMap<usize, i32>,
    primed: bool,
}

/// A floating "+N" / "−N" life-change numeral that rises and fades next to
/// a player's HUD corner. Driven by [`animate_life_flash`].
#[derive(Component)]
pub struct LifeFlash {
    remaining: f32,
    total: f32,
    /// Spawn-time `top` in px; the numeral rises from here as it fades.
    base_top: f32,
}

const LIFE_FLASH_SECS: f32 = 1.3;
/// How far (px) the numeral floats upward over its lifetime.
const LIFE_FLASH_RISE: f32 = 30.0;

/// Spawn a floating life-delta numeral whenever a player's life total
/// changes between views — restoring the change feedback that the removed
/// 3-D life crest used to give, as a 2-D element anchored to each seat's
/// HUD corner (viewer top-left, opponents stacked top-right).
pub fn trigger_life_flash(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut tracker: ResMut<LifeFlashTracker>,
    ui_fonts: Res<UiFonts>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };

    // Opponent ordering, for vertical stagger in the top-right strip.
    let mut opponents: Vec<usize> =
        cv.players.iter().map(|p| p.seat).filter(|s| *s != cv.your_seat).collect();
    opponents.sort_unstable();

    let was_primed = tracker.primed;
    let mut deltas: Vec<(usize, i32)> = Vec::new();
    for p in &cv.players {
        let prev = tracker.last.insert(p.seat, p.life);
        if was_primed && prev.is_some_and(|prev| prev != p.life) {
            deltas.push((p.seat, p.life - prev.unwrap()));
        }
    }
    tracker.primed = true;

    for (seat, delta) in deltas {
        let (text, color) = if delta < 0 {
            (format!("{delta}"), theme::TEXT_DANGER)
        } else {
            (format!("+{delta}"), theme::TEXT_GOOD)
        };
        let base_top = if seat == cv.your_seat {
            36.0
        } else {
            let idx = opponents.iter().position(|s| *s == seat).unwrap_or(0);
            36.0 + idx as f32 * 52.0
        };
        let mut node = Node {
            position_type: PositionType::Absolute,
            top: Val::Px(base_top),
            ..default()
        };
        // Anchor just outside the seat's corner panel.
        if seat == cv.your_seat {
            node.left = Val::Px(276.0);
        } else {
            node.right = Val::Px(286.0);
        }
        commands
            .spawn((
                node,
                Pickable::IGNORE,
                InGameRoot,
                LifeFlash { remaining: LIFE_FLASH_SECS, total: LIFE_FLASH_SECS, base_top },
            ))
            .with_children(|p| {
                p.spawn((Text::new(text), ui_fonts.tf(30.0), TextColor(color), Pickable::IGNORE));
            });
    }
}

/// Float each life-flash numeral upward and fade it out, despawning when
/// elapsed.
pub fn animate_life_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut LifeFlash, &mut Node, &Children)>,
    mut texts: Query<&mut TextColor>,
) {
    for (entity, mut flash, mut node, children) in &mut flashes {
        flash.remaining -= time.delta_secs();
        if flash.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = (flash.remaining / flash.total).clamp(0.0, 1.0); // 1 → 0
        node.top = Val::Px(flash.base_top - (1.0 - frac) * LIFE_FLASH_RISE);
        // Hold full opacity, then ease out over the final 60%.
        let alpha = (frac / 0.6).min(1.0);
        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                let c = tc.0.to_srgba();
                tc.0 = Color::srgba(c.red, c.green, c.blue, alpha);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{deck_chip_kind, storm_chip_visible, StatChipKind};

    #[test]
    fn storm_chip_hidden_until_second_spell() {
        assert!(!storm_chip_visible(0));
        assert!(!storm_chip_visible(1));
        assert!(storm_chip_visible(2));
        assert!(storm_chip_visible(7));
    }

    #[test]
    fn deck_chip_warns_only_when_low() {
        assert!(matches!(deck_chip_kind(3), StatChipKind::DeckLow));
        assert!(matches!(deck_chip_kind(4), StatChipKind::Deck));
    }
}
