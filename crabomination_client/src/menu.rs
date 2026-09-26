//! Pre-game main menu.
//!
//! Lets the user pick how to start the match:
//!
//! - **Play vs Bot** — in-process server, HeuristicBot opponent (no network).
//! - **Host LAN Game** — spawn a TCP listener; local player joins via an
//!   in-process channel; second seat is filled by the next remote client to
//!   connect. (Use this + a second client running "Join" on another machine.)
//! - **Join LAN Game** — connect to a remote `addr:port`.
//!
//! The menu writes [`PendingNetMode`] and transitions to [`AppState::InGame`];
//! [`crate::net_plugin::start_net_session`] reads it on entry.
//!
//! A simple keyboard-driven text input lets the user edit the join address
//! and the host port; clicks on the field activate it.

#[cfg(not(target_arch = "wasm32"))]
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crabomination::cube::build_cube_state;
use crabomination::sos_mode::build_sos_state;
use crabomination::demo::{build_commander_state, build_demo_state};
use crabomination::game::GameState;
// Aliased: `bevy::prelude::Color` is the other `Color` in this file.
use crabomination::mana::{Color as MtgColor, ColorSet};
use crabomination::sos_mode::College;
#[cfg(not(target_arch = "wasm32"))]
use crabomination::server::{run_match, tcp_seat};
use crabomination::server::{
    ClientChannel, HeuristicBot, SeatOccupant, run_match_full, seat_pair,
    SnapshotSink, SnapshotSinkState,
};

use crate::net_plugin::{NetInbox, NetOutbox};

// ── State + resources ────────────────────────────────────────────────────────

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    /// Audit-mode card picker. Selecting a card writes its name to
    /// `AuditTarget` and transitions into `InGame` with an
    /// audit-tailored state (see `crate::audit::build_audit_state`).
    Audit,
    /// 8-seat booster draft against 7 bots. Owned by
    /// `crate::systems::draft::DraftPlugin`; cards are drawn from
    /// either the cube pool or the Secrets of Strixhaven pool
    /// depending on `PendingDraftFormat`. On completion the resulting
    /// `DraftedDecks` resource is consumed by `start_net_session` so
    /// the post-draft match plays out via the normal InGame path.
    Drafting,
    /// Connected to a lobby server: browse, create (choosing a gamemode), or
    /// join a lobby. Owned by `crate::systems::lobby_ui`. Transitions to
    /// `InGame` once the server sends `MatchStarted` (the net session is
    /// already installed, so `start_net_session_from_menu` is a no-op then).
    Lobby,
    InGame,
}

/// The lobby server the user chose to connect to, plus the display name to
/// announce. Set by the menu's "Join LAN" action and consumed by
/// `lobby_ui::connect_to_lobby_server` on entry to [`AppState::Lobby`].
#[derive(Resource, Default)]
pub struct PendingLobbyServer(pub Option<LobbyConnect>);

/// A queued lobby connection request.
pub struct LobbyConnect {
    pub addr: String,
    pub name: String,
}

/// Set by the menu when the player picks "Draft"; read by the draft
/// plugin to choose which card pool the booster packs sample from.
/// Defaults to `Cube` so older save/restore paths still work.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingDraftFormat(pub MatchFormat);

impl Default for PendingDraftFormat {
    fn default() -> Self {
        Self(MatchFormat::Cube)
    }
}

/// Decks produced by `AppState::Drafting`'s opponent-select step. When
/// present at the `OnEnter(AppState::InGame)` boundary,
/// `start_net_session_from_menu` builds a 2-player match from these
/// decks via `crabomination::draft::build_draft_match_state` instead
/// of rolling a fresh random match. The resource is taken (drained) on
/// consumption so a follow-up rematch / new-game falls back to the
/// format's normal random path.
#[derive(Resource, Clone, Debug)]
pub struct DraftedDecks {
    pub player_deck: Vec<crabomination::cube::CardFactory>,
    pub opponent_deck: Vec<crabomination::cube::CardFactory>,
    pub opponent_label: String,
    /// CR 905.2b — the two seats' draft notes, carried into the match.
    pub notes: [crabomination::draft::DraftNotes; 2],
}

/// Filled in by the menu when the user picks an option; drained by
/// [`crate::net_plugin::start_net_session`] when entering `InGame`.
/// Carries the chosen format alongside the network mode so the in-game
/// match builder can pick the right deck pool.
#[derive(Resource, Default)]
pub struct PendingNetMode(pub Option<(NetMode, MatchFormat)>);

/// Inserted at startup with the value of `--load-state <path>`. When
/// `Some`, the menu auto-loads that file and skips straight into
/// inspection mode; otherwise the menu behaves normally.
#[derive(Resource, Default, Debug)]
pub struct CliBootHint(pub Option<std::path::PathBuf>);

/// Inserted at startup with the value of `--play <format>`. When `Some`,
/// the menu boots a local-bot match of that format directly (used to verify
/// format-specific layouts, e.g. the 4-player Commander table).
#[derive(Resource, Default, Debug)]
pub struct CliBootFormat(pub Option<MatchFormat>);

#[derive(Clone, Debug)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))] // HostLan{port} unused in browser
pub enum NetMode {
    /// In-process server, HeuristicBot opponent.
    LocalBot,
    /// In-process server with two HeuristicBots; the local UI is a spectator.
    SpectateBots,
    /// Bind a TCP listener on `port`; pair the local in-process seat against
    /// the next remote client to connect.
    HostLan { port: u16 },
    /// Load a `<repo>/debug/state-*.json` snapshot and run the client in
    /// inspection mode (no live server; the HUD is read-only). Used for
    /// reproducing reported bugs from a saved state.
    LoadDebugState { path: std::path::PathBuf },
    /// `--layout-fixture <seats>`: a local match from the layout harness's
    /// fixed busy board (`layout_harness::fixture_state`).
    LayoutFixture { seats: usize },
    /// Re-claim a seat in a still-running match after a crash/restart,
    /// using the resume token persisted under
    /// `net_plugin::RESUME_STORAGE_KEY`. No session is spawned here —
    /// `maybe_reconnect` drives the connection.
    Rejoin { addr: String, token: String },
}

/// Which deck pool the match draws from. Modern uses the BRG / Goryo's
/// demo decks (`demo::build_demo_state`). Cube rolls a fresh random
/// 2-color deck per seat from the curated cube pools (`cube::build_cube_state`).
/// SoS rolls a random Strixhaven college per seat and builds a 60-card
/// deck from that college's ✅-only Secrets of Strixhaven cards
/// (`sos_mode::build_sos_state`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MatchFormat {
    #[default]
    Modern,
    Cube,
    Sos,
    /// Sealed: your hand-built 40 from `decks/sealed.txt` against a
    /// randomly generated sealed deck — the same pool generator and
    /// (repaired) builder the recommender races its candidates against,
    /// so a deck that tests well here is being tested against the same
    /// field the tool reports on.
    Sealed,
    /// Commander pod: 100-card singleton, 40 life, commanders seated in
    /// the command zone before the opening-hand draw. The menu's local
    /// matches seat a 2-4 player pod (`commander_pod_state`) with the
    /// human's imported deck or a stock one; a restart / spectate / hosted
    /// game is the stock 4-player `demo::build_commander_state`.
    Commander,
}

impl MatchFormat {
    /// Build a fresh `GameState` for this format. Public so the
    /// game-over "New Game" button can launch a follow-up match
    /// without going back through the menu UI.
    pub fn build_state_for_restart(self) -> GameState {
        self.build_state()
    }

    fn build_state(self) -> GameState {
        match self {
            MatchFormat::Modern => build_demo_state(),
            MatchFormat::Cube => build_cube_state(),
            MatchFormat::Sos => build_sos_state(),
            MatchFormat::Sealed => build_sealed_state(),
            MatchFormat::Commander => build_commander_state(),
        }
    }

    /// Parse a `--play <format>` CLI argument. Case-insensitive; returns
    /// `None` for an unrecognised name so the launch falls through to the menu.
    pub fn from_cli(s: &str) -> Option<MatchFormat> {
        match s.to_ascii_lowercase().as_str() {
            "modern" => Some(MatchFormat::Modern),
            "cube" => Some(MatchFormat::Cube),
            "sos" => Some(MatchFormat::Sos),
            "sealed" => Some(MatchFormat::Sealed),
            "commander" | "edh" => Some(MatchFormat::Commander),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MatchFormat::Modern => "Modern",
            MatchFormat::Cube => "Cube",
            MatchFormat::Sos => "SoS",
            MatchFormat::Sealed => "Sealed",
            MatchFormat::Commander => "Commander",
        }
    }

    /// Cycle to the next gamemode, for the lobby's format picker.
    pub fn next(self) -> MatchFormat {
        match self {
            MatchFormat::Modern => MatchFormat::Cube,
            MatchFormat::Cube => MatchFormat::Sos,
            MatchFormat::Sos => MatchFormat::Sealed,
            MatchFormat::Sealed => MatchFormat::Commander,
            MatchFormat::Commander => MatchFormat::Modern,
        }
    }

    /// Map to the wire gamemode for a `CreateLobby` request.
    pub fn to_lobby_format(self) -> crabomination::net::LobbyFormat {
        use crabomination::net::LobbyFormat as LF;
        match self {
            MatchFormat::Modern => LF::Modern,
            MatchFormat::Cube => LF::Cube,
            MatchFormat::Sos => LF::Sos,
            // No wire gamemode for Sealed: a hosted lobby can't see the
            // local deck file, so a Sealed lobby would silently deal a
            // different deck than the one being tested. Falls back to
            // SoS for lobby creation; Sealed is a local vs-bot mode.
            MatchFormat::Sealed => LF::Sos,
            MatchFormat::Commander => LF::Commander,
        }
    }
}

/// Where the Sealed mode reads your deck from: `$CRAB_SEALED_DECK` if
/// set, otherwise `<repo>/decks/sealed.txt`.
pub fn sealed_deck_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("CRAB_SEALED_DECK") {
        return resolve_deck_path(std::path::PathBuf::from(&p));
    }
    repo_root()
        .map(|r| r.join("decks").join("sealed.txt"))
        .unwrap_or_else(|| std::path::PathBuf::from("decks/sealed.txt"))
}

/// A relative path is resolved against the repo root as well as
/// the working directory. `cargo run` leaves the child's cwd
/// wherever the shell was, so `-p crabomination_client` from
/// inside the crate directory made `decks/foo.txt` point at
/// `crabomination_client/decks/foo.txt` — which does not exist,
/// and the seat quietly filled with a generated deck instead.
fn resolve_deck_path(given: std::path::PathBuf) -> std::path::PathBuf {
    if given.exists() || given.is_absolute() {
        return given;
    }
    if let Some(root) = repo_root() {
        let rooted = root.join(&given);
        if rooted.exists() {
            return rooted;
        }
    }
    given
}

