use crate::card::{ActivatedAbility, CardDefinition, CardType, Effect, Supertype};
use crate::effect::{ManaPayload, PlayerRef, Value};
use crate::mana::{cost, generic};

/// Nykthos, Shrine to Nyx — Legendary Land. {T}: Add {C}. {2}, {T}: Choose
/// a color. Add mana of that color equal to your devotion to that color
/// (CR 700.5), via the `DevotionOfChosenColor` payload.
pub fn nykthos_shrine_to_nyx() -> CardDefinition {
    CardDefinition {
        name: "Nykthos, Shrine to Nyx",
        cost: cost(&[]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::DevotionOfChosenColor,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
