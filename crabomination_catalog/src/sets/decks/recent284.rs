//! Bloomburrow permanent **Gift** cards (CR 702.165) on the new
//! `Predicate::SourceGiftPromised` gate: the printed effect only fires when the
//! gift was promised at cast. Tests in `tests/recent_b/recent284.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Gift, Keyword, Predicate,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility, Value};
use crate::mana::{cost, g, generic};

/// Scrapshooter — {1}{G}{G} 4/4 Raccoon Archer. Reach. Gift a card. When it
/// enters, if the gift was promised, that opponent draws a card and you destroy
/// target artifact or enchantment an opponent controls.
pub fn scrapshooter() -> CardDefinition {
    CardDefinition {
        name: "Scrapshooter",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Archer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        // Creatures don't resolve a `gifted_effect`; the gift enables the
        // promise UI and the payload is folded into the gift-gated ETB.
        gift: Some(Box::new(Gift { label: "a card", gifted_effect: Effect::Noop })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceGiftPromised),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::Destroy {
                    what: target_filtered(
                        (R::Artifact.or(R::Enchantment)).and(R::ControlledByOpponent),
                    ),
                },
            ]),
        }],
        ..Default::default()
    }
}
