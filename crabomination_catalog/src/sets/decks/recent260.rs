//! MKM (Murders at Karlov Manor) gap batch — the Gruul Mole God.
//! Tests in `tests/recent_b/recent260.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, Supertype,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, Selector, TriggeredAbility, Value,
};
use crate::mana::{cost, g, generic, r};

/// Anzrag, the Quake-Mole — {2}{R}{G} Legendary Creature — Mole God 8/4. When it
/// becomes blocked, untap your creatures and take an extra combat phase.
/// {3}{R}{R}{G}{G}: Anzrag must be blocked this turn if able.
pub fn anzrag_the_quake_mole() -> CardDefinition {
    CardDefinition {
        name: "Anzrag, the Quake-Mole",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mole, CreatureType::God],
            ..Default::default()
        },
        power: 8,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    up_to: None,
                },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), r(), g(), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::MustBeBlocked,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
