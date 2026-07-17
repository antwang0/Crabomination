//! DSK (Duskmourn) gap batch — Survival creatures, Delirium payoffs, a
//! modal redirect, and Norin's blocked-creature blink. Tests in
//! `tests/recent239.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, MayPlayDuration, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{animate_land, deal, target_filtered};
use crate::game::types::TurnStep;
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate,
    Selector, SpreeMode, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

/// DSK **Survival** — "At the beginning of your second main phase, if this
/// creature is tapped, …". Models to a PostCombatMain trigger gated on the
/// source being tapped.
fn survival(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped }),
        effect: body,
    }
}

/// Betrayer's Bargain — {1}{R} Instant. Additional cost: sacrifice a creature
/// or enchantment or pay {2}. Deal 5 to target creature; if it would die this
/// turn, exile it instead.
pub fn betrayers_bargain() -> CardDefinition {
    CardDefinition {
        name: "Betrayer's Bargain",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay {
            filter: R::Creature.or(R::Enchantment),
            pay: 2,
        }],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            deal(5, target_filtered(R::Creature)),
        ]),
        ..Default::default()
    }
}

/// Untimely Malfunction — {1}{R} Instant. Choose one — destroy target artifact;
/// choose new targets for target spell; or one or two target creatures can't
/// block this turn.
pub fn untimely_malfunction() -> CardDefinition {
    CardDefinition {
        name: "Untimely Malfunction",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::ChooseNewTargetsForSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Omnivorous Flytrap — {2}{G} Plant 2/4. Delirium — Whenever it enters or
/// attacks, if 4+ card types in your graveyard, distribute two +1/+1 counters
/// among one or two target creatures; then if 6+ types, double the +1/+1
/// counters on those creatures.
pub fn omnivorous_flytrap() -> CardDefinition {
    let delirium_body = || {
        Effect::Seq(vec![
            Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 2,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardTypesInGraveyard(PlayerRef::You),
                    Value::Const(6),
                ),
                then: Box::new(Effect::DoubleCountersOnEach {
                    what: Selector::AllTargets,
                    kind: CounterType::PlusOnePlusOne,
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    let delirium = || Predicate::DeliriumActive { who: PlayerRef::You };
    CardDefinition {
        name: "Omnivorous Flytrap",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(delirium()),
                effect: delirium_body(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                    .with_filter(delirium()),
                effect: delirium_body(),
            },
        ],
        ..Default::default()
    }
}

/// Norin, Swift Survivalist — {R} Human Coward 2/1. Can't block. Whenever a
/// creature you control becomes blocked, you may exile it, then play it from
/// exile this turn.
pub fn norin_swift_survivalist() -> CardDefinition {
    CardDefinition {
        name: "Norin, Swift Survivalist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Coward],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile it, then play it from exile this turn".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Outlaw Stitcher — {3}{U} Human Warlock 1/4. When it enters, create a 2/2
/// blue-black Zombie Rogue, then put two +1/+1 counters on it for each spell
/// you've cast this turn other than the first. Plot {4}{U}.
pub fn outlaw_stitcher() -> CardDefinition {
    CardDefinition {
        name: "Outlaw Stitcher",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        plot_cost: Some(cost(&[generic(4), u()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Zombie Rogue".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue, Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Zombie, CreatureType::Rogue],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::NonNeg(Box::new(Value::Diff(
                            Box::new(Value::SpellsCastThisTurn(PlayerRef::You)),
                            Box::new(Value::ONE),
                        )))),
                    ),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Tumbleweed Rising — {1}{G} Sorcery. Create an X/X green Elemental, X = the
/// greatest power among creatures you control. Plot {2}{G}.
pub fn tumbleweed_rising() -> CardDefinition {
    CardDefinition {
        name: "Tumbleweed Rising",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(2), g()])),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Elemental".into(),
                power: 0,
                toughness: 0,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elemental],
                    ..Default::default()
                },
                dynamic_pt: Some((
                    Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                    Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                )),
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Bite Down on Crime — {3}{G} Sorcery. Target creature you control gets +2/+0,
/// then deals damage equal to its power to target creature you don't control.
/// (The optional "collect evidence 6 for {2} less" discount is not modeled —
/// the engine has no collect-evidence additional cost.)
pub fn bite_down_on_crime() -> CardDefinition {
    CardDefinition {
        name: "Bite Down on Crime",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    }
}

/// Trial of Agony — {R} Sorcery. Two target creatures an opponent controls: 5
/// damage to one, the other can't block this turn. (The "same opponent" and
/// "that player chooses" clauses are collapsed to the caster's pick, matching
/// the catalog's other opponent-choice removal.)
pub fn trial_of_agony() -> CardDefinition {
    let opp_creature = || R::Creature.and(R::ControlledByOpponent);
    CardDefinition {
        name: "Trial of Agony",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 0, filter: opp_creature() },
                amount: Value::Const(5),
            },
            Effect::GrantKeyword {
                what: Selector::TargetFiltered { slot: 1, filter: opp_creature() },
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Getaway Glamer — {W} Instant. Spree — +{1} blink target nontoken creature
/// (returns at the next end step); +{2} destroy target creature if no other
/// creature has greater power.
pub fn getaway_glamer() -> CardDefinition {
    CardDefinition {
        name: "Getaway Glamer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Spree {
            modes: vec![
                SpreeMode {
                    cost: cost(&[generic(1)]),
                    effect: Effect::ExileReturnNextEndStep {
                        what: target_filtered(R::Creature.and(R::NotToken)),
                    },
                },
                SpreeMode {
                    cost: cost(&[generic(2)]),
                    effect: Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: R::HasGreatestPowerAmongAllCreatures,
                        },
                        then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                        else_: Box::new(Effect::Noop),
                    },
                },
            ],
        },
        ..Default::default()
    }
}

/// Rootwise Survivor — {3}{G}{G} Human Survivor 3/4. Haste. Survival — put
/// three +1/+1 counters on up to one target land you control; it becomes a 0/0
/// Elemental in addition to its types and gains haste. (Haste is modeled as
/// permanent rather than until-your-next-turn.)
pub fn rootwise_survivor() -> CardDefinition {
    CardDefinition {
        name: "Rootwise Survivor",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Survivor],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![survival(animate_land(0, 3))],
        ..Default::default()
    }
}

/// Reluctant Role Model — {1}{W} Human Survivor 2/2. Survival — put a flying,
/// lifelink, or +1/+1 counter on it. Whenever it or another creature you
/// control dies, put those counters on up to one target creature. ("Up to one"
/// is modeled as a required target.)
pub fn reluctant_role_model() -> CardDefinition {
    CardDefinition {
        name: "Reluctant Role Model",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Survivor],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            survival(Effect::ChooseMode(vec![
                Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Flying, amount: Value::ONE },
                Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Lifelink, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::MoveAllCounters {
                    from: Selector::TriggerSource,
                    to: target_filtered(R::Creature),
                },
            },
        ],
        ..Default::default()
    }
}

