//! Return to Ravnica (RTR) gap wave 10: punisher/hellbent enchantments, a
//! reflexive opponent-sac Demon, a death-payoff enchantment, and a magecraft
//! burn enchantment. Tests in `classic_sets/rtr`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, generic, r, u};

/// Shrieking Affliction — {B} Enchantment. At the beginning of each opponent's
/// upkeep, if that player has one or fewer cards in hand, they lose 3 life.
pub fn shrieking_affliction() -> CardDefinition {
    CardDefinition {
        name: "Shrieking Affliction",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl)
                .with_filter(Predicate::ValueAtMost(
                    Value::HandSizeOf(PlayerRef::ActivePlayer),
                    Value::ONE,
                )),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::Const(3) },
        }],
        ..Default::default()
    }
}

/// Desecration Demon — {2}{B}{B} 6/6 Demon with flying. At the beginning of each
/// combat, any opponent may sacrifice a creature; if one does, tap this creature
/// and put a +1/+1 counter on it.
pub fn desecration_demon() -> CardDefinition {
    CardDefinition {
        name: "Desecration Demon",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::PlayersMayAccept {
                who: PlayerRef::EachOpponent,
                description: "Sacrifice a creature to Desecration Demon?".into(),
                on_accept: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::Target(0)),
                        count: Value::ONE,
                        filter: R::Creature,
                    },
                    Effect::Tap { what: Selector::This },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                ])),
                otherwise: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Death's Presence — {5}{G} Enchantment. Whenever a creature you control dies,
/// put X +1/+1 counters on target creature you control, where X is the power of
/// the creature that died.
pub fn deaths_presence() -> CardDefinition {
    CardDefinition {
        name: "Death's Presence",
        cost: cost(&[generic(5), crate::mana::g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Pyroconvergence — {4}{R} Enchantment. Whenever you cast a multicolored spell,
/// this enchantment deals 2 damage to any target.
pub fn pyroconvergence() -> CardDefinition {
    CardDefinition {
        name: "Pyroconvergence",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Multicolored },
            ),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Firemind's Foresight — {5}{U}{R} Instant. Search your library for an instant
/// card with mana value 3, reveal it, and put it into your hand. Repeat for mana
/// values 2 and 1, then shuffle.
pub fn fireminds_foresight() -> CardDefinition {
    let fetch = |mv: u32| Effect::Search {
        who: PlayerRef::You,
        filter: R::HasCardType(CardType::Instant).and(R::ManaValueExactly(mv)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Firemind's Foresight",
        cost: cost(&[generic(5), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![fetch(3), fetch(2), fetch(1)]),
        ..Default::default()
    }
}
