//! Class enchantment cards (CR 716) — the Bloomburrow Talent cycle
//! (Stormchaser's, Gossip's, Hunter's, Scavenger's, Bandit's) plus AFR's Wizard,
//! Cleric, and Warlock Class. Each is an Enchantment — Class that enters at level 1 and gains levels
//! at sorcery speed via its `{cost}: Level N` activated abilities
//! (`Effect::AdvanceClassLevel`, gated on `Predicate::SourceClassLevelIs`).
//! Higher-level abilities are gated on `Predicate::SourceClassLevelAtLeast`.
//! Tests in `tests/recent_b/recent286.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticEffect, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Convenience: an `Enchantment — Class` subtype block.
fn class_subtypes() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![EnchantmentSubtype::Class],
        ..Default::default()
    }
}

/// A 1/1 blue and red Otter with prowess (Bloomburrow's Otter token).
fn otter_prowess_token() -> TokenDefinition {
    TokenDefinition {
        name: "Otter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter],
            ..Default::default()
        },
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
                effect: Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
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
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                )
                .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::MayDo {
                    description: "Exile it, then return it under its owner's control".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Exile {
                            what: Selector::TriggerSource,
                        },
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
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::All(vec![
                    Predicate::SourceClassLevelAtLeast(3),
                    Predicate::SelectorExists(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    )),
                ])),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::ONE,
                },
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
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice three other nonland permanents".into(),
                    filter: R::ControlledByYou.and(R::Land.negate()),
                    count: Value::Const(3),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
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
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::OpponentControl,
                )
                .with_filter(Predicate::All(vec![
                    Predicate::SourceClassLevelAtLeast(2),
                    Predicate::ValueAtMost(
                        Value::HandSizeOf(PlayerRef::ActivePlayer),
                        Value::Const(1),
                    ),
                ])),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::OpponentsWithHandSizeAtMost(1),
                },
            },
        ],
        activated_abilities: vec![level_up(&[b()], 1), level_up(&[generic(3), b()], 2)],
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
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(2),
                },
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
            effect: StaticEffect::LifeGainBonus {
                target: PlayerStaticTarget::Controller,
                amount: 1,
            },
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
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
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

/// Warlock Class — {B} Enchantment — Class (AFR, CR 716).
/// L1: at your end step, if a creature died this turn, each opponent loses 1
/// life. L2: when this becomes level 2, look at the top three cards of your
/// library, put one into your hand and the rest into your graveyard. L3: at
/// your end step, each opponent loses life equal to the life they lost this
/// turn.
pub fn warlock_class() -> CardDefinition {
    CardDefinition {
        name: "Warlock Class",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::ONE,
                }),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::LookTopKeepOneRestToGraveyard {
                    count: Value::Const(3),
                    who: None,
                    exile_rest: false,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::LifeLostThisTurn(PlayerRef::EachOpponent),
                },
            },
        ],
        activated_abilities: vec![
            level_up(&[generic(1), b()], 1),
            level_up(&[generic(6), b()], 2),
        ],
        ..Default::default()
    }
}

/// The "Sword" Equipment token Blacksmith's Talent mints at level 1:
/// colorless artifact, equip {2}, "Equipped creature gets +1/+1".
fn sword_token() -> TokenDefinition {
    TokenDefinition {
        name: "Sword".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Blacksmith's Talent — {R} Enchantment — Class.
/// L1: create a Sword Equipment token (equip {2}, +1/+1). L2: at the beginning
/// of combat on your turn, attach target Equipment you control to target
/// creature you control. L3: during your turn, equipped creatures you control
/// have double strike and haste.
pub fn blacksmiths_talent() -> CardDefinition {
    // L3 grant: double strike + haste to your equipped creatures, gated on
    // level 3 and your turn (CR 716.2 / 611.2).
    let equipped = R::Creature.and(R::ControlledByYou).and(R::IsEquipped);
    let while_l3_your_turn = |kw: Keyword| StaticAbility {
        description: "During your turn, equipped creatures you control have double strike and haste.",
        effect: StaticEffect::WhileClassLevelAtLeast {
            n: 3,
            inner: Box::new(StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(equipped.clone()),
                    keyword: kw,
                }),
            }),
        },
    };
    CardDefinition {
        name: "Blacksmith's Talent",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: sword_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::Attach {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment)
                            .and(R::ControlledByYou),
                    },
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                },
            },
        ],
        static_abilities: vec![
            while_l3_your_turn(Keyword::DoubleStrike),
            while_l3_your_turn(Keyword::Haste),
        ],
        activated_abilities: vec![
            level_up(&[generic(2), r()], 1),
            level_up(&[generic(3), r()], 2),
        ],
        ..Default::default()
    }
}

