//! Commander: 2026-10-19 (`017ujEs3`) — Wan Shi Tong, All-Knowing's library
//! tuckers (`KeywordAct::CardsPutIntoLibrary`); closes Echo, Perceptive
//! Prodigy's deck too. Tests in `tests/recent_b/cmdr_017u_a.rs`.

use std::sync::Arc;

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Effect, KeywordAct, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{ManaCost, cost, generic, u};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A 1/1 colorless Spirit with "This token can't block or be blocked by
/// non-Spirit creatures."
fn spirit_token(count: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition: Arc::new(TokenDefinition {
            name: "Spirit".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
            keywords: vec![
                Keyword::CantBlockMatching(Box::new(R::HasCreatureType(CreatureType::Spirit).negate())),
                Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Spirit))),
            ],
            ..Default::default()
        }),
    }
}

/// "Whenever one or more cards are put into a library from anywhere" — any
/// library, any cause; a scry or reorder isn't one (Wan Shi Tong's ruling).
fn on_put_into_library(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Performed(KeywordAct::CardsPutIntoLibrary), EventScope::AnyPlayer),
        effect,
    }
}

fn nonland_permanent() -> R {
    R::Permanent.and(R::Nonland)
}

/// Wan Shi Tong, All-Knowing — {3}{U}{U} 4/4 legendary Bird Spirit. Flying.
/// ETB, target nonland permanent's owner puts it second from the top or on the
/// bottom of their library. Whenever one or more cards are put into a library
/// from anywhere, two 1/1 Spirits that can't block or be blocked by non-Spirits.
pub fn wan_shi_tong_all_knowing() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Move {
                what: target_filtered(nonland_permanent()),
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::SecondFromTopOrBottom },
            }),
            on_put_into_library(spirit_token(2)),
        ],
        ..creature(
            "Wan Shi Tong, All-Knowing",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Dutiful Knowledge Seeker — {2}{U} 2/2 Fox Spirit. Whenever one or more cards
/// are put into a library from anywhere, a +1/+1 counter on it. {3}: put target
/// card from a graveyard on the bottom of its owner's library.
pub fn dutiful_knowledge_seeker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_put_into_library(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
            },
            ..Default::default()
        }],
        ..creature(
            "Dutiful Knowledge Seeker",
            cost(&[generic(2), u()]),
            vec![CreatureType::Fox, CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Baboon Spirit — {2}{U} 2/4 Monkey Spirit. Whenever another nontoken Spirit
/// you control enters, a 1/1 Spirit (can't block or be blocked by non-Spirits).
/// {3}{U}: exile another target creature you control; return it under its
/// owner's control at the beginning of the next end step.
pub fn baboon_spirit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Spirit).and(R::NotToken),
                },
            ),
            effect: spirit_token(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::ExileReturnNextEndStep {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            },
            ..Default::default()
        }],
        ..creature("Baboon Spirit", cost(&[generic(2), u()]), vec![CreatureType::Monkey, CreatureType::Spirit], 2, 4)
    }
}

/// Swat Away — {2}{U}{U} instant; {2} less while a creature is attacking you.
/// The owner of target spell or creature puts it on their choice of the top or
/// bottom of their library.
pub fn swat_away() -> CardDefinition {
    CardDefinition {
        name: "Swat Away",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if: Some((
            Predicate::SelectorExists(Selector::EachPermanent(R::Creature.and(R::IsAttackingYou))),
            2,
        )),
        effect: Effect::Move {
            what: target_filtered(R::IsSpellOnStack.or(R::Creature)),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::OwnerChoice },
        },
        ..Default::default()
    }
}

/// Synchronized Eviction — {4}{U} instant; {2} less if you control at least two
/// creatures that share a creature type. Put target nonland permanent into its
/// owner's library second from the top.
pub fn synchronized_eviction() -> CardDefinition {
    CardDefinition {
        name: "Synchronized Eviction",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if: Some((
            Predicate::ValueAtLeast(Value::GreatestSharedCreatureTypeCount, Value::Const(2)),
            2,
        )),
        effect: Effect::Move {
            what: target_filtered(nonland_permanent()),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::FromTop(1) },
        },
        ..Default::default()
    }
}
