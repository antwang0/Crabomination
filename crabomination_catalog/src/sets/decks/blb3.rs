//! Bloomburrow gap batch — the mythic legends and Ral. Tests in
//! `tests/recent_b/blb3.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, Keyword, LandType, LoyaltyAbility, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u};

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A 1/1 blue and red Otter with prowess.
fn otter_token() -> TokenDefinition {
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

/// Ral, Crackling Wit — {2}{U}{R} Ral. Noncreature spells tick him up; the
/// ultimate hands out storm.
pub fn ral_crackling_wit() -> CardDefinition {
    CardDefinition {
        name: "Ral, Crackling Wit",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Ral],
            ..Default::default()
        },
        base_loyalty: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: otter_token(),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(2),
                        random: false,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                    Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Ral, Crackling Wit".into(),
                        triggered: Vec::new(),
                        statics: vec![StaticAbility {
                            description: "Instant and sorcery spells you cast have storm.",
                            effect: StaticEffect::GrantStormToISSpells,
                        }],
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Eluge, the Shoreless Sea — {1}{U}{U}{U} */*. Floods lands into Islands and
/// discounts your first spell each turn by how many it has drowned.
pub fn eluge_the_shoreless_sea() -> CardDefinition {
    let flood = || {
        TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(R::Land),
                kind: CounterType::Flood,
                amount: Value::ONE,
            },
        }
    };
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled {
            land_type: LandType::Island,
            base_p: 0,
            base_t: 0,
        }),
        static_abilities: vec![
            StaticAbility {
                description: "Each land with a flood counter on it is an Island in addition to \
                              its other types.",
                effect: StaticEffect::LandTypeChanger {
                    applies_to: Selector::EachPermanent(
                        R::Land.and(R::WithCounter(CounterType::Flood)),
                    ),
                    land_type: LandType::Island,
                    replace: false,
                },
            },
            StaticAbility {
                description: "The first instant or sorcery spell you cast each turn costs {1} \
                              less for each land you control with a flood counter on it.",
                effect: StaticEffect::CostReductionFirstInstantOrSorceryPerValue {
                    per: Value::PermanentCountControlledByMatching(
                        PlayerRef::You,
                        R::Land.and(R::WithCounter(CounterType::Flood)),
                    ),
                },
            },
        ],
        triggered_abilities: vec![
            flood(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: flood().effect,
            },
        ],
        ..legend(
            "Eluge, the Shoreless Sea",
            cost(&[generic(1), u(), u(), u()]),
            vec![CreatureType::Elemental, CreatureType::Fish],
            0,
            0,
        )
    }
}

/// A 1/1 black Rat that grows with the swarm.
fn swarm_rat() -> TokenDefinition {
    TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This token gets +1/+1 for each other Rat you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasCreatureType(CreatureType::Rat).and(R::OtherThanSource),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Vren, the Relentless — {2}{U}{B} 3/4. Opponents' creatures go to exile, and
/// every one becomes a Rat at end of turn.
pub fn vren_the_relentless() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: None },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crabomination_base::turn_step::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CreaturesExiledFromControlThisTurn(PlayerRef::EachOpponent),
                definition: swarm_rat(),
            },
        }],
        ..legend(
            "Vren, the Relentless",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Rat, CreatureType::Rogue],
            3,
            4,
        )
    }
}

/// Ygra, Eater of All — {3}{B}{G} 6/6. Turns the board into Food and eats it.
pub fn ygra_eater_of_all() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::SacrificeMatching(Box::new(
            R::HasArtifactSubtype(ArtifactSubtype::Food),
        )))],
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures are Food artifacts in addition to their other \
                              types.",
                effect: StaticEffect::AddCardTypeToMatching {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    card_type: CardType::Artifact,
                    artifact_subtype: Some(ArtifactSubtype::Food),
                },
            },
            StaticAbility {
                description: "…and have \"{2}, {T}, Sacrifice this permanent: You gain 3 life.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[generic(2)]),
                        tap_cost: true,
                        sac_cost: true,
                        effect: Effect::GainLife {
                            who: Selector::You,
                            amount: Value::Const(3),
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Food)
                        .or(R::Creature.and(R::OtherThanSource)),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..legend(
            "Ygra, Eater of All",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            6,
            6,
        )
    }
}
