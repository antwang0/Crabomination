use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::PlayerRef;
use crate::mana::{cost, r};

/// Goblin Guide — {R} 2/2 Haste
/// Whenever Goblin Guide attacks, defending player reveals the top card of their library.
/// If it's a land card, that player puts it into their hand.
pub fn goblin_guide() -> CardDefinition {
    CardDefinition {
        name: "Goblin Guide",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::RevealTopAndDrawIf {
                // Oracle: "defending player" — the specific opponent (or
                // planeswalker controller) Goblin Guide is attacking.
                who: PlayerRef::DefendingPlayer,
                reveal_filter: SelectionRequirement::Land,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}
