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

/// Quoted spans removed. A *granted* ability's own text ("Permanents you
/// control have \"…\"") is that ability's rule and not this card's, and the
/// clause ratchets below all read the card's own printing — 80 of the 92
/// first rows on the printed-`{T}` ratchet were quoted grants.
fn strip_quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;
    for ch in text.chars() {
        match ch {
            '"' | '\u{201c}' | '\u{201d}' => inside = !inside,
            _ if !inside => out.push(ch),
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
/// The MISSING direction is not ratcheted **in general**: it cannot see a
/// mechanic spelled as a bespoke triggered ability, so it is a list of cards
/// to read rather than a set of proven gaps. Run the script for that. It *is*
/// ratcheted on the subset where no bespoke spelling exists — see
/// `every_card_that_prints_a_keyword_line_carries_those_keywords` at the end
/// of this file, which found 25 cards on its first run.
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

/// **Every card's printed cost and body are the ones Scryfall prints.**
///
/// The Rust ratchet under `audit_catalog_stats.py`'s `cost` and `P/T`
/// columns, which read **0** across every set. It exists so the ~100
/// hand-written `*_stat_lines` tests do not have to: those asserted what a
/// test author typed for a hand-picked list of cards, where this asserts what
/// Wizards printed, for every card the cache can adjudicate.
///
/// Skipped, and each skip is a shape the cache cannot answer rather than a
/// card given a pass: a name the cache does not hold; a card it spells over
/// two faces (`card_faces`, or ` // ` in the name, cost or type line), since
/// the body here is the front face alone; a cost the cache leaves empty (a
/// land, where the engine writes `{0}`); and a `*` power or toughness.
#[test]
fn every_card_has_the_cost_and_body_its_printing_has() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    /// `{3}{B}{B}` -> a sorted `["3", "B", "B"]`, so the two sides may spell
    /// the same cost in different orders.
    fn pips(s: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut cur: Option<String> = None;
        for ch in s.chars() {
            match ch {
                '{' => cur = Some(String::new()),
                '}' => {
                    if let Some(p) = cur.take() {
                        out.push(p);
                    }
                }
                _ => {
                    if let Some(p) = cur.as_mut() {
                        p.push(ch);
                    }
                }
            }
        }
        out.sort();
        out
    }

    let mut costs = 0usize;
    let mut bodies = 0usize;
    let mut wrong: Vec<String> = Vec::new();

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
        // ⚠ A Vanguard avatar shares its name with an ordinary card and the
        // cache holds the *card* — "Maraxus of Keld" is a {4}{R}{R} Legends
        // creature to Scryfall and a costless command-zone avatar here. The
        // other four command-zone types are skipped for the same reason.
        if def.card_types.iter().any(|t| {
            matches!(
                t,
                crabomination::card::CardType::Vanguard
                    | crabomination::card::CardType::Scheme
                    | crabomination::card::CardType::Plane
                    | crabomination::card::CardType::Phenomenon
                    | crabomination::card::CardType::Conspiracy
            )
        }) {
            continue;
        }
        let printed = field("mana_cost");
        if !printed.is_empty() {
            costs += 1;
            let ours = def.cost.summary();
            if pips(&ours) != pips(printed) {
                wrong.push(format!("{}: cost {ours} vs printed {printed}", def.name));
            }
        }
        // ⚠ A Spacecraft's printed P/T is the one its highest station band
        // gives it (CR 311.9), and the engine spells that as a 0/0 body plus
        // `station` bands. Twenty of them read as body bugs before this gate
        // and every one is correct.
        //
        // `power`/`toughness` are strings in the cache and may be `*`, `1+*`
        // or absent; only a plain integer is comparable.
        if def.station.is_empty()
            && let (Some(p), Some(t)) = (
            field("power").parse::<i32>().ok(),
            field("toughness").parse::<i32>().ok(),
        )
        {
            bodies += 1;
            if (def.power, def.toughness) != (p, t) {
                wrong.push(format!(
                    "{}: body {}/{} vs printed {p}/{t}",
                    def.name, def.power, def.toughness
                ));
            }
        }
    }

    // Non-vacuity, as above: a moved cache or a renamed field would otherwise
    // turn this into a loop that compares nothing and passes.
    assert!(
        costs > 12_000 && bodies > 6_000,
        "only {costs} costs and {bodies} bodies were comparable — did the cache move?",
    );
    wrong.sort();
    wrong.dedup();
    assert!(
        wrong.is_empty(),
        "{} card(s) do not have their printed cost or body: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(60)]
    );
}

/// **Every card's type line is the one its printing has** — card types and
/// supertypes, compared as sets against Scryfall's `type_line`.
///
/// The sibling of `every_card_has_the_cost_and_body_its_printing_has`, and it
/// exists for the same reason: `audit_catalog_stats.py` reads 0 here and can
/// only see a factory that spells `CardDefinition { name: … }` at eight
/// spaces, so every helper-built card was unaudited.
///
/// Subtypes are deliberately **not** compared. Scryfall's right-hand side
/// mixes creature, land, artifact, enchantment, spell and planeswalker types
/// into one list, several are two words, and the engine keeps them in six
/// separate enums — that comparison needs its own pass, and doing it badly
/// here would bury the type-line signal in false rows.
#[test]
fn every_card_has_the_type_line_its_printing_has() {
    use crabomination::card::CardType;
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();

    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        let field = |k: &str| entry.get(k).and_then(|v| v.as_str()).unwrap_or("");
        let line = field("type_line");
        if entry.get("card_faces").is_some()
            || field("name").contains(" // ")
            || line.contains(" // ")
            || line.is_empty()
        {
            continue;
        }
        // A Vanguard avatar wears an ordinary card's name; see the sibling.
        if def.card_types.iter().any(|t| {
            matches!(
                t,
                CardType::Vanguard
                    | CardType::Scheme
                    | CardType::Plane
                    | CardType::Phenomenon
                    | CardType::Conspiracy
            )
        }) {
            continue;
        }
        checked += 1;

        // "Legendary Artifact Creature — Phyrexian Imp": everything left of
        // the em dash is a supertype or a card type, and both spell the
        // engine's variant name exactly.
        let head = line.split(" — ").next().unwrap_or(line);
        let mut printed_types: Vec<String> = Vec::new();
        let mut printed_supers: Vec<String> = Vec::new();
        for word in head.split_whitespace() {
            match word {
                "Basic" | "Legendary" | "Snow" | "World" | "Ongoing" => {
                    printed_supers.push(word.to_string())
                }
                // `Tribal` was renamed `Kindred`; old printings still say it.
                "Tribal" => printed_types.push("Kindred".to_string()),
                _ => printed_types.push(word.to_string()),
            }
        }
        // A type the engine has no variant for cannot be a finding — it is a
        // gap in `CardType`, which `audit_variant_coverage.py` owns.
        let known = |t: &String| CARD_TYPE_NAMES.contains(&t.as_str());
        if !printed_types.iter().all(known) {
            continue;
        }

        let mut ours: Vec<String> = def.card_types.iter().map(|t| format!("{t:?}")).collect();
        ours.sort();
        printed_types.sort();
        if ours != printed_types {
            wrong.push(format!("{}: types {ours:?} vs printed {printed_types:?}", def.name));
        }
        let mut our_supers: Vec<String> =
            def.supertypes.iter().map(|t| format!("{t:?}")).collect();
        our_supers.sort();
        printed_supers.sort();
        if our_supers != printed_supers {
            wrong.push(format!(
                "{}: supertypes {our_supers:?} vs printed {printed_supers:?}",
                def.name
            ));
        }
    }

    assert!(
        checked > 12_000,
        "only {checked} type lines were comparable — did the cache move?",
    );
    wrong.sort();
    wrong.dedup();
    assert!(
        wrong.is_empty(),
        "{} card(s) do not have their printed type line: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(60)]
    );
}

/// The `CardType` variant names, spelled once so the type-line test can tell
/// "the engine models this type differently" from "the engine has no such
/// type at all" — the second is `audit_variant_coverage.py`'s business.
const CARD_TYPE_NAMES: [&str; 14] = [
    "Land",
    "Creature",
    "Artifact",
    "Enchantment",
    "Planeswalker",
    "Battle",
    "Instant",
    "Sorcery",
    "Kindred",
    "Scheme",
    "Vanguard",
    "Conspiracy",
    "Plane",
    "Phenomenon",
];

/// **Every card's subtypes are the ones its printing has.**
///
/// The third of the printed-characteristics ratchets. Scryfall spells all six
/// kinds of subtype in one list after the em dash; the engine keeps them in
/// six enums, so both sides are flattened to a **set of normalized words**
/// (`{:?}` with the non-alphanumerics stripped, lower-cased) — that is what
/// makes `CreatureType::TimeLord` and the printed "Time Lord" the same thing,
/// and it is why a two-word subtype needs no special case.
///
/// Skipped, each with its reason:
/// * a subtype the engine has **no variant for** — that is a gap in the enum
///   and `audit_variant_coverage.py`'s business, not a wrong card. The set is
///   compared only when every printed word is one the engine can spell.
/// * a **Changeling**, which is every creature type (CR 702.73) and which the
///   engine approximates with a handful of them.
#[test]
fn every_card_has_the_subtypes_its_printing_has() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    fn norm(s: &str) -> String {
        s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
    }

    // Every subtype the engine can spell, so a printed word it cannot is a
    // missing variant rather than a wrong card.
    let spellable: HashSet<String> = {
        let mut v: HashSet<String> = HashSet::new();
        for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
            let d = factory();
            for s in subtype_words(&d) {
                v.insert(s);
            }
        }
        v
    };

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();

    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        let field = |k: &str| entry.get(k).and_then(|v| v.as_str()).unwrap_or("");
        let line = field("type_line");
        if entry.get("card_faces").is_some()
            || field("name").contains(" // ")
            || line.contains(" // ")
            || line.is_empty()
        {
            continue;
        }
        if def.keywords.iter().any(|k| format!("{k:?}").starts_with("Changeling")) {
            continue;
        }
        // ⚠ **A FILED DEFECT CLASS, EXCUSED BY SHAPE RATHER THAN BY NAME.**
        // ~48 nonbasic lands carry basic `land_types` their printed type line
        // does not have (Blackcleave Cliffs is `Land`, not `Land — Mountain
        // Swamp`). The types are *redundant*: `dual_land_with` gives the land
        // its colors through `tap_add` activated abilities, and `tri_land`
        // two definitions below is explicitly "untyped (matching the print)".
        // So the types buy nothing and cost correctness — those lands are
        // fetchable by Farseek, hit by Blood Moon, grant landwalk and count
        // for domain, none of which the printed cards do.
        //
        // Taking it moves the `cube` and `sealed` pools, so it is priced and
        // taken on its own (CARD_BACKLOG), not smuggled in under a ratchet.
        // The rule below excuses exactly that shape and nothing else.
        if def.card_types.contains(&crabomination::card::CardType::Land)
            && !line.contains(" — ")
            && def.activated_abilities.iter().any(|a| a.tap_cost)
            && subtype_words(&def)
                .iter()
                .all(|w| ["plains", "island", "swamp", "mountain", "forest"].contains(&w.as_str()))
        {
            continue;
        }
        // ⚠ Four names the cache cannot adjudicate, each for its own reason.
        // A name here is a claim about the *cache*, never about the card.
        if matches!(
            def.name,
            // A theme-deck placeholder: the cache holds `type_line: "Card"`
            // and `oracle_text: "(Theme color: {B})"`.
            "Bounty Hunter"
                // Strixhaven reprinted both as `Sorcery — Lesson`; the cache
                // holds the older same-named card, which is a plain Sorcery.
                | "Brilliant Plan"
                | "Reduce to Ashes"
                // A Cave in its Lost Caverns printing; the cache's entry has
                // no subtype, so the two are not the same card and this
                // cannot decide which the body should be.
                | "Hidden Grotto"
                // Modelled as a creature everywhere but the battlefield
                // (CR 306.1), which is what the printed Insect type buys.
                | "Grist, the Hunger Tide"
                // The types are the animation: "during your turn, Gideon
                // Blackblade is a 4/4 Human Soldier that's still a
                // planeswalker", wired as `StaticEffect::SelfIsCreatureIf`.
                | "Gideon Blackblade"
        ) {
            continue;
        }
        let Some(tail) = line.split(" — ").nth(1) else {
            // No subtypes printed and none in the body is the common case;
            // a body that invents one is still a finding.
            let ours = subtype_words(&def);
            if !ours.is_empty() {
                checked += 1;
                let mut o: Vec<String> = ours.into_iter().collect();
                o.sort();
                wrong.push(format!("INVENTED {}: {o:?} (nothing printed)", def.name));
            }
            continue;
        };
        let printed: HashSet<String> = tail.split_whitespace().map(norm).collect();
        if !printed.iter().all(|w| spellable.contains(w)) {
            continue;
        }
        checked += 1;
        let ours = subtype_words(&def);
        let mut extra: Vec<&String> = ours.difference(&printed).collect();
        let mut missing: Vec<&String> = printed.difference(&ours).collect();
        extra.sort();
        missing.sort();
        if !extra.is_empty() {
            wrong.push(format!("INVENTED {}: {extra:?}", def.name));
        }
        if !missing.is_empty() {
            wrong.push(format!("missing  {}: {missing:?}", def.name));
        }
    }

    assert!(
        checked > 6_000,
        "only {checked} subtype lines were comparable — did the cache move?",
    );
    wrong.sort();
    wrong.dedup();
    // **Only the INVENTED direction is asserted, and the asymmetry is the
    // point** — the same one `audit_keyword_drift.py` draws. A subtype the
    // engine has and the card does not is a wrong card at the table *now*: it
    // is fetched, tutored, anthemed and Blood-Mooned by things that should
    // not touch it. A subtype the card has and the engine lacks is a *gap*,
    // equally real but one that only ever fails to trigger something.
    //
    // The missing pile is 183 cards at the time of writing (Centaur Courser
    // is a Centaur where the card is a Centaur **Warrior**), and it is filed
    // in CARD_BACKLOG rather than fixed here: closing it moves the tribal
    // shape of the `cube` and `sealed` pools, so it is priced and taken on
    // its own.
    let invented: Vec<&String> = wrong.iter().filter(|w| w.starts_with("INVENTED")).collect();
    let missing = wrong.len() - invented.len();
    assert!(
        invented.is_empty(),
        "{} card(s) carry a subtype their printing does not ({missing} more are missing one, \
         which this does not assert — see CARD_BACKLOG):\n{}",
        invented.len(),
        invented.iter().take(60).map(|s| s.as_str()).collect::<Vec<_>>().join("\n"),
    );
}

