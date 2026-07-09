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
        Self::parse(env::var("CRAB_FORMAT").ok().as_deref())
    }

    /// Pure parser for `CRAB_FORMAT` (case-insensitive, whitespace-trimmed) so
    /// `"Cube"`, `" commander "`, etc. all resolve. Unknown values warn and
    /// fall back to demo. Split out from `from_env` for unit testing.
    pub(crate) fn parse(raw: Option<&str>) -> Self {
        let value = raw.map(|s| s.trim().to_ascii_lowercase());
        match value.as_deref() {
            Some("cube") => Self::Cube,
            Some("sos") | Some("strixhaven") => Self::Sos,
            Some("commander") | Some("edh") => Self::Commander,
            Some("demo") | Some("") | None => Self::Demo,
            Some(_) => {
                eprintln!(
                    "warning: CRAB_FORMAT={:?} not recognized — \
                     falling back to demo. Valid: \"demo\" | \"cube\" | \"sos\" | \"commander\".",
                    raw.unwrap_or_default()
                );
                Self::Demo
            }
        }
    }
    /// Decklist-override env keys that this format will silently ignore.
    /// Only the demo format honors `CRAB_DECK` / `CRAB_BOT_DECK`; the cube /
    /// SOS / commander formats build their own decks. Pure (takes the set of
    /// present keys) so it's unit-testable without touching the environment.
    pub(crate) fn ignored_override_keys<'a>(&self, set_keys: &[&'a str]) -> Vec<&'a str> {
        if matches!(self, Self::Demo) {
            return Vec::new();
        }
        ["CRAB_DECK", "CRAB_BOT_DECK"]
            .into_iter()
            .filter(|k| set_keys.contains(k))
            .collect()
    }

    pub(crate) fn build(&self) -> GameState {
        // Warn (don't silently misconfigure) when deck overrides are set for a
        // format that won't use them — mirrors `load_deck_env`'s philosophy.
        let present: Vec<&str> = ["CRAB_DECK", "CRAB_BOT_DECK"]
            .into_iter()
            .filter(|k| env::var(k).is_ok())
            .collect();
        for key in self.ignored_override_keys(&present) {
            eprintln!(
                "warning: {key} is set but ignored in {} format \
                 (deck overrides apply only to the demo format).",
                self.label(),
            );
        }
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
    let format = crabomination::format::Format::Modern;
    if let Err(errs) = crabomination::format::validate_deck(&defs, format) {
        eprintln!("{key}: deck is not Modern-legal:");
        for e in &errs {
            eprintln!("  - {e}");
        }
        std::process::exit(1);
    }
    // CR 702.139c — a sideboard companion must legalise the main deck.
    let side: Vec<_> = parsed.sideboard.iter().map(|f| f()).collect();
    for c in side.iter().filter(|c| c.companion.is_some()) {
        if let Err(e) = crabomination::format::companion_restriction_met(c, &defs, format.rules().min_deck_size) {
            eprintln!("{key}: {e}");
            std::process::exit(1);
        }
    }
    eprintln!("{key}: loaded {} cards from {path}", parsed.main.len());
    Some(parsed.main)
}

/// Default time the first client of a pair waits for an opponent before
/// being dropped. Configurable via `CRAB_PAIRING_TIMEOUT_SECS`.
pub(crate) const DEFAULT_PAIRING_TIMEOUT: Duration = Duration::from_secs(300);

/// Upper bound on `CRAB_PAIRING_TIMEOUT_SECS` (24h). A larger configured
/// value is clamped to this — an effectively-unbounded wait would pin a
/// connection slot indefinitely on a seat-0 client that never gets paired.
pub(crate) const MAX_PAIRING_TIMEOUT: Duration = Duration::from_secs(24 * 60 * 60);

/// Default total concurrent connection slots. A pair match consumes 2.
pub(crate) const DEFAULT_MAX_CONNS: usize = 100;

/// Default concurrent connection slots from any one remote IP.
pub(crate) const DEFAULT_MAX_CONNS_PER_IP: usize = 5;

