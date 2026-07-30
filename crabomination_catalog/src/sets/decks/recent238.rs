//! DSK + OTJ gap batch — Survival, an additional-cost Eye, two OTJ Spree
//! spells, and a from-hand-gated Squirrel. Tests in `tests/recent238.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, SelectionRequirement as R, Subtypes,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, SpreeMode,
    Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, u, w};

fn spree_mode(c: ManaCost, effect: Effect) -> SpreeMode {
    SpreeMode { cost: c, effect }
}

/// Prized Griffin — {4}{W} Griffin 3/4. Flying.
pub fn prized_griffin() -> CardDefinition {
    CardDefinition {
        name: "Prized Griffin",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Abhorrent Oculus — {2}{U} Eye 5/5. Additional cost: exile six cards from
/// your graveyard. Flying. At the beginning of each opponent's upkeep,
/// manifest dread.
pub fn abhorrent_oculus() -> CardDefinition {
    CardDefinition {
        name: "Abhorrent Oculus",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eye],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![AdditionalCastCost::ExileFromGraveyard {
            filter: R::Any,
            count: 6,
        }],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::ManifestDread {
                who: PlayerRef::You,
            },
        }],
        ..Default::default()
    }
}

/// Lively Dirge — {1}{B} Sorcery. Spree: +{1} search your library for a card,
/// put it into your graveyard, shuffle; +{2} return up to two creature cards
/// with total mana value 4 or less from your graveyard to the battlefield.
pub fn lively_dirge() -> CardDefinition {
    CardDefinition {
        name: "Lively Dirge",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Spree {
            modes: vec![
                spree_mode(
                    cost(&[generic(1)]),
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Any,
                        to: ZoneDest::Graveyard,
                    },
                ),
                spree_mode(
                    cost(&[generic(2)]),
                    Effect::ReturnGraveyardCreaturesUpToTotalManaValue {
                        max_total: Value::Const(4),
                        max_count: Value::Const(2),
                        counters: 0,
                    },
                ),
            ],
        },
        ..Default::default()
    }
}

/// Smuggler's Surprise — {G} Instant. Spree: +{2} mill four, take up to two
/// creature/land cards; +{4}{G} put up to two creature cards from your hand
/// onto the battlefield; +{1} creatures you control with power 4+ gain
/// hexproof and indestructible until end of turn.
pub fn smugglers_surprise() -> CardDefinition {
    let big = || R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4));
    CardDefinition {
        name: "Smuggler's Surprise",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Spree {
            modes: vec![
                spree_mode(
                    cost(&[generic(2)]),
                    Effect::MillThenToHandN {
                        amount: Value::Const(4),
                        filter: R::Creature.or(R::Land),
                        take: Value::Const(2),
                    },
                ),
                spree_mode(
                    cost(&[generic(4), g()]),
                    Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Creature,
                        count: Value::Const(2),
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                    },
                ),
                spree_mode(
                    cost(&[generic(1)]),
                    Effect::Seq(vec![
                        Effect::GrantKeyword {
                            what: Selector::EachPermanent(big()),
                            keyword: Keyword::Hexproof,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::EachPermanent(big()),
                            keyword: Keyword::Indestructible,
                            duration: Duration::EndOfTurn,
                        },
                    ]),
                ),
            ],
        },
        ..Default::default()
    }
}

/// Prairie Dog — {1}{W} Squirrel 2/2. Lifelink. At your end step, if you
/// haven't cast a spell from your hand this turn, put a +1/+1 counter on it.
/// {4}{W}: until end of turn, +1/+1 counter placements on your creatures get
/// an extra counter.
pub fn prairie_dog() -> CardDefinition {
    CardDefinition {
        name: "Prairie Dog",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::NoSpellCastFromHandThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::GrantExtraPlusOneCountersThisTurn {
                who: PlayerRef::You,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