/// Every subtype on a definition, from all six enums, as normalized words.
fn subtype_words(def: &crabomination::card::CardDefinition) -> HashSet<String> {
    fn norm(s: String) -> String {
        s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
    }
    let s = &def.subtypes;
    let mut out: HashSet<String> = HashSet::new();
    out.extend(s.creature_types.iter().map(|t| norm(format!("{t:?}"))));
    out.extend(s.land_types.iter().map(|t| norm(format!("{t:?}"))));
    out.extend(s.artifact_subtypes.iter().map(|t| norm(format!("{t:?}"))));
    out.extend(s.enchantment_subtypes.iter().map(|t| norm(format!("{t:?}"))));
    out.extend(s.spell_subtypes.iter().map(|t| norm(format!("{t:?}"))));
    out.extend(s.planeswalker_subtypes.iter().map(|t| norm(format!("{t:?}"))));
    out.extend(s.battle_subtypes.iter().map(|t| norm(format!("{t:?}"))));
    out
}

/// Cards whose free, unconditional, unlimited activated ability is genuinely
/// what the printed card says — a printed `{0}:` ability. Each entry carries
/// its reason; adding one is a claim about the *oracle*, not about the body.
const FREE_ACTIVATION_ALLOWED: &[(&str, &str)] = &[
    ("Blessing of Leeches", "printed `{0}: Regenerate enchanted creature.`"),
    ("Blinking Spirit", "printed `{0}: Return this creature to its owner's hand.`"),
    ("Chimeric Idol", "printed `{0}: Tap all lands you control. …`"),
    ("Dark Maze", "printed `{0}: This creature can attack this turn as though …`"),
    ("Flowstone Hellion", "printed `{0}: This creature gets +1/-1 until end of turn.`"),
    ("Frenetic Efreet", "printed `{0}: Flip a coin. …`"),
    ("Hopping Automaton", "printed `{0}: This creature gets -1/-1 and gains flying …`"),
    ("Lancers en-Kor", "printed `{0}:` damage-shift (the en-Kor cycle)"),
    ("Mist Dragon", "printed `{0}: gains flying` / `{0}: loses flying`"),
    ("Nomads en-Kor", "printed `{0}:` damage-shift (the en-Kor cycle)"),
    ("Opal Acrolith", "printed `{0}: This permanent becomes an enchantment.`"),
    ("Reconnaissance", "printed `{0}: Remove target attacking creature … from combat`"),
    ("Sentinel", "printed `{0}: Change this creature's base toughness …`"),
    ("Shaman en-Kor", "printed `{0}:` damage-shift (the en-Kor cycle)"),
    ("Soltari Guerrillas", "printed `{0}: The next time this creature would deal …`"),
    ("Spirit Mirror", "printed `{0}: Destroy target Reflection.`"),
    ("Spirit en-Kor", "printed `{0}:` damage-shift (the en-Kor cycle)"),
    ("Urza's Avenger", "printed `{0}: This creature gets -1/-1 and gains …`"),
    ("Warrior en-Kor", "printed `{0}:` damage-shift (the en-Kor cycle)"),
    // ⚠ NOT a printed `{0}`: the card's first ability is the *static*
    // "If this creature would be destroyed, regenerate it", modelled as a free
    // activated regenerate because the engine has no such replacement. Its
    // printed activated ability (`{1}:`, opponents only) is the second one and
    // is correctly costed. Primitive job: a self-regenerating static.
    ("Clergy of the Holy Nimbus", "static regeneration modelled as a free activation"),
];

/// **No shipped card carries an activated ability that is free, unconditional
/// and unlimited.**
///
/// Such an ability is an unbounded action loop the moment a bot wants it: the
/// engine will accept it forever, so a self-play game runs to its 50,000-action
/// cap instead of ending. Found the hard way — a `--decks cube --seed 2` game
/// reached the cap at turn 15 with **49,616 copies of Gravecrawler's
/// graveyard activation on the stack**, paid for by Pentad Prism, whose charge
/// counter was spelled as the ability's *effect* rather than its cost
/// (`CRAB_CAP_DIAG=1` names both). The Prism was free, so the mana was
/// unbounded, so the Gravecrawler activation was too.
///
/// The check is a comparison against `ActivatedAbility::default()` with only
/// the effect swapped in, so **it covers every cost field the struct has and
/// every one a later pass adds** — a new cost cannot slip past it the way
/// `remove_counter_cost` did.
#[test]
fn no_activated_ability_is_free_unconditional_and_unlimited() {
    use crabomination::card::ActivatedAbility;
    use crabomination::effect::StaticEffect;
    let allowed: HashSet<&str> = FREE_ACTIVATION_ALLOWED.iter().map(|(n, _)| *n).collect();
    let mut bad: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        checked += 1;
        // Printed abilities, plus the ones a static hands to other
        // permanents: `Cryptolith Rite`-shaped grants reach the same
        // `activate_ability` path as a printed one and are just as unbounded.
        let granted = def.static_abilities.iter().filter_map(|sa| match &sa.effect {
            StaticEffect::GrantActivatedAbility { ability, .. } => Some(ability),
            StaticEffect::GrantActivatedAbilityFromGraveyard { ability, .. } => {
                Some(&**ability)
            }
            _ => None,
        });
        for (i, ab) in def.activated_abilities.iter().chain(granted).enumerate() {
            let bare = ActivatedAbility { effect: ab.effect.clone(), ..Default::default() };
            if *ab == bare && !allowed.contains(def.name) {
                bad.push(format!("{} [{i}]", def.name));
            }
        }
    }
    assert!(checked > 15_000, "only {checked} factories walked — the ratchet is vacuous");
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} activated ability/abilities cost nothing, take no condition and have no \
         per-turn limit, so a bot can activate them without bound. Give the ability its \
         printed cost, or add the card to FREE_ACTIVATION_ALLOWED with its reason: {:?}",
        bad.len(),
        &bad[..bad.len().min(40)]
    );
}

/// **A card whose oracle says "in addition to its other types" must not be
/// built on a primitive that replaces them.**
///
/// Seven cards were, and they read alike from the outside: Ensoul Artifact,
/// Zoetic Glyph and Unable to Scream on `equipped_bonus.set_card_types` /
/// `set_creature_types`; Xenograft on `CreaturesYouControlAreChosenType`,
/// which is *Conspiracy's* replacing static; Prismatic Omen, Leyline of the
/// Guildpact and Nylea's Presence on a `GrantAllBasicLandTypes` that emitted
/// a `SetLandTypes`. An ensouled Darksteel Citadel stopped being a Land and
/// stopped tapping for mana; a Xenograft naming Sliver stopped your Goblins
/// being Goblins.
///
/// The signal is the oracle's own words, and the replacing primitives are
/// named here rather than inferred — a new one has to be added to the list,
/// which is the point: the list is the inventory of "this drops types".
#[test]
fn no_card_replaces_the_types_its_oracle_adds() {
    /// Every primitive whose effect is "the type list becomes exactly this".
    /// `{:?}` of the built definition is the medium, the same way the
    /// `debug_flag` word reads a definition.
    const REPLACERS: [&str; 4] = [
        "set_card_types: Some(",
        "set_creature_types: Some(",
        "set_land_types: Some(",
        // Conspiracy's static, and Conspiracy is the only card whose oracle
        // wants it ("Creatures you control are the chosen type", no "in
        // addition").
        "CreaturesYouControlAreChosenType",
    ];
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        let oracle = entry.get("oracle_text").and_then(|v| v.as_str()).unwrap_or("");
        if !oracle.contains("in addition to its other types")
            && !oracle.contains("in addition to their other types")
        {
            continue;
        }
        checked += 1;
        let dbg = format!("{def:?}");
        for r in REPLACERS {
            if dbg.contains(r) {
                wrong.push(format!("{}: `{r}` under an \"in addition\" oracle", def.name));
            }
        }
    }
    assert!(
        checked > 20,
        "only {checked} cards printed an \"in addition\" clause — did the cache move?",
    );
    wrong.sort();
    wrong.dedup();
    assert!(wrong.is_empty(), "{} card(s): {:?}", wrong.len(), wrong);
}

/// **CR 305.7 — setting a land's type line takes its abilities with it.**
///
/// The structural half of the rule above, and it needs no oracle text: an
/// effect that *sets* a land's subtypes removes the abilities the land had
/// from its rules text and its old types. Every route through
/// `Modification::SetLandTypes` pairs a `RemoveAllAbilities` — except
/// `equipped_bonus.set_land_types`, where the pairing is the card author's to
/// remember, and **two of its three users forgot**: Sea's Claim and Lingering
/// Mirage left a tri-land tapping for its three colours plus blue.
#[test]
fn a_land_type_setting_aura_takes_the_lands_abilities() {
    let mut wrong: Vec<&str> = Vec::new();
    let mut checked = 0usize;
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(bonus) = def.equipped_bonus.as_ref() else {
            continue;
        };
        if bonus.set_land_types.is_none() {
            continue;
        }
        checked += 1;
        if !bonus.remove_abilities {
            wrong.push(def.name);
        }
    }
    assert!(checked >= 3, "only {checked} auras set a land type — did the field move?");
    assert!(
        wrong.is_empty(),
        "{} aura(s) set a land's type line without CR 305.7's ability loss: {wrong:?}",
        wrong.len(),
    );
}


