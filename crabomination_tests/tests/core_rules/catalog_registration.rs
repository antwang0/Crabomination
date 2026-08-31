//! Guard against orphaned card factories — `pub fn … -> CardDefinition` in
//! `sets/decks/*.rs` that were defined but never wired into `all_factories.rs`,
//! so they never reach the card registry. Several such orphans accumulated
//! silently before this check existed.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn catalog_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../crabomination_catalog/src/sets")
}

#[test]
fn every_deck_card_factory_is_registered() {
    let sets = catalog_src();
    let registry = fs::read_to_string(sets.join("all_factories.rs")).expect("all_factories.rs");
    let registered: HashSet<String> = registry
        .match_indices("super::decks::")
        .map(|(i, _)| {
            registry[i + "super::decks::".len()..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        })
        .collect();

    let mut orphans = Vec::new();
    for entry in fs::read_dir(sets.join("decks")).expect("decks dir") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        // Match `pub fn NAME() -> CardDefinition` (no args = a card factory).
        for (i, _) in src.match_indices("pub fn ") {
            let rest = &src[i + "pub fn ".len()..];
            let name: String =
                rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            let after = &rest[name.len()..];
            if after.trim_start().starts_with("()")
                && after.contains("-> CardDefinition")
                && after[..after.find('{').unwrap_or(after.len())].contains("-> CardDefinition")
                && !registered.contains(&name)
            {
                orphans.push(format!("{}::{name}", path.file_name().unwrap().to_string_lossy()));
            }
        }
    }
    assert!(orphans.is_empty(), "unregistered card factories (add to all_factories.rs): {orphans:?}");
}

/// The same guard, widened from `sets/decks/` to the **whole catalog**.
///
/// Twenty-odd per-set `*_cards_are_registered` tests were each a hand-kept
/// list of that set's factories checked against the registry — the shape that
/// goes stale the moment someone adds a card and forgets the list, which is
/// exactly the failure it was meant to catch. One walk of the tree cannot go
/// stale: it enumerates every `pub fn … () -> CardDefinition` under
/// `sets/` and asserts the registry names it.
///
/// Matches on the factory's leaf name rather than its module path, because a
/// set that is a directory re-exports its factories through `mod.rs` and the
/// registry spells the re-exported path. The only thing that costs is an
/// orphan whose name happens to collide with a registered card in another
/// set; the check is still strictly wider than the per-set lists it replaces.
#[test]
fn every_card_factory_in_the_catalog_is_registered() {
    let sets = catalog_src();
    let registry = fs::read_to_string(sets.join("all_factories.rs")).expect("all_factories.rs");
    let registered: HashSet<&str> = registry
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|t| !t.is_empty())
        .collect();

    let mut files = Vec::new();
    let mut stack = vec![sets.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("sets dir") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files.sort();

    let mut orphans = Vec::new();
    let mut scanned = 0usize;
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        // The registry itself, and the module files that only re-export.
        if name == "all_factories.rs" || name == "mod.rs" {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        for (i, _) in src.match_indices("pub fn ") {
            let rest = &src[i + "pub fn ".len()..];
            let fname: String =
                rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            let after = &rest[fname.len()..];
            let head = &after[..after.find('{').unwrap_or(after.len())];
            if after.trim_start().starts_with("()") && head.contains("-> CardDefinition") {
                scanned += 1;
                if !registered.contains(fname.as_str()) {
                    orphans.push(format!("{name}::{fname}"));
                }
            }
        }
    }
    assert!(scanned > 20_000, "the walk found only {scanned} factories — did the tree move?");
    assert!(
        orphans.is_empty(),
        "{} unregistered card factories (add to all_factories.rs): {:?}",
        orphans.len(),
        &orphans[..orphans.len().min(30)]
    );
}

// ---------------------------------------------------------------------------
// The mechanic-keyword ratchet.
// ---------------------------------------------------------------------------

