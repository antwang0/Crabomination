//! Multiplayer table awareness: the HUD pieces a four-seat Commander pod
//! needs that a 1v1 table never did.
//!
//! * **Opponent summaries** — each opponent row leads with a large life
//!   total, a compact hand / library count, and one-click graveyard and
//!   exile counts that open the zone browsers filtered to that seat.
//! * **Eliminated seats** — a row for a player who has left the game
//!   (CR 104.3 / 800.4a) collapses to a dimmed "☠ OUT" summary.
//! * **Turn order** — a "▶ You → Bob → Carol → Dave" strip over the
//!   opponent rows, the active seat highlighted, eliminated seats struck.
//! * **Attack targets** — while the viewer builds an attack, a summary
//!   ("Bob ← Grizzly Bears, Llanowar Elves") sits over the confirm button,
//!   and a seat-coloured label floats over each player being attacked (the
//!   plan's defenders, or the declared attack's once it is in).
//!
//! The opponent row itself is still built by `update_opponent_stats_rows`;
//! this module provides the pieces it spawns plus its own systems, bundled
//! as [`TableAwarenessPlugin`].

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::{AttackTarget, TurnStep};
use crabomination::net::{ClientView, PlayerView};

use super::player_stats::{hand_chip_label, life_badge_style, LOW_LIBRARY_WARN};
pub(crate) use super::player_stats::seat_color;
use super::InGameRoot;
use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};
use crate::MainCamera;

pub struct TableAwarenessPlugin;