/// The workspace root, found by walking up from this crate until a
/// `Cargo.lock` appears. `None` if the binary was moved somewhere with
/// no workspace above it.
fn repo_root() -> Option<std::path::PathBuf> {
    let mut dir: &std::path::Path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("Cargo.lock").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Sealed: your hand-built deck against a randomly generated one.
///
/// The opponent is drawn from the same generator the recommender's
/// gauntlet uses, so "how does my build do here" is the same question
/// the tool answers — just played by hand. A missing or unparseable deck
/// file is not fatal: the seat falls back to a generated build so the
/// mode always starts, with the reason on stderr.
fn build_sealed_state() -> GameState {
    use rand::RngExt;
    let mut rng = rand::rng();
    let seed: u64 = rng.random();
    let packs = opponent_pack_count();
    let colors = opponent_colors();
    // The opponent's seat: generated (the default) unless
    // `$CRAB_SEALED_OPP` names a decklist — set it to race two
    // constructed decks in the client. Naming a deck is always an
    // explicit request, so a deck that cannot be played is a hard
    // error, never a silent substitute (same rule as CRAB_SEALED_DECK
    // below); `--opponent-packs` and `--opponent-colors` describe a deck
    // to *generate*, so both are meaningless for a pinned deck and are
    // ignored — with a line saying so, because "only Lorehold" quietly
    // not applying is the failure this mode has to make visible.
    let (opponent, opp_label) = match std::env::var("CRAB_SEALED_OPP") {
        Ok(p) => {
            if !colors.is_empty() {
                eprintln!(
                    "sealed: CRAB_SEALED_OPP pins the opponent's deck — --opponent-colors ignored"
                );
            }
            let opp_path = resolve_deck_path(std::path::PathBuf::from(&p));
            let refuse_opp = |reason: &str| -> ! {
                eprintln!(
                    "\nsealed: cannot play opponent deck {} — {reason}.\n\
                     CRAB_SEALED_OPP named this deck, so no substitute is being made.\n\
                     Note relative paths resolve against the working directory or the repo root;\n\
                     `cargo run` keeps the shell's directory, so an absolute path is safest.\n",
                    opp_path.display()
                );
                std::process::exit(2)
            };
            let text = std::fs::read_to_string(&opp_path)
                .unwrap_or_else(|e| refuse_opp(&format!("{e}")));
            let parse = crabomination::decklist::parse_decklist(&text);
            for bad in &parse.unknown {
                eprintln!("sealed: opponent deck — skipping unrecognized line {bad:?}");
            }
            if parse.main.len() < 40 {
                refuse_opp(&format!(
                    "it has {} maindeck cards and a sealed deck needs 40",
                    parse.main.len()
                ));
            }
            let label = opp_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Opponent deck".into());
            (parse.main, label)
        }
        Err(_) => {
            let (deck, label) =
                crabomination::selfplay::random_sealed_opponent_in_colors(seed, packs, colors);
            // A restricted lattice can come back empty if the pool has no
            // playable card in the named colours at all. That is a 0-card
            // opponent, i.e. an unplayable match; say which knob caused it
            // rather than dealing the player an opening hand against nothing.
            if deck.len() < 40 {
                let what = if colors.is_empty() {
                    format!("a {packs}-pack pool")
                } else {
                    format!(
                        "the {} cards of a {packs}-pack pool",
                        crabomination::selfplay::color_identity_name(colors),
                    )
                };
                eprintln!(
                    "\nsealed: the generated opponent came out at {} cards — {what} cannot fill a\n\
                     40-card deck. Try more packs, or a different colour pair.\n",
                    deck.len(),
                );
                std::process::exit(2);
            }
            (deck, label)
        }
    };

    let path = sealed_deck_path();
    // Whether the player *asked* for this specific deck. Falling back to
    // a generated build is right for the default path (a bare checkout
    // should still be playable) and wrong for an explicit request: the
    // mode then plays a deck the user never chose, and the only notice
    // is one stderr line under Bevy's startup logs. Explicit requests
    // fail loudly instead.
    let explicit = std::env::var_os("CRAB_SEALED_DECK").is_some();
    let refuse = |reason: &str| -> ! {
        eprintln!(
            "\nsealed: cannot play {} — {reason}.\n\
             CRAB_SEALED_DECK named this deck, so no substitute is being made.\n\
             Note relative paths resolve against the working directory or the repo root;\n\
             `cargo run` keeps the shell's directory, so an absolute path is safest.\n",
            path.display()
        );
        std::process::exit(2)
    };
    let player = match std::fs::read_to_string(&path) {
        Ok(text) => {
            let parse = crabomination::decklist::parse_decklist(&text);
            for bad in &parse.unknown {
                eprintln!("sealed: skipping unrecognized line {bad:?}");
            }
            if parse.main.len() < 40 {
                if explicit {
                    refuse(&format!(
                        "it has {} maindeck cards and a sealed deck needs 40",
                        parse.main.len()
                    ));
                }
                eprintln!(
                    "sealed: {} has {} maindeck cards (want 40) — filling the seat with a generated deck",
                    path.display(),
                    parse.main.len()
                );
                crabomination::selfplay::random_sealed_opponent(seed ^ 0xF00D).0
            } else {
                eprintln!("sealed: playing {} ({} cards) vs {opp_label}", path.display(), parse.main.len());
                parse.main
            }
        }
        Err(e) => {
            if explicit {
                refuse(&format!("{e}"));
            }
            eprintln!("sealed: {} unreadable ({e}) — using a generated deck", path.display());
            crabomination::selfplay::random_sealed_opponent(seed ^ 0xF00D).0
        }
    };
    let mut state =
        crabomination::draft::build_draft_match_state(player, opponent, "You".into(), opp_label);
    // CR 103.5 — someone wins the die roll. `GameState::new` always seats
    // player 0, which handed the human the play in every sealed game.
    let starting = if rng.random::<bool>() { 0 } else { 1 };
    state.set_starting_player(starting);
    eprintln!(
        "sealed: {} on the play",
        if starting == 0 { "you are" } else { "the opponent is" },
    );
    state
}

/// `--opponent-packs N` — how many boosters the bot's sealed pool is
/// opened from. Six is a standard pool; more cards means a better 40 out
/// of the same builder, which is the difficulty dial. Clamped to a sane
/// range so a typo can't spend a minute generating a pool.
fn opponent_pack_count() -> usize {
    opponent_pack_count_from(std::env::args().skip(1))
}

const MIN_OPPONENT_PACKS: usize = 1;
const MAX_OPPONENT_PACKS: usize = 60;

fn opponent_pack_count_from(args: impl Iterator<Item = String>) -> usize {
    args.collect::<Vec<_>>()
        .windows(2)
        .find_map(|w| (w[0] == "--opponent-packs").then(|| w[1].parse::<usize>().ok()))
        .flatten()
        .map(|n| {
            let clamped = n.clamp(MIN_OPPONENT_PACKS, MAX_OPPONENT_PACKS);
            if clamped != n {
                eprintln!("sealed: --opponent-packs {n} clamped to {clamped}");
            }
            clamped
        })
        .unwrap_or(crabomination::selfplay::SEALED_PACKS)
}

/// `--opponent-colors <spec>` — pin the generated sealed opponent to one
/// colour identity, so the seat across the table is always the same deck
/// archetype ("let me practise this build against Lorehold"). Absent, the
/// builder plays whichever colours its pool is deepest in.
///
/// A spec that doesn't parse **exits** rather than falling back to an
/// unrestricted opponent. `--opponent-packs` can degrade quietly because a
/// wrong pack count is visible in the game that follows; a wrong colour
/// spec is not — the player gets a deck of some other colour that looks
/// exactly like the one they asked for, and the whole point of the flag is
/// the guarantee. Same rule the pinned-deck paths above follow.
fn opponent_colors() -> ColorSet {
    match opponent_colors_from(std::env::args().skip(1)) {
        Ok(colors) => colors,
        Err(why) => {
            eprintln!("\nsealed: {why}\n");
            std::process::exit(2)
        }
    }
}

const OPPONENT_COLORS_ARG: &str = "--opponent-colors";

/// [`opponent_colors`] over an explicit argument list. `Ok(ColorSet::empty())`
/// when the flag is absent — the unrestricted default.
fn opponent_colors_from(args: impl Iterator<Item = String>) -> Result<ColorSet, String> {
    let args: Vec<String> = args.collect();
    let Some(at) = args.iter().position(|a| a == OPPONENT_COLORS_ARG) else {
        return Ok(ColorSet::empty());
    };
    let spec = args
        .get(at + 1)
        .filter(|s| !s.starts_with("--"))
        .ok_or_else(|| format!("{OPPONENT_COLORS_ARG} needs a value. {SPEC_HELP}"))?;
    parse_color_identity(spec)
}

const SPEC_HELP: &str = "Give a college (lorehold, prismari, quandrix, silverquill, \
                         witherbloom) or two-or-more WUBRG letters (rw, wubr).";

/// A colour-identity spec: a Strixhaven college name, or WUBRG letters with
/// `/`, `,`, `-` and spaces allowed as separators — so `lorehold`, `rw`,
/// `WR` and `r/w` are one set, and the sealed pool's five colleges are
/// nameable as themselves.
fn parse_color_identity(spec: &str) -> Result<ColorSet, String> {
    if let Some(college) = College::from_name(spec) {
        return Ok(college.colors().iter().collect());
    }
    let mut colors = ColorSet::empty();
    for ch in spec.chars().filter(|c| !"/,-_ \t".contains(*c)) {
        colors.insert(match ch.to_ascii_uppercase() {
            'W' => MtgColor::White,
            'U' => MtgColor::Blue,
            'B' => MtgColor::Black,
            'R' => MtgColor::Red,
            'G' => MtgColor::Green,
            _ => {
                return Err(format!(
                    "{OPPONENT_COLORS_ARG} {spec:?} is not a colour identity. {SPEC_HELP}"
                ));
            }
        });
    }
    // The sealed builder enumerates pairs and wider — it has no mono-colour
    // shape to offer — so one colour would silently produce no deck at all.
    if colors.len() < 2 {
        return Err(format!(
            "{OPPONENT_COLORS_ARG} {spec:?} names {} colour(s); the sealed builder only \
             builds two-or-more-colour decks. {SPEC_HELP}",
            colors.len(),
        ));
    }
    Ok(colors)
}

/// Active text-edit field in the menu.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
enum FocusedField {
    #[default]
    None,
    PlayerName,
    HostPort,
    JoinAddr,
    DeckPath,
}

#[derive(Resource)]
pub(crate) struct MenuFields {
    pub(crate) player_name: String,
    host_port: String,
    join_addr: String,
    /// Path to a plain-text decklist (Arena / MTGO format) for the
    /// "Play Deck vs Bot" import flow.
    /// The deck-file field; a hosted Commander lobby submits this list too.
    pub(crate) deck_path: String,
    focused: FocusedField,
    format: MatchFormat,
    /// Commander only: seats in the local pod, human included (2-4).
    pub(crate) pod_size: usize,
    /// Commander only: which stock decks the bots play.
    pub(crate) pod_opponents: PodOpponents,
}

/// The Commander pod sizes the menu offers, human seat included.
const POD_SIZES: [usize; 3] = [2, 3, 4];

/// The next pod size in the menu's cycle (2 → 3 → 4 → 2).
fn next_pod_size(n: usize) -> usize {
    let i = POD_SIZES.iter().position(|&s| s == n).unwrap_or(POD_SIZES.len() - 1);
    POD_SIZES[(i + 1) % POD_SIZES.len()]
}

/// Which stock Commander decks ([`crabomination::pod::target_decks`]) the
/// bots in a local pod play.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum PodOpponents {
    /// A different deck per bot, drawn at random each match.
    #[default]
    Random,
    /// Every bot plays `target_decks()[i]`.
    Deck(usize),
}

