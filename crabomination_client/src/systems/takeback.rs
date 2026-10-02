//! Take-backs in the client (TODO "Engine — Rollback / Undo system", steps
//! 1–5): the Undo button and `Z`, and what a rewind does to the client.
//!
//! The server keeps the history (`crabomination::server::undo`) and sends
//! this seat its undo points whenever they change; Undo asks for the latest.
//! A rewind (`ServerMsg::Rewound`, then a full view) puts the game back as it
//! was, and the client:
//! - lays the table out afresh, every card at its place at once, rather than
//!   animating time backwards — a spell back in hand from the stack, a card
//!   caught mid-flight, a back face played as a land each went wrong that
//!   way;
//! - drops what it was in the middle of: a target pick, a cast held for mana
//!   (whose re-submits would replay it), blocks it thinks are declared, an
//!   End Turn still fast-forwarding, an open picker or decision modal;
//! - holds auto-pass until the player acts, or the window they took back to
//!   is passed straight away;
//! - puts the log, the match stats and the life graph back as they were at
//!   the point (step 3: they follow the branch actually played), then says
//!   what was taken back and what it showed.
//!
//! With other people at the table a take-back is a request they allow or
//! decline (step 4): a banner over the table while it waits, Allow and
//! Decline for those asked, Withdraw for the one asking. The Take-backs
//! setting (step 5, [`TakebackMode`]) turns Undo off — and declines others'
//! requests for you — or limits it to your last action; in the default, a
//! list (Shift+Z, or right-click Undo) goes back several at once.
//!
//! The server marks each undo point in every client's stream
//! (`ServerMsg::UndoMark`), ahead of what its action sends. At a mark the
//! client notes where its log, stats and life graph are; a rewind to that
//! point puts them back. A mark, not a count of events: the TCP outbox may
//! drop an update a slow client never sees, and a count would then cut in
//! the wrong place.

use std::collections::{HashMap, VecDeque};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crabomination::net::{ClientMsg, ClientView, ServerMsg, UndoPointView};
use serde::{Deserialize, Serialize};

use crate::card::{Card, DrawCardAnimation, HandSlideAnimation, PlayCardAnimation, TapAnimation};
use crate::config::ConfigStore;
use crate::game::{GameLog, LogCut};
use crate::net_plugin::{CurrentView, NetOutbox, PendingManaCast};
use crate::systems::esc::{EscFocus, EscSurface};
use crate::systems::game_ui::LifeHistory;
use crate::systems::match_stats::MatchStats;
use crate::theme::{self, HoverTint, UiFonts};

/// A take-back message, in the order the server sent it.
#[derive(Debug, PartialEq)]
pub enum Heard {
    /// `ServerMsg::UndoMark`: undo point `id` was kept here.
    Mark(u64),
    /// `ServerMsg::Rewound`: `by` took back `label`, to point `to`, having
    /// seen `saw`.
    Rewound { by: usize, label: String, to: u64, saw: Vec<String> },
}

/// Where the client was at an undo point's mark.
struct Mark {
    id: u64,
    log: LogCut,
    stats: MatchStats,
    life: HashMap<usize, Vec<(u32, i32)>>,
}

/// The most marks kept: twice the server's history, every seat's points.
const MARKS_KEPT: usize = 128;

/// What the server says about take-backs.
#[derive(Resource, Default)]
pub struct Takeback {
    /// This seat's undo points, oldest first (`ServerMsg::UndoPoints`).
    pub points: Vec<UndoPointView>,
    /// Marks and take-backs heard and not yet applied, in order.
    pub heard: Vec<Heard>,
    /// Messages `poll_net` held back to the next frame: a mark or a rewind
    /// is applied before a frame's events are folded into the log and stats,
    /// so one that arrives behind events waits, or it would land on the
    /// wrong side of them. Kept here so leaving the match drops them.
    pub held: VecDeque<ServerMsg>,
    /// The marks kept, oldest first.
    marks: VecDeque<Mark>,
    /// The take-back request the table is deciding, if any.
    pub asked: Option<Asked>,
}

/// What Undo may take back: the Take-backs setting
/// (`GameplayConfig::takebacks`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TakebackMode {
    /// No Undo, and another player's request is declined for you.
    Off,
    /// Your last action only.
    LastAction,
    /// Any of your recent actions, from a list (Shift+Z, or right-click
    /// Undo).
    #[default]
    Recent,
}