/// Come Back Wrong — {2}{B} Sorcery. Destroy target creature. If a creature
/// card is put into a graveyard this way, return it to the battlefield under
/// your control, then sacrifice it at your next end step.
pub fn come_back_wrong() -> CardDefinition {
    CardDefinition {
        name: "Come Back Wrong",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::Target(0) },
            Effect::If {
                // Only reanimate if the destroy actually buried a creature card
                // (indestructible/regenerated creatures stay put; tokens vanish).
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::Creature.and(R::InGraveyard),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::NextEndStep,
                        body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Valgavoth's Onslaught — {X}{X}{G} Sorcery. Manifest dread X times, then put
/// X +1/+1 counters on each of those creatures.
pub fn valgavoths_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Valgavoth's Onslaught",
        cost: cost(&[x(), x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ManifestDreadRepeatThenCounters {
            count: Value::XFromCost,
            counters: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Altanak, the Thrice-Called — {5}{G}{G} Insect Beast 9/9. Trample. Whenever
/// it becomes the target of a spell/ability an opponent controls, draw a card.
/// {1}{G}, Discard this card: return target land card from your graveyard to
/// the battlefield tapped. (The draw trigger reuses the Battle Mammoth scope,
/// which fires for any permanent you control.)
pub fn altanak_the_thrice_called() -> CardDefinition {
    CardDefinition {
        name: "Altanak, the Thrice-Called",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Beast],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Land.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