/// `Keyword::Variant` -> the word the printed card spells.
///
/// Evergreen keywords are deliberately absent: `audit_catalog_stats.py`
/// already compares those, and this table exists for the ~95 *mechanic*
/// keywords nothing compared before.
const KEYWORD_WORD: &[(&str, &str)] = &[
    ("Morph", "morph"),
    ("Megamorph", "megamorph"),
    ("Disguise", "disguise"),
    ("Cycling", "cycling"),
    ("Landcycling", "cycling"),
    ("TypeCycling", "cycling"),
    ("Flashback", "flashback"),
    ("Escape", "escape"),
    ("Bestow", "bestow"),
    ("Madness", "madness"),
    ("Dash", "dash"),
    ("Kicker", "kicker"),
    ("Multikicker", "kicker"),
    ("Evoke", "evoke"),
    ("Unearth", "unearth"),
    ("Embalm", "embalm"),
    ("Eternalize", "eternalize"),
    ("Prowl", "prowl"),
    ("Ninjutsu", "ninjutsu"),
    ("Suspend", "suspend"),
    ("Overload", "overload"),
    ("Retrace", "retrace"),
    ("Miracle", "miracle"),
    ("Soulbond", "soulbond"),
    ("Soulshift", "soulshift"),
    ("Modular", "modular"),
    ("Graft", "graft"),
    ("Sunburst", "sunburst"),
    ("Affinity", "affinity"),
    ("Convoke", "convoke"),
    ("Delve", "delve"),
    ("Improvise", "improvise"),
    ("Emerge", "emerge"),
    ("Prototype", "prototype"),
    ("Disturb", "disturb"),
    ("Blitz", "blitz"),
    ("Casualty", "casualty"),
    ("Cleave", "cleave"),
    ("Channel", "channel"),
    ("Boast", "boast"),
    ("Impending", "impending"),
    ("Warp", "warp"),
    ("Spree", "spree"),
    ("Adapt", "adapt"),
    ("Monstrosity", "monstrosity"),
    ("Outlast", "outlast"),
    ("Renown", "renown"),
    ("Exploit", "exploit"),
    ("Awaken", "awaken"),
    ("Surge", "surge"),
    ("Escalate", "escalate"),
    ("Mutate", "mutate"),
    ("Forecast", "forecast"),
    ("Rebound", "rebound"),
    ("Bloodthirst", "bloodthirst"),
    ("Devour", "devour"),
    ("Annihilator", "annihilator"),
    ("Infect", "infect"),
    ("Wither", "wither"),
    ("Persist", "persist"),
    ("Undying", "undying"),
    ("Evolve", "evolve"),
    ("Extort", "extort"),
    ("Battalion", "battalion"),
    ("Bloodrush", "bloodrush"),
    ("Cipher", "cipher"),
    ("Dredge", "dredge"),
    ("Haunt", "haunt"),
    ("Storm", "storm"),
    ("Replicate", "replicate"),
    ("Conspire", "conspire"),
    ("Rampage", "rampage"),
    ("Bushido", "bushido"),
    ("Flanking", "flanking"),
    ("Shadow", "shadow"),
    ("Provoke", "provoke"),
    ("Amplify", "amplify"),
    ("Absorb", "absorb"),
    ("Vanishing", "vanishing"),
    ("Fading", "fading"),
    ("Echo", "echo"),
    ("Phasing", "phasing"),
    ("Banding", "banding"),
    ("Exalted", "exalted"),
    ("Frenzy", "frenzy"),
    ("Poisonous", "poisonous"),
    ("Toxic", "toxic"),
    ("Skulk", "skulk"),
    ("Crew", "crew"),
    ("Afflict", "afflict"),
    ("Ingest", "ingest"),
    ("Myriad", "myriad"),
    ("Riot", "riot"),
    ("Mentor", "mentor"),
    ("Afterlife", "afterlife"),
    ("Spectacle", "spectacle"),
    ("Adamant", "adamant"),
    ("Foretell", "foretell"),
];

/// Deliberate modellings: the card does not print the keyword, and the
/// engine's keyword is the closest primitive to what it *does* print. Keyed
/// by printed name, checked once so they need not be re-checked.
///
/// Keep this list short and keep every entry's reason with it. An entry here
/// is a claim that the engine is right and the oracle text is only spelling
/// the same rule a longer way; it is not a place to park a defect.
const ALLOWED_MODELLING: &[(&str, &str)] = &[
    // "you may pay {N} any number of times" (the Adversary cycle) is
    // multikicker in everything but name.
    ("Intrepid Adversary", "kicker"),
    ("Bloodthirsty Adversary", "kicker"),
    // "cast from your graveyard by paying {3}{R} and exiling four other
    // cards" is Escape 4 spelled out.
    ("Squee, Dubious Monarch", "escape"),
    // Pit Scorpion predates the keyword: "whenever this deals damage to a
    // player, that player gets a poison counter". Poisonous is combat-only,
    // so this is an approximation and not an equivalence.
    ("Pit Scorpion", "poisonous"),
    // `affinity_filter` is the engine's cost-reduction primitive, and these
    // spell the reduction out instead of printing the keyword ("costs {1}
    // less to cast for each creature on the battlefield").
    ("Carapace Forger", "affinity"),
    ("Temur Battlecrier", "affinity"),
    ("Spectral Denial", "affinity"),
    ("Rowdy Research", "affinity"),
    ("Static Snare", "affinity"),
    ("Gargantuan Leech", "affinity"),
    ("Blasphemous Act", "affinity"),
    ("Vanquish the Horde", "affinity"),
    ("Sea God's Scorn", "affinity"),
    ("Stone Idol Trap", "affinity"),
];

/// Reminder text stripped, the way `audit_keyword_drift.py` strips it.
///
/// Without this every Reach creature reads as printing Flying, because the
/// reminder says "can block creatures with flying". Parens do not nest in
/// oracle text, so a flat scan is the whole job.
fn strip_reminders(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch.to_ascii_lowercase()),
            _ => {}
        }
    }
    out
}

