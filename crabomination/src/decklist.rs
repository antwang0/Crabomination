//! Plain-text decklist parsing — the Arena / MTGO export formats.
//!
//! Accepted line shapes (whitespace-tolerant, case-insensitive names):
//!
//! ```text
//! 4 Lightning Bolt              MTGO / plain
//! 4x Lightning Bolt             common variant
//! Lightning Bolt                bare name = count 1
//! 4 Lightning Bolt (M21) 162    Arena (set/collector suffix stripped)
//! SB: 2 Abrade                  MTGO sideboard prefix
//! ```
//!
//! Section headers ("Deck", "Mainboard", "Sideboard", "Commander",
//! "Companion") and comment lines (`#`, `//`) are recognised; Arena's
//! blank-line-then-more-cards convention also switches to the sideboard.
//! Unresolvable names are reported rather than dropped silently, so an
//! import UI can show exactly what's missing from the catalog.

use std::collections::HashMap;

use crate::cube::CardFactory;

/// Result of parsing a decklist: factory lists expanded by count, plus
/// the lines that didn't resolve against the catalog.
pub struct DecklistParse {
    pub main: Vec<CardFactory>,
    pub sideboard: Vec<CardFactory>,
    /// `"4x Snapcaster Mage"`-style entries for names not in the catalog.
    pub unknown: Vec<String>,
}

/// Strip an Arena-style trailing `(SET) 123` / `(SET) 123a` collector
/// suffix from a card-name line.
fn strip_arena_suffix(name: &str) -> &str {
    let trimmed = name.trim_end();
    // Find a trailing "... (XXX) 999x?" — set code in parens then digits.
    if let Some(open) = trimmed.rfind('(') {
        let after = &trimmed[open..];
        if let Some(close) = after.find(')') {
            let tail = after[close + 1..].trim();
            let is_collector = !tail.is_empty()
                && tail.chars().all(|c| c.is_ascii_alphanumeric())
                && tail.chars().next().is_some_and(|c| c.is_ascii_digit());
            if is_collector {
                return trimmed[..open].trim_end();
            }
        }
    }
    trimmed
}

/// Split a leading `4 ` / `4x ` count off a line; `None` count = 1.
fn split_count(line: &str) -> (u32, &str) {
    let mut digits_end = 0;
    for (i, c) in line.char_indices() {
        if c.is_ascii_digit() {
            digits_end = i + 1;
        } else {
            break;
        }
    }
    if digits_end == 0 {
        return (1, line);
    }
    let count: u32 = line[..digits_end].parse().unwrap_or(1);
    let rest = &line[digits_end..];
    let rest = rest.strip_prefix(['x', 'X']).unwrap_or(rest);
    if !rest.starts_with(char::is_whitespace) {
        // "4thcoming Card Name" — the digits were part of the name.
        return (1, line);
    }
    (count.clamp(1, 99), rest.trim_start())
}

/// Parse `text` against the full card registry. Never fails — unknown
/// names land in `unknown` and section noise is skipped.
pub fn parse_decklist(text: &str) -> DecklistParse {
    let by_name: HashMap<String, CardFactory> = crate::card_registry::all_known_factories()
        .into_iter()
        .map(|f| (f().name.to_ascii_lowercase(), f))
        .collect();

    let mut parse = DecklistParse {
        main: Vec::new(),
        sideboard: Vec::new(),
        unknown: Vec::new(),
    };
    let mut in_sideboard = false;
    let mut seen_cards = false;

    for raw in text.lines() {
        let mut line = raw.trim();
        if line.is_empty() {
            // Arena convention: the blank line after the maindeck starts
            // the sideboard. Only once cards have actually been seen, so
            // leading blank lines don't flip the section.
            if seen_cards {
                in_sideboard = true;
            }
            continue;
        }
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        match line.to_ascii_lowercase().as_str() {
            "deck" | "mainboard" | "main" | "maindeck" => {
                in_sideboard = false;
                continue;
            }
            "sideboard" | "side" => {
                in_sideboard = true;
                continue;
            }
            // Commander / companion headers: the next line is still a card —
            // file it with the maindeck (no dedicated zone on import yet).
            "commander" | "companion" => {
                in_sideboard = false;
                continue;
            }
            _ => {}
        }
        let mut line_is_sideboard = in_sideboard;
        if let Some(rest) = line.strip_prefix("SB:").or_else(|| line.strip_prefix("sb:")) {
            line_is_sideboard = true;
            line = rest.trim_start();
        }
        let (count, name_part) = split_count(line);
        let name = strip_arena_suffix(name_part);
        if name.is_empty() {
            continue;
        }
        seen_cards = true;
        match by_name.get(&name.to_ascii_lowercase()) {
            Some(&factory) => {
                let bucket = if line_is_sideboard {
                    &mut parse.sideboard
                } else {
                    &mut parse.main
                };
                for _ in 0..count {
                    bucket.push(factory);
                }
            }
            None => parse.unknown.push(format!("{count}x {name}")),
        }
    }
    parse
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_counts_sections_and_arena_suffixes() {
        let text = "\
# my deck
Deck
4 Lightning Bolt
2x Grizzly Bears
Forest
3 Lightning Bolt (M21) 162

2 Lightning Bolt
SB: 1 Grizzly Bears
";
        let parsed = parse_decklist(text);
        // 4 + 3 bolts + 2 bears + 1 forest in the main; the post-blank-line
        // bolts and the SB: line land in the sideboard.
        assert_eq!(parsed.main.len(), 10, "main: 4+3 bolts, 2 bears, 1 forest");
        assert_eq!(parsed.sideboard.len(), 3, "side: 2 bolts + 1 bears");
        assert!(parsed.unknown.is_empty(), "unknown: {:?}", parsed.unknown);
        let bolts = parsed
            .main
            .iter()
            .filter(|f| f().name == "Lightning Bolt")
            .count();
        assert_eq!(bolts, 7);
    }

    #[test]
    fn reports_unknown_names_instead_of_dropping() {
        let parsed = parse_decklist("3 Definitely Not A Real Card\n2 Lightning Bolt\n");
        assert_eq!(parsed.main.len(), 2);
        assert_eq!(parsed.unknown, vec!["3x Definitely Not A Real Card".to_string()]);
    }

    #[test]
    fn bare_names_and_digit_led_names_parse() {
        let parsed = parse_decklist("Lightning Bolt\n");
        assert_eq!(parsed.main.len(), 1);
        // A leading count must be followed by whitespace to count as one.
        let (count, rest) = split_count("4x Ornithopter");
        assert_eq!((count, rest), (4, "Ornithopter"));
        let (count, rest) = split_count("Ornithopter");
        assert_eq!((count, rest), (1, "Ornithopter"));
    }
}
