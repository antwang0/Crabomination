//! MKM (Murders at Karlov Manor) split cards. `CardDefinition.split` carries
//! the right half; the left half lives on the parent. Tests in
//! `tests/recent_b/recent258.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, SplitCard,
    SplitHalf, Subtypes, TokenDefinition,
};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{Color, cost, generic, hybrid};

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Fuss // Bother — {2}{R/W} Instant // {4}{W/U}{W/U} Sorcery. Fuss puts a
/// +1/+1 counter on each attacking creature you control; Bother makes three
/// Thopters and surveils 2.
pub fn fuss_bother() -> CardDefinition {
    let rw = hybrid(Color::Red, Color::White);
    let wu = hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Fuss // Bother",
        cost: cost(&[generic(2), rw]),
        card_types: vec![CardType::Instant],
        effect: Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou)),
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), wu, wu]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(3),
                        definition: Box::new(thopter_token()),
                    },
                    Effect::Surveil {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Cease // Desist — {1}{B/G} Instant // {4}{G/W}{G/W} Sorcery. Cease exiles up
/// to two cards from a single graveyard and a target player gains 2 life and
/// draws; Desist destroys all artifacts and enchantments.
pub fn cease_desist() -> CardDefinition {
    let bg = hybrid(Color::Black, Color::Green);
    let gw = hybrid(Color::Green, Color::White);
    CardDefinition {
        name: "Cease // Desist",
        cost: cost(&[generic(1), bg]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileUpToNFromGraveyards {
                count: Value::Const(2),
                of: None,
                single: true,
            },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), gw, gw]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(R::Artifact.or(R::Enchantment)),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}
