//! Commander: 2026-10-25 (`01Y6ZRFx`) — the Spider-Man, Hometown Hero
//! deck's closers. Tests in `tests/recent_b/cmdr_01y6_e.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{ActivationTrait, Duration, Effect, Selector, StaticEffect};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, r};

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn wall() -> R {
    R::HasCreatureType(CreatureType::Wall)
}

/// Corrupt Eunuchs — {3}{R} 2/2 Human Advisor. ETB: 2 damage to target creature.
pub fn corrupt_eunuchs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
        }],
        ..creature("Corrupt Eunuchs", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Advisor], 2, 2)
    }
}

/// Frenzied Raider — {1}{R} 2/2 Demon Berserker. Whenever you activate a
/// boast ability, put a +1/+1 counter on it.
pub fn frenzied_raider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWith(ActivationTrait::Boast), EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Frenzied Raider", cost(&[generic(1), r()]), vec![CreatureType::Demon, CreatureType::Berserker], 2, 2)
    }
}

/// Haunted Guardian — {2} 2/1 Construct artifact creature, defender, first strike.
pub fn haunted_guardian() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender, Keyword::FirstStrike],
        ..creature("Haunted Guardian", cost(&[generic(2)]), vec![CreatureType::Construct], 2, 1)
    }
}

/// Skyclave Sentinel — {3} 2/3 Gargoyle artifact creature, kicker {4},
/// flying, defender. Kicked, it enters with two +1/+1 counters; with a +1/+1
/// counter on it, it can attack as though it didn't have defender.
pub fn skyclave_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Kicker(cost(&[generic(4)])), Keyword::Flying, Keyword::Defender],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(Value::Const(2)), Box::new(Value::TimesKicked)),
        )),
        static_abilities: vec![StaticAbility {
            description: "As long as this creature has a +1/+1 counter on it, it can attack as though it didn't have defender.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::WithCounter(CounterType::PlusOnePlusOne),
                },
            },
        }],
        ..creature("Skyclave Sentinel", cost(&[generic(3)]), vec![CreatureType::Gargoyle], 2, 3)
    }
}

/// Spartan Veteran — {R} 1/1 Human Soldier. First strike during your turn;
/// {2}: +1/+0 until end of turn.
pub fn spartan_veteran() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::FirstStrike }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Spartan Veteran", cost(&[r()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Yellow Scarves Cavalry — {1}{R} 1/1 Human Soldier, horsemanship; can't block.
pub fn yellow_scarves_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship, Keyword::CantBlock],
        ..creature("Yellow Scarves Cavalry", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Dwarven Armory — {2}{R}{R} enchantment. {2}, sacrifice a land: a +2/+2
/// counter on target creature; only during an upkeep step.
pub fn dwarven_armory() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Armory",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Land, 1)),
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusTwoPlusTwo,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Imminent Doom — {2}{R} enchantment, enters with a doom counter. Whenever
/// you cast a spell with mana value equal to its doom counters, it deals that
/// much damage to any target, then gets another doom counter.
pub fn imminent_doom() -> CardDefinition {
    let doom = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Doom };
    CardDefinition {
        name: "Imminent Doom",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Doom, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::ValueEquals(Value::ManaValueOf(Box::new(Selector::TriggerSource)), doom())),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)) },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Doom, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Redcap Melee — {R} instant. 4 damage to target creature or planeswalker;
/// if a nonred permanent is dealt damage this way, you sacrifice a land.
pub fn redcap_melee() -> CardDefinition {
    CardDefinition {
        name: "Redcap Melee",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(4) },
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::ValueAtLeast(Value::DamageDealtThisResolution, Value::ONE),
                    Predicate::EntityMatches { what: Selector::Target(0), filter: R::Not(Box::new(R::HasColor(Color::Red))) },
                ]),
                then: Box::new(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Land }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Blow Your House Down — {2}{R} sorcery. Up to three target creatures can't
/// block this turn; destroy any of them that are Walls.
pub fn blow_your_house_down() -> CardDefinition {
    CardDefinition {
        name: "Blow Your House Down",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: wall() },
                    then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
        ..Default::default()
    }
}
