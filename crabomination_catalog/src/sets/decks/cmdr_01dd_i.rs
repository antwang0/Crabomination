//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Bill Ferny, Bree
//! Swindler's Horses. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{mint_treasures, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, u, x};

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

/// Bill Ferny, Bree Swindler — {1}{U} 2/1 legendary Human Rogue. Whenever it
/// becomes blocked, choose one: a Treasure; or target opponent gains control
/// of target Horse you control, then Bill Ferny leaves combat and you create
/// three Treasures.
pub fn bill_ferny_bree_swindler() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                mint_treasures(1),
                Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::HasCreatureType(CreatureType::Horse).and(R::ControlledByYou),
                        },
                        to: Some(PlayerRef::Target(0)),
                        duration: Duration::Permanent,
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::Target(1),
                            filter: R::Not(Box::new(R::ControlledByYou)),
                        },
                        then: Box::new(Effect::Seq(vec![
                            Effect::RemoveFromCombat { what: Selector::This },
                            mint_treasures(3),
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            ]),
        }],
        ..creature(
            "Bill Ferny, Bree Swindler",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Phantasmal Mount — {1}{U} 1/1 Illusion Horse, flying. {T}: target creature
/// you control with toughness 2 or less gets +1/+1 and flying until end of
/// turn; whichever of the two leaves the battlefield this turn, the other is
/// sacrificed.
pub fn phantasmal_mount() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::ToughnessAtMost(2))),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                Effect::SacrificeEachWhenOtherLeavesThisTurn { a: Selector::This, b: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature("Phantasmal Mount", cost(&[generic(1), u()]), vec![CreatureType::Illusion, CreatureType::Horse], 1, 1)
    }
}

/// Surge Mare — {U}{U} 0/5 Horse Fish. Can't be blocked by green creatures.
/// Whenever it deals damage to an opponent, you may draw a card; if you do,
/// discard a card. {1}{U}: +2/-2 until end of turn.
pub fn surge_mare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasColor(Color::Green)))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::MayDo {
                description: "Draw a card, then discard a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Surge Mare", cost(&[u(), u()]), vec![CreatureType::Horse, CreatureType::Fish], 0, 5)
    }
}

/// Selective Snare — {X}{U} sorcery. Return X target creatures of the
/// creature type of your choice to their owners' hands. The choice is read
/// off the targets: each must share a creature type with the first.
pub fn selective_snare() -> CardDefinition {
    CardDefinition {
        name: "Selective Snare",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 10,
                filter: R::Creature.and(R::SharesCreatureTypeWithTargetSlot(0)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            }),
        },
        ..Default::default()
    }
}
