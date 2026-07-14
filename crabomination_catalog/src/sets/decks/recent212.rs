//! Foundations (FDN) gap batch 11 — a Goblin evasion-granter, a green overrun,
//! Aurelia's extra combat, a spell-punishing Elemental, a Cat counter-payoff,
//! and a Vampire lord. Tests in `tests/recent212.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    Selector, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, EventKind, EventScope, EventSpec, PlayerRef, Predicate, ZoneRef};
use crate::mana::{b, cost, g, generic, r, w, Color};

/// Goblin Smuggler — {2}{R} 2/2 Goblin Rogue. Haste; {T}: Another target
/// creature with power 2 or less can't be blocked this turn.
pub fn goblin_smuggler() -> CardDefinition {
    CardDefinition {
        name: "Goblin Smuggler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Joraga Invocation — {4}{G}{G} Sorcery. Each creature you control gets +3/+3
/// until end of turn and must be blocked this turn if able.
pub fn joraga_invocation() -> CardDefinition {
    CardDefinition {
        name: "Joraga Invocation",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                keyword: Keyword::MustBeBlocked,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Aurelia, the Warleader — {2}{R}{R}{W}{W} 3/4 Legendary Angel. Flying,
/// vigilance, haste; whenever Aurelia attacks for the first time each turn,
/// untap all creatures you control. After this phase, there is an additional
/// combat phase.
pub fn aurelia_the_warleader() -> CardDefinition {
    CardDefinition {
        name: "Aurelia, the Warleader",
        cost: cost(&[generic(2), r(), r(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    up_to: None,
                },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Mindsparker — {1}{R}{R} 3/2 Elemental. First strike; whenever an opponent
/// casts a white or blue instant or sorcery spell, this creature deals 2 damage
/// to that player.
pub fn mindsparker() -> CardDefinition {
    CardDefinition {
        name: "Mindsparker",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::White)
                        .or(R::HasColor(Color::Blue))
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Ingenious Leonin — {4}{W} 4/4 Cat Soldier. {3}{W}: Put a +1/+1 counter on
/// another target attacking creature you control. If that creature is a Cat, it
/// gains first strike until end of turn.
pub fn ingenious_leonin() -> CardDefinition {
    CardDefinition {
        name: "Ingenious Leonin",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::IsAttacking)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::HasCreatureType(CreatureType::Cat),
                    },
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crossway Troublemakers — {5}{B} 5/5 Vampire. Attacking Vampires you control
/// have deathtouch and lifelink. Whenever a Vampire you control dies, you may
/// pay 2 life. If you do, draw a card.
pub fn crossway_troublemakers() -> CardDefinition {
    CardDefinition {
        name: "Crossway Troublemakers",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 5,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Attacking Vampires you control have deathtouch and lifelink.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Vampire).and(R::IsAttacking),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Vampire),
                },
            ),
            effect: Effect::MayDo {
                description: "pay 2 life to draw a card".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
            },
        }],
        ..Default::default()
    }
}
