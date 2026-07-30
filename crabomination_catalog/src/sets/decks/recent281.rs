//! LTR gap batch 2 — a Ring-tempt Ent, a death-cantrip Bird, two combat
//! tricks, and a Goblin/Orc-slaying Knight. All on existing primitives. Tests
//! in `tests/recent_b/recent281.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, Supertype, TriggeredAbility,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, generic, r, u, w};

/// Enraged Huorn — {4}{G} 4/5 Treefolk. Trample. When it enters, the Ring
/// tempts you.
pub fn enraged_huorn() -> CardDefinition {
    CardDefinition {
        name: "Enraged Huorn",
        cost: cost(&[generic(4), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::RingTempts {
            who: PlayerRef::You,
        })],
        ..Default::default()
    }
}

/// Ithilien Kingfisher — {2}{U} 2/1 Bird. Flying. When it dies, draw a card.
pub fn ithilien_kingfisher() -> CardDefinition {
    CardDefinition {
        name: "Ithilien Kingfisher",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Escape from Orthanc — {W} Instant. Target creature gets +1/+3 and gains
/// flying until end of turn. Untap it.
pub fn escape_from_orthanc() -> CardDefinition {
    CardDefinition {
        name: "Escape from Orthanc",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Gimli's Fury — {1}{R} Instant. Target creature gets +3/+2 until end of turn.
/// If it's legendary, it also gains trample until end of turn.
pub fn gimlis_fury() -> CardDefinition {
    CardDefinition {
        name: "Gimli's Fury",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasSupertype(Supertype::Legendary),
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// East-Mark Cavalier — {1}{W} 2/2 Human Knight. Vigilance. Whenever it deals
/// combat damage to a Goblin or Orc, destroy that creature.
pub fn east_mark_cavalier() -> CardDefinition {
    CardDefinition {
        name: "East-Mark Cavalier",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToCreature,
                EventScope::SelfSource,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::HasCreatureType(CreatureType::Goblin)
                    .or(R::HasCreatureType(CreatureType::Orc)),
            }),
            effect: Effect::Destroy {
                what: Selector::Target(0),
            },
        }],
        ..Default::default()
    }
}