impl Plugin for TableAwarenessPlugin {
    fn build(&self, app: &mut App) {
        use crate::menu::AppState;
        app.add_systems(OnEnter(AppState::InGame), setup_attack_summary_panel)
            .add_systems(
                Update,
                (
                    handle_opponent_zone_chips,
                    update_attack_summary_panel,
                    sync_attack_target_labels,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                crate::systems::gizmos::draw_declared_attack_arrows
                    .after(crate::systems::animate::animate_combat_lurch)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

// ── Pure helpers ──────────────────────────────────────────────────────────────

/// The seat's display name from the viewer's side of the table: "You" for
/// the viewer, the player's name otherwise (seat number as a fallback).
pub(crate) fn seat_label(players: &[PlayerView], viewer: usize, seat: usize) -> String {
    if seat == viewer {
        return "You".to_string();
    }
    players
        .iter()
        .find(|p| p.seat == seat)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Seat {seat}"))
}

/// Large life readout: the total, with the change from the starting total
/// as a compact suffix (Speaker of the Heavens / commander-damage context).
pub(super) fn big_life_label(life: i32, starting_life: i32) -> String {
    match life - starting_life {
        0 => format!("\u{2665} {life}"),
        d if d > 0 => format!("\u{2665} {life} +{d}"),
        d => format!("\u{2665} {life} {d}"),
    }
}

/// One compact chip for the two hidden-zone counts a table scans for:
/// hand (with the max-hand-size cap when it matters) and library.
pub(super) fn vitals_label(hand: usize, max_hand_size: Option<usize>, library: usize) -> String {
    format!("{}  ▤ {library}", hand_chip_label(hand, max_hand_size))
}

/// "☠ OUT" plus the loss cause when the engine reports one (CR 104.3).
pub(super) fn eliminated_label(reason: Option<&str>) -> String {
    match reason {
        Some(r) if !r.is_empty() => format!("☠ OUT · {r}"),
        _ => "☠ OUT".to_string(),
    }
}

/// One seat of the turn-order strip.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TurnOrderEntry {
    pub seat: usize,
    pub label: String,
    pub active: bool,
    pub eliminated: bool,
}

/// Seats in turn order (ascending seat, wrapping — the engine's
/// `next_alive_seat` rotation), starting at the viewer so the strip reads
/// "You → next → …". Eliminated seats stay in (struck through by the
/// renderer) so the table's shape doesn't jump when someone dies.
pub(super) fn turn_order_entries(
    seats: &[(usize, String, bool)],
    viewer: usize,
    active: usize,
) -> Vec<TurnOrderEntry> {
    let mut sorted: Vec<&(usize, String, bool)> = seats.iter().collect();
    sorted.sort_by_key(|(s, ..)| *s);
    let start = sorted.iter().position(|(s, ..)| *s == viewer).unwrap_or(0);
    sorted.rotate_left(start);
    sorted
        .into_iter()
        .map(|(seat, name, eliminated)| TurnOrderEntry {
            seat: *seat,
            label: if *seat == viewer { "You".to_string() } else { name.clone() },
            active: *seat == active,
            eliminated: *eliminated,
        })
        .collect()
}

/// Plain-text rendering of the turn-order strip (the HUD renders the same
/// entries as coloured nodes; this is the shape it reads as).
#[cfg(test)]
pub(super) fn turn_order_text(entries: &[TurnOrderEntry]) -> String {
    entries
        .iter()
        .map(|e| {
            let mark = if e.active { "▶ " } else if e.eliminated { "☠ " } else { "" };
            format!("{mark}{}", e.label)
        })
        .collect::<Vec<_>>()
        .join(" → ")
}

/// Attackers grouped under one attack target, for the pre-confirm summary.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AttackGroup {
    /// The defending seat, when known (drives the colour).
    pub defender: Option<usize>,
    /// "Bob", or "Jace (Bob)" for a planeswalker / battle.
    pub target: String,
    pub attackers: Vec<String>,
}

/// Group a planned (or declared) attack by target, in order of first
/// appearance, naming each attacker. `defender_of` resolves the defending
/// seat of a target; `target_name` names a planeswalker / battle.
pub(super) fn attack_groups(
    plan: &[(CardId, AttackTarget)],
    card_name: impl Fn(CardId) -> String,
    defender_of: impl Fn(AttackTarget) -> Option<usize>,
    seat_name: impl Fn(usize) -> String,
) -> Vec<AttackGroup> {
    let mut groups: Vec<(AttackTarget, AttackGroup)> = Vec::new();
    for (attacker, target) in plan {
        let name = card_name(*attacker);
        if let Some((_, g)) = groups.iter_mut().find(|(t, _)| t == target) {
            g.attackers.push(name);
            continue;
        }
        let defender = defender_of(*target);
        let label = match target {
            AttackTarget::Player(seat) => seat_name(*seat),
            AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => match defender {
                Some(d) => format!("{} ({})", card_name(*id), seat_name(d)),
                None => card_name(*id),
            },
        };
        groups.push((
            *target,
            AttackGroup { defender, target: label, attackers: vec![name] },
        ));
    }
    groups.into_iter().map(|(_, g)| g).collect()
}

/// "Attacking: Bob ← Grizzly Bears, Llanowar Elves; Carol ← Wolf".
#[cfg(test)]
pub(super) fn attack_summary_text(groups: &[AttackGroup]) -> String {
    let body = groups
        .iter()
        .map(|g| format!("{} ← {}", g.target, g.attackers.join(", ")))
        .collect::<Vec<_>>()
        .join("; ");
    format!("Attacking: {body}")
}

/// Per defending seat, how many creatures are (or are planned to be)
/// attacking it or its permanents — the floating labels' counts, in seat
/// order.
pub(super) fn defender_counts(defenders: impl IntoIterator<Item = usize>) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for d in defenders {
        match out.iter_mut().find(|(s, _)| *s == d) {
            Some((_, n)) => *n += 1,
            None => out.push((d, 1)),
        }
    }
    out.sort_by_key(|(s, _)| *s);
    out
}

/// Defending seat of an attack target as the client can see it: the player
/// itself, a planeswalker's controller. A battle's protector isn't in the
/// view, so a planned battle attack resolves to `None` (declared attacks
/// carry the engine's `defending_player`).
pub(crate) fn plan_defender(cv: &ClientView, target: AttackTarget) -> Option<usize> {
    match target {
        AttackTarget::Player(s) => Some(s),
        AttackTarget::Planeswalker(id) => {
            cv.battlefield.iter().find(|c| c.id == id).map(|c| c.controller)
        }
        // CR 508.4 — a battle is defended by its protector.
        AttackTarget::Battle(id) => {
            cv.battlefield.iter().find(|c| c.id == id).and_then(|c| c.protected_by)
        }
    }
}

/// True while the viewer is assembling an attack plan this frame (their
/// DeclareAttackers step, holding priority).
fn planning_attack(cv: &ClientView) -> bool {
    cv.step == TurnStep::DeclareAttackers
        && cv.declares_attacks(cv.your_seat)
        && cv.priority == cv.your_seat
}

// ── Opponent summary (spawned into each opponent row) ────────────────────────

/// Which zone an [`OpponentZoneChip`] opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    Graveyard,
    Exile,
}

