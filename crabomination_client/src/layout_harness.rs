//! Layout harness: boot a fixed busy board and screenshot it.
//!
//! `--layout-fixture <seats>` starts a local match from [`fixture_state`] — a
//! mid-game board with full hands, land and creature rows, graveyards and
//! exile, paused in the viewer's main phase — instead of a fresh deal, so two
//! builds can be compared on the same table. `--screenshot <path>` saves the
//! primary window once the view has been up for `--screenshot-delay` seconds
//! (default 8, for card art to stream in) and then exits. `--window <WxH>`
//! opens the window at that size instead of maximized, so one machine can
//! render several aspect ratios. `--settings-open` opens the Esc menu.
//!
//!     cargo run --profile play -p crabomination_client -- \
//!         --layout-fixture 4 --window 1920x1080 --screenshot /tmp/pod.png

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use crabomination::game::{GameState, TurnStep};

/// Command-line options for the harness, parsed once in `main`.
#[derive(Resource, Clone, Debug, Default)]
pub struct HarnessArgs {
    /// Seats of the fixture board to boot into, if any.
    pub fixture_seats: Option<usize>,
    pub screenshot: Option<std::path::PathBuf>,
    pub screenshot_delay: f32,
    /// Forced window size (logical px); skips `maximize_window`.
    pub window: Option<(u32, u32)>,
    /// `--settings-open`: open the Esc menu once the view is up, so a
    /// screenshot can show it.
    pub settings_open: bool,
    /// `--viewer-out`: the viewer starts the fixture already knocked out
    /// (21 commander damage), for the "You're out" panel and the standings.
    pub viewer_out: bool,
    /// `--hold-seat N`: seat N is a connected player who never acts, so the
    /// game waits on them — a network pod that goes on without the viewer.
    pub hold_seat: Option<usize>,
    /// `--deck-picker`: stay on the menu with Commander selected and the deck
    /// picker open; the screenshot is of the menu, not a match.
    pub deck_picker: bool,
}

impl HarnessArgs {
    pub fn parse(args: &[String]) -> Self {
        let value = |flag: &str| {
            args.windows(2).find_map(|w| (w[0] == flag).then(|| w[1].clone()))
        };
        HarnessArgs {
            fixture_seats: value("--layout-fixture")
                .and_then(|v| v.parse().ok())
                .map(|n: usize| n.clamp(2, 4)),
            screenshot: value("--screenshot").map(std::path::PathBuf::from),
            screenshot_delay: value("--screenshot-delay")
                .and_then(|v| v.parse().ok())
                .unwrap_or(8.0),
            window: value("--window").and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            }),
            settings_open: args.iter().any(|a| a == "--settings-open"),
            viewer_out: args.iter().any(|a| a == "--viewer-out"),
            hold_seat: value("--hold-seat").and_then(|v| v.parse().ok()),
            deck_picker: args.iter().any(|a| a == "--deck-picker"),
        }
    }
}

/// Card names a fixture seat plays, by row. Looked up by name so a rename in
/// the catalog drops a card rather than breaking the harness.
const LANDS: &[&str] = &[
    "Forest", "Forest", "Forest", "Mountain", "Mountain", "Stomping Ground",
    "Wooded Foothills", "Island",
];
const CREATURES: &[&str] = &[
    "Llanowar Elves", "Grizzly Bears", "Tarmogoyf", "Serra Angel", "Shivan Dragon",
    "Birds of Paradise",
];
const OTHER_PERMANENTS: &[&str] = &["Sol Ring", "Oblivion Ring"];
const HAND: &[&str] = &[
    "Lightning Bolt", "Counterspell", "Wrath of God", "Grizzly Bears", "Forest",
    "Serra Angel", "Shivan Dragon",
];
const GRAVEYARD: &[&str] = &["Lightning Bolt", "Grizzly Bears", "Counterspell", "Forest"];

fn defs(names: &[&str]) -> Vec<crabomination::card::CardDefinition> {
    names.iter().filter_map(|n| crabomination::catalog::lookup_by_name(n)).collect()
}