impl PodOpponents {
    /// Cycle Random → deck 0 → deck 1 → … → Random over `n_decks` decks.
    fn next(self, n_decks: usize) -> PodOpponents {
        match self {
            PodOpponents::Random if n_decks > 0 => PodOpponents::Deck(0),
            PodOpponents::Deck(i) if i + 1 < n_decks => PodOpponents::Deck(i + 1),
            _ => PodOpponents::Random,
        }
    }

    fn label(self) -> String {
        match self {
            PodOpponents::Random => "Opponents: Random".to_string(),
            PodOpponents::Deck(i) => {
                let decks = crabomination::pod::target_decks();
                let name = decks.get(i).map_or("?", |d| d.name);
                format!("Opponents: {name}")
            }
        }
    }
}

/// The `count` bot decks for a local Commander pod. `Random` deals distinct
/// decks (the field has eight, the pod at most three bots) off `seed`.
fn commander_opponents(
    choice: PodOpponents,
    count: usize,
    seed: u64,
) -> Vec<crabomination::pod::PodDeck> {
    use rand::SeedableRng;
    use rand::seq::SliceRandom;
    let mut field = crabomination::pod::target_decks();
    match choice {
        PodOpponents::Deck(i) if i < field.len() => vec![field[i]; count],
        _ => {
            field.shuffle(&mut rand::rngs::StdRng::seed_from_u64(seed));
            (0..count).map(|i| field[i % field.len()]).collect()
        }
    }
}

impl Default for MenuFields {
    fn default() -> Self {
        // Text fields persist across launches via the config file; empty
        // config values fall back to the out-of-the-box defaults.
        let saved = crate::config::load().gameplay;
        let player_name = if saved.player_name.trim().is_empty() {
            // Seeded from the OS username so it's meaningful out of the box.
            default_player_name()
        } else {
            saved.player_name.chars().take(MAX_PLAYER_NAME_LEN).collect()
        };
        let join_addr = if saved.join_addr.trim().is_empty() {
            std::env::var("CRAB_SERVER").unwrap_or_else(|_| "127.0.0.1:7777".to_string())
        } else {
            saved.join_addr
        };
        let deck_path = if saved.deck_path.trim().is_empty() {
            "deck.txt".to_string()
        } else {
            saved.deck_path
        };
        Self {
            player_name,
            // Default to the same port the standalone `crabomination_server`
            // binary uses, so a remote client can run the standalone server
            // on the same port and these defaults work out-of-the-box.
            host_port: "7777".to_string(),
            join_addr,
            deck_path,
            focused: FocusedField::None,
            format: MatchFormat::default(),
            pod_size: 4,
            pod_opponents: PodOpponents::default(),
        }
    }
}

/// Maximum length of a player's display name (clamped on input and trimmed
/// before it's sent to the lobby server).
const MAX_PLAYER_NAME_LEN: usize = 20;

/// Seed the player name from the OS username, falling back to "Player".
fn default_player_name() -> String {
    let raw = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed.chars().take(MAX_PLAYER_NAME_LEN).collect()
    }
}

// ── Marker components ────────────────────────────────────────────────────────

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct PlayBotButton;

/// "Play Deck vs Bot" — reads the decklist file named in the DeckPath
/// field, validates it against the catalog, and starts a local-bot match
/// with the imported deck.
#[derive(Component)]
struct ImportDeckButton;

/// Feedback line under the import controls ("12 cards unknown: …",
/// "deck.txt not found", "Imported 60 cards").
#[derive(Component)]
struct MenuStatusText;

/// Live card-art download progress ("Downloading card art… 120/1500"),
/// fed by the background prefetch thread's `ImagePrefetch` counters.
#[derive(Component)]
struct DownloadProgressText;

/// Current menu feedback message (import errors, validation results,
/// connection failures — also set from `lobby_ui` when a connect bounces
/// the player back to the menu).
#[derive(Resource, Default)]
pub struct MenuStatus(pub String);

/// A successfully imported deck, consumed by `spawn_inprocess_bot`
/// (it outranks the format's stock decks, like `DraftedDecks`).
/// `commanders` is filled only by a Commander-mode import, which seats the
/// deck in a pod instead of a two-player match.
#[derive(Resource, Clone)]
pub struct ImportedDeck {
    pub main: Vec<crabomination::cube::CardFactory>,
    pub commanders: Vec<crabomination::cube::CardFactory>,
}

/// The Commander pod options row — hidden unless Commander is selected.
#[derive(Component)]
struct PodOptionsRow;

/// Cycles the pod size (2 / 3 / 4 players).
#[derive(Component)]
struct PodSizeButton;

/// Cycles the bots' decks (Random / each stock pod deck).
#[derive(Component)]
struct PodOpponentsButton;

/// Label of one of the pod cycling buttons.
#[derive(Component)]
enum PodLabel {
    Size,
    Opponents,
}

#[derive(Component)]
struct SpectateBotsButton;

#[derive(Component)]
struct LoadDebugStateButton;

#[derive(Component)]
struct AuditCardsButton;

#[derive(Component)]
struct DraftButton;

#[derive(Component)]
struct HostButton;

#[derive(Component)]
struct JoinButton;

/// "Rejoin Last Match" — shown only when a persisted resume token survived
/// a crash (see `net_plugin::RESUME_STORAGE_KEY`).
#[derive(Component)]
struct RejoinButton;

/// Parse the persisted `addr\ntoken\nformat` rejoin entry, if any.
fn load_persisted_resume() -> Option<(String, String, MatchFormat)> {
    let raw = crate::storage::load(crate::net_plugin::RESUME_STORAGE_KEY)?;
    let mut lines = raw.lines();
    let addr = lines.next()?.trim().to_string();
    let token = lines.next()?.trim().to_string();
    if addr.is_empty() || token.is_empty() {
        return None;
    }
    let format = match lines.next().unwrap_or_default().trim() {
        "Cube" => MatchFormat::Cube,
        "Sos" => MatchFormat::Sos,
        "Commander" => MatchFormat::Commander,
        _ => MatchFormat::Modern,
    };
    Some((addr, token, format))
}

#[derive(Component)]
struct FieldButton(FocusedField);

#[derive(Component)]
struct FieldText(FocusedField);

#[derive(Component)]
struct FormatToggleButton(MatchFormat);

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<PendingNetMode>()
            .init_resource::<PendingDraftFormat>()
            .init_resource::<PendingLobbyServer>()
            .init_resource::<MenuFields>()
            .init_resource::<MenuStatus>()
            .init_resource::<CliBootHint>()
            .init_resource::<CliBootFormat>()
            .add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(OnExit(AppState::Menu), despawn_menu)
            .add_systems(
                Update,
                (
                    handle_field_focus,
                    handle_text_input,
                    refresh_field_text,
                    handle_format_toggle,
                    refresh_format_toggle_visuals,
                    handle_pod_toggles,
                    refresh_pod_options,
                    handle_action_buttons,
                    refresh_menu_status,
                    update_download_progress,
                    apply_cli_boot_hint,
                )
                    .run_if(in_state(AppState::Menu)),
            );
    }
}

/// One-shot system: when the client was launched with `--load-state <path>`,
/// fire the load mode immediately and transition into InGame on the next
/// frame, bypassing the menu UI entirely.
fn apply_cli_boot_hint(
    mut hint: ResMut<CliBootHint>,
    mut boot_format: ResMut<CliBootFormat>,
    mut pending: ResMut<PendingNetMode>,
    mut next_state: ResMut<NextState<AppState>>,
    fields: Res<MenuFields>,
    mut harness: ResMut<crate::layout_harness::HarnessArgs>,
) {
    // `--play <format>` boots a local-bot match of that format directly.
    if let Some(format) = boot_format.0.take() {
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::InGame);
        return;
    }
    if let Some(seats) = harness.fixture_seats.take() {
        let format = if seats > 2 { MatchFormat::Commander } else { MatchFormat::Modern };
        pending.0 = Some((NetMode::LayoutFixture { seats }, format));
        next_state.set(AppState::InGame);
        return;
    }
    let Some(path) = hint.0.take() else { return };
    pending.0 = Some((NetMode::LoadDebugState { path }, fields.format));
    next_state.set(AppState::InGame);
}

// ── UI setup ─────────────────────────────────────────────────────────────────

use crate::theme::{
    self, HoverTint, UiFonts, BUTTON_ACCENT_BG, BUTTON_DANGER_BG, BUTTON_INFO_BG,
    BUTTON_PRIMARY_BG, BUTTON_WARN_BG, FIELD_BG, FIELD_BG_FOCUSED, PANEL_BG,
    RADIUS_BUTTON, RADIUS_PANEL,
};

