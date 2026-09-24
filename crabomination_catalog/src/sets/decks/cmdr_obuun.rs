//! Commander: the cards the **Land's Wrath** precon (ZNC, Obuun, Mul Daya
//! Ancestor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_obuun.rs`.
//!
//! Residuals (each also on its card):
//! - **Scaretiller** — the mode is the engine's (a land from hand when there
//!   is one, else the first land card in your graveyard, untargeted).
//! - **The Mending of Dominaria** — chapters I and II return your
//!   greatest-power creature card (the "may" is always taken).
//! - **Trove Warden** — the exiled cards return when it leaves the
//!   battlefield by any route, not only when it dies.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType, SelectionRequirement as R,
    Selector, SplitCard, SplitHalf, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{bolster, support, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{cost, g, generic, r, w};

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn with_counter() -> R {
    R::WithCounter(CounterType::PlusOnePlusOne)
}

/// Landfall — "whenever a land you control enters, `effect`."
fn landfall(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
        effect,
    }
}

fn basic_land() -> R {
    R::Land.and(R::HasSupertype(Supertype::Basic))
}

/// Armorcraft Judge — on entry, draw a card per creature you control with a
/// +1/+1 counter.
pub fn armorcraft_judge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(with_counter()),
                ))),
            },
        }],
        ..creature("Armorcraft Judge", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Artificer], 3, 3)
    }
}

/// Crush Contraband — choose one or both: exile target artifact; exile
/// target enchantment.
pub fn crush_contraband() -> CardDefinition {
    CardDefinition {
        name: "Crush Contraband",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Exile { what: target_filtered(R::Artifact) },
                Effect::Exile { what: target_filtered(R::Enchantment) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Elite Scaleguard — on entry, bolster 2; whenever a creature you control
/// with a +1/+1 counter attacks, tap target creature defending player
/// controls.
pub fn elite_scaleguard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: bolster(2),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: with_counter() },
                ),
                effect: Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) },
            },
        ],
        ..creature(
            "Elite Scaleguard",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Elvish Rejuvenator — on entry, look at the top five; you may put a land
/// from among them onto the battlefield tapped; the rest go to the bottom.
pub fn elvish_rejuvenator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::Const(5),
                filter: R::Land,
                then: None,
                max: Some(1),
                tapped: true,
                exile_rest: false,
            },
        }],
        ..creature("Elvish Rejuvenator", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 1)
    }
}

/// Hour of Revelation — costs {3} less with ten or more nonland permanents
/// on the battlefield; destroy all nonland permanents.
pub fn hour_of_revelation() -> CardDefinition {
    CardDefinition {
        name: "Hour of Revelation",
        cost: cost(&[generic(3), w(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {3} less to cast if there are ten or more nonland permanents on the battlefield.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 3,
                condition: Predicate::ValueAtLeast(
                    Value::CountOf(Box::new(Selector::EachPermanent(R::Nonland))),
                    Value::Const(10),
                ),
            },
        }],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Nonland),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Keeper of Fables — whenever one or more non-Human creatures you control
/// deal combat damage to a player, draw a card.
pub fn keeper_of_fables() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
                })
                .once_per_batch(),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..creature("Keeper of Fables", cost(&[generic(3), g(), g()]), vec![CreatureType::Cat], 4, 5)
    }
}

/// Murasa Rootgrazer — vigilance; {T}: put a basic land from hand onto the
/// battlefield; {T}: return target basic land you control to hand.
pub fn murasa_rootgrazer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: basic_land(),
                    count: Value::Const(1),
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(basic_land().and(R::ControlledByYou)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..creature("Murasa Rootgrazer", cost(&[g(), w()]), vec![CreatureType::Beast], 2, 3)
    }
}