/// Sane upper bound on connection-slot counts. A larger configured
/// `CRAB_MAX_CONNS` / `CRAB_MAX_CONNS_PER_IP` is clamped down to this
/// (mirroring [`MAX_PAIRING_TIMEOUT`]) so a typo like an extra zero can't try to
/// reserve an absurd slot budget / file-descriptor count. 100k is far above any
/// real deployment yet keeps the `Semaphore` permit count well-bounded.
pub(crate) const MAX_CONNS_CAP: usize = 100_000;

/// Parse a non-negative integer env var (e.g. connection caps). Falls back
/// to `default` for missing, empty, or non-numeric values. `0` is preserved
/// (callers treat 0 as "unlimited"). Surrounding whitespace is trimmed, matching
/// `Format::parse` — so `CRAB_MAX_CONNS=" 50 "` reads as `50`, not the default.
/// Pure core of [`usize_from_env_min`] — trims, then parses `raw`, falling back
/// to `default` for `None`/empty/non-numeric input. Split out so the parsing and
/// trimming behavior is unit-testable without touching the process environment.
pub(crate) fn parse_usize_or(raw: Option<&str>, key: &str, default: usize) -> usize {
    match raw.map(str::trim) {
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

/// Read `key` as a `usize`, enforcing a minimum: values below `min` (most
/// importantly `0`) are treated as a misconfig and fall back to `default`.
/// Pass `min == 0` for "any non-negative value is fine". The floor-only reader
/// behind the clamped connection-cap path — retained for the env-round-trip
/// tests that exercise the floor without the ceiling (`usize_from_env_clamped`
/// is the production entry point).
#[cfg(test)]
pub(crate) fn usize_from_env_min(key: &str, default: usize, min: usize) -> usize {
    parse_usize_min(env::var(key).ok().as_deref(), key, default, min)
}

/// Pure core of [`usize_from_env_min`]; testable without the process env.
pub(crate) fn parse_usize_min(raw: Option<&str>, key: &str, default: usize, min: usize) -> usize {
    let v = parse_usize_or(raw, key, default);
    if v < min {
        eprintln!("warning: {key}={v} below the minimum {min} — using default {default}");
        default
    } else {
        v
    }
}

/// Read `key` as a `usize`, enforcing both a floor (`min`) and a ceiling
/// (`max`). Below-floor values fall back to `default`; above-ceiling values are
/// clamped down to `max` (with a warning) rather than reset — a too-large cap is
/// still a usable cap, unlike a zero one. Pure core of
/// [`usize_from_env_clamped`].
pub(crate) fn parse_usize_clamped(
    raw: Option<&str>,
    key: &str,
    default: usize,
    min: usize,
    max: usize,
) -> usize {
    let v = parse_usize_min(raw, key, default, min);
    if v > max {
        eprintln!("warning: {key}={v} exceeds the maximum {max} — clamping to {max}");
        max
    } else {
        v
    }
}

/// Env wrapper for [`parse_usize_clamped`].
pub(crate) fn usize_from_env_clamped(key: &str, default: usize, min: usize, max: usize) -> usize {
    parse_usize_clamped(env::var(key).ok().as_deref(), key, default, min, max)
}

/// Read `CRAB_PAIRING_TIMEOUT_SECS` from the environment. Falls back to
/// `DEFAULT_PAIRING_TIMEOUT` for missing, empty, non-numeric, or zero values
/// (zero would mean "drop seat 0 instantly", almost certainly a misconfig).
pub(crate) fn pairing_timeout_from_env() -> Duration {
    parse_pairing_timeout(env::var("CRAB_PAIRING_TIMEOUT_SECS").ok().as_deref())
}

/// Pure core of [`pairing_timeout_from_env`]: trims, parses, zero-guards, and
/// clamps to [`MAX_PAIRING_TIMEOUT`]. Split out (like `parse_usize_or` /
/// `Format::parse`) so the whole ladder — default / zero / clamp / garbage — is
/// unit-testable without touching the process environment.
pub(crate) fn parse_pairing_timeout(raw: Option<&str>) -> Duration {
    match raw.map(str::trim) {
        None | Some("") => DEFAULT_PAIRING_TIMEOUT,
        Some(s) => match s.parse::<u64>() {
            Ok(0) => {
                eprintln!(
                    "warning: CRAB_PAIRING_TIMEOUT_SECS=0 ignored — using default {}s",
                    DEFAULT_PAIRING_TIMEOUT.as_secs(),
                );
                DEFAULT_PAIRING_TIMEOUT
            }
            Ok(n) => Duration::from_secs(n).min(MAX_PAIRING_TIMEOUT),
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


#[cfg(test)]
mod tests {
    use super::{
        parse_pairing_timeout, parse_usize_clamped, parse_usize_min, parse_usize_or, Format,
        DEFAULT_PAIRING_TIMEOUT, MAX_PAIRING_TIMEOUT,
    };

    #[test]
    fn parse_pairing_timeout_covers_the_ladder() {
        assert_eq!(parse_pairing_timeout(None), DEFAULT_PAIRING_TIMEOUT, "unset → default");
        assert_eq!(parse_pairing_timeout(Some("")), DEFAULT_PAIRING_TIMEOUT, "empty → default");
        assert_eq!(parse_pairing_timeout(Some("  90 ")).as_secs(), 90, "trimmed & parsed");
        assert_eq!(parse_pairing_timeout(Some("0")), DEFAULT_PAIRING_TIMEOUT, "zero → default");
        assert_eq!(parse_pairing_timeout(Some("nope")), DEFAULT_PAIRING_TIMEOUT, "garbage → default");
        assert_eq!(parse_pairing_timeout(Some("999999999")), MAX_PAIRING_TIMEOUT, "clamped to cap");
    }

    #[test]
    fn parse_usize_min_rejects_below_floor() {
        // Connection caps: a 0 (or below-floor) value is a misconfig → default.
        assert_eq!(parse_usize_min(Some("0"), "K", 9, 1), 9, "0 → default");
        assert_eq!(parse_usize_min(Some("1"), "K", 9, 1), 1, "at the floor is kept");
        assert_eq!(parse_usize_min(Some("50"), "K", 9, 1), 50, "above floor kept");
        assert_eq!(parse_usize_min(None, "K", 9, 1), 9, "unset → default");
        assert_eq!(parse_usize_min(Some("bad"), "K", 9, 1), 9, "garbage → default");
    }

    #[test]
    fn parse_usize_trims_and_falls_back() {
        assert_eq!(parse_usize_or(Some("50"), "K", 9), 50);
        assert_eq!(parse_usize_or(Some("  50 "), "K", 9), 50, "surrounding spaces trimmed");
        assert_eq!(parse_usize_or(Some("0"), "K", 9), 0, "0 preserved (unlimited)");
        assert_eq!(parse_usize_or(None, "K", 9), 9);
        assert_eq!(parse_usize_or(Some(""), "K", 9), 9);
        assert_eq!(parse_usize_or(Some("   "), "K", 9), 9, "blank → default");
        assert_eq!(parse_usize_or(Some("-3"), "K", 9), 9, "negative → default");
        assert_eq!(parse_usize_or(Some("ten"), "K", 9), 9, "non-numeric → default");
    }

    #[test]
    fn parse_usize_clamped_bounds_both_ends() {
        // Below floor → default; above ceiling → clamped to max; in-range passes.
        assert_eq!(parse_usize_clamped(Some("0"), "K", 100, 1, 100_000), 100, "below floor → default");
        assert_eq!(parse_usize_clamped(Some("50"), "K", 100, 1, 100_000), 50, "in range preserved");
        assert_eq!(
            parse_usize_clamped(Some("999999999"), "K", 100, 1, 100_000),
            100_000,
            "above ceiling clamped to max, not reset to default",
        );
        assert_eq!(parse_usize_clamped(None, "K", 100, 1, 100_000), 100, "unset → default");
    }

    #[test]
    fn format_parse_is_case_and_whitespace_insensitive() {
        assert!(matches!(Format::parse(Some("Cube")), Format::Cube));
        assert!(matches!(Format::parse(Some("  commander ")), Format::Commander));
        assert!(matches!(Format::parse(Some("EDH")), Format::Commander));
        assert!(matches!(Format::parse(Some("strixhaven")), Format::Sos));
    }

    #[test]
    fn format_parse_defaults_and_unknown_fall_back_to_demo() {
        assert!(matches!(Format::parse(None), Format::Demo));
        assert!(matches!(Format::parse(Some("")), Format::Demo));
        assert!(matches!(Format::parse(Some("   ")), Format::Demo));
        assert!(matches!(Format::parse(Some("pauper")), Format::Demo));
    }
}
