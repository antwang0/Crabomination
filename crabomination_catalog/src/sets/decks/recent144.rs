//! A Wilds of Eldraine (WOE) wave: tap-matters payoff (the `YouTapped` scope),
//! high-mana-value triggers, first-spell payoffs, and Adventure utility. Tests
//! in `crabomination/src/tests/recent144.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    CounteredSpellZone, Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition,
    PlayerRef, ZoneDest,
};
use crate::mana::{b, cost, generic, u, w};

/// One opponent creature (for the auto-picked reflexive taps below).
fn an_enemy_creature() -> Selector {
    Selector::take(
        Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
        Value::ONE,
    )
}

/// Icewrought Sentry — {2}{U} 2/3 Elemental Soldier with vigilance. Attacking,
/// pay {1}{U} to tap an opponent's creature; tapping enemy creatures pumps it.
pub fn icewrought_sentry() -> CardDefinition {
    CardDefinition {
        name: "Icewrought Sentry",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            on_attack(Effect::MayPay {
                description: "pay {1}{U} to tap an opponent's creature".into(),
                mana_cost: cost(&[generic(1), u()]),
                body: Box::new(Effect::Tap { what: an_enemy_creature() }),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::YouTapped).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                ),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Galvanic Giant // Storm Reading — {3}{U} 3/3 Giant Wizard; casting a mana
/// value 5+ spell taps and stuns an opponent's creature. Adventure {5}{U}{U}
/// Instant: draw four, discard two.
pub fn galvanic_giant() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Giant",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::ManaValueAtLeast(5))),
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ]),
        }],
        adventure: Some(Box::new(Adventure {
            name: "Storm Reading",
            cost: cost(&[generic(5), u(), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(4) },
                Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            ]),
        })),
        ..Default::default()
    }
}

/// Aquatic Alchemist // Bubble Up — {1}{U} 1/3 Elemental; grows the first time
/// you cast an instant or sorcery each turn. Adventure {2}{U} Sorcery: put an
/// instant or sorcery from your graveyard on top of your library.
pub fn aquatic_alchemist() -> CardDefinition {
    CardDefinition {
        name: "Aquatic Alchemist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery())
                .once_per_turn(),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Bubble Up",
            cost: cost(&[generic(2), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::InGraveyard),
                ),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
        })),
        ..Default::default()
    }
}

/// Threadbind Clique // Rip the Seams — {3}{U} 3/3 Faerie with flying. Adventure
/// {2}{W} Instant: destroy target tapped creature.
pub fn threadbind_clique() -> CardDefinition {
    CardDefinition {
        name: "Threadbind Clique",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Rip the Seams",
            cost: cost(&[generic(2), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
        })),
        ..Default::default()
    }
}

/// Twining Twins // Swift Spiral — {2}{U}{U} 4/4 Faerie Wizard with flying,
/// vigilance, ward {1}. Adventure {1}{W} Instant: flicker a nontoken creature.
pub fn twining_twins() -> CardDefinition {
    CardDefinition {
        name: "Twining Twins",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        adventure: Some(Box::new(Adventure {
            name: "Swift Spiral",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::ExileReturnNextEndStep {
                what: target_filtered(R::Creature.and(R::NotToken)),
            },
        })),
        ..Default::default()
    }
}

/// Spellscorn Coven // Take It Back — {3}{B} 2/3 Faerie Warlock with flying;
/// ETB each opponent discards. Adventure {2}{U} Instant: return target spell to
/// its owner's hand.
pub fn spellscorn_coven() -> CardDefinition {
    CardDefinition {
        name: "Spellscorn Coven",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        adventure: Some(Box::new(Adventure {
            name: "Take It Back",
            cost: cost(&[generic(2), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CounterSpellToZone {
                what: Selector::Target(0),
                zone: CounteredSpellZone::OwnerHand,
            },
        })),
        ..Default::default()
    }
}
