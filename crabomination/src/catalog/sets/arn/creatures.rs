use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Selector,
    Subtypes, TriggeredAbility, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic};

/// Juzám Djinn — {2}{B}{B} 5/5
/// At the beginning of your upkeep, Juzám Djinn deals 1 damage to you.
pub fn juzam_djinn() -> CardDefinition {
    CardDefinition {
        name: "Juzám Djinn",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}
