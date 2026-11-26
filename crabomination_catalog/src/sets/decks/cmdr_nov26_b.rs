//! EDHREC single-commander average decks short one card (scan of 2026-11-26,
//! second batch): Sahir, Visitor in Darkness (his own deck), Peace in Our Time
//! (Ambassador K'Ehleyr), Creeping Dread (Armus, Skin of Evil), Dominion
//! Infiltration (Grilka, Widowed Warrior). Tests in `tests/recent_b/cmdr_nov26.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::token_copy_of;
use crate::effect::{Duration, Effect, PlayerRef, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, w, x};

/// Sahir, Visitor in Darkness — {1}{B}{B} legendary 3/3 Human Wizard.
/// Eminence: at the beginning of your upkeep, if Sahir is in the command zone
/// or on the battlefield, it deals 1 damage to you. Whenever a source you
/// control deals exactly 1 damage to you and whenever a spell or ability you
/// control causes you to lose exactly 1 life, put a +1/+1 counter on Sahir;
/// the second resolution this turn also draws. A damage loss is the damage
/// half's alone (`LifeLostToOwnEffect` excludes it), so one ping is one trigger.
pub fn sahir_visitor_in_darkness() -> CardDefinition {
    let exactly_one = || Predicate::ValueEquals(Value::TriggerEventAmount, Value::ONE);
    let grow = || {
        Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::NthResolutionThisTurn {
                branches: vec![Effect::Noop, Effect::Draw { who: Selector::You, amount: Value::ONE }],
            },
        ])
    };
    let mut lost = EventSpec::new(EventKind::LifeLost, EventScope::YourControl).with_filter(exactly_one());
    lost.self_caused = true;
    CardDefinition {
        name: "Sahir, Visitor in Darkness",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                    .in_command_zone(),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::You), amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourSourceDamagedYou)
                    .with_filter(exactly_one()),
                effect: grow(),
            },
            TriggeredAbility { event: lost, effect: grow() },
        ],
        ..Default::default()
    }
}

/// Peace in Our Time — {3}{W} enchantment. At the beginning of combat on each
/// player's turn, that player may draw a card; if they do, they can't attack
/// this turn and you put a peace counter on it. At the beginning of your
/// upkeep, if it has twelve or more peace counters, you win the game.
pub fn peace_in_our_time() -> CardDefinition {
    let twelve = || Predicate::SourceHasCountersAtLeast { counter: CounterType::Peace, n: 12 };
    CardDefinition {
        name: "Peace in Our Time",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
                effect: Effect::MayDoBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Draw a card? (You can't attack this turn.)".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
                        Effect::CantAttackThisTurn {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByActivePlayer)),
                        },
                        Effect::AddCounter { what: Selector::This, kind: CounterType::Peace, amount: Value::ONE },
                    ])),
                },
            },
            // CR 603.4 — intervening "if": checked as it triggers and again
            // as it resolves.
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                    .with_filter(twelve()),
                effect: Effect::If {
                    cond: twelve(),
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Creeping Dread — {3}{B} enchantment. At the beginning of your upkeep, each
/// player discards a card; each opponent who discarded a card that shares a
/// card type with the card you discarded loses 3 life.
pub fn creeping_dread() -> CardDefinition {
    CardDefinition {
        name: "Creeping Dread",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE, random: false },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::If {
                        cond: Predicate::DiscardedSharesCardTypeWith { who: PlayerRef::Triggerer, with: PlayerRef::You },
                        then: Box::new(Effect::LoseLife {
                            who: Selector::Player(PlayerRef::Triggerer),
                            amount: Value::Const(3),
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Dominion Infiltration — {X}{R}{R}{R} sorcery. For each of X target creatures
/// your opponents control, create a token copy of it with changeling (a copy
/// exception, CR 707.9b); the tokens gain haste and are exiled at the next end
/// step.
pub fn dominion_infiltration() -> CardDefinition {
    let mut copy = token_copy_of(PlayerRef::You, Value::ONE, Selector::Target(0));
    if let Effect::CreateTokenCopyOf { extra_keywords, .. } = &mut copy {
        extra_keywords.push(Keyword::Changeling);
    }
    CardDefinition {
        name: "Dominion Infiltration",
        cost: cost(&[x(), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CapTargetsAtX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                    effect: Box::new(copy),
                }),
            },
            Effect::GrantKeyword { what: Selector::LastCreatedTokens, keyword: Keyword::Haste, duration: Duration::Permanent },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}