fn spawn_menu(mut commands: Commands, ui_fonts: Res<UiFonts>) {
    let tf = |size: f32| ui_fonts.tf(size);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG),
            MenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(18.0),
                    align_items: AlignItems::Center,
                    // Fixed width: the status / download-progress lines below
                    // change length as they update, and a fit-content panel
                    // would visibly resize with them.
                    width: Val::Px(560.0),
                    border_radius: BorderRadius::all(RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("Crabomination"),
                    tf(28.0),
                    TextColor(theme::ACCENT_GOLD),
                ));

                // Format selector — Modern (BRG / Goryo's demo decks) vs
                // Cube (random 2-color deck per seat).
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|fmt| {
                    fmt.spawn((
                        Text::new("Format"),
                        tf(13.0),
                        TextColor(theme::TEXT_BODY),
                    ));
                    fmt.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        format_toggle(row, &tf, MatchFormat::Modern);
                        format_toggle(row, &tf, MatchFormat::Cube);
                        format_toggle(row, &tf, MatchFormat::Sos);
                        format_toggle(row, &tf, MatchFormat::Sealed);
                        format_toggle(row, &tf, MatchFormat::Commander);
                    });
                    // Commander pod options — shown only while Commander is
                    // the selected format (`refresh_pod_options`). They
                    // apply to both "Play vs Bot" and "Play Deck vs Bot".
                    fmt.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            display: Display::None,
                            ..default()
                        },
                        PodOptionsRow,
                    ))
                    .with_children(|row| {
                        pod_toggle(row, &tf, PodSizeButton, PodLabel::Size);
                        pod_toggle(row, &tf, PodOpponentsButton, PodLabel::Opponents);
                    });
                });

                // Rejoin — offered only when a crash left a persisted resume
                // token behind (a clean exit clears it).
                if let Some((addr, _, _)) = load_persisted_resume() {
                    button(
                        p,
                        &tf,
                        &format!("Rejoin Last Match ({addr})"),
                        BUTTON_WARN_BG,
                        RejoinButton,
                    );
                }

                // Everything driven by an in-process match thread (vs Bot,
                // Draft, Spectate, Audit) or the local filesystem (debug
                // state) is native-only: wasm has no threads to run the
                // match on, so the browser build is online (lobby) play.
                let native = cfg!(not(target_arch = "wasm32"));

                // Play vs Bot
                if native {
                    button(p, &tf, "Play vs Bot", BUTTON_PRIMARY_BG, PlayBotButton);

                    // Draft — opens the 8-seat booster draft for the
                    // selected format (Cube or SoS). Modern / Commander
                    // fall back to the Cube pool.
                    button(p, &tf, "Draft (Cube / SoS)", BUTTON_INFO_BG, DraftButton);

                    // Spectate Bot vs Bot
                    button(
                        p,
                        &tf,
                        "Spectate Bot vs Bot",
                        BUTTON_ACCENT_BG,
                        SpectateBotsButton,
                    );

                    // Load Debug State (most recent file in <repo>/debug/)
                    button(
                        p,
                        &tf,
                        "Load Latest Debug State",
                        BUTTON_DANGER_BG,
                        LoadDebugStateButton,
                    );

                    // Audit Cards — opens the card picker for verifying
                    // individual card implementations one-by-one.
                    button(
                        p,
                        &tf,
                        "Audit Cards",
                        BUTTON_ACCENT_BG,
                        AuditCardsButton,
                    );
                }

                // Settings — window mode / resolution, quality, gameplay.
                button(
                    p,
                    &tf,
                    "Settings",
                    theme::BUTTON_NEUTRAL_BG,
                    crate::systems::settings_menu::OpenSettingsButton,
                );

                // Import a decklist (Arena / MTGO text format) and play
                // it against the bot. The field holds a file path; status
                // feedback (unknown cards, size problems) renders below.
                // (Native-only: file path + in-process match.)
                if native { p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|imp| {
                    button(imp, &tf, "Play Deck vs Bot", BUTTON_PRIMARY_BG, ImportDeckButton);
                    field(imp, &tf, "Deck file:", FocusedField::DeckPath);
                }); }

                // Display name (shown to other players in lobbies).
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|name| {
                    field(name, &tf, "Name:", FocusedField::PlayerName);
                });

                // Host LAN (native-only: browsers can't listen on TCP).
                if native { p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|host| {
                    button(host, &tf, "Host LAN Game", BUTTON_INFO_BG, HostButton);
                    field(
                        host,
                        &tf,
                        "Port:",
                        FocusedField::HostPort,
                    );
                }); }

                // Join LAN
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(280.0),
                    ..default()
                })
                .with_children(|join| {
                    button(join, &tf, "Join LAN Game", BUTTON_WARN_BG, JoinButton);
                    field(
                        join,
                        &tf,
                        "Server:",
                        FocusedField::JoinAddr,
                    );
                });

                p.spawn((
                    Text::new("Click a text field to edit. Backspace deletes, Ctrl+V pastes."),
                    tf(11.0),
                    TextColor(theme::TEXT_PLACEHOLDER),
                ));

                // Both live-updating lines are capped to the panel's inner
                // width so an extra-long message wraps instead of widening
                // the (fixed-width) panel.
                p.spawn((
                    Text::new(""),
                    tf(12.0),
                    TextColor(theme::ACCENT_ORANGE),
                    Node { max_width: Val::Px(504.0), ..default() },
                    MenuStatusText,
                ));

                p.spawn((
                    Text::new(""),
                    tf(11.0),
                    TextColor(theme::TEXT_SECONDARY),
                    Node { max_width: Val::Px(504.0), ..default() },
                    DownloadProgressText,
                ));
            });
        });
}

fn button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    label: &str,
    bg: Color,
    marker: M,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(bg),
            HoverTint::new(bg),
            marker,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                tf(16.0),
                TextColor(theme::TEXT_PRIMARY),
                Pickable::IGNORE,
            ));
        });
}

fn field(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    label: &str,
    which: FocusedField,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                tf(13.0),
                TextColor(theme::TEXT_BODY),
            ));
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(FIELD_BG),
                FieldButton(which),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(""),
                    tf(13.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                    FieldText(which),
                ));
            });
        });
}

fn format_toggle(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    format: MatchFormat,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(FIELD_BG),
            FormatToggleButton(format),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(format.label()),
                tf(13.0),
                TextColor(theme::TEXT_PRIMARY),
                Pickable::IGNORE,
            ));
        });
}

/// A cycling toggle in the `format_toggle` style; its label is written by
/// `refresh_pod_options`.
fn pod_toggle<M: Component>(
    parent: &mut ChildSpawnerCommands,
    tf: &impl Fn(f32) -> TextFont,
    marker: M,
    label: PodLabel,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(FIELD_BG),
            HoverTint::new(FIELD_BG),
            marker,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(""),
                tf(13.0),
                TextColor(theme::TEXT_PRIMARY),
                Pickable::IGNORE,
                label,
            ));
        });
}

fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MenuRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ── Input handling ───────────────────────────────────────────────────────────

fn handle_field_focus(
    mut fields: ResMut<MenuFields>,
    mut buttons: Query<(&Interaction, &FieldButton, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, fb, mut bg) in &mut buttons {
        if *interaction == Interaction::Pressed {
            fields.focused = fb.0;
        }
        let on = fields.focused == fb.0 && fb.0 != FocusedField::None;
        *bg = BackgroundColor(if on { FIELD_BG_FOCUSED } else { FIELD_BG });
    }
}

fn handle_text_input(
    mut fields: ResMut<MenuFields>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<bevy::clipboard::Clipboard>,
    mut status: ResMut<MenuStatus>,
    mut config: ResMut<crate::config::ConfigStore>,
) {
    if fields.focused == FocusedField::None {
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut next_focused = fields.focused;
    let mut buf = match fields.focused {
        FocusedField::PlayerName => fields.player_name.clone(),
        FocusedField::HostPort => fields.host_port.clone(),
        FocusedField::JoinAddr => fields.join_addr.clone(),
        FocusedField::DeckPath => fields.deck_path.clone(),
        FocusedField::None => return,
    };
    let max_len = match fields.focused {
        FocusedField::PlayerName => MAX_PLAYER_NAME_LEN,
        FocusedField::HostPort => 5,
        FocusedField::JoinAddr => 80,
        FocusedField::DeckPath => 160,
        FocusedField::None => return,
    };
    let mut changed = false;
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Backspace => {
                buf.pop();
                changed = true;
            }
            Key::Enter | Key::Escape => {
                next_focused = FocusedField::None;
            }
            Key::Character(s) => {
                // Ctrl+V pastes the clipboard through the same per-field
                // character filter typing goes through; other Ctrl chords
                // are swallowed so the shortcut letter isn't typed.
                if ctrl {
                    if s.as_str().eq_ignore_ascii_case("v") {
                        // Desktop reads resolve immediately (wasm text entry
                        // uses this same path, where a still-pending read is
                        // simply dropped — retry the paste a moment later).
                        if let Some(Ok(text)) = clipboard.fetch_text().poll_result() {
                            for ch in text.trim().chars() {
                                if buf.len() >= max_len {
                                    break;
                                }
                                if accepts_char(fields.focused, ch) {
                                    buf.push(ch);
                                    changed = true;
                                }
                            }
                        }
                    }
                    continue;
                }
                for ch in s.chars() {
                    if buf.len() >= max_len {
                        break;
                    }
                    if accepts_char(fields.focused, ch) {
                        buf.push(ch);
                        changed = true;
                    }
                }
            }
            _ => {}
        }
    }
    if changed {
        // Editing any field invalidates whatever error the last action
        // reported (bad path, invalid port…) — don't let it go stale.
        status.0.clear();
        match fields.focused {
            FocusedField::PlayerName => fields.player_name = buf,
            FocusedField::HostPort => fields.host_port = buf,
            FocusedField::JoinAddr => fields.join_addr = buf,
            FocusedField::DeckPath => fields.deck_path = buf,
            FocusedField::None => {}
        }
        // Persist the edited field so it survives relaunches. Writes go
        // through `ConfigStore` (the live document) — editing the file
        // directly would be undone by the next store-driven save.
        match fields.focused {
            FocusedField::PlayerName => {
                let v = fields.player_name.clone();
                crate::config::update_store(&mut config, |c| c.gameplay.player_name = v);
            }
            FocusedField::JoinAddr => {
                let v = fields.join_addr.clone();
                crate::config::update_store(&mut config, |c| c.gameplay.join_addr = v);
            }
            FocusedField::DeckPath => {
                let v = fields.deck_path.clone();
                crate::config::update_store(&mut config, |c| c.gameplay.deck_path = v);
            }
            _ => {}
        }
    }
    fields.focused = next_focused;
}

fn accepts_char(field: FocusedField, ch: char) -> bool {
    match field {
        // Letters, digits, spaces, and a few name-safe punctuation marks.
        FocusedField::PlayerName => {
            ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_' | '.')
        }
        FocusedField::HostPort => ch.is_ascii_digit(),
        FocusedField::JoinAddr => {
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-' | '_')
        }
        // Filesystem paths: also slashes, ~, spaces.
        FocusedField::DeckPath => {
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '\\' | '~' | '-' | '_' | ' ')
        }
        FocusedField::None => false,
    }
}

fn refresh_field_text(
    fields: Res<MenuFields>,
    mut q: Query<(&FieldText, &mut Text)>,
) {
    if !fields.is_changed() {
        return;
    }
    for (which, mut t) in &mut q {
        let value = match which.0 {
            FocusedField::PlayerName => &fields.player_name,
            FocusedField::HostPort => &fields.host_port,
            FocusedField::JoinAddr => &fields.join_addr,
            FocusedField::DeckPath => &fields.deck_path,
            FocusedField::None => continue,
        };
        let cursor = if fields.focused == which.0 { "_" } else { "" };
        t.0 = format!("{value}{cursor}");
    }
}

fn handle_format_toggle(
    mut fields: ResMut<MenuFields>,
    mut status: ResMut<MenuStatus>,
    buttons: Query<(&Interaction, &FormatToggleButton), Changed<Interaction>>,
) {
    for (interaction, toggle) in &buttons {
        if *interaction == Interaction::Pressed && fields.format != toggle.0 {
            fields.format = toggle.0;
            // Import-validation errors are format-dependent — a stale
            // "not legal in Modern" line under a new format misleads.
            status.0.clear();
        }
    }
}

fn refresh_format_toggle_visuals(
    fields: Res<MenuFields>,
    mut buttons: Query<(&FormatToggleButton, &mut BackgroundColor)>,
) {
    if !fields.is_changed() {
        return;
    }
    for (toggle, mut bg) in &mut buttons {
        *bg = BackgroundColor(if toggle.0 == fields.format {
            FIELD_BG_FOCUSED
        } else {
            FIELD_BG
        });
    }
}

