//! Commander: the 2026-10-17 (`01Htv6cF`) engine-blocked one-card gaps —
//! cards that needed a new primitive. Tests in `tests/recent_b/cmdr_oct17_b.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, Subtypes};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{ManaCost, cost, g, generic};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Green Dragon — {4}{G}{G} 4/4 flying Dragon. Poison Breath — ETB: until end
/// of turn, whenever a creature an opponent controls is dealt damage, destroy
/// it.
pub fn green_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DamagedOpponentCreaturesDieThisTurn)],
        ..creature("Green Dragon", cost(&[generic(4), g(), g()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Seasons Past — {4}{G}{G} Sorcery. Return any number of cards with
/// different mana values from your graveyard to your hand; put Seasons Past
/// on the bottom of its owner's library.
pub fn seasons_past() -> CardDefinition {
    CardDefinition {
        name: "Seasons Past",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        library_bottom_on_resolve: true,
        effect: Effect::ReturnDistinctManaValuesFromGraveyard { who: PlayerRef::You },
        ..Default::default()
    }
}
