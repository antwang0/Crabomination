//! The four remaining Amonkhet Monuments (Bontu's already ships in `modern`).
//! Each is a {3} Legendary Artifact reducing its color's creature spells by {1}
//! with a "whenever you cast a creature spell" rider, mirroring Bontu's
//! Monument. Tests in `tests/recent38.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{discard, draw, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate};
use crate::mana::{Color, cost, generic};

/// A {3} Legendary Monument: `color` creature spells cost {1} less, plus a
/// "whenever you cast a creature spell, `rider`" trigger.
fn monument(
    name: &'static str,
    color: Color,
    reduce_desc: &'static str,
    rider: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: reduce_desc,
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::Creature.and(SelectionRequirement::HasColor(color)),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                },
            ),
            effect: rider,
        }],
        ..Default::default()
    }
}

/// Oketra's Monument — White creature spells cost {1} less; whenever you cast a
/// creature spell, create a 1/1 white Warrior with vigilance.
pub fn oketras_monument() -> CardDefinition {
    let warrior = TokenDefinition {
        name: "Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Warrior],
            ..Default::default()
        },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    monument(
        "Oketra's Monument",
        Color::White,
        "White creature spells you cast cost {1} less to cast.",
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: warrior,
        },
    )
}

/// Kefnet's Monument — Blue creature spells cost {1} less; whenever you cast a
/// creature spell, target creature an opponent controls doesn't untap during
/// its controller's next untap step.
pub fn kefnets_monument() -> CardDefinition {
    monument(
        "Kefnet's Monument",
        Color::Blue,
        "Blue creature spells you cast cost {1} less to cast.",
        Effect::SkipNextUntap {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        },
    )
}

/// Hazoret's Monument — Red creature spells cost {1} less; whenever you cast a
/// creature spell, you may discard a card. If you do, draw a card.
pub fn hazorets_monument() -> CardDefinition {
    monument(
        "Hazoret's Monument",
        Color::Red,
        "Red creature spells you cast cost {1} less to cast.",
        Effect::MayDo {
            description: "Discard a card, then draw a card.".into(),
            body: Box::new(Effect::Seq(vec![discard(Selector::You, 1, false), draw(1)])),
        },
    )
}

/// Rhonas's Monument — Green creature spells cost {1} less; whenever you cast a
/// creature spell, target creature you control gets +2/+2 and gains trample
/// until end of turn.
pub fn rhonass_monument() -> CardDefinition {
    monument(
        "Rhonas's Monument",
        Color::Green,
        "Green creature spells you cast cost {1} less to cast.",
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}