/// Clickable graveyard / exile count on an opponent row. A `Button` of its
/// own, so a click opens the browser instead of falling through to the
/// row's player-targeting hit-region.
#[derive(Component, Clone, Copy)]
pub struct OpponentZoneChip {
    pub seat: usize,
    pub zone: ZoneKind,
}

fn spawn_text_chip(
    parent: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    text: String,
    size: f32,
    bg: Color,
    fg: Color,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(bg),
            Pickable::IGNORE,
        ))
        .with_children(|chip| {
            chip.spawn((Text::new(text), ui_fonts.tf(size), TextColor(fg), Pickable::IGNORE));
        });
}

fn spawn_seat_avatar(
    parent: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    seat: usize,
    name: &str,
    dim: bool,
) {
    let initial = name.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('?');
    let bg = if dim { Color::srgba(0.25, 0.25, 0.28, 0.9) } else { seat_color(seat) };
    parent
        .spawn((
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(bg),
            Pickable::IGNORE,
        ))
        .with_children(|a| {
            a.spawn((
                Text::new(if dim { "☠".to_string() } else { initial.to_string() }),
                ui_fonts.tf(16.0),
                TextColor(Color::srgb(0.08, 0.08, 0.10)),
                Pickable::IGNORE,
            ));
        });
}

fn spawn_zone_chip(
    parent: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    seat: usize,
    zone: ZoneKind,
    count: usize,
) {
    let (glyph, bg) = match zone {
        ZoneKind::Graveyard => ("✟", Color::srgba(0.16, 0.16, 0.16, 1.0)),
        ZoneKind::Exile => ("⌧", Color::srgba(0.20, 0.16, 0.26, 1.0)),
    };
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(bg),
            theme::HoverTint::new(bg),
            Button,
            OpponentZoneChip { seat, zone },
        ))
        .with_children(|chip| {
            chip.spawn((
                Text::new(format!("{glyph} {count}")),
                ui_fonts.tf(12.0),
                TextColor(theme::TEXT_SECONDARY),
                Pickable::IGNORE,
            ));
        });
}

/// Background of an opponent row: dimmed for an eliminated seat.
pub(super) fn opponent_row_bg(eliminated: bool) -> Color {
    if eliminated { Color::srgba(0.0, 0.0, 0.0, 0.45) } else { Color::NONE }
}

/// Spawn the leading summary of an opponent row: avatar, name (▶ while it's
/// their turn), a large life total, compact hand / library counts, and
/// clickable graveyard / exile counts. For an eliminated seat it spawns a
/// dimmed "☠ OUT" summary instead and returns `true` — the caller skips the
/// rest of the row, since none of that seat's state matters any more.
pub(super) fn spawn_opponent_summary(
    row: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    p: &PlayerView,
    cv: &ClientView,
) -> bool {
    if p.eliminated {
        spawn_seat_avatar(row, ui_fonts, p.seat, &p.name, true);
        spawn_text_chip(
            row,
            ui_fonts,
            p.name.clone(),
            13.0,
            Color::srgba(0.14, 0.14, 0.16, 1.0),
            theme::TEXT_MUTED,
        );
        spawn_text_chip(
            row,
            ui_fonts,
            eliminated_label(p.loss_reason.as_deref()),
            13.0,
            Color::srgba(0.30, 0.08, 0.08, 1.0),
            theme::TEXT_DANGER,
        );
        return true;
    }
    spawn_seat_avatar(row, ui_fonts, p.seat, &p.name, false);
    let active = cv.active_player == p.seat;
    let (name_bg, name_fg) = if active {
        (Color::srgba(0.36, 0.28, 0.10, 1.0), theme::ACCENT_GOLD)
    } else {
        (Color::srgba(0.18, 0.22, 0.34, 1.0), theme::TEXT_INFO)
    };
    let name = if active { format!("▶ {}", p.name) } else { p.name.clone() };
    spawn_text_chip(row, ui_fonts, name, 13.0, name_bg, name_fg);
    let (life_bg, life_fg) = life_badge_style(p.life);
    spawn_text_chip(row, ui_fonts, big_life_label(p.life, p.starting_life), 22.0, life_bg, life_fg);
    let vitals_bg = if p.library.size <= LOW_LIBRARY_WARN {
        Color::srgba(0.40, 0.26, 0.10, 1.0)
    } else {
        Color::srgba(0.18, 0.24, 0.20, 1.0)
    };
    spawn_text_chip(
        row,
        ui_fonts,
        vitals_label(p.hand.len(), p.max_hand_size, p.library.size),
        12.0,
        vitals_bg,
        theme::TEXT_PRIMARY,
    );
    spawn_zone_chip(row, ui_fonts, p.seat, ZoneKind::Graveyard, p.graveyard.len());
    let exiled = cv.exile.iter().filter(|c| c.owner == p.seat).count();
    spawn_zone_chip(row, ui_fonts, p.seat, ZoneKind::Exile, exiled);
    false
}

