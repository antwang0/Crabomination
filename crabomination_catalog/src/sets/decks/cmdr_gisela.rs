//! Commander: the cards the **Angels: They're Just Like Us but Cooler and
//! with Wings** Secret Lair Commander deck (SLD, Gisela, the Broken Blade)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_gisela.rs`.
//!
//! Residuals (each also on its card):
//! - **Angel of Destiny** — "each player this creature attacked this turn"
//!   is the last player it attacked (one combat a turn is the common case).
//! - **Dawnbreak Reclaimer** — both graveyard picks are the engine's (the
//!   cheapest creature card each way).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, SoulbondBonus, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, cost, generic, w};
use crate::sets::tap_add_colorless;
use std::sync::Arc;

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

/// "If you have at least 15 life more than your starting life total."
fn fifteen_over_start() -> Predicate {
    Predicate::ValueAtLeast(
        Value::LifeOf(PlayerRef::You),
        Value::Sum(vec![Value::StartingLifeTotal, Value::Const(15)]),
    )
}

fn at_your_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl), effect }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// Ajani, Strength of the Pride — +1 life per creature and planeswalker you
/// control; −2 an Ajani's Pridemate; 0 at 15 over your starting life, exile
/// Ajani and every artifact and creature your opponents control.
pub fn ajani_strength_of_the_pride() -> CardDefinition {
    let pridemate = Arc::new(TokenDefinition {
        name: "Ajani's Pridemate".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    });
    CardDefinition {
        name: "Ajani, Strength of the Pride",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Sum(vec![
                        Value::CountOf(Box::new(yours(R::Creature))),
                        Value::CountOf(Box::new(yours(R::Planeswalker))),
                    ]),
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: pridemate },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::If {
                    cond: fifteen_over_start(),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Exile { what: Selector::This },
                        Effect::Exile {
                            what: Selector::EachPermanent(
                                R::Artifact.or(R::Creature).and(R::ControlledByOpponent),
                            ),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}

/// Angel of Destiny — flying, double strike; your creatures' combat damage to
/// a player gains you and that player that much life; at your end step at 15
/// over your starting life, the players it attacked this turn lose.
pub fn angel_of_destiny() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::TriggerEventPlayer),
                        amount: Value::TriggerEventAmount,
                    },
                ]),
            },
            on_attack(Effect::RememberPlayerOnSource { who: PlayerRef::DefendingPlayer }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::All(vec![
                        fifteen_over_start(),
                        Predicate::EntityMatches { what: Selector::This, filter: R::AttackedThisTurn },
                    ])),
                effect: Effect::LoseGame { who: PlayerRef::ChosenPlayerOfSource },
            },
        ],
        ..creature(
            "Angel of Destiny",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Cleric],
            2,
            6,
        )
    }
}

/// Arch of Orazca — ascend; taps for {C}; {5}, {T}: draw with the city's
/// blessing. Permanent ascend (CR 702.131b) is checked as it enters and as
/// the draw is activated.
pub fn arch_of_orazca() -> CardDefinition {
    CardDefinition {
        name: "Arch of Orazca",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Ascend { who: PlayerRef::You })],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                condition: Some(Predicate::Any(vec![
                    Predicate::HasCityBlessing { who: PlayerRef::You },
                    Predicate::SelectorCountAtLeast {
                        sel: yours(R::Permanent),
                        n: Value::Const(10),
                    },
                ])),
                effect: Effect::Seq(vec![
                    Effect::Ascend { who: PlayerRef::You },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Arden Angel — flying; at your upkeep, from your graveyard, a d4: on a 1 it
/// returns to the battlefield.
pub fn arden_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::FromYourGraveyard),
            effect: Effect::RollDie {
                sides: 4,
                count: Value::Const(1),
                modifier: Value::Const(0),
                reroll_at_most: 0,
                results: vec![(
                    1,
                    1,
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                )],
                on_doubles: None,
            },
        }],
        ..creature("Arden Angel", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Breathkeeper Seraph — flying, soulbond; paired, each has "when this dies,
/// you may return it at the beginning of your next upkeep".
pub fn breathkeeper_seraph() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Soulbond],
        soulbond_bonus: Some(SoulbondBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Return it at the beginning of your next upkeep?".into(),
                    body: Box::new(Effect::AtYourNextUpkeep {
                        body: Box::new(Effect::Move {
                            what: Selector::This,
                            to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOf(Box::new(Selector::This)), tapped: false },
                        }),
                    }),
                },
            }],
            ..Default::default()
        }),
        ..creature("Breathkeeper Seraph", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Bruna, the Fading Light — as you cast it, may return an Angel or Human
/// creature card from your graveyard to the battlefield; melds with Gisela.
pub fn bruna_the_fading_light() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return an Angel or Human creature card to the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::HasCreatureType(CreatureType::Angel).or(R::HasCreatureType(CreatureType::Human)))
                            .from_your_graveyard(),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Bruna, the Fading Light",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Horror],
            5,
            7,
        )
    }
}

