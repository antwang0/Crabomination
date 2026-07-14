//! A Wilds of Eldraine (WOE) wave: Adventures, stun/tap control, Roles, and
//! graveyard value. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent134.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};
use super::woe_roles::{wicked_role, young_hero_role};



fn white_human_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    }
}

// ── Blue ──────────────────────────────────────────────────────────────────────

/// Beluna's Gatekeeper // Entry Denied — {5}{U} 6/5 Giant Soldier; Adventure
/// {1}{U} Sorcery returns target creature you don't control with mana value 3
/// or less to its owner's hand.
pub fn belunas_gatekeeper() -> CardDefinition {
    CardDefinition {
        name: "Beluna's Gatekeeper",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        adventure: Some(Box::new(Adventure {
            name: "Entry Denied",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        })),
        ..Default::default()
    }
}

/// Freeze in Place — {1}{U} Sorcery. Tap target creature an opponent controls
/// and put three stun counters on it. Scry 2.
pub fn freeze_in_place() -> CardDefinition {
    CardDefinition {
        name: "Freeze in Place",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::Const(3) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Succumb to the Cold — {2}{U} Instant. Tap one or two target creatures an
/// opponent controls and put a stun counter on each.
pub fn succumb_to_the_cold() -> CardDefinition {
    CardDefinition {
        name: "Succumb to the Cold",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByOpponent),
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            ])),
        },
        ..Default::default()
    }
}

// ── Red ───────────────────────────────────────────────────────────────────────

/// Bellowing Bruiser // Beat a Path — {4}{R} 4/4 Ogre with haste; Adventure
/// {2}{R} Sorcery makes up to two target creatures unable to block this turn.
pub fn bellowing_bruiser() -> CardDefinition {
    CardDefinition {
        name: "Bellowing Bruiser",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ogre], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        adventure: Some(Box::new(Adventure {
            name: "Beat a Path",
            cost: cost(&[generic(2), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
            },
        })),
        ..Default::default()
    }
}

// ── White ─────────────────────────────────────────────────────────────────────

/// Gallant Pie-Wielder — {2}{W} 2/3 Dwarf Knight with first strike. Celebration
/// — has double strike while two or more nonland permanents entered under your
/// control this turn.
pub fn gallant_pie_wielder() -> CardDefinition {
    CardDefinition {
        name: "Gallant Pie-Wielder",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Celebration — Gallant Pie-Wielder has double strike while two or more nonland permanents entered under your control this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CelebrationActive { who: PlayerRef::You },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
            },
        }],
        ..Default::default()
    }
}

/// Woodland Acolyte // Mend the Wilds — {2}{W} 2/2 Human Cleric; ETB draw a
/// card. Adventure {G} Instant puts a target permanent card from your graveyard
/// on top of your library.
pub fn woodland_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Woodland Acolyte",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        adventure: Some(Box::new(Adventure {
            name: "Mend the Wilds",
            cost: cost(&[g()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Top },
            },
        })),
        ..Default::default()
    }
}

/// Stroke of Midnight — {2}{W} Instant. Destroy target nonland permanent. Its
/// controller creates a 1/1 white Human creature token.
pub fn stroke_of_midnight() -> CardDefinition {
    CardDefinition {
        name: "Stroke of Midnight",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::ONE,
                definition: white_human_token(),
            },
            Effect::Destroy { what: target_filtered(R::Nonland) },
        ]),
        ..Default::default()
    }
}

/// Return Triumphant — {1}{W} Sorcery. Return target creature card with mana
/// value 3 or less from your graveyard to the battlefield, then create a Young
/// Hero Role token attached to it.
pub fn return_triumphant() -> CardDefinition {
    CardDefinition {
        name: "Return Triumphant",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::CreateTokenAttachedTo {
                target: Selector::LastMoved,
                definition: young_hero_role(),
            },
        ]),
        ..Default::default()
    }
}

// ── Black ─────────────────────────────────────────────────────────────────────

/// Conceited Witch // Price of Beauty — {2}{B} 2/3 Human Warlock with menace;
/// Adventure {B} Sorcery creates a Wicked Role token attached to target creature
/// you control.
pub fn conceited_witch() -> CardDefinition {
    CardDefinition {
        name: "Conceited Witch",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        adventure: Some(Box::new(Adventure {
            name: "Price of Beauty",
            cost: cost(&[b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: wicked_role(),
            },
        })),
        ..Default::default()
    }
}

/// Sugar Rush — {1}{B} Instant. Target creature gets +3/+0 until end of turn.
/// Draw a card.
pub fn sugar_rush() -> CardDefinition {
    CardDefinition {
        name: "Sugar Rush",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}