fn handle_pod_toggles(
    mut fields: ResMut<MenuFields>,
    size_q: Query<&Interaction, (Changed<Interaction>, With<PodSizeButton>)>,
    opp_q: Query<&Interaction, (Changed<Interaction>, With<PodOpponentsButton>)>,
) {
    if size_q.iter().any(|i| *i == Interaction::Pressed) {
        fields.pod_size = next_pod_size(fields.pod_size);
    }
    if opp_q.iter().any(|i| *i == Interaction::Pressed) {
        let n = crabomination::pod::target_decks().len();
        fields.pod_opponents = fields.pod_opponents.next(n);
    }
}

/// Show the pod row only under Commander and keep its labels current. Runs
/// every frame (the row is respawned with the menu, after `MenuFields` last
/// changed) but writes only on a difference, so change detection stays quiet.
fn refresh_pod_options(
    fields: Res<MenuFields>,
    mut rows: Query<&mut Node, With<PodOptionsRow>>,
    mut labels: Query<(&PodLabel, &mut Text)>,
) {
    let display =
        if fields.format == MatchFormat::Commander { Display::Flex } else { Display::None };
    for mut node in &mut rows {
        if node.display != display {
            node.display = display;
        }
    }
    for (which, mut text) in &mut labels {
        let want = match which {
            PodLabel::Size => format!("Players: {}", fields.pod_size),
            PodLabel::Opponents => fields.pod_opponents.label(),
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}

/// Mirror the background art-prefetch counters into the menu's progress
/// line. Empty when idle or done — the line only speaks while the
/// background thread is actually fetching.
fn update_download_progress(
    prefetch: Option<Res<crate::scryfall::ImagePrefetch>>,
    mut q: Query<&mut Text, With<DownloadProgressText>>,
) {
    use std::sync::atomic::Ordering;
    let Some(prefetch) = prefetch else { return };
    let total = prefetch.total.load(Ordering::Relaxed);
    let done = prefetch.done.load(Ordering::Relaxed);
    let finished = prefetch.finished.load(Ordering::Relaxed);
    let label = if finished || total == 0 {
        String::new()
    } else {
        format!("Downloading card art…  {done}/{total} (playable now — missing art shows placeholders)")
    };
    for mut t in &mut q {
        if t.0 != label {
            t.0 = label.clone();
        }
    }
}

/// Mirror `MenuStatus` into its text node.
fn refresh_menu_status(status: Res<MenuStatus>, mut q: Query<&mut Text, With<MenuStatusText>>) {
    if !status.is_changed() {
        return;
    }
    for mut t in &mut q {
        t.0 = status.0.clone();
    }
}

fn handle_action_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    mut pending_draft: ResMut<PendingDraftFormat>,
    mut lobby_server: ResMut<PendingLobbyServer>,
    mut status: ResMut<MenuStatus>,
    fields: Res<MenuFields>,
    import_q: Query<&Interaction, (Changed<Interaction>, With<ImportDeckButton>)>,
    play_q: Query<&Interaction, (Changed<Interaction>, With<PlayBotButton>)>,
    spectate_q: Query<&Interaction, (Changed<Interaction>, With<SpectateBotsButton>)>,
    load_q: Query<&Interaction, (Changed<Interaction>, With<LoadDebugStateButton>)>,
    audit_q: Query<&Interaction, (Changed<Interaction>, With<AuditCardsButton>)>,
    draft_q: Query<&Interaction, (Changed<Interaction>, With<DraftButton>)>,
    host_q: Query<&Interaction, (Changed<Interaction>, With<HostButton>)>,
    join_q: Query<&Interaction, (Changed<Interaction>, With<JoinButton>)>,
    rejoin_q: Query<&Interaction, (Changed<Interaction>, With<RejoinButton>)>,
) {
    if rejoin_q.iter().any(|i| *i == Interaction::Pressed) {
        match load_persisted_resume() {
            Some((addr, token, format)) => {
                pending.0 = Some((NetMode::Rejoin { addr, token }, format));
                next_state.set(AppState::InGame);
            }
            None => status.0 = "Rejoin unavailable: saved match info is gone.".to_string(),
        }
        return;
    }
    if audit_q.iter().any(|i| *i == Interaction::Pressed) {
        next_state.set(AppState::Audit);
        return;
    }
    let format = fields.format;
    if draft_q.iter().any(|i| *i == Interaction::Pressed) {
        // The draft plugin reads PendingDraftFormat at OnEnter; the
        // resulting DraftedDecks resource will be consumed by
        // start_net_session_from_menu when the user finishes drafting.
        // Stash the underlying mode so the post-draft match plays as a
        // local-bot game.
        pending_draft.0 = format;
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::Drafting);
        return;
    }
    if play_q.iter().any(|i| *i == Interaction::Pressed) {
        pending.0 = Some((NetMode::LocalBot, format));
        next_state.set(AppState::InGame);
        return;
    }
    if import_q.iter().any(|i| *i == Interaction::Pressed) {
        let path = fields.deck_path.trim();
        match std::fs::read_to_string(path) {
            Err(e) => status.0 = format!("Can't read {path}: {e}"),
            Ok(text) => {
                let parsed = crabomination::decklist::parse_decklist(&text);
                if !parsed.unknown.is_empty() {
                    // Refuse rather than silently playing a partial deck.
                    let shown = parsed.unknown.iter().take(4).cloned()
                        .collect::<Vec<_>>().join(", ");
                    let more = parsed.unknown.len().saturating_sub(4);
                    status.0 = format!(
                        "{} card(s) not in the catalog: {shown}{}",
                        parsed.unknown.len(),
                        if more > 0 { format!(" (+{more} more)") } else { String::new() },
                    );
                } else if format == MatchFormat::Commander {
                    // CR 903 — the commander section, pair, identity and
                    // 100-card singleton rules, all checked by the engine.
                    match parsed.commander_list() {
                        Err(errs) => status.0 = join_errors(&errs),
                        Ok(list) => {
                            status.0.clear();
                            commands.insert_resource(ImportedDeck {
                                main: list.main,
                                commanders: list.commanders,
                            });
                            pending.0 = Some((NetMode::LocalBot, format));
                            next_state.set(AppState::InGame);
                        }
                    }
                } else if let Err(errs) = crabomination::format::validate_deck(
                    &parsed.main.iter().map(|f| f()).collect::<Vec<_>>(),
                    // Validate against the menu's selected format, not a
                    // hardcoded one — Cube/SoS pools play limited-style
                    // 40-card rules. (Commander took the branch above.)
                    match format {
                        MatchFormat::Modern => crabomination::format::Format::Modern,
                        MatchFormat::Commander => crabomination::format::Format::Commander,
                        // Sealed lists are limited-legal 40s like the
                        // draft pools.
                        MatchFormat::Cube | MatchFormat::Sos | MatchFormat::Sealed => {
                            crabomination::format::Format::Draft
                        }
                    },
                ) {
                    status.0 = join_errors(&errs);
                } else {
                    status.0.clear();
                    commands.insert_resource(ImportedDeck {
                        main: parsed.main,
                        commanders: Vec::new(),
                    });
                    pending.0 = Some((NetMode::LocalBot, format));
                    next_state.set(AppState::InGame);
                }
            }
        }
        return;
    }
    if spectate_q.iter().any(|i| *i == Interaction::Pressed) {
        pending.0 = Some((NetMode::SpectateBots, format));
        next_state.set(AppState::InGame);
        return;
    }
    if load_q.iter().any(|i| *i == Interaction::Pressed) {
        match crate::debug_export::list_exports().into_iter().next() {
            Some(path) => {
                pending.0 = Some((NetMode::LoadDebugState { path }, format));
                next_state.set(AppState::InGame);
            }
            None => status.0 = "No debug state files found in <repo>/debug/.".to_string(),
        }
        return;
    }
    if host_q.iter().any(|i| *i == Interaction::Pressed) {
        if let Ok(port) = fields.host_port.parse::<u16>() {
            pending.0 = Some((NetMode::HostLan { port }, format));
            next_state.set(AppState::InGame);
        } else {
            status.0 = format!("Invalid host port `{}` — use 1-65535.", fields.host_port);
        }
        return;
    }
    if join_q.iter().any(|i| *i == Interaction::Pressed) {
        let addr = fields.join_addr.trim().to_string();
        if addr.is_empty() {
            status.0 = "Enter a server address (host:port) to join.".to_string();
        } else {
            // Connect, then browse lobbies on the server (which picks the
            // gamemode per lobby). The actual connect happens on entry to
            // `AppState::Lobby`.
            let name = sanitize_name(&fields.player_name);
            lobby_server.0 = Some(LobbyConnect { addr, name });
            next_state.set(AppState::Lobby);
        }
    }
}

/// The first three validation errors and how many more, for the one-line
/// menu status.
fn join_errors<E: std::fmt::Display>(errs: &[E]) -> String {
    crabomination::format::error_summary(errs, 3)
}

/// Trim a display name and fall back to "Player" when it's blank, so the
/// server never receives an empty name.
fn sanitize_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed.to_string()
    }
}

// ── Network setup invoked by `OnEnter(InGame)` ───────────────────────────────