/// Gisela, the Broken Blade — flying, first strike, lifelink; at your end
/// step with Bruna, the two meld into Brisela (CR 701.37).
pub fn gisela_the_broken_blade() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![at_your_end_step(Effect::Meld {
            partner: "Bruna, the Fading Light".into(),
            into: "Brisela, Voice of Nightmares".into(),
        })],
        ..creature(
            "Gisela, the Broken Blade",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Horror],
            4,
            3,
        )
    }
}

/// Brisela, Voice of Nightmares — the meld of Gisela and Bruna: 9/10 flying,
/// first strike, vigilance, lifelink; opponents can't cast spells with mana
/// value 3 or less.
pub fn brisela_voice_of_nightmares() -> CardDefinition {
    CardDefinition {
        // CR 202.1b — the melded face prints no mana cost.
        no_mana_cost: true,
        color_indicator: vec![Color::White],
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells with mana value 3 or less.",
            effect: StaticEffect::OpponentsCantCastMatching { filter: R::ManaValueAtMost(3) },
        }],
        ..creature(
            "Brisela, Voice of Nightmares",
            crate::mana::ManaCost::default(),
            vec![CreatureType::Eldrazi, CreatureType::Angel],
            9,
            10,
        )
    }
}

/// Cosmos Elixir — at your end step, draw above your starting life, else
/// gain 2.
pub fn cosmos_elixir() -> CardDefinition {
    CardDefinition {
        name: "Cosmos Elixir",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![at_your_end_step(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::LifeOf(PlayerRef::You),
                Value::Sum(vec![Value::StartingLifeTotal, Value::Const(1)]),
            ),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
        })],
        ..Default::default()
    }
}

/// Dawnbreak Reclaimer — flying; at your end step, a creature card from an
/// opponent's graveyard and one from yours may both return.
pub fn dawnbreak_reclaimer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![at_your_end_step(Effect::ChooseGraveyardCreaturesEachMayReturn)],
        ..creature("Dawnbreak Reclaimer", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Keeper of the Accord — at each opponent's end step, a Soldier if they
/// have more creatures than you, and a Plains if they have more lands.
pub fn keeper_of_the_accord() -> CardDefinition {
    let more = |filter: R| {
        Predicate::ValueAtLeast(
            Value::CountOf(Box::new(Selector::EachPermanent(filter.clone().and(R::ControlledByActivePlayer)))),
            Value::Sum(vec![Value::CountOf(Box::new(yours(filter))), Value::Const(1)]),
        )
    };
    let opp_end = || EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::OpponentControl);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: opp_end().with_filter(more(R::Creature)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: Arc::new(TokenDefinition {
                        name: "Soldier".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Soldier],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
            },
            TriggeredAbility {
                event: opp_end().with_filter(more(R::Land)),
                effect: Effect::MayDo {
                    description: "Search for a basic Plains?".into(),
                    body: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand.and(R::HasLandType(LandType::Plains)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    }),
                },
            },
        ],
        ..creature(
            "Keeper of the Accord",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Thalia's Lancers — first strike; may tutor a legendary card on entry.
pub fn thalias_lancers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a legendary card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasSupertype(Supertype::Legendary),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Thalia's Lancers",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// The Book of Exalted Deeds — a 3/3 flying Angel at your end step after 3+
/// life gained; exile it to give an Angel "you can't lose the game and your
/// opponents can't win the game".
pub fn the_book_of_exalted_deeds() -> CardDefinition {
    let angel = |filter: R| filter.and(R::HasCreatureType(CreatureType::Angel));
    CardDefinition {
        name: "The Book of Exalted Deeds",
        cost: cost(&[w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(3) },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Arc::new(TokenDefinition {
                    name: "Angel".into(),
                    power: 3,
                    toughness: 3,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
                    ..Default::default()
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w(), w()]),
            tap_cost: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(angel(R::Creature)),
                    kind: CounterType::Enlightened,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::ControllerCantLoseGame,
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
