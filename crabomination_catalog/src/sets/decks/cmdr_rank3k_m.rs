//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), thirteenth file. Tests in `tests/recent_b/cmdr_rank3k_m.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered, training};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
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

// ── Sifa Grent ───────────────────────────────────────────────────────────────

/// Sifa Grent — {3}{B}{R} legendary planeswalker, loyalty 4; can be your
/// commander. Whenever a creature dies, target opponent loses 1 life and you
/// gain 1. +1: goad up to two target creatures. −6: each opponent exiles the
/// top two cards of their library; until end of turn you may play those cards
/// without paying their mana costs.
pub fn sifa_grent() -> CardDefinition {
    let loyalty = |cost: i32, effect: Effect| crate::card::LoyaltyAbility {
        loyalty_cost: cost,
        effect,
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Sifa],
            ..Default::default()
        },
        can_be_commander: true,
        base_loyalty: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: target_filtered(R::Player.and(R::ControlledByOpponent)),
                    amount: Value::ONE,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Goad {
                        what: Selector::Target(0),
                    }),
                },
            ),
            loyalty(
                -6,
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(2),
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
            ),
        ],
        ..spell(
            "Sifa Grent",
            cost(&[generic(3), b(), r()]),
            CardType::Planeswalker,
            Effect::Noop,
        )
    }
}

/// Seifer, Balamb Rival — {2}{B}{R} legendary 4/3 first strike Human
/// Mercenary. Whenever you attack a player, goad target creature that player
/// controls. Whenever a creature attacking one of your opponents becomes
/// blocked by two or more creatures, it gains deathtouch until end of turn.
pub fn seifer_balamb_rival() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YouAttackedPlayer),
                effect: Effect::Goad {
                    what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                },
            },
            TriggeredAbility {
                // Any creature attacking one of your opponents, not only yours.
                event: EventSpec::new(EventKind::BecomesBlockedByNOrMore(2), EventScope::AnyPlayer).with_filter(
                    crate::card::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsAttackingOpponentPlayer,
                    },
                ),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Seifer, Balamb Rival",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            4,
            3,
        )
    }
}

// ── Nivea, Beloved Battlemage ────────────────────────────────────────────────

/// Nivea, Beloved Battlemage — {2}{W} legendary 3/1 Human Wizard Warrior.
/// Whenever a creature you control becomes blocked, you may untap it and
/// remove it from combat. {5}{W}{W}{W}, sacrifice Nivea: create an Akroma,
/// Angel of Wrath token ({5}{W}{W}{W}, mana value 8 — CR 111.3).
pub fn nivea_beloved_battlemage() -> CardDefinition {
    let akroma = TokenDefinition {
        name: "Akroma, Angel of Wrath".into(),
        cost: cost(&[generic(5), w(), w(), w()]),
        power: 6,
        toughness: 6,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::Flying,
            Keyword::FirstStrike,
            Keyword::Vigilance,
            Keyword::Trample,
            Keyword::Haste,
            Keyword::Protection(Color::Black),
            Keyword::Protection(Color::Red),
        ],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Untap the blocked creature and remove it from combat?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Untap {
                        what: Selector::TriggerSource,
                        up_to: None,
                    },
                    Effect::RemoveFromCombat {
                        what: Selector::TriggerSource,
                    },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w(), w(), w()]),
            sac_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(akroma),
            },
            ..Default::default()
        }],
        ..legend(
            "Nivea, Beloved Battlemage",
            cost(&[generic(2), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Wizard,
                CreatureType::Warrior,
            ],
            3,
            1,
        )
    }
}

/// Warrior's Resolve — {2}{W} enchantment. Creatures you control have
/// training. At the beginning of your end step, if you control a creature with
/// a +1/+1 counter on it that attacked this turn, draw a card.
pub fn warriors_resolve() -> CardDefinition {
    let veteran = R::Creature
        .and(R::ControlledByYou)
        .and(R::WithCounter(CounterType::PlusOnePlusOne))
        .and(R::AttackedThisTurn);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have training.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou),
                ability: Box::new(training()),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorExists(Selector::EachPermanent(veteran))),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..spell(
            "Warrior's Resolve",
            cost(&[generic(2), w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}
