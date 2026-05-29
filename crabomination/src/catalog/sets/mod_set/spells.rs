//! Additional Modern-staple instants not already covered by `instants.rs`:
//! enchantment removal, narrower counterspells, and Dovin's Veto's
//! "can't-be-countered" rider.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CounterType, Effect, EventKind, EventScope, EventSpec,
    SelectionRequirement, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, Selector, StaticEffect, Value};
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Static Prison — `{X}{2}{W}` Enchantment. Static Prison enters with X
/// stun counters on it. Tap target permanent. At the beginning of each
/// upkeep, remove a stun counter; while it has stun counters, that
/// permanent doesn't untap.
///
/// Push (modern_decks): the Stun-counter wire **now lands on the
/// targeted permanent** (was previously stamping the counters on
/// Static Prison itself, where they had no untap relevance). The
/// engine's existing Stun-counter mechanic (CR 122.1d) keeps the
/// target tapped — at each untap step, one stun counter is removed
/// instead of the target being untapped. So an X=2 Static Prison
/// taps the target now + keeps it tapped for X turn cycles. The
/// printed "at the beginning of your upkeep, remove a stun counter"
/// rider is naturally handled by the engine's stun-counter consume-
/// on-untap behavior.
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
                Effect::Tap {
                    what: target_filtered(SelectionRequirement::Permanent),
                },
                // Stamp X Stun counters on the TARGET, not on the
                // Prison itself. Each stun counter consumes an untap
                // in the target's next untap step (CR 122.1d).
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Permanent),
                    kind: CounterType::Stun,
                    amount: Value::XFromCost,
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Exploration — {G} Enchantment (Urza's Saga reprint). "You may play
/// an additional land on each of your turns." Single static grant of
/// `ExtraLandPerTurn`. The per-turn land cap is checked by
/// [`GameState::can_player_play_land`] (CR 305.2a), which sums every
/// `ExtraLandPerTurn` static effect controlled by the active player.
/// Stacks multiplicatively: two Explorations → three lands per turn.
pub fn exploration() -> CardDefinition {
    CardDefinition {
        name: "Exploration",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}
