//! Gap batch — MKM/TDM detectives, walls, and equipment on existing primitives
//! (plus the newly artifact-aware ETB suppressor). Tests in `tests/recent226.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{battalion, each_opponent, investigate, on_other_dies};
use crate::effect::{Duration, Effect, Selector, StaticEffect, Value, ZoneRef};
use crate::mana::{b, cost, generic, r, w};

/// Doorkeeper Thrull — {1}{W} 1/2 Thrull. Flash, flying; artifacts and creatures
/// entering the battlefield don't cause abilities to trigger.
pub fn doorkeeper_thrull() -> CardDefinition {
    CardDefinition {
        name: "Doorkeeper Thrull",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thrull], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Artifacts and creatures entering don't cause abilities to trigger.",
            effect: StaticEffect::SuppressCreatureEtbTriggers { also_dies: false, also_artifacts: true },
        }],
        ..Default::default()
    }
}

/// Sanctuary Wall — {1}{W} 0/4 Wall. Defender; {2}{W}, {T}: Tap target creature
/// and put a stun counter on it and on this creature. (The "you may" on the
/// stun is folded into an always-stun.)
pub fn sanctuary_wall() -> CardDefinition {
    CardDefinition {
        name: "Sanctuary Wall",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::Const(1) },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Stun, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// All-Out Assault — {2}{R}{W}{B} Enchantment. Creatures you control get +1/+1
/// and have deathtouch; ETB: an additional combat phase followed by an
/// additional main phase. (The "untap on your next attack" rider is omitted.)
pub fn all_out_assault() -> CardDefinition {
    let team = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "All-Out Assault",
        cost: cost(&[generic(2), r(), w(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: team.clone(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Creatures you control have deathtouch.",
                effect: StaticEffect::GrantKeyword { applies_to: team, keyword: Keyword::Deathtouch },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AdditionalCombatPhaseAfterMain { count: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Homicide Investigator — {1}{B} 2/2 Human Detective. Whenever one or more
/// creatures you control die, investigate. Only once each turn.
pub fn homicide_investigator() -> CardDefinition {
    let mut trig = on_other_dies(investigate(1));
    trig.event = trig.event.once_per_turn();
    CardDefinition {
        name: "Homicide Investigator",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![trig],
        ..Default::default()
    }
}

/// Lead Pipe — {B} Artifact — Clue Equipment. Equipped creature gets +2/+0 and
/// "whenever this creature dies, each opponent loses 1 life". {2}, Sacrifice
/// this Equipment: Draw a card. Equip {2}.
pub fn lead_pipe() -> CardDefinition {
    let dies = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::LoseLife { who: each_opponent(), amount: Value::Const(1) },
    };
    CardDefinition {
        name: "Lead Pipe",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue, ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            triggered_abilities: vec![dies],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Karlov Watchdog — {3}{W} 3/2 Dog. Vigilance; whenever you attack with three
/// or more creatures, creatures you control get +1/+1. (The "opponents can't
/// turn face up on your turn" clause is omitted.)
pub fn karlov_watchdog() -> CardDefinition {
    CardDefinition {
        name: "Karlov Watchdog",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![battalion(Effect::PumpPT {
            what: Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::Creature.and(R::ControlledByYou),
            },
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// No Witnesses — {2}{W}{W} Sorcery. Investigate, then destroy all creatures.
/// (The "each player who controls the most creatures investigates" is modeled
/// as a single investigate for you.)
pub fn no_witnesses() -> CardDefinition {
    CardDefinition {
        name: "No Witnesses",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            investigate(1),
            Effect::Destroy {
                what: Selector::EachMatching { zone: ZoneRef::Battlefield, filter: R::Creature },
            },
        ]),
        ..Default::default()
    }
}
