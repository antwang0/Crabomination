//! EDHREC single-commander average decks short only their commander (scan of
//! 2026-11-27): Itazura, Lingering Wick; Miss Highwater. Tests in `tests/recent_b/cmdr_nov26.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{Duration, Effect, PlayerRef, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u};

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

/// Sash and Waistcoat, Unmen — {1}{U} legendary 2/2 Insect Rogue. Whenever a
/// face-down creature attacks one of your opponents, it can't be blocked this
/// combat. {3}: manifest the top card of your library — any player may
/// activate it, as a sorcery (the activator is "you").
pub fn sash_and_waistcoat_unmen() -> CardDefinition {
    CardDefinition {
        name: "Sash and Waistcoat, Unmen",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::FaceDown.and(R::IsAttackingOpponentPlayer),
            }),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            sorcery_speed: true,
            effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Write into Being — {2}{U} sorcery. Look at the top two cards of your
/// library; manifest one, put the other on the top or bottom.
pub fn write_into_being() -> CardDefinition {
    CardDefinition {
        name: "Write into Being",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ManifestOneOfTopTwoRestTopOrBottom { who: PlayerRef::You },
        ..Default::default()
    }
}

/// Glava, Five-Advents Mage — {5}{G}{G} legendary 5/5 Human Wizard. Whenever
/// you cast a permanent spell with {X} in its mana cost or activate a non-mana
/// ability with {X} in its cost, you may have X become 5, once each turn
/// (taken when it raises X).
pub fn glava_five_advents_mage() -> CardDefinition {
    let five = || Effect::RaiseXOfTriggeringOncePerTurn { to: 5 };
    CardDefinition {
        name: "Glava, Five-Advents Mage",
        cost: cost(&[generic(5), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::CastSpellHasX,
                    Predicate::CastSpellMatches(R::PermanentCard),
                ])),
                effect: five(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl)
                    .with_filter(Predicate::ActivationCostHasX),
                effect: five(),
            },
        ],
        ..Default::default()
    }
}

/// Savage Summoning — {G} instant; can't be countered. Your next creature
/// spell this turn has flash, can't be countered, and enters with an
/// additional +1/+1 counter.
pub fn savage_summoning() -> CardDefinition {
    CardDefinition {
        name: "Savage Summoning",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::NextCreatureSpellHasFlashThisTurn,
            Effect::NextSpellCantBeCountered { filter: R::Creature },
            Effect::GrantNextCreatureSpellCounters { kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}
