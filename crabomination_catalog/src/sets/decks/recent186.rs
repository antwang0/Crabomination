//! DSK/BLB gap batch on existing primitives: Vanish from Sight (owner-choice
//! tuck + surveil), Hearthborn Battler (any-player second-spell ping), Inquisitive
//! Glimmer (enchantment cost reduction), Tidecaller Mentor (threshold ETB bounce),
//! and Thought-Stalker Warlock (life-loss-gated hand attack). Tests in
//! `crabomination/src/tests/recent186.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w};

/// Vanish from Sight — {3}{U} Instant. Target nonland permanent's owner puts it
/// on their choice of the top or bottom of their library. Surveil 1.
pub fn vanish_from_sight() -> CardDefinition {
    CardDefinition {
        name: "Vanish from Sight",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Nonland.and(R::Permanent)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::OwnerChoice,
                },
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Hearthborn Battler — {2}{R} 2/3 Lizard Warlock with haste. Whenever a player
/// casts their second spell each turn, it deals 2 damage to target opponent.
pub fn hearthborn_battler() -> CardDefinition {
    CardDefinition {
        name: "Hearthborn Battler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::Triggerer,
                    count: Value::Const(2),
                },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Inquisitive Glimmer — {W}{U} 2/3 Fox Glimmer. Enchantment spells you cast
/// cost {1} less. (The "Unlock costs cost {1} less" half needs Room doors.)
pub fn inquisitive_glimmer() -> CardDefinition {
    CardDefinition {
        name: "Inquisitive Glimmer",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Enchantment spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::Enchantment,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Tidecaller Mentor — {1}{U}{B} 3/3 Rat Wizard with menace. Threshold — when it
/// enters, if there are 7+ cards in your graveyard, return up to one target
/// nonland permanent to its owner's hand.
pub fn tidecaller_mentor() -> CardDefinition {
    CardDefinition {
        name: "Tidecaller Mentor",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::ThresholdActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Nonland.and(R::Permanent),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Thought-Stalker Warlock — {2}{B} 2/2 Lizard Warlock with menace. ETB: if the
/// opponent lost life this turn, they reveal their hand and you choose a nonland
/// card for them to discard; otherwise they discard a card. (Modeled against
/// each opponent — faithful in 1v1.)
pub fn thought_stalker_warlock() -> CardDefinition {
    CardDefinition {
        name: "Thought-Stalker Warlock",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn {
                who: PlayerRef::EachOpponent,
            },
            then: Box::new(Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Nonland,
            }),
            else_: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            }),
        })],
        ..Default::default()
    }
}
