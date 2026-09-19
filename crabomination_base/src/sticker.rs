//! CR 123 — stickers (Unfinity). Only **name stickers** (CR 123.6) are
//! modeled; ability, power/toughness and art stickers are not.
//!
//! The data half lives here (in the base crate) because `CardInstance`'s
//! `Deserialize` must rebuild a stickered name; the per-game half — which
//! three sheets each player drew (CR 123.2a/c) — is `ColdState::sticker_sheets`
//! in the engine, drawn lazily the first time a player is asked to put a
//! sticker on something, so a game without sticker cards stores and pays
//! nothing.
//!
//! ⚠ **The sheet contents are a STAND-IN, not Unfinity's printed sheets.**
//! The real inserts (CR 123.2 — listed on Gatherer) are not reproduced here
//! because their exact texts could not be verified; the words below are
//! invented, chosen so their unique-vowel counts (CR 123.6e) span 1-5 like
//! real name stickers do. Every player brings this same collection
//! (CR 123.2a's "at least ten, each unique" is not enforced — six sheets).

/// One sticker sheet: its (stand-in) name and the name stickers on it.
#[derive(Debug, Clone, Copy)]
pub struct StickerSheet {
    pub name: &'static str,
    pub name_stickers: [&'static str; NAME_STICKERS_PER_SHEET],
}

/// Name stickers per sheet in the stand-in collection.
pub const NAME_STICKERS_PER_SHEET: usize = 3;

/// CR 123.2c — a player has access to the stickers on this many sheets.
pub const SHEETS_PER_PLAYER: usize = 3;

/// The default sticker-sheet collection every player brings (STAND-IN — see
/// the module doc). Index = sheet id.
pub const DEFAULT_STICKER_SHEETS: &[StickerSheet] = &[
    StickerSheet { name: "Stand-in Sheet A", name_stickers: ["Wobbly", "Audacious", "Grim"] },
    StickerSheet { name: "Stand-in Sheet B", name_stickers: ["Eerie", "Fabulous", "Spry"] },
    StickerSheet { name: "Stand-in Sheet C", name_stickers: ["Quizzical", "Bold", "Tiny"] },
    StickerSheet { name: "Stand-in Sheet D", name_stickers: ["Rowdy", "Enormous", "Crisp"] },
    StickerSheet { name: "Stand-in Sheet E", name_stickers: ["Mysterious", "Sly", "Dapper"] },
    StickerSheet { name: "Stand-in Sheet F", name_stickers: ["Hilarious", "Gnarly", "Plump"] },
];

/// A name sticker's identity: `sheet * NAME_STICKERS_PER_SHEET + slot`.
/// CR 123.3a — two stickers are never the same sticker even with the same
/// text; the id is the sticker, the text is only what it says.
pub type NameStickerId = u16;

/// The ids of the name stickers on `sheet`.
pub fn name_stickers_on_sheet(sheet: u8) -> impl Iterator<Item = NameStickerId> {
    let base = sheet as usize * NAME_STICKERS_PER_SHEET;
    (base..base + NAME_STICKERS_PER_SHEET).map(|i| i as NameStickerId)
}

/// The word printed on name sticker `id` (`""` for an unknown id).
pub fn name_sticker_text(id: NameStickerId) -> &'static str {
    let (sheet, slot) =
        (id as usize / NAME_STICKERS_PER_SHEET, id as usize % NAME_STICKERS_PER_SHEET);
    DEFAULT_STICKER_SHEETS.get(sheet).map_or("", |s| s.name_stickers[slot])
}

/// CR 123.6e — the number of different vowels (A, E, I, O, U, Y; case-
/// insensitive) on a name sticker's text.
pub fn unique_vowels(text: &str) -> u32 {
    let mut seen = 0u8;
    for ch in text.chars() {
        let bit = match ch.to_ascii_lowercase() {
            'a' => 1,
            'e' => 2,
            'i' => 4,
            'o' => 8,
            'u' => 16,
            'y' => 32,
            _ => 0,
        };
        seen |= bit;
    }
    seen.count_ones()
}

/// CR 123.6a — a blank line ("_____") is not a word.
fn is_blank(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c == '_')
}

/// CR 123.6b — the position the engine picks for a new name sticker on an
/// object named `name`: where its first blank line sits (the spot a
/// "_____" card prints for the sticker), else the end of the name. Counted
/// in words (CR 123.6a), so blanks don't count.
pub fn default_name_sticker_position(name: &str) -> u8 {
    let mut words = 0u8;
    for w in name.split_whitespace() {
        if is_blank(w) {
            return words;
        }
        words = words.saturating_add(1);
    }
    words
}

/// CR 123.6b/c — `printed` with each `(sticker, position)` added in
/// timestamp order. A sticker goes after `position` words (blanks don't
/// count), or at the end if the name has fewer words; one that lands on a
/// blank line fills it. Other words are never modified or removed.
pub fn stickered_name(printed: &str, stickers: &[(NameStickerId, u8)]) -> String {
    let mut words: Vec<&str> = printed.split_whitespace().collect();
    for &(id, pos) in stickers {
        let text = name_sticker_text(id);
        // Index in `words` just after `pos` real words.
        let mut real = 0usize;
        let mut at = words.len();
        for (i, w) in words.iter().enumerate() {
            if real == pos as usize {
                at = i;
                break;
            }
            if !is_blank(w) {
                real += 1;
            }
        }
        if words.get(at).is_some_and(|w| is_blank(w)) {
            words[at] = text;
        } else {
            words.insert(at, text);
        }
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_vowels_counts_each_vowel_once_including_y() {
        assert_eq!(unique_vowels("Audacious"), 4); // a u i o
        assert_eq!(unique_vowels("Mysterious"), 5); // y e i o u
        assert_eq!(unique_vowels("Spry"), 1);
        assert_eq!(unique_vowels("EERIE"), 2);
    }

    #[test]
    fn a_sticker_fills_the_blank_else_goes_where_placed() {
        // "Wobbly" is sticker 0, "Grim" is sticker 2.
        let pos = default_name_sticker_position("_____ Goblin");
        assert_eq!(pos, 0);
        assert_eq!(stickered_name("_____ Goblin", &[(0, pos)]), "Wobbly Goblin");
        assert_eq!(default_name_sticker_position("Bear Cub"), 2);
        assert_eq!(stickered_name("Bear Cub", &[(2, 2)]), "Bear Cub Grim");
        assert_eq!(stickered_name("Bear Cub", &[(2, 1)]), "Bear Grim Cub");
        assert_eq!(stickered_name("Bear Cub", &[(2, 9)]), "Bear Cub Grim");
    }
}