impl TakebackMode {
    pub fn label(self) -> &'static str {
        match self {
            TakebackMode::Off => "Off",
            TakebackMode::LastAction => "Last action",
            TakebackMode::Recent => "Recent actions",
        }
    }

    /// The next setting, for a button that cycles them.
    pub fn next(self) -> Self {
        match self {
            TakebackMode::Off => TakebackMode::LastAction,
            TakebackMode::LastAction => TakebackMode::Recent,
            TakebackMode::Recent => TakebackMode::Off,
        }
    }
}

/// A take-back request the table is deciding (`ServerMsg::UndoRequested`).
#[derive(Clone, Debug, PartialEq)]
pub struct Asked {
    pub by: usize,
    pub label: String,
    /// What the stretch it would undo showed `by`.
    pub saw: Vec<String>,
    /// The seats still to answer.
    pub waiting: Vec<usize>,
    /// When it runs out, in `Time::elapsed_secs_f64`.
    pub deadline: f64,
    /// This seat has answered it.
    pub answered: bool,
}

impl Takeback {
    /// A request heard: logged the first time, kept with its time left. It
    /// comes again, with fewer seats waiting, as each allows.
    pub fn ask(&mut self, asked: Asked, view: &CurrentView, log: &mut GameLog) {
        let again = self.asked.as_ref().filter(|a| a.by == asked.by && a.label == asked.label);
        let answered = again.is_some_and(|a| a.answered);
        if again.is_none() {
            log.push_colored(format!("⟲ {} asked to take back: {}", who(view, asked.by), asked.label), theme::TEXT_INFO);
        }
        self.asked = Some(Asked { answered, ..asked });
    }

    /// `by`'s request ended without a rewind, for `reason`.
    pub fn declined(&mut self, by: usize, reason: &str, view: &CurrentView, log: &mut GameLog) {
        let line = match self.asked.take() {
            Some(a) => format!("⟲ Not taken back: {} — {reason}", a.label),
            None => format!("⟲ {}'s take-back: {reason}", who(view, by)),
        };
        log.push_colored(line, theme::TEXT_INFO);
    }
}

/// `seat` as this client names it: "You", or the player's name.
fn who(view: &CurrentView, seat: usize) -> String {
    view.0.as_ref().map_or_else(
        || format!("P{seat}"),
        |cv| crate::systems::game_ui::table_awareness::seat_label(&cv.players, cv.your_seat, seat),
    )
}

/// The auto-pass hold after a take-back: the count of deliberate actions
/// (`NetOutbox::deliberate_count`) when it was taken. It holds until the
/// player acts.
#[derive(Resource, Default)]
pub struct RewindHold(Option<u64>);

impl RewindHold {
    pub fn holds(&self, deliberate: u64) -> bool {
        self.0 == Some(deliberate)
    }
}

/// Frames left in which a rebuilt table's cards go straight to their places
/// (the spawn, then the tap sync the frame after).
#[derive(Resource, Default)]
pub struct RewindSnap(u8);

/// The Undo button in the action column.
#[derive(Component)]
pub struct UndoButton;

/// Its "Undo (Z)" text.
#[derive(Component)]
pub struct UndoButtonLabel;

/// The line under it naming what Undo takes back.
#[derive(Component)]
pub struct UndoButtonCaption;

/// `Z` or the Undo button: take back the latest action, when there is one.
/// `Shift+Z`, or a right-click on the button, opens the list instead (the
/// Recent setting). Its own system, behind `TextInputGuard`, because
/// `handle_game_input` returns early during a decision and off priority —
/// both times a take-back is wanted.
#[allow(clippy::too_many_arguments)]
pub fn request_undo(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    text_input: crate::systems::input_guard::TextInputGuard,
    view: Res<CurrentView>,
    takeback: Res<Takeback>,
    store: Res<ConfigStore>,
    outbox: Option<Res<NetOutbox>>,
    button: Query<Ref<Interaction>, With<UndoButton>>,
    mut list: ResMut<UndoHistoryOpen>,
) {
    let key = !text_input.typing() && keyboard.just_pressed(KeyCode::KeyZ);
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let clicked = button.iter().any(|i| i.is_changed() && *i == Interaction::Pressed);
    let right_clicked = mouse.just_pressed(MouseButton::Right) && button.iter().any(|i| *i != Interaction::None);
    let mode = store.0.gameplay.takebacks;
    if mode == TakebackMode::Off {
        return;
    }
    if (key && shift) || right_clicked {
        if mode == TakebackMode::Recent && can_ask(&view, &takeback) {
            list.0 = !list.0;
        }
        return;
    }
    if (key || clicked)
        && can_ask(&view, &takeback)
        && let Some(outbox) = outbox
    {
        outbox.submit_msg(ClientMsg::RequestUndo { to: None });
    }
}

