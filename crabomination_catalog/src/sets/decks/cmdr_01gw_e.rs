//! Commander: 2026-10-19 (`01GwrDKP`) closers, fifth batch (5-7-gap tier) —
//! Gorbag of Minas Morgul's Goblins, The Motherlode, Excavator's energy. Tests
//! in `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, Predicate, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, r, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CardType>, subtypes: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: types,
        subtypes: Subtypes { creature_types: subtypes, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn goblin() -> R {
    R::HasCreatureType(CreatureType::Goblin)
}

fn energy(n: i32) -> Effect {
    Effect::AddEnergy(Value::Const(n))
}

/// "Whenever this creature attacks, you may pay {E}×`n`. If you do, [then]."
fn attack_pay_energy(n: u32, then: Effect) -> crate::card::TriggeredAbility {
    on_attack(Effect::MayDo {
        description: format!("Pay {n} energy?"),
        body: Box::new(Effect::PayEnergy { amount: n, then: Box::new(then) }),
    })
}

fn self_counter() -> Effect {
    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

/// Bile-Vial Boggart — {B} 1/1 Goblin Assassin. When it dies, a -1/-1 counter
/// on up to one target creature.
pub fn bile_vial_boggart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            }),
        })],
        ..creature("Bile-Vial Boggart", cost(&[b()]), vec![CardType::Creature], vec![CreatureType::Goblin, CreatureType::Assassin], 1, 1)
    }
}

/// Facevaulter — {B} 1/1 Goblin Warrior. {B}, sacrifice a Goblin: +2/+2 until
/// end of turn.
pub fn facevaulter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((goblin(), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Facevaulter", cost(&[b()]), vec![CardType::Creature], vec![CreatureType::Goblin, CreatureType::Warrior], 1, 1)
    }
}

/// Shambling Goblin — {B} 1/1 Zombie Goblin. When it dies, target creature an
/// opponent controls gets -1/-1 until end of turn.
pub fn shambling_goblin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Shambling Goblin", cost(&[b()]), vec![CardType::Creature], vec![CreatureType::Zombie, CreatureType::Goblin], 1, 1)
    }
}

/// Squeaking Pie Sneak — {1}{B} 2/2 fear Goblin Rogue. As an additional cost,
/// reveal a Goblin card from your hand or pay {3}.
pub fn squeaking_pie_sneak() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::RevealFromHandOrPay { filter: goblin(), pay: 3 }],
        ..creature("Squeaking Pie Sneak", cost(&[generic(1), b()]), vec![CardType::Creature], vec![CreatureType::Goblin, CreatureType::Rogue], 2, 2)
    }
}

/// Warren Pilferers — {4}{B} 3/3 Goblin Rogue. ETB, return target creature card
/// from your graveyard to your hand; a Goblin card gives it haste this turn.
pub fn warren_pilferers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: goblin() },
                then: Box::new(Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Warren Pilferers", cost(&[generic(4), b()]), vec![CardType::Creature], vec![CreatureType::Goblin, CreatureType::Rogue], 3, 3)
    }
}

/// Consulate Turret — {3} artifact. {T}: {E}. {T}, pay {E}{E}{E}: 2 damage to
/// target player or planeswalker.
pub fn consulate_turret() -> CardDefinition {
    CardDefinition {
        name: "Consulate Turret",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: energy(1), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 3,
                effect: Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Deadlock Trap — {3} artifact, enters tapped. ETB, {E}{E}. {T}, pay {E}: tap
/// target creature or planeswalker; its activated abilities can't be
/// activated this turn.
pub fn deadlock_trap() -> CardDefinition {
    CardDefinition {
        name: "Deadlock Trap",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(energy(2))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 1,
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.or(R::Planeswalker)) },
                Effect::LockActivatedAbilitiesThisTurn { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aethertorch Renegade — {2}{R} 1/2 Human Rogue. ETB, four {E}. {T}, pay
/// {E}{E}: 1 damage to target creature. {T}, pay eight {E}: 6 damage to target
/// player or planeswalker.
pub fn aethertorch_renegade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(energy(4))],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 2,
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 8,
                effect: Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(6) },
                ..Default::default()
            },
        ],
        ..creature("Aethertorch Renegade", cost(&[generic(2), r()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Rogue], 1, 2)
    }
}

/// Maulfist Doorbuster — {3}{R} 4/2 Human Warrior. ETB, {E}{E}. Attacking, you
/// may pay {E}: target creature can't block this turn.
pub fn maulfist_doorbuster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(energy(2)),
            attack_pay_energy(1, Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..creature("Maulfist Doorbuster", cost(&[generic(3), r()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Warrior], 4, 2)
    }
}

/// Multiform Wonder — {5} 3/3 artifact creature Construct. ETB, three {E}. Pay
/// {E}: your choice of flying, vigilance or lifelink until end of turn. Pay
/// {E}: +2/-2 or -2/+2 until end of turn.
pub fn multiform_wonder() -> CardDefinition {
    let grant = |k: Keyword| Effect::GrantKeyword { what: Selector::This, keyword: k, duration: Duration::EndOfTurn };
    let shift = |p: i32| Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(-p), duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![etb(energy(3))],
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 1,
                effect: Effect::ChooseMode(vec![grant(Keyword::Flying), grant(Keyword::Vigilance), grant(Keyword::Lifelink)]),
                ..Default::default()
            },
            ActivatedAbility { energy_cost: 1, effect: Effect::ChooseMode(vec![shift(2), shift(-2)]), ..Default::default() },
        ],
        ..creature("Multiform Wonder", cost(&[generic(5)]), vec![CardType::Artifact, CardType::Creature], vec![CreatureType::Construct], 3, 3)
    }
}

/// Scrapper Champion — {3}{R} 2/2 double strike Human Artificer. ETB, {E}{E}.
/// Attacking, you may pay {E}{E} for a +1/+1 counter.
pub fn scrapper_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![etb(energy(2)), attack_pay_energy(2, self_counter())],
        ..creature("Scrapper Champion", cost(&[generic(3), r()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Artificer], 2, 2)
    }
}

/// Thriving Grubs — {1}{R} 2/1 Gremlin. ETB, {E}{E}. Attacking, you may pay
/// {E}{E} for a +1/+1 counter.
pub fn thriving_grubs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(energy(2)), attack_pay_energy(2, self_counter())],
        ..creature("Thriving Grubs", cost(&[generic(1), r()]), vec![CardType::Creature], vec![CreatureType::Gremlin], 2, 1)
    }
}
