//! Commander: 2026-10-19 (`016Xh16H`) — Teyo, Diamondblade Mage's deathtouch
//! tricks. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{Color, ManaCost, b, cost, generic};
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

fn instant(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn deathtouch(what: Selector) -> Effect {
    Effect::GrantKeyword { what, keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn }
}

/// Teyo, Diamondblade Mage — {3}{B} 3/1 legendary flash Human Warlock. ETB
/// target permanent you control gains deathtouch; a creature gets a +1/+1
/// counter, a planeswalker a loyalty counter.
pub fn teyo_diamondblade_mage() -> CardDefinition {
    let counter_if = |filter: R, kind: CounterType| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::Target(0), filter },
        then: Box::new(Effect::AddCounter { what: Selector::Target(0), kind, amount: Value::ONE }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            deathtouch(target_filtered(R::Permanent.and(R::ControlledByYou))),
            counter_if(R::Creature, CounterType::PlusOnePlusOne),
            counter_if(R::Planeswalker, CounterType::Loyalty),
        ]))],
        ..creature(
            "Teyo, Diamondblade Mage",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            1,
        )
    }
}

/// Death-Rattle Oni — {6}{B} 5/4 flash Demon Spirit, {2} less per creature
/// that died this turn. ETB destroy every other creature dealt damage this
/// turn.
pub fn death_rattle_oni() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        self_cost_reduction_per: Some((Value::CreaturesDiedThisTurnTotal, 2)),
        triggered_abilities: vec![etb(Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::DealtDamageThisTurn).and(R::OtherThanSource)),
        })],
        ..creature("Death-Rattle Oni", cost(&[generic(6), b()]), vec![CreatureType::Demon, CreatureType::Spirit], 5, 4)
    }
}

/// Fire Nation Ambushers — {2}{B} 3/2 flash Human Soldier.
pub fn fire_nation_ambushers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature("Fire Nation Ambushers", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 2)
    }
}

/// Onyx Mage — {1}{B} 2/1. {1}{B}: target creature you control gains
/// deathtouch.
pub fn onyx_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: deathtouch(target_filtered(R::Creature.and(R::ControlledByYou))),
            ..Default::default()
        }],
        ..creature("Onyx Mage", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Coat with Venom — {B} instant. Target creature gets +1/+2 and deathtouch.
pub fn coat_with_venom() -> CardDefinition {
    instant(
        "Coat with Venom",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            deathtouch(Selector::Target(0)),
        ]),
    )
}

/// Scarblade's Malice — {B} instant. Your creature gains deathtouch and
/// lifelink; when it dies this turn, a 2/2 black and green Elf.
pub fn scarblades_malice() -> CardDefinition {
    let elf = TokenDefinition {
        name: "Elf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        ..Default::default()
    };
    instant(
        "Scarblade's Malice",
        cost(&[b()]),
        Effect::Seq(vec![
            deathtouch(target_filtered(R::Creature.and(R::ControlledByYou))),
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            Effect::WhenTargetDiesThisTurn {
                filter: None,
                slot: 0,
                body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(elf) }),
            },
        ]),
    )
}

/// Simulacrum — {1}{B} instant. Gain life equal to the damage dealt to you
/// this turn; Simulacrum deals that much damage to target creature you
/// control.
pub fn simulacrum() -> CardDefinition {
    let x = || Value::DamageTakenThisTurn(PlayerRef::You);
    instant(
        "Simulacrum",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: x() },
            Effect::DealDamage { to: target_filtered(R::Creature.and(R::ControlledByYou)), amount: x() },
        ]),
    )
}