/// Whether this seat may ask for a take-back now: a point to go back to,
/// the game still on, and no request already out.
fn can_ask(view: &CurrentView, takeback: &Takeback) -> bool {
    view.0.as_ref().is_some_and(|cv| cv.game_over.is_none()) && !takeback.points.is_empty() && takeback.asked.is_none()
}

/// The longest caption under the Undo button, in characters.
const CAPTION_MAX: usize = 24;

/// The Undo button reads what it would take back, and greys out with
/// nothing to, with the setting off, or while a request waits.
pub fn update_undo_button(
    takeback: Res<Takeback>,
    store: Res<ConfigStore>,
    mut label: Query<&mut TextColor, (With<UndoButtonLabel>, Without<UndoButtonCaption>)>,
    mut caption: Query<(&mut Text, &mut TextColor), With<UndoButtonCaption>>,
) {
    if !takeback.is_changed() && !store.is_changed() {
        return;
    }
    let last = takeback.points.last();
    let (live, text) = if store.0.gameplay.takebacks == TakebackMode::Off {
        (false, "take-backs off".to_string())
    } else if takeback.asked.is_some() {
        (false, "waiting for an answer".to_string())
    } else {
        match last {
            Some(point) if point.label.chars().count() > CAPTION_MAX => {
                (true, format!("{}…", point.label.chars().take(CAPTION_MAX - 1).collect::<String>()))
            }
            Some(point) => (true, point.label.clone()),
            None => (false, "nothing to take back".to_string()),
        }
    };
    for mut color in &mut label {
        color.0 = if live { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
    }
    for (mut t, mut color) in &mut caption {
        if t.0 != text {
            t.0 = text.clone();
        }
        color.0 = theme::TEXT_MUTED;
    }
}

/// Whether the take-back list is open (Shift+Z, or right-click Undo, in the
/// Recent setting).
#[derive(Resource, Default)]
pub struct UndoHistoryOpen(pub bool);

/// The take-back list.
#[derive(Component)]
pub struct UndoHistoryPanel;

/// A row of the list: take back to just before point `.0`.
#[derive(Component)]
pub struct UndoHistoryRow(pub u64);

/// Show the list while it is open: this seat's points, newest first, each
/// a button that takes back to just before it — and every action after.
/// Esc closes it; so does a request going out, or the points running out.
pub fn sync_undo_history(
    mut commands: Commands,
    mut open: ResMut<UndoHistoryOpen>,
    takeback: Res<Takeback>,
    esc: Res<EscFocus>,
    fonts: Res<UiFonts>,
    existing: Query<Entity, With<UndoHistoryPanel>>,
) {
    if esc.owns(EscSurface::UndoHistory) || (open.0 && (takeback.points.is_empty() || takeback.asked.is_some())) {
        open.0 = false;
    }
    if !open.is_changed() && !takeback.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    if !open.0 {
        return;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                // Beside the action column, level with Undo.
                left: Val::Px(190.0),
                top: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(10.0)),
                max_height: Val::Percent(60.0),
                overflow: Overflow::clip_y(),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            UndoHistoryPanel,
            crate::systems::game_ui::InGameRoot,
            GlobalZIndex(theme::layer::MODAL),
        ))
        .with_children(|panel| {
            panel.spawn((Text::new("Take back to before…"), fonts.tf(13.0), TextColor(theme::TEXT_SECONDARY)));
            for point in takeback.points.iter().rev() {
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                        HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                        UndoHistoryRow(point.id),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(point_line(point)),
                            fonts.tf(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
            }
            panel.spawn((Text::new("Esc to close"), fonts.tf(10.0), TextColor(theme::TEXT_MUTED)));
        });
}

/// A point as the list shows it: "Turn 6 · Main 1 — cast Lightning Bolt".
pub fn point_line(point: &UndoPointView) -> String {
    format!("Turn {} · {} — {}", point.turn, crate::systems::game_ui::step_name(point.step), point.label)
}

