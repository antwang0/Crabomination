//! OTJ gap batch on existing primitives: Rodeo Pyromancers (first-spell ritual),
//! Scalestorm Summoner (ferocious attack token), Marauding Sphinx (crime
//! surveil), Raucous Entertainer (entered-this-turn counters), and Ruthless
//! Lawbringer (reflexive-sacrifice removal). Tests in
//! `crabomination/src/tests/recent189.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, Selector};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Rodeo Pyromancers — {3}{R} 3/4 Human Mercenary. Whenever you cast your first
/// spell each turn, add {R}{R}.
pub fn rodeo_pyromancers() -> CardDefinition {
    CardDefinition {
        name: "Rodeo Pyromancers",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::ONE,
                },
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::Const(2)),
            },
        }],
        ..Default::default()
    }
}

/// A 3/1 red Dinosaur token.
fn dinosaur_token() -> TokenDefinition {
    TokenDefinition {
        name: "Dinosaur".to_string(),
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        ..Default::default()
    }
}

/// Scalestorm Summoner — {2}{R} 3/3 Human Warlock. Whenever it attacks, if you
/// control a creature with power 4 or greater, create a 3/1 red Dinosaur token.
pub fn scalestorm_summoner() -> CardDefinition {
    CardDefinition {
        name: "Scalestorm Summoner",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::FerociousActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(dinosaur_token()),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Marauding Sphinx — {3}{U}{U} 3/5 Sphinx Rogue with flying, vigilance, ward {2}.
/// Whenever you commit a crime, surveil 2. Once each turn.
pub fn marauding_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Marauding Sphinx",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![
            Keyword::Flying,
            Keyword::Vigilance,
            Keyword::Ward(WardCost::generic(2)),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Raucous Entertainer — {1}{G} 2/2 Plant Bard. {1}, {T}: Put a +1/+1 counter on
/// each creature you control that entered this turn.
pub fn raucous_entertainer() -> CardDefinition {
    CardDefinition {
        name: "Raucous Entertainer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::EnteredThisTurn),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ruthless Lawbringer — {1}{W}{B} 3/2 Vampire Assassin. When it enters, you may
/// sacrifice another creature. When you do, destroy target nonland permanent.
pub fn ruthless_lawbringer() -> CardDefinition {
    CardDefinition {
        name: "Ruthless Lawbringer",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "You may sacrifice another creature.".to_string(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Nonland.and(R::Permanent)),
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}
