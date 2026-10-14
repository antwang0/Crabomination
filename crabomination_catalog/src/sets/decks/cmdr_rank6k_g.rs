//! Commander: one-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), seventh file. Tests in `tests/recent_b/cmdr_rank6k_g.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest, ZoneRef};
use crate::mana::{cost, generic, r, w};
use std::sync::Arc;

/// Island Sanctuary — {1}{W} enchantment. If you would draw a card during
/// your draw step, you may skip that draw instead; if you do, until your
/// next turn you can't be attacked except by creatures with flying and/or
/// islandwalk (CR 614.1a, `game::draw_skip_shield`).
pub fn island_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Island Sanctuary",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card during your draw step, instead you may skip that draw. If you do, until your next turn, you can't be attacked except by creatures with flying and/or islandwalk.",
            effect: StaticEffect::ControllerMaySkipDrawStepDrawForAttackShield {
                attackers: R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Landwalk(LandType::Island))),
            },
        }],
        ..Default::default()
    }
}

/// You've Been Caught Stealing — {1}{R} sorcery. Choose one: each creature
/// blocks this turn if able; or a Treasure for each opponent who was dealt
/// damage this turn.
pub fn youve_been_caught_stealing() -> CardDefinition {
    CardDefinition {
        name: "You've Been Caught Stealing",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::OpponentsDealtDamageThisTurn,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Livio, Oathsworn Sentinel — {1}{W} 2/2 legendary Human Knight, partner.
/// {1}{W}: choose another target creature; its controller may exile it with
/// an aegis counter on it. {2}{W}, {T}: return all exiled cards with aegis
/// counters to the battlefield under their owners' control. (Residual: the
/// return is one card at a time rather than simultaneous, CR 610.3.)
pub fn livio_oathsworn_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Livio, Oathsworn Sentinel",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::MayDoBy {
                    who: PlayerRef::ControllerOf(Box::new(target_filtered(R::Creature.and(R::OtherThanSource)))),
                    description: "Exile your creature with an aegis counter on it (Livio)?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                        Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Aegis, amount: Value::ONE },
                    ])),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                tap_cost: true,
                effect: Effect::ForEach {
                    selector: Selector::EachMatching { zone: ZoneRef::Exile, filter: R::WithCounter(CounterType::Aegis) },
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                            tapped: false,
                        },
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
