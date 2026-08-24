//! Frozen embedding-index assignment for [`super::encode::Vocab`].
//!
//! **Why this file exists.** `Vocab::sos_sealed()` used to derive its
//! indices from `draft::sos_draft_pool()` directly, in sorted-name order —
//! so adding one card to the SOS set shifted the index of every card that
//! sorts after it, and *every net trained before that addition became
//! wrong*. It failed loudly (the embedding row count no longer matched) for
//! the play net and silently for nothing, which is the only mercy in it:
//! all seven committed `deck-latest.safetensors` are 153 rows against a
//! live vocabulary of 164 and none of them can be loaded.
//!
//! **The contract now.** A name in [`VOCAB_SNAPSHOT`] owns index
//! `position + 1`, forever, whether or not it is still in the pool. A pool
//! name that is not in the snapshot is appended after it, in sorted order.
//! So a card addition — or a removal — grows the table at the end and never
//! moves an index a trained net depends on, and a net trained against a
//! smaller vocabulary can be zero-padded up to the current one instead of
//! being retired.
//!
//! **Do not reorder, remove, or edit an entry. Appending is the only safe
//! edit, and it is required, not tidying.** A pool card outside the
//! snapshot assigns itself an index so the build keeps working, but that
//! index is provisional: it comes from sorted order over the *unsnapshotted*
//! names, so a second card addition reshuffles it and reintroduces exactly
//! the defect this file closes, one generation later. Two additions is all
//! it takes. `encode`'s `vocab_covers_the_sos_pool` fails while any pool
//! name is unsnapshotted and prints the names to append — at the end of the
//! array, in the order `Vocab::sos_sealed()` gives them.
//!
//! Seeded 2026-08-24 from the then-current `Vocab::sos_sealed()`, which was
//! the sorted pool plus the five basics — so the seed *is* the mapping that
//! existed before this file, and `nets/champion.safetensors` (164 rows) is
//! unaffected by its introduction.

/// The names whose embedding indices are frozen. Index 0 is the reserved
/// unknown slot, so this list starts at index 1.
pub const VOCAB_SNAPSHOT: [&str; 163] = [
    "Additive Evolution",
    "Adrix and Nev, Twincasters",
    "Ambitious Augmenter",
    "Applied Geometry",
    "Arcane Omens",
    "Archaeomancer",
    "Archaic's Agony",
    "Archmage Emeritus",
    "Ark of Hunger",
    "Arnyn, Deathbloom Botanist",
    "Artistic Process",
    "Ascendant Dustspeaker",
    "Aziza, Mage Tower Captain",
    "Banishing Betrayal",
    "Berta, Wise Extrapolator",
    "Blech, Loafing Pest",
    "Bogwater Lumaret",
    "Burrog Banemaker",
    "Burrog Barrage",
    "Campus Composer",
    "Cauldron of Essence",
    "Charging Strifeknight",
    "Chase Inspiration",
    "Chelonian Tackle",
    "Choreographed Sparks",
    "Codie, Vociferous Codex",
    "Colorstorm Stallion",
    "Daydream",
    "Decorum Dissertation",
    "Deluge Virtuoso",
    "Divergent Equation",
    "Dualcaster Mage",
    "Duel Tactics",
    "Eager Glyphmage",
    "Echocasting Symposium",
    "Efflorescence",
    "Elemental Mascot",
    "Emeritus of Ideation",
    "Ennis, Debate Moderator",
    "Environmental Scientist",
    "Essenceknit Scholar",
    "Fields of Strife",
    "Fix What's Broken",
    "Flow State",
    "Follow the Lumarets",
    "Foolish Fate",
    "Forest",
    "Forum Necroscribe",
    "Forum of Amity",
    "Fractal Anomaly",
    "Fractal Mascot",
    "Fractal Tender",
    "Fractalize",
    "Garrison Excavator",
    "Geometer's Arthropod",
    "Germination Practicum",
    "Glorious Decay",
    "Graduation Day",
    "Grapple with Death",
    "Grave Researcher",
    "Grim Haruspex",
    "Growth Curve",
    "Hardened Academic",
    "Homesickness",
    "Imperious Inkmage",
    "Improvisation Capstone",
    "Informed Inkwright",
    "Inkling Mascot",
    "Inkshape Demonstrator",
    "Interjection",
    "Island",
    "Lecturing Scornmage",
    "Library of Alexandria",
    "Library of Leng",
    "Living History",
    "Lluwen, Exchange Student",
    "Lorehold Charm",
    "Lorehold, the Historian",
    "Lumaret's Favor",
    "Magmablood Archaic",
    "Magus of the Library",
    "Mana Sculpt",
    "Masterful Flourish",
    "Melancholic Poet",
    "Mica, Reader of Ruins",
    "Mindful Biomancer",
    "Molten Note",
    "Moseo, Vein's New Dean",
    "Mountain",
    "Murmuring Mystic",
    "Muse's Encouragement",
    "Nita, Forum Conciliator",
    "Noxious Newt",
    "Old-Growth Educator",
    "Oracle's Restoration",
    "Paradox Gardens",
    "Paradox Surveyor",
    "Pest Mascot",
    "Plains",
    "Planar Engineering",
    "Practiced Offense",
    "Practiced Scrollsmith",
    "Primary Research",
    "Prismari Charm",
    "Prismari, the Inspiration",
    "Procrastinate",
    "Proctor's Gaze",
    "Professor Dellian Fel",
    "Quandrix, the Proof",
    "Rabid Attack",
    "Ral Zarek, Guest Lecturer",
    "Rapier Wit",
    "Rapturous Moment",
    "Rearing Embermare",
    "Rehearsed Debater",
    "Resonating Lute",
    "Restoration Seminar",
    "Rubble Rouser",
    "Shattered Acolyte",
    "Shopkeeper's Bane",
    "Silverquill Charm",
    "Silverquill, the Disputant",
    "Slumbering Trudge",
    "Snarl Song",
    "Sneering Shadewriter",
    "Snooping Page",
    "Soaring Stoneglider",
    "Social Snub",
    "Spectacle Summit",
    "Spectacular Skywhale",
    "Spirit Mascot",
    "Splatter Technique",
    "Stand Up for Yourself",
    "Startled Relic Sloth",
    "Steal the Show",
    "Stirring Honormancer",
    "Stirring Hopesinger",
    "Stone Docent",
    "Strife Scholar",
    "Studious First-Year",
    "Suspend Aggression",
    "Swamp",
    "Sylvan Library",
    "Tablet of Discovery",
    "Textbook Tabulator",
    "Thornfist Striker",
    "Titan's Grave",
    "Tome Blast",
    "Top of the Class",
    "Topiary Lecturer",
    "Tragedy Feaster",
    "Traumatic Critique",
    "Unsubtle Mockery",
    "Vicious Rivalry",
    "Wander Off",
    "Wild Hypothesis",
    "Wildgrowth Archaic",
    "Wilt in the Heat",
    "Witherbloom, the Balancer",
    "Withering Curse",
    "Zaffai and the Tempests",
    "Zealous Lorecaster",
    "Zimone's Experiment",
];

