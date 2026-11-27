//! Commander: the Klingon closers for Curzon Dax, Gowron (Eager for Battle)
//! and Kruge's EDHREC average decks (scan of 2026-11-27). Tests in
//! `tests/recent_b/cmdr_01wr6s_a.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, generic, r};

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

fn klingon(name: &'static str, mana: ManaCost, other: CreatureType, p: i32, t: i32) -> CardDefinition {
    creature(name, mana, vec![CreatureType::Klingon, other], p, t)
}

/// Dahar Master — {3}{R} 4/3 Klingon Warrior. ETB: you may sacrifice another
/// nonland permanent; when you do, 2 damage to any target.
pub fn dahar_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another nonland permanent to deal 2 damage to any target?".into(),
            filter: R::Not(Box::new(R::Land)).and(R::OtherThanSource),
            count: Value::ONE,
            // CR 603.12 — "When you do": the target is chosen after the sacrifice.
            then: Box::new(Effect::ReflexiveTrigger {
                body: Box::new(Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::Const(2) }),
            }),
            else_: None,
        })],
        ..klingon("Dahar Master", cost(&[generic(3), r()]), CreatureType::Warrior, 4, 3)
    }
}

/// Klingon Strike Force — {3}{R}{R} 5/5 Klingon Warrior, trample, haste.
/// Dies: 2 damage to each opponent.
pub fn klingon_strike_force() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        ..klingon("Klingon Strike Force", cost(&[generic(3), r(), r()]), CreatureType::Warrior, 5, 5)
    }
}

/// Munitions Enthusiast — {1}{R} 3/2 Klingon Warrior, trample. Dies: 1
/// damage to any target.
pub fn munitions_enthusiast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_dies(Effect::DealDamage {
            to: target_filtered(R::any_target()),
            amount: Value::ONE,
        })],
        ..klingon("Munitions Enthusiast", cost(&[generic(1), r()]), CreatureType::Warrior, 3, 2)
    }
}

/// Targ Wrangler — {2}{R} 5/3 Klingon Warrior, menace. At the beginning of
/// your end step, if it attacked this turn, sacrifice a nonland permanent.
pub fn targ_wrangler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::AttackedThisTurn }),
            effect: Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Not(Box::new(R::Land)) },
        }],
        ..klingon("Targ Wrangler", cost(&[generic(2), r()]), CreatureType::Warrior, 5, 3)
    }
}

/// Kruge, Genesis Seeker — {2}{R} legendary 2/2 Klingon Officer. Attacks:
/// attacking Klingons you control get +1/+1 until end of turn.
pub fn kruge_genesis_seeker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Klingon).and(R::IsAttacking).and(R::ControlledByYou),
            ),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..klingon("Kruge, Genesis Seeker", cost(&[generic(2), r()]), CreatureType::Officer, 2, 2)
    }
}

/// Storied Veteran — {B} 2/1 Klingon Advisor; can't block. You may cast it
/// from your graveyard; if you do, it enters with a finality counter.
pub fn storied_veteran() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        static_abilities: vec![StaticAbility {
            description: "You may cast this card from your graveyard. If you do, it enters with a finality counter on it.",
            effect: StaticEffect::GraveyardCastWithLifeSurcharge { filter: R::HasName("Storied Veteran".into()), life: 0 },
        }],
        ..klingon("Storied Veteran", cost(&[b()]), CreatureType::Advisor, 2, 1)
    }
}

/// Vengeance Mission — {2}{B} enchantment. Whenever a creature dies, a quest
/// counter on it; target opponent loses 1 life, or 2 with seven or more
/// quest counters.
pub fn vengeance_mission() -> CardDefinition {
    let lose = |n: i32| Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(n) };
    CardDefinition {
        name: "Vengeance Mission",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
                Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::If {
                        cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Quest, n: 7 },
                        then: Box::new(lose(2)),
                        else_: Box::new(lose(1)),
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Defense Force Aggressor — {R} 2/1 Klingon Warrior (vanilla).
pub fn defense_force_aggressor() -> CardDefinition {
    klingon("Defense Force Aggressor", cost(&[r()]), CreatureType::Warrior, 2, 1)
}

/// Mek'leth Berserker — {3}{R} 4/3 Klingon Berserker, menace.
pub fn mekleth_berserker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        ..klingon("Mek'leth Berserker", cost(&[generic(3), r()]), CreatureType::Berserker, 4, 3)
    }
}

/// Warship Flight Crew — {1}{R} 2/2 Klingon Pilot (vanilla).
pub fn warship_flight_crew() -> CardDefinition {
    klingon("Warship Flight Crew", cost(&[generic(1), r()]), CreatureType::Pilot, 2, 2)
}
