//! Most-built commanders missing from the catalog (COMMANDER_BACKLOG §1,
//! regenerated 2026-09-27): Reaper King, Shroofus Sproutsire, Kratos, God of
//! War, Myrel, Shield of Argive, Doran, Besieged by Time, Gargos, Vicious
//! Watcher, and Syr Gwyn, Hero of Ashvale. All but Syr Gwyn are built from
//! primitives other cards already use; Syr Gwyn's "Equipment you control have
//! equip Knight {0}" is `StaticEffect::EquipmentYouControlEquipZeroFor`
//! (CR 702.6c, an equip ability restricted to a quality).
//!
//! - **Gargos** — "up to one target creature you don't control" takes the
//!   target when one exists; with none the trigger is removed (CR 603.3d),
//!   the same as choosing zero.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, mono_hybrid, r, w};

fn legend(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn trigger_source_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// Reaper King — other Scarecrows you control get +1/+1; another Scarecrow
/// of yours entering destroys target permanent.
pub fn reaper_king() -> CardDefinition {
    let scarecrow = || R::HasCreatureType(CreatureType::Scarecrow);
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Other Scarecrow creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    scarecrow().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_source_is(scarecrow().and(R::OtherThanSource))),
            effect: Effect::Destroy { what: target_filtered(R::Permanent) },
        }],
        ..legend(
            "Reaper King",
            cost(&[
                mono_hybrid(2, Color::White),
                mono_hybrid(2, Color::Blue),
                mono_hybrid(2, Color::Black),
                mono_hybrid(2, Color::Red),
                mono_hybrid(2, Color::Green),
            ]),
            vec![CreatureType::Scarecrow],
            6,
            6,
        )
    }
}

/// Shroofus Sproutsire — trample; a Saproling of yours dealing combat damage
/// to a player makes that many 1/1 Saprolings.
pub fn shroofus_sproutsire() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(trigger_source_is(R::HasCreatureType(CreatureType::Saproling))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: std::sync::Arc::new(saproling),
            },
        }],
        ..legend("Shroofus Sproutsire", cost(&[generic(2), g()]), vec![CreatureType::Saproling], 1, 1)
    }
}

/// Kratos, God of War — double strike; all creatures have haste; at each
/// player's end step, damage to that player equal to the creatures they
/// control that didn't attack this turn.
pub fn kratos_god_of_war() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "All creatures have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::PermanentCountControlledByMatching(
                    PlayerRef::ActivePlayer,
                    R::Creature.and(R::Not(Box::new(R::AttackedThisTurn))),
                ),
            },
        }],
        ..legend("Kratos, God of War", cost(&[r(), r(), r()]), vec![CreatureType::God, CreatureType::Warrior], 2, 3)
    }
}

/// Myrel, Shield of Argive — during your turn, opponents can't cast spells or
/// activate abilities of artifacts, creatures, or enchantments; attacking
/// makes a 1/1 Soldier artifact creature per Soldier you control.
pub fn myrel_shield_of_argive() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, your opponents can't cast spells or activate abilities of artifacts, creatures, or enchantments.",
            effect: StaticEffect::OpponentsCantActDuringYourTurn,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PermanentCountControlledByMatching(
                    PlayerRef::You,
                    R::HasCreatureType(CreatureType::Soldier),
                ),
                definition: std::sync::Arc::new(soldier),
            },
        }],
        ..legend(
            "Myrel, Shield of Argive",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Doran, Besieged by Time — creature spells with toughness greater than
/// power cost {1} less; a creature of yours attacking or blocking gets +X/+X,
/// X the difference between its power and toughness.
pub fn doran_besieged_by_time() -> CardDefinition {
    let pump = || {
        let p = || Value::PowerOf(Box::new(Selector::TriggerSource));
        let t = || Value::ToughnessOf(Box::new(Selector::TriggerSource));
        let x = Value::Max(Box::new(Value::Diff(Box::new(p()), Box::new(t()))), Box::new(Value::Diff(Box::new(t()), Box::new(p()))));
        Effect::PumpPT { what: Selector::TriggerSource, power: x.clone(), toughness: x, duration: Duration::EndOfTurn }
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature spell you cast with toughness greater than its power costs {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Creature.and(R::ToughnessGreaterThanPower), amount: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::YourControl), effect: pump() },
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::YourControl), effect: pump() },
        ],
        ..legend(
            "Doran, Besieged by Time",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Treefolk, CreatureType::Druid],
            0,
            5,
        )
    }
}

/// Gargos, Vicious Watcher — vigilance; Hydra spells cost {4} less; a
/// creature of yours becoming the target of a spell makes Gargos fight a
/// creature you don't control.
pub fn gargos_vicious_watcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Hydra spells you cast cost {4} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::HasCreatureType(CreatureType::Hydra), amount: 4 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                causer_filter: Some(R::IsSpellOnStack),
                ..EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
            },
            effect: Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
            },
        }],
        ..legend("Gargos, Vicious Watcher", cost(&[generic(3), g(), g(), g()]), vec![CreatureType::Hydra], 8, 7)
    }
}

/// Syr Gwyn, Hero of Ashvale — vigilance, menace; an equipped creature of
/// yours attacking draws a card and loses 1 life; Equipment you control have
/// equip Knight {0}.
pub fn syr_gwyn_hero_of_ashvale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Equipment you control have equip Knight {0}.",
            effect: StaticEffect::EquipmentYouControlEquipZeroFor { filter: R::HasCreatureType(CreatureType::Knight) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(trigger_source_is(R::IsEquipped)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Syr Gwyn, Hero of Ashvale",
            cost(&[generic(3), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            5,
            5,
        )
    }
}
