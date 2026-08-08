//! Bloomburrow gap batch — an expend Raccoon lord, and two Gift spells (a
//! flicker-removal instant and a gift wrath). Tests in
//! `tests/recent_b/recent285.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Gift, SelectionRequirement as R, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, cost, generic, w};

/// A tapped 1/1 blue Fish — the Bloomburrow blue gift token.
fn tapped_fish_token() -> TokenDefinition {
    TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    }
}

/// Parting Gust — {W}{W} Instant. Gift a tapped Fish. Exile target nontoken
/// creature. If the gift wasn't promised, return that card with a +1/+1 counter
/// at the beginning of the next end step.
pub fn parting_gust() -> CardDefinition {
    CardDefinition {
        name: "Parting Gust",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Instant],
        // No gift: exile-and-return (with a +1/+1 counter) at the next end step.
        effect: Effect::ExileReturnNextEndStep {
            what: target_filtered(R::Creature.and(R::NotToken)),
        },
        gift: Some(Box::new(Gift {
            label: "a tapped Fish",
            // Gift promised: opponent gets a tapped Fish; the creature is exiled
            // for good (no return).
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::ONE,
                    definition: Box::new(tapped_fish_token()),
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::NotToken)),
                    to: ZoneDest::Exile,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Starfall Invocation — {3}{W}{W} Sorcery. Gift a card. Destroy all creatures;
/// if the gift was promised, return a creature card from your graveyard to the
/// battlefield.
pub fn starfall_invocation() -> CardDefinition {
    CardDefinition {
        name: "Starfall Invocation",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DestroyNoRegen {
            what: Selector::EachPermanent(R::Creature),
        },
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                Effect::DestroyNoRegen {
                    what: Selector::EachPermanent(R::Creature),
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        })),
        ..Default::default()
    }
}