/// Read the queued `PendingNetMode` and install `NetOutbox`/`NetInbox`. Falls
/// back to a local Modern bot match if no choice was queued (e.g. tests
/// bypass the menu).
pub fn start_net_session_from_menu(world: &mut World) {
    // On the lobby path the net session is already installed (we connected to
    // the lobby server) and the match is running server-side; below we still
    // set up the in-game meta resources but skip re-spawning a session.
    let already_connected = world.contains_resource::<NetOutbox>();

    // Clear the play-by-play log so a new session (including an audit run)
    // doesn't show scrollback from the previous game.
    if let Some(mut log) = world.get_resource_mut::<crate::game::GameLog>() {
        log.entries.clear();
    }

    // Standing "Always Yes/No" trigger answers are per-game — card ids
    // don't carry across matches.
    world.insert_resource(crate::systems::decision_ui::AutoOptionalAnswers::default());

    let (mode, format) = world
        .get_resource_mut::<PendingNetMode>()
        .and_then(|mut r| r.0.take())
        .unwrap_or((NetMode::LocalBot, MatchFormat::Modern));

    // Stash the chosen format + mode kind so the game-over modal can
    // (a) launch an equivalent rematch without revisiting the menu,
    // and (b) decide whether to show the auto-rematch counter
    // (Spectate Bot vs Bot only — Human vs Bot would be jarring).
    //
    // These must be inserted even on the lobby path (already_connected): the
    // in-game game-over systems read `ActiveMatchFormat` as a required
    // resource every frame, so skipping them used to crash the moment a lobby
    // match started.
    world.insert_resource(crate::systems::game_over::ActiveMatchFormat(format));
    let kind = match &mode {
        NetMode::SpectateBots => crate::systems::game_over::ActiveMatchKind::SpectateBotVsBot,
        _ => crate::systems::game_over::ActiveMatchKind::HumanVsBot,
    };
    world.insert_resource(kind);

    // Lobby flow: the net session was already installed when we connected to
    // the lobby server, and the match is already running server-side — just
    // consume it. Re-running the spawn below would clobber the live connection.
    if already_connected {
        return;
    }

    match mode {
        NetMode::LocalBot => spawn_inprocess_bot(world, format),
        NetMode::SpectateBots => spawn_spectate_bots(world, format),
        NetMode::LayoutFixture { seats } => spawn_layout_fixture(world, seats),
        NetMode::LoadDebugState { path } => match spawn_loaded_debug_state(world, &path) {
            Ok(()) => eprintln!("net: loaded debug state from {}", path.display()),
            Err(e) => {
                eprintln!("net: load {} failed ({e}); falling back to local bot",
                    path.display());
                spawn_inprocess_bot(world, format);
            }
        },
        #[cfg(not(target_arch = "wasm32"))]
        NetMode::HostLan { port } => match spawn_host_lan(world, port, format) {
            Ok(()) => eprintln!(
                "net: hosting {fmt:?} on 0.0.0.0:{port} — waiting for opponent",
                fmt = format
            ),
            Err(e) => {
                eprintln!("net: host failed ({e}); falling back to local bot");
                spawn_inprocess_bot(world, format);
            }
        },
        // Browsers can't listen for TCP connections; the button is native-only
        // but the enum variant still exists, so route it somewhere sane.
        #[cfg(target_arch = "wasm32")]
        NetMode::HostLan { .. } => {
            eprintln!("net: hosting isn't available in the browser build");
        }
        // No session to spawn — arm the reconnect loop with the persisted
        // token and let `maybe_reconnect` (run_if lost) claim the seat.
        NetMode::Rejoin { addr, token } => {
            let mut resume = world.resource_mut::<crate::net_plugin::ResumeInfo>();
            resume.server_addr = Some(addr);
            resume.token = Some(token);
            resume.attempts = 0;
            resume.last_attempt = None;
            resume.lost = true;
            eprintln!("net: rejoining previous match…");
        }
    }
}

/// Stamp display names onto a freshly built local match state so the HUD
/// and log read "Alice" / "Bot" instead of the engine's "P0" / "P1"
/// placeholders. Seat 0 gets `seat0`; other seats get `other` (numbered
/// when there are several, e.g. Commander's three bots).
pub(crate) fn name_seats(state: &mut GameState, seat0: &str, other: &str) {
    let many_others = state.players.len() > 2;
    for (i, p) in state.players.iter_mut().enumerate() {
        p.name = if i == 0 {
            seat0.to_string()
        } else if many_others {
            format!("{other} {i}")
        } else {
            other.to_string()
        };
    }
}

/// The menu's display name (trimmed), falling back to "Player".
pub(crate) fn menu_player_name(world: &World) -> String {
    world
        .get_resource::<MenuFields>()
        .map(|f| f.player_name.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Player".to_string())
}

/// The bot a local seat gets: the lobby's pilot — the 256-iteration
/// search on the adopted default with the champion value net as its leaf
/// (round 74, 2026-09-15: 53.3 / 55.2 over the material leaf inside the
/// search, `server::lobby::default_bot` has the record) — so single-player
/// and hosted games face the same opponent. The net is loaded here on
/// first use from `CRAB_NET` or `nets/champion.safetensors`; a checkout
/// without the file plays the material leaf (the round-64 pilot, 55.25
/// over the heuristic default) and says so once on stderr, the same
/// fallback the server has. Before round 65 the client built its own
/// pilot — `net_eval_det1` at 64 iterations without the combat chains —
/// and forty September replays showed its attack judgment two generations
/// behind the lobby's (ML_NOTES "Round 65").
///
/// A pod (more than two seats) gets the heuristic default, as the lobby's
/// does: the net reads only two-seat games, and the search on the material
/// leaf is unmeasured in a pod.
pub(crate) fn local_bot(n_seats: usize) -> Box<dyn crabomination::server::Bot> {
    if n_seats > 2 {
        return Box::new(HeuristicBot::new());
    }
    load_champion_once();
    Box::new(crabomination::server::MctsBot::new(crabomination::server::MctsConfig {
        iterations: 256,
        horizon_turns: 3,
        weights: crabomination::server::EvalWeights::net_on_default(),
        search_threads: bot_search_threads(),
        ..crabomination::server::MctsConfig::default()
    }))
}

/// Load the champion value net into the bot's slot, once per process.
/// Missing file: the material leaf, announced. Bad file: announced and
/// the material leaf — a game must still start, unlike the server, whose
/// boot refuses a bad net because it advertises the bot's strength.
fn load_champion_once() {
    use crabomination::server::net_eval;
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let path = std::env::var("CRAB_NET").unwrap_or_else(|_| "nets/champion.safetensors".to_string());
        let p = std::path::Path::new(&path);
        if !p.exists() {
            eprintln!("value net {path} not found; the local bot searches on the material leaf");
            return;
        }
        match net_eval::load_slot(net_eval::SLOT_BEST, p) {
            Ok(()) => eprintln!("value net loaded from {path}; the local bot searches on its leaf"),
            Err(e) => eprintln!("value net {path}: {e}; the local bot searches on the material leaf"),
        }
    });
}

/// Worker threads for the bot's root search.
///
/// The ladder and the training actors run one game per core and leave the
/// search serial; this program runs *one* game and a human waits on every
/// decision of it, so the cores are sitting there. The budget is split
/// rather than multiplied (`MctsConfig::search_threads`), so this is
/// latency, not strength — see that field for the trade and the gate.
///
/// Half the machine, capped at 4. Half because Bevy has its own task and
/// render pools on this box and a search that takes every core stutters the
/// frame the human is looking at; capped at 4 because the budget is 256
/// rollouts and splitting it further leaves each worker's UCB1 allocating on
/// too few samples of its own.
fn bot_search_threads() -> usize {
    std::thread::available_parallelism().map(|n| (n.get() / 2).clamp(1, 4)).unwrap_or(1)
}

fn spawn_inprocess_bot(world: &mut World, format: MatchFormat) {
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
    let sink_for_match = Arc::clone(&sink);
    // Build the state first so we can size the occupant list to its
    // seat count. Two-player formats (Modern / Cube / SoS) end up with
    // [Human, Bot]; four-player formats (Commander) get
    // [Human, Bot, Bot, Bot]. Pre-fix this was hardcoded to two and
    // panicked when a Commander match brought 4 seats.
    //
    // Audit override: if the menu wrote a card name into `AuditTarget`
    // before entering InGame, build a tailored two-seat audit state
    // around that card instead of the chosen format.
    //
    // Draft override: if the user just finished an 8-seat draft, the
    // draft plugin will have written a `DraftedDecks` resource. Take
    // and consume it so a follow-up rematch falls back to the format's
    // random path. Drafted decks beat the audit + format paths.
    let audit_card: Option<String> = world
        .get_resource::<crate::audit::AuditTarget>()
        .and_then(|t| t.0.clone());
    let drafted: Option<DraftedDecks> = world
        .get_resource_mut::<DraftedDecks>()
        .map(|r| (*r).clone());
    if drafted.is_some() {
        world.remove_resource::<DraftedDecks>();
    }
    let human_name = menu_player_name(world);
    let imported: Option<ImportedDeck> = world.remove_resource::<ImportedDeck>();
    let (pod_size, pod_opponents) = world
        .get_resource::<MenuFields>()
        .map_or((4, PodOpponents::Random), |f| (f.pod_size, f.pod_opponents));
    // An imported deck outranks a draft and an audit, which outrank the
    // format's stock decks.
    let state = if imported.is_some() || (drafted.is_none() && audit_card.is_none()) {
        // Remembered so the game-over "New Game" deals this deck again
        // rather than the format's stock one.
        world.insert_resource(RematchDeck(imported.clone()));
        build_local_match_state(format, imported, pod_size, pod_opponents, &human_name)
    } else if let Some(decks) = drafted {
        world.insert_resource(RematchDeck(None));
        let mut state = crabomination::draft::build_draft_match_state(
            decks.player_deck,
            decks.opponent_deck,
            human_name,
            decks.opponent_label,
        );
        for (seat, notes) in decks.notes.into_iter().enumerate() {
            state.players[seat].draft_notes = notes;
        }
        state
    } else {
        world.insert_resource(RematchDeck(None));
        let name = audit_card.as_deref().unwrap_or_default();
        let mut state = crate::audit::build_audit_state(name).unwrap_or_else(|| {
            eprintln!("audit: unknown card '{name}', falling back to {:?}", format);
            format.build_state()
        });
        name_seats(&mut state, &human_name, "Bot");
        state
    };
    let occupants = local_occupants(server_seat, state.players.len());
    std::thread::spawn(move || {
        run_match_full(state, occupants, vec![], Some(sink_for_match));
    });
    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    world.insert_resource(LatestSnapshot(sink));
}

/// The human's seat 0 plus one local bot for every other seat of the state.
pub(crate) fn local_occupants(
    human: crabomination::server::SeatChannel,
    n_seats: usize,
) -> Vec<SeatOccupant> {
    std::iter::once(SeatOccupant::Human(human))
        .chain((1..n_seats).map(|_| SeatOccupant::Bot(local_bot(n_seats))))
        .collect()
}

/// The deck the running human-vs-bot match was started with (`None` for the
/// format's stock decks), kept so the game-over "New Game" deals it again.
/// Drafted and audit matches don't set it: their rematch is the format's.
#[derive(Resource, Clone, Default)]
pub struct RematchDeck(pub Option<ImportedDeck>);

/// A human-vs-bots state for `format`: an imported deck (a Commander list
/// seats a 2-4 player pod; any other list meets the format's stock
/// opponent), or the format's stock decks. Shared by the menu and the
/// game-over "New Game", so both deal the same kind of game.
pub(crate) fn build_local_match_state(
    format: MatchFormat,
    imported: Option<ImportedDeck>,
    pod_size: usize,
    pod_opponents: PodOpponents,
    human_name: &str,
) -> GameState {
    // Commander seats the human's deck — imported, or the first stock pod
    // deck for "Play vs Bot" — in seat 0 of a 2-4 player pod.
    let commander_deck = match (&imported, format) {
        (Some(deck), _) if !deck.commanders.is_empty() => {
            Some((deck.commanders.clone(), deck.main.clone()))
        }
        (None, MatchFormat::Commander) => {
            let stock = crabomination::pod::target_decks()[0];
            Some((stock.commanders.to_vec(), stock.main.to_vec()))
        }
        _ => None,
    };
    if let Some((commanders, main)) = commander_deck {
        return commander_pod_state(&commanders, &main, pod_size, pod_opponents, human_name);
    }
    if let Some(deck) = imported {
        // Imported decklist vs. an opponent chosen by the selected
        // format: Sealed rolls a fresh random sealed build (the same
        // generator the recommender's gauntlet uses — this is "how does
        // my deck do against the field", played by hand); everything
        // else keeps the stock Modern bot deck.
        let (opp_deck, opp_label) = if format == MatchFormat::Sealed {
            use rand::RngExt;
            let seed: u64 = rand::rng().random();
            crabomination::selfplay::random_sealed_opponent(seed)
        } else {
            (crabomination::demo::brg_combo_deck().to_vec(), "Bot".to_string())
        };
        return crabomination::draft::build_draft_match_state(
            deck.main,
            opp_deck,
            human_name.to_string(),
            opp_label,
        );
    }
    let mut state = format.build_state();
    name_seats(&mut state, human_name, "Bot");
    state
}

