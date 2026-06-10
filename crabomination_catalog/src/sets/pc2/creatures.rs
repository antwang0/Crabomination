use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::effect::{Selector, Value};
use crate::mana::{b, cost, u};

/// Baleful Strix — {U}{B} 1/1 Flying Deathtouch.
/// "When Baleful Strix enters the battlefield, draw a card."
pub fn baleful_strix() -> CardDefinition {
    CardDefinition {
        name: "Baleful Strix",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}
