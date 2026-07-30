//! Ravnica (RAV) gap wave 3: more simple creatures — an upkeep-drain flyer, an
//! ETB shrink, a cast-matters payoff, and a pair of tap-ability utility bodies.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, Predicate, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, u};

/// Moroii — {2}{U}{B} 4/4 Vampire with flying. At the beginning of your upkeep,
/// you lose 1 life.
pub fn moroii() -> CardDefinition {
    CardDefinition {
        name: "Moroii",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::LoseLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Keening Banshee — {2}{B}{B} 2/2 Spirit with flying. When it enters, target
/// creature gets -2/-2 until end of turn.
pub fn keening_banshee() -> CardDefinition {
    CardDefinition {
        name: "Keening Banshee",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Primordial Sage — {4}{G}{G} 4/5 Spirit. Whenever you cast a creature spell,
/// you may draw a card.
pub fn primordial_sage() -> CardDefinition {
    CardDefinition {
        name: "Primordial Sage",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                },
            ),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Junktroller — {4} 0/6 Golem artifact creature with defender. `{T}: Put target
/// card from a graveyard on the bottom of its owner's library.`
pub fn junktroller() -> CardDefinition {
    CardDefinition {
        name: "Junktroller",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 0,
        toughness: 6,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ivy Dancer — {2}{G} 1/2 Dryad Shaman. `{T}: Target creature gains forestwalk
/// until end of turn.`
pub fn ivy_dancer() -> CardDefinition {
    CardDefinition {
        name: "Ivy Dancer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Landwalk(LandType::Forest),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lore Broker — {1}{U} 1/2 Human Rogue. `{T}: Each player draws a card, then
/// discards a card.`
pub fn lore_broker() -> CardDefinition {
    CardDefinition {
        name: "Lore Broker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stoneshaker Shaman — {2}{R} 1/1 Human Shaman. At the beginning of each
/// player's end step, that player sacrifices an untapped land of their choice.
pub fn stoneshaker_shaman() -> CardDefinition {
    CardDefinition {
        name: "Stoneshaker Shaman",
        cost: cost(&[generic(2), crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::Land.and(R::Untapped),
            },
        }],
        ..Default::default()
    }
}
