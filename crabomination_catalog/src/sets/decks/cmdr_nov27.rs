//! EDHREC single-commander average decks short only their commander (scan of
//! 2026-11-27): Itazura, Lingering Wick; Miss Highwater. Tests in `tests/recent_b/cmdr_nov26.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{Effect, PlayerRef, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u};

/// Itazura, Lingering Wick — {2}{U}{R} legendary 1/5 Spirit. At the beginning
/// of your upkeep, exile the top three cards of your library; each opponent
/// secretly bids a number, the highest takes that much damage and may cast one
/// of them free, and you put one that wasn't cast into your hand.
pub fn itazura_lingering_wick() -> CardDefinition {
    CardDefinition {
        name: "Itazura, Lingering Wick",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(3) },
                    to: ZoneDest::Exile,
                },
                Effect::OpponentsBidDamageForFreeCast { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
        }],
        ..Default::default()
    }
}

/// Miss Highwater — {2}{B}{B} legendary 5/3 Demon Advisor with menace.
/// Combat damage to a player without a contract counter offers them a wheel
/// (discard their hand, draw seven) for a contract counter: while they hold
/// it, when they lose the game, you create a token copy of each artifact and
/// creature they controlled (`GameState::queue_contract_copies`).
pub fn miss_highwater() -> CardDefinition {
    let them = || Selector::Player(PlayerRef::TriggerEventPlayer);
    CardDefinition {
        name: "Miss Highwater",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon, CreatureType::Advisor], ..Default::default() },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).with_filter(
                Predicate::ValueEquals(Value::ContractCountersOf(PlayerRef::TriggerEventPlayer), Value::Const(0)),
            ),
            effect: Effect::MayDoBy {
                who: PlayerRef::TriggerEventPlayer,
                description: "Discard your hand and draw seven cards? (You get a contract counter.)".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: them(), amount: Value::Const(100), random: false },
                    Effect::Draw { who: them(), amount: Value::Const(7) },
                    Effect::GiveContractCounter {
                        who: PlayerRef::TriggerEventPlayer,
                        beneficiary: PlayerRef::ControllerOf(Box::new(Selector::This)),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}
