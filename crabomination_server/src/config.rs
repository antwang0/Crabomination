//! Boot-time configuration: format selection, decklist overrides, and
//! environment-variable parsing.

use std::env;
use std::time::Duration;

use crabomination::cube::build_cube_state;
use crabomination::demo::{build_commander_state, build_demo_state};
use crabomination::game::GameState;
use crabomination::net::LobbyFormat;

/// Format-builder enum that captures the environment configuration once at
/// boot, so each match thread doesn't re-read env vars.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Format {
    Demo,
    Cube,
    Sos,
    Commander,
}

impl Format {
    pub(crate) fn from_env() -> Self {
        match env::var("CRAB_FORMAT").ok().as_deref() {
            Some("cube") => Self::Cube,
            Some("sos") | Some("strixhaven") => Self::Sos,
            Some("commander") | Some("edh") => Self::Commander,
            Some("demo") | None => Self::Demo,
            Some(other) => {
                eprintln!(
                    "warning: CRAB_FORMAT={other:?} not recognized — \
                     falling back to demo. Valid: \"demo\" | \"cube\" | \"sos\" | \"commander\"."
                );
                Self::Demo
            }
        }
    }
    pub(crate) fn build(&self) -> GameState {
        match self {
            Self::Demo => {
                let overrides = deck_overrides();
                if overrides.seat0.is_some() || overrides.seat1.is_some() {
                    let seat0 = overrides
                        .seat0
                        .clone()
                        .unwrap_or_else(|| crabomination::demo::brg_combo_deck().to_vec());
                    let seat1 = overrides
                        .seat1
                        .clone()
                        .unwrap_or_else(|| crabomination::demo::goryos_vengeance_deck().to_vec());
                    crabomination::draft::build_draft_match_state(
                        seat0,
                        seat1,
                        "Player 1".into(),
                        "Player 2".into(),
                    )
                } else {
                    build_demo_state()
                }
            }
            Self::Cube => build_cube_state(),
            Self::Sos => crabomination::sos_mode::build_sos_state(),
            Self::Commander => build_commander_state(),
        }
    }
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Demo => "demo",
            Self::Cube => "cube",
            Self::Sos => "sos",
            Self::Commander => "commander",
        }
    }
    /// Map a wire `LobbyFormat` onto the local stats bucket. Modern (the
    /// client label for the demo decklists) folds into `Demo`.
    pub(crate) fn from_lobby(f: LobbyFormat) -> Self {
        match f {
            LobbyFormat::Modern => Self::Demo,
            LobbyFormat::Cube => Self::Cube,
            LobbyFormat::Sos => Self::Sos,
            LobbyFormat::Commander => Self::Commander,
        }
    }
}

/// Decklist overrides for demo-format matches, loaded once at boot from
/// `CRAB_DECK` (seat 0) / `CRAB_BOT_DECK` (seat 1).
#[derive(Default)]
pub(crate) struct DeckOverrides {
    seat0: Option<Vec<crabomination::cube::CardFactory>>,
    seat1: Option<Vec<crabomination::cube::CardFactory>>,
}

pub(crate) fn deck_overrides() -> &'static DeckOverrides {
    static OVERRIDES: std::sync::OnceLock<DeckOverrides> = std::sync::OnceLock::new();
    OVERRIDES.get_or_init(|| DeckOverrides {
        seat0: load_deck_env("CRAB_DECK"),
        seat1: load_deck_env("CRAB_BOT_DECK"),
    })
}

/// Read, parse, and Modern-validate the decklist at `$key`. Exits the
/// process on a bad list — a misconfigured server shouldn't serve the
/// wrong deck silently.
pub(crate) fn load_deck_env(key: &str) -> Option<Vec<crabomination::cube::CardFactory>> {
    let path = env::var(key).ok()?;
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("{key}: can't read {path}: {e}");
        std::process::exit(1);
    });
    let parsed = crabomination::decklist::parse_decklist(&text);
    if !parsed.unknown.is_empty() {
        eprintln!("{key}: {} card(s) not in the catalog: {}", parsed.unknown.len(),
            parsed.unknown.join(", "));
        std::process::exit(1);
    }
    let defs: Vec<_> = parsed.main.iter().map(|f| f()).collect();
    if let Err(errs) = crabomination::format::validate_deck(&defs, crabomination::format::Format::Modern) {
        eprintln!("{key}: deck is not Modern-legal:");
        for e in &errs {
            eprintln!("  - {e}");
        }
        std::process::exit(1);
    }
    eprintln!("{key}: loaded {} cards from {path}", parsed.main.len());
    Some(parsed.main)
}

/// Default time the first client of a pair waits for an opponent before
/// being dropped. Configurable via `CRAB_PAIRING_TIMEOUT_SECS`.
pub(crate) const DEFAULT_PAIRING_TIMEOUT: Duration = Duration::from_secs(300);

/// Default total concurrent connection slots. A pair match consumes 2.
pub(crate) const DEFAULT_MAX_CONNS: usize = 100;

/// Default concurrent connection slots from any one remote IP.
pub(crate) const DEFAULT_MAX_CONNS_PER_IP: usize = 5;

/// Parse a non-negative integer env var (e.g. connection caps). Falls back
/// to `default` for missing, empty, or non-numeric values. `0` is preserved
/// (callers treat 0 as "unlimited").
pub(crate) fn usize_from_env(key: &str, default: usize) -> usize {
    match env::var(key).ok().as_deref() {
        None | Some("") => default,
        Some(s) => match s.parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                eprintln!(
                    "warning: {key}={s:?} not a non-negative integer — using default {default}",
                );
                default
            }
        },
    }
}

/// Read `CRAB_PAIRING_TIMEOUT_SECS` from the environment. Falls back to
/// `DEFAULT_PAIRING_TIMEOUT` for missing, empty, non-numeric, or zero values
/// (zero would mean "drop seat 0 instantly", almost certainly a misconfig).
pub(crate) fn pairing_timeout_from_env() -> Duration {
    match env::var("CRAB_PAIRING_TIMEOUT_SECS").ok().as_deref() {
        None | Some("") => DEFAULT_PAIRING_TIMEOUT,
        Some(s) => match s.parse::<u64>() {
            Ok(0) => {
                eprintln!(
                    "warning: CRAB_PAIRING_TIMEOUT_SECS=0 ignored — using default {}s",
                    DEFAULT_PAIRING_TIMEOUT.as_secs(),
                );
                DEFAULT_PAIRING_TIMEOUT
            }
            Ok(n) => Duration::from_secs(n),
            Err(_) => {
                eprintln!(
                    "warning: CRAB_PAIRING_TIMEOUT_SECS={s:?} not a non-negative integer — \
                     using default {}s",
                    DEFAULT_PAIRING_TIMEOUT.as_secs(),
                );
                DEFAULT_PAIRING_TIMEOUT
            }
        },
    }
}

