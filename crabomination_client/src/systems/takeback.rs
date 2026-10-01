//! Take-backs in the client (TODO "Engine — Rollback / Undo system", steps
//! 1–3): the Undo button and `Z`, and what a rewind does to the client.
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
//! The server marks each undo point in every client's stream
//! (`ServerMsg::UndoMark`), ahead of what its action sends. At a mark the
//! client notes where its log, stats and life graph are; a rewind to that
//! point puts them back. A mark, not a count of events: the TCP outbox may
//! drop an update a slow client never sees, and a count would then cut in
//! the wrong place.

use std::collections::{HashMap, VecDeque};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crabomination::net::{ClientMsg, ServerMsg, UndoPointView};

use crate::card::{Card, DrawCardAnimation, HandSlideAnimation, PlayCardAnimation, TapAnimation};
use crate::game::LogCut;
use crate::net_plugin::{CurrentView, NetOutbox, PendingManaCast};
use crate::systems::game_ui::LifeHistory;
use crate::systems::match_stats::MatchStats;
use crate::theme;

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
/// Its own system, behind `TextInputGuard`, because `handle_game_input`
/// returns early during a decision and off priority — both times a
/// take-back is wanted.
pub fn request_undo(
    keyboard: Res<ButtonInput<KeyCode>>,
    text_input: crate::systems::input_guard::TextInputGuard,
    view: Res<CurrentView>,
    takeback: Res<Takeback>,
    outbox: Option<Res<NetOutbox>>,
    button: Query<&Interaction, (Changed<Interaction>, With<UndoButton>)>,
) {
    let key = !text_input.typing() && keyboard.just_pressed(KeyCode::KeyZ);
    if !key && !button.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    let Some(cv) = &view.0 else { return };
    if cv.game_over.is_some() || takeback.points.is_empty() {
        return;
    }
    if let Some(outbox) = outbox {
        outbox.submit_msg(ClientMsg::RequestUndo { to: None });
    }
}

/// The longest caption under the Undo button, in characters.
const CAPTION_MAX: usize = 24;

/// The Undo button reads what it would take back, and greys out with
/// nothing to.
pub fn update_undo_button(
    takeback: Res<Takeback>,
    mut label: Query<&mut TextColor, (With<UndoButtonLabel>, Without<UndoButtonCaption>)>,
    mut caption: Query<(&mut Text, &mut TextColor), With<UndoButtonCaption>>,
) {
    if !takeback.is_changed() {
        return;
    }
    let last = takeback.points.last();
    for mut color in &mut label {
        color.0 = if last.is_some() { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
    }
    let text = match last {
        Some(point) if point.label.chars().count() > CAPTION_MAX => {
            format!("{}…", point.label.chars().take(CAPTION_MAX - 1).collect::<String>())
        }
        Some(point) => point.label.clone(),
        None => "nothing to take back".to_string(),
    };
    for (mut t, mut color) in &mut caption {
        if t.0 != text {
            t.0 = text.clone();
        }
        color.0 = theme::TEXT_MUTED;
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
}