/// A 0/4 white Wall with defender (Builder's Talent's level-1 token).
fn wall_token() -> TokenDefinition {
    TokenDefinition {
        name: "Wall".into(),
        power: 0,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Builder's Talent — {1}{W} Enchantment — Class.
/// L1: create a 0/4 Wall. L2: whenever a noncreature, nonland permanent you
/// control enters, put a +1/+1 counter on target creature you control. L3:
/// on becoming level 3, return target noncreature, nonland permanent card from
/// your graveyard to the battlefield.
pub fn builders_talent() -> CardDefinition {
    let noncreature_nonland = R::Permanent
        .and(R::HasCardType(CardType::Creature).negate())
        .and(R::Land.negate());
    CardDefinition {
        name: "Builder's Talent",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: wall_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::All(vec![
                        Predicate::SourceClassLevelAtLeast(2),
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: noncreature_nonland.clone(),
                        },
                    ])),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(3)),
                effect: Effect::Move {
                    what: target_filtered(noncreature_nonland.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
        ],
        activated_abilities: vec![level_up(&[w()], 1), level_up(&[generic(4), w()], 2)],
        ..Default::default()
    }
}

/// Caretaker's Talent — {2}{W} Enchantment — Class.
/// L1: whenever one or more tokens you control enter, draw a card (once each
/// turn). L2: on becoming level 2, create a token copy of target token you
/// control. L3: creature tokens you control get +2/+2.
pub fn caretakers_talent() -> CardDefinition {
    CardDefinition {
        name: "Caretaker's Talent",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl)
                    .once_per_turn(),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::IsToken.and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control get +2/+2.",
            effect: StaticEffect::WhileClassLevelAtLeast {
                n: 3,
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::IsToken),
                    ),
                    power: 2,
                    toughness: 2,
                }),
            },
        }],
        activated_abilities: vec![level_up(&[w()], 1), level_up(&[generic(3), w()], 2)],
        ..Default::default()
    }
}

/// Innkeeper's Talent — {1}{G} Enchantment — Class.
/// L1: at the beginning of combat on your turn, put a +1/+1 counter on target
/// creature you control. L2: permanents you control with counters on them have
/// ward {1}. L3: if you would put one or more counters on a permanent or
/// player, put twice that many instead.
pub fn innkeepers_talent() -> CardDefinition {
    CardDefinition {
        name: "Innkeeper's Talent",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: class_subtypes(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Permanents you control with counters on them have ward {1}.",
                effect: StaticEffect::WhileClassLevelAtLeast {
                    n: 2,
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::EachPermanent(
                            R::Permanent.and(R::ControlledByYou).and(R::WithAnyCounter),
                        ),
                        keyword: Keyword::Ward(WardCost::generic(1)),
                    }),
                },
            },
            StaticAbility {
                description: "If you would put counters on a permanent or player, put twice as many instead.",
                effect: StaticEffect::WhileClassLevelAtLeast {
                    n: 3,
                    inner: Box::new(StaticEffect::DoubleCounters),
                },
            },
        ],
        activated_abilities: vec![level_up(&[g()], 1), level_up(&[generic(3), g()], 2)],
        ..Default::default()
    }
}