/// A row of the list clicked: ask to take back to just before it.
pub fn handle_undo_history_rows(
    rows: Query<(&Interaction, &UndoHistoryRow), Changed<Interaction>>,
    mut open: ResMut<UndoHistoryOpen>,
    outbox: Option<Res<NetOutbox>>,
) {
    let Some((_, row)) = rows.iter().find(|(i, _)| **i == Interaction::Pressed) else { return };
    if let Some(outbox) = outbox {
        outbox.submit_msg(ClientMsg::RequestUndo { to: Some(row.0) });
    }
    open.0 = false;
}

/// The banner over the table while a take-back request waits.
#[derive(Component)]
pub struct AskedBanner;

/// Its line, counting down.
#[derive(Component)]
pub struct AskedBannerText;

/// Allow (`true`) or decline (`false`) the request — from the seat that
/// asked, withdraw it.
#[derive(Component)]
pub struct AnswerButton(pub bool);

/// The banner's line: who asks to take back what, what it showed them, who
/// it waits for, and the seconds left.
pub fn asked_line(a: &Asked, cv: &ClientView, now: f64) -> String {
    use crate::systems::game_ui::table_awareness::seat_label;
    let left = (a.deadline - now).max(0.0).ceil() as u32;
    let names: Vec<String> = a.waiting.iter().map(|&s| seat_label(&cv.players, cv.your_seat, s)).collect();
    let waiting = format!("waiting for {}", names.join(", "));
    if a.by == cv.your_seat {
        return format!("⟲ You asked to take back: {} — {waiting} · {left}s", a.label);
    }
    let saw = if a.saw.is_empty() { String::new() } else { format!(" (they saw {})", a.saw.join(", ")) };
    let who = seat_label(&cv.players, cv.your_seat, a.by);
    if a.waiting.contains(&cv.your_seat) && !a.answered {
        format!("⟲ {who} asks to take back: {}{saw} · {left}s", a.label)
    } else {
        format!("⟲ {who} asks to take back: {}{saw} — {waiting} · {left}s", a.label)
    }
}

/// Put the banner up while a request waits — with Allow and Decline for a
/// seat it asks, Withdraw for the seat asking — and count it down.
pub fn sync_asked_banner(
    mut commands: Commands,
    takeback: Res<Takeback>,
    view: Res<CurrentView>,
    time: Res<Time>,
    fonts: Res<UiFonts>,
    existing: Query<Entity, With<AskedBanner>>,
    mut texts: Query<&mut Text, With<AskedBannerText>>,
) {
    let (Some(a), Some(cv)) = (&takeback.asked, &view.0) else {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    };
    let line = asked_line(a, cv, time.elapsed_secs_f64());
    if !takeback.is_changed() && !existing.is_empty() {
        for mut t in &mut texts {
            if t.0 != line {
                t.0 = line.clone();
            }
        }
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    let buttons: &[(&str, bool, Color)] = if a.by == cv.your_seat {
        &[("Withdraw", false, theme::BUTTON_NEUTRAL_BG)]
    } else if a.waiting.contains(&cv.your_seat) && !a.answered {
        &[("Allow", true, theme::BUTTON_PRIMARY_BG), ("Decline", false, theme::BUTTON_DANGER_BG)]
    } else {
        &[]
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(86.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            AskedBanner,
            crate::systems::game_ui::InGameRoot,
            Pickable::IGNORE,
            GlobalZIndex(theme::layer::BANNER_URGENT),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
            ))
            .with_children(|bar| {
                bar.spawn((Text::new(line), AskedBannerText, fonts.tf(15.0), TextColor(theme::TEXT_INFO), Pickable::IGNORE));
                for &(label, accept, bg) in buttons {
                    bar.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                            border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                            ..default()
                        },
                        BackgroundColor(bg),
                        HoverTint::new(bg),
                        AnswerButton(accept),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(label), fonts.tf(14.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                    });
                }
            });
        });
}