/// **No card carries a mechanic keyword its printed text does not have.**
///
/// This is the Rust ratchet under `scripts/audit_keyword_drift.py`'s INVENTED
/// direction, which is the half of that auditor that is a *proof*: the engine
/// holding a keyword the card does not print is a wrong card in play, not a
/// reading-list entry. Ten such cards shipped before the auditor existed —
/// Hellspark Elemental with Flashback instead of Unearth, Skirk Marauder with
/// landcycling instead of Morph, Lose Focus with Delve instead of Replicate —
/// each one a different card at the table than the one the deck lists. The
/// count is 0 and this test is what keeps it there.
///
/// The MISSING direction is deliberately *not* ratcheted: it cannot see a
/// mechanic spelled as a bespoke triggered ability, so it is a list of cards
/// to read rather than a set of proven gaps. Run the script for that.
///
/// Skipped, matching the script: any card the cache spells over two faces
/// (`card_faces`, or ` // ` in the name, cost or type line), because the body
/// here is the front face alone and the comparison would have to follow
/// `back_face`; and any card the cache does not hold.
#[test]
fn no_card_carries_a_mechanic_keyword_it_does_not_print() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    // ⚠ A name the cache could not resolve is stored as the string
    // `"__not_found__"`, not omitted. Keeping those turns every field lookup
    // into `""` and reports the card as carrying every keyword it has; seven
    // of the catalog's synthesised names read that way before this filter.
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let kw_word: std::collections::HashMap<&str, &str> = KEYWORD_WORD.iter().copied().collect();
    let allowed: HashSet<(&str, &str)> = ALLOWED_MODELLING.iter().copied().collect();

    let mut checked = 0usize;
    let mut invented: Vec<String> = Vec::new();

    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        let field = |k: &str| entry.get(k).and_then(|v| v.as_str()).unwrap_or("");
        if entry.get("card_faces").is_some()
            || field("name").contains(" // ")
            || field("mana_cost").contains(" // ")
            || field("type_line").contains(" // ")
        {
            continue;
        }
        checked += 1;
        let printed =
            strip_reminders(&format!("{} {}", field("oracle_text"), field("type_line")));

        // What the engine says this card has. `keywords` first, then the
        // mechanics that live in a field of their own instead — those are
        // invisible to any check that reads `keywords` alone.
        //
        // Only fields that *are* the printed keyword belong here.
        // `affinity_graveyard_filter`, `kicker_action_cost`,
        // `flashback_additional_cost`, `saga_chapters`, `room`, `case`,
        // `station`, `gift` and `soulbond_bonus` are implementation vehicles
        // carried by cards that spell the mechanic some other way, so they
        // answer the MISSING question and must not answer this one.
        let mut have: Vec<&str> = def
            .keywords
            .iter()
            .filter_map(|kw| {
                let rendered = format!("{kw:?}");
                let variant: String =
                    rendered.chars().take_while(|c| c.is_alphanumeric()).collect();
                kw_word.get(variant.as_str()).copied()
            })
            .collect();
        for (present, word) in [
            (def.foretell_cost.is_some(), "foretell"),
            (def.plot_cost.is_some(), "plot"),
            (def.buyback_additional_cost.is_some(), "buyback"),
            (def.entwine_additional_cost.is_some(), "entwine"),
            (def.splice_extra_cost.is_some(), "splice"),
            (def.affinity_filter.is_some(), "affinity"),
            (def.mutate.is_some(), "mutate"),
            (def.bestow.is_some(), "bestow"),
            (def.adventure.is_some(), "adventure"),
            (def.companion.is_some(), "companion"),
            (def.omen.is_some(), "omen"),
            (def.prototype.is_some(), "prototype"),
        ] {
            if present {
                have.push(word);
            }
        }
        have.sort_unstable();
        have.dedup();

        for word in have {
            if !printed.contains(word) && !allowed.contains(&(def.name, word)) {
                invented.push(format!("{} carries {word}", def.name));
            }
        }
    }

    // Without this the ratchet can go vacuous: a cache that failed to load,
    // a renamed field, or a `{:?}` that stopped starting with the variant
    // name would each turn the loop into a no-op that still passes.
    assert!(
        checked > 15_000,
        "only {checked} cards could be compared against the cache — did the cache or the \
         name field move? The ratchet is vacuous below that."
    );
    invented.sort();
    invented.dedup();
    assert!(
        invented.is_empty(),
        "{} card(s) carry a mechanic keyword the printed card does not have. Each one is a \
         different card in play than the deck lists. Fix the card, or — if the engine's \
         keyword is genuinely the closest primitive to what the card prints — add it to \
         ALLOWED_MODELLING with its reason: {:?}",
        invented.len(),
        &invented[..invented.len().min(30)]
    );
}
