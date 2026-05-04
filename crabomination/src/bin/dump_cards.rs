/// Dump every card definition from the catalog as JSON on stdout.
/// Used by scripts/verify_cards.py to compare our data against Scryfall.
use std::collections::HashSet;

use crabomination::card::{CardDefinition, CardType, Keyword, Supertype};
use crabomination::catalog::all_known_factories;
use crabomination::mana::{ManaCost, ManaSymbol};
use serde::Serialize;

#[derive(Serialize)]
struct CardSummary {
    name: String,
    mana_cost: String,
    cmc: u32,
    card_types: Vec<String>,
    supertypes: Vec<String>,
    creature_types: Vec<String>,
    land_types: Vec<String>,
    power: Option<i32>,
    toughness: Option<i32>,
    keywords: Vec<String>,
    base_loyalty: Option<u32>,
}

fn format_mana_cost(cost: &ManaCost) -> String {
    cost.symbols
        .iter()
        .map(|sym| match sym {
            ManaSymbol::Colored(c) => format!("{{{}}}", c.short_name()),
            ManaSymbol::Generic(n) => format!("{{{}}}", n),
            ManaSymbol::Colorless(n) => "{C}".repeat(*n as usize),
            ManaSymbol::Hybrid(a, b) => format!("{{{}/{}}}", a.short_name(), b.short_name()),
            ManaSymbol::Phyrexian(c) => format!("{{{}/P}}", c.short_name()),
            ManaSymbol::Snow => "{S}".to_string(),
            ManaSymbol::X => "{X}".to_string(),
        })
        .collect()
}

fn keyword_name(kw: &Keyword) -> Option<String> {
    // Only emit keywords that Scryfall also tracks as a keyword ability.
    // Non-standard or engine-internal keywords are skipped.
    let s = match kw {
        Keyword::Flying => "Flying",
        Keyword::Reach => "Reach",
        Keyword::Menace => "Menace",
        Keyword::Haste => "Haste",
        Keyword::Vigilance => "Vigilance",
        Keyword::FirstStrike => "First strike",
        Keyword::DoubleStrike => "Double strike",
        Keyword::Trample => "Trample",
        Keyword::Lifelink => "Lifelink",
        Keyword::Deathtouch => "Deathtouch",
        Keyword::Indestructible => "Indestructible",
        Keyword::Hexproof => "Hexproof",
        Keyword::Shroud => "Shroud",
        Keyword::Flash => "Flash",
        Keyword::Defender => "Defender",
        Keyword::Persist => "Persist",
        Keyword::Undying => "Undying",
        Keyword::Cascade => "Cascade",
        Keyword::Convoke => "Convoke",
        Keyword::Delve => "Delve",
        Keyword::Storm => "Storm",
        Keyword::Prowess => "Prowess",
        Keyword::Infect => "Infect",
        Keyword::Wither => "Wither",
        Keyword::Changeling => "Changeling",
        Keyword::Phasing => "Phasing",
        Keyword::Banding => "Banding",
        Keyword::Retrace => "Retrace",
        Keyword::Shadow => "Shadow",
        Keyword::Horsemanship => "Horsemanship",
        Keyword::Intimidate => "Intimidate",
        Keyword::Skulk => "Skulk",
        Keyword::Rebound => "Rebound",
        Keyword::Exert => "Exert",
        Keyword::Dredge(_) => "Dredge",
        Keyword::Annihilator(_) => "Annihilator",
        Keyword::Ward(_) => "Ward",
        Keyword::Flashback(_) => "Flashback",
        Keyword::Kicker(_) => "Kicker",
        Keyword::Echo(_) => "Echo",
        Keyword::CumulativeUpkeep(_) => "Cumulative upkeep",
        Keyword::Cycling(_) => "Cycling",
        Keyword::Morph(_) => "Morph",
        Keyword::Megamorph(_) => "Megamorph",
        Keyword::Equip(_) => "Equip",
        Keyword::Fortify(_) => "Fortify",
        Keyword::Protection(_) => "Protection",
        // Engine-internal keywords with no Scryfall equivalent:
        // (CantBlock is a card-text restriction, not a Scryfall-tagged
        // keyword, so it's filtered the same way as Unblockable.)
        Keyword::Regenerate(_) | Keyword::Unblockable | Keyword::CantBeCountered
        | Keyword::Recursion | Keyword::Inspired
        | Keyword::CantBlock | Keyword::CantAttack => return None,
    };
    Some(s.to_string())
}

fn card_type_str(ct: &CardType) -> &'static str {
    match ct {
        CardType::Land => "Land",
        CardType::Creature => "Creature",
        CardType::Artifact => "Artifact",
        CardType::Enchantment => "Enchantment",
        CardType::Planeswalker => "Planeswalker",
        CardType::Battle => "Battle",
        CardType::Instant => "Instant",
        CardType::Sorcery => "Sorcery",
        CardType::Kindred => "Kindred",
    }
}

fn supertype_str(st: &Supertype) -> &'static str {
    match st {
        Supertype::Basic => "Basic",
        Supertype::Legendary => "Legendary",
        Supertype::Snow => "Snow",
        Supertype::World => "World",
    }
}

fn summarize(def: &CardDefinition) -> CardSummary {
    CardSummary {
        name: def.name.to_string(),
        mana_cost: format_mana_cost(&def.cost),
        cmc: def.cost.cmc(),
        card_types: def.card_types.iter().map(card_type_str).map(str::to_string).collect(),
        supertypes: def.supertypes.iter().map(supertype_str).map(str::to_string).collect(),
        creature_types: def
            .subtypes
            .creature_types
            .iter()
            .map(|ct| format!("{ct:?}"))
            .collect(),
        land_types: def
            .subtypes
            .land_types
            .iter()
            .map(|lt| format!("{lt:?}"))
            .collect(),
        power: def.is_creature().then_some(def.power),
        toughness: def.is_creature().then_some(def.toughness),
        keywords: def.keywords.iter().filter_map(keyword_name).collect(),
        base_loyalty: def.is_planeswalker().then_some(def.base_loyalty),
    }
}

fn main() {
    let factories = all_known_factories();
    let mut seen: HashSet<String> = HashSet::new();
    let mut cards: Vec<CardSummary> = Vec::new();

    for factory in factories {
        let def = factory();
        // Skip duplicate names (same card in multiple decks/cube).
        if seen.insert(def.name.to_string()) {
            cards.push(summarize(&def));
            // Also include MDFC back faces as separate entries.
            if let Some(back) = &def.back_face
                && seen.insert(back.name.to_string())
            {
                cards.push(summarize(back));
            }
        }
    }

    cards.sort_by(|a, b| a.name.cmp(&b.name));
    println!("{}", serde_json::to_string_pretty(&cards).unwrap());
}
