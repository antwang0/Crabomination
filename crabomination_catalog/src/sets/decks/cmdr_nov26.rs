//! EDHREC single-commander average decks short only their commander (scan of
//! 2026-11-26). Tests in `tests/recent_b/cmdr_nov26.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Predicate, SelectionRequirement as R,
    Selector, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{Duration, Effect};
use crate::mana::{cost, r, u, w};

/// Massimo, the Magician — {U}{R}{W} legendary 3/2 Cat Wizard. Whenever it or
/// another creature you control enters, exile up to one target instant or
/// sorcery card with mana value 1 from your graveyard; that creature gains
/// "Whenever this creature deals combat damage to a player, copy the exiled
/// card. You may cast the copy without paying its mana cost." The card is
/// linked to the creature (`ExileLinkedTo`), so the granted trigger's
/// `CardExiledWithSource` is the creature's own card.
pub fn massimo_the_magician() -> CardDefinition {
    let spell = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let granted = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::CopyCardAndCastFree { what: Selector::CardExiledWithSource },
    };
    CardDefinition {
        name: "Massimo, the Magician",
        cost: cost(&[u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat, CreatureType::Wizard], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 1,
                filter: spell.and(R::ManaValueExactly(1)).from_your_graveyard(),
                effect: Box::new(Effect::Seq(vec![
                    Effect::ExileLinkedTo { what: Selector::Target(0), link: Selector::TriggerSource },
                    Effect::GrantTriggeredAbility {
                        what: Selector::TriggerSource,
                        trigger: Box::new(granted),
                        duration: Duration::Permanent,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}
