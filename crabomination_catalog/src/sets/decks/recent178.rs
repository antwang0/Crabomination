//! FDN/BLB/DSK gap batch, wave 3 — on existing primitives: Marching Duodrone
//! (on-attack team Treasure), Fiendish Panda (lifegain counters + death
//! reanimation), Quick-Draw Katana (during-turn Equipment), and Salvation Swan
//! (Bird-ETB blink). Tests in `crabomination/src/tests/recent178.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, on_dies};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{b, cost, generic, w};

/// Marching Duodrone — {2} 2/2 Construct. Whenever it attacks, each player
/// creates a Treasure token.
pub fn marching_duodrone() -> CardDefinition {
    CardDefinition {
        name: "Marching Duodrone",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::CreateToken {
            who: PlayerRef::EachPlayer,
            count: Value::ONE,
            definition: Box::new(crabomination_base::tokens::treasure_token()),
        })],
        ..Default::default()
    }
}

/// Fiendish Panda — {2}{W}{B} 3/2 Bear Demon. Whenever you gain life, put a
/// +1/+1 counter on it. When it dies, return another target non-Bear creature
/// card with mana value ≤ its power from your graveyard to the battlefield.
pub fn fiendish_panda() -> CardDefinition {
    CardDefinition {
        name: "Fiendish Panda",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Demon],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            on_dies(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature
                        .and(R::InYourGraveyard)
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Bear))))
                        .and(R::ManaValueAtMostSourcePower),
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            }),
        ],
        ..Default::default()
    }
}

/// Quick-Draw Katana — {2} Equipment. During your turn, equipped creature gets
/// +2/+0 and has first strike. Equip {2}. (The +2/+0 is modeled as always-on;
/// only the first-strike half is turn-gated.)
pub fn quick_draw_katana() -> CardDefinition {
    CardDefinition {
        name: "Quick-Draw Katana",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            during_your_turn_keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Salvation Swan — {3}{W} 3/3 Bird Cleric with flash and flying. Whenever this
/// or another Bird you control enters, exile up to one target creature you
/// control without flying and return it at the next end step. (The printed
/// flying counter on the returned creature is approximated away.)
pub fn salvation_swan() -> CardDefinition {
    CardDefinition {
        name: "Salvation Swan",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Bird)),
                }),
            // "Up to one target" is modeled as a single target (the trigger
            // simply does nothing when you control no eligible nonflyer).
            effect: Effect::ExileReturnNextEndStep {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature
                        .and(R::ControlledByYou)
                        .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                },
            },
        }],
        ..Default::default()
    }
}
