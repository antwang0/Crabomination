//! Fallout (PIP) — the rad-counter cards. Tests in `recent_b/pip`.
//!
//! Nuclear Fallout, the fourth, lives in `decks::recent329` — a concurrent
//! session shipped it the same day off the same `audit_variant_coverage` row.
//!
//! CR 122.1i / 728: a rad counter is a counter on a *player*. As their
//! precombat main phase begins, a player with rad counters mills that many
//! cards; for each nonland card milled this way they lose 1 life and remove a
//! rad counter. The turn-based action is `GameState::do_rad_counters`; these
//! are among the first shipped cards that put one on anybody.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::on_dies;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{ManaCost, b, cost, g, generic, u, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Contaminated Drink — {X}{U}{B} Instant. "Draw X cards, then you get half X
/// rad counters, rounded up."
pub fn contaminated_drink() -> CardDefinition {
    CardDefinition {
        name: "Contaminated Drink",
        cost: cost(&[x(), u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::XFromCost },
            Effect::AddRadCounters {
                who: Selector::You,
                amount: Value::HalvedRoundUp(Box::new(Value::XFromCost)),
            },
        ]),
        ..Default::default()
    }
}

/// Glowing One — {2}{G} 2/2 Zombie Mutant. Deathtouch; connects for four rad
/// counters; drains 1 life out of every nonland mill (its own included).
pub fn glowing_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddRadCounters {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(4),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardMilled, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Nonland,
                    },
                ),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Glowing One",
            cost(&[generic(2), g()]),
            vec![CreatureType::Zombie, CreatureType::Mutant],
            2,
            2,
        )
    }
}

/// Feral Ghoul — {2}{B} 2/2 Zombie Mutant. Menace; grows on your other
/// creatures dying, then dumps its power in rad counters on each opponent.
pub fn feral_ghoul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::OtherThanSource,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            // Last known information (CR 603.10e / 608.2h): the power read
            // here is the one it had as it left the battlefield, counters
            // included.
            on_dies(Effect::AddRadCounters {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..creature(
            "Feral Ghoul",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Mutant],
            2,
            2,
        )
    }
}
