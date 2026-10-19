//! Commander: 2026-10-19 (`016Xh16H`) — Beluna Grandsquall's Adventure
//! package (Gorion's shared cards live in `cmdr_01gw_f`). Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype,
};
use crate::effect::{Effect, Selector, Value};
use crate::mana::{ManaCost, cost, g, generic, r, u};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}


/// Beluna Grandsquall — {G}{U}{R} 4/4 legendary trample Giant Noble.
/// Permanent spells you cast that have an Adventure cost {1} less. Adventure
/// Seek Thrills {2}{G}{U}{R} instant: mill seven, then put every milled card
/// that has an Adventure into your hand.
pub fn beluna_grandsquall() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Permanent spells you cast that have an Adventure cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::HasAdventure.and(R::PermanentCard), amount: 1 },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Seek Thrills",
            cost: cost(&[generic(2), g(), u(), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::MillThenToHandN {
                amount: Value::Const(7),
                filter: R::HasAdventure,
                take: Value::Const(7),
                otherwise: None,
            },
        })),
        ..creature(
            "Beluna Grandsquall",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Giant, CreatureType::Noble],
            4,
            4,
        )
    }
}





/// Hearth Elemental — {5}{R} 4/5 Elemental; costs {1} less per instant,
/// sorcery or Adventure card in your graveyard. Adventure Stoke Genius {1}{R}
/// sorcery: discard your hand, then draw two.
pub fn hearth_elemental() -> CardDefinition {
    CardDefinition {
        affinity_graveyard_filter: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).or(R::HasAdventure)),
        adventure: Some(Box::new(Adventure {
            name: "Stoke Genius",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(100), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ]),
        })),
        ..creature("Hearth Elemental", cost(&[generic(5), r()]), vec![CreatureType::Elemental], 4, 5)
    }
}
