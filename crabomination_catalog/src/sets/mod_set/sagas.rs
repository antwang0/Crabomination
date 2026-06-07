//! Saga enchantments (CR 714) — chapter abilities tick off lore counters.
//!
//! Each card sets `saga_chapters: vec![(chapter, effect), …]`; the engine
//! adds the first lore counter on ETB and one more each precombat main,
//! firing the matching chapter, then sacrifices the Saga once the final
//! chapter resolves.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EnchantmentSubtype, Keyword,
    SelectionRequirement, Selector, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{mint_token, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, generic, w};

fn saga_subtypes() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![EnchantmentSubtype::Saga],
        ..Default::default()
    }
}

/// History of Benalia — {1}{W}{W} Saga. I, II — create a 2/2 white Knight
/// with vigilance. III — Knights you control get +2/+1 until end of turn.
pub fn history_of_benalia() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Knight],
            ..Default::default()
        },
        ..Default::default()
    };
    let mint = mint_token(knight, 1);
    CardDefinition {
        name: "History of Benalia",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga_subtypes(),
        saga_chapters: vec![
            (1, mint.clone()),
            (2, mint),
            (
                3,
                Effect::PumpPT {
                    what: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Knight)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(2),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..Default::default()
    }
}

/// The Birth of Meletis — {1}{W} Saga. I — search for a basic Plains to hand.
/// II — create a 0/4 colorless Wall artifact creature with defender. III —
/// gain 2 life.
pub fn the_birth_of_meletis() -> CardDefinition {
    use crate::card::{LandType, Supertype};
    let wall = TokenDefinition {
        name: "Wall".into(),
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..Default::default()
    };
    CardDefinition {
        name: "The Birth of Meletis",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga_subtypes(),
        saga_chapters: vec![
            (
                1,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasLandType(LandType::Plains)
                        .and(SelectionRequirement::HasSupertype(Supertype::Basic)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
            (2, mint_token(wall, 1)),
            (3, Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
        ],
        ..Default::default()
    }
}

/// Triumph of Gerrard — {1}{W} Saga. I, II — +1/+1 counter on the creature
/// you control with the greatest power. III — that creature gains flying,
/// first strike, and lifelink until end of turn.
pub fn triumph_of_gerrard() -> CardDefinition {
    let grant = |kw: Keyword| Effect::GrantKeyword {
        what: Selector::GreatestPowerYouControl,
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Triumph of Gerrard",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga_subtypes(),
        saga_chapters: vec![
            (1, plus_one_greatest()),
            (2, plus_one_greatest()),
            (
                3,
                Effect::Seq(vec![
                    grant(Keyword::Flying),
                    grant(Keyword::FirstStrike),
                    grant(Keyword::Lifelink),
                ]),
            ),
        ],
        ..Default::default()
    }
}

fn plus_one_greatest() -> Effect {
    Effect::AddCounter {
        what: Selector::GreatestPowerYouControl,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    }
}

/// The Eldest Reborn — {4}{B} Saga. I — each opponent sacrifices a creature
/// or planeswalker. II — each opponent discards a card. III — put target
/// creature or planeswalker card from a graveyard onto the battlefield under
/// your control.
pub fn the_eldest_reborn() -> CardDefinition {
    let creature_or_pw =
        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker);
    CardDefinition {
        name: "The Eldest Reborn",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: saga_subtypes(),
        saga_chapters: vec![
            (
                1,
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    filter: creature_or_pw.clone(),
                    count: Value::Const(1),
                },
            ),
            (
                2,
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: false,
                },
            ),
            (
                3,
                Effect::Move {
                    what: target_filtered(creature_or_pw),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ),
        ],
        ..Default::default()
    }
}