/// Answer a request: the banner's buttons, or — with take-backs off — a
/// decline sent for you the moment it asks.
pub fn answer_take_backs(
    buttons: Query<(&Interaction, &AnswerButton), Changed<Interaction>>,
    mut takeback: ResMut<Takeback>,
    view: Res<CurrentView>,
    store: Res<ConfigStore>,
    outbox: Option<Res<NetOutbox>>,
) {
    let (Some(cv), Some(outbox)) = (&view.0, outbox) else { return };
    let Some(a) = &takeback.asked else { return };
    let mine = a.by == cv.your_seat || (a.waiting.contains(&cv.your_seat) && !a.answered);
    let off = store.0.gameplay.takebacks == TakebackMode::Off && a.by != cv.your_seat;
    let pressed = buttons.iter().find(|(i, _)| **i == Interaction::Pressed).map(|(_, b)| b.0);
    let Some(accept) = pressed.or(off.then_some(false)) else { return };
    if !mine {
        return;
    }
    outbox.submit_msg(ClientMsg::RespondUndo { accept });
    if let Some(a) = &mut takeback.asked {
        a.answered = true;
    }
}

/// Everything a rewind leaves stale in the client.
#[derive(SystemParam)]
pub struct RewindResets<'w> {
    targeting: ResMut<'w, crate::game::TargetingState>,
    legal_targets: ResMut<'w, crate::game::LegalTargets>,
    pending_cast: ResMut<'w, PendingManaCast>,
    blocking: ResMut<'w, crate::game::BlockingState>,
    attacking: ResMut<'w, crate::game::AttackingState>,
    ff: ResMut<'w, crate::systems::game_ui::FastForward>,
    decisions: ResMut<'w, crate::systems::decision_ui::DecisionUiState>,
    life_ticker: ResMut<'w, crate::systems::game_ui::life_ticker::LifeTicker>,
    banner: ResMut<'w, crate::systems::game_ui::PhaseBannerTracker>,
}

impl RewindResets<'_> {
    fn reset(&mut self) {
        *self.targeting = Default::default();
        *self.legal_targets = Default::default();
        self.pending_cast.0 = None;
        *self.blocking = Default::default();
        *self.attacking = Default::default();
        // The hold-priority toggle is the player's setting, not the game's.
        let manual_priority = self.ff.manual_priority;
        *self.ff = crate::systems::game_ui::FastForward { manual_priority, ..Default::default() };
        // A modal for a decision the rewind took away goes; one the rewind
        // brought back is put up afresh from the view.
        *self.decisions = Default::default();
        // Re-primed on the restored view: no "+3" for life the rewind gave
        // back, no turn banner for the turn it went back to.
        *self.life_ticker = Default::default();
        *self.banner = Default::default();
    }
}

/// The pickers and menus, closed by a rewind.
#[derive(SystemParam)]
pub struct Pickers<'w> {
    alt_cast: ResMut<'w, crate::game::AltCastState>,
    helper_tap: ResMut<'w, crate::game::HelperTapState>,
    spree_cast: ResMut<'w, crate::game::SpreeCastState>,
    split_cast: ResMut<'w, crate::game::SplitCastState>,
    pay_times: ResMut<'w, crate::game::PayTimesState>,
    ability_menu: ResMut<'w, crate::game::AbilityMenuState>,
    hand_menu: ResMut<'w, crate::systems::game_ui::hand_menu::HandMenuState>,
}

