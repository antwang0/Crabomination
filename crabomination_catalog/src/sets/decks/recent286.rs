//! Class enchantment cards (CR 716) — the Bloomburrow Talent cycle
//! (Stormchaser's, Gossip's, Hunter's, Scavenger's, Bandit's) plus AFR's Wizard
//! and Cleric Class. Each is an Enchantment — Class that enters at level 1 and gains levels
//! at sorcery speed via its `{cost}: Level N` activated abilities
//! (`Effect::AdvanceClassLevel`, gated on `Predicate::SourceClassLevelIs`).
//! Higher-level abilities are gated on `Predicate::SourceClassLevelAtLeast`.
//! Tests in `tests/recent_b/recent286.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, StaticEffect,
    ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, u, w, Color};

/// Convenience: an `Enchantment — Class` subtype block.
fn class_subtypes() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() }
}

/// A 1/1 blue and red Otter with prowess (Bloomburrow's Otter token).
fn otter_prowess_token() -> TokenDefinition {
    TokenDefinition {
        name: "Otter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Otter], ..Default::default() },
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

/// The `{cost}: Level N. Activate only as a sorcery.` level-up ability — legal
/// only from level `N - 1`.
fn level_up(mana: &[crate::mana::ManaSymbol], from_level: u8) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(mana),
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from_level)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    }
}

/// Stormchaser's Talent — {U} Enchantment — Class.
/// L1: create a 1/1 Otter with prowess. L2: return target instant/sorcery from
/// your graveyard to your hand. L3: whenever you cast an instant or sorcery,
/// create a 1/1 Otter with prowess.
pub fn stormchasers_talent() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser's Talent",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: otter_prowess_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::Move {
                    what: target_filtered(
                        R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        cast_is_instant_or_sorcery(),
                        Predicate::SourceClassLevelAtLeast(3),
                    ]),
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: otter_prowess_token(),
                },
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(3), u()], 1),
            level_up(&[generic(5), u()], 2),
        ],
        ..Default::default()
    }
}

/// Gossip's Talent — {1}{U} Enchantment — Class.
/// L1: whenever a creature you control enters, surveil 1. L2: whenever you
/// attack, target attacking creature with power 3 or less can't be blocked
/// this turn. L3: whenever a creature you control deals combat damage to a
/// player, you may exile it, then return it under its owner's control.
pub fn gossips_talent() -> CardDefinition {
    CardDefinition {
        name: "Gossip's Talent",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::IsAttacking.and(R::PowerAtMost(3))),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::MayDo {
                    description: "Exile it, then return it under its owner's control".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Exile { what: Selector::TriggerSource },
                        Effect::Move {
                            what: Selector::TriggerSource,
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                                tapped: false,
                            },
                        },
                    ])),
                },
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(1), u()], 1),
            level_up(&[generic(3), u()], 2),
        ],
        ..Default::default()
    }
}

/// Hunter's Talent — {1}{G} Enchantment — Class.
/// L1: target creature you control deals damage equal to its power to target
/// creature you don't control. L2: whenever you attack, target attacking
/// creature gets +1/+0 and gains trample. L3: at the beginning of your end
/// step, if you control a creature with power 4+, draw a card.
pub fn hunters_talent() -> CardDefinition {
    CardDefinition {
        name: "Hunter's Talent",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::IsAttacking),
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                    .with_filter(Predicate::All(vec![
                        Predicate::SourceClassLevelAtLeast(3),
                        Predicate::SelectorExists(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                        )),
                    ])),
                effect: Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(1), g()], 1),
            level_up(&[generic(3), g()], 2),
        ],
        ..Default::default()
    }
}

/// Scavenger's Talent — {B} Enchantment — Class.
/// L1: whenever one or more creatures you control die, create a Food (once each
/// turn). L2: whenever you sacrifice a permanent, target player mills two. L3:
/// at the beginning of your end step, you may sacrifice three other nonland
/// permanents; if you do, return a creature card from your graveyard to the
/// battlefield with a finality counter on it.
pub fn scavengers_talent() -> CardDefinition {
    CardDefinition {
        name: "Scavenger's Talent",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .once_per_turn(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::food_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice three other nonland permanents".into(),
                    filter: R::ControlledByYou.and(R::Land.negate()),
                    count: Value::Const(3),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::AddCounter {
                            what: Selector::LastMoved,
                            kind: CounterType::Finality,
                            amount: Value::ONE,
                        },
                    ])),
                    else_: None,
                },
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(1), b()], 1),
            level_up(&[generic(2), b()], 2),
        ],
        ..Default::default()
    }
}

/// Bandit's Talent — {1}{B} Enchantment — Class.
/// L1: each opponent discards two cards unless they discard a nonland card.
/// L2: at each opponent's upkeep, if that player has one or fewer cards in
/// hand, they lose 2 life. L3: at the beginning of your draw step, draw an
/// extra card for each opponent with one or fewer cards in hand.
pub fn bandits_talent() -> CardDefinition {
    CardDefinition {
        name: "Bandit's Talent",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DiscardUnlessKind {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(2),
                    instead: R::Land.negate(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl)
                    .with_filter(Predicate::All(vec![
                        Predicate::SourceClassLevelAtLeast(2),
                        Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::ActivePlayer), Value::Const(1)),
                    ])),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::ActivePlayer)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::OpponentsWithHandSizeAtMost(1),
                },
            },
        ],
        activated_abilities: vec![
            level_up(&[b()], 1),
            level_up(&[generic(3), b()], 2),
        ],
        ..Default::default()
    }
}

/// Wizard Class — {U} Enchantment — Class (AFR, CR 716).
/// L1: you have no maximum hand size. L2: when this becomes level 2, draw two
/// cards. L3: whenever you draw a card, put a +1/+1 counter on target creature
/// you control.
pub fn wizard_class() -> CardDefinition {
    CardDefinition {
        name: "Wizard Class",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(2), u()], 1),
            level_up(&[generic(4), u()], 2),
        ],
        ..Default::default()
    }
}

/// Cleric Class — {W} Enchantment — Class (AFR, CR 716).
/// L1: if you would gain life, gain that much plus 1. L2: whenever you gain
/// life, put a +1/+1 counter on target creature you control. L3: when this
/// becomes level 3, return target creature card from your graveyard to the
/// battlefield and gain life equal to its toughness.
pub fn cleric_class() -> CardDefinition {
    use crate::effect::PlayerStaticTarget;
    CardDefinition {
        name: "Cleric Class",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        static_abilities: vec![StaticAbility {
            description: "Life gain is increased by 1",
            effect: StaticEffect::LifeGainBonus { target: PlayerStaticTarget::Controller, amount: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(3)),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::ToughnessOf(Box::new(Selector::LastMoved)),
                    },
                ]),
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(3), w()], 1),
            level_up(&[generic(4), w()], 2),
        ],
        ..Default::default()
    }
}