/// **A card that prints "loses all abilities" has to strip them.**
///
/// The rider is easy to drop because the card still *does* something without
/// it — Merfolk Trickster still tapped a blocker, and its doc comment said
/// the rider was dropped, which is how it stayed dropped. It is half the
/// card: it takes a blocker's flying, a ward, a death trigger.
///
/// The needles are the four primitives that strip, spelled out rather than
/// inferred, so a fifth has to be added here to count.
#[test]
fn a_card_that_prints_losing_all_abilities_strips_them() {
    /// Everything that reaches `Modification::RemoveAllAbilities`.
    const STRIPPERS: [&str; 7] = [
        "remove_abilities: true",
        "RemoveAllAbilities",
        "LoseAllAbilities",
        "CreaturesLoseAllAbilities",
        // Titania's Song's own static, which strips the whole artifact class.
        "NoncreatureArtifactsLoseAbilities",
        // CR 613's full rewrite: "becomes a 3/3 Elk with no abilities".
        "ResetCreature",
        "BecomeCreatureType",
    ];
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut wrong: Vec<&str> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        let oracle = entry.get("oracle_text").and_then(|v| v.as_str()).unwrap_or("");
        if !oracle.contains("loses all abilities") && !oracle.contains("lose all abilities") {
            continue;
        }
        checked += 1;
        let dbg = format!("{def:?}");
        if !STRIPPERS.iter().any(|n| dbg.contains(n)) {
            wrong.push(def.name);
        }
    }
    assert!(
        checked > 20,
        "only {checked} cards print the clause — did the cache move?",
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} of {checked} card(s) print \"loses all abilities\" and never strip: {wrong:?}",
        wrong.len(),
    );
}

/// Cards whose printed `{T}` activation is not in the tree, each because the
/// ability wants a primitive the engine does not have. **An entry is a filed
/// card job, not a pass** — the card is missing an ability until it is done.
const PRINTED_TAP_ALLOWED: &[(&str, &str)] = &[
    ("Biomancer's Familiar", "\"the next time target creature adapts this turn, it \
      adapts as though it had no +1/+1 counters\" — adapt is an `Effect::If` on a \
      counter test, not a keyword, so there is nothing for a delayed modifier to hook"),
    ("Conduit of Worlds", "\"if you haven't cast a spell this turn, you may cast that \
      card [from your graveyard]. If you do, you can't cast additional spells this \
      turn\" — wants a cast-from-graveyard that installs a one-spell lock"),
    ("Eladamri, Korvecdal", "\"reveal a card from your hand OR the top card of your \
      library\" — a chooser over two zones with a put-onto-battlefield tail; \
      `tap_others_cost` covers the cost half, nothing covers the effect"),
    ("Rydia, Summoner of Mist", "\"return target Saga card with mana value X from your \
      graveyard to the battlefield\" — no filter for a graveyard card of mana value X"),
    ("Sequence Engine", "\"exile target creature card with mana value X from a \
      graveyard\" — same missing filter; CARD_BACKLOG already files this card for \
      shipping a `RevealUntilFind` it does not print"),
];

/// **No card drops a printed `{T}` from its activation cost.**
///
/// A `{T}` an ability does not carry is the same defect class as a cost spelled
/// in the effect: the ability stops being once-a-turn and becomes repeatable,
/// which is an unbounded action loop under bot self-play (a Pentad Prism whose
/// charge counter was an *effect* is what ran a `--decks cube` game to the
/// 50,000-action cap). It is also simply a different card — a mana creature
/// that does not tap makes infinite mana.
///
/// The check is deliberately one-directional and coarse: **if the printed text
/// has an activation whose cost contains `{T}`, the body must carry at least
/// one `tap_cost` (or `untap_self_cost`, for `{Q}`)**. It does not try to pair
/// abilities up — a card with two activations, one tapping, is not a finding
/// — because the pairing is what a per-card read is for and a ratchet that
/// guesses it produces noise.
///
/// Reminder text is stripped first, so a basic land's `({T}: Add {G}.)` — which
/// is *entirely* reminder — is not a printed activation at all and the
/// direction of the assert makes that a non-event.
#[test]
fn no_card_drops_a_printed_tap_symbol() {
    use crabomination::effect::StaticEffect;
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    let allowed: HashSet<&str> = PRINTED_TAP_ALLOWED.iter().map(|(n, _)| *n).collect();

    /// ⚠ **A quoted ability belongs to something else.** "Enchanted land has
    /// `\"{T}, Sacrifice a creature: …\"`", "create a Treasure token with
    /// `\"{T}, Sacrifice this artifact: Add …\"`", "all creatures gain
    /// `\"{T}: …\"`" — the `{T}` is on the aura's host, the token, or a
    /// temporary grant, none of which is this card's own activation. That is
    /// 80 of the first 92 rows, so the quoted spans come out before anything
    /// is read.
    /// Does `text` hold an activation line whose **cost** (left of the first
    /// colon) carries `{t}`? Only the cost half counts: "…: Tap target
    /// creature" is not a tap cost, and `{t}` inside the effect half never is.
    fn prints_tap_activation(text: &str) -> bool {
        strip_quoted(text).lines().any(|line| {
            line.split_once(':')
                .is_some_and(|(cost, _)| cost.contains("{t}") && cost.len() < 120)
        })
    }

    let mut checked = 0usize;
    let mut bad: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        // Two-faced entries spell both halves into one blob; the body here is
        // the front face alone, so the comparison would have to follow
        // `back_face`. Same gate as the keyword ratchet above.
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        checked += 1;
        let text = strip_reminders(oracle);
        if !prints_tap_activation(&text) || allowed.contains(def.name) {
            continue;
        }
        // Granted abilities count: "Creatures you control have '{T}: Add one
        // mana of any color'" prints the `{T}` on this card and carries it on
        // the grant.
        let granted = def.static_abilities.iter().filter_map(|sa| match &sa.effect {
            StaticEffect::GrantActivatedAbility { ability, .. } => Some(ability),
            StaticEffect::GrantActivatedAbilityFromGraveyard { ability, .. } => Some(&**ability),
            _ => None,
        });
        let taps = def
            .activated_abilities
            .iter()
            .chain(granted)
            .any(|a| a.tap_cost || a.untap_self_cost || a.tap_other_filter.is_some());
        if !taps {
            bad.push(def.name.to_string());
        }
    }
    assert!(checked > 15_000, "only {checked} cards compared — the ratchet is vacuous");
    bad.sort();
    bad.dedup();
    assert!(
        bad.is_empty(),
        "{} card(s) print a `{{T}}` activation and carry no tap cost, so the ability is \
         repeatable in a turn. Give it `tap_cost`, or add the card to PRINTED_TAP_ALLOWED \
         with the reason the tap lives elsewhere: {:?}",
        bad.len(),
        &bad[..bad.len().min(40)]
    );
}

/// Cards whose `sorcery_speed` disagrees with the printed restriction, with
/// the reason it is right anyway.
const SORCERY_SPEED_ALLOWED: &[(&str, &str)] = &[];

/// **A card's `sorcery_speed` matches the restriction it prints.**
///
/// Both directions are defects and neither is cosmetic. A *missing* one lets
/// the bot activate at instant speed — one more window per turn to find a
/// loop in, and one more line the opponent cannot plan against. An *invented*
/// one takes the ability away for most of the turn; Hammer of Purphoros
/// carried one its oracle does not have, and its doc comment repeated it.
///
/// Reminder text and quoted spans come out first, for the same reasons as the
/// `{T}` ratchet: "Activate only as a sorcery" inside a quoted grant belongs
/// to the token or the enchanted permanent, not to this card.
#[test]
fn every_card_has_the_activation_timing_it_prints() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    let allowed: HashSet<&str> = SORCERY_SPEED_ALLOWED.iter().map(|(n, _)| *n).collect();

    let mut checked = 0usize;
    let mut missing: Vec<&str> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        if def.activated_abilities.is_empty() || allowed.contains(def.name) {
            continue;
        }
        checked += 1;
        // ⚠ **Reminder text comes out, and for this direction that is exactly
        // right.** A rider printed inside a parenthetical belongs to that
        // keyword's own ability — Equip, Saddle, Embalm, Scavenge, Outlast and
        // level up all say "only as a sorcery" in reminder text, and the
        // engine models each of those somewhere other than an
        // `ActivatedAbility`. Keeping reminders read Leonin Bola, Bulwark Ox,
        // Cursecloth Wrappings and Varolz as findings; stripping them leaves
        // exactly the cards whose own activation states the rider.
        let text = strip_quoted(&strip_reminders(oracle));
        // ⚠ **Only a card with exactly ONE printed activation is checked.** The
        // rider sits on one ability, and this test deliberately does not pair
        // printed abilities to modelled ones — so on a card with two, "the
        // text says sorcery somewhere" is not a claim about the ability the
        // tree has. Ghost Vacuum, Keys to the House and Secluded Starforge all
        // print the rider on their *second* activation and were findings until
        // this gate.
        let activations: Vec<&str> = text
            .lines()
            .filter(|l| l.split_once(':').is_some_and(|(c, _)| c.contains('{') && c.len() < 120))
            .collect();
        if activations.len() != 1 {
            continue;
        }
        let prints = text.contains("only as a sorcery")
            || text.contains("only any time you could cast a sorcery");
        let has = def.activated_abilities.iter().any(|a| a.sorcery_speed);
        // Only the *permissive* direction is a ratchet. The other one — a
        // sorcery restriction the card does not print — is a real class too
        // (the monstrosity helpers carried one for twenty-odd cards, fixed in
        // the same commit), but its remaining rows need a per-card read
        // against CR: a Class's level abilities are sorcery-speed by CR 716.4
        // and print nothing, and "activate only during your turn" is a
        // *different* rider the engine approximates with this flag. The triage
        // so far is in CARD_BACKLOG.
        if prints && !has {
            missing.push(def.name);
        }
    }
    assert!(checked > 1_000, "only {checked} cards had an activated ability — vacuous");
    missing.sort();
    assert!(
        missing.is_empty(),
        "{} card(s) print \"activate only as a sorcery\" and are instant-speed in the tree, \
         which hands the bot a window the card does not give it: {:?}",
        missing.len(),
        &missing[..missing.len().min(40)],
    );
}

/// **A card that prints "activate only once each turn" carries the limit.**
///
/// The permissive half again, and the same failure mode: without the limit the
/// bot gets every activation the mana allows instead of one, which is both a
/// different card and one more shape for an action loop to live in.
/// `once_per_turn` and `max_activations_per_turn` both satisfy it.
///
/// Same two gates as the timing ratchet, for the same reasons: reminder text
/// and quoted spans come out (the limit inside a granted ability's quotes is
/// that ability's), and a card with more than one printed activation is
/// skipped because the limit sits on one of them.
#[test]
fn every_card_has_the_activation_limit_it_prints() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut missing: Vec<&str> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        if def.activated_abilities.is_empty() {
            continue;
        }
        let text = strip_quoted(&strip_reminders(oracle));
        let activations: Vec<&str> = text
            .lines()
            .filter(|l| l.split_once(':').is_some_and(|(c, _)| c.contains('{') && c.len() < 120))
            .collect();
        if activations.len() != 1 {
            continue;
        }
        checked += 1;
        // ⚠ "This ability **triggers** only once each turn" is a limit on a
        // *triggered* ability and has nothing to do with an activation — six
        // cards read as findings on the bare phrase. Anchor on the verb.
        if !(text.contains("activate only once each turn")
            || text.contains("activate this ability only once each turn"))
        {
            continue;
        }
        let limited = def
            .activated_abilities
            .iter()
            .any(|a| a.once_per_turn || a.max_activations_per_turn.is_some() || a.activate_once);
        if !limited {
            missing.push(def.name);
        }
    }
    assert!(checked > 500, "only {checked} single-activation cards seen — vacuous");
    missing.sort();
    assert!(
        missing.is_empty(),
        "{} card(s) print \"activate only once each turn\" and are unlimited in the tree: {:?}",
        missing.len(),
        &missing[..missing.len().min(40)],
    );
}

