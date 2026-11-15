//! Match history: every game you finish is kept (format, opponents, result,
//! turns, finishing place), and the main menu's **History** panel shows
//! your record per format and the latest games.
//!
//! Stored as JSON under the client's storage key [`STORAGE_KEY`] (the OS
//! config dir natively, `localStorage` on the web), the newest
//! [`KEEP`] games. A spectated bot-vs-bot game isn't yours and isn't kept.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::net_plugin::CurrentView;
use crate::theme::{self, HoverTint, UiFonts};
use crate::ui_button::{Button, Interaction};

pub const STORAGE_KEY: &str = "match_history";
/// Games kept; older ones drop off.
pub const KEEP: usize = 200;
/// Games the panel lists.
const SHOWN: usize = 15;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchRecord {
    /// Seconds since the Unix epoch when the game ended (0 where the
    /// platform has no clock).
    pub ended: u64,
    /// `MatchFormat::label` ("Modern", "Commander", …).
    pub format: String,
    pub opponents: Vec<String>,
    /// `Some(true)` won, `Some(false)` lost, `None` a draw.
    pub won: Option<bool>,
    pub turns: u32,
    /// Finishing place and table size in a pod ("2nd of 4"); `None` in a duel.
    pub place: Option<(u8, u8)>,
}

/// The kept games, oldest first.
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct MatchHistory(pub Vec<MatchRecord>);

impl MatchHistory {
    pub fn load() -> Self {
        crate::storage::load(STORAGE_KEY).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    fn save(&self) {
        if let Ok(s) = serde_json::to_string(self) {
            crate::storage::save(STORAGE_KEY, &s);
        }
    }

    pub fn push(&mut self, record: MatchRecord) {
        self.0.push(record);
        let over = self.0.len().saturating_sub(KEEP);
        self.0.drain(..over);
    }

    /// `(format, wins, losses, draws)` per format, most played first.
    pub fn record_by_format(&self) -> Vec<(String, u32, u32, u32)> {
        let mut out: Vec<(String, u32, u32, u32)> = Vec::new();
        for m in &self.0 {
            let i = match out.iter().position(|(f, ..)| *f == m.format) {
                Some(i) => i,
                None => {
                    out.push((m.format.clone(), 0, 0, 0));
                    out.len() - 1
                }
            };
            match m.won {
                Some(true) => out[i].1 += 1,
                Some(false) => out[i].2 += 1,
                None => out[i].3 += 1,
            }
        }
        out.sort_by_key(|(_, w, l, d)| std::cmp::Reverse(w + l + d));
        out
    }
}

/// The viewer's record of a finished game, from its last view.
pub fn record_of(cv: &crabomination::net::ClientView, format: &str, ended: u64) -> Option<MatchRecord> {
    let winner = (*cv.game_over.as_ref()?)?;
    let won = Some(winner == cv.your_seat);
    let opponents = cv.players.iter().filter(|p| p.seat != cv.your_seat).map(|p| p.name.clone()).collect();
    // CR 800.4 — a pod ranks every seat (`GameState::placement`).
    let place = (cv.players.len() > 2)
        .then(|| cv.players.iter().find(|p| p.seat == cv.your_seat)?.placement)
        .flatten()
        .map(|at| (at as u8, cv.players.len() as u8));
    Some(MatchRecord { ended, format: format.to_string(), opponents, won, turns: cv.turn, place })
}

fn now_secs() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
}

/// Keep the game the moment it ends — once, and only one the viewer played.
pub fn record_finished_match(
    view: Res<CurrentView>,
    kind: Res<crate::systems::game_over::ActiveMatchKind>,
    format: Res<crate::systems::game_over::ActiveMatchFormat>,
    mut history: ResMut<MatchHistory>,
    mut recorded: Local<bool>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    match &cv.game_over {
        None => *recorded = false,
        Some(_) if *recorded => {}
        Some(_) => {
            *recorded = true;
            if *kind == crate::systems::game_over::ActiveMatchKind::SpectateBotVsBot {
                return;
            }
            // A draw has no winner seat; record it as one.
            let record = record_of(cv, format.0.label(), now_secs()).unwrap_or_else(|| MatchRecord {
                ended: now_secs(),
                format: format.0.label().to_string(),
                opponents: cv.players.iter().filter(|p| p.seat != cv.your_seat).map(|p| p.name.clone()).collect(),
                won: None,
                turns: cv.turn,
                place: None,
            });
            history.push(record);
            history.save();
        }
    }
}

// ── The menu panel ───────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct HistoryOpen(pub bool);

#[derive(Component)]
pub struct OpenHistoryButton;

#[derive(Component)]
pub struct HistoryPanelRoot;

#[derive(Component)]
pub struct HistoryCloseButton;

