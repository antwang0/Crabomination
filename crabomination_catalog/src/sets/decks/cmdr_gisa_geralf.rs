//! Commander: the cards the **Grave Danger** precon (SCD, Gisa and Geralf)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_fdc.rs`
//! (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Havengul Lich** — the cast permission lands, but the Lich does not gain
//!   the cast card's activated abilities.
//! - **Liliana, Untouched by Death** — the −3 covers the Zombie cards in your
//!   graveyard as it resolves, not ones that arrive later that turn.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, MayPlayDuration,
    PlaneswalkerSubtype, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, generic, u};
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

fn zombie() -> R {
    R::HasCreatureType(CreatureType::Zombie)
}

fn zombie_token(tapped: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped,
        ..Default::default()
    })
}

fn mill_self(n: i32) -> Effect {
    Effect::Mill { who: Selector::You, amount: Value::Const(n) }
}

/// Gisa and Geralf — mill four on entry; once during each of your turns,
/// cast a Zombie creature spell from your graveyard.
pub fn gisa_and_geralf() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(mill_self(4))],
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast a Zombie creature spell \
                          from your graveyard.",
            effect: StaticEffect::GraveyardCastOncePerTurn { filter: R::Creature.and(zombie()), exile_after: false },
        }],
        ..creature(
            "Gisa and Geralf",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Geralf's Mindcrusher — target player mills five on entry; undying.
pub fn geralfs_mindcrusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Undying],
        triggered_abilities: vec![etb(Effect::Mill {
            who: target_filtered(R::Player),
            amount: Value::Const(5),
        })],
        ..creature(
            "Geralf's Mindcrusher",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Zombie, CreatureType::Horror],
            5,
            5,
        )
    }
}

/// Grimoire of the Dead — discard for study counters; at three, sacrifice it
/// to put every creature card in every graveyard onto the battlefield under
/// your control as black Zombies.
pub fn grimoire_of_the_dead() -> CardDefinition {
    CardDefinition {
        name: "Grimoire of the Dead",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Study,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                remove_counter_cost: Some((CounterType::Study, 3)),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                            filter: R::Creature,
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::BecomeColor {
                        what: Selector::LastMoved,
                        colors: vec![Color::Black],
                        duration: Duration::Permanent,
                        additive: true,
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::LastMoved,
                        creature_types: vec![CreatureType::Zombie],
                        duration: Duration::Permanent,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Havengul Lich — {1}: you may cast target creature card in a graveyard this
/// turn. ⚠ It doesn't gain that card's activated abilities.
pub fn havengul_lich() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantMayPlay {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Havengul Lich",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Laboratory Drudge — each end step, draw if you used a graveyard this turn.
pub fn laboratory_drudge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::UsedGraveyardThisTurn { who: PlayerRef::You }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Laboratory Drudge",
            cost(&[generic(3), u()]),
            vec![CreatureType::Zombie, CreatureType::Horror],
            3,
            4,
        )
    }
}

/// Liliana's Devotee — Zombies you control get +1/+0; your end step, if a
/// creature died this turn, pay {1}{B} for a 2/2 Zombie.
pub fn lilianas_devotee() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Zombies you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(zombie().and(R::ControlledByYou)),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::ForAnyPlayer {
                    who: PlayerRef::EachPlayer,
                    pred: Box::new(Predicate::CreaturesDiedThisTurnAtLeast {
                        who: PlayerRef::Triggerer,
                        at_least: Value::ONE,
                    }),
                }),
            effect: Effect::MayPay {
                description: "Pay {1}{B} for a 2/2 Zombie?".into(),
                mana_cost: cost(&[generic(1), b()]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: zombie_token(false),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Liliana's Devotee",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            2,
            3,
        )
    }
}

