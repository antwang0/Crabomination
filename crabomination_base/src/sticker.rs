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
//! The sheets are Unfinity's 48 printed inserts (Scryfall set `sunf`); a
//! sheet's name is its three name stickers. Every seat brings all 48 (CR
//! 123.2a's "at least ten, each unique") and draws three at random.

/// One sticker sheet: its name and the name stickers on it.
#[derive(Debug, Clone, Copy)]
pub struct StickerSheet {
    pub name: &'static str,
    pub name_stickers: [&'static str; NAME_STICKERS_PER_SHEET],
}

/// Name stickers per Unfinity sheet.
pub const NAME_STICKERS_PER_SHEET: usize = 3;

/// CR 123.2c — a player has access to the stickers on this many sheets.
pub const SHEETS_PER_PLAYER: usize = 3;

/// The sticker-sheet collection every player brings: the Unfinity inserts,
/// in collector-number order. Index = sheet id.
pub const DEFAULT_STICKER_SHEETS: &[StickerSheet] = &[
    StickerSheet { name: "Eldrazi Guacamole Tightrope", name_stickers: ["Eldrazi", "Guacamole", "Tightrope"] },
    StickerSheet { name: "Trendy Circus Pirate", name_stickers: ["Trendy", "Circus", "Pirate"] },
    StickerSheet { name: "Night Brushwagg Ringmaster", name_stickers: ["Night", "Brushwagg", "Ringmaster"] },
    StickerSheet { name: "Urza's Dark Cannonball", name_stickers: ["Urza's", "Dark", "Cannonball"] },
    StickerSheet { name: "Misunderstood Trapeze Elf", name_stickers: ["Misunderstood", "Trapeze", "Elf"] },
    StickerSheet { name: "Zombie Cheese Magician", name_stickers: ["Zombie", "Cheese", "Magician"] },
    StickerSheet { name: "Carnival Elephant Meteor", name_stickers: ["Carnival", "Elephant", "Meteor"] },
    StickerSheet { name: "Happy Dead Squirrel", name_stickers: ["Happy", "Dead", "Squirrel"] },
    StickerSheet { name: "Slimy Burrito Illusion", name_stickers: ["Slimy", "Burrito", "Illusion"] },
    StickerSheet { name: "Spooky Clown Mox", name_stickers: ["Spooky", "Clown", "Mox"] },
    StickerSheet { name: "Mystic Doom Sandwich", name_stickers: ["Mystic", "Doom", "Sandwich"] },
    StickerSheet { name: "Narrow-Minded Baloney Fireworks", name_stickers: ["Narrow-Minded", "Baloney", "Fireworks"] },
    StickerSheet { name: "Unsanctioned Ancient Juggler", name_stickers: ["Unsanctioned", "Ancient", "Juggler"] },
    StickerSheet { name: "Deep-Fried Plague Myr", name_stickers: ["Deep-Fried", "Plague", "Myr"] },
    StickerSheet { name: "Contortionist Otter Storm", name_stickers: ["Contortionist", "Otter", "Storm"] },
    StickerSheet { name: "Sticky Kavu Daredevil", name_stickers: ["Sticky", "Kavu", "Daredevil"] },
    StickerSheet { name: "Goblin Coward Parade", name_stickers: ["Goblin", "Coward", "Parade"] },
    StickerSheet { name: "Phyrexian Midway Bamboozle", name_stickers: ["Phyrexian", "Midway", "Bamboozle"] },
    StickerSheet { name: "Eternal Acrobat Toast", name_stickers: ["Eternal", "Acrobat", "Toast"] },
    StickerSheet { name: "Jetpack Death Seltzer", name_stickers: ["Jetpack", "Death", "Seltzer"] },
    StickerSheet { name: "Demonic Tourist Laser", name_stickers: ["Demonic", "Tourist", "Laser"] },
    StickerSheet { name: "Cursed Firebreathing Yogurt", name_stickers: ["Cursed", "Firebreathing", "Yogurt"] },
    StickerSheet { name: "Ancestral Hot Dog Minotaur", name_stickers: ["Ancestral", "Hot Dog", "Minotaur"] },
    StickerSheet { name: "Familiar Beeble Mascot", name_stickers: ["Familiar", "Beeble", "Mascot"] },
    StickerSheet { name: "Giant Mana Cake", name_stickers: ["Giant", "Mana", "Cake"] },
    StickerSheet { name: "Snazzy Aether Homunculus", name_stickers: ["Snazzy", "Aether", "Homunculus"] },
    StickerSheet { name: "Squid Fire Knight", name_stickers: ["Squid", "Fire", "Knight"] },
    StickerSheet { name: "Cool Fluffy Loxodon", name_stickers: ["Cool", "Fluffy", "Loxodon"] },
    StickerSheet { name: "Space Fungus Snickerdoodle", name_stickers: ["Space", "Fungus", "Snickerdoodle"] },
    StickerSheet { name: "Playable Delusionary Hydra", name_stickers: ["Playable", "Delusionary", "Hydra"] },
    StickerSheet { name: "Wrinkly Monkey Shenanigans", name_stickers: ["Wrinkly", "Monkey", "Shenanigans"] },
    StickerSheet { name: "Geek Lotus Warrior", name_stickers: ["Geek", "Lotus", "Warrior"] },
    StickerSheet { name: "Primal Elder Kitty", name_stickers: ["Primal", "Elder", "Kitty"] },
    StickerSheet { name: "Sassy Gremlin Blood", name_stickers: ["Sassy", "Gremlin", "Blood"] },
    StickerSheet { name: "Yawgmoth Merfolk Soul", name_stickers: ["Yawgmoth", "Merfolk", "Soul"] },
    StickerSheet { name: "Unassuming Gelatinous Serpent", name_stickers: ["Unassuming", "Gelatinous", "Serpent"] },
    StickerSheet { name: "Squishy Sphinx Ninja", name_stickers: ["Squishy", "Sphinx", "Ninja"] },
    StickerSheet { name: "Unique Charmed Pants", name_stickers: ["Unique", "Charmed", "Pants"] },
    StickerSheet { name: "Unhinged Beast Hunt", name_stickers: ["Unhinged", "Beast", "Hunt"] },
    StickerSheet { name: "Wild Ogre Bupkis", name_stickers: ["Wild", "Ogre", "Bupkis"] },
    StickerSheet { name: "Notorious Sliver War", name_stickers: ["Notorious", "Sliver", "War"] },
    StickerSheet { name: "Weird Angel Flame", name_stickers: ["Weird", "Angel", "Flame"] },
    StickerSheet { name: "Vampire Champion Fury", name_stickers: ["Vampire", "Champion", "Fury"] },
    StickerSheet { name: "Trained Blessed Mind", name_stickers: ["Trained", "Blessed", "Mind"] },
    StickerSheet { name: "Unglued Pea-Brained Dinosaur", name_stickers: ["Unglued", "Pea-Brained", "Dinosaur"] },
    StickerSheet { name: "Elemental Time Flamingo", name_stickers: ["Elemental", "Time", "Flamingo"] },
    StickerSheet { name: "Unstable Robot Dragon", name_stickers: ["Unstable", "Robot", "Dragon"] },
    StickerSheet { name: "Werewolf Lightning Mage", name_stickers: ["Werewolf", "Lightning", "Mage"] },
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
        // CR 123.6a — a multi-word sticker ("Hot Dog") adds that many words.
        if words.get(at).is_some_and(|w| is_blank(w)) {
            words.remove(at);
        }
        for (k, w) in text.split_whitespace().enumerate() {
            words.insert(at + k, w);
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
        // "Eldrazi" is sticker 0, "Tightrope" is sticker 2.
        let pos = default_name_sticker_position("_____ Goblin");
        assert_eq!(pos, 0);
        assert_eq!(stickered_name("_____ Goblin", &[(0, pos)]), "Eldrazi Goblin");
        assert_eq!(default_name_sticker_position("Bear Cub"), 2);
        assert_eq!(stickered_name("Bear Cub", &[(2, 2)]), "Bear Cub Tightrope");
        assert_eq!(stickered_name("Bear Cub", &[(2, 1)]), "Bear Tightrope Cub");
        assert_eq!(stickered_name("Bear Cub", &[(2, 9)]), "Bear Cub Tightrope");
    }

    /// CR 123.6a — "Hot Dog" is two words: it fills a blank as two, and a
    /// later sticker placed after one word lands between them.
    #[test]
    fn a_two_word_sticker_counts_as_two_words() {
        let hot_dog = 22 * NAME_STICKERS_PER_SHEET as NameStickerId + 1;
        assert_eq!(name_sticker_text(hot_dog), "Hot Dog");
        assert_eq!(stickered_name("_____ Goblin", &[(hot_dog, 0)]), "Hot Dog Goblin");
        assert_eq!(stickered_name("_____ Goblin", &[(hot_dog, 0), (0, 1)]), "Hot Eldrazi Dog Goblin");
    }
}
