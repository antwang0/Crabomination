//! Nemesis (NMS), third wave. Tests in `classic_sets/nms3`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, TriggeredAbility, Value,
};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest, shortcut::target_filtered};
use crate::game::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, u, w};

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Parallax Wave — {2}{W}{W}. Fading 5, cashed in one exile at a time; when it
/// goes, everything it took comes back.
pub fn parallax_wave() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(5)],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature),
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
            ..Default::default()
        }],
        ..enchantment("Parallax Wave", cost(&[generic(2), w(), w()]))
    }
}

/// Parallax Inhibitor — {2}. Buys a whole extra turn off every fading
/// permanent you control.
pub fn parallax_inhibitor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::WithCounter(CounterType::Fade).and(R::ControlledByYou),
                ),
                kind: CounterType::Fade,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..artifact("Parallax Inhibitor", cost(&[generic(2)]))
    }
}

/// Accumulated Knowledge — {1}{U}. Draws better the more copies have been cast.
pub fn accumulated_knowledge() -> CardDefinition {
    CardDefinition {
        name: "Accumulated Knowledge",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CardsNamedLikeSourceInAllGraveyards,
            },
        ]),
        ..Default::default()
    }
}

/// Pack Hunt — {3}{G}. Digs up three more copies of whatever you point at.
pub fn pack_hunt() -> CardDefinition {
    CardDefinition {
        name: "Pack Hunt",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::SameNameAsTarget,
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Mind Slash — {1}{B}{B}. Trades creatures for hand-picked discards.
pub fn mind_slash() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::Creature, 1)),
            sorcery_speed: true,
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Any,
            },
            ..Default::default()
        }],
        ..enchantment("Mind Slash", cost(&[generic(1), b(), b()]))
    }
}

/// Rising Waters — {3}{U}. Lands stay tapped; you get one back each upkeep.
pub fn rising_waters() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Lands don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::EachPermanent(R::Land) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Untap {
                what: Selector::ControlledBy { who: PlayerRef::ActivePlayer, filter: R::Land },
                up_to: Some(Value::ONE),
            },
        }],
        ..enchantment("Rising Waters", cost(&[generic(3), u()]))
    }
}
