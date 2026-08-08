//! Duskmourn's remaining Room cycle (CR 709.5). Tests in
//! `tests/recent_b/dsk_rooms.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, Keyword, MayPlayDuration,
    RoomDoor, RoomDoors, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, r, u, w};
use crabomination_base::turn_step::TurnStep;

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
fn on_unlock(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
        effect,
    }
}

/// "At the beginning of your upkeep, …" trigger.
fn on_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

/// Central Elevator // Promising Stairs — {3}{U} // {2}{U}. Elevator tutors a
/// Room you don't already control; Stairs surveils each upkeep and wins on
/// eight differently-named unlocked doors.
pub fn central_elevator_promising_stairs() -> CardDefinition {
    room(
        "Central Elevator // Promising Stairs",
        cost(&[generic(3), u()]),
        RoomDoor {
            name: "Central Elevator".into(),
            cost: cost(&[generic(3), u()]),
            triggered_abilities: vec![on_unlock(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Room)
                    .and(R::NameNotSharedWithYourPermanents),
                to: ZoneDest::Hand(PlayerRef::You),
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Promising Stairs".into(),
            cost: cost(&[generic(2), u()]),
            triggered_abilities: vec![on_upkeep(Effect::Seq(vec![
                Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::DistinctUnlockedDoorNamesAtLeast {
                        who: PlayerRef::You,
                        count: 8,
                    },
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            ]))],
            ..Default::default()
        },
    )
}

/// Charred Foyer // Warped Space — {3}{R} // {4}{R}{R}. Foyer impulse-draws
/// each upkeep; Warped Space makes one exile cast a turn free.
pub fn charred_foyer_warped_space() -> CardDefinition {
    room(
        "Charred Foyer // Warped Space",
        cost(&[generic(3), r()]),
        RoomDoor {
            name: "Charred Foyer".into(),
            cost: cost(&[generic(3), r()]),
            triggered_abilities: vec![on_upkeep(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Warped Space".into(),
            cost: cost(&[generic(4), r(), r()]),
            static_abilities: vec![StaticAbility {
                description: "Once each turn, you may pay {0} rather than pay the mana cost \
                              for a spell you cast from exile.",
                effect: StaticEffect::FreeExileCastOncePerTurn,
            }],
            ..Default::default()
        },
    )
}

/// Dazzling Theater // Prop Room — {3}{W} // {2}{W}. Theater gives creature
/// spells convoke; Prop Room untaps your team off-turn.
pub fn dazzling_theater_prop_room() -> CardDefinition {
    room(
        "Dazzling Theater // Prop Room",
        cost(&[generic(3), w()]),
        RoomDoor {
            name: "Dazzling Theater".into(),
            cost: cost(&[generic(3), w()]),
            static_abilities: vec![StaticAbility {
                description: "Creature spells you cast have convoke.",
                effect: StaticEffect::GrantConvokeToSpells { filter: R::Creature },
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Prop Room".into(),
            cost: cost(&[generic(2), w()]),
            static_abilities: vec![StaticAbility {
                description: "Untap each creature you control during each other player's \
                              untap step.",
                effect: StaticEffect::UntapYoursEachUntapStepFiltered(R::Creature),
            }],
            ..Default::default()
        },
    )
}

/// A 1/1 white Toy artifact creature.
fn toy_token() -> TokenDefinition {
    TokenDefinition {
        name: "Toy".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Toy],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Dollmaker's Shop // Porcelain Gallery — {1}{W} // {4}{W}{W}. The Shop mints
/// a Toy on each non-Toy attack; the Gallery rewrites your team's base P/T.
pub fn dollmakers_shop_porcelain_gallery() -> CardDefinition {
    room(
        "Dollmaker's Shop // Porcelain Gallery",
        cost(&[generic(1), w()]),
        RoomDoor {
            name: "Dollmaker's Shop".into(),
            cost: cost(&[generic(1), w()]),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                    Predicate::AttackedWithCreatureMatching {
                        who: PlayerRef::You,
                        filter: R::Not(Box::new(R::HasCreatureType(CreatureType::Toy))),
                    },
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(toy_token()),
                },
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Porcelain Gallery".into(),
            cost: cost(&[generic(4), w(), w()]),
            static_abilities: vec![StaticAbility {
                description: "Creatures you control have base power and toughness each equal \
                              to the number of creatures you control.",
                effect: StaticEffect::SetBasePtForFilterFromValue {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::CreatureCountControlledBy(PlayerRef::You),
                    toughness: Value::CreatureCountControlledBy(PlayerRef::You),
                },
            }],
            ..Default::default()
        },
    )
}

/// Mirror Room // Fractured Realm — {2}{U} // {5}{U}{U}. The Room copies a
/// creature on unlock; the Realm doubles every trigger you control.
pub fn mirror_room_fractured_realm() -> CardDefinition {
    room(
        "Mirror Room // Fractured Realm",
        cost(&[generic(2), u()]),
        RoomDoor {
            name: "Mirror Room".into(),
            cost: cost(&[generic(2), u()]),
            triggered_abilities: vec![on_unlock(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                extra_creature_types: vec![CreatureType::Reflection],
                extra_card_types: Vec::new(),
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: Vec::new(),
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Fractured Realm".into(),
            cost: cost(&[generic(5), u(), u()]),
            static_abilities: vec![StaticAbility {
                description: "If a triggered ability of a permanent you control triggers, \
                              that ability triggers an additional time.",
                effect: StaticEffect::DoubleControllerPermanentTriggers,
            }],
            ..Default::default()
        },
    )
}

/// An X/X blue flying Spirit, X = unlocked doors you control.
fn door_spirit() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some((
            Value::UnlockedDoorsControlled(PlayerRef::You),
            Value::UnlockedDoorsControlled(PlayerRef::You),
        )),
        ..Default::default()
    }
}

/// Smoky Lounge // Misty Salon — {2}{R} // {3}{U}. The Lounge rituals two red
/// for Rooms only; the Salon mints a Spirit sized by your open doors.
pub fn smoky_lounge_misty_salon() -> CardDefinition {
    room(
        "Smoky Lounge // Misty Salon",
        cost(&[generic(2), r()]),
        RoomDoor {
            name: "Smoky Lounge".into(),
            cost: cost(&[generic(2), r()]),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::PreCombatMain),
                    EventScope::YourControl,
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Red, Color::Red])),
                        SpendRestriction::RoomSpellsOrDoors,
                    ),
                },
            }],
            ..Default::default()
        },
        RoomDoor {
            name: "Misty Salon".into(),
            cost: cost(&[generic(3), u()]),
            triggered_abilities: vec![on_unlock(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(door_spirit()),
            })],
            ..Default::default()
        },
    )
}
