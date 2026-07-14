//! WOE/OTJ gap batch on existing primitives: Rowdy Research (attacker-affinity
//! draw), Brave the Wilds (Bargain land-animate + land tutor), and Redrock
//! Sentinel (sac-a-land value engine). Tests in
//! `crabomination/src/tests/recent190.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{cost, g, generic, u};

/// Rowdy Research — {6}{U} Instant. Costs {1} less for each creature that
/// attacked this turn. Draw three cards.
pub fn rowdy_research() -> CardDefinition {
    CardDefinition {
        name: "Rowdy Research",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(R::AttackedThisTurn),
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Brave the Wilds — {G} Sorcery. Bargain. If bargained, target land you control
/// becomes a 3/3 Elemental with haste that's still a land. Search your library
/// for a basic land card and put it into your hand, then shuffle.
pub fn brave_the_wilds() -> CardDefinition {
    CardDefinition {
        name: "Brave the Wilds",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Bargain],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::SpellWasBargained,
                then: Box::new(Effect::BecomeCreature {
                    what: Selector::Target(0),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::Permanent,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Redrock Sentinel — {3} 2/4 Golem with defender. {2}, {T}, Sacrifice a land:
/// Draw a card and create a Treasure token.
pub fn redrock_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Redrock Sentinel",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::treasure_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
