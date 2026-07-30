//! An Outlaws of Thunder Junction (OTJ) batch of commons/uncommons reusing
//! existing primitives: a ward wall with a toughness-fueled pump, a flash
//! -2/-2 body, an ETB mill, a tap-ping, tapped-creature removal, an off-turn
//! discount flyer, and a conditional-attacker Defender. Tests in
//! `tests/recent124.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, Selector, StaticEffect, Value};
use crate::mana::{b, cost, generic, u, w};

/// Armored Armadillo — {W} 0/4 Armadillo with ward {1}. {3}{W}: gets +X/+0 until
/// end of turn, where X is its toughness.
pub fn armored_armadillo() -> CardDefinition {
    CardDefinition {
        name: "Armored Armadillo",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Armadillo],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ToughnessOf(Box::new(Selector::This)),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ambush Gigapede — {4}{B}{B} 6/2 Insect with flash. ETB: target creature an
/// opponent controls gets -2/-2 until end of turn.
pub fn ambush_gigapede() -> CardDefinition {
    CardDefinition {
        name: "Ambush Gigapede",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 6,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Desperate Bloodseeker — {1}{B} 2/2 Vampire with lifelink. ETB: target player
/// mills two.
pub fn desperate_bloodseeker() -> CardDefinition {
    CardDefinition {
        name: "Desperate Bloodseeker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Mill {
            who: target_filtered(R::Player),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Deadeye Duelist — {1}{R} 1/3 Human Assassin with reach. {1}, {T}: deals 1
/// damage to target opponent.
pub fn deadeye_duelist() -> CardDefinition {
    CardDefinition {
        name: "Deadeye Duelist",
        cost: cost(&[generic(1), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eriette's Lullaby — {1}{W} Sorcery. Destroy target tapped creature. You gain
/// 2 life.
pub fn eriettes_lullaby() -> CardDefinition {
    CardDefinition {
        name: "Eriette's Lullaby",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Tapped)),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Geyser Drake — {2}{U} 2/3 Drake with flying. During turns other than yours,
/// spells you cast cost {1} less.
pub fn geyser_drake() -> CardDefinition {
    CardDefinition {
        name: "Geyser Drake",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "During turns other than yours, spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReductionDuringOpponentsTurn {
                filter: R::Any,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Bristlepack Sentry — {1}{G} 3/3 Plant Wolf with defender. It can attack as
/// though it didn't have defender while you control a creature with power 4 or
/// greater.
pub fn bristlepack_sentry() -> CardDefinition {
    CardDefinition {
        name: "Bristlepack Sentry",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wolf],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Can attack as though it didn't have defender while you control a creature with power 4 or greater.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                },
            },
        }],
        ..Default::default()
    }
}
