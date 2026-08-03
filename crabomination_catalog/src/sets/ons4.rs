//! Onslaught (ONS) wave 11 — the set's last cards, each on a new primitive.
//! Tests in `classic_sets/ons2`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value,
    shortcut::target_filtered,
};
use crate::game::TurnStep;
use crate::mana::{cost, generic, r, u};

/// Artificial Evolution — CR 612 text change: swap one creature type for
/// another on target spell or permanent, indefinitely. Wall is not a legal
/// replacement.
pub fn artificial_evolution() -> CardDefinition {
    CardDefinition {
        name: "Artificial Evolution",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ReplaceCreatureTypeText {
            what: target_filtered(R::Any.or(R::IsSpellOnStack)),
        },
        ..Default::default()
    }
}

/// Butcher Orgg — divides its combat damage freely among the defending
/// player and any of their creatures, blocked or not.
pub fn butcher_orgg() -> CardDefinition {
    CardDefinition {
        name: "Butcher Orgg",
        cost: cost(&[generic(4), r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Orgg], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::DividesCombatDamageAmongDefenders],
        ..Default::default()
    }
}

/// Risky Move — hops to whoever's upkeep it is; the new controller then
/// gambles one of their own creatures on a coin flip.
pub fn risky_move() -> CardDefinition {
    CardDefinition {
        name: "Risky Move",
        cost: cost(&[generic(3), r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::ActivePlayer),
                    duration: Duration::Permanent,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::GainedControlOfThis, EventScope::SelfSource),
                effect: Effect::FlipCoin {
                    count: Value::Const(1),
                    on_heads: Box::new(Effect::Noop),
                    on_tails: Box::new(Effect::GainControl {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::Creature.and(R::ControlledByYou),
                        },
                        to: Some(PlayerRef::Target(1)),
                        duration: Duration::Permanent,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}