/// Naya Panorama — {T}: {C}; {1}, {T}, sacrifice it: a basic Mountain,
/// Forest or Plains onto the battlefield tapped.
pub fn naya_panorama() -> CardDefinition {
    CardDefinition {
        name: "Naya Panorama",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: basic_land().and(
                        R::HasLandType(LandType::Mountain)
                            .or(R::HasLandType(LandType::Forest))
                            .or(R::HasLandType(LandType::Plains)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Obuun, Mul Daya Ancestor — at the beginning of combat on your turn, up to
/// one target land you control becomes an X/X trampling, hasty Elemental
/// until end of turn (X = Obuun's power); landfall: a +1/+1 counter on
/// target creature.
pub fn obuun_mul_daya_ancestor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::BecomeCreature {
                        what: target_filtered(R::Land.and(R::ControlledByYou)),
                        power: Value::PowerOf(Box::new(Selector::This)),
                        toughness: Value::PowerOf(Box::new(Selector::This)),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![Keyword::Trample, Keyword::Haste],
                        duration: Duration::EndOfTurn,
                    }),
                },
            },
            landfall(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..creature(
            "Obuun, Mul Daya Ancestor",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Elf, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Scaretiller — whenever it becomes tapped, put a land from hand onto the
/// battlefield tapped, or return a land card from your graveyard tapped.
pub fn scaretiller() -> CardDefinition {
    let tapped_bf = || ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Hand,
                    filter: R::Land,
                }),
                then: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Land,
                    count: Value::Const(1),
                    tapped: true,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                }),
                else_: Box::new(Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Land }),
                        count: Box::new(Value::Const(1)),
                    },
                    to: tapped_bf(),
                }),
            },
        }],
        ..creature("Scaretiller", cost(&[generic(4)]), vec![CreatureType::Scarecrow], 1, 4)
    }
}

/// Struggle // Survive — Struggle: damage to target creature equal to the
/// lands you control. Survive (aftermath): each player shuffles their
/// graveyard into their library.
pub fn struggle_survive() -> CardDefinition {
    CardDefinition {
        name: "Struggle // Survive",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Land.and(R::ControlledByYou)))),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
            },
            fuse: false,
            aftermath: true,
        })),
        ..Default::default()
    }
}

/// Sylvan Reclamation — exile up to two target artifacts and/or
/// enchantments; basic landcycling {2}.
pub fn sylvan_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Reclamation",
        cost: cost(&[generic(3), g(), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Artifact.or(R::Enchantment),
            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// The Mending of Dominaria — I, II: mill two, then you may return a
/// creature card from your graveyard to your hand. III: return all land
/// cards from your graveyard to the battlefield, then shuffle your graveyard
/// into your library.
pub fn the_mending_of_dominaria() -> CardDefinition {
    let mill_and_regrow = || {
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::Move {
                what: Selector::TakeGreatestPower {
                    inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature }),
                    count: Box::new(Value::Const(1)),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ])
    };
    CardDefinition {
        name: "The Mending of Dominaria",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, mill_and_regrow()),
            (2, mill_and_regrow()),
            (
                3,
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Land },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Together Forever — on entry, support 2; {1}: when target creature with a
/// counter on it dies this turn, return it to its owner's hand.
pub fn together_forever() -> CardDefinition {
    CardDefinition {
        name: "Together Forever",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: support(2),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                slot: 0,
                filter: Some(R::Creature.and(R::WithAnyCounter)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Trove Warden — vigilance; landfall: exile target permanent card with mana
/// value 3 or less from your graveyard; when it dies, those cards return.
pub fn trove_warden() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![landfall(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::ManaValueAtMost(3)).from_your_graveyard()),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..creature("Trove Warden", cost(&[generic(2), w(), w()]), vec![CreatureType::Cat, CreatureType::Beast], 3, 4)
    }
}

/// Waker of the Wilds — {X}{G}{G}: X +1/+1 counters on target land you
/// control; it becomes a 0/0 hasty Elemental creature that's still a land.
pub fn waker_of_the_wilds() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x(), g(), g()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
                Effect::BecomeCreature {
                    what: Selector::Target(0),
                    power: Value::Const(0),
                    toughness: Value::Const(0),
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Waker of the Wilds",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            3,
            3,
        )
    }
}

