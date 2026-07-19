//! Bloomburrow permanent **Gift** cards (CR 702.165) on the new
//! `Predicate::SourceGiftPromised` gate: an ETB clause keys on whether the gift
//! was promised at cast (Scrapshooter fires only when it was; Kitnap's stun
//! only when it wasn't). Tests in `tests/recent_b/recent284.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, Gift, Keyword,
    Predicate, SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility, Value};
use crate::mana::{cost, g, generic, u};

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

/// Kitnap — {2}{U}{U} Aura. Gift a card. Enchant creature; you control it. When
/// it enters, tap enchanted creature; if the gift wasn't promised, put three
/// stun counters on it (and if it was, the opponent drew a card).
pub fn kitnap() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Kitnap",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        gift: Some(Box::new(Gift { label: "a card", gifted_effect: Effect::Noop })),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControlWhileSourceRemains { what: enchanted() },
            Effect::Tap { what: enchanted() },
            Effect::If {
                cond: Predicate::SourceGiftPromised,
                then: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::AddCounter {
                    what: enchanted(),
                    kind: CounterType::Stun,
                    amount: Value::Const(3),
                }),
            },
        ]))],
        ..Default::default()
    }
}
