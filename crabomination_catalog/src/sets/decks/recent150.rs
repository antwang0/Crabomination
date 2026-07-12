//! A cross-set wave (OTJ / DSK / WOE): surveil-riders, an impulse Plotter, a
//! manifest-dread Equipment, a Horror combat trick, and a choose-a-number
//! board wipe (the new `Effect::ChooseNumberDestroyByPower`). Tests in
//! `crabomination/src/tests/recent150.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EquipBonus, Keyword, LandType,
    MayPlayDuration, Predicate, SelectionRequirement as R, Selector, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, generic, r, u, w};

/// Consuming Ashes — {2}{B}{B} Instant. Exile target creature; if it had mana
/// value 3 or less, surveil 2.
pub fn consuming_ashes() -> CardDefinition {
    CardDefinition {
        name: "Consuming Ashes",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::ManaValueAtMost(3),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature) },
                Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
            ])),
            else_: Box::new(Effect::Exile { what: target_filtered(R::Creature) }),
        },
        ..Default::default()
    }
}

/// Failed Fording — {1}{U} Instant. Return target nonland permanent to hand; if
/// you control a Desert, surveil 1.
pub fn failed_fording() -> CardDefinition {
    CardDefinition {
        name: "Failed Fording",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Nonland),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Desert).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Harrier Strix — {U} 1/1 Bird with flying. ETB taps a permanent; {2}{U}: loot.
pub fn harrier_strix() -> CardDefinition {
    CardDefinition {
        name: "Harrier Strix",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Tap { what: target_filtered(R::Permanent) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Irascible Wolverine — {2}{R} 3/2. ETB exiles the top card; you may play it
/// this turn. Plot {2}{R}.
pub fn irascible_wolverine() -> CardDefinition {
    CardDefinition {
        name: "Irascible Wolverine",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolverine], ..Default::default() },
        power: 3,
        toughness: 2,
        plot_cost: Some(cost(&[generic(2), r()])),
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false, pay_own_cost: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Killer's Mask — {2}{B} Equipment. ETB manifests dread and attaches to that
/// creature; equipped creature has menace. Equip {2}.
pub fn killers_mask() -> CardDefinition {
    CardDefinition {
        name: "Killer's Mask",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Menace], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ManifestDread { who: PlayerRef::You },
            Effect::Attach {
                what: Selector::This,
                to: Selector::take(
                    Selector::EachPermanent(R::FaceDown.and(R::ControlledByYou)),
                    Value::ONE,
                ),
            },
        ]))],
        ..Default::default()
    }
}

/// Jump Scare — {W} Instant. Until end of turn, target creature gets +2/+2 and
/// gains flying. (The Horror-enchantment-creature type rider is cosmetic.)
pub fn jump_scare() -> CardDefinition {
    CardDefinition {
        name: "Jump Scare",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Expel the Interlopers — {3}{W}{W} Sorcery. Choose a number between 0 and 10.
/// Destroy all creatures with power greater than or equal to the chosen number.
pub fn expel_the_interlopers() -> CardDefinition {
    CardDefinition {
        name: "Expel the Interlopers",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseNumberDestroyByPower { max: 10 },
        ..Default::default()
    }
}
