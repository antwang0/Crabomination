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
//! "Companion", "Maybeboard"; a trailing `:` or `(N)` count and a `//` / `#`
//! comment prefix are tolerated) and comment lines (`#`, `//`) are
//! recognised; Arena's blank-line-then-more-cards convention also switches to
//! the sideboard.
//!
//! The commander designation (CR 903.3) is kept rather than folded into the
//! maindeck. Any of these marks a card as a commander:
//!
//! ```text
//! Commander                                  Arena / Moxfield header, leading
//! 1 Atraxa, Praetors' Voice                  or trailing ("Commander (1)" too)
//!
//! 1 Atraxa, Praetors' Voice *CMDR*           Moxfield inline marker
//! 1x Atraxa, Praetors' Voice (C16) 28 [Commander{top}]   Archidekt category
//! ```
//!
//! A blank line ends a Commander section and returns to the maindeck (Arena's
//! `Commander` / card / blank / `Deck` export shape). Other trailing export
//! markers — Moxfield's `*F*`, Archidekt's `[Category]` and `^Tag^` — are
//! stripped, and a Maybeboard / Considering / Tokens section is skipped.
//! MTGO's convention (the commander is the sideboard) is recognised by
//! [`DecklistParse::commander_list`], which is also where a Commander import
//! is validated. Unresolvable names are reported rather than dropped
//! silently, so an import UI can show exactly what's missing from the catalog.
//!
//! Names match without regard to case, accents and ligatures, or typographic
//! quotes and dashes ([`fold_name`]: "Lim-Dul", "Juzam Djinn", a curly
//! apostrophe pasted from a web page), and an unknown name carries the
//! catalog names it most likely meant ([`UnknownCard::suggestions`]).

use std::fmt;
use std::sync::OnceLock;

use crate::fxhash::HashMap;

use crate::cube::CardFactory;

/// Result of parsing a decklist: factory lists expanded by count, plus
/// the lines that didn't resolve against the catalog.
pub struct DecklistParse {
    pub main: Vec<CardFactory>,
    pub sideboard: Vec<CardFactory>,
    /// CR 717.2 — the supplementary Attraction deck. Attraction cards can't be
    /// in a main deck or sideboard, so they're routed here from any section.
    pub attractions: Vec<CardFactory>,
    /// CR 903.3 — the cards listed under a Commander header or marked
    /// `*CMDR*` / `[Commander]`: one, or two for a Partner / Background /
    /// Doctor's companion pair. The parser only files them; legality is
    /// [`DecklistParse::commander_list`]'s job.
    pub commanders: Vec<CardFactory>,
    /// The lines whose names aren't in the catalog, in list order.
    pub unknown: Vec<UnknownCard>,
    /// Every sideboard card got there by Arena's blank-line convention alone
    /// — no `Sideboard` header, no `SB:` prefix. A Commander deck has no
    /// sideboard (CR 903.5e), so in one a blank line is only grouping.
    pub sideboard_is_implicit: bool,
}

/// A decklist line whose card isn't in the catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownCard {
    pub count: u32,
    /// The name as the list wrote it (set / collector suffix and export
    /// markers stripped).
    pub name: String,
    /// Up to three catalog names it most likely meant: the cards it names
    /// with the subtitle left off ("Atraxa"), else the nearest names within a
    /// typo's reach ("Lightnig Bolt"). Empty when nothing is that close.
    pub suggestions: Vec<&'static str>,
}

/// `2x Lightnig Bolt (did you mean Lightning Bolt?)`.
impl fmt::Display for UnknownCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x {}", self.count, self.name)?;
        if let Some((last, rest)) = self.suggestions.split_last() {
            f.write_str(" (did you mean ")?;
            if !rest.is_empty() {
                write!(f, "{} or ", rest.join(", "))?;
            }
            write!(f, "{last}?)")?;
        }
        Ok(())
    }
}

/// A decklist read as a Commander deck: the command zone and the rest.
pub struct CommanderList {
    pub commanders: Vec<CardFactory>,
    pub main: Vec<CardFactory>,
}