/// Apply the marks and take-backs `poll_net` heard of, in order. At a mark,
/// note where the log, stats and life graph are. At a take-back, put them
/// back to its point's mark, log it, clear the table for a fresh layout,
/// reset the client and hold auto-pass. Runs in `PreUpdate` right after
/// `poll_net`: ahead of everything that folds in this frame's events, and
/// with the cleared table gone before any `Update` system runs — the visual
/// sync then lays the restored view out from scratch.
#[allow(clippy::too_many_arguments)]
pub fn apply_rewinds(
    mut commands: Commands,
    mut takeback: ResMut<Takeback>,
    mut view: ResMut<CurrentView>,
    cards: Query<Entity, With<Card>>,
    outbox: Option<Res<NetOutbox>>,
    mut resets: RewindResets,
    mut pickers: Pickers,
    (mut hold, mut snap): (ResMut<RewindHold>, ResMut<RewindSnap>),
    mut log: ResMut<crate::game::GameLog>,
    (mut stats, mut life): (ResMut<MatchStats>, ResMut<LifeHistory>),
) {
    if takeback.heard.is_empty() {
        return;
    }
    let mut rewound = false;
    for heard in std::mem::take(&mut takeback.heard) {
        match heard {
            Heard::Mark(id) => {
                let mark = Mark { id, log: log.cut(), stats: stats.clone(), life: life.samples.clone() };
                takeback.marks.push_back(mark);
                if takeback.marks.len() > MARKS_KEPT {
                    takeback.marks.pop_front();
                }
            }
            Heard::Rewound { by, label, to, saw } => {
                rewound = true;
                // The point's mark, and every later one, go with it. A
                // client that never heard the mark (it reconnected since)
                // keeps its log and stats as they are.
                if let Some(at) = takeback.marks.iter().position(|m| m.id == to) {
                    let mark = &takeback.marks[at];
                    log.rewind_to(&mark.log);
                    *stats = mark.stats.clone();
                    life.samples = mark.life.clone();
                    life.revision = life.revision.wrapping_add(1);
                    takeback.marks.truncate(at);
                }
                let (who, you) = view.0.as_ref().map_or_else(
                    || ("A player".to_string(), false),
                    |cv| {
                        let who = crate::systems::game_ui::table_awareness::seat_label(&cv.players, cv.your_seat, by);
                        (who, cv.your_seat == by)
                    },
                );
                let saw = if saw.is_empty() {
                    String::new()
                } else {
                    format!(" ({} saw {})", if you { "you" } else { "they" }, saw.join(", "))
                };
                log.push_colored(format!("⟲ {who} took back: {label}{saw}"), theme::TEXT_INFO);
            }
        }
    }
    if !rewound {
        return;
    }
    for e in &cards {
        commands.entity(e).despawn();
    }
    resets.reset();
    crate::systems::game_ui::close_pickers(
        &mut pickers.alt_cast,
        &mut pickers.helper_tap,
        &mut pickers.spree_cast,
        &mut pickers.split_cast,
        &mut pickers.pay_times,
        &mut pickers.ability_menu,
        &mut pickers.hand_menu,
    );
    if let Some(outbox) = &outbox {
        outbox.forget_last_cast();
        hold.0 = Some(outbox.deliberate_count());
    }
    snap.0 = 3;
    // The visual sync lays the table out on a view change: make sure this
    // frame's counts as one, even if the restored view landed earlier.
    view.set_changed();
}

