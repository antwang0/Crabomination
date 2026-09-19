use crate::card::{CardDefinition, CardType, LandType, SelectionRequirement};
use crate::effect::{ActivatedAbility, Effect, PlayerRef, ZoneDest};
use crate::mana::ManaCost;

/// Build a fetch land activated ability: {T}, pay 1 life, sacrifice this:
/// search your library for a `type_a` or `type_b` land and put it onto the
/// battlefield (untapped — Oracle text for Onslaught/Zendikar fetches says
/// "put it onto the battlefield", which means untapped).
///
/// The life and the sacrifice are activation COSTS (CR 602.2b / 119.4):
/// paid before the ability is on the stack, so a Stifled fetch is still gone
/// and still cost the life, the payment needs 1 life to make, and a
/// pay-life replacement (Ashiok, Wicked Manipulator) sees it. They were
/// `LoseLife` + `Move` steps of the resolution until 2026-09-19.
fn fetch_ability(type_a: LandType, type_b: LandType) -> ActivatedAbility {
    let filter =
        SelectionRequirement::HasLandType(type_a).or(SelectionRequirement::HasLandType(type_b));
    ActivatedAbility {
        energy_cost: 0,
        discard_cost: None,
        tap_cost: true,
        mana_cost: ManaCost::default(),
        // Search library for a land of either type, put it onto the
        // battlefield untapped.
        effect: Effect::Search {
            who: PlayerRef::You,
            filter,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: true,
        condition: None,
        life_cost: 1,
        from_graveyard: false,
        exile_self_cost: false,
        exile_other_filter: None,
        self_counter_cost_reduction: None,
        sac_other_filter: None,
        tap_other_filter: None,
        from_hand: false,
        ..Default::default()
    }
}

fn fetch_land(name: &'static str, type_a: LandType, type_b: LandType) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![fetch_ability(type_a, type_b)],
        ..Default::default()
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

/// Prismatic Vista — {T}, pay 1 life, sacrifice: search your library for a
/// basic land and put it onto the battlefield.
pub fn prismatic_vista() -> CardDefinition {
    CardDefinition {
        name: "Prismatic Vista",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