/// **A card that prints "this ability triggers only once each turn" carries
/// `EventSpec.once_per_turn`.**
///
/// The trigger-side sibling of the activation-limit ratchet, and the same
/// permissive failure: without the flag the ability fires on every matching
/// event instead of the first, which on a token-maker (Elvish Warmaster,
/// Baron Bertram Graywater) compounds with whatever else reads tokens
/// entering. The primitive exists — `EventSpec::once_per_turn` — so every one
/// of these is a one-field fix, not a missing mechanic.
///
/// The exception is a card whose limited trigger is *absent from the tree*
/// entirely — there is no `EventSpec` for the flag to sit on, so it is a card
/// job rather than a one-field fix. Those live in
/// `TRIGGER_LIMIT_ABILITY_DROPPED` with the oracle line they drop.
#[test]
fn every_card_has_the_trigger_limit_it_prints() {
    /// Cards printing a once-a-turn trigger limit on an ability that isn't
    /// modeled at all. Adding the flag here would be a lie; implementing the
    /// ability is the fix, and is filed in `CARD_BACKLOG.md`.
    const TRIGGER_LIMIT_ABILITY_DROPPED: &[&str] = &[
        // "Whenever Calix or an enchanted creature you control deals combat
        // damage to a player, you may create a token that's a copy of a
        // nonlegendary enchantment you control. Do this only once each turn."
        // Only the constellation half is modeled.
        "Calix, Guided by Fate",
    ];

    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut missing: Vec<&str> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        // ⚠ Quoted spans come out: Cursed Wombat's limit is inside a
        // *granted* ability ("Permanents you control have \"…triggers only
        // once each turn\""), which is that ability's rule and not this
        // card's own trigger.
        let text = strip_quoted(&strip_reminders(oracle));
        // Anchor on the verb: "activate only once each turn" is the other
        // ratchet's question, and the bare phrase catches both.
        if !(text.contains("triggers only once each turn")
            || text.contains("do this only once each turn"))
        {
            continue;
        }
        checked += 1;
        if TRIGGER_LIMIT_ABILITY_DROPPED.contains(&def.name) {
            continue;
        }
        // ⚠ The flag is read off the whole definition, not off
        // `def.triggered_abilities`. A modal card keeps its triggers inside
        // the mode (Hollowmurk Siege's Sultai half carries `.once_per_turn()`
        // and is invisible at the top level), and a top-level-only read
        // reports it as a miss.
        if !format!("{def:?}").contains("once_per_turn: true") {
            missing.push(def.name);
        }
    }
    assert!(checked > 3, "only {checked} cards print the clause — did the cache move?");
    missing.sort();
    assert!(
        missing.is_empty(),
        "{} of {checked} card(s) print a once-a-turn trigger limit and fire on every event: \
         {:?}",
        missing.len(),
        &missing[..missing.len().min(40)],
    );
}

// ── The printed-clause family, one body ─────────────────────────────────────

/// The shared body of the printed-clause ratchets below. Walk the catalog
/// against the cache, keep the cards whose **own** printed text satisfies
/// `prints`, and report the ones whose definition does not satisfy `models`.
///
/// `prints` is handed the card's oracle text with reminder text and quoted
/// spans already removed, lowercased, plus the card's lowercased name — every
/// clause in this family needs the name to tell "**this** creature can't
/// block" from "**enchanted** creature can't block", which is a different card
/// granting the restriction to something else.
///
/// The five older ratchets in this file each carry their own copy of this
/// walk; they predate the helper and are left alone rather than churned.
fn clause_ratchet(
    clause: &str,
    floor: usize,
    prints: impl Fn(&str, &str) -> bool,
    models: impl Fn(&crabomination::card::CardDefinition, &str) -> bool,
) {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut missing: Vec<&str> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        // A two-faced card's clause can belong to either face and the
        // top-level `oracle_text` is not the face's, so skip as elsewhere.
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        let text = strip_quoted(&strip_reminders(oracle));
        let name = def.name.to_lowercase();
        if !prints(&text, &name) {
            continue;
        }
        checked += 1;
        let body = format!("{def:?}");
        if !models(&def, &body) {
            missing.push(def.name);
        }
    }
    assert!(checked >= floor, "only {checked} cards print {clause:?} — did the cache move?");
    missing.sort();
    assert!(
        missing.is_empty(),
        "{} of {checked} card(s) print {clause:?} and do not model it: {:?}",
        missing.len(),
        &missing[..missing.len().min(40)],
    );
}

/// True if some line of `text` is about the card itself and satisfies `f`.
/// "Enchanted creature can't block" is an Aura granting the restriction to
/// somebody else and must not count as the Aura's own printing.
fn own_line(text: &str, name: &str, f: impl Fn(&str) -> bool) -> bool {
    text.lines()
        .map(str::trim)
        .any(|l| (l.starts_with("this ") || l.starts_with(name)) && f(l))
}

/// **A permanent that prints "This land enters tapped." enters tapped.**
///
/// A tempo gift the bot cannot see: an untapped Guildgate is a strictly better
/// card than the printed one, and every game played with it is mis-scored.
///
/// ⚠ **Unconditional lines only, and "unless" is not the only conditional.**
/// "…enters tapped **unless** you control two or more other lands" and
/// "…enters tapped **if** it's not your turn" (Eddymurk Crab, Slumbering
/// Trudge) are both different cards. "…enters tapped **and** you gain 1 life"
/// (Noble's Purse, Sphere of the Suns, Leviathan) does count — the rest of the
/// sentence does not change the tapped half.
///
/// ⚠ **Two spellings, and both are in the catalog**: `sets::etb_tap`'s trigger
/// (`Tap { what: This }`) and `StaticEffect::EntersTapped` (the rules-correct
/// replacement). Looking for only the trigger reports Leviathan as a miss.
#[test]
fn every_permanent_that_prints_entering_tapped_enters_tapped() {
    clause_ratchet(
        "enters tapped",
        100,
        |t, n| {
            own_line(t, n, |l| {
                l.contains(" enters tapped") && !l.contains("unless") && !l.contains(" if ")
            })
        },
        |_, body| body.contains("Tap { what: This }") || body.contains("EntersTapped {"),
    );
}

/// **A spell that prints "This spell can't be countered." carries
/// `Keyword::CantBeCountered`.**
///
/// The bot's counterspell heuristics otherwise see a legal target the printed
/// card does not offer — a wrong line taken with real frequency in blue
/// mirrors.
///
/// ⚠ **Self-referential lines only.** "Creature spells you control can't be
/// countered" (Cavern of Souls) is a static about *other* objects and belongs
/// to a different primitive; anchoring on the exact sentence keeps those out.
#[test]
fn every_spell_that_prints_uncounterable_carries_the_keyword() {
    clause_ratchet(
        "this spell can't be countered",
        20,
        |t, _| t.lines().map(str::trim).any(|l| l == "this spell can't be countered."),
        |def, _| def.keywords.contains(&crabomination::card::Keyword::CantBeCountered),
    );
}

/// **A permanent that prints "doesn't untap during your untap step" does not
/// untap.**
///
/// `StaticEffect::PreventUntap` is the primitive. Missing it is the difference
/// between a one-shot and a repeatable, which is a mana or a blocker every
/// turn for the rest of the game.
///
/// ⚠ 48 of the 102 cache rows are Auras — "**Enchanted** creature doesn't
/// untap" is the Aura granting it, not the Aura itself doing it — so the
/// self-anchor is doing most of the work here.
#[test]
fn every_permanent_that_prints_not_untapping_does_not_untap() {
    clause_ratchet(
        "doesn't untap during",
        20,
        |t, n| own_line(t, n, |l| l.contains("doesn't untap during")),
        // ⚠ The conditional variants count. Goblin Rock Sled ("if it attacked
        // during your last turn") is `Keyword::DoesntUntapIfAttackedLastTurn`
        // and Steel Dromedary ("if it has a +1/+1 counter on it") is
        // `DoesntUntapWhileCounter` — both correct, and both read as misses
        // against `PreventUntap` alone.
        |_, body| body.contains("PreventUntap") || body.contains("DoesntUntap"),
    );
}

/// **A creature that prints "This creature can't block." can't block.**
///
/// A combat restriction the bot reads straight out of `declare_blockers`, so a
/// miss is a blocker that should not exist — the most directly play-visible
/// class in this family.
#[test]
fn every_creature_that_prints_it_cannot_block_cannot_block() {
    clause_ratchet(
        "can't block",
        40,
        |t, n| own_line(t, n, |l| l == format!("{n} can't block.") || l == "this creature can't block."),
        |def, body| {
            def.keywords.contains(&crabomination::card::Keyword::Defender)
                || body.contains("CantBlock")
                || body.contains("Decayed")
        },
    );
}

/// **A creature that prints "attacks each combat if able" must attack.**
///
/// `Keyword::MustAttack`. The mirror of the previous test on the attack side,
/// and the same argument: the bot is free to hold back a creature the printed
/// card forces into combat, which is a strictly better card.
#[test]
fn every_creature_that_prints_it_must_attack_must_attack() {
    clause_ratchet(
        "attacks each combat if able",
        30,
        |t, n| {
            own_line(t, n, |l| {
                l == format!("{n} attacks each combat if able.")
                    || l == "this creature attacks each combat if able."
            })
        },
        |_, body| body.contains("MustAttack"),
    );
}

/// **A creature that prints "hexproof from …" carries a `HexproofFrom*`
/// keyword.**
///
/// ⚠ **The self-anchor does not work on a keyword line.** "Hexproof from
/// black" is printed bare, with no subject, so `own_line` cannot see it —
/// the rule here is that the line *starts* with the clause, which is what
/// separates the creature's own keyword from an Aura's grant ("**Enchanted**
/// creature has hexproof from black").
#[test]
fn every_creature_that_prints_hexproof_from_carries_it() {
    clause_ratchet(
        "hexproof from",
        5,
        |t, _| t.lines().map(str::trim).any(|l| l.starts_with("hexproof from")),
        |_, body| body.contains("HexproofFrom"),
    );
}

/// **A creature that prints "can't attack unless …" carries a
/// `CantAttackUnless*` keyword.**
///
/// The attack-side sibling of the can't-block ratchet, and the same argument:
/// without it the bot swings with a creature the printing holds back.
#[test]
fn every_creature_that_prints_an_attack_condition_carries_it() {
    clause_ratchet(
        "can't attack unless",
        20,
        |t, n| {
            own_line(t, n, |l| l.contains("can't attack unless"))
                // "…unless you pay {1} for each +1/+1 counter on it"
                // (Phyrexian Marauder) is an attack cost in *mana*, and the
                // only attack-cost keywords are `AttackCostSacrifice` and
                // `AttackCostBounce`. Primitive job, filed in CARD_BACKLOG.
                && n != "phyrexian marauder"
        },
        // ⚠ **Three prefixes, and the obvious one is the rarest.** The
        // restriction spells itself `CanAttackOnlyIfDefenderControls` /
        // `CanAttackOnlyIfYouControl` (the Sea Serpent cycle, War Falcon,
        // Scarred Puma) or `AttackCost*` (Leviathan, Exalted Dragon,
        // Floodtide Serpent) far more often than `CantAttackUnless*`.
        // Grepping only for the clause's own wording reported 13 correct
        // cards as defects.
        |_, body| {
            body.contains("CantAttack")
                || body.contains("CanAttackOnlyIf")
                || body.contains("AttackCost")
        },
    );
}

