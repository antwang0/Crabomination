//! DSK gap batch — Rooms (709.5) and Survivors, on existing primitives plus
//! the manifest-dread `LastMoved` rider. Tests in `tests/recent235.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, RoomDoor, RoomDoors,
    SelectionRequirement as R, StaticAbility, Subtypes,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{
    ActivatedAbility, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
};
use crate::mana::{ManaCost, cost, g, generic, u, w};

fn room(
    name: &'static str,
    parent_cost: ManaCost,
    left: RoomDoor,
    right: RoomDoor,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: parent_cost,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Room],
            ..Default::default()
        },
        room: Some(Box::new(RoomDoors { left, right })),
        ..Default::default()
    }
}

/// "When you unlock this door, …" trigger.
fn on_unlock(effect: Effect) -> crate::card::TriggeredAbility {
    crate::card::TriggeredAbility {
        event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
        effect,
    }
}

/// Surgical Suite // Hospital Room — {1}{W} // {3}{W} Room. Surgical Suite:
/// on unlock, reanimate a creature card MV≤3 from your graveyard. Hospital
/// Room: whenever you attack, put a +1/+1 counter on target attacking creature.
pub fn surgical_suite_hospital_room() -> CardDefinition {
    room(
        "Surgical Suite // Hospital Room",
        cost(&[generic(1), w()]),
        RoomDoor {
            name: "Surgical Suite".into(),
            cost: cost(&[generic(1), w()]),
            triggered_abilities: vec![on_unlock(Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::ManaValueAtMost(3))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Hospital Room".into(),
            cost: cost(&[generic(3), w()]),
            triggered_abilities: vec![crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        },
    )
}

/// Underwater Tunnel // Slimy Aquarium — {U} // {3}{U} Room. Underwater
/// Tunnel: on unlock, surveil 2. Slimy Aquarium: on unlock, manifest dread,
/// then put a +1/+1 counter on that creature.
pub fn underwater_tunnel_slimy_aquarium() -> CardDefinition {
    room(
        "Underwater Tunnel // Slimy Aquarium",
        cost(&[u()]),
        RoomDoor {
            name: "Underwater Tunnel".into(),
            cost: cost(&[u()]),
            triggered_abilities: vec![on_unlock(Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Slimy Aquarium".into(),
            cost: cost(&[generic(3), u()]),
            triggered_abilities: vec![on_unlock(Effect::Seq(vec![
                Effect::ManifestDread {
                    who: PlayerRef::You,
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]))],
            ..Default::default()
        },
    )
}

/// Moldering Gym // Weight Room — {2}{G} // {5}{G} Room. Moldering Gym: on
/// unlock, search a basic land onto the battlefield tapped. Weight Room: on
/// unlock, manifest dread, then put three +1/+1 counters on that creature.
pub fn moldering_gym_weight_room() -> CardDefinition {
    room(
        "Moldering Gym // Weight Room",
        cost(&[generic(2), g()]),
        RoomDoor {
            name: "Moldering Gym".into(),
            cost: cost(&[generic(2), g()]),
            triggered_abilities: vec![on_unlock(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Weight Room".into(),
            cost: cost(&[generic(5), g()]),
            triggered_abilities: vec![on_unlock(Effect::Seq(vec![
                Effect::ManifestDread {
                    who: PlayerRef::You,
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            ]))],
            ..Default::default()
        },
    )
}

/// Greenhouse // Rickety Gazebo — {2}{G} // {3}{G} Room. Greenhouse: lands you
/// control have "{T}: Add one mana of any color." Rickety Gazebo: on unlock,
/// mill four, then return up to two permanent cards from among them to hand.
pub fn greenhouse_rickety_gazebo() -> CardDefinition {
    room(
        "Greenhouse // Rickety Gazebo",
        cost(&[generic(2), g()]),
        RoomDoor {
            name: "Greenhouse".into(),
            cost: cost(&[generic(2), g()]),
            static_abilities: vec![StaticAbility {
                description: "Lands you control have \"{T}: Add one mana of any color.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::AnyOneColor(Value::ONE),
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Rickety Gazebo".into(),
            cost: cost(&[generic(3), g()]),
            triggered_abilities: vec![on_unlock(Effect::MillThenToHandN {
                amount: Value::Const(4),
                filter: R::Permanent,
                take: Value::Const(2),
            })],
            ..Default::default()
        },
    )
}

/// Walk-In Closet // Forgotten Cellar — {2}{G} // {3}{G}{G} Room. Walk-In
/// Closet: you may play lands from your graveyard. Forgotten Cellar: on unlock,
/// you may cast spells from your graveyard this turn, and cards that would go
/// to your graveyard this turn are exiled instead (Gaea's Will pair).
pub fn walk_in_closet_forgotten_cellar() -> CardDefinition {
    room(
        "Walk-In Closet // Forgotten Cellar",
        cost(&[generic(2), g()]),
        RoomDoor {
            name: "Walk-In Closet".into(),
            cost: cost(&[generic(2), g()]),
            static_abilities: vec![StaticAbility {
                description: "You may play lands from your graveyard.",
                effect: StaticEffect::MayPlayLandsFromGraveyard,
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Forgotten Cellar".into(),
            cost: cost(&[generic(3), g(), g()]),
            triggered_abilities: vec![on_unlock(Effect::Seq(vec![
                Effect::PlayFromGraveyardThisTurn,
                Effect::ExileYourGraveyardBoundThisTurn,
            ]))],
            ..Default::default()
        },
    )
}

/// Orphans of the Wheat — {1}{W} Human 2/1. Whenever this creature attacks,
/// tap any number of untapped creatures you control. It gets +1/+1 until end
/// of turn for each creature tapped this way.
pub fn orphans_of_the_wheat() -> CardDefinition {
    CardDefinition {
        name: "Orphans of the Wheat",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_attack(Effect::TapAnyNumberThenPumpPerTapped {
            filter: R::Creature
                .and(R::ControlledByYou)
                .and(R::Untapped)
                .and(R::OtherThanSource),
            power: 1,
            toughness: 1,
        })],
        ..Default::default()
    }
}
