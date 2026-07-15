//! Gap batch — OTJ/BLB spells & value creatures on existing primitives.
//! Tests in `tests/recent231.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, g, generic, r, u};

/// Volcanic Spite — {1}{R} Instant. Deals 3 damage to target creature,
/// planeswalker, or battle. You may put a card from your hand on the bottom of
/// your library. If you do, draw a card.
pub fn volcanic_spite() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Spite",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    R::Creature.or(R::Planeswalker).or(R::HasCardType(CardType::Battle)),
                ),
                amount: Value::Const(3),
            },
            Effect::MayDo {
                description: "Bottom a card from your hand, then draw a card?".into(),
                body: Box::new(Effect::BottomChosenFromHandAndDraw {
                    from: Selector::You,
                    count: Value::ONE,
                    filter: R::Any,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Lilysplash Mentor — {2}{G}{U} 4/4 Frog Druid. Reach. {1}{G}{U}: Exile
/// another target creature you control, then return it to the battlefield under
/// its owner's control with a +1/+1 counter on it. Activate only as a sorcery.
pub fn lilysplash_mentor() -> CardDefinition {
    CardDefinition {
        name: "Lilysplash Mentor",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), u()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