/// Spawn the turn-order strip ("▶ You → Bob → Carol → Dave") as a row of
/// seat-coloured names. Pods only — in 1v1 the turn text says it all.
pub(super) fn spawn_turn_order_strip(
    col: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    cv: &ClientView,
) {
    if cv.players.len() <= 2 {
        return;
    }
    let seats: Vec<(usize, String, bool)> =
        cv.players.iter().map(|p| (p.seat, p.name.clone(), p.eliminated)).collect();
    let entries = turn_order_entries(&seats, cv.your_seat, cv.active_player);
    col.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            min_height: Val::Px(20.0),
            padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|strip| {
        strip.spawn((
            Text::new("Turn order"),
            ui_fonts.tf(11.0),
            TextColor(theme::TEXT_MUTED),
            Pickable::IGNORE,
        ));
        for (i, e) in entries.iter().enumerate() {
            if i > 0 {
                strip.spawn((
                    Text::new("→"),
                    ui_fonts.tf(12.0),
                    TextColor(theme::TEXT_MUTED),
                    Pickable::IGNORE,
                ));
            }
            let (text, fg, bg) = if e.active {
                (format!("▶ {}", e.label), theme::TEXT_PRIMARY, Color::srgba(0.42, 0.32, 0.10, 1.0))
            } else if e.eliminated {
                (format!("☠ {}", e.label), theme::TEXT_MUTED, Color::NONE)
            } else {
                (e.label.clone(), seat_color(e.seat), Color::NONE)
            };
            strip
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    Pickable::IGNORE,
                ))
                .with_children(|n| {
                    n.spawn((Text::new(text), ui_fonts.tf(12.0), TextColor(fg), Pickable::IGNORE));
                });
        }
    });
}

/// Open the graveyard browser on that seat's graveyard, or the exile
/// browser filtered to the cards that seat owns.
pub fn handle_opponent_zone_chips(
    chips: Query<(&Interaction, &OpponentZoneChip), Changed<Interaction>>,
    mut graveyard: ResMut<crate::game::GraveyardBrowserState>,
    mut exile: ResMut<crate::game::ExileBrowserState>,
) {
    for (interaction, chip) in &chips {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match chip.zone {
            ZoneKind::Graveyard => {
                graveyard.open = true;
                graveyard.owner = chip.seat;
            }
            ZoneKind::Exile => {
                exile.open = true;
                exile.owner = Some(chip.seat);
            }
        }
        return;
    }
}

// ── Attack summary over the confirm button ───────────────────────────────────

#[derive(Component)]
pub struct AttackSummaryPanel;

#[derive(Component)]
pub struct AttackSummaryRows;

/// Bottom-centre panel just above the Attack All / Confirm Attack button
/// (which sits at `bottom: 140px`). Hidden until the plan has an attacker.
pub fn setup_attack_summary_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(186.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                display: Display::None,
                ..default()
            },
            AttackSummaryPanel,
            InGameRoot,
            Pickable::IGNORE,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    row_gap: Val::Px(2.0),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
                AttackSummaryRows,
                Pickable::IGNORE,
            ));
        });
}

/// Show "Attacking: Bob ← …" rows (one per target, in the defender's seat
/// colour) while the viewer's plan is non-empty.
pub fn update_attack_summary_panel(
    mut commands: Commands,
    view: Res<CurrentView>,
    attacking: Res<crate::game::AttackingState>,
    ui_fonts: Res<UiFonts>,
    card_names: Res<crate::game::CardNames>,
    mut panel_q: Query<&mut Node, With<AttackSummaryPanel>>,
    rows_q: Query<Entity, With<AttackSummaryRows>>,
    mut last: Local<Option<Vec<AttackGroup>>>,
) {
    let Ok(mut panel) = panel_q.single_mut() else { return };
    let groups = view.0.as_ref().and_then(|cv| {
        if !planning_attack(cv) || attacking.plan.is_empty() || cv.game_over.is_some() {
            return None;
        }
        let name = |id: CardId| {
            cv.battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| card_names.get(id))
        };
        Some(attack_groups(
            &attacking.plan,
            name,
            |t| plan_defender(cv, t),
            |s| seat_label(&cv.players, cv.your_seat, s),
        ))
    });
    let want = if groups.is_some() { Display::Flex } else { Display::None };
    if panel.display != want {
        panel.display = want;
    }
    if *last == groups {
        return;
    }
    *last = groups.clone();
    let Ok(rows) = rows_q.single() else { return };
    commands.entity(rows).despawn_children();
    let Some(groups) = groups else { return };
    commands.entity(rows).with_children(|col| {
        col.spawn((
            Text::new("Attacking:"),
            ui_fonts.tf(12.0),
            TextColor(theme::TEXT_SECONDARY),
            Pickable::IGNORE,
        ));
        for g in &groups {
            let color = g.defender.map(seat_color).unwrap_or(theme::ACCENT_GOLD);
            col.spawn((
                Text::new(format!("{} ← {}", g.target, g.attackers.join(", "))),
                ui_fonts.tf(14.0),
                TextColor(color),
                Pickable::IGNORE,
            ));
        }
    });
}

