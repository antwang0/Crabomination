//! Commander: two-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), eighth file. Tests in `tests/recent_b/cmdr_rank4k_h.rs`.

use crate::card::{
    ActivatedAbility, Adventure, AlternativeCost, CardDefinition, CardType,
    CounterType, CreatureType, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivationTrait, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};
use std::sync::Arc;

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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn ping(to: Selector) -> Effect {
    Effect::DealDamage {
        to,
        amount: Value::ONE,
    }
}

// ── Tor Wauki ────────────────────────────────────────────────────────────────

/// Blood Cultist — {1}{B}{R} 1/1 Human Wizard. {T}: 1 damage to target
/// creature. Whenever a creature dealt damage by it this turn dies, a +1/+1
/// counter on it.
pub fn blood_cultist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: ping(target_filtered(R::Creature)),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Blood Cultist",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Gideon the Oathless — {2}{B} legendary 3/3 Human Mercenary; ward—discard a
/// card. Whenever a creature an opponent controls enters, 1 damage to that
/// player. Whenever an opponent activates a loyalty ability, 1 damage to that
/// player.
pub fn gideon_the_oathless() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Discard(1))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: ping(Selector::Player(PlayerRef::ControllerOf(Box::new(
                    Selector::TriggerSource,
                )))),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::AbilityActivatedWith(ActivationTrait::Loyalty),
                    EventScope::OpponentControl,
                ),
                effect: ping(Selector::Player(PlayerRef::Triggerer)),
            },
        ],
        ..legend(
            "Gideon the Oathless",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

// ── Arachne, Psionic Weaver ──────────────────────────────────────────────────

/// Arachne, Psionic Weaver — {2}{W} legendary 3/3 Spider Human Hero;
/// web-slinging {W}. As she enters, choose a card type other than creature;
/// spells of the chosen type cost {1} more. (Residual: the look at an
/// opponent's hand that informs the choice isn't modelled.)
pub fn arachne_psionic_weaver() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[w()]),
            return_to_hand: Some((R::Creature.and(R::Tapped), 1)),
            ..Default::default()
        }),
        as_enters_effect: Some(Effect::ChooseCardTypeFromForSource {
            options: vec![
                CardType::Instant,
                CardType::Sorcery,
                CardType::Artifact,
                CardType::Enchantment,
                CardType::Planeswalker,
            ],
        }),
        static_abilities: vec![StaticAbility {
            description: "Spells of the chosen type cost {1} more to cast.",
            effect: StaticEffect::ChosenCardTypeSpellsCostMore { amount: 1 },
        }],
        ..legend(
            "Arachne, Psionic Weaver",
            cost(&[generic(2), w()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            3,
            3,
        )
    }
}

/// The Arkenstone // Seek the Heart — {5} legendary artifact. Creatures you
/// control get +1/+1. At the beginning of your end step, draw a card.
/// Adventure ({2}{W} sorcery): search your library for a legendary creature
/// card, reveal it, put it into your hand, then shuffle.
pub fn the_arkenstone() -> CardDefinition {
    CardDefinition {
        name: "The Arkenstone",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Seek the Heart",
            cost: cost(&[generic(2), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        })),
        ..Default::default()
    }
}

// ── Captain Mar-Vell, Space-Born ─────────────────────────────────────────────

/// Captain Marvel, Shooting Star — {5}{W}{W} legendary 6/6 flying Human Kree
/// Hero. Enters or attacks: exile up to one target creature; its controller
/// gains life equal to its power. Whenever another creature is exiled from
/// the battlefield, you gain life equal to its power.
pub fn captain_marvel_shooting_star() -> CardDefinition {
    let exile = || Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: R::Creature,
        effect: Box::new(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::Exile {
                what: Selector::Target(0),
            },
        ])),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(exile()),
            on_attack(exile()),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CardExiledFrom(crate::effect::exile_from::BATTLEFIELD),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..legend(
            "Captain Marvel, Shooting Star",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Human, CreatureType::Kree, CreatureType::Hero],
            6,
            6,
        )
    }
}

/// Celebrate the Mountain-king — {3}{W} enchantment. ETB: for each opponent,
/// exile up to one target nonland permanent that player controls until it
/// leaves. ETB: recruit.
pub fn celebrate_the_mountain_king() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Celebrate the Mountain-king",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 5,
                    min_targets: 0,
                    filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                    effect: Box::new(Effect::ExileUntilSourceLeaves {
                        what: Selector::Target(0),
                        return_to: ExileReturnZone::Battlefield,
                    }),
                }),
            }),
            etb(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::If {
                    cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(soldier),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        ],
        ..Default::default()
    }
}
