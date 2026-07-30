//! Tarkir: Dragonstorm / Aetherdrift staples that were deferred for want of a
//! primitive and are now unblocked: the counters-placed event
//! (`EventKind::AnyCounterAdded`, Stalwart Successor), the becomes-targeted
//! event (Surrak), and a cast-count-gated ETB (Effortless Master). Tests in
//! `tests/recent102.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{b, cost, g, generic, r, u};

/// Surrak, Elusive Hunter — {2}{G} 4/3 legend. Can't be countered, trample.
/// Whenever a creature you control becomes the target of an opponent's spell or
/// ability, draw a card. (The "creature spell you control" half is dropped.)
pub fn surrak_elusive_hunter() -> CardDefinition {
    CardDefinition {
        name: "Surrak, Elusive Hunter",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantBeCountered, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature,
            }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Effortless Master — {2}{U}{R} 4/3 Orc Monk, vigilance, menace. Enters with
/// two +1/+1 counters if you've cast two or more spells this turn.
pub fn effortless_master() -> CardDefinition {
    CardDefinition {
        name: "Effortless Master",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Monk],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellsCastThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(2),
            },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Stalwart Successor — {1}{B}{G} 3/2 Human Warrior, menace. The first time
/// counters are put on each creature you control each turn, put a +1/+1 counter
/// on that creature (`AnyCounterAdded` + a per-subject once-per-turn cap).
pub fn stalwart_successor() -> CardDefinition {
    CardDefinition {
        name: "Stalwart Successor",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                })
                .with_per_subject_cap(1),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}
