//! A second wave of recent-set staples filling small gaps (DFT / MKM / NEO /
//! WOE / DSK / ELD …). Each card has a functionality test in
//! `crabomination/src/tests/recent2.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Tangle — {1}{G} Instant. Prevent all combat damage this turn; each attacking
/// creature doesn't untap during its controller's next untap step.
pub fn tangle() -> CardDefinition {
    CardDefinition {
        name: "Tangle",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventAllCombatDamageThisTurn,
            Effect::SkipNextUntap {
                what: Selector::EachPermanent(SelectionRequirement::IsAttacking),
            },
        ]),
        ..Default::default()
    }
}

/// March of Otherworldly Light — {X}{W} Instant. Exile target artifact, creature,
/// or enchantment with mana value X or less. (The "exile white cards from hand
/// to reduce the cost" additional cost is dropped.)
pub fn march_of_otherworldly_light() -> CardDefinition {
    CardDefinition {
        name: "March of Otherworldly Light",
        cost: cost(&[generic(0), w()]), // {X}{W}; X paid as generic at cast time
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Or(
                    Box::new(SelectionRequirement::Or(
                        Box::new(SelectionRequirement::Artifact),
                        Box::new(SelectionRequirement::Creature),
                    )),
                    Box::new(SelectionRequirement::Enchantment),
                )
                .and(SelectionRequirement::ManaValueAtMostXFromCost),
            },
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Disdainful Stroke — {1}{U} Instant. Counter target spell with mana value 4
/// or greater.
pub fn disdainful_stroke() -> CardDefinition {
    CardDefinition {
        name: "Disdainful Stroke",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::ManaValueAtLeast(4)),
            ),
        },
        ..Default::default()
    }
}

/// Flame Lash — {3}{R} Instant. Deals 4 damage to any target.
pub fn flame_lash() -> CardDefinition {
    CardDefinition {
        name: "Flame Lash",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Virtue of Persistence // Locthwain Scorn — {5}{B}{B} Enchantment with an
/// Adventure. Enchantment: at the beginning of your upkeep, put target creature
/// card from a graveyard onto the battlefield under your control. Adventure
/// (Locthwain Scorn {1}{B} Sorcery): target creature gets -3/-3; you gain 2 life.
pub fn virtue_of_persistence() -> CardDefinition {
    CardDefinition {
        name: "Virtue of Persistence",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::InGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Locthwain Scorn",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        })),
        ..Default::default()
    }
}

/// Scrabbling Skullcrab — {U} 0/3 Crab Skeleton. Eerie — whenever an enchantment
/// you control enters, target player mills two cards. (The "fully unlock a Room"
/// half is dropped — Rooms aren't modeled.)
pub fn scrabbling_skullcrab() -> CardDefinition {
    CardDefinition {
        name: "Scrabbling Skullcrab",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Conduit of Worlds — {2}{G}{G} Artifact. You may play lands from your
/// graveyard. (The "{T}: cast a nonland permanent from your graveyard if you
/// haven't cast a spell this turn" half is dropped — the one-spell lock isn't
/// modeled.)
pub fn conduit_of_worlds() -> CardDefinition {
    CardDefinition {
        name: "Conduit of Worlds",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "You may play lands from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyard,
        }],
        ..Default::default()
    }
}

/// Hush — {3}{G} Sorcery. Destroy all enchantments. Cycling {2}.
pub fn hush() -> CardDefinition {
    CardDefinition {
        name: "Hush",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::DestroyNoRegen {
            what: Selector::EachPermanent(SelectionRequirement::Enchantment),
        },
        ..Default::default()
    }
}

/// Llanowar Greenwidow — {2}{G} 4/3 Spider with reach and trample. {7}{G},
/// exile from graveyard isn't required — return it from your graveyard to the
/// battlefield tapped (sorcery speed). (The Domain cost reduction and the
/// "exile if it would leave" rider are dropped.)
pub fn llanowar_greenwidow() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Llanowar Greenwidow",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), g()]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lord Skitter, Sewer King — {2}{B} 3/3 Legendary Rat Noble. Whenever another
/// Rat you control enters, exile a card from an opponent's graveyard. At the
/// beginning of combat on your turn, create a 1/1 black Rat that can't block.
pub fn lord_skitter_sewer_king() -> CardDefinition {
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    };
    CardDefinition {
        name: "Lord Skitter, Sewer King",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Rat)
                            .and(SelectionRequirement::OtherThanSource),
                    }),
                effect: Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::InGraveyard
                            .and(SelectionRequirement::ControlledByOpponent),
                    },
                    to: ZoneDest::Exile,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: rat,
                },
            },
        ],
        ..Default::default()
    }
}
