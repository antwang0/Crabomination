use crate::card::{CardDefinition, CardType, LandType, SelectionRequirement, Subtypes};
use crate::effect::{
    ActivatedAbility, Effect, PlayerRef, Selector, Value, ZoneDest,
};
use crate::mana::ManaCost;

/// Build a fetch land activated ability: {T}, sacrifice this: search your library
/// for a `type_a` or `type_b` land and put it onto the battlefield tapped.
fn fetch_ability(type_a: LandType, type_b: LandType) -> ActivatedAbility {
    let filter = SelectionRequirement::HasLandType(type_a)
        .or(SelectionRequirement::HasLandType(type_b));
    ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::Seq(vec![
            // Pay 1 life
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            // Sacrifice this land
            Effect::Move {
                what: Selector::This,
                to: ZoneDest::Graveyard,
            },
            // Search library for a land of either type, put it onto battlefield tapped
            Effect::Search {
                who: PlayerRef::You,
                filter,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        ]),
        once_per_turn: false,
        sorcery_speed: false,
    }
}

fn fetch_land(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![fetch_ability(type_a, type_b)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

// ── Onslaught fetch lands (allied-color pairs) ────────────────────────────────

pub fn flooded_strand() -> CardDefinition {
    fetch_land("Flooded Strand", LandType::Plains, LandType::Island)
}

pub fn polluted_delta() -> CardDefinition {
    fetch_land("Polluted Delta", LandType::Island, LandType::Swamp)
}

pub fn bloodstained_mire() -> CardDefinition {
    fetch_land("Bloodstained Mire", LandType::Swamp, LandType::Mountain)
}

pub fn wooded_foothills() -> CardDefinition {
    fetch_land("Wooded Foothills", LandType::Mountain, LandType::Forest)
}

pub fn windswept_heath() -> CardDefinition {
    fetch_land("Windswept Heath", LandType::Forest, LandType::Plains)
}

// ── Zendikar fetch lands (enemy-color pairs) ──────────────────────────────────

pub fn misty_rainforest() -> CardDefinition {
    fetch_land("Misty Rainforest", LandType::Forest, LandType::Island)
}

pub fn scalding_tarn() -> CardDefinition {
    fetch_land("Scalding Tarn", LandType::Island, LandType::Mountain)
}

pub fn verdant_catacombs() -> CardDefinition {
    fetch_land("Verdant Catacombs", LandType::Swamp, LandType::Forest)
}

pub fn arid_mesa() -> CardDefinition {
    fetch_land("Arid Mesa", LandType::Mountain, LandType::Plains)
}

pub fn marsh_flats() -> CardDefinition {
    fetch_land("Marsh Flats", LandType::Plains, LandType::Swamp)
}
