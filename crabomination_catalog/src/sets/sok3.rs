//! Saviors of Kamigawa (SOK) closure — the last eight gaps, each of which
//! needed a new engine primitive. Tests in `classic_sets/sok3`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

use super::bok::{arcane_instant, arcane_sorcery, legend, sorcery};

/// The 1/1 colorless Spirit Sekki trades its counters for.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

/// Ashes of the Fallen — {2} Artifact. As it enters, choose a creature type;
/// each creature card in your graveyard has that type too.
pub fn ashes_of_the_fallen() -> CardDefinition {
    CardDefinition {
        name: "Ashes of the Fallen",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCreatureType { what: Selector::This },
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature card in your graveyard has the chosen creature type.",
            effect: StaticEffect::YourGraveyardCreaturesHaveChosenType,
        }],
        ..Default::default()
    }
}

/// Choice of Damnations — {5}{B} Arcane sorcery. Target opponent picks a
/// number; you either drain them for it or make them keep only that many
/// permanents.
pub fn choice_of_damnations() -> CardDefinition {
    arcane_sorcery(
        "Choice of Damnations",
        cost(&[generic(5), b()]),
        Effect::PlayerChoosesNumber {
            who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
            prompt: "Choose a number".into(),
            max: Value::PermanentCountControlledBy(PlayerRef::Target(0)),
            then: Box::new(Effect::MayDoElse {
                description: "Have that player lose that much life?".into(),
                body: Box::new(Effect::LoseLife {
                    who: Selector::Target(0),
                    amount: Value::ChosenNumber,
                }),
                else_: Box::new(Effect::Sacrifice {
                    who: Selector::Target(0),
                    count: Value::NonNeg(Box::new(Value::Diff(
                        Box::new(Value::PermanentCountControlledBy(PlayerRef::Target(0))),
                        Box::new(Value::ChosenNumber),
                    ))),
                    filter: R::Permanent,
                }),
            }),
        },
    )
}

/// Kaho, Minamo Historian — {2}{U}{U} 2/2. Exiles three instants on entry;
/// {X}, {T} free-casts one of them with mana value X.
pub fn kaho_minamo_historian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::HasCardType(CardType::Instant),
                to: ZoneDest::ExileWithSourceStamp,
                count: Value::Const(3),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::MayCastExiledWithSource { filter: R::ManaValueExactlyXFromCost },
            ..Default::default()
        }],
        ..legend(
            "Kaho, Minamo Historian",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Murmurs from Beyond — {2}{U} Arcane instant. Reveal three; an opponent
/// bins one and the rest go to your hand.
pub fn murmurs_from_beyond() -> CardDefinition {
    arcane_instant(
        "Murmurs from Beyond",
        cost(&[generic(2), u()]),
        Effect::RevealTopOpponentBinsOne { count: 3 },
    )
}

/// Pain's Reward — {2}{B} Sorcery. Each player may bid life; the high bidder
/// pays it and draws four.
pub fn pains_reward() -> CardDefinition {
    sorcery(
        "Pain's Reward",
        cost(&[generic(2), b()]),
        Effect::LifeBidding {
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(4) }),
        },
    )
}

/// Pure Intentions — {W} Arcane instant. Undoes an opponent's discards for the
/// turn, and returns itself at the next end step if it was one of them.
pub fn pure_intentions() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::SelfSource)
                .with_filter(Predicate::CausedByOpponentSpellOrAbility),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        }],
        ..arcane_instant(
            "Pure Intentions",
            cost(&[w()]),
            Effect::WheneverOpponentMakesYouDiscardThisTurn {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        )
    }
}

/// Rally the Horde — {5}{R} Sorcery. Exile three at a time while the last is a
/// land, then a 1/1 Warrior per nonland exiled.
pub fn rally_the_horde() -> CardDefinition {
    sorcery(
        "Rally the Horde",
        cost(&[generic(5), r()]),
        Effect::ExileTopBatchesUntilLandLast {
            batch: 3,
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::NonlandCardsExiledThisEffect,
                definition: TokenDefinition {
                    name: "Warrior".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Warrior],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
        },
    )
}

/// Sekki, Seasons' Guide — {5}{G}{G}{G} 0/0. Enters with eight +1/+1 counters
/// and trades them for Spirits as damage comes in; eight Spirits bring it back.
pub fn sekki_seasons_guide() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(8))),
        static_abilities: vec![StaticAbility {
            description: "Damage to this is prevented; trade that many +1/+1 counters for Spirits.",
            effect: StaticEffect::PreventDamageToSelfTradingCounters {
                counter: CounterType::PlusOnePlusOne,
                token: Box::new(spirit_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Spirit), 8)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..legend(
            "Sekki, Seasons' Guide",
            cost(&[generic(5), g(), g(), g()]),
            vec![CreatureType::Spirit],
            0,
            0,
        )
    }
}
