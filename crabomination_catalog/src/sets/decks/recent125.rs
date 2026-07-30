//! An OTJ Mounts batch (Saddle, CR 702.171) plus a kicker-reanimator and an
//! outlaw-scaled combat trick. Tests in `tests/recent125.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{attacks_while_saddled, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, cost, g, generic, w};

/// A 1/1 white Sheep — Bridled Bighorn's saddled-attack token.
fn sheep_token() -> TokenDefinition {
    TokenDefinition {
        name: "Sheep".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sheep],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Bridled Bighorn — {3}{W} 3/4 Sheep Mount with vigilance. Saddle 2; whenever
/// it attacks while saddled, create a 1/1 white Sheep.
pub fn bridled_bighorn() -> CardDefinition {
    CardDefinition {
        name: "Bridled Bighorn",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sheep, CreatureType::Mount],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Saddle(2)],
        triggered_abilities: vec![attacks_while_saddled(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: sheep_token(),
        })],
        ..Default::default()
    }
}

/// Drover Grizzly — {2}{G} 4/2 Bear Mount. Saddle 1; whenever it attacks while
/// saddled, creatures you control gain trample until end of turn.
pub fn drover_grizzly() -> CardDefinition {
    CardDefinition {
        name: "Drover Grizzly",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Mount],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![attacks_while_saddled(Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Sun-Blessed Healer — {1}{W} 3/1 Human Cleric with lifelink and kicker {1}{W}.
/// ETB, if kicked: return target nonland permanent card with mana value 2 or
/// less from your graveyard to the battlefield.
pub fn sun_blessed_healer() -> CardDefinition {
    CardDefinition {
        name: "Sun-Blessed Healer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Lifelink, Keyword::Kicker(cost(&[generic(1), w()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::ManaValueAtMost(2))),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}