impl DecklistParse {
    /// Read this list as a Commander deck and validate it with
    /// [`crate::format::validate_commander_deck`]: legal commander(s), a valid
    /// pair (CR 702.124), 100 cards singleton counting the commanders
    /// (CR 903.5a/b), and every card inside the combined colour identity
    /// (CR 903.5c). `Err` carries readable messages, commander errors first.
    ///
    /// With no Commander section, MTGO's export convention is honoured: a
    /// sideboard of one or two legendary cards *is* the command zone. Anything
    /// else without one is refused — guessing which legend leads a 100-card
    /// pile is not the importer's call. Unknown names are the caller's to
    /// report; this reads only the cards that resolved.
    pub fn commander_list(&self) -> Result<CommanderList, Vec<String>> {
        let mut commanders = self.commanders.clone();
        let mut main = self.main.clone();
        let mut sideboard = self.sideboard.clone();
        if commanders.is_empty()
            && (1..=2).contains(&sideboard.len())
            && sideboard.iter().all(|f| f().is_legendary())
        {
            commanders = std::mem::take(&mut sideboard);
        }
        if commanders.is_empty() {
            return Err(vec![
                "No commander section: list your commander under a `Commander` header \
                 (or mark it *CMDR*)"
                    .to_string(),
            ]);
        }
        if self.sideboard_is_implicit {
            main.append(&mut sideboard);
        }
        let deck = crate::format::Deck {
            main: main.iter().map(|f| f()).collect(),
            commanders: commanders.iter().map(|f| f()).collect(),
            sideboard: sideboard.iter().map(|f| f()).collect(),
        };
        match crate::format::validate_commander_deck(&deck) {
            Ok(()) => Ok(CommanderList { commanders, main }),
            Err((generic, cmd)) => Err(cmd
                .iter()
                .map(ToString::to_string)
                .chain(generic.iter().map(ToString::to_string))
                .collect()),
        }
    }
}

/// Which part of the list a line belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Main,
    Side,
    Commander,
    /// Maybeboard / Considering / Tokens: listed, but not part of the deck.
    Ignored,
}

/// Recognise a section header, tolerating a trailing `:` or `(N)` count
/// ("Commander (1)", "Deck:") and a `//` / `#` comment prefix ("// Sideboard").
/// `None` for anything else, including an ordinary comment.
fn section_header(line: &str) -> Option<Section> {
    let mut h = line.trim();
    for prefix in ["//", "#"] {
        if let Some(rest) = h.strip_prefix(prefix) {
            h = rest.trim();
        }
    }
    let h = h.trim_end_matches(':').trim_end();
    let h = match h.rfind('(') {
        Some(open)
            if h.ends_with(')')
                && open + 1 < h.len() - 1
                && h[open + 1..h.len() - 1].chars().all(|c| c.is_ascii_digit()) =>
        {
            h[..open].trim_end()
        }
        _ => h,
    };
    match h.to_ascii_lowercase().as_str() {
        "deck" | "mainboard" | "main" | "maindeck" | "main deck" => Some(Section::Main),
        "sideboard" | "side" => Some(Section::Side),
        // CR 717.2 — the Attraction deck is its own supplementary list.
        // A header is optional: Attraction cards route there from any
        // section, since they can't legally be anywhere else.
        "attractions" | "attraction deck" => Some(Section::Main),
        "commander" | "commanders" => Some(Section::Commander),
        // Companion: the card still files with the maindeck (no dedicated
        // outside-the-game zone on import yet).
        "companion" => Some(Section::Main),
        "maybeboard" | "maybe" | "considering" | "tokens" => Some(Section::Ignored),
        // Arena's export opens with `About` / `Name <deck name>` before its
        // `Deck` header; the name isn't a card.
        "about" => Some(Section::Ignored),
        _ => None,
    }
}

