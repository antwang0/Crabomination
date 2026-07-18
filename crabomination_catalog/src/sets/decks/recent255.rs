//! MKM (Murders at Karlov Manor) Case enchantments — second batch.
//! Cases ride `CardDefinition.case` (`CaseData.to_solve` at the end step,
//! `solved_*` once solved). Tests in `tests/recent_b/recent255.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::cast_is_instant_or_sorcery;
use crate::effect::{
    Effect, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{b, cost, generic, u, w};

fn case_subtypes() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Case], ..Default::default() }
}

fn is_instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Case of the Ransacked Lab — {2}{U} Enchantment — Case. Your instants and
/// sorceries cost {1} less. Solve: cast 4+ instants/sorceries this turn. Solved:
/// whenever you cast an instant or sorcery, draw a card.
pub fn case_of_the_ransacked_lab() -> CardDefinition {
    CardDefinition {
        name: "Case of the Ransacked Lab",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: is_instant_or_sorcery(), amount: 1 },
        }],
        case: Some(Box::new(crate::card::CaseData {
            to_solve: Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(4),
            },
            solved_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_instant_or_sorcery()),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Stashed Skeleton — {1}{B} Enchantment — Case. ETB: create a 2/1
/// black Skeleton token and suspect it. Solve: you control no suspected
/// Skeletons. Solved: {1}{B}, Sacrifice: tutor any card to hand (sorcery speed).
pub fn case_of_the_stashed_skeleton() -> CardDefinition {
    CardDefinition {
        name: "Case of the Stashed Skeleton",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Skeleton".into(),
                    colors: vec![crate::mana::Color::Black],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Skeleton],
                        ..Default::default()
                    },
                    power: 2,
                    toughness: 1,
                    ..Default::default()
                },
            },
            Effect::Suspect { what: Selector::LastCreatedToken },
        ]))],
        case: Some(Box::new(crate::card::CaseData {
            to_solve: Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Skeleton)
                        .and(R::IsSuspected)
                        .and(R::ControlledByYou),
                ),
                n: Value::ONE,
            })),
            solved_activated: vec![ActivatedAbility {
                sac_cost: true,
                sorcery_speed: true,
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Any,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Case of the Pilfered Proof — {1}{W} Enchantment — Case. Whenever a Detective
/// you control enters or is turned face up, put a +1/+1 counter on it. Solve:
/// control 3+ Detectives. Solved: your token creations also mint a Clue.
pub fn case_of_the_pilfered_proof() -> CardDefinition {
    let counter_on_detective = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl).with_filter(Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: R::HasCreatureType(CreatureType::Detective),
        }),
        effect: Effect::AddCounter {
            what: Selector::TriggerSource,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    };
    CardDefinition {
        name: "Case of the Pilfered Proof",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: case_subtypes(),
        triggered_abilities: vec![
            counter_on_detective(EventKind::EntersBattlefield),
            counter_on_detective(EventKind::TurnedFaceUp),
        ],
        case: Some(Box::new(crate::card::CaseData {
            to_solve: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Detective).and(R::ControlledByYou),
                ),
                n: Value::Const(3),
            },
            solved_static: vec![StaticAbility {
                description: "If one or more tokens would be created under your control, \
                              those tokens plus a Clue token are created instead.",
                effect: StaticEffect::TokenCreationAddsToken {
                    definition: crabomination_base::tokens::clue_token(),
                },
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}
