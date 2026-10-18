//! Commander: 2026-10-18 (`019Y6J4R`) closers, eighteenth batch. Tests in
//! `tests/recent_b/cmdr_019y_r.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{Duration, Effect, Selector};
use crate::mana::{ManaCost, b, cost, generic, u};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Veiling Oddity — {3}{U} 2/3 Illusion. Suspend 4—{1}{U}. When the last
/// time counter is removed from this card while it's exiled, creatures can't
/// be blocked this turn (CR 702.62b — the trigger functions from exile; the
/// creatures there as it resolves, CR 611.2c).
pub fn veiling_oddity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(4, cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource)
                .while_suspended()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::WithCounter(CounterType::Time).negate(),
                }),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Veiling Oddity", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 2, 3)
    }
}

/// Jhessian Zombies — {4}{U}{B} 2/4 fear Zombie. Islandcycling {2},
/// swampcycling {2}: two typecycling abilities at one cost, written as one
/// whose search takes an Island or a Swamp card — the same choice, and
/// `GameAction::Landcycle` reads only a card's first cycling keyword.
pub fn jhessian_zombies() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Fear,
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                R::HasLandType(LandType::Island).or(R::HasLandType(LandType::Swamp)),
            ))),
        ],
        ..creature("Jhessian Zombies", cost(&[generic(4), u(), b()]), vec![CreatureType::Zombie], 2, 4)
    }
}