/// **A creature that prints "must be blocked if able" carries
/// `Keyword::MustBeBlocked`.**
///
/// CR 509.1c, the Lure effect. A missing one is a free attack every combat.
/// The cache has 19 such cards and the catalog carries 2 of them, so the
/// vacuity floor is 2 — raise it when the rest land.
#[test]
fn every_creature_that_prints_lure_carries_it() {
    clause_ratchet(
        "must be blocked if able",
        2,
        |t, n| own_line(t, n, |l| l.contains("must be blocked if able")),
        |_, body| body.contains("MustBeBlocked"),
    );
}

// ── The printed *keyword* ratchet — the MISSING direction, restricted ───────

/// The keywords this ratchet checks, as `(scryfall name, does the definition
/// carry it)`.
///
/// ⚠ **Membership is the whole argument.** A keyword belongs here only if the
/// engine reads it from `CardDefinition.keywords` and **nowhere else** — no
/// bespoke spelling can satisfy it, so a card that prints it and does not
/// carry it is a proven gap rather than a reading-list entry. That is why
/// `prowess`, `exalted`, `cascade`, `riot`, `battle cry`, `mentor`,
/// `training`, `myriad`, `melee` and `storm` are **not** here: each is
/// spelled as a triggered ability in this catalog (Monastery Mentor and Abbot
/// of Keral Keep both ride `shortcut::prowess_trigger()`), and including them
/// reported 68 correct cards on the first run.
#[allow(clippy::type_complexity)]
const PRINTED_KEYWORDS: &[(&str, fn(&crabomination::card::CardDefinition) -> bool)] = {
    use crabomination::card::Keyword as K;
    macro_rules! kw {
        ($name:literal, $variant:ident) => {
            ($name, (|d: &crabomination::card::CardDefinition| d.keywords.contains(&K::$variant))
                as fn(&crabomination::card::CardDefinition) -> bool)
        };
    }
    &[
        kw!("flying", Flying),
        kw!("reach", Reach),
        kw!("menace", Menace),
        kw!("trample", Trample),
        kw!("vigilance", Vigilance),
        kw!("first strike", FirstStrike),
        kw!("double strike", DoubleStrike),
        kw!("lifelink", Lifelink),
        kw!("deathtouch", Deathtouch),
        kw!("haste", Haste),
        kw!("defender", Defender),
        kw!("shroud", Shroud),
        kw!("flash", Flash),
        kw!("indestructible", Indestructible),
        kw!("intimidate", Intimidate),
        kw!("fear", Fear),
        kw!("skulk", Skulk),
        kw!("horsemanship", Horsemanship),
        kw!("shadow", Shadow),
        kw!("changeling", Changeling),
        kw!("devoid", Devoid),
        kw!("infect", Infect),
        kw!("wither", Wither),
        kw!("flanking", Flanking),
        kw!("decayed", Decayed),
        kw!("phasing", Phasing),
    ]
};

/// Words that may share a keyword line without disqualifying it, on top of
/// `PRINTED_KEYWORDS`' own names. A line is only a keyword line if **every**
/// comma-separated piece of it is one of these, which is what separates
/// "Flying, lifelink" from "choose first strike, vigilance, or lifelink".
const KEYWORD_LINE_ALSO: &[&str] = &[
    "prowess", "exalted", "cascade", "riot", "battle cry", "mentor", "training", "myriad",
    "storm", "melee", "convoke", "improvise", "delve", "soulbond", "sunburst", "persist",
    "undying", "split second", "epic", "compleated", "retrace", "jump-start", "afflict",
    "daybound", "nightbound", "banding", "changeling", "devoid",
];

/// Parameterized keywords, matched by prefix because their line carries a
/// cost or a quality ("hexproof from black", "ward {2}", "bushido 1").
const KEYWORD_LINE_PREFIXES: &[&str] = &[
    "hexproof", "protection", "ward", "landwalk", "forestwalk", "islandwalk", "swampwalk",
    "mountainwalk", "plainswalk", "bushido", "annihilator", "rampage", "absorb", "frenzy",
    "toxic", "poisonous", "modular", "crew", "saddle", "casualty", "escape", "dredge",
    "kicker", "multikicker", "bloodthirst", "echo", "cycling", "flashback", "buyback",
    "equip", "enchant", "fortify", "level up", "affinity", "graft", "vanishing", "fading",
    "amplify", "provoke", "soulshift", "splice", "entwine", "madness", "ninjutsu",
    "transmute", "haunt", "replicate", "forecast", "recover", "ripple", "suspend",
    "reinforce", "champion", "devour", "unearth", "conspire", "evoke", "prowl", "hideaway",
];

/// The set of keywords printed on `text` as a **keyword line** — a line whose
/// every comma-separated piece is a keyword.
fn printed_keyword_line_words(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in text.lines() {
        let line = line.trim().trim_end_matches('.');
        if line.is_empty() {
            continue;
        }
        let pieces: Vec<&str> = line.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
        let all_keywords = pieces.iter().all(|p| {
            PRINTED_KEYWORDS.iter().any(|(n, _)| n == p)
                || KEYWORD_LINE_ALSO.contains(p)
                || KEYWORD_LINE_PREFIXES.iter().any(|pre| p.starts_with(pre))
        });
        if all_keywords {
            out.extend(pieces.into_iter().map(str::to_string));
        }
    }
    out
}

/// **A card that prints a keyword on a keyword line carries that keyword.**
///
/// The MISSING half of `no_card_carries_a_mechanic_keyword_it_does_not_print`,
/// restricted to the keywords in `PRINTED_KEYWORDS` — the ones with no bespoke
/// spelling, where a gap is a proof rather than a reading list. It found **25
/// cards** on its first run, every one of them a strictly better card than the
/// printed one at the table: Supreme Phantom without flying, Nessian Asp
/// without reach, Goldhound without first strike or menace, Nivix Cyclops
/// without defender (and so free to attack every turn).
///
/// Two structural gates carry the whole false-positive load:
///
/// 1. **A keyword line, not a mention.** Angelic Skirmisher's "choose first
///    strike, vigilance, or lifelink", Steel Seraph's granted choice, a token
///    minted "with trample, lifelink, and haste" and a "put a flying,
///    lifelink, or +1/+1 counter on it" are all *mentions*; none is a line
///    whose every piece is a keyword.
/// 2. **Level-up cards are skipped.** Student of Warfare prints "First strike"
///    as a bare line inside a `LEVEL 2-6` band, which is a level ability and
///    not the creature's printed keyword.
///
/// Scryfall's own `keywords` array is the other half of the gate — it is what
/// says the word on the line *is* a keyword ability on this card.
#[test]
fn every_card_that_prints_a_keyword_line_carries_those_keywords() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut missing: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        // A two-faced card's `keywords` array unions both faces; the body
        // here is the front face alone.
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        let text = strip_reminders(oracle);
        // Gate 2 — a level band's keyword line is the band's, not the card's.
        if text.contains("level up") {
            continue;
        }
        checked += 1;
        let scryfall: HashSet<String> = entry
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).map(str::to_lowercase).collect())
            .unwrap_or_default();
        if scryfall.is_empty() {
            continue;
        }
        let lines = printed_keyword_line_words(&text);
        for (name, carried) in PRINTED_KEYWORDS {
            if scryfall.contains(*name) && lines.contains(*name) && !carried(&def) {
                missing.push(format!("{} is missing {name}", def.name));
            }
        }
    }

    assert!(
        checked > 15_000,
        "only {checked} cards could be compared against the cache — the ratchet is vacuous \
         below that"
    );
    missing.sort();
    assert!(
        missing.is_empty(),
        "{} card(s) print a keyword they do not carry: {:?}",
        missing.len(),
        &missing[..missing.len().min(40)],
    );
}

