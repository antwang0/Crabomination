//! Small gap batch — a removal-and-token instant, a lifelink dork, a vanilla
//! Cat, a power-matters Boar, and a kicker pump, all on existing primitives.
//! Tests in `tests/recent_b/recent266.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{Color, b, cost, g, generic, w};

/// Fungal Infection — {B} Instant. Target creature gets -1/-1 until end of
/// turn; create a 1/1 green Saproling.
pub fn fungal_infection() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Fungal Infection",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(saproling),
            },
        ]),
        ..Default::default()
    }
}

/// Prakhata Pillar-Bug — {3} 2/3 Insect artifact creature. {B}: gains lifelink
/// until end of turn.
pub fn prakhata_pillar_bug() -> CardDefinition {
    CardDefinition {
        name: "Prakhata Pillar-Bug",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Savai Sabertooth — {1}{W} 3/1 Cat vanilla.
pub fn savai_sabertooth() -> CardDefinition {
    CardDefinition {
        name: "Savai Sabertooth",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        ..Default::default()
    }
}

/// Territorial Boar — {1}{G} 2/2 Boar. Whenever a creature you control with
/// power 4 or greater enters, this gets +1/+1 and gains vigilance until end of
/// turn.
pub fn territorial_boar() -> CardDefinition {
    CardDefinition {
        name: "Territorial Boar",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtLeast(4)),
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Might of Murasa — {1}{G} Instant. Target creature gets +3/+3 (or +5/+5 if
/// kicked) until end of turn. Kicker {2}{G}.
pub fn might_of_murasa() -> CardDefinition {
    CardDefinition {
        name: "Might of Murasa",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), g()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}
