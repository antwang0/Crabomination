//! Aetherdrift (DFT) commons/uncommons on existing primitives: removal, a
//! flash pump, an Equipment Vehicle, scry/lifegain Vehicles, a saddled Mount,
//! stun-lock, and a top/bottom tuck. Tests in `tests/recent171.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::{attacks_while_saddled, etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{cost, g, generic, u, w};

/// Rover Blades — {3} Artifact — Equipment Vehicle 2/2. Double strike. Equipped
/// creature has double strike. Equip {4}. Crew 2.
pub fn rover_blades() -> CardDefinition {
    CardDefinition {
        name: "Rover Blades",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment, ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![
            Keyword::DoubleStrike,
            Keyword::Equip(cost(&[generic(4)])),
            Keyword::Crew(2),
        ],
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Spotcycle Scouter — {1}{W} Artifact — Vehicle 3/2. ETB: scry 2. Crew 1.
pub fn spotcycle_scouter() -> CardDefinition {
    CardDefinition {
        name: "Spotcycle Scouter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Veloheart Bike — {2}{G} Artifact — Vehicle 4/2. ETB: gain 2 life. {T}: Add
/// one mana of any color. Crew 2.
pub fn veloheart_bike() -> CardDefinition {
    CardDefinition {
        name: "Veloheart Bike",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Venomsac Lagac — {1}{G} 2/1 Lizard Mount. Deathtouch. Whenever it attacks
/// while saddled, it gets +0/+3 until end of turn. Saddle 2.
pub fn venomsac_lagac() -> CardDefinition {
    CardDefinition {
        name: "Venomsac Lagac",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch, Keyword::Saddle(2)],
        triggered_abilities: vec![attacks_while_saddled(Effect::PumpPT {
            what: Selector::This,
            power: Value::ZERO,
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Stall Out — {1}{U} Sorcery. Tap target creature or Vehicle, then put three
/// stun counters on it. Cycling {2}.
pub fn stall_out() -> CardDefinition {
    CardDefinition {
        name: "Stall Out",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Trip Up — {3}{U} Instant. Target nonland permanent's owner puts it on their
/// choice of the top or bottom of their library. Cycling {2}.
pub fn trip_up() -> CardDefinition {
    CardDefinition {
        name: "Trip Up",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Move {
            what: target_filtered(R::Nonland),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::OwnerChoice,
            },
        },
        ..Default::default()
    }
}

/// Spikeshell Harrier — {4}{U} 4/4 Robot Turtle. ETB: return target creature or
/// Vehicle an opponent controls to its owner's hand. (The "reduce the leading
/// opponent's speed" rider is dropped.)
pub fn spikeshell_harrier() -> CardDefinition {
    CardDefinition {
        name: "Spikeshell Harrier",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Turtle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Creature
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                    .and(R::ControlledByOpponent),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..Default::default()
    }
}