/// The vocabulary size at which indices were frozen — the snapshot plus the
/// reserved unknown slot at 0.
///
/// **A net trained at any *smaller* size predates the freeze and cannot be
/// padded**, only retrained. Its indices came from the sorted pool of its
/// own day, and eleven cards were inserted mid-order before this snapshot
/// was taken, so nothing can say which card each of its rows meant. Padding
/// one would load cleanly and mean the wrong cards, which is strictly worse
/// than the loud failure it replaced. See [`vocab_fit`].
pub const FROZEN_VOCAB_SIZE: usize = VOCAB_SNAPSHOT.len() + 1;

/// Whether a net trained with `have` embedding rows can be fitted to a
/// current vocabulary of `want`.
///
/// * `have == want` — nothing to do.
/// * `FROZEN_VOCAB_SIZE <= have < want` — pad. Frozen indices mean every
///   row the net has still names the card it was trained on, and the cards
///   it never saw embed as zeros, which is what index 0 does for an off-set
///   card anyway.
/// * `have < FROZEN_VOCAB_SIZE` — refuse; see [`FROZEN_VOCAB_SIZE`].
/// * `have > want` — refuse. The encoder has no index to give the extra
///   rows, so the net saw cards this build does not have.
pub fn vocab_fit(have: usize, want: usize) -> Result<(), String> {
    if have > want {
        return Err(format!(
            "net vocab {have} is larger than the encoder's {want}: it was trained against a \
             card set this build does not have, so its indices mean different cards"
        ));
    }
    if have < FROZEN_VOCAB_SIZE && have != want {
        return Err(format!(
            "net vocab {have} predates the frozen index assignment ({FROZEN_VOCAB_SIZE}). Its \
             rows were indexed by the sorted card pool of its day, so there is no way to line \
             them up with the current one — this checkpoint needs retraining, it is not \
             corrupt. Nets trained at {FROZEN_VOCAB_SIZE} or above survive a card addition."
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pre_freeze_net_is_refused_and_a_post_freeze_one_is_padded() {
        // The seven committed deck nets are 153 against a frozen floor of
        // 164 — the defect this module exists to close, and padding them
        // would be a silent wrong answer rather than a loud failure.
        assert!(vocab_fit(153, FROZEN_VOCAB_SIZE).is_err());
        assert!(vocab_fit(FROZEN_VOCAB_SIZE, FROZEN_VOCAB_SIZE + 8).is_ok());
        assert!(vocab_fit(FROZEN_VOCAB_SIZE + 8, FROZEN_VOCAB_SIZE).is_err());
        // An exact match always passes, whatever the size — that is how a
        // net older than the freeze still works on the build it was trained
        // against.
        assert!(vocab_fit(153, 153).is_ok());
        assert!(vocab_fit(FROZEN_VOCAB_SIZE, FROZEN_VOCAB_SIZE).is_ok());
    }
}