/// Strip trailing export markers — Moxfield's `*F*` / `*CMDR*`, Archidekt's
/// `[Category]` / `[Commander{top}]` and `^Have,#37d67a^` — and report
/// whether one of them designated the card a commander.
fn strip_markers(mut name: &str) -> (&str, bool) {
    let mut commander = false;
    loop {
        let t = name.trim_end();
        let open = match t.chars().last() {
            Some(c @ ('*' | '^')) => t[..t.len() - 1].rfind(c),
            Some(']') => t.rfind('['),
            _ => None,
        };
        // A marker must follow a name, never be the whole line.
        let Some(open) = open.filter(|&i| i > 0) else { return (t, commander) };
        let marker = t[open..].to_ascii_lowercase();
        if marker.contains("cmdr") || marker.contains("commander") {
            commander = true;
        }
        name = &t[..open];
    }
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

/// A card name as an import compares it: lowercase, accents and ligatures
/// dropped (Lim-Dûl → lim-dul, Æ → ae), typographic quotes and dashes made
/// plain, whitespace runs collapsed — so a list typed without accents or
/// copied off a web page still finds its cards.
pub fn fold_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars().flat_map(char::to_lowercase) {
        match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => out.push('a'),
            'æ' => out.push_str("ae"),
            'ç' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' => out.push('i'),
            'ñ' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => out.push('o'),
            'œ' => out.push_str("oe"),
            'ù' | 'ú' | 'û' | 'ü' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            '\u{2018}' | '\u{2019}' | '\u{02bc}' | '`' | '´' => out.push('\''),
            '\u{201c}' | '\u{201d}' => out.push('"'),
            '\u{2010}'..='\u{2015}' | '\u{2212}' => out.push('-'),
            c if c.is_whitespace() => {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            c => out.push(c),
        }
    }
    let len = out.trim_end().len();
    out.truncate(len);
    out
}

/// The catalog as an import reads it, built once.
struct ImportIndex {
    /// Every card under its folded name — and, where no card has that name,
    /// under its back face's (a list that names a double-faced card by its
    /// back). A name two factories share goes to the later one, as the
    /// per-parse map this replaced did.
    by_key: HashMap<String, CardFactory>,
    /// Each card name, folded and printed, for [`suggest`].
    names: Vec<(String, &'static str)>,
}

fn import_index() -> &'static ImportIndex {
    static INDEX: OnceLock<ImportIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut by_key: HashMap<String, CardFactory> = HashMap::default();
        let mut names = Vec::new();
        let mut backs = Vec::new();
        for f in crate::card_registry::all_known_factories() {
            let def = f();
            let key = fold_name(def.name);
            if by_key.insert(key.clone(), f).is_none() {
                names.push((key, def.name));
            }
            if let Some(back) = def.back_face.as_ref() {
                backs.push((fold_name(back.name), f));
            }
        }
        for (key, f) in backs {
            by_key.entry(key).or_insert(f);
        }
        ImportIndex { by_key, names }
    })
}

/// What an unknown (folded) name most likely meant — see
/// [`UnknownCard::suggestions`]. A typo's reach is one edit per five
/// letters, at least one and at most three.
fn suggest(key: &str) -> Vec<&'static str> {
    const MAX: usize = 3;
    let names = &import_index().names;
    // "Atraxa" for "Atraxa, Praetors' Voice": the name before a subtitle.
    let mut subtitled: Vec<&(String, &'static str)> = names
        .iter()
        .filter(|(k, _)| k.len() > key.len() && k.starts_with(key) && k[key.len()..].starts_with(','))
        .collect();
    if !subtitled.is_empty() {
        subtitled.sort_by_key(|(k, _)| k.len());
        return subtitled.iter().take(MAX).map(|(_, n)| *n).collect();
    }
    let reach = (key.chars().count() / 5).clamp(1, 3);
    let mut best = reach;
    let mut found: Vec<&'static str> = Vec::new();
    for (k, name) in names {
        if let Some(d) = edit_distance_within(key.as_bytes(), k.as_bytes(), best) {
            if d < best || found.is_empty() {
                found.clear();
            }
            best = d;
            if found.len() < MAX {
                found.push(name);
            }
        }
    }
    found
}

/// The optimal-string-alignment distance between `a` and `b` (insert,
/// delete, substitute, or swap two neighbours, one edit each), or `None`
/// once it has to exceed `max`.
fn edit_distance_within(a: &[u8], b: &[u8], max: usize) -> Option<usize> {
    if a.len().abs_diff(b.len()) > max {
        return None;
    }
    let mut before: Vec<usize> = vec![0; b.len() + 1];
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        let mut row_min = i;
        for j in 1..=b.len() {
            let sub = prev[j - 1] + usize::from(a[i - 1] != b[j - 1]);
            let mut d = sub.min(prev[j] + 1).min(cur[j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d = d.min(before[j - 2] + 1);
            }
            cur[j] = d;
            row_min = row_min.min(d);
        }
        if row_min > max {
            return None;
        }
        std::mem::swap(&mut before, &mut prev);
        std::mem::swap(&mut prev, &mut cur);
    }
    Some(prev[b.len()]).filter(|&d| d <= max)
}