/// A local Commander pod: `commanders` + `main` in the human's seat 0 and
/// `pod_size - 1` bots on stock decks, named after the deck they play. The
/// deal is seeded off the state's own stream, as `build_commander_state` is.
fn commander_pod_state(
    commanders: &[crabomination::cube::CardFactory],
    main: &[crabomination::cube::CardFactory],
    pod_size: usize,
    opponents: PodOpponents,
    human_name: &str,
) -> GameState {
    use rand::RngExt;
    let seed: u64 = rand::rng().random();
    let bots = commander_opponents(opponents, pod_size.clamp(2, 4) - 1, seed.rotate_left(32));
    let mut state =
        crabomination::demo::build_custom_commander_state_seeded(commanders, main, &bots, seed);
    state.players[0].name = human_name.to_string();
    for (i, deck) in bots.iter().enumerate() {
        state.players[i + 1].name = format!("Bot {}: {}", i + 1, deck.name);
    }
    state
}

/// Shared handle to the authoritative engine state for the running
/// in-process match. Read-locked by the export prompt to embed both a
/// structured `GameSnapshot` and the full `GameState` JSON in saved
/// debug exports. `None`/empty when no in-process match is running, or
/// when the local client is connected to a remote (TCP) match — in
/// those cases the engine state lives on a different machine.
#[derive(Resource, Default, Clone)]
pub struct LatestSnapshot(pub SnapshotSink);

impl LatestSnapshot {
    pub fn read(&self) -> SnapshotSinkState {
        self.0.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

/// Spectate-only mode: both seats are HeuristicBots, the local UI hooks
/// up a spectator channel that mirrors seat 0's projection. Any actions
/// the local UI submits are silently dropped server-side.
fn spawn_spectate_bots(world: &mut World, format: MatchFormat) {
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
    let sink_for_match = Arc::clone(&sink);
    // Size the bot list to the format's seat count — Commander brings
    // 4 seats, the other formats bring 2.
    let mut state = format.build_state();
    for (i, p) in state.players.iter_mut().enumerate() {
        p.name = format!("Bot {}", i + 1);
    }
    let n_seats = state.players.len();
    let occupants: Vec<SeatOccupant> = (0..n_seats)
        .map(|_| SeatOccupant::Bot(Box::new(HeuristicBot::new())))
        .collect();
    std::thread::spawn(move || {
        run_match_full(state, occupants, vec![server_seat], Some(sink_for_match));
    });
    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    world.insert_resource(LatestSnapshot(sink));
}

/// Direct host mode's seats: the host in seat 0, the joiner in seat 1, and a
/// local bot in every seat past those — a Commander pod deals four, and the
/// match actor wants one occupant a seat (it asserted, and the host thread
/// died as the joiner connected). Direct mode has no lobby handshake to
/// learn the joiner's name, so seat 1 is "Opponent".
pub(crate) fn host_lan_seats(
    state: &mut GameState,
    host_name: &str,
    host: crabomination::server::SeatChannel,
    joiner: crabomination::server::SeatChannel,
) -> Vec<SeatOccupant> {
    let n_seats = state.players.len();
    for (i, p) in state.players.iter_mut().enumerate() {
        p.name = match i {
            0 => host_name.to_string(),
            1 => "Opponent".to_string(),
            _ => format!("Bot {}", i - 1),
        };
    }
    [SeatOccupant::Human(host), SeatOccupant::Human(joiner)]
        .into_iter()
        .chain((2..n_seats).map(|_| SeatOccupant::Bot(local_bot(n_seats))))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_host_lan(world: &mut World, port: u16, format: MatchFormat) -> std::io::Result<()> {
    let bind = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind)?;
    let (server_seat0, ClientChannel { tx, rx }) = seat_pair();
    let host_name = menu_player_name(world);

    std::thread::spawn(move || {
        let (stream, peer) = match listener.accept() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("host: accept failed: {e}");
                return;
            }
        };
        eprintln!("host: opponent connected from {peer}");
        let server_seat1 = match tcp_seat(stream) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("host: tcp_seat failed: {e}");
                return;
            }
        };
        let mut state = format.build_state();
        let occupants = host_lan_seats(&mut state, &host_name, server_seat0, server_seat1);
        run_match(state, occupants);
        eprintln!("host: match ended");
    });

    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    Ok(())
}

/// The layout harness's fixture board, with its `--viewer-out` /
/// `--hold-seat` variations applied.
fn spawn_layout_fixture(world: &mut World, seats: usize) {
    let args = world.get_resource::<crate::layout_harness::HarnessArgs>().cloned().unwrap_or_default();
    let mut state = crate::layout_harness::fixture_state(seats);
    if args.viewer_out {
        crate::layout_harness::knock_out_viewer(&mut state);
    }
    let Some(held) = args.hold_seat.filter(|s| *s != 0 && *s < state.players.len()) else {
        spawn_restored_state(world, state, 0);
        return;
    };
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let (held_seat, held_client) = seat_pair();
    // The held player's end stays open and silent for the whole run.
    std::mem::forget(held_client);
    let mut held_seat = Some(held_seat);
    let mut human = Some(server_seat);
    let occupants: Vec<SeatOccupant> = (0..state.players.len())
        .map(|seat| match seat {
            0 => SeatOccupant::Human(human.take().expect("seat 0 once")),
            s if s == held => SeatOccupant::Human(held_seat.take().expect("held seat once")),
            _ => SeatOccupant::Bot(Box::new(HeuristicBot::new())),
        })
        .collect();
    std::thread::spawn(move || {
        run_match_full(state, occupants, vec![], None);
    });
    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
}

/// Run `state` as a live local match with the human in `viewer_seat` and a
/// HeuristicBot in every other seat (a restored pod keeps all its seats).
fn spawn_restored_state(world: &mut World, state: GameState, viewer_seat: usize) {
    let (server_seat, ClientChannel { tx, rx }) = seat_pair();
    let sink: SnapshotSink = Arc::new(Mutex::new(SnapshotSinkState::default()));
    let sink_for_match = Arc::clone(&sink);
    let mut human = Some(server_seat);
    let occupants: Vec<SeatOccupant> = (0..state.players.len())
        .map(|seat| match human.take_if(|_| seat == viewer_seat) {
            Some(h) => SeatOccupant::Human(h),
            None => SeatOccupant::Bot(Box::new(HeuristicBot::new())),
        })
        .collect();
    std::thread::spawn(move || {
        run_match_full(state, occupants, vec![], Some(sink_for_match));
    });
    world.insert_resource(NetOutbox::new(tx));
    world.insert_resource(NetInbox(Mutex::new(rx)));
    world.insert_resource(LatestSnapshot(sink));
}