/// A mid-game board for `seats` players (2 = a Modern 1v1, 3-4 = a Commander
/// pod), every seat with a full board, paused in seat 0's precombat main
/// phase with seat 0 holding priority — the match waits on the human, so the
/// table holds still for a screenshot.
pub fn fixture_state(seats: usize) -> GameState {
    let mut g = if seats <= 2 {
        crabomination::demo::build_demo_state_seeded(7)
    } else {
        let decks = crabomination::pod::pod_field(seats);
        let stock: Vec<crabomination::pod::SeatDeck<'_>> = decks
            .iter()
            .map(|d| crabomination::pod::SeatDeck { commanders: d.commanders, main: d.main })
            .collect();
        crabomination::demo::build_commander_pod_seeded(&stock, 7)
    };
    for seat in 0..g.players.len() {
        g.players[seat].name = if seat == 0 { "You".into() } else { format!("Bot {seat}") };
        for def in defs(LANDS) {
            g.add_card_to_battlefield(seat, def);
        }
        for (i, def) in defs(CREATURES).into_iter().enumerate() {
            let id = g.add_card_to_battlefield(seat, def);
            g.clear_sickness(id);
            // A couple tapped, so the rotated-card spacing shows.
            if i % 3 == 1
                && let Some(c) = g.battlefield_find_mut(id)
            {
                c.tapped = true;
            }
        }
        for def in defs(OTHER_PERMANENTS) {
            g.add_card_to_battlefield(seat, def);
        }
        let hand = if seat == 0 { HAND.len() } else { 5 };
        for def in defs(&HAND[..hand]) {
            g.add_card_to_hand(seat, def);
        }
        for def in defs(GRAVEYARD) {
            g.add_card_to_graveyard(seat, def);
        }
    }
    for def in defs(&["Lightning Bolt", "Grizzly Bears"]) {
        g.add_card_to_exile(1, def);
    }
    g.turn_number = 6;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// `--viewer-out`: knock seat 0 out of the fixture with 21 damage from
/// seat 1's commander, and hand the turn to seat 1.
pub fn knock_out_viewer(g: &mut GameState) {
    let Some(&commander) = g.players.get(1).and_then(|p| p.commanders.first()) else { return };
    g.commander_damage.insert((0, commander), 21);
    g.check_state_based_actions();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
}

/// `--deck-picker`: select Commander and open the deck picker, once.
pub fn open_deck_picker_for_screenshot(
    args: Res<HarnessArgs>,
    mut fields: ResMut<crate::menu::MenuFields>,
    mut picker: ResMut<crate::deck_picker::DeckPicker>,
    mut done: Local<bool>,
) {
    if args.deck_picker && !*done {
        fields.select_format(crate::menu::MatchFormat::Commander);
        picker.open = true;
        *done = true;
    }
}

/// `--settings-open`: open the Esc menu the first frame a view is up.
pub fn open_settings_for_screenshot(
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    mut settings: ResMut<crate::systems::quality::SettingsOpen>,
    mut done: Local<bool>,
) {
    if args.settings_open && !*done && view.0.is_some() {
        settings.0 = true;
        *done = true;
    }
}

/// Seconds since the first view arrived; `None` until then.
#[derive(Resource, Default)]
pub struct ScreenshotClock {
    since_view: Option<f32>,
    taken_at: Option<f32>,
}

/// Save the primary window to `--screenshot` once the view has been up for
/// the delay, then exit a moment later (the capture lands a frame or two
/// after the request).
pub fn capture_screenshot(
    mut commands: Commands,
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    time: Res<Time>,
    mut clock: ResMut<ScreenshotClock>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = &args.screenshot else { return };
    // A match screenshot waits for the first view; a menu one does not.
    if view.0.is_none() && !args.deck_picker {
        return;
    }
    let t = clock.since_view.get_or_insert(0.0);
    *t += time.delta_secs();
    let now = *t;
    match clock.taken_at {
        None if now >= args.screenshot_delay => {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
            clock.taken_at = Some(now);
        }
        Some(at) if now - at > 2.0 => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_flags_parse() {
        let args: Vec<String> = ["--layout-fixture", "4", "--window", "2560x1080", "--screenshot", "/tmp/a.png"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let h = HarnessArgs::parse(&args);
        assert_eq!(h.fixture_seats, Some(4));
        assert_eq!(h.window, Some((2560, 1080)));
        assert_eq!(h.screenshot.as_deref(), Some(std::path::Path::new("/tmp/a.png")));
        assert_eq!(h.screenshot_delay, 8.0);
        assert!(HarnessArgs::parse(&[]).fixture_seats.is_none());
        let out: Vec<String> = ["--viewer-out", "--hold-seat", "1"].iter().map(|s| s.to_string()).collect();
        let h = HarnessArgs::parse(&out);
        assert!(h.viewer_out);
        assert_eq!(h.hold_seat, Some(1));
    }

    #[test]
    fn fixture_boards_are_full_and_paused_on_the_viewer() {
        for seats in [2, 4] {
            let g = fixture_state(seats);
            assert_eq!(g.players.len(), seats);
            for p in 0..seats {
                let on_board = g.battlefield.iter().filter(|c| c.controller == p).count();
                assert!(on_board >= 14, "seat {p} of {seats}: {on_board} permanents");
                assert!(!g.players[p].hand.is_empty());
            }
            assert_eq!((g.step, g.active_player_idx), (TurnStep::PreCombatMain, 0));
        }
    }
}
