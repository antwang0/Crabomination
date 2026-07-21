//! Ravnica (RAV) gap wave 16: the Svogthos man-land. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, SelectionRequirement as R, Value,
};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::mana::{b, cost, g, generic};

/// Svogthos, the Restless Tomb — Land. {T}: Add {C}. {3}{B}{G}: Until end of
/// turn, this land becomes a Plant Zombie creature whose power and toughness
/// each equal the number of creature cards in your graveyard. It's still a land.
/// (The black-and-green color grant is cosmetic and omitted.)
pub fn svogthos_the_restless_tomb() -> CardDefinition {
    CardDefinition {
        name: "Svogthos, the Restless Tomb",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b(), g()]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                    toughness: Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::Creature,
                    },
                    creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