pub fn handle_history_open(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<HistoryOpen>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<OpenHistoryButton>)>,
    closes: Query<&Interaction, (Changed<Interaction>, With<HistoryCloseButton>)>,
) {
    if buttons.iter().any(|i| *i == Interaction::Pressed) {
        open.0 = !open.0;
    }
    if open.0 && (keyboard.just_pressed(KeyCode::Escape) || closes.iter().any(|i| *i == Interaction::Pressed)) {
        open.0 = false;
    }
}

fn result_text(m: &MatchRecord) -> String {
    let result = match (m.won, m.place) {
        (_, Some((place, of))) => format!("{} of {of}", ordinal(place)),
        (Some(true), None) => "Won".to_string(),
        (Some(false), None) => "Lost".to_string(),
        (None, _) => "Draw".to_string(),
    };
    format!("{result} · {} · turn {} · vs {}", m.format, m.turns, m.opponents.join(", "))
}

fn ordinal(n: u8) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

pub fn sync_history_panel(
    mut commands: Commands,
    open: Res<HistoryOpen>,
    history: Res<MatchHistory>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<HistoryPanelRoot>>,
) {
    if !open.is_changed() && !history.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    if !open.0 {
        return;
    }
    let tf = |s: f32| ui_fonts.tf(s);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            HistoryPanelRoot,
            GlobalZIndex(theme::layer::TOP_PROMPT),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(22.0)),
                    row_gap: Val::Px(6.0),
                    min_width: Val::Px(520.0),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|p| {
                p.spawn((Text::new("Match history"), tf(20.0), TextColor(theme::ACCENT_GOLD), Node { align_self: AlignSelf::Center, ..default() }));
                if history.0.is_empty() {
                    p.spawn((Text::new("No finished games yet."), tf(14.0), TextColor(theme::TEXT_SECONDARY)));
                }
                for (format, w, l, d) in history.record_by_format() {
                    let games = w + l + d;
                    let rate = (100 * w).checked_div(games).unwrap_or(0);
                    let draws = if d > 0 { format!(" – {d} drawn") } else { String::new() };
                    p.spawn((
                        Text::new(format!("{format}: {w} won – {l} lost{draws}  ({rate} %)")),
                        tf(14.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                }
                if !history.0.is_empty() {
                    p.spawn((Text::new("Latest"), tf(14.0), TextColor(theme::ACCENT_BLUE), Node { margin: UiRect::top(Val::Px(8.0)), ..default() }));
                }
                for m in history.0.iter().rev().take(SHOWN) {
                    let colour = match m.won {
                        Some(true) => theme::TEXT_PRIMARY,
                        _ => theme::TEXT_SECONDARY,
                    };
                    p.spawn((Text::new(result_text(m)), tf(12.0), TextColor(colour)));
                }
                p.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                        align_self: AlignSelf::Center,
                        margin: UiRect::top(Val::Px(10.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_PRIMARY_BG),
                    HoverTint::new(theme::BUTTON_PRIMARY_BG),
                    HistoryCloseButton,
                ))
                .with_children(|b| {
                    b.spawn((Text::new("Close (Esc)"), tf(14.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(format: &str, won: Option<bool>) -> MatchRecord {
        MatchRecord { ended: 0, format: format.into(), opponents: vec!["Bot 1".into()], won, turns: 9, place: None }
    }

    #[test]
    fn the_record_counts_per_format_and_keeps_the_newest() {
        let mut h = MatchHistory::default();
        h.push(rec("Modern", Some(true)));
        h.push(rec("Modern", Some(false)));
        h.push(rec("Modern", Some(true)));
        h.push(rec("Cube", None));
        assert_eq!(
            h.record_by_format(),
            vec![("Modern".to_string(), 2, 1, 0), ("Cube".to_string(), 0, 0, 1)]
        );
        for _ in 0..KEEP {
            h.push(rec("Cube", Some(true)));
        }
        assert_eq!(h.0.len(), KEEP);
        assert!(h.0.iter().all(|m| m.format == "Cube"), "the oldest dropped off");
        let back: MatchHistory = serde_json::from_str(&serde_json::to_string(&h).unwrap()).unwrap();
        assert_eq!(back.0.len(), KEEP);
    }

    #[test]
    fn a_finished_game_records_its_result() {
        let state = crabomination::demo::build_demo_state_seeded(7);
        let mut cv = crabomination::server::view::project(&state, 0);
        assert_eq!(record_of(&cv, "Modern", 1), None, "not over yet");
        cv.game_over = Some(Some(1));
        let r = record_of(&cv, "Modern", 1).unwrap();
        assert_eq!((r.won, r.place, r.opponents.len()), (Some(false), None, 1));
        assert_eq!(result_text(&r).split(" · ").next(), Some("Lost"));
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(12), "12th");
        assert_eq!(ordinal(23), "23rd");
    }
}
