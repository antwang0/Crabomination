//! Return to Ravnica (RTR) gap wave 14 — the last five, all mythic. Tests in
//! `classic_sets/rtr`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector,
    ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, x};

fn walker(
    name: &'static str,
    mana: ManaCost,
    subtype: PlaneswalkerSubtype,
    loyalty: u32,
    abilities: Vec<LoyaltyAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![subtype],
            ..Default::default()
        },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

/// Epic Experiment — {X}{U}{R} Sorcery. Exile the top X cards; cast the
/// instants and sorceries among them with mana value X or less for free; the
/// rest go to your graveyard.
pub fn epic_experiment() -> CardDefinition {
    CardDefinition {
        name: "Epic Experiment",
        cost: cost(&[x(), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileLinked {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::XFromCost,
                },
            },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::CardExiledWithSource,
                source_zone: Zone::Exile,
                filter: Some(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::ManaValueAtMostXFromCost),
                ),
            },
            Effect::Move {
                what: Selector::CardExiledWithSource,
                to: ZoneDest::Graveyard,
            },
        ]),
        ..Default::default()
    }
}

/// Jace, Architect of Thought — {2}{U}{U} planeswalker, loyalty 4. Shrinks
/// attackers, splits three cards with an opponent, or tutors a free spell for
/// each player.
pub fn jace_architect_of_thought() -> CardDefinition {
    walker(
        "Jace, Architect of Thought",
        cost(&[generic(2), u(), u()]),
        PlaneswalkerSubtype::Jace,
        4,
        vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::DelayUntil {
                    kind: DelayedTriggerKind::CreatureAttacksYouUntilYourNextTurn,
                    body: Box::new(Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(-1),
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::FactOrFiction {
                    count: Value::Const(3),
                    to_bottom: true,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::EachPlayer,
                        filter: R::Nonland,
                        to: ZoneDest::Exile,
                    },
                    Effect::CastAnyOrderWithoutPaying {
                        what: Selector::LastMoved,
                        source_zone: Zone::Exile,
                        filter: None,
                    },
                ]),
                ..Default::default()
            },
        ],
    )
}

/// Rakdos, Lord of Riots — {B}{B}{R}{R} 6/6 Demon with flying and trample.
/// Uncastable until an opponent has lost life, then your creature spells cost
/// {1} less per life they've lost this turn.
pub fn rakdos_lord_of_riots() -> CardDefinition {
    CardDefinition {
        name: "Rakdos, Lord of Riots",
        cost: cost(&[b(), b(), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        cast_condition: Some(Predicate::PlayerLostLifeThisTurn {
            who: PlayerRef::EachOpponent,
        }),
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {1} less to cast for each 1 life your \
                          opponents have lost this turn.",
            effect: StaticEffect::CostReductionByValue {
                filter: R::Creature,
                amount: Value::LifeLostThisTurn(PlayerRef::EachOpponent),
            },
        }],
        ..Default::default()
    }
}

/// Search the City — {4}{U} Enchantment. ETB exiles your top five; replaying a
/// card sharing a name with one returns it, and emptying the pile buys an extra
/// turn.
pub fn search_the_city() -> CardDefinition {
    CardDefinition {
        name: "Search the City",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::ExileLinked {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: Effect::SearchTheCityReturn,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::SearchTheCityReturn,
            },
        ],
        ..Default::default()
    }
}

/// Vraska the Unseen — {3}{B}{G} planeswalker, loyalty 5. Kills whatever
/// damages her, blows up a nonland permanent, or makes three assassins.
pub fn vraska_the_unseen() -> CardDefinition {
    let assassin = TokenDefinition {
        name: "Assassin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Assassin],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseGame {
                who: PlayerRef::DefendingPlayer,
            },
        }],
        ..Default::default()
    };
    walker(
        "Vraska the Unseen",
        cost(&[generic(3), b(), g()]),
        PlaneswalkerSubtype::Vraska,
        5,
        vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::GrantTriggeredAbility {
                    what: Selector::This,
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealtCombatDamage, EventScope::SelfSource),
                        effect: Effect::Destroy {
                            what: Selector::LastDamagerOf(Box::new(Selector::This)),
                        },
                    }),
                    duration: Duration::UntilNextTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy {
                    what: target_filtered(R::Nonland.and(R::Permanent)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: assassin,
                },
                ..Default::default()
            },
        ],
    )
}
