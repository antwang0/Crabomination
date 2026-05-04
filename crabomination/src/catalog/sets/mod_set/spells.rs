//! Additional Modern-staple instants not already covered by `instants.rs`:
//! enchantment removal, narrower counterspells, and Dovin's Veto's
//! "can't-be-countered" rider.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CounterType, Effect, EventKind, EventScope, EventSpec,
    SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, u, w};

/// Disenchant — {1}{W} Instant. Destroy target artifact or enchantment.
pub fn disenchant() -> CardDefinition {
    CardDefinition {
        name: "Disenchant",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Naturalize — {1}{G} Instant. Mirror of Disenchant in green.
pub fn naturalize() -> CardDefinition {
    CardDefinition {
        name: "Naturalize",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Nature's Claim — {G} Instant. Destroy target artifact or enchantment;
/// its controller gains 4 life.
pub fn natures_claim() -> CardDefinition {
    CardDefinition {
        name: "Nature's Claim",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(4),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Negate — {1}{U} Instant. Counter target noncreature spell.
pub fn negate() -> CardDefinition {
    CardDefinition {
        name: "Negate",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::Noncreature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Dispel — {U} Instant. Counter target instant spell.
pub fn dispel() -> CardDefinition {
    CardDefinition {
        name: "Dispel",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::HasCardType(CardType::Instant)),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Dovin's Veto — {W}{U} Instant. Counter target noncreature spell. This
/// spell can't be countered. The "can't be countered" rider is encoded as
/// `Keyword::CantBeCountered`; `caster_grants_uncounterable` flags the spell
/// so `CounterSpell` and `CounterUnlessPaid` skip it on the stack.
pub fn dovins_veto() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Dovin's Veto",
        cost: cost(&[w(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::Noncreature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Static Prison — `{X}{2}{W}` Enchantment. Static Prison enters with X
/// stun counters on it. Tap target permanent. At the beginning of each
/// upkeep, remove a stun counter; while it has stun counters, that
/// permanent doesn't untap.
///
/// Approximation: ETB stamps `Value::XFromCost` Stun counters on
/// itself and **also** taps the targeted permanent immediately. The
/// "while it has stun counters, target doesn't untap" suppression
/// clause and the upkeep counter-removal still ⏳ (no untap-
/// replacement primitive yet). The current wiring captures the most
/// important play: tap a permanent for at least one turn cycle.
pub fn static_prison() -> CardDefinition {
    use crate::mana::ManaSymbol;
    // Real Oracle: `{X}{2}{W}` Enchantment.
    let mut prison_cost = cost(&[generic(2), w()]);
    prison_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Static Prison",
        cost: prison_cost,
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Stun,
                    amount: Value::XFromCost,
                },
                Effect::Tap {
                    what: target_filtered(SelectionRequirement::Permanent),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}
