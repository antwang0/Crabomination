//! Card catalog — factory functions for every implemented Magic: The Gathering card.
//! Cards are grouped by the set in which they first appeared.

mod sets;

// Re-export everything so callers use `catalog::some_card()`.
pub use sets::all::*;
pub use sets::ap::*;
pub use sets::arn::*;
pub use sets::dis::*;
pub use sets::fem::*;
pub use sets::gpt::*;
pub use sets::ice::*;
pub use sets::inv::*;
pub use sets::lea::*;
pub use sets::m11::*;
pub use sets::ogw::*;
pub use sets::pc2::*;
pub use sets::por::*;
pub use sets::rav::*;
pub use sets::rtr::*;
pub use sets::tmp::*;
pub use sets::zen::*;
pub use sets::ths::*;
pub use sets::decks::*;
pub use sets::mod_set::*;
pub use sets::sos::*;
pub use sets::stx::*;

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::card::CardDefinition;

type CardFactory = fn() -> CardDefinition;

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
    // Dedupe by function-pointer address so repeated copies of the same
    // card across decks/cube don't bloat the registry.
    let mut seen = std::collections::HashSet::new();
    all.retain(|f| seen.insert(*f as usize));
    all
}

/// Build (once) a name → factory lookup from `all_known_factories`. Used
/// by snapshot deserialization. Token cards generated mid-game (Clue,
/// Treasure, etc.) are added separately via [`token_factories`].
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
    // Token cards (Clue, Treasure, Food, Blood) come from
    // `game::effects::*_token` — those don't go through `all_known_factories`
    // since they aren't in any deck. Try the token table first.
    if let Some(def) = lookup_token_by_name(name) {
        return Some(def);
    }
    name_index().get(name).map(|f| f())
}

fn lookup_token_by_name(name: &str) -> Option<CardDefinition> {
    use crate::game::effects::{blood_token, clue_token, food_token, token_to_card_definition, treasure_token};
    let token = match name {
        "Clue" => clue_token(),
        "Treasure" => treasure_token(),
        "Food" => food_token(),
        "Blood" => blood_token(),
        _ => return None,
    };
    Some(token_to_card_definition(&token))
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
    fn lookup_returns_none_for_unknown_card() {
        assert!(lookup_by_name("This Card Does Not Exist").is_none());
    }
}
