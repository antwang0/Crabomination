//! A Foundations wave — fight-with-counter, dig, Raid, Affinity, and an Eldrazi
//! pile of keywords. Tests in `crabomination/src/tests/recent162.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::game::TurnStep;
use crate::mana::{cost, g, generic, r, u, w};

/// Felling Blow — {2}{G} Sorcery. Put a +1/+1 counter on target creature you
/// control. Then that creature deals damage equal to its power to target
/// creature an opponent controls.
pub fn felling_blow() -> CardDefinition {
    CardDefinition {
        name: "Felling Blow",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Inspiration from Beyond — {2}{U} Sorcery. Mill three cards, then return an
/// instant or sorcery card from your graveyard to your hand. Flashback {5}{U}{U}.
pub fn inspiration_from_beyond() -> CardDefinition {
    CardDefinition {
        name: "Inspiration from Beyond",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), u(), u()]))],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::ReturnGraveyardCardsToHand {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                max: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Sower of Chaos — {3}{R} 4/3 Devil. {2}{R}: Target creature can't block this
/// turn.
pub fn sower_of_chaos() -> CardDefinition {
    CardDefinition {
        name: "Sower of Chaos",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Searslicer Goblin — {1}{R} 2/1 Goblin Warrior. Raid — at the beginning of
/// your end step, if you attacked this turn, create a 1/1 red Goblin token.
pub fn searslicer_goblin() -> CardDefinition {
    CardDefinition {
        name: "Searslicer Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: goblin_token(),
            },
        }],
        ..Default::default()
    }
}

fn goblin_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Sire of Seven Deaths — {7} 7/7 Eldrazi. First strike, vigilance, menace,
/// trample, reach, lifelink, ward—pay 7 life.
pub fn sire_of_seven_deaths() -> CardDefinition {
    CardDefinition {
        name: "Sire of Seven Deaths",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Vigilance,
            Keyword::Menace,
            Keyword::Trample,
            Keyword::Reach,
            Keyword::Lifelink,
            Keyword::Ward(WardCost::Life(7)),
        ],
        ..Default::default()
    }
}

/// Preposterous Proportions — {5}{G}{G} Sorcery. Creatures you control get
/// +10/+10 and gain vigilance until end of turn.
pub fn preposterous_proportions() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Preposterous Proportions",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: team(),
                power: Value::Const(10),
                toughness: Value::Const(10),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: team(),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Slumbering Cerberus — {1}{R} 4/2 Dog. Doesn't untap during your untap step.
/// Morbid — at the beginning of each end step, if a creature died this turn,
/// untap it.
pub fn slumbering_cerberus() -> CardDefinition {
    CardDefinition {
        name: "Slumbering Cerberus",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::ONE,
                }),
            effect: Effect::Untap {
                what: Selector::This,
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Squad Rallier — {3}{W} 3/4 Human Scout. {2}{W}: Look at the top four cards of
/// your library. You may reveal a creature card with power 2 or less from among
/// them and put it into your hand. Put the rest on the bottom in a random order.
pub fn squad_rallier() -> CardDefinition {
    CardDefinition {
        name: "Squad Rallier",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::LookPickToHand {
                then_if_picked: None,
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: false,
                pick_filter: Some(R::Creature.and(R::PowerAtMost(2))),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sphinx of Forgotten Lore — {2}{U}{U} 3/3 Sphinx. Flash, flying. Whenever it
/// attacks, target instant or sorcery card in your graveyard gains flashback
/// until end of turn (cost equal to its mana cost).
pub fn sphinx_of_forgotten_lore() -> CardDefinition {
    CardDefinition {
        name: "Sphinx of Forgotten Lore",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantFlashbackThisTurn {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
                Value::ONE,
            ),
        })],
        ..Default::default()
    }
}

/// Claws Out — {3}{W}{W} Instant. Affinity for Cats. Creatures you control get
/// +2/+2 until end of turn.
pub fn claws_out() -> CardDefinition {
    CardDefinition {
        name: "Claws Out",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(R::HasCreatureType(CreatureType::Cat).and(R::ControlledByYou)),
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Skyknight Squire — {1}{W} 1/1 Cat Scout. Whenever another creature you
/// control enters, put a +1/+1 counter on it. With three or more +1/+1 counters
/// it has flying.
pub fn skyknight_squire() -> CardDefinition {
    CardDefinition {
        name: "Skyknight Squire",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "With three or more +1/+1 counters on it, it has flying.",
            effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
                kind: CounterType::PlusOnePlusOne,
                n: 3,
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Luminous Rebuke — {4}{W} Instant. Costs {3} less if it targets a tapped
/// creature. Destroy target creature.
pub fn luminous_rebuke() -> CardDefinition {
    CardDefinition {
        name: "Luminous Rebuke",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((R::Tapped, 3)),
        effect: Effect::Destroy {
            what: target_filtered(R::Creature),
        },
        ..Default::default()
    }
}
