//! A Bloomburrow (BLB) wave: modal removal, combat tricks, a conditional
//! threaten, a graveyard-hate coyote, and a sorcery-speed haste-granter. All
//! ride existing primitives. Tests in `crabomination/src/tests/recent149.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, ExileReturnZone, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, g, generic, r, w};

/// Driftgloom Coyote — {3}{W}{W} 3/4. ETB exiles an opposing creature until this
/// leaves; if it had power 2 or less, this grows with a +1/+1 counter.
pub fn driftgloom_coyote() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Driftgloom Coyote",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Coyote],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::PowerAtMost(2),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::ExileUntilSourceLeaves {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    return_to: ExileReturnZone::Battlefield,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: crate::card::Value::ONE,
                },
            ])),
            else_: Box::new(Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                return_to: ExileReturnZone::Battlefield,
            }),
        })],
        ..Default::default()
    }
}

/// Early Winter — {4}{B} Instant. Choose one — exile target creature; or an
/// opponent exiles an enchantment they control (modeled as exile target
/// opposing enchantment).
pub fn early_winter() -> CardDefinition {
    CardDefinition {
        name: "Early Winter",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            Effect::Exile {
                what: target_filtered(R::Enchantment.and(R::ControlledByOpponent)),
            },
        ]),
        ..Default::default()
    }
}

/// High Stride — {G} Instant. Target creature gets +1/+3, gains reach, and
/// untaps.
pub fn high_stride() -> CardDefinition {
    CardDefinition {
        name: "High Stride",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: crate::card::Value::ONE,
                toughness: crate::card::Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Mabel's Mettle — {1}{W} Instant. Target creature gets +2/+2; up to one other
/// target creature gets +1/+1.
pub fn mabels_mettle() -> CardDefinition {
    CardDefinition {
        name: "Mabel's Mettle",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: crate::card::Value::Const(2),
                toughness: crate::card::Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: crate::card::Value::ONE,
                toughness: crate::card::Value::ONE,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Playful Shove — {1}{R} Sorcery. Deal 1 damage to any target, then draw a
/// card.
pub fn playful_shove() -> CardDefinition {
    CardDefinition {
        name: "Playful Shove",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Bare `Target(0)` is an any-target (creature / player / walker), as
            // Lightning Strike models direct burn.
            Effect::DealDamage { to: Selector::Target(0), amount: crate::card::Value::ONE },
            Effect::Draw { who: Selector::You, amount: crate::card::Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Psychic Whorl — {2}{B} Sorcery. Target opponent discards two cards; if you
/// control a Rat, surveil 2.
pub fn psychic_whorl() -> CardDefinition {
    CardDefinition {
        name: "Psychic Whorl",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: crate::card::Value::Const(2),
                random: false,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Rat).and(R::ControlledByYou),
                    ),
                    n: crate::card::Value::ONE,
                },
                then: Box::new(Effect::Surveil {
                    who: PlayerRef::You,
                    amount: crate::card::Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Reptilian Recruiter — {3}{R}{R} 4/2 Trample. ETB: if target creature's power
/// is 2 or less or you control another Lizard, take it until end of turn,
/// untapped and hasty.
pub fn reptilian_recruiter() -> CardDefinition {
    CardDefinition {
        name: "Reptilian Recruiter",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::Any(vec![
                Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::PowerAtMost(2),
                },
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Lizard)
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource),
                    ),
                    n: crate::card::Value::ONE,
                },
            ]),
            // Slot 0's filter lives on the first targeting effect in `then`.
            then: Box::new(Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature),
                    to: Some(PlayerRef::You),
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Raccoon Rallier — {1}{R} 2/2. {T}: target creature you control gains haste
/// until end of turn. Activate only as a sorcery.
pub fn raccoon_rallier() -> CardDefinition {
    CardDefinition {
        name: "Raccoon Rallier",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