// ── Floating per-defender labels ─────────────────────────────────────────────

/// Screen-space label over a player being attacked; payload is the seat.
#[derive(Component)]
pub struct AttackTargetLabel(pub usize);

/// Defending seats and attacker counts to label right now: the viewer's plan
/// while they're building one, else the declared attack (from the engine's
/// `defending_player`), else nothing.
fn current_defenders(cv: &ClientView, attacking: &crate::game::AttackingState) -> Vec<(usize, usize)> {
    if planning_attack(cv) && !attacking.plan.is_empty() {
        return defender_counts(attacking.plan.iter().filter_map(|(_, t)| plan_defender(cv, *t)));
    }
    defender_counts(cv.battlefield.iter().filter(|c| c.attacking).filter_map(|c| c.defending_player))
}

/// Float a seat-coloured "⚔ Bob ← 2" label over each defending player's
/// table anchor — the spot the attack arrows point at — so in a pod it is
/// obvious which opponent each attack is aimed at.
#[allow(clippy::type_complexity)]
pub fn sync_attack_target_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    attacking: Res<crate::game::AttackingState>,
    ui_fonts: Res<UiFonts>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut labels: Query<(Entity, &AttackTargetLabel, &mut Node, &Children)>,
    mut texts: Query<&mut Text>,
) {
    let defenders = match &view.0 {
        // Pods only: in 1v1 there is one place an attack can go.
        Some(cv) if cv.game_over.is_none() && cv.players.len() > 2 => {
            current_defenders(cv, &attacking)
        }
        _ => Vec::new(),
    };
    let Some(cv) = &view.0 else {
        for (e, ..) in &labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    let n = cv.players.len();
    let screen_pos = |seat: usize| -> Option<Vec2> {
        let mut anchor = crate::card::layout::player_hand_anchor(seat, cv.your_seat, n);
        anchor.y = 0.3;
        camera.world_to_viewport(cam_xform, anchor).ok()
    };
    let text_for = |seat: usize, count: usize| {
        format!("⚔ {} ← {count}", seat_label(&cv.players, cv.your_seat, seat))
    };

    let mut seen: Vec<usize> = Vec::new();
    for (e, label, mut node, children) in &mut labels {
        let Some(&(_, count)) = defenders.iter().find(|(s, _)| *s == label.0) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.push(label.0);
        match screen_pos(label.0) {
            Some(v) => {
                node.display = Display::Flex;
                node.left = Val::Px(v.x - 40.0);
                node.top = Val::Px(v.y - 14.0);
            }
            None => node.display = Display::None,
        }
        let want = text_for(label.0, count);
        for child in children.iter() {
            if let Ok(mut t) = texts.get_mut(child)
                && t.0 != want
            {
                t.0 = want.clone();
            }
        }
    }
    for (seat, count) in defenders {
        if seen.contains(&seat) {
            continue;
        }
        let Some(v) = screen_pos(seat) else { continue };
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(v.x - 40.0),
                    top: Val::Px(v.y - 14.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
                BorderColor::all(seat_color(seat)),
                GlobalZIndex(theme::layer::HUD),
                AttackTargetLabel(seat),
                InGameRoot,
                Pickable::IGNORE,
            ))
            .with_children(|l| {
                l.spawn((
                    Text::new(text_for(seat, count)),
                    ui_fonts.tf(14.0),
                    TextColor(seat_color(seat)),
                    Pickable::IGNORE,
                ));
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_life_label_shows_delta_from_start() {
        assert_eq!(big_life_label(40, 40), "\u{2665} 40");
        assert_eq!(big_life_label(33, 40), "\u{2665} 33 -7");
        assert_eq!(big_life_label(45, 40), "\u{2665} 45 +5");
    }

    #[test]
    fn vitals_label_is_hand_then_library() {
        assert_eq!(vitals_label(5, Some(7), 48), "✋ 5  ▤ 48");
        // The hand half keeps the cap vocabulary of the hand chip.
        assert_eq!(vitals_label(8, Some(7), 2), "✋ 8/7  ▤ 2");
        assert_eq!(vitals_label(3, None, 10), "✋ 3 ∞  ▤ 10");
    }

    #[test]
    fn eliminated_label_names_the_cause_when_known() {
        assert_eq!(eliminated_label(None), "☠ OUT");
        assert_eq!(eliminated_label(Some("")), "☠ OUT");
        assert_eq!(eliminated_label(Some("commander damage")), "☠ OUT · commander damage");
    }

    #[test]
    fn turn_order_starts_at_viewer_and_marks_active_and_dead() {
        let seats = vec![
            (2, "Carol".to_string(), false),
            (0, "Alice".to_string(), false),
            (3, "Dave".to_string(), true),
            (1, "Bob".to_string(), false),
        ];
        let entries = turn_order_entries(&seats, 1, 2);
        let order: Vec<usize> = entries.iter().map(|e| e.seat).collect();
        assert_eq!(order, vec![1, 2, 3, 0], "viewer first, then ascending with wrap");
        assert_eq!(entries[0].label, "You");
        assert!(entries[1].active && !entries[0].active);
        assert!(entries[2].eliminated);
        assert_eq!(turn_order_text(&entries), "You → ▶ Carol → ☠ Dave → Alice");
    }

    #[test]
    fn attack_groups_collect_attackers_per_target_in_first_seen_order() {
        let plan = vec![
            (CardId(10), AttackTarget::Player(2)),
            (CardId(11), AttackTarget::Player(1)),
            (CardId(12), AttackTarget::Player(2)),
            (CardId(13), AttackTarget::Planeswalker(CardId(50))),
        ];
        let card_name = |id: CardId| match id.0 {
            10 => "Grizzly Bears".to_string(),
            11 => "Wolf".to_string(),
            12 => "Llanowar Elves".to_string(),
            13 => "Goblin".to_string(),
            50 => "Jace".to_string(),
            _ => "?".to_string(),
        };
        let defender = |t: AttackTarget| match t {
            AttackTarget::Player(s) => Some(s),
            AttackTarget::Planeswalker(_) => Some(3),
            AttackTarget::Battle(_) => None,
        };
        let seat_name = |s: usize| ["You", "Bob", "Carol", "Dave"][s].to_string();
        let groups = attack_groups(&plan, card_name, defender, seat_name);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].target, "Carol");
        assert_eq!(groups[0].defender, Some(2));
        assert_eq!(groups[0].attackers, vec!["Grizzly Bears", "Llanowar Elves"]);
        assert_eq!(groups[2].target, "Jace (Dave)");
        assert_eq!(
            attack_summary_text(&groups),
            "Attacking: Carol ← Grizzly Bears, Llanowar Elves; Bob ← Wolf; Jace (Dave) ← Goblin"
        );
    }

    #[test]
    fn defender_counts_tally_per_seat_sorted() {
        assert_eq!(defender_counts([3, 1, 3, 3]), vec![(1, 1), (3, 3)]);
        assert!(defender_counts(Vec::<usize>::new()).is_empty());
    }

    #[test]
    fn plan_defender_resolves_planeswalker_controller() {
        let cv = ClientView::default();
        assert_eq!(plan_defender(&cv, AttackTarget::Player(2)), Some(2));
        // Unknown planeswalker / battle (not in view) resolve to None.
        assert_eq!(plan_defender(&cv, AttackTarget::Planeswalker(CardId(9))), None);
        assert_eq!(plan_defender(&cv, AttackTarget::Battle(CardId(9))), None);
        // CR 508.4 — a battle in view is defended by its protector.
        let mut battle = crate::systems::counter_tooltip::tests::make_permanent_view(0, 0);
        battle.id = CardId(9);
        battle.protected_by = Some(3);
        let cv = ClientView { battlefield: vec![battle], ..ClientView::default() };
        assert_eq!(plan_defender(&cv, AttackTarget::Battle(CardId(9))), Some(3));
    }
}