/// For a few frames after a rewind, every card the rebuild spawned goes
/// straight to its place: the table jumps back rather than being dealt
/// again from the decks.
pub fn snap_rebuilt_table(
    mut snap: ResMut<RewindSnap>,
    mut plays: Query<&mut PlayCardAnimation>,
    mut draws: Query<&mut DrawCardAnimation>,
    mut slides: Query<&mut HandSlideAnimation>,
    mut taps: Query<&mut TapAnimation>,
) {
    if snap.0 == 0 {
        return;
    }
    snap.0 -= 1;
    plays.iter_mut().for_each(|mut a| a.progress = 1.0);
    draws.iter_mut().for_each(|mut a| a.progress = 1.0);
    slides.iter_mut().for_each(|mut a| a.progress = 1.0);
    taps.iter_mut().for_each(|mut a| a.progress = 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hold lasts until the player's next deliberate action.
    #[test]
    fn the_hold_lasts_until_the_player_acts() {
        let hold = RewindHold(Some(4));
        assert!(hold.holds(4));
        assert!(!hold.holds(5), "they acted");
        assert!(!RewindHold::default().holds(0), "no rewind, no hold");
    }

    /// An app running `apply_rewinds` over every resource it touches.
    fn rewind_app() -> App {
        let mut app = App::new();
        app.init_resource::<Takeback>()
            .init_resource::<CurrentView>()
            .init_resource::<RewindHold>()
            .init_resource::<RewindSnap>()
            .init_resource::<crate::game::GameLog>()
            .init_resource::<MatchStats>()
            .init_resource::<LifeHistory>()
            .init_resource::<crate::game::TargetingState>()
            .init_resource::<crate::game::LegalTargets>()
            .init_resource::<PendingManaCast>()
            .init_resource::<crate::game::BlockingState>()
            .init_resource::<crate::game::AttackingState>()
            .init_resource::<crate::systems::game_ui::FastForward>()
            .init_resource::<crate::systems::decision_ui::DecisionUiState>()
            .init_resource::<crate::systems::game_ui::life_ticker::LifeTicker>()
            .init_resource::<crate::systems::game_ui::PhaseBannerTracker>()
            .init_resource::<crate::game::AltCastState>()
            .init_resource::<crate::game::HelperTapState>()
            .init_resource::<crate::game::SpreeCastState>()
            .init_resource::<crate::game::SplitCastState>()
            .init_resource::<crate::game::PayTimesState>()
            .init_resource::<crate::game::AbilityMenuState>()
            .init_resource::<crate::systems::game_ui::hand_menu::HandMenuState>()
            .add_systems(Update, apply_rewinds);
        app
    }

    fn rewound(label: &str, to: u64, saw: &[&str]) -> Heard {
        Heard::Rewound { by: 0, label: label.into(), to, saw: saw.iter().map(|s| s.to_string()).collect() }
    }

    /// A rewind clears what the client was in the middle of, keeping the
    /// player's hold-priority setting.
    #[test]
    fn a_rewind_resets_the_client() {
        let mut app = rewind_app();
        let world = app.world_mut();
        world.resource_mut::<crate::game::TargetingState>().active = true;
        world.resource_mut::<crate::game::BlockingState>().declared = true;
        {
            let mut ff = world.resource_mut::<crate::systems::game_ui::FastForward>();
            ff.end_turn = true;
            ff.manual_priority = true;
        }
        world.resource_mut::<crate::game::AbilityMenuState>().card_id = Some(crabomination::card::CardId(3));
        world.spawn(Card);
        world.resource_mut::<Takeback>().heard.push(rewound("cast Lightning Bolt", 1, &[]));
        app.update();

        let world = app.world_mut();
        assert!(!world.resource::<crate::game::TargetingState>().active);
        assert!(!world.resource::<crate::game::BlockingState>().declared);
        let ff = world.resource::<crate::systems::game_ui::FastForward>();
        assert!(!ff.end_turn && ff.manual_priority, "fast-forward off, the setting kept");
        assert!(world.resource::<crate::game::AbilityMenuState>().card_id.is_none());
        assert_eq!(world.query::<&Card>().iter(world).count(), 0, "the table is cleared for a fresh layout");
        assert!(world.resource::<Takeback>().heard.is_empty());
        let log = world.resource::<crate::game::GameLog>();
        assert!(log.entries.iter().any(|e| e.text.contains("took back: cast Lightning Bolt")));
    }

    /// A take-back puts the log, the match stats and the life graph back as
    /// they were at its point's mark: the stats of a game that never took
    /// the undone branch, a coalesced line's count as it was. Chat and
    /// notices stay, and the log says what the stretch showed.
    #[test]
    fn a_rewind_cuts_the_log_and_stats_back_to_its_mark() {
        use crabomination::card::CardId;
        use crabomination::net::GameEventWire as E;
        let kept = [E::CardDrawn { player: 0, card_id: CardId(1) }, E::LifeLost { player: 1, amount: 2 }];
        let undone = [E::SpellCast { player: 0, card_id: CardId(2), face: Default::default() }, E::LifeLost { player: 1, amount: 3 }];
        let mut app = rewind_app();
        let world = app.world_mut();
        for ev in &kept {
            world.resource_mut::<MatchStats>().fold(ev);
        }
        world.resource_mut::<crate::game::GameLog>().push_event("P1 lost 2 life", theme::TEXT_BODY);
        world.resource_mut::<LifeHistory>().samples.insert(1, vec![(1, 18)]);
        world.resource_mut::<Takeback>().heard.push(Heard::Mark(5));
        app.update();

        // The stretch the take-back undoes.
        let world = app.world_mut();
        for ev in &undone {
            world.resource_mut::<MatchStats>().fold(ev);
        }
        {
            let mut log = world.resource_mut::<crate::game::GameLog>();
            log.push_event("P1 lost 2 life", theme::TEXT_BODY);
            log.push("Bot: good game");
            log.push_event("You cast Lightning Bolt", theme::TEXT_BODY);
        }
        world.resource_mut::<LifeHistory>().samples.insert(1, vec![(1, 18), (2, 15)]);
        world.resource_mut::<Takeback>().heard.push(Heard::Mark(6));
        world.resource_mut::<Takeback>().heard.push(rewound("cast Lightning Bolt", 5, &["1 draw", "a coin flip"]));
        app.update();

        let world = app.world_mut();
        let mut never = MatchStats::default();
        kept.iter().for_each(|ev| never.fold(ev));
        assert_eq!(*world.resource::<MatchStats>(), never, "the stats of a game that never took the branch");
        let log: Vec<&str> = world.resource::<crate::game::GameLog>().entries.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(
            log,
            ["P1 lost 2 life", "Bot: good game", "⟲ A player took back: cast Lightning Bolt (they saw 1 draw, a coin flip)"],
            "the count is back to one; the chat stays"
        );
        assert_eq!(world.resource::<LifeHistory>().samples[&1], [(1, 18)]);
        assert!(world.resource::<Takeback>().marks.is_empty(), "the point's mark and the later one went with it");
    }

    fn demo_view(seat: usize) -> ClientView {
        let state = crabomination::demo::build_demo_state_seeded(7);
        crabomination::server::view::project(&state, seat)
    }

    fn opt_asked(by: usize, waiting: Vec<usize>) -> Asked {
        Asked { by, label: "cast Opt".into(), saw: vec!["1 draw".into()], waiting, deadline: 12.2, answered: false }
    }

    /// The banner says who asks to take back what, what it showed them, who
    /// it still waits for, and the seconds left — for the seat asked, the
    /// seat asking, and once answered.
    #[test]
    fn the_banner_names_the_request_and_who_it_waits_for() {
        let cv = demo_view(0);
        let them = crate::systems::game_ui::table_awareness::seat_label(&cv.players, 0, 1);
        let asked = opt_asked(1, vec![0]);
        assert_eq!(asked_line(&asked, &cv, 0.0), format!("⟲ {them} asks to take back: cast Opt (they saw 1 draw) · 13s"));
        let mine = opt_asked(0, vec![1]);
        assert_eq!(asked_line(&mine, &cv, 10.0), format!("⟲ You asked to take back: cast Opt — waiting for {them} · 3s"));
        let late = Asked { answered: true, ..asked };
        assert!(asked_line(&late, &cv, 20.0).ends_with("— waiting for You · 0s"), "answered: it says what it waits on");
    }

    /// A request is logged once, though it comes again as each seat allows,
    /// and a decline says why.
    #[test]
    fn a_request_is_logged_once_and_its_decline_says_why() {
        let view = CurrentView(Some(demo_view(0)));
        let mut log = GameLog::default();
        let mut takeback = Takeback::default();
        takeback.ask(opt_asked(1, vec![0, 2]), &view, &mut log);
        takeback.asked.as_mut().unwrap().answered = true;
        takeback.ask(opt_asked(1, vec![2]), &view, &mut log);
        assert!(takeback.asked.as_ref().unwrap().answered, "an answer given stays given");
        takeback.declined(1, "P2 declined", &view, &mut log);
        assert!(takeback.asked.is_none());
        let lines: Vec<&str> = log.entries.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].ends_with("asked to take back: cast Opt"), "{lines:?}");
        assert_eq!(lines[1], "⟲ Not taken back: cast Opt — P2 declined");
    }

    /// With take-backs off, another player's request is declined for you the
    /// moment it asks, once; your own is left alone.
    #[test]
    fn take_backs_off_declines_for_you() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut config = crate::config::Config::default();
        config.gameplay.takebacks = TakebackMode::Off;
        let mut app = App::new();
        app.insert_resource(ConfigStore(config))
            .insert_resource(CurrentView(Some(demo_view(0))))
            .insert_resource(NetOutbox::new(tx))
            .init_resource::<Takeback>()
            .add_systems(Update, answer_take_backs);
        app.world_mut().resource_mut::<Takeback>().asked = Some(opt_asked(1, vec![0]));
        app.update();
        app.update();
        let sent: Vec<ClientMsg> = rx.try_iter().collect();
        assert!(matches!(sent.as_slice(), [ClientMsg::RespondUndo { accept: false }]), "{sent:?}");
        app.world_mut().resource_mut::<Takeback>().asked = Some(opt_asked(0, vec![1]));
        app.update();
        assert!(rx.try_iter().next().is_none(), "a request of yours waits for the others");
    }

    /// The setting is saved by name and cycles through all three.
    #[test]
    fn the_setting_saves_and_cycles() {
        let mut config = crate::config::Config::default();
        assert_eq!(config.gameplay.takebacks, TakebackMode::Recent);
        config.gameplay.takebacks = TakebackMode::LastAction;
        let text = toml::to_string_pretty(&config).unwrap();
        assert!(text.contains("takebacks = \"last_action\""), "{text}");
        let back: crate::config::Config = toml::from_str(&text).unwrap();
        assert_eq!(back.gameplay.takebacks, TakebackMode::LastAction);
        let mut m = TakebackMode::Off;
        let seen: Vec<_> = (0..3).map(|_| { m = m.next(); m }).collect();
        assert_eq!(seen, [TakebackMode::LastAction, TakebackMode::Recent, TakebackMode::Off]);
    }
}