/// Liliana, Untouched by Death — +1 mill three, drain on a Zombie; −2 shrink
/// by your Zombies; −3 your graveyard's Zombies are castable this turn.
pub fn liliana_untouched_by_death() -> CardDefinition {
    let zombies = || {
        Value::CountOf(Box::new(Selector::EachPermanent(zombie().and(R::ControlledByYou))))
    };
    CardDefinition {
        name: "Liliana, Untouched by Death",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    mill_self(3),
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::CardsMilledThisEffectMatching { filter: zombie() },
                            Value::ONE,
                        ),
                        then: Box::new(Effect::Seq(vec![
                            Effect::LoseLife {
                                who: Selector::Player(PlayerRef::EachOpponent),
                                amount: Value::Const(2),
                            },
                            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Times(Box::new(zombies()), Box::new(Value::Const(-1))),
                    toughness: Value::Times(Box::new(zombies()), Box::new(Value::Const(-1))),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::GrantMayPlay {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: zombie(),
                    },
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lotleth Giant — undergrowth: 1 damage to target opponent per creature card
/// in your graveyard.
pub fn lotleth_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::OpponentPlayer),
            amount: Value::CountOf(Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature,
            })),
        })],
        ..creature(
            "Lotleth Giant",
            cost(&[generic(6), b()]),
            vec![CreatureType::Zombie, CreatureType::Giant],
            6,
            5,
        )
    }
}

/// Loyal Subordinate — menace; lieutenant: with your commander out, each
/// opponent loses 3 at your beginning of combat.
pub fn loyal_subordinate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        // CR 207.2c lieutenant — the "if you control your commander" gate is
        // the body's `If` (no intervening-if field on `TriggeredAbility`).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Loyal Subordinate", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 3, 1)
    }
}

/// Overseer of the Damned — flying; may destroy a creature on entry; an
/// opponent's nontoken creature dying makes you a tapped 2/2 Zombie.
pub fn overseer_of_the_damned() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Destroy target creature?".into(),
                body: Box::new(Effect::Destroy { what: target_filtered(R::Creature) }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsToken.negate(),
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: zombie_token(true),
                },
            },
        ],
        ..creature(
            "Overseer of the Damned",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Demon],
            5,
            5,
        )
    }
}

/// Scourge of Nel Toth — flying; castable from your graveyard for {B}{B} and
/// two sacrificed creatures instead of its mana cost.
pub fn scourge_of_nel_toth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(AlternativeCost {
            from_graveyard: true,
            mana_cost: cost(&[b(), b()]),
            sacrifice_permanents: Some((R::Creature, 2)),
            ..Default::default()
        }),
        ..creature(
            "Scourge of Nel Toth",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Sinister Sabotage — counter target spell; surveil 1.
pub fn sinister_sabotage() -> CardDefinition {
    CardDefinition {
        name: "Sinister Sabotage",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Unbreathing Horde — a counter per other Zombie you control and per Zombie
/// card in your graveyard; damage to it removes a counter instead.
pub fn unbreathing_horde() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Sum(vec![
                Value::CountOf(Box::new(Selector::EachPermanent(
                    zombie().and(R::ControlledByYou).and(R::OtherThanSource),
                ))),
                Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: zombie(),
                })),
            ]),
        )),
        static_abilities: vec![StaticAbility {
            description: "If this creature would be dealt damage, prevent that damage and remove \
                          a +1/+1 counter from it.",
            effect: StaticEffect::PreventDamageByRemovingCounters {
                kind: CounterType::PlusOnePlusOne,
                single: true,
            },
        }],
        ..creature("Unbreathing Horde", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 0, 0)
    }
}

/// Unstable Obelisk — {C}; {7}, {T}, sacrifice: destroy target permanent.
pub fn unstable_obelisk() -> CardDefinition {
    CardDefinition {
        name: "Unstable Obelisk",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(7)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Destroy { what: target_filtered(R::Permanent) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vela the Night-Clad — intimidate for your whole team; each creature of
/// yours leaving drains each opponent for 1.
pub fn vela_the_night_clad() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Intimidate],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have intimidate.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Intimidate,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Vela the Night-Clad",
            cost(&[generic(4), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Zombie Apocalypse — every Zombie creature card in your graveyard returns
/// tapped, then all Humans are destroyed.
pub fn zombie_apocalypse() -> CardDefinition {
    CardDefinition {
        name: "Zombie Apocalypse",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature.and(zombie()),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Human)),
                ),
            },
        ]),
        ..Default::default()
    }
}