/// **A card's printed numbers are the card's numbers: starting loyalty, and
/// power/toughness where the definition does not compute them.**
///
/// The other no-bespoke-spelling family. `base_loyalty` has exactly one
/// spelling and a planeswalker that enters on the wrong number is a different
/// card every game it is cast — Geyadrone Dihada and Nahiri, the Harbinger
/// both entered on 3 against a printed 4 until this ran.
///
/// P/T is only compared where `dynamic_pt` is `None`: a `*/4` body carries
/// `toughness: 0` plus a `DynamicPt` that supplies both halves, which is
/// correct and reads as a mismatch against the printed characteristic
/// (Enigma Drake, Renata, Tymaret are the three).
#[test]
fn every_card_carries_its_printed_numbers() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    // A card the cache holds under the same name but a different frame:
    // "Bounty Hunter" resolves to a non-Magic `Card` type line, and
    // "Maraxus of Keld" to the Vanguard avatar rather than the creature.
    const HOMONYMS: &[&str] = &["Bounty Hunter", "Maraxus of Keld"];

    let numeric = |v: Option<&serde_json::Value>| -> Option<i32> {
        v.and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok())
    };

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        if HOMONYMS.contains(&def.name) {
            continue;
        }
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        checked += 1;
        if let Some(loyalty) = numeric(entry.get("loyalty"))
            && def.card_types.contains(&crabomination::card::CardType::Planeswalker)
            && def.base_loyalty as i32 != loyalty
        {
            wrong.push(format!("{} enters on {} loyalty, printed {loyalty}", def.name, def.base_loyalty));
        }
        if def.dynamic_pt.is_none() {
            // CR 721 — a Spacecraft is a 0/0 artifact until it stations; the
            // printed P/T is the last band's, which is what the cache holds.
            let station_pt = def.station.last().and_then(|b| b.pt);
            let (ours_p, ours_t) = station_pt.unwrap_or((def.power, def.toughness));
            for (field, ours) in [("power", ours_p), ("toughness", ours_t)] {
                if let Some(printed) = numeric(entry.get(field))
                    && ours != printed
                {
                    wrong.push(format!("{} has {field} {ours}, printed {printed}", def.name));
                }
            }
        }
    }

    assert!(
        checked > 15_000,
        "only {checked} cards could be compared against the cache — the ratchet is vacuous \
         below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} card(s) carry a number the printing does not: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// Printed subtype words that are **not** creature types, so a creature card
/// carrying one on the same type line (an enchantment creature Saga, an
/// artifact creature Equipment) does not read as a missing creature type.
/// Only words that also name a `CreatureType` variant need listing; the rest
/// fail to deserialize and are skipped for free.
const NON_CREATURE_SUBTYPES: &[&str] = &[
    "Saga", "Aura", "Room", "Class", "Shrine", "Curse", "Equipment", "Vehicle", "Food",
    "Clue", "Treasure", "Blood", "Gold", "Powerstone", "Fortification", "Attraction",
    "Spacecraft", "Map", "Junk", "Incubator", "Background", "Rune", "Cartouche", "Shard",
    "Case", "Bobblehead", "Lander", "Graveborn", "Adventure", "Siege", "Plains", "Island",
    "Swamp", "Mountain", "Forest", "Desert", "Gate", "Lair", "Locus", "Mine", "Tower",
    "Cave", "Sphere", "Town",
];

/// **A creature's printed subtypes are the creature types it carries — both
/// directions.**
///
/// Tribal is the whole point: a missing type is a lord that skips the
/// creature, an Ally counter that never lands, a Phyrexian-matters trigger
/// that does not see it, and a bot that evaluates the board as though the
/// card were something else. It found **148 cards** on its first run and
/// **0 invented types**, which is why both directions are asserted here.
///
/// Three defects were one line each rather than one card each, which is what
/// makes this worth a ratchet:
///
/// * `tla.rs`'s `ally()` helper did not append `CreatureType::Ally` — ten of
///   its callers shipped without the type the helper is named for. Four
///   others were not Allies at all and now use a plainly named `ct()`.
/// * `ogw/creatures.rs`'s `processor()` gave every "Eldrazi Processor" only
///   `Eldrazi`.
///
/// `CreatureType` derives `Deserialize`, so "is this printed word a type we
/// model" is asked of the enum itself rather than a hand-kept list — five
/// words are genuinely unmodelled today (Astartes, Guest, Rigger, Lobster,
/// Gamer) and skip out through that.
#[test]
fn every_creature_carries_its_printed_subtypes() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    // The cache holds the Vanguard avatar under this name, not the creature.
    const HOMONYMS: &[&str] = &["Maraxus of Keld"];
    let normalize = |w: &str| w.replace(['-', '\'', '\u{2019}'], "");

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        if HOMONYMS.contains(&def.name) {
            continue;
        }
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(type_line) = entry.get("type_line").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some((base, sub)) = type_line.split_once('—') else {
            continue;
        };
        if !base.contains("Creature") {
            continue;
        }
        checked += 1;
        let printed: HashSet<String> = sub.split_whitespace().map(normalize).collect();
        let ours: HashSet<String> =
            def.subtypes.creature_types.iter().map(|t| format!("{t:?}")).collect();

        for word in &printed {
            if ours.contains(word) || NON_CREATURE_SUBTYPES.contains(&word.as_str()) {
                continue;
            }
            // The enum itself answers "do we model this type": a word that
            // is not a variant is a type the engine has no name for, and
            // that is a different (and much larger) job than a missing one.
            let modelled = serde_json::from_value::<crabomination::card::CreatureType>(
                serde_json::Value::String(word.clone()),
            )
            .is_ok();
            if modelled {
                wrong.push(format!("{} is not a {word}", def.name));
            }
        }
        for word in &ours {
            if !printed.contains(word) {
                wrong.push(format!("{} is a {word} and the printing is not", def.name));
            }
        }
    }

    assert!(
        checked > 5_000,
        "only {checked} creatures could be compared against the cache — the ratchet is \
         vacuous below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} creature subtype mismatch(es): {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// **Every `EventScope::SelfSource` trigger in the catalog must reach a
/// dispatcher that admits it.** A kind the scope check has no arm for is a
/// silently dead ability: the card ships, its test asserts something else, and
/// nothing ever fails.
///
/// Three shipped cards were dead this way — Sand Golem, Mangara's Blessing and
/// Pure Intentions' self-return half, all on the discard events, which
/// `event_matches_spec`'s `SelfSource` chain named nowhere.
///
/// The check reads the engine's own two walkers rather than a hand-kept list:
///
/// * `event_kind_matches`' `(EventKind::X, GameEvent::Y)` arms — the kind half;
/// * the `EventScope::SelfSource =>` block's `GameEvent::Y` names — the scope
///   half;
/// * `is_event_hardcoded`'s `GameEvent::Y => matches!(spec.scope,
///   EventScope::SelfSource)` arms, which are the kinds dispatched *inline* by
///   the emitting site and so never reach the scope chain at all.
///
/// It asks a **population** question the engine's own unit tests cannot: those
/// walk a fixed family (`is_graveyard_self_source_kind`'s, and that the two
/// walkers reading it agree), this walks every card in the catalog. The two
/// are complements — the kind gate inside that family is checked there, the
/// set of kinds cards actually use is checked here.
///
/// A catalog kind that none of the three reaches lands in `UNPROVEN`, whose
/// entries each carry the dedicated path that dispatches them. The ratchet is
/// that list: a **new** kind cannot join it by accident.
#[test]
fn every_self_source_trigger_kind_reaches_a_dispatcher() {
    use crabomination::effect::EventScope;

    /// Kinds the three walkers above cannot prove, each with the dedicated
    /// path that dispatches it. Adding a row here means having read that path.
    ///
    /// Every one of them is a *push* site: the emitting code walks the
    /// participants and queues the trigger itself, so the scope chain is never
    /// consulted and a missing arm there costs nothing. That is why the
    /// discard family was the exception — it had no push site and depended on
    /// the chain that did not name it.
    const UNPROVEN: &[(&str, &str)] = &[
        // The combat-damage hook: `fire_combat_damage_triggers` and its
        // creature-side twin push these off the damage they just dealt.
        ("DealsCombatDamage", "combat.rs fire_combat_damage_triggers"),
        ("DealsCombatDamageToPlayer", "combat.rs fire_combat_damage_triggers"),
        ("DealsCombatDamageToCreature", "combat.rs fire_combat_damage_triggers"),
        ("DealsCombatDamageToPlaneswalker", "combat.rs, the planeswalker leg"),
        ("ControllerDealtCombatDamage", "combat.rs, the controller leg"),
        // The non-combat damage halves, same file, same device.
        ("DealsDamage", "combat.rs fire_noncombat_damage_triggers"),
        ("DealsDamageToPlayer", "combat.rs fire_noncombat_damage_triggers"),
        ("DealsDamageToCreature", "combat.rs fire_noncombat_damage_triggers"),
        // "Whenever you attack" — pushed by `declare_attackers` off the batch.
        ("YouAttack", "combat.rs declare_attackers"),
        // CR 700.4 — the LKI death path collects the dying permanent's own
        // triggers directly (`stack.rs`, beside `CreatureDied` /
        // `PermanentLeavesBattlefield`, which the chain does name).
        ("PermanentDied", "stack.rs, the leave-trigger collection"),
        ("BecomesPlotted", "actions.rs, the plot action"),
        // Planechase (CR 901): the planar die and the walk push their own.
        ("ChaosEnsues", "stack.rs planar_trigger_pushes"),
        ("Encountered", "stack.rs planar_trigger_pushes"),
        ("PlaneswalkedAwayFrom", "stack.rs planar_trigger_pushes"),
        // CR 701.25 — a scheme set in motion, a turn-based action.
        ("SetInMotion", "stack.rs, the scheme turn-based action"),
    ];

    let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../crabomination/src/game");
    let events_rs = fs::read_to_string(engine.join("effects/events.rs")).expect("events.rs");
    let mod_rs = fs::read_to_string(engine.join("mod.rs")).expect("game/mod.rs");

    // (a) the kind -> event pairing, off `event_kind_matches`' arms.
    let kind_block = {
        let start = events_rs.find("match (&spec.kind, event) {").expect("kind match");
        &events_rs[start..]
    };
    let mut pairs: Vec<(String, String)> = Vec::new();
    for (i, _) in kind_block.match_indices("EventKind::") {
        let kind: String = kind_block[i + "EventKind::".len()..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        // The `GameEvent::` that follows before the next `EventKind::` is this
        // arm's event; arms are written `(EventKind::X, GameEvent::Y ..)`.
        let rest = &kind_block[i..];
        let next_kind = rest[1..].find("EventKind::").map(|j| j + 1).unwrap_or(rest.len());
        if let Some(j) = rest[..next_kind].find("GameEvent::") {
            let ev: String = rest[j + "GameEvent::".len()..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            pairs.push((kind, ev));
        }
    }
    assert!(pairs.len() > 100, "kind/event pairing came out empty: {}", pairs.len());

    // (b) the events the `SelfSource` scope arm names.
    let scope_block = {
        let start = events_rs.find("EventScope::SelfSource => matches!(").expect("scope arm");
        let end = events_rs[start..].find("EventScope::YourControl =>").expect("arm end");
        &events_rs[start..start + end]
    };
    let mut self_events: HashSet<String> = scope_block
        .match_indices("GameEvent::")
        .map(|(i, _)| {
            scope_block[i + "GameEvent::".len()..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        })
        .collect();
    assert!(self_events.len() > 20, "SelfSource arm came out empty: {self_events:?}");

    // (c) the events dispatched inline, which never reach (b).
    let hard_block = {
        let start = mod_rs.find("fn is_event_hardcoded(").expect("is_event_hardcoded");
        let end = mod_rs[start..].find("\n}\n").expect("fn end");
        &mod_rs[start..start + end]
    };
    for line in hard_block.lines() {
        if !line.contains("EventScope::SelfSource") && !line.trim_end().ends_with("=> true,") {
            continue;
        }
        if let Some(i) = line.find("GameEvent::") {
            self_events.insert(
                line[i + "GameEvent::".len()..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect(),
            );
        }
    }

    let mut checked = 0usize;
    let mut dead: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let equip = def.equipped_bonus.iter().flat_map(|b| b.triggered_abilities.iter());
        for ta in def.triggered_abilities.iter().chain(equip) {
            if ta.event.scope != EventScope::SelfSource {
                continue;
            }
            checked += 1;
            let kind = format!("{:?}", ta.event.kind);
            let kind = kind.split('(').next().unwrap_or(&kind).to_string();
            if UNPROVEN.iter().any(|(k, _)| *k == kind) {
                continue;
            }
            let reachable = pairs
                .iter()
                .any(|(k, ev)| *k == kind && self_events.contains(ev));
            if !reachable {
                dead.push(format!("{kind} (e.g. {})", def.name));
            }
        }
    }
    assert!(checked > 1_000, "population check: only {checked} SelfSource triggers walked");
    dead.sort();
    dead.dedup_by(|a, b| a.split(' ').next() == b.split(' ').next());
    assert!(
        dead.is_empty(),
        "{} SelfSource trigger kind(s) no dispatcher admits — the ability is dead. \
         Add the arm to `events.rs`' SelfSource chain, or add the kind to `UNPROVEN` with \
         the dedicated path that dispatches it: {:?}",
        dead.len(),
        dead
    );
}

/// **A card's printed type line is the card's type line** — supertypes, card
/// types, land types and artifact / enchantment subtypes, in both directions.
///
/// The join ratchet whose *invented* half is the loud one: **89 land types on
/// 45 lands** that print none. Every fastland, gainland, creature-land, Snarl
/// and Temple, the five Mirrodin artifact lands, Karakas, Bojuka Bog,
/// Mortuary Mire and Hidden Grotto carried the basic types they tap for,
/// which made each of them fetchable by "search for a Swamp card", live to
/// swampwalk, and counted for domain. Four shared helpers, not 45 mistakes.
///
/// 25 in the other direction, `Aura` on four enchantments among them — and
/// adding those four turned up two facts about the CR 704.5m orphan sweep
/// (`stack.rs` carries both, `CARD_BACKLOG` the write-up).
///
/// Two structural rules, each of which arrived as a false positive:
///
/// 1. **A subtype family is only compared when the printed type line carries
///    the card type that owns it.** "Legendary Planeswalker — Urza" names a
///    planeswalker subtype that spells a `LandType`.
/// 2. **A printed word satisfied by *any* of the card's subtype lists is
///    satisfied.** The type line does not say which family a word belongs to,
///    and "Book" is both an `ArtifactSubtype` and a `CreatureType` — Jayemdae
///    Tome against Codie, Vociferous Codex.
#[test]
fn every_card_carries_its_printed_type_line() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    const HOMONYMS: &[&str] = &["Maraxus of Keld", "Bounty Hunter", "Resurgent Belief"];
    let normalize = |w: &str| match w.replace(['-', '\'', '\u{2019}'], "").as_str() {
        // `{:?}` spellings the printed words do not match letter for letter.
        "Urzas" => "Urza".to_string(),
        other => other.to_string(),
    };
    // "Is this printed word a type we model" is asked of the enum, not of a
    // hand-kept list, so a word the engine has no name for is skipped rather
    // than reported as a missing type.
    let modelled = |label: &str, w: &String| {
        let v = serde_json::Value::String(w.clone());
        match label {
            "supertype" => serde_json::from_value::<crabomination::card::Supertype>(v).is_ok(),
            "card type" => serde_json::from_value::<crabomination::card::CardType>(v).is_ok(),
            "land type" => serde_json::from_value::<crabomination::card::LandType>(v).is_ok(),
            "artifact subtype" => {
                serde_json::from_value::<crabomination::card::ArtifactSubtype>(v).is_ok()
            }
            _ => serde_json::from_value::<crabomination::card::EnchantmentSubtype>(v).is_ok(),
        }
    };
    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        if HOMONYMS.contains(&def.name) { continue; }
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else { continue };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        { continue; }
        let Some(type_line) = entry.get("type_line").and_then(|v| v.as_str()) else { continue };
        checked += 1;
        let (base, sub) = match type_line.split_once('—') {
            Some((b, s)) => (b, s),
            None => (type_line, ""),
        };
        let base_words: HashSet<String> = base.split_whitespace().map(normalize).collect();
        let sub_words: HashSet<String> = sub.split_whitespace().map(normalize).collect();
        let mut all_ours: HashSet<String> =
            def.subtypes.creature_types.iter().map(|t| format!("{t:?}")).collect();
        all_ours.extend(def.subtypes.land_types.iter().map(|t| format!("{t:?}")));
        all_ours.extend(def.subtypes.artifact_subtypes.iter().map(|t| format!("{t:?}")));
        all_ours.extend(def.subtypes.enchantment_subtypes.iter().map(|t| format!("{t:?}")));
        all_ours.extend(def.subtypes.spell_subtypes.iter().map(|t| format!("{t:?}")));
        all_ours
            .extend(def.subtypes.planeswalker_subtypes.iter().map(|t| format!("{t:?}")));
        // A subtype family is only compared when the printed type line has
        // the card type that owns it — "Legendary Planeswalker — Urza" names
        // a planeswalker subtype that happens to spell a `LandType`.
        let owns = |t: &str| base.contains(t);
        for (label, ours, printed) in [
            ("supertype",
             def.supertypes.iter().map(|t| format!("{t:?}")).collect::<HashSet<_>>(),
             base_words.clone()),
            ("card type",
             def.card_types.iter().map(|t| format!("{t:?}")).collect::<HashSet<_>>(),
             base_words.clone()),
            ("land type",
             def.subtypes.land_types.iter().map(|t| format!("{t:?}")).collect::<HashSet<_>>(),
             if owns("Land") { sub_words.clone() } else { HashSet::new() }),
            ("artifact subtype",
             def.subtypes.artifact_subtypes.iter().map(|t| format!("{t:?}")).collect::<HashSet<_>>(),
             if owns("Artifact") { sub_words.clone() } else { HashSet::new() }),
            ("enchantment subtype",
             def.subtypes.enchantment_subtypes.iter().map(|t| format!("{t:?}")).collect::<HashSet<_>>(),
             if owns("Enchantment") { sub_words.clone() } else { HashSet::new() }),
        ] {
            if printed.is_empty() && !ours.is_empty() && !matches!(label, "supertype" | "card type")
            {
                // The family does not apply to this card at all; the
                // INVENTED direction below would report every entry.
                for w in &ours {
                    wrong.push(format!("{}: {label} {w} is not printed", def.name));
                }
                continue;
            }
            for w in ours.difference(&printed) {
                wrong.push(format!("{}: {label} {w} is not printed", def.name));
            }
            for w in printed.difference(&ours) {
                // A printed word satisfied by *any* subtype list is
                // satisfied: the type line does not say which family a word
                // belongs to, and "Book" is both an `ArtifactSubtype` and a
                // `CreatureType` (Jayemdae Tome against Codie).
                if modelled(label, w) && !all_ours.contains(w) {
                    wrong.push(format!("{}: printed {label} {w} is missing", def.name));
                }
            }
        }
    }
    assert!(
        checked > 15_000,
        "only {checked} cards could be compared against the cache — the ratchet is vacuous \
         below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} type-line mismatch(es): {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// The colours an `Effect` tree can add, as Scryfall spells them (`W`, `U`,
/// `B`, `R`, `G`, `C`).
///
/// `None` means **"this tree says something this walk does not model"** — a
/// context-dependent payload (`AnyColorOpponentCouldProduce`, `Restricted`,
/// `DevotionOfChosenColor`, …) or a container variant not descended here.
/// `None` propagates, and the caller skips the card: the walk is only allowed
/// to accuse a card whose whole mana story it read.
fn mana_colors_added(e: &crabomination::effect::Effect) -> Option<HashSet<char>> {
    use crabomination::effect::{Effect, ManaPayload};
    use crabomination::mana::Color;
    let letter = |c: &Color| match c {
        Color::White => 'W',
        Color::Blue => 'U',
        Color::Black => 'B',
        Color::Red => 'R',
        Color::Green => 'G',
    };
    let all = || "WUBRG".chars().collect::<HashSet<_>>();
    let union = |parts: Vec<Option<HashSet<char>>>| -> Option<HashSet<char>> {
        let mut out = HashSet::new();
        for p in parts {
            out.extend(p?);
        }
        Some(out)
    };
    match e {
        Effect::AddMana { pool, .. } => match pool {
            ManaPayload::Colors(cs) => Some(cs.iter().map(letter).collect()),
            ManaPayload::OfColor(c, _) => Some([letter(c)].into_iter().collect()),
            ManaPayload::OfColors(cs, _) => Some(cs.iter().map(letter).collect()),
            ManaPayload::Colorless(_) => Some(['C'].into_iter().collect()),
            ManaPayload::AnyOneColor(_) | ManaPayload::AnyColors(_) => Some(all()),
            _ => None,
        },
        Effect::Seq(inner) | Effect::ChooseMode(inner) => {
            union(inner.iter().map(mana_colors_added).collect())
        }
        Effect::ChooseN { modes, .. } => union(modes.iter().map(mana_colors_added).collect()),
        Effect::If { then, else_, .. } | Effect::IfRevealFromHand { then, else_, .. } => {
            union(vec![mana_colors_added(then), mana_colors_added(else_)])
        }
        Effect::MayDo { body, .. }
        | Effect::AtNextEndStep { body }
        | Effect::PayEnergy { then: body, .. } => mana_colors_added(body),
        // Anything else: a leaf that is not `AddMana` contributes nothing —
        // but a *container* this walk does not descend would silently
        // contribute nothing too, and that is a false accusation waiting to
        // happen. `Rhystic Cave`'s `unless_anyone_pays` wrapper is exactly
        // that shape and read as "produces no colour" on the first run.
        //
        // The debug rendering is the cheap oracle: if the subtree mentions
        // `AddMana` anywhere and this arm reached it, the walk did not read
        // the card and must say so.
        other => {
            if format!("{other:?}").contains("AddMana") {
                None
            } else {
                Some(HashSet::new())
            }
        }
    }
}

/// **A land taps for exactly the mana its printing produces.**
///
/// The fourth join ratchet, and the one whose defects are the most directly
/// deck-breaking: a land that cannot make the colour it prints is a mana base
/// the bot cannot route around, and one that makes a colour it does not print
/// is a free fixer in every deck that runs it.
///
/// Lands only. A mana rock or a creature can add mana through a bespoke
/// resolver the walk above cannot read, and Scryfall's `produced_mana`
/// carries entries for cards whose "production" is a Treasure token; lands
/// are the population where the printed field and the effect tree mean the
/// same thing.
///
/// ⚠ **The walk is allowed to skip.** `mana_colors_added` answers `None` for
/// a context-dependent payload (`AnyColorOpponentCouldProduce`, a `Restricted`
/// wrapper, `DevotionOfChosenColor`) and for any container it does not
/// descend, and a `None` anywhere skips the card. That is the safe direction:
/// the ratchet only accuses a card whose whole mana story it read.
#[test]
fn every_land_taps_for_the_mana_it_prints() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        if !def.card_types.contains(&crabomination::card::CardType::Land) {
            continue;
        }
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(printed) = entry.get("produced_mana").and_then(|v| v.as_array()) else {
            continue;
        };
        // Scryfall counts mana an ability *grants elsewhere* toward
        // `produced_mana`. Three shapes do that and none is readable from
        // this land's own effect tree:
        //   * a static that gives lands a basic type, so they tap for its
        //     colour (Urborg, Yavimaya) — including this land itself;
        //   * a static that grants a mana ability to other permanents or
        //     from another zone (Forgotten Monument, Riftstone Portal);
        //   * a saga chapter that grants one (Urza's Saga).
        let statics = format!("{:?}", def.static_abilities);
        if !def.saga_chapters.is_empty()
            || statics.contains("LandType")
            || statics.contains("Grant")
        {
            continue;
        }
        // The allow-list is empty. It held **Pit of Offerings** ("{T}: Add
        // one mana of any of the exiled cards' colors") until
        // `ManaPayload::AnyColorAmongExiledWithSource` was built for it.
        let printed: HashSet<char> =
            printed.iter().filter_map(|v| v.as_str()).filter_map(|s| s.chars().next()).collect();

        // The engine gives a land with a basic land type that type's mana
        // ability intrinsically, so the colours never appear in the effect
        // tree (Gingerbread Cabin is "Land — Forest" with no `AddMana`).
        let mut ours: HashSet<char> = def
            .subtypes
            .land_types
            .iter()
            .filter_map(|t| match t {
                crabomination::card::LandType::Plains => Some('W'),
                crabomination::card::LandType::Island => Some('U'),
                crabomination::card::LandType::Swamp => Some('B'),
                crabomination::card::LandType::Mountain => Some('R'),
                crabomination::card::LandType::Forest => Some('G'),
                _ => None,
            })
            .collect();
        let mut readable = true;
        for a in &def.activated_abilities {
            match mana_colors_added(&a.effect) {
                Some(cs) => ours.extend(cs),
                None => readable = false,
            }
        }
        for t in &def.triggered_abilities {
            match mana_colors_added(&t.effect) {
                Some(cs) => ours.extend(cs),
                None => readable = false,
            }
        }
        if !readable {
            continue;
        }
        checked += 1;
        // `{S}` (snow mana) has no `Color` here, and Scryfall does not list a
        // separate letter for it either — both sides spell it `C`.
        for c in printed.difference(&ours) {
            wrong.push(format!("{} does not tap for {c}", def.name));
        }
        for c in ours.difference(&printed) {
            wrong.push(format!("{} taps for {c} and the printing does not", def.name));
        }
    }

    // Four gates skip cards; the floor is what keeps a widened gate from
    // quietly emptying the population. 628 lands pass all four today.
    assert!(
        checked > 400,
        "only {checked} lands could be compared against the cache — the ratchet is vacuous \
         below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} land mana mismatch(es) over {checked} lands: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// **A planeswalker has every loyalty ability its printing has, and no
/// others.**
///
/// A missing loyalty ability is a whole line of the card that never happens,
/// and it is invisible to every other audit here: the tree is well-formed,
/// the card resolves, and its test asserts the abilities that *are* there.
///
/// The printed count is the number of oracle lines that open with a loyalty
/// cost — `+N:`, `−N:` (U+2212, which is what Scryfall uses, not a hyphen),
/// `0:`, and their `X` forms.
#[test]
fn every_planeswalker_has_its_printed_loyalty_abilities() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    /// A line opens a loyalty ability when it starts with `+`, `−`, `-` or a
    /// bare `0`, followed by a number or `X`, then a colon.
    fn printed_loyalty_lines(text: &str) -> usize {
        text.lines()
            .map(str::trim)
            .filter(|l| {
                let rest = l.strip_prefix(['+', '\u{2212}', '-']).unwrap_or(l);
                let Some((head, _)) = rest.split_once(':') else { return false };
                !head.is_empty()
                    && head.len() <= 2
                    // ⚠ `strip_reminders` lowercases, so the X of a `−X:`
                    // ability arrives as `x`.
                    && head.chars().all(|c| c.is_ascii_digit() || c == 'x' || c == 'X')
                    // A bare "0:" only counts when the line really is the
                    // whole ability, not a "0" inside prose.
                    && (l.starts_with(['+', '\u{2212}', '-']) || head != *l)
            })
            .count()
    }

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        if !def.card_types.contains(&crabomination::card::CardType::Planeswalker) {
            continue;
        }
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        checked += 1;
        let printed = printed_loyalty_lines(&strip_reminders(oracle));
        let ours = def.loyalty_abilities.len();
        if printed != ours {
            wrong.push(format!("{} has {ours} loyalty abilities, printed {printed}", def.name));
        }
    }

    assert!(
        checked > 100,
        "only {checked} planeswalkers could be compared against the cache — the ratchet is \
         vacuous below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} planeswalker(s) with the wrong loyalty-ability count: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// **A Saga has every chapter its printing has.**
///
/// The sibling of the loyalty-arity ratchet, and the same argument: a
/// dropped chapter is a printed line that never happens, and the tree it
/// leaves behind is well-formed.
///
/// A printed chapter opens with roman numerals and an em dash — `I —`,
/// `II —`, `I, II —` (one ability on two chapters), `III, IV —`. The count
/// is of distinct chapter *numbers*, which is what `saga_chapters` holds.
#[test]
fn every_saga_has_its_printed_chapters() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    fn printed_chapters(text: &str) -> HashSet<u32> {
        let roman = |w: &str| match w {
            "i" => Some(1),
            "ii" => Some(2),
            "iii" => Some(3),
            "iv" => Some(4),
            "v" => Some(5),
            _ => None,
        };
        let mut out = HashSet::new();
        for line in text.lines() {
            let Some((head, _)) = line.split_once('\u{2014}') else { continue };
            let head = head.trim();
            if head.is_empty() {
                continue;
            }
            let nums: Vec<Option<u32>> =
                head.split(',').map(|w| roman(w.trim())).collect();
            if nums.iter().all(Option::is_some) {
                out.extend(nums.into_iter().flatten());
            }
        }
        out
    }

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        let printed = printed_chapters(&strip_reminders(oracle));
        if printed.is_empty() && def.saga_chapters.is_empty() {
            continue;
        }
        checked += 1;
        let ours: HashSet<u32> = def.saga_chapters.iter().map(|(n, _)| *n).collect();
        for n in printed.difference(&ours) {
            wrong.push(format!("{} is missing chapter {n}", def.name));
        }
        for n in ours.difference(&printed) {
            wrong.push(format!("{} has a chapter {n} the printing does not", def.name));
        }
    }

    assert!(
        // Most Sagas printed since 2021 are transforming DFCs, which the
        // single-face join skips; 42 single-faced ones reach here.
        checked > 30,
        "only {checked} Sagas could be compared against the cache — the ratchet is vacuous \
         below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} Saga chapter mismatch(es) over {checked} Sagas: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// The largest number of modes any `ChooseMode` / `ChooseN` in `e` offers,
/// or `None` if the tree contains neither.
fn modal_arm_count(e: &crabomination::effect::Effect) -> Option<usize> {
    use crabomination::effect::Effect;
    let best = |a: Option<usize>, b: Option<usize>| match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, None) => x,
        (None, y) => y,
    };
    match e {
        Effect::ChooseMode(modes) => Some(modes.len()),
        Effect::ChooseN { modes, .. } => Some(modes.len()),
        Effect::Seq(inner) => inner.iter().map(modal_arm_count).fold(None, best),
        Effect::If { then, else_, .. } => best(modal_arm_count(then), modal_arm_count(else_)),
        Effect::MayDo { body, .. } | Effect::AtNextEndStep { body } => modal_arm_count(body),
        _ => None,
    }
}

/// **A modal card offers every mode its printing bullets.**
///
/// The third arity ratchet. A dropped mode is a line of the card the player
/// can never pick, and — like a dropped loyalty ability — it leaves a
/// well-formed tree that every structural audit passes.
///
/// The printed count is the number of `•` bullets. Cards whose definition
/// contains **no** `ChooseMode` / `ChooseN` at all are skipped: a modal card
/// can legitimately be spelled some other way (Spree and Escalate carry their
/// own fields, and a "choose one" on a *triggered* ability is a different
/// shape), and this ratchet is about arms that exist and are short.
#[test]
fn every_modal_card_offers_its_printed_modes() {
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/.scryfall_cache.json");
    let raw = fs::read_to_string(&cache_path).expect(".scryfall_cache.json");
    let cache: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("cache is a JSON object");
    let by_name: std::collections::HashMap<String, &serde_json::Value> = cache
        .iter()
        .filter(|(_, v)| v.is_object())
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        let Some(entry) = by_name.get(&def.name.to_lowercase()) else {
            continue;
        };
        if entry.get("card_faces").is_some()
            || entry.get("name").and_then(|v| v.as_str()).is_some_and(|n| n.contains(" // "))
        {
            continue;
        }
        let Some(oracle) = entry.get("oracle_text").and_then(|v| v.as_str()) else {
            continue;
        };
        let text = strip_reminders(oracle);
        let printed = text.lines().filter(|l| l.trim_start().starts_with('\u{2022}')).count();
        if printed == 0 {
            continue;
        }
        // Only the spell body — an ability's own modal arms answer a
        // different question and are counted with their own bullets.
        let Some(ours) = modal_arm_count(&def.effect) else {
            continue;
        };
        checked += 1;
        if ours < printed {
            wrong.push(format!("{} offers {ours} of its {printed} printed modes", def.name));
        }
    }

    assert!(
        checked > 100,
        "only {checked} modal cards could be compared against the cache — the ratchet is \
         vacuous below that"
    );
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "{} modal card(s) short of their printed modes, over {checked}: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// Every `TokenDefinition` nested anywhere in `def`, found by serde-walking
/// the serialized card — the same device `crabomination::audit` uses for the
/// dead-ability pass. Nothing else can reach a token: they are values inside
/// effect trees, not catalog entries, so **no ratchet in this file sees one**
/// unless it walks for them.
fn nested_tokens(def: &crabomination::card::CardDefinition) -> Vec<crabomination::card::TokenDefinition> {
    fn walk(v: &serde_json::Value, out: &mut Vec<crabomination::card::TokenDefinition>) {
        match v {
            serde_json::Value::Object(map) => {
                if map.contains_key("power")
                    && map.contains_key("toughness")
                    && map.contains_key("card_types")
                    && map.get("name").and_then(|n| n.as_str()).is_some()
                    && let Ok(t) = serde_json::from_value::<crabomination::card::TokenDefinition>(
                        v.clone(),
                    )
                {
                    out.push(t);
                }
                for (_, child) in map {
                    walk(child, out);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    walk(child, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    let Ok(v) = serde_json::to_value(def) else { return out };
    // Skip the top-level object, which is the card itself.
    if let serde_json::Value::Object(map) = &v {
        for (_, child) in map {
            walk(child, &mut out);
        }
    }
    out
}

/// **Every "artifact-that-does-a-thing" token is the one shared definition,
/// modulo whether it enters tapped.**
///
/// Treasure, Food, Clue, Blood, Powerstone and Map are minted by 400-odd
/// cards between them, each of which writes the token inline. One body per
/// token lives in `crabomination_base::tokens`; this asserts nobody has a
/// second one. A *private* second body is exactly how four cards came to
/// mint Treasures with no `Treasure` subtype.
///
/// `tapped` is normalised out because it is a property of the **minting**
/// clause, not of the token: "create a *tapped* Treasure token" is printed on
/// Hell to Pay, Magda, Season of the Bold and Kain, and all four are right.
#[test]
fn every_standard_artifact_token_is_the_shared_one() {
    use crabomination::card::TokenDefinition;
    type Canon = (&'static str, fn() -> TokenDefinition);
    let canon: &[Canon] = &[
        ("Treasure", crabomination_base::tokens::treasure_token),
        ("Food", crabomination_base::tokens::food_token),
        ("Clue", crabomination_base::tokens::clue_token),
        ("Blood", crabomination_base::tokens::blood_token),
        ("Powerstone", crabomination_base::tokens::powerstone_token),
        ("Map", crabomination_base::tokens::map_token),
    ];
    // The allow-list is empty, and it is worth saying why: **Goldspan
    // Dragon** was on it until its printed static was built. It minted a
    // bespoke two-mana Treasure; the card grants the two-mana ability to
    // *every* Treasure you control, and `GrantActivatedAbility` says so.
    const ALLOWED: &[&str] = &[];

    let mut checked = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        if ALLOWED.contains(&def.name) {
            continue;
        }
        for mut t in nested_tokens(&def) {
            let Some((_, make)) = canon.iter().find(|(n, _)| *n == t.name) else {
                continue;
            };
            checked += 1;
            let mut want = make();
            // The minting clause decides this, not the token.
            t.tapped = false;
            want.tapped = false;
            if format!("{t:?}") != format!("{want:?}") {
                wrong.push(format!("{}'s {} token is not the shared one", def.name, t.name));
            }
        }
    }
    assert!(
        checked > 300,
        "only {checked} standard artifact tokens were walked — the ratchet is vacuous below that"
    );
    wrong.sort();
    wrong.dedup();
    assert!(
        wrong.is_empty(),
        "{} card(s) carry their own copy of a shared token, over {checked}: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}

/// **A token named after a subtype the engine models carries that subtype.**
///
/// Tokens are values inside effect trees, not catalog entries, so **no other
/// ratchet in this file can see one** — the join ratchets walk
/// `all_catalog_card_factories()` and a token has no Scryfall row to join
/// against. This one walks for them.
///
/// It found the shape that motivates it: `decks/modern.rs` carried a
/// **private `treasure_token()` shadowing the shared one** in
/// `crabomination_base::tokens`, identical but for the missing
/// `ArtifactSubtype::Treasure`. Six call sites used it, so four cards minted
/// Treasures that no "sacrifice a Treasure" cost, Treasure-matters payoff or
/// artifact-subtype filter could see. The duplicate is deleted and its
/// callers point at the shared helper.
///
/// Name → subtype is asked of the enums (both derive `Deserialize`), so a
/// token whose name the engine has no subtype for is skipped for the right
/// reason: `Gold` (Theros) and `Shard` (Niko) are real printed subtypes with
/// no variant here, and adding one later needs no edit to this test.
#[test]
fn every_token_named_after_a_subtype_carries_it() {
    let mut wrong: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for factory in crabomination_catalog::sets::all_factories::all_catalog_card_factories() {
        let def = factory();
        for t in nested_tokens(&def) {
            let name = serde_json::Value::String(t.name.clone());
            if let Ok(sub) =
                serde_json::from_value::<crabomination::card::ArtifactSubtype>(name.clone())
            {
                checked += 1;
                if !t.subtypes.artifact_subtypes.contains(&sub) {
                    wrong.push(format!("{}: its {} token is not an {sub:?}", def.name, t.name));
                }
            } else if let Ok(sub) =
                serde_json::from_value::<crabomination::card::EnchantmentSubtype>(name)
            {
                checked += 1;
                if !t.subtypes.enchantment_subtypes.contains(&sub) {
                    wrong.push(format!("{}: its {} token is not a {sub:?}", def.name, t.name));
                }
            }
        }
    }
    assert!(
        checked > 300,
        "only {checked} named-after-a-subtype tokens were walked — the ratchet is vacuous \
         below that"
    );
    wrong.sort();
    wrong.dedup();
    assert!(
        wrong.is_empty(),
        "{} token(s) missing the subtype they are named for, over {checked}: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(40)],
    );
}
