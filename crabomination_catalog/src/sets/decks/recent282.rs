//! LTR gap batch 3 — a scry-cantrip, an indestructible-granting Eagle, and a
//! double-bounce with Ring tempt. All on existing primitives. Tests in
//! `tests/recent_b/recent282.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, generic, u, w};

/// Elven Farsight — {G} Sorcery. Scry 3, then you may reveal the top card of
/// your library; if it's a creature card, draw a card.
pub fn elven_farsight() -> CardDefinition {
    CardDefinition {
        name: "Elven Farsight",
        cost: cost(&[crate::mana::g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: R::Creature,
                may_graveyard_miss: false,
            },
        ]),
        ..Default::default()
    }
}

/// Eagle of Deliverance — {4}{W}{W} 5/5 Bird Soldier. Flying. When it enters,
/// put an indestructible counter on another target creature you control; draw a
/// card if that creature's power is 2 or less.
pub fn eagle_of_deliverance() -> CardDefinition {
    CardDefinition {
        name: "Eagle of Deliverance",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                kind: CounterType::Indestructible,
                amount: Value::ONE,
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::PowerAtMost(2) },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Horses of the Bruinen — {3}{U}{U} Sorcery. Return up to two target creatures
/// to their owners' hands. Scry 1. The Ring tempts you.
pub fn horses_of_the_bruinen() -> CardDefinition {
    CardDefinition {
        name: "Horses of the Bruinen",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            Effect::RingTempts { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}
