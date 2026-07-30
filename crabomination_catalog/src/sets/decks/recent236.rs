//! DSK gap batch 2 — clean Rooms (709.5) plus Terror of Towashi. Tests in
//! `tests/recent236.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, Keyword,
    MayPlayDuration, RoomDoor, RoomDoors, SelectionRequirement as R, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
    ZoneDest, ZoneRef,
};
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};

fn on_unlock(effect: Effect) -> crate::card::TriggeredAbility {
    crate::card::TriggeredAbility {
        event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
        effect,
    }
}

fn room(name: &'static str, c: ManaCost, left: RoomDoor, right: RoomDoor) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Room],
            ..Default::default()
        },
        room: Some(Box::new(RoomDoors { left, right })),
        ..Default::default()
    }
}

/// Grand Entryway // Elegant Rotunda — {1}{W} // {2}{W} Room. Grand Entryway:
/// on unlock, create a 1/1 white Glimmer enchantment creature. Elegant
/// Rotunda: on unlock, put a +1/+1 counter on each of up to two target
/// creatures.
pub fn grand_entryway_elegant_rotunda() -> CardDefinition {
    let glimmer = TokenDefinition {
        name: "Glimmer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Glimmer],
            ..Default::default()
        },
        ..Default::default()
    };
    room(
        "Grand Entryway // Elegant Rotunda",
        cost(&[generic(1), w()]),
        RoomDoor {
            name: "Grand Entryway".into(),
            cost: cost(&[generic(1), w()]),
            triggered_abilities: vec![on_unlock(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: glimmer,
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Elegant Rotunda".into(),
            cost: cost(&[generic(2), w()]),
            triggered_abilities: vec![on_unlock(Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            })],
            ..Default::default()
        },
    )
}

/// Derelict Attic // Widow's Walk — {2}{B} // {3}{B} Room. Derelict Attic: on
/// unlock, draw two cards and lose 2 life. Widow's Walk: whenever a creature
/// you control attacks alone, it gets +1/+0 and gains deathtouch until end of
/// turn.
pub fn derelict_attic_widows_walk() -> CardDefinition {
    room(
        "Derelict Attic // Widow's Walk",
        cost(&[generic(2), b()]),
        RoomDoor {
            name: "Derelict Attic".into(),
            cost: cost(&[generic(2), b()]),
            triggered_abilities: vec![on_unlock(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]))],
            ..Default::default()
        },
        RoomDoor {
            name: "Widow's Walk".into(),
            cost: cost(&[generic(3), b()]),
            triggered_abilities: vec![crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsAttackingAlone,
                    },
                ),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Deathtouch,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            }],
            ..Default::default()
        },
    )
}

/// Funeral Room // Awakening Hall — {2}{B} // {6}{B}{B} Room. Funeral Room:
/// whenever a creature you control dies, each opponent loses 1 life and you
/// gain 1. Awakening Hall: on unlock, return all creature cards from your
/// graveyard to the battlefield.
pub fn funeral_room_awakening_hall() -> CardDefinition {
    room(
        "Funeral Room // Awakening Hall",
        cost(&[generic(2), b()]),
        RoomDoor {
            name: "Funeral Room".into(),
            cost: cost(&[generic(2), b()]),
            triggered_abilities: vec![crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Awakening Hall".into(),
            cost: cost(&[generic(6), b(), b()]),
            triggered_abilities: vec![on_unlock(Effect::Move {
                what: Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::Creature,
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            })],
            ..Default::default()
        },
    )
}

/// Painter's Studio // Defaced Gallery — {2}{R} // {1}{R} Room. Painter's
/// Studio: on unlock, exile the top two cards of your library — you may play
/// them until the end of your next turn. Defaced Gallery: whenever you attack,
/// attacking creatures you control get +1/+0 until end of turn.
pub fn painters_studio_defaced_gallery() -> CardDefinition {
    room(
        "Painter's Studio // Defaced Gallery",
        cost(&[generic(2), r()]),
        RoomDoor {
            name: "Painter's Studio".into(),
            cost: cost(&[generic(2), r()]),
            triggered_abilities: vec![on_unlock(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                pay_own_cost: true,
                uncast_penalty: None,
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Defaced Gallery".into(),
            cost: cost(&[generic(1), r()]),
            triggered_abilities: vec![crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            }],
            ..Default::default()
        },
    )
}

/// Terror of Towashi — {2}{B}{B} Phyrexian Ogre 4/3. Deathtouch. Whenever it
/// attacks, you may pay {3}{B}. When you do, return target creature card from
/// your graveyard to the battlefield.
pub fn terror_of_towashi() -> CardDefinition {
    CardDefinition {
        name: "Terror of Towashi",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Ogre],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Terror of Towashi: pay {3}{B} to reanimate?".into(),
            mana_cost: cost(&[generic(3), b()]),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}
