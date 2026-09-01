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