/// Load a previously exported debug state file. If the export carries a
/// full `GameSnapshot` (the new format), the snapshot is restored into a
/// real `GameState` and run as an in-process match against a `HeuristicBot`
/// — meaning the user can keep playing from the saved board, which is
/// the whole point of this debug workflow.
///
/// Older view-only exports fall back to read-only inspection: the
/// `ClientView` is seeded into `CurrentView` directly, no `NetOutbox` is
/// installed (so the input handler bails out), and the player can poke
/// around the board but not advance it.
fn spawn_loaded_debug_state(world: &mut World, path: &std::path::Path) -> std::io::Result<()> {
    let export = crate::debug_export::load_debug_export(path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let log_prefix = format!(
        "Loaded debug state from {}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("?")
    );
    let bug_note = (!export.message.is_empty())
        .then(|| format!("  bug note: {}", export.message));

    // Prefer the bit-exact full GameState when present; fall back to
    // the schema-stable GameSnapshot (with the trigger-stack caveat);
    // fall back again to view-only inspection for legacy exports.
    let restored_with_dropped: Option<(GameState, usize, &'static str)> =
        if let Some(full) = export.full_state {
            Some((full, 0, "full GameState"))
        } else if let Some(snap) = export.snapshot.clone() {
            let dropped = snap.dropped_triggers;
            match snap.restore() {
                Ok(state) => Some((state, dropped, "snapshot")),
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("restore snapshot: {e}"),
                    ));
                }
            }
        } else {
            None
        };

    if let Some((restored, dropped_triggers, source_label)) = restored_with_dropped {
        spawn_restored_state(world, restored, export.view.your_seat);
        if let Some(mut log) = world.get_resource_mut::<crate::game::GameLog>() {
            log.push(log_prefix);
            if let Some(note) = bug_note {
                log.push(note);
            }
            log.push(format!(
                "Restored from {source_label} — playable from this state"
            ));
            if dropped_triggers > 0 {
                log.push(format!(
                    "  warning: {dropped_triggers} trigger(s) on the original stack were dropped"
                ));
            }
        }
        Ok(())
    } else {
        // Legacy view-only export: read-only inspection mode.
        if let Some(mut cv) = world.get_resource_mut::<crate::net_plugin::CurrentView>() {
            cv.0 = Some(export.view.clone());
        }
        if let Some(mut seat) = world.get_resource_mut::<crate::net_plugin::OurSeat>() {
            seat.0 = export.view.your_seat;
        }
        if let Some(mut log) = world.get_resource_mut::<crate::game::GameLog>() {
            log.push(log_prefix);
            if let Some(note) = bug_note {
                log.push(note);
            }
            log.push("View-only inspection (no full snapshot in this file)");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn opponent_packs_arg_parses_and_clamps() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // Absent: a standard six-pack sealed pool.
        assert_eq!(
            opponent_pack_count_from(args(&["--play", "sealed"]).into_iter()),
            crabomination::selfplay::SEALED_PACKS,
        );
        assert_eq!(
            opponent_pack_count_from(args(&["--play", "sealed", "--opponent-packs", "12"]).into_iter()),
            12,
        );
        // Garbage and out-of-range values must not take the mode down.
        assert_eq!(
            opponent_pack_count_from(args(&["--opponent-packs", "banana"]).into_iter()),
            crabomination::selfplay::SEALED_PACKS,
        );
        assert_eq!(opponent_pack_count_from(args(&["--opponent-packs", "0"]).into_iter()), 1);
        assert_eq!(
            opponent_pack_count_from(args(&["--opponent-packs", "9999"]).into_iter()),
            MAX_OPPONENT_PACKS,
        );
        // A trailing flag with no value is ignored rather than panicking.
        assert_eq!(
            opponent_pack_count_from(args(&["--opponent-packs"]).into_iter()),
            crabomination::selfplay::SEALED_PACKS,
        );
    }

    /// The search-worker count is a *latency* knob on a shared box: it has
    /// to leave Bevy's pools room on a big machine and must never ask for
    /// zero threads on a small one (`MctsConfig::search_threads` is a
    /// count, and 0 would mean "no rollouts at all" to `parallel_spend`).
    #[test]
    fn bot_search_threads_leaves_the_box_room_and_never_returns_zero() {
        let n = bot_search_threads();
        assert!((1..=4).contains(&n), "search workers out of range: {n}");
        let cores = std::thread::available_parallelism().map(|c| c.get()).unwrap_or(1);
        assert!(n <= cores.max(1), "asked for {n} workers on {cores} cores");
    }

    #[test]
    fn opponent_colors_arg_takes_a_college_or_its_letters() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let colors = |v: &[&str]| opponent_colors_from(args(v).into_iter());
        // Absent: no restriction, the builder picks its own colours.
        assert!(colors(&["--play", "sealed"]).unwrap().is_empty());

        let lorehold: ColorSet = [MtgColor::Red, MtgColor::White].iter().collect();
        for spec in ["lorehold", "Lorehold", " LOREHOLD ", "rw", "WR", "r/w", "r,w"] {
            assert_eq!(
                colors(&["--play", "sealed", "--opponent-colors", spec]).unwrap(),
                lorehold,
                "--opponent-colors {spec}",
            );
        }
        assert_eq!(colors(&["--opponent-colors", "wubr"]).unwrap().len(), 4);

        // Every rejection below would otherwise deal the player an opponent
        // in colours they did not ask for — the one outcome the flag exists
        // to rule out. A typo, a colour the game doesn't have, ...
        assert!(colors(&["--opponent-colors", "banana"]).is_err());
        assert!(colors(&["--opponent-colors", "rx"]).is_err());
        // ... a single colour (the builder has no mono-colour shape), ...
        assert!(colors(&["--opponent-colors", "r"]).is_err());
        assert!(colors(&["--opponent-colors", ""]).is_err());
        // ... and a flag whose value is missing or is the next flag. The
        // `--` test is what stops `--opponent-colors --bug` reading as
        // {Black, Blue, Green}: strip its dashes and it is a colour spec.
        assert!(colors(&["--opponent-colors"]).is_err());
        assert!(colors(&["--opponent-colors", "--play", "sealed"]).is_err());
        assert!(colors(&["--opponent-colors", "--bug"]).is_err());
    }
    use super::*;
    use crate::systems::game_over::ActiveMatchFormat;

    /// Regression: on the lobby path the net session is already installed
    /// (`NetOutbox` present), so `start_net_session_from_menu` skips re-spawning
    /// — but it must STILL insert `ActiveMatchFormat`, which the in-game
    /// game-over systems read as a required resource every frame. Skipping it
    /// used to crash the instant a lobby match started (e.g. on "Add Bot").
    #[test]
    fn lobby_path_still_inserts_active_match_format() {
        let mut world = World::new();
        // Simulate the lobby flow: a live connection is already installed.
        let (tx, _rx) = std::sync::mpsc::channel();
        world.insert_resource(NetOutbox::new(tx));

        start_net_session_from_menu(&mut world);

        assert!(
            world.contains_resource::<ActiveMatchFormat>(),
            "ActiveMatchFormat must be present on the lobby path or the \
             game-over systems panic on the first in-game frame",
        );
        assert!(
            world.contains_resource::<crate::systems::game_over::ActiveMatchKind>(),
        );
        // The live session must be left untouched (not clobbered by a respawn).
        assert!(world.contains_resource::<NetOutbox>());
    }

    /// Sealed reads the env override when set, and otherwise resolves a
    /// path inside the repo.
    ///
    /// The relative-path case is the one that bit in practice:
    /// `cargo run -p crabomination_client` leaves the child's working
    /// directory wherever the shell was, so running from inside the
    /// crate made `decks/foo.txt` resolve to
    /// `crabomination_client/decks/foo.txt` — nonexistent — and the mode
    /// quietly played a generated deck instead of the requested one.
    #[test]
    fn sealed_deck_path_follows_the_env_override() {
        // SAFETY: single-threaded within this test, and the value is
        // restored before returning.
        let prev = std::env::var("CRAB_SEALED_DECK").ok();
        unsafe { std::env::set_var("CRAB_SEALED_DECK", "/tmp/crab-sealed-test.txt") };
        assert_eq!(sealed_deck_path(), std::path::PathBuf::from("/tmp/crab-sealed-test.txt"));

        // A relative path that exists under the repo root resolves
        // there, whatever the working directory is.
        unsafe { std::env::set_var("CRAB_SEALED_DECK", "decks/sealed.txt") };
        let rooted = sealed_deck_path();
        assert!(
            rooted.is_absolute() && rooted.exists(),
            "a repo-relative deck must resolve against the repo root, got {}",
            rooted.display()
        );

        // A path that exists nowhere is returned as given, so the error
        // message names what the user actually typed.
        unsafe { std::env::set_var("CRAB_SEALED_DECK", "decks/no-such-deck-xyz.txt") };
        assert_eq!(
            sealed_deck_path(),
            std::path::PathBuf::from("decks/no-such-deck-xyz.txt")
        );

        unsafe { std::env::remove_var("CRAB_SEALED_DECK") };
        let fallback = sealed_deck_path();
        assert!(
            fallback.ends_with("decks/sealed.txt"),
            "default sealed path should live in the repo, got {}",
            fallback.display()
        );
        if let Some(v) = prev {
            unsafe { std::env::set_var("CRAB_SEALED_DECK", v) }
        }
    }

    /// The random sealed opponent is a real, legal-sized limited deck —
    /// this is what the menu's Sealed branch hands the bot seat.
    #[test]
    fn random_sealed_opponent_builds_a_playable_deck() {
        let (deck, label) = crabomination::selfplay::random_sealed_opponent(12_345);
        assert_eq!(deck.len(), 40, "sealed builds are 40 cards");
        assert!(label.starts_with("Sealed #"), "opponent label: {label}");
        // A different seed gives a different build, or "random sealed
        // opponent" is a fixed matchup wearing a seed.
        let (other, _) = crabomination::selfplay::random_sealed_opponent(999);
        let names = |d: &[crabomination::cube::CardFactory]| {
            d.iter().map(|f| f().name).collect::<Vec<_>>()
        };
        assert_ne!(names(&deck), names(&other));
    }

    #[test]
    fn pod_toggles_cycle_through_every_choice() {
        assert_eq!(next_pod_size(2), 3);
        assert_eq!(next_pod_size(3), 4);
        assert_eq!(next_pod_size(4), 2);
        let n = crabomination::pod::target_decks().len();
        let mut choice = PodOpponents::Random;
        let mut seen = Vec::new();
        for _ in 0..=n {
            choice = choice.next(n);
            seen.push(choice);
        }
        assert_eq!(seen[..n], (0..n).map(PodOpponents::Deck).collect::<Vec<_>>()[..]);
        assert_eq!(seen[n], PodOpponents::Random);
        assert_eq!(PodOpponents::Deck(1).label(), "Opponents: Judith (BR)");
    }

    #[test]
    fn commander_opponents_are_distinct_when_random_and_fixed_when_chosen() {
        let names = |v: Vec<crabomination::pod::PodDeck>| v.iter().map(|d| d.name).collect::<Vec<_>>();
        let random = names(commander_opponents(PodOpponents::Random, 3, 11));
        assert_eq!(random.len(), 3);
        let mut dedup = random.clone();
        dedup.sort_unstable();
        dedup.dedup();
        assert_eq!(dedup.len(), 3, "{random:?}");
        assert_eq!(random, names(commander_opponents(PodOpponents::Random, 3, 11)));
        assert_eq!(names(commander_opponents(PodOpponents::Deck(2), 2, 11)), ["Hanna (UW)"; 2]);
    }

    /// The human sits in seat 0 with their own commander, the bots are named
    /// after their decks, and the pod size is honoured.
    #[test]
    /// The game-over "New Game" rebuilds through `build_local_match_state`
    /// with the remembered deck: an imported Commander list comes back in
    /// seat 0 of a pod of the chosen size, and every seat gets an occupant.
    /// Before, a rematch dealt the stock pod and seated only two occupants.
    fn rematch_state_redeals_the_imported_commander_deck() {
        let stock = crabomination::pod::target_decks()[4];
        let deck = ImportedDeck { main: stock.main.to_vec(), commanders: stock.commanders.to_vec() };
        let state = build_local_match_state(MatchFormat::Commander, Some(deck), 3, PodOpponents::Random, "Ann");
        assert_eq!(state.players.len(), 3);
        assert_eq!(state.players[0].name, "Ann");
        assert_eq!(state.players[0].command.len(), 2, "Krark + Rograkh come back");
        let (seat, _client) = crabomination::server::seat_pair();
        assert_eq!(local_occupants(seat, state.players.len()).len(), 3);
        // No deck: the stock Commander pod, still sized by the menu option.
        let stock_pod = build_local_match_state(MatchFormat::Commander, None, 2, PodOpponents::Random, "Ann");
        assert_eq!(stock_pod.players.len(), 2);
    }

    #[test]
    fn commander_pod_state_seats_the_human_first() {
        let stock = crabomination::pod::target_decks()[4];
        let state = commander_pod_state(stock.commanders, stock.main, 3, PodOpponents::Deck(0), "Ann");
        assert_eq!(state.players.len(), 3);
        assert_eq!(state.players[0].name, "Ann");
        assert_eq!(state.players[0].command.len(), 2, "Krark + Rograkh");
        assert_eq!(state.players[1].name, "Bot 1: Sigarda (GW)");
        assert!(state.players.iter().all(|p| p.life == 40));
    }

    /// Hosting a LAN game seats one occupant per dealt seat: a Commander
    /// pod's four seats are the host, the joiner and two bots (two humans
    /// alone tripped the match actor's one-occupant-a-seat assert).
    #[test]
    fn a_hosted_lan_pod_fills_the_seats_past_two_with_bots() {
        for format in [MatchFormat::Modern, MatchFormat::Commander] {
            let mut state = format.build_state();
            let (host, _h) = crabomination::server::seat_pair();
            let (joiner, _j) = crabomination::server::seat_pair();
            let seats = host_lan_seats(&mut state, "Ann", host, joiner);
            assert_eq!(seats.len(), state.players.len(), "{format:?}");
            assert!(matches!(seats[1], SeatOccupant::Human(_)));
            assert!(seats[2..].iter().all(|o| matches!(o, SeatOccupant::Bot(_))));
            assert_eq!((state.players[0].name.as_str(), state.players[1].name.as_str()), ("Ann", "Opponent"));
        }
        let mut pod = MatchFormat::Commander.build_state();
        let (host, _h) = crabomination::server::seat_pair();
        let (joiner, _j) = crabomination::server::seat_pair();
        host_lan_seats(&mut pod, "Ann", host, joiner);
        assert_eq!(pod.players[3].name, "Bot 2");
    }
}

