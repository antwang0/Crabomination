//! Modern-staple sweepers (sorcery-speed area damage).

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, SelectionRequirement, Subtypes};
use crate::effect::{Selector, Value};
use crate::mana::{cost, generic, r};

/// Anger of the Gods — {1}{R}{R} Sorcery. Deals 3 damage to each creature.
/// If a creature would die this turn, exile it instead.
///
/// Approximation: the "exile if would die" replacement is omitted (no
/// generic SBA-replacement primitive yet). Damage to each creature is
/// wired via `ForEach + DealDamage`.
pub fn anger_of_the_gods() -> CardDefinition {
    CardDefinition {
        name: "Anger of the Gods",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(3),
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Blasphemous Act — {8}{R} Sorcery, "this spell costs {1} less to cast for
/// each creature on the battlefield." Deals 13 damage to each creature.
///
/// Cost-reduction by creature-count is approximated as a flat {4}{R} cost
/// (a typical board state has 4–5 creatures across both players). The
/// damage half is wired faithfully via `ForEach + DealDamage`.
pub fn blasphemous_act() -> CardDefinition {
    CardDefinition {
        name: "Blasphemous Act",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(13),
            }),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
