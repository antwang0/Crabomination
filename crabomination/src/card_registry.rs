//! Card name → factory registry used by snapshot deserialization.
//!
//! Moved out of `crabomination_catalog` because it aggregates the cube, demo
//! and SoS card pools, which are defined in this (top-level) crate. Installed
//! as the resolver for `crabomination_base`'s `CardInstance` `Deserialize` impl
//! at startup (see the `ctor` registration in `lib.rs`).

use std::collections::HashMap;
use std::sync::OnceLock;

use crabomination_catalog::sets;
use crabomination_catalog::CardFactory;

use crate::card::CardDefinition;

/// Aggregate of every card-producing factory the snapshot/restore path
/// knows about: cube cards, SoS cards, demo decks, and engine-baked
/// tokens. Used to resolve a card name back to its full `CardDefinition`
/// when loading a saved game state from disk.
pub fn all_known_factories() -> Vec<CardFactory> {
    let mut all: Vec<CardFactory> = crate::cube::all_cube_cards();
    all.extend(crate::sos_mode::all_sos_cards());
    for &f in crate::demo::brg_combo_deck() {
        all.push(f);
    }
    for &f in crate::demo::goryos_vengeance_deck() {
        all.push(f);
    }
    // STX (Strixhaven 2021) factory list — large but bounded; the
    // dedup pass below removes any factory the cube/sos pools already
    // exposed. Without this, mid-game snapshots involving STX
    // permanents would fail snapshot-reload at the `name → factory`
    // lookup stage.
    for &f in sets::stx::all_stx_card_factories() {
        all.push(f);
    }
    // Extra-turn spells (sets::xtra) — registered so mid-game snapshots
    // involving them round-trip through the name→factory lookup.
    let xtra: [CardFactory; 5] = [
        sets::xtra::time_walk,
        sets::xtra::time_warp,
        sets::xtra::temporal_manipulation,
        sets::xtra::capture_of_jingzhou,
        sets::xtra::nexus_of_fate,
    ];
    all.extend_from_slice(&xtra);
    // Dedupe by function-pointer address so repeated copies of the same
    // card across decks/cube don't bloat the registry.
    let mut seen = std::collections::HashSet::new();
    all.retain(|f| seen.insert(*f as usize));
    all
}

/// Build (once) a name → factory lookup from `all_known_factories`. Used
/// by snapshot deserialization. Token cards generated mid-game (Clue,
/// Treasure, etc.) are added separately via [`lookup_token_by_name`].
fn name_index() -> &'static HashMap<&'static str, CardFactory> {
    static INDEX: OnceLock<HashMap<&'static str, CardFactory>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map: HashMap<&'static str, CardFactory> = HashMap::new();
        for f in all_known_factories() {
            // Calling each factory once at index-build time is cheap (it
            // just allocates a struct) and gives us the card name. We
            // store the factory pointer so callers can re-create the
            // CardDefinition on demand.
            let def = f();
            map.entry(def.name).or_insert(f);
            // MDFC back face: index by the back face's name too so
            // serialized cards in their flipped form still resolve.
            if let Some(back) = def.back_face.as_ref() {
                map.entry(back.name).or_insert(f);
            }
        }
        map
    })
}

/// Look up a `CardDefinition` by name. Returns `None` if no catalog
/// factory produces a card with that name. Used by snapshot
/// deserialization to rebuild `CardInstance`s from saved game state.
pub fn lookup_by_name(name: &str) -> Option<CardDefinition> {
    // Token cards (Clue, Treasure, Food, Blood) come from the engine-baked
    // token factories — those don't go through `all_known_factories`
    // since they aren't in any deck. Try the token table first.
    if let Some(def) = lookup_token_by_name(name) {
        return Some(def);
    }
    name_index().get(name).map(|f| f())
}

fn lookup_token_by_name(name: &str) -> Option<CardDefinition> {
    use crate::game::effects::{blood_token, clue_token, food_token, token_to_card_definition, treasure_token};
    // Predefined utility tokens (CR 111.10): Clue, Treasure, Food, Blood.
    if let Some(token) = match name {
        "Clue" => Some(clue_token()),
        "Treasure" => Some(treasure_token()),
        "Food" => Some(food_token()),
        "Blood" => Some(blood_token()),
        _ => None,
    } {
        return Some(token_to_card_definition(&token));
    }
    // SOS / STX college tokens — minted by Inkling Summoning, Pest Summoning,
    // Spirit Mascot, Fractal Anomaly, Lorehold Excavation, etc. Snapshots
    // mid-game include these on the battlefield, so the round-trip-load
    // path needs to resolve them by name.
    if let Some(token) = match name {
        "Inkling" => Some(sets::sos::inkling_token()),
        "Pest" => Some(sets::stx::stx_pest_token()),
        "Spirit" => Some(sets::sos::spirit_token()),
        "Fractal" => Some(sets::sos::fractal_token()),
        _ => None,
    } {
        return Some(token_to_card_definition(&token));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_resolves_a_basic_land() {
        let def = lookup_by_name("Forest").expect("Forest should be in the registry");
        assert_eq!(def.name, "Forest");
    }

    #[test]
    fn lookup_resolves_a_cube_creature() {
        let def = lookup_by_name("Lightning Bolt").expect("Lightning Bolt should be in the registry");
        assert_eq!(def.name, "Lightning Bolt");
    }

    #[test]
    fn lookup_resolves_a_token() {
        let def = lookup_by_name("Treasure").expect("Treasure token should resolve via the token table");
        assert_eq!(def.name, "Treasure");
    }

    #[test]
    fn lookup_resolves_each_predefined_utility_token() {
        for token_name in ["Clue", "Treasure", "Food", "Blood"] {
            let def = lookup_by_name(token_name).expect(token_name);
            assert_eq!(def.name, token_name);
        }
    }

    #[test]
    fn lookup_resolves_sos_stx_college_tokens() {
        for token_name in ["Inkling", "Pest", "Spirit", "Fractal"] {
            let def = lookup_by_name(token_name).expect(token_name);
            assert_eq!(def.name, token_name);
            assert!(def.is_creature(),
                "{} token should be a creature definition", token_name);
        }
    }

    #[test]
    fn lookup_returns_none_for_unknown_card() {
        assert!(lookup_by_name("This Card Does Not Exist").is_none());
    }

    #[test]
    fn lookup_resolves_real_stx_cards_through_known_factories() {
        // Real STX cards from the 327-card set should be reachable via
        // snapshot deserialization (lookup_by_name → name_index →
        // all_known_factories). Without an `all_stx_factories()` slice
        // in the index, mid-game snapshots involving STX cards can't be
        // round-tripped through the saved-state JSON path. The known-
        // factories slice is queried lazily — this test stays cheap.
        let def = lookup_by_name("Witherbloom Apprentice")
            .expect("Witherbloom Apprentice should resolve via the STX catalog");
        assert_eq!(def.name, "Witherbloom Apprentice");
        let def = lookup_by_name("Spirited Companion")
            .expect("Spirited Companion should resolve via the STX catalog");
        assert_eq!(def.name, "Spirited Companion");
    }

    #[test]
    fn lookup_resolves_extra_turn_spells() {
        for name in ["Time Walk", "Time Warp", "Temporal Manipulation",
                     "Capture of Jingzhou", "Nexus of Fate"] {
            let def = lookup_by_name(name).expect(name);
            assert_eq!(def.name, name);
        }
    }
}