/// Parse `text` against the full card registry. Never fails — unknown
/// names land in `unknown` and section noise is skipped.
pub fn parse_decklist(text: &str) -> DecklistParse {
    let by_key = &import_index().by_key;

    let mut parse = DecklistParse {
        main: Vec::new(),
        sideboard: Vec::new(),
        attractions: Vec::new(),
        commanders: Vec::new(),
        unknown: Vec::new(),
        sideboard_is_implicit: true,
    };
    let mut section = Section::Main;
    let mut seen_cards = false;

    for raw in text.lines() {
        let mut line = raw.trim();
        if line.is_empty() {
            match section {
                // Arena convention: the blank line after the maindeck starts
                // the sideboard. Only once cards have actually been seen, so
                // leading blank lines don't flip the section.
                Section::Main if seen_cards => section = Section::Side,
                // Arena's Commander export is `Commander` / card / blank /
                // `Deck`; a list that drops the `Deck` header still means the
                // maindeck follows the blank line.
                Section::Commander => section = Section::Main,
                _ => {}
            }
            continue;
        }
        if let Some(next) = section_header(line) {
            if next == Section::Side {
                parse.sideboard_is_implicit = false;
            }
            section = next;
            continue;
        }
        if line.starts_with('#') || line.starts_with("//") || section == Section::Ignored {
            continue;
        }
        let mut line_section = section;
        if let Some(rest) = line.strip_prefix("SB:").or_else(|| line.strip_prefix("sb:")) {
            line_section = Section::Side;
            parse.sideboard_is_implicit = false;
            line = rest.trim_start();
        }
        let (count, name_part) = split_count(line);
        let (name_part, marked_commander) = strip_markers(name_part);
        if marked_commander {
            line_section = Section::Commander;
        }
        let name = strip_arena_suffix(name_part);
        if name.is_empty() {
            continue;
        }
        seen_cards = true;
        // `Front // Back` is how Arena and MTGO export split, modal
        // double-faced, and (here) prepare cards, and it is what a
        // human copying a card's printed name writes. The catalog keys
        // those cards by their front face alone, so try the full string
        // first — a card genuinely *named* with a slash still wins —
        // then fall back to the part before the separator.
        let name: &str = match by_key.contains_key(&fold_name(name)) {
            true => name,
            false => match name.split_once("//") {
                Some((front, _)) if !front.trim().is_empty() => front.trim(),
                _ => name,
            },
        };
        let key = fold_name(name);
        match by_key.get(&key) {
            Some(&factory) => {
                let bucket = if !factory().attraction_lights.is_empty() {
                    &mut parse.attractions
                } else {
                    match line_section {
                        Section::Side => &mut parse.sideboard,
                        Section::Commander => &mut parse.commanders,
                        Section::Main | Section::Ignored => &mut parse.main,
                    }
                };
                for _ in 0..count {
                    bucket.push(factory);
                }
            }
            None => parse.unknown.push(UnknownCard { count, name: name.to_string(), suggestions: suggest(&key) }),
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

    /// CR 717.2 — Attraction cards route to the Attraction deck no matter
    /// which section they were listed under.
    #[test]
    fn attractions_route_to_their_own_deck() {
        let parsed = parse_decklist(
            "Deck\n2 Lightning Bolt\n1 Information Booth\nSideboard\n1 Fortune Teller\n",
        );
        assert_eq!(parsed.main.len(), 2);
        assert!(parsed.sideboard.is_empty());
        assert_eq!(parsed.attractions.len(), 2);
        assert!(parsed.attractions.iter().all(|f| !f().attraction_lights.is_empty()));
    }

    /// Arena's export header — `About` and the deck's `Name` — is skipped,
    /// not read as two unknown cards.
    #[test]
    fn an_arena_about_header_is_not_cards() {
        let parsed = parse_decklist("About\nName Burn\n\nDeck\n4 Lightning Bolt\n");
        assert!(parsed.unknown.is_empty(), "{:?}", parsed.unknown);
        assert_eq!(parsed.main.len(), 4);
    }

    #[test]
    fn reports_unknown_names_instead_of_dropping() {
        let parsed = parse_decklist("3 Definitely Not A Real Card\n2 Lightning Bolt\n");
        assert_eq!(parsed.main.len(), 2);
        let [bogus] = parsed.unknown.as_slice() else { panic!("{:?}", parsed.unknown) };
        assert_eq!(bogus.to_string(), "3x Definitely Not A Real Card");
    }

    /// A list typed without accents, or pasted from a web page with curly
    /// apostrophes and odd spacing, finds its cards; a near miss names what
    /// it probably meant, and a name with nothing close offers nothing.
    #[test]
    fn names_match_past_accents_and_quotes_and_a_miss_suggests_the_card() {
        let parsed = parse_decklist(
            "1 Juzam Djinn
1 NAZGUL
1 Atraxa,  Praetors\u{2019} Voice
1 Jokulhaups
",
        );
        assert!(parsed.unknown.is_empty(), "{:?}", parsed.unknown);
        let names: Vec<_> = parsed.main.iter().map(|f| f().name).collect();
        assert_eq!(names, ["Juzám Djinn", "Nazgûl", "Atraxa, Praetors' Voice", "Jökulhaups"]);

        let parsed = parse_decklist("2 Lightnig Bolt
1 Sol Rnig
1 Atraxa
1 Qwxzv Plumbus
");
        let lines: Vec<String> = parsed.unknown.iter().map(ToString::to_string).collect();
        assert_eq!(
            lines,
            [
                "2x Lightnig Bolt (did you mean Lightning Bolt?)",
                "1x Sol Rnig (did you mean Sol Ring?)",
                "1x Atraxa (did you mean Atraxa, Grand Unifier or Atraxa, Praetors' Voice?)",
                "1x Qwxzv Plumbus",
            ],
        );
        assert!(parsed.main.is_empty(), "a suggestion is never imported for you");
    }

    /// Folding may only merge spellings of one name: two cards whose names
    /// fold alike would leave one of them impossible to import.
    #[test]
    fn no_two_card_names_fold_alike() {
        let mut by_key: HashMap<String, &'static str> = HashMap::default();
        let mut clashes = Vec::new();
        for f in crate::card_registry::all_known_factories() {
            let name = f().name;
            if let Some(other) = by_key.insert(fold_name(name), name)
                && other != name
            {
                clashes.push((other, name));
            }
        }
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn edit_distance_counts_a_swap_as_one_edit_and_stops_at_the_cap() {
        assert_eq!(edit_distance_within(b"sol ring", b"sol ring", 1), Some(0));
        assert_eq!(edit_distance_within(b"sol rnig", b"sol ring", 1), Some(1));
        assert_eq!(edit_distance_within(b"lightnig bolt", b"lightning bolt", 2), Some(1));
        assert_eq!(edit_distance_within(b"kitten", b"sitting", 3), Some(3));
        assert_eq!(edit_distance_within(b"kitten", b"sitting", 2), None);
        assert_eq!(fold_name("  Lim-D\u{fb}l\u{2019}s   Vault "), "lim-dul's vault");
        assert_eq!(fold_name("\u{c6}ther Vial"), "aether vial");
    }

    /// `Front // Back` — how Arena/MTGO export split, MDFC and prepare
    /// cards, and what a human writes when copying the printed name.
    /// The catalog keys prepare cards by their front face, so a
    /// hand-built sealed list full of `Studious First-Year // Rampant
    /// Growth` lines used to resolve to *nothing*, drop under 40 cards,
    /// and silently hand the seat a generated deck.
    #[test]
    fn split_and_prepare_names_resolve_to_their_front_face() {
        let parsed = parse_decklist(
            "1 Studious First-Year // Rampant Growth\n1 Emeritus of Ideation // Ancestral Recall\n",
        );
        assert!(parsed.unknown.is_empty(), "unresolved: {:?}", parsed.unknown);
        assert_eq!(parsed.main.len(), 2);
        assert_eq!(parsed.main[0]().name, "Studious First-Year");
        assert_eq!(parsed.main[1]().name, "Emeritus of Ideation");

        // The front-face fallback must not rescue a genuinely unknown
        // card, or typos would silently import as something else.
        let bogus = parse_decklist("1 Not A Card // Also Not A Card\n");
        assert_eq!(bogus.main.len(), 0);
        assert_eq!(bogus.unknown.len(), 1);

        // A plain front-face line keeps working, and a full-line `//`
        // comment is still a comment.
        let plain = parse_decklist("// a comment\n1 Studious First-Year\n");
        assert!(plain.unknown.is_empty());
        assert_eq!(plain.main.len(), 1);
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
