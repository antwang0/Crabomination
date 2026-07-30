//! DSK gap batch 3 — two Rooms, a graveyard-fueled draw spell, and a delirium
//! anthem creature. Tests in `tests/recent237.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, Keyword, RoomDoor, RoomDoors,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticEffect, Value,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

/// Ticket Booth // Tunnel of Hate — {2}{R} // {4}{R}{R} Room. Ticket Booth: on
/// unlock, manifest dread. Tunnel of Hate: whenever you attack, target
/// attacking creature gains double strike until end of turn.
pub fn ticket_booth_tunnel_of_hate() -> CardDefinition {
    room(
        "Ticket Booth // Tunnel of Hate",
        cost(&[generic(2), r()]),
        RoomDoor {
            name: "Ticket Booth".into(),
            cost: cost(&[generic(2), r()]),
            triggered_abilities: vec![on_unlock(Effect::ManifestDread {
                who: PlayerRef::You,
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Tunnel of Hate".into(),
            cost: cost(&[generic(4), r(), r()]),
            triggered_abilities: vec![crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            }],
            ..Default::default()
        },
    )
}

/// Restricted Office // Lecture Hall — {2}{W}{W} // {5}{U}{U} Room. Restricted
/// Office: on unlock, destroy all creatures with power 3 or greater. Lecture
/// Hall: other permanents you control have hexproof.
pub fn restricted_office_lecture_hall() -> CardDefinition {
    room(
        "Restricted Office // Lecture Hall",
        cost(&[generic(2), w(), w()]),
        RoomDoor {
            name: "Restricted Office".into(),
            cost: cost(&[generic(2), w(), w()]),
            triggered_abilities: vec![on_unlock(Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(3))),
            })],
            ..Default::default()
        },
        RoomDoor {
            name: "Lecture Hall".into(),
            cost: cost(&[generic(5), u(), u()]),
            static_abilities: vec![StaticAbility {
                description: "Other permanents you control have hexproof.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::ControlledByYou.and(R::OtherThanSource)),
                    keyword: Keyword::Hexproof,
                },
            }],
            ..Default::default()
        },
    )
}

/// Peer Past the Veil — {2}{R}{G} Instant. Discard your hand, then draw X
/// cards, where X is the number of card types among cards in your graveyard.
pub fn peer_past_the_veil() -> CardDefinition {
    CardDefinition {
        name: "Peer Past the Veil",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::HandSizeOf(PlayerRef::You),
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CardTypesInGraveyard(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

fn insect_token() -> TokenDefinition {
    TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The Swarmweaver — {2}{B}{G} Legendary Artifact Creature — Scarecrow 2/3.
/// ETB: create two 1/1 black-green flying Insects. Delirium — while four or
/// more card types are in your graveyard, Insects and Spiders you control get
/// +1/+1 and have deathtouch.
pub fn the_swarmweaver() -> CardDefinition {
    CardDefinition {
        name: "The Swarmweaver",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scarecrow],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: insect_token(),
        })],
        static_abilities: vec![StaticAbility {
            description: "Delirium — Insects and Spiders you control get +1/+1 and have deathtouch.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::DeliriumActive {
                    who: PlayerRef::You,
                },
                applies_to: Selector::EachPermanent(
                    R::ControlledByYou.and(
                        R::HasCreatureType(CreatureType::Insect)
                            .or(R::HasCreatureType(CreatureType::Spider)),
                    ),
                ),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Deathtouch],
            },
        }],
        ..Default::default()
    }
}
