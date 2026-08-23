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
