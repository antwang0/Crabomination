//! Ravnica (RAV) gap wave 16: the Svogthos man-land and Shadow of Doubt.
//! Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, SelectionRequirement as R, Value,
};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::mana::{Color, b, cost, g, generic, hybrid};

/// Svogthos, the Restless Tomb — Land. {T}: Add {C}. {3}{B}{G}: Until end of
/// turn, this land becomes a black and green Plant Zombie creature whose power
/// and toughness each equal the number of creature cards in your graveyard.
/// It's still a land. Approximation: the printed P/T is a characteristic-
/// defining ability; here the count is read once, as the ability resolves
/// (`BecomeCreature` sets a fixed P/T), so a creature card hitting your
/// graveyard later that turn doesn't grow it.
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
                effect: crate::effect::shortcut::colored_animation(Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::Creature,
                    },
                    toughness: Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::Creature,
                    },
                    creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                }, &[crate::mana::Color::Black, crate::mana::Color::Green]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Shadow of Doubt — {U/B}{U/B} Instant. Players can't search libraries this
/// turn. Draw a card.
pub fn shadow_of_doubt() -> CardDefinition {
    CardDefinition {
        name: "Shadow of Doubt",
        cost: cost(&[
            hybrid(Color::Blue, Color::Black),
            hybrid(Color::Blue, Color::Black),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventSearchesThisTurn,
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
