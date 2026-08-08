//! A Wilds of Eldraine (WOE) wave: Adventures, Celebration, Bargain, and cast-
//! Adventure payoffs. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent137.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, PlayerRef, ZoneDest,
    ZoneRef,
};
use crate::game::effects::food_token;
use crate::mana::{b, cost, g, generic, r, u, w};

// ── White ─────────────────────────────────────────────────────────────────────

/// Pests of Honor — {2}{W} 2/2 Mouse. Celebration — at combat on your turn, if
/// two or more nonland permanents entered under your control this turn, put a
/// +1/+1 counter on it.
pub fn pests_of_honor() -> CardDefinition {
    CardDefinition {
        name: "Pests of Honor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::CelebrationActive {
                who: PlayerRef::You,
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Shrouded Shepherd // Cleave Shadows — {1}{W} 2/2 Spirit Warrior; ETB target
/// creature you control gets +2/+2 until end of turn. Adventure {1}{B} Sorcery:
/// creatures your opponents control get -1/-1 until end of turn.
pub fn shrouded_shepherd() -> CardDefinition {
    CardDefinition {
        name: "Shrouded Shepherd",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        adventure: Some(Box::new(Adventure {
            name: "Cleave Shadows",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::PumpPT {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        })),
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

// ── Blue ──────────────────────────────────────────────────────────────────────

/// Storyteller Pixie — {3}{U} 3/3 Faerie Wizard with flying. Whenever you cast
/// an Adventure spell, draw a card.
pub fn storyteller_pixie() -> CardDefinition {
    CardDefinition {
        name: "Storyteller Pixie",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellIsAdventure),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Obyra's Attendants // Desperate Parry — {4}{U} 3/4 Faerie Wizard with flying.
/// Adventure {1}{U} Instant: target creature gets -4/-0 until end of turn.
pub fn obyras_attendants() -> CardDefinition {
    CardDefinition {
        name: "Obyra's Attendants",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Desperate Parry",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        })),
        ..Default::default()
    }
}

// ── Black ──────────────────────────────────────────────────────────────────────

/// High Fae Negotiator — {3}{B}{B} 3/5 Faerie Warlock with flying and Bargain.
/// ETB, if bargained, each opponent loses 3 life and you gain 3 life.
pub fn high_fae_negotiator() -> CardDefinition {
    CardDefinition {
        name: "High Fae Negotiator",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Bargain],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasBargained),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Fell Horseman // Deathly Ride — {3}{B} 3/3 Zombie Knight; when it dies, put it
/// on the bottom of its owner's library. Adventure {1}{B} Sorcery: return target
/// creature card from your graveyard to your hand.
pub fn fell_horseman() -> CardDefinition {
    CardDefinition {
        name: "Fell Horseman",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        adventure: Some(Box::new(Adventure {
            name: "Deathly Ride",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::Bottom,
                },
            },
        }],
        ..Default::default()
    }
}

// ── Red ───────────────────────────────────────────────────────────────────────

/// Minecart Daredevil // Ride the Rails — {2}{R} 4/2 Dwarf Knight. Adventure
/// {1}{R} Instant: target creature gets +2/+1 until end of turn.
pub fn minecart_daredevil() -> CardDefinition {
    CardDefinition {
        name: "Minecart Daredevil",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        adventure: Some(Box::new(Adventure {
            name: "Ride the Rails",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        })),
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Intrepid Trufflesnout // Go Hog Wild — {1}{G} 3/1 Boar; when it attacks alone,
/// create a Food. Adventure {1}{G} Instant: target creature gets +2/+2 until end
/// of turn.
pub fn intrepid_trufflesnout() -> CardDefinition {
    CardDefinition {
        name: "Intrepid Trufflesnout",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        adventure: Some(Box::new(Adventure {
            name: "Go Hog Wild",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Instant],
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(food_token()),
            },
        }],
        ..Default::default()
    }
}
